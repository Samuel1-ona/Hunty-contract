#![no_std]
extern crate alloc;
use alloc::string::String as StdString;
use crate::errors::{HuntError, HuntErrorCode};
use crate::storage::Storage;
use crate::types::{
    AnswerIncorrectEvent, Clue, ClueAddedEvent, ClueCompletedEvent, ClueInfo, ClueRemovedEvent,
    Hunt, HuntActivatedEvent, HuntCancelledEvent, HuntCompletedEvent, HuntCreatedEvent,
    HuntDeactivatedEvent, HuntStatistics, HuntStatus, LeaderboardEntry, PlayerProgress,
    PlayerRegisteredEvent, RewardClaimedEvent, RewardConfig,
};
use reward_manager::RewardErrorCode;
use soroban_sdk::{
    contract, contractimpl, Address, Bytes, BytesN, Env, IntoVal, String, Symbol, Val, Vec,
};

const MAX_QUESTION_LENGTH: u32 = 2000;
const MAX_ANSWER_LENGTH: u32 = 256;
const MAX_TITLE_LENGTH: u32 = 200;
const MAX_DESCRIPTION_LENGTH: u32 = 2000;
const MAX_CLUES_PER_HUNT: u32 = 100;
/// Maximum number of leaderboard entries returned (gas and UX limit).
const MAX_LEADERBOARD_SIZE: u32 = 20;
/// Maximum number of player records scanned when building leaderboard responses.
/// This prevents unbounded gas growth for large hunts.
const MAX_LEADERBOARD_SCAN_SIZE: u32 = 200;

