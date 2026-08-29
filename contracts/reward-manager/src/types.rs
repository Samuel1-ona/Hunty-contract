use soroban_sdk::{contracttype, Address, BytesN, Vec};

pub use reward_interface::{
    resolve_tier_amount, tiers_are_strictly_ascending, RewardConfig, TierError, TimeBasedRewardTier,
};

/// How XLM rewards are calculated from the pool at distribution time.
#[contracttype]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u32)]
pub enum DistributionMode {
    /// Fixed amount supplied by the caller (`RewardConfig.xlm_amount`).
    Fixed = 0,
    /// Share of the pool: `(player_score / total_scores) * pool_balance`.
    Proportional = 1,
}

/// On-chain receipt / proof of a completed reward distribution.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DistributionProof {
    /// Pool / hunt identifier.
    pub pool_id: u64,
    /// Recipient of the distribution.
    pub player: Address,
    /// XLM amount distributed (stroops).
    pub amount: i128,
    /// Ledger timestamp when the distribution was recorded.
    pub timestamp: u64,
    /// SHA-256 over (pool_id, player, amount, timestamp).
    pub hash: BytesN<32>,
}

/// Resolution outcome for a manually resolved failed distribution.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ResolutionStatus {
    Completed,
    Refunded,
}

/// Semantic versioning struct.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SemVer {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl SemVer {
    /// Returns true if the other version is compatible (same major, minor >= required).
    pub fn is_compatible_with(&self, required: &Self) -> bool {
        self.major == required.major
            && (self.minor > required.minor
                || (self.minor == required.minor && self.patch >= required.patch))
    }
}

/// Status of a reward distribution for a specific hunt and player.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DistributionStatus {
    /// Whether any reward has been distributed.
    pub distributed: bool,
    /// XLM amount distributed (0 if none).
    pub xlm_amount: i128,
    /// NFT ID if an NFT was minted.
    pub nft_id: Option<u64>,
    /// Whether NFT minting failed during distribution (retry available).
    pub nft_mint_failed: bool,
}

/// Internal record stored for each distribution.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DistributionRecord {
    pub xlm_amount: i128,
    pub nft_id: Option<u64>,
}

/// Configuration for a reward pool, set at creation time.
///
/// `time_based_tiers` is an optional list of (max_elapsed_seconds, xlm_amount)
/// pairs that define a conditional reward schedule based on how quickly a
/// player completes a hunt. When the list is empty, time-based conditional
/// rewards are disabled and the rest of the system behaves exactly as
/// before this feature was added. When the list is non-empty it must be
/// sorted in strictly ascending order of `max_completion_secs` (validated
/// in `set_pool_tiers`). The list can be updated after pool creation via
/// `set_pool_tiers` and queried via `get_pool_config`.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RewardPoolConfig {
    /// Address of the hunt creator who owns this pool.
    /// Anyone may fund the pool (see `fund_reward_pool`); the creator is the
    /// only address authorized to manage its configuration and to trigger
    /// `refund_pool`, which pays out the remaining balance pro rata across
    /// every address that funded it.
    pub creator: Address,
    /// Addresses allowed to distribute rewards for this pool.
    /// Only the creator can manage this list.
    pub delegates: Vec<Address>,
    /// Minimum XLM amount per distribution. 0 means no minimum enforced.
    pub min_distribution_amount: i128,
    /// Optional time-based reward tiers. When empty, the per-winner amount
    /// is computed from `xlm_pool / max_winners` as before. When populated,
    /// the appropriate tier's `xlm_amount` is selected at distribution time
    /// based on the player's (completion_time - registration_time) elapsed.
    pub time_based_tiers: Vec<TimeBasedRewardTier>,
    /// Whether distributions from this pool are temporarily frozen.
    /// When `true`, `distribute_rewards` and other distribution functions
    /// will reject calls with `RewardErrorCode::PoolFrozen`.
    pub frozen: bool,
    /// Token address for the reward pool (e.g., XLM, USDC, or other SAC tokens).
    pub token_address: Address,
    /// Optional NFT contract address for NFT-only or mixed reward pools.
    pub nft_contract: Option<Address>,
    /// Target funding amount for progress tracking (0 = disabled).
    pub target_amount: i128,
    /// Minimum seconds between distributions (0 = disabled).
    pub min_distribution_interval_secs: u64,
    /// Distribution mode (Fixed or Proportional).
    pub distribution_mode: DistributionMode,
    /// Optional vesting period in seconds. When > 0, XLM rewards are not
    /// transferred immediately at distribution time. Instead, a `VestingRecord`
    /// is created and the player must call `claim_vested` to receive tokens
    /// proportionally as time elapses. 0 means vesting is disabled (instant payout).
    pub vesting_period_secs: u64,
    /// Unix timestamp after which claims are no longer allowed (0 = disabled).
    pub claim_deadline: u64,
    /// Creator royalty basis points (0-10000) for NFT secondary market sales.
    /// Only applied when minting reward NFTs from this pool.
    pub nft_royalty_bps: u32,
    /// Whether reward NFTs minted from this pool are transferable.
    /// If false, NFTs are soulbound to the initial recipient.
    pub nft_transferable: bool,
}

