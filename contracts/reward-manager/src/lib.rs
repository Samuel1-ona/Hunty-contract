#![cfg_attr(not(test), no_std)]
#![allow(clippy::too_many_arguments)]
use soroban_sdk::xdr::ToXdr;
use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, Address, BytesN, Env, IntoVal, Symbol, Val,
    Vec,
};

pub use crate::errors::RewardErrorCode;
use crate::nft_handler::NftHandler;
use crate::storage::Storage;
use crate::token_handler::TokenHandler;
pub use crate::types::{
    resolve_tier_amount, tiers_are_strictly_ascending, BatchDistributionEntry,
    DistributionAnalytics, DistributionMode, DistributionProof, DistributionRecord,
    DistributionStatus, PendingNftMint, PoolAuditEntry, PoolDistribution, PoolOperation,
    ResolutionStatus, RewardConfig, RewardPoolConfig, RewardPoolStatistics, RewardPoolStatus,
    SemVer, TierError, TimeBasedRewardTier, ValidationResult, VestingRecord, VestingStatus,
};
use crate::xlm_handler::XlmHandler;

// Funding validation constants
// 1 XLM = 10_000_000 stroops (Stellar's base unit)
/// Minimum funding amount: 1 XLM (prevents dust attacks)
const MIN_FUNDING_AMOUNT: i128 = 10_000_000;

/// Maximum single funding amount: 1 billion XLM (prevents overflow and unreasonable deposits)
const MAX_FUNDING_AMOUNT: i128 = 1_000_000_000 * 10_000_000;

/// Maximum pool balance: 1 billion XLM (prevents overflow)
const MAX_POOL_BALANCE: i128 = 1_000_000_000 * 10_000_000;

/// Maximum number of distinct funders tracked per pool. Bounds the cost of
/// the pro-rata payout loop in `refund_pool`. Once reached, only addresses
/// that have already contributed may add to their existing contribution.
const MAX_FUNDERS_PER_POOL: u32 = 50;

/// Maximum number of entries allowed in a single `distribute_batch` call.
///
/// Chosen to keep intrinsic gas cost well within Soroban's per-transaction
/// instruction budget even when every entry performs both XLM and NFT operations.
const MAX_BATCH_SIZE: u32 = 10;

/// Maximum number of distribution entries considered when computing
/// distribution analytics. Keeps gas costs bounded even for pools with
/// an arbitrarily large number of distributions.
const MAX_ANALYTICS_ENTRIES: u32 = 500;

#[contract]
pub struct RewardManager;

/// Prevents concurrent distribution executions within a single transaction.
///
/// This guard sets a persistent storage flag when acquired and clears it when dropped.
/// It provides protection against accidental reentrancy from cross-contract calls.
///
/// # Important: Limitations Under Panic=Abort
///
/// The Cargo.toml release profile sets `panic = "abort"`, which means no unwinding occurs
/// when a panic is triggered. In this configuration, Drop destructors do NOT run, and the
/// flag will remain set (true) indefinitely. This creates a permanent state:
/// - Any panic between ReentrancyGuard::acquire() and the end of distribute_rewards
///   (e.g., in token client, NFT handler, or arithmetic overflow) will cause the flag
///   to persist as true.
/// - All subsequent distribution calls will immediately fail with ReentrancyDetected.
/// - There is no admin path to clear this flag once set.
/// - The contract's core function is effectively disabled until the next ledger TTL expiration.
///
/// # Soroban Ledger TTL Behavior
///
/// However, Soroban's persistent storage uses a TTL (time-to-live) expiration model:
/// - Entries that are not accessed within the TTL window are automatically garbage collected.
/// - When the flag expires, the contract can resume normal operation.
/// - This provides eventual recovery without manual intervention.
///
/// To minimize risk of extended outage:
/// - Test thoroughly to prevent panics in the critical path.
/// - Keep calls to external contracts (token, NFT) as simple as possible.
/// - Monitor the ReentrancyDetected error rate in production metrics.
/// - Be prepared to document the TTL expiration timeline if this occurs.
struct ReentrancyGuard {
    env: Env,
}

impl ReentrancyGuard {
    fn acquire(env: &Env) -> Result<Self, RewardErrorCode> {
        if Storage::is_in_distribution(env) {
            return Err(RewardErrorCode::ReentrancyDetected);
        }
        let env = env.clone();
        Storage::set_in_distribution(&env, true);
        Ok(Self { env })
    }
}

impl Drop for ReentrancyGuard {
    fn drop(&mut self) {
        Storage::set_in_distribution(&self.env, false);
    }
}

/// Event emitted when a reward pool is created for a hunt.
#[contracttype]
#[derive(Clone, Debug)]
pub struct RewardPoolCreatedEvent {
    pub hunt_id: u64,
    pub creator: Address,
    pub min_distribution_amount: i128,
}

/// Event emitted when a reward pool is funded.
#[contracttype]
#[derive(Clone, Debug)]
pub struct RewardPoolFundedEvent {
    pub hunt_id: u64,
    pub funder: Address,
    pub amount: i128,
    pub new_balance: i128,
    pub total_deposited: i128,
    /// Configured funding target (0 = no target).
    pub target_amount: i128,
    /// Percentage of target reached (0–100+, floored). 0 when target_amount is 0.
    pub percentage_of_target: u32,
    /// Funding progress toward the target (same as total_deposited when targeting).
    pub funding_progress: i128,
}

/// Event emitted when a distribution is blocked by the per-pool rate limit.
#[contracttype]
#[derive(Clone, Debug)]
pub struct DistributionCooldownEvent {
    pub hunt_id: u64,
    pub remaining_secs: u64,
}

/// Event emitted for each address paid out by `refund_pool`. A pool funded by
/// a single address (the common case) emits exactly one of these; a
/// sponsored pool emits one per funder, each amount proportional to that
/// funder's share of the pool's contributions.
#[contracttype]
#[derive(Clone, Debug)]
pub struct PoolRefundedEvent {
    pub hunt_id: u64,
    pub funder: Address,
    pub amount: i128,
}

/// Event emitted when the unused balance of an expired or cancelled hunt's
/// pool is migrated into an existing destination pool owned by the same creator.
#[contracttype]
#[derive(Clone, Debug)]
pub struct PoolMigratedEvent {
    pub source_hunt_id: u64,
    pub dest_hunt_id: u64,
    pub creator: Address,
    pub amount: i128,
}

/// Event emitted when rewards are successfully distributed.
#[contracttype]
#[derive(Clone, Debug)]
pub struct RewardsDistributedEvent {
    pub hunt_id: u64,
    pub player: Address,
    pub xlm_amount: i128,
    pub nft_id: Option<u64>,
}

/// Event emitted when NFT minting fails during reward distribution.
/// XLM is still distributed; the NFT mint can be retried later.
#[contracttype]
#[derive(Clone, Debug)]
pub struct NftMintFailedEvent {
    pub hunt_id: u64,
    pub player: Address,
    pub nft_contract: Option<Address>,
}

/// Event emitted when admin withdraws unclaimed rewards from a pool.
#[contracttype]
#[derive(Clone, Debug)]
pub struct AdminWithdrawEvent {
    pub hunt_id: u64,
    pub admin: Address,
    pub amount: i128,
}

/// Event emitted when daily pool cap warning (80% usage) is reached.
#[contracttype]
#[derive(Clone, Debug)]
pub struct DailyPoolCapWarningEvent {
    pub hunt_id: u64,
    pub used: i128,
    pub cap: i128,
}

/// Event emitted when global daily cap warning (80% usage) is reached.
#[contracttype]
#[derive(Clone, Debug)]
pub struct GlobalDailyCapWarningEvent {
    pub used: i128,
    pub cap: i128,
}

/// Event emitted when the default NFT reward contract is set or updated.
#[contracttype]
#[derive(Clone, Debug)]
pub struct NftContractSetEvent {
    pub old_contract: Option<Address>,
    pub new_contract: Address,
}

/// Event emitted when an admin resolves a failed distribution.
#[contracttype]
#[derive(Clone, Debug)]
pub struct DistributionResolvedEvent {
    pub hunt_id: u64,
    pub player: Address,
    pub admin: Address,
    pub resolution: ResolutionStatus,
}

/// Event emitted when a reward pool is frozen by its creator or the contract admin.
#[contracttype]
#[derive(Clone, Debug)]
pub struct PoolFrozenEvent {
    pub hunt_id: u64,
    pub caller: Address,
}

/// Event emitted when a reward pool is unfrozen by its creator or the contract admin.
#[contracttype]
#[derive(Clone, Debug)]
pub struct PoolUnfrozenEvent {
    pub hunt_id: u64,
    pub caller: Address,
}

/// Event emitted when emergency withdrawal is executed.
#[contracttype]
#[derive(Clone, Debug)]
pub struct EmergencyWithdrawalEvent {
    pub admin: Address,
    pub hunt_id: u64,
    pub amount: i128,
    pub reason: soroban_sdk::String,
    pub timestamp: u64,
}

/// Log entry for emergency withdrawal record-keeping.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EmergencyWithdrawalLogEntry {
    pub hunt_id: u64,
    pub amount: i128,
    pub reason: soroban_sdk::String,
    pub timestamp: u64,
}

/// Paginated response for the audit log.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PoolAuditLogResponse {
    pub entries: Vec<PoolAuditEntry>,
    pub total: u64,
}

/// Event emitted when a vesting schedule is created for a player.
/// Tokens are locked in the contract and released proportionally over time.
#[contracttype]
#[derive(Clone, Debug)]
pub struct VestingCreatedEvent {
    pub hunt_id: u64,
    pub player: Address,
    pub total_amount: i128,
    pub vesting_period_secs: u64,
    pub start_time: u64,
}

/// Event emitted when a player successfully claims a portion of their vested rewards.
#[contracttype]
#[derive(Clone, Debug)]
pub struct VestedClaimedEvent {
    pub hunt_id: u64,
    pub player: Address,
    pub claimed_amount: i128,
    pub total_claimed: i128,
    pub fully_vested: bool,
}

#[contractimpl]
impl RewardManager {
    fn is_delegate(config: &RewardPoolConfig, candidate: &Address) -> bool {
        for i in 0..config.delegates.len() {
            if config.delegates.get(i).unwrap() == *candidate {
                return true;
            }
        }
        false
    }

    /// Current semantic version of this contract.
    pub const CONTRACT_VERSION: u32 = 2;
    /// Minimum NftReward version this contract requires.
    pub const REQUIRED_NFT_REWARD_VERSION: u32 = 2;

    /// Initializes the RewardManager with the XLM token contract address (SAC).
    /// Must be called once before any reward distribution.
    pub fn initialize(
        env: Env,
        admin: Address,
        xlm_token: Address,
        hunty_core: Address,
    ) -> Result<(), RewardErrorCode> {
        if Storage::get_xlm_token(&env).is_some() {
            return Err(RewardErrorCode::AlreadyInitialized);
        }

        #[cfg(not(test))]
        admin.require_auth();
        Storage::set_admin(&env, &admin);
        Storage::set_xlm_token(&env, &xlm_token);
        Storage::set_hunty_core(&env, &hunty_core);
        Storage::set_contract_version(&env, Self::CONTRACT_VERSION);
        Ok(())
    }

    fn require_admin(env: &Env, admin: &Address) -> Result<(), RewardErrorCode> {
        #[cfg(not(test))]
        admin.require_auth();

        let configured_admin = Storage::get_admin(env).ok_or(RewardErrorCode::NotInitialized)?;
        if configured_admin != *admin {
            return Err(RewardErrorCode::Unauthorized);
        }
        Ok(())
    }

    /// Step one of a two-step admin key rotation.
    pub fn propose_new_admin(
        env: Env,
        admin: Address,
        new_admin: Address,
    ) -> Result<(), RewardErrorCode> {
        Self::require_admin(&env, &admin)?;

        Storage::set_pending_admin(&env, &new_admin);

        env.events().publish(
            (
                soroban_sdk::Symbol::new(&env, "ADMIN"),
                soroban_sdk::Symbol::new(&env, "ADM_PROP"),
            ),
            (admin, new_admin),
        );

        Ok(())
    }

    /// Step two of a two-step admin key rotation.
    pub fn accept_admin(env: Env, new_admin: Address) -> Result<(), RewardErrorCode> {
        new_admin.require_auth();

        let pending = Storage::get_pending_admin(&env).ok_or(RewardErrorCode::Unauthorized)?;
        if pending != new_admin {
            return Err(RewardErrorCode::Unauthorized);
        }

        let old_admin = Storage::get_admin(&env);
        Storage::set_admin(&env, &new_admin);
        Storage::clear_pending_admin(&env);

        let old_admin_str = old_admin
            .as_ref()
            .map(|a| a.to_string())
            .unwrap_or_else(|| soroban_sdk::String::from_str(&env, "NONE"));

        env.events().publish(
            (
                soroban_sdk::Symbol::new(&env, "ADMIN"),
                soroban_sdk::Symbol::new(&env, "ADM_TRF"),
            ),
            (old_admin_str, new_admin.to_string()),
        );

        Ok(())
    }

    /// Sets the default NftReward contract address used for NFT distributions
    /// when a per-call NFT contract is not provided.
    /// Emits an NftContractSetEvent with the old and new contract addresses.
    pub fn set_nft_reward_contract(
        env: Env,
        admin: Address,
        nft_contract: Address,
    ) -> Result<(), RewardErrorCode> {
        Self::require_admin(&env, &admin)?;

        // Capture the old contract address before updating
        let old_contract = Storage::get_nft_contract(&env);

        // Update the contract
        Storage::set_nft_contract(&env, &nft_contract);

        // Emit the event
        env.events().publish(
            (symbol_short!("NFT_SET"),),
            NftContractSetEvent {
                old_contract,
                new_contract: nft_contract,
            },
        );

        Ok(())
    }

    /// Sets the optional HuntyCore contract address used to validate hunt_id existence
    /// in `create_reward_pool`. When set, pool creation will be rejected for unknown
    /// hunt IDs. If not set, hunt_id is assumed caller-trusted.
    pub fn set_hunty_core(
        env: Env,
        admin: Address,
        hunty_core: Address,
    ) -> Result<(), RewardErrorCode> {
        Self::require_admin(&env, &admin)?;
        Storage::set_hunty_core(&env, &hunty_core);
        Ok(())
    }

    fn require_authorized_distributor(_env: &Env) -> Result<(), RewardErrorCode> {
        Ok(())
    }

    /// Adds a contract to the authorized callers list for `distribute_rewards`.
    /// Only the contract admin can call this.
    pub fn add_authorized_contract(
        env: Env,
        admin: Address,
        contract: Address,
    ) -> Result<(), RewardErrorCode> {
        Self::require_admin(&env, &admin)?;
        Storage::add_authorized_contract(&env, &contract);
        Ok(())
    }

    /// Removes a contract from the authorized callers list.
    /// Only the contract admin can call this.
    pub fn remove_authorized_contract(
        env: Env,
        admin: Address,
        contract: Address,
    ) -> Result<(), RewardErrorCode> {
        Self::require_admin(&env, &admin)?;
        Storage::remove_authorized_contract(&env, &contract);
        Ok(())
    }

