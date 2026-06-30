use crate::errors::HuntError;
use crate::types::{Clue, Hunt, HuntCache, LeaderboardIndexEntry, PlayerProgress};
use soroban_sdk::{symbol_short, Address, Env, IntoVal, Map, Vec};

// Instance TTL constants used by blacklist and contract-pause storage.
const INSTANCE_TTL_THRESHOLD: u32 = 518_400;
const INSTANCE_TTL_EXTEND_TO: u32 = 518_400;

/// Storage access layer for hunts, clues, and player progress.
/// Provides type-safe, efficient storage operations with consistent key management.
pub struct Storage;

// ========== TTL Constants ==========
/// Default TTL threshold (ledgers) - extend when below this.
const TTL_THRESHOLD_CRITICAL: u32 = 50_000;
/// Default TTL extend to (ledgers) - for admin/config data.
const TTL_EXTEND_CRITICAL: u32 = 500_000;
/// TTL threshold for active hunt data (ledgers).
const TTL_THRESHOLD_ACTIVE: u32 = 30_000;
/// TTL extend to for active hunt data (ledgers) - ~30 days at 5s/ledger.
const TTL_EXTEND_ACTIVE: u32 = 300_000;
/// TTL threshold for default data (ledgers).
const TTL_THRESHOLD_DEFAULT: u32 = 10_000;
/// TTL extend to for default data (ledgers) - ~7 days.
const TTL_EXTEND_DEFAULT: u32 = 100_000;
/// TTL threshold for completed/archived data (ledgers).
const TTL_THRESHOLD_SHORT: u32 = 5_000;
/// TTL extend to for completed/archived data (ledgers) - ~3 days.
const TTL_EXTEND_SHORT: u32 = 50_000;

/// TTL policy categories for different data types.
pub enum TtlPolicy {
    Critical,
    Active,
    Default,
    Short,
}

/// Extends TTL for a storage key based on the given policy.
pub fn extend_ttl<K: IntoVal<Env, soroban_sdk::Val>>(env: &Env, key: &K, policy: TtlPolicy) {
    let (threshold, extend_to) = match policy {
        TtlPolicy::Critical => (TTL_THRESHOLD_CRITICAL, TTL_EXTEND_CRITICAL),
        TtlPolicy::Active => (TTL_THRESHOLD_ACTIVE, TTL_EXTEND_ACTIVE),
        TtlPolicy::Default => (TTL_THRESHOLD_DEFAULT, TTL_EXTEND_DEFAULT),
        TtlPolicy::Short => (TTL_THRESHOLD_SHORT, TTL_EXTEND_SHORT),
    };
    env.storage()
        .persistent()
        .extend_ttl(key, threshold, extend_to);
}

impl Storage {
    // Symbol constants for key prefixes to prevent collisions
    // Using symbol_short for efficient key generation
    // Shortened, unique storage key prefixes (reduced to minimal unique prefixes)
    const HUNT_KEY: soroban_sdk::Symbol = symbol_short!("HUNT");
    const CLUE_KEY: soroban_sdk::Symbol = symbol_short!("CLU");
    const PROGRESS_KEY: soroban_sdk::Symbol = symbol_short!("PR");
    const PLAYERS_LIST_KEY: soroban_sdk::Symbol = symbol_short!("PL");
    const LEADERBOARD_KEY: soroban_sdk::Symbol = symbol_short!("LBD");
    const CLUES_LIST_KEY: soroban_sdk::Symbol = symbol_short!("CLS");
    const HUNT_COUNTER_KEY: soroban_sdk::Symbol = symbol_short!("CN");
    const CLUE_COUNTER_KEY: soroban_sdk::Symbol = symbol_short!("CC");
    const REQUIRED_CLUES_KEY: soroban_sdk::Symbol = symbol_short!("REQ");
    const REWARD_MGR_KEY: soroban_sdk::Symbol = symbol_short!("R");
    const BAN_KEY: soroban_sdk::Symbol = symbol_short!("BA");
    const SUBMISSION_KEY: soroban_sdk::Symbol = symbol_short!("S");
    const ADMIN_KEY: soroban_sdk::Symbol = symbol_short!("AD");
    const VIEW_ONLY_KEY: soroban_sdk::Symbol = symbol_short!("V");
    const GLOBAL_VIEW_ONLY_KEY: soroban_sdk::Symbol = symbol_short!("GV");
    const PAUSE_REGISTRATIONS_KEY: soroban_sdk::Symbol = symbol_short!("PAUSE_RE");
    const PAUSE_ANSWERS_KEY: soroban_sdk::Symbol = symbol_short!("PAUSE_A");
    const PAUSE_REWARDS_KEY: soroban_sdk::Symbol = symbol_short!("PAUSE_RW");
    const CONTRACT_PAUSED_KEY: soroban_sdk::Symbol = symbol_short!("CPAUSED");
    const BLACKLIST_KEY: soroban_sdk::Symbol = symbol_short!("BLKLST");
    const HUNT_CACHE_KEY: soroban_sdk::Symbol = symbol_short!("HC");
    const CACHE_HIT_KEY: soroban_sdk::Symbol = symbol_short!("CHIT");
    const CACHE_MISS_KEY: soroban_sdk::Symbol = symbol_short!("CMISS");

