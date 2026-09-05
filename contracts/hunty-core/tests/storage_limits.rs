/// Storage Limits Testing Module
/// Tests behavior when approaching and exceeding storage limits for:
/// - Maximum clues per hunt (100)
/// - Maximum title/description lengths
/// - Maximum answer length
/// - Large number of hunts
use hunty_core::{HuntyCore, HuntyCoreClient};
use soroban_sdk::testutils::{Address as _, Ledger as _};
use soroban_sdk::{Address, Env, String, Vec};

fn setup_client(env: &Env) -> (HuntyCoreClient<'_>, Address) {
    let core_id = env.register(HuntyCore, ());
    let client = HuntyCoreClient::new(env, &core_id);
    let creator = Address::generate(env);
    (client, creator)
}

// ============================================================================
// Tests for Maximum Clues Per Hunt (100)
// ============================================================================

#[test]
fn test_add_maximum_clues_at_limit() {
    let env = Env::default();
    env.ledger().set_timestamp(1_700_000_000);
    env.mock_all_auths();

    let (client, creator) = setup_client(&env);

    let hunt_id = client.create_hunt(
        &creator,
        &String::from_str(&env, "Max Clues Hunt"),
        &String::from_str(&env, "Testing 100 clues"),
        &None,
        &None,
        &0u32,
        &None,
        &None,
    );

    // Add exactly 100 clues
    for i in 0..100 {
        let question = String::from_str(&env, &format!("Question {}", i));
        let answer = String::from_str(&env, &format!("Answer {}", i));
        client.add_clue(&hunt_id, &question, &answer, &10u32, &false, &None, &None);
    }

    // Verify all 100 clues were added
    let hunt = client.get_hunt_info(&hunt_id);
    assert_eq!(hunt.total_clues, 100, "Should have exactly 100 clues");
    let page1 = client.list_clues(&hunt_id, &0u32, &50u32);
    assert_eq!(page1.len(), 50);
    let page2 = client.list_clues(&hunt_id, &50u32, &50u32);
    assert_eq!(page2.len(), 50);
}

#[test]
fn test_exceed_maximum_clues_fails() {
    let env = Env::default();
    env.ledger().set_timestamp(1_700_000_000);
    env.mock_all_auths();

    let (client, creator) = setup_client(&env);

    let hunt_id = client.create_hunt(
        &creator,
        &String::from_str(&env, "Over Limit Hunt"),
        &String::from_str(&env, "Testing clue overflow"),
        &None,
        &None,
        &0u32,
        &None,
        &None,
    );

    // Add 100 clues successfully
    for i in 0..100 {
        let question = String::from_str(&env, &format!("Question {}", i));
        let answer = String::from_str(&env, &format!("Answer {}", i));
        client.add_clue(&hunt_id, &question, &answer, &10u32, &false, &None, &None);
    }

    // Attempt to add 101st clue - should fail
    let question_101 = String::from_str(&env, "Question 101");
    let answer_101 = String::from_str(&env, "Answer 101");
    let result = client.try_add_clue(
        &hunt_id,
        &question_101,
        &answer_101,
        &10u32,
        &false,
        &None,
        &None,
    );

    assert!(
        result.is_err(),
        "Adding 101st clue should fail with TooManyClues error"
    );
}

#[test]
fn test_clue_storage_at_boundary() {
    let env = Env::default();
    env.ledger().set_timestamp(1_700_000_000);
    env.mock_all_auths();

    let (client, creator) = setup_client(&env);

    let hunt_id = client.create_hunt(
        &creator,
        &String::from_str(&env, "Boundary Hunt"),
        &String::from_str(&env, "Testing boundary conditions"),
        &None,
        &None,
        &0u32,
        &None,
        &None,
    );

    // Add 99 clues
    for i in 0..99 {
        let question = String::from_str(&env, &format!("Q{}", i));
        let answer = String::from_str(&env, &format!("A{}", i));
        client.add_clue(&hunt_id, &question, &answer, &10u32, &false, &None, &None);
    }

    // Verify 99 clues
    let hunt = client.get_hunt_info(&hunt_id);
    assert_eq!(hunt.total_clues, 99);

    // Add 100th clue - should succeed
    let question_100 = String::from_str(&env, "Q99");
    let answer_100 = String::from_str(&env, "A99");
    client.add_clue(
        &hunt_id,
        &question_100,
        &answer_100,
        &10u32,
        &false,
        &None,
        &None,
    );

    // Verify all 100 clues
    let hunt = client.get_hunt_info(&hunt_id);
    assert_eq!(hunt.total_clues, 100);

    // Add 101st - should fail
    let question_over = String::from_str(&env, "Over limit");
    let answer_over = String::from_str(&env, "Overflow");
    let result = client.try_add_clue(
        &hunt_id,
        &question_over,
        &answer_over,
        &10u32,
        &false,
        &None,
        &None,
    );
    assert!(result.is_err());
}

