#![no_std]
use crate::errors::{HuntError, HuntErrorCode};
use crate::storage::Storage;
use crate::types::{
    AnswerIncorrectEvent, CacheStats, Clue, ClueAddedEvent, ClueAliasesAddedEvent,
    ClueCompletedEvent, ClueInfo, CreatorBlacklistedEvent, CreatorRemovedFromBlacklistEvent,
    Hunt, HuntActivatedEvent, HuntCache, HuntCancelledEvent, HuntCompletedEvent,
    HuntCreatedEvent, HuntDeactivatedEvent, HuntStatistics, HuntStatus, LeaderboardEntry,
    LeaderboardIndexEntry, PlayerProgress, PlayerRegisteredEvent, RateLimitStatus,
    RewardClaimFailedEvent, RewardClaimedEvent, RewardConfig, RewardManagerSetEvent,
    TimeBonusConfig,
};
use reward_interface::RewardErrorCode;
use soroban_sdk::{
    contract, contractimpl, Address, Bytes, BytesN, Env, IntoVal, String, Symbol, Val, Vec, TryFromVal,
};
use soroban_sdk::xdr::ToXdr;

const MAX_TITLE_BYTES: u32 = 200;
const MAX_DESCRIPTION_BYTES: u32 = 2000;
const MAX_QUESTION_LENGTH: u32 = 2000;
const MAX_ANSWER_LENGTH: u32 = 256;
const MAX_CLUES_PER_HUNT: u32 = 100;
/// Maximum number of leaderboard entries returned (gas and UX limit).
const MAX_LEADERBOARD_SIZE: u32 = 20;
/// Maximum batch size for paginated list operations (gas protection).
const MAX_BATCH_SIZE: u32 = 50;
/// Default page size for paginated queries.
const DEFAULT_PAGE_SIZE: u32 = 20;
/// Maximum allowed age for a submission envelope before it is considered stale.
const ANSWER_SUBMISSION_WINDOW_SECS: u64 = 300;
/// Small forward-skew allowance so near-simultaneous signing and inclusion does not fail.
const ANSWER_SUBMISSION_FUTURE_SKEW_SECS: u64 = 30;

#[contract]
pub struct HuntyCore;

#[contractimpl]
impl HuntyCore {
    /// Sets the contract admin once. Subsequent calls require current admin auth via set_admin.
    pub fn initialize_admin(env: Env, admin: Address) -> Result<(), HuntErrorCode> {
        admin.require_auth();
        if Storage::get_admin(&env).is_some() {
            return Err(HuntErrorCode::Unauthorized);
        }
        Storage::set_admin(&env, &admin);
        Ok(())
    }

    /// Pauses all player operations (registrations, answers, rewards) globally.
    pub fn pause_contract(env: Env, admin: Address) -> Result<(), HuntErrorCode> {
        Self::require_admin(&env, &admin)?;
        Storage::set_contract_paused(&env, true);
        Ok(())
    }

    /// Resumes all player operations.
    pub fn unpause_contract(env: Env, admin: Address) -> Result<(), HuntErrorCode> {
        Self::require_admin(&env, &admin)?;
        Storage::set_contract_paused(&env, false);
        Ok(())
    }

    /// Returns whether the global contract pause is active.
    pub fn is_contract_paused(env: Env) -> bool {
        Storage::is_contract_paused(&env)
    }

    fn require_admin(env: &Env, admin: &Address) -> Result<(), HuntErrorCode> {
        admin.require_auth();
        let stored_admin = Storage::get_admin(env).ok_or(HuntErrorCode::Unauthorized)?;
        if stored_admin != *admin {
            return Err(HuntErrorCode::Unauthorized);
        }
        Ok(())
    }

    fn ensure_not_paused(env: &Env) -> Result<(), HuntErrorCode> {
        if Storage::is_contract_paused(env) {
            return Err(HuntErrorCode::ContractPaused);
        }
        Ok(())
    }

    // ========== Cache-Aware Helpers ==========

    /// Loads a hunt's cache from instance storage.
    /// On cache miss, falls back to persistent storage, populates the cache, and returns it.
    fn get_hunt_cache_or_load(env: &Env, hunt_id: u64) -> Result<HuntCache, HuntErrorCode> {
        if let Some(cache) = Storage::get_hunt_cache(env, hunt_id) {
            return Ok(cache);
        }
        let hunt = Storage::get_hunt(env, hunt_id).ok_or(HuntErrorCode::HuntNotFound)?;
        let cache = HuntCache::from_hunt(&hunt);
        Storage::save_hunt_cache(env, &hunt);
        Ok(cache)
    }

    /// Validates that a hunt exists and is active using the instance cache (cheaper).
    /// Returns the cache on success so callers can use start_multiplier_bps etc.
    fn validate_hunt_active_cached(env: &Env, hunt_id: u64) -> Result<HuntCache, HuntErrorCode> {
        let cache = Self::get_hunt_cache_or_load(env, hunt_id)?;
        if cache.status != HuntStatus::Active {
            return Err(HuntErrorCode::InvalidHuntStatus);
        }
        let current_time = env.ledger().timestamp();
        if !cache.is_active(current_time) {
            return Err(HuntErrorCode::HuntNotActive);
        }
        Ok(cache)
    }

    /// Validates that a hunt exists and is in draft status using the instance cache.
    fn validate_hunt_draft_cached(env: &Env, hunt_id: u64) -> Result<HuntCache, HuntErrorCode> {
        let cache = Self::get_hunt_cache_or_load(env, hunt_id)?;
        if cache.status != HuntStatus::Draft {
            return Err(HuntErrorCode::InvalidHuntStatus);
        }
        Ok(cache)
    }

    /// Returns aggregate cache-monitoring counters. Higher hit rate = more reads served
    /// from the cheap instance-storage cache instead of persistent storage.
    /// These counters reset across ledger instance TTL boundaries.
    pub fn get_cache_stats(env: Env) -> CacheStats {
        let hits = Storage::get_cache_hits(&env);
        let misses = Storage::get_cache_misses(&env);
        let hit_rate_bps = Storage::get_cache_hit_rate_bps(&env);
        CacheStats { hits, misses, hit_rate_bps }
    }

    /// Returns the HuntCache for a hunt, repopulating it from persistent storage on miss.
    /// Useful for off-chain indexers that want to verify cache parity with persistent state.
    pub fn get_hunt_cache_view(env: Env, hunt_id: u64) -> Result<HuntCache, HuntErrorCode> {
        Self::get_hunt_cache_or_load(&env, hunt_id)
    }

    /// Admin: resets cache hit/miss counters to zero so a new measurement window can start.
    pub fn reset_cache_monitoring(env: Env, admin: Address) -> Result<(), HuntErrorCode> {
        Self::require_admin(&env, &admin)?;
        Storage::reset_cache_counters(&env);
        env.events().publish(
            (Symbol::new(&env, "CacheMonitoringReset"),),
            (admin,),
        );
        Ok(())
    }


    /// Creates a new scavenger hunt with the provided metadata.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `creator` - The address of the hunt creator (typically use env.invoker() from the caller)
    /// * `title` - The title of the hunt (max 200 characters)
    /// * `description` - The description of the hunt (max 2000 characters)
    /// * `start_time` - Optional start timestamp (0 means no start time restriction)
    /// * `end_time` - Optional end timestamp (0 means no end time restriction)
    ///
    /// # Returns
    /// The unique hunt ID of the newly created hunt
    ///
    /// # Errors
    /// * `InvalidTitle` - If title is empty or exceeds maximum length
    /// * `InvalidDescription` - If description exceeds maximum length
    /// * `InvalidAddress` - If creator address is invalid
    pub fn create_hunt(
        env: Env,
        creator: Address,
        title: String,
        description: String,
        _start_time: Option<u64>,
        end_time: Option<u64>,
        max_submissions_per_minute: u32,
        start_multiplier_bps: Option<u32>,
    ) -> Result<u64, HuntErrorCode> {
        monitoring::Monitoring::record_invocation(&env, 50_000, true);
        if Storage::is_creator_blacklisted(&env, &creator) {
            return Err(HuntErrorCode::AddressBlacklisted);
        }

        // Validate and sanitize title/description at byte level
        let title = crate::sanitization::StringSanitizer::sanitize(
            &env,
            &title,
            MAX_TITLE_BYTES,
            false,
        )
        .map_err(|_| HuntErrorCode::InvalidTitle)?;

        let description = crate::sanitization::StringSanitizer::sanitize(
            &env,
            &description,
            MAX_DESCRIPTION_BYTES,
            true,
        )
        .map_err(|_| HuntErrorCode::InvalidDescription)?;

        let current_time = env.ledger().timestamp();
        rate_limit::RateLimiter::check_and_increment(&env, &creator, current_time)?;

        // Generate unique hunt ID
        let hunt_id = Storage::next_hunt_id(&env);

        // Initialize reward config with zero pool
        let reward_config = RewardConfig::new(
            &env,
            0,     // xlm_pool: zero initially
            false, // nft_enabled: false initially
            None,  // nft_contract: None initially
            0,     // max_winners: 0 initially
            0,     // nft_rarity: zero initially
            0,     // nft_tier: zero initially
        );

        // Create the hunt with Draft status
        let hunt = Hunt {
            hunt_id,
            creator: creator.clone(),
            title: title.clone(),
            description: description.clone(),
            status: HuntStatus::Draft,
            created_at: current_time,
            activated_at: 0, // Will be set when hunt is activated
            end_time: end_time.unwrap_or(0),
            reward_config,
            time_bonus_start_bps: None,
            time_bonus_min_bps: None,
            time_bonus_decay_secs: None,
            total_clues: 0, // Empty clue list initially
            required_clues: 0,
            completed_count: 0,
            max_submissions_per_minute,
            start_multiplier_bps: start_multiplier_bps.unwrap_or(20000),
            max_attempts_per_clue: 0,
        };

        // Store the hunt
        Storage::save_hunt(&env, &hunt);

        // Emit HuntCreated event
        let event = HuntCreatedEvent {
            hunt_id,
            creator: creator.clone(),
            title: title.clone(),
        };
        env.events()
            .publish((Symbol::new(&env, "HuntCreated"), hunt_id), event);

        Ok(hunt_id)
    }