    // ========== Cache Monitoring ==========

    /// Records a cache hit (cache was present and returned).
    pub fn record_cache_hit(env: &Env) {
        let count: u64 = env.storage().instance().get(&Self::CACHE_HIT_KEY).unwrap_or(0);
        env.storage().instance().set(&Self::CACHE_HIT_KEY, &(count + 1));
    }

    /// Records a cache miss (cache was absent, fallback to persistent).
    pub fn record_cache_miss(env: &Env) {
        let count: u64 = env.storage().instance().get(&Self::CACHE_MISS_KEY).unwrap_or(0);
        env.storage().instance().set(&Self::CACHE_MISS_KEY, &(count + 1));
    }

    /// Returns the total cache hits across all hunts.
    pub fn get_cache_hits(env: &Env) -> u64 {
        env.storage().instance().get(&Self::CACHE_HIT_KEY).unwrap_or(0)
    }

    /// Returns the total cache misses across all hunts.
    pub fn get_cache_misses(env: &Env) -> u64 {
        env.storage().instance().get(&Self::CACHE_MISS_KEY).unwrap_or(0)
    }

    /// Returns cache hit rate as basis points (0-10000).
    pub fn get_cache_hit_rate_bps(env: &Env) -> u32 {
        let hits: u64 = Self::get_cache_hits(env);
        let misses: u64 = Self::get_cache_misses(env);
        let total = hits.saturating_add(misses);
        if total == 0 {
            return 0;
        }
        ((hits.saturating_mul(10000)).checked_div(total).unwrap_or(0)) as u32
    }

    // Pause functions (granular: registrations, answers, rewards)
    pub fn set_pause_registrations(env: &Env, paused: bool) {
        env.storage().instance().set(&Self::PAUSE_REGISTRATIONS_KEY, &paused);
    }
    pub fn is_pause_registrations(env: &Env) -> bool {
        env.storage().instance().get(&Self::PAUSE_REGISTRATIONS_KEY).unwrap_or(false)
    }

    pub fn set_pause_answers(env: &Env, paused: bool) {
        env.storage().instance().set(&Self::PAUSE_ANSWERS_KEY, &paused);
    }
    pub fn is_pause_answers(env: &Env) -> bool {
        env.storage().instance().get(&Self::PAUSE_ANSWERS_KEY).unwrap_or(false)
    }

    pub fn set_pause_rewards(env: &Env, paused: bool) {
        env.storage().instance().set(&Self::PAUSE_REWARDS_KEY, &paused);
    }
    pub fn is_pause_rewards(env: &Env) -> bool {
        env.storage().instance().get(&Self::PAUSE_REWARDS_KEY).unwrap_or(false)
    }

    // Global contract pause (emergency stop for all operations)
    pub fn set_contract_paused(env: &Env, paused: bool) {
        env.storage().instance().set(&Self::CONTRACT_PAUSED_KEY, &paused);
        env.storage().instance().extend_ttl(INSTANCE_TTL_THRESHOLD, INSTANCE_TTL_EXTEND_TO);
    }
    pub fn is_contract_paused(env: &Env) -> bool {
        env.storage().instance().get(&Self::CONTRACT_PAUSED_KEY).unwrap_or(false)
    }

    // ========== Hunt Storage Functions ==========

