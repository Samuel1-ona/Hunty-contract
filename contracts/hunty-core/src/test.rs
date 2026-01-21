#[cfg(test)]
extern crate std;

use std::string::ToString;


#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::{Env, String, symbol_short, vec};
    use crate::errors::{HuntErrorCode, HuntError};
    use soroban_sdk::log;



    #[test]
    fn test_placeholder() {
        // TODO: Add comprehensive tests
    }

     #[test]
    fn test_error_with_context_display() {
        let err = HuntError::HuntNotFound { hunt_id: 42 };
        let hunt_error: HuntErrorCode = err.into();
        assert_eq!(hunt_error, HuntErrorCode::HuntNotFound)
    }


    #[test]
    fn test_hunt_not_found_message() {
        let err = HuntError::HuntNotFound { hunt_id: 42 };

        assert_eq!(
            err.to_string(),
            "Hunt not found: ID 42"
        );
    }

     #[test]
    fn test_clue_not_found_message() {
        let err = HuntError::ClueNotFound { hunt_id: 10 };

        assert_eq!(
            err.to_string(),
            "Clue not found for hunt 10"
        );
    }

    #[test]
    fn test_invalid_hunt_status_message() {
        let err = HuntError::InvalidHuntStatus;

        assert_eq!(
            err.to_string(),
            "Invalid hunt status"
        );
    }

    #[test]
    fn test_insufficient_reward_pool_message() {
        let err = HuntError::InsufficientRewardPool{ required: 10000, available: 500};

        assert_eq!(
            err.to_string(),
            "Insufficient reward pool: required 10000, available 500"
        );
    }
}