    /// Creates a new draft hunt by copying clues from an existing completed hunt.
    ///
    /// The template hunt must already be completed. The copied hunt starts as a fresh
    /// draft with a new hunt ID, creator, title, and description, but reuses the
    /// template's clue questions, hashes, points, and required flags.
    pub fn create_hunt_from_template(
        env: Env,
        template_hunt_id: u64,
        creator: Address,
        title: String,
        description: String,
        start_time: Option<u64>,
        end_time: Option<u64>,
    ) -> Result<u64, HuntErrorCode> {
        let template_hunt =
            Storage::get_hunt(&env, template_hunt_id).ok_or(HuntErrorCode::HuntNotFound)?;

        if template_hunt.status != HuntStatus::Completed {
            return Err(HuntErrorCode::InvalidHuntStatus);
        }

        let hunt_id = Self::create_hunt(
            env.clone(),
            creator.clone(),
            title,
            description,
            start_time,
            end_time,
            template_hunt.max_submissions_per_minute,
            None,
        )?;

        let template_clues = Storage::list_clues_for_hunt(&env, template_hunt_id, 0, MAX_CLUES_PER_HUNT);
        let mut hunt = Storage::get_hunt(&env, hunt_id).ok_or(HuntErrorCode::HuntNotFound)?;
        hunt.max_attempts_per_clue = template_hunt.max_attempts_per_clue;

        for i in 0..template_clues.len() {
            let clue = template_clues.get(i).unwrap();
            let cloned_clue = Clue {
                clue_id: Storage::next_clue_id(&env, hunt_id),
                question: clue.question,
                answer_hashes: clue.answer_hashes,
                points: clue.points,
                is_required: clue.is_required,
                difficulty: clue.difficulty,
            };

            Storage::save_clue(&env, hunt_id, &cloned_clue);

            // Track required clue IDs for gas-efficient completion checks
            if cloned_clue.is_required {
                let mut required_ids = Storage::get_required_clues(&env, hunt_id);
                required_ids.push_back(cloned_clue.clue_id);
                Storage::set_required_clues(&env, hunt_id, &required_ids);
            }

            hunt.total_clues += 1;
            if cloned_clue.is_required {
                hunt.required_clues += 1;
            }

            let event = ClueAddedEvent {
                hunt_id,
                clue_id: cloned_clue.clue_id,
                creator: creator.clone(),
                question: cloned_clue.question.clone(),
                points: cloned_clue.points,
                is_required: cloned_clue.is_required,
                difficulty: cloned_clue.difficulty,
            };
            env.events()
                .publish((Symbol::new(&env, "ClueAdded"), hunt_id, cloned_clue.clue_id), event);
        }

        Storage::save_hunt(&env, &hunt);
        Ok(hunt_id)
    }

    /// Sets an optional time-based scoring bonus for a draft hunt.
    /// The bonus is applied to each clue score as it is completed.
    pub fn set_time_bonus_config(
        env: Env,
        hunt_id: u64,
        caller: Address,
        time_bonus_config: Option<TimeBonusConfig>,
    ) -> Result<(), HuntErrorCode> {
        caller.require_auth();

        let mut hunt = Storage::get_hunt(&env, hunt_id).ok_or(HuntErrorCode::HuntNotFound)?;

        if caller != hunt.creator {
            return Err(HuntErrorCode::Unauthorized);
        }

        if hunt.status != HuntStatus::Draft {
            return Err(HuntErrorCode::InvalidHuntStatus);
        }

        if let Some(config) = time_bonus_config.as_ref() {
            if !config.is_valid() {
                return Err(HuntErrorCode::InvalidTimeBonusConfig);
            }
        }

        match time_bonus_config {
            Some(config) => {
                hunt.time_bonus_start_bps = Some(config.start_multiplier_bps);
                hunt.time_bonus_min_bps = Some(config.min_multiplier_bps);
                hunt.time_bonus_decay_secs = Some(config.decay_duration_secs);
            }
            None => {
                hunt.time_bonus_start_bps = None;
                hunt.time_bonus_min_bps = None;
                hunt.time_bonus_decay_secs = None;
            }
        }
        Storage::save_hunt(&env, &hunt);
        Ok(())
    }

    /// Updates a draft hunt's title and description. Only the hunt creator can update it.
    pub fn update_hunt(
        env: Env,
        hunt_id: u64,
        caller: Address,
        max_attempts_per_clue: u32,
    ) -> Result<(), HuntErrorCode> {
        if max_attempts_per_clue == 0 {
            return Err(HuntErrorCode::InvalidMaxAttempts);
        }

        // Fast validation using instance cache
        let cache = Self::get_hunt_cache_or_load(&env, hunt_id)?;
        if cache.creator != caller {
            return Err(HuntErrorCode::Unauthorized);
        }
        if cache.status != HuntStatus::Draft {
            return Err(HuntErrorCode::InvalidHuntStatus);
        }

        // Load full hunt from persistent for mutation
        let mut hunt = Storage::get_hunt_or_error(&env, hunt_id).map_err(HuntErrorCode::from)?;
        hunt.max_attempts_per_clue = max_attempts_per_clue;
        Storage::save_hunt(&env, &hunt);
        Ok(())
    }

    /// Adds a clue to a hunt. Only the hunt creator can add clues.
    /// Answers are hashed with SHA256 before storage; the hash is never exposed.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `hunt_id` - The hunt to add the clue to
    /// * `question` - The clue question text (max 2000 chars, non-empty)
    /// * `answer` - Plain-text answer; normalized (trimmed, lowercased) then hashed
    /// * `points` - Points awarded for solving this clue
    /// * `is_required` - Whether this clue must be solved to complete the hunt
    ///
    /// # Returns
    /// The sequential clue ID assigned within the hunt
    ///
    /// # Errors
    /// * `HuntNotFound` - Hunt does not exist
    /// * `InvalidHuntStatus` - Hunt is not in Draft
    /// * `Unauthorized` - Caller is not the hunt creator
    /// * `TooManyClues` - Hunt already has max clues
    /// * `InvalidQuestion` - Question empty or too long
    /// * `InvalidAnswer` - Answer empty or too long
    pub fn add_clue(
        env: Env,
        hunt_id: u64,
        question: String,
        answer: String,
        points: u32,
        is_required: bool,
        difficulty: Option<u32>,
    ) -> Result<u32, HuntErrorCode> {
        // Fast validation using instance cache (cheaper than persistent read)
        let cache = Self::get_hunt_cache_or_load(&env, hunt_id)?;
        if cache.status != HuntStatus::Draft {
            return Err(HuntErrorCode::InvalidHuntStatus);
        }
        cache.creator.require_auth();
        if Storage::get_clue_counter(&env, hunt_id) >= MAX_CLUES_PER_HUNT {
            return Err(HuntErrorCode::from(HuntError::TooManyClues {
                hunt_id,
                limit: MAX_CLUES_PER_HUNT,
            }));
        }
        let question = crate::sanitization::StringSanitizer::sanitize(
            &env,
            &question,
            MAX_QUESTION_LENGTH,
            false,
        )
        .map_err(|_| HuntErrorCode::InvalidQuestion)?;
        let clue_id = Storage::next_clue_id(&env, hunt_id);
        let answer_hash = Self::normalize_and_hash_answer(&env, hunt_id, clue_id, &answer)
            .map_err(HuntErrorCode::from)?;
        let mut answer_hashes = Vec::new(&env);
        answer_hashes.push_back(answer_hash);
        let clue = Clue {
            clue_id,
            question: question.clone(),
            answer_hashes,
            points,
            is_required,
            difficulty: difficulty.unwrap_or(1),
        };
        Storage::save_clue(&env, hunt_id, &clue);
        let mut updated = Storage::get_hunt_or_error(&env, hunt_id).map_err(HuntErrorCode::from)?;
        updated.total_clues += 1;
        if is_required {
            updated.required_clues += 1;
        }
        Storage::save_hunt(&env, &updated);
        let event = ClueAddedEvent {
            hunt_id,
            clue_id,
            creator: updated.creator.clone(),
            question,
            points,
            is_required,
            difficulty,
        };
        env.events()
            .publish((Symbol::new(&env, "ClueAdded"), hunt_id, clue_id), event);
        Ok(clue_id)
    }