// ============================================================================
// Tests for Title Length Limit (200 bytes)
// ============================================================================

#[test]
fn test_title_at_maximum_length() {
    let env = Env::default();
    env.ledger().set_timestamp(1_700_000_000);
    env.mock_all_auths();

    let (client, creator) = setup_client(&env);

    let max_title = "a".repeat(200);
    let title = String::from_str(&env, &max_title);

    let result = client.try_create_hunt(
        &creator,
        &title,
        &String::from_str(&env, "Valid description"),
        &None,
        &None,
        &0u32,
        &None,
        &None,
    );

    assert!(result.is_ok(), "Title at max length should succeed");
}

#[test]
fn test_title_exceeds_maximum_length() {
    let env = Env::default();
    env.ledger().set_timestamp(1_700_000_000);
    env.mock_all_auths();

    let (client, creator) = setup_client(&env);

    let over_max_title = "b".repeat(201);
    let title = String::from_str(&env, &over_max_title);

    let result = client.try_create_hunt(
        &creator,
        &title,
        &String::from_str(&env, "Valid description"),
        &None,
        &None,
        &0u32,
        &None,
        &None,
    );

    assert!(result.is_err(), "Title exceeding max length should fail");
}

#[test]
fn test_empty_title_fails() {
    let env = Env::default();
    env.ledger().set_timestamp(1_700_000_000);
    env.mock_all_auths();

    let (client, creator) = setup_client(&env);

    let title = String::from_str(&env, "");

    let result = client.try_create_hunt(
        &creator,
        &title,
        &String::from_str(&env, "Valid description"),
        &None,
        &None,
        &0u32,
        &None,
        &None,
    );

    assert!(result.is_err(), "Empty title should fail");
}

// ============================================================================
// Tests for Description Length Limit (2000 bytes)
// ============================================================================

#[test]
fn test_description_at_maximum_length() {
    let env = Env::default();
    env.ledger().set_timestamp(1_700_000_000);
    env.mock_all_auths();

    let (client, creator) = setup_client(&env);

    let max_description = "c".repeat(2000);
    let description = String::from_str(&env, &max_description);

    let result = client.try_create_hunt(
        &creator,
        &String::from_str(&env, "Valid Title"),
        &description,
        &None,
        &None,
        &0u32,
        &None,
        &None,
    );

    assert!(result.is_ok(), "Description at max length should succeed");
}

#[test]
fn test_description_exceeds_maximum_length() {
    let env = Env::default();
    env.ledger().set_timestamp(1_700_000_000);
    env.mock_all_auths();

    let (client, creator) = setup_client(&env);

    let over_max_description = "d".repeat(2001);
    let description = String::from_str(&env, &over_max_description);

    let result = client.try_create_hunt(
        &creator,
        &String::from_str(&env, "Valid Title"),
        &description,
        &None,
        &None,
        &0u32,
        &None,
        &None,
    );

    assert!(
        result.is_err(),
        "Description exceeding max length should fail"
    );
}

#[test]
fn test_empty_description_allowed() {
    let env = Env::default();
    env.ledger().set_timestamp(1_700_000_000);
    env.mock_all_auths();

    let (client, creator) = setup_client(&env);

    let result = client.try_create_hunt(
        &creator,
        &String::from_str(&env, "Valid Title"),
        &String::from_str(&env, ""),
        &None,
        &None,
        &0u32,
        &None,
        &None,
    );

    assert!(result.is_ok(), "Empty description should be allowed");
}

// ============================================================================
// Tests for Question Length Limit (2000 bytes)
// ============================================================================

#[test]
fn test_question_at_maximum_length() {
    let env = Env::default();
    env.ledger().set_timestamp(1_700_000_000);
    env.mock_all_auths();

    let (client, creator) = setup_client(&env);

    let hunt_id = client.create_hunt(
        &creator,
        &String::from_str(&env, "Title"),
        &String::from_str(&env, "Description"),
        &None,
        &None,
        &0u32,
        &None,
        &None,
    );

    let max_question = "e".repeat(2000);
    let question = String::from_str(&env, &max_question);
    let answer = String::from_str(&env, "answer");

    let result = client.try_add_clue(&hunt_id, &question, &answer, &10u32, &false, &None, &None);
    assert!(result.is_ok(), "Question at max length should succeed");
}

