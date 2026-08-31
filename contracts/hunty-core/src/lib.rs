#![no_std]
#![allow(clippy::too_many_arguments)]

mod errors;
mod migration;
mod monitoring;
mod rate_limit;
mod sanitization;
mod storage;
pub mod types;

use crate::errors::{HuntError, HuntErrorCode};
use crate::storage::Storage;
use crate::types::{
    AnswerIncorrectEvent, AnswerPreviewedEvent, BatchClueInput, Clue, ClueAddedEvent, ClueAliasesAddedEvent,
    ClueCompletedEvent, ClueInfo, CreatorBlacklistedEvent, CreatorRemovedFromBlacklistEvent, GcReport, Hunt,
    HuntActivatedEvent, HuntArchivedEvent, HuntCache, HuntCancelledEvent, HuntClonedEvent,
    HuntClosedEvent, HuntCompletedEvent, HuntCreatedEvent, HuntDeactivatedEvent,
    HuntDescriptionUpdatedEvent, HuntGarbageCollectedEvent, HuntReactivatedEvent, HuntStatistics, HuntStatus,
    HuntStatusChangedEvent, InviteCodeGeneratedEvent, InviteCodeRevokedEvent, LeaderboardEntry,
    LeaderboardIndexEntry, LeaderboardResult, PlayerProgress, PlayerRegisteredEvent,
    PlayerRegisteredWithInviteEvent, RewardClaimedEvent, RewardConfig, RewardManagerSetEvent,
    TimeBonusConfig,
};
use reward_interface::RewardErrorCode;
use soroban_sdk::{
    contract, contractimpl, contracttype, Address, Bytes, BytesN, Env, IntoVal, String, Symbol,
    Val, Vec,
};

const MAX_TITLE_BYTES: u32 = 200;
// Must stay <= crate::sanitization::SANITIZE_STACK_CAP (2048). Raising these
// above the sanitizer stack CAP without increasing SANITIZE_STACK_CAP will
// return SanitizeError::LimitTooLarge for every call using that limit.
const MAX_DESCRIPTION_BYTES: u32 = 2000;
/// Sentinel value for `max_submissions_per_minute` indicating no rate limit.
const UNLIMITED_SUBMISSIONS_PER_MINUTE: u32 = 0;

const HUNT_CREATION_WINDOW_SECS: u64 = 86_400;
const HUNT_CREATION_LIMIT: u32 = 10;

#[derive(Clone)]
#[contracttype]
enum CreationRateLimitKey {
    Timestamps(Address),
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::testutils::Address as _;

    #[test]
    fn max_submissions_per_minute_zero_is_unlimited_sentinel() {
        assert_eq!(UNLIMITED_SUBMISSIONS_PER_MINUTE, 0);
    }

    #[test]
    fn hunt_creation_rate_limit_is_rolling_across_utc_midnight() {
        let env = Env::default();
        env.ledger().set_timestamp(86_399);
        let creator = Address::generate(&env);
        for _ in 0..10 {
            assert!(
                HuntyCore::check_hunt_creation_rate_limit(&env, &creator, env.ledger().timestamp())
                    .is_ok()
            );
        }
        assert!(
            HuntyCore::check_hunt_creation_rate_limit(&env, &creator, env.ledger().timestamp())
                .is_err()
        );
        env.ledger().set_timestamp(86_401);
        assert!(
            HuntyCore::check_hunt_creation_rate_limit(&env, &creator, env.ledger().timestamp())
                .is_err()
        );
    }
}
const MAX_QUESTION_LENGTH: u32 = 2000;
const MAX_ANSWER_LENGTH: u32 = 256;
const MAX_CATEGORY_BYTES: u32 = 64;
const MAX_CATEGORIES_PER_HUNT: u32 = 5;
const MAX_CLUES_PER_HUNT: u32 = 100;
/// Maximum number of leaderboard entries returned (gas and UX limit).
const MAX_LEADERBOARD_SIZE: u32 = 20;
/// Maximum number of player records scanned when building leaderboard responses.
const MAX_LEADERBOARD_SCAN_SIZE: u32 = 200;
/// Maximum batch size for paginated list operations (gas protection).
const MAX_BATCH_SIZE: u32 = 50;
/// Maximum hunt records scanned by discovery queries in one invocation.
const MAX_HUNT_SEARCH_SCAN_SIZE: u32 = 200;
/// Default page size for paginated queries.
#[allow(dead_code)]
const DEFAULT_PAGE_SIZE: u32 = 20;
/// Maximum allowed age for a submission envelope before it is considered stale.
pub(crate) const ANSWER_SUBMISSION_WINDOW_SECS: u64 = 300;
/// Small forward-skew allowance so near-simultaneous signing and inclusion does not fail.
const ANSWER_SUBMISSION_FUTURE_SKEW_SECS: u64 = 30;
/// Minimum allowed duration between hunt creation and a non-zero end time (ledger seconds).
pub(crate) const MIN_HUNT_DURATION: u64 = 3600;
/// Maximum number of members allowed in a team.
#[allow(dead_code)]
const MAX_TEAM_SIZE: u32 = 10;
/// Minimum points a clue can be worth.
pub(crate) const MIN_CLUE_POINTS: u32 = 1;
/// Maximum points a clue can be worth. A clue above this cap multiplies into
/// a score that saturates u32 and flattens the leaderboard into a tie.
pub(crate) const MAX_CLUE_POINTS: u32 = 10_000;
/// Lowest difficulty tier for a clue. 1 = easiest.
pub(crate) const MIN_CLUE_DIFFICULTY: u32 = 1;
/// Highest difficulty tier for a clue. These are the tiers the UI exposes:
/// 1 = easiest, 5 = hardest. Difficulty is a multiplier on a clue's points.
pub(crate) const MAX_CLUE_DIFFICULTY: u32 = 5;

#[contract]
pub struct HuntyCore;

// Exported contract functions with many parameters trigger this lint both on
// the original fns and on the SDK-generated dispatch wrappers.
#[allow(clippy::too_many_arguments)]
#[contractimpl]
impl HuntyCore {
    /// Sets the contract admin once. Subsequent calls require current admin auth via set_admin.
    pub fn initialize_admin(env: Env, admin: Address) -> Result<(), HuntErrorCode> {
        admin.require_auth();
        if Storage::get_admin(&env).is_some() {
            return Err(HuntErrorCode::Unauthorized);
        }
            Ok(())
    }