    /// Adds alternative acceptable answers to an existing clue (synonyms).
    /// Only the hunt creator can add aliases, and only while the hunt is in Draft status.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `hunt_id` - The hunt containing the clue
    /// * `clue_id` - The existing clue to add aliases to
    /// * `answers` - Alternative answers that should also be accepted
    ///
    /// # Errors
    /// * `HuntNotFound` - Hunt does not exist
    /// * `InvalidHuntStatus` - Hunt is not in Draft
    /// * `Unauthorized` - Caller is not the hunt creator
    /// * `ClueNotFound` - Clue does not exist
    /// * `InvalidAnswer` - Any answer is empty or exceeds max length
    pub fn add_clue_aliases(
        env: Env,
        hunt_id: u64,
        clue_id: u32,
        answers: Vec<String>,
    ) -> Result<(), HuntErrorCode> {
        // Fast validation using instance cache
        let cache = Self::validate_hunt_draft_cached(&env, hunt_id)?;
        cache.creator.require_auth();

        let mut clue = Storage::get_clue_or_error(&env, hunt_id, clue_id).map_err(HuntErrorCode::from)?;

        for i in 0..answers.len() {
            let answer = answers.get(i).unwrap();
            let hash =
                Self::normalize_and_hash_answer(&env, hunt_id, clue_id, &answer).map_err(HuntErrorCode::from)?;
            clue.answer_hashes.push_back(hash);
        }

        Storage::save_clue(&env, hunt_id, &clue);

        let event = ClueAliasesAddedEvent {
            hunt_id,
            clue_id,
            creator: cache.creator.clone(),
            aliases_count: answers.len(),
        };
        env.events().publish(
            (Symbol::new(&env, "ClueAliasesAdded"), hunt_id, clue_id),
            event,
        );

        Ok(())
    }

    /// Returns clue information for a hunt/clue. Does not expose the answer hash.
    pub fn get_clue(env: Env, hunt_id: u64, clue_id: u32) -> Result<ClueInfo, HuntErrorCode> {
        let clue =
            Storage::get_clue_or_error(&env, hunt_id, clue_id).map_err(HuntErrorCode::from)?;
        Ok(ClueInfo {
            clue_id: clue.clue_id,
            question: clue.question,
            points: clue.points,
            is_required: clue.is_required,
            difficulty: clue.difficulty,
        })
    }

    /// Returns all clues for a hunt (question, points, required). Answer hashes are not exposed.
    /// This loads all clues; for large hunts use `list_clues_paginated` to limit gas cost.
    /// Estimated gas: O(n) where n = total_clues, ~5_000 gas per clue.
    pub fn list_clues(env: Env, hunt_id: u64) -> Vec<ClueInfo> {
        let clue_count = Storage::get_clue_counter(&env, hunt_id);
        let raw = Storage::list_clues_for_hunt(&env, hunt_id, 0, clue_count);
        let mut out = Vec::new(&env);
        let limit = core::cmp::min(raw.len(), MAX_BATCH_SIZE);
        for i in 0..limit {
            let c = raw.get(i).unwrap();
            out.push_back(ClueInfo {
                clue_id: c.clue_id,
                question: c.question,
                points: c.points,
                is_required: c.is_required,
                difficulty: c.difficulty,
            });
        }
        out
    }

    /// Returns a paginated slice of clues for a hunt. Useful for large hunts to bound gas.
    /// Page is 0-indexed. Max page_size is capped at MAX_BATCH_SIZE (50).
    /// Estimated gas: O(page_size) ~5_000 gas per clue + 10_000 overhead.
    pub fn list_clues_paginated(
        env: Env,
        hunt_id: u64,
        page: u32,
        page_size: u32,
    ) -> Vec<ClueInfo> {
        let effective_page_size = core::cmp::min(page_size, MAX_BATCH_SIZE);
        let offset = page.saturating_mul(effective_page_size);
        let raw = Storage::list_clues_for_hunt(&env, hunt_id, offset, effective_page_size);
        let mut out = Vec::new(&env);
        for i in 0..raw.len() {
            if let Some(c) = raw.get(i) {
                out.push_back(ClueInfo {
                    clue_id: c.clue_id,
                    question: c.question,
                    points: c.points,
                    is_required: c.is_required,
                    difficulty: c.difficulty,
                });
            }
        }
        out
    }

    /// Normalizes answer (trim, lowercase) and returns SHA256 hash as BytesN<32>.
    /// Uses hunt_id and clue_id as salt to prevent rainbow table precomputation.
    /// Hashing scheme: SHA256(hunt_id || clue_id || normalized_answer)
    pub(crate) fn normalize_and_hash_answer(
        env: &Env,
        hunt_id: u64,
        clue_id: u32,
        answer: &String,
    ) -> Result<BytesN<32>, HuntError> {
        let answer = crate::sanitization::StringSanitizer::sanitize(
            env,
            answer,
            MAX_ANSWER_LENGTH,
            false,
        )
        .map_err(|_| HuntError::InvalidAnswer)?;
        let n = answer.len();
        if n == 0 {
            return Err(HuntError::InvalidAnswer);
        }
        let mut buf = [0u8; 256 + 12];
        buf[..8].copy_from_slice(&hunt_id.to_be_bytes());
        buf[8..12].copy_from_slice(&clue_id.to_be_bytes());
        answer.copy_into_slice(&mut buf[12..12 + n as usize]);
        let total_len = 12 + n as usize;
        let mut start = 12usize;
        let mut end = total_len;
        while start < end && Self::is_ascii_space(buf[start]) {
            start += 1;
        }
        while end > start && Self::is_ascii_space(buf[end - 1]) {
            end -= 1;
        }
        if start >= end {
            return Err(HuntError::InvalidAnswer);
        }
        for b in &mut buf[start..end] {
            if b.is_ascii_uppercase() {
                *b += b'a' - b'A';
            }
        }
        let normalized = Bytes::from_slice(env, &buf[..end]);
        let hash = env.crypto().sha256(&normalized);
        Ok(hash.to_bytes())
    }

    #[inline]
    fn is_ascii_space(b: u8) -> bool {
        b.is_ascii_whitespace()
    }

    fn require_admin(env: &Env, admin: &Address) -> Result<(), HuntErrorCode> {
        admin.require_auth();
        let stored_admin = Storage::get_admin(env).ok_or(HuntErrorCode::Unauthorized)?;
        if stored_admin != admin.clone() {
            return Err(HuntErrorCode::Unauthorized);
        }
        Ok(())
    }

    fn validate_rarity(v: u32) -> bool {
        v <= 5
    }

    pub fn activate_hunt(env: Env, hunt_id: u64, caller: Address) -> Result<(), HuntErrorCode> {
        // Fast validation using instance cache
        let cache = Self::get_hunt_cache_or_load(&env, hunt_id)?;
        if caller != cache.creator {
            return Err(HuntErrorCode::Unauthorized);
        }
        if cache.status != HuntStatus::Draft {
            return Err(HuntErrorCode::InvalidHuntStatus);
        }
        if cache.total_clues == 0 {
            return Err(HuntErrorCode::NoCluesAdded);
        }
        if cache.required_clues == 0 {
            return Err(HuntErrorCode::NoRequiredClues);
        }
        if Storage::get_reward_manager(&env).is_some() && cache.max_winners == 0 {
            return Err(HuntErrorCode::NoRewardsConfigured);
        }

        let current_time = env.ledger().timestamp();
        if cache.end_time != 0 && cache.end_time <= current_time {
            return Err(HuntErrorCode::HuntEndTimeInPast);
        }

        // Validation passed — load full hunt from persistent for mutation
        let mut hunt = Storage::get_hunt(&env, hunt_id).ok_or(HuntErrorCode::HuntNotFound)?;

        // Check reward pool has sufficient balance if reward manager is configured
        if let Some(reward_manager_addr) = Storage::get_reward_manager(&env) {
            let mut balance_args: Vec<Val> = Vec::new(&env);
            balance_args.push_back(hunt_id.into_val(&env));
            let pool_balance = match env.try_invoke_contract::<i128, RewardErrorCode>(
                &reward_manager_addr,
                &Symbol::new(&env, "get_pool_balance"),
                balance_args,
            ) {
                Ok(Ok(balance)) => balance,
                _ => return Err(HuntErrorCode::InsufficientRewardPool),
            };
            hunt.reward_config.xlm_pool = pool_balance;
            if !hunt.has_rewards_available() {
                return Err(HuntErrorCode::InsufficientRewardPool);
            }
        }

        hunt.status = HuntStatus::Active;
        hunt.activated_at = current_time;

        Storage::save_hunt(&env, &hunt);

        // Emit HuntActivated event
        let event = HuntActivatedEvent {
            hunt_id,
            activated_at: current_time,
        };

        env.events()
            .publish((Symbol::new(&env, "HuntActivated"), hunt_id), event);

        // Emit HuntStatusChanged event
        Self::emit_hunt_status_changed(
            &env,
            hunt_id,
            HuntStatus::Draft,
            HuntStatus::Active,
            current_time,
        );

        Ok(())
    }
    Storage::set_admin(&env, &admin);

    pub fn deactivate_hunt(env: Env, hunt_id: u64, caller: Address) -> Result<(), HuntErrorCode> {
        // Fast validation using instance cache
        let cache = Self::get_hunt_cache_or_load(&env, hunt_id)?;
        if caller != cache.creator {
            return Err(HuntErrorCode::Unauthorized);
        }
        if cache.status != HuntStatus::Active {
            return Err(HuntErrorCode::InvalidHuntStatus);
        }

        // Validation passed — load full hunt from persistent for mutation
        let mut hunt = Storage::get_hunt(&env, hunt_id).ok_or(HuntErrorCode::HuntNotFound)?;
        hunt.status = HuntStatus::Draft;

        Storage::save_hunt(&env, &hunt);

        let event = HuntDeactivatedEvent { hunt_id };

        env.events()
            .publish((Symbol::new(&env, "HuntDeactivated"), hunt_id), event);

        Self::emit_hunt_status_changed(
            &env,
            hunt_id,
            HuntStatus::Active,
            HuntStatus::Paused,
            env.ledger().timestamp(),
        );

        Ok(())
    }

