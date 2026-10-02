//! Integration tests for:
//! - Issue #328: configurable registration deadline
//! - Issue #332: hunt completion percentage tracking
//! - Issue #333: partial scoring for incomplete hunts
//! - Issue #334: team-based hunts
//! - Issue #1028: request_hint rejects already-completed clues

use hunty_core::types::HuntPrivacyChangedEvent;
use hunty_core::{HuntyCore, HuntyCoreClient};
use soroban_sdk::testutils::{Address as _, Events as _, Ledger as _};
use soroban_sdk::{Address, Env, IntoVal, String, Symbol, TryFromVal, Val};

const START_TS: u64 = 1_700_000_000;

/// Registers the contract and creates a hunt with two required clues
/// ("a1" and "a2"). Returns (client, creator, hunt_id).
fn setup_hunt(env: &Env, end_time: Option<u64>) -> (HuntyCoreClient<'_>, Address, u64) {
    let core_id = env.register(HuntyCore, ());
    let client = HuntyCoreClient::new(env, &core_id);
    let creator = Address::generate(env);

    let hunt_id = client.create_hunt(
        &creator,
        &String::from_str(env, "Test Hunt"),
        &String::from_str(env, "A hunt for testing"),
        &None,
        &end_time,
        &0u32,
        &None,
    );
    client.add_clue(
        &hunt_id,
        &String::from_str(env, "Question one"),
        &String::from_str(env, "a1"),
        &10u32,
        &true,
        &None,
        &None,
    );
    client.add_clue(
        &hunt_id,
        &String::from_str(env, "Question two"),
        &String::from_str(env, "a2"),
        &10u32,
        &true,
        &None,
        &None,
    );

    (client, creator, hunt_id)
}

#[test]
fn test_invite_codes_over_256_bytes_are_rejected() {
    let env = Env::default();
    env.ledger().set_timestamp(START_TS);
    env.mock_all_auths();

    let (client, creator, hunt_id) = setup_hunt(&env, None);
    let oversized_code = String::from_str(&env, &"x".repeat(300));

    assert!(client
        .try_generate_invite_code(&hunt_id, &creator, &oversized_code)
        .is_err());

    client.generate_invite_code(&hunt_id, &creator, &String::from_str(&env, "valid-code"));
    client.set_hunt_privacy(&hunt_id, &creator, &true);
    client.activate_hunt(&hunt_id, &creator);

    let player = Address::generate(&env);
    assert!(client
        .try_register_with_invite(&hunt_id, &player, &oversized_code)
        .is_err());
}

#[test]
fn test_set_hunt_privacy_emits_privacy_event_only() {
    let env = Env::default();
    env.ledger().set_timestamp(START_TS);
    env.mock_all_auths();

    let core_id = env.register(HuntyCore, ());
    let client = HuntyCoreClient::new(&env, &core_id);
    let creator = Address::generate(&env);
    let hunt_id = client.create_hunt(
        &creator,
        &String::from_str(&env, "Privacy test"),
        &String::from_str(&env, "Privacy event regression"),
        &None,
        &None,
        &0u32,
        &None,
    );

    client.set_hunt_privacy(&hunt_id, &creator, &true);

    let privacy_topic: Val = Symbol::new(&env, "HuntPrivacyChanged").into_val(&env);
    let status_topic: Val = Symbol::new(&env, "HuntStatusChanged").into_val(&env);
    let mut found_privacy_event = false;
    for (_, topics, data) in env.events().all().iter() {
        if topics.len() == 0 {
            continue;
        }
        let topic = topics.get(0).unwrap();
        if topic.get_payload() == privacy_topic.get_payload() {
            let event = HuntPrivacyChangedEvent::try_from_val(&env, &data).unwrap();
            assert_eq!(event.hunt_id, hunt_id);
            assert!(event.is_private);
            assert_eq!(event.changed_at, START_TS);
            found_privacy_event = true;
        }
        assert_ne!(topic.get_payload(), status_topic.get_payload());
    }
    assert!(found_privacy_event, "expected HuntPrivacyChanged event");
}

fn submit(
    client: &HuntyCoreClient,
    env: &Env,
    hunt_id: u64,
    clue_id: u32,
    player: &Address,
    answer: &str,
    nonce: u64,
) {
    client.submit_answer(
        &hunt_id,
        &clue_id,
        player,
        &String::from_str(env, answer),
        &nonce,
        &env.ledger().timestamp(),
    );
}

// ========== Issue #328: registration deadline ==========

#[test]
fn test_registration_deadline_enforced() {
    let env = Env::default();
    env.ledger().set_timestamp(START_TS);
    env.mock_all_auths();

    let (client, creator, hunt_id) = setup_hunt(&env, None);

    // Deadline can be set while the hunt is still in Draft
    client.set_registration_deadline(&hunt_id, &creator, &(START_TS + 100));
    client.activate_hunt(&hunt_id, &creator);

    // Deadline can be updated while the hunt is Active
    client.set_registration_deadline(&hunt_id, &creator, &(START_TS + 200));
    assert_eq!(
        client.get_hunt_info(&hunt_id).registration_deadline,
        START_TS + 200
    );

    // Registration works before the deadline
    let early_player = Address::generate(&env);
    client.register_player(&hunt_id, &early_player);

    // Registration is rejected once the deadline has passed
    env.ledger().set_timestamp(START_TS + 200);
    let late_player = Address::generate(&env);
    assert!(client.try_register_player(&hunt_id, &late_player).is_err());
}

