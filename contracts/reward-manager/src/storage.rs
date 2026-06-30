use soroban_sdk::{symbol_short, Address, Env};

use crate::types::{DistributionRecord, RewardPoolConfig, PoolAuditEntry, PoolOperation};
pub struct Storage;

impl Storage {
    // Shortened storage prefixes for reward-manager
    const ADMIN_KEY: soroban_sdk::Symbol = symbol_short!("ADMI");
    const XLM_TOKEN_KEY: soroban_sdk::Symbol = symbol_short!("X");
    const NFT_CONTRACT_KEY: soroban_sdk::Symbol = symbol_short!("NFTA");
    // Daily spending caps
    const DAILY_POOL_CAP_KEY: soroban_sdk::Symbol = symbol_short!("DPC");
    const DAILY_GLOBAL_CAP_KEY: soroban_sdk::Symbol = symbol_short!("DGR");
    // Daily distribution tracking
    const DAILY_POOL_DIST_KEY: soroban_sdk::Symbol = symbol_short!("DPD");
    const DAILY_GLOBAL_DIST_KEY: soroban_sdk::Symbol = symbol_short!("DGD");
    const DISTRIBUTION_KEY: soroban_sdk::Symbol = symbol_short!("DI");
    const DIST_RECORD_KEY: soroban_sdk::Symbol = symbol_short!("DR");
    const DIST_NONCE_KEY: soroban_sdk::Symbol = symbol_short!("DN");
    const DIST_RESOLVE_KEY: soroban_sdk::Symbol = symbol_short!("DRS");
    const POOL_KEY: soroban_sdk::Symbol = symbol_short!("POOL");
    const POOL_CFG_KEY: soroban_sdk::Symbol = symbol_short!("PC");
    const POOL_DEP_KEY: soroban_sdk::Symbol = symbol_short!("PDE");
    const POOL_DST_KEY: soroban_sdk::Symbol = symbol_short!("PDS");
    const TOTAL_XLM_DST_KEY: soroban_sdk::Symbol = symbol_short!("TXL");
    const HUNTY_CORE_KEY: soroban_sdk::Symbol = symbol_short!("H");
    const IN_DISTRIBUTION_KEY: soroban_sdk::Symbol = symbol_short!("IN_");
    const PAUSED_KEY: soroban_sdk::Symbol = symbol_short!("PA");
    const EMERGENCY_LOG_KEY: soroban_sdk::Symbol = symbol_short!("EML");
    const PENDING_NFT_KEY: soroban_sdk::Symbol = symbol_short!("PNFT");

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
        env.storage()
            .persistent()
            .set(&Self::HUNTY_CORE_KEY, address);
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

    pub fn get_distribution_nonce(env: &Env, hunt_id: u64, player: &Address) -> u64 {
        let key = Self::distribution_nonce_key(hunt_id, player);
        env.storage().instance().get(&key).unwrap_or(0)
    }

    pub fn set_distribution_nonce(env: &Env, hunt_id: u64, player: &Address, nonce: u64) {
        let key = Self::distribution_nonce_key(hunt_id, player);
        env.storage().instance().set(&key, &nonce);
    }

    pub fn increment_distribution_nonce(env: &Env, hunt_id: u64, player: &Address) -> u64 {
        let current_nonce = Self::get_distribution_nonce(env, hunt_id, player);
        let new_nonce = current_nonce + 1;
        Self::set_distribution_nonce(env, hunt_id, player, new_nonce);
        new_nonce
    }

    fn distribution_record_key(
        hunt_id: u64,
        player: &Address,
    ) -> (soroban_sdk::Symbol, u64, Address) {
        (Self::DIST_RECORD_KEY, hunt_id, player.clone())
    }

    fn distribution_nonce_key(
        hunt_id: u64,
        player: &Address,
    ) -> (soroban_sdk::Symbol, u64, Address) {
        (Self::DIST_NONCE_KEY, hunt_id, player.clone())
    }

    // ========== Distribution Resolution ==========

    pub fn set_distribution_resolution(
        env: &Env,
        hunt_id: u64,
        player: &Address,
        resolution: &ResolutionStatus,
    ) {
        let key = Self::dist_resolve_key(hunt_id, player);
        env.storage().persistent().set(&key, resolution);
    }

    pub fn get_distribution_resolution(
        env: &Env,
        hunt_id: u64,
        player: &Address,
    ) -> Option<ResolutionStatus> {
        let key = Self::dist_resolve_key(hunt_id, player);
        env.storage().persistent().get(&key)
    }