#[test]
fn test_question_exceeds_maximum_length() {
    let env = Env::default();
    env.ledger().set_timestamp(1_700_000_000);
    env.mock_all_auths();

    let (client, creator) = setup_client(&env);

    let hunt_id = client.create_hunt(
        &creator,
        &String::from_str(&env, "Title"),
        &String::from_str(&env, "Description"),
        &None,
        &None,
        &0u32,
        &None,
        &None,
    );

    let over_max_question = "f".repeat(2001);
    let question = String::from_str(&env, &over_max_question);
    let answer = String::from_str(&env, "answer");

    let result = client.try_add_clue(&hunt_id, &question, &answer, &10u32, &false, &None, &None);
    assert!(result.is_err(), "Question exceeding max length should fail");
}

// ============================================================================
// Tests for Answer Length Limit (256 bytes)
// ============================================================================

#[test]
fn test_answer_at_maximum_length() {
    let env = Env::default();
    env.ledger().set_timestamp(1_700_000_000);
    env.mock_all_auths();

    let (client, creator) = setup_client(&env);

    let hunt_id = client.create_hunt(
        &creator,
        &String::from_str(&env, "Title"),
        &String::from_str(&env, "Description"),
        &None,
        &None,
        &0u32,
        &None,
        &None,
    );

    let max_answer = "g".repeat(256);
    let question = String::from_str(&env, "Question");
    let answer = String::from_str(&env, &max_answer);

    let result = client.try_add_clue(&hunt_id, &question, &answer, &10u32, &false, &None, &None);
    assert!(result.is_ok(), "Answer at max length should succeed");
}

#[test]
fn test_answer_exceeds_maximum_length() {
    let env = Env::default();
    env.ledger().set_timestamp(1_700_000_000);
    env.mock_all_auths();

    let (client, creator) = setup_client(&env);

    let hunt_id = client.create_hunt(
        &creator,
        &String::from_str(&env, "Title"),
        &String::from_str(&env, "Description"),
        &None,
        &None,
        &0u32,
        &None,
        &None,
    );

    let over_max_answer = "h".repeat(257);
    let question = String::from_str(&env, "Question");
    let answer = String::from_str(&env, &over_max_answer);

    let result = client.try_add_clue(&hunt_id, &question, &answer, &10u32, &false, &None, &None);
    assert!(result.is_err(), "Answer exceeding max length should fail");
}

// ============================================================================
// Tests for Large Number of Hunts
// ============================================================================

#[test]
fn test_create_multiple_hunts_sequential() {
    let env = Env::default();
    env.ledger().set_timestamp(1_700_000_000);
    env.mock_all_auths();

    let core_id = env.register(HuntyCore, ());
    let client = HuntyCoreClient::new(&env, &core_id);

    let mut hunt_ids = Vec::new(&env);

    // Create 20 hunts with distinct creators to stay within per-creator rate limit
    for i in 0..20 {
        let creator = Address::generate(&env);
        let hunt_id = client.create_hunt(
            &creator,
            &String::from_str(&env, &format!("Hunt {}", i)),
            &String::from_str(&env, &format!("Description {}", i)),
            &None,
            &None,
            &0u32,
            &None,
            &None,
        );
        hunt_ids.push_back(hunt_id);
    }

    assert_eq!(hunt_ids.len(), 20, "Should have created 20 hunts");

    for i in 0..20 {
        let hunt_id = hunt_ids.get(i).unwrap();
        let hunt = client.get_hunt_info(&hunt_id);
        assert_eq!(hunt.hunt_id, hunt_id, "Hunt {} should have correct ID", i);
    }
}

#[test]
fn test_create_hunts_with_full_clue_set() {
    let env = Env::default();
    env.ledger().set_timestamp(1_700_000_000);
    env.mock_all_auths();

    let core_id = env.register(HuntyCore, ());
    let client = HuntyCoreClient::new(&env, &core_id);

    // Create 3 hunts with distinct creators
    for hunt_num in 0..3 {
        let creator = Address::generate(&env);
        let hunt_id = client.create_hunt(
            &creator,
            &String::from_str(&env, &format!("Full Hunt {}", hunt_num)),
            &String::from_str(&env, &format!("Hunt with clues {}", hunt_num)),
            &None,
            &None,
            &0u32,
            &None,
            &None,
        );

        for clue_num in 0..20 {
            let question = String::from_str(&env, &format!("Hunt {} Clue {}", hunt_num, clue_num));
            let answer = String::from_str(&env, &format!("Answer {} {}", hunt_num, clue_num));
            client.add_clue(&hunt_id, &question, &answer, &10u32, &false, &None, &None);
        }

        let hunt = client.get_hunt_info(&hunt_id);
        assert_eq!(
            hunt.total_clues, 20,
            "Hunt {} should have 20 clues",
            hunt_num
        );
    }
}

