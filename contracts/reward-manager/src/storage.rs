use soroban_sdk::{symbol_short, Address, Env, Vec};

use crate::types::{
    DistributionProof, DistributionRecord, PoolAuditEntry, PoolDistribution, ResolutionStatus,
    RewardPoolConfig, VestingRecord,
};

pub struct Storage;

#[allow(dead_code)]
impl Storage {
    // Shortened storage prefixes for reward-manager
    const ADMIN_KEY: soroban_sdk::Symbol = symbol_short!("ADMI");
    const PENDING_ADMIN_KEY: soroban_sdk::Symbol = symbol_short!("PADMI");
    const XLM_TOKEN_KEY: soroban_sdk::Symbol = symbol_short!("X");
    const NFT_CONTRACT_KEY: soroban_sdk::Symbol = symbol_short!("NFTA");
    /// Ring-buffer capacity for the per-pool audit log.
    const MAX_AUDIT_ENTRIES_PER_POOL: u64 = 50;
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
    const DIST_PROOF_KEY: soroban_sdk::Symbol = symbol_short!("DPRF");
    const LAST_DIST_TS_KEY: soroban_sdk::Symbol = symbol_short!("LDTS");
    const POOL_KEY: soroban_sdk::Symbol = symbol_short!("POOL");
    const POOL_CFG_KEY: soroban_sdk::Symbol = symbol_short!("PCFG");
    const POOL_DEP_KEY: soroban_sdk::Symbol = symbol_short!("PDEP");
    const POOL_DST_KEY: soroban_sdk::Symbol = symbol_short!("PDST");
    const POOL_RFD_KEY: soroban_sdk::Symbol = symbol_short!("PRFD");
    const POOL_DIST_COUNT_KEY: soroban_sdk::Symbol = symbol_short!("PDCNT");
    const POOL_LAST_DIST_TS_KEY: soroban_sdk::Symbol = symbol_short!("PLDTS");
    const POOL_DISTRIBUTIONS_KEY: soroban_sdk::Symbol = symbol_short!("PLDIST");
    const TOTAL_XLM_DST_KEY: soroban_sdk::Symbol = symbol_short!("TXDST");
    const HUNTY_CORE_KEY: soroban_sdk::Symbol = symbol_short!("HCORE");
    const IN_DISTRIBUTION_KEY: soroban_sdk::Symbol = symbol_short!("IN_DIST");
    const HAS_AUTH_KEY: soroban_sdk::Symbol = symbol_short!("HAUTH");
    const AUDIT_COUNT_KEY: soroban_sdk::Symbol = symbol_short!("AUDC");
    const AUDIT_LOG_KEY: soroban_sdk::Symbol = symbol_short!("AUDL");
    const PAUSED_KEY: soroban_sdk::Symbol = symbol_short!("PAUSE");
    // Granular pause flags (issue #628), mirroring the per-operation pauses
    // hunty-core already exposes. The global PAUSED_KEY above still overrides
    // both, so an emergency stop remains a single call.
    const PAUSE_FUNDING_KEY: soroban_sdk::Symbol = symbol_short!("PAUSE_FD");
    const PAUSE_DIST_KEY: soroban_sdk::Symbol = symbol_short!("PAUSE_DS");
    const EMERGENCY_LOG_KEY: soroban_sdk::Symbol = symbol_short!("EMLOG");
    /// List of distinct addresses that have funded a pool and not yet been refunded.
    const POOL_FUNDERS_KEY: soroban_sdk::Symbol = symbol_short!("PFNDRS");
    /// Per-(hunt_id, funder) cumulative amount contributed and not yet refunded.
    const POOL_FUNDER_CONTRIB_KEY: soroban_sdk::Symbol = symbol_short!("PFCONT");

    pub const PENDING_NFT_KEY: soroban_sdk::Symbol = symbol_short!("PNFT");

    // ========== Vesting ==========
    const VESTING_KEY: soroban_sdk::Symbol = symbol_short!("VEST");

    // ========== Admin ==========

    pub fn set_admin(env: &Env, address: &Address) {
        env.storage().persistent().set(&Self::ADMIN_KEY, address);
    }

    pub fn get_admin(env: &Env) -> Option<Address> {
        env.storage().persistent().get(&Self::ADMIN_KEY)
    }

    pub fn set_pending_admin(env: &Env, address: &Address) {
        env.storage()
            .persistent()
            .set(&Self::PENDING_ADMIN_KEY, address);
    }