    /// Saves a Hunt struct with a unique key based on hunt_id.
    /// Also automatically saves/refreshes the instance-storage cache
    /// so that subsequent reads can use the cheaper HuntCache path.
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
        let policy = match hunt.status {
            crate::types::HuntStatus::Active => TtlPolicy::Active,
            crate::types::HuntStatus::Completed | crate::types::HuntStatus::Cancelled => {
                TtlPolicy::Short
            }
            _ => TtlPolicy::Default,
        };
        extend_ttl(env, &key, policy);
        // Keep instance cache in sync
        Self::save_hunt_cache(env, hunt);
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
        let result: Option<Hunt> = env.storage().persistent().get(&key);
        if let Some(ref hunt) = result {
            let policy = match hunt.status {
                crate::types::HuntStatus::Active => TtlPolicy::Active,
                crate::types::HuntStatus::Completed | crate::types::HuntStatus::Cancelled => {
                    TtlPolicy::Short
                }
                _ => TtlPolicy::Default,
            };
            extend_ttl(env, &key, policy);
        }
        result
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

    // ========== Hunt Cache Functions (instance storage) ==========

    /// Saves a compact HuntCache to instance storage for faster reads.
    /// The cache contains only frequently-accessed fields (no title/description strings).
    /// Also extends the instance TTL so the cache stays warm for active hunts.
    pub fn save_hunt_cache(env: &Env, hunt: &Hunt) {
        let cache = HuntCache::from_hunt(hunt);
        let key = Self::hunt_cache_key(hunt.hunt_id);
        env.storage().instance().set(&key, &cache);
        env.storage().instance().extend_ttl(INSTANCE_TTL_THRESHOLD, INSTANCE_TTL_EXTEND_TO);
    }

    /// Retrieves a HuntCache from instance storage.
    /// Returns None if no cache exists for this hunt_id.
    /// Records cache hit/miss for monitoring.
    pub fn get_hunt_cache(env: &Env, hunt_id: u64) -> Option<HuntCache> {
        let key = Self::hunt_cache_key(hunt_id);
        let result: Option<HuntCache> = env.storage().instance().get(&key);
        if result.is_some() {
            Self::record_cache_hit(env);
            env.storage().instance().extend_ttl(INSTANCE_TTL_THRESHOLD, INSTANCE_TTL_EXTEND_TO);
        } else {
            Self::record_cache_miss(env);
        }
        result
    }

    /// Removes the HuntCache for a given hunt from instance storage.
    /// Use when a hunt is updated and the cache should be refreshed.
    pub fn invalidate_hunt_cache(env: &Env, hunt_id: u64) {
        let key = Self::hunt_cache_key(hunt_id);
        env.storage().instance().remove(&key);
    }

    /// Bumps the instance TTL for the hunt cache without modifying its value.
    /// Useful for keeping hot hunt caches alive between operations.
    pub fn bump_hunt_cache_ttl(env: &Env, hunt_id: u64) {
        let key = Self::hunt_cache_key(hunt_id);
        if env.storage().instance().has(&key) {
            env.storage().instance().extend_ttl(INSTANCE_TTL_THRESHOLD, INSTANCE_TTL_EXTEND_TO);
        }
    }

    /// Resets cache hit/miss counters (admin use only).
    pub fn reset_cache_counters(env: &Env) {
        env.storage().instance().remove(&Self::CACHE_HIT_KEY);
        env.storage().instance().remove(&Self::CACHE_MISS_KEY);
    }