    pub fn cancel_hunt(env: Env, hunt_id: u64, caller: Address) -> Result<(), HuntErrorCode> {
        caller.require_auth();

        // Fast validation using instance cache
        let cache = Self::get_hunt_cache_or_load(&env, hunt_id)?;
        if caller != cache.creator {
            return Err(HuntErrorCode::Unauthorized);
        }
        if cache.status == HuntStatus::Completed {
            return Err(HuntErrorCode::InvalidHuntStatus);
        }
        if cache.status == HuntStatus::Cancelled {
            return Err(HuntErrorCode::InvalidHuntStatus);
        }

        let old_status = cache.status;

        // Load full hunt from persistent for mutation
        let mut hunt = Storage::get_hunt(&env, hunt_id).ok_or(HuntErrorCode::HuntNotFound)?;

        // Handle refunds for any remaining funded reward pool balance.
        if let Some(reward_manager_addr) = Storage::get_reward_manager(&env) {
            let mut balance_args: Vec<Val> = Vec::new(&env);
            balance_args.push_back(hunt_id.into_val(&env));
            let pool_balance = match env.try_invoke_contract::<i128, RewardErrorCode>(
                &reward_manager_addr,
                &Symbol::new(&env, "get_pool_balance"),
                balance_args,
            ) {
                Ok(Ok(balance)) => balance,
                _ => return Err(HuntErrorCode::RefundFailed),
            };

            if pool_balance > 0 {
                let mut refund_args: Vec<Val> = Vec::new(&env);
                refund_args.push_back(caller.clone().into_val(&env));
                refund_args.push_back(hunt_id.into_val(&env));
                let refund_result = env.try_invoke_contract::<(), RewardErrorCode>(
                    &reward_manager_addr,
                    &Symbol::new(&env, "refund_pool"),
                    refund_args,
                );
                if !matches!(refund_result, Ok(Ok(()))) {
                    return Err(HuntErrorCode::RefundFailed);
                }
            }
        }

        // Cancel hunt
        hunt.status = HuntStatus::Cancelled;

        // Persist
        Storage::save_hunt(&env, &hunt);

        // Emit event
        let event = HuntCancelledEvent { hunt_id };

        env.events()
            .publish((Symbol::new(&env, "HuntCancelled"), hunt_id), event);

        Self::emit_hunt_status_changed(
            &env,
            hunt_id,
            old_status,
            HuntStatus::Cancelled,
            env.ledger().timestamp(),
        );

        Ok(())
    }

    pub fn get_hunt_info(env: Env, hunt_id: u64) -> Result<Hunt, HuntErrorCode> {
        let hunt = Storage::get_hunt(&env, hunt_id).ok_or(HuntErrorCode::HuntNotFound)?;

        match hunt.status {
            HuntStatus::Draft
            | HuntStatus::Active
            | HuntStatus::Completed
            | HuntStatus::Cancelled
            | HuntStatus::Paused => {}
        }

        // Return the full Hunt struct
        Ok(hunt)
    }

    /// Sets the RewardManager contract address for cross-contract reward distribution.
    pub fn set_reward_manager(
        env: Env,
        admin: Address,
        reward_manager: Address,
    ) -> Result<(), HuntErrorCode> {
        Self::require_admin(&env, &admin)?;
        let old_address = Storage::get_reward_manager(&env);
        Storage::set_reward_manager(&env, &reward_manager);
        let event = RewardManagerSetEvent {
            old_address,
            new_address: reward_manager.clone(),
        };
        env.events()
            .publish((Symbol::new(&env, "RewardManagerSet"),), event);
        Ok(())
    }

    /// Sets the admin address. Can only be called once (to initialize).
    /// Subsequent calls require current admin authorization.
    pub fn set_admin(env: Env, new_admin: Address) {
        if let Some(current) = Storage::get_admin(&env) {
            current.require_auth();
        }
        Storage::set_admin(&env, &new_admin);
    }

    /// Blacklists a creator address, preventing them from creating new hunts.
    /// Caller must be the admin.
    pub fn blacklist_creator(env: Env, admin: Address, creator: Address) -> Result<(), HuntErrorCode> {
        admin.require_auth();
        let stored_admin = Storage::get_admin(&env).ok_or(HuntErrorCode::Unauthorized)?;
        if admin != stored_admin {
            return Err(HuntErrorCode::Unauthorized);
        }
        Storage::blacklist_creator(&env, &creator);
        env.events().publish(
            (Symbol::new(&env, "CreatorBlacklisted"), creator.clone()),
            CreatorBlacklistedEvent { creator, admin },
        );
        Ok(())
    }

    /// Removes a creator from the blacklist, restoring their ability to create hunts.
    /// Caller must be the admin.
    pub fn remove_from_blacklist(env: Env, admin: Address, creator: Address) -> Result<(), HuntErrorCode> {
        admin.require_auth();
        let stored_admin = Storage::get_admin(&env).ok_or(HuntErrorCode::Unauthorized)?;
        if admin != stored_admin {
            return Err(HuntErrorCode::Unauthorized);
        }
        Storage::remove_from_blacklist(&env, &creator);
        env.events().publish(
            (Symbol::new(&env, "CreatorRemovedFromBlacklist"), creator.clone()),
            CreatorRemovedFromBlacklistEvent { creator, admin },
        );
        Ok(())
    }

    /// Returns true if the given address is blacklisted.
    pub fn is_blacklisted(env: Env, creator: Address) -> bool {
        Storage::is_blacklisted(&env, &creator)
    }

    /// Completes a hunt for a player and distributes rewards.
    ///
    /// This function verifies that the player has completed all required clues,
    /// then distributes rewards via the RewardManager contract (if configured)
    /// and updates the player's reward status.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `hunt_id` - The hunt ID
    /// * `player` - The player claiming completion/rewards
    ///
    /// # Returns
    /// `Ok(())` on successful reward claim
    ///
    /// # Errors
    /// * `HuntNotFound` - Hunt does not exist
    /// * `PlayerNotRegistered` - Player is not registered
    /// * `HuntNotCompleted` - Player hasn't completed all required clues
    /// * `RewardAlreadyClaimed` - Player already claimed their reward
    /// * `NoRewardsConfigured` - No rewards set up for this hunt
    /// * `InsufficientRewardPool` - All reward slots taken
    /// * `RewardDistributionFailed` - Cross-contract call failed
    pub fn complete_hunt(env: Env, hunt_id: u64, player: Address) -> Result<(), HuntErrorCode> {
    player.require_auth();
    
    if Storage::is_pause_rewards(&env) {
        return Err(HuntErrorCode::RewardsPaused);
    }

    let mut hunt = Storage::get_hunt_or_error(&env, hunt_id).map_err(HuntErrorCode::from)?;

    let mut progress = Storage::get_player_progress_or_error(&env, hunt_id, &player)
        .map_err(HuntErrorCode::from)?;

        // Verify the player has completed all required clues
        if !progress.is_completed {
            return Err(HuntErrorCode::HuntNotCompleted);
        }

        // Prevent double-claiming
        if progress.reward_claimed {
            return Err(HuntErrorCode::RewardAlreadyClaimed);
        }

        // Example migration logic - extend this for future schema changes
        let mut steps = 0u32;

        if current < 2 && target_version >= 2 {
            // Example: Migrate old player progress structure
            Self::migrate_v1_to_v2(&env, dry_run)?;
            steps += 1;
        }

        let reward_amount = hunt.reward_config.reward_per_winner();
        let nft_awarded = hunt.reward_config.nft_enabled;

        if !Self::validate_rarity(hunt.reward_config.nft_rarity) {
            return Err(HuntErrorCode::InvalidRarity);
        }

        // Call RewardManager if configured and there are rewards to distribute
        if let Some(reward_manager_addr) = Storage::get_reward_manager(&env) {
            let xlm_amount = if reward_amount > 0 {
                Some(reward_amount)
            } else {
                None
            };
            let (nft_contract, nft_title, nft_desc, nft_uri, nft_hunt_title) = if nft_awarded {
                hunt.reward_config
                    .nft_contract
                    .clone()
                    .map(|nft_contract| {
                        (
                            Some(nft_contract),
                            hunt.title.clone(),
                            hunt.description.clone(),
                            String::from_str(&env, ""),
                            hunt.title.clone(),
                        )
                    })
                    .unwrap_or((
                        None,
                        String::from_str(&env, ""),
                        String::from_str(&env, ""),
                        String::from_str(&env, ""),
                        String::from_str(&env, ""),
                    ))
            } else {
                (
                    None,
                    String::from_str(&env, ""),
                    String::from_str(&env, ""),
                    String::from_str(&env, ""),
                    String::from_str(&env, ""),
                )
            };
            let rm_reward_config = reward_interface::RewardConfig {
                xlm_amount,
                nft_contract,
                nft_title,
                nft_description: nft_desc,
                nft_image_uri: nft_uri,
                nft_hunt_title,
                nft_rarity: hunt.reward_config.nft_rarity,
                nft_tier: hunt.reward_config.nft_tier,
            };

            // Only call RewardManager when there is at least one reward type
            if rm_reward_config.is_valid() {
                let mut args: Vec<Val> = Vec::new(&env);
                args.push_back(hunt_id.into_val(env));
                args.push_back(player.clone().into_val(env));
                args.push_back(rm_reward_config.into_val(env));

                let result = env.try_invoke_contract::<(), RewardErrorCode>(
                    &reward_manager_addr,
                    &Symbol::new(&env, "distribute_rewards"),
                    args,
                );
                if !matches!(result, Ok(Ok(()))) {
                    return Err(HuntErrorCode::RewardDistributionFailed);
                }
            }
        }

        // Update player progress
        progress.reward_claimed = true;
        Storage::save_player_progress(&env, &progress);

        // Update hunt reward config
        hunt.reward_config.claimed_count += 1;
        Storage::save_hunt(&env, &hunt);

        // Emit RewardClaimedEvent
        let event = RewardClaimedEvent {
            hunt_id,
            player: player.clone(),
            xlm_amount: reward_amount,
            nft_awarded,
        };
        env.events()
            .publish((Symbol::new(&env, "RewardClaimed"), hunt_id), event);

        Ok(())
    }

