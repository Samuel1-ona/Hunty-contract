use crate::errors::HuntError;
use crate::types::{Clue, Hunt, PlayerProgress, StoredPlayerProgress};
use soroban_sdk::{symbol_short, Address, Env, Vec};

// ~30 days at 5s/ledger. Instance TTL is bumped on every write — one call covers all keys.
const INSTANCE_TTL_THRESHOLD: u32 = 518_400;
const INSTANCE_TTL_EXTEND_TO: u32 = 518_400;

// Persistent entries (player progress + index) live ~90 days; bumped when threshold < 30 days.
const PERSISTENT_TTL_THRESHOLD: u32 = 518_400;
const PERSISTENT_TTL_EXTEND_TO: u32 = 1_555_200;

/// Storage access layer for hunts, clues, and player progress.
/// Provides type-safe, efficient storage operations with consistent key management.
pub struct Storage;

impl Storage {
    // Symbol constants for key prefixes to prevent collisions
    // Using symbol_short for efficient key generation
    const HUNT_KEY: soroban_sdk::Symbol = symbol_short!("HUNT");
    const CLUE_KEY: soroban_sdk::Symbol = symbol_short!("CLUE");
    const PROGRESS_KEY: soroban_sdk::Symbol = symbol_short!("PROG");
    const PLAYER_ENTRY_KEY: soroban_sdk::Symbol = symbol_short!("PLRS");
    const PLAYER_COUNT_KEY: soroban_sdk::Symbol = symbol_short!("PLCT");
    const CLUE_ENTRY_KEY: soroban_sdk::Symbol = symbol_short!("CLST");
    const CLUE_LIST_COUNT_KEY: soroban_sdk::Symbol = symbol_short!("CLCT");
    const HUNT_COUNTER_KEY: soroban_sdk::Symbol = symbol_short!("CNTR");
    const CLUE_COUNTER_KEY: soroban_sdk::Symbol = symbol_short!("CCNT");
    const REWARD_MGR_KEY: soroban_sdk::Symbol = symbol_short!("RWDMGR");
    const ADMIN_KEY: soroban_sdk::Symbol = symbol_short!("ADMIN");
    const PAUSED_KEY: soroban_sdk::Symbol = symbol_short!("PAUSED");

    // ========== Hunt Storage Functions ==========

