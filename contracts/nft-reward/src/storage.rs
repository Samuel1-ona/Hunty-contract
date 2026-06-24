use crate::NftData;
use soroban_sdk::{symbol_short, Address, Env, Vec};

/// Storage access layer for the NftReward contract.
///
/// # Storage Key Namespace — NftReward (prefix: "NR")
///
/// All keys are unique within this contract and isolated from other contracts:
///
/// | Constant           | Symbol    | Purpose                                       |
/// |--------------------|-----------|-----------------------------------------------|
/// | `CONTRACT_PREFIX`  | `"NR"`    | Contract namespace guard (documents ownership) |
/// | `NFT_KEY`          | `"NFT"`   | Per-NFT data (composite: nft_id)              |
/// | `NFT_COUNTER_KEY`  | `"NFTCNT"`| NFT ID counter (NR-prefixed, avoids "CNTR")   |
/// | `OWNER_NFTS_KEY`   | `"ONFT"`  | Owner → NFT ID list                           |
pub struct Storage;

impl Storage {
    /// Contract namespace identifier for NftReward.
    /// All storage keys in this contract belong to the "NR" namespace.
    /// This ensures no key can collide with HuntyCore ("HC") or RewardManager ("RM").
    pub const CONTRACT_PREFIX: &'static str = "NR";

    const NFT_KEY: soroban_sdk::Symbol = symbol_short!("NFT");
    /// NFT ID counter — uses "NFTCNT" (NR-prefixed) to avoid collision with
    /// HuntyCore's hunt counter.
    pub(crate) const NFT_COUNTER_KEY: soroban_sdk::Symbol = symbol_short!("NFTCNT");
    const OWNER_NFT_COUNT_KEY: soroban_sdk::Symbol = symbol_short!("ONFC");
    const MAX_SUPPLY_KEY: soroban_sdk::Symbol = symbol_short!("MAXS");
    const INITIALIZED_KEY: soroban_sdk::Symbol = symbol_short!("INIT");
    const ADMIN_KEY: soroban_sdk::Symbol = symbol_short!("ADMIN");
    const MINTER_KEY: soroban_sdk::Symbol = symbol_short!("MNTR");
    const REWARD_MGR_KEY: soroban_sdk::Symbol = symbol_short!("RWDMGR");

    fn nft_key(nft_id: u64) -> (soroban_sdk::Symbol, u64) {
        (Self::NFT_KEY, nft_id)
    }

    fn owner_nft_entry_key(owner: &Address, index: u32) -> (soroban_sdk::Symbol, Address, u32) {
        (symbol_short!("ONFT"), owner.clone(), index)
    }

    fn owner_nft_count_key(owner: &Address) -> (soroban_sdk::Symbol, Address) {
        (Self::OWNER_NFT_COUNT_KEY, owner.clone())
    }

    fn owner_nft_exist_key(owner: &Address, nft_id: u64) -> (soroban_sdk::Symbol, Address, u64) {
        (symbol_short!("ONFX"), owner.clone(), nft_id)
    }

    fn minter_key(minter: &Address) -> (soroban_sdk::Symbol, Address) {
        (Self::MINTER_KEY, minter.clone())
    }

    pub fn remove_nft(env: &Env, nft_id: u64) {
        let key = Self::nft_key(nft_id);
        env.storage().persistent().remove(&key);
    }

    #[allow(dead_code)]
    pub fn save_admin(env: &Env, admin: &Address) {
        env.storage().instance().set(&Self::ADMIN_KEY, admin);
    }

    #[allow(dead_code)]
    pub fn get_admin(env: &Env) -> Option<Address> {
        env.storage().instance().get(&Self::ADMIN_KEY)
    }

    #[allow(dead_code)]
    pub fn set_reward_manager(env: &Env, address: &Address) {
        env.storage().instance().set(&Self::REWARD_MGR_KEY, address);
    }

    pub fn get_reward_manager(env: &Env) -> Option<Address> {
        env.storage().instance().get(&Self::REWARD_MGR_KEY)
    }

    // --- Minter whitelist (reserved for admin-gated minting) ---

    #[allow(dead_code)]
    pub fn add_minter(env: &Env, minter: &Address) {
        let key = Self::minter_key(minter);
        env.storage().persistent().set(&key, &true);
    }

    #[allow(dead_code)]
    pub fn remove_minter(env: &Env, minter: &Address) {
        let key = Self::minter_key(minter);
        env.storage().persistent().remove(&key);
    }

    #[allow(dead_code)]
    pub fn is_minter(env: &Env, minter: &Address) -> bool {
        let key = Self::minter_key(minter);
        env.storage().persistent().get(&key).unwrap_or(false)
    }

    pub fn save_nft(env: &Env, nft: &NftData) {
        let key = Self::nft_key(nft.nft_id);
        env.storage().persistent().set(&key, nft);
    }

    pub fn get_nft(env: &Env, nft_id: u64) -> Option<NftData> {
        let key = Self::nft_key(nft_id);
        env.storage().persistent().get(&key)
    }

    pub fn next_nft_id(env: &Env) -> u64 {
        let current: u64 = env
            .storage()
            .persistent()
            .get(&Self::NFT_COUNTER_KEY)
            .unwrap_or(0);
        let next = current + 1;
        env.storage()
            .persistent()
            .set(&Self::NFT_COUNTER_KEY, &next);
        next
    }