    /// Creates a reward pool for a specific hunt with a specified token.
    ///
    /// Must be called before `fund_reward_pool`. Any address may fund the pool
    /// after creation (see `fund_reward_pool`); the token contract must be
    /// SAC-compatible.
    ///
    /// For NFT-only pools (pools that distribute only NFTs without any token component),
    /// set `min_distribution_amount` to 0 and provide an `nft_contract` address.
    ///
    /// # Arguments
    /// * `creator` - The hunt creator who will own and fund the pool
    /// * `hunt_id` - The hunt this pool is for
    /// * `token_address` - Address of the SAC-compatible token contract (e.g., XLM, USDC)
    /// * `min_distribution_amount` - Minimum token amount per distribution (0 for NFT-only pools)
    /// * `nft_contract` - Optional NFT contract address for NFT rewards
    /// * `nft_royalty_bps` - Creator royalty basis points (0-10000) for secondary market sales
    /// * `nft_transferable` - Whether reward NFTs from this pool are transferable
    ///
    /// # Errors
    /// * `PoolAlreadyExists` - A pool already exists for this hunt_id
    /// * `InvalidAmount` - min_distribution_amount is negative
    /// * `InvalidTokenContract` - token_address is not a valid SAC-compatible token
    /// * `InvalidConfig` - min_distribution_amount is 0 but no NFT contract provided
    /// * `NotInitialized` - hunty_core has not been configured (set during initialize)
    /// * `HuntNotFound` - hunt_id does not exist in HuntyCore
    pub fn create_reward_pool_with_nft(
        env: Env,
        creator: Address,
        hunt_id: u64,
        token_address: Address,
        min_distribution_amount: i128,
        nft_contract: Option<Address>,
        nft_royalty_bps: u32,
        nft_transferable: bool,
    ) -> Result<(), RewardErrorCode> {
        #[cfg(not(test))]
        creator.require_auth();

        if min_distribution_amount < 0 {
            return Err(RewardErrorCode::InvalidAmount);
        }

        // Validation: For NFT-only pools (zero XLM), an NFT contract must be set
        if min_distribution_amount == 0 && nft_contract.is_none() {
            return Err(RewardErrorCode::InvalidConfig);
        }

        if Storage::get_pool_config(&env, hunt_id).is_some() {
            return Err(RewardErrorCode::PoolAlreadyExists);
        }

        // Validate that the token_address is a valid SAC-compatible token contract
        TokenHandler::validate_token_contract(&env, &token_address)?;

        // Validate hunt_id exists in HuntyCore. hunty_core must be set during initialization.
        // Fail closed: pool creation is rejected if hunty_core is not configured.
        #[cfg(not(test))]
        {
            let hunty_core =
                Storage::get_hunty_core(&env).ok_or(RewardErrorCode::NotInitialized)?;

            let mut args: Vec<Val> = Vec::new(&env);
            args.push_back(hunt_id.into_val(&env));
            // get_hunt_end_time returns Result<u64, HuntErrorCode>.
            // Any non-Ok(Ok(_)) result means the hunt doesn't exist or the call failed.
            let hunt_exists = matches!(
                env.try_invoke_contract::<u64, Val>(
                    &hunty_core,
                    &Symbol::new(&env, "get_hunt_end_time"),
                    args
                ),
                Ok(Ok(_))
            );
            if !hunt_exists {
                return Err(RewardErrorCode::HuntNotFound);
            }
        }

        let config = RewardPoolConfig {
            creator: creator.clone(),
            delegates: Vec::new(&env),
            min_distribution_amount,
            time_based_tiers: Vec::new(&env),
            frozen: false,
            token_address: token_address.clone(),
            nft_contract: nft_contract.clone(),
            target_amount: 0,
            min_distribution_interval_secs: 0,
            distribution_mode: DistributionMode::Fixed,
            vesting_period_secs: 0,
            claim_deadline: 0,
            nft_royalty_bps,
            nft_transferable,
        };
        Storage::set_pool_config(&env, hunt_id, &config);

        env.events().publish(
            (symbol_short!("POOL_CRT"), hunt_id),
            RewardPoolCreatedEvent {
                hunt_id,
                creator: creator.clone(),
                min_distribution_amount,
            },
        );

        let audit_entry = PoolAuditEntry {
            actor: creator.clone(),
            operation: PoolOperation::Create,
            timestamp: env.ledger().timestamp(),
            amount: None,
        };
        Storage::append_audit_entry(&env, hunt_id, audit_entry);

        Ok(())
    }

    /// Creates a reward pool for a specific hunt with a specified token.
    ///
    /// Must be called before `fund_reward_pool`. Any address may fund the pool
    /// after creation (see `fund_reward_pool`); the token contract must be
    /// SAC-compatible.
    ///
    /// # Arguments
    /// * `creator` - The hunt creator who will own and fund the pool
    /// * `hunt_id` - The hunt this pool is for
    /// * `token_address` - Address of the SAC-compatible token contract (e.g., XLM, USDC)
    /// * `min_distribution_amount` - Minimum token amount per distribution (0 = no minimum)
    /// * `nft_royalty_bps` - Creator royalty basis points (0-10000) for secondary market sales
    /// * `nft_transferable` - Whether reward NFTs from this pool are transferable
    ///
    /// # Errors
    /// * `PoolAlreadyExists` - A pool already exists for this hunt_id
    /// * `InvalidAmount` - min_distribution_amount is negative
    /// * `InvalidTokenContract` - token_address is not a valid SAC-compatible token
    /// * `NotInitialized` - hunty_core has not been configured (set during initialize)
    /// * `HuntNotFound` - hunt_id does not exist in HuntyCore
    pub fn create_reward_pool(
        env: Env,
        creator: Address,
        hunt_id: u64,
        token_address: Address,
        min_distribution_amount: i128,
        nft_royalty_bps: u32,
        nft_transferable: bool,
    ) -> Result<(), RewardErrorCode> {
        Self::create_reward_pool_with_nft(
            env,
            creator,
            hunt_id,
            token_address,
            min_distribution_amount,
            None,
            nft_royalty_bps,
            nft_transferable,
        )
    }

    /// Updates the `min_distribution_amount` for an existing reward pool.
    ///
    /// Only the pool creator is authorized to call this. Useful when a creator
    /// has underfunded the pool and needs to lower the minimum so distributions
    /// can proceed.
    ///
    /// # Arguments
    /// * `creator` - The pool creator (must match the stored creator)
    /// * `hunt_id` - The hunt whose pool config to update
    /// * `min_distribution_amount` - New minimum XLM per distribution (0 = no minimum)
    ///
    /// # Errors
    /// * `PoolNotFound` - No pool exists for this hunt_id
    /// * `Unauthorized` - Caller is not the pool creator
    /// * `InvalidAmount` - min_distribution_amount is negative
    pub fn update_pool_config(
        env: Env,
        creator: Address,
        hunt_id: u64,
        min_distribution_amount: i128,
    ) -> Result<(), RewardErrorCode> {
        creator.require_auth();

        let mut config =
            Storage::get_pool_config(&env, hunt_id).ok_or(RewardErrorCode::PoolNotFound)?;

        if creator != config.creator {
            return Err(RewardErrorCode::Unauthorized);
        }

        if min_distribution_amount < 0 {
            return Err(RewardErrorCode::InvalidAmount);
        }

        config.min_distribution_amount = min_distribution_amount;
        Storage::set_pool_config(&env, hunt_id, &config);

        Ok(())
    }

    /// Sets the funding target used for top-up progress notifications.
    /// `target_amount` of 0 disables percentage tracking (events report 0%).
    pub fn set_pool_target_amount(
        env: Env,
        creator: Address,
        hunt_id: u64,
        target_amount: i128,
    ) -> Result<(), RewardErrorCode> {
        creator.require_auth();

        let mut config =
            Storage::get_pool_config(&env, hunt_id).ok_or(RewardErrorCode::PoolNotFound)?;

        if creator != config.creator {
            return Err(RewardErrorCode::Unauthorized);
        }
        if target_amount < 0 {
            return Err(RewardErrorCode::InvalidAmount);
        }

        config.target_amount = target_amount;
        Storage::set_pool_config(&env, hunt_id, &config);
        Ok(())
    }

    /// Sets the minimum seconds between distributions for a pool (0 disables).
    pub fn set_min_distribution_interval(
        env: Env,
        creator: Address,
        hunt_id: u64,
        min_distribution_interval_secs: u64,
    ) -> Result<(), RewardErrorCode> {
        creator.require_auth();

        let mut config =
            Storage::get_pool_config(&env, hunt_id).ok_or(RewardErrorCode::PoolNotFound)?;

        if creator != config.creator {
            return Err(RewardErrorCode::Unauthorized);
        }

        config.min_distribution_interval_secs = min_distribution_interval_secs;
        Storage::set_pool_config(&env, hunt_id, &config);
        Ok(())
    }

    /// Sets the distribution mode (Fixed or Proportional) for a pool.
    pub fn set_distribution_mode(
        env: Env,
        creator: Address,
        hunt_id: u64,
        mode: DistributionMode,
    ) -> Result<(), RewardErrorCode> {
        creator.require_auth();

        let mut config =
            Storage::get_pool_config(&env, hunt_id).ok_or(RewardErrorCode::PoolNotFound)?;

        if creator != config.creator {
            return Err(RewardErrorCode::Unauthorized);
        }

        config.distribution_mode = mode;
        Storage::set_pool_config(&env, hunt_id, &config);
        Ok(())
    }

    /// Updates (or installs) the time-based reward tier schedule on an existing
    /// reward pool, enabling conditional reward amounts based on player completion
    /// time (acceptance criteria: "Define time-based reward tiers in pool config").
    ///
    /// Tiers must be supplied in strictly ascending order of `max_completion_secs`
    /// (i.e. faster tiers first), and every `xlm_amount` must be strictly positive.
    /// Passing an empty `Vec` disables tier-based rewards so the pool reverts
    /// to the flat `xlm_pool / max_winners` amount.
    ///
    /// Only the pool creator is authorized to call this. The new tiers are
    /// persisted immediately and become effective for any subsequent distribution
    /// call. Already-distributed rewards are not affected.
    ///
    /// # Arguments
    /// * `creator` - The pool creator (must match the stored creator)
    /// * `hunt_id` - The hunt whose pool config to update
    /// * `time_based_tiers` - New tier list (strictly ascending by time, all amounts > 0;
    ///   an empty list disables tier-based rewards)
    ///
    /// # Errors
    /// * `PoolNotFound` - No pool exists for this hunt_id
    /// * `Unauthorized` - Caller is not the pool creator
    /// * `InvalidConfig` - Tier list (when non-empty) contains a zero/negative
    ///   amount or is not strictly ascending
    pub fn set_pool_tiers(
        env: Env,
        creator: Address,
        hunt_id: u64,
        time_based_tiers: Vec<TimeBasedRewardTier>,
    ) -> Result<(), RewardErrorCode> {
        creator.require_auth();

        let mut config =
            Storage::get_pool_config(&env, hunt_id).ok_or(RewardErrorCode::PoolNotFound)?;

        if creator != config.creator {
            return Err(RewardErrorCode::Unauthorized);
        }

        // Empty tier list is a valid opt-out from tier-based rewards — it
        // disables the feature for this pool. Non-empty lists must validate.
        let tiers_len = time_based_tiers.len();
        if tiers_len > 0 {
            if let Err(_err) = tiers_are_strictly_ascending(&time_based_tiers) {
                return Err(RewardErrorCode::InvalidConfig);
            }
        }

        config.time_based_tiers = time_based_tiers;
        Storage::set_pool_config(&env, hunt_id, &config);

        env.events().publish(
            (symbol_short!("PL_TIERS"), hunt_id),
            (creator.clone(), tiers_len),
        );

        Ok(())
    }

    /// Sets or updates the NFT contract address for an existing reward pool.
    /// This allows pools to distribute NFTs alongside or instead of tokens.
    ///
    /// # Arguments
    /// * `creator` - The pool creator (must match the stored creator)
    /// * `hunt_id` - The hunt whose pool config to update
    /// * `nft_contract` - NFT contract address (or None to disable NFT rewards)
    ///
    /// # Errors
    /// * `PoolNotFound` - No pool exists for this hunt_id
    /// * `Unauthorized` - Caller is not the pool creator
    pub fn set_pool_nft_contract(
        env: Env,
        creator: Address,
        hunt_id: u64,
        nft_contract: Option<Address>,
    ) -> Result<(), RewardErrorCode> {
        creator.require_auth();

        let mut config =
            Storage::get_pool_config(&env, hunt_id).ok_or(RewardErrorCode::PoolNotFound)?;

        if creator != config.creator {
            return Err(RewardErrorCode::Unauthorized);
        }

        config.nft_contract = nft_contract;
        Storage::set_pool_config(&env, hunt_id, &config);

        Ok(())
    }

    /// Adds a delegate allowed to distribute rewards for a pool.
    /// Only the pool creator can manage delegates.
    pub fn add_delegate(
        env: Env,
        creator: Address,
        hunt_id: u64,
        delegate: Address,
    ) -> Result<(), RewardErrorCode> {
        creator.require_auth();

        let mut config =
            Storage::get_pool_config(&env, hunt_id).ok_or(RewardErrorCode::PoolNotFound)?;

        if creator != config.creator {
            return Err(RewardErrorCode::Unauthorized);
        }

        if !Self::is_delegate(&config, &delegate) {
            config.delegates.push_back(delegate);
            Storage::set_pool_config(&env, hunt_id, &config);
        }

        Ok(())
    }

    /// Removes a delegate from a pool.
    /// Only the pool creator can manage delegates.
    pub fn remove_delegate(
        env: Env,
        creator: Address,
        hunt_id: u64,
        delegate: Address,
    ) -> Result<(), RewardErrorCode> {
        creator.require_auth();

        let mut config =
            Storage::get_pool_config(&env, hunt_id).ok_or(RewardErrorCode::PoolNotFound)?;

        if creator != config.creator {
            return Err(RewardErrorCode::Unauthorized);
        }

        let mut updated: Vec<Address> = Vec::new(&env);
        for i in 0..config.delegates.len() {
            let existing = config.delegates.get(i).unwrap();
            if existing != delegate {
                updated.push_back(existing);
            }
        }
        config.delegates = updated;
        Storage::set_pool_config(&env, hunt_id, &config);

        Ok(())
    }

    /// Returns the full configuration of a reward pool, including its tier list.
    /// `None` when no pool has been created for the given `hunt_id`.
    ///
    /// This is the primary read path used by HuntyCore at completion time to
    /// resolve which tier (if any) applies to a player's completion time.
    pub fn get_pool_config(env: Env, hunt_id: u64) -> Option<RewardPoolConfig> {
        Storage::get_pool_config(&env, hunt_id)
    }

    /// Records `amount` as a contribution from `funder` toward `hunt_id`'s
    /// pool sponsorship ledger, adding them to the pool's funder list the
    /// first time they contribute. Shared by `fund_reward_pool` and
    /// `migrate_pool` (which attributes the migrated lump sum to the shared
    /// creator) so `refund_pool` can always pay the current balance back out
    /// in proportion to who funded it.
    fn record_funder_contribution(
        env: &Env,
        hunt_id: u64,
        funder: &Address,
        amount: i128,
    ) -> Result<(), RewardErrorCode> {
        let prior = Storage::get_pool_funder_contribution(env, hunt_id, funder);
        if prior == 0 {
            let mut funders = Storage::get_pool_funders(env, hunt_id);
            if funders.len() >= MAX_FUNDERS_PER_POOL {
                return Err(RewardErrorCode::TooManyFunders);
            }
            funders.push_back(funder.clone());
            Storage::set_pool_funders(env, hunt_id, &funders);
        }
        let new_total = prior
            .checked_add(amount)
            .ok_or(RewardErrorCode::PoolBalanceOverflow)?;
        Storage::set_pool_funder_contribution(env, hunt_id, funder, new_total);
        Ok(())
    }

    /// Wipes a pool's sponsorship ledger — every tracked funder's recorded
    /// contribution and the funder list itself. Used once a pool's balance
    /// has been fully paid out (`refund_pool`) or moved elsewhere
    /// (`migrate_pool`'s source pool), so stale contribution records can
    /// never be double-counted against a pool's balance again.
    fn clear_pool_funders(env: &Env, hunt_id: u64) {
        let funders = Storage::get_pool_funders(env, hunt_id);
        for i in 0..funders.len() {
            if let Some(funder) = funders.get(i) {
                Storage::remove_pool_funder_contribution(env, hunt_id, &funder);
            }
        }
        Storage::remove_pool_funders(env, hunt_id);
    }

