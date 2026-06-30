use crate::HuntyCore;
use soroban_sdk::testutils::{Address as _, Ledger as _};
use soroban_sdk::{Address, Env, String};

/// Helper to execute contract operations within the contract context.
/// Wraps calls with `env.as_contract()` for proper storage isolation.
fn execute_in_contract<T, F>(env: &Env, contract_id: &Address, f: F) -> T
where
    F: FnOnce(&Env) -> T,
{
    env.as_contract(contract_id, || f(env))
}
#[cfg(test)]
extern crate std;

use std::string::ToString;

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::{Address, Env, IntoVal, String, Symbol, TryIntoVal, Vec};
    // Bring Soroban testutils traits into scope (generate addresses, set ledger info, register contracts).
    use crate::errors::{HuntError, HuntErrorCode};
    use crate::storage::Storage;
    use crate::types::{
        ClueAddedEvent, CreatorBlacklistedEvent, CreatorRemovedFromBlacklistEvent,
        HuntCompletedEvent, HuntCreatedEvent, HuntStatus, HuntStatusChangedEvent,
        PlayerRegisteredEvent, RewardClaimFailedEvent, TimeBonusConfig,
    };
    use crate::HuntyCore;
    use nft_reward::NftReward;
    use reward_manager::RewardManager;
    use soroban_sdk::testutils::{Address as _, Events as _, Ledger as _, Register as _};
    use soroban_sdk::{token, String as SorobanString, TryFromVal, Val};

    /// Runs a closure inside a registered HuntyCore contract context so storage is accessible.
    fn with_core_contract<T>(env: &Env, f: impl FnOnce(&Env, &Address) -> T) -> T {
        let contract_id = env.register_contract(None, super::HuntyCore);
        env.as_contract(&contract_id, || f(env, &contract_id))
    }

    fn find_hunt_status_changed_event(env: &Env) -> Option<HuntStatusChangedEvent> {
        let expected_topic = Symbol::new(env, "HuntStatusChanged").into_val(env);
        let events = env.events().all();
        let mut idx = 0;
        while idx < events.len() {
            let event = events.get(idx).unwrap();
            let topics = &event.1;
            if topics.len() > 0 {
                let topic = topics.get(0).unwrap();
                if *topic == expected_topic {
                    return HuntStatusChangedEvent::try_from_val(env, &event.2).ok();
                }
            }
            idx += 1;
        }
        None
    }

    fn find_event<T: TryFromVal<Env, Val>>(env: &Env, topic_name: &str) -> Option<(Vec<Val>, T)> {
        let expected_topic = Symbol::new(env, topic_name).into_val(env);
        let events = env.events().all();
        let mut idx = 0;
        while idx < events.len() {
            let event = events.get(idx).unwrap();
            let topics = event.1.clone();
            if topics.len() > 0 && topics.get(0).unwrap() == expected_topic {
                if let Ok(data) = T::try_from_val(env, &event.2) {
                    return Some((topics, data));
                }
            }
            idx += 1;
        }
        None
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
            RewardManager::initialize(env.clone(), token_admin.clone(), token_address.clone())
                .unwrap();
        });
        if let Some(nft) = nft_contract {
            env.mock_all_auths();
            env.as_contract(&reward_manager_id, || {
                RewardManager::set_nft_reward_contract(
                    env.clone(),
                    token_admin.clone(),
                    nft.clone(),
                )
                .unwrap();
            });
        }

        (reward_manager_id, token_address, token_admin)
    }

    #[test]
    fn test_error_with_context_display() {
        let err = HuntError::HuntNotFound { hunt_id: 42 };
        let hunt_error: HuntErrorCode = err.into();
        assert_eq!(hunt_error, HuntErrorCode::HuntNotFound)
    }

    #[test]
    fn test_all_error_codes_are_unique() {
        let mut seen = std::collections::BTreeSet::new();
        let variants: &[(HuntErrorCode, &str)] = &[
            (HuntErrorCode::HuntNotFound, "HuntNotFound"),
            (HuntErrorCode::ClueNotFound, "ClueNotFound"),
            (HuntErrorCode::InvalidHuntStatus, "InvalidHuntStatus"),
            (HuntErrorCode::PlayerNotRegistered, "PlayerNotRegistered"),
            (HuntErrorCode::ClueAlreadyCompleted, "ClueAlreadyCompleted"),
            (HuntErrorCode::InvalidAnswer, "InvalidAnswer"),
            (HuntErrorCode::HuntNotActive, "HuntNotActive"),
            (HuntErrorCode::Unauthorized, "Unauthorized"),
            (HuntErrorCode::InsufficientRewardPool, "InsufficientRewardPool"),
            (HuntErrorCode::DuplicateRegistration, "DuplicateRegistration"),
            (HuntErrorCode::InvalidTitle, "InvalidTitle"),
            (HuntErrorCode::InvalidDescription, "InvalidDescription"),
            (HuntErrorCode::InvalidAddress, "InvalidAddress"),
            (HuntErrorCode::TooManyClues, "TooManyClues"),
            (HuntErrorCode::InvalidQuestion, "InvalidQuestion"),
            (HuntErrorCode::RefundFailed, "RefundFailed"),
            (HuntErrorCode::NoCluesAdded, "NoCluesAdded"),
            (HuntErrorCode::HuntNotCompleted, "HuntNotCompleted"),
            (HuntErrorCode::RewardAlreadyClaimed, "RewardAlreadyClaimed"),
            (HuntErrorCode::RewardDistributionFailed, "RewardDistributionFailed"),
            (HuntErrorCode::NoRewardsConfigured, "NoRewardsConfigured"),
            (HuntErrorCode::DuplicateSubmission, "DuplicateSubmission"),
            (HuntErrorCode::SubmissionExpired, "SubmissionExpired"),
            (HuntErrorCode::BannedPlayer, "BannedPlayer"),
            (HuntErrorCode::NoRequiredClues, "NoRequiredClues"),
            (HuntErrorCode::RateLimitExceeded, "RateLimitExceeded"),
            (HuntErrorCode::ScoreOverflow, "ScoreOverflow"),
            (HuntErrorCode::RegistrationsPaused, "RegistrationsPaused"),
            (HuntErrorCode::AnswersPaused, "AnswersPaused"),
            (HuntErrorCode::RewardsPaused, "RewardsPaused"),
            (HuntErrorCode::HuntEndTimeInPast, "HuntEndTimeInPast"),
            (HuntErrorCode::NoPendingAdmin, "NoPendingAdmin"),
            (HuntErrorCode::PendingAdminMismatch, "PendingAdminMismatch"),
            (HuntErrorCode::InvalidRarity, "InvalidRarity"),
            (HuntErrorCode::InvalidTimeBonusConfig, "InvalidTimeBonusConfig"),
            (HuntErrorCode::AddressBlacklisted, "AddressBlacklisted"),
            (HuntErrorCode::ContractPaused, "ContractPaused"),
        ];
        for (variant, name) in variants {
            let code = *variant as u32;
            assert!(
                seen.insert(code),
                "Duplicate HuntErrorCode value {} for variant '{}'",
                code,
                name
            );
        }
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
    fn test_submit_answer_with_hash_works() {
        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);
        let creator = Address::generate(&env);
        let player1 = Address::generate(&env);
        let player2 = Address::generate(&env);
        let contract_id = env.register(HuntyCore, ());

        // Create hunt
        env.mock_all_auths();
        let hunt_id = as_core_contract(&env, &contract_id, |env| {
            HuntyCore::create_hunt(
                env.clone(),
                creator.clone(),
                String::from_str(env, "Hash Hunt"),
                String::from_str(env, "Test hashing paths"),
                None,
                None,
                0,
                None,
            )
        })
        .unwrap();

        // Add a clue with answer "Paris"
        env.mock_all_auths();
        let clue_id = as_core_contract(&env, &contract_id, |env| {
            HuntyCore::add_clue(
                env.clone(),
                hunt_id,
                String::from_str(env, "Capital of France?"),
                String::from_str(env, "Paris"),
                10,
                true,
                None,
            )
        })
        .unwrap();

        // Register two players
        env.as_contract(&contract_id, || {
            HuntyCore::register_player(env.clone(), hunt_id, player1.clone()).unwrap();
        });
        env.as_contract(&contract_id, || {
            HuntyCore::register_player(env.clone(), hunt_id, player2.clone()).unwrap();
        });

        // Submit plaintext answer for player1
        let res1 = env.as_contract(&contract_id, || {
            HuntyCore::submit_answer(
                env.clone(),
                hunt_id,
                clue_id,
                player1.clone(),
                String::from_str(&env, "Paris"),
                1,
                env.ledger().timestamp(),
            )
        });
        assert!(res1.is_ok());

        // Compute precomputed hash (uses same normalization helper) and submit for player2
        let pre_hash = HuntyCore::normalize_and_hash_answer(&env, hunt_id, clue_id, &String::from_str(&env, "Paris")).unwrap();
        let res2 = env.as_contract(&contract_id, || {
            HuntyCore::submit_answer_with_hash(
                env.clone(),
                hunt_id,
                clue_id,
                player2.clone(),
                pre_hash.clone(),
                1,
                env.ledger().timestamp(),
            )
        });
        assert!(res2.is_ok());
    }

    #[test]
    fn test_hunt_completion_ranks() {
        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);
        let creator = Address::generate(&env);
        let player1 = Address::generate(&env);
        let player2 = Address::generate(&env);
        let player3 = Address::generate(&env);
        let contract_id = env.register(HuntyCore, ());

        // Create hunt
        env.mock_all_auths();
        let hunt_id = as_core_contract(&env, &contract_id, |env| {
            HuntyCore::create_hunt(
                env.clone(),
                creator.clone(),
                String::from_str(env, "Rank Hunt"),
                String::from_str(env, "Test ranking"),
                None,
                None,
                0,
                None,
            )
        })
        .unwrap();

        let question = String::from_str(&env, "What is 2+2?");
        let answer = String::from_str(&env, "4");
        env.mock_all_auths();
        as_core_contract(&env, &contract_id, |env| {
            HuntyCore::add_clue(env.clone(), hunt_id, question.clone(), answer.clone(), 10, true, None).unwrap();
        });
        env.mock_all_auths();
        as_core_contract(&env, &contract_id, |env| {
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

        // Player1 completes
        env.mock_all_auths();
        as_core_contract(&env, &contract_id, |env| {
            submit_answer(env, hunt_id, 1, player1.clone(), answer.clone(), 1)
            .unwrap();
        });
        let board1 = as_core_contract(&env, &contract_id, |env| {
            HuntyCore::get_hunt_leaderboard(env.clone(), hunt_id, 10).unwrap()
        });
        let first = board1.get(0).unwrap();
        assert_eq!(first.player, player1);
        assert_eq!(first.rank, 1);
        assert!(first.is_completed);

        // Player2 completes
        env.mock_all_auths();
        as_core_contract(&env, &contract_id, |env| {
            submit_answer(env, hunt_id, 1, player2.clone(), answer.clone(), 2)
            .unwrap();
        });
        let board2 = as_core_contract(&env, &contract_id, |env| {
            HuntyCore::get_hunt_leaderboard(env.clone(), hunt_id, 10).unwrap()
        });
        let first_after_second = board2.get(0).unwrap();
        let second_after_second = board2.get(1).unwrap();
        assert_eq!(first_after_second.player, player1);
        assert_eq!(first_after_second.rank, 1);
        assert_eq!(second_after_second.player, player2);
        assert_eq!(second_after_second.rank, 2);
        assert!(second_after_second.is_completed);

        // Duplicate attempt by Player2 (should not emit new event)
        env.mock_all_auths();
        let dup_result = as_core_contract(&env, &contract_id, |env| {
            HuntyCore::submit_answer(
                env.clone(),
                hunt_id,
                1,
                player2.clone(),
                answer.clone(),
                2,
                env.ledger().timestamp(),
            )
        });
        assert_eq!(dup_result, Err(HuntErrorCode::DuplicateSubmission));
        let board_dup = as_core_contract(&env, &contract_id, |env| {
            HuntyCore::get_hunt_leaderboard(env.clone(), hunt_id, 10).unwrap()
        });
        let first_after_dup = board_dup.get(0).unwrap();
        let second_after_dup = board_dup.get(1).unwrap();
        assert_eq!(first_after_dup.player, player1);
        assert_eq!(first_after_dup.rank, 1);
        assert_eq!(second_after_dup.player, player2);
        assert_eq!(second_after_dup.rank, 2);
    }

    #[test]
    fn test_submit_answer_rejects_expired_submission_timestamp() {
        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);
        let creator = Address::generate(&env);
        let player = Address::generate(&env);
        let question = String::from_str(&env, "What is 2+2?");
        let answer = String::from_str(&env, "4");

        let contract_id = env.register(HuntyCore, ());
        env.mock_all_auths();
        let hunt_id = as_core_contract(&env, &contract_id, |env| {
            HuntyCore::create_hunt(
                env.clone(),
                creator.clone(),
                String::from_str(env, "Replay Hunt"),
                String::from_str(env, "Replay protection"),
                None,
                None,
                0,
                None,
            )
            .unwrap()
        });
        env.mock_all_auths();
        as_core_contract(&env, &contract_id, |env| {
            HuntyCore::add_clue(env.clone(), hunt_id, question.clone(), answer.clone(), 10, true, None)
                .unwrap();
        });
        env.mock_all_auths();
        as_core_contract(&env, &contract_id, |env| {
            HuntyCore::activate_hunt(env.clone(), hunt_id, creator.clone()).unwrap();
        });
        env.mock_all_auths();
        as_core_contract(&env, &contract_id, |env| {
            HuntyCore::register_player(env.clone(), hunt_id, player.clone()).unwrap();
        });

        env.mock_all_auths();
        as_core_contract(&env, &contract_id, |env| {
            let result = HuntyCore::submit_answer(
                env.clone(),
                hunt_id,
                1,
                player.clone(),
                answer.clone(),
                1,
                env.ledger().timestamp() - ANSWER_SUBMISSION_WINDOW_SECS - 1,
            );
            assert_eq!(result, Err(HuntErrorCode::SubmissionExpired));
        });
    }

    #[test]
    fn test_hunt_created_event_topics_and_data() {
        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);
        let creator = Address::generate(&env);
        let title = String::from_str(&env, "Indexed Hunt");

        with_core_contract(&env, |env, _cid| {
            env.mock_all_auths();
            let hunt_id = HuntyCore::create_hunt(
                env.clone(),
                creator.clone(),
                title.clone(),
                String::from_str(env, "Event payload coverage"),
                None,
                None,
                0,
                None,
            )
            .unwrap();

            let (topics, event) =
                find_event::<HuntCreatedEvent>(env, "HuntCreated").expect("missing HuntCreated");
            assert_eq!(topics.len(), 2);
            assert_eq!(topics.get(0).unwrap(), Symbol::new(env, "HuntCreated").into_val(env));
            assert_eq!(topics.get(1).unwrap(), hunt_id.into_val(env));
            assert_eq!(event.hunt_id, hunt_id);
            assert_eq!(event.creator, creator);
            assert_eq!(event.title, title);
        });
    }

    #[test]
    fn test_clue_added_event_topics_and_data() {
        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);
        let creator = Address::generate(&env);
        let question = String::from_str(&env, "What walks on four legs?");

        let contract_id = env.register(HuntyCore, ());
        env.mock_all_auths();
        let hunt_id = as_core_contract(&env, &contract_id, |env| {
            HuntyCore::create_hunt(
                env.clone(),
                creator.clone(),
                String::from_str(env, "Clue Event Hunt"),
                String::from_str(env, "Verifies indexed clue metadata"),
                None,
                None,
                0,
                None,
            )
            .unwrap()
        });

        env.mock_all_auths();
        as_core_contract(&env, &contract_id, |env| {
            let clue_id = HuntyCore::add_clue(
                env.clone(),
                hunt_id,
                question.clone(),
                String::from_str(env, "Human"),
                25,
                true,
                Some(3),
            )
            .unwrap();

            let (topics, event) =
                find_event::<ClueAddedEvent>(env, "ClueAdded").expect("missing ClueAdded");
            assert_eq!(topics.len(), 3);
            assert_eq!(topics.get(0).unwrap(), Symbol::new(env, "ClueAdded").into_val(env));
            assert_eq!(topics.get(1).unwrap(), hunt_id.into_val(env));
            assert_eq!(topics.get(2).unwrap(), clue_id.into_val(env));
            assert_eq!(event.hunt_id, hunt_id);
            assert_eq!(event.clue_id, clue_id);
            assert_eq!(event.creator, creator);
            assert_eq!(event.question, question);
            assert_eq!(event.points, 25);
            assert!(event.is_required);
        });
    }

    #[test]
    fn test_player_registered_event_topics_and_data() {
        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);
        let creator = Address::generate(&env);
        let player = Address::generate(&env);

        let contract_id = env.register(HuntyCore, ());
        env.mock_all_auths();
        let hunt_id = as_core_contract(&env, &contract_id, |env| {
            let hunt_id = HuntyCore::create_hunt(
                env.clone(),
                creator.clone(),
                String::from_str(env, "Registration Event Hunt"),
                String::from_str(env, "Verifies player registration indexing"),
                None,
                None,
                0,
                None,
            )
            .unwrap();
            HuntyCore::add_clue(
                env.clone(),
                hunt_id,
                String::from_str(env, "Q"),
                String::from_str(env, "A"),
                10,
                true,
                None,
            )
            .unwrap();
            HuntyCore::activate_hunt(env.clone(), hunt_id, creator.clone()).unwrap();
            hunt_id
        });

        env.mock_all_auths();
        as_core_contract(&env, &contract_id, |env| {
            HuntyCore::register_player(env.clone(), hunt_id, player.clone()).unwrap();

            let (topics, event) = find_event::<PlayerRegisteredEvent>(env, "PlayerRegistered")
                .expect("missing PlayerRegistered");
            assert_eq!(topics.len(), 2);
            assert_eq!(
                topics.get(0).unwrap(),
                Symbol::new(env, "PlayerRegistered").into_val(env)
            );
            assert_eq!(topics.get(1).unwrap(), hunt_id.into_val(env));
            assert_eq!(event.hunt_id, hunt_id);
            assert_eq!(event.player, player);
        });
    }

    #[test]
    fn test_processed_submission_tracking_expires_after_window() {
        let env = Env::default();
        let start_time = 1_700_000_000;
        env.ledger().set_timestamp(start_time);
        let creator = Address::generate(&env);
        let player = Address::generate(&env);
        let question = String::from_str(&env, "What is 2+2?");
        let answer = String::from_str(&env, "4");

        let contract_id = env.register(HuntyCore, ());
        env.mock_all_auths();
        let hunt_id = as_core_contract(&env, &contract_id, |env| {
            HuntyCore::create_hunt(
                env.clone(),
                creator.clone(),
                String::from_str(env, "Replay Hunt"),
                String::from_str(env, "Replay protection"),
                None,
                None,
                0,
                None,
            )
            .unwrap()
        });
        env.mock_all_auths();
        as_core_contract(&env, &contract_id, |env| {
            HuntyCore::add_clue(env.clone(), hunt_id, question.clone(), answer.clone(), 10, true, None)
                .unwrap();
        });
        env.mock_all_auths();
        as_core_contract(&env, &contract_id, |env| {
            HuntyCore::activate_hunt(env.clone(), hunt_id, creator.clone()).unwrap();
        });
        env.mock_all_auths();
        as_core_contract(&env, &contract_id, |env| {
            HuntyCore::register_player(env.clone(), hunt_id, player.clone()).unwrap();
        });

        env.mock_all_auths();
        as_core_contract(&env, &contract_id, |env| {
            let submitted_at = env.ledger().timestamp();
            HuntyCore::submit_answer(
                env.clone(),
                hunt_id,
                1,
                player.clone(),
                answer.clone(),
                7,
                submitted_at,
            )
            .unwrap();

            assert_eq!(
                Storage::get_processed_submission_expiry(
                    env,
                    hunt_id,
                    1,
                    &player,
                    7,
                    submitted_at,
                ),
                Some(submitted_at + ANSWER_SUBMISSION_WINDOW_SECS)
            );

            env.ledger()
                .set_timestamp(submitted_at + ANSWER_SUBMISSION_WINDOW_SECS + 1);
            HuntyCore::assert_submission_not_replayed(
                env,
                hunt_id,
                1,
                &player,
                7,
                submitted_at,
                env.ledger().timestamp(),
            )
            .unwrap();

            assert_eq!(
                Storage::get_processed_submission_expiry(
                    env,
                    hunt_id,
                    1,
                    &player,
                    7,
                    submitted_at,
                ),
                None
            );
        });
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
                0,
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
        assert_eq!(hunt.time_bonus_config(), None);
        assert!(hunt.created_at > 0);
        assert_eq!(hunt.activated_at, 0);
        assert_eq!(hunt.end_time, 0);
    }

    #[test]
    fn test_time_bonus_scoring_decreases_over_time() {
        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);

        let creator = Address::generate(&env);
        let player_fast = Address::generate(&env);
        let player_mid = Address::generate(&env);
        let player_slow = Address::generate(&env);
        let title = String::from_str(&env, "Time Bonus Hunt");
        let description = String::from_str(&env, "A hunt with a decaying score bonus");
        let question = String::from_str(&env, "What time is it?");
        let answer = String::from_str(&env, "now");
        let bonus = TimeBonusConfig {
            start_multiplier_bps: 20_000,
            min_multiplier_bps: 10_000,
            decay_duration_secs: 100,
        };

        let contract_id = env.register_contract(None, super::HuntyCore);
        let hunt_id = as_core_contract(&env, &contract_id, |env| {
            HuntyCore::create_hunt(
                env.clone(),
                creator.clone(),
                title.clone(),
                description.clone(),
                None,
                None,
            )
            .unwrap()
        });

        env.mock_all_auths();
        as_core_contract(&env, &contract_id, |env| {
            HuntyCore::set_time_bonus_config(
                env.clone(),
                hunt_id,
                creator.clone(),
                Some(bonus.clone()),
            )
            .unwrap();
            let hunt = Storage::get_hunt(env, hunt_id).unwrap();
            assert_eq!(hunt.time_bonus_config(), Some(bonus.clone()));
            HuntyCore::add_clue(
                env.clone(),
                hunt_id,
                question.clone(),
                answer.clone(),
                10,
                true, 1)
            .unwrap();
            HuntyCore::activate_hunt(env.clone(), hunt_id, creator.clone()).unwrap();
        });

        env.mock_all_auths();
        as_core_contract(&env, &contract_id, |env| {
            HuntyCore::register_player(env.clone(), hunt_id, player_fast.clone()).unwrap();
            HuntyCore::register_player(env.clone(), hunt_id, player_mid.clone()).unwrap();
            HuntyCore::register_player(env.clone(), hunt_id, player_slow.clone()).unwrap();
        });

        env.ledger().set_timestamp(1_700_000_000);
        env.mock_all_auths();
        as_core_contract(&env, &contract_id, |env| {
            HuntyCore::submit_answer(env.clone(), hunt_id, 1, player_fast.clone(), answer.clone())
                .unwrap();
        });

        env.ledger().set_timestamp(1_700_000_050);
        env.mock_all_auths();
        as_core_contract(&env, &contract_id, |env| {
            HuntyCore::submit_answer(env.clone(), hunt_id, 1, player_mid.clone(), answer.clone())
                .unwrap();
        });

        env.ledger().set_timestamp(1_700_000_100);
        env.mock_all_auths();
        as_core_contract(&env, &contract_id, |env| {
            HuntyCore::submit_answer(env.clone(), hunt_id, 1, player_slow.clone(), answer.clone())
                .unwrap();
        });

        let fast_progress = as_core_contract(&env, &contract_id, |env| {
            HuntyCore::get_player_progress(env.clone(), hunt_id, player_fast.clone()).unwrap()
        });
        let mid_progress = as_core_contract(&env, &contract_id, |env| {
            HuntyCore::get_player_progress(env.clone(), hunt_id, player_mid.clone()).unwrap()
        });
        let slow_progress = as_core_contract(&env, &contract_id, |env| {
            HuntyCore::get_player_progress(env.clone(), hunt_id, player_slow.clone()).unwrap()
        });

        assert_eq!(fast_progress.total_score, 20);
        assert_eq!(mid_progress.total_score, 15);
        assert_eq!(slow_progress.total_score, 10);

        let board = as_core_contract(&env, &contract_id, |env| {
            HuntyCore::get_hunt_leaderboard(env.clone(), hunt_id, 3, 0).unwrap()
        });

        assert_eq!(board.len(), 3);
        assert_eq!(board.get(0).unwrap().player, player_fast);
        assert_eq!(board.get(0).unwrap().score, 20);
        assert_eq!(board.get(1).unwrap().player, player_mid);
        assert_eq!(board.get(1).unwrap().score, 15);
        assert_eq!(board.get(2).unwrap().player, player_slow);
        assert_eq!(board.get(2).unwrap().score, 10);
    }

    #[test]
    fn test_create_hunt_with_end_time() {
        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);
        let creator = Address::generate(&env);
        let title = String::from_str(&env, "Timed Hunt");
        let description = String::from_str(&env, "A hunt with an end time");
        let end_time = 1_700_086_400u64; // 1 day in the future

        let hunt = with_core_contract(&env, |env, _cid| {
            let hunt_id = HuntyCore::create_hunt(
                env.clone(),
                creator.clone(),
                title.clone(),
                description.clone(),
                None,
                Some(end_time),
                0,
                None,
            )
            .unwrap();
            Storage::get_hunt(env, hunt_id).unwrap()
        });
        assert_eq!(hunt.end_time, end_time);
    }

    #[test]
    fn test_create_hunt_invalid_end_time() {
        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);
        let creator = Address::generate(&env);
        let title = String::from_str(&env, "Expired Hunt");
        let description = String::from_str(&env, "A hunt with an expired end time");
        let end_time = 1_700_000_000; // equal to current time (invalid)

        let result = with_core_contract(&env, |env, _cid| {
            HuntyCore::create_hunt(
                env.clone(),
                creator.clone(),
                title.clone(),
                description.clone(),
                None,
                Some(end_time),
            )
        });
        assert_eq!(result, Err(HuntErrorCode::InvalidEndTime));

        let end_time_past = 1_699_999_999; // in the past (invalid)
        let result_past = with_core_contract(&env, |env, _cid| {
            HuntyCore::create_hunt(
                env.clone(),
                creator.clone(),
                title.clone(),
                description.clone(),
                None,
                Some(end_time_past),
            )
        });
        assert_eq!(result_past, Err(HuntErrorCode::InvalidEndTime));
    }


    #[test]
    fn test_create_hunt_empty_title() {
        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);
        let creator = Address::generate(&env);
        let title = String::from_str(&env, "");
        let description = String::from_str(&env, "Valid description");

        let result = with_core_contract(&env, |env, _cid| {
            HuntyCore::create_hunt(env.clone(), creator, title, description, None, None, 0, None)
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
            HuntyCore::create_hunt(env.clone(), creator, long_title, description, None, None, 0, None)
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
            HuntyCore::create_hunt(env.clone(), creator, title, description, None, None, 0, None)
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
            HuntyCore::create_hunt(env.clone(), creator, title, long_description, None, None, 0, None)
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
            HuntyCore::create_hunt(env.clone(), creator, title, description, None, None, 0, None)
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
                0,
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
                0,
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
                0,
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
    fn test_create_hunt_twice_returns_different_ids() {
        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);
        let creator = Address::generate(&env);
        let title = String::from_str(&env, "Test Hunt");
        let description = String::from_str(&env, "Description");

        let (first_hunt_id, second_hunt_id) = with_core_contract(&env, |env, _cid| {
            let first_hunt_id = HuntyCore::create_hunt(
                env.clone(),
                creator.clone(),
                title.clone(),
                description.clone(),
                None,
                None,
            )
            .unwrap();
            let second_hunt_id =
                HuntyCore::create_hunt(env.clone(), creator, title, description, None, None)
                    .unwrap();

            (first_hunt_id, second_hunt_id)
        });

        assert_ne!(first_hunt_id, second_hunt_id);
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
                0,
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
                0,
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

        let (start_counter, hunt_id1, counter_after_1, hunt_id2, counter_after_2, hunt_count) =
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
                    0,
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
                    0,
                    None,
                )
                .unwrap();

                // Counter should be 2 after second hunt
                let counter_after_2 = Storage::get_hunt_counter(env);
                let hunt_count = HuntyCore::get_hunt_count(env.clone());

                (
                    start_counter,
                    hunt_id1,
                    counter_after_1,
                    hunt_id2,
                    counter_after_2,
                    hunt_count,
                )
            });

        assert_eq!(start_counter, 0);
        assert_eq!(counter_after_1, 1);
        assert_eq!(hunt_id1, 1);
        assert_eq!(counter_after_2, 2);
        assert_eq!(hunt_id2, 2);
        assert_eq!(hunt_count, 2);
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
                HuntyCore::create_hunt(env.clone(), creator, title, description, None, None, 0, None)
                    .unwrap();
            Storage::get_hunt(env, hunt_id).unwrap()
        });
        let reward_config = hunt.reward_config;

        // Verify default reward config values
        assert_eq!(reward_config.xlm_pool, 0);
        assert_eq!(reward_config.nft_enabled, false);
        assert_eq!(reward_config.distribution_config.nft_contract, None);
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
                HuntyCore::create_hunt(env.clone(), creator, title, description, None, None, 0, None)
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

    #[test]
    fn test_create_hunt_from_template_copies_completed_hunt_clues() {
        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);
        let contract_id = env.register_contract(None, super::HuntyCore);

        let template_creator = Address::generate(&env);
        let new_creator = Address::generate(&env);
        let player = Address::generate(&env);
        let title = String::from_str(&env, "Template Hunt");
        let description = String::from_str(&env, "Completed hunt used as a template");
        let cloned_title = String::from_str(&env, "Remixed Hunt");
        let cloned_description = String::from_str(&env, "Fresh draft from template");
        let q1 = String::from_str(&env, "What is 2 + 2?");
        let q2 = String::from_str(&env, "What is 3 + 3?");
        let a1 = String::from_str(&env, "four");
        let a2 = String::from_str(&env, "six");

        let template_hunt_id = as_core_contract(&env, &contract_id, |env| {
            HuntyCore::create_hunt(
                env.clone(),
                template_creator.clone(),
                title,
                description,
                None,
                None,
            )
            .unwrap()
        });

        let mut template_hunt = as_core_contract(&env, &contract_id, |env| {
            Storage::get_hunt(env, template_hunt_id).unwrap()
        });
        template_hunt.reward_config = crate::types::HuntRewardConfig::new(&env, 0, false, None, 1, 0, 0);
        as_core_contract(&env, &contract_id, |env| {
            Storage::save_hunt(env, &template_hunt);
        });

        env.mock_all_auths();
        as_core_contract(&env, &contract_id, |env| {
            HuntyCore::add_clue(env.clone(), template_hunt_id, q1, a1.clone(), 10, true, 1).unwrap();
        });
        env.mock_all_auths();
        as_core_contract(&env, &contract_id, |env| {
            HuntyCore::add_clue(env.clone(), template_hunt_id, q2, a2.clone(), 20, false, 1)
                .unwrap();
        });
        env.mock_all_auths();
        as_core_contract(&env, &contract_id, |env| {
            HuntyCore::activate_hunt(env.clone(), template_hunt_id, template_creator.clone())
                .unwrap();
        });
        env.mock_all_auths();
        as_core_contract(&env, &contract_id, |env| {
            HuntyCore::register_player(env.clone(), template_hunt_id, player.clone()).unwrap();
        });
        env.mock_all_auths();
        as_core_contract(&env, &contract_id, |env| {
            HuntyCore::submit_answer(
                env.clone(),
                template_hunt_id,
                1,
                player.clone(),
                a1.clone(),
            )
            .unwrap();
        });
        env.mock_all_auths();
        as_core_contract(&env, &contract_id, |env| {
            HuntyCore::complete_hunt(env.clone(), template_hunt_id, player.clone()).unwrap();
        });

        let template_hunt = as_core_contract(&env, &contract_id, |env| {
            Storage::get_hunt(env, template_hunt_id).unwrap()
        });
        let template_clues =
            as_core_contract(&env, &contract_id, |env| Storage::list_clues_for_hunt(env, template_hunt_id, 0, 100));

        let cloned_hunt_id = as_core_contract(&env, &contract_id, |env| {
            HuntyCore::create_hunt_from_template(
                env.clone(),
                template_hunt_id,
                new_creator.clone(),
                cloned_title,
                cloned_description,
                None,
                None,
            )
            .unwrap()
        });

        let cloned_hunt =
            as_core_contract(&env, &contract_id, |env| Storage::get_hunt(env, cloned_hunt_id).unwrap());
        let cloned_clues =
            as_core_contract(&env, &contract_id, |env| Storage::list_clues_for_hunt(env, cloned_hunt_id, 0, 100));

        assert_eq!(template_hunt.status, HuntStatus::Completed);
        assert_eq!(cloned_hunt.status, HuntStatus::Draft);
        assert_eq!(cloned_hunt.creator, new_creator);
        assert_eq!(cloned_hunt.total_clues, 2);
        assert_eq!(cloned_hunt.required_clues, 1);
        assert_eq!(template_clues.len(), cloned_clues.len());

        for i in 0..template_clues.len() {
            assert_eq!(template_clues.get(i).unwrap(), cloned_clues.get(i).unwrap());
        }
    }

    #[test]
    fn test_create_hunt_from_template_rejects_incomplete_template() {
        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);
        let contract_id = env.register_contract(None, super::HuntyCore);

        let creator = Address::generate(&env);
        let new_creator = Address::generate(&env);
        let title = String::from_str(&env, "Template Hunt");
        let description = String::from_str(&env, "Not completed yet");
        let q = String::from_str(&env, "Question?");
        let a = String::from_str(&env, "answer");

        let template_hunt_id = as_core_contract(&env, &contract_id, |env| {
            HuntyCore::create_hunt(
                env.clone(),
                creator.clone(),
                title,
                description,
                None,
                None,
            )
            .unwrap()
        });
        env.mock_all_auths();
        as_core_contract(&env, &contract_id, |env| {
            HuntyCore::add_clue(env.clone(), template_hunt_id, q, a, 10, true, 1).unwrap();
        });
        env.mock_all_auths();
        as_core_contract(&env, &contract_id, |env| {
            HuntyCore::activate_hunt(env.clone(), template_hunt_id, creator.clone()).unwrap();
        });

        let err = as_core_contract(&env, &contract_id, |env| {
            HuntyCore::create_hunt_from_template(
                env.clone(),
                template_hunt_id,
                new_creator,
                String::from_str(env, "Cloned"),
                String::from_str(env, "Draft from template"),
                None,
                None,
            )
            .unwrap_err()
        });

        assert_eq!(err, HuntErrorCode::InvalidHuntStatus);
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
                0,
                None,
            )
            .unwrap();
            let clue_id =
                HuntyCore::add_clue(env.clone(), hunt_id, question.clone(), answer, 10, true, 1)
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
                HuntyCore::create_hunt(env.clone(), creator, title, description, None, None, 0, None)
                    .unwrap();
            let _ = HuntyCore::add_clue(env.clone(), hunt_id, question, answer, 10, true, 1);
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
            let hid = HuntyCore::create_hunt(env.clone(), creator, title, description, None, None, 0, None)
                .unwrap();
            let id1 = HuntyCore::add_clue(env.clone(), hid, q1, a.clone(), 1, false, 1).unwrap();
            let id2 = HuntyCore::add_clue(env.clone(), hid, q2, a.clone(), 1, false, 1).unwrap();
            let id3 = HuntyCore::add_clue(env.clone(), hid, q3, a, 1, false, 1).unwrap();
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
                0,
                None,
            )
            .unwrap();
            let cid =
                HuntyCore::add_clue(env.clone(), hid, question.clone(), answer1, 5, false, 1).unwrap();
            let c = Storage::get_clue(env, hid, cid).unwrap();
            let h1 = c.answer_hashes.get(0).unwrap();
            let hid2 = HuntyCore::create_hunt(
                env.clone(),
                Address::generate(&env),
                String::from_str(&env, "H2"),
                description,
                None,
                None,
                0,
                None,
            )
            .unwrap();
            let _cid2 =
                HuntyCore::add_clue(env.clone(), hid2, question, answer2, 5, false, 1).unwrap();
            let c2 = Storage::get_clue(env, hid2, _cid2).unwrap();
            let h2 = c2.answer_hashes.get(0).unwrap();
            (h1, h2)
        });

        assert_eq!(
            hash1, hash2,
            "normalized '  ANSWER  ' and 'answer' must hash the same"
        );
    }

    #[test]
    fn test_add_clue_whitespace_answer_normalization_and_hashing() {
        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);
        env.mock_all_auths();
        let creator = Address::generate(&env);
        let title = String::from_str(&env, "Hunt");
        let description = String::from_str(&env, "Desc");
        let question = String::from_str(&env, "Whitespace answer?");
        let answer1 = String::from_str(&env, "\t\n answer \r\n");
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
            let cid = HuntyCore::add_clue(env.clone(), hid, question.clone(), answer1, 5, false, 1)
                .unwrap();
            let c = Storage::get_clue(env, hid, cid).unwrap();
            let h1 = c.answer_hashes.get(0).unwrap();
            let hid2 = HuntyCore::create_hunt(
                env.clone(),
                Address::generate(&env),
                String::from_str(&env, "H2"),
                description,
                None,
                None,
            )
            .unwrap();
            let _cid2 = HuntyCore::add_clue(env.clone(), hid2, question, answer2, 5, false, 1).unwrap();
            let c2 = Storage::get_clue(env, hid2, _cid2).unwrap();
            let h2 = c2.answer_hashes.get(0).unwrap();
            (h1, h2)
        });

        assert_eq!(
            hash1, hash2,
            "normalized '\t\n answer \r\n' and 'answer' must hash the same"
        );
    }

    #[test]
    fn test_add_clue_unicode_answer_normalization_and_hashing() {
        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);
        env.mock_all_auths();
        let creator = Address::generate(&env);
        let title = String::from_str(&env, "Hunt");
        let description = String::from_str(&env, "Desc");
        let question = String::from_str(&env, "Same answer?");
        let answer1 = String::from_str(&env, "Café");
        let answer2 = String::from_str(&env, "café");

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
                HuntyCore::add_clue(env.clone(), hid, question.clone(), answer1, 5, false, 1).unwrap();
            let c = Storage::get_clue(env, hid, cid).unwrap();
            let h1 = c.answer_hashes.get(0).unwrap();
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
                HuntyCore::add_clue(env.clone(), hid2, question, answer2, 5, false, 1).unwrap();
            let c2 = Storage::get_clue(env, hid2, _cid2).unwrap();
            let h2 = c2.answer_hashes.get(0).unwrap();
            (h1, h2)
        });

        assert_eq!(
            hash1, hash2,
            "normalized 'Café' and 'café' must hash the same"
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
            let hid = HuntyCore::create_hunt(env.clone(), creator, title, description, None, None, 0, None)
                .unwrap();
            let _ = HuntyCore::add_clue(env.clone(), hid, question.clone(), answer, 7, true, 1);
            HuntyCore::get_clue(env.clone(), hid, 1).unwrap()
        });

        // Prove at compile-time that `ClueInfo` has exactly these fields, and NO `answer_hash` field.
        // The raw `Clue` (with hash) cannot be fetched through the public API (`get_clue` returns `ClueInfo`).
        let ClueInfo {
            clue_id,
            question: ret_question,
            points,
            is_required,
            ..
        } = info;

        assert_eq!(clue_id, 1);
        assert_eq!(ret_question, question);
        assert_eq!(points, 7);
        assert!(is_required);
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
            let hid = HuntyCore::create_hunt(env.clone(), creator, title, description, None, None, 0, None)
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
            let hid = HuntyCore::create_hunt(env.clone(), creator.clone(), title.clone(), description.clone(), None, None)
                .unwrap();
            HuntyCore::list_clues(env.clone(), hid, 0, 10)
        });

        let expected = Vec::new(&env);
        assert_eq!(list, expected);
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
            let hid = HuntyCore::create_hunt(env.clone(), creator, title, description, None, None, 0, None)
                .unwrap();
            HuntyCore::add_clue(env.clone(), hid, q1, a.clone(), 1, false, 1).unwrap();
            HuntyCore::add_clue(env.clone(), hid, q2, a, 2, true, 1).unwrap();
            HuntyCore::list_clues(env.clone(), hid, 0, 10)
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
    fn test_list_clues_pagination() {
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

        let (list1, list2, list_all) = with_core_contract(&env, |env, _cid| {
            let hid = HuntyCore::create_hunt(env.clone(), creator, title, description, None, None)
                .unwrap();
            HuntyCore::add_clue(env.clone(), hid, q1, a.clone(), 1, false, 1).unwrap();
            HuntyCore::add_clue(env.clone(), hid, q2, a.clone(), 2, true, 1).unwrap();
            HuntyCore::add_clue(env.clone(), hid, q3, a, 3, false, 1).unwrap();
            (
                HuntyCore::list_clues(env.clone(), hid, 0, 2),
                HuntyCore::list_clues(env.clone(), hid, 2, 2),
                HuntyCore::list_clues(env.clone(), hid, 0, 10),
            )
        });

        // Validate results
        assert_eq!(list1.len(), 2);
        assert_eq!(list2.len(), 1);
        assert_eq!(list_all.len(), 3);
        
        assert_eq!(list1.get(0).unwrap().clue_id, 1);
        assert_eq!(list1.get(1).unwrap().clue_id, 2);
        assert_eq!(list2.get(0).unwrap().clue_id, 3);
    }

    #[test]
    fn test_add_clue_hunt_not_found() {
        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);
        env.mock_all_auths();
        let question = String::from_str(&env, "Q");
        let answer = String::from_str(&env, "a");

        let err = with_core_contract(&env, |env, _cid| {
            HuntyCore::add_clue(env.clone(), 9999, question, answer, 1, false, 1).unwrap_err()
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
            let hid = HuntyCore::create_hunt(env.clone(), creator, title, description, None, None, 0, None)
                .unwrap();
            HuntyCore::add_clue(env.clone(), hid, empty, answer, 1, false, 1).unwrap_err()
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
            let hid = HuntyCore::create_hunt(env.clone(), creator, title, description, None, None, 0, None)
                .unwrap();
            HuntyCore::add_clue(env.clone(), hid, question, empty, 1, false, 1).unwrap_err()
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
            let hid = HuntyCore::create_hunt(env.clone(), creator, title, description, None, None, 0, None)
                .unwrap();
            HuntyCore::add_clue(env.clone(), hid, question, ws, 1, false, 1).unwrap_err()
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
            let hid = HuntyCore::create_hunt(env.clone(), creator, title, description, None, None, 0, None)
                .unwrap();
            for _ in 0..MAX_CLUES {
                HuntyCore::add_clue(env.clone(), hid, question.clone(), answer.clone(), 1, false, 1)
                    .unwrap();
            }
            HuntyCore::add_clue(env.clone(), hid, question, answer, 1, false, 1).unwrap_err()
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
                0,
                None,
            )
            .unwrap();
            let mut h = Storage::get_hunt(env, hid).unwrap();
            h.status = HuntStatus::Active;
            Storage::save_hunt(env, &h);
            HuntyCore::add_clue(env.clone(), hid, question, answer, 1, false, 1).unwrap_err()
        });

        assert_eq!(err, HuntErrorCode::InvalidHuntStatus);
    }

    #[test]
    fn test_add_clue_after_activation_fails() {
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

            // Add a required clue to allow activation
            HuntyCore::add_clue(env.clone(), hid, question.clone(), answer.clone(), 1, true, 1)
                .unwrap();

            // Activate the hunt
            HuntyCore::activate_hunt(env.clone(), hid, creator.clone()).unwrap();

            // Attempt to add a clue after activation (should fail)
            HuntyCore::add_clue(env.clone(), hid, question, answer, 1, false, 1).unwrap_err()
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
            let hid = HuntyCore::create_hunt(env.clone(), creator, title, description, None, None, 0, None)
                .unwrap();
            HuntyCore::add_clue(env.clone(), hid, long_q, answer, 1, false, 1).unwrap_err()
        });

        assert_eq!(err, HuntErrorCode::InvalidQuestion);
    }

    // ========== add_clue_aliases() Tests ==========

    #[test]
    fn test_add_clue_aliases_success() {
        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);
        let creator = Address::generate(&env);
        let contract_id = env.register_contract(None, super::HuntyCore);

        let hid = as_core_contract(&env, &contract_id, |env| {
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
        let cid = as_core_contract(&env, &contract_id, |env| {
            HuntyCore::add_clue(
                env.clone(),
                hid,
                String::from_str(env, "Capital of USA?"),
                String::from_str(env, "Washington"),
                10,
                true, 1)
            .unwrap()
        });
        env.mock_all_auths();
        as_core_contract(&env, &contract_id, |env| {
            let aliases = Vec::from_array(
                env,
                [
                    String::from_str(env, "Washington D.C."),
                    String::from_str(env, "DC"),
                ],
            );
            HuntyCore::add_clue_aliases(env.clone(), hid, cid, aliases).unwrap();
            let clue = Storage::get_clue(env, hid, cid).unwrap();
            assert_eq!(clue.answer_hashes.len(), 3);
        });
    }

    #[test]
    fn test_add_clue_aliases_answers_accepted() {
        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);
        let creator = Address::generate(&env);
        let player = Address::generate(&env);
        let contract_id = env.register_contract(None, super::HuntyCore);

        let hunt_id = as_core_contract(&env, &contract_id, |env| {
            HuntyCore::create_hunt(
                env.clone(),
                creator.clone(),
                String::from_str(env, "Geo Hunt"),
                String::from_str(env, "Desc"),
                None,
                None,
            )
            .unwrap()
        });
        env.mock_all_auths();
        let cid = as_core_contract(&env, &contract_id, |env| {
            HuntyCore::add_clue(
                env.clone(),
                hunt_id,
                String::from_str(env, "Capital of USA?"),
                String::from_str(env, "Washington"),
                10,
                true, 1)
            .unwrap()
        });
        env.mock_all_auths();
        as_core_contract(&env, &contract_id, |env| {
            let aliases = Vec::from_array(
                env,
                [
                    String::from_str(env, "Washington D.C."),
                    String::from_str(env, "DC"),
                ],
            );
            HuntyCore::add_clue_aliases(env.clone(), hunt_id, cid, aliases).unwrap();
        });
        env.mock_all_auths();
        as_core_contract(&env, &contract_id, |env| {
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
                String::from_str(env, "Washington"),
            )
            .unwrap();
        });
        let progress = as_core_contract(&env, &contract_id, |env| {
            HuntyCore::get_player_progress(env.clone(), hunt_id, player.clone()).unwrap()
        });
        assert!(progress.is_completed);

        // Now test alias answers work — register a new player for each alias
        for alias in ["Washington D.C.", "DC"] {
            let p = Address::generate(&env);
            env.mock_all_auths();
            as_core_contract(&env, &contract_id, |env| {
                HuntyCore::register_player(env.clone(), hunt_id, p.clone()).unwrap();
            });
            env.mock_all_auths();
            as_core_contract(&env, &contract_id, |env| {
                HuntyCore::submit_answer(
                    env.clone(),
                    hunt_id,
                    1,
                    p.clone(),
                    String::from_str(env, alias),
                )
                .unwrap();
            });
            let progress = as_core_contract(&env, &contract_id, |env| {
                HuntyCore::get_player_progress(env.clone(), hunt_id, p.clone()).unwrap()
            });
            assert!(progress.is_completed);
        }
    }

    #[test]
    fn test_add_clue_aliases_hunt_not_found() {
        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);
        env.mock_all_auths();
        let aliases = Vec::from_array(&env, [String::from_str(&env, "alias")]);

        let err = with_core_contract(&env, |env, _cid| {
            HuntyCore::add_clue_aliases(env.clone(), 9999, 1, aliases).unwrap_err()
        });
        assert_eq!(err, HuntErrorCode::HuntNotFound);
    }

    #[test]
    fn test_add_clue_aliases_clue_not_found() {
        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);
        env.mock_all_auths();
        let creator = Address::generate(&env);

        let err = with_core_contract(&env, |env, _cid| {
            let hid = HuntyCore::create_hunt(
                env.clone(),
                creator,
                String::from_str(env, "Hunt"),
                String::from_str(env, "Desc"),
                None,
                None,
            )
            .unwrap();
            let aliases = Vec::from_array(env, [String::from_str(env, "alias")]);
            HuntyCore::add_clue_aliases(env.clone(), hid, 999, aliases).unwrap_err()
        });
        assert_eq!(err, HuntErrorCode::ClueNotFound);
    }

    #[test]
    fn test_add_clue_aliases_invalid_hunt_status() {
        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);
        let creator = Address::generate(&env);
        let contract_id = env.register_contract(None, super::HuntyCore);

        let hid = as_core_contract(&env, &contract_id, |env| {
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
        let cid = as_core_contract(&env, &contract_id, |env| {
            HuntyCore::add_clue(
                env.clone(),
                hid,
                String::from_str(env, "Q"),
                String::from_str(env, "a"),
                1,
                true, 1)
            .unwrap()
        });
        env.mock_all_auths();
        as_core_contract(&env, &contract_id, |env| {
            let mut h = Storage::get_hunt(env, hid).unwrap();
            h.status = HuntStatus::Active;
            Storage::save_hunt(env, &h);
        });
        env.mock_all_auths();
        let err = as_core_contract(&env, &contract_id, |env| {
            let aliases = Vec::from_array(env, [String::from_str(env, "alias")]);
            HuntyCore::add_clue_aliases(env.clone(), hid, cid, aliases).unwrap_err()
        });
        assert_eq!(err, HuntErrorCode::InvalidHuntStatus);
    }

    #[test]
    fn test_add_clue_aliases_preserves_existing_hashes() {
        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);
        let creator = Address::generate(&env);
        let contract_id = env.register_contract(None, super::HuntyCore);

        let hid = as_core_contract(&env, &contract_id, |env| {
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
        let cid = as_core_contract(&env, &contract_id, |env| {
            HuntyCore::add_clue(
                env.clone(),
                hid,
                String::from_str(env, "Q"),
                String::from_str(env, "original"),
                5,
                true, 1)
            .unwrap()
        });
        let original_hash = as_core_contract(&env, &contract_id, |env| {
            let clue_before = Storage::get_clue(env, hid, cid).unwrap();
            clue_before.answer_hashes.get(0).unwrap()
        });
        env.mock_all_auths();
        as_core_contract(&env, &contract_id, |env| {
            let aliases = Vec::from_array(
                env,
                [String::from_str(env, "alias1"), String::from_str(env, "alias2")],
            );
            HuntyCore::add_clue_aliases(env.clone(), hid, cid, aliases).unwrap();
            let clue_after = Storage::get_clue(env, hid, cid).unwrap();
            assert_eq!(clue_after.answer_hashes.len(), 3);
            assert_eq!(clue_after.answer_hashes.get(0).unwrap(), original_hash);
        });
    }

    #[test]
    fn test_add_clue_aliases_empty_answer_fails() {
        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);
        let creator = Address::generate(&env);
        let contract_id = env.register_contract(None, super::HuntyCore);

        let hid = as_core_contract(&env, &contract_id, |env| {
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
        let cid = as_core_contract(&env, &contract_id, |env| {
            HuntyCore::add_clue(
                env.clone(),
                hid,
                String::from_str(env, "Q"),
                String::from_str(env, "a"),
                1,
                true, 1)
            .unwrap()
        });
        env.mock_all_auths();
        let err = as_core_contract(&env, &contract_id, |env| {
            let aliases =
                Vec::from_array(env, [String::from_str(env, ""), String::from_str(env, "valid")]);
            HuntyCore::add_clue_aliases(env.clone(), hid, cid, aliases).unwrap_err()
        });
        assert_eq!(err, HuntErrorCode::InvalidAnswer);
    }

    #[test]
    #[should_panic]
    fn test_add_clue_aliases_creator_only() {
        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);
        // Do NOT mock auth — require_auth(attacker) will panic
        let creator = Address::generate(&env);
        let attacker = Address::generate(&env);
        let aliases = Vec::from_array(&env, [String::from_str(&env, "alias")]);

        with_core_contract(&env, |env, _cid| {
            let hid = HuntyCore::create_hunt(
                env.clone(),
                creator.clone(),
                String::from_str(env, "Hunt"),
                String::from_str(env, "Desc"),
                None,
                None,
            )
            .unwrap();
            let cid = HuntyCore::add_clue(
                env.clone(),
                hid,
                String::from_str(env, "Q"),
                String::from_str(env, "a"),
                1,
                true, 1)
            .unwrap();
            let _ = HuntyCore::add_clue_aliases(env.clone(), hid, cid, aliases);
        });
    }

    #[test]
    fn test_add_clue_zero_points() {
        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);
        env.mock_all_auths();
        let creator = Address::generate(&env);
        let title = String::from_str(&env, "Hunt");
        let description = String::from_str(&env, "Desc");
        let question = String::from_str(&env, "Q");
        let answer = String::from_str(&env, "a");

        let err = with_core_contract(&env, |env, _cid| {
            let hid = HuntyCore::create_hunt(env.clone(), creator, title, description, None, None)
                .unwrap();
            HuntyCore::add_clue(env.clone(), hid, question, answer, 0, false, 1).unwrap_err()
        });

        assert_eq!(err, HuntErrorCode::InvalidPoints);
    }

    #[test]
    fn test_add_clue_invalid_difficulty_zero() {
        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);
        env.mock_all_auths();
        let creator = Address::generate(&env);
        let title = String::from_str(&env, "Hunt");
        let description = String::from_str(&env, "Desc");
        let question = String::from_str(&env, "Q");
        let answer = String::from_str(&env, "a");

        let err = with_core_contract(&env, |env, _cid| {
            let hid = HuntyCore::create_hunt(env.clone(), creator, title, description, None, None)
                .unwrap();
            HuntyCore::add_clue(env.clone(), hid, question, answer, 1, false, 0).unwrap_err()
        });

        assert_eq!(err, HuntErrorCode::InvalidDifficulty);
    }

    #[test]
    fn test_add_clue_invalid_difficulty_exceeds_max() {
        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);
        env.mock_all_auths();
        let creator = Address::generate(&env);
        let title = String::from_str(&env, "Hunt");
        let description = String::from_str(&env, "Desc");
        let question = String::from_str(&env, "Q");
        let answer = String::from_str(&env, "a");

        let err = with_core_contract(&env, |env, _cid| {
            let hid = HuntyCore::create_hunt(env.clone(), creator, title, description, None, None)
                .unwrap();
            HuntyCore::add_clue(env.clone(), hid, question, answer, 1, false, 11).unwrap_err()
        });

        assert_eq!(err, HuntErrorCode::InvalidDifficulty);
    }

    #[test]
    fn test_clue_difficulty_multiplier_in_scoring() {
        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);
        env.mock_all_auths();

        let creator = Address::generate(&env);
        let player = Address::generate(&env);
        let question = String::from_str(&env, "Question");
        let answer = String::from_str(&env, "answer");

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

            // Add clue with 10 points and difficulty 3 (should give 30 points when solved)
            HuntyCore::add_clue(
                env.clone(),
                hunt_id,
                question.clone(),
                answer.clone(),
                10,
                true,
                3,
            )
            .unwrap();

            // Activate hunt
            HuntyCore::activate_hunt(env.clone(), hunt_id, creator.clone()).unwrap();

            // Register player
            HuntyCore::register_player(env.clone(), hunt_id, player.clone()).unwrap();

            // Verify initial score is 0
            let progress =
                HuntyCore::get_player_progress(env.clone(), hunt_id, player.clone()).unwrap();
            assert_eq!(progress.total_score, 0);

            // Submit correct answer
            HuntyCore::submit_answer(env.clone(), hunt_id, 1, player.clone(), answer).unwrap();

            // Verify score is 30 (10 * 3)
            let progress =
                HuntyCore::get_player_progress(env.clone(), hunt_id, player.clone()).unwrap();
            assert_eq!(progress.total_score, 30);
            assert!(progress.is_completed);
        });
    }

    #[test]
    fn test_clue_list_includes_difficulty() {
        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);
        env.mock_all_auths();

        let creator = Address::generate(&env);
        let question = String::from_str(&env, "Q");
        let answer = String::from_str(&env, "a");

        with_core_contract(&env, |env, _cid| {
            let hunt_id = HuntyCore::create_hunt(
                env.clone(),
                creator,
                String::from_str(env, "Hunt"),
                String::from_str(env, "Desc"),
                None,
                None,
            )
            .unwrap();

            // Add clue with difficulty 5
            HuntyCore::add_clue(env.clone(), hunt_id, question, answer, 20, true, 5).unwrap();

            // Get clue and verify difficulty is included
            let info = HuntyCore::get_clue(env.clone(), hunt_id, 1).unwrap();
            assert_eq!(info.difficulty, 5);
            assert_eq!(info.points, 20);

            // List clues and verify difficulty is included
            let list = HuntyCore::list_clues(env.clone(), hunt_id, 0, 10);
            assert_eq!(list.len(), 1);
            let c = list.get(0).unwrap();
            assert_eq!(c.difficulty, 5);
            assert_eq!(c.points, 20);
        });
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
                0,
                None,
            )
            .unwrap();

            // Add a VALID required clue
            HuntyCore::add_clue(env.clone(), hunt_id, question, answer, 1, true, 1).unwrap();

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
                0,
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
                0,
                None,
            )
            .unwrap();

            let err = HuntyCore::activate_hunt(env.clone(), hunt_id, creator.clone()).unwrap_err();
            assert_eq!(err, HuntErrorCode::NoCluesAdded);
        });
    }

    #[test]
    fn test_activate_hunt_no_required_clues() {
        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);
        env.mock_all_auths();
        let creator = Address::generate(&env);

        let title = String::from_str(&env, "Test Hunt");
        let description = String::from_str(&env, "Test description");
        let question = String::from_str(&env, "Optional clue question");
        let answer = String::from_str(&env, "answer");

        with_core_contract(&env, |env, _cid| {
            let hunt_id = HuntyCore::create_hunt(
                env.clone(),
                creator.clone(),
                title,
                description,
                None,
                None,
                0,
                None,
            )
            .unwrap();

            // Add only an optional clue (is_required = false)
            HuntyCore::add_clue(env.clone(), hunt_id, question, answer, 1, false, 1).unwrap();

            // Activating should fail because there are no required clues
            let err = HuntyCore::activate_hunt(env.clone(), hunt_id, creator.clone()).unwrap_err();
            assert_eq!(err, HuntErrorCode::NoRequiredClues);
        });
    }

    #[test]
    fn test_activate_hunt_end_time_in_past() {
        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);
        env.mock_all_auths();
        let creator = Address::generate(&env);

        let question = String::from_str(&env, "Valid question");
        let answer = String::from_str(&env, "a");

        with_core_contract(&env, |env, _cid| {
            // Create a hunt with end_time in the past
            let hunt_id = HuntyCore::create_hunt(
                env.clone(),
                creator.clone(),
                String::from_str(env, "Expired Hunt"),
                String::from_str(env, "This hunt has an end_time in the past"),
                Some(1_699_999_999), // end_time < current_time (1_700_000_000)
                None,
                0,
            )
            .unwrap();

            HuntyCore::add_clue(env.clone(), hunt_id, question, answer, 1, true).unwrap();

            let err = HuntyCore::activate_hunt(env.clone(), hunt_id, creator.clone()).unwrap_err();
            assert_eq!(err, HuntErrorCode::HuntEndTimeInPast);
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
                0,
                None,
            )
            .unwrap();

            // Add a VALID clue first
            HuntyCore::add_clue(env.clone(), hunt_id, question, answer, 1, true, 1).unwrap();

            // Activate hunt
            HuntyCore::activate_hunt(env.clone(), hunt_id, creator.clone()).unwrap();

            // Deactivate hunt — status must be Paused, not Draft (issue #91).
            HuntyCore::deactivate_hunt(env.clone(), hunt_id, creator.clone()).unwrap();

            let hunt = Storage::get_hunt(env, hunt_id).unwrap();
            assert_eq!(hunt.status, HuntStatus::Paused);
        });
    }

    // ── Issue #91: Paused-state tests ─────────────────────────────────────────

    #[test]
    fn test_deactivate_sets_paused_not_draft() {
        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);
        env.mock_all_auths();
        let creator = Address::generate(&env);
        let question = String::from_str(&env, "Q");
        let answer = String::from_str(&env, "a");
        with_core_contract(&env, |env, _cid| {
            let hunt_id = HuntyCore::create_hunt(
                env.clone(), creator.clone(),
                String::from_str(env, "Hunt"), String::from_str(env, "Desc"), None, None,
            ).unwrap();
            HuntyCore::add_clue(env.clone(), hunt_id, question, answer, 1, true, 1).unwrap();
            HuntyCore::activate_hunt(env.clone(), hunt_id, creator.clone()).unwrap();
            HuntyCore::deactivate_hunt(env.clone(), hunt_id, creator.clone()).unwrap();
            let hunt = Storage::get_hunt(env, hunt_id).unwrap();
            assert_eq!(hunt.status, HuntStatus::Paused);
            assert_ne!(hunt.status, HuntStatus::Draft);
        });
    }

    #[test]
    fn test_reactivate_from_paused_succeeds() {
        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);
        env.mock_all_auths();
        let creator = Address::generate(&env);
        let question = String::from_str(&env, "Q");
        let answer = String::from_str(&env, "a");
        with_core_contract(&env, |env, _cid| {
            let hunt_id = HuntyCore::create_hunt(
                env.clone(), creator.clone(),
                String::from_str(env, "Hunt"), String::from_str(env, "Desc"), None, None,
            ).unwrap();
            HuntyCore::add_clue(env.clone(), hunt_id, question, answer, 1, true, 1).unwrap();
            HuntyCore::activate_hunt(env.clone(), hunt_id, creator.clone()).unwrap();
            HuntyCore::deactivate_hunt(env.clone(), hunt_id, creator.clone()).unwrap();
            HuntyCore::activate_hunt(env.clone(), hunt_id, creator.clone()).unwrap();
            let hunt = Storage::get_hunt(env, hunt_id).unwrap();
            assert_eq!(hunt.status, HuntStatus::Active);
        });
    }

    #[test]
    fn test_deactivate_draft_hunt_fails() {
        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);
        env.mock_all_auths();
        let creator = Address::generate(&env);
        let question = String::from_str(&env, "Q");
        let answer = String::from_str(&env, "a");
        with_core_contract(&env, |env, _cid| {
            let hunt_id = HuntyCore::create_hunt(
                env.clone(), creator.clone(),
                String::from_str(env, "Hunt"), String::from_str(env, "Desc"), None, None,
            ).unwrap();
            HuntyCore::add_clue(env.clone(), hunt_id, question, answer, 1, true, 1).unwrap();
            // Hunt is Draft — deactivate must reject it.
            let err = HuntyCore::deactivate_hunt(env.clone(), hunt_id, creator.clone()).unwrap_err();
            assert_eq!(err, HuntErrorCode::InvalidHuntStatus);
        });
    }

    #[test]
    fn test_cannot_add_clue_to_paused_hunt() {
        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);
        env.mock_all_auths();
        let creator = Address::generate(&env);
        let question = String::from_str(&env, "Q");
        let answer = String::from_str(&env, "a");
        with_core_contract(&env, |env, _cid| {
            let hunt_id = HuntyCore::create_hunt(
                env.clone(), creator.clone(),
                String::from_str(env, "Hunt"), String::from_str(env, "Desc"), None, None,
            ).unwrap();
            HuntyCore::add_clue(env.clone(), hunt_id, question.clone(), answer.clone(), 1, true, 1).unwrap();
            HuntyCore::activate_hunt(env.clone(), hunt_id, creator.clone()).unwrap();
            HuntyCore::deactivate_hunt(env.clone(), hunt_id, creator.clone()).unwrap();
            let err = HuntyCore::add_clue(env.clone(), hunt_id, question, answer, 1, false, 1).unwrap_err();
            assert_eq!(err, HuntErrorCode::InvalidHuntStatus);
        });
    }

    #[test]
    fn test_register_player_blocked_when_paused() {
        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);
        env.mock_all_auths();
        let creator = Address::generate(&env);
        let player = Address::generate(&env);
        let question = String::from_str(&env, "Q");
        let answer = String::from_str(&env, "a");
        with_core_contract(&env, |env, _cid| {
            let hunt_id = HuntyCore::create_hunt(
                env.clone(), creator.clone(),
                String::from_str(env, "Hunt"), String::from_str(env, "Desc"), None, None,
            ).unwrap();
            HuntyCore::add_clue(env.clone(), hunt_id, question, answer, 1, true, 1).unwrap();
            HuntyCore::activate_hunt(env.clone(), hunt_id, creator.clone()).unwrap();
            HuntyCore::deactivate_hunt(env.clone(), hunt_id, creator.clone()).unwrap();
            let err = HuntyCore::register_player(env.clone(), hunt_id, player.clone()).unwrap_err();
            assert_eq!(err, HuntErrorCode::InvalidHuntStatus);
        });
    }

    #[test]
    fn test_cancel_from_paused_succeeds() {
        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);
        env.mock_all_auths();
        let creator = Address::generate(&env);
        let question = String::from_str(&env, "Q");
        let answer = String::from_str(&env, "a");
        with_core_contract(&env, |env, _cid| {
            let hunt_id = HuntyCore::create_hunt(
                env.clone(), creator.clone(),
                String::from_str(env, "Hunt"), String::from_str(env, "Desc"), None, None,
            ).unwrap();
            HuntyCore::add_clue(env.clone(), hunt_id, question, answer, 1, true, 1).unwrap();
            HuntyCore::activate_hunt(env.clone(), hunt_id, creator.clone()).unwrap();
            HuntyCore::deactivate_hunt(env.clone(), hunt_id, creator.clone()).unwrap();
            HuntyCore::cancel_hunt(env.clone(), hunt_id, creator.clone()).unwrap();
            let hunt = Storage::get_hunt(env, hunt_id).unwrap();
            assert_eq!(hunt.status, HuntStatus::Cancelled);
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
                0,
                None,
            )
            .unwrap();

            // Add a VALID clue first
            HuntyCore::add_clue(env.clone(), hunt_id, question, answer, 1, true, 1).unwrap();

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
                0,
                None,
            )
            .unwrap();

            // Add a VALID clue first
            HuntyCore::add_clue(env.clone(), hunt_id, question, answer, 1, true, 1).unwrap();

            // Activate hunt
            HuntyCore::activate_hunt(env.clone(), hunt_id, creator.clone()).unwrap();

            // Cancelled hunt
            HuntyCore::cancel_hunt(env.clone(), hunt_id, creator.clone()).unwrap();

            let hunt = Storage::get_hunt(env, hunt_id).unwrap();
            assert_eq!(hunt.status, HuntStatus::Cancelled);

            let status_event = find_hunt_status_changed_event(&env)
                .expect("expected HuntStatusChanged event after cancellation");
            assert_eq!(status_event.hunt_id, hunt_id);
            assert_eq!(status_event.old_status, HuntStatus::Active);
            assert_eq!(status_event.new_status, HuntStatus::Cancelled);
            assert!(status_event.changed_at > 0);
        });
    }

    #[test]
    fn test_cancel_hunt_emits_canceller_and_timestamp() {
        let env = Env::default();
        let cancelled_at = 1_700_000_123;
        env.ledger().set_timestamp(cancelled_at);
        env.mock_all_auths();

        let creator = Address::generate(&env);
        let question = String::from_str(&env, "Valid question");
        let answer = String::from_str(&env, "a");

        with_core_contract(&env, |env, cid| {
            let hunt_id = HuntyCore::create_hunt(
                env.clone(),
                creator.clone(),
                String::from_str(env, "Test Hunt"),
                String::from_str(env, "Test description"),
                None,
                None,
            )
            .unwrap();

            HuntyCore::add_clue(env.clone(), hunt_id, question, answer, 1, true, 1).unwrap();
            HuntyCore::activate_hunt(env.clone(), hunt_id, creator.clone()).unwrap();
            HuntyCore::cancel_hunt(env.clone(), hunt_id, creator.clone()).unwrap();

            let events = env.events().all();
            let (contract, topics, data): (Address, Vec<Val>, Val) =
                events.get(events.len() - 1).unwrap();
            assert_eq!(contract, cid.clone().into());
            assert_eq!(topics.len(), 2);
            assert_eq!(
                Symbol::try_from_val(env, &topics.get(0).unwrap()).unwrap(),
                Symbol::new(env, "HuntCancelled")
            );
            assert_eq!(u64::try_from_val(env, &topics.get(1).unwrap()).unwrap(), hunt_id);

            let event = HuntCancelledEvent::try_from_val(env, &data).unwrap();
            assert_eq!(
                event,
                HuntCancelledEvent {
                    hunt_id,
                    cancelled_by: creator.clone(),
                    cancelled_at,
                }
            );
        });
    }

    #[test]
    fn test_cancel_hunt_refunds_reward_pool_balance() {
        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);
        env.mock_all_auths();

        let creator = Address::generate(&env);
        let question = String::from_str(&env, "Valid question");
        let answer = String::from_str(&env, "a");

        let core_id = env.register_contract(None, super::HuntyCore);
        let (reward_manager_id, token_address, _) = setup_reward_manager(&env, None);
        let sac = token::StellarAssetClient::new(&env, &token_address);
        sac.mint(&creator, &5_000);

        let hunt_id = as_core_contract(&env, &core_id, |env| {
            let hunt_id = HuntyCore::create_hunt(
                env.clone(),
                creator.clone(),
                String::from_str(env, "Refund Hunt"),
                String::from_str(env, "Should refund on cancel"),
                None,
                None,
                0,
                None,
            )
            .unwrap();
            HuntyCore::add_clue(env.clone(), hunt_id, question, answer, 1, true, 1).unwrap();
            HuntyCore::activate_hunt(env.clone(), hunt_id, creator.clone()).unwrap();
            HuntyCore::set_reward_manager(env.clone(), creator.clone(), reward_manager_id.clone());
            hunt_id
        });

        env.mock_all_auths();
        env.as_contract(&reward_manager_id, || {
            RewardManager::create_reward_pool(env.clone(), creator.clone(), hunt_id, 0).unwrap();
        });
        env.mock_all_auths();
        env.as_contract(&reward_manager_id, || {
            RewardManager::fund_reward_pool(env.clone(), creator.clone(), hunt_id, 5_000).unwrap();
        });

        env.mock_all_auths();
        as_core_contract(&env, &core_id, |env| {
            HuntyCore::cancel_hunt(env.clone(), hunt_id, creator.clone()).unwrap();
        });

        env.as_contract(&reward_manager_id, || {
            assert_eq!(RewardManager::get_pool_balance(env.clone(), hunt_id), 0);
        });

        let token_client = token::Client::new(&env, &token_address);
        assert_eq!(token_client.balance(&creator), 5_000);
        assert_eq!(token_client.balance(&reward_manager_id), 0);
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
                0,
                None,
            )
            .unwrap();

            // Add a VALID clue first
            HuntyCore::add_clue(env.clone(), hunt_id, question, answer, 1, true, 1).unwrap();

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
                0,
                None,
            )
            .unwrap();

            // Add a VALID clue first
            HuntyCore::add_clue(env.clone(), hunt_id, question, answer, 1, true, 1).unwrap();

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
                0,
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
                0,
                None,
            )
            .unwrap();
            HuntyCore::add_clue(env.clone(), hunt_id, question, answer, 10, true, 1).unwrap();
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
    fn test_blacklist_creator_blocks_hunt_creation_and_emits_event() {
        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let creator = Address::generate(&env);

        with_core_contract(&env, |env, cid| {
            HuntyCore::initialize_admin(env.clone(), admin.clone()).unwrap();
            HuntyCore::blacklist_creator(env.clone(), admin.clone(), creator.clone()).unwrap();

            assert!(HuntyCore::is_blacklisted(env.clone(), creator.clone()));

            let err = HuntyCore::create_hunt(
                env.clone(),
                creator.clone(),
                String::from_str(env, "Blacklisted Hunt"),
                String::from_str(env, "Should not be created"),
                None,
                None,
                5u32,
                None,
            )
            .unwrap_err();
            assert_eq!(err, HuntErrorCode::AddressBlacklisted);

            let events = env.events().all();
            let (contract, topics, data): (Address, Vec<Val>, Val) =
                events.get(events.len() - 1).unwrap();
            assert_eq!(contract, cid.clone().into());
            assert_eq!(topics.len(), 2);
            assert_eq!(
                Symbol::try_from_val(env, &topics.get(0).unwrap()).unwrap(),
                Symbol::new(env, "CreatorBlacklisted")
            );
            assert_eq!(u64::try_from_val(env, &topics.get(1).unwrap()).unwrap(), 0);

            let event = CreatorBlacklistedEvent::try_from_val(env, &data).unwrap();
            assert_eq!(event.creator, creator);
            assert_eq!(event.admin, admin);
        });
    }

    #[test]
    fn test_remove_from_blacklist_allows_hunt_creation_and_emits_event() {
        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let creator = Address::generate(&env);

        with_core_contract(&env, |env, cid| {
            HuntyCore::initialize_admin(env.clone(), admin.clone()).unwrap();
            HuntyCore::blacklist_creator(env.clone(), admin.clone(), creator.clone()).unwrap();
            HuntyCore::remove_from_blacklist(env.clone(), admin.clone(), creator.clone())
                .unwrap();

            assert!(!HuntyCore::is_blacklisted(env.clone(), creator.clone()));

            let hunt_id = HuntyCore::create_hunt(
                env.clone(),
                creator.clone(),
                String::from_str(env, "Recovered Hunt"),
                String::from_str(env, "Should be created"),
                None,
                None,
                5u32,
                None,
            )
            .unwrap();
            assert_eq!(hunt_id, 1);

            let events = env.events().all();
            let (_contract, topics, _data): (Address, Vec<Val>, Val) =
                events.get(events.len() - 1).unwrap();
            assert_eq!(topics.len(), 2);
            assert_eq!(
                Symbol::try_from_val(env, &topics.get(0).unwrap()).unwrap(),
                Symbol::new(env, "HuntCreated")
            );
            assert_eq!(u64::try_from_val(env, &topics.get(1).unwrap()).unwrap(), hunt_id);
        });
    }

    #[test]
    fn test_pause_contract_blocks_registration_until_unpaused() {
        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let creator = Address::generate(&env);
        let player = Address::generate(&env);
        let question = String::from_str(&env, "Q");
        let answer = String::from_str(&env, "a");

        with_core_contract(&env, |env, _cid| {
            HuntyCore::initialize_admin(env.clone(), admin.clone()).unwrap();
            let hunt_id = HuntyCore::create_hunt(
                env.clone(),
                creator.clone(),
                String::from_str(env, "Hunt"),
                String::from_str(env, "Desc"),
                None,
                None,
                5u32,
                None,
            )
            .unwrap();
            HuntyCore::add_clue(env.clone(), hunt_id, question, answer, 1, true, None).unwrap();
            HuntyCore::activate_hunt(env.clone(), hunt_id, creator.clone()).unwrap();
            HuntyCore::pause_contract(env.clone(), admin.clone()).unwrap();

            let err =
                HuntyCore::register_player(env.clone(), hunt_id, player.clone()).unwrap_err();
            assert_eq!(err, HuntErrorCode::ContractPaused);
            assert!(HuntyCore::is_contract_paused(env.clone()));

            HuntyCore::unpause_contract(env.clone(), admin.clone()).unwrap();
            HuntyCore::register_player(env.clone(), hunt_id, player.clone()).unwrap();
        });
    }

    #[test]
    fn test_pause_contract_requires_admin() {
        let env = Env::default();
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let attacker = Address::generate(&env);

        let err = with_core_contract(&env, |env, _cid| {
            HuntyCore::initialize_admin(env.clone(), admin.clone()).unwrap();
            HuntyCore::pause_contract(env.clone(), attacker.clone()).unwrap_err()
        });

        assert_eq!(err, HuntErrorCode::Unauthorized);
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
                0,
                None,
            )
            .unwrap();
            HuntyCore::add_clue(env.clone(), hunt_id, question, answer, 1, true, 1).unwrap();
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
    fn test_register_player_allowed_after_reactivation() {
        // A player who registered in a previous activation cycle must be able to
        // re-register after the hunt is deactivated and reactivated.
        let env = Env::default();
        env.mock_all_auths();

        let creator = Address::generate(&env);
        let player = Address::generate(&env);
        let question = String::from_str(&env, "Q");
        let answer = String::from_str(&env, "a");

        let (hunt_id, core_id) = with_core_contract(&env, |env, cid| {
            env.ledger().set_timestamp(1_000);
            let hunt_id = HuntyCore::create_hunt(
                env.clone(),
                creator.clone(),
                String::from_str(env, "Hunt"),
                String::from_str(env, "Desc"),
                None,
                None,
                0,
                None,
            )
            .unwrap();
            HuntyCore::add_clue(env.clone(), hunt_id, question, answer, 1, true, None).unwrap();
            (hunt_id, cid.clone())
        });

        // First activation
        as_core_contract(&env, &core_id, |env| {
            HuntyCore::activate_hunt(env.clone(), hunt_id, creator.clone()).unwrap();
        });

        // Player registers
        as_core_contract(&env, &core_id, |env| {
            HuntyCore::register_player(env.clone(), hunt_id, player.clone()).unwrap();
        });

        let first_progress = as_core_contract(&env, &core_id, |env| {
            HuntyCore::get_player_progress(env.clone(), hunt_id, player.clone()).unwrap()
        });

        // Creator deactivates
        as_core_contract(&env, &core_id, |env| {
            HuntyCore::deactivate_hunt(env.clone(), hunt_id, creator.clone()).unwrap();
        });

        env.ledger().set_timestamp(2_000);

        // Reactivate
        as_core_contract(&env, &core_id, |env| {
            HuntyCore::activate_hunt(env.clone(), hunt_id, creator.clone()).unwrap();
        });

        let hunt = as_core_contract(&env, &core_id, |env| {
            Storage::get_hunt(env, hunt_id).unwrap()
        });
        assert!(first_progress.started_at < hunt.activated_at);

        // Player should be able to register again — old progress is stale
        as_core_contract(&env, &core_id, |env| {
            HuntyCore::register_player(env.clone(), hunt_id, player.clone()).unwrap();
        });

        let latest_progress = as_core_contract(&env, &core_id, |env| {
            HuntyCore::get_player_progress(env.clone(), hunt_id, player.clone()).unwrap()
        });
        assert!(latest_progress.started_at >= hunt.activated_at);
        assert_eq!(latest_progress.completed_clues.len(), 0);

        // But a second call in the same cycle must still be rejected
        let err = as_core_contract(&env, &core_id, |env| {
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
                0,
                None,
            )
            .unwrap();
            HuntyCore::add_clue(env.clone(), hunt_id, question, answer, 1, true, 1).unwrap();
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
                0,
                None,
            )
            .unwrap();
            HuntyCore::add_clue(env.clone(), hunt_id, question, answer, 1, true, 1).unwrap();
            HuntyCore::activate_hunt(env.clone(), hunt_id, creator.clone()).unwrap();
            // Move time past end_time
            env.ledger().set_timestamp(1_700_000_002);
            HuntyCore::register_player(env.clone(), hunt_id, player.clone()).unwrap_err()
        });

        assert_eq!(err, HuntErrorCode::HuntNotActive);
    }

    #[test]
    fn test_submit_answer_hunt_ended() {
        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);
        env.mock_all_auths();

        let creator = Address::generate(&env);
        let player = Address::generate(&env);
        let question = String::from_str(&env, "Q");
        let answer = String::from_str(&env, "a");
        let end_time = 1_700_000_001; // One second after "now"

        let (hunt_id, core_id) = with_core_contract(&env, |env, cid| {
            let hunt_id = HuntyCore::create_hunt(
                env.clone(),
                creator.clone(),
                String::from_str(env, "Hunt"),
                String::from_str(env, "Desc"),
                None,
                Some(end_time),
            )
            .unwrap();
            HuntyCore::add_clue(env.clone(), hunt_id, question, answer.clone(), 1, true, 1).unwrap();
            HuntyCore::activate_hunt(env.clone(), hunt_id, creator.clone()).unwrap();
            HuntyCore::register_player(env.clone(), hunt_id, player.clone()).unwrap();
            (hunt_id, cid.clone())
        });

        // Move time past end_time
        env.ledger().set_timestamp(1_700_000_002);
        env.mock_all_auths();

        let err = as_core_contract(&env, &core_id, |env| {
            HuntyCore::submit_answer(env.clone(), hunt_id, 1, player.clone(), answer.clone())
                .unwrap_err()
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
                0,
                None,
            )
            .unwrap();
            HuntyCore::add_clue(env.clone(), hunt_id, question, answer, 1, true, 1).unwrap();
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
                0,
                None,
            )
            .unwrap();
            HuntyCore::add_clue(env.clone(), hunt_id, question, answer, 1, true, 1).unwrap();
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
                0,
                None,
            )
            .unwrap();
            HuntyCore::add_clue(env.clone(), hunt_id, question, answer, 1, true, 1).unwrap();
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
        let contract_id = env.register_contract(None, super::HuntyCore);
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
                0,
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
                true, 1)
            .unwrap();
            HuntyCore::activate_hunt(env.clone(), hunt_id, creator.clone()).unwrap();
        });
        env.mock_all_auths();
        as_core_contract(&env, &contract_id, |env| {
            HuntyCore::register_player(env.clone(), hunt_id, player.clone()).unwrap();
        });
        env.mock_all_auths();
        as_core_contract(&env, &contract_id, |env| {
            HuntyCore::submit_answer(env.clone(), hunt_id, 1, player.clone(), answer.clone())
                .unwrap();
        });
        let progress = as_core_contract(&env, &contract_id, |env| {
            HuntyCore::get_player_progress(env.clone(), hunt_id, player.clone()).unwrap()
        });
        assert_eq!(progress.player, player);
        assert_eq!(progress.hunt_id, hunt_id);
        assert_eq!(progress.completed_clues.len(), 1);
        assert_eq!(progress.required_completed_count, 1);
        assert_eq!(progress.total_score, 10);
        assert!(progress.is_completed);
        assert!(progress.completed_at > 0);
    }

    #[test]
    fn test_pause_contract_blocks_answer_submission_until_unpaused() {
        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);
        env.mock_all_auths();

        let admin = Address::generate(&env);
        let creator = Address::generate(&env);
        let player = Address::generate(&env);
        let question = String::from_str(&env, "Q");
        let answer = String::from_str(&env, "a");

        let contract_id = env.register_contract(None, super::HuntyCore);
        env.mock_all_auths();
        let hunt_id = as_core_contract(&env, &contract_id, |env| {
            HuntyCore::initialize_admin(env.clone(), admin.clone()).unwrap();
            let hunt_id = HuntyCore::create_hunt(
                env.clone(),
                creator.clone(),
                String::from_str(env, "Hunt"),
                String::from_str(env, "Desc"),
                None,
                None,
                0,
                None,
            )
            .unwrap();
            HuntyCore::add_clue(env.clone(), hunt_id, question, answer.clone(), 10, true, 1)
                .unwrap();
            HuntyCore::activate_hunt(env.clone(), hunt_id, creator.clone()).unwrap();
            HuntyCore::register_player(env.clone(), hunt_id, player.clone()).unwrap();
            HuntyCore::pause_contract(env.clone(), admin.clone()).unwrap();
            hunt_id
        });

        env.mock_all_auths();
        let err = as_core_contract(&env, &contract_id, |env| {
            HuntyCore::submit_answer(
                env.clone(),
                hunt_id,
                1,
                player.clone(),
                answer.clone(),
            )
            .unwrap_err()
        });
        assert_eq!(err, HuntErrorCode::ContractPaused);

        env.mock_all_auths();
        as_core_contract(&env, &contract_id, |env| {
            HuntyCore::unpause_contract(env.clone(), admin.clone()).unwrap();
            HuntyCore::submit_answer(env.clone(), hunt_id, 1, player.clone(), answer)
                .unwrap();
        });
    }

    #[test]
    fn test_required_completed_counter_is_not_double_incremented_on_resubmit() {
        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);

        let creator = Address::generate(&env);
        let player = Address::generate(&env);
        let question = String::from_str(&env, "Q");
        let answer = String::from_str(&env, "a");

        let contract_id = env.register_contract(None, super::HuntyCore);
        let hunt_id = as_core_contract(&env, &contract_id, |env| {
            let hunt_id = HuntyCore::create_hunt(
                env.clone(),
                creator.clone(),
                String::from_str(env, "Hunt"),
                String::from_str(env, "Desc"),
                None,
                None,
            )
            .unwrap();
            HuntyCore::add_clue(env.clone(), hunt_id, question, answer.clone(), 10, true, 1).unwrap();
            HuntyCore::activate_hunt(env.clone(), hunt_id, creator.clone()).unwrap();
            hunt_id
        });

        env.mock_all_auths();
        as_core_contract(&env, &contract_id, |env| {
            HuntyCore::register_player(env.clone(), hunt_id, player.clone()).unwrap();
        });
        env.mock_all_auths();
        as_core_contract(&env, &contract_id, |env| {
            HuntyCore::submit_answer(env.clone(), hunt_id, 1, player.clone(), answer.clone()).unwrap();
        });
        env.mock_all_auths();
        let resubmit = as_core_contract(&env, &contract_id, |env| {
            HuntyCore::submit_answer(env.clone(), hunt_id, 1, player.clone(), answer)
        });

        assert_eq!(resubmit, Err(HuntErrorCode::ClueAlreadyCompleted));

        let progress = as_core_contract(&env, &contract_id, |env| {
            HuntyCore::get_player_progress(env.clone(), hunt_id, player.clone()).unwrap()
        });
        assert_eq!(progress.required_completed_count, 1);
    }


    fn test_required_completed_counter_stays_isolated_per_player() {
        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);

        let creator = Address::generate(&env);
        let player_a = Address::generate(&env);
        let player_b = Address::generate(&env);
        let answer = String::from_str(&env, "a");

        let contract_id = env.register_contract(None, super::HuntyCore);
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

        as_core_contract(&env, &contract_id, |env| {
            HuntyCore::add_clue(
                env.clone(),
                hunt_id,
                String::from_str(env, "Q1"),
                answer.clone(),
                5,
                true,
                1,
            )
            .unwrap();
        });

        as_core_contract(&env, &contract_id, |env| {
            HuntyCore::add_clue(
                env.clone(),
                hunt_id,
                String::from_str(env, "Q2"),
                answer.clone(),
                5,
                true,
                1,
            )
            .unwrap();
        });

        as_core_contract(&env, &contract_id, |env| {
            HuntyCore::activate_hunt(env.clone(), hunt_id, creator.clone()).unwrap();
        });

        env.mock_all_auths();
        as_core_contract(&env, &contract_id, |env| {
            HuntyCore::register_player(env.clone(), hunt_id, player_a.clone()).unwrap();
            HuntyCore::register_player(env.clone(), hunt_id, player_b.clone()).unwrap();
        });
        env.mock_all_auths();
        as_core_contract(&env, &contract_id, |env| {
            HuntyCore::submit_answer(env.clone(), hunt_id, 1, player_a.clone(), answer.clone())
                .unwrap();
            HuntyCore::submit_answer(env.clone(), hunt_id, 2, player_b.clone(), answer.clone())
                .unwrap();
        });

        let progress_a = as_core_contract(&env, &contract_id, |env| {
            HuntyCore::get_player_progress(env.clone(), hunt_id, player_a.clone()).unwrap()
        });
        let progress_b = as_core_contract(&env, &contract_id, |env| {
            HuntyCore::get_player_progress(env.clone(), hunt_id, player_b.clone()).unwrap()
        });

        assert_eq!(progress_a.required_completed_count, 1);
        assert_eq!(progress_b.required_completed_count, 1);
        assert!(!progress_a.is_completed);
        assert!(!progress_b.is_completed);
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
                0,
                None,
            )
            .unwrap();
            HuntyCore::add_clue(env.clone(), hunt_id, question, answer, 1, true, 1).unwrap();
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

        let contract_id = env.register_contract(None, super::HuntyCore);
        let hunt_id = as_core_contract(&env, &contract_id, |env| {
            HuntyCore::create_hunt(
                env.clone(),
                creator.clone(),
                String::from_str(env, "Hunt"),
                String::from_str(env, "Desc"),
                None,
                None,
                0,
                None,
            )
            .unwrap()
        });
        env.mock_all_auths();
        as_core_contract(&env, &contract_id, |env| {
            HuntyCore::add_clue(env.clone(), hunt_id, q1, a.clone(), 5, false, 1).unwrap();
        });
        env.mock_all_auths();
        as_core_contract(&env, &contract_id, |env| {
            HuntyCore::add_clue(env.clone(), hunt_id, q2.clone(), a.clone(), 10, true, 1).unwrap();
            HuntyCore::activate_hunt(env.clone(), hunt_id, creator.clone()).unwrap();
        });
        env.mock_all_auths();
        as_core_contract(&env, &contract_id, |env| {
            HuntyCore::register_player(env.clone(), hunt_id, player.clone()).unwrap();
        });
        env.mock_all_auths();
        as_core_contract(&env, &contract_id, |env| {
            HuntyCore::submit_answer(env.clone(), hunt_id, 1, player.clone(), a.clone()).unwrap();
        });
        env.mock_all_auths();
        as_core_contract(&env, &contract_id, |env| {
            HuntyCore::submit_answer(env.clone(), hunt_id, 2, player.clone(), a, 1, env.ledger().timestamp()).unwrap();
        });
        let list = as_core_contract(&env, &contract_id, |env| {
            HuntyCore::get_completed_clues(env.clone(), hunt_id, player.clone())
        });

        assert_eq!(list.len(), 2);
        assert_eq!(list.get(0).unwrap(), 1);
        assert_eq!(list.get(1).unwrap(), 2);
    }

    #[test]
    fn test_submit_answer_clue_already_completed_does_not_double_count_score() {
        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);

        let creator = Address::generate(&env);
        let player = Address::generate(&env);
        let question = String::from_str(&env, "Q");
        let answer = String::from_str(&env, "a");

        let contract_id = env.register_contract(None, super::HuntyCore);
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
            HuntyCore::add_clue(env.clone(), hunt_id, question, answer.clone(), 10, true, 1).unwrap();
            HuntyCore::activate_hunt(env.clone(), hunt_id, creator.clone()).unwrap();
        });

        env.mock_all_auths();
        as_core_contract(&env, &contract_id, |env| {
            HuntyCore::register_player(env.clone(), hunt_id, player.clone()).unwrap();
        });

        env.mock_all_auths();
        as_core_contract(&env, &contract_id, |env| {
            HuntyCore::submit_answer(env.clone(), hunt_id, 1, player.clone(), answer.clone())
                .unwrap();
        });

        env.mock_all_auths();
        let err = as_core_contract(&env, &contract_id, |env| {
            HuntyCore::submit_answer(env.clone(), hunt_id, 1, player.clone(), answer.clone())
                .unwrap_err()
        });

        assert_eq!(err, HuntErrorCode::ClueAlreadyCompleted);

        let progress = as_core_contract(&env, &contract_id, |env| {
            HuntyCore::get_player_progress(env.clone(), hunt_id, player.clone()).unwrap()
        });

        assert_eq!(progress.completed_clues.len(), 1);
        assert_eq!(progress.total_score, 10);
    }

    #[test]
    fn test_get_hunt_leaderboard_hunt_not_found() {
        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);

        let err = with_core_contract(&env, |env, _cid| {
            HuntyCore::get_hunt_leaderboard(env.clone(), 9999, 10, 0).unwrap_err()
        });

        assert_eq!(err, HuntErrorCode::HuntNotFound);
    }

    #[test]
    fn test_get_hunt_leaderboard_with_0_registered_players() {
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
                0,
                None,
            )
            .unwrap();
            HuntyCore::add_clue(env.clone(), hunt_id, question, answer, 1, true, 1).unwrap();
            HuntyCore::activate_hunt(env.clone(), hunt_id, creator.clone()).unwrap();
            HuntyCore::get_hunt_leaderboard(env.clone(), hunt_id, 10, 0).unwrap()
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

        let contract_id = env.register_contract(None, super::HuntyCore);
        let hunt_id = as_core_contract(&env, &contract_id, |env| {
            HuntyCore::create_hunt(
                env.clone(),
                creator.clone(),
                String::from_str(env, "Hunt"),
                String::from_str(env, "Desc"),
                None,
                None,
                0,
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
                false, 1)
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
                true, 1)
            .unwrap();
            HuntyCore::activate_hunt(env.clone(), hunt_id, creator.clone()).unwrap();
        });
        env.mock_all_auths();
        as_core_contract(&env, &contract_id, |env| {
            HuntyCore::register_player(env.clone(), hunt_id, player_a.clone(), 1, env.ledger().timestamp()).unwrap();
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
            HuntyCore::submit_answer(env.clone(), hunt_id, 1, player_b.clone(), answer.clone())
                .unwrap();
        });
        env.mock_all_auths();
        as_core_contract(&env, &contract_id, |env| {
            HuntyCore::submit_answer(env.clone(), hunt_id, 2, player_b.clone(), answer.clone())
                .unwrap();
        });
        env.ledger().set_timestamp(1_700_000_002);
        env.mock_all_auths();
        as_core_contract(&env, &contract_id, |env| {
            HuntyCore::submit_answer(env.clone(), hunt_id, 1, player_a.clone(), answer.clone())
                .unwrap();
        });
        env.mock_all_auths();
        as_core_contract(&env, &contract_id, |env| {
            HuntyCore::submit_answer(env.clone(), hunt_id, 2, player_a.clone(), answer.clone())
                .unwrap();
        });
        env.ledger().set_timestamp(1_700_000_003);
        env.mock_all_auths();
        as_core_contract(&env, &contract_id, |env| {
            HuntyCore::submit_answer(env.clone(), hunt_id, 1, player_c.clone(), answer.clone())
                .unwrap();
        });
        let board = as_core_contract(&env, &contract_id, |env| {
            HuntyCore::get_hunt_leaderboard(env.clone(), hunt_id, 10, 0).unwrap()
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
        let creator = Address::generate(&env);
        let question = String::from_str(&env, "Q");
        let answer = String::from_str(&env, "a");

        let contract_id = env.register(HuntyCore, ());
        env.mock_all_auths();
        let hunt_id = as_core_contract(&env, &contract_id, |env| {
            HuntyCore::create_hunt(
                env.clone(),
                creator.clone(),
                String::from_str(env, "Hunt"),
                String::from_str(env, "Desc"),
                None,
                None,
                0,
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
                1,
                true,
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
                1,
                true,
                None,
            )
            .unwrap();
            HuntyCore::activate_hunt(env.clone(), hunt_id, creator.clone()).unwrap();
            let mut players = Vec::new(env);
            for _ in 0..5 {
                players.push_back(Address::generate(env));
            }
            for i in 0..5 {
                let p = players.get(i).unwrap();
                HuntyCore::register_player(env.clone(), hunt_id, p.clone()).unwrap();
                submit_answer(env, hunt_id, 1, p.clone(), answer.clone(), i as u64 + 1).unwrap();
            }
        });
        let board = as_core_contract(&env, &contract_id, |env| {
            HuntyCore::get_hunt_leaderboard(env.clone(), hunt_id, 2).unwrap()
        });

        assert_eq!(board.len(), 2);
        assert_eq!(board.get(0).unwrap().rank, 1);
        assert_eq!(board.get(1).unwrap().rank, 2);
    }

    #[test]
    fn test_get_hunt_leaderboard_offset_pagination() {
        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);

        let creator = Address::generate(&env);
        let question = String::from_str(&env, "Q");
        let answer = String::from_str(&env, "a");

        // 3 players: player_a scores 10 (completes first), player_b scores 10 (completes second),
        // player_c scores 5 (optional clue only). Ranking: a=1, b=2, c=3.
        let contract_id = env.register_contract(None, super::HuntyCore);
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
            HuntyCore::add_clue(env.clone(), hunt_id, question.clone(), answer.clone(), 10, true, 1)
                .unwrap();
        });
        env.mock_all_auths();
        as_core_contract(&env, &contract_id, |env| {
            HuntyCore::add_clue(env.clone(), hunt_id, question.clone(), answer.clone(), 5, false, 1)
                .unwrap();
            HuntyCore::activate_hunt(env.clone(), hunt_id, creator.clone()).unwrap();
        });

        let player_a = Address::generate(&env);
        let player_b = Address::generate(&env);
        let player_c = Address::generate(&env);

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
            HuntyCore::submit_answer(env.clone(), hunt_id, 1, player_a.clone(), answer.clone())
                .unwrap();
        });
        env.ledger().set_timestamp(1_700_000_002);
        env.mock_all_auths();
        as_core_contract(&env, &contract_id, |env| {
            HuntyCore::submit_answer(env.clone(), hunt_id, 1, player_b.clone(), answer.clone())
                .unwrap();
        });
        env.mock_all_auths();
        as_core_contract(&env, &contract_id, |env| {
            HuntyCore::submit_answer(env.clone(), hunt_id, 2, player_c.clone(), answer.clone())
                .unwrap();
        });

        // offset=0 returns full board
        let page1 = as_core_contract(&env, &contract_id, |env| {
            HuntyCore::get_hunt_leaderboard(env.clone(), hunt_id, 10, 0).unwrap()
        });
        assert_eq!(page1.len(), 3);
        assert_eq!(page1.get(0).unwrap().player, player_a);
        assert_eq!(page1.get(0).unwrap().rank, 1);

        // offset=1 skips rank 1, returns b(2) and c(3)
        let page2 = as_core_contract(&env, &contract_id, |env| {
            HuntyCore::get_hunt_leaderboard(env.clone(), hunt_id, 10, 1).unwrap()
        });
        assert_eq!(page2.len(), 2);
        assert_eq!(page2.get(0).unwrap().player, player_b);
        assert_eq!(page2.get(0).unwrap().rank, 2);
        assert_eq!(page2.get(1).unwrap().player, player_c);
        assert_eq!(page2.get(1).unwrap().rank, 3);

        // offset=2 returns only c(3)
        let page3 = as_core_contract(&env, &contract_id, |env| {
            HuntyCore::get_hunt_leaderboard(env.clone(), hunt_id, 10, 2).unwrap()
        });
        assert_eq!(page3.len(), 1);
        assert_eq!(page3.get(0).unwrap().player, player_c);
        assert_eq!(page3.get(0).unwrap().rank, 3);

        // offset beyond all entries returns empty
        let empty = as_core_contract(&env, &contract_id, |env| {
            HuntyCore::get_hunt_leaderboard(env.clone(), hunt_id, 10, 100).unwrap()
        });
        assert_eq!(empty.len(), 0);
    }

    /// Issue #428: players with equal scores are tie-broken by completion time
    /// (earlier completion ranks higher).
    #[test]
    fn test_get_hunt_leaderboard_equal_scores_tiebreak_by_completion_time() {
        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);

        let creator = Address::generate(&env);
        let player_early = Address::generate(&env);
        let player_late = Address::generate(&env);
        let question = String::from_str(&env, "Q");
        let answer = String::from_str(&env, "a");

        let contract_id = env.register(HuntyCore, ());
        env.mock_all_auths();
        let hunt_id = as_core_contract(&env, &contract_id, |env| {
            HuntyCore::create_hunt(
                env.clone(),
                creator.clone(),
                String::from_str(env, "Hunt"),
                String::from_str(env, "Desc"),
                None,
                None,
                0,
                None,
            )
            .unwrap()
        });
        env.mock_all_auths();
        as_core_contract(&env, &contract_id, |env| {
            HuntyCore::add_clue(env.clone(), hunt_id, question.clone(), answer.clone(), 10, true, None)
                .unwrap();
            HuntyCore::activate_hunt(env.clone(), hunt_id, creator.clone()).unwrap();
        });

        // Both players register at the same timestamp so their start time is identical.
        env.mock_all_auths();
        as_core_contract(&env, &contract_id, |env| {
            HuntyCore::register_player(env.clone(), hunt_id, player_early.clone()).unwrap();
        });
        env.mock_all_auths();
        as_core_contract(&env, &contract_id, |env| {
            HuntyCore::register_player(env.clone(), hunt_id, player_late.clone()).unwrap();
        });

        // Both complete within the same scoring window (< 50s) so scores are equal,
        // but `player_early` completes one second before `player_late`.
        env.ledger().set_timestamp(1_700_000_001);
        env.mock_all_auths();
        as_core_contract(&env, &contract_id, |env| {
            submit_answer(env, hunt_id, 1, player_early.clone(), answer.clone(), 1).unwrap();
        });
        env.ledger().set_timestamp(1_700_000_002);
        env.mock_all_auths();
        as_core_contract(&env, &contract_id, |env| {
            submit_answer(env, hunt_id, 1, player_late.clone(), answer.clone(), 2).unwrap();
        });

        let board = as_core_contract(&env, &contract_id, |env| {
            HuntyCore::get_hunt_leaderboard(env.clone(), hunt_id, 10).unwrap()
        });

        let first = board.get(0).unwrap();
        let second = board.get(1).unwrap();
        assert_eq!(board.len(), 2);
        assert_eq!(first.score, second.score);
        assert_eq!(first.player, player_early);
        assert_eq!(second.player, player_late);
        assert_eq!(first.rank, 1);
        assert_eq!(second.rank, 2);
        assert!(first.completed_at < second.completed_at);
    }

    /// Issue #428: a leaderboard with a single player returns exactly that player at rank 1.
    #[test]
    fn test_get_hunt_leaderboard_single_player() {
        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);

        let creator = Address::generate(&env);
        let player = Address::generate(&env);
        let question = String::from_str(&env, "Q");
        let answer = String::from_str(&env, "a");

        let contract_id = env.register(HuntyCore, ());
        env.mock_all_auths();
        let hunt_id = as_core_contract(&env, &contract_id, |env| {
            HuntyCore::create_hunt(
                env.clone(),
                creator.clone(),
                String::from_str(env, "Hunt"),
                String::from_str(env, "Desc"),
                None,
                None,
                0,
                None,
            )
            .unwrap()
        });
        env.mock_all_auths();
        as_core_contract(&env, &contract_id, |env| {
            HuntyCore::add_clue(env.clone(), hunt_id, question.clone(), answer.clone(), 10, true, None)
                .unwrap();
            HuntyCore::activate_hunt(env.clone(), hunt_id, creator.clone()).unwrap();
        });
        env.mock_all_auths();
        as_core_contract(&env, &contract_id, |env| {
            HuntyCore::register_player(env.clone(), hunt_id, player.clone()).unwrap();
        });
        env.ledger().set_timestamp(1_700_000_001);
        env.mock_all_auths();
        as_core_contract(&env, &contract_id, |env| {
            submit_answer(env, hunt_id, 1, player.clone(), answer.clone(), 1).unwrap();
        });

        let board = as_core_contract(&env, &contract_id, |env| {
            HuntyCore::get_hunt_leaderboard(env.clone(), hunt_id, 10).unwrap()
        });

        assert_eq!(board.len(), 1);
        let only = board.get(0).unwrap();
        assert_eq!(only.rank, 1);
        assert_eq!(only.player, player);
        assert!(only.is_completed);
        assert!(only.score > 0);
    }

    /// The maintained leaderboard index is updated on score changes only, so
    /// registered zero-score players are excluded until they earn points.
    #[test]
    fn test_get_hunt_leaderboard_excludes_zero_score_players() {
        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);

        let creator = Address::generate(&env);
        let scorer = Address::generate(&env);
        let zero_a = Address::generate(&env);
        let zero_b = Address::generate(&env);
        let question = String::from_str(&env, "Q");
        let answer = String::from_str(&env, "a");

        let contract_id = env.register(HuntyCore, ());
        env.mock_all_auths();
        let hunt_id = as_core_contract(&env, &contract_id, |env| {
            HuntyCore::create_hunt(
                env.clone(),
                creator.clone(),
                String::from_str(env, "Hunt"),
                String::from_str(env, "Desc"),
                None,
                None,
                0,
                None,
            )
            .unwrap()
        });
        env.mock_all_auths();
        as_core_contract(&env, &contract_id, |env| {
            HuntyCore::add_clue(env.clone(), hunt_id, question.clone(), answer.clone(), 10, true, None)
                .unwrap();
            HuntyCore::activate_hunt(env.clone(), hunt_id, creator.clone()).unwrap();
        });

        // One player scores; two register but never submit a correct answer (zero score).
        env.mock_all_auths();
        as_core_contract(&env, &contract_id, |env| {
            HuntyCore::register_player(env.clone(), hunt_id, scorer.clone()).unwrap();
        });
        env.mock_all_auths();
        as_core_contract(&env, &contract_id, |env| {
            HuntyCore::register_player(env.clone(), hunt_id, zero_a.clone()).unwrap();
        });
        env.mock_all_auths();
        as_core_contract(&env, &contract_id, |env| {
            HuntyCore::register_player(env.clone(), hunt_id, zero_b.clone()).unwrap();
        });
        env.ledger().set_timestamp(1_700_000_001);
        env.mock_all_auths();
        as_core_contract(&env, &contract_id, |env| {
            submit_answer(env, hunt_id, 1, scorer.clone(), answer.clone(), 1).unwrap();
        });

        let board = as_core_contract(&env, &contract_id, |env| {
            HuntyCore::get_hunt_leaderboard(env.clone(), hunt_id, 10).unwrap()
        });

        assert_eq!(board.len(), 1);
        let first = board.get(0).unwrap();
        assert_eq!(first.player, scorer);
        assert_eq!(first.rank, 1);
        assert!(first.score > 0);
    }

    #[test]
    fn test_get_hunt_leaderboard_maintains_top_n_on_score_updates() {
        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);

        let creator = Address::generate(&env);
        let question = String::from_str(&env, "Q");
        let answer = String::from_str(&env, "a");
        let contract_id = env.register(HuntyCore, ());

        env.mock_all_auths();
        let hunt_id = as_core_contract(&env, &contract_id, |env| {
            HuntyCore::create_hunt(
                env.clone(),
                creator.clone(),
                String::from_str(env, "Top N Hunt"),
                String::from_str(env, "Leaderboard index maintenance"),
                None,
                None,
                0,
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
                None,
            )
            .unwrap();
            HuntyCore::activate_hunt(env.clone(), hunt_id, creator.clone()).unwrap();
        });

        let total_players = crate::MAX_LEADERBOARD_SIZE + 1;
        let mut players = Vec::new(&env);
        for i in 0..total_players {
            let player = Address::generate(&env);
            players.push_back(player.clone());
            env.mock_all_auths();
            as_core_contract(&env, &contract_id, |env| {
                HuntyCore::register_player(env.clone(), hunt_id, player.clone()).unwrap();
            });
            env.ledger().set_timestamp(1_700_000_001 + i as u64);
            env.mock_all_auths();
            as_core_contract(&env, &contract_id, |env| {
                submit_answer(env, hunt_id, 1, player.clone(), answer.clone(), i as u64 + 1).unwrap();
            });
        }

        let initial_board = as_core_contract(&env, &contract_id, |env| {
            HuntyCore::get_hunt_leaderboard(env.clone(), hunt_id, crate::MAX_LEADERBOARD_SIZE)
                .unwrap()
        });
        assert_eq!(initial_board.len(), crate::MAX_LEADERBOARD_SIZE);

        let promoted_player = players.get(total_players - 1).unwrap();
        env.mock_all_auths();
        as_core_contract(&env, &contract_id, |env| {
            let mut progress = Storage::get_player_progress(env, hunt_id, &promoted_player).unwrap();
            progress.total_score = 999;
            Storage::save_player_progress(env, &progress);
            HuntyCore::update_leaderboard_index(env, &progress);
        });

        let board = as_core_contract(&env, &contract_id, |env| {
            HuntyCore::get_hunt_leaderboard(env.clone(), hunt_id, crate::MAX_LEADERBOARD_SIZE)
                .unwrap()
        });
        assert_eq!(board.len(), crate::MAX_LEADERBOARD_SIZE);
        assert_eq!(board.get(0).unwrap().player, promoted_player);
        assert_eq!(board.get(0).unwrap().score, 999);
    }

    /// Stress test: many players update scores while the maintained index stays bounded.
    #[test]
    fn test_get_hunt_leaderboard_bounded_index_size() {
        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);

        let creator = Address::generate(&env);
        let question = String::from_str(&env, "Q");
        let answer = String::from_str(&env, "a");

        let contract_id = env.register(HuntyCore, ());
        let hunt_id = as_core_contract(&env, &contract_id, |env| {
            HuntyCore::create_hunt(
                env.clone(),
                creator.clone(),
                String::from_str(env, "Stress Hunt"),
                String::from_str(env, "Desc"),
                None,
                None,
                0,
                None,
            )
            .unwrap()
        });
        env.mock_all_auths();
        as_core_contract(&env, &contract_id, |env| {
            HuntyCore::add_clue(env.clone(), hunt_id, question.clone(), answer.clone(), 10, true, None)
                .unwrap();
            HuntyCore::activate_hunt(env.clone(), hunt_id, creator.clone()).unwrap();
        });

        let num_players = 200;
        let mut players = Vec::new(&env);
        for i in 0..num_players {
            let player = Address::generate(&env);
            players.push_back(player.clone());
            env.mock_all_auths();
            as_core_contract(&env, &contract_id, |env| {
                HuntyCore::register_player(env.clone(), hunt_id, player.clone()).unwrap();
            });
            // Make every other player complete the hunt with varying scores
            if i % 2 == 0 {
                env.ledger().set_timestamp(1_700_000_000 + i as u64 + 1);
                env.mock_all_auths();
                as_core_contract(&env, &contract_id, |env| {
                    submit_answer(env, hunt_id, 1, player.clone(), answer.clone(), i as u64 + 1).unwrap();
                });
            }
        }

        // Get leaderboard and verify it's correctly sorted
        let board = as_core_contract(&env, &contract_id, |env| {
            HuntyCore::get_hunt_leaderboard(env.clone(), hunt_id, crate::MAX_LEADERBOARD_SIZE).unwrap()
        });

        // Verify we have up to MAX_LEADERBOARD_SIZE entries
        assert!(board.len() <= crate::MAX_LEADERBOARD_SIZE);
        // Verify ordering (score descending, then completion time ascending)
        let mut last_score = u32::MAX;
        let mut last_completed_at = 0;
        for i in 0..board.len() {
            let entry = board.get(i).unwrap();
            assert!(entry.score <= last_score);
            if entry.score == last_score && entry.is_completed {
                assert!(entry.completed_at >= last_completed_at);
            }
            last_score = entry.score;
            if entry.is_completed {
                last_completed_at = entry.completed_at;
            }
        }
    }

    /// Test that leaderboard works correctly with pagination (even though the function doesn't have explicit pagination, verify that it returns the correct top N)
    #[test]
    fn test_get_hunt_leaderboard_pagination_effect() {
        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);

        let creator = Address::generate(&env);
        let question = String::from_str(&env, "Q");
        let answer = String::from_str(&env, "a");

        let contract_id = env.register(HuntyCore, ());
        let hunt_id = as_core_contract(&env, &contract_id, |env| {
            HuntyCore::create_hunt(
                env.clone(),
                creator.clone(),
                String::from_str(env, "Pagination Test"),
                String::from_str(env, "Desc"),
                None,
                None,
                0,
                None,
            )
            .unwrap()
        });
        env.mock_all_auths();
        as_core_contract(&env, &contract_id, |env| {
            HuntyCore::add_clue(env.clone(), hunt_id, question.clone(), answer.clone(), 10, true, None)
                .unwrap();
            HuntyCore::activate_hunt(env.clone(), hunt_id, creator.clone()).unwrap();
        });

        // Create 10 players
        let num_players = 10;
        let mut players = Vec::new(&env);
        for i in 0..num_players {
            let player = Address::generate(&env);
            players.push_back(player.clone());
            env.mock_all_auths();
            as_core_contract(&env, &contract_id, |env| {
                HuntyCore::register_player(env.clone(), hunt_id, player.clone()).unwrap();
            });
            // Make all complete, with different scores (higher i = higher score)
            env.ledger().set_timestamp(1_700_000_000 + i as u64 + 1);
            env.mock_all_auths();
            as_core_contract(&env, &contract_id, |env| {
                submit_answer(env, hunt_id, 1, player.clone(), answer.clone(), i as u64 + 1).unwrap();
            });
        }

        // Get leaderboard with limit 5
        let board_5 = as_core_contract(&env, &contract_id, |env| {
            HuntyCore::get_hunt_leaderboard(env.clone(), hunt_id, 5).unwrap()
        });
        assert_eq!(board_5.len(), 5);

        // Get leaderboard with limit 10
        let board_10 = as_core_contract(&env, &contract_id, |env| {
            HuntyCore::get_hunt_leaderboard(env.clone(), hunt_id, 10).unwrap()
        });
        assert_eq!(board_10.len(), 10);

        // Verify that the first 5 of board_10 match board_5 exactly
        for i in 0..5 {
            let entry_5 = board_5.get(i).unwrap();
            let entry_10 = board_10.get(i).unwrap();
            assert_eq!(entry_5.rank, entry_10.rank);
            assert_eq!(entry_5.player, entry_10.player);
            assert_eq!(entry_5.score, entry_10.score);
            assert_eq!(entry_5.completed_at, entry_10.completed_at);
        }
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
                0,
                None,
            )
            .unwrap();
            HuntyCore::add_clue(env.clone(), hunt_id, question, answer, 1, true, 1).unwrap();
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

        let contract_id = env.register_contract(None, super::HuntyCore);
        let hunt_id = as_core_contract(&env, &contract_id, |env| {
            HuntyCore::create_hunt(
                env.clone(),
                creator.clone(),
                String::from_str(env, "Hunt"),
                String::from_str(env, "Desc"),
                None,
                None,
                0,
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
                true, 1)
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
            HuntyCore::submit_answer(env.clone(), hunt_id, 1, player1.clone(), answer.clone())
                .unwrap();
        });
        env.mock_all_auths();
        as_core_contract(&env, &contract_id, |env| {
            HuntyCore::submit_answer(env.clone(), hunt_id, 1, player2.clone(), answer.clone())
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
        let contract_id = env.register_contract(None, super::HuntyCore);
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
                0,
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
                true, 1)
            .unwrap();

            // Update reward config on the hunt
            let mut hunt = Storage::get_hunt(env, hunt_id).unwrap();
            hunt.reward_config =
                crate::types::HuntRewardConfig::new(env, xlm_pool, false, None, max_winners, 0, 0);
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
            HuntyCore::submit_answer(env.clone(), hunt_id, 1, player.clone(), answer.clone())
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
        let core_id = env.register_contract(None, super::HuntyCore);
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
                0,
                None,
            )
            .unwrap();

            HuntyCore::add_clue(
                env.clone(),
                hunt_id,
                SorobanString::from_str(env, "What is 1+1?"),
                SorobanString::from_str(env, "2"),
                10,
                true, 1)
            .unwrap();

            // Configure rewards on the hunt: 3 winners sharing 9_000 XLM
            let mut hunt = Storage::get_hunt(env, hunt_id).unwrap();
            hunt.reward_config = crate::types::HuntRewardConfig::new(
                env,
                9_000,
                true,
                Some(nft_contract_id.clone()),
                3,
                0,
                0,
            );
            Storage::save_hunt(env, &hunt);

            HuntyCore::activate_hunt(env.clone(), hunt_id, creator.clone()).unwrap();

            hunt_id
        });

        // Fund RewardManager pool for this hunt
        env.mock_all_auths();
        env.as_contract(&reward_manager_id, || {
            RewardManager::create_reward_pool(env.clone(), funder.clone(), hunt_id, 0).unwrap();
        });
        env.mock_all_auths();
        env.as_contract(&reward_manager_id, || {
            RewardManager::fund_reward_pool(env.clone(), funder.clone(), hunt_id, 9_000).unwrap();
        });

        // Wire HuntyCore -> RewardManager
        env.mock_all_auths();
        as_core_contract(&env, &core_id, |env| {
            HuntyCore::set_reward_manager(env.clone(), creator.clone(), reward_manager_id.clone());
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
                1,
                env.ledger().timestamp(),
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
        let nft_client = nft_reward::NftRewardClient::new(&env, &nft_contract_id);
        let owned_nfts = nft_client.get_player_nfts(&player, &0, &100);
        assert!(owned_nfts.len() >= 1);
        assert!(owned_nfts.iter().any(|id| id == minted_nft_id));

        let nft = nft_client.get_nft(&minted_nft_id).unwrap();
        assert_eq!(nft.hunt_id, hunt_id);
        assert_eq!(nft.owner, player);
        assert_eq!(
            nft.metadata.title,
            SorobanString::from_str(&env, "Integrated Hunt")
        );
        assert_eq!(nft.metadata.description, SorobanString::from_str(&env, ""));
    }

    #[test]
    fn test_complete_hunt_uses_reward_manager_pool_balance_when_local_pool_is_zero() {
        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);

        let creator = Address::generate(&env);
        let player = Address::generate(&env);
        let funder = Address::generate(&env);

        let (reward_manager_id, token_address, token_admin) = setup_reward_manager(&env, None);
        let core_id = env.register_contract(None, super::HuntyCore);

        let hunt_id = as_core_contract(&env, &core_id, |env| {
            let hunt_id = HuntyCore::create_hunt(
                env.clone(),
                creator.clone(),
                SorobanString::from_str(env, "Pool-backed hunt"),
                SorobanString::from_str(env, "Uses reward manager balance"),
                None,
                None,
            )
            .unwrap();

            HuntyCore::add_clue(
                env.clone(),
                hunt_id,
                SorobanString::from_str(env, "1+1?"),
                SorobanString::from_str(env, "2"),
                10,
                true, 1)
            .unwrap();

            let mut hunt = Storage::get_hunt(env, hunt_id).unwrap();
            hunt.reward_config = crate::types::HuntRewardConfig::new(env, 0, false, None, 3, 0, 0);
            Storage::save_hunt(env, &hunt);

            HuntyCore::activate_hunt(env.clone(), hunt_id, creator.clone()).unwrap();

            hunt_id
        });

        let token_client = token::StellarAssetClient::new(&env, &token_address);
        token_client.mint(&funder, &9_000);
        let _ = token_admin;

        env.as_contract(&reward_manager_id, || {
            RewardManager::create_reward_pool(env.clone(), funder.clone(), hunt_id, 0).unwrap();
        });
        env.mock_all_auths();
        env.as_contract(&reward_manager_id, || {
            RewardManager::fund_reward_pool(env.clone(), funder.clone(), hunt_id, 9_000).unwrap();
        });

        env.mock_all_auths();
        as_core_contract(&env, &core_id, |env| {
            HuntyCore::set_reward_manager(env.clone(), creator.clone(), reward_manager_id.clone());
        });

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
        env.mock_all_auths();
        as_core_contract(&env, &core_id, |env| {
            HuntyCore::complete_hunt(env.clone(), hunt_id, player.clone()).unwrap();
        });

        let player_balance = token::Client::new(&env, &token_address).balance(&player);
        assert_eq!(player_balance, 3_000);

        env.as_contract(&reward_manager_id, || {
            assert_eq!(RewardManager::get_pool_balance(env.clone(), hunt_id), 6_000);
        });

        let hunt = as_core_contract(&env, &core_id, |env| {
            HuntyCore::get_hunt_info(env.clone(), hunt_id).unwrap()
        });
        assert_eq!(hunt.reward_config.xlm_pool, 9_000);
    }

    #[test]
    fn test_get_hunt_info_syncs_reward_pool_balance_from_manager() {
        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);
        env.mock_all_auths();

        let creator = Address::generate(&env);
        let funder = Address::generate(&env);
        let core_id = env.register_contract(None, super::HuntyCore);
        let (reward_manager_id, token_address, _) = setup_reward_manager(&env, None);

        let hunt_id = as_core_contract(&env, &core_id, |env| {
            let hunt_id = HuntyCore::create_hunt(
                env.clone(),
                creator.clone(),
                SorobanString::from_str(env, "Synced Hunt"),
                SorobanString::from_str(env, "Should sync pool balance"),
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
                true, 1)
            .unwrap();

            let mut hunt = Storage::get_hunt(env, hunt_id).unwrap();
            hunt.reward_config = crate::types::HuntRewardConfig::new(env, 0, false, None, 3, 0, 0);
            Storage::save_hunt(env, &hunt);

            HuntyCore::activate_hunt(env.clone(), hunt_id, creator.clone()).unwrap();
            hunt_id
        });

        env.as_contract(&reward_manager_id, || {
            RewardManager::create_reward_pool(env.clone(), funder.clone(), hunt_id, 0).unwrap();
        });
        env.mock_all_auths();
        env.as_contract(&reward_manager_id, || {
            RewardManager::fund_reward_pool(env.clone(), funder.clone(), hunt_id, 9_000).unwrap();
        });

        env.mock_all_auths();
        as_core_contract(&env, &core_id, |env| {
            HuntyCore::set_reward_manager(env.clone(), creator.clone(), reward_manager_id.clone());
        });

        let hunt = as_core_contract(&env, &core_id, |env| {
            HuntyCore::get_hunt_info(env.clone(), hunt_id).unwrap()
        });

        assert_eq!(hunt.reward_config.xlm_pool, 9_000);
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
            HuntyCore::set_reward_manager(env.clone(), creator.clone(), reward_manager_id.clone());
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
        let core_id = env.register_contract(None, super::HuntyCore);
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
                0,
                None,
            )
            .unwrap();

            HuntyCore::add_clue(
                env.clone(),
                hunt_id,
                SorobanString::from_str(env, "What is 1+1?"),
                SorobanString::from_str(env, "2"),
                10,
                true, 1)
            .unwrap();

            // Configure rewards: xlm_pool = 6_000, max_winners = 3
            let mut hunt = Storage::get_hunt(env, hunt_id).unwrap();
            hunt.reward_config = crate::types::HuntRewardConfig::new(
                env,
                6_000,
                true,
                Some(nft_contract_id.clone()),
                3,
                0,
                0,
            );
            Storage::save_hunt(env, &hunt);

            HuntyCore::activate_hunt(env.clone(), hunt_id, creator.clone()).unwrap();

            hunt_id
        });

        // Fund RewardManager pool
        env.mock_all_auths();
        env.as_contract(&reward_manager_id, || {
            RewardManager::create_reward_pool(env.clone(), funder.clone(), hunt_id, 0).unwrap();
        });
        env.mock_all_auths();
        env.as_contract(&reward_manager_id, || {
            RewardManager::fund_reward_pool(env.clone(), funder.clone(), hunt_id, 6_000).unwrap();
        });

        // Wire HuntyCore -> RewardManager
        env.mock_all_auths();
        as_core_contract(&env, &core_id, |env| {
            HuntyCore::set_reward_manager(env.clone(), creator.clone(), reward_manager_id.clone());
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
                    1,
                    env.ledger().timestamp(),
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
        let nfts1 = nft_client.get_player_nfts(&player1, &0, &100);
        let nfts2 = nft_client.get_player_nfts(&player2, &0, &100);
        let nfts3 = nft_client.get_player_nfts(&player3, &0, &100);
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
    fn test_batch_complete_hunt_success() {
        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);
        let creator = Address::generate(&env);
        let player1 = Address::generate(&env);
        let player2 = Address::generate(&env);
        let player3 = Address::generate(&env);

        let contract_id = env.register_contract(None, super::HuntyCore);

        // Setup hunt and players
        env.mock_all_auths();
        let hunt_id = as_core_contract(&env, &contract_id, |env| {
            let hid = HuntyCore::create_hunt(
                env.clone(),
                creator.clone(),
                String::from_str(env, "Batch Hunt"),
                String::from_str(env, "Desc"),
                None,
                None,
                0,
                None,
            )
            .unwrap();

            HuntyCore::add_clue(
                env.clone(),
                hid,
                String::from_str(env, "Q"),
                String::from_str(env, "a"),
                10,
                true,
                1,
            )
            .unwrap();

            let mut hunt = Storage::get_hunt(env, hid).unwrap();
            hunt.reward_config = crate::types::HuntRewardConfig::new(env, 1000, false, None, 10, 0, 0);
            Storage::save_hunt(env, &hunt);

            HuntyCore::activate_hunt(env.clone(), hid, creator.clone()).unwrap();
            hid
        });

        // Register and complete for all players
        for p in [&player1, &player2, &player3] {
            env.mock_all_auths();
            as_core_contract(&env, &contract_id, |env| {
                HuntyCore::register_player(env.clone(), hunt_id, (*p).clone()).unwrap();
            });
            env.mock_all_auths();
            as_core_contract(&env, &contract_id, |env| {
                HuntyCore::submit_answer(
                    env.clone(),
                    hunt_id,
                    1,
                    (*p).clone(),
                    String::from_str(env, "a"),
                    1,
                    env.ledger().timestamp(),
                )
                .unwrap();
            });
        }

        // Batch complete by creator
        env.mock_all_auths();
        as_core_contract(&env, &contract_id, |env| {
            let players = Vec::from_array(env, [player1.clone(), player2.clone(), player3.clone()]);
            HuntyCore::batch_complete_hunt(env.clone(), hunt_id, creator.clone(), players).unwrap();
        });

        // Verify all players claimed
        for p in [player1, player2, player3] {
            let progress = as_core_contract(&env, &contract_id, |env| {
                HuntyCore::get_player_progress(env.clone(), hunt_id, p).unwrap()
            });
            assert!(progress.reward_claimed);
        }

        // Verify hunt claimed_count
        let hunt = as_core_contract(&env, &contract_id, |env| {
            HuntyCore::get_hunt_info(env.clone(), hunt_id).unwrap()
        });
        assert_eq!(hunt.reward_config.claimed_count, 3);
    }

    #[test]
    fn test_batch_complete_hunt_mixed_success_failure() {
        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);
        let creator = Address::generate(&env);
        let player_a = Address::generate(&env);
        let player_b = Address::generate(&env); // not registered
        let player_c = Address::generate(&env);
        let player_d = Address::generate(&env);

        let contract_id = env.register_contract(None, super::HuntyCore);

        let hunt_id = as_core_contract(&env, &contract_id, |env| {
            let hid = HuntyCore::create_hunt(
                env.clone(),
                creator.clone(),
                String::from_str(env, "Batch Hunt"),
                String::from_str(env, "Desc"),
                None,
                None,
            )
            .unwrap();

            HuntyCore::add_clue(
                env.clone(),
                hid,
                String::from_str(env, "Q"),
                String::from_str(env, "a"),
                10,
                true, 1)
            .unwrap();

            let mut hunt = Storage::get_hunt(env, hid).unwrap();
            hunt.reward_config = crate::types::HuntRewardConfig::new(env, 1000, false, None, 10, 0, 0);
            Storage::save_hunt(env, &hunt);

            HuntyCore::activate_hunt(env.clone(), hid, creator.clone()).unwrap();
            hid
        });

        env.mock_all_auths();
        as_core_contract(&env, &contract_id, |env| {
            for p in [&player_a, &player_c, &player_d] {
                HuntyCore::register_player(env.clone(), hunt_id, (*p).clone()).unwrap();
                HuntyCore::submit_answer(
                    env.clone(),
                    hunt_id,
                    1,
                    (*p).clone(),
                    String::from_str(env, "a"),
                )
                .unwrap();
            }
        });

        env.mock_all_auths();
        as_core_contract(&env, &contract_id, |env| {
            HuntyCore::complete_hunt(env.clone(), hunt_id, player_c.clone()).unwrap();
        });

        env.mock_all_auths();
        as_core_contract(&env, &contract_id, |env| {
            let players = Vec::from_array(
                env,
                [
                    player_a.clone(),
                    player_b.clone(), // not registered
                    player_c.clone(), // already claimed
                    player_d.clone(),
                ],
            );
            HuntyCore::batch_complete_hunt(env.clone(), hunt_id, creator.clone(), players).unwrap();
        });

        for p in [player_a, player_d] {
            let progress = as_core_contract(&env, &contract_id, |env| {
                HuntyCore::get_player_progress(env.clone(), hunt_id, p).unwrap()
            });
            assert!(progress.reward_claimed);
        }

        let err = as_core_contract(&env, &contract_id, |env| {
            HuntyCore::get_player_progress(env.clone(), hunt_id, player_b.clone()).unwrap_err()
        });
        assert_eq!(err, HuntErrorCode::PlayerNotRegistered);

        let failure_topic = Symbol::new(&env, "RewardClaimFailed");
        let events = env.events().all();
        let mut failure_events = 0;
        let mut saw_unregistered_player = false;
        let mut saw_already_claimed_player = false;

        for i in 0..events.len() {
            let (_contract, topics, data) = events.get(i).unwrap();
            let topic: Symbol = topics.get(0).unwrap().try_into_val(&env).unwrap();
            if topic == failure_topic {
                failure_events += 1;
                let event: RewardClaimFailedEvent = data.try_into_val(&env).unwrap();
                assert_eq!(event.hunt_id, hunt_id);

                if event.player == player_b {
                    assert_eq!(event.error_code, HuntErrorCode::PlayerNotRegistered as u32);
                    saw_unregistered_player = true;
                } else if event.player == player_c {
                    assert_eq!(event.error_code, HuntErrorCode::RewardAlreadyClaimed as u32);
                    saw_already_claimed_player = true;
                } else {
                    panic!("unexpected RewardClaimFailedEvent player");
                }
            }
        }

        assert_eq!(failure_events, 2);
        assert!(saw_unregistered_player);
        assert!(saw_already_claimed_player);

        let hunt = as_core_contract(&env, &contract_id, |env| {
            HuntyCore::get_hunt_info(env.clone(), hunt_id).unwrap()
        });
        assert_eq!(hunt.reward_config.claimed_count, 3);
    }

    #[test]
    fn test_complete_hunt_not_completed() {
        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);
        let creator = Address::generate(&env);
        let player = Address::generate(&env);
        let contract_id = env.register_contract(None, super::HuntyCore);

        // Create hunt with 2 required clues
        let hunt_id = as_core_contract(&env, &contract_id, |env| {
            HuntyCore::create_hunt(
                env.clone(),
                creator.clone(),
                String::from_str(env, "Hunt"),
                String::from_str(env, "Desc"),
                None,
                None,
                0,
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
                true, 1)
            .unwrap();
            HuntyCore::add_clue(
                env.clone(),
                hunt_id,
                String::from_str(env, "Q2"),
                String::from_str(env, "a2"),
                10,
                true, 1)
            .unwrap();

            let mut hunt = Storage::get_hunt(env, hunt_id).unwrap();
            hunt.reward_config = crate::types::HuntRewardConfig::new(env, 1000, false, None, 5, 0, 0);
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
                1,
                env.ledger().timestamp(),
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
                1,
                env.ledger().timestamp(),
            )
            .unwrap();
        });

        // Player2 tries to claim — no slots left (Hunt is now Completed)
        env.mock_all_auths();
        let result = as_core_contract(&env, &contract_id, |env| {
            HuntyCore::complete_hunt(env.clone(), hunt_id, player2.clone())
        });
        assert_eq!(result, Err(HuntErrorCode::InvalidHuntStatus));
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

    #[test]
    fn test_set_reward_manager_non_admin_fails() {
        let env = Env::default();
        env.mock_all_auths();
        env.ledger().set_timestamp(1_700_000_000);

        let admin = Address::generate(&env);
        let non_admin = Address::generate(&env);

        // Deploy HuntyCore
        let core_id = env.register_contract(None, super::HuntyCore);

        // Initialize admin
        as_core_contract(&env, &core_id, |env| {
            HuntyCore::initialize_admin(env.clone(), admin.clone()).unwrap();
        });

        // Deploy RewardManager
        let reward_manager_id = env.register(RewardManager, ());
        let token_admin = Address::generate(&env);
        let token_contract = env.register_stellar_asset_contract_v2(token_admin.clone());
        let token_address = token_contract.address();

        env.as_contract(&reward_manager_id, || {
            RewardManager::initialize(env.clone(), token_admin.clone(), token_address.clone())
                .unwrap();
        });

        // Non-admin tries to set RewardManager on HuntyCore.
        // Access control should cause Unauthorized failure.
        let result = as_core_contract(&env, &core_id, |env| {
            HuntyCore::set_reward_manager(env.clone(), non_admin.clone(), reward_manager_id.clone())
        });

        assert_eq!(result, Err(HuntErrorCode::Unauthorized));

        // Sanity: admin should be able to set (auth succeeds when invoker==admin)
        let ok = as_core_contract(&env, &core_id, |env| {
            HuntyCore::set_reward_manager(env.clone(), admin.clone(), reward_manager_id.clone())
        });
        assert_eq!(ok, Ok(()));
    }

    #[test]
    fn test_complete_hunt_invalid_status() {
        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);
        let creator = Address::generate(&env);
        let player = Address::generate(&env);

        let (hunt_id, contract_id) =
            setup_completed_hunt_with_rewards(&env, &creator, &player, 5, 1000);