#[test]
fn test_registration_deadline_in_past_rejected() {
    let env = Env::default();
    env.ledger().set_timestamp(START_TS);
    env.mock_all_auths();

    let (client, creator, hunt_id) = setup_hunt(&env, None);
    assert!(client
        .try_set_registration_deadline(&hunt_id, &creator, &(START_TS - 1))
        .is_err());

    // Non-creator cannot set the deadline
    let stranger = Address::generate(&env);
    assert!(client
        .try_set_registration_deadline(&hunt_id, &stranger, &(START_TS + 100))
        .is_err());
}

// ========== Issue #332: completion percentage tracking ==========

#[test]
fn test_hunt_completion_rate() {
    let env = Env::default();
    env.ledger().set_timestamp(START_TS);
    env.mock_all_auths();

    let (client, creator, hunt_id) = setup_hunt(&env, None);
    client.activate_hunt(&hunt_id, &creator);

    // No players yet: rate is 0
    assert_eq!(client.get_hunt_completion_rate(&hunt_id), 0);

    let finisher = Address::generate(&env);
    let slacker = Address::generate(&env);
    client.register_player(&hunt_id, &finisher);
    client.register_player(&hunt_id, &slacker);

    // finisher completes both required clues -> hunt completed
    submit(&client, &env, hunt_id, 1, &finisher, "a1", 1);
    submit(&client, &env, hunt_id, 2, &finisher, "a2", 2);

    assert_eq!(client.get_hunt_info(&hunt_id).completed_count, 1);
    assert_eq!(client.get_hunt_completion_rate(&hunt_id), 50);
}

// ========== Issue #333: partial scoring ==========

#[test]
fn test_partial_score_claim() {
    let env = Env::default();
    env.ledger().set_timestamp(START_TS);
    env.mock_all_auths();

    let end_time = START_TS + 3_600;
    let (client, creator, hunt_id) = setup_hunt(&env, Some(end_time));
    client.set_allow_partial_scoring(&hunt_id, &creator, &true);
    client.activate_hunt(&hunt_id, &creator);

    let partial_player = Address::generate(&env);
    let full_player = Address::generate(&env);
    client.register_player(&hunt_id, &partial_player);
    client.register_player(&hunt_id, &full_player);

    // partial_player solves only one of the two required clues
    submit(&client, &env, hunt_id, 1, &partial_player, "a1", 1);
    // full_player completes the hunt
    submit(&client, &env, hunt_id, 1, &full_player, "a1", 2);
    submit(&client, &env, hunt_id, 2, &full_player, "a2", 3);

    // Claiming before the hunt ends is rejected
    assert!(client
        .try_claim_partial_score(&hunt_id, &partial_player)
        .is_err());

    env.ledger().set_timestamp(end_time + 1);

    // Partial score equals the score earned from the completed clue
    let expected = client
        .get_player_progress(&hunt_id, &partial_player)
        .total_score;
    assert!(expected > 0);
    assert_eq!(
        client.claim_partial_score(&hunt_id, &partial_player),
        expected
    );

    // Double-claim is rejected
    assert!(client
        .try_claim_partial_score(&hunt_id, &partial_player)
        .is_err());

    // Fully-completed players cannot claim a partial score
    assert!(client
        .try_claim_partial_score(&hunt_id, &full_player)
        .is_err());
}

#[test]
fn test_partial_score_requires_flag() {
    let env = Env::default();
    env.ledger().set_timestamp(START_TS);
    env.mock_all_auths();

    let end_time = START_TS + 3_600;
    let (client, creator, hunt_id) = setup_hunt(&env, Some(end_time));
    client.activate_hunt(&hunt_id, &creator);

    let player = Address::generate(&env);
    client.register_player(&hunt_id, &player);
    submit(&client, &env, hunt_id, 1, &player, "a1", 1);

    env.ledger().set_timestamp(end_time + 1);
    // allow_partial_scoring was never enabled
    assert!(client.try_claim_partial_score(&hunt_id, &player).is_err());
}

// ========== Issue #334: team-based hunts ==========

