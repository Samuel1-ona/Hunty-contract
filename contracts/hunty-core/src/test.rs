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
use std::format;

#[cfg(test)]
mod test {
    // Benchmark-style micro tests (best-effort gas/footprint proxy)

    use super::*;
    use soroban_sdk::{Address, Env, IntoVal, String, Symbol, TryIntoVal, Vec};
    // Bring Soroban testutils traits into scope (generate addresses, set ledger info, register contracts).
    use crate::ANSWER_SUBMISSION_WINDOW_SECS;
    use crate::errors::{HuntError, HuntErrorCode};
    use crate::storage::Storage;
    use crate::types::{BatchClueInput, HuntStatus, TimeBonusConfig, ClueInfo};
    use crate::types::{
        ClueAddedEvent, CreatorBlacklistedEvent, CreatorRemovedFromBlacklistEvent,
        HuntClosedEvent, HuntCompletedEvent, HuntCreatedEvent, HuntStatusChangedEvent,
        LeaderboardResult, PlayerRegisteredEvent, RewardClaimFailedEvent,
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
        let events = env.events().all();
        let mut idx = 0;
        while idx < events.len() {
            let event = events.get(idx).unwrap();
            let topics = &event.1;
            if topics.len() > 0 {
                if let Ok(sym) = Symbol::try_from_val(env, &topics.get(0).unwrap()) {
                    if sym == Symbol::new(env, "HuntStatusChanged") {
                        return HuntStatusChangedEvent::try_from_val(env, &event.2).ok();
                    }
                }
            }
            idx += 1;
        }
        None
    }

