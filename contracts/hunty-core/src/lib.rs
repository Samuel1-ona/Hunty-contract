#![no_std]
use crate::errors::{HuntError, HuntErrorCode};
use crate::storage::Storage;
use crate::types::{
    Clue, ClueAddedEvent, ClueInfo, Hunt, HuntActivatedEvent, HuntCancelledEvent, HuntCreatedEvent,
    HuntDeactivatedEvent, HuntStatus, PlayerProgress, PlayerRegisteredEvent, RewardConfig,
};
use soroban_sdk::{contract, contractimpl, Address, Bytes, BytesN, Env, String, Symbol, Vec};

const MAX_QUESTION_LENGTH: u32 = 2000;
const MAX_ANSWER_LENGTH: u32 = 256;
const MAX_CLUES_PER_HUNT: u32 = 100;

#[contract]
pub struct HuntyCore;

#[contractimpl]
impl HuntyCore {
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
    ) -> Result<u64, HuntErrorCode> {
        // Validate creator address - in Soroban, Address is always valid if constructed,
        // but we ensure it's not a zero/null address pattern if needed
        // For now, we accept any valid Address type

        // Validate title
        let title_len = title.len();
        if title_len == 0 {
            return Err(HuntErrorCode::InvalidTitle);
        }
        const MAX_TITLE_LENGTH: u32 = 200;
        if title_len > MAX_TITLE_LENGTH {
            return Err(HuntErrorCode::InvalidTitle);
        }

        // Validate description
        const MAX_DESCRIPTION_LENGTH: u32 = 2000;
        if description.len() > MAX_DESCRIPTION_LENGTH {
            return Err(HuntErrorCode::InvalidDescription);
        }

        // Get current timestamp
        let current_time = env.ledger().timestamp();

        // Generate unique hunt ID
        let hunt_id = Storage::next_hunt_id(&env);

        // Initialize reward config with zero pool
        let reward_config = RewardConfig::new(
            0,     // xlm_pool: zero initially
            false, // nft_enabled: false initially
            None,  // nft_contract: None initially
            0,     // max_winners: 0 initially
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
            total_clues: 0, // Empty clue list initially
            required_clues: 0,
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
    ) -> Result<u32, HuntErrorCode> {
        let hunt = Storage::get_hunt_or_error(&env, hunt_id).map_err(HuntErrorCode::from)?;
        if hunt.status != HuntStatus::Draft {
            return Err(HuntErrorCode::InvalidHuntStatus);
        }
        hunt.creator.require_auth();
        if Storage::get_clue_counter(&env, hunt_id) >= MAX_CLUES_PER_HUNT {
            return Err(HuntErrorCode::from(HuntError::TooManyClues {
                hunt_id,
                limit: MAX_CLUES_PER_HUNT,
            }));
        }
        let qlen = question.len();
        if qlen == 0 || qlen > MAX_QUESTION_LENGTH {
            return Err(HuntErrorCode::InvalidQuestion);
        }
        let answer_hash =
            Self::normalize_and_hash_answer(&env, &answer).map_err(HuntErrorCode::from)?;
        let clue_id = Storage::next_clue_id(&env, hunt_id);
        let clue = Clue {
            clue_id,
            question: question.clone(),
            answer_hash,
            points,
            is_required,
        };
        Storage::save_clue(&env, hunt_id, &clue);
        let mut updated = hunt;
        updated.total_clues += 1;
        Storage::save_hunt(&env, &updated);
        let event = ClueAddedEvent {
            hunt_id,
            clue_id,
            creator: updated.creator.clone(),
            question,
            points,
            is_required,
        };
        env.events()
            .publish((Symbol::new(&env, "ClueAdded"), hunt_id, clue_id), event);
        Ok(clue_id)
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
        })
    }

    /// Returns all clues for a hunt (question, points, required). Answer hashes are not exposed.
    pub fn list_clues(env: Env, hunt_id: u64) -> Vec<ClueInfo> {
        let raw = Storage::list_clues_for_hunt(&env, hunt_id);
        let mut out = Vec::new(&env);
        for i in 0..raw.len() {
            let c = raw.get(i).unwrap();
            out.push_back(ClueInfo {
                clue_id: c.clue_id,
                question: c.question,
                points: c.points,
                is_required: c.is_required,
            });
        }
        out
    }

    /// Normalizes answer (trim, lowercase) and returns SHA256 hash as BytesN<32>.
    fn normalize_and_hash_answer(env: &Env, answer: &String) -> Result<BytesN<32>, HuntError> {
        let n = answer.len();
        if n == 0 {
            return Err(HuntError::InvalidAnswer);
        }
        if n > MAX_ANSWER_LENGTH {
            return Err(HuntError::InvalidAnswer);
        }
        let mut buf = [0u8; 256];
        answer.copy_into_slice(&mut buf[..n as usize]);
        let mut start = 0usize;
        let mut end = n as usize;
        while start < end && Self::is_ascii_space(buf[start]) {
            start += 1;
        }
        while end > start && Self::is_ascii_space(buf[end - 1]) {
            end -= 1;
        }
        if start >= end {
            return Err(HuntError::InvalidAnswer);
        }
        for i in start..end {
            let b = buf[i];
            if b >= b'A' && b <= b'Z' {
                buf[i] = b + (b'a' - b'A');
            }
        }
        let normalized = Bytes::from_slice(env, &buf[start..end]);
        let hash = env.crypto().sha256(&normalized);
        Ok(hash.to_bytes())
    }

    #[inline]
    fn is_ascii_space(b: u8) -> bool {
        b == 0x20 || b == 0x09 || b == 0x0a || b == 0x0d
    }

    pub fn activate_hunt(env: Env, hunt_id: u64, caller: Address) -> Result<(), HuntErrorCode> {
        let mut hunt = Storage::get_hunt(&env, hunt_id).ok_or(HuntErrorCode::HuntNotFound)?;

        // Verify caller is the creator

        if caller != hunt.creator {
            return Err(HuntErrorCode::Unauthorized);
        }

        if hunt.status != HuntStatus::Draft {
            return Err(HuntErrorCode::InvalidHuntStatus);
        }

        if hunt.total_clues == 0 {
            return Err(HuntErrorCode::NoCluesAdded);
        }

        let current_time = env.ledger().timestamp();
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
        Ok(())
    }

    pub fn deactivate_hunt(env: Env, hunt_id: u64, caller: Address) -> Result<(), HuntErrorCode> {
        // Load hunt
        let mut hunt = Storage::get_hunt(&env, hunt_id).ok_or(HuntErrorCode::HuntNotFound)?;

        // Verify caller is creator
        if caller != hunt.creator {
            return Err(HuntErrorCode::Unauthorized);
        }

        // Check hunt is Active
        if hunt.status != HuntStatus::Active {
            return Err(HuntErrorCode::InvalidHuntStatus);
        }

        hunt.status = HuntStatus::Draft;

        Storage::save_hunt(&env, &hunt);

        let event = HuntDeactivatedEvent { hunt_id };

        env.events()
            .publish((Symbol::new(&env, "HuntDeactivated"), hunt_id), event);

        Ok(())
    }

    pub fn cancel_hunt(env: Env, hunt_id: u64, caller: Address) -> Result<(), HuntErrorCode> {
        // Load hunt
        let mut hunt = Storage::get_hunt(&env, hunt_id).ok_or(HuntErrorCode::HuntNotFound)?;

        // Verify caller is creator
        if caller != hunt.creator {
            return Err(HuntErrorCode::Unauthorized);
        }

        // Cannot cancel a completed hunt
        if hunt.status == HuntStatus::Completed {
            return Err(HuntErrorCode::InvalidHuntStatus);
        }

        // If already cancelled, treat as invalid
        if hunt.status == HuntStatus::Cancelled {
            return Err(HuntErrorCode::InvalidHuntStatus);
        }

        // Handle refunds if reward pool was funded
        // TODO - HANDLE REFUND

        // Cancel hunt
        hunt.status = HuntStatus::Cancelled;

        // Persist
        Storage::save_hunt(&env, &hunt);

        // Emit event
        let event = HuntCancelledEvent { hunt_id };

        env.events()
            .publish((Symbol::new(&env, "HuntCancelled"), hunt_id), event);

        Ok(())
    }

    pub fn get_hunt_info(env: Env, hunt_id: u64) -> Result<Hunt, HuntErrorCode> {
        let hunt = Storage::get_hunt(&env, hunt_id).ok_or(HuntErrorCode::HuntNotFound)?;

        match hunt.status {
            HuntStatus::Draft
            | HuntStatus::Active
            | HuntStatus::Completed
            | HuntStatus::Cancelled => {}
        }

        // Return the full Hunt struct
        Ok(hunt)
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

        let hunt = Storage::get_hunt(&env, hunt_id).ok_or(HuntErrorCode::HuntNotFound)?;

        if hunt.status != HuntStatus::Active {
            return Err(HuntErrorCode::InvalidHuntStatus);
        }

        let current_time = env.ledger().timestamp();
        if !hunt.is_active(current_time) {
            return Err(HuntErrorCode::HuntNotActive);
        }

        if Storage::get_player_progress(&env, hunt_id, &player).is_some() {
            return Err(HuntErrorCode::DuplicateRegistration);
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

    /// Returns player progress for a hunt, or None if not registered.
    pub fn get_player_progress(
        env: Env,
        hunt_id: u64,
        player: Address,
    ) -> Result<PlayerProgress, HuntErrorCode> {
        Storage::get_player_progress(&env, hunt_id, &player)
            .ok_or(HuntErrorCode::PlayerNotRegistered)
    }
}

mod errors;
mod storage;
mod types;

#[cfg(test)]
mod test;
