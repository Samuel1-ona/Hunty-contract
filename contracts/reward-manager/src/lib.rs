#![cfg_attr(not(test), no_std)]
use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, Address, Env, IntoVal, Symbol, Val, Vec,
};

pub use crate::errors::RewardErrorCode;
use crate::nft_handler::NftHandler;
use crate::storage::Storage;
pub use crate::types::{
    resolve_tier_amount, tiers_are_strictly_ascending, DistributionRecord, DistributionStatus,
    RewardConfig, RewardPoolConfig, RewardPoolStatus, SemVer, TierError, TimeBasedRewardTier,
    ValidationResult,
    DistributionRecord, DistributionStatus, ResolutionStatus, RewardConfig, RewardPoolConfig,
    RewardPoolStatus, SemVer, ValidationResult,
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

#[contract]
pub struct RewardManager;

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

#[contractimpl]
impl RewardManager {
    /// Current semantic version of this contract.
    pub const CONTRACT_VERSION: u32 = 2;
    /// Minimum NftReward version this contract requires.
    pub const REQUIRED_NFT_REWARD_VERSION: u32 = 2;

    /// Initializes the RewardManager with the XLM token contract address (SAC).
    /// Must be called once before any reward distribution.
    pub fn initialize(env: Env, admin: Address, xlm_token: Address) -> Result<(), RewardErrorCode> {
        if Storage::get_xlm_token(&env).is_some() {
            return Err(RewardErrorCode::AlreadyInitialized);
        }

        admin.require_auth();
        Storage::set_admin(&env, &admin);
        Storage::set_xlm_token(&env, &xlm_token);
        Storage::set_contract_version(&env, Self::CONTRACT_VERSION);
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
        admin.require_auth();
        let configured_admin = Storage::get_admin(&env).ok_or(RewardErrorCode::NotInitialized)?;
        if configured_admin != admin {
            return Err(RewardErrorCode::Unauthorized);
        }

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
        admin.require_auth();
        let configured_admin = Storage::get_admin(&env).ok_or(RewardErrorCode::NotInitialized)?;
        if configured_admin != admin {
            return Err(RewardErrorCode::Unauthorized);
        }
        Storage::set_hunty_core(&env, &hunty_core);
        Ok(())
    }

    /// Adds a contract to the authorized callers list for `distribute_rewards`.
    /// Only the contract admin can call this.
    pub fn add_authorized_contract(
        env: Env,
        admin: Address,
        contract: Address,
    ) -> Result<(), RewardErrorCode> {
        admin.require_auth();
        let configured_admin = Storage::get_admin(&env).ok_or(RewardErrorCode::NotInitialized)?;
        if configured_admin != admin {
            return Err(RewardErrorCode::Unauthorized);
        }
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
        admin.require_auth();
        let configured_admin = Storage::get_admin(&env).ok_or(RewardErrorCode::NotInitialized)?;
        if configured_admin != admin {
            return Err(RewardErrorCode::Unauthorized);
        }
        Storage::remove_authorized_contract(&env, &contract);
        Ok(())
    }

    /// Creates a reward pool for a specific hunt.
    ///
    /// Must be called before `fund_reward_pool`. Only the creator is authorized
    /// to fund the pool after creation.
    ///
    /// # Arguments
    /// * `creator` - The hunt creator who will own and fund the pool
    /// * `hunt_id` - The hunt this pool is for
    /// * `min_distribution_amount` - Minimum XLM per distribution (0 = no minimum)
    ///
    /// # Errors
    /// * `PoolAlreadyExists` - A pool already exists for this hunt_id
    /// * `InvalidAmount` - min_distribution_amount is negative
    /// * `HuntNotFound` - hunt_id does not exist in HuntyCore (only when `set_hunty_core` has been called)
    pub fn create_reward_pool(
        env: Env,
        creator: Address,
        hunt_id: u64,
        min_distribution_amount: i128,
    ) -> Result<(), RewardErrorCode> {
        #[cfg(not(test))]
        creator.require_auth();

        if min_distribution_amount < 0 {
            return Err(RewardErrorCode::InvalidAmount);
        }

        if Storage::get_pool_config(&env, hunt_id).is_some() {
            return Err(RewardErrorCode::PoolAlreadyExists);
        }

        // Validate hunt_id exists in HuntyCore when the core contract is configured.
        // If not configured, hunt_id is caller-trusted (no cross-contract call is made).
        if let Some(hunty_core) = Storage::get_hunty_core(&env) {
            let mut args: Vec<Val> = Vec::new(&env);
            args.push_back(hunt_id.into_val(&env));
            // get_hunt_info returns Result<Hunt, HuntErrorCode>.
            // We use Val as the success type to avoid importing Hunt from hunty-core.
            // Any non-Ok(Ok(_)) result means the hunt doesn't exist or the call failed.
            let hunt_exists = matches!(
                env.try_invoke_contract::<Val, Val>(
                    &hunty_core,
                    &Symbol::new(&env, "get_hunt_info"),
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
            min_distribution_amount,
            time_based_tiers: Vec::new(&env),
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
            (symbol_short!("POOL_TIERS"), hunt_id),
            (creator.clone(), tiers_len),
        );

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

    /// Funds the reward pool for a specific hunt.
    ///
    /// The pool must have been created via `create_reward_pool` first.
    /// Only the original pool creator is authorized to fund it.
    /// Transfers XLM from the funder to this contract and records the balance.
    ///
    /// # Validation
    /// - Minimum funding: 1 XLM (10,000,000 stroops) to prevent dust attacks
    /// - Maximum single funding: 1 billion XLM to prevent overflow
    /// - Pool balance limit: 1 billion XLM total to prevent overflow
    /// - Rejects zero or negative amounts
    ///
    /// # Arguments
    /// * `funder` - The address funding the pool (must be the pool creator)
    /// * `hunt_id` - The hunt to fund
    /// * `amount` - XLM amount to add to the pool (must be > 0)
    ///
    /// # Errors
    /// * `PoolNotFound` - Pool has not been created yet
    /// * `Unauthorized` - Funder is not the pool creator
    /// * `InvalidAmount` - Amount is <= 0
    /// * `BelowMinimumFunding` - Amount is less than 1 XLM (dust attack prevention)
    /// * `ExceedsMaximumFunding` - Amount exceeds 1 billion XLM
    /// * `PoolBalanceOverflow` - Adding this amount would exceed pool balance limit
    /// * `NotInitialized` - XLM token address not set
    pub fn fund_reward_pool(
        env: Env,
        funder: Address,
        hunt_id: u64,
        amount: i128,
    ) -> Result<(), RewardErrorCode> {
        if amount <= 0 {
            return Err(RewardErrorCode::InvalidAmount);
        }

        // Validate minimum funding amount (1 XLM) to prevent dust attacks
        if amount < MIN_FUNDING_AMOUNT {
            return Err(RewardErrorCode::BelowMinimumFunding);
        }

        // Validate maximum single funding amount to prevent overflow
        if amount > MAX_FUNDING_AMOUNT {
            return Err(RewardErrorCode::ExceedsMaximumFunding);
        }

        let pool_config =
            Storage::get_pool_config(&env, hunt_id).ok_or(RewardErrorCode::PoolNotFound)?;

        if funder != pool_config.creator {
            return Err(RewardErrorCode::Unauthorized);
        }

        pool_config.creator.require_auth();

        let xlm_token = Storage::get_xlm_token(&env).ok_or(RewardErrorCode::NotInitialized)?;

        // Check for overflow before adding to pool balance
        let current = Storage::get_pool_balance(&env, hunt_id);
        let new_balance = current.checked_add(amount)
            .ok_or(RewardErrorCode::PoolBalanceOverflow)?;
        
        // Validate the new balance doesn't exceed maximum pool balance
        if new_balance > MAX_POOL_BALANCE {
            return Err(RewardErrorCode::PoolBalanceOverflow);
        }

        // Transfer XLM from funder to this contract
        let contract_addr = env.current_contract_address();
        let client = soroban_sdk::token::Client::new(&env, &xlm_token);
        client.transfer(&funder, &contract_addr, &amount);

        // Update pool balance and cumulative deposit total
        Storage::set_pool_balance(&env, hunt_id, new_balance);

        let total_deposited = Storage::get_pool_total_deposited(&env, hunt_id)
            .checked_add(amount)
            .ok_or(RewardErrorCode::PoolBalanceOverflow)?;
        Storage::set_pool_total_deposited(&env, hunt_id, total_deposited);

        env.events().publish(
            (symbol_short!("POOL_FND"), hunt_id),
            RewardPoolFundedEvent {
                hunt_id,
                funder,
                amount,
                new_balance,
                total_deposited,
            },
        );

        let audit_entry = PoolAuditEntry {
            actor: funder.clone(),
            operation: PoolOperation::Fund,
            timestamp: env.ledger().timestamp(),
            amount: Some(amount),
        };
        Storage::append_audit_entry(&env, hunt_id, audit_entry);

        Ok(())
    }

    /// Refunds the entire remaining pool balance for a hunt back to the pool creator.
    /// Can only be called by the same creator that owns the pool.
    pub fn refund_pool(env: Env, creator: Address, hunt_id: u64) -> Result<(), RewardErrorCode> {
        let pool_config =
            Storage::get_pool_config(&env, hunt_id).ok_or(RewardErrorCode::PoolNotFound)?;
        if creator != pool_config.creator {
            return Err(RewardErrorCode::Unauthorized);
        }

        let balance = Storage::get_pool_balance(&env, hunt_id);
        if balance == 0 {
            return Ok(());
        }

        let xlm_token = Storage::get_xlm_token(&env).ok_or(RewardErrorCode::NotInitialized)?;

        let contract_addr = env.current_contract_address();
        let client = soroban_sdk::token::Client::new(&env, &xlm_token);
        client.transfer(&contract_addr, &creator, &balance);

        Storage::set_pool_balance(&env, hunt_id, 0);

        let audit_entry = PoolAuditEntry {
            actor: creator.clone(),
            operation: PoolOperation::Withdraw,
            timestamp: env.ledger().timestamp(),
            amount: Some(balance),
        };
        Storage::append_audit_entry(&env, hunt_id, audit_entry);

        Ok(())
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
            let meets_balance = required_amount > 0 && balance >= required_amount;
            let meets_minimum = config.min_distribution_amount == 0
                || required_amount >= config.min_distribution_amount;
            meets_balance && meets_minimum
        } else {
            false
        };

        ValidationResult {
            is_valid,
            balance,
            required: required_amount,
        }
    }

    pub fn set_daily_pool_cap(env: Env, admin: Address, hunt_id: u64, cap: i128) -> Result<(), RewardErrorCode> {
        admin.require_auth();
        let configured_admin = Storage::get_admin(&env).ok_or(RewardErrorCode::NotInitialized)?;
        if configured_admin != admin { return Err(RewardErrorCode::Unauthorized); }
        Storage::set_daily_pool_cap(&env, hunt_id, cap);
        Ok(())
    }

    pub fn set_daily_global_cap(env: Env, admin: Address, cap: i128) -> Result<(), RewardErrorCode> {
        admin.require_auth();
        let configured_admin = Storage::get_admin(&env).ok_or(RewardErrorCode::NotInitialized)?;
        if configured_admin != admin { return Err(RewardErrorCode::Unauthorized); }
        Storage::set_daily_global_cap(&env, cap);
        Ok(())
    }

    pub fn distribute_rewards(
        env: Env,
        hunt_id: u64,
        player_address: Address,
        reward_config: RewardConfig,
    ) -> Result<(), RewardErrorCode> {
        // Validate caller is an authorized contract (when configured)
        if Storage::has_authorized_contracts(&env) {
            let caller = env.caller();
            if !Storage::is_authorized_contract(&env, &caller) {
                return Err(RewardErrorCode::Unauthorized);
            }
        }

        // Validate configuration
        if !reward_config.is_valid() {
            return Err(RewardErrorCode::InvalidConfig);
        }

        // Prevent double distribution using monotonic nonce
        // Get current distribution state before any mutations
        let distribution_record = Storage::get_distribution_record(&env, hunt_id, &player_address);
        let current_nonce = Storage::get_distribution_nonce(&env, hunt_id, &player_address);
        
        // Detect replay: if record exists but nonce hasn't been incremented, it's a replay attempt
        if distribution_record.is_some() && current_nonce == 0 {
            return Err(RewardErrorCode::AlreadyDistributed);
        }
        
        // Verify distribution state consistency
        let expected_nonce = if distribution_record.is_some() { 1 } else { 0 };
        if current_nonce != expected_nonce {
            return Err(RewardErrorCode::ReplayDetected);
        }

        let _reentrancy_guard = ReentrancyGuard::acquire(&env)?;

        let mut xlm_amount = 0i128;
        let mut nft_id: Option<u64> = None;

        // Route to XLM handler if configured
        if reward_config.has_xlm() {
            let amount = reward_config.xlm_amount.unwrap();
            if amount <= 0 {
                return Err(RewardErrorCode::InvalidAmount);
            }

            // Enforce pool minimum distribution amount if a pool config exists
            if let Some(pool_config) = Storage::get_pool_config(&env, hunt_id) {
                if pool_config.min_distribution_amount > 0
                    && amount < pool_config.min_distribution_amount
                {
                    return Err(RewardErrorCode::BelowMinimumAmount);
                }
            }

            let xlm_token = Storage::get_xlm_token(&env).ok_or(RewardErrorCode::NotInitialized)?;

            let pool_balance = Storage::get_pool_balance(&env, hunt_id);
            if pool_balance < amount {
                return Err(RewardErrorCode::InsufficientPool);
            }

            let contract_addr = env.current_contract_address();

            if !XlmHandler::validate_pool(&env, &xlm_token, &contract_addr, amount) {
                return Err(RewardErrorCode::PoolBalanceDivergence);
            }

            // Check caps
            let day = env.ledger().timestamp() / 86400;
            Storage::add_daily_pool_distributed(&env, hunt_id, day, amount);
            Storage::add_daily_global_distributed(&env, day, amount);

            let pool_cap = Storage::get_daily_pool_cap(&env, hunt_id);
            if pool_cap > 0 {
                let used = Storage::get_daily_pool_distributed(&env, hunt_id, day);
                if used > pool_cap { return Err(RewardErrorCode::DailyCapExceeded); }
                if used >= (pool_cap * 8 / 10) {
                    env.events().publish((symbol_short!("DP_WARN"),), DailyPoolCapWarningEvent { hunt_id, used, cap: pool_cap });
                }
            }

            let global_cap = Storage::get_daily_global_cap(&env);
            if global_cap > 0 {
                let global_used = Storage::get_daily_global_distributed(&env, day);
                if global_used > global_cap { return Err(RewardErrorCode::GlobalDailyCapExceeded); }
                if global_used >= (global_cap * 8 / 10) {
                    env.events().publish((symbol_short!("DG_WARN"),), GlobalDailyCapWarningEvent { used: global_used, cap: global_cap });
                }
            }

            XlmHandler::distribute_xlm(
                &env,
                &xlm_token,
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
                        },
                    );
                }
            }
        }

        // Record distribution with monotonic nonce to prevent replay attacks
        Storage::set_distribution_record(
            &env,
            hunt_id,
            &player_address,
            &DistributionRecord { xlm_amount, nft_id },
        );
        
        // Increment nonce atomically after successful distribution
        // Instance storage is immutable and not subject to TTL expiration
        Storage::increment_distribution_nonce(&env, hunt_id, &player_address);

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
            amount: if xlm_amount > 0 { Some(xlm_amount) } else { None },
        };
        Storage::append_audit_entry(&env, hunt_id, audit_entry);

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
        admin.require_auth();
        let configured_admin = Storage::get_admin(&env).ok_or(RewardErrorCode::NotInitialized)?;
        if configured_admin != admin {
            return Err(RewardErrorCode::Unauthorized);
        }

        let pending = Storage::get_pending_nft_mint(&env, hunt_id, &player)
            .ok_or(RewardErrorCode::NftMintPendingNotFound)?;

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
        Storage::is_distributed(&env, hunt_id, &player)
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
        admin.require_auth();
        let configured_admin = Storage::get_admin(&env).ok_or(RewardErrorCode::NotInitialized)?;
        if configured_admin != admin {
            return Err(RewardErrorCode::Unauthorized);
        }

        if !Storage::is_distributed(&env, hunt_id, &player) {
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

    /// Allows the admin to withdraw any unclaimed (surplus) XLM remaining in a reward pool.
    ///
    /// This is needed when a hunt concludes with fewer winners than anticipated,
    /// leaving unspent XLM locked in the pool. Only the contract admin may call this.
    ///
    /// # Arguments
    /// * `admin` - The contract admin address (must match the stored admin)
    /// * `hunt_id` - The hunt whose remaining pool balance to withdraw
    /// * `recipient` - The address that will receive the withdrawn XLM
    ///
    /// # Errors
    /// * `NotInitialized` - Contract has not been initialized (no admin set)
    /// * `Unauthorized` - Caller is not the contract admin
    /// * `PoolNotFound` - No pool exists for this hunt_id
    /// * `InvalidAmount` - Pool balance is zero (nothing to withdraw)
    pub fn admin_withdraw_unclaimed(
        env: Env,
        admin: Address,
        hunt_id: u64,
        recipient: Address,
        amount: i128,
    ) -> Result<(), RewardErrorCode> {
        if amount < 0 {
            return Err(RewardErrorCode::InvalidAmount);
        }
        #[cfg(not(test))]
        admin.require_auth();

        let configured_admin = Storage::get_admin(&env).ok_or(RewardErrorCode::NotInitialized)?;
        if configured_admin != admin {
            return Err(RewardErrorCode::Unauthorized);
        }

        // Ensure the pool exists
        Storage::get_pool_config(&env, hunt_id).ok_or(RewardErrorCode::PoolNotFound)?;

        let balance = Storage::get_pool_balance(&env, hunt_id);
        let withdraw_amount = if amount == 0 { balance } else { amount };

        if withdraw_amount <= 0 || withdraw_amount > balance {
            return Err(RewardErrorCode::InvalidAmount);
        }

        monitoring::Monitoring::record_large_withdrawal(&env, withdraw_amount);
        monitoring::Monitoring::record_invocation(&env, 80_000, true);

        let xlm_token = Storage::get_xlm_token(&env).ok_or(RewardErrorCode::NotInitialized)?;

        let contract_addr = env.current_contract_address();
        let client = soroban_sdk::token::Client::new(&env, &xlm_token);
        client.transfer(&contract_addr, &recipient, &withdraw_amount);

        Storage::set_pool_balance(&env, hunt_id, balance - withdraw_amount);

        env.events().publish(
            (symbol_short!("ADM_WDR"), hunt_id),
            AdminWithdrawEvent {
                hunt_id,
                admin: admin.clone(),
                amount: withdraw_amount,
            },
        );

        let audit_entry = PoolAuditEntry {
            actor: admin.clone(),
            operation: PoolOperation::Withdraw,
            timestamp: env.ledger().timestamp(),
            amount: Some(withdraw_amount),
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
        #[cfg(not(test))]
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
        #[cfg(not(test))]
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
        #[cfg(not(test))]
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
        migration::RewardManagerMigration::get_upgrade_history(&env, offset, limit)
    }

    pub fn run_migration(
        env: Env,
        admin: Address,
        target_version: u32,
        dry_run: bool,
    ) -> Result<migration::MigrationReport, hunty_migration::UpgradeAuthError> {
        let from_version = migration::RewardManagerMigration::get_schema_version(&env);
        let report =
            migration::RewardManagerMigration::run_migration(&env, &admin, target_version, dry_run)?;
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

pub mod errors;
mod migration;
mod monitoring;
mod nft_handler;
mod storage;
mod types;
mod xlm_handler;

#[cfg(test)]
mod test;