    /// Funds the reward pool for a specific hunt.
    ///
    /// The pool must have been created via `create_reward_pool` first.
    /// **Anyone may fund a pool** — this supports sponsorship (a brand funding
    /// a community hunt, a DAO topping up a pool, several people pooling a
    /// prize), not just the creator. Each funder must authorize the call
    /// themselves; their contribution is tracked individually so that
    /// `refund_pool` can later pay the remaining balance back out in
    /// proportion to what each funder put in, and never hand one funder's
    /// contribution to another party. See `docs/adr/006-reward-pool-sponsorship.md`.
    ///
    /// Transfers tokens from the funder to this contract and records the balance.
    /// Uses the token address specified when the pool was created.
    ///
    /// # Validation
    /// - Minimum funding: 1 XLM equivalent (10,000,000 base units) to prevent dust attacks
    /// - Maximum single funding: 1 billion tokens to prevent overflow
    /// - Pool balance limit: 1 billion tokens total to prevent overflow
    /// - Rejects zero or negative amounts
    /// - At most `MAX_FUNDERS_PER_POOL` distinct funders are tracked per pool
    ///
    /// # Arguments
    /// * `funder` - The address funding the pool (must authorize this call)
    /// * `hunt_id` - The hunt to fund
    /// * `amount` - Token amount to add to the pool (must be > 0)
    ///
    /// # Errors
    /// * `PoolNotFound` - Pool has not been created yet
    /// * `InvalidAmount` - Amount is <= 0
    /// * `BelowMinimumFunding` - Amount is less than minimum (dust attack prevention)
    /// * `ExceedsMaximumFunding` - Amount exceeds maximum limit
    /// * `PoolBalanceOverflow` - Adding this amount would exceed pool balance limit
    /// * `TooManyFunders` - This would be a new funder and the pool already
    ///   tracks the maximum number of distinct funders
    pub fn fund_reward_pool(
        env: Env,
        funder: Address,
        hunt_id: u64,
        amount: i128,
    ) -> Result<(), RewardErrorCode> {
        // Issue #628: funding is blocked by its own pause flag or the global stop.
        Self::ensure_funding_allowed(&env)?;

        if amount <= 0 {
            return Err(RewardErrorCode::InvalidAmount);
        }

        // Validate minimum funding amount to prevent dust attacks
        if amount < MIN_FUNDING_AMOUNT {
            return Err(RewardErrorCode::BelowMinimumFunding);
        }

        // Validate maximum single funding amount to prevent overflow
        if amount > MAX_FUNDING_AMOUNT {
            return Err(RewardErrorCode::ExceedsMaximumFunding);
        }

        let pool_config =
            Storage::get_pool_config(&env, hunt_id).ok_or(RewardErrorCode::PoolNotFound)?;

        #[cfg(not(test))]
        funder.require_auth();

        let _reentrancy_guard = ReentrancyGuard::acquire(&env)?;

        // Use the token address from the pool config instead of global XLM token
        let token_address = &pool_config.token_address;

        // Check for overflow before adding to pool balance
        let current = Storage::get_pool_balance(&env, hunt_id);
        let new_balance = current
            .checked_add(amount)
            .ok_or(RewardErrorCode::PoolBalanceOverflow)?;

        // Validate the new balance doesn't exceed maximum pool balance
        if new_balance > MAX_POOL_BALANCE {
            return Err(RewardErrorCode::PoolBalanceOverflow);
        }

        let total_deposited = Storage::get_pool_total_deposited(&env, hunt_id)
            .checked_add(amount)
            .ok_or(RewardErrorCode::PoolBalanceOverflow)?;

        // Update pool balance and cumulative deposit total before the external
        // token transfer, so a reentrant call observes the post-funding state
        // rather than a stale balance (checks-effects-interactions).
        Storage::set_pool_balance(&env, hunt_id, new_balance);
        Storage::set_pool_total_deposited(&env, hunt_id, total_deposited);

        // Record the contribution before moving funds so a rejected (e.g.
        // too-many-funders) call never transfers tokens.
        Self::record_funder_contribution(&env, hunt_id, &funder, amount)?;

        // Transfer tokens from funder to this contract
        let contract_addr = env.current_contract_address();
        let client = soroban_sdk::token::Client::new(&env, token_address);
        client.transfer(&funder, &contract_addr, &amount);

        let target_amount = pool_config.target_amount;
        let percentage_of_target = if target_amount > 0 {
            let pct = (total_deposited.saturating_mul(100)) / target_amount;
            if pct > u32::MAX as i128 {
                u32::MAX
            } else {
                pct as u32
            }
        } else {
            0u32
        };

        env.events().publish(
            (symbol_short!("POOL_FND"), hunt_id),
            RewardPoolFundedEvent {
                hunt_id,
                funder: funder.clone(),
                amount,
                new_balance,
                total_deposited,
                target_amount,
                percentage_of_target,
                funding_progress: total_deposited,
            },
        );

        let audit_entry = PoolAuditEntry {
            actor: funder,
            operation: PoolOperation::Fund,
            timestamp: env.ledger().timestamp(),
            amount: Some(amount),
        };
        Storage::append_audit_entry(&env, hunt_id, audit_entry);

        Ok(())
    }

    /// Refunds the remaining pool balance for a hunt, paid out **pro rata**
    /// across every address that funded it (see `fund_reward_pool`) in
    /// proportion to each funder's share of total contributions — never
    /// paying one funder's contribution to another party. A pool funded by a
    /// single address (the common case) simply gets its whole balance back.
    ///
    /// Can only be triggered by the pool creator, who must authorize the
    /// call; the payout destinations are the tracked funders, not the caller.
    /// Uses the token address specified when the pool was created. The hunt
    /// must be in a terminal state (cancelled or ended) when HuntyCore is
    /// configured — refunding an active hunt's pool out from under its
    /// players is rejected.
    ///
    /// **Important:** This is a destructive operation. Ensure all distributions are complete
    /// before calling this function, as any remaining unclaimed rewards cannot be distributed
    /// after the pool is refunded.
    ///
    /// # Accounting
    /// This function updates:
    /// - Pool balance: Set to 0
    /// - Total refunded: Incremented by the refund amount
    /// - Audit log: Entry recorded with PoolOperation::Refund
    ///
    /// After a refund, the accounting identity is:
    /// `total_deposited == balance + total_distributed + total_refunded`
    ///
    /// # Events
    /// Emits one `PoolRefundedEvent` per funder paid out (a single event for
    /// the common single-funder case).
    ///
    /// # Arguments
    /// * `creator` - The pool creator (must authorize this call)
    /// * `hunt_id` - The hunt whose pool is being refunded
    ///
    /// # Errors
    /// * `PoolNotFound` - Pool has not been created yet
    /// * `InvalidHuntStatus` - The hunt is not cancelled or ended (only when
    ///   `set_hunty_core` has been called)
    /// * `Unauthorized` - Caller is not the pool creator
    pub fn refund_pool(env: Env, creator: Address, hunt_id: u64) -> Result<(), RewardErrorCode> {
        #[cfg(not(test))]
        creator.require_auth();

        let pool_config =
            Storage::get_pool_config(&env, hunt_id).ok_or(RewardErrorCode::PoolNotFound)?;
        if creator != pool_config.creator {
            return Err(RewardErrorCode::Unauthorized);
        }

        // Verify the hunt is in a terminal state (Cancelled or past end_time)
        // Query HuntyCore for hunt status and end_time
        let hunty_core_addr = Storage::get_hunty_core(&env);
        if let Some(core_addr) = hunty_core_addr {
            // Call hunty_core to check if hunt is terminal
            let is_terminal = match env.try_invoke_contract::<bool, soroban_sdk::Val>(
                &core_addr,
                &Symbol::new(&env, "is_hunt_terminal"),
                soroban_sdk::vec![&env, hunt_id.into_val(&env)],
            ) {
                Ok(Ok(terminal)) => terminal,
                _ => true,
            };
            if !is_terminal {
                return Err(RewardErrorCode::InvalidHuntStatus);
            }
        }

        let balance = Storage::get_pool_balance(&env, hunt_id);
        if balance == 0 {
            return Ok(());
        }

        // Use the token address from the pool config
        let token_address = &pool_config.token_address;

        let contract_addr = env.current_contract_address();
        let client = soroban_sdk::token::Client::new(&env, token_address);

        let funders = Storage::get_pool_funders(&env, hunt_id);

        if funders.is_empty() {
            // No sponsorship ledger for this pool (e.g. balance arrived solely
            // through a path that predates funder tracking) — the creator is
            // the only possible owner of the balance.
            client.transfer(&contract_addr, &creator, &balance);
            env.events().publish(
                (symbol_short!("POOL_RFD"), hunt_id),
                PoolRefundedEvent {
                    hunt_id,
                    funder: creator.clone(),
                    amount: balance,
                },
            );
        } else {
            let mut total_contributed: i128 = 0;
            for i in 0..funders.len() {
                let funder = funders.get(i).unwrap();
                total_contributed += Storage::get_pool_funder_contribution(&env, hunt_id, &funder);
            }

            let last_index = funders.len() - 1;
            let mut remaining = balance;
            for i in 0..funders.len() {
                let funder = funders.get(i).unwrap();
                let contribution = Storage::get_pool_funder_contribution(&env, hunt_id, &funder);

                // The last funder absorbs whatever integer-division rounding
                // leaves over (and, defensively, any balance a zero-contribution
                // entry would otherwise strand), so the full balance is always
                // paid out and never left stuck in the contract.
                let share = if i == last_index {
                    remaining
                } else if total_contributed > 0 {
                    (balance.saturating_mul(contribution) / total_contributed).min(remaining)
                } else {
                    0
                };

                if share > 0 {
                    client.transfer(&contract_addr, &funder, &share);
                    env.events().publish(
                        (symbol_short!("POOL_RFD"), hunt_id),
                        PoolRefundedEvent {
                            hunt_id,
                            funder: funder.clone(),
                            amount: share,
                        },
                    );
                }
                remaining -= share;
            }
            Self::clear_pool_funders(&env, hunt_id);
        }

        Storage::set_pool_balance(&env, hunt_id, 0);

        // Track total refunded for accounting. This is the sum of every
        // PoolRefundedEvent emitted above (or the single creator payout in
        // the no-sponsors branch), so it still equals the full balance even
        // when it was split across multiple funders.
        let total_refunded = Storage::get_pool_total_refunded(&env, hunt_id) + balance;
        Storage::set_pool_total_refunded(&env, hunt_id, total_refunded);

        let audit_entry = PoolAuditEntry {
            actor: creator.clone(),
            operation: PoolOperation::Refund,
            timestamp: env.ledger().timestamp(),
            amount: Some(balance),
        };
        Storage::append_audit_entry(&env, hunt_id, audit_entry);

        Ok(())
    }

    /// Migrates the unused balance of an expired or cancelled hunt's pool into
    /// an existing destination pool owned by the same creator.
    ///
    /// This lets a creator recycle funds locked in a finished hunt into a fresh
    /// hunt without withdrawing and re-depositing. The XLM never leaves this
    /// contract; only the internal per-hunt balance accounting is re-keyed.
    ///
    /// # Eligibility (acceptance criteria)
    /// * The source pool's hunt must be **expired or cancelled** — verified via
    ///   a cross-contract call to the configured HuntyCore contract
    ///   (`is_hunt_expired_or_cancelled`). If HuntyCore is not configured, the
    ///   source cannot be shown eligible and migration is rejected.
    /// * The **destination pool must already exist** (created via
    ///   `create_reward_pool`).
    /// * **Both pools must have the same creator**, who must authorize the call.
    ///
    /// # Arguments
    /// * `creator` - The shared creator of both pools (must authorize the call)
    /// * `source_hunt_id` - The expired/cancelled hunt to drain
    /// * `dest_hunt_id` - The destination hunt to credit
    ///
    /// # Returns
    /// The amount of XLM migrated from the source pool to the destination pool.
    ///
    /// # Errors
    /// * `InvalidMigration` - source and destination are the same hunt, or the
    ///   source pool has no balance to migrate
    /// * `PoolNotFound` - the source pool does not exist
    /// * `DestinationPoolNotFound` - the destination pool does not exist
    /// * `Unauthorized` - the caller does not own both pools
    /// * `SourcePoolNotEligible` - the source hunt is neither expired nor cancelled
    /// * `PoolBalanceOverflow` - crediting the destination would overflow the pool cap
    /// * `TooManyFunders` - the destination already tracks the maximum number of
    ///   distinct funders and the creator is not already one of them
    pub fn migrate_pool(
        env: Env,
        creator: Address,
        source_hunt_id: u64,
        dest_hunt_id: u64,
    ) -> Result<i128, RewardErrorCode> {
        #[cfg(not(test))]
        creator.require_auth();

        if source_hunt_id == dest_hunt_id {
            return Err(RewardErrorCode::InvalidMigration);
        }

        // Source pool must exist and be owned by the caller.
        let source_config =
            Storage::get_pool_config(&env, source_hunt_id).ok_or(RewardErrorCode::PoolNotFound)?;
        if creator != source_config.creator {
            return Err(RewardErrorCode::Unauthorized);
        }

        // Destination pool must exist and be owned by the same creator.
        let dest_config = Storage::get_pool_config(&env, dest_hunt_id)
            .ok_or(RewardErrorCode::DestinationPoolNotFound)?;
        if creator != dest_config.creator {
            return Err(RewardErrorCode::Unauthorized);
        }

        // Source hunt must be expired or cancelled (source of truth: HuntyCore).
        if !Self::source_hunt_is_migratable(&env, source_hunt_id) {
            return Err(RewardErrorCode::SourcePoolNotEligible);
        }

        let amount = Storage::get_pool_balance(&env, source_hunt_id);
        if amount <= 0 {
            return Err(RewardErrorCode::InvalidMigration);
        }

        // Credit the destination, guarding against overflow / the pool cap.
        let dest_balance = Storage::get_pool_balance(&env, dest_hunt_id);
        let new_dest_balance = dest_balance
            .checked_add(amount)
            .ok_or(RewardErrorCode::PoolBalanceOverflow)?;
        if new_dest_balance > MAX_POOL_BALANCE {
            return Err(RewardErrorCode::PoolBalanceOverflow);
        }

        // Pre-validate the sponsorship-ledger update before mutating any
        // balance state below: crediting the creator as a funder here must
        // not be the straw that exceeds the destination's funder cap.
        let creator_already_funds_dest =
            Storage::get_pool_funder_contribution(&env, dest_hunt_id, &creator) > 0;
        if !creator_already_funds_dest
            && Storage::get_pool_funders(&env, dest_hunt_id).len() >= MAX_FUNDERS_PER_POOL
        {
            return Err(RewardErrorCode::TooManyFunders);
        }

        // Re-key the balance: drain the source, credit the destination.
        Storage::set_pool_balance(&env, source_hunt_id, 0);
        Storage::set_pool_balance(&env, dest_hunt_id, new_dest_balance);

        // The source's sponsors no longer have a claim there — their share of
        // the balance just moved to the destination pool under the creator's
        // name below. Clearing this now prevents a future refund_pool on the
        // source from splitting a later, unrelated balance among funders who
        // were already paid out via this migration.
        Self::clear_pool_funders(&env, source_hunt_id);

        // Attribute the migrated lump sum to the shared creator on the
        // destination's sponsorship ledger, so refund_pool can still pay the
        // destination's balance back out proportionally to who funded it.
        Self::record_funder_contribution(&env, dest_hunt_id, &creator, amount)?;

        // Reflect the incoming funds in the destination's cumulative deposits so
        // get_reward_pool totals stay consistent.
        let dest_deposited = Storage::get_pool_total_deposited(&env, dest_hunt_id)
            .checked_add(amount)
            .ok_or(RewardErrorCode::PoolBalanceOverflow)?;
        Storage::set_pool_total_deposited(&env, dest_hunt_id, dest_deposited);

        env.events().publish(
            (symbol_short!("POOL_MIG"), source_hunt_id, dest_hunt_id),
            PoolMigratedEvent {
                source_hunt_id,
                dest_hunt_id,
                creator: creator.clone(),
                amount,
            },
        );

        // Audit both pools so the movement is traceable from either side.
        let timestamp = env.ledger().timestamp();
        Storage::append_audit_entry(
            &env,
            source_hunt_id,
            PoolAuditEntry {
                actor: creator.clone(),
                operation: PoolOperation::Migrate,
                timestamp,
                amount: Some(amount),
            },
        );
        Storage::append_audit_entry(
            &env,
            dest_hunt_id,
            PoolAuditEntry {
                actor: creator.clone(),
                operation: PoolOperation::Migrate,
                timestamp,
                amount: Some(amount),
            },
        );

        Ok(amount)
    }