#[test]
fn test_get_hunt_statistics_mixed_completion_states() {
    let env = Env::default();
    env.ledger().set_timestamp(1_700_000_000);

    let creator = Address::generate(&env);
    let player1 = Address::generate(&env);
    let player2 = Address::generate(&env);
    let player3 = Address::generate(&env);
    let question = String::from_str(&env, "Q");
    let answer = String::from_str(&env, "a");

    // Register contract and create hunt
    let contract_id = env.register(super::HuntyCore, ());
    let hunt_id = execute_in_contract(&env, &contract_id, |env| {
        HuntyCore::create_hunt(
            env.clone(),
            creator.clone(),
            String::from_str(env, "Mixed Hunt"),
            String::from_str(env, "Desc"),
            None,
            None,
        )
        .unwrap()
    });

    // Add a single required clue worth 10 points and activate
    env.mock_all_auths();
    execute_in_contract(&env, &contract_id, |env| {
        HuntyCore::add_clue(
            env.clone(),
            hunt_id,
            question.clone(),
            answer.clone(),
            10,
            true, 1)
        .unwrap();
        HuntyCore::activate_hunt(env.clone(), hunt_id, creator.clone()).unwrap();
    });

    // Register three players
    env.mock_all_auths();
    execute_in_contract(&env, &contract_id, |env| {
        HuntyCore::register_player(env.clone(), hunt_id, player1.clone()).unwrap();
    });
    env.mock_all_auths();
    execute_in_contract(&env, &contract_id, |env| {
        HuntyCore::register_player(env.clone(), hunt_id, player2.clone()).unwrap();
    });
    env.mock_all_auths();
    execute_in_contract(&env, &contract_id, |env| {
        HuntyCore::register_player(env.clone(), hunt_id, player3.clone()).unwrap();
    });

    // Player1 and Player2 solve the required clue
    env.mock_all_auths();
    execute_in_contract(&env, &contract_id, |env| {
        HuntyCore::submit_answer(env.clone(), hunt_id, 1, player1.clone(), answer.clone()).unwrap();
    });
    env.mock_all_auths();
    execute_in_contract(&env, &contract_id, |env| {
        HuntyCore::submit_answer(env.clone(), hunt_id, 1, player2.clone(), answer.clone()).unwrap();
    });

    // Player3 remains incomplete (no submissions)

    // Fetch statistics and validate exact invariants
    let stats = execute_in_contract(&env, &contract_id, |env| {
        HuntyCore::get_hunt_statistics(env.clone(), hunt_id).unwrap()
    });

    // 3 players total, 2 completed -> floor(2/3*100) == 66
    assert_eq!(stats.total_players, 3);
    assert_eq!(stats.completed_count, 2);
    assert_eq!(stats.completion_rate_percent, 66);

    // Two players solved the single 10-point required clue => total 20
    // Average must be computed over all 3 participants: floor(20 / 3) == 6
    assert_eq!(stats.total_score_sum, 20);
    assert_eq!(stats.average_score, 6);
}

        // Try to complete the hunt — should fail with InvalidHuntStatus
        env.mock_all_auths();
        let result = as_core_contract(&env, &contract_id, |env| {
            HuntyCore::complete_hunt(env.clone(), hunt_id, player.clone())
        });
        assert_eq!(result, Err(HuntErrorCode::InvalidHuntStatus));
    }

    #[test]
    fn test_reward_per_winner_when_pool_less_than_winners() {
        let env = Env::default();
        let config = crate::types::HuntRewardConfig::new(&env, 5, false, None, 10, 0, 0);
        let amount = config.reward_per_winner();
        assert_eq!(
            amount, 0,
            "xlm_pool=5 / max_winners=10 must be 0 (integer division)"
        );
    }

    #[test]
    fn test_reward_per_winner_zero_max_winners() {
        let env = Env::default();
        let config = crate::types::HuntRewardConfig::new(&env, 100, false, None, 0, 0, 0);
        let amount = config.reward_per_winner();
        assert_eq!(amount, 0, "max_winners=0 must return 0");
    }

    #[test]
    fn test_reward_per_winner_exact_division() {
        let env = Env::default();
        let config = crate::types::HuntRewardConfig::new(&env, 100, false, None, 10, 0, 0);
        let amount = config.reward_per_winner();
        assert_eq!(amount, 10, "xlm_pool=100 / max_winners=10 must be 10");
    }

    #[test]
    fn test_reward_per_winner_rounds_down() {
        let env = Env::default();
        let config = crate::types::HuntRewardConfig::new(&env, 7, false, None, 3, 0, 0);
        let amount = config.reward_per_winner();
        assert_eq!(amount, 2, "xlm_pool=7 / max_winners=3 must round down to 2");
    }

    // ========== Score Calculation Invariants Tests ==========
    #[test]
    fn test_score_calculation_invariants() {
        use crate::types::{Clue, Hunt};
        use crate::HuntyCore;
        use soroban_sdk::Env;

        let env = Env::default();

        // Test 1: Score is always non-negative
        let hunt = Hunt {
            hunt_id: 1,
            creator: soroban_sdk::Address::generate(&env),
            title: soroban_sdk::String::from_str(&env, "Test"),
            description: soroban_sdk::String::from_str(&env, "Test"),
            status: crate::types::HuntStatus::Active,
            created_at: 0,
            activated_at: 0,
            end_time: 0,
            reward_config: crate::types::RewardConfig::new(0, false, None, 0),
            total_clues: 0,
            required_clues: 0,
            completed_count: 0,
            max_submissions_per_minute: 0,
            start_multiplier_bps: 20000,
        };

        let clue = Clue {
            clue_id: 1,
            question: soroban_sdk::String::from_str(&env, "Q"),
            answer_hash: soroban_sdk::BytesN::from_array(&env, &[0u8; 32]),
            points: 10,
            is_required: true,
            difficulty: 1,
        };

        let score1 = HuntyCore::calculate_score(&hunt, &clue, 0, 0);
        assert!(score1 >= 0, "Score must be non-negative");

        let score2 = HuntyCore::calculate_score(&hunt, &clue, 0, 1000);
        assert!(score2 >= 0, "Score must be non-negative even with large time");

        // Test 2: Higher difficulty always means higher score (same time)
        let clue_easy = Clue { difficulty: 1, ..clue.clone() };
        let clue_hard = Clue { difficulty: 5, ..clue.clone() };
        let score_easy = HuntyCore::calculate_score(&hunt, &clue_easy, 0, 50);
        let score_hard = HuntyCore::calculate_score(&hunt, &clue_hard, 0, 50);
        assert!(score_hard > score_easy, "Higher difficulty must yield higher score");

        // Test 3: Time bonus never exceeds start multiplier
        let score_at_start = HuntyCore::calculate_score(&hunt, &clue, 0, 0);
        let base_with_difficulty = clue.points * clue.difficulty;
        let max_possible_score = base_with_difficulty * hunt.start_multiplier_bps / 10000;
        assert_eq!(score_at_start, max_possible_score, "Score at start must be max possible");

        let score_later = HuntyCore::calculate_score(&hunt, &clue, 0, 100);
        assert!(score_later <= max_possible_score, "Later scores must not exceed start bonus");

        // Test 4: (Unit test for sum) Progress total score should sum clues
        // We test this via contract interaction
        let contract_id = env.register(HuntyCore, ());
        let creator = soroban_sdk::Address::generate(&env);
        let player = soroban_sdk::Address::generate(&env);

        env.mock_all_auths();
        as_core_contract(&env, &contract_id, |env| {
            let hunt_id = HuntyCore::create_hunt(
                env.clone(),
                creator.clone(),
                soroban_sdk::String::from_str(env, "Test"),
                soroban_sdk::String::from_str(env, "Test"),
                None,
                None,
                0,
                Some(20000),
            ).unwrap();

            HuntyCore::add_clue(
                env.clone(),
                hunt_id,
                soroban_sdk::String::from_str(env, "Q1"),
                soroban_sdk::String::from_str(env, "A1"),
                10,
                true,
                Some(1),
            ).unwrap();

            HuntyCore::add_clue(
                env.clone(),
                hunt_id,
                soroban_sdk::String::from_str(env, "Q2"),
                soroban_sdk::String::from_str(env, "A2"),
                10,
                false,
                Some(1),
            ).unwrap();

            HuntyCore::activate_hunt(env.clone(), hunt_id, creator.clone()).unwrap();
            HuntyCore::register_player(env.clone(), hunt_id, player.clone()).unwrap();

            let time = env.ledger().timestamp();
            submit_answer(env.clone(), hunt_id, 1, player.clone(), soroban_sdk::String::from_str(env, "A1"), 1).unwrap();
            let progress1 = HuntyCore::get_player_progress(env.clone(), hunt_id, player.clone()).unwrap();
            let score_clue1 = progress1.total_score;

            submit_answer(env.clone(), hunt_id, 2, player.clone(), soroban_sdk::String::from_str(env, "A2"), 2).unwrap();
            let progress2 = HuntyCore::get_player_progress(env.clone(), hunt_id, player.clone()).unwrap();
            let total = progress2.total_score;

            // Since both submitted at same time (same multiplier), total should be sum of individual scores
            assert_eq!(total, score_clue1 * 2, "Total score should be sum of clue scores");
        });
    }

    // ========== Fuzz Tests for Answer Validation ==========
    #[test]
    fn fuzz_answer_validation() {
        use crate::sanitization::StringSanitizer;
        use soroban_sdk::{Env, String};

        let env = Env::default();

        // Test 1: Boundary lengths
        // Test empty string
        let empty = String::from_str(&env, "");
        let res_empty = StringSanitizer::sanitize(&env, &empty, 256, false);
        assert!(res_empty.is_err());

        // Test exactly max length
        let max_str = "a".repeat(256);
        let max_input = String::from_str(&env, &max_str);
        let res_max = StringSanitizer::sanitize(&env, &max_input, 256, false);
        assert!(res_max.is_ok());

        // Test over max length
        let over_str = "a".repeat(257);
        let over_input = String::from_str(&env, &over_str);
        let res_over = StringSanitizer::sanitize(&env, &over_input, 256, false);
        assert!(res_over.is_err());

        // Test 2: Special characters
        let special_chars = [
            "test\nwith\nnewlines",
            "test\r\nwith\r\ncrlf",
            "test\twith\ttabs",
            "test with spaces   ",
            "test@#$%^&*()_+",
            "test with emoji 😊",
            "test with chinese 中文",
            "test with arabic العربية",
            "test with russian русский",
        ];
        for s in special_chars {
            let input = String::from_str(&env, s);
            let res = StringSanitizer::sanitize(&env, &input, 256, false);
            assert!(res.is_ok());
        }

        // Test 3: Disallowed control characters
        let controls = [
            "\x00", // null
            "\x07", // bell
            "\x1B", // escape
            "\x08", // backspace
        ];
        for c in controls {
            let input = String::from_str(&env, &format!("test{}test", c));
            let res = StringSanitizer::sanitize(&env, &input, 256, false);
            assert!(res.is_err());
        }

        // Test 4: Normalize and hash should never panic
        use crate::HuntyCore;
        let safe_inputs = [
            "test",
            "   test   ",
            "TEST",
            "Test 123",
            "test with unicode 日本語",
            "test with spaces",
        ];
        for s in safe_inputs {
            let input = String::from_str(&env, s);
            let _ = HuntyCore::normalize_and_hash_answer(&env, 1, 1, &input);
        }

        // Test 5: Long strings
        let long_str = "x".repeat(2000);
        let long_input = String::from_str(&env, &long_str);
        let res = StringSanitizer::sanitize(&env, &long_input, 256, false);
        assert!(res.is_err());
    }

    // ========== Full Hunt Lifecycle Integration Tests ==========
    #[test]
    fn test_full_lifecycle_xlm_rewards() {
        use crate::types::{ClueAddedEvent, ClueCompletedEvent, HuntActivatedEvent, HuntCompletedEvent, PlayerRegisteredEvent};
        use soroban_sdk::testutils::Events as _;

        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);
        let creator = Address::generate(&env);
        let player = Address::generate(&env);
        let funder = Address::generate(&env);

        // Set up reward manager
        let (reward_manager_id, token_address, token_admin) = setup_reward_manager(&env, None);
        let sac_client = token::StellarAssetClient::new(&env, &token_address);
        let token_client = token::Client::new(&env, &token_address);
        sac_client.mint(&funder, &10_000);

        // Deploy hunty core
        let core_id = env.register(HuntyCore, ());

        // 1. Create hunt
        env.mock_all_auths();
        let hunt_id = as_core_contract(&env, &core_id, |env| {
            HuntyCore::create_hunt(
                env.clone(),
                creator.clone(),
                String::from_str(env, "XLM Only Hunt"),
                String::from_str(env, "Integration test hunt"),
                None,
                None,
                0,
                Some(20000),
            ).unwrap()
        });

        // 2. Add clues
        let q1 = String::from_str(&env, "2+2?");
        let a1 = String::from_str(&env, "4");
        let q2 = String::from_str(&env, "3*3?");
        let a2 = String::from_str(&env, "9");

        env.mock_all_auths();
        let clue1_id = as_core_contract(&env, &core_id, |env| {
            HuntyCore::add_clue(
                env.clone(),
                hunt_id,
                q1.clone(),
                a1.clone(),
                10,
                true,
                Some(1),
            ).unwrap()
        });

        env.mock_all_auths();
        let clue2_id = as_core_contract(&env, &core_id, |env| {
            HuntyCore::add_clue(
                env.clone(),
                hunt_id,
                q2.clone(),
                a2.clone(),
                20,
                false,
                Some(2),
            ).unwrap()
        });

        // Configure reward config
        as_core_contract(&env, &core_id, |env| {
            let mut hunt = Storage::get_hunt(env, hunt_id).unwrap();
            hunt.reward_config = crate::types::RewardConfig::new(6000, false, None, 2);
            Storage::save_hunt(env, &hunt);
        });

        // 3. Activate hunt
        env.mock_all_auths();
        as_core_contract(&env, &core_id, |env| {
            HuntyCore::activate_hunt(env.clone(), hunt_id, creator.clone()).unwrap();
        });

        // 4. Register player
        env.mock_all_auths();
        as_core_contract(&env, &core_id, |env| {
            HuntyCore::register_player(env.clone(), hunt_id, player.clone()).unwrap();
        });

        // 5. Submit answers
        env.mock_all_auths();
        as_core_contract(&env, &core_id, |env| {
            submit_answer(env.clone(), hunt_id, clue1_id, player.clone(), a1.clone(), 1).unwrap();
            submit_answer(env.clone(), hunt_id, clue2_id, player.clone(), a2.clone(), 2).unwrap();
        });

        // Set up reward pool
        env.as_contract(&reward_manager_id, || {
            RewardManager::create_reward_pool(env.clone(), funder.clone(), hunt_id, 0).unwrap();
        });
        env.mock_all_auths();
        env.as_contract(&reward_manager_id, || {
            RewardManager::fund_reward_pool(env.clone(), funder.clone(), hunt_id, 6000).unwrap();
        });

        as_core_contract(&env, &core_id, |env| {
            HuntyCore::set_reward_manager(env.clone(), reward_manager_id.clone());
        });

        // 6. Complete hunt (claim reward)
        env.mock_all_auths();
        as_core_contract(&env, &core_id, |env| {
            HuntyCore::complete_hunt(env.clone(), hunt_id, player.clone()).unwrap();
        });

        // Verify balances
        assert_eq!(token_client.balance(&player), 3000);
        assert_eq!(token_client.balance(&reward_manager_id), 3000);

        // Verify events
        let events = env.events().all();
        let event_symbols: Vec<_> = events.iter().map(|e| e.0.1).collect();

        assert!(event_symbols.contains(&symbol_short!("ClueAdded")));
        assert!(event_symbols.contains(&symbol_short!("HuntActivated")));
        assert!(event_symbols.contains(&symbol_short!("PlayerRegistered")));
        assert!(event_symbols.contains(&symbol_short!("ClueCompleted")));
        assert!(event_symbols.contains(&symbol_short!("HuntCompleted")));
    }

    #[test]
    fn test_full_lifecycle_nft_rewards() {
        use soroban_sdk::testutils::Events as _;
        use nft_reward::NftReward;

        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);
        let creator = Address::generate(&env);
        let player = Address::generate(&env);
        let funder = Address::generate(&env);

        let nft_contract_id = env.register(NftReward, ());

        let (reward_manager_id, token_address, token_admin) = setup_reward_manager(&env, Some(&nft_contract_id));

        let core_id = env.register(HuntyCore, ());

        env.mock_all_auths();
        let hunt_id = as_core_contract(&env, &core_id, |env| {
            HuntyCore::create_hunt(
                env.clone(),
                creator.clone(),
                String::from_str(env, "NFT Only Hunt"),
                String::from_str(env, "Test NFT rewards"),
                None,
                None,
                0,
                None,
            ).unwrap()
        });

        let q = String::from_str(&env, "2+2?");
        let a = String::from_str(&env, "4");
        env.mock_all_auths();
        let clue_id = as_core_contract(&env, &core_id, |env| {
            HuntyCore::add_clue(
                env.clone(),
                hunt_id,
                q.clone(),
                a.clone(),
                10,
                true,
                None,
            ).unwrap()
        });

        as_core_contract(&env, &core_id, |env| {
            let mut hunt = Storage::get_hunt(env, hunt_id).unwrap();
            hunt.reward_config = crate::types::RewardConfig::new(0, true, Some(nft_contract_id.clone()), 1);
            Storage::save_hunt(env, &hunt);
        });

        env.mock_all_auths();
        as_core_contract(&env, &core_id, |env| {
            HuntyCore::activate_hunt(env.clone(), hunt_id, creator.clone()).unwrap();
        });

        env.mock_all_auths();
        as_core_contract(&env, &core_id, |env| {
            HuntyCore::register_player(env.clone(), hunt_id, player.clone()).unwrap();
        });

        env.mock_all_auths();
        as_core_contract(&env, &core_id, |env| {
            submit_answer(env.clone(), hunt_id, clue_id, player.clone(), a.clone(), 1).unwrap();
        });

        env.as_contract(&reward_manager_id, || {
            RewardManager::create_reward_pool(env.clone(), funder.clone(), hunt_id, 0).unwrap();
        });

        as_core_contract(&env, &core_id, |env| {
            HuntyCore::set_reward_manager(env.clone(), reward_manager_id.clone());
        });

        env.mock_all_auths();
        as_core_contract(&env, &core_id, |env| {
            HuntyCore::complete_hunt(env.clone(), hunt_id, player.clone()).unwrap();
        });

        let nft_client = nft_reward::NftRewardClient::new(&env, &nft_contract_id);
        let player_nfts = nft_client.get_player_nfts(&player, &0, &10);
        assert_eq!(player_nfts.len(), 1);
    }

    #[test]
    fn test_full_lifecycle_both_rewards() {
        use soroban_sdk::testutils::Events as _;
        use nft_reward::NftReward;

        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);
        let creator = Address::generate(&env);
        let player = Address::generate(&env);
        let funder = Address::generate(&env);

        let nft_contract_id = env.register(NftReward, ());

        let (reward_manager_id, token_address, token_admin) = setup_reward_manager(&env, Some(&nft_contract_id));
        let sac_client = token::StellarAssetClient::new(&env, &token_address);
        let token_client = token::Client::new(&env, &token_address);
        sac_client.mint(&funder, &10_000);

        let core_id = env.register(HuntyCore, ());

        env.mock_all_auths();
        let hunt_id = as_core_contract(&env, &core_id, |env| {
            HuntyCore::create_hunt(
                env.clone(),
                creator.clone(),
                String::from_str(env, "Both Rewards Hunt"),
                String::from_str(env, "Test XLM + NFT"),
                None,
                None,
                0,
                Some(30000),
            ).unwrap()
        });

        let q1 = String::from_str(&env, "2+2?");
        let a1 = String::from_str(&env, "4");
        let q2 = String::from_str(&env, "3*3?");
        let a2 = String::from_str(&env, "9");

        env.mock_all_auths();
        as_core_contract(&env, &core_id, |env| {
            HuntyCore::add_clue(env.clone(), hunt_id, q1.clone(), a1.clone(), 10, true, Some(1)).unwrap();
            HuntyCore::add_clue(env.clone(), hunt_id, q2.clone(), a2.clone(), 20, false, Some(2)).unwrap();
        });

        as_core_contract(&env, &core_id, |env| {
            let mut hunt = Storage::get_hunt(env, hunt_id).unwrap();
            hunt.reward_config = crate::types::RewardConfig::new(8000, true, Some(nft_contract_id.clone()), 2);
            Storage::save_hunt(env, &hunt);
        });

        env.mock_all_auths();
        as_core_contract(&env, &core_id, |env| {
            HuntyCore::activate_hunt(env.clone(), hunt_id, creator.clone()).unwrap();
        });

        env.mock_all_auths();
        as_core_contract(&env, &core_id, |env| {
            HuntyCore::register_player(env.clone(), hunt_id, player.clone()).unwrap();
        });

        env.mock_all_auths();
        as_core_contract(&env, &core_id, |env| {
            submit_answer(env.clone(), hunt_id, 1, player.clone(), a1.clone(), 1).unwrap();
            submit_answer(env.clone(), hunt_id, 2, player.clone(), a2.clone(), 2).unwrap();
        });

        env.as_contract(&reward_manager_id, || {
            RewardManager::create_reward_pool(env.clone(), funder.clone(), hunt_id, 0).unwrap();
        });
        env.mock_all_auths();
        env.as_contract(&reward_manager_id, || {
            RewardManager::fund_reward_pool(env.clone(), funder.clone(), hunt_id, 8000).unwrap();
        });
        as_core_contract(&env, &core_id, |env| {
            HuntyCore::set_reward_manager(env.clone(), reward_manager_id.clone());
        });

        env.mock_all_auths();
        as_core_contract(&env, &core_id, |env| {
            HuntyCore::complete_hunt(env.clone(), hunt_id, player.clone()).unwrap();
        });

        assert_eq!(token_client.balance(&player), 4000);

        let nft_client = nft_reward::NftRewardClient::new(&env, &nft_contract_id);
        let player_nfts = nft_client.get_player_nfts(&player, &0, &10);
        assert_eq!(player_nfts.len(), 1);

        let events = env.events().all();
        let event_symbols: Vec<_> = events.iter().map(|e| e.0.1).collect();

        assert!(event_symbols.contains(&symbol_short!("ClueAdded")));
        assert!(event_symbols.contains(&symbol_short!("HuntActivated")));
        assert!(event_symbols.contains(&symbol_short!("PlayerRegistered")));
        assert!(event_symbols.contains(&symbol_short!("ClueCompleted")));
        assert!(event_symbols.contains(&symbol_short!("HuntCompleted")));
    }

    // ========== Storage-tier consistency tests (issue #84: TTL mismatch) ==========
    //
    // These tests guard against re-introducing instance storage for hunt/clue data.
    // Previously, Hunt structs and clue indexes lived in instance storage (shared
    // TTL) while player progress used persistent storage (per-key TTL).  If the
    // instance entry expired, all hunt/clue data was lost while player records
    // survived, causing permanent inconsistency.  All data must now live in
    // persistent storage so TTLs age together.

    /// Hunt data must remain readable after a player registers.
    /// In the buggy code, registering a player bumped only persistent TTLs; the
    /// instance entry could expire independently, making the hunt invisible.
    #[test]
    fn test_hunt_data_readable_after_player_registration() {
        let env = Env::default();
        env.mock_all_auths();
        env.ledger().set_timestamp(1_700_000_000);

        let creator = Address::generate(&env);
        let player = Address::generate(&env);
        let contract_id = env.register(HuntyCore, ());

        let hunt_id = as_core_contract(&env, &contract_id, |env| {
            HuntyCore::create_hunt(
                env.clone(),
                creator.clone(),
                String::from_str(env, "TTL Hunt"),
                String::from_str(env, "Hunt for TTL mismatch test"),
                None,
                None,
                0,
                None,
            )
            .unwrap()
        });

        as_core_contract(&env, &contract_id, |env| {
            HuntyCore::add_clue(
                env.clone(),
                hunt_id,
                String::from_str(env, "What is 2+2?"),
                String::from_str(env, "four"),
                10,
                true,
                None,
            )
            .unwrap();
            HuntyCore::activate_hunt(env.clone(), hunt_id, creator.clone()).unwrap();
            HuntyCore::register_player(env.clone(), hunt_id, player.clone()).unwrap();
        });

        // After player registration, hunt data must still be readable.
        as_core_contract(&env, &contract_id, |env| {
            let hunt = Storage::get_hunt(env, hunt_id).expect("hunt must survive player registration");
            assert_eq!(hunt.hunt_id, hunt_id);
            assert_eq!(hunt.status, HuntStatus::Active);
            assert_eq!(hunt.total_clues, 1);
        });
    }

    /// Clue index (previously in instance storage) must remain correct after
    /// player operations touch only persistent storage entries.
    #[test]
    fn test_clue_index_readable_after_player_submits_answer() {
        let env = Env::default();
        env.mock_all_auths();
        env.ledger().set_timestamp(1_700_000_000);

        let creator = Address::generate(&env);
        let player = Address::generate(&env);
        let contract_id = env.register(HuntyCore, ());

        let hunt_id = as_core_contract(&env, &contract_id, |env| {
            let hid = HuntyCore::create_hunt(
                env.clone(),
                creator.clone(),
                String::from_str(env, "Clue Index Hunt"),
                String::from_str(env, "Testing clue index persistence"),
                None,
                None,
                0,
                None,
            )
            .unwrap();
            HuntyCore::add_clue(
                env.clone(),
                hid,
                String::from_str(env, "What is the capital of France?"),
                String::from_str(env, "paris"),
                20,
                true,
                None,
            )
            .unwrap();
            HuntyCore::add_clue(
                env.clone(),
                hid,
                String::from_str(env, "What is 3 * 3?"),
                String::from_str(env, "nine"),
                10,
                false,
                None,
            )
            .unwrap();
            HuntyCore::activate_hunt(env.clone(), hid, creator.clone()).unwrap();
            HuntyCore::register_player(env.clone(), hid, player.clone()).unwrap();
            submit_answer(env, hid, 1, player.clone(), String::from_str(env, "paris"), 1).unwrap();
            hid
        });

        // Clue list query must still return both clues after the player submitted an answer.
        as_core_contract(&env, &contract_id, |env| {
            let clues = Storage::list_clues_for_hunt(env, hunt_id);
            assert_eq!(clues.len(), 2, "both clues must be in persistent index after player submission");
            let clue1 = Storage::get_clue(env, hunt_id, 1).expect("clue 1 must be readable");
            let clue2 = Storage::get_clue(env, hunt_id, 2).expect("clue 2 must be readable");
            assert_eq!(clue1.points, 20);
            assert_eq!(clue2.points, 10);
        });
    }

    /// Full end-to-end consistency: after every stage of a hunt lifecycle,
    /// hunt metadata, clue index, and player progress must all be readable.
    #[test]
    fn test_hunt_clue_and_player_data_consistent_across_full_lifecycle() {
        let env = Env::default();
        env.mock_all_auths();
        env.ledger().set_timestamp(1_700_000_000);

        let creator = Address::generate(&env);
        let player_a = Address::generate(&env);
        let player_b = Address::generate(&env);
        let contract_id = env.register(HuntyCore, ());

        let hunt_id = as_core_contract(&env, &contract_id, |env| {
            HuntyCore::create_hunt(
                env.clone(),
                creator.clone(),
                String::from_str(env, "Full Lifecycle Hunt"),
                String::from_str(env, "Consistency check across all stages"),
                None,
                None,
                0,
                None,
            )
            .unwrap()
        });

        // Stage 1: add clues — hunt and clue data both readable.
        as_core_contract(&env, &contract_id, |env| {
            HuntyCore::add_clue(env.clone(), hunt_id, String::from_str(env, "Q1"), String::from_str(env, "ans1"), 10, true, None).unwrap();
            HuntyCore::add_clue(env.clone(), hunt_id, String::from_str(env, "Q2"), String::from_str(env, "ans2"), 20, false, None).unwrap();
            HuntyCore::add_clue(env.clone(), hunt_id, String::from_str(env, "Q3"), String::from_str(env, "ans3"), 30, false, None).unwrap();
        });

        as_core_contract(&env, &contract_id, |env| {
            let hunt = Storage::get_hunt(env, hunt_id).unwrap();
            assert_eq!(hunt.total_clues, 3, "stage 1: hunt must report 3 clues");
            assert_eq!(Storage::list_clues_for_hunt(env, hunt_id).len(), 3, "stage 1: clue index must have 3 entries");
        });

        // Stage 2: activate and register two players.
        as_core_contract(&env, &contract_id, |env| {
            HuntyCore::activate_hunt(env.clone(), hunt_id, creator.clone()).unwrap();
            HuntyCore::register_player(env.clone(), hunt_id, player_a.clone()).unwrap();
            HuntyCore::register_player(env.clone(), hunt_id, player_b.clone()).unwrap();
        });

        as_core_contract(&env, &contract_id, |env| {
            let hunt = Storage::get_hunt(env, hunt_id).unwrap();
            assert_eq!(hunt.status, HuntStatus::Active, "stage 2: hunt must be active");
            assert_eq!(Storage::list_clues_for_hunt(env, hunt_id).len(), 3, "stage 2: clue index intact after registration");
            let prog_a = Storage::get_player_progress(env, hunt_id, &player_a).expect("player A must be registered");
            let prog_b = Storage::get_player_progress(env, hunt_id, &player_b).expect("player B must be registered");
            assert!(!prog_a.is_completed);
            assert!(!prog_b.is_completed);
        });

        // Stage 3: player A completes the required clue.
        as_core_contract(&env, &contract_id, |env| {
            submit_answer(env, hunt_id, 1, player_a.clone(), String::from_str(env, "ans1"), 1).unwrap();
        });

        // After player A's submission, hunt and clue data must be unchanged and readable.
        as_core_contract(&env, &contract_id, |env| {
            let hunt = Storage::get_hunt(env, hunt_id).unwrap();
            assert_eq!(hunt.total_clues, 3, "stage 3: hunt total_clues must not be mutated by player submission");
            assert_eq!(Storage::list_clues_for_hunt(env, hunt_id).len(), 3, "stage 3: clue index must be unchanged");
            let prog_a = Storage::get_player_progress(env, hunt_id, &player_a).unwrap();
            assert!(prog_a.total_score > 0, "stage 3: player A score must be > 0 after solving clue 1");
            let prog_b = Storage::get_player_progress(env, hunt_id, &player_b).unwrap();
            assert_eq!(prog_b.total_score, 0, "stage 3: player B score must still be 0");
        });
    }

    /// Multiple independent hunts must each maintain their own isolated clue
    /// indexes in persistent storage (no cross-contamination from shared instance).
    #[test]
    fn test_multiple_hunts_maintain_isolated_persistent_clue_indexes() {
        let env = Env::default();
        env.mock_all_auths();
        env.ledger().set_timestamp(1_700_000_000);

        let creator = Address::generate(&env);
        let contract_id = env.register(HuntyCore, ());

        let (hunt_a, hunt_b) = as_core_contract(&env, &contract_id, |env| {
            let a = HuntyCore::create_hunt(
                env.clone(),
                creator.clone(),
                String::from_str(env, "Hunt A"),
                String::from_str(env, "First hunt"),
                None,
                None,
                0,
                None,
            )
            .unwrap();
            let b = HuntyCore::create_hunt(
                env.clone(),
                creator.clone(),
                String::from_str(env, "Hunt B"),
                String::from_str(env, "Second hunt"),
                None,
                None,
                0,
                None,
            )
            .unwrap();
            // Hunt A gets 2 clues, Hunt B gets 1.
            HuntyCore::add_clue(env.clone(), a, String::from_str(env, "Q1"), String::from_str(env, "a1"), 5, true, None).unwrap();
            HuntyCore::add_clue(env.clone(), a, String::from_str(env, "Q2"), String::from_str(env, "a2"), 5, false, None).unwrap();
            HuntyCore::add_clue(env.clone(), b, String::from_str(env, "Q1"), String::from_str(env, "b1"), 15, true, None).unwrap();
            (a, b)
        });

        as_core_contract(&env, &contract_id, |env| {
            let clues_a = Storage::list_clues_for_hunt(env, hunt_a);
            let clues_b = Storage::list_clues_for_hunt(env, hunt_b);
            assert_eq!(clues_a.len(), 2, "Hunt A must have exactly 2 clues in its persistent index");
            assert_eq!(clues_b.len(), 1, "Hunt B must have exactly 1 clue in its persistent index");
            assert_eq!(Storage::get_clue_counter(env, hunt_a), 2);
            assert_eq!(Storage::get_clue_counter(env, hunt_b), 1);
        });
    }

    /// Hunt counter lives in persistent storage: creating hunts across multiple
    /// ledger calls must yield sequentially incrementing IDs.
    #[test]
    fn test_hunt_counter_increments_sequentially_in_persistent_storage() {
        let env = Env::default();
        env.mock_all_auths();
        env.ledger().set_timestamp(1_700_000_000);

        let creator = Address::generate(&env);
        let contract_id = env.register(HuntyCore, ());

        let mut ids = std::vec::Vec::<u64>::new();
        for _ in 0..5 {
            let id = as_core_contract(&env, &contract_id, |env| {
                HuntyCore::create_hunt(
                    env.clone(),
                    creator.clone(),
                    String::from_str(env, "Sequential Hunt"),
                    String::from_str(env, "Counter test"),
                    None,
                    None,
                    0,
                    None,
                )
                .unwrap()
            });
            ids.push(id);
        }

        for (i, id) in ids.iter().enumerate() {
            assert_eq!(*id, (i as u64) + 1, "hunt IDs must be sequential starting from 1");
        }

        as_core_contract(&env, &contract_id, |env| {
            assert_eq!(Storage::get_hunt_counter(env), 5, "persistent counter must reflect all 5 created hunts");
        });
    }

    // ========== Concurrent Player Simulation Tests ==========

    /// Test multiple players registering for the same hunt at the same timestamp
    #[test]
    fn test_multiple_players_register_simultaneously() {
        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);
        let creator = Address::generate(&env);
        let contract_id = env.register(HuntyCore, ());

        let hunt_id = as_core_contract(&env, &contract_id, |env| {
            HuntyCore::create_hunt(
                env.clone(),
                creator.clone(),
                String::from_str(env, "Concurrent Registration Test"),
                String::from_str(env, "Test simultaneous registrations"),
                None,
                None,
                0,
                None,
            )
            .unwrap()
        });
        env.mock_all_auths();
        as_core_contract(&env, &contract_id, |env| {
            HuntyCore::add_clue(
                env.clone(),
                hunt_id,
                String::from_str(env, "Q"),
                String::from_str(env, "A"),
                10,
                true,
                None,
            )
            .unwrap();
            HuntyCore::activate_hunt(env.clone(), hunt_id, creator.clone()).unwrap();
        });

        // Simulate 20 players registering
        let num_players = 20;
        let mut players = Vec::new(&env);
        for _ in 0..num_players {
            let player = Address::generate(&env);
            players.push_back(player.clone());
            env.mock_all_auths();
            as_core_contract(&env, &contract_id, |env| {
                HuntyCore::register_player(env.clone(), hunt_id, player.clone()).unwrap();
            });
        }

        // Verify all players are registered
        as_core_contract(&env, &contract_id, |env| {
            for player in players.iter() {
                let progress = Storage::get_player_progress(env, hunt_id, player).unwrap();
                assert_eq!(progress.player, *player);
                assert!(!progress.is_completed);
            }
            let leaderboard = HuntyCore::get_hunt_leaderboard(env.clone(), hunt_id, 100).unwrap();
            assert_eq!(leaderboard.len(), num_players);
        });
    }

    /// Test multiple players submitting answers for the same clue at the same timestamp
    #[test]
    fn test_multiple_players_submit_answers_simultaneously() {
        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);
        let creator = Address::generate(&env);
        let question = String::from_str(&env, "Q");
        let answer = String::from_str(&env, "A");
        let contract_id = env.register(HuntyCore, ());

        let hunt_id = as_core_contract(&env, &contract_id, |env| {
            HuntyCore::create_hunt(
                env.clone(),
                creator.clone(),
                String::from_str(env, "Concurrent Answer Test"),
                String::from_str(env, "Test simultaneous answer submissions"),
                None,
                None,
                0,
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
                None,
            )
            .unwrap();
            HuntyCore::activate_hunt(env.clone(), hunt_id, creator.clone()).unwrap();
        });

        // Register 15 players, all submit answers
        let num_players = 15;
        let mut players = Vec::new(&env);
        for i in 0..num_players {
            let player = Address::generate(&env);
            players.push_back(player.clone());
            env.mock_all_auths();
            as_core_contract(&env, &contract_id, |env| {
                HuntyCore::register_player(env.clone(), hunt_id, player.clone()).unwrap();
            });
            env.mock_all_auths();
            as_core_contract(&env, &contract_id, |env| {
                submit_answer(env, hunt_id, 1, player.clone(), answer.clone(), i as u64 + 1).unwrap();
            });
        }

        // Verify all players have their progress recorded correctly
        as_core_contract(&env, &contract_id, |env| {
            for player in players.iter() {
                let progress = Storage::get_player_progress(env, hunt_id, player).unwrap();
                assert!(progress.is_completed);
                assert!(progress.total_score > 0);
            }
            let stats = HuntyCore::get_hunt_statistics(env.clone(), hunt_id).unwrap();
            assert_eq!(stats.completed_count, num_players);
            assert_eq!(stats.total_players, num_players);
        });
    }

    /// Test race condition scenario for reward claiming with max winners limit
    #[test]
    fn test_reward_claiming_race_condition() {
        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);
        let creator = Address::generate(&env);
        let question = String::from_str(&env, "Q");
        let answer = String::from_str(&env, "A");
        let contract_id = env.register(HuntyCore, ());

        let hunt_id = as_core_contract(&env, &contract_id, |env| {
            let id = HuntyCore::create_hunt(
                env.clone(),
                creator.clone(),
                String::from_str(env, "Reward Race Test"),
                String::from_str(env, "Test reward claiming with max winners"),
                None,
                None,
                0,
                None,
            )
            .unwrap();
            
            // Set up reward config with max 3 winners
            let mut hunt = Storage::get_hunt(env, id).unwrap();
            hunt.reward_config = crate::types::RewardConfig::new(0, false, None, 3);
            Storage::save_hunt(env, &hunt);
            id
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
                None,
            )
            .unwrap();
            HuntyCore::activate_hunt(env.clone(), hunt_id, creator.clone()).unwrap();
        });

        // Register 10 players, all complete the hunt
        let num_players = 10;
        let mut players = Vec::new(&env);
        for i in 0..num_players {
            let player = Address::generate(&env);
            players.push_back(player.clone());
            env.mock_all_auths();
            as_core_contract(&env, &contract_id, |env| {
                HuntyCore::register_player(env.clone(), hunt_id, player.clone()).unwrap();
            });
            env.ledger().set_timestamp(1_700_000_000 + i as u64 + 1);
            env.mock_all_auths();
            as_core_contract(&env, &contract_id, |env| {
                submit_answer(env, hunt_id, 1, player.clone(), answer.clone(), i as u64 + 1).unwrap();
            });
        }

        // Verify leaderboard ordering and max winners
        as_core_contract(&env, &contract_id, |env| {
            let leaderboard = HuntyCore::get_hunt_leaderboard(env.clone(), hunt_id, 10).unwrap();
            assert_eq!(leaderboard.len(), num_players);
            // First 3 players should have rank 1-3
            for i in 0..3 {
                let entry = leaderboard.get(i).unwrap();
                assert_eq!(entry.rank, i as u32 + 1);
            }
        });
    }

    /// Test state consistency after multiple concurrent-like operations
    #[test]
    fn test_concurrent_operations_state_consistency() {
        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);
        let creator = Address::generate(&env);
        let question = String::from_str(&env, "Q");
        let answer = String::from_str(&env, "A");
        let contract_id = env.register(HuntyCore, ());

        // Create and set up hunt
        let hunt_id = as_core_contract(&env, &contract_id, |env| {
            let id = HuntyCore::create_hunt(
                env.clone(),
                creator.clone(),
                String::from_str(env, "State Consistency Test"),
                String::from_str(env, "Test state after multiple operations"),
                None,
                None,
                0,
                None,
            )
            .unwrap();
            HuntyCore::add_clue(env.clone(), id, question.clone(), answer.clone(), 10, true, None).unwrap();
            HuntyCore::add_clue(env.clone(), id, String::from_str(env, "Q2"), String::from_str(env, "A2"), 20, false, None).unwrap();
            HuntyCore::activate_hunt(env.clone(), id, creator.clone()).unwrap();
            id
        });

        // 10 players perform mixed operations
        let num_players = 10;
        let mut players = Vec::new(&env);
        for i in 0..num_players {
            let player = Address::generate(&env);
            players.push_back(player.clone());
            env.mock_all_auths();
            as_core_contract(&env, &contract_id, |env| {
                HuntyCore::register_player(env.clone(), hunt_id, player.clone()).unwrap();
            });
            env.mock_all_auths();
            as_core_contract(&env, &contract_id, |env| {
                submit_answer(env, hunt_id, 1, player.clone(), answer.clone(), i as u64 + 1).unwrap();
            });
            if i % 2 == 0 {
                env.mock_all_auths();
                as_core_contract(&env, &contract_id, |env| {
                    submit_answer(env, hunt_id, 2, player.clone(), String::from_str(env, "A2"), i as u64 + 1).unwrap();
                });
            }
        }

        // Verify all state is consistent
        as_core_contract(&env, &contract_id, |env| {
            let hunt = Storage::get_hunt(env, hunt_id).unwrap();
            assert_eq!(hunt.total_clues, 2);
            
            let clues = Storage::list_clues_for_hunt(env, hunt_id);
            assert_eq!(clues.len(), 2);
            
            for player in players.iter() {
                let progress = Storage::get_player_progress(env, hunt_id, player).unwrap();
                assert!(progress.total_score >= 10);
            }
            
            let stats = HuntyCore::get_hunt_statistics(env.clone(), hunt_id).unwrap();
            assert_eq!(stats.total_players, num_players);
            assert_eq!(stats.completed_count, num_players);
        });
    }

    // ========== Blacklist Tests ==========

    #[test]
    fn test_set_admin_and_blacklist_creator() {
        let env = Env::default();
        env.mock_all_auths();
        let admin = Address::generate(&env);
        let creator = Address::generate(&env);
        let contract_id = env.register_contract(None, HuntyCore);

        as_core_contract(&env, &contract_id, |env| {
            HuntyCore::set_admin(env.clone(), admin.clone());
        });
        as_core_contract(&env, &contract_id, |env| {
            HuntyCore::blacklist_creator(env.clone(), admin.clone(), creator.clone()).unwrap();
        });
        let blacklisted = as_core_contract(&env, &contract_id, |env| {
            HuntyCore::is_blacklisted(env.clone(), creator.clone())
        });
        assert!(blacklisted);
    }

    #[test]
    fn test_remove_from_blacklist() {
        let env = Env::default();
        env.mock_all_auths();
        let admin = Address::generate(&env);
        let creator = Address::generate(&env);
        let contract_id = env.register_contract(None, HuntyCore);

        as_core_contract(&env, &contract_id, |env| {
            HuntyCore::set_admin(env.clone(), admin.clone());
        });
        as_core_contract(&env, &contract_id, |env| {
            HuntyCore::blacklist_creator(env.clone(), admin.clone(), creator.clone()).unwrap();
        });
        as_core_contract(&env, &contract_id, |env| {
            HuntyCore::remove_from_blacklist(env.clone(), admin.clone(), creator.clone()).unwrap();
        });
        let blacklisted = as_core_contract(&env, &contract_id, |env| {
            HuntyCore::is_blacklisted(env.clone(), creator.clone())
        });
        assert!(!blacklisted);
    }

    #[test]
    fn test_blacklisted_creator_cannot_create_hunt() {
        let env = Env::default();
        env.mock_all_auths();
        let admin = Address::generate(&env);
        let creator = Address::generate(&env);
        let contract_id = env.register_contract(None, HuntyCore);

        as_core_contract(&env, &contract_id, |env| {
            HuntyCore::set_admin(env.clone(), admin.clone());
        });
        as_core_contract(&env, &contract_id, |env| {
            HuntyCore::blacklist_creator(env.clone(), admin.clone(), creator.clone()).unwrap();
        });
        let result = as_core_contract(&env, &contract_id, |env| {
            HuntyCore::create_hunt(
                env.clone(),
                creator.clone(),
                String::from_str(env, "Test Hunt"),
                String::from_str(env, "Description"),
                None,
                None,
            )
        });
        assert_eq!(result, Err(HuntErrorCode::CreatorBlacklisted));
    }

    #[test]
    fn test_non_blacklisted_creator_can_create_hunt() {
        let env = Env::default();
        env.mock_all_auths();
        let admin = Address::generate(&env);
        let creator = Address::generate(&env);
        let other = Address::generate(&env);
        let contract_id = env.register_contract(None, HuntyCore);

        as_core_contract(&env, &contract_id, |env| {
            HuntyCore::set_admin(env.clone(), admin.clone());
        });
        as_core_contract(&env, &contract_id, |env| {
            HuntyCore::blacklist_creator(env.clone(), admin.clone(), creator.clone()).unwrap();
        });
        let result = as_core_contract(&env, &contract_id, |env| {
            HuntyCore::create_hunt(
                env.clone(),
                other.clone(),
                String::from_str(env, "Hunt by Other"),
                String::from_str(env, "Description"),
                None,
                None,
            )
        });
        assert!(result.is_ok());
    }

    #[test]
    fn test_blacklist_non_admin_unauthorized() {
        let env = Env::default();
        env.mock_all_auths();
        let admin = Address::generate(&env);
        let not_admin = Address::generate(&env);
        let creator = Address::generate(&env);
        let contract_id = env.register_contract(None, HuntyCore);

        as_core_contract(&env, &contract_id, |env| {
            HuntyCore::set_admin(env.clone(), admin.clone());
        });
        let result = as_core_contract(&env, &contract_id, |env| {
            HuntyCore::blacklist_creator(env.clone(), not_admin.clone(), creator.clone())
        });
        assert_eq!(result, Err(HuntErrorCode::Unauthorized));
    }

    #[test]
    fn test_is_blacklisted_false_by_default() {
        let env = Env::default();
        let creator = Address::generate(&env);
        let result = with_core_contract(&env, |env, _cid| {
            HuntyCore::is_blacklisted(env.clone(), creator.clone())
        });
        assert!(!result);
    }

    #[test]
    fn test_incremental_score_consistency() {
        let env = Env::default();
        let player = Address::generate(&env);
        let hunt_id = 1u64;
        let mut progress = PlayerProgress::new(&env, player, hunt_id, 0);
        progress.complete_clue(&env, 1, 10).unwrap();
        progress.complete_clue(&env, 2, 20).unwrap();
        progress.complete_clue(&env, 3, 30).unwrap();
        assert_eq!(progress.total_score, 60);
    }
}