#[test]
fn test_hunt_storage_pressure_mixed_operations() {
    let env = Env::default();
    env.ledger().set_timestamp(1_700_000_000);
    env.mock_all_auths();

    let core_id = env.register(HuntyCore, ());
    let client = HuntyCoreClient::new(&env, &core_id);

    let mut hunt_ids = Vec::new(&env);
    for hunt_num in 0..3 {
        let creator = Address::generate(&env);
        let hunt_id = client.create_hunt(
            &creator,
            &String::from_str(&env, &format!("Pressure Hunt {}", hunt_num)),
            &String::from_str(&env, &format!("Testing storage pressure {}", hunt_num)),
            &None,
            &None,
            &0u32,
            &None,
            &None,
        );

        let clue_count = 10 + (hunt_num * 5);
        for clue_num in 0..clue_count {
            let question = String::from_str(&env, &format!("Q{}-{}", hunt_num, clue_num));
            let answer = String::from_str(&env, &format!("A{}-{}", hunt_num, clue_num));
            client.add_clue(&hunt_id, &question, &answer, &10u32, &false, &None, &None);
        }

        hunt_ids.push_back((hunt_id, clue_count));
    }

    for i in 0..3 {
        let (hunt_id, expected_clues) = hunt_ids.get(i).unwrap();
        let hunt = client.get_hunt_info(&hunt_id);
        assert_eq!(
            hunt.total_clues, expected_clues,
            "Hunt should have {} clues",
            expected_clues
        );
    }
}

#[test]
fn test_storage_limits_comprehensive_stress() {
    let env = Env::default();
    env.ledger().set_timestamp(1_700_000_000);
    env.mock_all_auths();

    let (client, creator) = setup_client(&env);

    let hunt_id = client.create_hunt(
        &creator,
        &String::from_str(&env, &"i".repeat(200)),
        &String::from_str(&env, &"j".repeat(2000)),
        &None,
        &None,
        &0u32,
        &None,
        &None,
    );

    // Add 100 clues with maximum question and answer lengths
    for i in 0..100 {
        let question = String::from_str(&env, &format!("{}Q{}", "k".repeat(1990), i));
        let answer = String::from_str(&env, &format!("{}A{}", "l".repeat(240), i));
        client.add_clue(&hunt_id, &question, &answer, &10u32, &false, &None, &None);
    }

    let hunt = client.get_hunt_info(&hunt_id);
    assert_eq!(hunt.total_clues, 100);
}

#[test]
fn test_multiple_hunts_at_maximum_size() {
    let env = Env::default();
    env.ledger().set_timestamp(1_700_000_000);
    env.mock_all_auths();

    let core_id = env.register(HuntyCore, ());
    let client = HuntyCoreClient::new(&env, &core_id);

    for hunt_num in 0..2 {
        let creator = Address::generate(&env);
        let hunt_id = client.create_hunt(
            &creator,
            &String::from_str(&env, &format!("Max Hunt {}", hunt_num)),
            &String::from_str(&env, &"m".repeat(2000)),
            &None,
            &None,
            &0u32,
            &None,
            &None,
        );

        for _ in 0..50 {
            let question = String::from_str(&env, &"n".repeat(2000));
            let answer = String::from_str(&env, &"o".repeat(256));
            client.add_clue(&hunt_id, &question, &answer, &10u32, &false, &None, &None);
        }
    }

    let hunt_1 = client.get_hunt_info(&1);
    assert_eq!(hunt_1.total_clues, 50);
}

#[test]
fn test_single_rate_limit_storage_entry_across_days() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, creator) = setup_client(&env);

    let days = [1_700_000_000u64, 1_700_086_400, 1_700_172_800];

    for (i, day) in days.iter().enumerate() {
        env.ledger().set_timestamp(*day);
        client.create_hunt(
            &creator,
            &String::from_str(&env, &format!("Rate Limit Hunt {}", i)),
            &String::from_str(&env, "Storage bound test"),
            &None,
            &None,
            &0u32,
            &None,
            &None,
        );
    }
}
