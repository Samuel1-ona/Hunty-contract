/// Storage Limits Testing Module
/// Tests behavior when approaching and exceeding storage limits for:
/// - Maximum clues per hunt (100)
/// - Maximum title/description lengths
/// - Maximum answer length
/// - Large number of hunts

use hunty_core::HuntyCore;
use soroban_sdk::testutils::{Address as _, Ledger as _};
use soroban_sdk::{Address, Env, String};

fn setup_core_contract(env: &Env) -> Address {
    let core_id = env.register(HuntyCore, ());
    core_id
}

fn as_core_contract<T>(env: &Env, contract_id: &Address, f: impl FnOnce(&Env) -> T) -> T {
    env.as_contract(contract_id, || f(env))
}

// ============================================================================
// Tests for Maximum Clues Per Hunt (100)
// ============================================================================

#[test]
fn test_add_maximum_clues_at_limit() {
    let env = Env::default();
    env.ledger().set_timestamp(1_700_000_000);
    env.mock_all_auths();

    let core_id = setup_core_contract(&env);
    let creator = Address::generate(&env);

    let hunt_id = as_core_contract(&env, &core_id, |env| {
        let hunt_id = HuntyCore::create_hunt(
            env.clone(),
            creator.clone(),
            String::from_str(env, "Max Clues Hunt"),
            String::from_str(env, "Testing 100 clues"),
            None,
            None,
            0,
            None,
        )
        .unwrap();

        // Add exactly 100 clues
        for i in 0..100 {
            let question = String::from_str(
                env,
                &format!("Question {}", i),
            );
            let answer = String::from_str(
                env,
                &format!("Answer {}", i),
            );
            HuntyCore::add_clue(env.clone(), hunt_id, question, answer, 10, false).unwrap();
        }

        hunt_id
    });

    as_core_contract(&env, &core_id, |env| {
        // Verify all 100 clues were added
        let clues = HuntyCore::list_clues(env.clone(), hunt_id).unwrap();
        assert_eq!(clues.len(), 100, "Should have exactly 100 clues");
    });
}

#[test]
fn test_exceed_maximum_clues_fails() {
    let env = Env::default();
    env.ledger().set_timestamp(1_700_000_000);
    env.mock_all_auths();

    let core_id = setup_core_contract(&env);
    let creator = Address::generate(&env);

    as_core_contract(&env, &core_id, |env| {
        let hunt_id = HuntyCore::create_hunt(
            env.clone(),
            creator.clone(),
            String::from_str(env, "Over Limit Hunt"),
            String::from_str(env, "Testing clue overflow"),
            None,
            None,
            0,
            None,
        )
        .unwrap();

        // Add 100 clues successfully
        for i in 0..100 {
            let question = String::from_str(
                env,
                &format!("Question {}", i),
            );
            let answer = String::from_str(
                env,
                &format!("Answer {}", i),
            );
            HuntyCore::add_clue(env.clone(), hunt_id, question, answer, 10, false).unwrap();
        }

        // Attempt to add 101st clue - should fail
        let question_101 = String::from_str(env, "Question 101");
        let answer_101 = String::from_str(env, "Answer 101");
        let result = HuntyCore::add_clue(env.clone(), hunt_id, question_101, answer_101, 10, false);

        assert!(
            result.is_err(),
            "Adding 101st clue should fail with TooManyClues error"
        );
    });
}

