use crate::{NftData, NftCore, NftMetadata};
use soroban_sdk::{symbol_short, Address, Env, Vec, Symbol};

/// Storage layer for NFTs.
pub struct Storage;

impl Storage {
    // Shortened storage prefixes for nft-reward
    const NFT_KEY: soroban_sdk::Symbol = symbol_short!("NF");
    const NFT_CORE_KEY: soroban_sdk::Symbol = symbol_short!("NC");
    const NFT_META_KEY: soroban_sdk::Symbol = symbol_short!("NM");
    const NFT_COUNTER_KEY: soroban_sdk::Symbol = symbol_short!("CN");
    const OWNER_NFT_COUNT_KEY: soroban_sdk::Symbol = symbol_short!("ONFC");
    const HUNT_NFT_COUNT_KEY: soroban_sdk::Symbol = symbol_short!("HN");
    const MAX_SUPPLY_KEY: soroban_sdk::Symbol = symbol_short!("MA");
    const INITIALIZED_KEY: soroban_sdk::Symbol = symbol_short!("I");
    const ADMIN_KEY: soroban_sdk::Symbol = symbol_short!("A");
    const MINTER_KEY: soroban_sdk::Symbol = symbol_short!("MN");
    const REWARD_MGR_KEY: soroban_sdk::Symbol = symbol_short!("R");
    const NFT_VERSION_KEY: soroban_sdk::Symbol = symbol_short!("NV");
    const TOTAL_HUNTS_KEY: soroban_sdk::Symbol = symbol_short!("TH");
    const TOTAL_OWNERS_KEY: soroban_sdk::Symbol = symbol_short!("TO");
    const ALL_NFTS_KEY: soroban_sdk::Symbol = symbol_short!("AN");
    const CONTRACT_VERSION_KEY: soroban_sdk::Symbol = symbol_short!("CV");

    fn nft_key(nft_id: u64) -> (soroban_sdk::Symbol, u64) {
        (Self::NFT_KEY, nft_id)
    }

    fn nft_core_key(nft_id: u64) -> (soroban_sdk::Symbol, u64) {
        (Self::NFT_CORE_KEY, nft_id)
    }

    fn nft_metadata_key(nft_id: u64) -> (soroban_sdk::Symbol, u64) {
        (Self::NFT_META_KEY, nft_id)
    }

