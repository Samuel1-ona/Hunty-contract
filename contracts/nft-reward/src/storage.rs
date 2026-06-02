use crate::NftData;
use soroban_sdk::{symbol_short, Address, Env, Vec};

/// Storage layer for NFTs.
pub struct Storage;

impl Storage {
    const NFT_KEY: soroban_sdk::Symbol = symbol_short!("NFT");
    const NFT_COUNTER_KEY: soroban_sdk::Symbol = symbol_short!("CNTR");
    const OWNER_NFT_COUNT_KEY: soroban_sdk::Symbol = symbol_short!("ONFC");

    fn nft_key(nft_id: u64) -> (soroban_sdk::Symbol, u64) {
        (Self::NFT_KEY, nft_id)
    }

    /// Key for a single owner-nft entry: (ONFT, owner, index)
    fn owner_nft_entry_key(owner: &Address, index: u32) -> (soroban_sdk::Symbol, Address, u32) {
        (symbol_short!("ONFT"), owner.clone(), index)
    }

    /// Key for the count of NFTs owned: (ONFC, owner)
    fn owner_nft_count_key(owner: &Address) -> (soroban_sdk::Symbol, Address) {
        (Self::OWNER_NFT_COUNT_KEY, owner.clone())
    }

    /// Key for existence check: (ONFX, owner, nft_id)
    fn owner_nft_exist_key(owner: &Address, nft_id: u64) -> (soroban_sdk::Symbol, Address, u64) {
        (symbol_short!("ONFX"), owner.clone(), nft_id)
    }

    /// Removes an NFT from persistent storage.
    pub fn remove_nft(env: &Env, nft_id: u64) {
        let key = Self::nft_key(nft_id);
        env.storage().persistent().remove(&key);
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

    /// Adds an NFT ID to the owner's index.
    /// Each entry is stored at its own key so no single entry grows unboundedly.
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

    /// Removes an NFT ID from the owner's index.
    pub fn remove_nft_from_owner(env: &Env, owner: &Address, nft_id: u64) {
        let count_key = Self::owner_nft_count_key(owner);
        let count: u32 = env.storage().persistent().get(&count_key).unwrap_or(0);

        let exist_key = Self::owner_nft_exist_key(owner, nft_id);
        if !env.storage().persistent().has(&exist_key) {
            return;
        }

        // Find the entry index and swap-remove with the last entry
        let mut found = false;
        for i in 0..count {
            let entry_key = Self::owner_nft_entry_key(owner, i);
            if let Some(stored_id) = env.storage().persistent().get::<_, u64>(&entry_key) {
                if stored_id == nft_id {
                    let last_idx = count - 1;
                    if i != last_idx {
                        let last_key = Self::owner_nft_entry_key(owner, last_idx);
                        if let Some(last_id) = env.storage().persistent().get::<_, u64>(&last_key) {
                            env.storage().persistent().set(&entry_key, &last_id);
                        }
                        env.storage().persistent().remove(&last_key);
                    } else {
                        env.storage().persistent().remove(&entry_key);
                    }
                    found = true;
                    break;
                }
            }
        }

        if found {
            env.storage().persistent().set(&count_key, &(count - 1));
        }
        env.storage().persistent().remove(&exist_key);
    }

    /// Gets all NFT IDs owned by an address by reading individual entries.
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
}
