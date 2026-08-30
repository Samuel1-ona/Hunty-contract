use soroban_sdk::{contracttype, Address, BytesN, Env, Map, String, Vec};

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HuntStatus {
    Draft,
    Active,
    Completed,
    Cancelled,
    Paused,
    EmergencyStopped,
    Archived,
}

/// Controls who can view the leaderboard for a hunt.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LeaderboardVisibility {
    /// Anyone can view the leaderboard (default).
    Public,
    /// Only players who have registered for the hunt can view the leaderboard.
    RegisteredOnly,
    /// Only the hunt creator can view the leaderboard.
    CreatorOnly,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RewardConfig {
    pub xlm_pool: i128,
    pub nft_enabled: bool,
    pub nft_contract: Option<Address>,
    pub max_winners: u32,
    pub claimed_count: u32,
    pub nft_rarity: u32,
    pub nft_tier: u32,
    pub nft_image_uri: Option<String>,
}

pub type HuntRewardConfig = RewardConfig;

#[contracttype]
#[derive(Clone, Debug)]
pub struct Hunt {
    pub hunt_id: u64,
    pub creator: Address,
    pub title: String,
    pub description: String,
    pub categories: Vec<String>,
    pub difficulty_rating: u32,
    pub difficulty_override: Option<u32>,
    pub status: HuntStatus,
    pub created_at: u64,
    pub activated_at: u64,
    pub start_time: u64,
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
    /// Registration cutoff timestamp. 0 = no deadline (registration open while active).
    pub registration_deadline: u64,
    /// When true, players may claim their partial score after the hunt ends.
    pub allow_partial_scoring: bool,
    /// When true, players may form teams and share clue progress.
    pub team_mode: bool,
    /// Default point value applied to clues with 0 points. Clue-level points override this.
    pub default_points: u32,
    /// Minimum seconds a player must wait between attempts on the same clue.
    pub attempt_cooldown_secs: u32,
    /// Maximum number of players allowed to register. 0 = unlimited.
    pub max_players: u32,
    /// When true, only players with a valid invite code may register.
    pub is_private: bool,
    /// SHA256 hash (salted with hunt_id) of the invite code, if configured.
    pub invite_code_hash: Option<BytesN<32>>,
    /// Dynamically recalculated on every `get_hunt` read; not meaningful when read from a raw struct literal.
    pub remaining_slots: u32,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HuntCache {
    pub hunt_id: u64,
    pub creator: Address,
    pub status: HuntStatus,
    pub start_time: u64,
    pub end_time: u64,
    pub total_clues: u32,
    pub required_clues: u32,
    pub max_winners: u32,
}

impl HuntCache {
    pub fn from_hunt(hunt: &Hunt) -> Self {
        Self {
            hunt_id: hunt.hunt_id,
            creator: hunt.creator.clone(),
            status: hunt.status.clone(),
            start_time: hunt.start_time,
            end_time: hunt.end_time,
            total_clues: hunt.total_clues,
            required_clues: hunt.required_clues,
            max_winners: hunt.reward_config.max_winners,
        }
    }
}

/// Stored clue with SHA256 answer hash. The hash is never exposed via get_clue/list_clues or events.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Clue {
    pub clue_id: u32,
    pub question: String,
    pub answer_hashes: Vec<BytesN<32>>,
    pub points: u32,
    pub is_required: bool,
    pub difficulty: u32,
    pub weight: u32,
    pub hint: Option<String>,
    pub hint_penalty_points: u32,
}

/// Input payload for adding multiple clues in one contract invocation.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BatchClueInput {
    pub question: String,
    pub answer: String,
    pub points: u32,
    pub is_required: bool,
    /// Difficulty tier (1-5, 1 = easiest, 5 = hardest).
    /// Difficulty multiplies the clue's points: points earned = points * difficulty.
    pub difficulty: u32,
}

/// Clue info returned by get_clue/list_clues. Excludes answer hash.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClueInfo {
    pub clue_id: u32,
    pub question: String,
    pub points: u32,
    pub is_required: bool,
    pub difficulty: u32,
    pub weight: u32,
    pub hint_available: bool,
    pub hint_penalty_points: u32,
}

#[contracttype]
#[derive(Clone)]
pub struct HuntCancelledEvent {
    pub hunt_id: u64,
}

/// Emitted when a creator force-closes a hunt early (marks it Completed) while
/// preserving player scores and any already-distributed rewards. `rewarded_players`
/// is the number of completed players who received a final reward as part of closing.
#[contracttype]
#[derive(Clone)]
pub struct HuntClosedEvent {
    pub hunt_id: u64,
    pub closed_at: u64,
    pub rewarded_players: u32,
}