    fn nft_version_key(nft_id: u64) -> (soroban_sdk::Symbol, u64) {
        (Self::NFT_VERSION_KEY, nft_id)
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

    fn hunt_nft_count_key(hunt_id: u64) -> (soroban_sdk::Symbol, u64) {
        (Self::HUNT_NFT_COUNT_KEY, hunt_id)
    }

    fn hunt_nft_exist_key(hunt_id: u64, nft_id: u64) -> (soroban_sdk::Symbol, u64, u64) {
        (symbol_short!("HNFX"), hunt_id, nft_id)
    }

    fn hunt_nft_entry_key(hunt_id: u64, index: u32) -> (soroban_sdk::Symbol, u64, u32) {
        (symbol_short!("HNFT"), hunt_id, index)
    }

    fn minter_key(minter: &Address) -> (soroban_sdk::Symbol, Address) {
        (Self::MINTER_KEY, minter.clone())
    }

    fn operator_key(
        owner: &Address,
        operator: &Address,
    ) -> (soroban_sdk::Symbol, Address, Address) {
        (symbol_short!("OPKEY"), owner.clone(), operator.clone())
    }

    fn locker_key(locker: &Address) -> (soroban_sdk::Symbol, Address) {
        (symbol_short!("LOCKR"), locker.clone())
    }

    pub fn save_admin(env: &Env, admin: &Address) {
        env.storage().instance().set(&Self::ADMIN_KEY, admin);
    }

    pub fn get_admin(env: &Env) -> Option<Address> {
        env.storage().instance().get(&Self::ADMIN_KEY)
    }

    pub fn set_reward_manager(env: &Env, address: &Address) {
        env.storage().instance().set(&Self::REWARD_MGR_KEY, address);
    }

    pub fn save_reward_manager(env: &Env, address: &Address) {
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
        let core = NftCore {
            nft_id: nft.nft_id,
            hunt_id: nft.hunt_id,
            owner: nft.owner.clone(),
            completion_player: nft.completion_player.clone(),
            transferable: nft.transferable,
            minted_at: nft.minted_at,
            locked: nft.locked,
        };
        let core_key = Self::nft_core_key(nft.nft_id);
        env.storage().persistent().set(&core_key, &core);

        let meta_key = Self::nft_metadata_key(nft.nft_id);
        let existing_meta: Option<NftMetadata> = env.storage().persistent().get(&meta_key);
        if existing_meta.as_ref() != Some(&nft.metadata) {
            env.storage().persistent().set(&meta_key, &nft.metadata);
        }

        // Also add to all NFTs list for iteration (only if not already present)
        let mut all_nfts = env
            .storage()
            .persistent()
            .get(&Self::ALL_NFTS_KEY)
            .unwrap_or_else(|| Vec::new(env));

        // Check if NFT ID already exists to avoid duplicates
        if all_nfts.first_index_of(nft.nft_id).is_none() {
            all_nfts.push_back(nft.nft_id);
            env.storage().persistent().set(&Self::ALL_NFTS_KEY, &all_nfts);
        }
    }

    pub fn get_nft(env: &Env, nft_id: u64) -> Option<NftData> {
        let core_key = Self::nft_core_key(nft_id);
        let core: Option<NftCore> = env.storage().persistent().get(&core_key);
        
        let meta_key = Self::nft_metadata_key(nft_id);
        let meta: Option<NftMetadata> = env.storage().persistent().get(&meta_key);

        if let (Some(c), Some(m)) = (core, meta) {
            Some(NftData {
                nft_id: c.nft_id,
                hunt_id: c.hunt_id,
                owner: c.owner,
                completion_player: c.completion_player,
                metadata: m,
                transferable: c.transferable,
                minted_at: c.minted_at,
                locked: c.locked,
            })
        } else {
            None
        }
    }

    pub fn remove_nft(env: &Env, nft_id: u64) {
        let core_key = Self::nft_core_key(nft_id);
        env.storage().persistent().remove(&core_key);

        let meta_key = Self::nft_metadata_key(nft_id);
        env.storage().persistent().remove(&meta_key);

        let version_key = Self::nft_version_key(nft_id);
        env.storage().persistent().remove(&version_key);

        // Also remove from ALL_NFTS_KEY list
        let mut all_nfts = env
            .storage()
            .persistent()
            .get(&Self::ALL_NFTS_KEY)
            .unwrap_or_else(|| Vec::new(env));
        if let Some(idx) = all_nfts.first_index_of(nft_id) {
            all_nfts.remove(idx);
            env.storage().persistent().set(&Self::ALL_NFTS_KEY, &all_nfts);
        }
    }

    pub fn set_nft_version(env: &Env, nft_id: u64, version: u32) {
        let key = Self::nft_version_key(nft_id);
        env.storage().persistent().set(&key, &version);
    }

    /// Reads the metadata schema version for an NFT.
    /// Legacy NFTs (written before versioning existed) have no version key
    /// and are treated as version 1.
    pub fn get_nft_version(env: &Env, nft_id: u64) -> u32 {
        let key = Self::nft_version_key(nft_id);
        env.storage().persistent().get(&key).unwrap_or(1)
    }

    /// Returns true if an explicit version key exists for the given NFT.
    /// Used by migration to detect NFTs that still need a version assigned.
    pub fn has_nft_version_key(env: &Env, nft_id: u64) -> bool {
        let key = Self::nft_version_key(nft_id);
        env.storage().persistent().has(&key)
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

    pub fn get_nft_count_for_hunt(env: &Env, hunt_id: u64) -> u64 {
        let all_ids = Self::get_all_nft_ids(env);
        let mut count = 0u64;
        for nft_id in all_ids.iter() {
            if let Some(nft) = Self::get_nft(env, nft_id) {
                if nft.hunt_id == hunt_id {
                    count += 1;
                }
            }
        }
        count
    }

    pub fn mark_hunt_minted(env: &Env, hunt_id: u64) {
        let hunt_key = (symbol_short!("HMNT"), hunt_id);
        if !env.storage().persistent().has(&hunt_key) {
            env.storage().persistent().set(&hunt_key, &());
            let current_total: u64 = env
                .storage()
                .persistent()
                .get(&Self::TOTAL_HUNTS_KEY)
                .unwrap_or(0);
            env.storage()
                .persistent()
                .set(&Self::TOTAL_HUNTS_KEY, &(current_total + 1));
        }
    }

    pub fn get_total_hunts(env: &Env) -> u64 {
        env.storage()
            .persistent()
            .get(&Self::TOTAL_HUNTS_KEY)
            .unwrap_or(0)
    }

    pub fn get_total_owners(env: &Env) -> u64 {
        env.storage()
            .persistent()
            .get(&Self::TOTAL_OWNERS_KEY)
            .unwrap_or(0)
    }

    pub fn set_max_supply(env: &Env, max_supply: Option<u64>) {
        env.storage()
            .persistent()
            .set(&Self::MAX_SUPPLY_KEY, &max_supply);
        env.storage()
            .persistent()
            .set(&Self::INITIALIZED_KEY, &true);
    }

    pub fn get_max_supply(env: &Env) -> Option<u64> {
        env.storage()
            .persistent()
            .get::<_, Option<u64>>(&Self::MAX_SUPPLY_KEY)
            .flatten()
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

        if count == 0 {
            let current_total: u64 = env
                .storage()
                .persistent()
                .get(&Self::TOTAL_OWNERS_KEY)
                .unwrap_or(0);
            env.storage()
                .persistent()
                .set(&Self::TOTAL_OWNERS_KEY, &(current_total + 1));
        }
    }

    pub fn add_nft_to_hunt(env: &Env, hunt_id: u64, nft_id: u64) {
        let count_key = Self::hunt_nft_count_key(hunt_id);
        let count: u32 = env.storage().persistent().get(&count_key).unwrap_or(0);

        let exist_key = Self::hunt_nft_exist_key(hunt_id, nft_id);
        if env.storage().persistent().has(&exist_key) {
            return;
        }

        env.storage()
            .persistent()
            .set(&Self::hunt_nft_entry_key(hunt_id, count), &nft_id);
        env.storage().persistent().set(&count_key, &(count + 1));
        env.storage().persistent().set(&exist_key, &());
    }

    pub fn get_hunt_nft_count(env: &Env, hunt_id: u64) -> u32 {
        let count_key = Self::hunt_nft_count_key(hunt_id);
        env.storage().persistent().get(&count_key).unwrap_or(0)
    }

    pub fn get_hunt_nfts(env: &Env, hunt_id: u64, offset: u32, limit: u32) -> Vec<u64> {
        let count_key = Self::hunt_nft_count_key(hunt_id);
        let count: u32 = env.storage().persistent().get(&count_key).unwrap_or(0);
        if offset >= count {
            return Vec::new(env);
        }
        let end = offset.saturating_add(limit).min(count);
        let mut ids = Vec::new(env);
        for i in offset..end {
            let entry_key = Self::hunt_nft_entry_key(hunt_id, i);
            if let Some(id) = env.storage().persistent().get(&entry_key) {
                ids.push_back(id);
            }
        }
        ids
    }

    pub fn remove_nft_from_hunt(env: &Env, hunt_id: u64, nft_id: u64) {
        let count_key = Self::hunt_nft_count_key(hunt_id);
        let count: u32 = env.storage().persistent().get(&count_key).unwrap_or(0);
        let exist_key = Self::hunt_nft_exist_key(hunt_id, nft_id);
        if !env.storage().persistent().has(&exist_key) {
            return;
        }

        for i in 0..count {
            let entry_key = Self::hunt_nft_entry_key(hunt_id, i);
            if let Some(stored_id) = env.storage().persistent().get::<_, u64>(&entry_key) {
                if stored_id == nft_id {
                    let last_idx = count - 1;
                    if i != last_idx {
                        let last_key = Self::hunt_nft_entry_key(hunt_id, last_idx);
                        if let Some(last_id) = env.storage().persistent().get::<_, u64>(&last_key) {
                            env.storage().persistent().set(&entry_key, &last_id);
                        }
                        env.storage().persistent().remove(&last_key);
                    } else {
                        env.storage().persistent().remove(&entry_key);
                    }
                    env.storage().persistent().set(&count_key, &(count - 1));
                    env.storage().persistent().remove(&exist_key);
                    return;
                }
            }
        }
    }

    /// Returns all minted NFT IDs from the persisted all-NFTs index.
    pub fn get_all_nft_ids(env: &Env) -> Vec<u64> {
        env.storage()
            .persistent()
            .get(&Self::ALL_NFTS_KEY)
            .unwrap_or_else(|| Vec::new(env))
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

    // --- Locker management ---

    /// Adds an authorized locker contract. Admin only.
    pub fn add_locker(env: &Env, locker: &Address) {
        let key = Self::locker_key(locker);
        env.storage().persistent().set(&key, &true);
    }

    /// Removes an authorized locker contract. Admin only.
    pub fn remove_locker(env: &Env, locker: &Address) {
        let key = Self::locker_key(locker);
        env.storage().persistent().remove(&key);
    }

    /// Returns true if `locker` is an authorized locker contract.
    pub fn is_locker(env: &Env, locker: &Address) -> bool {
        let key = Self::locker_key(locker);
        env.storage().persistent().get(&key).unwrap_or(false)
    }

    // --- Contract version ---

    pub fn set_contract_version(env: &Env, version: u32) {
        env.storage()
            .instance()
            .set(&Self::CONTRACT_VERSION_KEY, &version);
    }

    pub fn get_contract_version(env: &Env) -> Option<u32> {
        env.storage().instance().get(&Self::CONTRACT_VERSION_KEY)
    }
}