    /// Registers a player for an active hunt. The caller must pass their address and authorize;
    /// only that identity can register themselves. Initializes player progress and prevents
    /// duplicate registrations. Registration is only allowed while the hunt is active and
    /// (if set) before end_time.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `hunt_id` - The hunt to register for
    /// * `player` - The address of the player (must authorize the call via require_auth)
    ///
    /// # Returns
    /// `Ok(())` on success
    ///
    /// # Errors
    /// * `HuntNotFound` - Hunt does not exist
    /// * `InvalidHuntStatus` - Hunt is not in Active status
    /// * `HuntNotActive` - Hunt has ended (past end_time)
    /// * `DuplicateRegistration` - Player is already registered for this hunt
    pub fn register_player(env: Env, hunt_id: u64, player: Address) -> Result<(), HuntErrorCode> {
        player.require_auth();

        if Storage::is_pause_registrations(&env) {
            return Err(HuntErrorCode::RegistrationsPaused);
        }

        let current_time = env.ledger().timestamp();

        // Cache read: cheaper than loading full Hunt from persistent storage
        let _cache = Self::validate_hunt_active_cached(&env, hunt_id)?;

        if Storage::get_player_progress(&env, hunt_id, &player).is_some() {
            return Err(HuntErrorCode::DuplicateRegistration);
        }
        // Add data transformation logic here, e.g.:
        // - Update existing Hunt structs
        // - Re-key old storage entries
        // - Add new fields with defaults
        Ok(())
    }

    /// Verifies a candidate answer without recording progress or emitting events.
    pub fn preview_answer(
        env: Env,
        hunt_id: u64,
        clue_id: u32,
        player: Address,
        answer: String,
    ) -> bool {
        let Ok(cache) = Self::validate_hunt_active_cached(&env, hunt_id) else {
            return false;
        };

        if Storage::get_player_progress(&env, hunt_id, &player).is_none() {
            return false;
        }

        let Some(clue) = Storage::get_clue(&env, hunt_id, clue_id) else {
            return false;
        };
        let Ok(submitted_hash) = Self::normalize_and_hash_answer(&env, hunt_id, clue_id, &answer) else {
            return false;
        };

        clue.answer_hashes.contains(&submitted_hash)
    }

    /// This function verifies the submitted answer by hashing it and comparing
    /// with the stored answer hash. If correct, updates player progress and emits
    /// success events. If incorrect, emits an analytics event and returns an error.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `hunt_id` - The hunt ID
    /// * `clue_id` - The clue ID to answer
    /// * `player` - The address of the player submitting the answer
    /// * `answer` - The plain-text answer submission
    /// * `submission_nonce` - Caller-chosen unique nonce for this submission envelope
    /// * `submitted_at` - Client timestamp captured when the submission was signed
    ///
    /// # Returns
    /// `Ok(())` on successful answer verification and progress update
    ///
    /// # Errors
    /// * `HuntNotFound` - Hunt does not exist
    /// * `HuntNotActive` - Hunt is not currently active or has ended
    /// * `PlayerNotRegistered` - Player has not registered for this hunt
    /// * `ClueNotFound` - Clue does not exist in this hunt
    /// * `ClueAlreadyCompleted` - Player has already completed this clue
    /// * `InvalidAnswer` - Submitted answer does not match the stored hash
    /// * `DuplicateSubmission` - Submission nonce/timestamp envelope was already processed
    /// * `SubmissionExpired` - Submission timestamp is too old or too far in the future
    ///
    /// # Events
    /// * `ClueCompleted` - Emitted when answer is correct
    /// * `HuntCompleted` - Emitted when all required clues are completed
    /// * `AnswerIncorrect` - Emitted when answer is wrong (for analytics)
    pub(crate) fn calculate_score(
        hunt: &Hunt,
        clue: &Clue,
        started_at: u64,
        completed_at: u64,
    ) -> u32 {
        let elapsed = completed_at.saturating_sub(started_at);
        let decrease_steps = elapsed / 50; // Decrease every 50 seconds
        let decrease_bps = decrease_steps * 5000; // 5000 bps = 0.5x per step
        let multiplier_bps = core::cmp::max(
            10000, // Minimum 1x
            hunt.start_multiplier_bps.saturating_sub(decrease_bps as u32),
        );
        let base_points = clue.points.saturating_mul(clue.difficulty);
        (base_points as u64 * multiplier_bps as u64 / 10000) as u32
    }

    /// Score calculation variant that reads start_multiplier_bps from HuntCache
    /// instead of the full Hunt struct. Avoids a persistent storage read when
    /// the cache is already loaded.
    fn calculate_score_from_cache(
        cache: &HuntCache,
        clue: &Clue,
        started_at: u64,
        completed_at: u64,
    ) -> u32 {
        let elapsed = completed_at.saturating_sub(started_at);
        let decrease_steps = elapsed / 50;
        let decrease_bps = decrease_steps * 5000;
        let multiplier_bps = core::cmp::max(
            10000,
            cache.start_multiplier_bps.saturating_sub(decrease_bps as u32),
        );
        let base_points = clue.points.saturating_mul(clue.difficulty);
        (base_points as u64 * multiplier_bps as u64 / 10000) as u32
    }

    pub fn submit_answer(
        env: Env,
        hunt_id: u64,
        clue_id: u32,
        player: Address,
        answer: String,
        submission_nonce: u64,
        submitted_at: u64,
    ) -> Result<(), HuntErrorCode> {
        // Require player authorization
        player.require_auth();
        
        if Storage::is_pause_answers(&env) {
            return Err(HuntErrorCode::AnswersPaused);
        }

        let current_time = env.ledger().timestamp();

        // Fast validation using instance cache (cheaper than persistent read)
        let cache = Self::validate_hunt_active_cached(&env, hunt_id)?;

        if Storage::is_banned(&env, hunt_id, &player) {
            return Err(HuntErrorCode::BannedPlayer);
        }

        Self::validate_submission_timestamp(current_time, submitted_at)
            .map_err(HuntErrorCode::from)?;
        Self::assert_submission_not_replayed(
            &env,
            hunt_id,
            clue_id,
            &player,
            submission_nonce,
            submitted_at,
            current_time,
        )
        .map_err(HuntErrorCode::from)?;

        Storage::save_processed_submission(
            &env,
            hunt_id,
            clue_id,
            &player,
            submission_nonce,
            submitted_at,
            submitted_at.saturating_add(ANSWER_SUBMISSION_WINDOW_SECS),
        );

        let mut progress = Storage::get_player_progress(&env, hunt_id, &player)
            .ok_or(HuntErrorCode::PlayerNotRegistered)?;

        let clue = Storage::get_clue(&env, hunt_id, clue_id).ok_or(HuntErrorCode::ClueNotFound)?;

        if progress.has_completed_clue(clue_id) {
            return Err(HuntErrorCode::ClueAlreadyCompleted);
        }

        if cache.max_submissions_per_minute > 0 {
            let mut updated_submissions = Vec::new(&env);
            for i in 0..progress.recent_submissions.len() {
                let ts = progress.recent_submissions.get(i).unwrap();
                if current_time < ts + 60 {
                    updated_submissions.push_back(ts);
                }
            }
            progress.recent_submissions = updated_submissions;

            if progress.recent_submissions.len() >= cache.max_submissions_per_minute {
                let oldest_ts = progress.recent_submissions.get(0).unwrap();
                let elapsed = current_time.saturating_sub(oldest_ts);
                let cooldown_remaining = 60u64.saturating_sub(elapsed);
                return Err(HuntErrorCode::from(HuntError::RateLimitExceeded {
                    cooldown_remaining,
                }));
            }
        }

        let submitted_hash = Self::normalize_and_hash_answer(&env, hunt_id, clue_id, &answer)
            .map_err(HuntErrorCode::from)?;

        let mut answer_correct = false;
        for i in 0..clue.answer_hashes.len() {
            if clue.answer_hashes.get(i).unwrap() == submitted_hash {
                answer_correct = true;
                break;
            }
        }

        if !answer_correct {
            // Track and enforce attempt limit
            if hunt.max_attempts_per_clue > 0 {
                let attempts = progress.failed_attempts.get(clue_id).unwrap_or(0) + 1;
                progress.failed_attempts.set(clue_id, attempts);
                Storage::save_player_progress(&env, &progress);
                if attempts >= hunt.max_attempts_per_clue {
                    return Err(HuntErrorCode::MaxAttemptsExceeded);
                }
            } else {
                Storage::save_player_progress(&env, &progress);
            }
            let incorrect_event = AnswerIncorrectEvent {
                hunt_id,
                player: player.clone(),
                clue_id,
                timestamp: current_time,
            };
            env.events().publish(
                (Symbol::new(&env, "AnswerIncorrect"), hunt_id, clue_id),
                incorrect_event,
            );
            return Err(HuntErrorCode::InvalidAnswer);
        }

        let score = Self::calculate_score_from_cache(&cache, &clue, progress.started_at, current_time);
        progress.complete_clue(&env, clue_id, score)?;

        if cache.max_submissions_per_minute > 0 {
            progress.recent_submissions = Vec::new(&env);
        }

        let all_required_completed =
            Self::check_all_required_clues_completed(&env, hunt_id, &progress);

        // If all required clues completed, mark hunt as completed for this player
        if all_required_completed && !progress.is_completed {
            progress.is_completed = true;
            progress.completed_at = current_time;

            // Load full hunt from persistent only when we need to mutate it
            let mut hunt_mut =
                Storage::get_hunt(&env, hunt_id).ok_or(HuntErrorCode::HuntNotFound)?;
            hunt_mut.completed_count += 1;
            let rank = hunt_mut.completed_count;
            Storage::save_hunt(&env, &hunt_mut);
            let hunt_completed_event = HuntCompletedEvent {
                hunt_id,
                player: player.clone(),
                total_score: progress.total_score,
                completion_time: current_time,
                completion_rank: rank,
            };
            env.events().publish(
                (Symbol::new(&env, "HuntCompleted"), hunt_id),
                hunt_completed_event,
            );
        }

        Storage::save_player_progress(&env, &progress);
        Self::update_leaderboard_index(&env, &progress);

        let clue_completed_event = ClueCompletedEvent {
            hunt_id,
            player: player.clone(),
            clue_id,
            points_earned: score,
        };
        env.events().publish(
            (Symbol::new(&env, "ClueCompleted"), hunt_id, clue_id),
            clue_completed_event,
        );

        Ok(())
    }