#[contracttype]
#[derive(Clone)]
pub struct HuntDeactivatedEvent {
    pub hunt_id: u64,
}

#[contracttype]
#[derive(Clone)]
pub struct HuntActivatedEvent {
    pub hunt_id: u64,
    pub activated_at: u64,
}

#[contracttype]
#[derive(Clone)]
pub struct HuntReactivatedEvent {
    pub hunt_id: u64,
    pub activated_at: u64,
}

#[contracttype]
#[derive(Clone)]
pub struct HuntArchivedEvent {
    pub hunt_id: u64,
}

/// Result of a `gc_hunt` sweep (issue #446).
///
/// Counts are split by storage tier because the two are charged and expire
/// differently on Soroban: instance entries share the contract's own TTL, while
/// persistent entries each carry their own. An operator reclaiming space needs
/// to see which tier actually shrank.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct GcReport {
    pub hunt_id: u64,
    /// Entries removed from persistent storage.
    pub persistent_removed: u32,
    /// Entries removed from instance storage.
    pub instance_removed: u32,
    /// `persistent_removed + instance_removed`.
    pub total_removed: u32,
    /// Players whose per-hunt entries were swept.
    pub players_swept: u32,
    /// Clues whose per-hunt entries were swept.
    pub clues_swept: u32,
    /// Teams whose per-hunt entries were swept.
    pub teams_swept: u32,
}

/// Emitted once a cancelled or archived hunt's storage has been reclaimed.
#[contracttype]
#[derive(Clone)]
pub struct HuntGarbageCollectedEvent {
    pub hunt_id: u64,
    pub total_removed: u32,
    pub collected_at: u64,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Location {
    pub latitude: i64,  // Degrees * 1_000_000
    pub longitude: i64, // Degrees * 1_000_000
    pub radius: u32,
}

/// Internal compact storage representation of player progress.
/// Does not store `player` or `hunt_id` — those are already the storage key.
///
/// ## Compact encoding
/// - Timestamps are delta-encoded as `u32` offsets from the hunt's `activated_at`,
///   saving 4 bytes each vs full `u64` UNIX timestamps. The max delta (~136 years)
///   far exceeds any realistic hunt duration.
/// - Boolean fields (`is_completed`, `reward_claimed`) are packed into `flags`.
#[contracttype]
#[derive(Clone, Debug)]
pub struct StoredPlayerProgress {
    pub completed_clues: Vec<u32>,
    pub hinted_clues: Vec<u32>,
    pub total_score: u32,

    /// Seconds elapsed from hunt `activated_at` to player registration.
    /// Reconstruct absolute: `activated_at + started_at_delta`.
    pub started_at_delta: u32,

    /// Seconds elapsed from player registration to hunt completion, or 0 if not completed.
    /// Reconstruct absolute: `activated_at + started_at_delta + completed_at_delta`.
    pub completed_at_delta: u32,

    /// Bit flags for boolean fields to reduce storage footprint.
    /// BIT0 (1): is_completed
    /// BIT1 (2): reward_claimed
    /// BIT2–BIT31: reserved for future use
    pub flags: u32,
    pub recent_submissions: Vec<u64>,
    pub clue_last_attempts: Map<u32, u64>,
    pub required_completed_count: u32,
    /// The player's finishing position among all completions for this hunt,
    /// frozen at the moment `is_completed` was set to `true`. 0 = not yet completed.
    pub completion_rank: u32,
}

/// Public view of player progress, with `player` and `hunt_id` reconstructed from the key.
#[contracttype]
#[derive(Clone, Debug)]
pub struct PlayerProgress {
    pub player: Address,
    pub hunt_id: u64,
    pub completed_clues: Vec<u32>,
    pub completed_clue_index: Map<u32, bool>,
    pub hinted_clues: Vec<u32>,
    pub total_score: u32,
    pub required_completed_count: u32,
    pub started_at: u64,
    pub completed_at: u64,
    pub is_completed: bool,
    pub reward_claimed: bool,
    /// The player's finishing position among all completions for this hunt,
    /// frozen at the moment `is_completed` was set to `true`.  Zero means the
    /// player has not yet completed the hunt.
    pub completion_rank: u32,
    pub recent_submissions: Vec<u64>,
    pub clue_last_attempts: Map<u32, u64>,
}

impl PlayerProgress {
    pub fn new(env: &Env, player: Address, hunt_id: u64, current_time: u64) -> Self {
        Self {
            player,
            hunt_id,
            completed_clues: Vec::new(env),
            completed_clue_index: Map::new(env),
            hinted_clues: Vec::new(env),
            total_score: 0,
            required_completed_count: 0,
            started_at: current_time,
            completed_at: 0,
            is_completed: false,
            reward_claimed: false,
            completion_rank: 0,
            recent_submissions: Vec::new(env),
            clue_last_attempts: Map::new(env),
        }
    }

