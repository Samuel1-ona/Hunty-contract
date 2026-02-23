#[cfg(test)]
extern crate std;

use std::string::ToString;

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::{Address, Env, String, Vec};
    // Bring Soroban testutils traits into scope (generate addresses, set ledger info, register contracts).
    use crate::errors::{HuntError, HuntErrorCode};
    use crate::storage::Storage;
    use crate::types::HuntStatus;
    use crate::HuntyCore;
    use nft_reward::{NftMetadata, NftReward};
    use reward_manager::RewardManager;
    use soroban_sdk::testutils::{Address as _, Ledger as _, Register as _};
    use soroban_sdk::{token, String as SorobanString};

    /// Runs a closure inside a registered HuntyCore contract context so storage is accessible.
    fn with_core_contract<T>(env: &Env, f: impl FnOnce(&Env, &Address) -> T) -> T {
        let contract_id = env.register_contract(None, HuntyCore);
        env.as_contract(&contract_id, || f(env, &contract_id))
    }

    /// Runs a closure in the given contract's context. Use when multiple invocations must share
    /// the same storage; call once per step that uses require_auth (Soroban allows one auth per frame).
    fn as_core_contract<T>(env: &Env, contract_id: &Address, f: impl FnOnce(&Env) -> T) -> T {
        env.as_contract(contract_id, || f(env))
    }

    /// Helper to set up RewardManager with XLM token and optional default NFT contract.
    fn setup_reward_manager(
        env: &Env,
        nft_contract: Option<&Address>,
    ) -> (Address, Address, Address) {
        let reward_manager_id = env.register(RewardManager, ());
        let token_admin = Address::generate(env);
        let token_contract = env.register_stellar_asset_contract_v2(token_admin.clone());
        let token_address = token_contract.address();

        env.as_contract(&reward_manager_id, || {
            RewardManager::initialize(env.clone(), token_address.clone());
            if let Some(nft) = nft_contract {
                RewardManager::set_nft_reward_contract(env.clone(), nft.clone());
            }
        });

        (reward_manager_id, token_address, token_admin)
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

        assert_eq!(err.to_string(), "Hunt not found: ID 42");
    }

    #[test]
    fn test_clue_not_found_message() {
        let err = HuntError::ClueNotFound { hunt_id: 10 };

        assert_eq!(err.to_string(), "Clue not found for hunt 10");
    }

    #[test]
    fn test_invalid_hunt_status_message() {
        let err = HuntError::InvalidHuntStatus;

        assert_eq!(err.to_string(), "Invalid hunt status");
    }

    #[test]
    fn test_insufficient_reward_pool_message() {
        let err = HuntError::InsufficientRewardPool {
            required: 10000,
            available: 500,
        };

        assert_eq!(
            err.to_string(),
            "Insufficient reward pool: required 10000, available 500"
        );
    }

    // ========== create_hunt() Tests ==========

    #[test]
    fn test_create_hunt_success() {
        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);
        let creator = Address::generate(&env);
        let title = String::from_str(&env, "Test Hunt");
        let description = String::from_str(&env, "This is a test hunt description");

        let (hunt_id, hunt) = with_core_contract(&env, |env, _cid| {
            let hunt_id = HuntyCore::create_hunt(
                env.clone(),
                creator.clone(),
                title.clone(),
                description.clone(),
                None,
                None,
            )
            .unwrap();
            let hunt = Storage::get_hunt(env, hunt_id).unwrap();
            (hunt_id, hunt)
        });

        // Verify hunt ID is 1 (first hunt)
        assert_eq!(hunt_id, 1);
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
        env.ledger().set_timestamp(1_700_000_000);
        let creator = Address::generate(&env);
        let title = String::from_str(&env, "Timed Hunt");
        let description = String::from_str(&env, "A hunt with an end time");
        let end_time = 1000000u64;

        let hunt = with_core_contract(&env, |env, _cid| {
            let hunt_id = HuntyCore::create_hunt(
                env.clone(),
                creator.clone(),
                title.clone(),
                description.clone(),
                None,
                Some(end_time),
            )
            .unwrap();
            Storage::get_hunt(env, hunt_id).unwrap()
        });
        assert_eq!(hunt.end_time, end_time);
    }

    #[test]
    fn test_create_hunt_empty_title() {
        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);
        let creator = Address::generate(&env);
        let title = String::from_str(&env, "");
        let description = String::from_str(&env, "Valid description");

        let result = with_core_contract(&env, |env, _cid| {
            HuntyCore::create_hunt(env.clone(), creator, title, description, None, None)
        });

        assert_eq!(result, Err(HuntErrorCode::InvalidTitle));
    }

    #[test]
    fn test_create_hunt_title_too_long() {
        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);
        let creator = Address::generate(&env);
        // Create a title longer than 200 characters
        let long_title = String::from_str(&env, &"a".repeat(201));
        let description = String::from_str(&env, "Valid description");

        let result = with_core_contract(&env, |env, _cid| {
            HuntyCore::create_hunt(env.clone(), creator, long_title, description, None, None)
        });

        assert_eq!(result, Err(HuntErrorCode::InvalidTitle));
    }

    #[test]
    fn test_create_hunt_title_exactly_max_length() {
        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);
        let creator = Address::generate(&env);
        // Create a title exactly 200 characters (should be valid)
        let title = String::from_str(&env, &"a".repeat(200));
        let description = String::from_str(&env, "Valid description");

        let result = with_core_contract(&env, |env, _cid| {
            HuntyCore::create_hunt(env.clone(), creator, title, description, None, None)
        });

        assert!(result.is_ok());
    }

    #[test]
    fn test_create_hunt_description_too_long() {
        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);
        let creator = Address::generate(&env);
        let title = String::from_str(&env, "Valid Title");
        // Create a description longer than 2000 characters
        let long_description = String::from_str(&env, &"a".repeat(2001));

        let result = with_core_contract(&env, |env, _cid| {
            HuntyCore::create_hunt(env.clone(), creator, title, long_description, None, None)
        });

        assert_eq!(result, Err(HuntErrorCode::InvalidDescription));
    }

    #[test]
    fn test_create_hunt_description_exactly_max_length() {
        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);
        let creator = Address::generate(&env);
        let title = String::from_str(&env, "Valid Title");
        // Create a description exactly 2000 characters (should be valid)
        let description = String::from_str(&env, &"a".repeat(2000));

        let result = with_core_contract(&env, |env, _cid| {
            HuntyCore::create_hunt(env.clone(), creator, title, description, None, None)
        });

        assert!(result.is_ok());
    }

    #[test]
    fn test_create_hunt_unique_ids() {
        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);
        let creator = Address::generate(&env);
        let title1 = String::from_str(&env, "Hunt 1");
        let title2 = String::from_str(&env, "Hunt 2");
        let title3 = String::from_str(&env, "Hunt 3");
        let description = String::from_str(&env, "Description");

        let (hunt_id1, hunt_id2, hunt_id3) = with_core_contract(&env, |env, _cid| {
            let hunt_id1 = HuntyCore::create_hunt(
                env.clone(),
                creator.clone(),
                title1,
                description.clone(),
                None,
                None,
            )
            .unwrap();
            let hunt_id2 = HuntyCore::create_hunt(
                env.clone(),
                creator.clone(),
                title2,
                description.clone(),
                None,
                None,
            )
            .unwrap();
            let hunt_id3 = HuntyCore::create_hunt(
                env.clone(),
                creator.clone(),
                title3,
                description,
                None,
                None,
            )
            .unwrap();
            (hunt_id1, hunt_id2, hunt_id3)
        });

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
        env.ledger().set_timestamp(1_700_000_000);
        let creator1 = Address::generate(&env);
        let creator2 = Address::generate(&env);
        let title = String::from_str(&env, "Test Hunt");
        let description = String::from_str(&env, "Description");

        let (hunt_id1, hunt_id2, hunt1, hunt2) = with_core_contract(&env, |env, _cid| {
            let hunt_id1 = HuntyCore::create_hunt(
                env.clone(),
                creator1.clone(),
                title.clone(),
                description.clone(),
                None,
                None,
            )
            .unwrap();
            let hunt_id2 = HuntyCore::create_hunt(
                env.clone(),
                creator2.clone(),
                title,
                description,
                None,
                None,
            )
            .unwrap();
            let hunt1 = Storage::get_hunt(env, hunt_id1).unwrap();
            let hunt2 = Storage::get_hunt(env, hunt_id2).unwrap();
            (hunt_id1, hunt_id2, hunt1, hunt2)
        });

        assert_eq!(hunt1.creator, creator1);
        assert_eq!(hunt2.creator, creator2);
        assert_ne!(hunt1.creator, hunt2.creator);
    }

    #[test]
    fn test_create_hunt_counter_increments() {
        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);
        let creator = Address::generate(&env);
        let title = String::from_str(&env, "Test Hunt");
        let description = String::from_str(&env, "Description");

        let (start_counter, hunt_id1, counter_after_1, hunt_id2, counter_after_2) =
            with_core_contract(&env, |env, _cid| {
                // Verify counter starts at 0
                let start_counter = Storage::get_hunt_counter(env);

                // Create first hunt
                let hunt_id1 = HuntyCore::create_hunt(
                    env.clone(),
                    creator.clone(),
                    title.clone(),
                    description.clone(),
                    None,
                    None,
                )
                .unwrap();

                // Counter should be 1 after first hunt
                let counter_after_1 = Storage::get_hunt_counter(env);

                // Create second hunt
                let hunt_id2 = HuntyCore::create_hunt(
                    env.clone(),
                    creator.clone(),
                    title,
                    description,
                    None,
                    None,
                )
                .unwrap();

                // Counter should be 2 after second hunt
                let counter_after_2 = Storage::get_hunt_counter(env);

                (
                    start_counter,
                    hunt_id1,
                    counter_after_1,
                    hunt_id2,
                    counter_after_2,
                )
            });

        assert_eq!(start_counter, 0);
        assert_eq!(counter_after_1, 1);
        assert_eq!(hunt_id1, 1);
        assert_eq!(counter_after_2, 2);
        assert_eq!(hunt_id2, 2);
    }

    #[test]
    fn test_create_hunt_default_reward_config() {
        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);
        let creator = Address::generate(&env);
        let title = String::from_str(&env, "Test Hunt");
        let description = String::from_str(&env, "Description");

        let hunt = with_core_contract(&env, |env, _cid| {
            let hunt_id =
                HuntyCore::create_hunt(env.clone(), creator, title, description, None, None)
                    .unwrap();
            Storage::get_hunt(env, hunt_id).unwrap()
        });
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
        env.ledger().set_timestamp(1_700_000_000);
        let creator = Address::generate(&env);
        let title = String::from_str(&env, "Test Hunt");
        let description = String::from_str(&env, "Description");

        let (hunt, current_time) = with_core_contract(&env, |env, _cid| {
            let hunt_id =
                HuntyCore::create_hunt(env.clone(), creator, title, description, None, None)
                    .unwrap();
            (
                Storage::get_hunt(env, hunt_id).unwrap(),
                env.ledger().timestamp(),
            )
        });

        // Created timestamp should be set and reasonable (within a few seconds)
        assert!(hunt.created_at > 0);
        assert!(hunt.created_at <= current_time);
        // Allow some small time difference for test execution
        assert!(current_time - hunt.created_at < 10);
    }

    // ========== add_clue() / get_clue() / list_clues() Tests ==========

    #[test]
    fn test_add_clue_success() {
        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);
        env.mock_all_auths();
        let creator = Address::generate(&env);
        let title = String::from_str(&env, "Test Hunt");
        let description = String::from_str(&env, "Description");
        let question = String::from_str(&env, "What is 2 + 2?");
        let answer = String::from_str(&env, "four");

        let (hunt_id, clue_id, hunt, info) = with_core_contract(&env, |env, _cid| {
            let hunt_id = HuntyCore::create_hunt(
                env.clone(),
                creator.clone(),
                title,
                description.clone(),
                None,
                None,
            )
            .unwrap();
            let clue_id =
                HuntyCore::add_clue(env.clone(), hunt_id, question.clone(), answer, 10, true)
                    .unwrap();
            let hunt = Storage::get_hunt(env, hunt_id).unwrap();
            let info = HuntyCore::get_clue(env.clone(), hunt_id, clue_id).unwrap();
            (hunt_id, clue_id, hunt, info)
        });

        assert_eq!(hunt_id, 1);
        assert_eq!(clue_id, 1);
        assert_eq!(hunt.total_clues, 1);
        assert_eq!(info.clue_id, 1);
        assert_eq!(info.question, question);
        assert_eq!(info.points, 10);
        assert!(info.is_required);
    }

    #[test]
    #[should_panic]
    fn test_add_clue_unauthorized() {
        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);
        // Do NOT mock auth — require_auth(creator) will fail.
        let creator = Address::generate(&env);
        let title = String::from_str(&env, "Test Hunt");
        let description = String::from_str(&env, "Description");
        let question = String::from_str(&env, "What is 2 + 2?");
        let answer = String::from_str(&env, "four");

        with_core_contract(&env, |env, _cid| {
            let hunt_id =
                HuntyCore::create_hunt(env.clone(), creator, title, description, None, None)
                    .unwrap();
            let _ = HuntyCore::add_clue(env.clone(), hunt_id, question, answer, 10, true);
        });
    }

    #[test]
    fn test_add_clue_sequential_ids() {
        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);
        env.mock_all_auths();
        let creator = Address::generate(&env);
        let title = String::from_str(&env, "Hunt");
        let description = String::from_str(&env, "Desc");
        let q1 = String::from_str(&env, "Q1");
        let q2 = String::from_str(&env, "Q2");
        let q3 = String::from_str(&env, "Q3");
        let a = String::from_str(&env, "a");

        let (id1, id2, id3) = with_core_contract(&env, |env, _cid| {
            let hid = HuntyCore::create_hunt(env.clone(), creator, title, description, None, None)
                .unwrap();
            let id1 = HuntyCore::add_clue(env.clone(), hid, q1, a.clone(), 1, false).unwrap();
            let id2 = HuntyCore::add_clue(env.clone(), hid, q2, a.clone(), 1, false).unwrap();
            let id3 = HuntyCore::add_clue(env.clone(), hid, q3, a, 1, false).unwrap();
            (id1, id2, id3)
        });

        assert_eq!(id1, 1);
        assert_eq!(id2, 2);
        assert_eq!(id3, 3);
    }

    #[test]
    fn test_add_clue_answer_normalization_and_hashing() {
        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);
        env.mock_all_auths();
        let creator = Address::generate(&env);
        let title = String::from_str(&env, "Hunt");
        let description = String::from_str(&env, "Desc");
        let question = String::from_str(&env, "Same answer?");
        let answer1 = String::from_str(&env, "  ANSWER  ");
        let answer2 = String::from_str(&env, "answer");

        let (hash1, hash2) = with_core_contract(&env, |env, _cid| {
            let hid = HuntyCore::create_hunt(
                env.clone(),
                creator,
                title,
                description.clone(),
                None,
                None,
            )
            .unwrap();
            let cid =
                HuntyCore::add_clue(env.clone(), hid, question.clone(), answer1, 5, false).unwrap();
            let c = Storage::get_clue(env, hid, cid).unwrap();
            let h1 = c.answer_hash;
            let hid2 = HuntyCore::create_hunt(
                env.clone(),
                Address::generate(&env),
                String::from_str(&env, "H2"),
                description,
                None,
                None,
            )
            .unwrap();
            let _cid2 =
                HuntyCore::add_clue(env.clone(), hid2, question, answer2, 5, false).unwrap();
            let c2 = Storage::get_clue(env, hid2, _cid2).unwrap();
            let h2 = c2.answer_hash;
            (h1, h2)
        });

        assert_eq!(
            hash1, hash2,
            "normalized '  ANSWER  ' and 'answer' must hash the same"
        );
    }

    #[test]
    fn test_get_clue_excludes_answer_hash() {
        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);
        env.mock_all_auths();
        let creator = Address::generate(&env);
        let title = String::from_str(&env, "Hunt");
        let description = String::from_str(&env, "Desc");
        let question = String::from_str(&env, "Secret?");
        let answer = String::from_str(&env, "secret");

        let info = with_core_contract(&env, |env, _cid| {
            let hid = HuntyCore::create_hunt(env.clone(), creator, title, description, None, None)
                .unwrap();
            let _ = HuntyCore::add_clue(env.clone(), hid, question.clone(), answer, 7, true);
            HuntyCore::get_clue(env.clone(), hid, 1).unwrap()
        });

        assert_eq!(info.question, question);
        assert_eq!(info.points, 7);
        assert!(info.is_required);
        // ClueInfo has no answer_hash field — we never expose it.
    }

    #[test]
    fn test_get_clue_not_found() {
        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);
        env.mock_all_auths();
        let creator = Address::generate(&env);
        let title = String::from_str(&env, "Hunt");
        let description = String::from_str(&env, "Desc");

        let err = with_core_contract(&env, |env, _cid| {
            let hid = HuntyCore::create_hunt(env.clone(), creator, title, description, None, None)
                .unwrap();
            HuntyCore::get_clue(env.clone(), hid, 999).unwrap_err()
        });

        assert_eq!(err, HuntErrorCode::ClueNotFound);
    }

    #[test]
    fn test_list_clues_empty() {
        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);
        env.mock_all_auths();
        let creator = Address::generate(&env);
        let title = String::from_str(&env, "Hunt");
        let description = String::from_str(&env, "Desc");

        let list = with_core_contract(&env, |env, _cid| {
            let hid = HuntyCore::create_hunt(env.clone(), creator, title, description, None, None)
                .unwrap();
            HuntyCore::list_clues(env.clone(), hid)
        });

        assert_eq!(list.len(), 0);
    }

    #[test]
    fn test_list_clues_returns_all() {
        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);
        env.mock_all_auths();
        let creator = Address::generate(&env);
        let title = String::from_str(&env, "Hunt");
        let description = String::from_str(&env, "Desc");
        let q1 = String::from_str(&env, "Q1");
        let q2 = String::from_str(&env, "Q2");
        let a = String::from_str(&env, "a");

        let list = with_core_contract(&env, |env, _cid| {
            let hid = HuntyCore::create_hunt(env.clone(), creator, title, description, None, None)
                .unwrap();
            HuntyCore::add_clue(env.clone(), hid, q1, a.clone(), 1, false).unwrap();
            HuntyCore::add_clue(env.clone(), hid, q2, a, 2, true).unwrap();
            HuntyCore::list_clues(env.clone(), hid)
        });

        assert_eq!(list.len(), 2);
        let c1 = list.get(0).unwrap();
        let c2 = list.get(1).unwrap();
        assert_eq!(c1.clue_id, 1);
        assert_eq!(c2.clue_id, 2);
        assert_eq!(c1.points, 1);
        assert_eq!(c2.points, 2);
        assert!(!c1.is_required);
        assert!(c2.is_required);
    }

    #[test]
    fn test_add_clue_hunt_not_found() {
        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);
        env.mock_all_auths();
        let question = String::from_str(&env, "Q");
        let answer = String::from_str(&env, "a");

        let err = with_core_contract(&env, |env, _cid| {
            HuntyCore::add_clue(env.clone(), 9999, question, answer, 1, false).unwrap_err()
        });

        assert_eq!(err, HuntErrorCode::HuntNotFound);
    }

    #[test]
    fn test_add_clue_invalid_question_empty() {
        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);
        env.mock_all_auths();
        let creator = Address::generate(&env);
        let title = String::from_str(&env, "Hunt");
        let description = String::from_str(&env, "Desc");
        let empty = String::from_str(&env, "");
        let answer = String::from_str(&env, "a");

        let err = with_core_contract(&env, |env, _cid| {
            let hid = HuntyCore::create_hunt(env.clone(), creator, title, description, None, None)
                .unwrap();
            HuntyCore::add_clue(env.clone(), hid, empty, answer, 1, false).unwrap_err()
        });

        assert_eq!(err, HuntErrorCode::InvalidQuestion);
    }

    #[test]
    fn test_add_clue_invalid_answer_empty() {
        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);
        env.mock_all_auths();
        let creator = Address::generate(&env);
        let title = String::from_str(&env, "Hunt");
        let description = String::from_str(&env, "Desc");
        let question = String::from_str(&env, "Q");
        let empty = String::from_str(&env, "");

        let err = with_core_contract(&env, |env, _cid| {
            let hid = HuntyCore::create_hunt(env.clone(), creator, title, description, None, None)
                .unwrap();
            HuntyCore::add_clue(env.clone(), hid, question, empty, 1, false).unwrap_err()
        });

        assert_eq!(err, HuntErrorCode::InvalidAnswer);
    }

    #[test]
    fn test_add_clue_invalid_answer_whitespace_only() {
        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);
        env.mock_all_auths();
        let creator = Address::generate(&env);
        let title = String::from_str(&env, "Hunt");
        let description = String::from_str(&env, "Desc");
        let question = String::from_str(&env, "Q");
        let ws = String::from_str(&env, "   \t  ");

        let err = with_core_contract(&env, |env, _cid| {
            let hid = HuntyCore::create_hunt(env.clone(), creator, title, description, None, None)
                .unwrap();
            HuntyCore::add_clue(env.clone(), hid, question, ws, 1, false).unwrap_err()
        });

        assert_eq!(err, HuntErrorCode::InvalidAnswer);
    }

    #[test]
    fn test_add_clue_too_many_clues() {
        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);
        env.mock_all_auths();
        let creator = Address::generate(&env);
        let title = String::from_str(&env, "Hunt");
        let description = String::from_str(&env, "Desc");
        let question = String::from_str(&env, "Q");
        let answer = String::from_str(&env, "a");

        const MAX_CLUES: u32 = 100;
        let err = with_core_contract(&env, |env, _cid| {
            let hid = HuntyCore::create_hunt(env.clone(), creator, title, description, None, None)
                .unwrap();
            for _ in 0..MAX_CLUES {
                HuntyCore::add_clue(env.clone(), hid, question.clone(), answer.clone(), 1, false)
                    .unwrap();
            }
            HuntyCore::add_clue(env.clone(), hid, question, answer, 1, false).unwrap_err()
        });

        assert_eq!(err, HuntErrorCode::TooManyClues);
    }

    #[test]
    fn test_add_clue_invalid_hunt_status_not_draft() {
        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);
        env.mock_all_auths();
        let creator = Address::generate(&env);
        let title = String::from_str(&env, "Hunt");
        let description = String::from_str(&env, "Desc");
        let question = String::from_str(&env, "Q");
        let answer = String::from_str(&env, "a");

        let err = with_core_contract(&env, |env, _cid| {
            let hid = HuntyCore::create_hunt(
                env.clone(),
                creator.clone(),
                title,
                description,
                None,
                None,
            )
            .unwrap();
            let mut h = Storage::get_hunt(env, hid).unwrap();
            h.status = HuntStatus::Active;
            Storage::save_hunt(env, &h);
            HuntyCore::add_clue(env.clone(), hid, question, answer, 1, false).unwrap_err()
        });

        assert_eq!(err, HuntErrorCode::InvalidHuntStatus);
    }

    #[test]
    fn test_add_clue_invalid_question_too_long() {
        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);
        env.mock_all_auths();
        let creator = Address::generate(&env);
        let title = String::from_str(&env, "Hunt");
        let description = String::from_str(&env, "Desc");
        let long_q = String::from_str(&env, &"a".repeat(2001));
        let answer = String::from_str(&env, "a");

        let err = with_core_contract(&env, |env, _cid| {
            let hid = HuntyCore::create_hunt(env.clone(), creator, title, description, None, None)
                .unwrap();
            HuntyCore::add_clue(env.clone(), hid, long_q, answer, 1, false).unwrap_err()
        });

        assert_eq!(err, HuntErrorCode::InvalidQuestion);
    }

    #[test]
    fn test_activate_hunt_success() {
        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);
        env.mock_all_auths();

        let creator = Address::generate(&env);
        let title = String::from_str(&env, "Test Hunt");
        let description = String::from_str(&env, "This is a test hunt description");

        let question = String::from_str(&env, "Valid question");
        let answer = String::from_str(&env, "a");

        with_core_contract(&env, |env, _cid| {
            let hunt_id = HuntyCore::create_hunt(
                env.clone(),
                creator.clone(),
                title,
                description,
                None,
                None,
            )
            .unwrap();

            // Add a VALID clue
            HuntyCore::add_clue(env.clone(), hunt_id, question, answer, 1, false).unwrap();

            // Activate hunt
            HuntyCore::activate_hunt(env.clone(), hunt_id, creator.clone()).unwrap();

            let hunt = Storage::get_hunt(env, hunt_id).unwrap();
            assert_eq!(hunt.status, HuntStatus::Active);
            assert!(hunt.activated_at > 0);
        });
    }

    #[test]
    fn test_activate_hunt_not_found() {
        let env = Env::default();
        let creator = Address::generate(&env);

        with_core_contract(&env, |env, _cid| {
            let err = HuntyCore::activate_hunt(env.clone(), 999, creator.clone()).unwrap_err();
            assert_eq!(err, HuntErrorCode::HuntNotFound);
        });
    }

    #[test]
    fn test_activate_hunt_unauthorized() {
        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);
        env.mock_all_auths();
        let creator = Address::generate(&env);
        let attacker = Address::generate(&env);

        let title = String::from_str(&env, "Test Hunt");
        let description = String::from_str(&env, "Test description");

        with_core_contract(&env, |env, _cid| {
            let hunt_id = HuntyCore::create_hunt(
                env.clone(),
                creator.clone(),
                title,
                description,
                None,
                None,
            )
            .unwrap();

            let err = HuntyCore::activate_hunt(env.clone(), hunt_id, attacker.clone()).unwrap_err();
            assert_eq!(err, HuntErrorCode::Unauthorized);
        });
    }

    #[test]
    fn test_activate_hunt_no_clues() {
        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);
        env.mock_all_auths();
        let creator = Address::generate(&env);

        let title = String::from_str(&env, "Test Hunt");
        let description = String::from_str(&env, "Test description");

        with_core_contract(&env, |env, _cid| {
            let hunt_id = HuntyCore::create_hunt(
                env.clone(),
                creator.clone(),
                title,
                description,
                None,
                None,
            )
            .unwrap();

            let err = HuntyCore::activate_hunt(env.clone(), hunt_id, creator.clone()).unwrap_err();
            assert_eq!(err, HuntErrorCode::NoCluesAdded);
        });
    }

    #[test]
    fn test_deactivate_hunt_success() {
        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);
        env.mock_all_auths();

        let creator = Address::generate(&env);
        let question = String::from_str(&env, "Valid question");
        let answer = String::from_str(&env, "a");

        with_core_contract(&env, |env, _cid| {
            // Create hunt
            let hunt_id = HuntyCore::create_hunt(
                env.clone(),
                creator.clone(),
                String::from_str(env, "Test Hunt"),
                String::from_str(env, "Test description"),
                None,
                None,
            )
            .unwrap();

            // Add a VALID clue first
            HuntyCore::add_clue(env.clone(), hunt_id, question, answer, 1, false).unwrap();

            // Activate hunt
            HuntyCore::activate_hunt(env.clone(), hunt_id, creator.clone()).unwrap();

            // Deactivate hunt
            HuntyCore::deactivate_hunt(env.clone(), hunt_id, creator.clone()).unwrap();

            let hunt = Storage::get_hunt(env, hunt_id).unwrap();
            assert_eq!(hunt.status, HuntStatus::Draft);
        });
    }

    #[test]
    fn test_deactivate_hunt_not_found() {
        let env = Env::default();
        let creator = Address::generate(&env);

        with_core_contract(&env, |env, _cid| {
            let err = HuntyCore::deactivate_hunt(env.clone(), 404, creator.clone()).unwrap_err();
            assert_eq!(err, HuntErrorCode::HuntNotFound);
        });
    }

    #[test]
    fn test_deactivate_hunt_unauthorized() {
        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);
        env.mock_all_auths();

        let creator = Address::generate(&env);
        let attacker = Address::generate(&env);
        let question = String::from_str(&env, "Valid question");
        let answer = String::from_str(&env, "a");

        with_core_contract(&env, |env, _cid| {
            // Create hunt
            let hunt_id = HuntyCore::create_hunt(
                env.clone(),
                creator.clone(),
                String::from_str(env, "Test Hunt"),
                String::from_str(env, "Test description"),
                None,
                None,
            )
            .unwrap();

            // Add a VALID clue first
            HuntyCore::add_clue(env.clone(), hunt_id, question, answer, 1, false).unwrap();

            // Activate hunt
            HuntyCore::activate_hunt(env.clone(), hunt_id, creator.clone()).unwrap();

            // Deactivate hunt
            let err =
                HuntyCore::deactivate_hunt(env.clone(), hunt_id, attacker.clone()).unwrap_err();
            assert_eq!(err, HuntErrorCode::Unauthorized);
        });
    }

    #[test]
    fn test_cancel_hunt_from_active_success() {
        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);
        env.mock_all_auths();

        let creator = Address::generate(&env);
        let question = String::from_str(&env, "Valid question");
        let answer = String::from_str(&env, "a");

        with_core_contract(&env, |env, _cid| {
            // Create hunt
            let hunt_id = HuntyCore::create_hunt(
                env.clone(),
                creator.clone(),
                String::from_str(env, "Test Hunt"),
                String::from_str(env, "Test description"),
                None,
                None,
            )
            .unwrap();

            // Add a VALID clue first
            HuntyCore::add_clue(env.clone(), hunt_id, question, answer, 1, false).unwrap();

            // Activate hunt
            HuntyCore::activate_hunt(env.clone(), hunt_id, creator.clone()).unwrap();

            // Cancelled hunt
            HuntyCore::cancel_hunt(env.clone(), hunt_id, creator.clone()).unwrap();

            let hunt = Storage::get_hunt(env, hunt_id).unwrap();
            assert_eq!(hunt.status, HuntStatus::Cancelled);
        });
    }

    #[test]
    fn test_cancel_hunt_not_found() {
        let env = Env::default();
        let creator = Address::generate(&env);

        with_core_contract(&env, |env, _cid| {
            let err = HuntyCore::cancel_hunt(env.clone(), 999, creator.clone()).unwrap_err();
            assert_eq!(err, HuntErrorCode::HuntNotFound);
        });
    }

    #[test]
    fn test_cancel_hunt_unauthorized() {
        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);
        env.mock_all_auths();

        let creator = Address::generate(&env);
        let attacker = Address::generate(&env);
        let question = String::from_str(&env, "Valid question");
        let answer = String::from_str(&env, "a");

        with_core_contract(&env, |env, _cid| {
            // Create hunt
            let hunt_id = HuntyCore::create_hunt(
                env.clone(),
                creator.clone(),
                String::from_str(env, "Test Hunt"),
                String::from_str(env, "Test description"),
                None,
                None,
            )
            .unwrap();

            // Add a VALID clue first
            HuntyCore::add_clue(env.clone(), hunt_id, question, answer, 1, false).unwrap();

            // Activate hunt
            HuntyCore::activate_hunt(env.clone(), hunt_id, creator.clone()).unwrap();

            // Deactivate hunt
            let err = HuntyCore::cancel_hunt(env.clone(), hunt_id, attacker.clone()).unwrap_err();
            assert_eq!(err, HuntErrorCode::Unauthorized);
        });
    }

    #[test]
    fn test_cancel_hunt_already_cancelled() {
        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);
        env.mock_all_auths();

        let creator = Address::generate(&env);
        let attacker = Address::generate(&env);
        let question = String::from_str(&env, "Valid question");
        let answer = String::from_str(&env, "a");

        with_core_contract(&env, |env, _cid| {
            // Create hunt
            let hunt_id = HuntyCore::create_hunt(
                env.clone(),
                creator.clone(),
                String::from_str(env, "Test Hunt"),
                String::from_str(env, "Test description"),
                None,
                None,
            )
            .unwrap();

            // Add a VALID clue first
            HuntyCore::add_clue(env.clone(), hunt_id, question, answer, 1, false).unwrap();

            // Activate hunt
            HuntyCore::activate_hunt(env.clone(), hunt_id, creator.clone()).unwrap();

            // Deactivate hunt
            HuntyCore::cancel_hunt(env.clone(), hunt_id, creator.clone()).unwrap();
            let err = HuntyCore::cancel_hunt(env.clone(), hunt_id, creator.clone()).unwrap_err();
            assert_eq!(err, HuntErrorCode::InvalidHuntStatus);
        });
    }

    #[test]
    fn test_get_hunt_info() {
        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);
        env.mock_all_auths();

        let creator = Address::generate(&env);
        let attacker = Address::generate(&env);
        let question = String::from_str(&env, "Valid question");
        let answer = String::from_str(&env, "a");

        with_core_contract(&env, |env, _cid| {
            let hunt_id = HuntyCore::create_hunt(
                env.clone(),
                creator.clone(),
                String::from_str(env, "Query Hunt"),
                String::from_str(env, "Desc"),
                None,
                None,
            )
            .unwrap();

            let info = HuntyCore::get_hunt_info(env.clone(), hunt_id).unwrap();

            assert_eq!(info.hunt_id, hunt_id);
            assert_eq!(info.creator, creator);
            assert_eq!(info.title, String::from_str(env, "Query Hunt"));
            assert_eq!(info.status, HuntStatus::Draft);
        });
    }

    // ========== register_player() Tests ==========

    #[test]
    fn test_register_player_success() {
        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);
        env.mock_all_auths();

        let creator = Address::generate(&env);
        let player = Address::generate(&env);
        let question = String::from_str(&env, "Valid question");
        let answer = String::from_str(&env, "a");

        with_core_contract(&env, |env, _cid| {
            let hunt_id = HuntyCore::create_hunt(
                env.clone(),
                creator.clone(),
                String::from_str(env, "Active Hunt"),
                String::from_str(env, "Desc"),
                None,
                None,
            )
            .unwrap();
            HuntyCore::add_clue(env.clone(), hunt_id, question, answer, 10, false).unwrap();
            HuntyCore::activate_hunt(env.clone(), hunt_id, creator.clone()).unwrap();

            HuntyCore::register_player(env.clone(), hunt_id, player.clone()).unwrap();

            let progress =
                HuntyCore::get_player_progress(env.clone(), hunt_id, player.clone()).unwrap();
            assert_eq!(progress.player, player);
            assert_eq!(progress.hunt_id, hunt_id);
            assert_eq!(progress.completed_clues.len(), 0);
            assert_eq!(progress.total_score, 0);
            assert_eq!(progress.is_completed, false);
            assert_eq!(progress.reward_claimed, false);
            assert!(progress.started_at > 0);
            assert_eq!(progress.completed_at, 0);
        });
    }

    #[test]
    fn test_register_player_duplicate_fails() {
        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);
        env.mock_all_auths();

        let creator = Address::generate(&env);
        let player = Address::generate(&env);
        let question = String::from_str(&env, "Q");
        let answer = String::from_str(&env, "a");

        // Pre-populate storage with existing progress so that the single register_player
        // call hits the duplicate check (mock_all_auths only allows one auth per test frame).
        let err = with_core_contract(&env, |env, _cid| {
            let hunt_id = HuntyCore::create_hunt(
                env.clone(),
                creator.clone(),
                String::from_str(env, "Hunt"),
                String::from_str(env, "Desc"),
                None,
                None,
            )
            .unwrap();
            HuntyCore::add_clue(env.clone(), hunt_id, question, answer, 1, false).unwrap();
            HuntyCore::activate_hunt(env.clone(), hunt_id, creator.clone()).unwrap();

            let current_time = env.ledger().timestamp();
            let existing =
                crate::types::PlayerProgress::new(env, player.clone(), hunt_id, current_time);
            Storage::save_player_progress(env, &existing);

            HuntyCore::register_player(env.clone(), hunt_id, player.clone()).unwrap_err()
        });

        assert_eq!(err, HuntErrorCode::DuplicateRegistration);
    }

    #[test]
    fn test_register_player_hunt_not_found() {
        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);
        env.mock_all_auths();
        let player = Address::generate(&env);

        let err = with_core_contract(&env, |env, _cid| {
            HuntyCore::register_player(env.clone(), 9999, player.clone()).unwrap_err()
        });

        assert_eq!(err, HuntErrorCode::HuntNotFound);
    }

    #[test]
    fn test_register_player_hunt_not_active_draft() {
        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);
        env.mock_all_auths();

        let creator = Address::generate(&env);
        let player = Address::generate(&env);
        let question = String::from_str(&env, "Q");
        let answer = String::from_str(&env, "a");

        let err = with_core_contract(&env, |env, _cid| {
            let hunt_id = HuntyCore::create_hunt(
                env.clone(),
                creator.clone(),
                String::from_str(env, "Hunt"),
                String::from_str(env, "Desc"),
                None,
                None,
            )
            .unwrap();
            HuntyCore::add_clue(env.clone(), hunt_id, question, answer, 1, false).unwrap();
            // Hunt is still Draft, not activated
            HuntyCore::register_player(env.clone(), hunt_id, player.clone()).unwrap_err()
        });

        assert_eq!(err, HuntErrorCode::InvalidHuntStatus);
    }

    #[test]
    fn test_register_player_hunt_ended() {
        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);
        env.mock_all_auths();

        let creator = Address::generate(&env);
        let player = Address::generate(&env);
        let question = String::from_str(&env, "Q");
        let answer = String::from_str(&env, "a");
        let end_time = 1_700_000_001; // One second after "now"

        let err = with_core_contract(&env, |env, _cid| {
            let hunt_id = HuntyCore::create_hunt(
                env.clone(),
                creator.clone(),
                String::from_str(env, "Hunt"),
                String::from_str(env, "Desc"),
                None,
                Some(end_time),
            )
            .unwrap();
            HuntyCore::add_clue(env.clone(), hunt_id, question, answer, 1, false).unwrap();
            HuntyCore::activate_hunt(env.clone(), hunt_id, creator.clone()).unwrap();
            // Move time past end_time
            env.ledger().set_timestamp(1_700_000_002);
            HuntyCore::register_player(env.clone(), hunt_id, player.clone()).unwrap_err()
        });

        assert_eq!(err, HuntErrorCode::HuntNotActive);
    }

    #[test]
    fn test_register_player_multiple_players_same_hunt() {
        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);
        env.mock_all_auths();

        let creator = Address::generate(&env);
        let player1 = Address::generate(&env);
        let player2 = Address::generate(&env);
        let player3 = Address::generate(&env);
        let question = String::from_str(&env, "Q");
        let answer = String::from_str(&env, "a");

        with_core_contract(&env, |env, _cid| {
            let hunt_id = HuntyCore::create_hunt(
                env.clone(),
                creator.clone(),
                String::from_str(env, "Hunt"),
                String::from_str(env, "Desc"),
                None,
                None,
            )
            .unwrap();
            HuntyCore::add_clue(env.clone(), hunt_id, question, answer, 1, false).unwrap();
            HuntyCore::activate_hunt(env.clone(), hunt_id, creator.clone()).unwrap();

            HuntyCore::register_player(env.clone(), hunt_id, player1.clone()).unwrap();
            HuntyCore::register_player(env.clone(), hunt_id, player2.clone()).unwrap();
            HuntyCore::register_player(env.clone(), hunt_id, player3.clone()).unwrap();

            let p1 = HuntyCore::get_player_progress(env.clone(), hunt_id, player1.clone()).unwrap();
            let p2 = HuntyCore::get_player_progress(env.clone(), hunt_id, player2.clone()).unwrap();
            let p3 = HuntyCore::get_player_progress(env.clone(), hunt_id, player3.clone()).unwrap();

            assert_eq!(p1.player, player1);
            assert_eq!(p2.player, player2);
            assert_eq!(p3.player, player3);
            assert_eq!(p1.hunt_id, hunt_id);
            assert_eq!(p2.hunt_id, hunt_id);
            assert_eq!(p3.hunt_id, hunt_id);
        });
    }

    #[test]
    #[should_panic]
    fn test_register_player_unauthorized() {
        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);
        // Do NOT mock auth — player.require_auth() will fail if not authorized
        let creator = Address::generate(&env);
        let player = Address::generate(&env);
        let question = String::from_str(&env, "Q");
        let answer = String::from_str(&env, "a");

        with_core_contract(&env, |env, _cid| {
            let hunt_id = HuntyCore::create_hunt(
                env.clone(),
                creator.clone(),
                String::from_str(env, "Hunt"),
                String::from_str(env, "Desc"),
                None,
                None,
            )
            .unwrap();
            HuntyCore::add_clue(env.clone(), hunt_id, question, answer, 1, false).unwrap();
            HuntyCore::activate_hunt(env.clone(), hunt_id, creator.clone()).unwrap();
            HuntyCore::register_player(env.clone(), hunt_id, player.clone()).unwrap();
        });
    }

    #[test]
    fn test_get_player_progress_not_registered() {
        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);
        env.mock_all_auths();

        let creator = Address::generate(&env);
        let player = Address::generate(&env);
        let question = String::from_str(&env, "Q");
        let answer = String::from_str(&env, "a");

        let err = with_core_contract(&env, |env, _cid| {
            let hunt_id = HuntyCore::create_hunt(
                env.clone(),
                creator.clone(),
                String::from_str(env, "Hunt"),
                String::from_str(env, "Desc"),
                None,
                None,
            )
            .unwrap();
            HuntyCore::add_clue(env.clone(), hunt_id, question, answer, 1, false).unwrap();
            HuntyCore::activate_hunt(env.clone(), hunt_id, creator.clone()).unwrap();
            // Player never registered
            HuntyCore::get_player_progress(env.clone(), hunt_id, player.clone()).unwrap_err()
        });

        assert_eq!(err, HuntErrorCode::PlayerNotRegistered);
    }

    // ========== Player Progress Query Tests ==========

    #[test]
    fn test_get_player_progress_returns_state_after_submit() {
        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);
        let contract_id = env.register_contract(None, HuntyCore);
        let creator = Address::generate(&env);
        let player = Address::generate(&env);
        let question = String::from_str(&env, "Q1");
        let answer = String::from_str(&env, "a");

        let hunt_id = as_core_contract(&env, &contract_id, |env| {
            HuntyCore::create_hunt(
                env.clone(),
                creator.clone(),
                String::from_str(env, "Hunt"),
                String::from_str(env, "Desc"),
                None,
                None,
            )
            .unwrap()
        });
        env.mock_all_auths();
        as_core_contract(&env, &contract_id, |env| {
            HuntyCore::add_clue(
                env.clone(),
                hunt_id,
                question.clone(),
                answer.clone(),
                10,
                true,
            )
            .unwrap();
            HuntyCore::activate_hunt(env.clone(), hunt_id, creator.clone()).unwrap();
        });
        env.mock_all_auths();
        as_core_contract(&env, &contract_id, |env| {
            HuntyCore::register_player(env.clone(), hunt_id, player.clone()).unwrap();
        });
        env.mock_all_auths();
        as_core_contract(&env, &contract_id, |env| {
            HuntyCore::submit_answer(
                env.clone(),
                hunt_id,
                1,
                player.clone(),
                answer.clone(),
            )
            .unwrap();
        });
        let progress = as_core_contract(&env, &contract_id, |env| {
            HuntyCore::get_player_progress(env.clone(), hunt_id, player.clone()).unwrap()
        });
        assert_eq!(progress.player, player);
        assert_eq!(progress.hunt_id, hunt_id);
        assert_eq!(progress.completed_clues.len(), 1);
        assert_eq!(progress.total_score, 10);
        assert!(progress.is_completed);
        assert!(progress.completed_at > 0);
    }

    #[test]
    fn test_get_completed_clues_empty_when_not_registered() {
        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);
        env.mock_all_auths();

        let creator = Address::generate(&env);
        let player = Address::generate(&env);
        let question = String::from_str(&env, "Q");
        let answer = String::from_str(&env, "a");

        let list = with_core_contract(&env, |env, _cid| {
            let hunt_id = HuntyCore::create_hunt(
                env.clone(),
                creator.clone(),
                String::from_str(env, "Hunt"),
                String::from_str(env, "Desc"),
                None,
                None,
            )
            .unwrap();
            HuntyCore::add_clue(env.clone(), hunt_id, question, answer, 1, false).unwrap();
            HuntyCore::activate_hunt(env.clone(), hunt_id, creator.clone()).unwrap();
            HuntyCore::get_completed_clues(env.clone(), hunt_id, player.clone())
        });

        assert_eq!(list.len(), 0);
    }

    #[test]
    fn test_get_completed_clues_returns_ids_after_submit() {
        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);

        let creator = Address::generate(&env);
        let player = Address::generate(&env);
        let q1 = String::from_str(&env, "Q1");
        let q2 = String::from_str(&env, "Q2");
        let a = String::from_str(&env, "a");

        let contract_id = env.register_contract(None, HuntyCore);
        let hunt_id = as_core_contract(&env, &contract_id, |env| {
            HuntyCore::create_hunt(
                env.clone(),
                creator.clone(),
                String::from_str(env, "Hunt"),
                String::from_str(env, "Desc"),
                None,
                None,
            )
            .unwrap()
        });
        env.mock_all_auths();
        as_core_contract(&env, &contract_id, |env| {
            HuntyCore::add_clue(env.clone(), hunt_id, q1, a.clone(), 5, false).unwrap();
        });
        env.mock_all_auths();
        as_core_contract(&env, &contract_id, |env| {
            HuntyCore::add_clue(env.clone(), hunt_id, q2.clone(), a.clone(), 10, false).unwrap();
            HuntyCore::activate_hunt(env.clone(), hunt_id, creator.clone()).unwrap();
        });
        env.mock_all_auths();
        as_core_contract(&env, &contract_id, |env| {
            HuntyCore::register_player(env.clone(), hunt_id, player.clone()).unwrap();
        });
        env.mock_all_auths();
        as_core_contract(&env, &contract_id, |env| {
            HuntyCore::submit_answer(env.clone(), hunt_id, 1, player.clone(), a.clone())
                .unwrap();
        });
        env.mock_all_auths();
        as_core_contract(&env, &contract_id, |env| {
            HuntyCore::submit_answer(env.clone(), hunt_id, 2, player.clone(), a).unwrap();
        });
        let list = as_core_contract(&env, &contract_id, |env| {
            HuntyCore::get_completed_clues(env.clone(), hunt_id, player.clone())
        });

        assert_eq!(list.len(), 2);
        assert_eq!(list.get(0).unwrap(), 1);
        assert_eq!(list.get(1).unwrap(), 2);
    }

    #[test]
    fn test_get_hunt_leaderboard_hunt_not_found() {
        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);

        let err = with_core_contract(&env, |env, _cid| {
            HuntyCore::get_hunt_leaderboard(env.clone(), 9999, 10).unwrap_err()
        });

        assert_eq!(err, HuntErrorCode::HuntNotFound);
    }

    #[test]
    fn test_get_hunt_leaderboard_empty() {
        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);
        env.mock_all_auths();

        let creator = Address::generate(&env);
        let question = String::from_str(&env, "Q");
        let answer = String::from_str(&env, "a");

        let board = with_core_contract(&env, |env, _cid| {
            let hunt_id = HuntyCore::create_hunt(
                env.clone(),
                creator.clone(),
                String::from_str(env, "Hunt"),
                String::from_str(env, "Desc"),
                None,
                None,
            )
            .unwrap();
            HuntyCore::add_clue(env.clone(), hunt_id, question, answer, 1, false).unwrap();
            HuntyCore::activate_hunt(env.clone(), hunt_id, creator.clone()).unwrap();
            HuntyCore::get_hunt_leaderboard(env.clone(), hunt_id, 10).unwrap()
        });

        assert_eq!(board.len(), 0);
    }

    #[test]
    fn test_get_hunt_leaderboard_sorted_by_score_then_completion_time() {
        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);

        let creator = Address::generate(&env);
        let player_a = Address::generate(&env);
        let player_b = Address::generate(&env);
        let player_c = Address::generate(&env);
        let question = String::from_str(&env, "Q");
        let answer = String::from_str(&env, "a");

        let contract_id = env.register_contract(None, HuntyCore);
        let hunt_id = as_core_contract(&env, &contract_id, |env| {
            HuntyCore::create_hunt(
                env.clone(),
                creator.clone(),
                String::from_str(env, "Hunt"),
                String::from_str(env, "Desc"),
                None,
                None,
            )
            .unwrap()
        });
        env.mock_all_auths();
        as_core_contract(&env, &contract_id, |env| {
            HuntyCore::add_clue(
                env.clone(),
                hunt_id,
                question.clone(),
                answer.clone(),
                10,
                false,
            )
            .unwrap();
        });
        env.mock_all_auths();
        as_core_contract(&env, &contract_id, |env| {
            HuntyCore::add_clue(
                env.clone(),
                hunt_id,
                question.clone(),
                answer.clone(),
                5,
                false,
            )
            .unwrap();
            HuntyCore::activate_hunt(env.clone(), hunt_id, creator.clone()).unwrap();
        });
        env.mock_all_auths();
        as_core_contract(&env, &contract_id, |env| {
            HuntyCore::register_player(env.clone(), hunt_id, player_a.clone()).unwrap();
        });
        env.mock_all_auths();
        as_core_contract(&env, &contract_id, |env| {
            HuntyCore::register_player(env.clone(), hunt_id, player_b.clone()).unwrap();
        });
        env.mock_all_auths();
        as_core_contract(&env, &contract_id, |env| {
            HuntyCore::register_player(env.clone(), hunt_id, player_c.clone()).unwrap();
        });
        env.ledger().set_timestamp(1_700_000_001);
        env.mock_all_auths();
        as_core_contract(&env, &contract_id, |env| {
            HuntyCore::submit_answer(
                env.clone(),
                hunt_id,
                1,
                player_b.clone(),
                answer.clone(),
            )
            .unwrap();
        });
        env.mock_all_auths();
        as_core_contract(&env, &contract_id, |env| {
            HuntyCore::submit_answer(
                env.clone(),
                hunt_id,
                2,
                player_b.clone(),
                answer.clone(),
            )
            .unwrap();
        });
        env.ledger().set_timestamp(1_700_000_002);
        env.mock_all_auths();
        as_core_contract(&env, &contract_id, |env| {
            HuntyCore::submit_answer(
                env.clone(),
                hunt_id,
                1,
                player_a.clone(),
                answer.clone(),
            )
            .unwrap();
        });
        env.mock_all_auths();
        as_core_contract(&env, &contract_id, |env| {
            HuntyCore::submit_answer(
                env.clone(),
                hunt_id,
                2,
                player_a.clone(),
                answer.clone(),
            )
            .unwrap();
        });
        env.ledger().set_timestamp(1_700_000_003);
        env.mock_all_auths();
        as_core_contract(&env, &contract_id, |env| {
            HuntyCore::submit_answer(
                env.clone(),
                hunt_id,
                1,
                player_c.clone(),
                answer.clone(),
            )
            .unwrap();
        });
        let board = as_core_contract(&env, &contract_id, |env| {
            HuntyCore::get_hunt_leaderboard(env.clone(), hunt_id, 10).unwrap()
        });

        let e1 = board.get(0).unwrap();
        let e2 = board.get(1).unwrap();
        let e3 = board.get(2).unwrap();
        assert_eq!(board.len(), 3);
        assert_eq!(e1.rank, 1);
        assert_eq!(e2.rank, 2);
        assert_eq!(e3.rank, 3);
        assert_eq!(e1.score, 15);
        assert_eq!(e2.score, 15);
        assert_eq!(e3.score, 10);
        assert_eq!(e1.player, player_b);
        assert_eq!(e2.player, player_a);
        assert_eq!(e3.player, player_c);
        assert!(e1.completed_at < e2.completed_at);
    }

    #[test]
    fn test_get_hunt_leaderboard_limit_capped() {
        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);
        env.mock_all_auths();

        let creator = Address::generate(&env);
        let question = String::from_str(&env, "Q");
        let answer = String::from_str(&env, "a");

        let board = with_core_contract(&env, |env, _cid| {
            let hunt_id = HuntyCore::create_hunt(
                env.clone(),
                creator.clone(),
                String::from_str(env, "Hunt"),
                String::from_str(env, "Desc"),
                None,
                None,
            )
            .unwrap();
            HuntyCore::add_clue(env.clone(), hunt_id, question.clone(), answer.clone(), 1, false)
                .unwrap();
            HuntyCore::activate_hunt(env.clone(), hunt_id, creator.clone()).unwrap();
            let mut players = Vec::new(env);
            for _ in 0..5 {
                players.push_back(Address::generate(env));
            }
            for i in 0..5 {
                let p = players.get(i).unwrap();
                HuntyCore::register_player(env.clone(), hunt_id, p.clone()).unwrap();
            }
            HuntyCore::get_hunt_leaderboard(env.clone(), hunt_id, 2).unwrap()
        });

        assert_eq!(board.len(), 2);
        assert_eq!(board.get(0).unwrap().rank, 1);
        assert_eq!(board.get(1).unwrap().rank, 2);
    }

    #[test]
    fn test_get_hunt_statistics_hunt_not_found() {
        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);

        let err = with_core_contract(&env, |env, _cid| {
            HuntyCore::get_hunt_statistics(env.clone(), 9999).unwrap_err()
        });

        assert_eq!(err, HuntErrorCode::HuntNotFound);
    }

    #[test]
    fn test_get_hunt_statistics_empty_players() {
        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);
        env.mock_all_auths();

        let creator = Address::generate(&env);
        let question = String::from_str(&env, "Q");
        let answer = String::from_str(&env, "a");

        let stats = with_core_contract(&env, |env, _cid| {
            let hunt_id = HuntyCore::create_hunt(
                env.clone(),
                creator.clone(),
                String::from_str(env, "Hunt"),
                String::from_str(env, "Desc"),
                None,
                None,
            )
            .unwrap();
            HuntyCore::add_clue(env.clone(), hunt_id, question, answer, 1, false).unwrap();
            HuntyCore::activate_hunt(env.clone(), hunt_id, creator.clone()).unwrap();
            HuntyCore::get_hunt_statistics(env.clone(), hunt_id).unwrap()
        });

        assert_eq!(stats.total_players, 0);
        assert_eq!(stats.completed_count, 0);
        assert_eq!(stats.completion_rate_percent, 0);
        assert_eq!(stats.total_score_sum, 0);
        assert_eq!(stats.average_score, 0);
    }

    #[test]
    fn test_get_hunt_statistics_aggregates_correctly() {
        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);

        let creator = Address::generate(&env);
        let player1 = Address::generate(&env);
        let player2 = Address::generate(&env);
        let player3 = Address::generate(&env);
        let question = String::from_str(&env, "Q");
        let answer = String::from_str(&env, "a");

        let contract_id = env.register_contract(None, HuntyCore);
        let hunt_id = as_core_contract(&env, &contract_id, |env| {
            HuntyCore::create_hunt(
                env.clone(),
                creator.clone(),
                String::from_str(env, "Hunt"),
                String::from_str(env, "Desc"),
                None,
                None,
            )
            .unwrap()
        });
        env.mock_all_auths();
        as_core_contract(&env, &contract_id, |env| {
            HuntyCore::add_clue(
                env.clone(),
                hunt_id,
                question.clone(),
                answer.clone(),
                10,
                true,
            )
            .unwrap();
            HuntyCore::activate_hunt(env.clone(), hunt_id, creator.clone()).unwrap();
        });
        env.mock_all_auths();
        as_core_contract(&env, &contract_id, |env| {
            HuntyCore::register_player(env.clone(), hunt_id, player1.clone()).unwrap();
        });
        env.mock_all_auths();
        as_core_contract(&env, &contract_id, |env| {
            HuntyCore::register_player(env.clone(), hunt_id, player2.clone()).unwrap();
        });
        env.mock_all_auths();
        as_core_contract(&env, &contract_id, |env| {
            HuntyCore::register_player(env.clone(), hunt_id, player3.clone()).unwrap();
        });
        env.mock_all_auths();
        as_core_contract(&env, &contract_id, |env| {
            HuntyCore::submit_answer(
                env.clone(),
                hunt_id,
                1,
                player1.clone(),
                answer.clone(),
            )
            .unwrap();
        });
        env.mock_all_auths();
        as_core_contract(&env, &contract_id, |env| {
            HuntyCore::submit_answer(
                env.clone(),
                hunt_id,
                1,
                player2.clone(),
                answer.clone(),
            )
            .unwrap();
        });
        let stats = as_core_contract(&env, &contract_id, |env| {
            HuntyCore::get_hunt_statistics(env.clone(), hunt_id).unwrap()
        });

        assert_eq!(stats.total_players, 3);
        assert_eq!(stats.completed_count, 2);
        assert_eq!(stats.completion_rate_percent, 66);
        assert_eq!(stats.total_score_sum, 20);
        assert_eq!(stats.average_score, 6);
    }

    // ========== complete_hunt() Tests ==========

    /// Helper: creates a hunt, adds a required clue, activates, registers a player,
    /// submits the correct answer, and configures rewards. Returns (hunt_id, contract_id).
    fn setup_completed_hunt_with_rewards(
        env: &Env,
        creator: &Address,
        player: &Address,
        max_winners: u32,
        xlm_pool: i128,
    ) -> (u64, Address) {
        let contract_id = env.register_contract(None, HuntyCore);
        let question = String::from_str(env, "What is 1+1?");
        let answer = String::from_str(env, "2");

        // Create hunt
        let hunt_id = as_core_contract(env, &contract_id, |env| {
            HuntyCore::create_hunt(
                env.clone(),
                creator.clone(),
                String::from_str(env, "Reward Hunt"),
                String::from_str(env, "A hunt with rewards"),
                None,
                None,
            )
            .unwrap()
        });

        // Add clue and activate
        env.mock_all_auths();
        as_core_contract(env, &contract_id, |env| {
            HuntyCore::add_clue(
                env.clone(),
                hunt_id,
                question.clone(),
                answer.clone(),
                10,
                true,
            )
            .unwrap();

            // Update reward config on the hunt
            let mut hunt = Storage::get_hunt(env, hunt_id).unwrap();
            hunt.reward_config = crate::types::RewardConfig::new(
                xlm_pool,
                false,
                None,
                max_winners,
            );
            Storage::save_hunt(env, &hunt);

            HuntyCore::activate_hunt(env.clone(), hunt_id, creator.clone()).unwrap();
        });

        // Register player
        env.mock_all_auths();
        as_core_contract(env, &contract_id, |env| {
            HuntyCore::register_player(env.clone(), hunt_id, player.clone()).unwrap();
        });

        // Submit correct answer (triggers is_completed = true)
        env.mock_all_auths();
        as_core_contract(env, &contract_id, |env| {
            HuntyCore::submit_answer(
                env.clone(),
                hunt_id,
                1,
                player.clone(),
                answer.clone(),
            )
            .unwrap();
        });

        (hunt_id, contract_id)
    }

    // ========== Cross-Contract Integration Tests ==========

    #[test]
    fn test_complete_hunt_with_reward_manager_and_nft_reward_full_flow() {
        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);
        env.mock_all_auths();

        let creator = Address::generate(&env);
        let player = Address::generate(&env);
        let funder = Address::generate(&env);

        // Register contracts
        let core_id = env.register_contract(None, HuntyCore);
        let nft_contract_id = env.register_contract(None, NftReward);

        // Setup RewardManager with XLM token and default NFT contract
        let (reward_manager_id, token_address, token_admin) =
            setup_reward_manager(&env, Some(&nft_contract_id));

        // Mint XLM to funder
        let sac_client = token::StellarAssetClient::new(&env, &token_address);
        sac_client.mint(&funder, &10_000);

        // Create hunt, add required clue, configure rewards, activate, register player, complete clues
        let hunt_id = as_core_contract(&env, &core_id, |env| {
            let hunt_id = HuntyCore::create_hunt(
                env.clone(),
                creator.clone(),
                SorobanString::from_str(env, "Integrated Hunt"),
                SorobanString::from_str(env, "Hunt with XLM + NFT rewards"),
                None,
                None,
            )
            .unwrap();

            HuntyCore::add_clue(
                env.clone(),
                hunt_id,
                SorobanString::from_str(env, "What is 1+1?"),
                SorobanString::from_str(env, "2"),
                10,
                true,
            )
            .unwrap();

            // Configure rewards on the hunt: 3 winners sharing 9_000 XLM
            let mut hunt = Storage::get_hunt(env, hunt_id).unwrap();
            hunt.reward_config = crate::types::RewardConfig::new(
                9_000,
                true,
                Some(nft_contract_id.clone()),
                3,
            );
            Storage::save_hunt(env, &hunt);

            HuntyCore::activate_hunt(env.clone(), hunt_id, creator.clone()).unwrap();

            hunt_id
        });

        // Fund RewardManager pool for this hunt
        env.as_contract(&reward_manager_id, || {
            RewardManager::fund_reward_pool(env.clone(), funder.clone(), hunt_id, 9_000).unwrap();
        });

        // Wire HuntyCore -> RewardManager
        env.mock_all_auths();
        as_core_contract(&env, &core_id, |env| {
            HuntyCore::set_reward_manager(env.clone(), reward_manager_id.clone());
        });

        // Register player and complete hunt
        env.mock_all_auths();
        as_core_contract(&env, &core_id, |env| {
            HuntyCore::register_player(env.clone(), hunt_id, player.clone()).unwrap();
        });
        env.mock_all_auths();
        as_core_contract(&env, &core_id, |env| {
            HuntyCore::submit_answer(
                env.clone(),
                hunt_id,
                1,
                player.clone(),
                SorobanString::from_str(env, "2"),
            )
            .unwrap();
        });

        // Player claims completion and triggers cross-contract reward distribution
        env.mock_all_auths();
        as_core_contract(&env, &core_id, |env| {
            HuntyCore::complete_hunt(env.clone(), hunt_id, player.clone()).unwrap();
        });

        // Verify player progress updated in HuntyCore
        let progress = as_core_contract(&env, &core_id, |env| {
            HuntyCore::get_player_progress(env.clone(), hunt_id, player.clone()).unwrap()
        });
        assert!(progress.reward_claimed);

        // Verify hunt claimed_count incremented
        let hunt = as_core_contract(&env, &core_id, |env| {
            HuntyCore::get_hunt_info(env.clone(), hunt_id).unwrap()
        });
        assert_eq!(hunt.reward_config.claimed_count, 1);

        // Verify RewardManager XLM pool and balances
        let rm_balance = {
            let client = token::Client::new(&env, &token_address);
            client.balance(&reward_manager_id)
        };
        let player_balance = {
            let client = token::Client::new(&env, &token_address);
            client.balance(&player)
        };

        // reward_per_winner = 9_000 / 3 = 3_000
        assert_eq!(player_balance, 3_000);

        env.as_contract(&reward_manager_id, || {
            assert_eq!(RewardManager::get_pool_balance(env.clone(), hunt_id), 6_000);
        });
        assert_eq!(rm_balance, 6_000);

        // Verify RewardManager distribution status (includes NFT id)
        let status = env.as_contract(&reward_manager_id, || {
            RewardManager::get_distribution_status(env.clone(), hunt_id, player.clone())
        });
        assert!(status.distributed);
        assert_eq!(status.xlm_amount, 3_000);
        assert!(status.nft_id.is_some());

        // Verify NFT was minted to the player with correct metadata
        let minted_nft_id = status.nft_id.unwrap();
        let nft_client =
            nft_reward::NftRewardClient::new(&env, &nft_contract_id);
        let owned_nfts = nft_client.get_player_nfts(&player);
        assert!(owned_nfts.len() >= 1);
        assert!(owned_nfts.iter().any(|id| id == minted_nft_id));

        let nft = nft_client.get_nft(&minted_nft_id).unwrap();
        assert_eq!(nft.hunt_id, hunt_id);
        assert_eq!(nft.owner, player);
        assert_eq!(
            nft.metadata.title,
            SorobanString::from_str(&env, "Integrated Hunt")
        );
    }

    #[test]
    fn test_complete_hunt_reward_manager_failure_is_propagated() {
        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);
        env.mock_all_auths();

        let creator = Address::generate(&env);
        let player = Address::generate(&env);

        // Create a completed hunt with rewards configured (but no RewardManager funding/initialization)
        let (hunt_id, core_id) =
            setup_completed_hunt_with_rewards(&env, &creator, &player, 5, 1_000);

        // Deploy RewardManager but DO NOT call initialize or fund_reward_pool so distribution fails
        let reward_manager_id = env.register(RewardManager, ());

        // Wire HuntyCore -> RewardManager
        env.mock_all_auths();
        as_core_contract(&env, &core_id, |env| {
            HuntyCore::set_reward_manager(env.clone(), reward_manager_id.clone());
        });

        // Attempt to complete hunt - RewardManager::distribute_rewards should fail
        env.mock_all_auths();
        let result = as_core_contract(&env, &core_id, |env| {
            HuntyCore::complete_hunt(env.clone(), hunt_id, player.clone())
        });

        // HuntyCore must surface a generic RewardDistributionFailed error
        assert_eq!(result, Err(HuntErrorCode::RewardDistributionFailed));
    }

    #[test]
    fn test_complete_hunt_multiple_players_shared_reward_manager() {
        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);
        env.mock_all_auths();

        let creator = Address::generate(&env);
        let player1 = Address::generate(&env);
        let player2 = Address::generate(&env);
        let player3 = Address::generate(&env);
        let funder = Address::generate(&env);

        // Register contracts
        let core_id = env.register_contract(None, HuntyCore);
        let nft_contract_id = env.register_contract(None, NftReward);

        // Setup RewardManager with XLM token and default NFT contract
        let (reward_manager_id, token_address, _) =
            setup_reward_manager(&env, Some(&nft_contract_id));

        // Mint XLM to funder: 3 players * 2_000 each = 6_000
        let sac_client = token::StellarAssetClient::new(&env, &token_address);
        sac_client.mint(&funder, &6_000);

        // Create hunt, add required clue, configure rewards, activate
        let hunt_id = as_core_contract(&env, &core_id, |env| {
            let hunt_id = HuntyCore::create_hunt(
                env.clone(),
                creator.clone(),
                SorobanString::from_str(env, "Multi Hunt"),
                SorobanString::from_str(env, "Multiple winners"),
                None,
                None,
            )
            .unwrap();

            HuntyCore::add_clue(
                env.clone(),
                hunt_id,
                SorobanString::from_str(env, "What is 1+1?"),
                SorobanString::from_str(env, "2"),
                10,
                true,
            )
            .unwrap();

            // Configure rewards: xlm_pool = 6_000, max_winners = 3
            let mut hunt = Storage::get_hunt(env, hunt_id).unwrap();
            hunt.reward_config = crate::types::RewardConfig::new(
                6_000,
                true,
                Some(nft_contract_id.clone()),
                3,
            );
            Storage::save_hunt(env, &hunt);

            HuntyCore::activate_hunt(env.clone(), hunt_id, creator.clone()).unwrap();

            hunt_id
        });

        // Fund RewardManager pool
        env.as_contract(&reward_manager_id, || {
            RewardManager::fund_reward_pool(env.clone(), funder.clone(), hunt_id, 6_000).unwrap();
        });

        // Wire HuntyCore -> RewardManager
        env.mock_all_auths();
        as_core_contract(&env, &core_id, |env| {
            HuntyCore::set_reward_manager(env.clone(), reward_manager_id.clone());
        });

        // Helper closure to register, answer, and claim for a player
        let claim_for = |env: &Env, player: &Address| {
            env.mock_all_auths();
            as_core_contract(env, &core_id, |env| {
                HuntyCore::register_player(env.clone(), hunt_id, player.clone()).unwrap();
            });
            env.mock_all_auths();
            as_core_contract(env, &core_id, |env| {
                HuntyCore::submit_answer(
                    env.clone(),
                    hunt_id,
                    1,
                    player.clone(),
                    SorobanString::from_str(env, "2"),
                )
                .unwrap();
            });
            env.mock_all_auths();
            as_core_contract(env, &core_id, |env| {
                HuntyCore::complete_hunt(env.clone(), hunt_id, player.clone()).unwrap();
            });
        };

        // Three players complete and claim
        claim_for(&env, &player1);
        claim_for(&env, &player2);
        claim_for(&env, &player3);

        // Each winner should have received 2_000 XLM and one NFT
        let token_client = token::Client::new(&env, &token_address);
        assert_eq!(token_client.balance(&player1), 2_000);
        assert_eq!(token_client.balance(&player2), 2_000);
        assert_eq!(token_client.balance(&player3), 2_000);

        // Pool should now be empty for this hunt
        env.as_contract(&reward_manager_id, || {
            assert_eq!(RewardManager::get_pool_balance(env.clone(), hunt_id), 0);
        });

        let nft_client = nft_reward::NftRewardClient::new(&env, &nft_contract_id);
        let nfts1 = nft_client.get_player_nfts(&player1);
        let nfts2 = nft_client.get_player_nfts(&player2);
        let nfts3 = nft_client.get_player_nfts(&player3);
        assert!(nfts1.len() >= 1);
        assert!(nfts2.len() >= 1);
        assert!(nfts3.len() >= 1);

        // HuntyCore claimed_count should be 3
        let hunt = as_core_contract(&env, &core_id, |env| {
            HuntyCore::get_hunt_info(env.clone(), hunt_id).unwrap()
        });
        assert_eq!(hunt.reward_config.claimed_count, 3);
    }

    #[test]
    fn test_complete_hunt_success_no_reward_manager() {
        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);
        let creator = Address::generate(&env);
        let player = Address::generate(&env);

        let (hunt_id, contract_id) =
            setup_completed_hunt_with_rewards(&env, &creator, &player, 5, 1000);

        // Complete hunt (no RewardManager set — should still succeed)
        env.mock_all_auths();
        as_core_contract(&env, &contract_id, |env| {
            HuntyCore::complete_hunt(env.clone(), hunt_id, player.clone()).unwrap();
        });

        // Verify progress updated
        let progress = as_core_contract(&env, &contract_id, |env| {
            HuntyCore::get_player_progress(env.clone(), hunt_id, player.clone()).unwrap()
        });
        assert!(progress.reward_claimed);

        // Verify hunt claimed_count incremented
        let hunt = as_core_contract(&env, &contract_id, |env| {
            HuntyCore::get_hunt_info(env.clone(), hunt_id).unwrap()
        });
        assert_eq!(hunt.reward_config.claimed_count, 1);
    }

    #[test]
    fn test_complete_hunt_not_completed() {
        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);
        let creator = Address::generate(&env);
        let player = Address::generate(&env);
        let contract_id = env.register_contract(None, HuntyCore);

        // Create hunt with 2 required clues
        let hunt_id = as_core_contract(&env, &contract_id, |env| {
            HuntyCore::create_hunt(
                env.clone(),
                creator.clone(),
                String::from_str(env, "Hunt"),
                String::from_str(env, "Desc"),
                None,
                None,
            )
            .unwrap()
        });

        env.mock_all_auths();
        as_core_contract(&env, &contract_id, |env| {
            HuntyCore::add_clue(
                env.clone(),
                hunt_id,
                String::from_str(env, "Q1"),
                String::from_str(env, "a1"),
                10,
                true,
            )
            .unwrap();
            HuntyCore::add_clue(
                env.clone(),
                hunt_id,
                String::from_str(env, "Q2"),
                String::from_str(env, "a2"),
                10,
                true,
            )
            .unwrap();

            let mut hunt = Storage::get_hunt(env, hunt_id).unwrap();
            hunt.reward_config =
                crate::types::RewardConfig::new(1000, false, None, 5);
            Storage::save_hunt(env, &hunt);

            HuntyCore::activate_hunt(env.clone(), hunt_id, creator.clone()).unwrap();
        });

        // Register and answer only 1 of 2 required clues
        env.mock_all_auths();
        as_core_contract(&env, &contract_id, |env| {
            HuntyCore::register_player(env.clone(), hunt_id, player.clone()).unwrap();
        });
        env.mock_all_auths();
        as_core_contract(&env, &contract_id, |env| {
            HuntyCore::submit_answer(
                env.clone(),
                hunt_id,
                1,
                player.clone(),
                String::from_str(env, "a1"),
            )
            .unwrap();
        });

        // Try to complete — should fail
        env.mock_all_auths();
        let result = as_core_contract(&env, &contract_id, |env| {
            HuntyCore::complete_hunt(env.clone(), hunt_id, player.clone())
        });
        assert_eq!(result, Err(HuntErrorCode::HuntNotCompleted));
    }

    #[test]
    fn test_complete_hunt_double_claim() {
        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);
        let creator = Address::generate(&env);
        let player = Address::generate(&env);

        let (hunt_id, contract_id) =
            setup_completed_hunt_with_rewards(&env, &creator, &player, 5, 1000);

        // First claim — success
        env.mock_all_auths();
        as_core_contract(&env, &contract_id, |env| {
            HuntyCore::complete_hunt(env.clone(), hunt_id, player.clone()).unwrap();
        });

        // Second claim — should fail
        env.mock_all_auths();
        let result = as_core_contract(&env, &contract_id, |env| {
            HuntyCore::complete_hunt(env.clone(), hunt_id, player.clone())
        });
        assert_eq!(result, Err(HuntErrorCode::RewardAlreadyClaimed));
    }

    #[test]
    fn test_complete_hunt_max_winners_reached() {
        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);
        let creator = Address::generate(&env);
        let player1 = Address::generate(&env);
        let player2 = Address::generate(&env);

        // max_winners = 1
        let (hunt_id, contract_id) =
            setup_completed_hunt_with_rewards(&env, &creator, &player1, 1, 1000);

        // Player1 claims successfully
        env.mock_all_auths();
        as_core_contract(&env, &contract_id, |env| {
            HuntyCore::complete_hunt(env.clone(), hunt_id, player1.clone()).unwrap();
        });

        // Register and complete for player2
        env.mock_all_auths();
        as_core_contract(&env, &contract_id, |env| {
            HuntyCore::register_player(env.clone(), hunt_id, player2.clone()).unwrap();
        });
        env.mock_all_auths();
        as_core_contract(&env, &contract_id, |env| {
            HuntyCore::submit_answer(
                env.clone(),
                hunt_id,
                1,
                player2.clone(),
                String::from_str(env, "2"),
            )
            .unwrap();
        });

        // Player2 tries to claim — no slots left
        env.mock_all_auths();
        let result = as_core_contract(&env, &contract_id, |env| {
            HuntyCore::complete_hunt(env.clone(), hunt_id, player2.clone())
        });
        assert_eq!(result, Err(HuntErrorCode::InsufficientRewardPool));
    }

    #[test]
    fn test_complete_hunt_no_rewards_configured() {
        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);
        let creator = Address::generate(&env);
        let player = Address::generate(&env);

        // max_winners = 0, xlm_pool = 0 (default from create_hunt)
        let (hunt_id, contract_id) =
            setup_completed_hunt_with_rewards(&env, &creator, &player, 0, 0);

        env.mock_all_auths();
        let result = as_core_contract(&env, &contract_id, |env| {
            HuntyCore::complete_hunt(env.clone(), hunt_id, player.clone())
        });
        assert_eq!(result, Err(HuntErrorCode::NoRewardsConfigured));
    }

    #[test]
    fn test_complete_hunt_player_not_registered() {
        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);
        let creator = Address::generate(&env);
        let player = Address::generate(&env);
        let stranger = Address::generate(&env);

        let (hunt_id, contract_id) =
            setup_completed_hunt_with_rewards(&env, &creator, &player, 5, 1000);

        env.mock_all_auths();
        let result = as_core_contract(&env, &contract_id, |env| {
            HuntyCore::complete_hunt(env.clone(), hunt_id, stranger.clone())
        });
        assert_eq!(result, Err(HuntErrorCode::PlayerNotRegistered));
    }
}