    /// Variant of `submit_answer` which accepts a precomputed SHA256 answer hash.
    /// This avoids on-chain normalization and hashing when the client supplies
    /// the correctly computed `answer_hash = SHA256(hunt_id || clue_id || normalized_answer)`.
    /// Use this from off-chain callers that can perform normalization+hashing cheaply.
    pub fn submit_answer_with_hash(
        env: Env,
        hunt_id: u64,
        clue_id: u32,
        player: Address,
        answer_hash: BytesN<32>,
        submission_nonce: u64,
        submitted_at: u64,
    ) -> Result<(), HuntErrorCode> {
        // Require player authorization
        player.require_auth();

        if Storage::is_pause_answers(&env) {
            return Err(HuntErrorCode::AnswersPaused);
        }

        let current_time = env.ledger().timestamp();

        // Fast validation using instance cache (cheaper than persistent read)
        let cache = Self::validate_hunt_active_cached(&env, hunt_id)?;

        if Storage::is_banned(&env, hunt_id, &player) {
            return Err(HuntErrorCode::BannedPlayer);
        }

        Self::validate_submission_timestamp(current_time, submitted_at)
            .map_err(HuntErrorCode::from)?;
        Self::assert_submission_not_replayed(
            &env,
            hunt_id,
            clue_id,
            &player,
            submission_nonce,
            submitted_at,
            current_time,
        )
        .map_err(HuntErrorCode::from)?;

        Storage::save_processed_submission(
            &env,
            hunt_id,
            clue_id,
            &player,
            submission_nonce,
            submitted_at,
            submitted_at.saturating_add(ANSWER_SUBMISSION_WINDOW_SECS),
        );

        let mut progress = Storage::get_player_progress(&env, hunt_id, &player)
            .ok_or(HuntErrorCode::PlayerNotRegistered)?;

        let clue = Storage::get_clue(&env, hunt_id, clue_id).ok_or(HuntErrorCode::ClueNotFound)?;

        if progress.has_completed_clue(clue_id) {
            return Err(HuntErrorCode::ClueAlreadyCompleted);
        }

        if cache.max_submissions_per_minute > 0 {
            let mut updated_submissions = Vec::new(&env);
            for i in 0..progress.recent_submissions.len() {
                let ts = progress.recent_submissions.get(i).unwrap();
                if current_time < ts + 60 {
                    updated_submissions.push_back(ts);
                }
            }
            progress.recent_submissions = updated_submissions;

            if progress.recent_submissions.len() >= cache.max_submissions_per_minute {
                let oldest_ts = progress.recent_submissions.get(0).unwrap();
                let elapsed = current_time.saturating_sub(oldest_ts);
                let cooldown_remaining = 60u64.saturating_sub(elapsed);
                return Err(HuntErrorCode::from(HuntError::RateLimitExceeded {
                    cooldown_remaining,
                }));
            }
        }

        if answer_hash != clue.answer_hash {
            if cache.max_submissions_per_minute > 0 {
                progress.recent_submissions.push_back(current_time);
                Storage::save_player_progress(&env, &progress);
            }
            let incorrect_event = AnswerIncorrectEvent {
                hunt_id,
                player: player.clone(),
                clue_id,
                timestamp: current_time,
            };
            env.events().publish(
                (Symbol::new(&env, "AnswerIncorrect"), hunt_id, clue_id),
                incorrect_event,
            );
            return Err(HuntErrorCode::InvalidAnswer);
        }

        let score = Self::calculate_score_from_cache(&cache, &clue, progress.started_at, current_time);
        progress.complete_clue(&env, clue_id, score)?;

        if cache.max_submissions_per_minute > 0 {
            progress.recent_submissions = Vec::new(&env);
        }

        let all_required_completed =
            Self::check_all_required_clues_completed(&env, hunt_id, &progress);

        if all_required_completed && !progress.is_completed {
            progress.is_completed = true;
            progress.completed_at = current_time;

            let mut hunt_mut =
                Storage::get_hunt(&env, hunt_id).ok_or(HuntErrorCode::HuntNotFound)?;
            hunt_mut.completed_count += 1;
            let rank = hunt_mut.completed_count;
            Storage::save_hunt(&env, &hunt_mut);
            let hunt_completed_event = HuntCompletedEvent {
                hunt_id,
                player: player.clone(),
                total_score: progress.total_score,
                completion_time: current_time,
                completion_rank: rank,
            };
            env.events().publish(
                (Symbol::new(&env, "HuntCompleted"), hunt_id),
                hunt_completed_event,
            );
        }

        Storage::save_player_progress(&env, &progress);

        let clue_completed_event = ClueCompletedEvent {
            hunt_id,
            player: player.clone(),
            clue_id,
            points_earned: score,
        };
        env.events().publish(
            (Symbol::new(&env, "ClueCompleted"), hunt_id, clue_id),
            clue_completed_event,
        );

        Ok(())
    }

    fn completion_rank(env: &Env, hunt_id: u64) -> u32 {
        let players = Storage::get_hunt_players(env, hunt_id);
        let mut completed_players = 0u32;
        for i in 0..players.len() {
            let progress = players.get(i).unwrap();
            if progress.is_completed {
                completed_players += 1;
            }
        }
        completed_players.saturating_add(1)
    }

    fn validate_submission_timestamp(
        current_time: u64,
        submitted_at: u64,
    ) -> Result<(), HuntError> {
        if submitted_at > current_time.saturating_add(ANSWER_SUBMISSION_FUTURE_SKEW_SECS) {
            return Err(HuntError::SubmissionExpired {
                submitted_at,
                current_time,
            });
        }
        if current_time.saturating_sub(submitted_at) > ANSWER_SUBMISSION_WINDOW_SECS {
            return Err(HuntError::SubmissionExpired {
                submitted_at,
                current_time,
            });
        }
        Ok(())
    }

    fn assert_submission_not_replayed(
        env: &Env,
        hunt_id: u64,
        clue_id: u32,
        player: &Address,
        submission_nonce: u64,
        submitted_at: u64,
        current_time: u64,
    ) -> Result<(), HuntError> {
        if let Some(expires_at) = Storage::get_processed_submission_expiry(
            env,
            hunt_id,
            clue_id,
            player,
            submission_nonce,
            submitted_at,
        ) {
            if current_time <= expires_at {
                return Err(HuntError::DuplicateSubmission { hunt_id, clue_id });
            }

            Storage::remove_processed_submission(
                env,
                hunt_id,
                clue_id,
                player,
                submission_nonce,
                submitted_at,
            );
        }

        Ok(())
    }

    /// Checks if a player has completed all required clues for a hunt.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `hunt_id` - The hunt ID
    /// * `progress` - The player's progress data
    ///
    /// # Returns
    /// `true` if all required clues are completed, `false` otherwise
    fn check_all_required_clues_completed(
        env: &Env,
        hunt_id: u64,
        progress: &PlayerProgress,
    ) -> bool {
        // First get the hunts required clue count
        let hunt = match Storage::get_hunt(env, hunt_id) {
            Some(h) => h,
            None => return false,
        };

        if hunt.required_clues == 0 {
            return true;
        }

        // Quick early exit: player hasn't completed enough clues total
        if progress.completed_clues.len() < hunt.required_clues as u32 {
            return false;
        }

        // Load only the required clue IDs (much cheaper than loading full clues)
        let required_ids = Storage::get_required_clues(env, hunt_id);

        // If the list is empty but hunt has required clues, fall back to scanning
        // all clues (backward compatibility for pre-migration hunts)
        if required_ids.is_empty() {
            let clue_count = Storage::get_clue_counter(env, hunt_id);
            let all_clues = Storage::list_clues_for_hunt(env, hunt_id, 0, clue_count);
            for i in 0..all_clues.len() {
                let clue = all_clues.get(i).unwrap();
                if clue.is_required && !progress.has_completed_clue(clue.clue_id) {
                    return false;
                }
            }
            return true;
        }

        // Fast path: check only the required clue IDs
        for i in 0..required_ids.len() {
            let cid = required_ids.get(i).unwrap();
            if !progress.has_completed_clue(cid) {
                return false;
            }
        }

        true
    }

