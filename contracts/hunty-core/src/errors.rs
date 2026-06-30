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
    DuplicateSubmission = 22,
    SubmissionExpired = 23,
    BannedPlayer = 24,
    NoRequiredClues = 25,
    RateLimitExceeded = 26,
    ScoreOverflow = 27,
    RegistrationsPaused = 28,
    AnswersPaused = 29,
    RewardsPaused = 30,
    HuntEndTimeInPast = 31,
    NoPendingAdmin = 32,
    PendingAdminMismatch = 33,
    InvalidRarity = 34,
    InvalidTimeBonusConfig = 35,
    AddressBlacklisted = 36,
    ContractPaused = 37,
    InvalidMaxAttempts = 38,
    MaxAttemptsExceeded = 39,
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
    DuplicateSubmission { hunt_id: u64, clue_id: u32 },
    SubmissionExpired { submitted_at: u64, current_time: u64 },
    BannedPlayer { hunt_id: u64, player: soroban_sdk::Address },
    NoRequiredClues { hunt_id: u64 },
    RateLimitExceeded { cooldown_remaining: u64 },
    ScoreOverflow,
    RegistrationsPaused,
    AnswersPaused,
    RewardsPaused,
    HuntEndTimeInPast { end_time: u64, current_time: u64 },
    NoPendingAdmin,
    PendingAdminMismatch { expected: soroban_sdk::Address, actual: soroban_sdk::Address },
    AdminAlreadyProposed { pending: soroban_sdk::Address },
    InvalidRarity { value: u32 },
    InvalidTimeBonusConfig,
    AddressBlacklisted,
    ContractPaused,
    InvalidMaxAttempts,
    MaxAttemptsExceeded { clue_id: u32, limit: u32 },
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
            HuntError::DuplicateSubmission { hunt_id, clue_id } => {
                write!(
                    f,
                    "Duplicate submission detected for hunt {} clue {}",
                    hunt_id, clue_id
                )
            }
            HuntError::SubmissionExpired {
                submitted_at,
                current_time,
            } => {
                write!(
                    f,
                    "Submission expired or invalid: submitted_at {}, current_time {}",
                    submitted_at, current_time
                )
            }
            HuntError::BannedPlayer { hunt_id, player } => {
                write!(f, "Player {:?} is banned from hunt {}", player, hunt_id)
            }
            HuntError::NoRequiredClues { hunt_id } => {
                write!(f, "Hunt {} has no required clues; at least one required clue must exist before activation", hunt_id)
            }
            HuntError::RateLimitExceeded { cooldown_remaining } => {
                write!(f, "Rate limit exceeded. Try again in {} seconds", cooldown_remaining)
            }
            HuntError::ScoreOverflow => {
                write!(f, "Score calculation overflow")
            }
            HuntError::RegistrationsPaused => {
                write!(f, "Registrations are currently paused")
            }
            HuntError::AnswersPaused => {
                write!(f, "Answer submissions are currently paused")
            }
            HuntError::RewardsPaused => {
                write!(f, "Reward claims are currently paused")
            }
            HuntError::HuntEndTimeInPast { end_time, current_time } => {
                write!(f, "Hunt end_time {} is in the past (current time: {})", end_time, current_time)
            }
            HuntError::NoPendingAdmin => {
                write!(f, "No pending admin rotation to accept")
            }
            HuntError::PendingAdminMismatch { expected, actual } => {
                write!(
                    f,
                    "Pending admin mismatch: expected {}, got {}",
                    expected, actual
                )
            }
            HuntError::AdminAlreadyProposed { pending } => {
                write!(f, "Admin rotation already proposed for {}", pending)
            }
            HuntError::AddressBlacklisted => {
                write!(f, "Address is blacklisted from creating hunts")
            }
            HuntError::InvalidRarity { value } => {
                write!(f, "Invalid rarity value: {}", value)
            }
            HuntError::InvalidTimeBonusConfig => {
                write!(f, "Invalid time bonus configuration")
            }
            HuntError::ContractPaused => {
                write!(f, "Contract is currently paused")
            }
            HuntError::InvalidMaxAttempts => {
                write!(f, "max_attempts_per_clue must be greater than zero")
            }
            HuntError::MaxAttemptsExceeded { clue_id, limit } => {
                write!(f, "Max attempts ({}) exceeded for clue {}", limit, clue_id)
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
            HuntError::DuplicateSubmission { .. } => HuntErrorCode::DuplicateSubmission,
            HuntError::SubmissionExpired { .. } => HuntErrorCode::SubmissionExpired,
            HuntError::BannedPlayer { .. } => HuntErrorCode::BannedPlayer,
            HuntError::NoRequiredClues { .. } => HuntErrorCode::NoRequiredClues,
            HuntError::RateLimitExceeded { .. } => HuntErrorCode::RateLimitExceeded,
            HuntError::ScoreOverflow => HuntErrorCode::ScoreOverflow,
            HuntError::RegistrationsPaused => HuntErrorCode::RegistrationsPaused,
            HuntError::AnswersPaused => HuntErrorCode::AnswersPaused,
            HuntError::RewardsPaused => HuntErrorCode::RewardsPaused,
            HuntError::HuntEndTimeInPast { .. } => HuntErrorCode::HuntEndTimeInPast,
            HuntError::NoPendingAdmin => HuntErrorCode::NoPendingAdmin,
            HuntError::PendingAdminMismatch { .. } => HuntErrorCode::PendingAdminMismatch,
            HuntError::AdminAlreadyProposed { .. } => HuntErrorCode::Unauthorized,
            HuntError::InvalidRarity { .. } => HuntErrorCode::InvalidRarity,
            HuntError::InvalidTimeBonusConfig => HuntErrorCode::InvalidTimeBonusConfig,
            HuntError::AddressBlacklisted => HuntErrorCode::AddressBlacklisted,
            HuntError::ContractPaused => HuntErrorCode::ContractPaused,
            HuntError::InvalidMaxAttempts => HuntErrorCode::InvalidMaxAttempts,
            HuntError::MaxAttemptsExceeded { .. } => HuntErrorCode::MaxAttemptsExceeded,
        }
    }
}