    /// Pack boolean flags into a single u32
    #[allow(dead_code)]
    fn bools_to_flags(is_completed: bool, reward_claimed: bool) -> u32 {
        let mut flags = 0u32;
        if is_completed {
            flags |= 0x01;
        }
        if reward_claimed {
            flags |= 0x02;
        }
        flags
    }

    /// Convert to the compact form stored on-chain (drops redundant key fields).
    ///
    /// `activated_at` is the hunt's activation timestamp, used to delta-encode
    /// `started_at` and `completed_at` into compact `u32` offsets.
    pub fn to_stored(&self, activated_at: u64) -> StoredPlayerProgress {
        let flags = Self::bools_to_flags(self.is_completed, self.reward_claimed);

        // Delta-encode timestamps relative to hunt activation.
        let started_at_delta = self.started_at.saturating_sub(activated_at) as u32;
        let completed_at_delta = if self.completed_at == 0 {
            0u32
        } else {
            self.completed_at.saturating_sub(self.started_at) as u32
        };

        StoredPlayerProgress {
            completed_clues: self.completed_clues.clone(),
            hinted_clues: self.hinted_clues.clone(),
            total_score: self.total_score,
            started_at_delta,
            completed_at_delta,
            flags,
            recent_submissions: self.recent_submissions.clone(),
            clue_last_attempts: self.clue_last_attempts.clone(),
            required_completed_count: self.required_completed_count,
            completion_rank: self.completion_rank,
        }
    }

    /// Reconstruct from stored form plus the key fields.
    ///
    /// `activated_at` is the hunt's activation timestamp, used to reconstruct
    /// absolute timestamps from the stored deltas.
    pub fn from_stored(
        env: &Env,
        stored: StoredPlayerProgress,
        player: Address,
        hunt_id: u64,
        activated_at: u64,
    ) -> Self {
        let mut completed_clue_index = Map::new(env);
        for i in 0..stored.completed_clues.len() {
            // Stored state may be inconsistent — skip missing entries instead of aborting.
            let Some(clue_id) = stored.completed_clues.get(i) else {
                continue;
            };
            completed_clue_index.set(clue_id, true);
        }

        let started_at = activated_at + (stored.started_at_delta as u64);
        let completed_at = if stored.completed_at_delta == 0 {
            0u64
        } else {
            started_at + (stored.completed_at_delta as u64)
        };

        Self {
            player,
            hunt_id,
            completed_clues: stored.completed_clues,
            completed_clue_index,
            hinted_clues: stored.hinted_clues,
            total_score: stored.total_score,
            required_completed_count: stored.required_completed_count,
            started_at,
            completed_at,
            is_completed: (stored.flags & 0b0000_0001) != 0,
            reward_claimed: (stored.flags & 0b0000_0010) != 0,
            completion_rank: stored.completion_rank,
            recent_submissions: stored.recent_submissions,
            clue_last_attempts: stored.clue_last_attempts,
        }
    }

    pub fn has_completed_clue(&self, clue_id: u32) -> bool {
        for i in 0..self.completed_clues.len() {
            // Stored state may be inconsistent — treat missing entries as not completed.
            let Some(stored_id) = self.completed_clues.get(i) else {
                return false;
            };
            if stored_id == clue_id {
                return true;
            }
        }
        false
    }

    pub fn has_requested_hint(&self, clue_id: u32) -> bool {
        for i in 0..self.hinted_clues.len() {
            // Stored state may be inconsistent — treat missing entries as not hinted.
            let Some(stored_id) = self.hinted_clues.get(i) else {
                return false;
            };
            if stored_id == clue_id {
                return true;
            }
        }
        false
    }

    pub fn request_hint(
        &mut self,
        clue_id: u32,
        penalty: u32,
    ) -> Result<(), crate::errors::HuntErrorCode> {
        if self.has_requested_hint(clue_id) {
            return Err(crate::errors::HuntErrorCode::HintAlreadyUnlocked);
        }
        if self.total_score < penalty {
            return Err(crate::errors::HuntErrorCode::InsufficientScore);
        }
        self.total_score = self.total_score.saturating_sub(penalty);
        self.hinted_clues.push_back(clue_id);
        Ok(())
    }

