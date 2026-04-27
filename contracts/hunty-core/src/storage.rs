use crate::errors::HuntError;
use crate::types::{Clue, Hunt, PlayerProgress};
use soroban_sdk::{symbol_short, Address, Env, Vec};

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

    // ========== Hunt Storage Functions ==========

    /// Saves a Hunt struct with a unique key based on hunt_id.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `hunt` - The Hunt struct to store
    ///
    /// # Panics
    /// Panics if storage operation fails
    pub fn save_hunt(env: &Env, hunt: &Hunt) {
        let key = Self::hunt_key(hunt.hunt_id);
        env.storage().instance().set(&key, hunt);
    }

    /// Retrieves a hunt by ID, returning an Option.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `hunt_id` - The unique identifier of the hunt
    ///
    /// # Returns
    /// * `Some(Hunt)` if the hunt exists, `None` otherwise
    pub fn get_hunt(env: &Env, hunt_id: u64) -> Option<Hunt> {
        let key = Self::hunt_key(hunt_id);
        env.storage().instance().get(&key)
    }

    /// Retrieves a hunt by ID or returns an error if not found.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `hunt_id` - The unique identifier of the hunt
    ///
    /// # Returns
    /// * `Ok(Hunt)` if the hunt exists
    /// * `Err(HuntError)` if the hunt is not found
    pub fn get_hunt_or_error(env: &Env, hunt_id: u64) -> Result<Hunt, HuntError> {
        Self::get_hunt(env, hunt_id).ok_or(HuntError::HuntNotFound { hunt_id })
    }

    // ========== Clue Storage Functions ==========

    /// Stores a clue using composite keys (hunt_id + clue_id).
    /// Also maintains a list of clue IDs for the hunt.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `hunt_id` - The hunt this clue belongs to
    /// * `clue` - The Clue struct to store
    pub fn save_clue(env: &Env, hunt_id: u64, clue: &Clue) {
        // Store the clue with composite key
        let key = Self::clue_key(hunt_id, clue.clue_id);
        env.storage().instance().set(&key, clue);

        // Update the list of clue IDs for this hunt
        Self::add_clue_to_list(env, hunt_id, clue.clue_id);
    }

    /// Retrieves an individual clue by hunt_id and clue_id.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `hunt_id` - The hunt this clue belongs to
    /// * `clue_id` - The unique identifier of the clue within the hunt
    ///
    /// # Returns
    /// * `Some(Clue)` if the clue exists, `None` otherwise
    pub fn get_clue(env: &Env, hunt_id: u64, clue_id: u32) -> Option<Clue> {
        let key = Self::clue_key(hunt_id, clue_id);
        env.storage().instance().get(&key)
    }

    /// Retrieves a clue or returns an error if not found.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `hunt_id` - The hunt this clue belongs to
    /// * `clue_id` - The unique identifier of the clue within the hunt
    ///
    /// # Returns
    /// * `Ok(Clue)` if the clue exists
    /// * `Err(HuntError)` if the clue is not found
    pub fn get_clue_or_error(env: &Env, hunt_id: u64, clue_id: u32) -> Result<Clue, HuntError> {
        Self::get_clue(env, hunt_id, clue_id).ok_or(HuntError::ClueNotFound { hunt_id })
    }

    /// Returns all clues for a specific hunt.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `hunt_id` - The hunt to get clues for
    ///
    /// # Returns
    /// A Vec containing all Clue structs for the hunt, in clue_id order
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

    /// Stores player state/progress for a hunt.
    /// Also maintains a list of registered players for the hunt.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `progress` - The PlayerProgress struct to store
    pub fn save_player_progress(env: &Env, progress: &PlayerProgress) {
        // Store the progress with composite key (hunt_id + player address)
        let key = Self::progress_key(progress.hunt_id, &progress.player);
        env.storage().persistent().set(&key, progress);

        // Update the list of players for this hunt
        Self::add_player_to_list(env, progress.hunt_id, &progress.player);
    }

    /// Retrieves player progress for a specific hunt and player.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `hunt_id` - The hunt the player is registered for
    /// * `player` - The player's address
    ///
    /// # Returns
    /// * `Some(PlayerProgress)` if progress exists, `None` otherwise
    pub fn get_player_progress(
        env: &Env,
        hunt_id: u64,
        player: &Address,
    ) -> Option<PlayerProgress> {
        let key = Self::progress_key(hunt_id, player);
        env.storage().persistent().get(&key)
    }

    /// Retrieves player progress or returns an error if not found.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `hunt_id` - The hunt the player is registered for
    /// * `player` - The player's address
    ///
    /// # Returns
    /// * `Ok(PlayerProgress)` if progress exists
    /// * `Err(HuntError)` if the player is not registered
    pub fn get_player_progress_or_error(
        env: &Env,
        hunt_id: u64,
        player: &Address,
    ) -> Result<PlayerProgress, HuntError> {
        Self::get_player_progress(env, hunt_id, player)
            .ok_or(HuntError::PlayerNotRegistered { hunt_id })
    }

    /// Returns all registered players for a hunt.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `hunt_id` - The hunt to get players for
    ///
    /// # Returns
    /// A Vec containing all PlayerProgress structs for the hunt
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

    /// Generates a storage key for a hunt using a symbol prefix and hunt_id.
    /// Uses tuple key (HUNT_KEY, hunt_id) for efficient storage access.
    fn hunt_key(hunt_id: u64) -> (soroban_sdk::Symbol, u64) {
        (Self::HUNT_KEY, hunt_id)
    }

    /// Generates a composite storage key for a clue.
    /// Uses tuple key (CLUE_KEY, hunt_id, clue_id) for efficient storage access.
    fn clue_key(hunt_id: u64, clue_id: u32) -> (soroban_sdk::Symbol, u64, u32) {
        (Self::CLUE_KEY, hunt_id, clue_id)
    }

    /// Generates a composite storage key for player progress.
    /// Uses tuple key (PROGRESS_KEY, hunt_id, player) for efficient storage access.
    fn progress_key(hunt_id: u64, player: &Address) -> (soroban_sdk::Symbol, u64, Address) {
        (Self::PROGRESS_KEY, hunt_id, player.clone())
    }

    /// Key for a single clue-list entry: (CLST, hunt_id, index)
    fn clue_entry_key(hunt_id: u64, index: u32) -> (soroban_sdk::Symbol, u64, u32) {
        (Self::CLUE_ENTRY_KEY, hunt_id, index)
    }

    /// Key for the number of entries in the clue list for a hunt.
    fn clue_list_count_key(hunt_id: u64) -> (soroban_sdk::Symbol, u64) {
        (Self::CLUE_LIST_COUNT_KEY, hunt_id)
    }

    /// Generates a storage key for the clue counter per hunt.
    fn clue_counter_key(hunt_id: u64) -> (soroban_sdk::Symbol, u64) {
        (Self::CLUE_COUNTER_KEY, hunt_id)
    }

    /// Key for a single player-list entry: (PLRS, hunt_id, index)
    fn player_entry_key(hunt_id: u64, index: u32) -> (soroban_sdk::Symbol, u64, u32) {
        (Self::PLAYER_ENTRY_KEY, hunt_id, index)
    }

    /// Key for the number of entries in the player list for a hunt.
    fn player_count_key(hunt_id: u64) -> (soroban_sdk::Symbol, u64) {
        (Self::PLAYER_COUNT_KEY, hunt_id)
    }

    // ========== Internal Helper Functions ==========

    /// Adds a clue ID to the per-hunt clue index.
    /// Each entry is stored at its own key so no single entry grows unboundedly.
    fn add_clue_to_list(env: &Env, hunt_id: u64, clue_id: u32) {
        let count_key = Self::clue_list_count_key(hunt_id);
        let count: u32 = env.storage().instance().get(&count_key).unwrap_or(0);

        // O(1) existence check
        let exist_key = (symbol_short!("CLEX"), hunt_id, clue_id);
        if env.storage().instance().has(&exist_key) {
            return;
        }

        env.storage().instance().set(&Self::clue_entry_key(hunt_id, count), &clue_id);
        env.storage().instance().set(&count_key, &(count + 1));
        env.storage().instance().set(&exist_key, &());
    }

    /// Retrieves all clue IDs for a hunt by reading individual entries.
    fn get_clue_ids_for_hunt(env: &Env, hunt_id: u64) -> Vec<u32> {
        let count_key = Self::clue_list_count_key(hunt_id);
        let count: u32 = env.storage().instance().get(&count_key).unwrap_or(0);
        let mut ids = Vec::new(env);
        for i in 0..count {
            let entry_key = Self::clue_entry_key(hunt_id, i);
            if let Some(id) = env.storage().instance().get(&entry_key) {
                ids.push_back(id);
            }
        }
        ids
    }

    /// Adds a player address to the per-hunt player index.
    /// Each entry is stored at its own key so no single entry grows unboundedly.
    fn add_player_to_list(env: &Env, hunt_id: u64, player: &Address) {
        let count_key = Self::player_count_key(hunt_id);
        let count: u32 = env.storage().persistent().get(&count_key).unwrap_or(0);

        // O(1) existence check
        let exist_key = (symbol_short!("PLEX"), hunt_id, player.clone());
        if env.storage().persistent().has(&exist_key) {
            return;
        }

        env.storage().persistent().set(&Self::player_entry_key(hunt_id, count), player);
        env.storage().persistent().set(&count_key, &(count + 1));
        env.storage().persistent().set(&exist_key, &());
    }

    /// Retrieves all player addresses for a hunt by reading individual entries.
    fn get_player_addresses_for_hunt(env: &Env, hunt_id: u64) -> Vec<Address> {
        let count_key = Self::player_count_key(hunt_id);
        let count: u32 = env.storage().persistent().get(&count_key).unwrap_or(0);
        let mut addrs = Vec::new(env);
        for i in 0..count {
            let entry_key = Self::player_entry_key(hunt_id, i);
            if let Some(addr) = env.storage().persistent().get(&entry_key) {
                addrs.push_back(addr);
            }
        }
        addrs
    }

    // ========== Hunt Counter Functions ==========

    /// Increments and returns the next hunt ID.
    /// This ensures unique, sequential hunt IDs.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    ///
    /// # Returns
    /// The next available hunt ID (starting from 1)
    pub fn next_hunt_id(env: &Env) -> u64 {
        let key = Self::HUNT_COUNTER_KEY;
        let current: u64 = env.storage().instance().get(&key).unwrap_or(0);
        let next = current + 1;
        env.storage().instance().set(&key, &next);
        next
    }

    /// Gets the current hunt counter value without incrementing.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    ///
    /// # Returns
    /// The current hunt counter value (0 if no hunts have been created)
    pub fn get_hunt_counter(env: &Env) -> u64 {
        let key = Self::HUNT_COUNTER_KEY;
        env.storage().instance().get(&key).unwrap_or(0)
    }

    // ========== Clue Counter (per hunt) Functions ==========

    /// Increments and returns the next clue ID for a hunt.
    /// Clue IDs are sequential within each hunt, starting from 1.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `hunt_id` - The hunt to allocate a clue ID for
    ///
    /// # Returns
    /// The next available clue ID for the hunt
    pub fn next_clue_id(env: &Env, hunt_id: u64) -> u32 {
        let key = Self::clue_counter_key(hunt_id);
        let current: u32 = env.storage().instance().get(&key).unwrap_or(0);
        let next = current + 1;
        env.storage().instance().set(&key, &next);
        next
    }

    /// Gets the current clue counter for a hunt without incrementing.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `hunt_id` - The hunt to get the clue count for
    ///
    /// # Returns
    /// The number of clues added so far for the hunt (0 if none)
    pub fn get_clue_counter(env: &Env, hunt_id: u64) -> u32 {
        let key = Self::clue_counter_key(hunt_id);
        env.storage().instance().get(&key).unwrap_or(0)
    }

    // ========== Reward Manager Storage Functions ==========

    pub fn set_reward_manager(env: &Env, address: &Address) {
        env.storage()
            .instance()
            .set(&Self::REWARD_MGR_KEY, address);
    }

    pub fn get_reward_manager(env: &Env) -> Option<Address> {
        env.storage().instance().get(&Self::REWARD_MGR_KEY)
    }
}