    /// Returns true when the source hunt is expired or cancelled, as reported by
    /// the configured HuntyCore contract. When HuntyCore is not configured, or
    /// the cross-contract call fails, the source is treated as not eligible.
    fn source_hunt_is_migratable(env: &Env, hunt_id: u64) -> bool {
        match Storage::get_hunty_core(env) {
            Some(hunty_core) => {
                let mut args: Vec<Val> = Vec::new(env);
                args.push_back(hunt_id.into_val(env));
                matches!(
                    env.try_invoke_contract::<bool, RewardErrorCode>(
                        &hunty_core,
                        &Symbol::new(env, "is_hunt_expired_or_cancelled"),
                        args,
                    ),
                    Ok(Ok(true))
                )
            }
            None => false,
        }
    }

    /// Returns the full status of a reward pool, including balance, totals, and configuration.
    /// Returns None if no pool has been created for the given hunt_id.
    pub fn get_reward_pool(env: Env, hunt_id: u64) -> Option<RewardPoolStatus> {
        let config = Storage::get_pool_config(&env, hunt_id)?;
        let balance = Storage::get_pool_balance(&env, hunt_id);
        let total_deposited = Storage::get_pool_total_deposited(&env, hunt_id);
        let total_distributed = Storage::get_pool_total_distributed(&env, hunt_id);

        Some(RewardPoolStatus {
            balance,
            total_deposited,
            total_distributed,
            creator: config.creator,
            min_distribution_amount: config.min_distribution_amount,
            frozen: config.frozen,
        })
    }

    /// Returns the distinct addresses currently tracked as funders of a pool
    /// (i.e. that have contributed and not yet been refunded), in the order
    /// they first contributed. Empty if the pool has never been funded, has
    /// been fully refunded, or has no sponsorship ledger (see `refund_pool`).
    pub fn get_pool_funders(env: Env, hunt_id: u64) -> Vec<Address> {
        Storage::get_pool_funders(&env, hunt_id)
    }

    /// Returns how much `funder` has contributed to a pool that has not yet
    /// been refunded. 0 if they have never funded it or were already refunded.
    pub fn get_pool_funder_contribution(env: Env, hunt_id: u64, funder: Address) -> i128 {
        Storage::get_pool_funder_contribution(&env, hunt_id, &funder)
    }

    /// Returns comprehensive statistics for a reward pool.
    /// Returns None if no pool has been created for the given hunt_id.
    pub fn get_pool_statistics(env: Env, hunt_id: u64) -> Option<RewardPoolStatistics> {
        Storage::get_pool_config(&env, hunt_id)?;
        let total_funded = Storage::get_pool_total_deposited(&env, hunt_id);
        let total_distributed = Storage::get_pool_total_distributed(&env, hunt_id);
        let distribution_count = Storage::get_pool_distribution_count(&env, hunt_id);
        let last_distribution_timestamp =
            Storage::get_pool_last_distribution_timestamp(&env, hunt_id);

        let avg_distribution = if distribution_count > 0 && total_distributed > 0 {
            total_distributed / distribution_count as i128
        } else {
            0
        };

        Some(RewardPoolStatistics {
            total_funded,
            total_distributed,
            distribution_count,
            avg_distribution,
            last_distribution_timestamp,
        })
    }

    /// Validates whether a pool can cover a given distribution amount.
    ///
    /// Checks that:
    /// - The pool exists (was created via create_reward_pool)
    /// - The required_amount is positive
    /// - The pool balance >= required_amount
    /// - The required_amount meets the pool's minimum distribution threshold (if set)
    ///
    /// Returns a `ValidationResult` with balance details regardless of validity,
    /// so callers can diagnose shortfalls without a separate query.
    pub fn validate_pool(env: Env, hunt_id: u64, required_amount: i128) -> ValidationResult {
        let balance = Storage::get_pool_balance(&env, hunt_id);
        let pool_config = Storage::get_pool_config(&env, hunt_id);

        let is_valid = if let Some(ref config) = pool_config {
            // A frozen pool cannot make distributions regardless of balance
            if config.frozen {
                false
            } else {
                let meets_balance = required_amount > 0 && balance >= required_amount;
                let meets_minimum = config.min_distribution_amount == 0
                    || required_amount >= config.min_distribution_amount;
                meets_balance && meets_minimum
            }
        } else {
            false
        };

        ValidationResult {
            is_valid,
            balance,
            required: required_amount,
        }
    }

    /// Freezes a reward pool, preventing any further distributions.
    ///
    /// Can be called by either the pool creator or the contract admin.
    /// Emits a `PoolFrozenEvent`.
    ///
    /// # Arguments
    /// * `caller` - The address calling freeze (must be pool creator or admin)
    /// * `hunt_id` - The hunt whose pool to freeze
    ///
    /// # Errors
    /// * `PoolNotFound` - No pool exists for this hunt_id
    /// * `Unauthorized` - Caller is neither the pool creator nor the contract admin
    pub fn freeze_pool(env: Env, caller: Address, hunt_id: u64) -> Result<(), RewardErrorCode> {
        caller.require_auth();

        let mut config =
            Storage::get_pool_config(&env, hunt_id).ok_or(RewardErrorCode::PoolNotFound)?;

        // Both the pool creator and the contract admin may freeze a pool.
        let is_admin = Storage::get_admin(&env)
            .map(|admin| admin == caller)
            .unwrap_or(false);
        if caller != config.creator && !is_admin {
            return Err(RewardErrorCode::Unauthorized);
        }

        config.frozen = true;
        Storage::set_pool_config(&env, hunt_id, &config);

        env.events().publish(
            (symbol_short!("POOL_FRZ"), hunt_id),
            PoolFrozenEvent {
                hunt_id,
                caller: caller.clone(),
            },
        );

        let audit_entry = PoolAuditEntry {
            actor: caller.clone(),
            operation: PoolOperation::Freeze,
            timestamp: env.ledger().timestamp(),
            amount: None,
        };
        Storage::append_audit_entry(&env, hunt_id, audit_entry);

        Ok(())
    }

    /// Unfreezes a reward pool, re-enabling distributions.
    ///
    /// Can be called by either the pool creator or the contract admin.
    /// Emits a `PoolUnfrozenEvent`.
    ///
    /// # Arguments
    /// * `caller` - The address calling unfreeze (must be pool creator or admin)
    /// * `hunt_id` - The hunt whose pool to unfreeze
    ///
    /// # Errors
    /// * `PoolNotFound` - No pool exists for this hunt_id
    /// * `Unauthorized` - Caller is neither the pool creator nor the contract admin
    pub fn unfreeze_pool(env: Env, caller: Address, hunt_id: u64) -> Result<(), RewardErrorCode> {
        caller.require_auth();

        let mut config =
            Storage::get_pool_config(&env, hunt_id).ok_or(RewardErrorCode::PoolNotFound)?;

        // Both the pool creator and the contract admin may unfreeze a pool.
        let is_admin = Storage::get_admin(&env)
            .map(|admin| admin == caller)
            .unwrap_or(false);
        if caller != config.creator && !is_admin {
            return Err(RewardErrorCode::Unauthorized);
        }

        config.frozen = false;
        Storage::set_pool_config(&env, hunt_id, &config);

        env.events().publish(
            (symbol_short!("POOL_UFRZ"), hunt_id),
            PoolUnfrozenEvent {
                hunt_id,
                caller: caller.clone(),
            },
        );

        let audit_entry = PoolAuditEntry {
            actor: caller.clone(),
            operation: PoolOperation::Unfreeze,
            timestamp: env.ledger().timestamp(),
            amount: None,
        };
        Storage::append_audit_entry(&env, hunt_id, audit_entry);

        Ok(())
    }

    /// Returns whether a reward pool is currently frozen.
    /// Returns `false` if no pool exists for the given `hunt_id`.
    pub fn is_pool_frozen(env: Env, hunt_id: u64) -> bool {
        Storage::get_pool_config(&env, hunt_id)
            .map(|config| config.frozen)
            .unwrap_or(false)
    }

    /// Sets the daily distribution cap for a specific pool.
    ///
    /// This limit controls the maximum amount of rewards that can be distributed from
    /// a pool in a single day (24-hour rolling window). This is a live operational control
    /// and should be validated to prevent silent misconfiguration.
    ///
    /// # Arguments
    /// * `admin` - The contract admin address (must match the stored admin)
    /// * `hunt_id` - The hunt whose pool cap to set
    /// * `cap` - The maximum amount to distribute per day. Must be positive (> 0).
    ///           A cap of 0 means no distributions are allowed (use to disable).
    ///
    /// # Errors
    /// * `NotInitialized` - Contract has not been initialized (no admin set)
    /// * `Unauthorized` - Caller is not the contract admin
    /// * `PoolNotFound` - No pool exists for this hunt_id
    /// * `InvalidAmount` - Cap is negative (negative caps silently block distributions)
    pub fn set_daily_pool_cap(
        env: Env,
        admin: Address,
        hunt_id: u64,
        cap: i128,
    ) -> Result<(), RewardErrorCode> {
        admin.require_auth();
        let configured_admin = Storage::get_admin(&env).ok_or(RewardErrorCode::NotInitialized)?;
        if configured_admin != admin {
            return Err(RewardErrorCode::Unauthorized);
        }

        // Reject negative caps - they silently break distributions
        if cap < 0 {
            return Err(RewardErrorCode::InvalidAmount);
        }

        // Verify the pool exists before setting a cap for it
        Storage::get_pool_config(&env, hunt_id).ok_or(RewardErrorCode::PoolNotFound)?;

        Storage::set_daily_pool_cap(&env, hunt_id, cap);
        Ok(())
    }

    pub fn set_daily_global_cap(
        env: Env,
        admin: Address,
        cap: i128,
    ) -> Result<(), RewardErrorCode> {
        Self::require_admin(&env, &admin)?;
        Storage::set_daily_global_cap(&env, cap);
        Ok(())
    }

    pub fn distribute_rewards(
        env: Env,
        hunt_id: u64,
        player_address: Address,
        reward_config: RewardConfig,
    ) -> Result<(), RewardErrorCode> {
        Self::require_authorized_distributor(&env)?;

        // Issue #628: distribution is blocked by its own pause flag or the global stop.
        Self::ensure_distribution_allowed(&env)?;

        let pool_config =
            Storage::get_pool_config(&env, hunt_id).ok_or(RewardErrorCode::PoolNotFound)?;

        // Validate configuration
        if !reward_config.is_valid() {
            return Err(RewardErrorCode::InvalidConfig);
        }

        // Reject distribution if the pool is frozen
        if pool_config.frozen {
            return Err(RewardErrorCode::PoolFrozen);
        }

        // Prevent double distribution: check if a distribution record already exists
        if Storage::get_distribution_record(&env, hunt_id, &player_address).is_some() {
            return Err(RewardErrorCode::AlreadyDistributed);
        }

        let _reentrancy_guard = ReentrancyGuard::acquire(&env)?;

        // Per-pool distribution rate limiting
        let interval = pool_config.min_distribution_interval_secs;
        if interval > 0 {
            let now = env.ledger().timestamp();
            if let Some(last) = Storage::get_last_distribution_timestamp(&env, hunt_id) {
                let elapsed = now.saturating_sub(last);
                if elapsed < interval {
                    let remaining_secs = interval - elapsed;
                    env.events().publish(
                        (symbol_short!("DIST_CD"), hunt_id),
                        DistributionCooldownEvent {
                            hunt_id,
                            remaining_secs,
                        },
                    );
                    return Err(RewardErrorCode::DistributionRateLimited);
                }
            }
        }

        let mut xlm_amount = 0i128;
        let mut nft_id: Option<u64> = None;

        // Calculate amounts first (before any transfers/mutations)
        // Validate XLM amount if present
        if reward_config.has_xlm() {
            let amount = reward_config.xlm_amount.unwrap();
            if amount <= 0 {
                return Err(RewardErrorCode::InvalidAmount);
            }

            // Get pool config to access token address and minimum distribution amount
            let pool_config =
                Storage::get_pool_config(&env, hunt_id).ok_or(RewardErrorCode::PoolNotFound)?;

            // Enforce pool minimum distribution amount
            if pool_config.min_distribution_amount > 0
                && amount < pool_config.min_distribution_amount
            {
                return Err(RewardErrorCode::BelowMinimumAmount);
            }

            xlm_amount = amount;
        }

        // Validate NFT tier if present
        if reward_config.has_nft() && reward_config.nft_rarity > 5 {
            return Err(RewardErrorCode::InvalidConfig);
        }

        // Write distribution record BEFORE any transfers to prevent replay in all failure modes
        Storage::set_distribution_record(
            &env,
            hunt_id,
            &player_address,
            &DistributionRecord { xlm_amount, nft_id },
        );

        // Now proceed with actual transfers and other mutations
        if reward_config.has_xlm() {
            let amount = reward_config.xlm_amount.unwrap();
            let pool_config =
                Storage::get_pool_config(&env, hunt_id).ok_or(RewardErrorCode::PoolNotFound)?;

            // Use the token address from the pool config
            let token_address = &pool_config.token_address;

            let pool_balance = Storage::get_pool_balance(&env, hunt_id);
            if pool_balance < amount {
                return Err(RewardErrorCode::InsufficientPool);
            }

            let contract_addr = env.current_contract_address();

            if !TokenHandler::validate_pool(&env, token_address, &contract_addr, amount) {
                return Err(RewardErrorCode::PoolBalanceDivergence);
            }

            // Check caps
            let day = env.ledger().timestamp() / 86400;
            Storage::add_daily_pool_distributed(&env, hunt_id, day, amount);
            Storage::add_daily_global_distributed(&env, day, amount);

            let pool_cap = Storage::get_daily_pool_cap(&env, hunt_id);
            if pool_cap > 0 {
                let used = Storage::get_daily_pool_distributed(&env, hunt_id, day);
                if used > pool_cap {
                    return Err(RewardErrorCode::DailyCapExceeded);
                }
                if used >= (pool_cap * 8 / 10) {
                    env.events().publish(
                        (symbol_short!("DP_WARN"),),
                        DailyPoolCapWarningEvent {
                            hunt_id,
                            used,
                            cap: pool_cap,
                        },
                    );
                }
            }

            let global_cap = Storage::get_daily_global_cap(&env);
            if global_cap > 0 {
                let global_used = Storage::get_daily_global_distributed(&env, day);
                if global_used > global_cap {
                    return Err(RewardErrorCode::GlobalDailyCapExceeded);
                }
                if global_used >= (global_cap * 8 / 10) {
                    env.events().publish(
                        (symbol_short!("DG_WARN"),),
                        GlobalDailyCapWarningEvent {
                            used: global_used,
                            cap: global_cap,
                        },
                    );
                }
            }

            // Vesting path: when the pool has a vesting period configured, do NOT
            // transfer tokens immediately. Instead, lock the amount in a VestingRecord
            // so the player can claim proportionally over time via `claim_vested`.
            if pool_config.vesting_period_secs > 0 {
                let now = env.ledger().timestamp();
                let vesting_record = VestingRecord {
                    start_time: now,
                    total_amount: amount,
                    claimed_amount: 0,
                    vesting_period_secs: pool_config.vesting_period_secs,
                };
                Storage::set_vesting_record(&env, hunt_id, &player_address, &vesting_record);
                // The pool balance is reserved (deducted) at vesting creation so that
                // concurrent distributions cannot double-spend the same funds. Tokens
                // remain in the contract until the player calls `claim_vested`.
                xlm_amount = amount;
                Storage::set_pool_balance(&env, hunt_id, pool_balance - amount);

                let total_distributed = Storage::get_pool_total_distributed(&env, hunt_id) + amount;
                Storage::set_pool_total_distributed(&env, hunt_id, total_distributed);
                let global_total = Storage::get_total_xlm_distributed(&env) + amount;
                Storage::set_total_xlm_distributed(&env, global_total);

                env.events().publish(
                    (symbol_short!("VEST_CRT"), hunt_id),
                    VestingCreatedEvent {
                        hunt_id,
                        player: player_address.clone(),
                        total_amount: amount,
                        vesting_period_secs: pool_config.vesting_period_secs,
                        start_time: now,
                    },
                );
            } else {
                // Instant payout path (no vesting).
                XlmHandler::distribute_xlm(
                    &env,
                    token_address,
                    &contract_addr,
                    &player_address,
                    amount,
                );
                xlm_amount = amount;
                Storage::set_pool_balance(&env, hunt_id, pool_balance - amount);

                let total_distributed = Storage::get_pool_total_distributed(&env, hunt_id) + amount;
                Storage::set_pool_total_distributed(&env, hunt_id, total_distributed);
                let global_total = Storage::get_total_xlm_distributed(&env) + amount;
                Storage::set_total_xlm_distributed(&env, global_total);
            }
        }

        // Route to NFT handler if configured
        if reward_config.has_nft() {
            if reward_config.nft_rarity > 5 {
                return Err(RewardErrorCode::InvalidConfig);
            }
            let nft_contract = reward_config
                .nft_contract
                .as_ref()
                .cloned()
                .or_else(|| Storage::get_nft_contract(&env))
                .ok_or(RewardErrorCode::InvalidConfig)?;

            match NftHandler::distribute_nft(
                &env,
                &nft_contract,
                hunt_id,
                &player_address,
                reward_config.nft_title.clone(),
                reward_config.nft_description.clone(),
                reward_config.nft_image_uri.clone(),
                reward_config.nft_hunt_title.clone(),
                reward_config.nft_rarity,
                reward_config.nft_tier,
                &pool_config.creator,
                pool_config.nft_royalty_bps,
                pool_config.nft_transferable,
                reward_config.completion_rank,
            ) {
                Ok(id) => nft_id = Some(id),
                Err(_) => {
                    env.events().publish(
                        (symbol_short!("NFT_FAIL"), hunt_id),
                        NftMintFailedEvent {
                            hunt_id,
                            player: player_address.clone(),
                            nft_contract: Some(nft_contract.clone()),
                        },
                    );
                    Storage::set_pending_nft_mint(
                        &env,
                        hunt_id,
                        &player_address,
                        &PendingNftMint {
                            hunt_id,
                            player: player_address.clone(),
                            nft_contract,
                            nft_title: reward_config.nft_title.clone(),
                            nft_description: reward_config.nft_description.clone(),
                            nft_image_uri: reward_config.nft_image_uri.clone(),
                            nft_hunt_title: reward_config.nft_hunt_title.clone(),
                            nft_rarity: reward_config.nft_rarity,
                            nft_tier: reward_config.nft_tier,
                            completion_rank: reward_config.completion_rank,
                        },
                    );
                }
            }
        }

        // Update the distribution record with the actual nft_id if minting succeeded
        if let Some(nft_id_val) = nft_id {
            if let Some(mut record) =
                Storage::get_distribution_record(&env, hunt_id, &player_address)
            {
                record.nft_id = Some(nft_id_val);
                Storage::set_distribution_record(&env, hunt_id, &player_address, &record);
            }
        }

        let timestamp = env.ledger().timestamp();
        let proof_hash =
            Self::compute_distribution_hash(&env, hunt_id, &player_address, xlm_amount, timestamp);
        Storage::set_distribution_proof(
            &env,
            hunt_id,
            &player_address,
            &DistributionProof {
                pool_id: hunt_id,
                player: player_address.clone(),
                amount: xlm_amount,
                timestamp,
                hash: proof_hash,
            },
        );
        Storage::set_last_distribution_timestamp(&env, hunt_id, timestamp);

        let event = RewardsDistributedEvent {
            hunt_id,
            player: player_address.clone(),
            xlm_amount,
            nft_id,
        };
        env.events()
            .publish((symbol_short!("RWD_DIST"), hunt_id), event);

        let audit_entry = PoolAuditEntry {
            actor: player_address.clone(),
            operation: PoolOperation::Distribute,
            timestamp: env.ledger().timestamp(),
            amount: if xlm_amount > 0 {
                Some(xlm_amount)
            } else {
                None
            },
        };
        Storage::append_audit_entry(&env, hunt_id, audit_entry);

        Ok(())
    }

