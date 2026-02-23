use soroban_sdk::{symbol_short, Address, Env};

use crate::types::DistributionRecord;

pub struct Storage;

impl Storage {
    const XLM_TOKEN_KEY: soroban_sdk::Symbol = symbol_short!("XLMTKN");
    const NFT_CONTRACT_KEY: soroban_sdk::Symbol = symbol_short!("NFTADR");
    const DISTRIBUTION_KEY: soroban_sdk::Symbol = symbol_short!("DIST");
    const DIST_RECORD_KEY: soroban_sdk::Symbol = symbol_short!("DREC");
    const POOL_KEY: soroban_sdk::Symbol = symbol_short!("POOL");

    // ========== XLM Token Address ==========

    pub fn set_xlm_token(env: &Env, address: &Address) {
        env.storage().persistent().set(&Self::XLM_TOKEN_KEY, address);
    }

    pub fn get_xlm_token(env: &Env) -> Option<Address> {
        env.storage().persistent().get(&Self::XLM_TOKEN_KEY)
    }

    // ========== Default NFT Contract Address ==========

    pub fn set_nft_contract(env: &Env, address: &Address) {
        env.storage()
            .persistent()
            .set(&Self::NFT_CONTRACT_KEY, address);
    }

    pub fn get_nft_contract(env: &Env) -> Option<Address> {
        env.storage().persistent().get(&Self::NFT_CONTRACT_KEY)
    }

    // ========== Distribution Tracking ==========

    pub fn set_distributed(env: &Env, hunt_id: u64, player: &Address) {
        let key = Self::distribution_key(hunt_id, player);
        env.storage().persistent().set(&key, &true);
    }

    pub fn is_distributed(env: &Env, hunt_id: u64, player: &Address) -> bool {
        let key = Self::distribution_key(hunt_id, player);
        env.storage().persistent().get(&key).unwrap_or(false)
    }

    /// Stores the full distribution record (xlm_amount, nft_id) for status queries.
    pub fn set_distribution_record(
        env: &Env,
        hunt_id: u64,
        player: &Address,
        record: &DistributionRecord,
    ) {
        let key = Self::distribution_record_key(hunt_id, player);
        env.storage().persistent().set(&key, record);
    }

    pub fn get_distribution_record(
        env: &Env,
        hunt_id: u64,
        player: &Address,
    ) -> Option<DistributionRecord> {
        let key = Self::distribution_record_key(hunt_id, player);
        env.storage().persistent().get(&key)
    }

    fn distribution_record_key(hunt_id: u64, player: &Address) -> (soroban_sdk::Symbol, u64, Address) {
        (Self::DIST_RECORD_KEY, hunt_id, player.clone())
    }

    // ========== Reward Pool Balance (per hunt) ==========

    pub fn set_pool_balance(env: &Env, hunt_id: u64, balance: i128) {
        let key = Self::pool_key(hunt_id);
        env.storage().persistent().set(&key, &balance);
    }

    pub fn get_pool_balance(env: &Env, hunt_id: u64) -> i128 {
        let key = Self::pool_key(hunt_id);
        env.storage().persistent().get(&key).unwrap_or(0)
    }

    // ========== Key Helpers ==========

    fn distribution_key(hunt_id: u64, player: &Address) -> (soroban_sdk::Symbol, u64, Address) {
        (Self::DISTRIBUTION_KEY, hunt_id, player.clone())
    }

    fn pool_key(hunt_id: u64) -> (soroban_sdk::Symbol, u64) {
        (Self::POOL_KEY, hunt_id)
    }
}
