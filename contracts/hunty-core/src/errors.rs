use soroban_sdk::{contracterror, String};
use thiserror::Error;
use core::fmt;

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
}

#[derive(Error, Debug)]
pub enum HuntError {
    #[error("Hunt not found: ID {hunt_id}")]
    HuntNotFound { hunt_id: u64 },
    #[error("Clue not found for hunt {hunt_id}")]
    ClueNotFound { hunt_id: u64 },
    #[error("Invalid hunt status")]
    InvalidHuntStatus,
    #[error("Player not registered for hunt {hunt_id}")]
    PlayerNotRegistered { hunt_id: u64 },
    #[error("Clue already completed for hunt {hunt_id}")]
    ClueAlreadyCompleted { hunt_id: u64 },
    #[error("Invalid answer submitted")]
    InvalidAnswer,
    #[error("Hunt not active: ID {hunt_id}")]
    HuntNotActive { hunt_id: u64 },
    #[error("Unauthorized access")]
    Unauthorized,
    #[error("Insufficient reward pool: required {required}, available {available}")]
    InsufficientRewardPool { required: i128, available: i128 },
    #[error("Duplicate registration for hunt {hunt_id}")]
    DuplicateRegistration { hunt_id: u64 },
}


impl From<HuntError> for HuntErrorCode {
    fn from(err: HuntError) -> Self {
        match err {
            HuntError::HuntNotFound { .. } => HuntErrorCode::HuntNotFound,
            HuntError::ClueNotFound { .. } => HuntErrorCode::ClueNotFound,
            HuntError::InvalidHuntStatus { .. } => HuntErrorCode::InvalidHuntStatus,
            HuntError::PlayerNotRegistered { .. } => HuntErrorCode::PlayerNotRegistered,
            HuntError::ClueAlreadyCompleted { .. } => HuntErrorCode::ClueAlreadyCompleted,
            HuntError::InvalidAnswer { .. } => HuntErrorCode::InvalidAnswer,
            HuntError::HuntNotActive { .. } => HuntErrorCode::HuntNotActive,
            HuntError::Unauthorized { .. } => HuntErrorCode::Unauthorized,
            HuntError::InsufficientRewardPool { .. } => HuntErrorCode::InsufficientRewardPool,
            HuntError::DuplicateRegistration { .. } => HuntErrorCode::DuplicateRegistration,
        }
    }
}