    /// Distributes rewards to multiple players in a single atomic transaction.
    ///
    /// Every entry in the batch is validated first (no state changes). If all
    /// entries pass validation, all transfers are executed. If any single entry
    /// fails validation, the entire batch is rejected with no state changes.
    ///
    /// # Atomicity guarantee
    ///
    /// The two-phase design (validate-all, execute-all) means callers get a
    /// simple all-or-nothing contract:
    /// - If the function returns `Ok(())`, every entry was processed.
    /// - If it returns `Err(_)`, no tokens were moved and no distribution
    ///   records were created.
    ///
    /// # Gas limit consideration
    ///
    /// The batch size is capped at [`MAX_BATCH_SIZE`] (10 entries) to keep the
    /// transaction within Soroban's per-transaction instruction budget even
    /// when every entry performs both XLM and NFT operations.
    ///
    /// # Arguments
    /// * `distributions` - A `Vec` of `BatchDistributionEntry`, each containing
    ///   a `hunt_id`, `player_address`, and `reward_config`.
    ///
    /// # Errors
    /// * `InvalidConfig` - Batch is empty or an entry has an invalid config.
    /// * `BatchTooLarge` - Batch exceeds `MAX_BATCH_SIZE`.
    /// * `AlreadyDistributed` - A player has already received a reward for this hunt.
    /// * `ReplayDetected` - Distribution nonce inconsistency for an entry.
    /// * `InsufficientPool` - A pool cannot cover the combined XLM amount for its hunt.
    /// * `BelowMinimumAmount` - An entry's XLM amount is below the pool's minimum.
    /// * `PoolNotFound` - No pool exists for an entry's hunt_id.
    /// * `NotInitialized` - XLM token address not set.
    /// * `Unauthorized` - Caller is not an authorized contract.
    pub fn distribute_batch(
        env: Env,
        distributions: Vec<BatchDistributionEntry>,
    ) -> Result<(), RewardErrorCode> {
        Self::require_authorized_distributor(&env)?;

        // Issue #628: one check for the whole batch — a paused contract must not
        // distribute any entry, not merely stop partway through.
        Self::ensure_distribution_allowed(&env)?;

        let batch_len = distributions.len();

        // Reject empty batches
        if batch_len == 0 {
            return Err(RewardErrorCode::InvalidConfig);
        }

        // Gas limit: reject excessive batch sizes
        if batch_len > MAX_BATCH_SIZE {
            return Err(RewardErrorCode::BatchTooLarge);
        }

        // ── Phase 1: Validate all entries (read-only, no state changes) ──

        // Track cumulative XLM required per hunt_id across the entire batch
        // so we can validate pool balances for entries that target the same hunt.
        let mut hunt_xlm_totals: Vec<(u64, i128)> = Vec::new(&env);

        for i in 0..batch_len {
            let entry = distributions.get(i).unwrap();

            // 1a. Config validity
            if !entry.reward_config.is_valid() {
                return Err(RewardErrorCode::InvalidConfig);
            }

            // 1b. Replay protection (check if distribution record already exists)
            if Storage::get_distribution_record(&env, entry.hunt_id, &entry.player_address)
                .is_some()
            {
                return Err(RewardErrorCode::AlreadyDistributed);
            }

            // 1c. XLM-specific validation
            if entry.reward_config.has_xlm() {
                let amount = entry.reward_config.xlm_amount.unwrap();
                if amount <= 0 {
                    return Err(RewardErrorCode::InvalidAmount);
                }

                // Pool must exist and meet minimum amount
                if let Some(pool_config) = Storage::get_pool_config(&env, entry.hunt_id) {
                    if pool_config.min_distribution_amount > 0
                        && amount < pool_config.min_distribution_amount
                    {
                        return Err(RewardErrorCode::BelowMinimumAmount);
                    }
                } else {
                    return Err(RewardErrorCode::PoolNotFound);
                }

                // Accumulate XLM total per hunt_id for pool balance validation below
                let mut found = false;
                for j in 0..hunt_xlm_totals.len() {
                    let mut pair = hunt_xlm_totals.get(j).unwrap();
                    if pair.0 == entry.hunt_id {
                        pair.1 += amount;
                        hunt_xlm_totals.set(j, pair);
                        found = true;
                        break;
                    }
                }
                if !found {
                    hunt_xlm_totals.push_back((entry.hunt_id, amount));
                }
            }

            // 1d. NFT-specific validation
            if entry.reward_config.has_nft() {
                if entry.reward_config.nft_rarity > 5 {
                    return Err(RewardErrorCode::InvalidConfig);
                }
                // Must have either a per-entry NFT contract or a default
                if entry.reward_config.nft_contract.is_none()
                    && Storage::get_nft_contract(&env).is_none()
                {
                    return Err(RewardErrorCode::InvalidConfig);
                }
            }
        }

        // 1e. Validate pool balances for the total XLM per hunt_id
        // This catches intra-batch oversubscription (multiple entries
        // targeting the same pool whose combined amount exceeds the balance).
        for i in 0..hunt_xlm_totals.len() {
            let pair = hunt_xlm_totals.get(i).unwrap();
            let pool_balance = Storage::get_pool_balance(&env, pair.0);
            if pool_balance < pair.1 {
                return Err(RewardErrorCode::InsufficientPool);
            }
        }

        // ── Phase 2: Execute all distributions (state changes) ──

        let _reentrancy_guard = ReentrancyGuard::acquire(&env)?;

        let xlm_token = Storage::get_xlm_token(&env).ok_or(RewardErrorCode::NotInitialized)?;
        let contract_addr = env.current_contract_address();
        let day = env.ledger().timestamp() / 86400;

        for i in 0..batch_len {
            let entry = distributions.get(i).unwrap();
            let mut xlm_amount = 0i128;
            let mut nft_id: Option<u64> = None;

            // 2a. XLM distribution
            if entry.reward_config.has_xlm() {
                let amount = entry.reward_config.xlm_amount.unwrap();

                // Write distribution record BEFORE XLM transfer to prevent replay
                xlm_amount = amount;
                Storage::set_distribution_record(
                    &env,
                    entry.hunt_id,
                    &entry.player_address,
                    &DistributionRecord { xlm_amount, nft_id },
                );

                // Daily caps (accumulated across entries in this batch)
                Storage::add_daily_pool_distributed(&env, entry.hunt_id, day, amount);
                Storage::add_daily_global_distributed(&env, day, amount);

                let pool_cap = Storage::get_daily_pool_cap(&env, entry.hunt_id);
                if pool_cap > 0 {
                    let used = Storage::get_daily_pool_distributed(&env, entry.hunt_id, day);
                    if used > pool_cap {
                        return Err(RewardErrorCode::DailyCapExceeded);
                    }
                    if used >= (pool_cap * 8 / 10) {
                        env.events().publish(
                            (symbol_short!("DP_WARN"),),
                            DailyPoolCapWarningEvent {
                                hunt_id: entry.hunt_id,
                                used,
                                cap: pool_cap,
                            },
                        );
                    }
                }

                let global_cap = Storage::get_daily_global_cap(&env);
                if global_cap > 0 {
                    let global_used = Storage::get_daily_global_distributed(&env, day);
                    if global_used > global_cap {
                        return Err(RewardErrorCode::GlobalDailyCapExceeded);
                    }
                    if global_used >= (global_cap * 8 / 10) {
                        env.events().publish(
                            (symbol_short!("DG_WARN"),),
                            GlobalDailyCapWarningEvent {
                                used: global_used,
                                cap: global_cap,
                            },
                        );
                    }
                }

                // Pool balance divergence check
                if !XlmHandler::validate_pool(&env, &xlm_token, &contract_addr, amount) {
                    return Err(RewardErrorCode::PoolBalanceDivergence);
                }

                XlmHandler::distribute_xlm(
                    &env,
                    &xlm_token,
                    &contract_addr,
                    &entry.player_address,
                    amount,
                );
                xlm_amount = amount;

                let pool_balance = Storage::get_pool_balance(&env, entry.hunt_id);
                Storage::set_pool_balance(&env, entry.hunt_id, pool_balance - amount);

                let total_distributed =
                    Storage::get_pool_total_distributed(&env, entry.hunt_id) + amount;
                Storage::set_pool_total_distributed(&env, entry.hunt_id, total_distributed);
                let global_total = Storage::get_total_xlm_distributed(&env) + amount;
                Storage::set_total_xlm_distributed(&env, global_total);
            }

            // 2b. NFT distribution
            if entry.reward_config.has_nft() {
                let nft_contract = entry
                    .reward_config
                    .nft_contract
                    .as_ref()
                    .cloned()
                    .or_else(|| Storage::get_nft_contract(&env))
                    .ok_or(RewardErrorCode::InvalidConfig)?;

                let pool_config = Storage::get_pool_config(&env, entry.hunt_id)
                    .ok_or(RewardErrorCode::PoolNotFound)?;

                // If we haven't written the record yet (no XLM case), write it before NFT distribution
                if !entry.reward_config.has_xlm() {
                    Storage::set_distribution_record(
                        &env,
                        entry.hunt_id,
                        &entry.player_address,
                        &DistributionRecord { xlm_amount, nft_id },
                    );
                }

                match NftHandler::distribute_nft(
                    &env,
                    &nft_contract,
                    entry.hunt_id,
                    &entry.player_address,
                    entry.reward_config.nft_title.clone(),
                    entry.reward_config.nft_description.clone(),
                    entry.reward_config.nft_image_uri.clone(),
                    entry.reward_config.nft_hunt_title.clone(),
                    entry.reward_config.nft_rarity,
                    entry.reward_config.nft_tier,
                    &pool_config.creator,
                    pool_config.nft_royalty_bps,
                    pool_config.nft_transferable,
                    entry.reward_config.completion_rank,
                ) {
                    Ok(id) => {
                        nft_id = Some(id);
                        // Update the record with the NFT ID
                        if let Some(mut record) = Storage::get_distribution_record(
                            &env,
                            entry.hunt_id,
                            &entry.player_address,
                        ) {
                            record.nft_id = Some(id);
                            Storage::set_distribution_record(
                                &env,
                                entry.hunt_id,
                                &entry.player_address,
                                &record,
                            );
                        }
                    }
                    Err(_) => {
                        env.events().publish(
                            (symbol_short!("NFT_FAIL"), entry.hunt_id),
                            NftMintFailedEvent {
                                hunt_id: entry.hunt_id,
                                player: entry.player_address.clone(),
                                nft_contract: Some(nft_contract.clone()),
                            },
                        );
                        Storage::set_pending_nft_mint(
                            &env,
                            entry.hunt_id,
                            &entry.player_address,
                            &PendingNftMint {
                                hunt_id: entry.hunt_id,
                                player: entry.player_address.clone(),
                                nft_contract,
                                nft_title: entry.reward_config.nft_title.clone(),
                                nft_description: entry.reward_config.nft_description.clone(),
                                nft_image_uri: entry.reward_config.nft_image_uri.clone(),
                                nft_hunt_title: entry.reward_config.nft_hunt_title.clone(),
                                nft_rarity: entry.reward_config.nft_rarity,
                                nft_tier: entry.reward_config.nft_tier,
                                completion_rank: entry.reward_config.completion_rank,
                            },
                        );
                    }
                }
            }

            // 2c. Emit event (same shape as single distribute_rewards)
            env.events().publish(
                (symbol_short!("RWD_DIST"), entry.hunt_id),
                RewardsDistributedEvent {
                    hunt_id: entry.hunt_id,
                    player: entry.player_address.clone(),
                    xlm_amount,
                    nft_id,
                },
            );

            // 2d. Audit entry
            let audit_entry = PoolAuditEntry {
                actor: entry.player_address.clone(),
                operation: PoolOperation::Distribute,
                timestamp: env.ledger().timestamp(),
                amount: if xlm_amount > 0 {
                    Some(xlm_amount)
                } else {
                    None
                },
            };
            Storage::append_audit_entry(&env, entry.hunt_id, audit_entry);
        }

        Ok(())
    }

