use core::fmt;
use soroban_sdk::{contracterror, String};

#[contracterror]
#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum HuntErrorCode {
    HuntNotFound = 1,
    ClueNotFound = 2,
    InvalidHuntStatus = 3,
    PlayerNotRegistered = 4,
    ClueAlreadyCompleted = 5,
    InvalidAnswer = 6,
    HuntNotActive = 7,
    Unauthorized = 8,
    InsufficientRewardPool = 9,
    DuplicateRegistration = 10,
    InvalidTitle = 11,
    InvalidDescription = 12,
    InvalidAddress = 13,
    TooManyClues = 14,
    InvalidQuestion = 15,
    RefundFailed = 16,
    NoCluesAdded = 17,
    HuntNotCompleted = 18,
    RewardAlreadyClaimed = 19,
    RewardDistributionFailed = 20,
    NoRewardsConfigured = 21,
}

#[derive(Debug)]
pub enum HuntError {
    HuntNotFound { hunt_id: u64 },
    ClueNotFound { hunt_id: u64 },
    InvalidHuntStatus,
    PlayerNotRegistered { hunt_id: u64 },
    ClueAlreadyCompleted { hunt_id: u64 },
    InvalidAnswer,
    HuntNotActive { hunt_id: u64 },
    Unauthorized,
    InsufficientRewardPool { required: i128, available: i128 },
    DuplicateRegistration { hunt_id: u64 },
    InvalidTitle { reason: String },
    InvalidDescription { reason: String },
    InvalidAddress,
    TooManyClues { hunt_id: u64, limit: u32 },
    InvalidQuestion,
    HuntNotCompleted { hunt_id: u64 },
    RewardAlreadyClaimed { hunt_id: u64 },
    RewardDistributionFailed { hunt_id: u64 },
    NoRewardsConfigured { hunt_id: u64 },
}

impl fmt::Display for HuntError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HuntError::HuntNotFound { hunt_id } => {
                write!(f, "Hunt not found: ID {}", hunt_id)
            }
            HuntError::ClueNotFound { hunt_id } => {
                write!(f, "Clue not found for hunt {}", hunt_id)
            }
            HuntError::InvalidHuntStatus => {
                write!(f, "Invalid hunt status")
            }
            HuntError::PlayerNotRegistered { hunt_id } => {
                write!(f, "Player not registered for hunt {}", hunt_id)
            }
            HuntError::ClueAlreadyCompleted { hunt_id } => {
                write!(f, "Clue already completed for hunt {}", hunt_id)
            }
            HuntError::InvalidAnswer => {
                write!(f, "Invalid answer submitted")
            }
            HuntError::HuntNotActive { hunt_id } => {
                write!(f, "Hunt not active: ID {}", hunt_id)
            }
            HuntError::Unauthorized => {
                write!(f, "Unauthorized access")
            }
            HuntError::InsufficientRewardPool {
                required,
                available,
            } => {
                write!(
                    f,
                    "Insufficient reward pool: required {}, available {}",
                    required, available
                )
            }
            HuntError::DuplicateRegistration { hunt_id } => {
                write!(f, "Duplicate registration for hunt {}", hunt_id)
            }
            HuntError::InvalidTitle { reason } => {
                write!(f, "Invalid title: {:?}", reason)
            }
            HuntError::InvalidDescription { reason } => {
                write!(f, "Invalid description: {:?}", reason)
            }
            HuntError::InvalidAddress => {
                write!(f, "Invalid address")
            }
            HuntError::TooManyClues { hunt_id, limit } => {
                write!(f, "Too many clues for hunt {} (limit {})", hunt_id, limit)
            }
            HuntError::InvalidQuestion => {
                write!(f, "Invalid question (empty or exceeds max length)")
            }
            HuntError::HuntNotCompleted { hunt_id } => {
                write!(f, "Hunt {} not completed by player", hunt_id)
            }
            HuntError::RewardAlreadyClaimed { hunt_id } => {
                write!(f, "Reward already claimed for hunt {}", hunt_id)
            }
            HuntError::RewardDistributionFailed { hunt_id } => {
                write!(f, "Reward distribution failed for hunt {}", hunt_id)
            }
            HuntError::NoRewardsConfigured { hunt_id } => {
                write!(f, "No rewards configured for hunt {}", hunt_id)
            }
        }
    }
}

impl From<HuntError> for HuntErrorCode {
    fn from(err: HuntError) -> Self {
        match err {
            HuntError::HuntNotFound { .. } => HuntErrorCode::HuntNotFound,
            HuntError::ClueNotFound { .. } => HuntErrorCode::ClueNotFound,
            HuntError::InvalidHuntStatus => HuntErrorCode::InvalidHuntStatus,
            HuntError::PlayerNotRegistered { .. } => HuntErrorCode::PlayerNotRegistered,
            HuntError::ClueAlreadyCompleted { .. } => HuntErrorCode::ClueAlreadyCompleted,
            HuntError::InvalidAnswer => HuntErrorCode::InvalidAnswer,
            HuntError::HuntNotActive { .. } => HuntErrorCode::HuntNotActive,
            HuntError::Unauthorized => HuntErrorCode::Unauthorized,
            HuntError::InsufficientRewardPool { .. } => HuntErrorCode::InsufficientRewardPool,
            HuntError::DuplicateRegistration { .. } => HuntErrorCode::DuplicateRegistration,
            HuntError::InvalidTitle { .. } => HuntErrorCode::InvalidTitle,
            HuntError::InvalidDescription { .. } => HuntErrorCode::InvalidDescription,
            HuntError::InvalidAddress => HuntErrorCode::InvalidAddress,
            HuntError::TooManyClues { .. } => HuntErrorCode::TooManyClues,
            HuntError::InvalidQuestion => HuntErrorCode::InvalidQuestion,
            HuntError::HuntNotCompleted { .. } => HuntErrorCode::HuntNotCompleted,
            HuntError::RewardAlreadyClaimed { .. } => HuntErrorCode::RewardAlreadyClaimed,
            HuntError::RewardDistributionFailed { .. } => HuntErrorCode::RewardDistributionFailed,
            HuntError::NoRewardsConfigured { .. } => HuntErrorCode::NoRewardsConfigured,
        }
    }
}