#[test]
fn test_clue_storage_at_boundary() {
    let env = Env::default();
    env.ledger().set_timestamp(1_700_000_000);
    env.mock_all_auths();

    let core_id = setup_core_contract(&env);
    let creator = Address::generate(&env);

    as_core_contract(&env, &core_id, |env| {
        let hunt_id = HuntyCore::create_hunt(
            env.clone(),
            creator.clone(),
            String::from_str(env, "Boundary Hunt"),
            String::from_str(env, "Testing boundary conditions"),
            None,
            None,
            0,
            None,
        )
        .unwrap();

        // Add 99 clues
        for i in 0..99 {
            let question = String::from_str(
                env,
                &format!("Q{}", i),
            );
            let answer = String::from_str(
                env,
                &format!("A{}", i),
            );
            HuntyCore::add_clue(env.clone(), hunt_id, question, answer, 10, false).unwrap();
        }

        // Verify 99 clues
        let clues = HuntyCore::list_clues(env.clone(), hunt_id).unwrap();
        assert_eq!(clues.len(), 99);

        // Add 100th clue - should succeed
        let question_100 = String::from_str(env, "Q99");
        let answer_100 = String::from_str(env, "A99");
        HuntyCore::add_clue(env.clone(), hunt_id, question_100, answer_100, 10, false).unwrap();

        // Verify all 100 clues
        let clues = HuntyCore::list_clues(env.clone(), hunt_id).unwrap();
        assert_eq!(clues.len(), 100);

        // Add 101st - should fail
        let question_over = String::from_str(env, "Over limit");
        let answer_over = String::from_str(env, "Overflow");
        let result = HuntyCore::add_clue(env.clone(), hunt_id, question_over, answer_over, 10, false);
        assert!(result.is_err());
    });
}

// ============================================================================
// Tests for Title Length Limit (200 bytes)
// ============================================================================

#[test]
fn test_title_at_maximum_length() {
    let env = Env::default();
    env.ledger().set_timestamp(1_700_000_000);
    env.mock_all_auths();

    let core_id = setup_core_contract(&env);
    let creator = Address::generate(&env);

    as_core_contract(&env, &core_id, |env| {
        // Create title exactly 200 bytes
        let max_title = "a".repeat(200);
        let title = String::from_str(env, &max_title);

        let result = HuntyCore::create_hunt(
            env.clone(),
            creator.clone(),
            title,
            String::from_str(env, "Valid description"),
            None,
            None,
            0,
            None,
        );

        assert!(result.is_ok(), "Title at max length should succeed");
    });
}

#[test]
fn test_title_exceeds_maximum_length() {
    let env = Env::default();
    env.ledger().set_timestamp(1_700_000_000);
    env.mock_all_auths();

    let core_id = setup_core_contract(&env);
    let creator = Address::generate(&env);

    as_core_contract(&env, &core_id, |env| {
        // Create title > 200 bytes
        let over_max_title = "b".repeat(201);
        let title = String::from_str(env, &over_max_title);

        let result = HuntyCore::create_hunt(
            env.clone(),
            creator.clone(),
            title,
            String::from_str(env, "Valid description"),
            None,
            None,
            0,
            None,
        );

        assert!(
            result.is_err(),
            "Title exceeding max length should fail"
        );
    });
}

#[test]
fn test_empty_title_fails() {
    let env = Env::default();
    env.ledger().set_timestamp(1_700_000_000);
    env.mock_all_auths();

    let core_id = setup_core_contract(&env);
    let creator = Address::generate(&env);

    as_core_contract(&env, &core_id, |env| {
        // Empty title
        let title = String::from_str(env, "");

        let result = HuntyCore::create_hunt(
            env.clone(),
            creator.clone(),
            title,
            String::from_str(env, "Valid description"),
            None,
            None,
            0,
            None,
        );

        assert!(result.is_err(), "Empty title should fail");
    });
}

// ============================================================================
// Tests for Description Length Limit (2000 bytes)
// ============================================================================

#[test]
fn test_description_at_maximum_length() {
    let env = Env::default();
    env.ledger().set_timestamp(1_700_000_000);
    env.mock_all_auths();

    let core_id = setup_core_contract(&env);
    let creator = Address::generate(&env);

    as_core_contract(&env, &core_id, |env| {
        // Create description exactly 2000 bytes
        let max_description = "c".repeat(2000);
        let description = String::from_str(env, &max_description);

        let result = HuntyCore::create_hunt(
            env.clone(),
            creator.clone(),
            String::from_str(env, "Valid Title"),
            description,
            None,
            None,
            0,
            None,
        );

        assert!(
            result.is_ok(),
            "Description at max length should succeed"
        );
    });
}