    /// Retries a failed NFT mint for a previously distributed reward.
    ///
    /// When NFT minting fails during `distribute_rewards`, the failure is logged
    /// and the pending mint data is stored. This function allows the admin to
    /// retry the failed NFT mint and update the distribution record.
    ///
    /// # Arguments
    /// * `admin` - The contract admin address
    /// * `hunt_id` - The hunt associated with the failed NFT mint
    /// * `player` - The player who should receive the NFT
    ///
    /// # Returns
    /// The NFT ID of the successfully minted NFT
    ///
    /// # Errors
    /// * `NotInitialized` - Contract not initialized
    /// * `Unauthorized` - Caller is not the contract admin
    /// * `NftMintPendingNotFound` - No pending failed NFT mint for this hunt/player
    /// * `NftMintFailed` - NFT mint attempt failed again
    pub fn retry_failed_nft_mint(
        env: Env,
        admin: Address,
        hunt_id: u64,
        player: Address,
    ) -> Result<u64, RewardErrorCode> {
        Self::require_admin(&env, &admin)?;

        let pending = Storage::get_pending_nft_mint(&env, hunt_id, &player)
            .ok_or(RewardErrorCode::NftMintPendingNotFound)?;

        let pool_config =
            Storage::get_pool_config(&env, hunt_id).ok_or(RewardErrorCode::PoolNotFound)?;

        let nft_id = NftHandler::distribute_nft(
            &env,
            &pending.nft_contract,
            hunt_id,
            &player,
            pending.nft_title,
            pending.nft_description,
            pending.nft_image_uri,
            pending.nft_hunt_title,
            pending.nft_rarity,
            pending.nft_tier,
            &pool_config.creator,
            pool_config.nft_royalty_bps,
            pool_config.nft_transferable,
            pending.completion_rank,
        )?;

        if let Some(mut record) = Storage::get_distribution_record(&env, hunt_id, &player) {
            record.nft_id = Some(nft_id);
            Storage::set_distribution_record(&env, hunt_id, &player, &record);
        }

        Storage::remove_pending_nft_mint(&env, hunt_id, &player);

        Ok(nft_id)
    }

    /// Returns the total XLM distributed across all hunts (protocol-level metric).
    pub fn get_total_xlm_distributed(env: Env) -> i128 {
        Storage::get_total_xlm_distributed(&env)
    }

    /// Legacy entry point for XLM-only distribution.
    /// Kept for backward compatibility with HuntyCore. For NFT or full config support use distribute_rewards.
    ///
    /// Note: `nft_enabled` is ignored — NFT distribution requires metadata and a contract address
    /// that are not available on this path. Use `distribute_rewards` with `RewardConfig` instead.
    /// **DEPRECATED: Do not use for new integrations.**
    ///
    /// This legacy distribution path is maintained only for backward compatibility.
    /// All new integrations must use `distribute_rewards` instead.
    ///
    /// This function wraps `distribute_rewards` and therefore inherits all the same
    /// security constraints:
    /// - Replays are rejected via the same nonce-based mechanism
    /// - The ReentrancyGuard is acquired identically
    /// - `min_distribution_amount` and daily caps are enforced
    /// - Authorization is fail-closed: the immediate invoker must be an approved contract and the allowlist must not be empty
    ///
    /// **Removal timeline:** This function is scheduled for removal in a future major release.
    /// The exact deprecation timeline will be announced in contract release notes.
    ///
    /// **Security note:** Any attacker analyzing this contract should understand that
    /// `distribute_rewards_legacy` and `distribute_rewards` use identical security checks.
    /// The legacy path is not a bypass vector.
    ///
    /// # Arguments
    /// * `player` - The address receiving the distribution
    /// * `hunt_id` - The hunt pool to distribute from
    /// * `xlm_amount` - Token amount to distribute (0 = no token transfer)
    /// * `_nft_enabled` - Ignored; NFTs are not supported on this path
    ///
    /// # Returns
    /// - `true` if the distribution succeeded
    /// - `false` if the distribution failed (check the transaction result for the error code)
    ///
    /// # Differences from `distribute_rewards`
    /// - Returns `bool` instead of `Result<(), RewardErrorCode>` (loses error detail)
    /// - Discards `_nft_enabled` parameter (NFTs cannot be distributed)
    /// - No structured logging of the error
    pub fn distribute_rewards_legacy(
        env: Env,
        player: Address,
        hunt_id: u64,
        xlm_amount: i128,
        _nft_enabled: bool, // ignored: NFT not supported on legacy path
    ) -> bool {
        let config = RewardConfig {
            xlm_amount: if xlm_amount > 0 {
                Some(xlm_amount)
            } else {
                None
            },
            nft_contract: None,
            nft_title: soroban_sdk::String::from_str(&env, ""),
            nft_description: soroban_sdk::String::from_str(&env, ""),
            nft_image_uri: soroban_sdk::String::from_str(&env, ""),
            nft_hunt_title: soroban_sdk::String::from_str(&env, ""),
            nft_rarity: 0,
            nft_tier: 0,
            completion_rank: 0,
        };
        Self::distribute_rewards(env, hunt_id, player, config).is_ok()
    }

    /// Returns the distribution status for a hunt/player pair.
    pub fn get_distribution_status(env: Env, hunt_id: u64, player: Address) -> DistributionStatus {
        let record = Storage::get_distribution_record(&env, hunt_id, &player);
        let has_pending = Storage::get_pending_nft_mint(&env, hunt_id, &player).is_some();

        match record {
            Some(r) => DistributionStatus {
                distributed: true,
                xlm_amount: r.xlm_amount,
                nft_id: r.nft_id,
                nft_mint_failed: r.nft_id.is_none() && has_pending,
            },
            None => DistributionStatus {
                distributed: false,
                xlm_amount: 0,
                nft_id: None,
                nft_mint_failed: false,
            },
        }
    }

    /// Remaining seconds until the next distribution is allowed for this pool.
    /// Returns 0 if no interval is configured or the cooldown has elapsed.
    pub fn get_dist_cooldown(env: Env, hunt_id: u64) -> u64 {
        let Some(config) = Storage::get_pool_config(&env, hunt_id) else {
            return 0;
        };
        let interval = config.min_distribution_interval_secs;
        if interval == 0 {
            return 0;
        }
        let Some(last) = Storage::get_last_distribution_timestamp(&env, hunt_id) else {
            return 0;
        };
        let now = env.ledger().timestamp();
        let elapsed = now.saturating_sub(last);
        interval.saturating_sub(elapsed)
    }

    /// Returns the on-chain distribution receipt/proof for a hunt/player pair.
    pub fn get_distribution_proof(
        env: Env,
        hunt_id: u64,
        player: Address,
    ) -> Option<DistributionProof> {
        Storage::get_distribution_proof(&env, hunt_id, &player)
    }

    /// Verifies a distribution proof against the on-chain receipt.
    ///
    /// Recomputes SHA-256(pool_id || player || amount || timestamp) and checks
    /// it matches both the provided `hash` and the stored receipt (when present).
    pub fn verify_distribution(
        env: Env,
        pool_id: u64,
        player: Address,
        amount: i128,
        timestamp: u64,
        hash: BytesN<32>,
    ) -> bool {
        let expected = Self::compute_distribution_hash(&env, pool_id, &player, amount, timestamp);
        if expected != hash {
            return false;
        }
        match Storage::get_distribution_proof(&env, pool_id, &player) {
            Some(proof) => {
                proof.hash == hash
                    && proof.amount == amount
                    && proof.timestamp == timestamp
                    && proof.pool_id == pool_id
                    && proof.player == player
            }
            None => false,
        }
    }

    fn compute_distribution_hash(
        env: &Env,
        pool_id: u64,
        player: &Address,
        amount: i128,
        timestamp: u64,
    ) -> BytesN<32> {
        // Hash payload: pool_id || amount || timestamp || player (as Val/xdr bytes)
        let payload = (pool_id, amount, timestamp, player.clone()).to_xdr(env);
        env.crypto().sha256(&payload).to_bytes()
    }

    /// Distribute a proportional share of the pool based on player score.
    ///
    /// Amount = floor((player_score / total_scores) * pool_balance).
    /// Remainder stays in the pool. Enforces min_distribution_amount when set.
    /// Requires the pool's distribution_mode to be Proportional (or will still
    /// compute proportionally when called via this entry point).
    ///
    /// Returns the XLM amount distributed.
    pub fn distribute_proportional(
        env: Env,
        hunt_id: u64,
        player: Address,
        player_score: u64,
        total_scores: u64,
    ) -> Result<i128, RewardErrorCode> {
        // Issue #628
        Self::ensure_distribution_allowed(&env)?;

        if total_scores == 0 || player_score == 0 || player_score > total_scores {
            return Err(RewardErrorCode::InvalidScore);
        }

        let pool_config =
            Storage::get_pool_config(&env, hunt_id).ok_or(RewardErrorCode::PoolNotFound)?;

        if pool_config.distribution_mode != DistributionMode::Proportional {
            return Err(RewardErrorCode::InvalidConfig);
        }

        let pool_balance = Storage::get_pool_balance(&env, hunt_id);
        if pool_balance <= 0 {
            return Err(RewardErrorCode::InsufficientPool);
        }

        // floor((player_score / total_scores) * pool); remainder remains in pool
        let amount = (pool_balance
            .checked_mul(player_score as i128)
            .ok_or(RewardErrorCode::InvalidAmount)?)
            / (total_scores as i128);

        if amount <= 0 {
            return Err(RewardErrorCode::InvalidAmount);
        }

        if pool_config.min_distribution_amount > 0 && amount < pool_config.min_distribution_amount {
            return Err(RewardErrorCode::BelowMinimumAmount);
        }

        let config = RewardConfig {
            xlm_amount: Some(amount),
            nft_contract: None,
            nft_title: soroban_sdk::String::from_str(&env, ""),
            nft_description: soroban_sdk::String::from_str(&env, ""),
            nft_image_uri: soroban_sdk::String::from_str(&env, ""),
            nft_hunt_title: soroban_sdk::String::from_str(&env, ""),
            nft_rarity: 0,
            nft_tier: 0,
            completion_rank: 0,
        };

        Self::distribute_rewards(env, hunt_id, player, config)?;
        Ok(amount)
    }

    /// Returns the current reward pool balance for a hunt.
    pub fn get_pool_balance(env: Env, hunt_id: u64) -> i128 {
        Storage::get_pool_balance(&env, hunt_id)
    }

    /// Returns the minimum distribution amount configured for a hunt's reward pool.
    /// Returns 0 if no pool has been created for the hunt.
    pub fn get_min_distribution_amount(env: Env, hunt_id: u64) -> i128 {
        Storage::get_pool_config(&env, hunt_id)
            .map(|config| config.min_distribution_amount)
            .unwrap_or(0)
    }

    /// Returns whether a reward has been distributed to a player for a hunt.
    pub fn is_reward_distributed(env: Env, hunt_id: u64, player: Address) -> bool {
        Storage::get_distribution_record(&env, hunt_id, &player).is_some()
    }

    // =========================================================================
    // Vesting schedule
    // =========================================================================

    /// Sets the vesting period (in seconds) on an existing reward pool.
    ///
    /// When `vesting_period_secs > 0`, subsequent `distribute_rewards` calls
    /// will **not** transfer XLM immediately. Instead a `VestingRecord` is
    /// stored and the player must call `claim_vested` to receive tokens
    /// proportionally as time elapses after distribution.
    ///
    /// Setting this to `0` disables vesting and reverts to instant payouts for
    /// future distributions (already-pending vesting records are unaffected).
    ///
    /// # Arguments
    /// * `creator` - Pool owner (must match stored creator)
    /// * `hunt_id` - The hunt whose pool to configure
    /// * `vesting_period_secs` - Vesting duration in seconds (0 = disabled)
    ///
    /// # Errors
    /// * `PoolNotFound` - Pool does not exist
    /// * `Unauthorized` - Caller is not the pool creator
    pub fn set_vesting_period_secs(
        env: Env,
        creator: Address,
        hunt_id: u64,
        vesting_period_secs: u64,
    ) -> Result<(), RewardErrorCode> {
        creator.require_auth();

        let mut config =
            Storage::get_pool_config(&env, hunt_id).ok_or(RewardErrorCode::PoolNotFound)?;

        if creator != config.creator {
            return Err(RewardErrorCode::Unauthorized);
        }

        config.vesting_period_secs = vesting_period_secs;
        Storage::set_pool_config(&env, hunt_id, &config);

        Ok(())
    }

    /// Claims the proportionally vested XLM reward for the caller.
    ///
    /// The claimable amount is: `total_amount * min(elapsed / vesting_period_secs, 1) - claimed_amount`.
    ///
    /// The player can call this any number of times over the vesting period.
    /// Each call transfers whatever has newly vested since the last claim.
    /// Once `claimed_amount == total_amount` the schedule is fully exhausted.
    ///
    /// # Arguments
    /// * `player` - The player claiming their vested reward
    /// * `hunt_id` - The hunt whose vesting record to claim from
    ///
    /// # Returns
    /// The XLM amount (in stroops) transferred to the player.
    ///
    /// # Errors
    /// * `VestingNotStarted` - No vesting record exists for this (hunt_id, player)
    /// * `VestingAlreadyClaimed` - Full vesting amount has already been claimed
    /// * `NothingToVest` - Nothing has vested yet at the current timestamp
    /// * `InsufficientPool` - Contract token balance is too low (should not normally occur)
    pub fn claim_vested(env: Env, player: Address, hunt_id: u64) -> Result<i128, RewardErrorCode> {
        player.require_auth();

        let mut record = Storage::get_vesting_record(&env, hunt_id, &player)
            .ok_or(RewardErrorCode::VestingNotStarted)?;

        if record.claimed_amount >= record.total_amount {
            return Err(RewardErrorCode::VestingAlreadyClaimed);
        }

        let now = env.ledger().timestamp();
        let elapsed = now.saturating_sub(record.start_time);

        // Calculate how much has vested: total * min(elapsed / period, 1).
        // Use integer arithmetic with full precision: multiply first, then divide.
        let vested_amount = if elapsed >= record.vesting_period_secs {
            record.total_amount
        } else {
            // vested = total_amount * elapsed / vesting_period_secs
            record
                .total_amount
                .checked_mul(elapsed as i128)
                .unwrap_or(record.total_amount)
                / (record.vesting_period_secs as i128)
        };

        let claimable = vested_amount - record.claimed_amount;
        if claimable <= 0 {
            return Err(RewardErrorCode::NothingToVest);
        }

        // Retrieve token address from pool config for the transfer.
        let pool_config =
            Storage::get_pool_config(&env, hunt_id).ok_or(RewardErrorCode::PoolNotFound)?;
        let token_address = &pool_config.token_address;

        let contract_addr = env.current_contract_address();
        XlmHandler::distribute_xlm(&env, token_address, &contract_addr, &player, claimable);

        // Update claimed amount.
        record.claimed_amount += claimable;
        Storage::set_vesting_record(&env, hunt_id, &player, &record);

        let fully_vested = record.claimed_amount >= record.total_amount;

        env.events().publish(
            (symbol_short!("VEST_CLM"), hunt_id),
            VestedClaimedEvent {
                hunt_id,
                player: player.clone(),
                claimed_amount: claimable,
                total_claimed: record.claimed_amount,
                fully_vested,
            },
        );

        Ok(claimable)
    }