    pub fn complete_clue(
        &mut self,
        _env: &Env,
        clue_id: u32,
        points: u32,
    ) -> Result<(), crate::errors::HuntErrorCode> {
        if !self.has_completed_clue(clue_id) {
            self.completed_clues.push_back(clue_id);
            self.total_score = self
                .total_score
                .checked_add(points)
                .ok_or(crate::errors::HuntErrorCode::ScoreOverflow)?;
        }
        Ok(())
    }
}

impl Hunt {
    pub fn is_active(&self, current_time: u64) -> bool {
        self.status == HuntStatus::Active
            && (self.start_time == 0 || current_time >= self.start_time)
            && (self.end_time == 0 || current_time < self.end_time)
    }

    pub fn has_rewards_available(&self) -> bool {
        self.reward_config.claimed_count < self.reward_config.max_winners
    }
}

impl RewardConfig {
    pub fn new(
        _env: &Env,
        xlm_pool: i128,
        nft_enabled: bool,
        nft_contract: Option<Address>,
        max_winners: u32,
        nft_rarity: u32,
        nft_tier: u32,
        nft_image_uri: Option<String>,
    ) -> Self {
        Self {
            xlm_pool,
            nft_enabled,
            nft_contract,
            max_winners,
            claimed_count: 0,
            nft_rarity,
            nft_tier,
            nft_image_uri,
        }
    }

