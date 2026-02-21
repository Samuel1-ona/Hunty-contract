use soroban_sdk::{contracttype, Address, String};

/// Configuration for distributing rewards. Uses only primitive/Option types for reliable contracttype.
/// At least one of xlm_amount or nft_contract must be set for a valid distribution.
#[contracttype]
#[derive(Clone, Debug)]
pub struct RewardConfig {
    /// XLM amount to distribute. None if no XLM rewards.
    pub xlm_amount: Option<i128>,
    /// NFT contract address. None if no NFT rewards.
    pub nft_contract: Option<Address>,
    /// NFT title. Used when nft_contract is Some.
    pub nft_title: String,
    /// NFT description. Used when nft_contract is Some.
    pub nft_description: String,
    /// NFT image URI. Used when nft_contract is Some.
    pub nft_image_uri: String,
    /// Hunt title (for metadata context). Defaults to nft_title when same.
    pub nft_hunt_title: String,
    /// Rarity tier: 0 = default, 1-5 = common to legendary.
    pub nft_rarity: u32,
    /// Custom tier (0 = none).
    pub nft_tier: u32,
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

impl RewardConfig {
    /// Returns true if XLM rewards are configured.
    pub fn has_xlm(&self) -> bool {
        self.xlm_amount
            .map(|a| a > 0)
            .unwrap_or(false)
    }

    /// Returns true if NFT rewards are configured.
    pub fn has_nft(&self) -> bool {
        self.nft_contract.is_some()
    }

    /// Returns true if at least one reward type is configured.
    pub fn is_valid(&self) -> bool {
        self.has_xlm() || self.has_nft()
    }
}
