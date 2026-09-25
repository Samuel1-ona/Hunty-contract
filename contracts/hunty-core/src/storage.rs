use crate::errors::HuntError;

use crate::types::{
    Clue, GcReport, Hunt, HuntCache, HuntStatus, LeaderboardIndexEntry, PlayerProgress,
    RewardConfig, StoredPlayerProgress, Team, TeamProgress,
};

use soroban_sdk::xdr::FromXdr;

use soroban_sdk::{contracttype, symbol_short, Address, Env, IntoVal, TryFromVal, Val, Vec};

// Instance TTL constants used by blacklist and contract-pause storage.

const INSTANCE_TTL_THRESHOLD: u32 = 518_400;

const INSTANCE_TTL_EXTEND_TO: u32 = 518_400;

const PERSISTENT_TTL_THRESHOLD: u32 = 172_800;

const PERSISTENT_TTL_EXTEND_TO: u32 = 518_400;

/// Legacy indexes are bounded by the contract's 100-clue limit during a lazy

/// migration, even if an old/corrupt count key contains a larger value.

const MAX_MIGRATED_CLUE_INDEX_ENTRIES: u32 = 100;

/// Maximum number of addresses in a hunt's view-only list, and in the global

/// view-only list. Keeps enumeration bounded so `get_view_only_list` and

/// `is_view_only` stay within the invocation budget as the list grows.

pub(crate) const MAX_VIEW_ONLY_ENTRIES: u32 = 200;

#[contracttype]
#[derive(Clone, Debug)]

pub struct CreatorDailyHuntCount {
    pub day: u64,

    pub count: u32,
}

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

// Several helpers and key constants are reserved for upcoming modules and the

// migration framework; keep them without triggering dead-code warnings.

#[allow(dead_code)]
impl Storage {
    // Symbol constants for key prefixes to prevent collisions

    // Using symbol_short for efficient key generation

    // Shortened, unique storage key prefixes (reduced to minimal unique prefixes)

    const HUNT_KEY: soroban_sdk::Symbol = symbol_short!("HUNT");

    const HUNT_CACHE_KEY: soroban_sdk::Symbol = symbol_short!("HC");

    const CLUE_KEY: soroban_sdk::Symbol = symbol_short!("CLU");

    const PROGRESS_KEY: soroban_sdk::Symbol = symbol_short!("PR");

    const PLAYERS_LIST_KEY: soroban_sdk::Symbol = symbol_short!("PL");

    const LEADERBOARD_KEY: soroban_sdk::Symbol = symbol_short!("LBD");

    const CLUES_LIST_KEY: soroban_sdk::Symbol = symbol_short!("CLS");

    const PLAYER_ENTRY_KEY: soroban_sdk::Symbol = symbol_short!("PLRS");

    const PLAYER_COUNT_KEY: soroban_sdk::Symbol = symbol_short!("PLCT");

    const CLUE_ENTRY_KEY: soroban_sdk::Symbol = symbol_short!("CLST");

    const CLUE_LIST_COUNT_KEY: soroban_sdk::Symbol = symbol_short!("CLCT");

    const CLUE_EXISTS_KEY: soroban_sdk::Symbol = symbol_short!("CLEX");

    const PLAYER_EXISTS_KEY: soroban_sdk::Symbol = symbol_short!("PLEX");

    const HUNT_COUNTER_KEY: soroban_sdk::Symbol = symbol_short!("CN");

    const CLUE_COUNTER_KEY: soroban_sdk::Symbol = symbol_short!("CC");

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

    const REQUIRED_CLUES_KEY: soroban_sdk::Symbol = symbol_short!("REQCL");

    const CACHE_HIT_KEY: soroban_sdk::Symbol = symbol_short!("CHIT");

    const CACHE_MISS_KEY: soroban_sdk::Symbol = symbol_short!("CMISS");

    const PLAYER_HUNTS_KEY: soroban_sdk::Symbol = symbol_short!("PHNT");

    const TEAM_KEY: soroban_sdk::Symbol = symbol_short!("TEAM");

    const TEAM_COUNT_KEY: soroban_sdk::Symbol = symbol_short!("TMCT");

    const PLAYER_TEAM_KEY: soroban_sdk::Symbol = symbol_short!("PLTM");

    const TEAM_PROGRESS_KEY: soroban_sdk::Symbol = symbol_short!("TMPR");

    // Pause functions (granular: registrations, answers, rewards)

    pub fn set_pause_registrations(env: &Env, paused: bool) {
        env.storage()
            .instance()
            .set(&Self::PAUSE_REGISTRATIONS_KEY, &paused);
    }

    pub fn is_pause_registrations(env: &Env) -> bool {
        env.storage()
            .instance()
            .get(&Self::PAUSE_REGISTRATIONS_KEY)
            .unwrap_or(false)
    }

    pub fn set_pause_answers(env: &Env, paused: bool) {
        env.storage()
            .instance()
            .set(&Self::PAUSE_ANSWERS_KEY, &paused);
    }

    pub fn is_pause_answers(env: &Env) -> bool {
        env.storage()
            .instance()
            .get(&Self::PAUSE_ANSWERS_KEY)
            .unwrap_or(false)
    }

    pub fn set_pause_rewards(env: &Env, paused: bool) {
        env.storage()
            .instance()
            .set(&Self::PAUSE_REWARDS_KEY, &paused);
    }

    pub fn is_pause_rewards(env: &Env) -> bool {
        env.storage()
            .instance()
            .get(&Self::PAUSE_REWARDS_KEY)
            .unwrap_or(false)
    }

    // Global contract pause (emergency stop for all operations)

    pub fn set_contract_paused(env: &Env, paused: bool) {
        env.storage()
            .instance()
            .set(&Self::CONTRACT_PAUSED_KEY, &paused);

        env.storage()
            .instance()
            .extend_ttl(INSTANCE_TTL_THRESHOLD, INSTANCE_TTL_EXTEND_TO);
    }

    pub fn is_contract_paused(env: &Env) -> bool {
        env.storage()
            .instance()
            .get(&Self::CONTRACT_PAUSED_KEY)
            .unwrap_or(false)
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

        // The hunt record is authoritative persistent data. Remove a legacy

        // instance copy only after the persistent write succeeds so an upgrade

        // can never leave a hunt with two disagreeing records.

        env.storage().persistent().set(&key, hunt);

        env.storage().instance().remove(&key);

        let policy = match hunt.status {
            crate::types::HuntStatus::Active => TtlPolicy::Active,

            crate::types::HuntStatus::Completed | crate::types::HuntStatus::Cancelled => {
                TtlPolicy::Short
            }

            _ => TtlPolicy::Default,
        };

        extend_ttl(env, &key, policy);

        // HuntCache is deliberately an instance-storage read optimization; it

        // is rebuilt from the persistent record and is never authoritative.

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

        let persistent_raw: Option<Val> = env.storage().persistent().get(&key);

        // Decode both the current layout and the layout used by older

        // deployments. The same decoder is used for the persistent and legacy

        // instance locations so a migration cannot change the value semantics.

        let decode = |value: &Val| -> Option<Hunt> {
            if let Ok(hunt) = Hunt::try_from_val(env, value) {
                return Some(hunt);
            }

            #[contracttype]
            #[derive(Clone, Debug)]

            struct LegacyHunt {
                pub hunt_id: u64,

                pub creator: Address,

                pub title: soroban_sdk::String,

                pub description: soroban_sdk::String,

                pub status: HuntStatus,

                pub created_at: u64,

                pub activated_at: u64,

                pub end_time: u64,

                pub reward_config: RewardConfig,

                pub time_bonus_start_bps: Option<u32>,

                pub time_bonus_min_bps: Option<u32>,

                pub time_bonus_decay_secs: Option<u64>,

                pub total_clues: u32,

                pub required_clues: u32,

                pub completed_count: u32,

                pub max_submissions_per_minute: u32,

                pub max_attempts_per_clue: u32,

                pub start_multiplier_bps: u32,
            }

            LegacyHunt::try_from_val(env, value)
                .ok()
                .map(|legacy| Hunt {
                    hunt_id: legacy.hunt_id,

                    creator: legacy.creator,

                    title: legacy.title,

                    description: legacy.description,

                    categories: Vec::new(env),

                    difficulty_rating: 0,

                    difficulty_override: None,

                    status: legacy.status,

                    created_at: legacy.created_at,

                    activated_at: legacy.activated_at,

                    start_time: 0,

                    end_time: legacy.end_time,

                    reward_config: legacy.reward_config,

                    time_bonus_start_bps: legacy.time_bonus_start_bps,

                    time_bonus_min_bps: legacy.time_bonus_min_bps,

                    time_bonus_decay_secs: legacy.time_bonus_decay_secs,

                    total_clues: legacy.total_clues,

                    required_clues: legacy.required_clues,

                    completed_count: legacy.completed_count,

                    max_submissions_per_minute: legacy.max_submissions_per_minute,

                    max_attempts_per_clue: legacy.max_attempts_per_clue,

                    start_multiplier_bps: legacy.start_multiplier_bps,

                    registration_deadline: 0,

                    allow_partial_scoring: false,

                    team_mode: false,

                    default_points: 100,

                    attempt_cooldown_secs: 0,

                    max_players: 0,

                    is_private: false,

                    invite_code_hash: None,

                    remaining_slots: 0,
                })
        };

        let mut result = persistent_raw.as_ref().and_then(decode);

        let mut promoted_legacy = false;

        if result.is_none() {
            // Before the storage-tier fix, HUNT lived in instance storage. A

            // lazy promotion is safe and idempotent: callers can migrate data

            // while the old entries are still live, without requiring a

            // destructive all-at-once state rewrite.

            if let Some(legacy_raw) = env.storage().instance().get::<_, Val>(&key) {
                result = decode(&legacy_raw);

                promoted_legacy = result.is_some();
            }
        }

        if promoted_legacy {
            if let Some(ref hunt) = result {
                env.storage().persistent().set(&key, hunt);

                env.storage().instance().remove(&key);
            }
        } else if result.is_some() {
            // Clear a stale legacy copy after a successful canonical read.

            env.storage().instance().remove(&key);
        }

        if let Some(ref mut hunt) = result {
            let policy = match hunt.status {
                crate::types::HuntStatus::Active => TtlPolicy::Active,

                crate::types::HuntStatus::Completed | crate::types::HuntStatus::Cancelled => {
                    TtlPolicy::Short
                }

                _ => TtlPolicy::Default,
            };

            extend_ttl(env, &key, policy);

            // Dynamically calculate remaining slots

            let count = Self::get_player_count(env, hunt_id);

            hunt.remaining_slots = if hunt.max_players == 0 {
                0
            } else {
                hunt.max_players.saturating_sub(count)
            };
        }

        result
    }