    #[allow(dead_code)]
    fn get_player_total_completed_hunts(env: &Env, player: &Address) -> u32 {
        // This would ideally use a global player stats storage
        // For now, we can implement a simple version or extend Storage
        Storage::get_player_completed_hunt_count(env, player)
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

    #[allow(dead_code)]
    fn ensure_not_paused(env: &Env) -> Result<(), HuntErrorCode> {
        if Storage::is_contract_paused(env) {
            return Err(HuntErrorCode::ContractPaused);
        }
        Ok(())
    }

    fn check_hunt_creation_rate_limit(
        env: &Env,
        creator: &Address,
        current_time: u64,
    ) -> Result<(), HuntErrorCode> {
        let key = CreationRateLimitKey::Timestamps(creator.clone());
        let timestamps: Vec<u64> = env
            .storage()
            .persistent()
            .get(&key)
            .unwrap_or_else(|| Vec::new(env));
        let mut recent = Vec::new(env);
        for i in 0..timestamps.len() {
            // SAFETY: i is within the vector bounds established by the enclosing loop
            let ts = timestamps.get(i).unwrap();
            if current_time.saturating_sub(ts) < HUNT_CREATION_WINDOW_SECS {
                recent.push_back(ts);
            }
        }
        if recent.len() >= HUNT_CREATION_LIMIT {
            return Err(HuntErrorCode::from(HuntError::RateLimitExceeded));
        }
        recent.push_back(current_time);
        env.storage().persistent().set(&key, &recent);
        Ok(())
    }

    /// Creates a new scavenger hunt with the provided metadata.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `creator` - The address of the hunt creator (typically use env.invoker() from the caller)
    /// * `title` - The title of the hunt (max 200 characters)
    /// * `description` - The description of the hunt (max 2000 characters)
    /// * `start_time` - Optional start timestamp. When set, players cannot register
    ///   or submit answers until the ledger timestamp reaches this value. 0 means
    ///   no start time restriction (immediately playable once activated).
    /// * `end_time` - Optional end timestamp (0 means no end time restriction)
    /// * `max_submissions_per_minute` - Maximum number of submissions allowed per
    ///   minute per player. [`UNLIMITED_SUBMISSIONS_PER_MINUTE`] (0) means no limit.
    ///
    /// # Returns
    /// The unique hunt ID of the newly created hunt
    ///
    /// # Errors
    /// * `InvalidTitle` - If title is empty or exceeds maximum length
    /// * `InvalidDescription` - If description exceeds maximum length
    /// * `InvalidAddress` - If creator address is invalid
    #[allow(clippy::too_many_arguments)]
    pub fn create_hunt(
        env: Env,
        creator: Address,
        title: String,
        description: String,
        start_time: Option<u64>,
        end_time: Option<u64>,
        max_submissions_per_minute: u32,
        start_multiplier_bps: Option<u32>,
        default_points: Option<u32>,
    ) -> Result<u64, HuntErrorCode> {
        creator.require_auth();
        monitoring::Monitoring::record_invocation(&env, 50_000, true);
        if Storage::is_blacklisted(&env, &creator) {
            return Err(HuntErrorCode::AddressBlacklisted);
        }

        // Validate and sanitize title/description at byte level
        let title =
            crate::sanitization::StringSanitizer::sanitize::<MAX_TITLE_BYTES>(&env, &title, false)
                .map_err(|_| HuntErrorCode::InvalidTitle)?;

        let description = crate::sanitization::StringSanitizer::sanitize::<MAX_DESCRIPTION_BYTES>(
            &env,
            &description,
            true,
        )
        .map_err(|_| HuntErrorCode::InvalidDescription)?;

        let current_time = env.ledger().timestamp();
        Self::check_hunt_creation_rate_limit(&env, &creator, current_time)?;

        let end_time_val = end_time.unwrap_or(0);
        if end_time_val != 0 && end_time_val < current_time.saturating_add(MIN_HUNT_DURATION) {
            return Err(HuntErrorCode::HuntEndTimeInPast);
        }

        // Generate unique hunt ID
        let hunt_id = Storage::next_hunt_id(&env);

        // Initialize reward config with zero pool
        let reward_config = RewardConfig::new(
            &env, 0,     // xlm_pool: zero initially
            false, // nft_enabled: false initially
            None,  // nft_contract: None initially
            0,     // max_winners: 0 initially
            0,     // nft_rarity: zero initially
            0,     // nft_tier: zero initially
            None,  // nft_image_uri: None initially
        );

        // Create the hunt with Draft status
        let hunt = Hunt {
            hunt_id,
            creator: creator.clone(),
            title: title.clone(),
            description: description.clone(),
            categories: Vec::new(&env),
            difficulty_rating: 0,
            difficulty_override: None,
            status: HuntStatus::Draft,
            created_at: current_time,
            activated_at: 0, // Will be set when hunt is activated
            start_time: start_time.unwrap_or(0),
            end_time: end_time_val,
            reward_config,
            time_bonus_start_bps: None,
            time_bonus_min_bps: None,
            time_bonus_decay_secs: None,
            total_clues: 0, // Empty clue list initially
            required_clues: 0,
            completed_count: 0,
            max_submissions_per_minute,
            max_attempts_per_clue: 5,
            start_multiplier_bps: start_multiplier_bps.unwrap_or(20000),
            registration_deadline: 0,
            allow_partial_scoring: false,
            team_mode: false,
            default_points: default_points.unwrap_or(100),
            attempt_cooldown_secs: 0,
            max_players: 0,
            is_private: false,
            invite_code_hash: None,
            remaining_slots: 0,
        };

        // Store the hunt
        Storage::save_hunt(&env, &hunt);

        // Emit HuntCreated event
        let event = HuntCreatedEvent {
            hunt_id,
            creator: creator.clone(),
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
    /// Clones an existing hunt into a new draft.
    /// The caller must be the original hunt creator.
    /// All clues are duplicated with new clue IDs.
    /// Returns the new hunt ID.
    pub fn clone_hunt(
        env: Env,
        template_hunt_id: u64,
        caller: Address,
    ) -> Result<u64, HuntErrorCode> {
        // Ensure caller is authenticated
        caller.require_auth();
        // Load template hunt
        let template_hunt =
            Storage::get_hunt(&env, template_hunt_id).ok_or(HuntErrorCode::HuntNotFound)?;
        // Ensure the template hunt is completed before cloning
        if template_hunt.status != HuntStatus::Completed {
            return Err(HuntErrorCode::InvalidHuntStatus);
        }
        // Only the original creator can clone the hunt
        if caller != template_hunt.creator {
            return Err(HuntErrorCode::Unauthorized);
        }
        let hunt_id = Self::create_hunt(
            env.clone(),
            caller.clone(),
            template_hunt.title.clone(),
            template_hunt.description.clone(),
            None,
            None,
            template_hunt.max_submissions_per_minute,
            Some(template_hunt.start_multiplier_bps),
            Some(template_hunt.default_points),
        )?;

        let mut hunt = Storage::get_hunt(&env, hunt_id).ok_or(HuntErrorCode::HuntNotFound)?;
        // Clone each clue from the template
        let template_clues =
            Storage::list_clues_for_hunt(&env, template_hunt_id, 0, MAX_CLUES_PER_HUNT);
        for i in 0..template_clues.len() {
            // SAFETY: i is within the vector bounds established by the enclosing loop
            let clue = template_clues.get(i).unwrap();
            let cloned_clue = Clue {
                clue_id: Storage::next_clue_id(&env, hunt_id),
                question: clue.question.clone(),
                answer_hashes: clue.answer_hashes.clone(),
                points: clue.points,
                is_required: clue.is_required,
                difficulty: clue.difficulty,
                weight: clue.weight,
                hint: clue.hint,
                hint_penalty_points: clue.hint_penalty_points,
            };
            Storage::save_clue(&env, hunt_id, &cloned_clue);
            hunt.total_clues += 1;
            if cloned_clue.is_required {
                hunt.required_clues += 1;
            }
            // Emit clue added event for the cloned hunt
            let event = ClueAddedEvent {
                hunt_id,
                clue_id: cloned_clue.clue_id,
                creator: caller.clone(),
                question: cloned_clue.question.clone(),
                points: cloned_clue.points,
                is_required: cloned_clue.is_required,
                difficulty: cloned_clue.difficulty,
                weight: cloned_clue.weight,
            };
            env.events().publish(
                (Symbol::new(&env, "ClueAdded"), hunt_id, cloned_clue.clue_id),
                event,
            );
        }
        // Persist the updated hunt metadata
        Storage::save_hunt(&env, &hunt);
        // Emit HuntCloned event
        let clone_event = HuntClonedEvent {
            original_hunt_id: template_hunt_id,
            new_hunt_id: hunt_id,
            creator: caller.clone(),
        };
        env.events()
            .publish((Symbol::new(&env, "HuntCloned"), hunt_id), clone_event);
        Ok(hunt_id)
    }

    pub fn set_time_bonus_config(
        env: Env,
        hunt_id: u64,
        caller: Address,
        time_bonus_config: Option<TimeBonusConfig>,
    ) -> Result<(), HuntErrorCode> {
        caller.require_auth();

        let mut hunt = Storage::get_hunt(&env, hunt_id).ok_or(HuntErrorCode::HuntNotFound)?;

        if !Storage::is_authorized_creator_or_co_creator(&env, hunt_id, &caller) {
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

    /// Updates the maximum number of attempts allowed per clue and attempt cooldown duration for a draft hunt.
    /// Only the hunt creator or co-creator can update it.
    pub fn set_max_attempts_per_clue(
        env: Env,
        hunt_id: u64,
        caller: Address,
        max_attempts_per_clue: u32,
        attempt_cooldown_secs: u32,
    ) -> Result<(), HuntErrorCode> {
        if max_attempts_per_clue == 0 {
            return Err(HuntErrorCode::InvalidMaxAttempts);
        }

        let mut hunt = Storage::get_hunt_or_error(&env, hunt_id).map_err(HuntErrorCode::from)?;
        if !Storage::is_authorized_creator_or_co_creator(&env, hunt_id, &caller) {
            return Err(HuntErrorCode::Unauthorized);
        }
        if hunt.status != HuntStatus::Draft {
            return Err(HuntErrorCode::InvalidHuntStatus);
        }

        hunt.max_attempts_per_clue = max_attempts_per_clue;
        hunt.attempt_cooldown_secs = attempt_cooldown_secs;
        Storage::save_hunt(&env, &hunt);
        Ok(())
    }

    /// Updates a hunt's description. Only the hunt creator can update it, and it can be updated for any hunt status.
    pub fn update_hunt_description(
        env: Env,
        hunt_id: u64,
        caller: Address,
        description: String,
    ) -> Result<(), HuntErrorCode> {
        caller.require_auth();

        let mut hunt = Storage::get_hunt_or_error(&env, hunt_id).map_err(HuntErrorCode::from)?;
        if !Storage::is_authorized_creator_or_co_creator(&env, hunt_id, &caller) {
            return Err(HuntErrorCode::Unauthorized);
        }

        // Validate and sanitize description
        let description = crate::sanitization::StringSanitizer::sanitize::<MAX_DESCRIPTION_BYTES>(
            &env,
            &description,
            true,
        )
        .map_err(|_| HuntErrorCode::InvalidDescription)?;

        hunt.description = description.clone();
        Storage::save_hunt(&env, &hunt);

        // Emit event
        let event = HuntDescriptionUpdatedEvent {
            hunt_id,
            creator: caller,
            description,
        };
        env.events().publish(
            (Symbol::new(&env, "HuntDescriptionUpdated"), hunt_id),
            event,
        );

        Ok(())
    }

    /// Sets the maximum players for a hunt. Only the hunt creator can set it, and only in Draft status.
    pub fn set_max_players(
        env: Env,
        hunt_id: u64,
        caller: Address,
        max_players: u32,
    ) -> Result<(), HuntErrorCode> {
        caller.require_auth();

        let mut hunt = Storage::get_hunt_or_error(&env, hunt_id).map_err(HuntErrorCode::from)?;
        if !Storage::is_authorized_creator_or_co_creator(&env, hunt_id, &caller) {
            return Err(HuntErrorCode::Unauthorized);
        }
        if hunt.status != HuntStatus::Draft {
            return Err(HuntErrorCode::InvalidHuntStatus);
        }

        hunt.max_players = max_players;
        Storage::save_hunt(&env, &hunt);
        Ok(())
    }

    /// Exposes the end time of a hunt.
    pub fn get_hunt_end_time(env: Env, hunt_id: u64) -> Result<u64, HuntErrorCode> {
        let hunt = Storage::get_hunt(&env, hunt_id).ok_or(HuntErrorCode::HuntNotFound)?;
        Ok(hunt.end_time)
    }

    /// Adds a clue to a hunt. Only the hunt creator can add clues.
    /// Answers are hashed with SHA256 before storage; the hash is never exposed.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `hunt_id` - The hunt to add the clue to
    /// * `question` - The clue question text (max 2000 chars, non-empty)
    /// * `answer` - Plain-text answer; normalized (trimmed, lowercased) then hashed
    /// * `points` - Points awarded for solving this clue (must be within 1..=10_000)
    /// * `is_required` - Whether this clue must be solved to complete the hunt
    /// * `difficulty` - Optional difficulty tier (defaults to 1) used as a multiplier on
    ///   the clue's points. Valid scale is 1..=5, where 1 is easiest and 5 is hardest.
    /// * `weight` - Optional weight multiplier (defaults to 1)
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
    /// * `InvalidPoints` - Points are outside the allowed 1..=10_000 range
    /// * `InvalidDifficulty` - Difficulty is outside the allowed 1..=5 tier scale
    #[allow(clippy::too_many_arguments)]
    pub fn add_clue(
        env: Env,
        hunt_id: u64,
        question: String,
        answer: String,
        points: u32,
        is_required: bool,
        difficulty: Option<u32>,
        weight: Option<u32>,
    ) -> Result<u32, HuntErrorCode> {
        let hunt = Storage::get_hunt_or_error(&env, hunt_id).map_err(HuntErrorCode::from)?;
        hunt.creator.require_auth();
        if hunt.status != HuntStatus::Draft {
            return Err(HuntErrorCode::InvalidHuntStatus);
        }
        if Storage::get_clue_counter(&env, hunt_id) >= MAX_CLUES_PER_HUNT {
            return Err(HuntErrorCode::from(HuntError::TooManyClues));
        }

        let clue_id = Self::insert_clue(
            &env,
            hunt_id,
            &hunt.creator,
            question,
            answer,
            points,
            is_required,
            difficulty,
            weight,
        )?;
        let mut updated = hunt;
        updated.total_clues += 1;
        if is_required {
            updated.required_clues += 1;
        }
        Storage::save_hunt(&env, &updated);

        Ok(clue_id)
    }

    /// Adds multiple clues to a draft hunt in one invocation. Only the hunt creator can add clues.
    ///
    /// The batch is validated against the per-hunt clue cap before writing any new clues,
    /// so a request that would exceed the limit fails without partially adding clues.
    pub fn add_clues(
        env: Env,
        hunt_id: u64,
        clues: Vec<BatchClueInput>,
    ) -> Result<Vec<u32>, HuntErrorCode> {
        let hunt = Storage::get_hunt_or_error(&env, hunt_id).map_err(HuntErrorCode::from)?;
        hunt.creator.require_auth();
        if hunt.status != HuntStatus::Draft {
            return Err(HuntErrorCode::InvalidHuntStatus);
        }

        let existing = Storage::get_clue_counter(&env, hunt_id);
        if existing.saturating_add(clues.len()) > MAX_CLUES_PER_HUNT {
            return Err(HuntErrorCode::from(HuntError::TooManyClues));
        }

        let mut clue_ids = Vec::new(&env);
        let mut batch_required = 0u32;
        for i in 0..clues.len() {
            // SAFETY: i is within the vector bounds established by the enclosing loop
            let clue = clues.get(i).unwrap();
            let clue_id = Self::insert_clue(
                &env,
                hunt_id,
                &hunt.creator,
                clue.question,
                clue.answer,
                clue.points,
                clue.is_required,
                Some(clue.difficulty),
                None, // weight defaults to 1 in insert_clue
            )?;
            clue_ids.push_back(clue_id);
            if clue.is_required {
                batch_required += 1;
            }
        }

        let mut updated = hunt;
        updated.total_clues += clues.len();
        updated.required_clues += batch_required;
        Storage::save_hunt(&env, &updated);

        Ok(clue_ids)
    }

    #[allow(clippy::too_many_arguments)]
    fn insert_clue(
        env: &Env,
        hunt_id: u64,
        creator: &Address,
        question: String,
        answer: String,
        points: u32,
        is_required: bool,
        difficulty: Option<u32>,
        weight: Option<u32>,
    ) -> Result<u32, HuntErrorCode> {
        let difficulty_val = difficulty.unwrap_or(MIN_CLUE_DIFFICULTY);
        if difficulty_val < MIN_CLUE_DIFFICULTY || difficulty_val > MAX_CLUE_DIFFICULTY {
            return Err(HuntErrorCode::InvalidDifficulty);
        }

        let qlen = question.len();
        if qlen == 0 || qlen > MAX_QUESTION_LENGTH {
            return Err(HuntErrorCode::InvalidQuestion);
        }

        // Clue points must stay within [MIN_CLUE_POINTS, MAX_CLUE_POINTS].
        // 0 is treated as unset (invalid), and a value above the cap multiplies
        // into a score that saturates u32, tying the leaderboard.
        if points < MIN_CLUE_POINTS || points > MAX_CLUE_POINTS {
            return Err(HuntErrorCode::InvalidPoints);
        }
        let final_points = points;
        let question = crate::sanitization::StringSanitizer::sanitize::<MAX_QUESTION_LENGTH>(
            env,
            &question,
            false,
        )
        .map_err(|_| HuntErrorCode::InvalidQuestion)?;

        let weight_val = weight.unwrap_or(1);
        if weight_val == 0 {
            return Err(HuntErrorCode::from(HuntError::InvalidWeight));
        }

        let clue_id = Storage::next_clue_id(env, hunt_id);
        let answer_hash = Self::normalize_and_hash_answer(env, hunt_id, clue_id, &answer)
            .map_err(HuntErrorCode::from)?;
        let mut answer_hashes = Vec::new(env);
        answer_hashes.push_back(answer_hash);

        let clue = Clue {
            clue_id,
            question: question.clone(),
            answer_hashes,
            points: final_points,
            is_required,
            difficulty: difficulty_val,
            weight: weight_val,
            hint: None,
            hint_penalty_points: 0,
        };

        Storage::save_clue(env, hunt_id, &clue);

        let mut updated = Storage::get_hunt_or_error(env, hunt_id).map_err(HuntErrorCode::from)?;
        updated.total_clues += 1;
        if is_required {
            updated.required_clues += 1;
        }
        Self::recalculate_hunt_difficulty(env, hunt_id, &mut updated);
        Storage::save_hunt(env, &updated);
        let event = ClueAddedEvent {
            hunt_id,
            clue_id,
            creator: creator.clone(),
            question,
            points: final_points,
            is_required,
            difficulty: difficulty_val,
            weight: weight_val,
        };
        env.events()
            .publish((Symbol::new(env, "ClueAdded"), hunt_id, clue_id), event);

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
        let hunt = Storage::get_hunt_or_error(&env, hunt_id).map_err(HuntErrorCode::from)?;
        if hunt.status != HuntStatus::Draft {
            return Err(HuntErrorCode::InvalidHuntStatus);
        }
        hunt.creator.require_auth();

        let mut clue =
            Storage::get_clue_or_error(&env, hunt_id, clue_id).map_err(HuntErrorCode::from)?;

        for i in 0..answers.len() {
            // SAFETY: i is within the vector bounds established by the enclosing loop
            let answer = answers.get(i).unwrap();
            let hash = Self::normalize_and_hash_answer(&env, hunt_id, clue_id, &answer)
                .map_err(HuntErrorCode::from)?;
            clue.answer_hashes.push_back(hash);
        }

        Storage::save_clue(&env, hunt_id, &clue);

        let event = ClueAliasesAddedEvent {
            hunt_id,
            clue_id,
            creator: hunt.creator.clone(),
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
            weight: clue.weight,
            hint_available: clue.hint.is_some(),
            hint_penalty_points: clue.hint_penalty_points,
        })
    }

    /// Returns paginated clues for a hunt. Answer hashes are not exposed.
    pub fn list_clues(env: Env, hunt_id: u64, offset: u32, limit: u32) -> Vec<ClueInfo> {
        let raw = Storage::list_clues_for_hunt(&env, hunt_id, offset, limit.min(MAX_BATCH_SIZE));
        let mut out = Vec::new(&env);
        let limit = core::cmp::min(raw.len(), MAX_BATCH_SIZE);
        for i in 0..limit {
            // SAFETY: i is in [0, limit) and limit <= raw.len()
            let c = raw.get(i).unwrap();
            out.push_back(ClueInfo {
                clue_id: c.clue_id,
                question: c.question,
                points: c.points,
                is_required: c.is_required,
                difficulty: c.difficulty,
                weight: c.weight,
                hint_available: c.hint.is_some(),
                hint_penalty_points: c.hint_penalty_points,
            });
        }
        out
    }

    /// Returns a list of all hunts (paginated).
    pub fn list_hunts(env: Env, offset: u32, limit: u32) -> Vec<Hunt> {
        let counter = Storage::get_hunt_counter(&env);
        let mut hunts = Vec::new(&env);
        let mut current = offset;
        let max_to_check = offset + limit.min(MAX_BATCH_SIZE) + 100; // Add buffer for skipped archived
        let end_check = max_to_check.min(counter as u32);

        while current < end_check && hunts.len() < limit.min(MAX_BATCH_SIZE) {
            let hunt_id = (current as u64) + 1;
            if let Some(hunt) = Storage::get_hunt(&env, hunt_id) {
                if hunt.status != HuntStatus::Archived {
                    hunts.push_back(hunt);
                }
            }
            current += 1;
        }

        hunts
    }

    /// Searches hunts by partial title match over a caller-bounded hunt-id window.
    pub fn search_hunts(
        env: Env,
        title_substring: String,
        offset: u32,
        limit: u32,
        scan_limit: u32,
    ) -> Vec<Hunt> {
        let counter = Storage::get_hunt_counter(&env);
        let mut hunts = Vec::new(&env);
        let mut current = offset;
        let effective_limit = limit.min(MAX_BATCH_SIZE);
        let effective_scan = scan_limit.min(MAX_HUNT_SEARCH_SCAN_SIZE);
        let end_check = offset.saturating_add(effective_scan).min(counter as u32);

        while current < end_check && hunts.len() < effective_limit {
            let hunt_id = (current as u64) + 1;
            if let Some(hunt) = Storage::get_hunt(&env, hunt_id) {
                if hunt.status != HuntStatus::Archived
                    && Self::title_contains(&hunt.title, &title_substring)
                {
                    hunts.push_back(hunt);
                }
            }
            current += 1;
        }

        hunts
    }

    /// Updates categories for a draft hunt. At most five categories are allowed.
    pub fn set_hunt_categories(
        env: Env,
        hunt_id: u64,
        caller: Address,
        categories: Vec<String>,
    ) -> Result<(), HuntErrorCode> {
        caller.require_auth();
        let mut hunt = Storage::get_hunt_or_error(&env, hunt_id).map_err(HuntErrorCode::from)?;
        if !Storage::is_authorized_creator_or_co_creator(&env, hunt_id, &caller) {
            return Err(HuntErrorCode::Unauthorized);
        }
        if hunt.status != HuntStatus::Draft {
            return Err(HuntErrorCode::InvalidHuntStatus);
        }
        hunt.categories = Self::validate_categories(&env, categories)?;
        Storage::save_hunt(&env, &hunt);
        Ok(())
    }

    /// Returns hunts whose categories include the exact category string.
    pub fn get_hunts_by_category(
        env: Env,
        category: String,
        offset: u32,
        limit: u32,
        scan_limit: u32,
    ) -> Vec<Hunt> {
        let Ok(category) = crate::sanitization::StringSanitizer::sanitize::<MAX_CATEGORY_BYTES>(
            &env,
            &category,
            false,
        ) else {
            return Vec::new(&env);
        };

        let counter = Storage::get_hunt_counter(&env);
        let mut hunts = Vec::new(&env);
        let mut current = offset;
        let effective_limit = limit.min(MAX_BATCH_SIZE);
        let effective_scan = scan_limit.min(MAX_HUNT_SEARCH_SCAN_SIZE);
        let end_check = offset.saturating_add(effective_scan).min(counter as u32);

        while current < end_check && hunts.len() < effective_limit {
            let hunt_id = (current as u64) + 1;
            if let Some(hunt) = Storage::get_hunt(&env, hunt_id) {
                if hunt.status != HuntStatus::Archived && Self::hunt_has_category(&hunt, &category)
                {
                    hunts.push_back(hunt);
                }
            }
            current += 1;
        }

        hunts
    }

    /// Sets or clears a manual hunt difficulty override. Without an override,
    /// the rating is the average clue difficulty.
    pub fn set_hunt_difficulty_override(
        env: Env,
        hunt_id: u64,
        caller: Address,
        difficulty_override: Option<u32>,
    ) -> Result<(), HuntErrorCode> {
        caller.require_auth();
        let mut hunt = Storage::get_hunt_or_error(&env, hunt_id).map_err(HuntErrorCode::from)?;
        if !Storage::is_authorized_creator_or_co_creator(&env, hunt_id, &caller) {
            return Err(HuntErrorCode::Unauthorized);
        }
        if let Some(value) = difficulty_override {
            Self::validate_difficulty(value)?;
            hunt.difficulty_override = Some(value);
        } else {
            hunt.difficulty_override = None;
        }
        Self::recalculate_hunt_difficulty(&env, hunt_id, &mut hunt);
        Storage::save_hunt(&env, &hunt);
        Ok(())
    }

    /// Sets or clears the optional hint for a draft clue.
    pub fn set_clue_hint(
        env: Env,
        hunt_id: u64,
        clue_id: u32,
        caller: Address,
        hint: Option<String>,
        hint_penalty_points: u32,
    ) -> Result<(), HuntErrorCode> {
        caller.require_auth();
        let hunt = Storage::get_hunt_or_error(&env, hunt_id).map_err(HuntErrorCode::from)?;
        if !Storage::is_authorized_creator_or_co_creator(&env, hunt_id, &caller) {
            return Err(HuntErrorCode::Unauthorized);
        }
        if hunt.status != HuntStatus::Draft {
            return Err(HuntErrorCode::InvalidHuntStatus);
        }
        let mut clue =
            Storage::get_clue_or_error(&env, hunt_id, clue_id).map_err(HuntErrorCode::from)?;
        clue.hint = match hint {
            Some(value) => Some(
                crate::sanitization::StringSanitizer::sanitize::<MAX_QUESTION_LENGTH>(
                    &env,
                    &value,
                    false,
                )
                .map_err(|_| HuntErrorCode::InvalidQuestion)?,
            ),
            None => None,
        };
        clue.hint_penalty_points = if clue.hint.is_some() {
            hint_penalty_points
        } else {
            0
        };
        Storage::save_clue(&env, hunt_id, &clue);
        Ok(())
    }

    /// Unlocks a clue hint for a registered player and deducts the clue's hint penalty.
    pub fn request_hint(
        env: Env,
        hunt_id: u64,
        clue_id: u32,
        player: Address,
    ) -> Result<String, HuntErrorCode> {
        player.require_auth();
        let _cache = Self::validate_hunt_active_cached(&env, hunt_id)?;
        let clue =
            Storage::get_clue_or_error(&env, hunt_id, clue_id).map_err(HuntErrorCode::from)?;
        let hint = clue.hint.clone().ok_or(HuntErrorCode::HintNotAvailable)?;
        let mut progress = Storage::get_player_progress_or_error(&env, hunt_id, &player)
            .map_err(HuntErrorCode::from)?;
        progress.request_hint(clue_id, clue.hint_penalty_points)?;
        Storage::save_player_progress(&env, &progress);
        Self::update_leaderboard_index(&env, &progress);
        Ok(hint)
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
                    weight: c.weight,
                    hint_available: c.hint.is_some(),
                    hint_penalty_points: c.hint_penalty_points,
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
        let answer =
            crate::sanitization::StringSanitizer::sanitize::<MAX_ANSWER_LENGTH>(env, answer, false)
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

    fn validate_categories(
        env: &Env,
        categories: Vec<String>,
    ) -> Result<Vec<String>, HuntErrorCode> {
        if categories.len() > MAX_CATEGORIES_PER_HUNT {
            return Err(HuntErrorCode::TooManyCategories);
        }

        let mut sanitized = Vec::new(env);
        for i in 0..categories.len() {
            // SAFETY: i is within the vector bounds established by the enclosing loop
            let category = categories.get(i).unwrap();
            let category = crate::sanitization::StringSanitizer::sanitize::<MAX_CATEGORY_BYTES>(
                env,
                &category,
                false,
            )
            .map_err(|_| HuntErrorCode::InvalidCategory)?;
            sanitized.push_back(category);
        }
        Ok(sanitized)
    }

    fn validate_difficulty(value: u32) -> Result<(), HuntErrorCode> {
        if value < MIN_CLUE_DIFFICULTY || value > MAX_CLUE_DIFFICULTY {
            return Err(HuntErrorCode::InvalidDifficulty);
        }
        Ok(())
    }

    fn recalculate_hunt_difficulty(env: &Env, hunt_id: u64, hunt: &mut Hunt) {
        if let Some(override_value) = hunt.difficulty_override {
            hunt.difficulty_rating = override_value;
            return;
        }

        let clues = Storage::list_clues_for_hunt(env, hunt_id, 0, MAX_CLUES_PER_HUNT);
        if clues.is_empty() {
            hunt.difficulty_rating = 0;
            return;
        }

        let mut total = 0u32;
        for i in 0..clues.len() {
            // SAFETY: i is within the vector bounds established by the enclosing loop
            total = total.saturating_add(clues.get(i).unwrap().difficulty);
        }
        hunt.difficulty_rating = total / clues.len();
    }

    fn title_contains(title: &String, needle: &String) -> bool {
        Self::string_contains_bounded::<{ MAX_TITLE_BYTES as usize }, { MAX_TITLE_BYTES as usize }>(
            title, needle,
        )
    }

    fn hunt_has_category(hunt: &Hunt, category: &String) -> bool {
        for i in 0..hunt.categories.len() {
            if Self::strings_equal_bounded::<{ MAX_CATEGORY_BYTES as usize }>(
                // SAFETY: i is within the vector bounds established by the enclosing loop
                &hunt.categories.get(i).unwrap(),
                category,
            ) {
                return true;
            }
        }
        false
    }

    fn strings_equal_bounded<const N: usize>(left: &String, right: &String) -> bool {
        if left.len() != right.len() || left.len() as usize > N {
            return false;
        }
        let mut left_buf = [0u8; N];
        let mut right_buf = [0u8; N];
        let len = left.len() as usize;
        left.copy_into_slice(&mut left_buf[..len]);
        right.copy_into_slice(&mut right_buf[..len]);
        left_buf[..len] == right_buf[..len]
    }

    fn string_contains_bounded<const H: usize, const N: usize>(
        haystack: &String,
        needle: &String,
    ) -> bool {
        let haystack_len = haystack.len() as usize;
        let needle_len = needle.len() as usize;
        if needle_len == 0 {
            return true;
        }
        if needle_len > haystack_len || haystack_len > H || needle_len > N {
            return false;
        }

        let mut haystack_buf = [0u8; H];
        let mut needle_buf = [0u8; N];
        haystack.copy_into_slice(&mut haystack_buf[..haystack_len]);
        needle.copy_into_slice(&mut needle_buf[..needle_len]);

        let last_start = haystack_len - needle_len;
        for start in 0..=last_start {
            if haystack_buf[start..start + needle_len] == needle_buf[..needle_len] {
                return true;
            }
        }
        false
    }

    fn get_hunt_cache_or_load(env: &Env, hunt_id: u64) -> Result<HuntCache, HuntErrorCode> {
        if let Some(cache) = Storage::get_hunt_cache(env, hunt_id) {
            return Ok(cache);
        }
        let hunt = Storage::get_hunt(env, hunt_id).ok_or(HuntErrorCode::HuntNotFound)?;
        Storage::save_hunt_cache(env, &hunt);
        Ok(HuntCache::from_hunt(&hunt))
    }

    fn validate_hunt_active_cached(env: &Env, hunt_id: u64) -> Result<HuntCache, HuntErrorCode> {
        let cache = Self::get_hunt_cache_or_load(env, hunt_id)?;
        let current_time = env.ledger().timestamp();
        if cache.status != HuntStatus::Active
            || (cache.start_time != 0 && current_time < cache.start_time)
            || (cache.end_time != 0 && current_time >= cache.end_time)
        {
            return Err(HuntErrorCode::HuntNotActive);
        }
        Ok(cache)
    }

    fn emit_hunt_status_changed(
        env: &Env,
        hunt_id: u64,
        old_status: HuntStatus,
        new_status: HuntStatus,
        changed_at: u64,
    ) {
        let event = HuntStatusChangedEvent {
            hunt_id,
            old_status,
            new_status,
            changed_at,
        };
        env.events()
            .publish((Symbol::new(env, "HuntStatusChanged"), hunt_id), event);
    }

    fn validate_rarity(v: u32) -> bool {
        v <= 5
    }

    fn validate_nft_image_uri(uri: &String) -> bool {
        let len = uri.len();
        if len == 0 || len > 200 {
            return false;
        }
        let mut buf = [0u8; 200];
        uri.copy_into_slice(&mut buf[..len as usize]);
        let text = unsafe { core::str::from_utf8_unchecked(&buf[..len as usize]) };

        if text.starts_with("https://") {
            let authority = &text[8..];
            return !authority.is_empty() && !authority.bytes().all(|b| b == b' ');
        }
        if text.starts_with("ipfs://") {
            let cid = &text[7..];
            return cid.len() >= 46;
        }
        false
    }

    /// Resolves the XLM amount for the completing player.
    ///
    /// If the hunt's rewardManager-configured pool has a non-empty
    /// `time_based_tiers` list, this returns the tier's `xlm_amount`
    /// whose `max_completion_secs >= (completion_at - registration_at)`.
    /// If the elapsed time exceeds every configured tier, the last
    /// (slowest) tier's amount is used as a fallback. If the pool has no
    /// tiers configured (or is unreachable), this falls back to the
    /// flat `hunt.reward_config.reward_per_winner()` amount.
    fn resolve_reward_amount(env: &Env, hunt: &Hunt, progress: &PlayerProgress) -> i128 {
        let reward_manager_addr = match Storage::get_reward_manager(env) {
            Some(addr) => addr,
            None => return hunt.reward_config.reward_per_winner(),
        };

        // Fetch pool config from RewardManager. Tiers live there.
        // The Result<_, RewardErrorCode> shape lets us distinguish "pool
        // missing" (a legitimate no-tiers case) from any contract error,
        // and falls back to the flat reward on every non-Ok outcome.
        let mut args: Vec<Val> = Vec::new(env);
        args.push_back(hunt.hunt_id.into_val(env));
        // get_pool_config returns Option<RewardPoolConfig> — T must match.
        let pool_config: Option<reward_interface::RewardPoolConfig> = env
            .try_invoke_contract::<Option<reward_interface::RewardPoolConfig>, reward_interface::RewardErrorCode>(
                &reward_manager_addr,
                &Symbol::new(env, "get_pool_config"),
                args,
            )
            .ok()
            .and_then(|r| r.ok())
            .flatten();

        let tiers = match pool_config.as_ref() {
            Some(cfg) => &cfg.time_based_tiers,
            None => return hunt.reward_config.reward_per_winner(),
        };

        if tiers.is_empty() {
            return hunt.reward_config.reward_per_winner();
        }

        // Compute elapsed time. If started_at is missing, zero selects the smallest tier.
        let elapsed = progress.completed_at.saturating_sub(progress.started_at);

        match reward_interface::resolve_tier_amount(tiers, elapsed) {
            Some(amount) if amount > 0 => amount,
            _ => hunt.reward_config.reward_per_winner(),
        }
    }

    pub fn activate_hunt(env: Env, hunt_id: u64, caller: Address) -> Result<(), HuntErrorCode> {
        // Fast validation using instance cache
        let cache = Self::get_hunt_cache_or_load(&env, hunt_id)?;
        caller.require_auth();
        if caller != cache.creator {
            return Err(HuntErrorCode::Unauthorized);
        }

        // Validation passed — load full hunt from persistent for mutation
        let mut hunt = Storage::get_hunt(&env, hunt_id).ok_or(HuntErrorCode::HuntNotFound)?;
        let old_status = hunt.status.clone();

        let current_time = env.ledger().timestamp();

        if old_status == HuntStatus::Draft {
            // Initial activation from draft: perform all checks
            if cache.total_clues == 0 {
                return Err(HuntErrorCode::NoCluesAdded);
            }
            if cache.required_clues == 0 {
                return Err(HuntErrorCode::NoRequiredClues);
            }

            debug_assert_eq!(cache.max_winners, hunt.reward_config.max_winners);

            let reward_manager = Storage::get_reward_manager(&env);

            if reward_manager.is_some() && hunt.reward_config.max_winners == 0 {
                return Err(HuntErrorCode::NoRewardsConfigured);
            }

            if hunt.reward_config.nft_enabled {
                if !Self::validate_rarity(hunt.reward_config.nft_rarity) {
                    return Err(HuntErrorCode::InvalidRarity);
                }
                match hunt.reward_config.nft_image_uri.as_ref() {
                    Some(uri) => {
                        if !Self::validate_nft_image_uri(uri) {
                            return Err(HuntErrorCode::NoRewardsConfigured);
                        }
                    }
                    None => return Err(HuntErrorCode::NoRewardsConfigured),
                }
            }

            // Check reward pool has sufficient balance if reward manager is configured
            if let Some(ref reward_manager_addr) = reward_manager {
                let mut balance_args: Vec<Val> = Vec::new(&env);
                balance_args.push_back(hunt_id.into_val(&env));

                // Query the pool balance from the reward manager
                let pool_balance = match env.try_invoke_contract::<i128, RewardErrorCode>(
                    reward_manager_addr,
                    &Symbol::new(&env, "get_pool_balance"),
                    balance_args.clone(),
                ) {
                    Ok(Ok(balance)) => balance,
                    _ => return Err(HuntErrorCode::InsufficientRewardPool),
                };
                hunt.reward_config.xlm_pool = pool_balance;

                // Query the minimum distribution amount for this pool
                let min_distribution_amount = match env
                    .try_invoke_contract::<i128, RewardErrorCode>(
                        reward_manager_addr,
                        &Symbol::new(&env, "get_min_distribution_amount"),
                        balance_args,
                    ) {
                    Ok(Ok(amount)) => amount,
                    _ => 0,
                };

                // Validate pool balance >= min_distribution_amount * max_winners
                if min_distribution_amount > 0 && hunt.reward_config.max_winners > 0 {
                    let required = min_distribution_amount
                        .saturating_mul(hunt.reward_config.max_winners as i128);
                    if pool_balance < required {
                        return Err(HuntErrorCode::InsufficientRewardPool);
                    }
                }

                if !hunt.has_rewards_available() {
                    return Err(HuntErrorCode::InsufficientRewardPool);
                }
            }
        } else if old_status == HuntStatus::Paused {
            // Reactivation from paused: just basic checks
            // No need to recheck clues, rewards, etc. since it was already activated before
        } else {
            // Invalid status for activation
            return Err(HuntErrorCode::InvalidHuntStatus);
        }

        // Reject activation/reactivation if end_time is set and already in the past
        if hunt.end_time != 0 && hunt.end_time <= current_time {
            return Err(HuntErrorCode::HuntEndTimeInPast);
        }

        hunt.status = HuntStatus::Active;
        hunt.activated_at = current_time;

        Storage::save_hunt(&env, &hunt);

        // Emit appropriate event
        if old_status == HuntStatus::Draft {
            let event = HuntActivatedEvent {
                hunt_id,
                activated_at: current_time,
            };
            env.events()
                .publish((Symbol::new(&env, "HuntActivated"), hunt_id), event);
        } else if old_status == HuntStatus::Paused {
            let event = HuntReactivatedEvent {
                hunt_id,
                activated_at: current_time,
            };
            env.events()
                .publish((Symbol::new(&env, "HuntReactivated"), hunt_id), event);
        }

        // Emit HuntStatusChanged event
        Self::emit_hunt_status_changed(&env, hunt_id, old_status, HuntStatus::Active, current_time);

        Ok(())
    }

    pub fn deactivate_hunt(env: Env, hunt_id: u64, caller: Address) -> Result<(), HuntErrorCode> {
        // Fast validation using instance cache
        caller.require_auth();
        let cache = Self::get_hunt_cache_or_load(&env, hunt_id)?;
        if caller != cache.creator {
            return Err(HuntErrorCode::Unauthorized);
        }
        if cache.status != HuntStatus::Active {
            return Err(HuntErrorCode::InvalidHuntStatus);
        }

        // Validation passed — load full hunt from persistent for mutation
        let mut hunt = Storage::get_hunt(&env, hunt_id).ok_or(HuntErrorCode::HuntNotFound)?;
        hunt.status = HuntStatus::Paused;

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
        // Require the caller to authorize. Without this, an attacker could spoof `caller`
        // and cancel hunts by passing the creator address.
        caller.require_auth();

        // Fast validation using instance cache
        let cache = Self::get_hunt_cache_or_load(&env, hunt_id)?;
        if caller != cache.creator {
            return Err(HuntErrorCode::Unauthorized);
        }

        // Cannot cancel a completed hunt

        // Cannot cancel a completed or already-cancelled hunt
        if cache.status == HuntStatus::Completed {
            return Err(HuntErrorCode::InvalidHuntStatus);
        }
        if cache.status == HuntStatus::Cancelled {
            return Err(HuntErrorCode::InvalidHuntStatus);
        }

        let old_status = cache.status.clone();

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

    /// Force-closes (ends early) an in-progress hunt on behalf of its creator.
    ///
    /// Unlike [`cancel_hunt`], closing preserves all player scores and any
    /// rewards already collected: it marks the hunt `Completed` and triggers a
    /// final reward distribution for every player who has completed the hunt but
    /// not yet claimed. Players who have not completed the hunt keep their
    /// progress and are simply not rewarded. Any unspent reward-pool balance is
    /// left intact (a creator can refund it separately via [`cancel_hunt`] flows
    /// only while a hunt is still cancellable — see project docs).
    ///
    /// Only the creator may close a hunt, and only while it is `Active` or
    /// `Paused`. Closing a `Draft`, `Completed`, `Cancelled`, `EmergencyStopped`,
    /// or `Archived` hunt is rejected with `InvalidHuntStatus`.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `hunt_id` - The hunt to close
    /// * `caller` - The creator (must authorize the call via require_auth)
    ///
    /// # Returns
    /// `Ok(())` on success
    ///
    /// # Errors
    /// * `HuntNotFound` - Hunt does not exist
    /// * `Unauthorized` - Caller is not the hunt creator
    /// * `InvalidHuntStatus` - Hunt is not in an early-closable status
    /// * `RewardsPaused` - Reward distribution is globally paused
    /// * `InvalidRarity` - The hunt's configured NFT rarity is out of range
    /// * `RewardDistributionFailed` - A RewardManager cross-contract call failed
    pub fn close_hunt(env: Env, hunt_id: u64, caller: Address) -> Result<(), HuntErrorCode> {
        caller.require_auth();

        // Fast validation using instance cache
        let cache = Self::get_hunt_cache_or_load(&env, hunt_id)?;
        if caller != cache.creator {
            return Err(HuntErrorCode::Unauthorized);
        }
        // Only an in-progress hunt (Active or Paused) can be closed early.
        if cache.status != HuntStatus::Active && cache.status != HuntStatus::Paused {
            return Err(HuntErrorCode::InvalidHuntStatus);
        }

        // Closing distributes rewards, so honor the global rewards pause.
        if Storage::is_pause_rewards(&env) {
            return Err(HuntErrorCode::RewardsPaused);
        }

        let old_status = cache.status;

        // Load full hunt from persistent for mutation
        let mut hunt = Storage::get_hunt(&env, hunt_id).ok_or(HuntErrorCode::HuntNotFound)?;

        // Trigger final reward distribution for every completed, unclaimed player.
        // Scores and previously-claimed rewards are preserved untouched.
        let players = Storage::get_hunt_players(&env, hunt_id);
        let mut rewarded_players = 0u32;
        for i in 0..players.len() {
            // SAFETY: i is within the vector bounds established by the enclosing loop
            let mut progress = players.get(i).unwrap();
            if progress.is_completed && !progress.reward_claimed {
                Self::distribute_player_reward(&env, &mut hunt, &mut progress)?;
                rewarded_players = rewarded_players.saturating_add(1);
            }
        }

        // Mark the hunt inactive (closed early == Completed) and persist once.
        hunt.status = HuntStatus::Completed;
        Storage::save_hunt(&env, &hunt);

        let closed_at = env.ledger().timestamp();

        // Emit a dedicated close event plus the generic status-change event.
        let event = HuntClosedEvent {
            hunt_id,
            closed_at,
            rewarded_players,
        };
        env.events()
            .publish((Symbol::new(&env, "HuntClosed"), hunt_id), event);

        Self::emit_hunt_status_changed(&env, hunt_id, old_status, HuntStatus::Completed, closed_at);

        Ok(())
    }

    pub fn archive_hunt(env: Env, hunt_id: u64, caller: Address) -> Result<(), HuntErrorCode> {
        caller.require_auth();

        // Fast validation using instance cache
        let cache = Self::get_hunt_cache_or_load(&env, hunt_id)?;

        // Check if caller is creator OR admin
        let is_creator = caller == cache.creator;
        let is_admin = Storage::get_admin(&env) == Some(caller.clone());

        if !is_creator && !is_admin {
            return Err(HuntErrorCode::Unauthorized);
        }

        // Only allow archiving Completed or Cancelled hunts
        if cache.status != HuntStatus::Completed && cache.status != HuntStatus::Cancelled {
            return Err(HuntErrorCode::InvalidHuntStatus);
        }

        // Load full hunt from persistent for mutation
        let mut hunt = Storage::get_hunt(&env, hunt_id).ok_or(HuntErrorCode::HuntNotFound)?;
        let old_status = hunt.status.clone();

        // Archive the hunt
        hunt.status = HuntStatus::Archived;

        // Persist
        Storage::save_hunt(&env, &hunt);

        // Emit event
        let event = HuntArchivedEvent { hunt_id };
        env.events()
            .publish((Symbol::new(&env, "HuntArchived"), hunt_id), event);

        Self::emit_hunt_status_changed(
            &env,
            hunt_id,
            old_status,
            HuntStatus::Archived,
            env.ledger().timestamp(),
        );

        Ok(())
    }

    /// Reclaims the storage of a cancelled or archived hunt (issue #446).
    ///
    /// A cancelled hunt keeps every clue, player-progress, team, leaderboard
    /// and bookkeeping entry it ever wrote. Nothing referenced those entries
    /// any more, but nothing removed them either, so they sat in persistent
    /// storage paying rent until their TTL lapsed.
    ///
    /// Only `Cancelled` and `Archived` hunts may be collected — those are the
    /// two terminal states. Anything else is rejected with `InvalidHuntStatus`,
    /// because collecting a live hunt would destroy player progress.
    ///
    /// The sweep is **idempotent**: running it twice reports zero the second
    /// time rather than failing, so an interrupted call is safe to retry.
    ///
    /// # Authorization
    /// The hunt creator or the contract admin.
    ///
    /// # Returns
    /// A [`GcReport`] describing what was reclaimed.
    pub fn gc_hunt(env: Env, hunt_id: u64, caller: Address) -> Result<GcReport, HuntErrorCode> {
        caller.require_auth();

        // Read status from the full record rather than the instance cache: the
        // cache entry is itself one of the things this function deletes, so a
        // retry after a partial sweep must not depend on it.
        let hunt = Storage::get_hunt(&env, hunt_id).ok_or(HuntErrorCode::HuntNotFound)?;

        let is_creator = caller == hunt.creator;
        let is_admin = Storage::get_admin(&env) == Some(caller.clone());
        if !is_creator && !is_admin {
            return Err(HuntErrorCode::Unauthorized);
        }

        if hunt.status != HuntStatus::Cancelled && hunt.status != HuntStatus::Archived {
            return Err(HuntErrorCode::InvalidHuntStatus);
        }

        let report = Storage::gc_hunt_storage(&env, hunt_id);

        let collected_at = env.ledger().timestamp();
        env.events().publish(
            (Symbol::new(&env, "HuntGarbageCollected"), hunt_id),
            HuntGarbageCollectedEvent {
                hunt_id,
                total_removed: report.total_removed,
                collected_at,
            },
        );

        Ok(report)
    }

    /// Reports how much storage a hunt currently occupies, without removing
    /// anything. Read-only, so it needs no authorization — hunt existence and
    /// size are already public via `get_hunt_info`.
    pub fn get_hunt_storage_footprint(env: Env, hunt_id: u64) -> GcReport {
        Storage::count_hunt_storage_entries(&env, hunt_id)
    }

    pub fn get_hunt_info(env: Env, hunt_id: u64) -> Result<Hunt, HuntErrorCode> {
        let hunt = Storage::get_hunt(&env, hunt_id).ok_or(HuntErrorCode::HuntNotFound)?;

        // Return the full Hunt struct. Hunt info is intentionally available in
        // every status (Draft, Active, Completed, Cancelled, Paused,
        // EmergencyStopped, Archived); there is no per-status gating to apply
        // for a read-only getter, so the previous exhaustive-but-empty match
        // over `hunt.status` was dead code and has been removed.
        Ok(hunt)
    }

    /// Convenience helper used in tests to set reward configuration on a hunt.
    /// Sets nft_image_uri to a placeholder when nft_enabled is true.
    pub fn set_reward_config(
        env: Env,
        hunt_id: u64,
        max_winners: u32,
        xlm_pool: i128,
        nft_enabled: bool,
        nft_contract: Option<Address>,
    ) -> Result<(), HuntErrorCode> {
        let mut hunt = Storage::get_hunt(&env, hunt_id).ok_or(HuntErrorCode::HuntNotFound)?;
        let uri = if nft_enabled {
            Some(String::from_str(&env, "https://example.com/nft.png"))
        } else {
            None
        };
        hunt.reward_config = RewardConfig::new(
            &env,
            xlm_pool,
            nft_enabled,
            nft_contract,
            max_winners,
            0,
            0,
            uri,
        );
        Storage::save_hunt(&env, &hunt);
        Ok(())
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

    /// Blacklists a creator address, preventing them from creating new hunts.
    /// Caller must be the admin.
    pub fn blacklist_creator(
        env: Env,
        admin: Address,
        creator: Address,
    ) -> Result<(), HuntErrorCode> {
        Self::require_admin(&env, &admin)?;
        Storage::blacklist_creator(&env, &creator);
        env.events().publish(
            (Symbol::new(&env, "CreatorBlacklisted"), creator.clone()),
            CreatorBlacklistedEvent { creator, admin },
        );
        Ok(())
    }

    /// Removes a creator from the blacklist, restoring their ability to create hunts.
    /// Caller must be the admin.
    pub fn remove_from_blacklist(
        env: Env,
        admin: Address,
        creator: Address,
    ) -> Result<(), HuntErrorCode> {
        Self::require_admin(&env, &admin)?;
        Storage::remove_from_blacklist(&env, &creator);
        env.events().publish(
            (
                Symbol::new(&env, "CreatorRemovedFromBlacklist"),
                creator.clone(),
            ),
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
    /// Reward amounts can be either flat (`xlm_pool / max_winners`) or
    /// time-based (configured via `RewardManager::set_pool_tiers`), in which
    /// case the amount depends on `completion_at - started_at` for the
    /// completing player.
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
    /// * `InvalidHuntStatus` - Hunt is not Active (e.g. already Completed or Cancelled)
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

        // Reward claims are only valid while the hunt is Active (a Cancelled hunt's
        // pool has already been refunded; Draft/Paused/Archived hunts never had one claimed).
        if hunt.status != HuntStatus::Active {
            return Err(HuntErrorCode::InvalidHuntStatus);
        }

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

        if hunt.reward_config.max_winners == 0 {
            return Err(HuntErrorCode::NoRewardsConfigured);
        }

        // #832: Enforce max_winners cap before any reward movement
        if hunt.reward_config.claimed_count >= hunt.reward_config.max_winners {
            return Err(HuntErrorCode::InsufficientRewardPool);
        }

        // Distribute the reward, mark the player as claimed, and emit the event.
        Self::distribute_player_reward(&env, &mut hunt, &mut progress)?;

        // Persist the hunt's updated claimed_count.
        Storage::save_hunt(&env, &hunt);

        Ok(())
    }

    /// Distributes the reward for a single completed, unclaimed player.
    ///
    /// Resolves the player's XLM amount (flat or tier-based), invokes the
    /// RewardManager (if configured and there is at least one reward type),
    /// marks the player's progress as claimed, increments the hunt's
    /// `claimed_count` (in memory — the caller is responsible for persisting
    /// the hunt), and emits a `RewardClaimed` event.
    ///
    /// The caller must ensure `progress.is_completed == true` and
    /// `progress.reward_claimed == false` before invoking this.
    ///
    /// # Errors
    /// * `InvalidRarity` - The hunt's configured NFT rarity is out of range
    /// * `RewardDistributionFailed` - The RewardManager cross-contract call failed
    fn distribute_player_reward(
        env: &Env,
        hunt: &mut Hunt,
        progress: &mut PlayerProgress,
    ) -> Result<(), HuntErrorCode> {
        // ===================== TIER-BASED AMOUNT RESOLUTION =====================
        // If the reward pool has a tier schedule configured, the appropriate
        // tier's xlm_amount replaces the flat `xlm_pool / max_winners` amount.
        // Tier resolution is `(max_completion_secs - registration_time)` based.
        let reward_amount = Self::resolve_reward_amount(env, hunt, progress);
        // =======================================================================
        let nft_awarded = hunt.reward_config.nft_enabled;

        // #834: Only validate rarity when NFT rewards are actually enabled
        if nft_awarded && !Self::validate_rarity(hunt.reward_config.nft_rarity) {
            return Err(HuntErrorCode::InvalidRarity);
        }

        // Call RewardManager if configured and there are rewards to distribute
        if let Some(reward_manager_addr) = Storage::get_reward_manager(env) {
            let xlm_amount = if reward_amount > 0 {
                Some(reward_amount)
            } else {
                None
            };
            // #833: Thread nft_image_uri from hunt.reward_config into the cross-contract call
            let (nft_contract, nft_title, nft_desc, nft_uri, nft_hunt_title) = if nft_awarded {
                hunt.reward_config
                    .nft_contract
                    .clone()
                    .map(|nft_contract| {
                        let uri = hunt
                            .reward_config
                            .nft_image_uri
                            .clone()
                            .unwrap_or_else(|| String::from_str(env, ""));
                        (
                            Some(nft_contract),
                            hunt.title.clone(),
                            hunt.description.clone(),
                            uri,
                            hunt.title.clone(),
                        )
                    })
                    .unwrap_or((
                        None,
                        String::from_str(env, ""),
                        String::from_str(env, ""),
                        String::from_str(env, ""),
                        String::from_str(env, ""),
                    ))
            } else {
                (
                    None,
                    String::from_str(env, ""),
                    String::from_str(env, ""),
                    String::from_str(env, ""),
                    String::from_str(env, ""),
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
                completion_rank: progress.completion_rank,
            };

            // Only call RewardManager when there is at least one reward type
            if rm_reward_config.is_valid() {
                let mut args: Vec<Val> = Vec::new(env);
                args.push_back(hunt.hunt_id.into_val(env));
                args.push_back(progress.player.clone().into_val(env));
                args.push_back(rm_reward_config.into_val(env));

                let result = env.try_invoke_contract::<(), RewardErrorCode>(
                    &reward_manager_addr,
                    &Symbol::new(env, "distribute_rewards"),
                    args,
                );
                if !matches!(result, Ok(Ok(()))) {
                    return Err(HuntErrorCode::RewardDistributionFailed);
                }
            }
        }

        // Update player progress
        progress.reward_claimed = true;
        Storage::save_player_progress(env, progress);

        // #832: Use checked_add for claimed_count to guard against overflow
        hunt.reward_config.claimed_count = hunt
            .reward_config
            .claimed_count
            .checked_add(1)
            .ok_or(HuntErrorCode::InsufficientRewardPool)?;

        // Once every reward slot has been claimed, the hunt itself is done.
        // The status change is persisted by the caller along with claimed_count.
        let just_completed = hunt.reward_config.claimed_count >= hunt.reward_config.max_winners;
        if just_completed {
            hunt.status = HuntStatus::Completed;
        }

        // Emit RewardClaimedEvent
        let event = RewardClaimedEvent {
            hunt_id: hunt.hunt_id,
            player: progress.player.clone(),
            xlm_amount: reward_amount,
            nft_awarded,
        };
        env.events()
            .publish((Symbol::new(env, "RewardClaimed"), hunt.hunt_id), event);

        if just_completed {
            let current_time = env.ledger().timestamp();
            Self::emit_hunt_status_changed(
                env,
                hunt.hunt_id,
                HuntStatus::Active,
                HuntStatus::Completed,
                current_time,
            );
        }

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

        let hunt = Storage::get_hunt(&env, hunt_id).ok_or(HuntErrorCode::HuntNotFound)?;

        if hunt.status != HuntStatus::Active {
            return Err(HuntErrorCode::InvalidHuntStatus);
        }

        let current_time = env.ledger().timestamp();

        // Reject registration if the hunt has not started yet
        if hunt.start_time != 0 && current_time < hunt.start_time {
            return Err(HuntErrorCode::HuntNotStarted);
        }

        // Reject public registration for private hunts
        if hunt.is_private {
            return Err(HuntErrorCode::InvalidHuntStatus);
        }

        // Cache read: cheaper than loading full Hunt from persistent storage
        let _cache = Self::validate_hunt_active_cached(&env, hunt_id)?;

        // Enforce the registration deadline if the creator configured one
        if hunt.registration_deadline != 0 && current_time >= hunt.registration_deadline {
            return Err(HuntErrorCode::RegistrationsPaused);
        }

        let progress = PlayerProgress::new(&env, player.clone(), hunt_id, current_time);
        Storage::save_player_progress(&env, &progress);

        let event = PlayerRegisteredEvent {
            hunt_id,
            player: player.clone(),
        };
        env.events()
            .publish((Symbol::new(&env, "PlayerRegistered"), hunt_id), event);

        Ok(())
    }

    /// Generates or updates the invite code for a private hunt.
    ///
    /// The invite code is hashed with SHA256 (using hunt_id as salt) and only the hash
    /// is stored on-chain. The plain-text code is never persisted or emitted in events.
    /// Calling this function overwrites any previously set invite code.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `hunt_id` - The hunt to generate an invite code for
    /// * `creator` - The hunt creator (must authorize the call)
    /// * `invite_code` - The plain-text invite code to hash and store
    ///
    /// # Returns
    /// `Ok(())` on success
    ///
    /// # Errors
    /// * `HuntNotFound` - Hunt does not exist
    /// * `Unauthorized` - Caller is not the hunt creator
    /// * `InvalidHuntStatus` - Hunt is not in Draft status
    pub fn generate_invite_code(
        env: Env,
        hunt_id: u64,
        creator: Address,
        invite_code: String,
    ) -> Result<(), HuntErrorCode> {
        creator.require_auth();

        let mut hunt = Storage::get_hunt(&env, hunt_id).ok_or(HuntErrorCode::HuntNotFound)?;

        if hunt.creator != creator {
            return Err(HuntErrorCode::Unauthorized);
        }

        if hunt.status != HuntStatus::Draft {
            return Err(HuntErrorCode::InvalidHuntStatus);
        }

        // Hash the invite code with hunt_id as salt to prevent rainbow-table attacks.
        // Use the same buffer-based approach as normalize_and_hash_answer for consistency.
        let code_len = invite_code.len() as usize;
        if code_len == 0 {
            return Err(HuntErrorCode::InvalidAnswer);
        }
        let mut buf = [0u8; 264]; // 8 (hunt_id) + 256 (max invite code)
        buf[..8].copy_from_slice(&hunt_id.to_be_bytes());
        invite_code.copy_into_slice(&mut buf[8..8 + code_len]);
        let salted = Bytes::from_slice(&env, &buf[..8 + code_len]);
        let hash = env.crypto().sha256(&salted);
        let hash_bytes: BytesN<32> = hash.to_bytes();

        hunt.invite_code_hash = Some(hash_bytes);
        Storage::save_hunt(&env, &hunt);

        let event = InviteCodeGeneratedEvent {
            hunt_id,
            creator: creator.clone(),
        };
        env.events()
            .publish((Symbol::new(&env, "InviteCodeGenerated"), hunt_id), event);

        Ok(())
    }

    /// Sets whether a hunt is private (invite-only).
    ///
    /// Only the hunt creator can call this, and only while the hunt is in Draft status.
    /// When making a hunt private, an invite code must already be configured via
    /// `generate_invite_code` before the hunt can be activated.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `hunt_id` - The hunt to update privacy for
    /// * `creator` - The hunt creator (must authorize the call)
    /// * `is_private` - Whether the hunt should be invite-only
    ///
    /// # Returns
    /// `Ok(())` on success
    ///
    /// # Errors
    /// * `HuntNotFound` - Hunt does not exist
    /// * `Unauthorized` - Caller is not the hunt creator
    /// * `InvalidHuntStatus` - Hunt is not in Draft status
    pub fn set_hunt_privacy(
        env: Env,
        hunt_id: u64,
        creator: Address,
        is_private: bool,
    ) -> Result<(), HuntErrorCode> {
        creator.require_auth();

        let mut hunt = Storage::get_hunt(&env, hunt_id).ok_or(HuntErrorCode::HuntNotFound)?;

        if hunt.creator != creator {
            return Err(HuntErrorCode::Unauthorized);
        }

        if hunt.status != HuntStatus::Draft {
            return Err(HuntErrorCode::InvalidHuntStatus);
        }

        hunt.is_private = is_private;
        Storage::save_hunt(&env, &hunt);

        // Emit a status-changed event so off-chain indexers can track privacy toggles
        // Privacy toggling does not change the hunt's status: it stays in Draft.
        let current_time = env.ledger().timestamp();
        let old_status = HuntStatus::Draft;
        Self::emit_hunt_status_changed(
            &env,
            hunt_id,
            old_status,
            hunt.status.clone(),
            current_time,
        );

        Ok(())
    }

    /// Clears the invite code for a private hunt, effectively pausing new registrations.
    /// The hunt creator can generate a new code later via `generate_invite_code`.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `hunt_id` - The hunt to revoke the invite code for
    /// * `creator` - The hunt creator (must authorize the call)
    ///
    /// # Returns
    /// `Ok(())` on success
    ///
    /// # Errors
    /// * `HuntNotFound` - Hunt does not exist
    /// * `Unauthorized` - Caller is not the hunt creator
    /// * `InvalidHuntStatus` - Hunt is not in Draft status
    pub fn revoke_invite_code(
        env: Env,
        hunt_id: u64,
        creator: Address,
    ) -> Result<(), HuntErrorCode> {
        creator.require_auth();

        let mut hunt = Storage::get_hunt(&env, hunt_id).ok_or(HuntErrorCode::HuntNotFound)?;

        if hunt.creator != creator {
            return Err(HuntErrorCode::Unauthorized);
        }

        if hunt.status != HuntStatus::Draft {
            return Err(HuntErrorCode::InvalidHuntStatus);
        }

        hunt.invite_code_hash = None;
        Storage::save_hunt(&env, &hunt);

        let event = InviteCodeRevokedEvent {
            hunt_id,
            creator: creator.clone(),
        };
        env.events()
            .publish((Symbol::new(&env, "InviteCodeRevoked"), hunt_id), event);

        Ok(())
    }

    /// Registers a player for a private hunt using a valid invite code.
    ///
    /// The provided invite code is hashed (with hunt_id as salt) and compared against
    /// the stored `invite_code_hash`. If they match, the player is registered.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `hunt_id` - The private hunt to register for
    /// * `player` - The address of the player (must authorize the call via require_auth)
    /// * `invite_code` - The plain-text invite code to validate
    ///
    /// # Returns
    /// `Ok(())` on success
    ///
    /// # Errors
    /// * `HuntNotFound` - Hunt does not exist
    /// * `InvalidHuntStatus` - Hunt is not in Active status, is not private (use
    ///   `register_player` instead), or has no invite code configured
    /// * `InvalidAnswer` - The provided invite code is empty or does not match
    /// * `DuplicateRegistration` - Player is already registered for this hunt
    pub fn register_with_invite(
        env: Env,
        hunt_id: u64,
        player: Address,
        invite_code: String,
    ) -> Result<(), HuntErrorCode> {
        player.require_auth();

        if Storage::is_pause_registrations(&env) {
            return Err(HuntErrorCode::RegistrationsPaused);
        }

        let hunt = Storage::get_hunt(&env, hunt_id).ok_or(HuntErrorCode::HuntNotFound)?;

        if hunt.status != HuntStatus::Active {
            return Err(HuntErrorCode::InvalidHuntStatus);
        }

        // Ensure the hunt is private and has an invite code configured.
        // If hunt is not private, tell the caller to use register_player instead.
        if !hunt.is_private {
            return Err(HuntErrorCode::InvalidHuntStatus);
        }

        let stored_hash = hunt
            .invite_code_hash
            .ok_or(HuntErrorCode::InvalidHuntStatus)?;

        // Hash the provided invite code with the same salt (hunt_id) and compare.
        // Use the same buffer-based approach as generate_invite_code for consistency.
        let code_len = invite_code.len() as usize;
        if code_len == 0 {
            return Err(HuntErrorCode::InvalidAnswer);
        }
        let mut buf = [0u8; 264]; // 8 (hunt_id) + 256 (max invite code)
        buf[..8].copy_from_slice(&hunt_id.to_be_bytes());
        invite_code.copy_into_slice(&mut buf[8..8 + code_len]);
        let salted = Bytes::from_slice(&env, &buf[..8 + code_len]);
        let computed_hash = env.crypto().sha256(&salted);
        let computed_hash_bytes: BytesN<32> = computed_hash.to_bytes();

        if computed_hash_bytes != stored_hash {
            return Err(HuntErrorCode::InvalidAnswer);
        }

        let current_time = env.ledger().timestamp();

        // Reject registration if the hunt has not started yet
        if hunt.start_time != 0 && current_time < hunt.start_time {
            return Err(HuntErrorCode::HuntNotStarted);
        }

        // Cache read: cheaper than loading full Hunt from persistent storage
        let _cache = Self::validate_hunt_active_cached(&env, hunt_id)?;

        if Storage::get_player_progress(&env, hunt_id, &player).is_some() {
            return Err(HuntErrorCode::DuplicateRegistration);
        }

        if hunt.max_players > 0 {
            let count = Storage::get_player_count(&env, hunt_id);
            if count >= hunt.max_players {
                return Err(HuntErrorCode::HuntFull);
            }
        }

        let progress = PlayerProgress::new(&env, player.clone(), hunt_id, current_time);
        Storage::save_player_progress(&env, &progress);

        let event = PlayerRegisteredWithInviteEvent {
            hunt_id,
            player: player.clone(),
        };
        env.events().publish(
            (Symbol::new(&env, "PlayerRegisteredWithInvite"), hunt_id),
            event,
        );

        Ok(())
    }

    /// Verifies a candidate answer for a registered player with authorization and rate limiting.
    ///
    /// Unlike `submit_answer`, `preview_answer` does not mark the clue as completed, award points,
    /// or emit clue completion events, but requires player authorization and enforces the same
    /// per-minute rate limits and attempt cooldowns to prevent brute-force dictionary attacks.
    pub fn preview_answer(
        env: Env,
        hunt_id: u64,
        clue_id: u32,
        player: Address,
        answer: String,
    ) -> Result<bool, HuntErrorCode> {
        // Require player authorization
        player.require_auth();

        if Storage::is_pause_answers(&env) {
            return Err(HuntErrorCode::AnswersPaused);
        }

        let hunt = Storage::get_hunt(&env, hunt_id).ok_or(HuntErrorCode::HuntNotFound)?;

        let current_time = env.ledger().timestamp();
        let _cache = Self::validate_hunt_active_cached(&env, hunt_id)?;

        if Storage::is_banned(&env, hunt_id, &player) {
            return Err(HuntErrorCode::BannedPlayer);
        }

        let mut progress = Storage::get_player_progress(&env, hunt_id, &player)
            .ok_or(HuntErrorCode::PlayerNotRegistered)?;

        let clue = Storage::get_clue(&env, hunt_id, clue_id).ok_or(HuntErrorCode::ClueNotFound)?;

        if progress.has_completed_clue(clue_id) {
            return Err(HuntErrorCode::ClueAlreadyCompleted);
        }

        if Self::team_has_completed_clue(&env, &hunt, &player, clue_id) {
            return Err(HuntErrorCode::ClueAlreadyCompleted);
        }

        if hunt.max_submissions_per_minute > 0 {
            let mut updated_submissions = Vec::new(&env);
            for i in 0..progress.recent_submissions.len() {
                let ts = progress
                    .recent_submissions
                    .get(i)
                    .ok_or(HuntErrorCode::CorruptPlayerProgress)?;
                if current_time < ts + 60 {
                    updated_submissions.push_back(ts);
                }
            }
            progress.recent_submissions = updated_submissions;

            if progress.recent_submissions.len() >= hunt.max_submissions_per_minute {
                return Err(HuntErrorCode::RateLimitExceeded);
            }
            progress.recent_submissions.push_back(current_time);
        }

        if hunt.attempt_cooldown_secs > 0 {
            if let Some(last_attempt) = progress.clue_last_attempts.get(clue_id) {
                if current_time < last_attempt + (hunt.attempt_cooldown_secs as u64) {
                    return Err(HuntErrorCode::from(HuntError::AttemptCooldownNotExpired));
                }
            }
            progress.clue_last_attempts.set(clue_id, current_time);
        }

        Storage::save_player_progress(&env, &progress);

        let submitted_hash = Self::normalize_and_hash_answer(&env, hunt_id, clue_id, &answer)
            .map_err(HuntErrorCode::from)?;

        let correct = Self::is_answer_correct(&clue, &submitted_hash);
        let preview_event = AnswerPreviewedEvent {
            hunt_id,
            player: player.clone(),
            clue_id,
            is_correct: correct,
            timestamp: current_time,
        };
        env.events().publish(
            (Symbol::new(&env, "AnswerPreviewed"), hunt_id, clue_id),
            preview_event,
        );

        Ok(correct)
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
                                           // Use saturating multiplication to prevent overflow on very large elapsed times
        let decrease_bps = decrease_steps.saturating_mul(5000); // 5000 bps = 0.5x per step
                                                                // Cap at u32::MAX before truncating to prevent silent wrap-around
        let decrease_bps_u32 = if decrease_bps > u32::MAX as u64 {
            u32::MAX
        } else {
            decrease_bps as u32
        };
        let multiplier_bps = core::cmp::max(
            10000, // Minimum 1x
            hunt.start_multiplier_bps.saturating_sub(decrease_bps_u32),
        );
        let base_points = clue
            .points
            .saturating_mul(clue.difficulty)
            .saturating_mul(clue.weight);
        // Use saturating arithmetic for the score multiplication to prevent overflow
        let score = (base_points as u64)
            .saturating_mul(multiplier_bps as u64)
            .saturating_div(10000);
        // Clamp to u32::MAX to prevent silent truncation
        if score > u32::MAX as u64 {
            u32::MAX
        } else {
            score as u32
        }
    }

    /// In team mode, returns true if any teammate has already completed this clue.
    fn team_has_completed_clue(env: &Env, hunt: &Hunt, player: &Address, clue_id: u32) -> bool {
        if !hunt.team_mode {
            return false;
        }
        let Some(team_id) = Storage::get_player_team(env, hunt.hunt_id, player) else {
            return false;
        };
        let team_progress = Storage::get_team_progress(env, hunt.hunt_id, team_id);
        team_progress.completed_clues.contains(clue_id)
    }

    /// In team mode, records a clue completion against the player's team so
    /// teammates see it as already solved and share the earned score.
    fn record_team_clue_completion(
        env: &Env,
        hunt: &Hunt,
        player: &Address,
        clue_id: u32,
        score: u32,
    ) {
        if !hunt.team_mode {
            return;
        }
        let Some(team_id) = Storage::get_player_team(env, hunt.hunt_id, player) else {
            return;
        };
        let mut team_progress = Storage::get_team_progress(env, hunt.hunt_id, team_id);
        if team_progress.completed_clues.contains(clue_id) {
            return;
        }
        team_progress.completed_clues.push_back(clue_id);
        team_progress.total_score = team_progress.total_score.saturating_add(score);
        Storage::save_team_progress(env, hunt.hunt_id, team_id, &team_progress);
    }

    fn is_answer_correct(clue: &Clue, submitted_hash: &BytesN<32>) -> bool {
        for i in 0..clue.answer_hashes.len() {
            // Stored state: prefer typed absence over panic on inconsistent clue data.
            let Some(stored_hash) = clue.answer_hashes.get(i) else {
                return false;
            };
            if stored_hash == *submitted_hash {
                return true;
            }
        }
        false
    }

    #[allow(clippy::too_many_arguments)]
    fn finalize_answer_submission(
        env: &Env,
        hunt: &Hunt,
        clue: &Clue,
        progress: &mut PlayerProgress,
        player: &Address,
        hunt_id: u64,
        clue_id: u32,
        current_time: u64,
        answer_correct: bool,
        record_failed_submission: bool,
    ) -> Result<(), HuntErrorCode> {
        if !answer_correct {
            if record_failed_submission && hunt.max_submissions_per_minute > 0 {
                progress.recent_submissions.push_back(current_time);
            }
            Storage::save_player_progress(env, progress);
            let incorrect_event = AnswerIncorrectEvent {
                hunt_id,
                player: player.clone(),
                clue_id,
                timestamp: current_time,
            };
            env.events().publish(
                (Symbol::new(env, "AnswerIncorrect"), hunt_id, clue_id),
                incorrect_event,
            );
            return Err(HuntErrorCode::InvalidAnswer);
        }

        let score = Self::calculate_score(hunt, clue, progress.started_at, current_time);
        progress.complete_clue(env, clue_id, score)?;
        Self::record_team_clue_completion(env, hunt, player, clue_id, score);

        if hunt.max_submissions_per_minute > 0 {
            progress.recent_submissions = Vec::new(env);
        }

        let all_required_completed =
            Self::check_all_required_clues_completed(env, hunt_id, progress);

        if all_required_completed && !progress.is_completed {
            progress.is_completed = true;
            progress.completed_at = current_time;

            let mut hunt_mut =
                Storage::get_hunt(env, hunt_id).ok_or(HuntErrorCode::HuntNotFound)?;
            hunt_mut.completed_count += 1;
            let rank = hunt_mut.completed_count;
            Storage::save_hunt(env, &hunt_mut);
            Storage::increment_player_completed_hunt_count(env, player);
            // Freeze the rank on the player's progress record so it is
            // available as an authoritative value at reward-claim time.
            progress.completion_rank = rank;
            let hunt_completed_event = HuntCompletedEvent {
                hunt_id,
                player: player.clone(),
                total_score: progress.total_score,
                completion_time: current_time,
                completion_rank: rank,
            };
            env.events().publish(
                (Symbol::new(env, "HuntCompleted"), hunt_id),
                hunt_completed_event,
            );
        }

        Storage::save_player_progress(env, progress);
        Self::update_leaderboard_index(env, progress);

        let clue_completed_event = ClueCompletedEvent {
            hunt_id,
            player: player.clone(),
            clue_id,
            points_earned: score,
        };
        env.events().publish(
            (Symbol::new(env, "ClueCompleted"), hunt_id, clue_id),
            clue_completed_event,
        );

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
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

        // 1. Verify hunt exists and is active
        let hunt = Storage::get_hunt(&env, hunt_id).ok_or(HuntErrorCode::HuntNotFound)?;

        let current_time = env.ledger().timestamp();

        // Fast validation using instance cache (cheaper than persistent read)
        let _cache = Self::validate_hunt_active_cached(&env, hunt_id)?;

        if Storage::is_banned(&env, hunt_id, &player) {
            return Err(HuntErrorCode::BannedPlayer);
        }

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

        // All cheap validation (player registration, clue existence, completion state, rate
        // limits) runs BEFORE we write the processed-submission entry.  This prevents nonce
        // exhaustion on validation failures and stops unregistered addresses from bloating
        // ledger storage.  The replay guard above is a read-only check and stays in place.
        let mut progress = Storage::get_player_progress(&env, hunt_id, &player)
            .ok_or(HuntErrorCode::PlayerNotRegistered)?;

        let clue = Storage::get_clue(&env, hunt_id, clue_id).ok_or(HuntErrorCode::ClueNotFound)?;

        if progress.has_completed_clue(clue_id) {
            return Err(HuntErrorCode::ClueAlreadyCompleted);
        }

        // In team mode, a clue solved by any teammate counts as completed for the team
        if Self::team_has_completed_clue(&env, &hunt, &player, clue_id) {
            return Err(HuntErrorCode::ClueAlreadyCompleted);
        }

        if hunt.max_submissions_per_minute > 0 {
            let mut updated_submissions = Vec::new(&env);
            for i in 0..progress.recent_submissions.len() {
                // Stored state may be inconsistent — return a typed error instead of aborting.
                let ts = progress
                    .recent_submissions
                    .get(i)
                    .ok_or(HuntErrorCode::CorruptPlayerProgress)?;
                if current_time < ts + 60 {
                    updated_submissions.push_back(ts);
                }
            }
            progress.recent_submissions = updated_submissions;

            if progress.recent_submissions.len() >= hunt.max_submissions_per_minute {
                // Stored state may be inconsistent — return a typed error instead of aborting.
                let oldest_ts = progress
                    .recent_submissions
                    .get(0)
                    .ok_or(HuntErrorCode::CorruptPlayerProgress)?;
                let elapsed = current_time.saturating_sub(oldest_ts);
                let _cooldown_remaining = 60u64.saturating_sub(elapsed);
                return Err(HuntErrorCode::from(HuntError::RateLimitExceeded));
            }
            progress.recent_submissions.push_back(current_time);
        }

        if hunt.attempt_cooldown_secs > 0 {
            if let Some(last_attempt) = progress.clue_last_attempts.get(clue_id) {
                if current_time < last_attempt + (hunt.attempt_cooldown_secs as u64) {
                    let _cooldown_remaining =
                        (last_attempt + (hunt.attempt_cooldown_secs as u64)) - current_time;
                    return Err(HuntErrorCode::from(HuntError::AttemptCooldownNotExpired));
                }
            }
            progress.clue_last_attempts.set(clue_id, current_time);
        }

        // All validation passed — mark the nonce as consumed so the same envelope cannot be
        // replayed, then proceed to answer evaluation.
        Storage::save_processed_submission(
            &env,
            hunt_id,
            clue_id,
            &player,
            submission_nonce,
            submitted_at,
            submitted_at.saturating_add(ANSWER_SUBMISSION_WINDOW_SECS),
        );

        let submitted_hash = Self::normalize_and_hash_answer(&env, hunt_id, clue_id, &answer)
            .map_err(HuntErrorCode::from)?;

        let answer_correct = Self::is_answer_correct(&clue, &submitted_hash);
        Self::finalize_answer_submission(
            &env,
            &hunt,
            &clue,
            &mut progress,
            &player,
            hunt_id,
            clue_id,
            current_time,
            answer_correct,
            false,
        )?;

        Ok(())
    }

    /// Variant of `submit_answer` which accepts a precomputed SHA256 answer hash.
    /// This avoids on-chain normalization and hashing when the client supplies
    /// the correctly computed `answer_hash = SHA256(hunt_id || clue_id || normalized_answer)`.
    /// Use this from off-chain callers that can perform normalization+hashing cheaply.
    #[allow(clippy::too_many_arguments)]
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

        // 1. Verify hunt exists and is active
        let hunt = Storage::get_hunt(&env, hunt_id).ok_or(HuntErrorCode::HuntNotFound)?;

        let current_time = env.ledger().timestamp();
        if !hunt.is_active(current_time) {
            return Err(HuntErrorCode::HuntNotActive);
        }

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

        // All cheap validation (player registration, clue existence, completion state, rate
        // limits) runs BEFORE we write the processed-submission entry.  This prevents nonce
        // exhaustion on validation failures and stops unregistered addresses from bloating
        // ledger storage.  The replay guard above is a read-only check and stays in place.
        let mut progress = Storage::get_player_progress(&env, hunt_id, &player)
            .ok_or(HuntErrorCode::PlayerNotRegistered)?;

        let clue = Storage::get_clue(&env, hunt_id, clue_id).ok_or(HuntErrorCode::ClueNotFound)?;

        if progress.has_completed_clue(clue_id) {
            return Err(HuntErrorCode::ClueAlreadyCompleted);
        }

        // In team mode, a clue solved by any teammate counts as completed for the team
        if Self::team_has_completed_clue(&env, &hunt, &player, clue_id) {
            return Err(HuntErrorCode::ClueAlreadyCompleted);
        }

        if hunt.max_submissions_per_minute > 0 {
            let mut updated_submissions = Vec::new(&env);
            for i in 0..progress.recent_submissions.len() {
                // Stored state may be inconsistent — return a typed error instead of aborting.
                let ts = progress
                    .recent_submissions
                    .get(i)
                    .ok_or(HuntErrorCode::CorruptPlayerProgress)?;
                if current_time < ts + 60 {
                    updated_submissions.push_back(ts);
                }
            }
            progress.recent_submissions = updated_submissions;

            if progress.recent_submissions.len() >= hunt.max_submissions_per_minute {
                // Stored state may be inconsistent — return a typed error instead of aborting.
                let oldest_ts = progress
                    .recent_submissions
                    .get(0)
                    .ok_or(HuntErrorCode::CorruptPlayerProgress)?;
                let elapsed = current_time.saturating_sub(oldest_ts);
                let _cooldown_remaining = 60u64.saturating_sub(elapsed);
                return Err(HuntErrorCode::from(HuntError::RateLimitExceeded));
            }
            progress.recent_submissions.push_back(current_time);
        }

        // All validation passed — mark the nonce as consumed so the same envelope cannot be
        // replayed, then proceed to answer evaluation.
        Storage::save_processed_submission(
            &env,
            hunt_id,
            clue_id,
            &player,
            submission_nonce,
            submitted_at,
            submitted_at.saturating_add(ANSWER_SUBMISSION_WINDOW_SECS),
        );

        let answer_correct = Self::is_answer_correct(&clue, &answer_hash);
        Self::finalize_answer_submission(
            &env,
            &hunt,
            &clue,
            &mut progress,
            &player,
            hunt_id,
            clue_id,
            current_time,
            answer_correct,
            true,
        )?;

        Ok(())
    }

    #[allow(dead_code)]
    fn completion_rank(env: &Env, hunt_id: u64) -> u32 {
        let players = Storage::get_hunt_players(env, hunt_id);
        let mut completed_players = 0u32;
        for i in 0..players.len() {
            // SAFETY: i is within the vector bounds established by the enclosing loop
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
            return Err(HuntError::SubmissionExpired);
        }
        if current_time.saturating_sub(submitted_at) > ANSWER_SUBMISSION_WINDOW_SECS {
            return Err(HuntError::SubmissionExpired);
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
                return Err(HuntError::DuplicateSubmission);
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
        let Some(hunt) = Storage::get_hunt(env, hunt_id) else {
            return false;
        };

        if hunt.required_clues == 0 {
            return true;
        }

        // Quick early exit: player hasn't completed enough clues total
        if progress.completed_clues.len() < hunt.required_clues {
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
                // SAFETY: i is within the vector bounds established by the enclosing loop
                let clue = all_clues.get(i).unwrap();
                if clue.is_required && !progress.has_completed_clue(clue.clue_id) {
                    return false;
                }
            }
            return true;
        }

        // Fast path: check only the required clue IDs
        for i in 0..required_ids.len() {
            // SAFETY: i is within the vector bounds established by the enclosing loop
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
    ///
    /// Access is governed by the hunt's `leaderboard_visibility` setting:
    /// * `Public` – any caller (pass `None` for anonymous access).
    /// * `RegisteredOnly` – caller must be a registered player for the hunt.
    /// * `CreatorOnly` – caller must be the hunt creator.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `hunt_id` - The hunt to query
    /// * `limit` - Maximum entries to return (capped at `MAX_LEADERBOARD_SIZE`)
    /// * `caller` - Optional address of the requester; required for non-Public visibility
    pub fn get_hunt_leaderboard(
        env: Env,
        hunt_id: u64,
        limit: u32,
    ) -> Result<LeaderboardResult, HuntErrorCode> {
        // Cache existence check (cheaper than loading full Hunt)
        Storage::get_hunt_cache(&env, hunt_id).ok_or(HuntErrorCode::HuntNotFound)?;
        let total_players = Storage::get_hunt_players(&env, hunt_id).len();
        let effective_limit = core::cmp::min(limit, MAX_LEADERBOARD_SIZE);
        let entries = Storage::get_leaderboard_index(&env, hunt_id);
        let mut result = Vec::new(&env);
        let result_len = core::cmp::min(effective_limit, entries.len());
        for i in 0..result_len {
            // SAFETY: i is in [0, result_len) where result_len <= entries.len()
            let entry = entries.get(i).unwrap();
            result.push_back(LeaderboardEntry {
                rank: i + 1,
                player: entry.player,
                score: entry.score,
                completed_at: entry.completed_at,
                is_completed: entry.is_completed,
            });
        }

        let truncated = entries.len() < total_players;
        Ok(LeaderboardResult {
            entries: result,
            total_players,
            truncated,
        })
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
        caller: Option<Address>,
    ) -> Result<crate::types::LeaderboardWindow, HuntErrorCode> {
        let hunt = Storage::get_hunt(&env, hunt_id).ok_or(HuntErrorCode::HuntNotFound)?;

        // Enforce visibility
        match &hunt.leaderboard_visibility {
            LeaderboardVisibility::Public => {
                // No restrictions — anyone may query.
            }
            LeaderboardVisibility::RegisteredOnly => {
                let addr = caller
                    .as_ref()
                    .ok_or(HuntErrorCode::LeaderboardVisibilityUnauthorized)?;
                addr.require_auth();
                if Storage::get_player_progress(&env, hunt_id, addr).is_none() {
                    return Err(HuntErrorCode::LeaderboardVisibilityUnauthorized);
                }
            }
            LeaderboardVisibility::CreatorOnly => {
                let addr = caller
                    .as_ref()
                    .ok_or(HuntErrorCode::LeaderboardVisibilityUnauthorized)?;
                addr.require_auth();
                if *addr != hunt.creator {
                    return Err(HuntErrorCode::LeaderboardVisibilityUnauthorized);
                }
            }
        }

        let queried_at = env.ledger().timestamp();
        let players = Storage::get_hunt_players(&env, hunt_id);
        let total_players = players.len();

        let start = core::cmp::min(start_index, total_players);
        let capped_window = core::cmp::min(window_size, MAX_LEADERBOARD_SCAN_SIZE);
        let end = core::cmp::min(start.saturating_add(capped_window), total_players);

        let mut rows = Vec::new(&env);
        for i in start..end {
            // SAFETY: start..end is clamped to [0, players.len())
            let p = players.get(i).unwrap();
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
    #[allow(dead_code)]
    fn leaderboard_best_index(
        entries: &Vec<(Address, u32, u64, bool)>,
        selected: &Vec<u32>,
    ) -> Option<u32> {
        let n = entries.len();
        let mut best_idx: Option<u32> = None;
        for i in 0..n {
            let mut taken = false;
            for j in 0..selected.len() {
                // SAFETY: j is in [0, selected.len()) — loop bound guarantees existence
                if selected.get(j).unwrap() == i {
                    taken = true;
                    break;
                }
            }
            if taken {
                continue;
            }
            // SAFETY: i is within the vector bounds established by the enclosing loop
            let (_, score, completed_at, _) = entries.get(i).unwrap();
            let better = match best_idx {
                None => true,
                Some(bi) => {
                    // SAFETY: bi was set from a previously validated index in this vec
                    let (_, b_score, b_completed_at, _) = entries.get(bi).unwrap();
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
                    match score.cmp(&b_score) {
                        core::cmp::Ordering::Greater => true,
                        core::cmp::Ordering::Equal => a_val < b_val,
                        core::cmp::Ordering::Less => false,
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
            // SAFETY: i is within the vector bounds established by the enclosing loop
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
            // SAFETY: i is within the vector bounds established by the enclosing loop
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
        let _ = Storage::get_hunt(&env, hunt_id).ok_or(HuntErrorCode::HuntNotFound)?;
        let players = Storage::get_hunt_players(&env, hunt_id);
        let total_players = players.len();
        let mut completed_count: u32 = 0;
        let mut total_score_sum: u64 = 0;
        for i in 0..players.len() {
            // SAFETY: i is within the vector bounds established by the enclosing loop
            let p = players.get(i).unwrap();
            if p.is_completed {
                completed_count = completed_count
                    .checked_add(1)
                    .ok_or(HuntErrorCode::ScoreOverflow)?;
            }
            total_score_sum = total_score_sum
                .checked_add(p.total_score as u64)
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

        let hunt = Storage::get_hunt(&env, hunt_id).ok_or(HuntErrorCode::HuntNotFound)?;

        if hunt.creator != creator {
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

        let hunt = Storage::get_hunt(&env, hunt_id).ok_or(HuntErrorCode::HuntNotFound)?;

        if hunt.creator != creator {
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

    pub fn add_co_creator(
        env: Env,
        hunt_id: u64,
        creator: Address,
        new_co_creator: Address,
    ) -> Result<(), HuntErrorCode> {
        creator.require_auth();
        let hunt = Storage::get_hunt(&env, hunt_id).ok_or(HuntErrorCode::HuntNotFound)?;
        if hunt.creator != creator {
            return Err(HuntErrorCode::Unauthorized);
        }
        Storage::add_co_creator(&env, hunt_id, &new_co_creator);
        Ok(())
    }

    pub fn remove_co_creator(
        env: Env,
        hunt_id: u64,
        creator: Address,
        co_creator_to_remove: Address,
    ) -> Result<(), HuntErrorCode> {
        creator.require_auth();
        let hunt = Storage::get_hunt(&env, hunt_id).ok_or(HuntErrorCode::HuntNotFound)?;
        if hunt.creator != creator {
            return Err(HuntErrorCode::Unauthorized);
        }
        Storage::remove_co_creator(&env, hunt_id, &co_creator_to_remove);
        Ok(())
    }

    pub fn get_co_creators(env: Env, hunt_id: u64) -> Vec<Address> {
        Storage::get_co_creators(&env, hunt_id)
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
        Self::require_admin(&env, &admin)?;

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

        let old_admin_str = old_admin
            .as_ref()
            .map(|a| a.to_string())
            .unwrap_or_else(|| String::from_str(&env, "NONE"));

        env.events().publish(
            (Symbol::new(&env, "ADMIN"), Symbol::new(&env, "ADM_TRF")),
            (old_admin_str, new_admin.to_string()),
        );

        Ok(())
    }

    pub fn add_global_view_only(
        env: Env,
        admin: Address,
        viewer: Address,
    ) -> Result<(), HuntErrorCode> {
        Self::require_admin(&env, &admin)?;

        Storage::add_global_view_only(&env, &viewer);
        Ok(())
    }

    pub fn remove_global_view_only(
        env: Env,
        admin: Address,
        viewer: Address,
    ) -> Result<(), HuntErrorCode> {
        Self::require_admin(&env, &admin)?;

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
        Self::require_admin(&env, &admin)?;

        Storage::set_pause_registrations(&env, true);
        Ok(())
    }

    pub fn unpause_registrations(env: Env, admin: Address) -> Result<(), HuntErrorCode> {
        Self::require_admin(&env, &admin)?;

        Storage::set_pause_registrations(&env, false);
        Ok(())
    }

    pub fn pause_answers(env: Env, admin: Address) -> Result<(), HuntErrorCode> {
        Self::require_admin(&env, &admin)?;

        Storage::set_pause_answers(&env, true);
        Ok(())
    }

    pub fn unpause_answers(env: Env, admin: Address) -> Result<(), HuntErrorCode> {
        Self::require_admin(&env, &admin)?;

        Storage::set_pause_answers(&env, false);
        Ok(())
    }

    pub fn pause_rewards(env: Env, admin: Address) -> Result<(), HuntErrorCode> {
        Self::require_admin(&env, &admin)?;

        Storage::set_pause_rewards(&env, true);
        Ok(())
    }

    pub fn unpause_rewards(env: Env, admin: Address) -> Result<(), HuntErrorCode> {
        Self::require_admin(&env, &admin)?;

        Storage::set_pause_rewards(&env, false);
        Ok(())
    }

    // Query pause state
    pub fn get_pause_state(env: Env) -> (bool, bool, bool) {
        (
            Storage::is_pause_registrations(&env),
            Storage::is_pause_answers(&env),
            Storage::is_pause_rewards(&env),
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
        migration::HuntyCoreMigration::run_migration(&env, &admin, target_version, dry_run)
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

    #[cfg(debug_assertions)]
    #[allow(dead_code)]
    fn sync_hunt_clue_counts(env: &Env, hunt_id: u64, hunt: &Hunt) {
        let clues = Storage::list_clues_for_hunt(env, hunt_id, 0, u32::MAX);
        let mut total = 0u32;
        let mut required = 0u32;
        for i in 0..clues.len() {
            // SAFETY: i is within the vector bounds established by the enclosing loop
            let clue = clues.get(i).unwrap();
            total += 1;
            if clue.is_required {
                required += 1;
            }
        }
        assert_eq!(
            hunt.total_clues, total,
            "total_clues drifted for hunt {hunt_id}"
        );
        assert_eq!(
            hunt.required_clues, required,
            "required_clues drifted for hunt {hunt_id}"
        );
    }
}