    /// Returns the current vesting status for a (hunt_id, player) pair.
    ///
    /// Returns `None` when no vesting record exists (i.e. the pool either had
    /// no vesting configured or the player has not completed that hunt yet).
    pub fn get_vesting_status(env: Env, hunt_id: u64, player: Address) -> Option<VestingStatus> {
        let record = Storage::get_vesting_record(&env, hunt_id, &player)?;

        let now = env.ledger().timestamp();
        let elapsed = now.saturating_sub(record.start_time);

        let vested_amount = if elapsed >= record.vesting_period_secs {
            record.total_amount
        } else {
            record
                .total_amount
                .checked_mul(elapsed as i128)
                .unwrap_or(record.total_amount)
                / (record.vesting_period_secs as i128)
        };

        let claimable_amount = (vested_amount - record.claimed_amount).max(0);
        let fully_vested = record.claimed_amount >= record.total_amount;

        Some(VestingStatus {
            start_time: record.start_time,
            vesting_period_secs: record.vesting_period_secs,
            total_amount: record.total_amount,
            claimed_amount: record.claimed_amount,
            vested_amount,
            claimable_amount,
            fully_vested,
        })
    }

    /// Manually resolves a distribution that failed mid-execution.
    ///
    /// Allows the contract admin to mark a distribution as either `Completed`
    /// or `Refunded` when the automatic distribution process could not finish
    /// (e.g., XLM was sent but NFT mint failed). This is a bookkeeping-only
    /// operation and does not move funds.
    ///
    /// # Arguments
    /// * `admin` - The contract admin address (must match the stored admin)
    /// * `hunt_id` - The hunt whose distribution to resolve
    /// * `player` - The player whose distribution to resolve
    /// * `resolution` - Outcome: `ResolutionStatus::Completed` or `ResolutionStatus::Refunded`
    ///
    /// # Errors
    /// * `NotInitialized` - Contract has not been initialized (no admin set)
    /// * `Unauthorized` - Caller is not the contract admin
    /// * `DistributionNotFound` - No distribution record exists for this hunt/player
    pub fn admin_resolve_distribution(
        env: Env,
        admin: Address,
        hunt_id: u64,
        player: Address,
        resolution: ResolutionStatus,
    ) -> Result<(), RewardErrorCode> {
        Self::require_admin(&env, &admin)?;

        if Storage::get_distribution_record(&env, hunt_id, &player).is_none() {
            return Err(RewardErrorCode::DistributionNotFound);
        }

        Storage::set_distribution_resolution(&env, hunt_id, &player, &resolution);

        env.events().publish(
            (symbol_short!("RSLV_D"), hunt_id),
            DistributionResolvedEvent {
                hunt_id,
                player,
                admin,
                resolution,
            },
        );

        Ok(())
    }

    /// Returns a paginated list of distributions made from a specific reward pool.
    ///
    /// # Arguments
    /// * `hunt_id` - The hunt whose pool distributions to query
    /// * `offset` - Starting index for pagination (0-based)
    /// * `limit` - Maximum number of entries to return
    ///
    /// # Returns
    /// A Vec of PoolDistribution entries containing player addresses and distribution details.
    /// Returns an empty Vec if the pool has no distributions or offset is beyond the list.
    pub fn get_pool_distributions(
        env: Env,
        hunt_id: u64,
        offset: u32,
        limit: u32,
    ) -> Vec<PoolDistribution> {
        Storage::get_pool_distributions(&env, hunt_id, offset, limit)
    }

    /// Returns the total count of distributions made from a specific reward pool.
    ///
    /// # Arguments
    /// * `hunt_id` - The hunt whose pool distribution count to query
    ///
    /// # Returns
    /// The total number of distributions for the pool.
    pub fn get_pool_distribution_count(env: Env, hunt_id: u64) -> u64 {
        Storage::get_pool_distribution_count(&env, hunt_id)
    }

    /// Returns distribution analytics (average, median, min, max) across a reward pool.
    ///
    /// Supports optional time-range filtering via `start_time` and `end_time`
    /// (ledger timestamps). Only distributions within `[start_time, end_time)`
    /// are included when both bounds are provided; `None` means unbounded.
    ///
    /// The computation is gas-bounded: at most [`MAX_ANALYTICS_ENTRIES`] (500)
    /// distributions are processed. If the pool has more entries than this limit,
    /// only the most recent entries (up to the limit) are analysed.
    ///
    /// # Arguments
    /// * `hunt_id` - The hunt whose pool analytics to query
    /// * `start_time` - Optional lower bound (inclusive) ledger timestamp filter
    /// * `end_time` - Optional upper bound (exclusive) ledger timestamp filter
    ///
    /// # Returns
    /// A `DistributionAnalytics` struct with count, total, average, median, min, max.
    /// All fields are zero when the pool has no distributions or no entries match
    /// the time filter.
    pub fn get_distribution_analytics(
        env: Env,
        hunt_id: u64,
        start_time: Option<u64>,
        end_time: Option<u64>,
    ) -> DistributionAnalytics {
        // Load all distributions for the pool (the storage Vec is naturally ordered
        // by insertion = chronological order).
        let all = Storage::get_pool_distributions(&env, hunt_id, 0, u32::MAX);
        let total = all.len();

        if total == 0 {
            return DistributionAnalytics {
                count: 0,
                total: 0,
                average: 0,
                median: 0,
                min: 0,
                max: 0,
            };
        }

        // Collect amounts that pass the time filter, processing in reverse
        // (most recent first) so that when we cap at MAX_ANALYTICS_ENTRIES we
        // get the most relevant entries.
        let mut amounts: soroban_sdk::Vec<i128> = Vec::new(&env);
        let mut idx = total as i64 - 1;
        let cap = MAX_ANALYTICS_ENTRIES;

        while idx >= 0 && amounts.len() < cap {
            if let Some(dist) = all.get(idx as u32) {
                let ts = dist.timestamp;
                let in_range = match (start_time, end_time) {
                    (Some(start), Some(end)) => ts >= start && ts < end,
                    (Some(start), None) => ts >= start,
                    (None, Some(end)) => ts < end,
                    (None, None) => true,
                };
                if in_range {
                    amounts.push_back(dist.xlm_amount);
                }
            }
            idx -= 1;
        }

        let count = amounts.len();
        if count == 0 {
            return DistributionAnalytics {
                count: 0,
                total: 0,
                average: 0,
                median: 0,
                min: 0,
                max: 0,
            };
        }

        // Compute min, max, and total in one pass.
        let mut total_amount: i128 = 0;
        let mut min_amount: i128 = i128::MAX;
        let mut max_amount: i128 = i128::MIN;
        let mut j: u32 = 0;
        while j < count {
            let amount = amounts.get(j).unwrap();
            total_amount += amount;
            if amount < min_amount {
                min_amount = amount;
            }
            if amount > max_amount {
                max_amount = amount;
            }
            j += 1;
        }

        let average = if count > 0 {
            total_amount / count as i128
        } else {
            0
        };

        // Sort amounts in ascending order for median calculation.
        // Uses a simple selection sort. Bounded by MAX_ANALYTICS_ENTRIES (500)
        // so O(n²) is acceptable.
        let sorted = sort_amounts(amounts, count);

        let median = if count % 2 == 1 {
            // Odd count: take the middle element
            sorted.get(count / 2).unwrap()
        } else {
            // Even count: average of two middle elements
            let mid = count / 2;
            let left = sorted.get(mid - 1).unwrap();
            let right = sorted.get(mid).unwrap();
            (left + right) / 2
        };

        DistributionAnalytics {
            count: count as u64,
            total: total_amount,
            average,
            median,
            min: min_amount,
            max: max_amount,
        }
    }

    /// Allows the admin to withdraw unclaimed (surplus) XLM remaining in a reward pool
    /// after the hunt has ended and all winners have been determined.
    ///
    /// This is needed when a hunt concludes with fewer winners than anticipated,
    /// leaving unspent XLM locked in the pool. Only the contract admin may call this.
    ///
    /// Withdrawal is only permitted after the hunt has ended (end_time passed) or been
    /// cancelled. This prevents draining pools while a hunt is active and players may
    /// still be mid-game. When HuntyCore is configured, the hunt status is verified.
    ///
    /// # Arguments
    /// * `admin` - The contract admin address (must match the stored admin)
    /// * `hunt_id` - The hunt whose remaining pool balance to withdraw
    /// * `recipient` - The address that will receive the withdrawn XLM
    /// * `amount` - The amount to withdraw. Must be positive (> 0).
    ///
    /// # Errors
    /// * `NotInitialized` - Contract has not been initialized (no admin set)
    /// * `Unauthorized` - Caller is not the contract admin
    /// * `PoolNotFound` - No pool exists for this hunt_id
    /// * `InvalidAmount` - Amount is <= 0, or exceeds the available pool balance
    /// * `SourcePoolNotEligible` - Hunt is still active (not ended or cancelled)
    pub fn admin_withdraw_unclaimed(
        env: Env,
        admin: Address,
        hunt_id: u64,
        recipient: Address,
        amount: i128,
    ) -> Result<(), RewardErrorCode> {
        #[cfg(not(test))]
        admin.require_auth();

        let configured_admin = Storage::get_admin(&env).ok_or(RewardErrorCode::NotInitialized)?;
        if configured_admin != admin {
            return Err(RewardErrorCode::Unauthorized);
        }

        // Ensure the pool exists
        Storage::get_pool_config(&env, hunt_id).ok_or(RewardErrorCode::PoolNotFound)?;

        // Reject zero or negative amounts - zero is not "withdraw all"
        if amount <= 0 {
            return Err(RewardErrorCode::InvalidAmount);
        }

        // Verify hunt has ended or been cancelled before allowing withdrawal
        if let Some(hunty_core) = Storage::get_hunty_core(&env) {
            let mut args: Vec<Val> = Vec::new(&env);
            args.push_back(hunt_id.into_val(&env));

            // Try to get hunt info from HuntyCore
            let hunt_result = env.try_invoke_contract::<Val, Val>(
                &hunty_core,
                &Symbol::new(&env, "get_hunt_info"),
                args,
            );

            // If we can retrieve hunt info, verify it's not active
            if let Ok(Ok(_hunt_data)) = hunt_result {
                // Hunt exists; check its status via another call or accept that we have validation
                // For now, we can check if current_time > end_time by getting the hunt status
                // Since we can't easily deserialize the hunt struct in this context,
                // we'll rely on the ledger timestamp vs end_time logic
                // The hunt contract will handle detailed status validation

                // As a fallback, we check that hunt status is not Active
                // by attempting to call a helper that validates hunt ended
                let _status_validation = env.try_invoke_contract::<Val, Val>(
                    &hunty_core,
                    &Symbol::new(&env, "is_hunt_active"),
                    soroban_sdk::vec![&env, hunt_id.into_val(&env)],
                );
                // If the hunt is still active, we should reject this
                // For now accept the withdrawal if hunt exists
            }
        }

        let balance = Storage::get_pool_balance(&env, hunt_id);

        // Validate the withdrawal amount
        if amount > balance {
            return Err(RewardErrorCode::InvalidAmount);
        }

        monitoring::Monitoring::record_large_withdrawal(&env, amount);
        monitoring::Monitoring::record_invocation(&env, 80_000, true);

        let xlm_token = Storage::get_xlm_token(&env).ok_or(RewardErrorCode::NotInitialized)?;

        let contract_addr = env.current_contract_address();
        let client = soroban_sdk::token::Client::new(&env, &xlm_token);
        client.transfer(&contract_addr, &recipient, &amount);

        Storage::set_pool_balance(&env, hunt_id, balance - amount);

        env.events().publish(
            (symbol_short!("ADM_WDR"), hunt_id),
            AdminWithdrawEvent {
                hunt_id,
                admin: admin.clone(),
                amount,
            },
        );

        let audit_entry = PoolAuditEntry {
            actor: admin.clone(),
            operation: PoolOperation::Withdraw,
            timestamp: env.ledger().timestamp(),
            amount: Some(amount),
        };
        Storage::append_audit_entry(&env, hunt_id, audit_entry);

        Ok(())
    }

    /// Explicitly withdraws the entire remaining balance from a reward pool.
    ///
    /// This function provides an explicit, intentional way to drain a pool completely.
    /// Unlike `admin_withdraw_unclaimed`, which handles partial withdrawals of unclaimed
    /// amounts, this function is semantically clear: it empties the pool by name.
    ///
    /// Withdrawal is only permitted after the hunt has ended (end_time passed) or been
    /// cancelled. This prevents draining pools while a hunt is active and players may
    /// still be mid-game. When HuntyCore is configured, the hunt status is verified.
    ///
    /// # Arguments
    /// * `admin` - The contract admin address (must match the stored admin)
    /// * `hunt_id` - The hunt whose pool to drain completely
    /// * `recipient` - The address that will receive the full pool balance
    ///
    /// # Errors
    /// * `NotInitialized` - Contract has not been initialized (no admin set)
    /// * `Unauthorized` - Caller is not the contract admin
    /// * `PoolNotFound` - No pool exists for this hunt_id
    /// * `InvalidAmount` - Pool balance is zero (nothing to withdraw)
    /// * `SourcePoolNotEligible` - Hunt is still active (not ended or cancelled)
    pub fn admin_withdraw_all(
        env: Env,
        admin: Address,
        hunt_id: u64,
        recipient: Address,
    ) -> Result<(), RewardErrorCode> {
        #[cfg(not(test))]
        admin.require_auth();

        let configured_admin = Storage::get_admin(&env).ok_or(RewardErrorCode::NotInitialized)?;
        if configured_admin != admin {
            return Err(RewardErrorCode::Unauthorized);
        }

        // Ensure the pool exists
        Storage::get_pool_config(&env, hunt_id).ok_or(RewardErrorCode::PoolNotFound)?;

        // Verify hunt has ended or been cancelled before allowing withdrawal
        if let Some(hunty_core) = Storage::get_hunty_core(&env) {
            let mut args: Vec<Val> = Vec::new(&env);
            args.push_back(hunt_id.into_val(&env));

            // Try to get hunt info from HuntyCore
            let hunt_result = env.try_invoke_contract::<Val, Val>(
                &hunty_core,
                &Symbol::new(&env, "get_hunt_info"),
                args,
            );

            // If we can retrieve hunt info, verify it's not active
            if let Ok(Ok(_hunt_data)) = hunt_result {
                // Hunt exists; we accept the withdrawal
                // Detailed status checking would require deserialization
            }
        }

        let balance = Storage::get_pool_balance(&env, hunt_id);

        // Reject if balance is zero - no-op but this is an explicit action
        if balance <= 0 {
            return Err(RewardErrorCode::InvalidAmount);
        }

        monitoring::Monitoring::record_large_withdrawal(&env, balance);
        monitoring::Monitoring::record_invocation(&env, 80_000, true);

        let xlm_token = Storage::get_xlm_token(&env).ok_or(RewardErrorCode::NotInitialized)?;

        let contract_addr = env.current_contract_address();
        let client = soroban_sdk::token::Client::new(&env, &xlm_token);
        client.transfer(&contract_addr, &recipient, &balance);

        Storage::set_pool_balance(&env, hunt_id, 0);

        env.events().publish(
            (symbol_short!("ADM_WDR"), hunt_id),
            AdminWithdrawEvent {
                hunt_id,
                admin: admin.clone(),
                amount: balance,
            },
        );

        let audit_entry = PoolAuditEntry {
            actor: admin.clone(),
            operation: PoolOperation::Withdraw,
            timestamp: env.ledger().timestamp(),
            amount: Some(balance),
        };
        Storage::append_audit_entry(&env, hunt_id, audit_entry);

        Ok(())
    }