    /// Returns whether the authoritative persistent hunt record exists.

    /// A cache hit alone is not sufficient because it can outlive (or mask)

    /// a legacy instance record during an upgrade.

    pub fn has_persistent_hunt(env: &Env, hunt_id: u64) -> bool {
        env.storage().persistent().has(&Self::hunt_key(hunt_id))
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
        Self::get_hunt(env, hunt_id).ok_or(HuntError::HuntNotFound)
    }

    // ========== Hunt Cache Functions (instance storage) ==========

    /// Saves a compact HuntCache to instance storage for faster reads.

    /// The cache contains only frequently-accessed fields (no title/description strings).

    /// Also extends the instance TTL so the cache stays warm for active hunts.

    pub fn save_hunt_cache(env: &Env, hunt: &Hunt) {
        let cache = HuntCache::from_hunt(hunt);

        let key = Self::hunt_cache_key(hunt.hunt_id);

        env.storage().instance().set(&key, &cache);

        env.storage()
            .instance()
            .extend_ttl(INSTANCE_TTL_THRESHOLD, INSTANCE_TTL_EXTEND_TO);
    }

    /// Retrieves a HuntCache from instance storage.

    /// Returns None if no cache exists for this hunt_id.

    /// Records cache hit/miss for monitoring.

    pub fn get_hunt_cache(env: &Env, hunt_id: u64) -> Option<HuntCache> {
        let key = Self::hunt_cache_key(hunt_id);

        let result: Option<HuntCache> = env.storage().instance().get(&key);

        if result.is_some() {
            Self::record_cache_hit(env);

            env.storage()
                .instance()
                .extend_ttl(INSTANCE_TTL_THRESHOLD, INSTANCE_TTL_EXTEND_TO);
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
            env.storage()
                .instance()
                .extend_ttl(INSTANCE_TTL_THRESHOLD, INSTANCE_TTL_EXTEND_TO);
        }
    }

    /// Resets cache hit/miss counters (admin use only).

    pub fn reset_cache_counters(env: &Env) {
        env.storage().instance().remove(&Self::CACHE_HIT_KEY);

        env.storage().instance().remove(&Self::CACHE_MISS_KEY);
    }

    pub fn record_cache_hit(env: &Env) {
        let count: u64 = env
            .storage()
            .instance()
            .get(&Self::CACHE_HIT_KEY)
            .unwrap_or(0);

        env.storage()
            .instance()
            .set(&Self::CACHE_HIT_KEY, &(count + 1));
    }