    /// Checks whether a HuntCache exists in instance storage.
    /// Useful for cheap existence checks without loading the full Hunt struct.
    pub fn has_hunt_cache(env: &Env, hunt_id: u64) -> bool {
        let key = Self::hunt_cache_key(hunt_id);
        env.storage().instance().has(&key)
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
        extend_ttl(env, &key, TtlPolicy::Active);

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
        let result: Option<Clue> = env.storage().persistent().get(&key);
        if result.is_some() {
            extend_ttl(env, &key, TtlPolicy::Active);
        }
        result
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

    pub fn list_clues_for_hunt(env: &Env, hunt_id: u64, offset: u32, limit: u32) -> Vec<Clue> {
        let clue_ids = Self::get_clue_ids_for_hunt(env, hunt_id, offset, limit);
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
        let policy = if progress.is_completed || progress.reward_claimed {
            TtlPolicy::Short
        } else {
            TtlPolicy::Default
        };
        extend_ttl(env, &key, policy);

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
        let raw_val: Option<soroban_sdk::Val> = env.storage().persistent().get(&key);
        raw_val.map(|val| {
            if let Ok(bytes) = soroban_sdk::Bytes::try_from_val(env, &val) {
                let stored: StoredPlayerProgress = StoredPlayerProgress::from_xdr(env, &bytes).unwrap();
                PlayerProgress::from_stored(stored, player.clone(), hunt_id)
            } else {
                let stored: StoredPlayerProgress = StoredPlayerProgress::try_from_val(env, &val).unwrap();
                PlayerProgress::from_stored(stored, player.clone(), hunt_id)
            }
        })
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

    pub fn save_leaderboard_index(
        env: &Env,
        hunt_id: u64,
        entries: &Vec<LeaderboardIndexEntry>,
    ) {
        let key = Self::leaderboard_key(hunt_id);
        env.storage().persistent().set(&key, entries);
        extend_ttl(env, &key, TtlPolicy::Active);
    }

    pub fn get_leaderboard_index(env: &Env, hunt_id: u64) -> Vec<LeaderboardIndexEntry> {
        let key = Self::leaderboard_key(hunt_id);
        let result: Option<Vec<LeaderboardIndexEntry>> = env.storage().persistent().get(&key);
        if result.is_some() {
            extend_ttl(env, &key, TtlPolicy::Active);
        }
        result.unwrap_or_else(|| Vec::new(env))
    }

    // ========== Helper Functions for Key Generation ==========

    /// Generates a storage key for a hunt using a symbol prefix and hunt_id.
    /// Uses tuple key (HUNT_KEY, hunt_id) for efficient storage access.
    fn hunt_key(hunt_id: u64) -> (soroban_sdk::Symbol, u64) {
        (Self::HUNT_KEY, hunt_id)
    }

    fn hunt_cache_key(hunt_id: u64) -> (soroban_sdk::Symbol, u64) {
        (Self::HUNT_CACHE_KEY, hunt_id)
    }

    /// Generates a composite storage key for a clue.
    /// Uses tuple key (CLUE_KEY, hunt_id, clue_id) for efficient storage access.
    fn clue_key(hunt_id: u64, clue_id: u32) -> (soroban_sdk::Symbol, u64, u32) {
        (Self::CLUE_KEY, hunt_id, clue_id)
    }

    pub fn progress_key(hunt_id: u64, player: &Address) -> (soroban_sdk::Symbol, u64, Address) {
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

    fn required_clues_key(hunt_id: u64) -> (soroban_sdk::Symbol, u64) {
        (Self::REQUIRED_CLUES_KEY, hunt_id)
    }

    fn player_entry_key(hunt_id: u64, index: u32) -> (soroban_sdk::Symbol, u64, u32) {
        (Self::PLAYER_ENTRY_KEY, hunt_id, index)
    }

    fn player_count_key(hunt_id: u64) -> (soroban_sdk::Symbol, u64) {
        (Self::PLAYER_COUNT_KEY, hunt_id)
    }

    fn clue_exists_key(hunt_id: u64, clue_id: u32) -> (soroban_sdk::Symbol, u64, u32) {
        (symbol_short!("CLEX"), hunt_id, clue_id)
    }

    fn leaderboard_key(hunt_id: u64) -> (soroban_sdk::Symbol, u64) {
        (Self::LEADERBOARD_KEY, hunt_id)
    }

    /// Key for view-only addresses for a hunt.
    fn view_only_key(hunt_id: u64) -> (soroban_sdk::Symbol, u64) {
        (Self::VIEW_ONLY_KEY, hunt_id)
    }

    /// Generates a storage key for a processed answer submission envelope.
    fn processed_submission_key(
        hunt_id: u64,
        clue_id: u32,
        player: &Address,
        submission_nonce: u64,
        submitted_at: u64,
    ) -> (soroban_sdk::Symbol, u64, u32, Address, u64, u64) {
        (
            Self::SUBMISSION_KEY,
            hunt_id,
            clue_id,
            player.clone(),
            submission_nonce,
            submitted_at,
        )
    }

    // ========== Internal Helper Functions ==========

    /// Adds a clue ID to the list of clues for a hunt.
    /// This maintains an index for efficient listing.
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

    fn get_clue_ids_for_hunt(env: &Env, hunt_id: u64, offset: u32, limit: u32) -> Vec<u32> {
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

    pub fn get_player_addresses_for_hunt(env: &Env, hunt_id: u64) -> Vec<Address> {
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
        extend_ttl(env, &key, TtlPolicy::Critical);
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
        let result: Option<u64> = env.storage().persistent().get(&key);
        if result.is_some() {
            extend_ttl(env, &key, TtlPolicy::Critical);
        }
        result.unwrap_or(0)
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
        extend_ttl(env, &key, TtlPolicy::Active);
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
        let result: Option<u32> = env.storage().persistent().get(&key);
        if result.is_some() {
            extend_ttl(env, &key, TtlPolicy::Active);
        }
        result.unwrap_or(0)
    }

    // ========== Required Clues Storage (on-demand loading) ==========

    /// Saves the list of required clue IDs for a hunt.
    /// This allows `check_all_required_clues_completed` to verify completion
    /// without loading full clue data, significantly reducing gas.
    pub fn set_required_clues(env: &Env, hunt_id: u64, clue_ids: &soroban_sdk::Vec<u32>) {
        let key = Self::required_clues_key(hunt_id);
        env.storage().persistent().set(&key, clue_ids);
        extend_ttl(env, &key, TtlPolicy::Active);
    }

    /// Returns the list of required clue IDs for a hunt.
    /// Falls back to an empty vec if no list has been stored (e.g. pre-migration hunts).
    pub fn get_required_clues(env: &Env, hunt_id: u64) -> soroban_sdk::Vec<u32> {
        let key = Self::required_clues_key(hunt_id);
        let result: Option<soroban_sdk::Vec<u32>> = env.storage().persistent().get(&key);
        if result.is_some() {
            extend_ttl(env, &key, TtlPolicy::Active);
        }
        result.unwrap_or_else(|| soroban_sdk::Vec::new(env))
    }

    // ========== Reward Manager Storage Functions ==========

    pub fn set_reward_manager(env: &Env, address: &Address) {
        env.storage()
            .persistent()
            .set(&Self::REWARD_MGR_KEY, address);
        extend_ttl(env, &Self::REWARD_MGR_KEY, TtlPolicy::Critical);
    }

    pub fn get_reward_manager(env: &Env) -> Option<Address> {
        let result: Option<Address> = env.storage().persistent().get(&Self::REWARD_MGR_KEY);
        if result.is_some() {
            extend_ttl(env, &Self::REWARD_MGR_KEY, TtlPolicy::Critical);
        }
        result
    }

    pub fn save_processed_submission(
        env: &Env,
        hunt_id: u64,
        clue_id: u32,
        player: &Address,
        submission_nonce: u64,
        submitted_at: u64,
        expires_at: u64,
    ) {
        let key = Self::processed_submission_key(
            hunt_id,
            clue_id,
            player,
            submission_nonce,
            submitted_at,
        );
        env.storage().persistent().set(&key, &expires_at);
    }

    pub fn get_processed_submission_expiry(
        env: &Env,
        hunt_id: u64,
        clue_id: u32,
        player: &Address,
        submission_nonce: u64,
        submitted_at: u64,
    ) -> Option<u64> {
        let key = Self::processed_submission_key(
            hunt_id,
            clue_id,
            player,
            submission_nonce,
            submitted_at,
        );
        env.storage().persistent().get(&key)
    }

    pub fn remove_processed_submission(
        env: &Env,
        hunt_id: u64,
        clue_id: u32,
        player: &Address,
        submission_nonce: u64,
        submitted_at: u64,
    ) {
        let key = Self::processed_submission_key(
            hunt_id,
            clue_id,
            player,
            submission_nonce,
            submitted_at,
        );
        env.storage().persistent().remove(&key);
    }

    // --- Contract version ---

    #[allow(dead_code)]
    pub fn set_contract_version(env: &Env, version: u32) {
        env.storage()
            .instance()
            .set(&symbol_short!("CVER"), &version);
    }

    #[allow(dead_code)]
    pub fn get_contract_version(env: &Env) -> Option<u32> {
        env.storage().instance().get(&symbol_short!("CVER"))
    }

    // ========== View-Only Access Functions ==========

    /// Adds an address to the view-only list for a specific hunt.
    /// View-only addresses can read hunt data but cannot modify it.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `hunt_id` - The hunt to grant view-only access for
    /// * `address` - The address to grant view-only access
    pub fn add_view_only(env: &Env, hunt_id: u64, address: &Address) {
        let key = Self::view_only_key(hunt_id);
        let mut view_only_list = env
            .storage()
            .instance()
            .get::<_, Vec<Address>>(&key)
            .unwrap_or_else(|| Vec::new(env));
        
        // Check if address already exists to avoid duplicates
        if view_only_list.first_index_of(address).is_none() {
            view_only_list.push_back(address.clone());
            env.storage().instance().set(&key, &view_only_list);
        }
    }

    /// Removes an address from the view-only list for a specific hunt.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `hunt_id` - The hunt to revoke view-only access for
    /// * `address` - The address to revoke view-only access
    pub fn remove_view_only(env: &Env, hunt_id: u64, address: &Address) {
        let key = Self::view_only_key(hunt_id);
        let mut view_only_list = env
            .storage()
            .instance()
            .get::<_, Vec<Address>>(&key)
            .unwrap_or_else(|| Vec::new(env));
        
        if let Some(idx) = view_only_list.first_index_of(address) {
            view_only_list.remove(idx);
            env.storage().instance().set(&key, &view_only_list);
        }
    }

    /// Checks if an address has view-only access for a specific hunt.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `hunt_id` - The hunt to check view-only access for
    /// * `address` - The address to check
    ///
    /// # Returns
    /// `true` if the address has view-only access, `false` otherwise
    pub fn is_view_only(env: &Env, hunt_id: u64, address: &Address) -> bool {
        let key = Self::view_only_key(hunt_id);
        let view_only_list = env
            .storage()
            .instance()
            .get::<_, Vec<Address>>(&key)
            .unwrap_or_else(|| Vec::new(env));
        
        view_only_list.first_index_of(address).is_some()
    }

    /// Gets all view-only addresses for a specific hunt.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `hunt_id` - The hunt to get view-only addresses for
    ///
    /// # Returns
    /// A vector of all addresses with view-only access for the hunt
    pub fn get_view_only_list(env: &Env, hunt_id: u64) -> Vec<Address> {
        let key = Self::view_only_key(hunt_id);
        env.storage()
            .instance()
            .get::<_, Vec<Address>>(&key)
            .unwrap_or_else(|| Vec::new(env))
    }

    // ========== Global Admin Functions ==========

    /// Checks if an address is the contract admin
    pub fn is_admin(env: &Env, address: &Address) -> bool {
        if let Some(admin) = Self::get_admin(env) {
            admin == *address
        } else {
            false
        }
    }
    
    /// Sets the contract admin address.
    /// The admin can manage global view-only access.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `admin` - The admin address
    pub fn set_admin(env: &Env, admin: &Address) {
        env.storage().instance().set(&Self::ADMIN_KEY, admin);
    }

    /// Gets the contract admin address.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    ///
    /// # Returns
    /// The admin address if set, None otherwise
    pub fn get_admin(env: &Env) -> Option<Address> {
        env.storage().instance().get(&Self::ADMIN_KEY)
    }

    const PENDING_ADMIN_KEY: soroban_sdk::Symbol = symbol_short!("ADM_PEND");

    /// Stores a proposed admin address pending acceptance via `accept_admin`.
    pub fn set_pending_admin(env: &Env, admin: &Address) {
        env.storage()
            .instance()
            .set(&Self::PENDING_ADMIN_KEY, admin);
    }

    /// Returns the proposed admin address, if any.
    pub fn get_pending_admin(env: &Env) -> Option<Address> {
        env.storage().instance().get(&Self::PENDING_ADMIN_KEY)
    }

    /// Clears any proposed admin address.
    pub fn clear_pending_admin(env: &Env) {
        env.storage().instance().remove(&Self::PENDING_ADMIN_KEY);
    }
    
    // Backward compatibility: general pause
    const PAUSE_KEY: soroban_sdk::Symbol = symbol_short!("PAUSE");
    pub fn set_paused(env: &Env, paused: bool) {
        env.storage().instance().set(&Self::PAUSE_KEY, &paused);
    }
    pub fn is_paused(env: &Env) -> bool {
        env.storage().instance().get(&Self::PAUSE_KEY).unwrap_or(false)
    }
    
    // Blacklist functions for backward compatibility
    const BLACKLIST_KEY: soroban_sdk::Symbol = symbol_short!("BLACKLIST");
    pub fn set_blacklisted(env: &Env, address: &Address, blacklisted: bool) {
        if blacklisted {
            let mut list = env.storage().instance().get(&Self::BLACKLIST_KEY).unwrap_or_else(|| Vec::new(env));
            if list.first_index_of(address).is_none() {
                list.push_back(address.clone());
                env.storage().instance().set(&Self::BLACKLIST_KEY, &list);
            }
        } else {
            let mut list = env.storage().instance().get(&Self::BLACKLIST_KEY).unwrap_or_else(|| Vec::new(env));
            if let Some(idx) = list.first_index_of(address) {
                list.remove(idx);
                env.storage().instance().set(&Self::BLACKLIST_KEY, &list);
            }
        }
    }
    pub fn is_blacklisted(env: &Env, address: &Address) -> bool {
        let list: Vec<Address> = env.storage().instance().get(&Self::BLACKLIST_KEY).unwrap_or_else(|| Vec::new(env));
        list.first_index_of(address).is_some()
    }
    
    // Helper functions for emergency stop (placeholder for now)
    pub fn get_active_hunt_ids(_env: &Env) -> Vec<u64> {
        Vec::new(_env)
    }
    pub fn set_hunt_status(_env: &Env, _hunt_id: u64, _status: crate::types::HuntStatus) {
        // Placeholder
    }

    /// Adds an address to the global view-only list.
    /// Global view-only addresses can read ALL hunt data.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `address` - The address to grant global view-only access
    pub fn add_global_view_only(env: &Env, address: &Address) {
        let mut view_only_list = env
            .storage()
            .instance()
            .get::<_, Vec<Address>>(&Self::GLOBAL_VIEW_ONLY_KEY)
            .unwrap_or_else(|| Vec::new(env));
        
        // Check if address already exists to avoid duplicates
        if view_only_list.first_index_of(address).is_none() {
            view_only_list.push_back(address.clone());
            env.storage().instance().set(&Self::GLOBAL_VIEW_ONLY_KEY, &view_only_list);
        }
    }

    /// Removes an address from the global view-only list.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `address` - The address to revoke global view-only access
    pub fn remove_global_view_only(env: &Env, address: &Address) {
        let mut view_only_list = env
            .storage()
            .instance()
            .get::<_, Vec<Address>>(&Self::GLOBAL_VIEW_ONLY_KEY)
            .unwrap_or_else(|| Vec::new(env));
        
        if let Some(idx) = view_only_list.first_index_of(address) {
            view_only_list.remove(idx);
            env.storage().instance().set(&Self::GLOBAL_VIEW_ONLY_KEY, &view_only_list);
        }
    }

    /// Checks if an address has global view-only access.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `address` - The address to check
    ///
    /// # Returns
    /// `true` if the address has global view-only access, `false` otherwise
    pub fn is_global_view_only(env: &Env, address: &Address) -> bool {
        let view_only_list = env
            .storage()
            .instance()
            .get::<_, Vec<Address>>(&Self::GLOBAL_VIEW_ONLY_KEY)
            .unwrap_or_else(|| Vec::new(env));
        
        view_only_list.first_index_of(address).is_some()
    }

    /// Gets all global view-only addresses.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    ///
    /// # Returns
    /// A vector of all addresses with global view-only access
    pub fn get_global_view_only_list(env: &Env) -> Vec<Address> {
        env.storage()
            .instance()
            .get::<_, Vec<Address>>(&Self::GLOBAL_VIEW_ONLY_KEY)
            .unwrap_or_else(|| Vec::new(env))
    }

    // ========== Ban Storage Functions ==========

    fn ban_key(hunt_id: u64, player: &Address) -> (soroban_sdk::Symbol, u64, Address) {
        (Self::BAN_KEY, hunt_id, player.clone())
    }

    pub fn ban_player(env: &Env, hunt_id: u64, player: &Address) {
        env.storage().persistent().set(&Self::ban_key(hunt_id, player), &());
    }

    pub fn unban_player(env: &Env, hunt_id: u64, player: &Address) {
        env.storage().persistent().remove(&Self::ban_key(hunt_id, player));
    }

    pub fn is_banned(env: &Env, hunt_id: u64, player: &Address) -> bool {
        env.storage().persistent().has(&Self::ban_key(hunt_id, player))
    }

    // ========== Admin Storage Functions ==========

    pub fn set_admin(env: &Env, admin: &Address) {
        env.storage().instance().set(&Self::ADMIN_KEY, admin);
    }

    pub fn get_admin(env: &Env) -> Option<Address> {
        env.storage().instance().get(&Self::ADMIN_KEY)
    }

    // ========== Blacklist Storage Functions ==========

    fn blacklist_key(creator: &Address) -> (soroban_sdk::Symbol, Address) {
        (symbol_short!("BLKLST"), creator.clone())
    }

    pub fn blacklist_creator(env: &Env, creator: &Address) {
        env.storage()
            .instance()
            .set(&Self::blacklist_key(creator), &true);
    }

    pub fn remove_from_blacklist(env: &Env, creator: &Address) {
        env.storage()
            .instance()
            .remove(&Self::blacklist_key(creator));
    }

    pub fn is_blacklisted(env: &Env, creator: &Address) -> bool {
        env.storage()
            .instance()
            .get::<_, bool>(&Self::blacklist_key(creator))
            .unwrap_or(false)
    }

    pub fn set_creator_blacklisted(env: &Env, creator: &Address, blacklisted: bool) {
        let mut blacklist: Map<Address, bool> = env
            .storage()
            .instance()
            .get(&Self::BLACKLIST_KEY)
            .unwrap_or(Map::new(env));
        blacklist.set(creator.clone(), blacklisted);
        env.storage().instance().set(&Self::BLACKLIST_KEY, &blacklist);
        env.storage()
            .instance()
            .extend_ttl(INSTANCE_TTL_THRESHOLD, INSTANCE_TTL_EXTEND_TO);
    }

    pub fn is_creator_blacklisted(env: &Env, creator: &Address) -> bool {
        let blacklist: Map<Address, bool> = env
            .storage()
            .instance()
            .get(&Self::BLACKLIST_KEY)
            .unwrap_or(Map::new(env));
        blacklist.get(creator.clone()).unwrap_or(false)
    }

    // ========== Hunt creation rate limiting ==========

    pub fn get_rate_limit_admin(env: &Env) -> Option<Address> {
        env.storage().instance().get(&symbol_short!("HRLADM"))
    }

    pub fn set_rate_limit_admin(env: &Env, admin: &Address) {
        env.storage().instance().set(&symbol_short!("HRLADM"), admin);
    }

    pub fn get_default_hunt_creation_limit(env: &Env) -> u32 {
        env.storage()
            .instance()
            .get(&symbol_short!("HRLDEF"))
            .unwrap_or(crate::rate_limit::DEFAULT_HUNT_CREATION_LIMIT)
    }

    pub fn set_default_hunt_creation_limit(env: &Env, limit: u32) {
        env.storage().instance().set(&symbol_short!("HRLDEF"), &limit);
    }

    pub fn get_creator_limit_override(env: &Env, creator: &Address) -> Option<u32> {
        let key = (symbol_short!("HRLOVR"), creator.clone());
        env.storage().persistent().get(&key)
    }

    pub fn set_creator_limit_override(env: &Env, creator: &Address, limit: u32) {
        let key = (symbol_short!("HRLOVR"), creator.clone());
        env.storage().persistent().set(&key, &limit);
    }

    pub fn get_effective_hunt_creation_limit(env: &Env, creator: &Address) -> u32 {
        Self::get_creator_limit_override(env, creator)
            .unwrap_or_else(|| Self::get_default_hunt_creation_limit(env))
    }

    fn creator_daily_count_key(creator: &Address, day: u64) -> (soroban_sdk::Symbol, Address, u64) {
        (symbol_short!("HRLCT"), creator.clone(), day)
    }

    pub fn get_creator_daily_hunt_count(env: &Env, creator: &Address, day: u64) -> u32 {
        let key = Self::creator_daily_count_key(creator, day);
        env.storage().persistent().get(&key).unwrap_or(0)
    }

    pub fn set_creator_daily_hunt_count(env: &Env, creator: &Address, day: u64, count: u32) {
        let key = Self::creator_daily_count_key(creator, day);
        env.storage().persistent().set(&key, &count);
    }
}
