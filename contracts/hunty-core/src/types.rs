use soroban_sdk::{contracttype, Address, BytesN, Env, Map, String, Vec};

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HuntStatus {
    Draft,
    Active,
    Completed,
    Cancelled,
    /// Hunt was active but the creator temporarily stopped it.
    /// Distinct from Draft (never activated) so re-registration logic and
    /// UIs can unambiguously communicate "this hunt is paused, not new".
    /// Valid transitions: Paused → Active (re-activate), Paused → Cancelled.
    Paused,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HuntRewardConfig {
    pub xlm_pool: i128,
    pub max_winners: u32,
    pub claimed_count: u32,
    pub nft_enabled: bool,
    pub distribution_config: reward_manager::RewardConfig,
}

/// Optional time-based scoring bonus for a hunt.
/// Multipliers are stored in basis points to avoid floating point math:
/// 10_000 = 1.0x, 15_000 = 1.5x, 20_000 = 2.0x.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TimeBonusConfig {
    /// Multiplier applied at activation time.
    pub start_multiplier_bps: u32,
    /// Minimum multiplier after the decay period completes.
    pub min_multiplier_bps: u32,
    /// Number of seconds after activation over which the multiplier decays.
    pub decay_duration_secs: u64,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct Hunt {
    pub hunt_id: u64,
    pub creator: Address,
    pub title: String,
    pub description: String,
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
    pub max_attempts_per_clue: u32,
}

/// Stored clue with SHA256 answer hashes (supports multiple acceptable answers).
/// Hashes are never exposed via get_clue/list_clues or events.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Clue {
    pub clue_id: u32,
    pub question: String,
    pub answer_hashes: Vec<BytesN<32>>,
    pub points: u32,
    pub is_required: bool,
    /// Difficulty multiplier (1-10). Points earned = points * difficulty.
    pub difficulty: u8,
}