/// Full status of a reward pool, returned by get_reward_pool().
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RewardPoolStatus {
    /// Current available balance for distributions.
    pub balance: i128,
    /// Cumulative total deposited into this pool across all fund calls.
    pub total_deposited: i128,
    /// Cumulative total distributed from this pool.
    pub total_distributed: i128,
    /// Pool creator / only authorized funder.
    pub creator: Address,
    /// Minimum XLM per distribution (0 = no minimum).
    pub min_distribution_amount: i128,
    /// Whether distributions from this pool are temporarily frozen.
    pub frozen: bool,
}

/// Pending NFT mint that failed and can be retried by the admin.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PendingNftMint {
    pub hunt_id: u64,
    pub player: Address,
    pub nft_contract: Address,
    pub nft_title: soroban_sdk::String,
    pub nft_description: soroban_sdk::String,
    pub nft_image_uri: soroban_sdk::String,
    pub nft_hunt_title: soroban_sdk::String,
    pub nft_rarity: u32,
    pub nft_tier: u32,
}

/// Result of a pool validation check, returned by validate_pool().
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ValidationResult {
    /// Whether the pool has sufficient funds for the required amount
    /// and the required amount meets the pool's minimum distribution size.
    pub is_valid: bool,
    /// Current pool balance at time of check.
    pub balance: i128,
    /// Required amount that was checked against.
    pub required: i128,
}

/// Operation type for the pool audit log.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PoolOperation {
    Create,
    Fund,
    Distribute,
    Withdraw,
    Freeze,
    Unfreeze,
    /// Unused balance was migrated out to (or into) another hunt's pool.
    Migrate,
    /// Pool balance was refunded to the pool creator.
    Refund,
}

/// Comprehensive statistics for a reward pool, returned by get_pool_statistics().
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RewardPoolStatistics {
    /// Total XLM funded (deposited) into the pool.
    pub total_funded: i128,
    /// Total XLM distributed from the pool.
    pub total_distributed: i128,
    /// Number of successful distributions made from this pool.
    pub distribution_count: u64,
    /// Average XLM amount per distribution (0 if none).
    pub avg_distribution: i128,
    /// Ledger timestamp of the most recent distribution (0 if none).
    pub last_distribution_timestamp: u64,
}

/// A single entry in the pool audit log.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PoolAuditEntry {
    /// Who triggered the operation.
    pub actor: Address,
    /// Operation performed.
    pub operation: PoolOperation,
    /// Timestamp (ledger time).
    pub timestamp: u64,
    /// The XLM amount involved, if applicable.
    pub amount: Option<i128>,
}

/// Entry for batch distribution calls.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BatchDistributionEntry {
    pub hunt_id: u64,
    pub player_address: Address,
    pub reward_config: RewardConfig,
}

/// Record of a completed distribution for a specific pool.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PoolDistribution {
    pub player: Address,
    pub xlm_amount: i128,
    pub nft_id: Option<u64>,
    pub timestamp: u64,
}

/// On-chain storage record for a time-locked vesting reward.
///
/// Created by `distribute_rewards` when `vesting_period_secs > 0`.
/// Tokens are NOT transferred immediately; the player must call
/// `claim_vested` to receive their proportional share as time elapses.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VestingRecord {
    /// Ledger timestamp when vesting began (i.e. when distribute_rewards was called).
    pub start_time: u64,
    /// Total XLM amount (in stroops) locked in this vesting schedule.
    pub total_amount: i128,
    /// Cumulative XLM amount already claimed by the player.
    pub claimed_amount: i128,
    /// Vesting period in seconds (copied from the pool config at distribution time).
    pub vesting_period_secs: u64,
}

/// Read-only view of a player's vesting status for a specific hunt.
/// Returned by `get_vesting_status`.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VestingStatus {
    /// Ledger timestamp when vesting began.
    pub start_time: u64,
    /// Full vesting duration in seconds.
    pub vesting_period_secs: u64,
    /// Total XLM locked under this schedule.
    pub total_amount: i128,
    /// Cumulative XLM already claimed.
    pub claimed_amount: i128,
    /// XLM that has vested so far: `total_amount * min(elapsed / vesting_period_secs, 1)`.
    pub vested_amount: i128,
    /// XLM available to claim right now: `vested_amount - claimed_amount`.
    pub claimable_amount: i128,
    /// True once `claimed_amount >= total_amount`.
    pub fully_vested: bool,
}

/// Statistical summary of distributions across a reward pool,
/// returned by get_distribution_analytics().
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DistributionAnalytics {
    /// Number of distributions included in the analytics window.
    pub count: u64,
    /// Total XLM distributed in the analytics window (stroops).
    pub total: i128,
    /// Average (mean) XLM amount per distribution (stroops). 0 if count is 0.
    pub average: i128,
    /// Median XLM amount across distributions (stroops). 0 if count is 0.
    pub median: i128,
    /// Minimum XLM amount in a single distribution (stroops). 0 if count is 0.
    pub min: i128,
    /// Maximum XLM amount in a single distribution (stroops). 0 if count is 0.
    pub max: i128,
}
