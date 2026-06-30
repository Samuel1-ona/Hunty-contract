use soroban_sdk::{contracttype, Address};

pub use reward_interface::RewardConfig;

/// Outcome of a manually resolved distribution.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ResolutionStatus {
    Completed,
    Refunded,
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
}

/// Internal record stored for each distribution.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DistributionRecord {
    pub xlm_amount: i128,
    pub nft_id: Option<u64>,
}

/// Configuration for a reward pool, set at creation time.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RewardPoolConfig {
    /// Address of the hunt creator who owns this pool.
    /// Only the creator is authorized to fund it.
    pub creator: Address,
    /// Minimum XLM amount per distribution. 0 means no minimum enforced.
    pub min_distribution_amount: i128,
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