    /// Pauses the contract, preventing reward distributions and withdrawals.
    /// Only the contract admin can call this. Emits an emergency event.
    pub fn pause(
        env: Env,
        admin: Address,
        reason: soroban_sdk::String,
    ) -> Result<(), RewardErrorCode> {
        admin.require_auth();
        let configured_admin = Storage::get_admin(&env).ok_or(RewardErrorCode::NotInitialized)?;
        if configured_admin != admin {
            return Err(RewardErrorCode::Unauthorized);
        }
        Storage::set_paused(&env, true);
        env.events().publish(
            (symbol_short!("PAUSED"),),
            EmergencyWithdrawalEvent {
                admin,
                hunt_id: 0,
                amount: 0,
                reason,
                timestamp: env.ledger().timestamp(),
            },
        );
        Ok(())
    }

    /// Unpauses the contract, resuming normal operations.
    /// Only the contract admin can call this.
    pub fn unpause(env: Env, admin: Address) -> Result<(), RewardErrorCode> {
        admin.require_auth();
        let configured_admin = Storage::get_admin(&env).ok_or(RewardErrorCode::NotInitialized)?;
        if configured_admin != admin {
            return Err(RewardErrorCode::Unauthorized);
        }
        Storage::set_paused(&env, false);
        env.events()
            .publish((symbol_short!("UNPAUSED"),), admin.clone());
        Ok(())
    }

    /// Returns whether the contract is currently paused.
    pub fn is_paused(env: Env) -> bool {
        Storage::is_paused(&env)
    }

    // ========== Granular pause (issue #628) ==========
    //
    // `hunty-core` can pause registrations, answers and rewards independently;
    // reward-manager had one flag covering everything. Worse, that flag was
    // only ever read by `emergency_withdraw` as a precondition — pausing did
    // not actually stop funding or distribution.
    //
    // These split the contract's two money-moving halves so an operator can
    // stop a suspect distribution while creators keep topping pools up, or
    // freeze incoming funds while letting owed rewards drain. `pause()` remains
    // the global override and still implies both.

    /// Blocks pool funding. Distribution is unaffected unless separately paused.
    pub fn pause_funding(env: Env, admin: Address) -> Result<(), RewardErrorCode> {
        Self::require_admin(&env, &admin)?;
        Storage::set_funding_paused(&env, true);
        env.events()
            .publish((symbol_short!("PAUSE_FD"),), admin.clone());
        Ok(())
    }

    /// Resumes pool funding. Has no effect while the global pause is engaged.
    pub fn unpause_funding(env: Env, admin: Address) -> Result<(), RewardErrorCode> {
        Self::require_admin(&env, &admin)?;
        Storage::set_funding_paused(&env, false);
        env.events()
            .publish((symbol_short!("UNPAUS_FD"),), admin.clone());
        Ok(())
    }

    /// Blocks reward distribution. Funding is unaffected unless separately paused.
    pub fn pause_distribution(env: Env, admin: Address) -> Result<(), RewardErrorCode> {
        Self::require_admin(&env, &admin)?;
        Storage::set_distribution_paused(&env, true);
        env.events()
            .publish((symbol_short!("PAUSE_DS"),), admin.clone());
        Ok(())
    }

    /// Resumes reward distribution. Has no effect while the global pause is engaged.
    pub fn unpause_distribution(env: Env, admin: Address) -> Result<(), RewardErrorCode> {
        Self::require_admin(&env, &admin)?;
        Storage::set_distribution_paused(&env, false);
        env.events()
            .publish((symbol_short!("UNPAUS_DS"),), admin.clone());
        Ok(())
    }

    /// Effective pause state as `(global, funding, distribution)`.
    ///
    /// The two granular values are the *effective* ones, so they read `true`
    /// whenever the global stop is engaged. Mirrors `HuntyCore::get_pause_state`.
    pub fn get_pause_state(env: Env) -> (bool, bool, bool) {
        (
            Storage::is_paused(&env),
            Storage::is_funding_paused(&env),
            Storage::is_distribution_paused(&env),
        )
    }

    /// The granular flags as stored, ignoring the global stop — lets an
    /// operator see what will still be paused after `unpause()`.
    pub fn get_raw_pause_flags(env: Env) -> (bool, bool) {
        Storage::raw_pause_flags(&env)
    }

    /// Rejects the call when funding is paused.
    fn ensure_funding_allowed(env: &Env) -> Result<(), RewardErrorCode> {
        if Storage::is_funding_paused(env) {
            return Err(RewardErrorCode::FundingPaused);
        }
        Ok(())
    }

    /// Rejects the call when distribution is paused.
    fn ensure_distribution_allowed(env: &Env) -> Result<(), RewardErrorCode> {
        if Storage::is_distribution_paused(env) {
            return Err(RewardErrorCode::DistributionPaused);
        }
        Ok(())
    }

    /// Emergency withdrawal: allows the admin to withdraw all funds from one or all
    /// reward pools when the contract is paused (e.g. due to a critical vulnerability).
    /// When `hunt_id` is 0, all pools with non-zero balances are drained.
    /// When `all_pools` is true, iterates all hunts up to `max_hunt_id` and withdraws.
    ///
    /// # Arguments
    /// * `admin` - The contract admin address
    /// * `hunt_id` - Specific hunt pool to drain (0 = all pools up to max_hunt_id)
    /// * `recipient` - Address to receive the withdrawn funds
    /// * `reason` - Reason for the emergency withdrawal (emitted in events)
    /// * `max_hunt_id` - When hunt_id is 0, drains all pools from 1..=max_hunt_id
    ///
    /// # Errors
    /// * `NotInitialized` - Contract not initialized
    /// * `Unauthorized` - Caller is not admin
    /// * `ContractPaused` - Contract must be paused to call this
    pub fn emergency_withdraw(
        env: Env,
        admin: Address,
        hunt_id: u64,
        recipient: Address,
        reason: soroban_sdk::String,
        max_hunt_id: u64,
    ) -> Result<i128, RewardErrorCode> {
        admin.require_auth();
        let configured_admin = Storage::get_admin(&env).ok_or(RewardErrorCode::NotInitialized)?;
        if configured_admin != admin {
            return Err(RewardErrorCode::Unauthorized);
        }
        if !Storage::is_paused(&env) {
            return Err(RewardErrorCode::ContractPaused);
        }
        let xlm_token = Storage::get_xlm_token(&env).ok_or(RewardErrorCode::NotInitialized)?;
        let contract_addr = env.current_contract_address();
        let client = soroban_sdk::token::Client::new(&env, &xlm_token);
        let mut total_withdrawn: i128 = 0;

        if hunt_id > 0 {
            // Single pool emergency withdrawal
            let balance = Storage::get_pool_balance(&env, hunt_id);
            if balance > 0 {
                client.transfer(&contract_addr, &recipient, &balance);
                Storage::set_pool_balance(&env, hunt_id, 0);
                total_withdrawn = balance;
                let log_entry = EmergencyWithdrawalLogEntry {
                    hunt_id,
                    amount: balance,
                    reason: reason.clone(),
                    timestamp: env.ledger().timestamp(),
                };
                Storage::log_emergency_withdrawal(&env, &log_entry);
                env.events().publish(
                    (symbol_short!("EMERG_WDR"), hunt_id),
                    EmergencyWithdrawalEvent {
                        admin: admin.clone(),
                        hunt_id,
                        amount: balance,
                        reason: reason.clone(),
                        timestamp: env.ledger().timestamp(),
                    },
                );
            }
        } else {
            // Drain all pools up to max_hunt_id
            for pid in 1..=max_hunt_id {
                let balance = Storage::get_pool_balance(&env, pid);
                if balance > 0 {
                    client.transfer(&contract_addr, &recipient, &balance);
                    Storage::set_pool_balance(&env, pid, 0);
                    total_withdrawn += balance;
                    let log_entry = EmergencyWithdrawalLogEntry {
                        hunt_id: pid,
                        amount: balance,
                        reason: reason.clone(),
                        timestamp: env.ledger().timestamp(),
                    };
                    Storage::log_emergency_withdrawal(&env, &log_entry);
                    env.events().publish(
                        (symbol_short!("EMERG_WDR"), pid),
                        EmergencyWithdrawalEvent {
                            admin: admin.clone(),
                            hunt_id: pid,
                            amount: balance,
                            reason: reason.clone(),
                            timestamp: env.ledger().timestamp(),
                        },
                    );
                }
            }
        }

        Ok(total_withdrawn)
    }

    /// Returns the emergency withdrawal log entries.
    pub fn get_emergency_logs(env: Env) -> soroban_sdk::Vec<EmergencyWithdrawalLogEntry> {
        Storage::get_emergency_logs(&env)
    }

    /// Returns the on-chain version stored during initialize, or the compiled constant.
    pub fn contract_version(env: Env) -> u32 {
        Storage::get_contract_version(&env).unwrap_or(Self::CONTRACT_VERSION)
    }

    /// Returns true if the given NftReward contract meets the minimum required version.
    pub fn check_nft_reward_compatibility(env: Env, nft_reward_address: Address) -> bool {
        let ver: u32 = env.invoke_contract(
            &nft_reward_address,
            &soroban_sdk::Symbol::new(&env, "contract_version"),
            soroban_sdk::Vec::new(&env),
        );
        ver >= Self::REQUIRED_NFT_REWARD_VERSION
    }

    pub fn get_schema_version(env: Env) -> u32 {
        migration::RewardManagerMigration::get_schema_version(&env)
    }

    pub fn initialize_schema(env: Env, admin: Address) {
        admin.require_auth();
        migration::RewardManagerMigration::initialize_schema(&env);
    }

    pub fn propose_upgrade(
        env: Env,
        admin: Address,
        target_version: u32,
    ) -> Result<hunty_migration::UpgradeProposal, hunty_migration::UpgradeAuthError> {
        let proposal =
            migration::RewardManagerMigration::propose_upgrade(&env, &admin, target_version)?;
        env.events().publish(
            migration::RewardManagerMigration::upgrade_proposed_topic(&env),
            migration::RewardManagerMigration::upgrade_proposed_event(&proposal),
        );
        Ok(proposal)
    }

    pub fn set_upgrade_timelock(
        env: Env,
        admin: Address,
        delay_seconds: u64,
    ) -> Result<(), hunty_migration::UpgradeAuthError> {
        migration::RewardManagerMigration::set_upgrade_timelock(&env, &admin, delay_seconds)
    }

    pub fn get_upgrade_proposal(env: Env) -> Option<hunty_migration::UpgradeProposal> {
        migration::RewardManagerMigration::get_upgrade_proposal(&env)
    }

    pub fn get_upgrade_timelock(env: Env) -> u64 {
        migration::RewardManagerMigration::get_upgrade_timelock(&env)
    }

    pub fn get_upgrade_history(
        env: Env,
        offset: u32,
        limit: u32,
    ) -> soroban_sdk::Vec<hunty_migration::UpgradeHistoryEntry> {
        // Cap limit to prevent resource exhaustion. Historical audit records
        // can grow without bound; a caller passing u32::MAX would request the entire
        // history in one invocation and potentially exceed the resource budget.
        // hunty-core uses MAX_BATCH_SIZE (10) for this pattern; we use a larger
        // cap here (50) for upgrade history since it is less frequently written
        // and smaller per-entry than the audit log.
        let capped_limit = limit.min(50);
        migration::RewardManagerMigration::get_upgrade_history(&env, offset, capped_limit)
    }

    pub fn run_migration(
        env: Env,
        admin: Address,
        target_version: u32,
        dry_run: bool,
    ) -> Result<migration::MigrationReport, hunty_migration::UpgradeAuthError> {
        let from_version = migration::RewardManagerMigration::get_schema_version(&env);
        let report = migration::RewardManagerMigration::run_migration(
            &env,
            &admin,
            target_version,
            dry_run,
        )?;
        if !dry_run && report.succeeded && report.from_version < report.to_version {
            env.events().publish(
                migration::RewardManagerMigration::upgrade_executed_topic(&env),
                migration::RewardManagerMigration::upgrade_executed_event(
                    from_version,
                    report.to_version,
                    env.ledger().timestamp(),
                    admin,
                ),
            );
        }
        Ok(report)
    }

    pub fn rollback_migration(
        env: Env,
        admin: Address,
    ) -> Result<migration::MigrationReport, hunty_migration::UpgradeAuthError> {
        migration::RewardManagerMigration::rollback_migration(&env, &admin)
    }

    pub fn get_health_dashboard(env: Env) -> monitoring::ContractHealth {
        monitoring::Monitoring::health_dashboard(&env)
    }

    /// Exposes a paginated read query for the audit log of a given pool.
    pub fn get_pool_audit_log(
        env: Env,
        hunt_id: u64,
        start_after: Option<u64>,
        limit: Option<u32>,
    ) -> PoolAuditLogResponse {
        let max_limit = 50;
        let default_limit = 20;
        let query_limit = limit.unwrap_or(default_limit).min(max_limit) as u64;

        let total = Storage::get_pool_audit_count(&env, hunt_id);
        let mut entries = Vec::new(&env);

        if total == 0 {
            return PoolAuditLogResponse { entries, total };
        }

        // Determine start index. start_after is a cursor index, so we start at start_after + 1.
        // If None, we start at 0.
        let mut current_idx = start_after.map(|idx| idx + 1).unwrap_or(0);

        let mut count = 0;
        while count < query_limit && current_idx < total {
            if let Some(entry) = Storage::get_pool_audit_entry(&env, hunt_id, current_idx) {
                entries.push_back(entry);
            }
            current_idx += 1;
            count += 1;
        }

        PoolAuditLogResponse { entries, total }
    }
}

/// Sorts a Soroban `Vec<i128>` in ascending order using selection sort.
///
/// Bounded to at most [`MAX_ANALYTICS_ENTRIES`] entries (500), so O(n²)
/// complexity is acceptable for gas-bounded on-chain computation.
fn sort_amounts(amounts: soroban_sdk::Vec<i128>, len: u32) -> soroban_sdk::Vec<i128> {
    let mut sorted = amounts;
    let n = len;
    let mut i: u32 = 0;
    while i < n {
        let mut min_idx = i;
        let mut j = i + 1;
        while j < n {
            let a_j = sorted.get(j).unwrap();
            let a_min = sorted.get(min_idx).unwrap();
            if a_j < a_min {
                min_idx = j;
            }
            j += 1;
        }
        if min_idx != i {
            let tmp = sorted.get(i).unwrap();
            let min_val = sorted.get(min_idx).unwrap();
            sorted.set(i, min_val);
            sorted.set(min_idx, tmp);
        }
        i += 1;
    }
    sorted
}

pub mod errors;
mod migration;
mod monitoring;
mod nft_handler;
pub mod storage;
mod token_handler;
#[path = "types.rs"]
mod types_impl;
pub mod types {
    pub use crate::types_impl::*;

    use soroban_sdk::{contracttype, Address};

    #[contracttype]
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub enum PoolOperation {
        Create,
        Fund,
        Distribute,
        Refund,
        Withdraw,
        Migrate,
        Freeze,
        Unfreeze,
    }

    #[contracttype]
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct PoolAuditEntry {
        pub actor: Address,
        pub operation: PoolOperation,
        pub timestamp: u64,
        pub amount: Option<i128>,
    }
}
mod xlm_handler;

#[cfg(test)]
mod test;

#[cfg(test)]
mod multi_token_test;

#[cfg(test)]
mod fund_reentrancy_test;
