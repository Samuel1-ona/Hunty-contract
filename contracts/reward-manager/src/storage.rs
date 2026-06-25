use soroban_sdk::{symbol_short, Address, Env};

use crate::types::{DistributionRecord, RewardPoolConfig};

pub struct Storage;

impl Storage {
    const ADMIN_KEY: soroban_sdk::Symbol = symbol_short!("ADMIN");
    const XLM_TOKEN_KEY: soroban_sdk::Symbol = symbol_short!("XLMTKN");
    const NFT_CONTRACT_KEY: soroban_sdk::Symbol = symbol_short!("NFTADR");
    // Daily spending caps
    const DAILY_POOL_CAP_KEY: soroban_sdk::Symbol = symbol_short!("DPCAP");
    const DAILY_GLOBAL_CAP_KEY: soroban_sdk::Symbol = symbol_short!("DGRCAP");
    // Daily distribution tracking
    const DAILY_POOL_DIST_KEY: soroban_sdk::Symbol = symbol_short!("DPDST");
    const DAILY_GLOBAL_DIST_KEY: soroban_sdk::Symbol = symbol_short!("DGDST");
    const DISTRIBUTION_KEY: soroban_sdk::Symbol = symbol_short!("DIST");
    const DIST_RECORD_KEY: soroban_sdk::Symbol = symbol_short!("DREC");
    const POOL_KEY: soroban_sdk::Symbol = symbol_short!("POOL");
    const POOL_CFG_KEY: soroban_sdk::Symbol = symbol_short!("PCFG");
    const POOL_DEP_KEY: soroban_sdk::Symbol = symbol_short!("PDEP");
    const POOL_DST_KEY: soroban_sdk::Symbol = symbol_short!("PDST");
    const TOTAL_XLM_DST_KEY: soroban_sdk::Symbol = symbol_short!("TXLMDST");
    const HUNTY_CORE_KEY: soroban_sdk::Symbol = symbol_short!("HCORE");
    const TOTAL_XLM_DST_KEY: soroban_sdk::Symbol = symbol_short!("TXDST");
    const IN_DISTRIBUTION_KEY: soroban_sdk::Symbol = symbol_short!("IN_DIST");

    // ========== Admin ==========

    pub fn set_admin(env: &Env, address: &Address) {
        env.storage().persistent().set(&Self::ADMIN_KEY, address);
    }

    pub fn get_admin(env: &Env) -> Option<Address> {
        env.storage().persistent().get(&Self::ADMIN_KEY)
    }

    // ========== XLM Token Address ==========

    pub fn set_xlm_token(env: &Env, address: &Address) {
        env.storage()
            .persistent()
            .set(&Self::XLM_TOKEN_KEY, address);
    }

    pub fn get_xlm_token(env: &Env) -> Option<Address> {
        env.storage().persistent().get(&Self::XLM_TOKEN_KEY)
    }

    // ========== HuntyCore Contract Address ==========

    pub fn set_hunty_core(env: &Env, address: &Address) {
        env.storage().persistent().set(&Self::HUNTY_CORE_KEY, address);
    }