    pub fn get_pending_admin(env: &Env) -> Option<Address> {
        env.storage().persistent().get(&Self::PENDING_ADMIN_KEY)
    }

    pub fn clear_pending_admin(env: &Env) {
        env.storage().persistent().remove(&Self::PENDING_ADMIN_KEY);
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

    // ========== Pool Distribution List ==========

    /// Adds a distribution record to the pool's distribution list.
    pub fn add_pool_distribution(env: &Env, hunt_id: u64, distribution: PoolDistribution) {
        let key = Self::pool_distributions_key(hunt_id);
        let mut distributions: Vec<PoolDistribution> = env
            .storage()
            .persistent()
            .get(&key)
            .unwrap_or_else(|| Vec::new(env));
        distributions.push_back(distribution);
        env.storage().persistent().set(&key, &distributions);
    }

    /// Returns paginated list of distributions for a pool.
    /// Returns up to `limit` entries starting from `offset`.
    pub fn get_pool_distributions(
        env: &Env,
        hunt_id: u64,
        offset: u32,
        limit: u32,
    ) -> Vec<PoolDistribution> {
        let key = Self::pool_distributions_key(hunt_id);
        let all_distributions: Vec<PoolDistribution> = env
            .storage()
            .persistent()
            .get(&key)
            .unwrap_or_else(|| Vec::new(env));

        let total = all_distributions.len();
        if offset >= total {
            return Vec::new(env);
        }

        let end_index = core::cmp::min(offset + limit, total);
        let mut result = Vec::new(env);
        for i in offset..end_index {
            if let Some(distribution) = all_distributions.get(i) {
                result.push_back(distribution.clone());
            }
        }
        result
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

    fn dist_resolve_key(hunt_id: u64, player: &Address) -> (soroban_sdk::Symbol, u64, Address) {
        (Self::DIST_RESOLVE_KEY, hunt_id, player.clone())
    }

    // ========== Distribution Rate Limit (per pool) ==========

    pub fn set_last_distribution_timestamp(env: &Env, hunt_id: u64, timestamp: u64) {
        let key = (Self::LAST_DIST_TS_KEY, hunt_id);
        env.storage().persistent().set(&key, &timestamp);
        // Marker so timestamp 0 (common in tests / genesis) is distinct from "never distributed".
        let flag_key = (Self::LAST_DIST_TS_KEY, hunt_id, true);
        env.storage().persistent().set(&flag_key, &true);
    }

    pub fn get_last_distribution_timestamp(env: &Env, hunt_id: u64) -> Option<u64> {
        let flag_key = (Self::LAST_DIST_TS_KEY, hunt_id, true);
        if !env.storage().persistent().get(&flag_key).unwrap_or(false) {
            return None;
        }
        let key = (Self::LAST_DIST_TS_KEY, hunt_id);
        Some(env.storage().persistent().get(&key).unwrap_or(0))
    }

    // ========== Distribution Proof / Receipt ==========

    pub fn set_distribution_proof(
        env: &Env,
        hunt_id: u64,
        player: &Address,
        proof: &DistributionProof,
    ) {
        let key = Self::distribution_proof_key(hunt_id, player);
        env.storage().persistent().set(&key, proof);
    }

    pub fn get_distribution_proof(
        env: &Env,
        hunt_id: u64,
        player: &Address,
    ) -> Option<DistributionProof> {
        let key = Self::distribution_proof_key(hunt_id, player);
        env.storage().persistent().get(&key)
    }

    fn distribution_proof_key(
        hunt_id: u64,
        player: &Address,
    ) -> (soroban_sdk::Symbol, u64, Address) {
        (Self::DIST_PROOF_KEY, hunt_id, player.clone())
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

    pub fn set_pool_total_refunded(env: &Env, hunt_id: u64, amount: i128) {
        let key = Self::pool_rfd_key(hunt_id);
        env.storage().persistent().set(&key, &amount);
    }

    pub fn get_pool_total_refunded(env: &Env, hunt_id: u64) -> i128 {
        let key = Self::pool_rfd_key(hunt_id);
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

    // ========== Pool Distribution Count & Last Timestamp ==========

    pub fn set_pool_distribution_count(env: &Env, hunt_id: u64, count: u64) {
        let key = (Self::POOL_DIST_COUNT_KEY, hunt_id);
        env.storage().persistent().set(&key, &count);
    }

    pub fn get_pool_distribution_count(env: &Env, hunt_id: u64) -> u64 {
        let key = (Self::POOL_DIST_COUNT_KEY, hunt_id);
        env.storage().persistent().get(&key).unwrap_or(0)
    }

    pub fn increment_pool_distribution_count(env: &Env, hunt_id: u64) -> u64 {
        let current = Self::get_pool_distribution_count(env, hunt_id);
        let new = current + 1;
        Self::set_pool_distribution_count(env, hunt_id, new);
        new
    }

    pub fn set_pool_last_distribution_timestamp(env: &Env, hunt_id: u64, timestamp: u64) {
        let key = (Self::POOL_LAST_DIST_TS_KEY, hunt_id);
        env.storage().persistent().set(&key, &timestamp);
    }

    pub fn get_pool_last_distribution_timestamp(env: &Env, hunt_id: u64) -> u64 {
        let key = (Self::POOL_LAST_DIST_TS_KEY, hunt_id);
        env.storage().persistent().get(&key).unwrap_or(0)
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
        env.storage()
            .persistent()
            .set(&Self::DAILY_GLOBAL_CAP_KEY, &cap);
    }

    pub fn get_daily_global_cap(env: &Env) -> i128 {
        env.storage()
            .persistent()
            .get(&Self::DAILY_GLOBAL_CAP_KEY)
            .unwrap_or(0)
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

    // ========== Authorized Cross-Contract Callers ==========

    fn authorized_contract_key(contract: &Address) -> (soroban_sdk::Symbol, Address) {
        (symbol_short!("AUTH"), contract.clone())
    }

    pub fn has_authorized_contracts(env: &Env) -> bool {
        env.storage()
            .instance()
            .get(&Self::HAS_AUTH_KEY)
            .unwrap_or(false)
    }

    pub fn add_authorized_contract(env: &Env, contract: &Address) {
        let key = Self::authorized_contract_key(contract);
        env.storage().persistent().set(&key, &true);
        env.storage().instance().set(&Self::HAS_AUTH_KEY, &true);
    }

    pub fn remove_authorized_contract(env: &Env, contract: &Address) {
        let key = Self::authorized_contract_key(contract);
        env.storage().persistent().remove(&key);
    }

    pub fn is_authorized_contract(env: &Env, contract: &Address) -> bool {
        let key = Self::authorized_contract_key(contract);
        env.storage().persistent().get(&key).unwrap_or(false)
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

    fn pool_rfd_key(hunt_id: u64) -> (soroban_sdk::Symbol, u64) {
        (Self::POOL_RFD_KEY, hunt_id)
    }

    fn pool_distributions_key(hunt_id: u64) -> (soroban_sdk::Symbol, u64) {
        (Self::POOL_DISTRIBUTIONS_KEY, hunt_id)
    }

    // ========== Pool Funders (sponsorship tracking) ==========

    /// Returns the distinct addresses that have funded a pool and have not yet
    /// been refunded, in the order they first contributed.
    pub fn get_pool_funders(env: &Env, hunt_id: u64) -> Vec<Address> {
        let key = Self::pool_funders_key(hunt_id);
        env.storage()
            .persistent()
            .get(&key)
            .unwrap_or_else(|| Vec::new(env))
    }

    pub fn set_pool_funders(env: &Env, hunt_id: u64, funders: &Vec<Address>) {
        let key = Self::pool_funders_key(hunt_id);
        env.storage().persistent().set(&key, funders);
    }

    pub fn remove_pool_funders(env: &Env, hunt_id: u64) {
        let key = Self::pool_funders_key(hunt_id);
        env.storage().persistent().remove(&key);
    }

    /// Cumulative amount `funder` has contributed to a pool that has not yet
    /// been refunded. 0 if they have never funded it or were already refunded.
    pub fn get_pool_funder_contribution(env: &Env, hunt_id: u64, funder: &Address) -> i128 {
        let key = Self::pool_funder_contribution_key(hunt_id, funder);
        env.storage().persistent().get(&key).unwrap_or(0)
    }

    pub fn set_pool_funder_contribution(env: &Env, hunt_id: u64, funder: &Address, amount: i128) {
        let key = Self::pool_funder_contribution_key(hunt_id, funder);
        env.storage().persistent().set(&key, &amount);
    }

    pub fn remove_pool_funder_contribution(env: &Env, hunt_id: u64, funder: &Address) {
        let key = Self::pool_funder_contribution_key(hunt_id, funder);
        env.storage().persistent().remove(&key);
    }

    fn pool_funders_key(hunt_id: u64) -> (soroban_sdk::Symbol, u64) {
        (Self::POOL_FUNDERS_KEY, hunt_id)
    }

    fn pool_funder_contribution_key(
        hunt_id: u64,
        funder: &Address,
    ) -> (soroban_sdk::Symbol, u64, Address) {
        (Self::POOL_FUNDER_CONTRIB_KEY, hunt_id, funder.clone())
    }

    // ========== Audit Log ==========

    pub fn append_audit_entry(env: &Env, hunt_id: u64, entry: PoolAuditEntry) {
        let count_key = (Self::AUDIT_COUNT_KEY, hunt_id);
        let current_count: u64 = env.storage().persistent().get(&count_key).unwrap_or(0);

        let index = current_count % Self::MAX_AUDIT_ENTRIES_PER_POOL;
        let log_key = (Self::AUDIT_LOG_KEY, hunt_id, index);

        env.storage().persistent().set(&log_key, &entry);
        env.storage()
            .persistent()
            .set(&count_key, &(current_count + 1));
    }

    pub fn get_pool_audit_count(env: &Env, hunt_id: u64) -> u64 {
        let count_key = (Self::AUDIT_COUNT_KEY, hunt_id);
        env.storage().persistent().get(&count_key).unwrap_or(0)
    }

    pub fn get_pool_audit_entry(env: &Env, hunt_id: u64, index: u64) -> Option<PoolAuditEntry> {
        let log_key = (
            Self::AUDIT_LOG_KEY,
            hunt_id,
            index % Self::MAX_AUDIT_ENTRIES_PER_POOL,
        );
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

    // ---- Granular pause flags (issue #628) ----
    //
    // `hunty-core` can pause registrations, answers and rewards independently.
    // reward-manager had a single flag, so stopping a suspect distribution also
    // stopped creators topping their pools up. These split the two halves.

    pub fn set_funding_paused(env: &Env, paused: bool) {
        env.storage()
            .instance()
            .set(&Self::PAUSE_FUNDING_KEY, &paused);
    }

    /// True when funding is blocked, either by its own flag or by the global stop.
    pub fn is_funding_paused(env: &Env) -> bool {
        Self::is_paused(env)
            || env
                .storage()
                .instance()
                .get(&Self::PAUSE_FUNDING_KEY)
                .unwrap_or(false)
    }

    pub fn set_distribution_paused(env: &Env, paused: bool) {
        env.storage().instance().set(&Self::PAUSE_DIST_KEY, &paused);
    }

    /// True when distribution is blocked, either by its own flag or by the
    /// global stop.
    pub fn is_distribution_paused(env: &Env) -> bool {
        Self::is_paused(env)
            || env
                .storage()
                .instance()
                .get(&Self::PAUSE_DIST_KEY)
                .unwrap_or(false)
    }

    /// The two granular flags on their own, ignoring the global stop. Used by
    /// `get_pause_state` so an operator can tell a granular pause apart from an
    /// emergency stop.
    pub fn raw_pause_flags(env: &Env) -> (bool, bool) {
        (
            env.storage()
                .instance()
                .get(&Self::PAUSE_FUNDING_KEY)
                .unwrap_or(false),
            env.storage()
                .instance()
                .get(&Self::PAUSE_DIST_KEY)
                .unwrap_or(false),
        )
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

    pub fn pending_nft_key(hunt_id: u64, player: &Address) -> (soroban_sdk::Symbol, u64, Address) {
        (Self::PENDING_NFT_KEY, hunt_id, player.clone())
    }

    // ========== Vesting Records ==========

    /// Stores a vesting record for a (hunt_id, player) pair.
    /// Called at distribution time when vesting_period_secs > 0.
    pub fn set_vesting_record(env: &Env, hunt_id: u64, player: &Address, record: &VestingRecord) {
        let key = Self::vesting_key(hunt_id, player);
        env.storage().persistent().set(&key, record);
    }

    /// Returns the vesting record for a (hunt_id, player) pair, or None if it
    /// does not exist (i.e. no vesting is pending for this player).
    pub fn get_vesting_record(env: &Env, hunt_id: u64, player: &Address) -> Option<VestingRecord> {
        let key = Self::vesting_key(hunt_id, player);
        env.storage().persistent().get(&key)
    }

    fn vesting_key(hunt_id: u64, player: &Address) -> (soroban_sdk::Symbol, u64, Address) {
        (Self::VESTING_KEY, hunt_id, player.clone())
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