#[test]
fn test_description_exceeds_maximum_length() {
    let env = Env::default();
    env.ledger().set_timestamp(1_700_000_000);
    env.mock_all_auths();

    let core_id = setup_core_contract(&env);
    let creator = Address::generate(&env);

    as_core_contract(&env, &core_id, |env| {
        // Create description > 2000 bytes
        let over_max_description = "d".repeat(2001);
        let description = String::from_str(env, &over_max_description);

        let result = HuntyCore::create_hunt(
            env.clone(),
            creator.clone(),
            String::from_str(env, "Valid Title"),
            description,
            None,
            None,
            0,
            None,
        );

        assert!(
            result.is_err(),
            "Description exceeding max length should fail"
        );
    });
}

#[test]
fn test_empty_description_allowed() {
    let env = Env::default();
    env.ledger().set_timestamp(1_700_000_000);
    env.mock_all_auths();

    let core_id = setup_core_contract(&env);
    let creator = Address::generate(&env);

    as_core_contract(&env, &core_id, |env| {
        // Empty description is allowed
        let result = HuntyCore::create_hunt(
            env.clone(),
            creator.clone(),
            String::from_str(env, "Valid Title"),
            String::from_str(env, ""),
            None,
            None,
            0,
            None,
        );

        assert!(result.is_ok(), "Empty description should be allowed");
    });
}

// ============================================================================
// Tests for Question Length Limit (2000 bytes)
// ============================================================================

#[test]
fn test_question_at_maximum_length() {
    let env = Env::default();
    env.ledger().set_timestamp(1_700_000_000);
    env.mock_all_auths();

    let core_id = setup_core_contract(&env);
    let creator = Address::generate(&env);

    as_core_contract(&env, &core_id, |env| {
        let hunt_id = HuntyCore::create_hunt(
            env.clone(),
            creator.clone(),
            String::from_str(env, "Title"),
            String::from_str(env, "Description"),
            None,
            None,
            0,
            None,
        )
        .unwrap();

        // Create question exactly 2000 bytes
        let max_question = "e".repeat(2000);
        let question = String::from_str(env, &max_question);
        let answer = String::from_str(env, "answer");

        let result = HuntyCore::add_clue(env.clone(), hunt_id, question, answer, 10, false);
        assert!(result.is_ok(), "Question at max length should succeed");
    });
}

#[test]
fn test_question_exceeds_maximum_length() {
    let env = Env::default();
    env.ledger().set_timestamp(1_700_000_000);
    env.mock_all_auths();

    let core_id = setup_core_contract(&env);
    let creator = Address::generate(&env);

    as_core_contract(&env, &core_id, |env| {
        let hunt_id = HuntyCore::create_hunt(
            env.clone(),
            creator.clone(),
            String::from_str(env, "Title"),
            String::from_str(env, "Description"),
            None,
            None,
            0,
            None,
        )
        .unwrap();

        // Create question > 2000 bytes
        let over_max_question = "f".repeat(2001);
        let question = String::from_str(env, &over_max_question);
        let answer = String::from_str(env, "answer");

        let result = HuntyCore::add_clue(env.clone(), hunt_id, question, answer, 10, false);
        assert!(
            result.is_err(),
            "Question exceeding max length should fail"
        );
    });
}

// ============================================================================
// Tests for Answer Length Limit (256 bytes)
// ============================================================================

#[test]
fn test_answer_at_maximum_length() {
    let env = Env::default();
    env.ledger().set_timestamp(1_700_000_000);
    env.mock_all_auths();

    let core_id = setup_core_contract(&env);
    let creator = Address::generate(&env);

    as_core_contract(&env, &core_id, |env| {
        let hunt_id = HuntyCore::create_hunt(
            env.clone(),
            creator.clone(),
            String::from_str(env, "Title"),
            String::from_str(env, "Description"),
            None,
            None,
            0,
            None,
        )
        .unwrap();

        // Create answer exactly 256 bytes
        let max_answer = "g".repeat(256);
        let question = String::from_str(env, "Question");
        let answer = String::from_str(env, &max_answer);

        let result = HuntyCore::add_clue(env.clone(), hunt_id, question, answer, 10, false);
        assert!(result.is_ok(), "Answer at max length should succeed");
    });
}

