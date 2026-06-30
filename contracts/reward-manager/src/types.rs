use soroban_sdk::{contracttype, Address};

pub use reward_interface::RewardConfig;

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
