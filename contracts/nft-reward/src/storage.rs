use crate::NftData;
use soroban_sdk::{symbol_short, Address, Env, Vec};

/// Storage layer for NFTs.
pub struct Storage;

impl Storage {
    const NFT_KEY: soroban_sdk::Symbol = symbol_short!("NFT");
    const NFT_COUNTER_KEY: soroban_sdk::Symbol = symbol_short!("CNTR");
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
}