    fn dist_resolve_key(
        hunt_id: u64,
        player: &Address,
    ) -> (soroban_sdk::Symbol, u64, Address) {
        (Self::DIST_RESOLVE_KEY, hunt_id, player.clone())
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
        env.storage()
            .persistent()
            .set(&Self::TOTAL_XLM_DST_KEY, &amount);
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

    // ========== Reentrancy Guard ==========

    pub fn set_in_distribution(env: &Env, value: bool) {
        env.storage()
            .persistent()
            .set(&Self::IN_DISTRIBUTION_KEY, &value);
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

    // ========== Audit Log ==========

    pub fn append_audit_entry(env: &Env, hunt_id: u64, entry: PoolAuditEntry) {
        let count_key = (Self::AUDIT_COUNT_KEY, hunt_id);
        let current_count: u64 = env.storage().persistent().get(&count_key).unwrap_or(0);
        
        let index = current_count % Self::MAX_AUDIT_ENTRIES_PER_POOL;
        let log_key = (Self::AUDIT_LOG_KEY, hunt_id, index);
        
        env.storage().persistent().set(&log_key, &entry);
        env.storage().persistent().set(&count_key, &(current_count + 1));
    }

    pub fn get_pool_audit_count(env: &Env, hunt_id: u64) -> u64 {
        let count_key = (Self::AUDIT_COUNT_KEY, hunt_id);
        env.storage().persistent().get(&count_key).unwrap_or(0)
    }

    pub fn get_pool_audit_entry(env: &Env, hunt_id: u64, index: u64) -> Option<PoolAuditEntry> {
        let log_key = (Self::AUDIT_LOG_KEY, hunt_id, index % Self::MAX_AUDIT_ENTRIES_PER_POOL);
        env.storage().persistent().get(&log_key)
    }

    // ========== Pause / Emergency State ==========

    pub fn set_paused(env: &Env, paused: bool) {
        env.storage().instance().set(&Self::PAUSED_KEY, &paused);
    }

    pub fn is_paused(env: &Env) -> bool {
        env.storage()
            .instance()
            .get(&Self::PAUSED_KEY)
            .unwrap_or(false)
    }

    pub fn log_emergency_withdrawal(env: &Env, log_entry: &crate::EmergencyWithdrawalLogEntry) {
        let key = Self::emergency_log_key();
        let mut logs: soroban_sdk::Vec<crate::EmergencyWithdrawalLogEntry> = env
            .storage()
            .instance()
            .get(&key)
            .unwrap_or_else(|| soroban_sdk::Vec::new(env));
        logs.push_back(log_entry.clone());
        env.storage().instance().set(&key, &logs);
    }

    pub fn get_emergency_logs(env: &Env) -> soroban_sdk::Vec<crate::EmergencyWithdrawalLogEntry> {
        let key = Self::emergency_log_key();
        env.storage()
            .instance()
            .get(&key)
            .unwrap_or_else(|| soroban_sdk::Vec::new(env))
    }

    fn emergency_log_key() -> soroban_sdk::Symbol {
        Self::EMERGENCY_LOG_KEY
    }

    // ========== Pending NFT Mints (for retry) ==========

    pub fn set_pending_nft_mint(
        env: &Env,
        hunt_id: u64,
        player: &Address,
        pending: &crate::PendingNftMint,
    ) {
        let key = Self::pending_nft_key(hunt_id, player);
        env.storage().persistent().set(&key, pending);
    }

    pub fn get_pending_nft_mint(
        env: &Env,
        hunt_id: u64,
        player: &Address,
    ) -> Option<crate::PendingNftMint> {
        let key = Self::pending_nft_key(hunt_id, player);
        env.storage().persistent().get(&key)
    }

    pub fn remove_pending_nft_mint(env: &Env, hunt_id: u64, player: &Address) {
        let key = Self::pending_nft_key(hunt_id, player);
        env.storage().persistent().remove(&key);
    }

    fn pending_nft_key(
        hunt_id: u64,
        player: &Address,
    ) -> (soroban_sdk::Symbol, u64, Address) {
        (Self::PENDING_NFT_KEY, hunt_id, player.clone())
    }

    // --- Contract version ---

    pub fn set_contract_version(env: &Env, version: u32) {
        env.storage()
            .instance()
            .set(&symbol_short!("CVER"), &version);
    }

    pub fn get_contract_version(env: &Env) -> Option<u32> {
        env.storage().instance().get(&symbol_short!("CVER"))
    }
}