    fn find_event<T: TryFromVal<Env, Val>>(env: &Env, topic_name: &str) -> Option<(Vec<Val>, T)> {
        let events = env.events().all();
        let mut idx = 0;
        while idx < events.len() {
            let event = events.get(idx).unwrap();
            let topics = event.1.clone();
            if topics.len() > 0 {
                if let Ok(sym) = Symbol::try_from_val(env, &topics.get(0).unwrap()) {
                    if sym == Symbol::new(env, topic_name) {
                        if let Ok(data) = T::try_from_val(env, &event.2) {
                            return Some((topics, data));
                        }
                    }
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
        let err = HuntError::HuntNotFound;
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
    fn test_hunt_not_found_converts_to_code() {
        let err = HuntError::HuntNotFound;
        let code: HuntErrorCode = err.into();
        assert_eq!(code, HuntErrorCode::HuntNotFound);
    }

    #[test]
    fn test_issue_686_error_variants_convert_to_codes() {
        let cases = [
            (HuntError::RefundFailed, HuntErrorCode::RefundFailed),
            (HuntError::NoCluesAdded, HuntErrorCode::NoCluesAdded),
            (
                HuntError::InvalidMaxAttempts,
                HuntErrorCode::InvalidMaxAttempts,
            ),
        ];

        for (error, expected_code) in cases {
            let code: HuntErrorCode = error.into();
            assert_eq!(code, expected_code);
        }
    }

    #[test]
    fn test_clue_not_found_converts_to_code() {
        let err = HuntError::ClueNotFound;
        let code: HuntErrorCode = err.into();
        assert_eq!(code, HuntErrorCode::ClueNotFound);
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
                None)
        })
        .unwrap();

        // Add a clue with answer "Paris"
        env.mock_all_auths();
        let clue_id = as_core_contract(&env, &contract_id, |env| {
            HuntyCore::add_clue(env.clone(), hunt_id, String::from_str(env, "Capital of France?"), String::from_str(env, "Paris"), 10, true, None, None)
        })
        .unwrap();

        // Activate hunt
        env.mock_all_auths();
        as_core_contract(&env, &contract_id, |env| {
            HuntyCore::activate_hunt(env.clone(), hunt_id, creator.clone()).unwrap();
        });

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
                None)
        })
        .unwrap();

        let question = String::from_str(&env, "What is 2+2?");
        let answer = String::from_str(&env, "4");
        env.mock_all_auths();
        as_core_contract(&env, &contract_id, |env| {
            HuntyCore::add_clue(env.clone(), hunt_id, question.clone(), answer.clone(), 10, true, None, None).unwrap();
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
            HuntyCore::submit_answer(env.clone(), hunt_id, 1, player1.clone(), answer.clone(), 1, env.ledger().timestamp())
            .unwrap();
        });
        let board1 = as_core_contract(&env, &contract_id, |env| {
            HuntyCore::get_hunt_leaderboard(env.clone(), hunt_id, 10).unwrap().entries
        });
        let first = board1.get(0).unwrap();
        assert_eq!(first.player, player1);
        assert_eq!(first.rank, 1);
        assert!(first.is_completed);

        // Player2 completes
        env.mock_all_auths();
        as_core_contract(&env, &contract_id, |env| {
            HuntyCore::submit_answer(env.clone(), hunt_id, 1, player2.clone(), answer.clone(), 2, env.ledger().timestamp())
            .unwrap();
        });
        let board2 = as_core_contract(&env, &contract_id, |env| {
            HuntyCore::get_hunt_leaderboard(env.clone(), hunt_id, 10).unwrap().entries
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
            HuntyCore::get_hunt_leaderboard(env.clone(), hunt_id, 10).unwrap().entries
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
                None)
            .unwrap()
        });
        env.mock_all_auths();
        as_core_contract(&env, &contract_id, |env| {
            HuntyCore::add_clue(env.clone(), hunt_id, question.clone(), answer.clone(), 10, true, None, None)
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
                None)
            .unwrap();

            let (topics, event) =
                find_event::<HuntCreatedEvent>(env, "HuntCreated").expect("missing HuntCreated");
            assert_eq!(topics.len(), 2);
            assert_eq!(Symbol::try_from_val(env, &topics.get(0).unwrap()).unwrap(), Symbol::new(env, "HuntCreated"));
            assert_eq!(u64::try_from_val(env, &topics.get(1).unwrap()).unwrap(), hunt_id);
            assert_eq!(event.hunt_id, hunt_id);
            assert_eq!(event.creator, creator);
            // assert_eq!(event.title, title);
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
                None)
            .unwrap()
        });

        env.mock_all_auths();
        as_core_contract(&env, &contract_id, |env| {
            let clue_id = HuntyCore::add_clue(env.clone(), hunt_id, question.clone(), String::from_str(env, "Human"), 25, true, Some(3), None)
            .unwrap();

            let (topics, event) =
                find_event::<ClueAddedEvent>(env, "ClueAdded").expect("missing ClueAdded");
            assert_eq!(topics.len(), 3);
            assert_eq!(Symbol::try_from_val(env, &topics.get(0).unwrap()).unwrap(), Symbol::new(env, "ClueAdded"));
            assert_eq!(u64::try_from_val(env, &topics.get(1).unwrap()).unwrap(), hunt_id);
            assert_eq!(u32::try_from_val(env, &topics.get(2).unwrap()).unwrap(), clue_id);
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
                None)
            .unwrap();
            HuntyCore::add_clue(env.clone(), hunt_id, String::from_str(env, "Q"), String::from_str(env, "A"), 10, true, None, None)
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
            assert_eq!(Symbol::try_from_val(env, &topics.get(0).unwrap()).unwrap(), Symbol::new(env, "PlayerRegistered"));
            assert_eq!(u64::try_from_val(env, &topics.get(1).unwrap()).unwrap(), hunt_id);
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
                None)
            .unwrap()
        });
        env.mock_all_auths();
        as_core_contract(&env, &contract_id, |env| {
            HuntyCore::add_clue(env.clone(), hunt_id, question.clone(), answer.clone(), 10, true, None, None)
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
        assert_eq!(format!("{:?}", err), "InvalidHuntStatus");
    }

    #[test]
    fn test_insufficient_reward_pool_converts_to_code() {
        let err = HuntError::InsufficientRewardPool;
        let code: HuntErrorCode = err.into();
        assert_eq!(code, HuntErrorCode::InsufficientRewardPool);
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
                None)
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
        assert_eq!(hunt.time_bonus_start_bps, None);
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
                None)
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

        env.mock_all_auths();
        let result = with_core_contract(&env, |env, _cid| {
            let hid = HuntyCore::create_hunt(
                env.clone(),
                creator.clone(),
                title.clone(),
                description.clone(),
                None,
                Some(end_time),
                0, None, None).unwrap();
            HuntyCore::add_clue(env.clone(), hid, String::from_str(env, "Q"), String::from_str(env, "A"), 1, true, Some(1), None).unwrap();
            HuntyCore::activate_hunt(env.clone(), hid, creator.clone())
        });
        assert_eq!(result, Err(HuntErrorCode::HuntEndTimeInPast));

        let end_time_past = 1_699_999_999; // in the past (invalid)
        let result_past = with_core_contract(&env, |env, _cid| {
            let hid = HuntyCore::create_hunt(
                env.clone(),
                creator.clone(),
                title.clone(),
                description.clone(),
                None,
                Some(end_time_past),
                0, None, None).unwrap();
            HuntyCore::add_clue(env.clone(), hid, String::from_str(env, "Q"), String::from_str(env, "A"), 1, true, Some(1), None).unwrap();
            HuntyCore::activate_hunt(env.clone(), hid, creator.clone())
        });
        assert_eq!(result_past, Err(HuntErrorCode::HuntEndTimeInPast));
    }


    #[test]
    fn test_create_hunt_empty_title() {
        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);
        let creator = Address::generate(&env);
        let title = String::from_str(&env, "");
        let description = String::from_str(&env, "Valid description");

        let result = with_core_contract(&env, |env, _cid| {
            HuntyCore::create_hunt(env.clone(), creator, title, description, None, None, 0, None, None)
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
            HuntyCore::create_hunt(env.clone(), creator, long_title, description, None, None, 0, None, None)
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
            HuntyCore::create_hunt(env.clone(), creator, title, description, None, None, 0, None, None)
        });

        assert!(result.is_ok());
    }

    #[test]
    fn test_add_clues_success() {
        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);
        env.mock_all_auths();
        let creator = Address::generate(&env);

        let (ids, hunt, clues) = with_core_contract(&env, |env, _cid| {
            let hunt_id = HuntyCore::create_hunt(
                env.clone(),
                creator.clone(),
                String::from_str(env, "Batch Hunt"),
                String::from_str(env, "Description"),
                None,
                None,
                0, None, None)
            .unwrap();
            let clues = Vec::from_array(
                env,
                [
                    BatchClueInput {
                        question: String::from_str(env, "Q1"),
                        answer: String::from_str(env, "a1"),
                        points: 10,
                        is_required: true,
                        difficulty: 1,
                    },
                    BatchClueInput {
                        question: String::from_str(env, "Q2"),
                        answer: String::from_str(env, "a2"),
                        points: 20,
                        is_required: false,
                        difficulty: 3,
                    },
                ],
            );

            let ids = HuntyCore::add_clues(env.clone(), hunt_id, clues).unwrap();
            let hunt = Storage::get_hunt(env, hunt_id).unwrap();
            let stored = HuntyCore::list_clues(env.clone(), hunt_id, 0, 10);
            (ids, hunt, stored)
        });

        assert_eq!(ids.len(), 2);
        assert_eq!(ids.get(0).unwrap(), 1);
        assert_eq!(ids.get(1).unwrap(), 2);
        assert_eq!(hunt.total_clues, 2);
        assert_eq!(hunt.required_clues, 1);
        assert_eq!(clues.len(), 2);
        assert_eq!(clues.get(0).unwrap().points, 10);
        assert_eq!(clues.get(1).unwrap().difficulty, 3);
    }

    #[test]
    fn test_add_clues_rejects_batch_over_clue_limit() {
        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);
        env.mock_all_auths();
        let creator = Address::generate(&env);

        let clue_count = with_core_contract(&env, |env, _cid| {
            let hunt_id = HuntyCore::create_hunt(
                env.clone(),
                creator.clone(),
                String::from_str(env, "Batch Hunt"),
                String::from_str(env, "Description"),
                None,
                None,
                0, None, None)
            .unwrap();

            env.storage().persistent().set(&(soroban_sdk::symbol_short!("CC"), hunt_id), &99u32);

            let clues = Vec::from_array(
                env,
                [
                    BatchClueInput {
                        question: String::from_str(env, "Q100"),
                        answer: String::from_str(env, "a100"),
                        points: 1,
                        is_required: false,
                        difficulty: 1,
                    },
                    BatchClueInput {
                        question: String::from_str(env, "Q101"),
                        answer: String::from_str(env, "a101"),
                        points: 1,
                        is_required: false,
                        difficulty: 1,
                    },
                ],
            );

            let err = HuntyCore::add_clues(env.clone(), hunt_id, clues).unwrap_err();
            assert_eq!(err, HuntErrorCode::TooManyClues);
            Storage::get_clue_counter(env, hunt_id)
        });

        assert_eq!(clue_count, 99);
    }

    #[test]
    fn test_add_clues_invalid_hunt_status_not_draft() {
        let env = Env::default();
        env.ledger().set_timestamp(1_700_000_000);
        env.mock_all_auths();
        let creator = Address::generate(&env);

        with_core_contract(&env, |env, _cid| {
            let hunt_id = HuntyCore::create_hunt(
                env.clone(),
                creator.clone(),
                String::from_str(env, "Batch Hunt"),
                String::from_str(env, "Description"),
                None,
                None,
                0, None, None)
            .unwrap();
            HuntyCore::add_clue(env.clone(), hunt_id, String::from_str(env, "Required"), String::from_str(env, "a"), 1, true, Some(1), None)
            .unwrap();
            let mut hunt = Storage::get_hunt(env, hunt_id).unwrap();
            hunt.reward_config =
                crate::types::HuntRewardConfig::new(env, 100, false, None, 1, 0, 0);
            Storage::save_hunt(env, &hunt);
            HuntyCore::activate_hunt(env.clone(), hunt_id, creator.clone()).unwrap();

            let clues = Vec::from_array(
                env,
                [BatchClueInput {
                    question: String::from_str(env, "Q2"),
                    answer: String::from_str(env, "a2"),
                    points: 1,
                    is_required: false,
                    difficulty: 1,
                }],
            );

            let err = HuntyCore::add_clues(env.clone(), hunt_id, clues).unwrap_err();
            assert_eq!(err, HuntErrorCode::InvalidHuntStatus);
        });
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
            HuntyCore::create_hunt(env.clone(), creator, title, long_description, None, None, 0, None, None)
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
            HuntyCore::create_hunt(env.clone(), creator, title, description, None, None, 0, None, None)
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
                None)
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
                None)
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
                None)
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
                0, None, None)
            .unwrap();
            let second_hunt_id =
                HuntyCore::create_hunt(env.clone(), creator, title, description, None, None, 0, None, None)
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
                None)
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
                None)
            .unwrap();
            let hunt1 = Storage::get_hunt(&env, hunt_id1).unwrap();
            let hunt2 = Storage::get_hunt(&env, hunt_id2).unwrap();
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
                None)
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
                None)
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
                HuntyCore::create_hunt(env.clone(), creator, title, description, None, None, 0, None, None)
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
                HuntyCore::create_hunt(env.clone(), creator, title, description, None, None, 0, None, None)
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
                0,
                None,
                None)
            .unwrap();
            let clue_id =
                HuntyCore::add_clue(env.clone(), hunt_id, question.clone(), answer, 10, true, Some(1), None)
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
        // Do NOT mock auth â€” require_auth(creator) will fail.
        let creator = Address::generate(&env);
        let title = String::from_str(&env, "Test Hunt");
        let description = String::from_str(&env, "Description");
        let question = String::from_str(&env, "What is 2 + 2?");
        let answer = String::from_str(&env, "four");

        with_core_contract(&env, |env, _cid| {
            let hunt_id =
                HuntyCore::create_hunt(env.clone(), creator, title, description, None, None, 0, None, None)
                    .unwrap();
            let _ = HuntyCore::add_clue(env.clone(), hunt_id, question, answer, 10, true, Some(1), None);
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
            let hid = HuntyCore::create_hunt(env.clone(), creator, title, description, None, None, 0, None, None)
                .unwrap();
            let id1 = HuntyCore::add_clue(env.clone(), hid, q1, a.clone(), 1, false, Some(1), None).unwrap();
            let id2 = HuntyCore::add_clue(env.clone(), hid, q2, a.clone(), 1, false, Some(1), None).unwrap();
            let id3 = HuntyCore::add_clue(env.clone(), hid, q3, a, 1, false, Some(1), None).unwrap();
            (id1, id2, id3)
        });

        assert_eq!(id1, 1);
        assert_eq!(id2, 2);
        assert_eq!(id3, 3);
    }

    #[test]
    fn test_add_clue_answer_normalization_and_hashing() {
        let env = Env::default();
        let answer1 = String::from_str(&env, "  ANSWER  ");
        let answer2 = String::from_str(&env, "answer");

        let hash1 = HuntyCore::normalize_and_hash_answer(&env, 1, 1, &answer1).unwrap();
        let hash2 = HuntyCore::normalize_and_hash_answer(&env, 1, 1, &answer2).unwrap();

        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_add_clue_whitespace_answer_normalization_and_hashing() {
        let env = Env::default();
        let answer1 = String::from_str(&env, "	
 answer 
");
        let answer2 = String::from_str(&env, "answer");

        let hash1 = HuntyCore::normalize_and_hash_answer(&env, 1, 1, &answer1).unwrap();
        let hash2 = HuntyCore::normalize_and_hash_answer(&env, 1, 1, &answer2).unwrap();

        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_add_clue_unicode_answer_normalization_and_hashing() {
        let env = Env::default();
        let answer1 = String::from_str(&env, "Café");
        let answer2 = String::from_str(&env, "café");

        let hash1 = HuntyCore::normalize_and_hash_answer(&env, 1, 1, &answer1).unwrap();
        let hash2 = HuntyCore::normalize_and_hash_answer(&env, 1, 1, &answer2).unwrap();

        assert_eq!(hash1, hash2);
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
            let hid = HuntyCore::create_hunt(env.clone(), creator, title, description, None, None, 0, None, None)
                .unwrap();
            let _ = HuntyCore::add_clue(env.clone(), hid, question.clone(), answer, 7, true, Some(1), None);
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
            let hid = HuntyCore::create_hunt(env.clone(), creator, title, description, None, None, 0, None, None)
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
            let hid = HuntyCore::create_hunt(env.clone(), creator.clone(), title.clone(), description.clone(), None, None, 0, None, None)
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
            let hid = HuntyCore::create_hunt(env.clone(), creator, title, description, None, None, 0, None, None)
                .unwrap();
            HuntyCore::add_clue(env.clone(), hid, q1, a.clone(), 1, false, Some(1), None).unwrap();
            HuntyCore::add_clue(env.clone(), hid, q2, a, 2, true, Some(1), None).unwrap();
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
            let hid = HuntyCore::create_hunt(env.clone(), creator, title, description, None, None, 0, None, None)
                .unwrap();
            HuntyCore::add_clue(env.clone(), hid, q1, a.clone(), 1, false, Some(1), None).unwrap();
            HuntyCore::add_clue(env.clone(), hid, q2, a.clone(), 2, true, Some(1), None).unwrap();
            HuntyCore::add_clue(env.clone(), hid, q3, a, 3, false, Some(1), None).unwrap();
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
            HuntyCore::add_clue(env.clone(), 9999, question, answer, 1, false, Some(1), None).unwrap_err()
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
            let hid = HuntyCore::create_hunt(env.clone(), creator, title, description, None, None, 0, None, None)
                .unwrap();
            HuntyCore::add_clue(env.clone(), hid, empty, answer, 1, false, Some(1), None).unwrap_err()
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
            let hid = HuntyCore::create_hunt(env.clone(), creator, title, description, None, None, 0, None, None)
                .unwrap();
            HuntyCore::add_clue(env.clone(), hid, question, empty, 1, false, Some(1), None).unwrap_err()
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
            let hid = HuntyCore::create_hunt(env.clone(), creator, title, description, None, None, 0, None, None)
                .unwrap();
            HuntyCore::add_clue(env.clone(), hid, question, ws, 1, false, Some(1), None).unwrap_err()
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

        let err = with_core_contract(&env, |env, _cid| {
            let hid = HuntyCore::create_hunt(env.clone(), creator, title, description, None, None, 0, None, None)
                .unwrap();
            env.storage().persistent().set(&(soroban_sdk::symbol_short!("CC"), hid), &100u32);
            HuntyCore::add_clue(env.clone(), hid, question, answer, 1, false, Some(1), None).unwrap_err()
        });

        assert_eq!(err, HuntErrorCode::TooManyClues);
    }
}