    /// Returns player progress for a hunt (read-only).
    /// Includes completed clues, score, and completion status.
    /// Returns error if player is not registered.
    pub fn get_player_progress(
        env: Env,
        hunt_id: u64,
        player: Address,
    ) -> Result<PlayerProgress, HuntErrorCode> {
        Storage::get_player_progress(&env, hunt_id, &player)
            .ok_or(HuntErrorCode::PlayerNotRegistered)
    }

    /// Returns the list of clue IDs that the player has completed for a hunt (read-only).
    /// Useful for UI to show progress. Returns empty vec if player is not registered.
    pub fn get_completed_clues(env: Env, hunt_id: u64, player: Address) -> Vec<u32> {
        match Storage::get_player_progress(&env, hunt_id, &player) {
            Some(progress) => progress.completed_clues,
            None => Vec::new(&env),
        }
    }

    /// Returns the total number of hunts created (read-only).
    pub fn get_hunt_count(env: Env) -> u64 {
        Storage::get_hunt_counter(&env)
    }

    /// Returns ranked players for a hunt with pagination support (read-only).
    /// Sorted by score descending, then by completion time ascending (earlier = better).
    /// Limit is capped at 20 to control gas. Returns error if hunt does not exist.
    pub fn get_hunt_leaderboard(
        env: Env,
        hunt_id: u64,
        limit: u32,
    ) -> Result<Vec<LeaderboardEntry>, HuntErrorCode> {
        // Cache existence check (cheaper than loading full Hunt)
        Storage::get_hunt_cache(&env, hunt_id).ok_or(HuntErrorCode::HuntNotFound)?;
        let effective_limit = core::cmp::min(limit, MAX_LEADERBOARD_SIZE);
        let entries = Storage::get_leaderboard_index(&env, hunt_id);
        let mut result = Vec::new(&env);
        let result_len = core::cmp::min(effective_limit, entries.len());
        for i in 0..result_len {
            let entry = entries.get(i).unwrap();
            result.push_back(LeaderboardEntry {
                rank: i + 1,
                player: entry.player,
                score: entry.score,
                completed_at: entry.completed_at,
                is_completed: entry.is_completed,
            });
        }

        Ok(result)
    }

    /// Scans a bounded window of registered players for a hunt and returns
    /// their compact rows. This method enables clients to page through all
    /// registered players in multiple calls (bounded by `MAX_LEADERBOARD_SCAN_SIZE`)
    /// and merge results off-chain to build a full leaderboard without a single
    /// large on-chain scan.
    pub fn get_hunt_leaderboard_window(
        env: Env,
        hunt_id: u64,
        start_index: u32,
        window_size: u32,
    ) -> Result<crate::types::LeaderboardWindow, HuntErrorCode> {
        // Cache existence check (cheaper than loading full Hunt)
        Storage::get_hunt_cache(&env, hunt_id).ok_or(HuntErrorCode::HuntNotFound)?;
        let queried_at = env.ledger().timestamp();
        let players = Storage::get_hunt_players(&env, hunt_id);
        let total_players = players.len();

        let start = core::cmp::min(start_index, total_players);
        let capped_window = core::cmp::min(window_size, MAX_LEADERBOARD_SCAN_SIZE);
        let end = core::cmp::min(start.saturating_add(capped_window), total_players);

        let mut rows = Vec::new(&env);
        for i in start..end {
            let p = players.get(i as u32).unwrap();
            rows.push_back(crate::types::LeaderboardRow {
                index: i,
                player: p.player.clone(),
                score: p.total_score,
                completed_at: p.completed_at,
                is_completed: p.is_completed,
            });
        }

        let next_index = end;
        let finished = end >= total_players;

        Ok(crate::types::LeaderboardWindow {
            entries: rows,
            next_index,
            finished,
            queried_at,
        })
    }

    /// Picks the index of the best entry not in `selected`. Order: score desc, then completed_at asc (0 = last).
    fn leaderboard_best_index(
        entries: &Vec<(Address, u32, u64, bool)>,
        selected: &Vec<u32>,
    ) -> Option<u32> {
        let n = entries.len();
        let mut best_idx: Option<u32> = None;
        for i in 0..n {
            let mut taken = false;
            for j in 0..selected.len() {
                if selected.get(j).unwrap() == i {
                    taken = true;
                    break;
                }
            }
            if taken {
                continue;
            }
            let (_, score, completed_at, _) = entries.get(i).unwrap();
            let better = match best_idx {
                None => true,
                Some(bi) => {
                    let (_, b_score, b_completed_at, _) = entries.get(bi).unwrap();
                    if score > b_score {
                        true
                    } else if score == b_score {
                        let a_val = if completed_at == 0 {
                            u64::MAX
                        } else {
                            completed_at
                        };
                        let b_val = if b_completed_at == 0 {
                            u64::MAX
                        } else {
                            b_completed_at
                        };
                        a_val < b_val
                    } else {
                        false
                    }
                }
            };
            if better {
                best_idx = Some(i);
            }
        }
        best_idx
    }

    fn update_leaderboard_index(env: &Env, progress: &PlayerProgress) {
        let mut entries = Storage::get_leaderboard_index(env, progress.hunt_id);
        let updated = LeaderboardIndexEntry {
            player: progress.player.clone(),
            score: progress.total_score,
            completed_at: progress.completed_at,
            is_completed: progress.is_completed,
        };

        let mut existing_idx: Option<u32> = None;
        for i in 0..entries.len() {
            let entry = entries.get(i).unwrap();
            if entry.player == progress.player {
                existing_idx = Some(i);
                break;
            }
        }

        if let Some(i) = existing_idx {
            entries.remove(i);
        }

        let mut insert_at = entries.len();
        for i in 0..entries.len() {
            let current = entries.get(i).unwrap();
            if Self::leaderboard_entry_precedes(&updated, &current) {
                insert_at = i;
                break;
            }
        }

        if entries.len() < MAX_LEADERBOARD_SIZE || insert_at < MAX_LEADERBOARD_SIZE {
            entries.insert(insert_at, updated);
            if entries.len() > MAX_LEADERBOARD_SIZE {
                entries.pop_back();
            }
        }

        Storage::save_leaderboard_index(env, progress.hunt_id, &entries);
    }

    fn leaderboard_entry_precedes(
        candidate: &LeaderboardIndexEntry,
        current: &LeaderboardIndexEntry,
    ) -> bool {
        if candidate.score != current.score {
            return candidate.score > current.score;
        }

        let candidate_completed_at = if candidate.completed_at == 0 {
            u64::MAX
        } else {
            candidate.completed_at
        };
        let current_completed_at = if current.completed_at == 0 {
            u64::MAX
        } else {
            current.completed_at
        };

        candidate_completed_at < current_completed_at
    }