/// Clue info returned by get_clue/list_clues. Excludes answer hash.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClueInfo {
    pub clue_id: u32,
    pub question: String,
    pub points: u32,
    pub is_required: bool,
    /// Difficulty multiplier (1-10).
    pub difficulty: u8,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HuntCancelledEvent {
    pub hunt_id: u64,
    pub cancelled_by: Address,
    pub cancelled_at: u64,
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
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Location {
    pub latitude: i64,  // Degrees * 1_000_000
    pub longitude: i64, // Degrees * 1_000_000
    pub radius: u32,
}

impl Default for Location {
    fn default() -> Self {
        Self {
            latitude: 0,
            longitude: 0,
            radius: 0,
        }
    }
}

/// Internal storage representation of player progress.
/// Does not store `player` or `hunt_id` — those are already the storage key.
#[contracttype]
#[derive(Clone, Debug)]
pub struct StoredPlayerProgress {
    pub completed_clues: Vec<u32>,
    pub clue_attempts: Map<u32, u32>,
    pub total_score: u32,
    pub required_completed_count: u32,
    pub started_at: u64,
    pub completed_at: u64,
    pub is_completed: bool,
    pub reward_claimed: bool,
    pub clue_attempts: Map<u32, u32>,
}

/// Public view of player progress, with `player` and `hunt_id` reconstructed from the key.
#[contracttype]
#[derive(Clone, Debug)]
pub struct PlayerProgress {
    pub player: Address,
    pub hunt_id: u64,
    pub completed_clues: Vec<u32>,
    pub completed_clue_index: Map<u32, bool>,
    pub required_completed_count: u32,
    pub total_score: u32,
    pub required_completed_count: u32,
    pub started_at: u64,
    pub completed_at: u64,
    pub is_completed: bool,
    pub reward_claimed: bool,
    pub clue_attempts: Map<u32, u32>,
}

impl PlayerProgress {
    pub fn new(env: &Env, player: Address, hunt_id: u64, current_time: u64) -> Self {
        Self {
            player,
            hunt_id,
            completed_clues: Vec::new(env),
            completed_clue_index: Map::new(env),
            required_completed_count: 0,
            total_score: 0,
            required_completed_count: 0,
            started_at: current_time,
            completed_at: 0,
            is_completed: false,
            reward_claimed: false,
            clue_attempts: Map::new(env),
        }
    }

    /// Convert to the compact form stored on-chain (drops redundant key fields).
    pub fn to_stored(&self) -> StoredPlayerProgress {
        StoredPlayerProgress {
            completed_clues: self.completed_clues.clone(),
            clue_attempts: self.clue_attempts.clone(),
            total_score: self.total_score,
            required_completed_count: self.required_completed_count,
            started_at: self.started_at,
            completed_at: self.completed_at,
            is_completed: self.is_completed,
            reward_claimed: self.reward_claimed,
            clue_attempts: self.clue_attempts.clone(),
        }
    }

    /// Reconstruct from stored form plus the key fields.
    pub fn from_stored(
        env: &Env,
        stored: StoredPlayerProgress,
        player: Address,
        hunt_id: u64,
    ) -> Self {
        let mut completed_clue_index = Map::new(env);
        for i in 0..stored.completed_clues.len() {
            let clue_id = stored.completed_clues.get(i).unwrap();
            completed_clue_index.set(clue_id, true);
        }

        Self {
            player,
            hunt_id,
            completed_clues: stored.completed_clues,
            completed_clue_index,
            required_completed_count: stored.required_completed_count,
            total_score: stored.total_score,
            required_completed_count: stored.required_completed_count,
            started_at: stored.started_at,
            completed_at: stored.completed_at,
            is_completed: stored.is_completed,
            reward_claimed: stored.reward_claimed,
            clue_attempts: stored.clue_attempts,
        }
    }

    pub fn has_completed_clue(&self, clue_id: u32) -> bool {
        self.completed_clue_index.get(clue_id).is_some()
    }

    pub fn complete_clue(&mut self, _env: &Env, clue_id: u32, points: u32, is_required: bool) {
        if !self.has_completed_clue(clue_id) {
            self.completed_clues.push_back(clue_id);
            self.completed_clue_index.set(clue_id, true);
            if is_required {
                self.required_completed_count += 1;
            }
            self.total_score = self.total_score.saturating_add(points);
        }
    }

    /// Increments the attempt counter for a clue and returns the new attempt number.
    pub fn record_attempt(&mut self, clue_id: u32) -> u32 {
        let current = self.clue_attempts.get(clue_id).unwrap_or(0);
        let next = current + 1;
        self.clue_attempts.set(clue_id, next);
        next
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

    pub fn time_bonus_multiplier_bps(&self, current_time: u64) -> u32 {
        match (
            self.time_bonus_start_bps,
            self.time_bonus_min_bps,
            self.time_bonus_decay_secs,
        ) {
            (Some(start), Some(min), Some(duration)) => {
                let config = TimeBonusConfig {
                    start_multiplier_bps: start,
                    min_multiplier_bps: min,
                    decay_duration_secs: duration,
                };
                let elapsed = current_time.saturating_sub(self.activated_at);
                config.multiplier_bps_at(elapsed)
            }
            _ => 10_000,
        }
    }

    pub fn time_bonus_config(&self) -> Option<TimeBonusConfig> {
        match (
            self.time_bonus_start_bps,
            self.time_bonus_min_bps,
            self.time_bonus_decay_secs,
        ) {
            (Some(start), Some(min), Some(duration)) => Some(TimeBonusConfig {
                start_multiplier_bps: start,
                min_multiplier_bps: min,
                decay_duration_secs: duration,
            }),
            _ => None,
        }
    }

    pub fn bonus_score(&self, points: u32, current_time: u64) -> u32 {
        let multiplier_bps = self.time_bonus_multiplier_bps(current_time) as u128;
        let scaled = (points as u128 * multiplier_bps) / 10_000;
        core::cmp::min(scaled, u32::MAX as u128) as u32
    }
}

impl HuntRewardConfig {
    pub fn new(
        env: &Env,
        xlm_pool: i128,
        nft_enabled: bool,
        nft_contract: Option<Address>,
        max_winners: u32,
        nft_rarity: u32,
        nft_tier: u32,
    ) -> Self {
        Self {
            xlm_pool,
            max_winners,
            claimed_count: 0,
            nft_enabled,
            distribution_config: reward_manager::RewardConfig {
                xlm_amount: None,
                nft_contract,
                nft_title: String::from_str(env, ""),
                nft_description: String::from_str(env, ""),
                nft_image_uri: String::from_str(env, ""),
                nft_hunt_title: String::from_str(env, ""),
                nft_rarity,
                nft_tier,
            },
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

// Events
#[contracttype]
#[derive(Clone, Debug)]
pub struct HuntCreatedEvent {
    pub hunt_id: u64,
    pub creator: Address,
    pub title: String,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct HuntStatusChangedEvent {
    pub hunt_id: u64,
    pub old_status: HuntStatus,
    pub new_status: HuntStatus,
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
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct RewardClaimedEvent {
    pub hunt_id: u64,
    pub player: Address,
    pub xlm_amount: i128,
    pub nft_awarded: bool,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct RewardClaimFailedEvent {
    pub hunt_id: u64,
    pub player: Address,
    pub error_code: u32,
}

/// Emitted when a clue is added. Does not expose the question or answer hash.
#[contracttype]
#[derive(Clone, Debug)]
pub struct ClueAddedEvent {
    pub hunt_id: u64,
    pub clue_id: u32,
    pub creator: Address,
    pub points: u32,
    pub is_required: bool,
    /// Difficulty multiplier (1-10).
    pub difficulty: u8,
}

/// Emitted when a player registers for an active hunt.
#[contracttype]
#[derive(Clone, Debug)]
pub struct PlayerRegisteredEvent {
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
    pub attempt_number: u32,
}

/// Leaderboard entry for a single player in a hunt (read-only query result).
/// `queried_at` is the ledger timestamp at the moment the leaderboard was fetched,
/// giving frontend caches a reliable "last refreshed" anchor distinct from
/// the per-player `completed_at`.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LeaderboardEntry {
    pub rank: u32,
    pub player: Address,
    pub score: u32,
    pub completed_at: u64,
    pub is_completed: bool,
    pub queried_at: u64,
}

/// Lightweight row returned when scanning a window of players. Includes the
/// original player index so callers can merge/paginate results client-side.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LeaderboardRow {
    pub index: u32,
    pub player: Address,
    pub score: u32,
    pub completed_at: u64,
    pub is_completed: bool,
}

/// Result of a single leaderboard scan window. Clients may call repeatedly
/// with `next_index` until `finished` is true, merging `entries` off-chain to
/// produce a global top-N leaderboard without requiring a single large on-chain
/// scan (which would be expensive in gas).
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LeaderboardWindow {
    pub entries: Vec<LeaderboardRow>,
    pub next_index: u32,
    pub finished: bool,
    pub queried_at: u64,
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
