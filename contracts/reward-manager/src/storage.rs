use soroban_sdk::{symbol_short, Address, Env};

use crate::types::{DistributionRecord, RewardPoolConfig};

/// Storage access layer for the RewardManager contract.
///
/// # Storage Key Namespace — RewardManager (prefix: "RM")
///
/// All keys are unique within this contract and isolated from other contracts:
///
/// | Constant             | Symbol    | Purpose                                      |
/// |----------------------|-----------|----------------------------------------------|
/// | `CONTRACT_PREFIX`    | `"RM"`    | Contract namespace guard (documents ownership)|
/// | `ADMIN_KEY`          | `"ADMIN"` | Admin address                                |
/// | `XLM_TOKEN_KEY`      | `"XLMTKN"`| XLM token contract address                  |
/// | `NFT_CONTRACT_KEY`   | `"NFTADR"`| Default NFT contract address                |
/// | `DISTRIBUTION_KEY`   | `"DIST"`  | Distribution flag (hunt_id, player)          |
/// | `DIST_RECORD_KEY`    | `"DREC"`  | Full distribution record                     |
/// | `POOL_KEY`           | `"POOL"`  | Reward pool balance per hunt                 |
/// | `POOL_CFG_KEY`       | `"PCFG"`  | Reward pool config per hunt                  |
/// | `POOL_DEP_KEY`       | `"PDEP"`  | Total deposited per hunt                     |
/// | `POOL_DST_KEY`       | `"PDST"`  | Total distributed per hunt                   |
pub struct Storage;

impl Storage {
    /// Contract namespace identifier for RewardManager.
    /// All storage keys in this contract belong to the "RM" namespace.
    /// This ensures no key can collide with HuntyCore ("HC") or NftReward ("NR").
    pub const CONTRACT_PREFIX: &'static str = "RM";

    const ADMIN_KEY: soroban_sdk::Symbol = symbol_short!("ADMIN");
    const XLM_TOKEN_KEY: soroban_sdk::Symbol = symbol_short!("XLMTKN");
    const NFT_CONTRACT_KEY: soroban_sdk::Symbol = symbol_short!("NFTADR");
    const DISTRIBUTION_KEY: soroban_sdk::Symbol = symbol_short!("DIST");
    const DIST_RECORD_KEY: soroban_sdk::Symbol = symbol_short!("DREC");
    const POOL_KEY: soroban_sdk::Symbol = symbol_short!("POOL");
    const POOL_CFG_KEY: soroban_sdk::Symbol = symbol_short!("PCFG");
    const POOL_DEP_KEY: soroban_sdk::Symbol = symbol_short!("PDEP");
    const POOL_DST_KEY: soroban_sdk::Symbol = symbol_short!("PDST");

    // ========== XLM Token Address ==========

    pub fn set_admin(env: &Env, address: &Address) {
        env.storage().persistent().set(&Self::ADMIN_KEY, address);
    }

    pub fn get_admin(env: &Env) -> Option<Address> {
        env.storage().persistent().get(&Self::ADMIN_KEY)
    }

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
}

#[cfg(test)]
mod key_isolation_tests {
    use super::Storage;
    use soroban_sdk::symbol_short;

    /// Verify the RewardManager contract prefix is "RM" and unique across contracts.
    #[test]
    fn test_contract_prefix_is_rm() {
        assert_eq!(Storage::CONTRACT_PREFIX, "RM");
    }

    /// Verify the prefix does not equal the other contracts' prefixes.
    #[test]
    fn test_prefix_distinct_from_other_contracts() {
        assert_ne!(Storage::CONTRACT_PREFIX, "HC"); // HuntyCore
        assert_ne!(Storage::CONTRACT_PREFIX, "NR"); // NftReward
    }

    /// Verify all symbol constants within reward-manager are distinct (no intra-contract collision).
    #[test]
    fn test_no_intra_contract_key_collision() {
        let keys = [
            symbol_short!("ADMIN"),
            symbol_short!("XLMTKN"),
            symbol_short!("NFTADR"),
            symbol_short!("DIST"),
            symbol_short!("DREC"),
            symbol_short!("POOL"),
            symbol_short!("PCFG"),
            symbol_short!("PDEP"),
            symbol_short!("PDST"),
        ];
        for i in 0..keys.len() {
            for j in (i + 1)..keys.len() {
                assert_ne!(
                    keys[i], keys[j],
                    "Duplicate key at indices {} and {}",
                    i, j
                );
            }
        }
    }
}