#[test]
fn test_answer_exceeds_maximum_length() {
    let env = Env::default();
    env.ledger().set_timestamp(1_700_000_000);
    env.mock_all_auths();

    let core_id = setup_core_contract(&env);
    let creator = Address::generate(&env);

    as_core_contract(&env, &core_id, |env| {
        let hunt_id = HuntyCore::create_hunt(
            env.clone(),
            creator.clone(),
            String::from_str(env, "Title"),
            String::from_str(env, "Description"),
            None,
            None,
            0,
            None,
        )
        .unwrap();

        // Create answer > 256 bytes
        let over_max_answer = "h".repeat(257);
        let question = String::from_str(env, "Question");
        let answer = String::from_str(env, &over_max_answer);

        let result = HuntyCore::add_clue(env.clone(), hunt_id, question, answer, 10, false);
        assert!(
            result.is_err(),
            "Answer exceeding max length should fail"
        );
    });
}

// ============================================================================
// Tests for Large Number of Hunts
// ============================================================================

#[test]
fn test_create_multiple_hunts_sequential() {
    let env = Env::default();
    env.ledger().set_timestamp(1_700_000_000);
    env.mock_all_auths();

    let core_id = setup_core_contract(&env);
    let creator = Address::generate(&env);

    as_core_contract(&env, &core_id, |env| {
        let mut hunt_ids = Vec::new();

        // Create 50 hunts sequentially
        for i in 0..50 {
            let hunt_id = HuntyCore::create_hunt(
                env.clone(),
                creator.clone(),
                String::from_str(env, &format!("Hunt {}", i)),
                String::from_str(env, &format!("Description {}", i)),
                None,
                None,
                0,
                None,
            )
            .unwrap();
            hunt_ids.push(hunt_id);
        }

        // Verify all hunts were created
        assert_eq!(hunt_ids.len(), 50, "Should have created 50 hunts");

        // Verify each hunt can be retrieved
        for (i, hunt_id) in hunt_ids.iter().enumerate() {
            let hunt = HuntyCore::get_hunt(env.clone(), *hunt_id).unwrap();
            assert_eq!(
                hunt.hunt_id, *hunt_id,
                "Hunt {} should have correct ID",
                i
            );
        }
    });
}

#[test]
fn test_create_hunts_with_full_clue_set() {
    let env = Env::default();
    env.ledger().set_timestamp(1_700_000_000);
    env.mock_all_auths();

    let core_id = setup_core_contract(&env);
    let creator = Address::generate(&env);

    as_core_contract(&env, &core_id, |env| {
        // Create 10 hunts, each with 100 clues
        for hunt_num in 0..10 {
            let hunt_id = HuntyCore::create_hunt(
                env.clone(),
                creator.clone(),
                String::from_str(env, &format!("Full Hunt {}", hunt_num)),
                String::from_str(env, &format!("Hunt with all clues {}", hunt_num)),
                None,
                None,
                0,
                None,
            )
            .unwrap();

            // Add 100 clues to each hunt
            for clue_num in 0..100 {
                let question = String::from_str(
                    env,
                    &format!("Hunt {} Clue {}", hunt_num, clue_num),
                );
                let answer = String::from_str(
                    env,
                    &format!("Answer {} {}", hunt_num, clue_num),
                );
                HuntyCore::add_clue(env.clone(), hunt_id, question, answer, 10, false).unwrap();
            }

            // Verify clues were added
            let clues = HuntyCore::list_clues(env.clone(), hunt_id).unwrap();
            assert_eq!(
                clues.len(),
                100,
                "Hunt {} should have 100 clues",
                hunt_num
            );
        }
    });
}

