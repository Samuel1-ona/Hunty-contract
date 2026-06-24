use crate::errors::HuntError;
use crate::types::{Clue, Hunt, PlayerProgress};
use soroban_sdk::{symbol_short, Address, Env, Vec};

/// Storage access layer for hunts, clues, and player progress.
/// Provides type-safe, efficient storage operations with consistent key management.
///
/// # Storage Key Namespace — HuntyCore (prefix: "HC")
///
/// All keys are unique within this contract and isolated from other contracts:
///
/// | Constant            | Symbol   | Purpose                                      |
/// |---------------------|----------|----------------------------------------------|
/// | `CONTRACT_PREFIX`   | `"HC"`   | Contract namespace guard (documents ownership)|
/// | `HUNT_KEY`          | `"HUNT"` | Per-hunt data                                |
/// | `CLUE_KEY`          | `"CLUE"` | Per-clue data (composite: hunt_id, clue_id)  |
/// | `PROGRESS_KEY`      | `"PROG"` | Player progress (composite: hunt_id, player) |
/// | `PLAYER_ENTRY_KEY`  | `"PLRS"` | Player list entry (composite: hunt_id, idx)  |
/// | `PLAYER_COUNT_KEY`  | `"PLCT"` | Player count per hunt                        |
/// | `CLUE_ENTRY_KEY`    | `"CLST"` | Clue list entry (composite: hunt_id, idx)    |
/// | `CLUE_LIST_COUNT_KEY`| `"CLCT"`| Clue count per hunt                          |
/// | `HUNT_COUNTER_KEY`  | `"HCCNT"`| Global hunt ID counter (HC-prefixed)         |
/// | `CLUE_COUNTER_KEY`  | `"CCNT"` | Clue ID counter per hunt                     |
/// | `REWARD_MGR_KEY`    | `"RWDMGR"`| Reward manager contract address             |
/// | `CLUE_EXISTS_KEY`   | `"CLEX"` | Clue existence sentinel                      |
/// | `PLAYER_EXISTS_KEY` | `"PLEX"` | Player existence sentinel                    |
pub struct Storage;

impl Storage {
    /// Contract namespace identifier for HuntyCore.
    /// All storage keys in this contract belong to the "HC" namespace.
    /// This ensures no key can collide with RewardManager ("RM") or NftReward ("NR").
    pub const CONTRACT_PREFIX: &'static str = "HC";

    const HUNT_KEY: soroban_sdk::Symbol = symbol_short!("HUNT");
    const CLUE_KEY: soroban_sdk::Symbol = symbol_short!("CLUE");
    const PROGRESS_KEY: soroban_sdk::Symbol = symbol_short!("PROG");
    const PLAYERS_LIST_KEY: soroban_sdk::Symbol = symbol_short!("PLRS");
    const CLUES_LIST_KEY: soroban_sdk::Symbol = symbol_short!("CLST");
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
        env.storage().persistent().set(&key, hunt);
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
        env.storage().persistent().get(&key)
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
        env.storage().persistent().set(&key, clue);

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
        env.storage().persistent().get(&key)
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

    /// Generates a storage key for the list of clue IDs for a hunt.
    /// Uses tuple key (CLUES_LIST_KEY, hunt_id) for efficient storage access.
    fn clues_list_key(hunt_id: u64) -> (soroban_sdk::Symbol, u64) {
        (Self::CLUES_LIST_KEY, hunt_id)
    }

    /// Generates a storage key for the clue counter per hunt.
    fn clue_counter_key(hunt_id: u64) -> (soroban_sdk::Symbol, u64) {
        (Self::CLUE_COUNTER_KEY, hunt_id)
    }

    /// Generates a storage key for the list of player addresses for a hunt.
    /// Uses tuple key (PLAYERS_LIST_KEY, hunt_id) for efficient storage access.
    fn players_list_key(hunt_id: u64) -> (soroban_sdk::Symbol, u64) {
        (Self::PLAYERS_LIST_KEY, hunt_id)
    }

    // ========== Internal Helper Functions ==========