    pub fn get_hunty_core(env: &Env) -> Option<Address> {
        env.storage().persistent().get(&Self::HUNTY_CORE_KEY)
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

    fn distribution_record_key(
        hunt_id: u64,
        player: &Address,
    ) -> (soroban_sdk::Symbol, u64, Address) {
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

    // ========== Reward Pool Configuration (per hunt) ==========

    pub fn set_pool_config(env: &Env, hunt_id: u64, config: &RewardPoolConfig) {
        let key = Self::pool_config_key(hunt_id);
        env.storage().persistent().set(&key, config);
    }

    pub fn get_pool_config(env: &Env, hunt_id: u64) -> Option<RewardPoolConfig> {
        let key = Self::pool_config_key(hunt_id);
        env.storage().persistent().get(&key)
    }

    // ========== Pool Deposit / Distribution Totals (per hunt) ==========

    pub fn set_pool_total_deposited(env: &Env, hunt_id: u64, amount: i128) {
        let key = Self::pool_dep_key(hunt_id);
        env.storage().persistent().set(&key, &amount);
    }

    pub fn get_pool_total_deposited(env: &Env, hunt_id: u64) -> i128 {
        let key = Self::pool_dep_key(hunt_id);
        env.storage().persistent().get(&key).unwrap_or(0)
    }

    pub fn set_pool_total_distributed(env: &Env, hunt_id: u64, amount: i128) {
        let key = Self::pool_dst_key(hunt_id);
        env.storage().persistent().set(&key, &amount);
    }

    pub fn get_pool_total_distributed(env: &Env, hunt_id: u64) -> i128 {
        let key = Self::pool_dst_key(hunt_id);
        env.storage().persistent().get(&key).unwrap_or(0)
    }

    // ========== Global Total XLM Distributed (across all hunts) ==========

    pub fn set_total_xlm_distributed(env: &Env, amount: i128) {
        env.storage().persistent().set(&Self::TOTAL_XLM_DST_KEY, &amount);
    }

    pub fn get_total_xlm_distributed(env: &Env) -> i128 {
        env.storage()
            .persistent()
            .get(&Self::TOTAL_XLM_DST_KEY)
            .unwrap_or(0)
    }

    // Daily pool cap getters/setters
    pub fn set_daily_pool_cap(env: &Env, hunt_id: u64, cap: i128) {
        let key = (Self::DAILY_POOL_CAP_KEY, hunt_id);
        env.storage().persistent().set(&key, &cap);
    }

    pub fn get_daily_pool_cap(env: &Env, hunt_id: u64) -> i128 {
        let key = (Self::DAILY_POOL_CAP_KEY, hunt_id);
        env.storage().persistent().get(&key).unwrap_or(0)
    }

    pub fn set_daily_global_cap(env: &Env, cap: i128) {
        env.storage().persistent().set(&Self::DAILY_GLOBAL_CAP_KEY, &cap);
    }

    pub fn get_daily_global_cap(env: &Env) -> i128 {
        env.storage().persistent().get(&Self::DAILY_GLOBAL_CAP_KEY).unwrap_or(0)
    }

    // Daily distribution tracking
    pub fn add_daily_pool_distributed(env: &Env, hunt_id: u64, day: u64, amount: i128) {
        let key = (Self::DAILY_POOL_DIST_KEY, hunt_id, day);
        let cur = env.storage().persistent().get(&key).unwrap_or(0);
        env.storage().persistent().set(&key, &(cur + amount));
    }

    pub fn get_daily_pool_distributed(env: &Env, hunt_id: u64, day: u64) -> i128 {
        let key = (Self::DAILY_POOL_DIST_KEY, hunt_id, day);
        env.storage().persistent().get(&key).unwrap_or(0)
    }

    pub fn add_daily_global_distributed(env: &Env, day: u64, amount: i128) {
        let key = (Self::DAILY_GLOBAL_DIST_KEY, day);
        let cur = env.storage().persistent().get(&key).unwrap_or(0);
        env.storage().persistent().set(&key, &(cur + amount));
    }

    pub fn get_daily_global_distributed(env: &Env, day: u64) -> i128 {
        let key = (Self::DAILY_GLOBAL_DIST_KEY, day);
        env.storage().persistent().get(&key).unwrap_or(0)
    }
        env.storage().persistent().get(&Self::TOTAL_XLM_DST_KEY).unwrap_or(0)
    }

    // ========== Reentrancy Guard ==========

    pub fn set_in_distribution(env: &Env, value: bool) {
        env.storage().persistent().set(&Self::IN_DISTRIBUTION_KEY, &value);
    }

    pub fn is_in_distribution(env: &Env) -> bool {
        env.storage()
            .persistent()
            .get(&Self::IN_DISTRIBUTION_KEY)
            .unwrap_or(false)
    }

    // ========== Key Helpers ==========

    fn distribution_key(hunt_id: u64, player: &Address) -> (soroban_sdk::Symbol, u64, Address) {
        (Self::DISTRIBUTION_KEY, hunt_id, player.clone())
    }

    fn pool_key(hunt_id: u64) -> (soroban_sdk::Symbol, u64) {
        (Self::POOL_KEY, hunt_id)
    }

    fn pool_config_key(hunt_id: u64) -> (soroban_sdk::Symbol, u64) {
        (Self::POOL_CFG_KEY, hunt_id)
    }

    fn pool_dep_key(hunt_id: u64) -> (soroban_sdk::Symbol, u64) {
        (Self::POOL_DEP_KEY, hunt_id)
    }

    fn pool_dst_key(hunt_id: u64) -> (soroban_sdk::Symbol, u64) {
        (Self::POOL_DST_KEY, hunt_id)
    }

    // --- Contract version ---

    pub fn set_contract_version(env: &Env, version: u32) {
        env.storage().instance().set(&symbol_short!("CVER"), &version);
    }

    pub fn get_contract_version(env: &Env) -> Option<u32> {
        env.storage().instance().get(&symbol_short!("CVER"))
    }
}