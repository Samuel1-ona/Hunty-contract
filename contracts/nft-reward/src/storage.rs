use crate::NftData;
use soroban_sdk::{symbol_short, Address, Env, Vec};

/// Storage layer for NFTs.
pub struct Storage;

impl Storage {
    const NFT_KEY: soroban_sdk::Symbol = symbol_short!("NFT");
    const NFT_COUNTER_KEY: soroban_sdk::Symbol = symbol_short!("CNTR");
    const OWNER_NFT_COUNT_KEY: soroban_sdk::Symbol = symbol_short!("ONFC");
    const MAX_SUPPLY_KEY: soroban_sdk::Symbol = symbol_short!("MAXS");
    const INITIALIZED_KEY: soroban_sdk::Symbol = symbol_short!("INIT");

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

    /// Key for operator approval: (OPER, owner, operator)
    fn operator_key(owner: &Address, operator: &Address) -> (soroban_sdk::Symbol, Address, Address) {
        (symbol_short!("OPER"), owner.clone(), operator.clone())
    }

    // --- Initialization / max supply ---

    /// Marks the contract initialized and stores the optional max supply cap.
    pub fn set_max_supply(env: &Env, max_supply: Option<u64>) {
        env.storage().persistent().set(&Self::MAX_SUPPLY_KEY, &max_supply);
        env.storage().persistent().set(&Self::INITIALIZED_KEY, &true);
    }

    /// Returns the configured max supply cap, if one has been stored.
    pub fn get_max_supply(env: &Env) -> Option<u64> {
        env.storage()
            .persistent()
            .get(&Self::MAX_SUPPLY_KEY)
            .unwrap_or(None)
    }

    /// Returns whether the contract has been initialized.
    pub fn is_initialized(env: &Env) -> bool {
        env.storage()
            .persistent()
            .get(&Self::INITIALIZED_KEY)
            .unwrap_or(false)
    }

    // --- NFT storage ---

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

    /// Removes an NFT from persistent storage.
    pub fn remove_nft(env: &Env, nft_id: u64) {
        let key = Self::nft_key(nft_id);
        env.storage().persistent().remove(&key);
    }

    // --- NFT counter ---

    /// Increments and returns the next NFT ID.
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

    /// Gets the current NFT counter (total minted).
    pub fn get_nft_counter(env: &Env) -> u64 {
        env.storage()
            .persistent()
            .get(&Self::NFT_COUNTER_KEY)
            .unwrap_or(0)
    }

    // --- Owner NFT index ---

    /// Adds an NFT ID to the owner's index.
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

    /// Gets all NFT IDs owned by an address.
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

    pub fn set_contract_version(env: &Env, version: &crate::SemVer) {
        env.storage().instance().set(&symbol_short!("CVER"), version);
    }

    pub fn get_contract_version(env: &Env) -> Option<crate::SemVer> {
        env.storage().instance().get(&symbol_short!("CVER"))
    }
}