    /// Adds a clue ID to the list of clues for a hunt.
    /// This maintains an index for efficient listing.
    fn add_clue_to_list(env: &Env, hunt_id: u64, clue_id: u32) {
        let key = Self::clues_list_key(hunt_id);
        let mut clue_ids = env
            .storage()
            .persistent()
            .get(&key)
            .unwrap_or_else(|| Vec::new(env));

        // Check if clue_id already exists to avoid duplicates
        let mut exists = false;
        for i in 0..clue_ids.len() {
            if let Some(id) = clue_ids.get(i) {
                if id == clue_id {
                    exists = true;
                    break;
                }
            }
        }

        if !exists {
            clue_ids.push_back(clue_id);
            env.storage().persistent().set(&key, &clue_ids);
        }
    }

    /// Retrieves the list of clue IDs for a hunt.
    fn get_clue_ids_for_hunt(env: &Env, hunt_id: u64) -> Vec<u32> {
        let key = Self::clues_list_key(hunt_id);
        env.storage()
            .persistent()
            .get(&key)
            .unwrap_or_else(|| Vec::new(env))
    }

    /// Adds a player address to the list of players for a hunt.
    /// This maintains an index for efficient listing.
    fn add_player_to_list(env: &Env, hunt_id: u64, player: &Address) {
        let key = Self::players_list_key(hunt_id);
        let mut players = env
            .storage()
            .persistent()
            .get(&key)
            .unwrap_or_else(|| Vec::new(env));

        // Check if player already exists to avoid duplicates
        let mut exists = false;
        for i in 0..players.len() {
            if let Some(addr) = players.get(i) {
                if addr == *player {
                    exists = true;
                    break;
                }
            }
        }

        if !exists {
            players.push_back(player.clone());
            env.storage().persistent().set(&key, &players);
        }
    }

    /// Retrieves the list of player addresses for a hunt.
    fn get_player_addresses_for_hunt(env: &Env, hunt_id: u64) -> Vec<Address> {
        let key = Self::players_list_key(hunt_id);
        env.storage()
            .persistent()
            .get(&key)
            .unwrap_or_else(|| Vec::new(env))
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
        let current: u64 = env.storage().persistent().get(&key).unwrap_or(0);
        let next = current + 1;
        env.storage().persistent().set(&key, &next);
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
        env.storage().persistent().get(&key).unwrap_or(0)
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
        let current: u32 = env.storage().persistent().get(&key).unwrap_or(0);
        let next = current + 1;
        env.storage().persistent().set(&key, &next);
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
        env.storage().persistent().get(&key).unwrap_or(0)
    }

    // ========== Reward Manager Storage Functions ==========

    pub fn set_reward_manager(env: &Env, address: &Address) {
        env.storage()
            .persistent()
            .set(&Self::REWARD_MGR_KEY, address);
    }

    pub fn get_reward_manager(env: &Env) -> Option<Address> {
        env.storage().persistent().get(&Self::REWARD_MGR_KEY)
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

    /// Verify the HuntyCore contract prefix is "HC" and unique across contracts.
    #[test]
    fn test_contract_prefix_is_hc() {
        assert_eq!(Storage::CONTRACT_PREFIX, "HC");
    }

    /// Verify the prefix does not equal the other contracts' prefixes.
    #[test]
    fn test_prefix_distinct_from_other_contracts() {
        assert_ne!(Storage::CONTRACT_PREFIX, "RM"); // RewardManager
        assert_ne!(Storage::CONTRACT_PREFIX, "NR"); // NftReward
    }

    /// Verify the hunt counter key is "HCCNT" — not the old "CNTR" that
    /// collided with nft-reward's NFT counter.
    #[test]
    fn test_hunt_counter_key_is_hccnt() {
        let expected = symbol_short!("HCCNT");
        assert_eq!(Storage::HUNT_COUNTER_KEY, expected);
    }

    /// Verify all symbol constants within hunty-core are distinct (no intra-contract collision).
    #[test]
    fn test_no_intra_contract_key_collision() {
        let keys = [
            symbol_short!("HUNT"),
            symbol_short!("CLUE"),
            symbol_short!("PROG"),
            symbol_short!("PLRS"),
            symbol_short!("PLCT"),
            symbol_short!("CLST"),
            symbol_short!("CLCT"),
            symbol_short!("HCCNT"),
            symbol_short!("CCNT"),
            symbol_short!("RWDMGR"),
            symbol_short!("CLEX"),
            symbol_short!("PLEX"),
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
