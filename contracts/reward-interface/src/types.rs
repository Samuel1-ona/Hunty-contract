use soroban_sdk::{contracttype, Address, String};

/// Configuration for distributing rewards across the HuntyCore ↔ RewardManager boundary.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RewardConfig {
    pub xlm_amount: Option<i128>,
    pub nft_contract: Option<Address>,
    pub nft_title: String,
    pub nft_description: String,
    pub nft_image_uri: String,
    pub nft_hunt_title: String,
    pub nft_rarity: u32,
    pub nft_tier: u32,
}

impl RewardConfig {
    pub fn has_xlm(&self) -> bool {
        self.xlm_amount.map(|a| a > 0).unwrap_or(false)
    }

    pub fn has_nft(&self) -> bool {
        self.nft_contract.is_some()
    }

    pub fn is_valid(&self) -> bool {
        self.has_xlm() || self.has_nft()
    }
}