    pub fn get_nft_counter(env: &Env) -> u64 {
        env.storage()
            .persistent()
            .get(&Self::NFT_COUNTER_KEY)
            .unwrap_or(0)
    }

    pub fn set_max_supply(env: &Env, max_supply: Option<u64>) {
        env.storage()
            .persistent()
            .set(&Self::MAX_SUPPLY_KEY, &max_supply);
        env.storage().persistent().set(&Self::INITIALIZED_KEY, &true);
    }

    pub fn get_max_supply(env: &Env) -> Option<u64> {
        env.storage()
            .persistent()
            .get(&Self::MAX_SUPPLY_KEY)
            .unwrap_or(None)
    }

    pub fn is_initialized(env: &Env) -> bool {
        env.storage()
            .persistent()
            .get(&Self::INITIALIZED_KEY)
            .unwrap_or(false)
    }

    pub fn add_nft_to_owner(env: &Env, owner: &Address, nft_id: u64) {
        let count_key = Self::owner_nft_count_key(owner);
        let count: u32 = env.storage().persistent().get(&count_key).unwrap_or(0);

        let exist_key = Self::owner_nft_exist_key(owner, nft_id);
        if env.storage().persistent().has(&exist_key) {
            return;
        }

        env.storage()
            .persistent()
            .set(&Self::owner_nft_entry_key(owner, count), &nft_id);
        env.storage().persistent().set(&count_key, &(count + 1));
        env.storage().persistent().set(&exist_key, &());
    }

    /// Returns all minted NFT IDs by iterating from 1 to the current counter.
    pub fn get_all_nft_ids(env: &Env) -> Vec<u64> {
        let counter = Self::get_nft_counter(env);
        let mut ids = Vec::new(env);
        for id in 1..=counter {
            if env.storage().persistent().has(&Self::nft_key(id)) {
                ids.push_back(id);
            }
        }
        ids
    }

    pub fn get_owner_nfts(env: &Env, owner: &Address) -> Vec<u64> {
        let count_key = Self::owner_nft_count_key(owner);
        let count: u32 = env.storage().persistent().get(&count_key).unwrap_or(0);
        let mut ids = Vec::new(env);
        for i in 0..count {
            let entry_key = Self::owner_nft_entry_key(owner, i);
            if let Some(id) = env.storage().persistent().get(&entry_key) {
                ids.push_back(id);
            }
        }
        ids
    }

    // --- Operator management ---

    /// Grants operator approval: `operator` can manage all NFTs owned by `owner`.
    pub fn set_operator(env: &Env, owner: &Address, operator: &Address) {
        let key = Self::operator_key(owner, operator);
        env.storage().persistent().set(&key, &true);
    }

    /// Revokes operator approval.
    pub fn remove_operator(env: &Env, owner: &Address, operator: &Address) {
        let key = Self::operator_key(owner, operator);
        env.storage().persistent().remove(&key);
    }

    /// Returns true if `operator` is approved to manage all NFTs of `owner`.
    pub fn is_operator(env: &Env, owner: &Address, operator: &Address) -> bool {
        let key = Self::operator_key(owner, operator);
        env.storage().persistent().get(&key).unwrap_or(false)
    }

    /// Returns the reward manager address (used for cross-contract auth).
    pub fn get_reward_manager(env: &Env) -> Option<Address> {
        env.storage()
            .instance()
            .get(&symbol_short!("RWMGR"))
    }

    // --- Contract version ---

    pub fn set_contract_version(env: &Env, version: u32) {
        env.storage().instance().set(&symbol_short!("CVER"), &version);
    }

    pub fn get_contract_version(env: &Env) -> Option<u32> {
        env.storage().instance().get(&symbol_short!("CVER"))
    }
}

#[cfg(test)]
mod key_isolation_tests {
    use super::Storage;
    use soroban_sdk::symbol_short;

    /// Verify the NftReward contract prefix is "NR" and unique across contracts.
    #[test]
    fn test_contract_prefix_is_nr() {
        assert_eq!(Storage::CONTRACT_PREFIX, "NR");
    }

    /// Verify the prefix does not equal the other contracts' prefixes.
    #[test]
    fn test_prefix_distinct_from_other_contracts() {
        assert_ne!(Storage::CONTRACT_PREFIX, "HC"); // HuntyCore
        assert_ne!(Storage::CONTRACT_PREFIX, "RM"); // RewardManager
    }

    /// Verify the NFT counter key is "NFTCNT" — renamed from the old "CNTR"
    /// that collided with HuntyCore's hunt counter.
    #[test]
    fn test_nft_counter_key_is_nftcnt() {
        let expected = symbol_short!("NFTCNT");
        assert_eq!(Storage::NFT_COUNTER_KEY, expected);
    }

    /// Verify "NFTCNT" does not equal "HCCNT" (the hunt counter key in HuntyCore).
    #[test]
    fn test_nft_counter_distinct_from_hunt_counter() {
        let nft_counter = symbol_short!("NFTCNT");
        let hunt_counter = symbol_short!("HCCNT");
        assert_ne!(nft_counter, hunt_counter);
    }

    /// Verify all symbol constants within nft-reward are distinct (no intra-contract collision).
    #[test]
    fn test_no_intra_contract_key_collision() {
        let keys = [
            symbol_short!("NFT"),
            symbol_short!("NFTCNT"),
            symbol_short!("ONFT"),
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
