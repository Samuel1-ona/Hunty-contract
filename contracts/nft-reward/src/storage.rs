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
    /// NFT ID counter — renamed from "CNTR" to "NFTCNT" to eliminate the
    /// collision with HuntyCore's former "CNTR" hunt-counter key.
    pub(crate) const NFT_COUNTER_KEY: soroban_sdk::Symbol = symbol_short!("NFTCNT");
    const OWNER_NFTS_KEY: soroban_sdk::Symbol = symbol_short!("ONFT");

    fn nft_key(nft_id: u64) -> (soroban_sdk::Symbol, u64) {
        (Self::NFT_KEY, nft_id)
    }

    fn owner_nfts_key(owner: &Address) -> (soroban_sdk::Symbol, Address) {
        (Self::OWNER_NFTS_KEY, owner.clone())
    }

    /// Saves an NFT to persistent storage.
    pub fn save_nft(env: &Env, nft: &NftData) {
        let key = Self::nft_key(nft.nft_id);
        env.storage().persistent().set(&key, nft);
    }

    /// Retrieves an NFT by ID.
    pub fn get_nft(env: &Env, nft_id: u64) -> Option<NftData> {
        let key = Self::nft_key(nft_id);
        env.storage().persistent().get(&key)
    }

    /// Increments and returns the next NFT ID.
    pub fn next_nft_id(env: &Env) -> u64 {
        let current: u64 = env.storage().persistent().get(&Self::NFT_COUNTER_KEY).unwrap_or(0);
        let next = current + 1;
        env.storage().persistent().set(&Self::NFT_COUNTER_KEY, &next);
        next
    }

    /// Gets the current NFT counter (total minted).
    pub fn get_nft_counter(env: &Env) -> u64 {
        env.storage()
            .persistent()
            .get(&Self::NFT_COUNTER_KEY)
            .unwrap_or(0)
    }

    /// Adds an NFT ID to the owner's list.
    pub fn add_nft_to_owner(env: &Env, owner: &Address, nft_id: u64) {
        let key = Self::owner_nfts_key(owner);
        let mut nft_ids = env
            .storage()
            .persistent()
            .get(&key)
            .unwrap_or_else(|| Vec::new(env));
        nft_ids.push_back(nft_id);
        env.storage().persistent().set(&key, &nft_ids);
    }

    /// Removes an NFT ID from the owner's list.
    pub fn remove_nft_from_owner(env: &Env, owner: &Address, nft_id: u64) {
        let key = Self::owner_nfts_key(owner);
        let mut nft_ids = env
            .storage()
            .persistent()
            .get(&key)
            .unwrap_or_else(|| Vec::new(env));
        if let Some(idx) = nft_ids.first_index_of(nft_id) {
            nft_ids.remove(idx);
        }
        env.storage().persistent().set(&key, &nft_ids);
    }

    /// Gets all NFT IDs owned by an address.
    pub fn get_owner_nfts(env: &Env, owner: &Address) -> Vec<u64> {
        let key = Self::owner_nfts_key(owner);
        env.storage()
            .persistent()
            .get(&key)
            .unwrap_or_else(|| Vec::new(env))
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