    pub fn record_cache_miss(env: &Env) {
        let count: u64 = env
            .storage()
            .instance()
            .get(&Self::CACHE_MISS_KEY)
            .unwrap_or(0);

        env.storage()
            .instance()
            .set(&Self::CACHE_MISS_KEY, &(count + 1));
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
        // Store the clue with a per-clue persistent key and per-key TTL. Remove

        // the pre-migration instance copy only after the canonical write.

        let key = Self::clue_key(hunt_id, clue.clue_id);

        env.storage().persistent().set(&key, clue);

        env.storage().instance().remove(&key);

        extend_ttl(env, &key, TtlPolicy::Active);

        // Update the persistent clue index and required-clue index.

        Self::add_clue_to_list(env, hunt_id, clue.clue_id);

        if clue.is_required {
            Self::add_required_clue(env, hunt_id, clue.clue_id);
        }
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

        let persistent_raw: Option<Val> = env.storage().persistent().get(&key);

        let decode = |value: &Val| -> Option<Clue> {
            // First try to deserialize as the current Clue layout.

            if let Ok(clue) = Clue::try_from_val(env, value) {
                return Some(clue);
            }

            // If that fails, try the pre-tier-migration layout and fill fields

            // introduced later with safe defaults.

            #[contracttype]
            #[derive(Clone, Debug)]

            struct LegacyClue {
                pub clue_id: u32,

                pub question: soroban_sdk::String,

                pub answer_hashes: soroban_sdk::Vec<soroban_sdk::BytesN<32>>,

                pub points: u32,

                pub is_required: bool,

                pub difficulty: u32,
            }

            LegacyClue::try_from_val(env, value)
                .ok()
                .map(|legacy| Clue {
                    clue_id: legacy.clue_id,

                    question: legacy.question,

                    answer_hashes: legacy.answer_hashes,

                    points: legacy.points,

                    is_required: legacy.is_required,

                    difficulty: legacy.difficulty,

                    weight: 1,

                    hint: None,

                    hint_penalty_points: 0,
                })
        };

        let mut result = persistent_raw.as_ref().and_then(decode);

        let mut promoted_legacy = false;

        if result.is_none() {
            // Older deployments kept the clue record in instance storage. Read

            // and promote it on first access, preserving compatibility during a

            // rolling upgrade.

            if let Some(legacy_raw) = env.storage().instance().get::<_, Val>(&key) {
                result = decode(&legacy_raw);

                promoted_legacy = result.is_some();
            }
        }

        if promoted_legacy {
            if let Some(ref clue) = result {
                env.storage().persistent().set(&key, clue);

                env.storage().instance().remove(&key);
            }
        } else if result.is_some() {
            env.storage().instance().remove(&key);
        }

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
        Self::get_clue(env, hunt_id, clue_id).ok_or(HuntError::ClueNotFound)
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
        // Store the progress with composite key (hunt_id + player address),

        // in compact form (key fields player/hunt_id are not duplicated).

        let key = Self::progress_key(progress.hunt_id, &progress.player);

        let activated_at = Self::get_hunt(env, progress.hunt_id)
            .map(|h| h.activated_at)
            .unwrap_or(0);

        env.storage()
            .persistent()
            .set(&key, &progress.to_stored(activated_at));

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

    ///   Safely attempts to retrieve and deserialize player progress.

    ///   Returns `Ok(None)` if not registered, `Ok(Some(progress))` if successful,

    ///   or `Err(HuntError::CorruptPlayerProgress)` if storage deserialization fails.

    pub fn try_get_player_progress(
        env: &Env,

        hunt_id: u64,

        player: &Address,
    ) -> Result<Option<PlayerProgress>, HuntError> {
        let key = Self::progress_key(hunt_id, player);

        let activated_at = Self::get_hunt(env, hunt_id)
            .map(|h| h.activated_at)
            .unwrap_or(0);

        let raw_val: Option<Val> = env.storage().persistent().get(&key);

        let val = match raw_val {
            Some(v) => v,

            None => return Ok(None),
        };

        if let Ok(stored) = StoredPlayerProgress::try_from_val(env, &val) {
            return Ok(Some(PlayerProgress::from_stored(
                env,
                stored,
                player.clone(),
                hunt_id,
                activated_at,
            )));
        }

        if let Ok(bytes) = soroban_sdk::Bytes::try_from_val(env, &val) {
            if let Ok(stored) = StoredPlayerProgress::from_xdr(env, &bytes) {
                return Ok(Some(PlayerProgress::from_stored(
                    env,
                    stored,
                    player.clone(),
                    hunt_id,
                    activated_at,
                )));
            }
        }

        Err(HuntError::CorruptPlayerProgress)
    }

    /// Retrieves player progress as an Option.

    /// Returns `None` if not registered or if progress entry is corrupt.

    pub fn get_player_progress(
        env: &Env,

        hunt_id: u64,

        player: &Address,
    ) -> Option<PlayerProgress> {
        Self::try_get_player_progress(env, hunt_id, player)
            .ok()
            .flatten()
    }

    /// Retrieves player progress or returns an error if not found or if entry is corrupt.

    ///

    /// # Arguments

    /// * `env` - The Soroban environment

    /// * `hunt_id` - The hunt the player is registered for

    /// * `player` - The player's address

    ///

    /// # Returns

    /// * `Ok(PlayerProgress)` if progress exists and is valid

    /// * `Err(HuntError::PlayerNotRegistered)` if the player is not registered

    /// * `Err(HuntError::CorruptPlayerProgress)` if progress deserialization fails

    pub fn get_player_progress_or_error(
        env: &Env,

        hunt_id: u64,

        player: &Address,
    ) -> Result<PlayerProgress, HuntError> {
        match Self::try_get_player_progress(env, hunt_id, player)? {
            Some(progress) => Ok(progress),

            None => Err(HuntError::PlayerNotRegistered),
        }
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

    pub fn save_leaderboard_index(env: &Env, hunt_id: u64, entries: &Vec<LeaderboardIndexEntry>) {
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

    fn player_entry_key(hunt_id: u64, index: u32) -> (soroban_sdk::Symbol, u64, u32) {
        (Self::PLAYER_ENTRY_KEY, hunt_id, index)
    }

    fn player_count_key(hunt_id: u64) -> (soroban_sdk::Symbol, u64) {
        (Self::PLAYER_COUNT_KEY, hunt_id)
    }

    fn clue_exists_key(hunt_id: u64, clue_id: u32) -> (soroban_sdk::Symbol, u64, u32) {
        (Self::CLUE_EXISTS_KEY, hunt_id, clue_id)
    }

    fn player_exists_key(hunt_id: u64, player: &Address) -> (soroban_sdk::Symbol, u64, Address) {
        (Self::PLAYER_EXISTS_KEY, hunt_id, player.clone())
    }

    fn leaderboard_key(hunt_id: u64) -> (soroban_sdk::Symbol, u64) {
        (Self::LEADERBOARD_KEY, hunt_id)
    }

    fn required_clues_key(hunt_id: u64) -> (soroban_sdk::Symbol, u64) {
        (Self::REQUIRED_CLUES_KEY, hunt_id)
    }

    /// Key for view-only addresses for a hunt.

    fn view_only_key(hunt_id: u64) -> (soroban_sdk::Symbol, u64) {
        (Self::VIEW_ONLY_KEY, hunt_id)
    }

    /// O(1) membership marker for a view-only address on a hunt.

    fn view_only_member_key(
        hunt_id: u64,

        address: &Address,
    ) -> (soroban_sdk::Symbol, u64, Address) {
        (symbol_short!("VOMEM"), hunt_id, address.clone())
    }

    /// O(1) membership marker for an address in the global view-only list.

    fn global_view_only_member_key(address: &Address) -> (soroban_sdk::Symbol, Address) {
        (symbol_short!("GVOMEM"), address.clone())
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

    /// Extends a canonical per-hunt index key using the same persistent TTL

    /// policy as the hunt and player records it describes.

    fn touch_persistent_index<K: IntoVal<Env, Val>>(env: &Env, key: &K) {
        env.storage().persistent().extend_ttl(
            key,
            PERSISTENT_TTL_THRESHOLD,
            PERSISTENT_TTL_EXTEND_TO,
        );
    }

    /// Copies the pre-migration instance clue index into persistent storage.

    /// The operation is idempotent and intentionally bounded by the contract's

    /// maximum clue count, so it is safe to run lazily on every index read.

    fn migrate_clue_index_from_instance(env: &Env, hunt_id: u64) {
        let legacy_count: u32 = env
            .storage()
            .instance()
            .get(&Self::clue_list_count_key(hunt_id))
            .unwrap_or(0)
            .min(MAX_MIGRATED_CLUE_INDEX_ENTRIES);

        if legacy_count == 0 {
            return;
        }

        let count_key = Self::clue_list_count_key(hunt_id);

        let mut count: u32 = env.storage().persistent().get(&count_key).unwrap_or(0);

        for index in 0..legacy_count {
            let legacy_entry_key = Self::clue_entry_key(hunt_id, index);

            let clue_id: Option<u32> = env.storage().instance().get(&legacy_entry_key);

            if let Some(clue_id) = clue_id {
                let exists_key = Self::clue_exists_key(hunt_id, clue_id);

                let mut already_indexed = false;

                for canonical_index in 0..count {
                    let canonical_key = Self::clue_entry_key(hunt_id, canonical_index);

                    if env.storage().persistent().get::<_, u32>(&canonical_key) == Some(clue_id) {
                        already_indexed = true;

                        break;
                    }
                }

                if !already_indexed {
                    let entry_key = Self::clue_entry_key(hunt_id, count);

                    env.storage().persistent().set(&entry_key, &clue_id);

                    Self::touch_persistent_index(env, &entry_key);

                    count = count.saturating_add(1);
                }

                // Promote the marker even when the entry was already present.

                env.storage().persistent().set(&exists_key, &());

                Self::touch_persistent_index(env, &exists_key);

                env.storage().instance().remove(&exists_key);
            }

            env.storage().instance().remove(&legacy_entry_key);
        }

        if count > 0 {
            env.storage().persistent().set(&count_key, &count);

            Self::touch_persistent_index(env, &count_key);
        }

        env.storage().instance().remove(&count_key);
    }

    /// Adds a clue ID to the persistent clue index for a hunt.

    fn add_clue_to_list(env: &Env, hunt_id: u64, clue_id: u32) {
        Self::migrate_clue_index_from_instance(env, hunt_id);

        let count_key = Self::clue_list_count_key(hunt_id);

        let count: u32 = env.storage().persistent().get(&count_key).unwrap_or(0);

        let exists_key = Self::clue_exists_key(hunt_id, clue_id);

        // The canonical marker is persistent. Verify the corresponding entry

        // as well: a partially expired/migrated index must self-heal instead

        // of permanently hiding a clue behind a stale marker.

        if env.storage().persistent().has(&exists_key) {
            let mut indexed = false;

            for index in 0..count {
                let indexed_key = Self::clue_entry_key(hunt_id, index);

                if env.storage().persistent().get::<_, u32>(&indexed_key) == Some(clue_id) {
                    indexed = true;

                    break;
                }
            }

            if indexed {
                Self::touch_persistent_index(env, &exists_key);

                return;
            }

            env.storage().persistent().remove(&exists_key);
        }

        if env.storage().instance().has(&exists_key) {
            env.storage().persistent().set(&exists_key, &());

            Self::touch_persistent_index(env, &exists_key);

            env.storage().instance().remove(&exists_key);

            return;
        }

        let entry_key = Self::clue_entry_key(hunt_id, count);

        env.storage().persistent().set(&entry_key, &clue_id);

        Self::touch_persistent_index(env, &entry_key);

        env.storage()
            .persistent()
            .set(&count_key, &count.saturating_add(1));

        Self::touch_persistent_index(env, &count_key);

        env.storage().persistent().set(&exists_key, &());

        Self::touch_persistent_index(env, &exists_key);
    }

    fn add_required_clue(env: &Env, hunt_id: u64, clue_id: u32) {
        let key = Self::required_clues_key(hunt_id);

        let mut ids: Vec<u32> = env
            .storage()
            .persistent()
            .get(&key)
            .unwrap_or_else(|| Vec::new(env));

        if ids.first_index_of(clue_id).is_none() {
            ids.push_back(clue_id);

            env.storage().persistent().set(&key, &ids);

            extend_ttl(env, &key, TtlPolicy::Active);
        }
    }

    pub fn set_required_clues(env: &Env, hunt_id: u64, ids: &Vec<u32>) {
        let key = Self::required_clues_key(hunt_id);

        env.storage().persistent().set(&key, ids);

        extend_ttl(env, &key, TtlPolicy::Active);
    }

    pub fn get_required_clues(env: &Env, hunt_id: u64) -> Vec<u32> {
        let key = Self::required_clues_key(hunt_id);

        let result: Option<Vec<u32>> = env.storage().persistent().get(&key);

        if result.is_some() {
            extend_ttl(env, &key, TtlPolicy::Active);
        }

        result.unwrap_or_else(|| Vec::new(env))
    }

    fn get_clue_ids_for_hunt(env: &Env, hunt_id: u64, offset: u32, limit: u32) -> Vec<u32> {
        Self::migrate_clue_index_from_instance(env, hunt_id);

        let count_key = Self::clue_list_count_key(hunt_id);

        let count: u32 = env.storage().persistent().get(&count_key).unwrap_or(0);

        if env.storage().persistent().has(&count_key) {
            Self::touch_persistent_index(env, &count_key);
        }

        let mut ids = Vec::new(env);

        let start = offset;

        let end = core::cmp::min(offset.saturating_add(limit), count);

        if start >= count {
            return ids;
        }

        for i in start..end {
            let entry_key = Self::clue_entry_key(hunt_id, i);

            if let Some(id) = env.storage().persistent().get(&entry_key) {
                Self::touch_persistent_index(env, &entry_key);

                ids.push_back(id);
            }
        }

        ids
    }

    /// Copies legacy instance player-index entries and markers to the

    /// persistent index. The cap protects a migration from a corrupt count.

    fn migrate_player_index_from_instance(env: &Env, hunt_id: u64) {
        let legacy_count: u32 = env
            .storage()
            .instance()
            .get(&Self::player_count_key(hunt_id))
            .unwrap_or(0)
            .min(10_000);

        if legacy_count == 0 {
            return;
        }

        let count_key = Self::player_count_key(hunt_id);

        let mut count: u32 = env.storage().persistent().get(&count_key).unwrap_or(0);

        for index in 0..legacy_count {
            let legacy_key = Self::player_entry_key(hunt_id, index);

            let player: Option<Address> = env.storage().instance().get(&legacy_key);

            if let Some(player) = player {
                let marker = Self::player_exists_key(hunt_id, &player);

                let mut already_listed = false;

                for canonical_index in 0..count {
                    let canonical_key = Self::player_entry_key(hunt_id, canonical_index);

                    if env.storage().persistent().get::<_, Address>(&canonical_key)
                        == Some(player.clone())
                    {
                        already_listed = true;

                        break;
                    }
                }

                if !already_listed {
                    let entry_key = Self::player_entry_key(hunt_id, count);

                    env.storage().persistent().set(&entry_key, &player);

                    Self::touch_persistent_index(env, &entry_key);

                    count = count.saturating_add(1);
                }

                env.storage().persistent().set(&marker, &());

                Self::touch_persistent_index(env, &marker);

                env.storage().instance().remove(&marker);
            }

            env.storage().instance().remove(&legacy_key);
        }

        if count > 0 {
            env.storage().persistent().set(&count_key, &count);

            Self::touch_persistent_index(env, &count_key);
        }

        env.storage().instance().remove(&count_key);
    }

    /// Adds a player to the persistent registration index.

    fn add_player_to_list(env: &Env, hunt_id: u64, player: &Address) {
        Self::migrate_player_index_from_instance(env, hunt_id);

        let count_key = Self::player_count_key(hunt_id);

        let count: u32 = env.storage().persistent().get(&count_key).unwrap_or(0);

        let marker = Self::player_exists_key(hunt_id, player);

        if env.storage().persistent().has(&marker) {
            let mut listed = false;

            for index in 0..count {
                let entry_key = Self::player_entry_key(hunt_id, index);

                if env.storage().persistent().get::<_, Address>(&entry_key) == Some(player.clone())
                {
                    listed = true;

                    break;
                }
            }

            if listed {
                Self::touch_persistent_index(env, &marker);

                return;
            }

            env.storage().persistent().remove(&marker);
        }

        let entry_key = Self::player_entry_key(hunt_id, count);

        env.storage().persistent().set(&entry_key, player);

        Self::touch_persistent_index(env, &entry_key);

        env.storage()
            .persistent()
            .set(&count_key, &count.saturating_add(1));

        Self::touch_persistent_index(env, &count_key);

        env.storage().persistent().set(&marker, &());

        Self::touch_persistent_index(env, &marker);

        // A legacy marker may remain when its old count was already lost.

        env.storage().instance().remove(&marker);
    }

    /// Returns the number of registered players for a hunt.

    pub fn get_player_count(env: &Env, hunt_id: u64) -> u32 {
        Self::migrate_player_index_from_instance(env, hunt_id);

        let count_key = Self::player_count_key(hunt_id);

        let count: u32 = env.storage().persistent().get(&count_key).unwrap_or(0);

        if env.storage().persistent().has(&count_key) {
            Self::touch_persistent_index(env, &count_key);
        }

        count
    }

    // ========== Global Player Statistics ==========

    fn player_completed_count_key(player: &Address) -> (soroban_sdk::Symbol, Address) {
        (Self::PLAYER_HUNTS_KEY, player.clone())
    }

    /// Returns the total number of hunts this player has completed across all hunts.

    pub fn get_player_completed_hunt_count(env: &Env, player: &Address) -> u32 {
        let key = Self::player_completed_count_key(player);

        env.storage().persistent().get(&key).unwrap_or(0)
    }

    /// Increments the player's global completed-hunt counter.

    pub fn increment_player_completed_hunt_count(env: &Env, player: &Address) {
        let key = Self::player_completed_count_key(player);

        let count: u32 = env.storage().persistent().get(&key).unwrap_or(0);

        env.storage()
            .persistent()
            .set(&key, &count.saturating_add(1));

        extend_ttl(env, &key, TtlPolicy::Default);
    }

    // ========== Team Storage Functions ==========

    fn team_key(hunt_id: u64, team_id: u32) -> (soroban_sdk::Symbol, u64, u32) {
        (Self::TEAM_KEY, hunt_id, team_id)
    }

    fn team_count_key(hunt_id: u64) -> (soroban_sdk::Symbol, u64) {
        (Self::TEAM_COUNT_KEY, hunt_id)
    }

    fn player_team_key(hunt_id: u64, player: &Address) -> (soroban_sdk::Symbol, u64, Address) {
        (Self::PLAYER_TEAM_KEY, hunt_id, player.clone())
    }

    fn team_progress_key(hunt_id: u64, team_id: u32) -> (soroban_sdk::Symbol, u64, u32) {
        (Self::TEAM_PROGRESS_KEY, hunt_id, team_id)
    }

    /// Increments and returns the next team ID for a hunt (sequential from 1).

    pub fn next_team_id(env: &Env, hunt_id: u64) -> u32 {
        let key = Self::team_count_key(hunt_id);

        let current: u32 = env.storage().persistent().get(&key).unwrap_or(0);

        let next = current + 1;

        env.storage().persistent().set(&key, &next);

        extend_ttl(env, &key, TtlPolicy::Active);

        next
    }

    /// Returns the number of teams created for a hunt.

    pub fn get_team_count(env: &Env, hunt_id: u64) -> u32 {
        let key = Self::team_count_key(hunt_id);

        env.storage().persistent().get(&key).unwrap_or(0)
    }

    pub fn save_team(env: &Env, team: &Team) {
        let key = Self::team_key(team.hunt_id, team.team_id);

        env.storage().persistent().set(&key, team);

        extend_ttl(env, &key, TtlPolicy::Active);
    }

    pub fn get_team(env: &Env, hunt_id: u64, team_id: u32) -> Option<Team> {
        let key = Self::team_key(hunt_id, team_id);

        env.storage().persistent().get(&key)
    }

    /// Records which team a player belongs to within a hunt.

    pub fn set_player_team(env: &Env, hunt_id: u64, player: &Address, team_id: u32) {
        let key = Self::player_team_key(hunt_id, player);

        env.storage().persistent().set(&key, &team_id);

        extend_ttl(env, &key, TtlPolicy::Active);
    }

    /// Returns the team ID a player belongs to within a hunt, if any.

    pub fn get_player_team(env: &Env, hunt_id: u64, player: &Address) -> Option<u32> {
        let key = Self::player_team_key(hunt_id, player);

        env.storage().persistent().get(&key)
    }

    pub fn save_team_progress(env: &Env, hunt_id: u64, team_id: u32, progress: &TeamProgress) {
        let key = Self::team_progress_key(hunt_id, team_id);

        env.storage().persistent().set(&key, progress);

        extend_ttl(env, &key, TtlPolicy::Active);
    }

    /// Returns team progress, defaulting to empty when never written.

    pub fn get_team_progress(env: &Env, hunt_id: u64, team_id: u32) -> TeamProgress {
        let key = Self::team_progress_key(hunt_id, team_id);

        env.storage()
            .persistent()
            .get(&key)
            .unwrap_or_else(|| TeamProgress {
                completed_clues: Vec::new(env),

                total_score: 0,
            })
    }

    pub fn get_player_addresses_for_hunt(env: &Env, hunt_id: u64) -> Vec<Address> {
        Self::migrate_player_index_from_instance(env, hunt_id);

        let count_key = Self::player_count_key(hunt_id);

        let count: u32 = env.storage().persistent().get(&count_key).unwrap_or(0);

        if env.storage().persistent().has(&count_key) {
            Self::touch_persistent_index(env, &count_key);
        }

        let mut addrs = Vec::new(env);

        for i in 0..count {
            let entry_key = Self::player_entry_key(hunt_id, i);

            if let Some(addr) = env.storage().persistent().get::<_, Address>(&entry_key) {
                Self::touch_persistent_index(env, &entry_key);

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

        let current = Self::get_hunt_counter(env);

        let next = current.saturating_add(1);

        env.storage().persistent().set(&key, &next);

        env.storage().instance().remove(&key);

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

        if let Some(value) = env.storage().persistent().get::<_, u64>(&key) {
            extend_ttl(env, &key, TtlPolicy::Critical);

            env.storage().instance().remove(&key);

            return value;
        }

        // Legacy deployments kept the global counter in instance storage.

        // Promote it before allocating a new ID so upgrades cannot reuse IDs.

        if let Some(value) = env.storage().instance().get::<_, u64>(&key) {
            env.storage().persistent().set(&key, &value);

            env.storage().instance().remove(&key);

            extend_ttl(env, &key, TtlPolicy::Critical);

            return value;
        }

        0
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

        let current = Self::get_clue_counter(env, hunt_id);

        let next = current.saturating_add(1);

        env.storage().persistent().set(&key, &next);

        env.storage().instance().remove(&key);

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

        if let Some(value) = env.storage().persistent().get::<_, u32>(&key) {
            extend_ttl(env, &key, TtlPolicy::Active);

            env.storage().instance().remove(&key);

            return value;
        }

        // Preserve clue IDs when upgrading a contract that stored the counter

        // in instance storage.

        if let Some(value) = env.storage().instance().get::<_, u32>(&key) {
            env.storage().persistent().set(&key, &value);

            env.storage().instance().remove(&key);

            extend_ttl(env, &key, TtlPolicy::Active);

            return value;
        }

        0
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

    // ========== View-Only Access Functions ==========

    /// Adds an address to the view-only list for a specific hunt.

    /// View-only addresses can read hunt data but cannot modify it.

    ///

    /// Membership is tracked with an O(1) per-address marker so `is_view_only`

    /// does not scan the list; the ordered list is kept only for enumeration.

    /// Additions beyond `MAX_VIEW_ONLY_ENTRIES` are rejected.

    ///

    /// # Arguments

    /// * `env` - The Soroban environment

    /// * `hunt_id` - The hunt to grant view-only access for

    /// * `address` - The address to grant view-only access

    ///

    /// # Returns

    /// * `Ok(())` if the address is (now) in the list

    /// * `Err(HuntError::HuntFull)` if the list has reached its maximum size

    pub fn add_view_only(
        env: &Env,

        hunt_id: u64,

        address: &Address,
    ) -> Result<(), crate::errors::HuntError> {
        let member_key = Self::view_only_member_key(hunt_id, address);

        // O(1) de-duplication: an address is either a member or not.

        if env.storage().instance().has(&member_key) {
            return Ok(());
        }

        let key = Self::view_only_key(hunt_id);

        let mut view_only_list = env
            .storage()
            .instance()
            .get::<_, Vec<Address>>(&key)
            .unwrap_or_else(|| Vec::new(env));

        if view_only_list.len() >= MAX_VIEW_ONLY_ENTRIES {
            return Err(crate::errors::HuntError::HuntFull);
        }

        view_only_list.push_back(address.clone());

        env.storage().instance().set(&key, &view_only_list);

        env.storage().instance().set(&member_key, &());

        env.storage()
            .instance()
            .extend_ttl(INSTANCE_TTL_THRESHOLD, INSTANCE_TTL_EXTEND_TO);

        Ok(())
    }

    /// Removes an address from the view-only list for a specific hunt.

    ///

    /// # Arguments

    /// * `env` - The Soroban environment

    /// * `hunt_id` - The hunt to revoke view-only access for

    /// * `address` - The address to revoke view-only access

    pub fn remove_view_only(env: &Env, hunt_id: u64, address: &Address) {
        let member_key = Self::view_only_member_key(hunt_id, address);

        if !env.storage().instance().has(&member_key) {
            return;
        }

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

        env.storage().instance().remove(&member_key);

        env.storage()
            .instance()
            .extend_ttl(INSTANCE_TTL_THRESHOLD, INSTANCE_TTL_EXTEND_TO);
    }

    /// Checks if an address has view-only access for a specific hunt.

    ///

    /// Backed by a per-address membership key, so this is an O(1) lookup and

    /// does not scan the view-only list.

    ///

    /// # Arguments

    /// * `env` - The Soroban environment

    /// * `hunt_id` - The hunt to check view-only access for

    /// * `address` - The address to check

    ///

    /// # Returns

    /// `true` if the address has view-only access, `false` otherwise

    pub fn is_view_only(env: &Env, hunt_id: u64, address: &Address) -> bool {
        env.storage()
            .instance()
            .has(&Self::view_only_member_key(hunt_id, address))
    }

    /// Gets a page of view-only addresses for a specific hunt.

    ///

    /// # Arguments

    /// * `env` - The Soroban environment

    /// * `hunt_id` - The hunt to get view-only addresses for

    /// * `offset` - The number of entries to skip before returning results

    /// * `limit` - The maximum number of entries to return

    ///

    /// # Returns

    /// A vector of addresses with view-only access for the hunt, respecting

    /// the requested `offset`/`limit` window.

    pub fn get_view_only_list(env: &Env, hunt_id: u64, offset: u32, limit: u32) -> Vec<Address> {
        let key = Self::view_only_key(hunt_id);

        let view_only_list = env
            .storage()
            .instance()
            .get::<_, Vec<Address>>(&key)
            .unwrap_or_else(|| Vec::new(env));

        let start = offset;

        let end = core::cmp::min(offset.saturating_add(limit), view_only_list.len());

        let mut result = Vec::new(env);

        if start >= view_only_list.len() {
            return result;
        }

        for i in start..end {
            if let Some(addr) = view_only_list.get(i) {
                result.push_back(addr);
            }
        }

        result
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
        env.storage()
            .instance()
            .get(&Self::PAUSE_KEY)
            .unwrap_or(false)
    }

    // ========== Blacklist Storage Functions ==========

    //

    // Single canonical representation: one persistent key per address,

    // stored under (symbol_short!("BLKLST"), address).  All paths

    // (contract entry-points *and* admin helpers) must go through

    // `set_blacklisted` / `is_blacklisted` defined here.

    fn blacklist_key(creator: &Address) -> (soroban_sdk::Symbol, Address) {
        (symbol_short!("BLKLST"), creator.clone())
    }

    /// Adds `creator` to the blacklist.

    pub fn blacklist_creator(env: &Env, creator: &Address) {
        env.storage()
            .instance()
            .set(&Self::blacklist_key(creator), &true);

        env.storage()
            .instance()
            .extend_ttl(INSTANCE_TTL_THRESHOLD, INSTANCE_TTL_EXTEND_TO);
    }

    /// Removes `creator` from the blacklist.

    pub fn remove_from_blacklist(env: &Env, creator: &Address) {
        env.storage()
            .instance()
            .remove(&Self::blacklist_key(creator));
    }

    /// Returns `true` if `creator` is currently blacklisted.

    /// This is the single canonical reader used by **both** the public

    /// `is_blacklisted` query *and* the `create_hunt` enforcement check.

    pub fn is_blacklisted(env: &Env, creator: &Address) -> bool {
        env.storage()
            .instance()
            .get::<_, bool>(&Self::blacklist_key(creator))
            .unwrap_or(false)
    }

    // ========== Emergency-stop helpers ==========

    /// Returns the IDs of every hunt whose current status is [`HuntStatus::Active`].

    ///

    /// Iterates all hunt IDs from 1 to the current counter value.

    /// The instance-storage cache is used when available for an O(1) status

    /// check per hunt; the method falls back to the full persistent record when

    /// the cache is cold.

    pub fn get_active_hunt_ids(env: &Env) -> Vec<u64> {
        let counter = Self::get_hunt_counter(env);

        let mut active = Vec::new(env);

        for hunt_id in 1..=counter {
            // Prefer the cheap instance-cache path.

            let status = if let Some(cache) = Self::get_hunt_cache(env, hunt_id) {
                cache.status
            } else if let Some(hunt) = Self::get_hunt(env, hunt_id) {
                hunt.status
            } else {
                continue;
            };

            if status == crate::types::HuntStatus::Active {
                active.push_back(hunt_id);
            }
        }

        active
    }

    /// Updates the status of an existing hunt and persists the change.

    ///

    /// Loads the full [`Hunt`] record, sets `hunt.status` to `status`, then

    /// delegates back to [`Self::save_hunt`], which handles TTL selection and

    /// keeps the instance-storage cache coherent with the persistent record.

    ///

    /// # Panics

    /// Does **not** panic if the hunt is missing ΓÇö the call is silently ignored

    /// so that a bulk operation (e.g. emergency-stop-all) can continue with

    /// the remaining hunts.

    pub fn set_hunt_status(env: &Env, hunt_id: u64, status: crate::types::HuntStatus) {
        if let Some(mut hunt) = Self::get_hunt(env, hunt_id) {
            hunt.status = status;

            Self::save_hunt(env, &hunt);
        }
    }

    /// Adds an address to the global view-only list.

    /// Global view-only addresses can read ALL hunt data.

    ///

    /// Membership is tracked with an O(1) per-address marker so

    /// `is_global_view_only` does not scan the list; the ordered list is kept

    /// only for enumeration. Additions beyond `MAX_VIEW_ONLY_ENTRIES` are

    /// rejected.

    ///

    /// # Arguments

    /// * `env` - The Soroban environment

    /// * `address` - The address to grant global view-only access

    ///

    /// # Returns

    /// * `Ok(())` if the address is (now) in the list

    /// * `Err(HuntError::HuntFull)` if the list has reached its maximum size

    pub fn add_global_view_only(
        env: &Env,

        address: &Address,
    ) -> Result<(), crate::errors::HuntError> {
        let member_key = Self::global_view_only_member_key(address);

        if env.storage().instance().has(&member_key) {
            return Ok(());
        }

        let mut view_only_list = env
            .storage()
            .instance()
            .get::<_, Vec<Address>>(&Self::GLOBAL_VIEW_ONLY_KEY)
            .unwrap_or_else(|| Vec::new(env));

        if view_only_list.len() >= MAX_VIEW_ONLY_ENTRIES {
            return Err(crate::errors::HuntError::HuntFull);
        }

        view_only_list.push_back(address.clone());

        env.storage()
            .instance()
            .set(&Self::GLOBAL_VIEW_ONLY_KEY, &view_only_list);

        env.storage().instance().set(&member_key, &());

        env.storage()
            .instance()
            .extend_ttl(INSTANCE_TTL_THRESHOLD, INSTANCE_TTL_EXTEND_TO);

        Ok(())
    }

    /// Removes an address from the global view-only list.

    ///

    /// # Arguments

    /// * `env` - The Soroban environment

    /// * `address` - The address to revoke global view-only access

    pub fn remove_global_view_only(env: &Env, address: &Address) {
        let member_key = Self::global_view_only_member_key(address);

        let mut view_only_list = env
            .storage()
            .instance()
            .get::<_, Vec<Address>>(&Self::GLOBAL_VIEW_ONLY_KEY)
            .unwrap_or_else(|| Vec::new(env));

        if let Some(idx) = view_only_list.first_index_of(address) {
            view_only_list.remove(idx);

            env.storage()
                .instance()
                .set(&Self::GLOBAL_VIEW_ONLY_KEY, &view_only_list);

            env.storage().instance().remove(&member_key);
        }
    }

    /// Checks if an address has global view-only access.

    ///

    /// Backed by a per-address membership key, so this is an O(1) lookup and

    /// does not scan the global view-only list.

    ///

    /// # Arguments

    /// * `env` - The Soroban environment

    /// * `address` - The address to check

    ///

    /// # Returns

    /// `true` if the address has global view-only access, `false` otherwise

    pub fn is_global_view_only(env: &Env, address: &Address) -> bool {
        env.storage()
            .instance()
            .has(&Self::global_view_only_member_key(address))
    }

    /// Gets a page of global view-only addresses.

    ///

    /// # Arguments

    /// * `env` - The Soroban environment

    /// * `offset` - The number of entries to skip before returning results

    /// * `limit` - The maximum number of entries to return

    ///

    /// # Returns

    /// A vector of addresses with global view-only access, respecting the

    /// requested `offset`/`limit` window.

    pub fn get_global_view_only_list(env: &Env, offset: u32, limit: u32) -> Vec<Address> {
        let view_only_list = env
            .storage()
            .instance()
            .get::<_, Vec<Address>>(&Self::GLOBAL_VIEW_ONLY_KEY)
            .unwrap_or_else(|| Vec::new(env));

        let start = offset;

        let end = core::cmp::min(offset.saturating_add(limit), view_only_list.len());

        let mut result = Vec::new(env);

        if start >= view_only_list.len() {
            return result;
        }

        for i in start..end {
            if let Some(addr) = view_only_list.get(i) {
                result.push_back(addr);
            }
        }

        result
    }

    // ========== Ban Storage Functions ==========

    fn ban_key(hunt_id: u64, player: &Address) -> (soroban_sdk::Symbol, u64, Address) {
        (Self::BAN_KEY, hunt_id, player.clone())
    }

    pub fn ban_player(env: &Env, hunt_id: u64, player: &Address) {
        env.storage()
            .persistent()
            .set(&Self::ban_key(hunt_id, player), &());
    }

    pub fn unban_player(env: &Env, hunt_id: u64, player: &Address) {
        env.storage()
            .persistent()
            .remove(&Self::ban_key(hunt_id, player));
    }

    pub fn is_banned(env: &Env, hunt_id: u64, player: &Address) -> bool {
        env.storage()
            .persistent()
            .has(&Self::ban_key(hunt_id, player))
    }

    // ========== Hunt creation rate limiting ==========

    pub fn get_rate_limit_admin(env: &Env) -> Option<Address> {
        env.storage().instance().get(&symbol_short!("HRLADM"))
    }

    pub fn set_rate_limit_admin(env: &Env, admin: &Address) {
        env.storage()
            .instance()
            .set(&symbol_short!("HRLADM"), admin);
    }

    pub fn get_default_hunt_creation_limit(env: &Env) -> u32 {
        env.storage()
            .instance()
            .get(&symbol_short!("HRLDEF"))
            .unwrap_or(crate::rate_limit::DEFAULT_HUNT_CREATION_LIMIT)
    }

    pub fn set_default_hunt_creation_limit(env: &Env, limit: u32) {
        env.storage()
            .instance()
            .set(&symbol_short!("HRLDEF"), &limit);
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

    fn creator_daily_count_key(creator: &Address) -> (soroban_sdk::Symbol, Address) {
        (symbol_short!("HRLCT"), creator.clone())
    }

    pub fn get_creator_daily_hunt_count(env: &Env, creator: &Address, day: u64) -> u32 {
        let key = Self::creator_daily_count_key(creator);

        let stored: Option<CreatorDailyHuntCount> = env.storage().persistent().get(&key);

        match stored {
            Some(entry) if entry.day == day => entry.count,

            _ => 0,
        }
    }

    pub fn set_creator_daily_hunt_count(env: &Env, creator: &Address, day: u64, count: u32) {
        let key = Self::creator_daily_count_key(creator);

        let entry = CreatorDailyHuntCount { day, count };

        env.storage().persistent().set(&key, &entry);
    }

    // ========== Co-Creators Storage Functions ==========

    pub fn get_co_creators(env: &Env, hunt_id: u64) -> Vec<Address> {
        let key = (symbol_short!("COCRTR"), hunt_id);

        env.storage()
            .instance()
            .get(&key)
            .unwrap_or_else(|| Vec::new(env))
    }

    pub fn add_co_creator(env: &Env, hunt_id: u64, address: &Address) {
        let key = (symbol_short!("COCRTR"), hunt_id);

        let mut list = Self::get_co_creators(env, hunt_id);

        if list.first_index_of(address).is_none() {
            list.push_back(address.clone());

            env.storage().instance().set(&key, &list);

            env.storage().instance().extend_ttl(518400, 518400);
        }
    }

    pub fn remove_co_creator(env: &Env, hunt_id: u64, address: &Address) {
        let key = (symbol_short!("COCRTR"), hunt_id);

        let mut list = Self::get_co_creators(env, hunt_id);

        if let Some(idx) = list.first_index_of(address) {
            list.remove(idx);

            env.storage().instance().set(&key, &list);
        }
    }

    pub fn is_authorized_creator_or_co_creator(env: &Env, hunt_id: u64, caller: &Address) -> bool {
        if let Some(hunt) = Self::get_hunt(env, hunt_id) {
            if &hunt.creator == caller {
                return true;
            }

            let co_creators = Self::get_co_creators(env, hunt_id);

            return co_creators.first_index_of(caller).is_some();
        }

        false
    }

    // ========== Storage Garbage Collection (issue #446) ==========

    //

    // Soroban has no key-prefix scan, so a hunt's entries cannot be discovered

    // at runtime ΓÇö they have to be reconstructed from the same key builders

    // that wrote them. That makes this function the authoritative inventory of

    // everything a hunt owns.

    //

    // **If you add a per-hunt storage key, add it here too**, and to the table

    // in `docs/STORAGE_KEYS.md`. A key missing from this list is a permanent

    // leak: once the hunt row is gone there is nothing left to enumerate it

    // from.

    //

    // Counters (`player_count`, `clue_list_count`, `team_count`) are read

    // *before* anything is removed and are deleted last, because they are what

    // the per-entity loops iterate over.

    /// Removes a single persistent key if present, counting the removal.

    fn gc_remove_persistent<K: IntoVal<Env, Val>>(env: &Env, key: &K, count: &mut u32) {
        if env.storage().persistent().has(key) {
            env.storage().persistent().remove(key);

            *count = count.saturating_add(1);
        }
    }

    /// Removes a single instance key if present, counting the removal.

    fn gc_remove_instance<K: IntoVal<Env, Val>>(env: &Env, key: &K, count: &mut u32) {
        if env.storage().instance().has(key) {
            env.storage().instance().remove(key);

            *count = count.saturating_add(1);
        }
    }

    /// Reports how many storage entries a hunt currently owns, without removing

    /// anything. Lets a caller size a sweep before committing to it, and gives

    /// the tests a way to assert that a sweep actually reached zero.

    pub fn count_hunt_storage_entries(env: &Env, hunt_id: u64) -> GcReport {
        Self::sweep_hunt_storage(env, hunt_id, false)
    }

    /// Removes every storage entry belonging to `hunt_id` and reports what went.

    ///

    /// Callers are responsible for authorisation and for checking hunt status ΓÇö

    /// see `HuntyCore::gc_hunt`. This layer is deliberately unconditional so it

    /// can also be exercised directly by tests.

    pub fn gc_hunt_storage(env: &Env, hunt_id: u64) -> GcReport {
        Self::sweep_hunt_storage(env, hunt_id, true)
    }

    /// Shared walk over a hunt's key surface.

    ///

    /// `remove = false` counts what exists; `remove = true` deletes it. Keeping

    /// both behaviours in one function is what stops the counter and the

    /// collector from drifting apart as keys are added.

    fn sweep_hunt_storage(env: &Env, hunt_id: u64, remove: bool) -> GcReport {
        let mut persistent_removed: u32 = 0;

        let mut instance_removed: u32 = 0;

        // Read the counters first ΓÇö the loops below depend on them.

        let persistent_player_count: u32 = env
            .storage()
            .persistent()
            .get(&Self::player_count_key(hunt_id))
            .unwrap_or(0);

        let legacy_player_count: u32 = env
            .storage()
            .instance()
            .get(&Self::player_count_key(hunt_id))
            .unwrap_or(0);

        let player_count = core::cmp::max(persistent_player_count, legacy_player_count);

        let persistent_clue_count: u32 = env
            .storage()
            .persistent()
            .get(&Self::clue_list_count_key(hunt_id))
            .unwrap_or(0);

        let legacy_clue_count: u32 = env
            .storage()
            .instance()
            .get(&Self::clue_list_count_key(hunt_id))
            .unwrap_or(0);

        let clue_count = core::cmp::max(persistent_clue_count, legacy_clue_count);

        let team_count: u32 = env
            .storage()
            .persistent()
            .get(&Self::team_count_key(hunt_id))
            .unwrap_or(0);

        // ΓöÇΓöÇ Per-player entries ΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇ

        for index in 0..player_count {
            let entry_key = Self::player_entry_key(hunt_id, index);

            let persistent_player: Option<Address> = env.storage().persistent().get(&entry_key);

            let legacy_player: Option<Address> = env.storage().instance().get(&entry_key);

            let player = persistent_player.or(legacy_player);

            if let Some(player) = player {
                // progress, team membership, ban flag and the O(1) marker

                let progress = Self::progress_key(hunt_id, &player);

                let player_team = Self::player_team_key(hunt_id, &player);

                let ban = Self::ban_key(hunt_id, &player);

                let exists = Self::player_exists_key(hunt_id, &player);

                if remove {
                    Self::gc_remove_persistent(env, &progress, &mut persistent_removed);

                    Self::gc_remove_persistent(env, &player_team, &mut persistent_removed);

                    Self::gc_remove_persistent(env, &ban, &mut persistent_removed);

                    Self::gc_remove_persistent(env, &exists, &mut persistent_removed);

                    Self::gc_remove_instance(env, &exists, &mut instance_removed);
                } else {
                    for key in [&progress, &player_team] {
                        if env.storage().persistent().has(key) {
                            persistent_removed = persistent_removed.saturating_add(1);
                        }
                    }

                    if env.storage().persistent().has(&ban) {
                        persistent_removed = persistent_removed.saturating_add(1);
                    }

                    if env.storage().persistent().has(&exists) {
                        persistent_removed = persistent_removed.saturating_add(1);
                    }

                    if env.storage().instance().has(&exists) {
                        instance_removed = instance_removed.saturating_add(1);
                    }
                }
            }

            if remove {
                Self::gc_remove_persistent(env, &entry_key, &mut persistent_removed);

                Self::gc_remove_instance(env, &entry_key, &mut instance_removed);
            } else {
                if env.storage().persistent().has(&entry_key) {
                    persistent_removed = persistent_removed.saturating_add(1);
                }

                if env.storage().instance().has(&entry_key) {
                    instance_removed = instance_removed.saturating_add(1);
                }
            }
        }

        // ΓöÇΓöÇ Per-clue entries ΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇ

        for index in 0..clue_count {
            let entry_key = Self::clue_entry_key(hunt_id, index);

            let persistent_clue_id: Option<u32> = env.storage().persistent().get(&entry_key);

            let legacy_clue_id: Option<u32> = env.storage().instance().get(&entry_key);

            // A partially migrated hunt can have the same clue ID in both

            // tiers. Process each distinct ID once, but remove both entries.

            if persistent_clue_id.or(legacy_clue_id).is_some() {
                let clue_id = persistent_clue_id.or(legacy_clue_id).unwrap();

                let clue = Self::clue_key(hunt_id, clue_id);

                let clue_exists = Self::clue_exists_key(hunt_id, clue_id);

                if remove {
                    Self::gc_remove_persistent(env, &clue, &mut persistent_removed);

                    Self::gc_remove_instance(env, &clue, &mut instance_removed);

                    Self::gc_remove_persistent(env, &clue_exists, &mut persistent_removed);

                    Self::gc_remove_instance(env, &clue_exists, &mut instance_removed);
                } else {
                    if env.storage().persistent().has(&clue) {
                        persistent_removed = persistent_removed.saturating_add(1);
                    }

                    if env.storage().instance().has(&clue) {
                        instance_removed = instance_removed.saturating_add(1);
                    }

                    if env.storage().persistent().has(&clue_exists) {
                        persistent_removed = persistent_removed.saturating_add(1);
                    }

                    if env.storage().instance().has(&clue_exists) {
                        instance_removed = instance_removed.saturating_add(1);
                    }
                }
            }

            if remove {
                Self::gc_remove_persistent(env, &entry_key, &mut persistent_removed);

                Self::gc_remove_instance(env, &entry_key, &mut instance_removed);
            } else {
                if env.storage().persistent().has(&entry_key) {
                    persistent_removed = persistent_removed.saturating_add(1);
                }

                if env.storage().instance().has(&entry_key) {
                    instance_removed = instance_removed.saturating_add(1);
                }
            }
        }

        // ΓöÇΓöÇ Per-team entries ΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇ

        // Team ids are handed out sequentially from 1 by `next_team_id`.

        for team_id in 1..=team_count {
            let team = Self::team_key(hunt_id, team_id);

            let progress = Self::team_progress_key(hunt_id, team_id);

            if remove {
                Self::gc_remove_persistent(env, &team, &mut persistent_removed);

                Self::gc_remove_persistent(env, &progress, &mut persistent_removed);
            } else {
                if env.storage().persistent().has(&team) {
                    persistent_removed = persistent_removed.saturating_add(1);
                }

                if env.storage().persistent().has(&progress) {
                    persistent_removed = persistent_removed.saturating_add(1);
                }
            }
        }

        // ΓöÇΓöÇ Per-view-only-address entries ΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇ

        // Each member marker shares the hunt_id, so it must be enumerated from

        // the view-only list (bounded by MAX_VIEW_ONLY_ENTRIES).

        let view_only_list: Vec<Address> = env
            .storage()
            .instance()
            .get(&Self::view_only_key(hunt_id))
            .unwrap_or_else(|| Vec::new(env));

        for viewer in view_only_list.iter() {
            let member_key = Self::view_only_member_key(hunt_id, &viewer);

            if remove {
                Self::gc_remove_instance(env, &member_key, &mut instance_removed);
            } else if env.storage().instance().has(&member_key) {
                instance_removed = instance_removed.saturating_add(1);
            }
        }

        // ΓöÇΓöÇ Hunt-level persistent keys ΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇ

        let hunt = Self::hunt_key(hunt_id);

        let leaderboard = Self::leaderboard_key(hunt_id);

        let required_clues = Self::required_clues_key(hunt_id);

        let clue_counter = Self::clue_counter_key(hunt_id);

        let player_count_key = Self::player_count_key(hunt_id);

        let team_count_key = Self::team_count_key(hunt_id);

        // ΓöÇΓöÇ Hunt-level instance keys ΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇΓöÇ

        let cache = Self::hunt_cache_key(hunt_id);

        let view_only = Self::view_only_key(hunt_id);

        let clue_list_count = Self::clue_list_count_key(hunt_id);

        let co_creators = (symbol_short!("COCRTR"), hunt_id);

        if remove {
            Self::gc_remove_persistent(env, &hunt, &mut persistent_removed);

            Self::gc_remove_persistent(env, &leaderboard, &mut persistent_removed);

            Self::gc_remove_persistent(env, &required_clues, &mut persistent_removed);

            Self::gc_remove_persistent(env, &clue_counter, &mut persistent_removed);

            Self::gc_remove_persistent(env, &clue_list_count, &mut persistent_removed);

            Self::gc_remove_persistent(env, &player_count_key, &mut persistent_removed);

            Self::gc_remove_instance(env, &player_count_key, &mut instance_removed);

            Self::gc_remove_persistent(env, &team_count_key, &mut persistent_removed);

            Self::gc_remove_instance(env, &cache, &mut instance_removed);

            Self::gc_remove_instance(env, &view_only, &mut instance_removed);

            Self::gc_remove_instance(env, &clue_list_count, &mut instance_removed);

            Self::gc_remove_instance(env, &co_creators, &mut instance_removed);
        } else {
            if env.storage().persistent().has(&hunt) {
                persistent_removed = persistent_removed.saturating_add(1);
            }

            if env.storage().persistent().has(&leaderboard) {
                persistent_removed = persistent_removed.saturating_add(1);
            }

            if env.storage().persistent().has(&required_clues) {
                persistent_removed = persistent_removed.saturating_add(1);
            }

            if env.storage().persistent().has(&clue_counter) {
                persistent_removed = persistent_removed.saturating_add(1);
            }

            if env.storage().persistent().has(&clue_list_count) {
                persistent_removed = persistent_removed.saturating_add(1);
            }

            if env.storage().instance().has(&clue_counter) {
                instance_removed = instance_removed.saturating_add(1);
            }

            if env.storage().persistent().has(&player_count_key) {
                persistent_removed = persistent_removed.saturating_add(1);
            }

            if env.storage().instance().has(&player_count_key) {
                instance_removed = instance_removed.saturating_add(1);
            }

            if env.storage().persistent().has(&team_count_key) {
                persistent_removed = persistent_removed.saturating_add(1);
            }

            if env.storage().instance().has(&cache) {
                instance_removed = instance_removed.saturating_add(1);
            }

            if env.storage().instance().has(&view_only) {
                instance_removed = instance_removed.saturating_add(1);
            }

            if env.storage().instance().has(&clue_list_count) {
                instance_removed = instance_removed.saturating_add(1);
            }

            if env.storage().instance().has(&co_creators) {
                instance_removed = instance_removed.saturating_add(1);
            }
        }

        GcReport {
            hunt_id,

            persistent_removed,

            instance_removed,

            total_removed: persistent_removed.saturating_add(instance_removed),

            players_swept: player_count,

            clues_swept: clue_count,

            teams_swept: team_count,
        }
    }
}

#[cfg(test)]

mod index_tier_tests {

    use super::*;

    use crate::HuntyCore;

    use soroban_sdk::{testutils::Address as _, Address, Env, String};

    fn create_active_hunt_with_player(env: &Env) -> (Address, u64, Address) {
        let contract_id = env.register(HuntyCore, ());

        let creator = Address::generate(env);

        let player = Address::generate(env);

        let hunt_id = env.as_contract(&contract_id, || {
            let id = HuntyCore::create_hunt(
                env.clone(),
                creator.clone(),
                String::from_str(env, "Index tier hunt"),
                String::from_str(env, "Index tier regression"),
                None,
                None,
                0,
                None,
                None,
            )
            .unwrap();

            HuntyCore::add_clue(
                env.clone(),
                id,
                String::from_str(env, "Question"),
                String::from_str(env, "answer"),
                10,
                true,
                None,
                None,
            )
            .unwrap();

            HuntyCore::activate_hunt(env.clone(), id, creator.clone()).unwrap();

            HuntyCore::register_player(env.clone(), id, player.clone()).unwrap();

            id
        });

        (contract_id, hunt_id, player)
    }

    #[test]

    fn clue_and_player_indexes_use_persistent_storage() {
        let env = Env::default();

        env.mock_all_auths();

        let (contract_id, hunt_id, player) = create_active_hunt_with_player(&env);

        env.as_contract(&contract_id, || {
            let clue_entry = Storage::clue_entry_key(hunt_id, 0);

            let clue_count = Storage::clue_list_count_key(hunt_id);

            let clue_marker = Storage::clue_exists_key(hunt_id, 1);

            let player_entry = Storage::player_entry_key(hunt_id, 0);

            let player_count = Storage::player_count_key(hunt_id);

            let player_marker = Storage::player_exists_key(hunt_id, &player);

            for key in [clue_entry, clue_count, clue_marker] {
                assert!(env.storage().persistent().has(&key));

                assert!(!env.storage().instance().has(&key));
            }

            for key in [player_entry, player_count, player_marker] {
                assert!(env.storage().persistent().has(&key));

                assert!(!env.storage().instance().has(&key));
            }
        });
    }

    #[test]

    fn legacy_instance_indexes_are_promoted_without_duplicates() {
        let env = Env::default();

        env.mock_all_auths();

        let (contract_id, hunt_id, player) = create_active_hunt_with_player(&env);

        env.as_contract(&contract_id, || {
            let clue_entry = Storage::clue_entry_key(hunt_id, 0);

            let clue_count = Storage::clue_list_count_key(hunt_id);

            let clue_marker = Storage::clue_exists_key(hunt_id, 1);

            let player_entry = Storage::player_entry_key(hunt_id, 0);

            let player_count = Storage::player_count_key(hunt_id);

            let player_marker = Storage::player_exists_key(hunt_id, &player);

            let clue_id: u32 = env.storage().persistent().get(&clue_entry).unwrap();

            env.storage().persistent().remove(&clue_entry);

            env.storage().instance().set(&clue_entry, &clue_id);

            let clue_count_value: u32 = env.storage().persistent().get(&clue_count).unwrap();

            env.storage().persistent().remove(&clue_count);

            env.storage().instance().set(&clue_count, &clue_count_value);

            env.storage().persistent().remove(&clue_marker);

            env.storage().instance().set(&clue_marker, &());

            let player_value: Address = env.storage().persistent().get(&player_entry).unwrap();

            env.storage().persistent().remove(&player_entry);

            env.storage().instance().set(&player_entry, &player_value);

            let player_count_value: u32 = env.storage().persistent().get(&player_count).unwrap();

            env.storage().persistent().remove(&player_count);

            env.storage()
                .instance()
                .set(&player_count, &player_count_value);

            env.storage().persistent().remove(&player_marker);

            env.storage().instance().set(&player_marker, &());

            assert_eq!(Storage::get_player_count(&env, hunt_id), 1);

            assert_eq!(
                Storage::get_clue_ids_for_hunt(&env, hunt_id, 0, 10).len(),
                1
            );

            assert_eq!(
                Storage::get_player_addresses_for_hunt(&env, hunt_id).len(),
                1
            );

            assert!(env.storage().persistent().has(&clue_entry));

            assert!(env.storage().persistent().has(&clue_count));

            assert!(env.storage().persistent().has(&clue_marker));

            assert!(env.storage().persistent().has(&player_entry));

            assert!(env.storage().persistent().has(&player_count));

            assert!(env.storage().persistent().has(&player_marker));

            assert!(!env.storage().instance().has(&clue_entry));

            assert!(!env.storage().instance().has(&clue_count));

            assert!(!env.storage().instance().has(&clue_marker));

            assert!(!env.storage().instance().has(&player_entry));

            assert!(!env.storage().instance().has(&player_count));

            assert!(!env.storage().instance().has(&player_marker));
        });
    }
}