    pub fn save_hunt(env: &Env, hunt: &Hunt) {
        let key = Self::hunt_key(hunt.hunt_id);
        env.storage().instance().set(&key, hunt);
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_TTL_THRESHOLD, INSTANCE_TTL_EXTEND_TO);
    }

    pub fn get_hunt(env: &Env, hunt_id: u64) -> Option<Hunt> {
        let key = Self::hunt_key(hunt_id);
        env.storage().instance().get(&key)
    }

    pub fn get_hunt_or_error(env: &Env, hunt_id: u64) -> Result<Hunt, HuntError> {
        Self::get_hunt(env, hunt_id).ok_or(HuntError::HuntNotFound { hunt_id })
    }

    // ========== Clue Storage Functions ==========

    pub fn save_clue(env: &Env, hunt_id: u64, clue: &Clue) {
        let key = Self::clue_key(hunt_id, clue.clue_id);
        env.storage().instance().set(&key, clue);
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_TTL_THRESHOLD, INSTANCE_TTL_EXTEND_TO);
        Self::add_clue_to_list(env, hunt_id, clue.clue_id);
    }

    pub fn get_clue(env: &Env, hunt_id: u64, clue_id: u32) -> Option<Clue> {
        let key = Self::clue_key(hunt_id, clue_id);
        env.storage().instance().get(&key)
    }

    pub fn get_clue_or_error(env: &Env, hunt_id: u64, clue_id: u32) -> Result<Clue, HuntError> {
        Self::get_clue(env, hunt_id, clue_id).ok_or(HuntError::ClueNotFound { hunt_id })
    }

    pub fn list_clues_for_hunt(env: &Env, hunt_id: u64) -> Vec<Clue> {
        let clue_ids = Self::get_clue_ids_for_hunt(env, hunt_id);
        let mut clues = Vec::new(env);

        for i in 0..clue_ids.len() {
            if let Some(clue_id) = clue_ids.get(i) {
                if let Some(clue) = Self::get_clue(env, hunt_id, clue_id) {
                    clues.push_back(clue);
                }
            }
        }

        clues
    }

    // ========== Player Progress Storage Functions ==========

    pub fn save_player_progress(env: &Env, progress: &PlayerProgress) {
        let key = Self::progress_key(progress.hunt_id, &progress.player);
        env.storage().persistent().set(&key, &progress.to_stored());
        env.storage()
            .persistent()
            .extend_ttl(&key, PERSISTENT_TTL_THRESHOLD, PERSISTENT_TTL_EXTEND_TO);
        Self::add_player_to_list(env, progress.hunt_id, &progress.player);
    }

    pub fn get_player_progress(
        env: &Env,
        hunt_id: u64,
        player: &Address,
    ) -> Option<PlayerProgress> {
        let key = Self::progress_key(hunt_id, player);
        let result = env
            .storage()
            .persistent()
            .get::<_, StoredPlayerProgress>(&key)
            .map(|stored| PlayerProgress::from_stored(env, stored, player.clone(), hunt_id))
    }

    pub fn get_player_progress_or_error(
        env: &Env,
        hunt_id: u64,
        player: &Address,
    ) -> Result<PlayerProgress, HuntError> {
        Self::get_player_progress(env, hunt_id, player)
            .ok_or(HuntError::PlayerNotRegistered { hunt_id })
    }

    pub fn get_hunt_players(env: &Env, hunt_id: u64) -> Vec<PlayerProgress> {
        let player_addresses = Self::get_player_addresses_for_hunt(env, hunt_id);
        let mut progress_list = Vec::new(env);

        for i in 0..player_addresses.len() {
            if let Some(player) = player_addresses.get(i) {
                if let Some(progress) = Self::get_player_progress(env, hunt_id, &player) {
                    progress_list.push_back(progress);
                }
            }
        }

        progress_list
    }

    // ========== Helper Functions for Key Generation ==========

    fn hunt_key(hunt_id: u64) -> (soroban_sdk::Symbol, u64) {
        (Self::HUNT_KEY, hunt_id)
    }

    fn clue_key(hunt_id: u64, clue_id: u32) -> (soroban_sdk::Symbol, u64, u32) {
        (Self::CLUE_KEY, hunt_id, clue_id)
    }

    fn progress_key(hunt_id: u64, player: &Address) -> (soroban_sdk::Symbol, u64, Address) {
        (Self::PROGRESS_KEY, hunt_id, player.clone())
    }

    fn clue_entry_key(hunt_id: u64, index: u32) -> (soroban_sdk::Symbol, u64, u32) {
        (Self::CLUE_ENTRY_KEY, hunt_id, index)
    }

    fn clue_list_count_key(hunt_id: u64) -> (soroban_sdk::Symbol, u64) {
        (Self::CLUE_LIST_COUNT_KEY, hunt_id)
    }

    fn clue_counter_key(hunt_id: u64) -> (soroban_sdk::Symbol, u64) {
        (Self::CLUE_COUNTER_KEY, hunt_id)
    }

    fn player_entry_key(hunt_id: u64, index: u32) -> (soroban_sdk::Symbol, u64, u32) {
        (Self::PLAYER_ENTRY_KEY, hunt_id, index)
    }

    fn player_count_key(hunt_id: u64) -> (soroban_sdk::Symbol, u64) {
        (Self::PLAYER_COUNT_KEY, hunt_id)
    }

    // ========== Internal Helper Functions ==========

    fn add_clue_to_list(env: &Env, hunt_id: u64, clue_id: u32) {
        let count_key = Self::clue_list_count_key(hunt_id);
        let count: u32 = env.storage().instance().get(&count_key).unwrap_or(0);

        // O(1) existence check
        let exist_key = Self::clue_exists_key(hunt_id, clue_id);
        if env.storage().instance().has(&exist_key) {
            return;
        }

        env.storage()
            .instance()
            .set(&Self::clue_entry_key(hunt_id, count), &clue_id);
        env.storage().instance().set(&count_key, &(count + 1));
        env.storage().instance().set(&exist_key, &());
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_TTL_THRESHOLD, INSTANCE_TTL_EXTEND_TO);
    }

    fn get_clue_ids_for_hunt(env: &Env, hunt_id: u64) -> Vec<u32> {
        let count_key = Self::clue_list_count_key(hunt_id);
        let count: u32 = env.storage().instance().get(&count_key).unwrap_or(0);
        let mut ids = Vec::new(env);
        let start = offset;
        let end = core::cmp::min(offset.saturating_add(limit), count);
        if start >= count {
            return ids;
        }
        for i in start..end {
            let entry_key = Self::clue_entry_key(hunt_id, i);
            if let Some(id) = env.storage().instance().get(&entry_key) {
                ids.push_back(id);
            }
        }
        ids
    }

    fn add_player_to_list(env: &Env, hunt_id: u64, player: &Address) {
        let count_key = Self::player_count_key(hunt_id);
        let count: u32 = env.storage().persistent().get(&count_key).unwrap_or(0);

        // O(1) existence check
        let exist_key = (symbol_short!("PLEX"), hunt_id, player.clone());
        if env.storage().persistent().has(&exist_key) {
            // Bump the exist marker so this player's slot never silently expires
            env.storage().persistent().extend_ttl(
                &exist_key,
                PERSISTENT_TTL_THRESHOLD,
                PERSISTENT_TTL_EXTEND_TO,
            );
            return;
        }

        env.storage()
            .persistent()
            .set(&Self::player_entry_key(hunt_id, count), player);
        let entry_key = Self::player_entry_key(hunt_id, count);
        env.storage().persistent().set(&entry_key, player);
        env.storage().persistent().extend_ttl(
            &entry_key,
            PERSISTENT_TTL_THRESHOLD,
            PERSISTENT_TTL_EXTEND_TO,
        );
        env.storage().persistent().set(&count_key, &(count + 1));
        env.storage().persistent().extend_ttl(
            &count_key,
            PERSISTENT_TTL_THRESHOLD,
            PERSISTENT_TTL_EXTEND_TO,
        );
        env.storage().persistent().set(&exist_key, &());
        env.storage().persistent().extend_ttl(
            &exist_key,
            PERSISTENT_TTL_THRESHOLD,
            PERSISTENT_TTL_EXTEND_TO,
        );
    }

    fn get_player_addresses_for_hunt(env: &Env, hunt_id: u64) -> Vec<Address> {
        let count_key = Self::player_count_key(hunt_id);
        let count: u32 = env.storage().persistent().get(&count_key).unwrap_or(0);
        let mut addrs = Vec::new(env);
        for i in 0..count {
            let entry_key = Self::player_entry_key(hunt_id, i);
            if let Some(addr) = env.storage().persistent().get::<_, Address>(&entry_key) {
                env.storage().persistent().extend_ttl(
                    &entry_key,
                    PERSISTENT_TTL_THRESHOLD,
                    PERSISTENT_TTL_EXTEND_TO,
                );
                addrs.push_back(addr);
            }
        }
        addrs
    }

    fn clue_exists_key(hunt_id: u64, clue_id: u32) -> (soroban_sdk::Symbol, u64, u32) {
        (symbol_short!("CLEX"), hunt_id, clue_id)
    }

    // ========== Hunt Counter Functions ==========

    pub fn next_hunt_id(env: &Env) -> u64 {
        let key = Self::HUNT_COUNTER_KEY;
        let current: u64 = env.storage().instance().get(&key).unwrap_or(0);
        let next = current + 1;
        env.storage().instance().set(&key, &next);
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_TTL_THRESHOLD, INSTANCE_TTL_EXTEND_TO);
        next
    }

    pub fn get_hunt_counter(env: &Env) -> u64 {
        let key = Self::HUNT_COUNTER_KEY;
        env.storage().instance().get(&key).unwrap_or(0)
    }

    // ========== Clue Counter (per hunt) Functions ==========

    pub fn next_clue_id(env: &Env, hunt_id: u64) -> u32 {
        let key = Self::clue_counter_key(hunt_id);
        let current: u32 = env.storage().instance().get(&key).unwrap_or(0);
        let next = current + 1;
        env.storage().instance().set(&key, &next);
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_TTL_THRESHOLD, INSTANCE_TTL_EXTEND_TO);
        next
    }

    /// Gets the current number of indexed clues for a hunt.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `hunt_id` - The hunt to get the clue count for
    ///
    /// # Returns
    /// The number of clues currently stored for the hunt (0 if none)
    pub fn get_clue_counter(env: &Env, hunt_id: u64) -> u32 {
        let key = Self::clue_list_count_key(hunt_id);
        env.storage().instance().get(&key).unwrap_or(0)
    }

    // ========== Reward Manager Storage Functions ==========

    pub fn set_reward_manager(env: &Env, address: &Address) {
        env.storage().instance().set(&Self::REWARD_MGR_KEY, address);
        env.storage()
            .instance()
            .set(&Self::REWARD_MGR_KEY, address);
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_TTL_THRESHOLD, INSTANCE_TTL_EXTEND_TO);
    }

    pub fn get_reward_manager(env: &Env) -> Option<Address> {
        env.storage().instance().get(&Self::REWARD_MGR_KEY)
    }

    // ========== Contract Admin / Pause Storage Functions ==========

    pub fn set_admin(env: &Env, admin: &Address) {
        env.storage().instance().set(&Self::ADMIN_KEY, admin);
    }

    pub fn get_admin(env: &Env) -> Option<Address> {
        env.storage().instance().get(&Self::ADMIN_KEY)
    }

    pub fn set_contract_paused(env: &Env, paused: bool) {
        env.storage().instance().set(&Self::PAUSED_KEY, &paused);
    }

    pub fn is_contract_paused(env: &Env) -> bool {
        env.storage()
            .instance()
            .get(&Self::PAUSED_KEY)
            .unwrap_or(false)
    }
}
