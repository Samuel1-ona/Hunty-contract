use soroban_sdk::{contracttype, Address, String, Vec, Env};

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HuntStatus {
    Draft,
    Active,
    Completed,
    Cancelled,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RewardConfig {
    pub xlm_pool: i128,
    pub nft_enabled: bool,
    pub nft_contract: Option<Address>,
    pub max_winners: u32,
    pub claimed_count: u32,
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
    pub end_time: u64,
    pub reward_config: RewardConfig,
    pub total_clues: u32,
    pub required_clues: u32,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Clue {
    pub clue_id: u32,
    pub question: String,
    pub answer_hash: String, 
    pub points: u32,
    pub is_required: bool,
    pub hint: String, 
    pub has_location: bool,
    pub location: Location,
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

#[contracttype]
#[derive(Clone, Debug)]
pub struct PlayerProgress {
    pub player: Address,
    pub hunt_id: u64,
    pub completed_clues: Vec<u32>,
    pub total_score: u32,
    pub started_at: u64,
    pub completed_at: u64,
    pub is_completed: bool,
    pub reward_claimed: bool,
}

impl PlayerProgress {
    pub fn new(env: &Env, player: Address, hunt_id: u64, current_time: u64) -> Self {
        Self {
            player,
            hunt_id,
            completed_clues: Vec::new(env),
            total_score: 0,
            started_at: current_time,
            completed_at: 0,
            is_completed: false,
            reward_claimed: false,
        }
    }

    pub fn has_completed_clue(&self, clue_id: u32) -> bool {
        for i in 0..self.completed_clues.len() {
            if self.completed_clues.get(i).unwrap() == clue_id {
                return true;
            }
        }
        false
    }

    pub fn complete_clue(&mut self, _env: &Env, clue_id: u32, points: u32) {
        if !self.has_completed_clue(clue_id) {
            self.completed_clues.push_back(clue_id);
            self.total_score += points;
        }
    }
}

impl Hunt {
    pub fn is_active(&self, current_time: u64) -> bool {
        self.status == HuntStatus::Active 
            && (self.end_time == 0 || current_time < self.end_time)
    }

    pub fn has_rewards_available(&self) -> bool {
        self.reward_config.claimed_count < self.reward_config.max_winners
    }
}

impl RewardConfig {
    pub fn new(
        xlm_pool: i128,
        nft_enabled: bool,
        nft_contract: Option<Address>,
        max_winners: u32,
    ) -> Self {
        Self {
            xlm_pool,
            nft_enabled,
            nft_contract,
            max_winners,
            claimed_count: 0,
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