    pub fn reward_per_winner(&self) -> i128 {
        if self.max_winners == 0 {
            0
        } else {
            self.xlm_pool / (self.max_winners as i128)
        }
    }
}

// Events
#[contracttype]
#[derive(Clone, Debug)]
pub struct HuntCreatedEvent {
    pub hunt_id: u64,
    pub creator: Address,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CreatorBlacklistedEvent {
    pub creator: Address,
    pub admin: Address,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CreatorRemovedFromBlacklistEvent {
    pub creator: Address,
    pub admin: Address,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HuntStatusChangedEvent {
    pub hunt_id: u64,
    pub old_status: HuntStatus,
    pub new_status: HuntStatus,
    pub changed_at: u64,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct ClueCompletedEvent {
    pub hunt_id: u64,
    pub player: Address,
    pub clue_id: u32,
    pub points_earned: u32,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct HuntCompletedEvent {
    pub hunt_id: u64,
    pub player: Address,
    pub total_score: u32,
    pub completion_time: u64,
    pub completion_rank: u32,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct RewardClaimedEvent {
    pub hunt_id: u64,
    pub player: Address,
    pub xlm_amount: i128,
    pub nft_awarded: bool,
}

/// Emitted when a hunt is cloned.
#[contracttype]
#[derive(Clone, Debug)]
pub struct HuntClonedEvent {
    pub original_hunt_id: u64,
    pub new_hunt_id: u64,
    pub creator: Address,
}

/// Emitted when a clue is added. Does not expose the answer hash.
#[contracttype]
#[derive(Clone, Debug)]
pub struct ClueAddedEvent {
    pub hunt_id: u64,
    pub clue_id: u32,
    pub creator: Address,
    pub question: String,
    pub points: u32,
    pub is_required: bool,
    /// Difficulty tier (1-5, 1 = easiest, 5 = hardest).
    pub difficulty: u32,
    /// Weight multiplier (default 1).
    pub weight: u32,
}

/// Emitted when a player registers for an active hunt.
#[contracttype]
#[derive(Clone, Debug)]
pub struct PlayerRegisteredEvent {
    pub hunt_id: u64,
    pub player: Address,
}

/// Emitted when a hunt creator generates or updates the invite code for a private hunt.
/// The invite code itself is never emitted or stored — only its hash.
#[contracttype]
#[derive(Clone, Debug)]
pub struct InviteCodeGeneratedEvent {
    pub hunt_id: u64,
    pub creator: Address,
}

/// Emitted when a hunt creator clears the invite code, pausing new registrations.
#[contracttype]
#[derive(Clone, Debug)]
pub struct InviteCodeRevokedEvent {
    pub hunt_id: u64,
    pub creator: Address,
}

/// Emitted when a player successfully registers using an invite code.
#[contracttype]
#[derive(Clone, Debug)]
pub struct PlayerRegisteredWithInviteEvent {
    pub hunt_id: u64,
    pub player: Address,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct PlayerBannedEvent {
    pub hunt_id: u64,
    pub player: Address,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct PlayerUnbannedEvent {
    pub hunt_id: u64,
    pub player: Address,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct AnswerIncorrectEvent {
    pub hunt_id: u64,
    pub player: Address,
    pub clue_id: u32,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct AnswerPreviewedEvent {
    pub hunt_id: u64,
    pub player: Address,
    pub clue_id: u32,
    pub is_correct: bool,
    pub timestamp: u64,
}

/// Leaderboard entry for a single player in a hunt (read-only query result).
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LeaderboardEntry {
    pub rank: u32,
    pub player: Address,
    pub score: u32,
    pub completed_at: u64,
    pub is_completed: bool,
}

/// Stored top-N leaderboard entry maintained incrementally on score changes.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LeaderboardIndexEntry {
    pub player: Address,
    pub score: u32,
    pub completed_at: u64,
    pub is_completed: bool,
}

/// Wrapper returned by `get_hunt_leaderboard` that includes truncation
/// information so callers can tell when the visible entries are incomplete.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LeaderboardResult {
    pub entries: Vec<LeaderboardEntry>,
    pub total_players: u32,
    pub truncated: bool,
}

/// Aggregate statistics for a hunt (read-only query result).
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HuntStatistics {
    pub total_players: u32,
    pub completed_count: u32,
    pub completion_rate_percent: u32,
    pub total_score_sum: u64,
    pub average_score: u32,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct ClueAliasesAddedEvent {
    pub hunt_id: u64,
    pub clue_id: u32,
    pub creator: Address,
    pub aliases_count: u32,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct HuntDescriptionUpdatedEvent {
    pub hunt_id: u64,
    pub creator: Address,
    pub description: String,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct RewardManagerSetEvent {
    pub old_address: Option<Address>,
    pub new_address: Address,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TimeBonusConfig {
    pub start_multiplier_bps: u32,
    pub min_multiplier_bps: u32,
    pub decay_duration_secs: u64,
}

impl TimeBonusConfig {
    pub fn is_valid(&self) -> bool {
        self.decay_duration_secs > 0
            && self.start_multiplier_bps >= self.min_multiplier_bps
            && self.min_multiplier_bps >= 10_000
    }

    pub fn multiplier_bps_at(&self, elapsed_secs: u64) -> u32 {
        if self.decay_duration_secs == 0 {
            return self.min_multiplier_bps;
        }

        if elapsed_secs >= self.decay_duration_secs {
            return self.min_multiplier_bps;
        }

        let start = self.start_multiplier_bps as u128;
        let min = self.min_multiplier_bps as u128;
        let span = start.saturating_sub(min);
        let decay = (span * elapsed_secs as u128) / self.decay_duration_secs as u128;
        (start.saturating_sub(decay)) as u32
    }
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RateLimitStatus {
    pub creations_today: u32,
    pub daily_limit: u32,
    pub cooldown_seconds: u64,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LeaderboardRow {
    pub index: u32,
    pub player: Address,
    pub score: u32,
    pub completed_at: u64,
    pub is_completed: bool,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LeaderboardWindow {
    pub entries: Vec<LeaderboardRow>,
    pub next_index: u32,
    pub finished: bool,
    pub queried_at: u64,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct RewardClaimFailedEvent {
    pub hunt_id: u64,
    pub player: Address,
    pub error_code: u32,
}

/// A team competing in a team-mode hunt.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Team {
    pub team_id: u32,
    pub hunt_id: u64,
    pub name: String,
    pub leader: Address,
    pub members: Vec<Address>,
}

/// Shared progress for a team: clues completed by any member and the combined score.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TeamProgress {
    pub completed_clues: Vec<u32>,
    pub total_score: u32,
}

/// Team leaderboard entry (read-only query result), ranked by shared team score.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TeamLeaderboardEntry {
    pub rank: u32,
    pub team_id: u32,
    pub name: String,
    pub score: u32,
    pub member_count: u32,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct TeamCreatedEvent {
    pub hunt_id: u64,
    pub team_id: u32,
    pub leader: Address,
    pub name: String,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct TeamMemberJoinedEvent {
    pub hunt_id: u64,
    pub team_id: u32,
    pub player: Address,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct RegistrationDeadlineSetEvent {
    pub hunt_id: u64,
    pub registration_deadline: u64,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct PartialScoreClaimedEvent {
    pub hunt_id: u64,
    pub player: Address,
    pub partial_score: u32,
    pub clues_completed: u32,
}