#[test]
fn test_hunt_storage_pressure_mixed_operations() {
    let env = Env::default();
    env.ledger().set_timestamp(1_700_000_000);
    env.mock_all_auths();

    let core_id = setup_core_contract(&env);
    let creator = Address::generate(&env);

    as_core_contract(&env, &core_id, |env| {
        // Create 5 hunts with varying clue counts
        let mut hunt_ids = Vec::new();
        for hunt_num in 0..5 {
            let hunt_id = HuntyCore::create_hunt(
                env.clone(),
                creator.clone(),
                String::from_str(env, &format!("Pressure Hunt {}", hunt_num)),
                String::from_str(env, &format!("Testing storage pressure {}", hunt_num)),
                None,
                None,
                0,
                None,
            )
            .unwrap();

            // Add clues: hunt 0 gets 20, hunt 1 gets 40, hunt 2 gets 60, hunt 3 gets 80, hunt 4 gets 100
            let clue_count = 20 + (hunt_num * 20);
            for clue_num in 0..clue_count {
                let question = String::from_str(
                    env,
                    &format!("Q{}-{}", hunt_num, clue_num),
                );
                let answer = String::from_str(
                    env,
                    &format!("A{}-{}", hunt_num, clue_num),
                );
                HuntyCore::add_clue(env.clone(), hunt_id, question, answer, 10, false).unwrap();
            }

            hunt_ids.push((hunt_id, clue_count));
        }

        // Verify all hunts and their clue counts
        for (hunt_id, expected_clues) in hunt_ids.iter() {
            let clues = HuntyCore::list_clues(env.clone(), *hunt_id).unwrap();
            assert_eq!(
                clues.len() as u64, *expected_clues,
                "Hunt should have {} clues",
                expected_clues
            );
        }
    });
}

// ============================================================================
// Tests for Combined Stress
// ============================================================================

#[test]
fn test_storage_limits_comprehensive_stress() {
    let env = Env::default();
    env.ledger().set_timestamp(1_700_000_000);
    env.mock_all_auths();

    let core_id = setup_core_contract(&env);
    let creator = Address::generate(&env);

    as_core_contract(&env, &core_id, |env| {
        // Create hunt with maximum title and description
        let hunt_id = HuntyCore::create_hunt(
            env.clone(),
            creator.clone(),
            String::from_str(env, &"i".repeat(200)),
            String::from_str(env, &"j".repeat(2000)),
            None,
            None,
            0,
            None,
        )
        .unwrap();

        // Add 100 clues with maximum question and answer lengths
        for i in 0..100 {
            let question = String::from_str(
                env,
                &format!("{}Q{}", "k".repeat(1990), i),
            );
            let answer = String::from_str(
                env,
                &format!("{}A{}", "l".repeat(240), i),
            );
            HuntyCore::add_clue(env.clone(), hunt_id, question, answer, 10, false).unwrap();
        }

        // Verify the hunt
        let hunt = HuntyCore::get_hunt(env.clone(), hunt_id).unwrap();
        assert_eq!(hunt.total_clues, 100);

        let clues = HuntyCore::list_clues(env.clone(), hunt_id).unwrap();
        assert_eq!(clues.len(), 100);
    });
}

#[test]
fn test_multiple_hunts_at_maximum_size() {
    let env = Env::default();
    env.ledger().set_timestamp(1_700_000_000);
    env.mock_all_auths();

    let core_id = setup_core_contract(&env);
    let creator = Address::generate(&env);

    as_core_contract(&env, &core_id, |env| {
        // Create 3 hunts, each at maximum capacity
        for hunt_num in 0..3 {
            let hunt_id = HuntyCore::create_hunt(
                env.clone(),
                creator.clone(),
                String::from_str(env, &format!("Max Hunt {}", hunt_num)),
                String::from_str(env, &"m".repeat(2000)),
                None,
                None,
                0,
                None,
            )
            .unwrap();

            // Add 100 clues
            for clue_num in 0..100 {
                let question = String::from_str(
                    env,
                    &"n".repeat(2000),
                );
                let answer = String::from_str(
                    env,
                    &"o".repeat(256),
                );
                HuntyCore::add_clue(env.clone(), hunt_id, question, answer, 10, false).unwrap();
            }
        }

        // Try to retrieve and verify stats on first hunt (should be findable via counter)
        let hunt_1 = HuntyCore::get_hunt(env.clone(), 0).unwrap();
        assert_eq!(hunt_1.total_clues, 100);
    });
}