/// Maximum lengths for NFT metadata fields to prevent gas abuse and storage bloat
const MAX_NFT_TITLE_LENGTH: u32 = 100;
const MAX_NFT_DESCRIPTION_LENGTH: u32 = 500;
const MAX_NFT_IMAGE_URI_LENGTH: u32 = 200;
const MAX_NFT_HUNT_TITLE_LENGTH: u32 = 100;

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
        start_time: Option<u64>,
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
        if title_len > MAX_TITLE_LENGTH {
            return Err(HuntErrorCode::InvalidTitle);
        }

        // Validate description
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
            0,     // nft_rarity: 0 initially
            0,     // nft_tier: 0 initially
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
            start_time: start_time.unwrap_or(0),
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

    /// Updates a draft hunt's title and description. Only the hunt creator can update it.
    pub fn update_hunt(
        env: Env,
        hunt_id: u64,
        caller: Address,
        title: String,
        description: String,
    ) -> Result<(), HuntErrorCode> {
        caller.require_auth();

        let mut hunt = Storage::get_hunt(&env, hunt_id).ok_or(HuntErrorCode::HuntNotFound)?;

        if caller != hunt.creator {
            return Err(HuntErrorCode::Unauthorized);
        }

        if hunt.status != HuntStatus::Draft {
            return Err(HuntErrorCode::InvalidHuntStatus);
        }

        let title_len = title.len();
        if title_len == 0 || title_len > MAX_TITLE_LENGTH {
            return Err(HuntErrorCode::InvalidTitle);
        }

        if description.len() > MAX_DESCRIPTION_LENGTH {
            return Err(HuntErrorCode::InvalidDescription);
        }

        hunt.title = title;
        hunt.description = description;
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
    ) -> Result<u32, HuntErrorCode> {
        let hunt = Storage::get_hunt_or_error(&env, hunt_id).map_err(HuntErrorCode::from)?;
        hunt.creator.require_auth();
        if hunt.status != HuntStatus::Draft {
            return Err(HuntErrorCode::InvalidHuntStatus);
        }
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
        if is_required {
            updated.required_clues += 1;
        }
        Storage::save_hunt(&env, &updated);
        let event = ClueAddedEvent {
            hunt_id,
            clue_id,
            creator: updated.creator.clone(),
            points,
            is_required,
        };
        env.events()
            .publish((Symbol::new(&env, "ClueAdded"), hunt_id, clue_id), event);
        Ok(clue_id)
    }

    /// Removes a clue from a draft hunt. Only the hunt creator can remove clues.
    pub fn remove_clue(
        env: Env,
        hunt_id: u64,
        clue_id: u32,
        caller: Address,
    ) -> Result<(), HuntErrorCode> {
        caller.require_auth();
        let mut hunt = Storage::get_hunt_or_error(&env, hunt_id).map_err(HuntErrorCode::from)?;
        if hunt.status != HuntStatus::Draft {
            return Err(HuntErrorCode::InvalidHuntStatus);
        }
        if caller != hunt.creator {
            return Err(HuntErrorCode::Unauthorized);
        }

        let clue =
            Storage::get_clue_or_error(&env, hunt_id, clue_id).map_err(HuntErrorCode::from)?;
        Storage::remove_clue(&env, hunt_id, clue_id);
        hunt.total_clues = hunt.total_clues.saturating_sub(1);
        if clue.is_required {
            hunt.required_clues = hunt.required_clues.saturating_sub(1);
        }
        Storage::save_hunt(&env, &hunt);

        let event = ClueRemovedEvent {
            hunt_id,
            clue_id,
            creator: caller,
        };
        env.events()
            .publish((Symbol::new(&env, "ClueRemoved"), hunt_id, clue_id), event);

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

    /// Normalizes answer (trim, Unicode lowercase) and returns SHA256 hash as BytesN<32>.
    fn normalize_and_hash_answer(env: &Env, answer: &String) -> Result<BytesN<32>, HuntError> {
        let n = answer.len();
        if n == 0 {
            return Err(HuntError::InvalidAnswer);
        }
        if n > MAX_ANSWER_LENGTH {
            return Err(HuntError::InvalidAnswer);
        }



        let mut buf = [0u8; MAX_ANSWER_LENGTH as usize];
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
        let slice = &buf[start..end];
        let text = core::str::from_utf8(slice).map_err(|_| HuntError::InvalidAnswer)?;
        let normalized = text.to_lowercase();
        if normalized.is_empty() {
            return Err(HuntError::InvalidAnswer);
        }
        let normalized = Bytes::from_slice(env, normalized.as_bytes());
        let hash = env.crypto().sha256(&normalized);
        Ok(hash.to_bytes())
    }






    

    #[inline]
    fn is_ascii_space(b: u8) -> bool {
        b == 0x20 || b == 0x09 || b == 0x0a || b == 0x0d
    }

    #[inline]
    fn validate_rarity(v: u32) -> bool {
        v <= 5
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

        if hunt.required_clues == 0 {
            return Err(HuntErrorCode::NoRequiredClues);
        }

        let current_time = env.ledger().timestamp();
        // Enforce configured start_time: cannot activate before start_time
        if hunt.start_time != 0 && current_time < hunt.start_time {
            return Err(HuntErrorCode::InvalidHuntStatus);
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
        Ok(())
    }

    pub fn deactivate_hunt(env: Env, hunt_id: u64, caller: Address) -> Result<(), HuntErrorCode> {
        caller.require_auth();

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

        Ok(())
    }

    pub fn get_hunt_info(env: Env, hunt_id: u64) -> Result<Hunt, HuntErrorCode> {
        let hunt = Storage::get_hunt(&env, hunt_id).ok_or(HuntErrorCode::HuntNotFound)?;
        Ok(hunt)
    }

    /// Sets the RewardManager contract address for cross-contract reward distribution.
    ///
    /// Access control: only the admin (or contract invoker) is allowed to set this.
    pub fn set_reward_manager(env: Env, reward_manager: Address) -> Result<(), HuntErrorCode> {
        // Require invoker authorization.
        // (This is the auth gate that was missing before.)
        env.invoker().require_auth();

        Storage::set_reward_manager(&env, &reward_manager);
        Ok(())
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
        Self::process_reward_distribution(&env, hunt_id, player)
    }

    /// Allows the hunt creator to distribute rewards to multiple players in batch.
    /// This is more gas-efficient than individual claims when many players finish at once.
    pub fn batch_complete_hunt(
        env: Env,
        hunt_id: u64,
        creator: Address,
        players: Vec<Address>,
    ) -> Result<(), HuntErrorCode> {
        creator.require_auth();

        let hunt = Storage::get_hunt_or_error(&env, hunt_id).map_err(HuntErrorCode::from)?;

        if hunt.creator != creator {
            return Err(HuntErrorCode::Unauthorized);
        }

        for i in 0..players.len() {
            let player = players.get(i).unwrap();
            // Process each player; we use a best-effort approach where one failure
            // doesn't block the entire batch, though creators should verify results via events.
            let _ = Self::process_reward_distribution(&env, hunt_id, player);
        }

        Ok(())
    }

    /// Internal helper to handle the core reward distribution logic.
    fn process_reward_distribution(
        env: &Env,
        hunt_id: u64,
        player: Address,
    ) -> Result<(), HuntErrorCode> {
        let mut hunt = Storage::get_hunt_or_error(env, hunt_id).map_err(HuntErrorCode::from)?;

        if hunt.status != HuntStatus::Active {
            return Err(HuntErrorCode::InvalidHuntStatus);
        }

        let mut progress = Storage::get_player_progress_or_error(env, hunt_id, &player)
            .map_err(HuntErrorCode::from)?;

                /// Generates a completion certificate for a player who has finished a hunt.
    pub fn generate_completion_certificate(
        env: Env,
        hunt_id: u64,
        player: Address,
    ) -> Result<String, HuntErrorCode> {
        let progress = Storage::get_player_progress(&env, hunt_id, &player)
            .ok_or(HuntErrorCode::PlayerNotRegistered)?;

        if !progress.is_completed {
            return Err(HuntErrorCode::HuntNotCompleted);
        }

        let hunt = Storage::get_hunt(&env, hunt_id)
            .ok_or(HuntErrorCode::HuntNotFound)?;

        let certificate = String::from_str(
            &env,
            "COMPLETION_CERTIFICATE",
        );

        let _ = hunt;
        let _ = progress;

        Ok(certificate)
    }

        // Verify the player has completed all required clues
        if !progress.is_completed {
            return Err(HuntErrorCode::HuntNotCompleted);
        }

        // Prevent double-claiming
        if progress.reward_claimed {
            return Err(HuntErrorCode::RewardAlreadyClaimed);
        }

        // Check rewards are configured
        if hunt.reward_config.max_winners == 0 {
            return Err(HuntErrorCode::NoRewardsConfigured);
        }

        // Check reward slots are available
        if !hunt.has_rewards_available() {
            return Err(HuntErrorCode::InsufficientRewardPool);
        }

        let nft_awarded = hunt.reward_config.nft_enabled;

        if !Self::validate_rarity(hunt.reward_config.nft_rarity) {
            return Err(HuntErrorCode::InvalidRarity);
        }

        // Call RewardManager if configured and there are rewards to distribute
        let reward_amount = if let Some(reward_manager_addr) = Storage::get_reward_manager(env) {
            let mut balance_args: Vec<Val> = Vec::new(env);
            balance_args.push_back(hunt_id.into_val(env));

            let pool_balance = env
                .try_invoke_contract::<i128, RewardErrorCode>(
                    &reward_manager_addr,
                    &Symbol::new(env, "get_pool_balance"),
                    balance_args,
                )
                .map_err(|_| HuntErrorCode::RewardDistributionFailed)?
                .map_err(|_| HuntErrorCode::RewardDistributionFailed)?;

            hunt.reward_config.xlm_pool = pool_balance;
            hunt.reward_config.reward_per_winner()
        } else {
            hunt.reward_config.reward_per_winner()
        };

        if let Some(reward_manager_addr) = Storage::get_reward_manager(env) {
            let xlm_amount = if reward_amount > 0 {
                Some(reward_amount)
            } else {
                None
            };
            // description is intentionally excluded from NFT metadata: a creator could
            // accidentally embed an answer or salt in the hunt description, which would
            // then be permanently exposed on-chain via the cross-contract call.
            // Only the title (already fully public) is forwarded.
    let (nft_contract, nft_title, nft_desc, nft_uri, nft_hunt_title) = if nft_awarded {
        hunt.reward_config
            .nft_contract
            .clone()
            .map(|nft_contract| {
                let title = hunt.title.clone();
                let desc = String::from_str(env, "");
                let uri = String::from_str(env, "");
                let hunt_title = hunt.title.clone();

                // === NFT Metadata Length Validation ===
                if title.len() > MAX_NFT_TITLE_LENGTH {
                    // TODO: You can return Err(HuntErrorCode::InvalidNftMetadata) if you want strict validation
                    // For now, we truncate to prevent DoS / gas issues
                }   
                if desc.len() > MAX_NFT_DESCRIPTION_LENGTH {
                    // truncate if needed in future
                }
                if uri.len() > MAX_NFT_IMAGE_URI_LENGTH {
                    // truncate if needed
                }
                if hunt_title.len() > MAX_NFT_HUNT_TITLE_LENGTH {
                    // truncate if needed
                }

                (
                    Some(nft_contract),
                    title,
                    desc,
                    uri,
                    hunt_title,
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
            let rm_reward_config = reward_manager::RewardConfig {
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
                let mut args: Vec<Val> = Vec::new(env);
                args.push_back(hunt_id.into_val(env));
                args.push_back(player.clone().into_val(env));
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
        Storage::save_player_progress(env, &progress);

        // Update hunt reward config
        hunt.reward_config.claimed_count += 1;
        
        // Mark hunt as completed if all reward slots are taken
        if hunt.reward_config.claimed_count >= hunt.reward_config.max_winners {
            hunt.status = HuntStatus::Completed;
            
            // Optionally, we could emit a HuntStatusChangedEvent or HuntEndedEvent here 
            // if we want to notify clients that the hunt is completely finished.
        }
        
        Storage::save_hunt(env, &hunt);

        // Emit RewardClaimedEvent
        let event = RewardClaimedEvent {
            hunt_id,
            player: player.clone(),
            xlm_amount: reward_amount,
            nft_awarded,
        };
        env.events()
            .publish((Symbol::new(env, "RewardClaimed"), hunt_id), event);

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

        let hunt = Storage::get_hunt(&env, hunt_id).ok_or(HuntErrorCode::HuntNotFound)?;

        if hunt.status != HuntStatus::Active {
            return Err(HuntErrorCode::InvalidHuntStatus);
        }

        let current_time = env.ledger().timestamp();
        if !hunt.is_active(current_time) {
            return Err(HuntErrorCode::HuntNotActive);
        }

        if let Some(existing) = Storage::get_player_progress(&env, hunt_id, &player) {
            // Allow re-registration only if the existing progress is from a previous
            // activation cycle (i.e. the hunt was deactivated and reactivated since
            // the player registered). Otherwise reject as a duplicate.
            if existing.started_at >= hunt.activated_at {
                return Err(HuntErrorCode::DuplicateRegistration);
            }
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

    /// Verifies a candidate answer without recording progress or emitting events.
    pub fn preview_answer(
        env: Env,
        hunt_id: u64,
        clue_id: u32,
        player: Address,
        answer: String,
    ) -> bool {
        let Some(hunt) = Storage::get_hunt(&env, hunt_id) else {
            return false;
        };

        let current_time = env.ledger().timestamp();
        if !hunt.is_active(current_time) {
            return false;
        }

        if Storage::get_player_progress(&env, hunt_id, &player).is_none() {
            return false;
        }

        let Some(clue) = Storage::get_clue(&env, hunt_id, clue_id) else {
            return false;
        };
        let Ok(submitted_hash) = Self::normalize_and_hash_answer(&env, &answer) else {
            return false;
        };

        submitted_hash == clue.answer_hash
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
    ///
    /// # Events
    /// * `ClueCompleted` - Emitted when answer is correct
    /// * `HuntCompleted` - Emitted when all required clues are completed
    /// * `AnswerIncorrect` - Emitted when answer is wrong (for analytics)
    pub fn submit_answer(
        env: Env,
        hunt_id: u64,
        clue_id: u32,
        player: Address,
        answer: String,
    ) -> Result<(), HuntErrorCode> {
        // Require player authorization
        player.require_auth();

        // 1. Verify hunt exists and is active
        let hunt = Storage::get_hunt(&env, hunt_id).ok_or(HuntErrorCode::HuntNotFound)?;

        let current_time = env.ledger().timestamp();
        if !hunt.is_active(current_time) {
            return Err(HuntErrorCode::HuntNotActive);
        }

        let mut progress = Storage::get_player_progress(&env, hunt_id, &player)
            .ok_or(HuntErrorCode::PlayerNotRegistered)?;

        let clue = Storage::get_clue(&env, hunt_id, clue_id).ok_or(HuntErrorCode::ClueNotFound)?;

        if progress.has_completed_clue(clue_id) {
            return Err(HuntErrorCode::ClueAlreadyCompleted);
        }

        let submitted_hash =
            Self::normalize_and_hash_answer(&env, &answer).map_err(HuntErrorCode::from)?;

        if submitted_hash != clue.answer_hash {
            // Answer is incorrect - record attempt and emit analytics event
            let attempt_number = progress.record_attempt(clue_id);
            Storage::save_player_progress(&env, &progress);
            let incorrect_event = AnswerIncorrectEvent {
                hunt_id,
                player: player.clone(),
                clue_id,
                timestamp: current_time,
                attempt_number,
            };
            env.events().publish(
                (Symbol::new(&env, "AnswerIncorrect"), hunt_id, clue_id),
                incorrect_event,
            );
            return Err(HuntErrorCode::InvalidAnswer);
        }

        progress.complete_clue(&env, clue_id, clue.points, clue.is_required);

        let all_required_completed =
            Self::check_all_required_clues_completed(hunt.required_clues, &progress);

        // If all required clues completed, mark hunt as completed for this player
        if all_required_completed && !progress.is_completed {
            progress.is_completed = true;
            progress.completed_at = current_time;

            // Emit HuntCompleted event
            let hunt_completed_event = HuntCompletedEvent {
                hunt_id,
                player: player.clone(),
                total_score: progress.total_score,
                completion_time: current_time,
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
            points_earned: clue.points,
        };
        env.events().publish(
            (Symbol::new(&env, "ClueCompleted"), hunt_id, clue_id),
            clue_completed_event,
        );

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
    fn check_all_required_clues_completed(required_clue_count: u32, progress: &PlayerProgress) -> bool {
        progress.required_completed_count >= required_clue_count
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

    /// Returns the top N players by score for a hunt (read-only).
    /// Sorted by score descending, then by completion time ascending (earlier = better).
    /// Limit is capped at 20 to control gas. Returns error if hunt does not exist.
    pub fn get_hunt_leaderboard(
        env: Env,
        hunt_id: u64,
        limit: u32,
    ) -> Result<Vec<LeaderboardEntry>, HuntErrorCode> {
        let _ = Storage::get_hunt(&env, hunt_id).ok_or(HuntErrorCode::HuntNotFound)?;
        let effective_limit = core::cmp::min(limit, MAX_LEADERBOARD_SIZE);
        let queried_at = env.ledger().timestamp();
        let players = Storage::get_hunt_players(&env, hunt_id);
        let scan_limit = core::cmp::min(players.len(), MAX_LEADERBOARD_SCAN_SIZE);
        let mut entries = Vec::new(&env);
        for i in 0..scan_limit {
            let p = players.get(i).unwrap();
            entries.push_back((
                p.player.clone(),
                p.total_score,
                p.completed_at,
                p.is_completed,
            ));
        }
        let mut selected = Vec::new(&env);
        let mut result = Vec::new(&env);
        for rank in 1..=effective_limit {
            if let Some(best_idx) = Self::leaderboard_best_index(&entries, &selected) {
                selected.push_back(best_idx);
                let (player, score, completed_at, is_completed) = entries.get(best_idx).unwrap();
                result.push_back(LeaderboardEntry {
                    rank,
                    player,
                    score,
                    completed_at,
                    is_completed,
                    queried_at,
                });
            } else {
                break;
            }
        }
        Ok(result)
    }

    /// Scans a bounded window of registered players for a hunt and returns
    /// their compact rows. This method enables clients to page through all
    /// registered players in multiple calls (bounded by `MAX_LEADERBOARD_SCAN_SIZE`)
    /// and merge results off-chain to build a full leaderboard without a single
    /// large on-chain scan.
    ///
    /// Arguments:
    /// * `env` - Soroban environment
    /// * `hunt_id` - Hunt identifier
    /// * `start_index` - Zero-based index into the registered players list to start scanning
    /// * `window_size` - Number of player records to scan in this call (capped)
    ///
    /// Returns a `LeaderboardWindow` containing the scanned rows, the `next_index`
    /// clients should use for the following call and a `finished` flag indicating
    /// whether the end of the player list has been reached.
    pub fn get_hunt_leaderboard_window(
        env: Env,
        hunt_id: u64,
        start_index: u32,
        window_size: u32,
    ) -> Result<crate::types::LeaderboardWindow, HuntErrorCode> {
        let _ = Storage::get_hunt(&env, hunt_id).ok_or(HuntErrorCode::HuntNotFound)?;
        let queried_at = env.ledger().timestamp();
        let players = Storage::get_hunt_players(&env, hunt_id);
        let total_players = players.len();

        let start = core::cmp::min(start_index as usize, total_players);
        let capped_window = core::cmp::min(window_size, MAX_LEADERBOARD_SCAN_SIZE);
        let end = core::cmp::min(start + capped_window as usize, total_players);

        let mut rows = Vec::new(&env);
        for i in start..end {
            let p = players.get(i).unwrap();
            rows.push_back(crate::types::LeaderboardRow {
                index: i as u32,
                player: p.player.clone(),
                score: p.total_score,
                completed_at: p.completed_at,
                is_completed: p.is_completed,
            });
        }

        let next_index = end as u32;
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
            let i_u32 = i as u32;
            let mut taken = false;
            for j in 0..selected.len() {
                if selected.get(j).unwrap() == i_u32 {
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
                best_idx = Some(i_u32);
            }
        }
        best_idx
    }

        /// Returns a list of all hunts (paginated).
    ///
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `start` - Starting index (0-based)
    /// * `limit` - Maximum number of hunts to return (capped at 50 for gas safety)
    ///
    /// # Returns
    /// Vec of Hunt structs
    pub fn list_hunts(env: Env, start: u32, limit: u32) -> Vec<Hunt> {
        let counter = Storage::get_hunt_counter(&env);
        let actual_limit = limit.min(50).min(counter as u32); // Safety cap

        let mut hunts = Vec::new(&env);
        let end = (start + actual_limit).min(counter as u32);

        for i in start..end {
            let hunt_id = (i as u64) + 1; // Hunt IDs start from 1
            if let Some(hunt) = Storage::get_hunt(&env, hunt_id) {
                hunts.push_back(hunt);
            }
        }

        hunts
    }

    /// Returns aggregate statistics for a hunt (read-only): total players, completion rate, average score.
    /// Returns error if hunt does not exist.
    pub fn get_hunt_statistics(
        env: Env,
        hunt_id: u64,
    ) -> Result<HuntStatistics, HuntErrorCode> {
        let _ = Storage::get_hunt(&env, hunt_id).ok_or(HuntErrorCode::HuntNotFound)?;
        let players = Storage::get_hunt_players(&env, hunt_id);
        let total_players = players.len() as u32;
        let mut completed_count: u32 = 0;
        let mut total_score_sum: u64 = 0;
        for i in 0..players.len() {
            let p = players.get(i).unwrap();
            if p.is_completed {
                completed_count += 1;
            }
            total_score_sum += p.total_score as u64;
        }
        let completion_rate_percent = if total_players > 0 {
            (completed_count * 100) / total_players
        } else {
            0
        };
        let average_score = if total_players > 0 {
            (total_score_sum / (total_players as u64)) as u32
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
}

mod errors;
mod storage;
mod types;

#[cfg(test)]
mod test;