#[test]
fn test_team_hunt_shared_progress_and_leaderboard() {
    let env = Env::default();
    env.ledger().set_timestamp(START_TS);
    env.mock_all_auths();

    let (client, creator, hunt_id) = setup_hunt(&env, None);
    client.set_team_mode(&hunt_id, &creator, &true);
    client.activate_hunt(&hunt_id, &creator);

    let alice = Address::generate(&env);
    let bob = Address::generate(&env);
    let carol = Address::generate(&env);
    client.register_player(&hunt_id, &alice);
    client.register_player(&hunt_id, &bob);
    client.register_player(&hunt_id, &carol);

    let alpha = client.create_team(&hunt_id, &alice, &String::from_str(&env, "Alpha"));
    client.join_team(&hunt_id, &alpha, &bob);
    let beta = client.create_team(&hunt_id, &carol, &String::from_str(&env, "Beta"));

    assert_eq!(client.get_player_team(&hunt_id, &bob), Some(alpha));
    assert_eq!(client.get_team(&hunt_id, &alpha).members.len(), 2);

    // A player cannot join a second team
    assert!(client.try_join_team(&hunt_id, &beta, &bob).is_err());

    // Alice solves clue 1: it is recorded for team Alpha
    submit(&client, &env, hunt_id, 1, &alice, "a1", 1);
    let alpha_progress = client.get_team_progress(&hunt_id, &alpha);
    assert_eq!(alpha_progress.completed_clues.len(), 1);
    assert!(alpha_progress.total_score > 0);

    // Bob (same team) cannot re-solve a clue his teammate already completed
    let res = client.try_submit_answer(
        &hunt_id,
        &1u32,
        &bob,
        &String::from_str(&env, "a1"),
        &2u64,
        &env.ledger().timestamp(),
    );
    assert!(res.is_err());

    // Carol (team Beta) can still solve clue 1 independently
    submit(&client, &env, hunt_id, 1, &carol, "a1", 3);

    // Bob solves clue 2 for Alpha: shared team score now covers both clues
    submit(&client, &env, hunt_id, 2, &bob, "a2", 4);
    let alpha_progress = client.get_team_progress(&hunt_id, &alpha);
    assert_eq!(alpha_progress.completed_clues.len(), 2);

    // Team leaderboard ranks Alpha (2 clues) above Beta (1 clue)
    let board = client.get_team_leaderboard(&hunt_id, &10u32);
    assert_eq!(board.len(), 2);
    let first = board.get(0).unwrap();
    let second = board.get(1).unwrap();
    assert_eq!(first.team_id, alpha);
    assert_eq!(first.rank, 1);
    assert_eq!(first.member_count, 2);
    assert_eq!(second.team_id, beta);
    assert!(first.score > second.score);
}

#[test]
fn test_team_functions_require_team_mode() {
    let env = Env::default();
    env.ledger().set_timestamp(START_TS);
    env.mock_all_auths();

    let (client, creator, hunt_id) = setup_hunt(&env, None);
    client.activate_hunt(&hunt_id, &creator);

    let player = Address::generate(&env);
    client.register_player(&hunt_id, &player);

    // team_mode was never enabled for this hunt
    assert!(client
        .try_create_team(&hunt_id, &player, &String::from_str(&env, "Nope"))
        .is_err());
}

#[test]
fn test_first_clue_hint_request_saturates_at_zero() {
    let env = Env::default();
    env.ledger().set_timestamp(START_TS);
    env.mock_all_auths();

    let (client, creator, hunt_id) = setup_hunt(&env, None);

    // Set a hint for clue 1 with a penalty of 5
    client.set_clue_hint(
        &hunt_id,
        &1u32,
        &creator,
        &Some(String::from_str(&env, "It's a1")),
        &5u32,
    );

    client.activate_hunt(&hunt_id, &creator);

    let player = Address::generate(&env);
    client.register_player(&hunt_id, &player);

    // Player has 0 score. They can still request the hint.
    let hint = client.request_hint(&hunt_id, &1u32, &player);
    assert_eq!(hint, String::from_str(&env, "It's a1"));

    // Player's score should be 0 (saturates instead of going negative)
    let progress = client.get_player_progress(&hunt_id, &player);
    assert_eq!(progress.total_score, 0);
    assert_eq!(progress.hinted_clues.len(), 1);
}

/// Issue #1028: request_hint must reject clues the player already completed.
#[test]
fn test_request_hint_rejected_for_completed_clue() {
    let env = Env::default();
    env.ledger().set_timestamp(START_TS);
    env.mock_all_auths();

    let (client, creator, hunt_id) = setup_hunt(&env, None);

    // Attach a hint to clue 1 with a penalty of 5 points.
    client.set_clue_hint(
        &hunt_id,
        &1u32,
        &creator,
        &Some(String::from_str(&env, "It's a1")),
        &5u32,
    );

    client.activate_hunt(&hunt_id, &creator);

    let player = Address::generate(&env);
    client.register_player(&hunt_id, &player);

    // Solve clue 1 so it is marked completed.
    client.submit_answer(&hunt_id, &1u32, &player, &String::from_str(&env, "a1"));

    // Requesting a hint for an already-completed clue must fail.
    assert!(
        client.try_request_hint(&hunt_id, &1u32, &player).is_err(),
        "expected ClueAlreadyCompleted error when hinting a solved clue"
    );

    // Score must be unchanged: no hint penalty applied.
    let progress = client.get_player_progress(&hunt_id, &player);
    assert_eq!(
        progress.hinted_clues.len(),
        0,
        "hinted_clues should be empty — hint was rejected"
    );
}