    /// Returns aggregate statistics for a hunt (read-only): total players, completion rate, average score.
    /// Returns error if hunt does not exist.
    pub fn get_hunt_statistics(env: Env, hunt_id: u64) -> Result<HuntStatistics, HuntErrorCode> {
        // Cache existence check (cheaper than loading full Hunt)
        Storage::get_hunt_cache(&env, hunt_id).ok_or(HuntErrorCode::HuntNotFound)?;
        let players = Storage::get_hunt_players(&env, hunt_id);
        let total_players = players.len();
        let mut completed_count: u32 = 0;
        let mut total_score_sum: u64 = 0;
        for i in 0..players.len() {
            let p = players.get(i).unwrap();
            if p.is_completed {
                completed_count = completed_count.checked_add(1).ok_or(HuntErrorCode::ScoreOverflow)?;
            }
            total_score_sum = total_score_sum.checked_add(p.total_score as u64)
                .ok_or(HuntErrorCode::ScoreOverflow)?;
        }
        let completion_rate_percent = if total_players > 0 {
            completed_count
                .checked_mul(100)
                .ok_or(HuntErrorCode::ScoreOverflow)?
                / total_players
        } else {
            0
        };
        let average_score = if total_players > 0 {
            total_score_sum
                .checked_div(u64::from(total_players))
                .unwrap_or(0) as u32
        } else {
            0
        };
        Ok(HuntStatistics {
            total_players,
            completed_count,
            completion_rate_percent,
            total_score_sum,
            average_score,
        })
    }

// -----------------------------------------------------------------------------
// View-Only Access Management
// -----------------------------------------------------------------------------

pub fn add_view_only_access(
    env: Env,
    hunt_id: u64,
    creator: Address,
    viewer: Address,
) -> Result<(), HuntErrorCode> {
    creator.require_auth();

    // Cache read (cheaper than loading full Hunt from persistent)
    let cache = Storage::get_hunt_cache(&env, hunt_id)
        .ok_or(HuntErrorCode::HuntNotFound)?;

    if cache.creator != creator {
        return Err(HuntErrorCode::Unauthorized);
    }

    Storage::add_view_only(&env, hunt_id, &viewer);
    Ok(())
}

pub fn remove_view_only_access(
    env: Env,
    hunt_id: u64,
    creator: Address,
    viewer: Address,
) -> Result<(), HuntErrorCode> {
    creator.require_auth();

    // Cache read (cheaper than loading full Hunt from persistent)
    let cache = Storage::get_hunt_cache(&env, hunt_id)
        .ok_or(HuntErrorCode::HuntNotFound)?;

    if cache.creator != creator {
        return Err(HuntErrorCode::Unauthorized);
    }

    Storage::remove_view_only(&env, hunt_id, &viewer);
    Ok(())
}

pub fn is_view_only(env: Env, hunt_id: u64, address: Address) -> bool {
    Storage::is_view_only(&env, hunt_id, &address)
}

pub fn get_view_only_list(env: Env, hunt_id: u64) -> Vec<Address> {
    Storage::get_view_only_list(&env, hunt_id)
}

pub fn initialize_admin(
    env: Env,
    admin: Address,
) -> Result<(), HuntErrorCode> {
    admin.require_auth();

    if Storage::get_admin(&env).is_some() {
        return Err(HuntErrorCode::Unauthorized);
    }

    Storage::set_admin(&env, &admin);
    Ok(())
}

/// Step one of a two-step admin key rotation.
///
/// The current admin proposes a new admin. The change is NOT applied until the
/// proposed address calls `accept_admin`, which prevents accidental lockout: a
/// typo in `propose_new_admin` can simply be overwritten or ignored, and the
/// current admin never loses access until the new admin actively accepts.
pub fn propose_new_admin(
    env: Env,
    admin: Address,
    new_admin: Address,
) -> Result<(), HuntErrorCode> {
    admin.require_auth();

    let current_admin =
        Storage::get_admin(&env).ok_or(HuntErrorCode::Unauthorized)?;
    if current_admin != admin {
        return Err(HuntErrorCode::Unauthorized);
    }

    // A pending rotation can be overwritten by the current admin at any time.
    Storage::set_pending_admin(&env, &new_admin);

    env.events().publish(
        (Symbol::new(&env, "ADMIN"), Symbol::new(&env, "ADM_PROP")),
        (admin, new_admin),
    );

    Ok(())
}

/// Step two of a two-step admin key rotation.
///
/// The proposed new admin accepts the role, completing the rotation. Only the
/// address stored by `propose_new_admin` may accept, so a wrong proposal cannot
/// silently take over the contract.
pub fn accept_admin(env: Env, new_admin: Address) -> Result<(), HuntErrorCode> {
    new_admin.require_auth();

    let pending = Storage::get_pending_admin(&env).ok_or(HuntErrorCode::NoPendingAdmin)?;
    if pending != new_admin {
        return Err(HuntErrorCode::PendingAdminMismatch);
    }

    let old_admin = Storage::get_admin(&env);
    Storage::set_admin(&env, &new_admin);
    Storage::clear_pending_admin(&env);

    env.events().publish(
        (Symbol::new(&env, "ADMIN"), Symbol::new(&env, "ADM_TRF")),
        (old_admin, new_admin),
    );

    Ok(())
}

pub fn add_global_view_only(
    env: Env,
    admin: Address,
    viewer: Address,
) -> Result<(), HuntErrorCode> {
    admin.require_auth();

    let configured_admin =
        Storage::get_admin(&env).ok_or(HuntErrorCode::Unauthorized)?;

    if configured_admin != admin {
        return Err(HuntErrorCode::Unauthorized);
    }

    Storage::add_global_view_only(&env, &viewer);
    Ok(())
}

pub fn remove_global_view_only(
    env: Env,
    admin: Address,
    viewer: Address,
) -> Result<(), HuntErrorCode> {
    admin.require_auth();

    let configured_admin =
        Storage::get_admin(&env).ok_or(HuntErrorCode::Unauthorized)?;

    if configured_admin != admin {
        return Err(HuntErrorCode::Unauthorized);
    }

    Storage::remove_global_view_only(&env, &viewer);
    Ok(())
}

pub fn is_global_view_only(env: Env, address: Address) -> bool {
    Storage::is_global_view_only(&env, &address)
}

pub fn get_global_view_only_list(env: Env) -> Vec<Address> {
    Storage::get_global_view_only_list(&env)
}

// Pause controls
pub fn pause_registrations(env: Env, admin: Address) -> Result<(), HuntErrorCode> {
    admin.require_auth();
    
    let configured_admin = Storage::get_admin(&env).ok_or(HuntErrorCode::Unauthorized)?;
    if configured_admin != admin {
        return Err(HuntErrorCode::Unauthorized);
    }
    
    Storage::set_pause_registrations(&env, true);
    Ok(())
}

pub fn unpause_registrations(env: Env, admin: Address) -> Result<(), HuntErrorCode> {
    admin.require_auth();
    
    let configured_admin = Storage::get_admin(&env).ok_or(HuntErrorCode::Unauthorized)?;
    if configured_admin != admin {
        return Err(HuntErrorCode::Unauthorized);
    }
    
    Storage::set_pause_registrations(&env, false);
    Ok(())
}

pub fn pause_answers(env: Env, admin: Address) -> Result<(), HuntErrorCode> {
    admin.require_auth();
    
    let configured_admin = Storage::get_admin(&env).ok_or(HuntErrorCode::Unauthorized)?;
    if configured_admin != admin {
        return Err(HuntErrorCode::Unauthorized);
    }
    
    Storage::set_pause_answers(&env, true);
    Ok(())
}

pub fn unpause_answers(env: Env, admin: Address) -> Result<(), HuntErrorCode> {
    admin.require_auth();
    
    let configured_admin = Storage::get_admin(&env).ok_or(HuntErrorCode::Unauthorized)?;
    if configured_admin != admin {
        return Err(HuntErrorCode::Unauthorized);
    }
    
    Storage::set_pause_answers(&env, false);
    Ok(())
}

pub fn pause_rewards(env: Env, admin: Address) -> Result<(), HuntErrorCode> {
    admin.require_auth();
    
    let configured_admin = Storage::get_admin(&env).ok_or(HuntErrorCode::Unauthorized)?;
    if configured_admin != admin {
        return Err(HuntErrorCode::Unauthorized);
    }
    
    Storage::set_pause_rewards(&env, true);
    Ok(())
}

pub fn unpause_rewards(env: Env, admin: Address) -> Result<(), HuntErrorCode> {
    admin.require_auth();
    
    let configured_admin = Storage::get_admin(&env).ok_or(HuntErrorCode::Unauthorized)?;
    if configured_admin != admin {
        return Err(HuntErrorCode::Unauthorized);
    }
    
    Storage::set_pause_rewards(&env, false);
    Ok(())
}

// Query pause state
pub fn get_pause_state(env: Env) -> (bool, bool, bool) {
    (
        Storage::is_pause_registrations(&env),
        Storage::is_pause_answers(&env),
        Storage::is_pause_rewards(&env)
    )
}

// -----------------------------------------------------------------------------
// Schema Migration & Monitoring
// -----------------------------------------------------------------------------

pub fn get_schema_version(env: Env) -> u32 {
    migration::HuntyCoreMigration::get_schema_version(&env)
}

pub fn initialize_schema(env: Env, admin: Address) {
    admin.require_auth();
    migration::HuntyCoreMigration::initialize_schema(&env, &admin);
}

pub fn run_migration(
    env: Env,
    admin: Address,
    target_version: u32,
    dry_run: bool,
) -> Result<migration::MigrationReport, hunty_migration::UpgradeAuthError> {
    admin.require_auth();
    migration::HuntyCoreMigration::run_migration(
        &env,
        &admin,
        target_version,
        dry_run,
    )
}

pub fn rollback_migration(
    env: Env,
    admin: Address,
) -> Result<migration::MigrationReport, hunty_migration::UpgradeAuthError> {
    migration::HuntyCoreMigration::rollback_migration(&env, &admin)
}

pub fn get_health_dashboard(env: Env) -> monitoring::ContractHealth {
    monitoring::Monitoring::health_dashboard(&env)
}

    fn sync_hunt_clue_counts(env: &Env, hunt_id: u64, hunt: &mut Hunt) {
        let clues = Storage::list_clues_for_hunt(env, hunt_id, 0, u32::MAX);
        let mut total = 0;
        let mut required = 0;
        for i in 0..clues.len() {
            let clue = clues.get(i).unwrap();
            total += 1;
            if clue.is_required {
                required += 1;
            }
        }
        hunt.total_clues = total;
        hunt.required_clues = required;
    }

    fn sync_reward_pool_balance(env: &Env, hunt_id: u64, hunt: &mut Hunt) {
        if let Some(reward_manager_addr) = Storage::get_reward_manager(env) {
            let mut balance_args: Vec<Val> = Vec::new(env);
            balance_args.push_back(hunt_id.into_val(env));

            if let Ok(Ok(pool_balance)) = env.try_invoke_contract::<i128, RewardErrorCode>(
                &reward_manager_addr,
                &Symbol::new(env, "get_pool_balance"),
                balance_args,
            ) {
                hunt.reward_config.xlm_pool = pool_balance;
            }
        }
    }

mod admin;
mod errors;
mod migration;
mod monitoring;
mod rate_limit;
mod sanitization;
mod storage;
pub mod types;

#[cfg(test)]
mod test;
