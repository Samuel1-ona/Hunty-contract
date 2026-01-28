#[cfg(test)]
extern crate std;

use std::string::ToString;


#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::{Env, String, Address};
    use crate::errors::{HuntErrorCode, HuntError};
    use crate::types::{HuntStatus, RewardConfig};
    use crate::storage::Storage;
    use crate::HuntyCore;

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

    // ========== create_hunt() Tests ==========

    #[test]
    fn test_create_hunt_success() {
        let env = Env::default();
        let creator = Address::generate(&env);
        let title = String::from_str(&env, "Test Hunt");
        let description = String::from_str(&env, "This is a test hunt description");

        let contract = HuntyCore;
        let hunt_id = contract.create_hunt(
            env.clone(),
            creator.clone(),
            title.clone(),
            description.clone(),
            None,
            None,
        ).unwrap();

        // Verify hunt ID is 1 (first hunt)
        assert_eq!(hunt_id, 1);

        // Verify hunt was stored correctly
        let hunt = Storage::get_hunt(&env, hunt_id).unwrap();
        assert_eq!(hunt.hunt_id, hunt_id);
        assert_eq!(hunt.creator, creator);
        assert_eq!(hunt.title, title);
        assert_eq!(hunt.description, description);
        assert_eq!(hunt.status, HuntStatus::Draft);
        assert_eq!(hunt.total_clues, 0);
        assert_eq!(hunt.required_clues, 0);
        assert_eq!(hunt.reward_config.xlm_pool, 0);
        assert_eq!(hunt.reward_config.nft_enabled, false);
        assert_eq!(hunt.reward_config.max_winners, 0);
        assert_eq!(hunt.reward_config.claimed_count, 0);
        assert!(hunt.created_at > 0);
        assert_eq!(hunt.activated_at, 0);
        assert_eq!(hunt.end_time, 0);
    }

    #[test]
    fn test_create_hunt_with_end_time() {
        let env = Env::default();
        let creator = Address::generate(&env);
        let title = String::from_str(&env, "Timed Hunt");
        let description = String::from_str(&env, "A hunt with an end time");
        let end_time = 1000000u64;

        let contract = HuntyCore;
        let hunt_id = contract.create_hunt(
            env.clone(),
            creator.clone(),
            title.clone(),
            description.clone(),
            None,
            Some(end_time),
        ).unwrap();

        let hunt = Storage::get_hunt(&env, hunt_id).unwrap();
        assert_eq!(hunt.end_time, end_time);
    }

    #[test]
    fn test_create_hunt_empty_title() {
        let env = Env::default();
        let creator = Address::generate(&env);
        let title = String::from_str(&env, "");
        let description = String::from_str(&env, "Valid description");

        let contract = HuntyCore;
        let result = contract.create_hunt(
            env.clone(),
            creator,
            title,
            description,
            None,
            None,
        );

        assert_eq!(result, Err(HuntErrorCode::InvalidTitle));
    }

    #[test]
    fn test_create_hunt_title_too_long() {
        let env = Env::default();
        let creator = Address::generate(&env);
        // Create a title longer than 200 characters
        let long_title = String::from_str(&env, &"a".repeat(201));
        let description = String::from_str(&env, "Valid description");

        let contract = HuntyCore;
        let result = contract.create_hunt(
            env.clone(),
            creator,
            long_title,
            description,
            None,
            None,
        );

        assert_eq!(result, Err(HuntErrorCode::InvalidTitle));
    }

    #[test]
    fn test_create_hunt_title_exactly_max_length() {
        let env = Env::default();
        let creator = Address::generate(&env);
        // Create a title exactly 200 characters (should be valid)
        let title = String::from_str(&env, &"a".repeat(200));
        let description = String::from_str(&env, "Valid description");

        let contract = HuntyCore;
        let result = contract.create_hunt(
            env.clone(),
            creator,
            title,
            description,
            None,
            None,
        );

        assert!(result.is_ok());
    }

    #[test]
    fn test_create_hunt_description_too_long() {
        let env = Env::default();
        let creator = Address::generate(&env);
        let title = String::from_str(&env, "Valid Title");
        // Create a description longer than 2000 characters
        let long_description = String::from_str(&env, &"a".repeat(2001));

        let contract = HuntyCore;
        let result = contract.create_hunt(
            env.clone(),
            creator,
            title,
            long_description,
            None,
            None,
        );

        assert_eq!(result, Err(HuntErrorCode::InvalidDescription));
    }

    #[test]
    fn test_create_hunt_description_exactly_max_length() {
        let env = Env::default();
        let creator = Address::generate(&env);
        let title = String::from_str(&env, "Valid Title");
        // Create a description exactly 2000 characters (should be valid)
        let description = String::from_str(&env, &"a".repeat(2000));

        let contract = HuntyCore;
        let result = contract.create_hunt(
            env.clone(),
            creator,
            title,
            description,
            None,
            None,
        );

        assert!(result.is_ok());
    }

    #[test]
    fn test_create_hunt_unique_ids() {
        let env = Env::default();
        let creator = Address::generate(&env);
        let title1 = String::from_str(&env, "Hunt 1");
        let title2 = String::from_str(&env, "Hunt 2");
        let title3 = String::from_str(&env, "Hunt 3");
        let description = String::from_str(&env, "Description");

        let contract = HuntyCore;
        
        let hunt_id1 = contract.create_hunt(
            env.clone(),
            creator.clone(),
            title1,
            description.clone(),
            None,
            None,
        ).unwrap();

        let hunt_id2 = contract.create_hunt(
            env.clone(),
            creator.clone(),
            title2,
            description.clone(),
            None,
            None,
        ).unwrap();

        let hunt_id3 = contract.create_hunt(
            env.clone(),
            creator.clone(),
            title3,
            description,
            None,
            None,
        ).unwrap();

        // Verify IDs are unique and sequential
        assert_eq!(hunt_id1, 1);
        assert_eq!(hunt_id2, 2);
        assert_eq!(hunt_id3, 3);
        assert_ne!(hunt_id1, hunt_id2);
        assert_ne!(hunt_id2, hunt_id3);
    }

    #[test]
    fn test_create_hunt_different_creators() {
        let env = Env::default();
        let creator1 = Address::generate(&env);
        let creator2 = Address::generate(&env);
        let title = String::from_str(&env, "Test Hunt");
        let description = String::from_str(&env, "Description");

        let contract = HuntyCore;
        
        let hunt_id1 = contract.create_hunt(
            env.clone(),
            creator1.clone(),
            title.clone(),
            description.clone(),
            None,
            None,
        ).unwrap();

        let hunt_id2 = contract.create_hunt(
            env.clone(),
            creator2.clone(),
            title,
            description,
            None,
            None,
        ).unwrap();

        // Verify each hunt stores its creator correctly
        let hunt1 = Storage::get_hunt(&env, hunt_id1).unwrap();
        let hunt2 = Storage::get_hunt(&env, hunt_id2).unwrap();
        
        assert_eq!(hunt1.creator, creator1);
        assert_eq!(hunt2.creator, creator2);
        assert_ne!(hunt1.creator, hunt2.creator);
    }

    #[test]
    fn test_create_hunt_counter_increments() {
        let env = Env::default();
        let creator = Address::generate(&env);
        let title = String::from_str(&env, "Test Hunt");
        let description = String::from_str(&env, "Description");

        // Verify counter starts at 0
        assert_eq!(Storage::get_hunt_counter(&env), 0);

        let contract = HuntyCore;
        
        // Create first hunt
        let hunt_id1 = contract.create_hunt(
            env.clone(),
            creator.clone(),
            title.clone(),
            description.clone(),
            None,
            None,
        ).unwrap();

        // Counter should be 1 after first hunt
        assert_eq!(Storage::get_hunt_counter(&env), 1);
        assert_eq!(hunt_id1, 1);

        // Create second hunt
        let hunt_id2 = contract.create_hunt(
            env.clone(),
            creator.clone(),
            title,
            description,
            None,
            None,
        ).unwrap();

        // Counter should be 2 after second hunt
        assert_eq!(Storage::get_hunt_counter(&env), 2);
        assert_eq!(hunt_id2, 2);
    }

    #[test]
    fn test_create_hunt_default_reward_config() {
        let env = Env::default();
        let creator = Address::generate(&env);
        let title = String::from_str(&env, "Test Hunt");
        let description = String::from_str(&env, "Description");

        let contract = HuntyCore;
        let hunt_id = contract.create_hunt(
            env.clone(),
            creator,
            title,
            description,
            None,
            None,
        ).unwrap();

        let hunt = Storage::get_hunt(&env, hunt_id).unwrap();
        let reward_config = hunt.reward_config;

        // Verify default reward config values
        assert_eq!(reward_config.xlm_pool, 0);
        assert_eq!(reward_config.nft_enabled, false);
        assert_eq!(reward_config.nft_contract, None);
        assert_eq!(reward_config.max_winners, 0);
        assert_eq!(reward_config.claimed_count, 0);
    }

    #[test]
    fn test_create_hunt_created_at_timestamp() {
        let env = Env::default();
        let creator = Address::generate(&env);
        let title = String::from_str(&env, "Test Hunt");
        let description = String::from_str(&env, "Description");

        let contract = HuntyCore;
        let hunt_id = contract.create_hunt(
            env.clone(),
            creator,
            title,
            description,
            None,
            None,
        ).unwrap();

        let hunt = Storage::get_hunt(&env, hunt_id).unwrap();
        let current_time = env.ledger().timestamp();

        // Created timestamp should be set and reasonable (within a few seconds)
        assert!(hunt.created_at > 0);
        assert!(hunt.created_at <= current_time);
        // Allow some small time difference for test execution
        assert!(current_time - hunt.created_at < 10);
    }
}

