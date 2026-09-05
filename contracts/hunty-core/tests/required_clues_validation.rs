/// Required Clues Validation Tests
/// Tests that hunt activation requires at least one required clue to succeed.
///
/// Acceptance Criteria:
/// - Create hunt, add only optional clues
/// - Attempt activation → should fail with NoRequiredClues
/// - Add one required clue → activation should succeed
use hunty_core::{HuntyCore, HuntyCoreClient};
use soroban_sdk::testutils::{Address as _, Ledger as _};
use soroban_sdk::{Address, Env, String};

fn setup_hunt(env: &Env) -> (HuntyCoreClient<'_>, Address, u64) {
    let core_id = env.register(HuntyCore, ());
    let client = HuntyCoreClient::new(env, &core_id);
    let creator = Address::generate(env);

    let hunt_id = client.create_hunt(
        &creator,
        &String::from_str(env, "Test Hunt"),
        &String::from_str(env, "A hunt for testing required clues"),
        &None,
        &None,
        &0u32,
        &None,
        &None,
    );
    (client, creator, hunt_id)
}

#[test]
fn test_activate_hunt_with_zero_required_clues_fails() {
    let env = Env::default();
    env.ledger().set_timestamp(1_700_000_000);
    env.mock_all_auths();

    let (client, creator, hunt_id) = setup_hunt(&env);

    // Add 5 optional clues (is_required = false)
    for i in 0..5 {
        let question = String::from_str(&env, &format!("Optional question {}", i));
        let answer = String::from_str(&env, &format!("Optional answer {}", i));
        client.add_clue(&hunt_id, &question, &answer, &10u32, &false, &None, &None);
    }

    // Verify we have 5 clues but 0 required
    let hunt = client.get_hunt_info(&hunt_id);
    assert_eq!(hunt.total_clues, 5, "Hunt should have 5 total clues");
    assert_eq!(hunt.required_clues, 0, "Hunt should have 0 required clues");

    // Attempt to activate the hunt - should fail with NoRequiredClues
    let result = client.try_activate_hunt(&hunt_id, &creator);
    assert!(
        result.is_err(),
        "Activation should fail when there are zero required clues"
    );
}

#[test]
fn test_activate_hunt_with_one_required_clue_succeeds() {
    let env = Env::default();
    env.ledger().set_timestamp(1_700_000_000);
    env.mock_all_auths();

    let (client, creator, hunt_id) = setup_hunt(&env);

    // Add 3 optional clues
    for i in 0..3 {
        let question = String::from_str(&env, &format!("Optional question {}", i));
        let answer = String::from_str(&env, &format!("Optional answer {}", i));
        client.add_clue(&hunt_id, &question, &answer, &10u32, &false, &None, &None);
    }

    // Add 1 required clue
    client.add_clue(
        &hunt_id,
        &String::from_str(&env, "Required question"),
        &String::from_str(&env, "Required answer"),
        &20u32,
        &true,
        &None,
        &None,
    );

    // Verify hunt has 4 total clues with 1 required
    let hunt = client.get_hunt_info(&hunt_id);
    assert_eq!(hunt.total_clues, 4, "Hunt should have 4 total clues");
    assert_eq!(hunt.required_clues, 1, "Hunt should have 1 required clue");

    // Activate the hunt - should succeed
    let result = client.try_activate_hunt(&hunt_id, &creator);
    assert!(
        result.is_ok(),
        "Activation should succeed with at least one required clue"
    );

    // Verify hunt status changed to Active
    let activated_hunt = client.get_hunt_info(&hunt_id);
    assert_eq!(
        activated_hunt.status,
        hunty_core::types::HuntStatus::Active,
        "Hunt status should be Active after successful activation"
    );
}

#[test]
fn test_activate_hunt_after_adding_required_clue() {
    let env = Env::default();
    env.ledger().set_timestamp(1_700_000_000);
    env.mock_all_auths();

    let (client, creator, hunt_id) = setup_hunt(&env);

    // Add 3 optional clues
    for i in 0..3 {
        let question = String::from_str(&env, &format!("Optional question {}", i));
        let answer = String::from_str(&env, &format!("Optional answer {}", i));
        client.add_clue(&hunt_id, &question, &answer, &10u32, &false, &None, &None);
    }

    // Verify hunt has 3 clues but 0 required
    let hunt = client.get_hunt_info(&hunt_id);
    assert_eq!(hunt.total_clues, 3);
    assert_eq!(hunt.required_clues, 0);

    // Try to activate - should fail
    let result = client.try_activate_hunt(&hunt_id, &creator);
    assert!(
        result.is_err(),
        "Activation should fail with only optional clues"
    );

    // Now add a required clue
    client.add_clue(
        &hunt_id,
        &String::from_str(&env, "The required question"),
        &String::from_str(&env, "The required answer"),
        &25u32,
        &true,
        &None,
        &None,
    );

    // Verify hunt now has 1 required clue
    let hunt = client.get_hunt_info(&hunt_id);
    assert_eq!(hunt.total_clues, 4);
    assert_eq!(hunt.required_clues, 1);

    // Try to activate again - should succeed
    let result = client.try_activate_hunt(&hunt_id, &creator);
    assert!(
        result.is_ok(),
        "Activation should succeed after adding required clue"
    );
}

#[test]
fn test_activate_hunt_with_multiple_required_clues_succeeds() {
    let env = Env::default();
    env.ledger().set_timestamp(1_700_000_000);
    env.mock_all_auths();

    let (client, creator, hunt_id) = setup_hunt(&env);

    // Add 2 optional clues
    for i in 0..2 {
        let question = String::from_str(&env, &format!("Optional question {}", i));
        let answer = String::from_str(&env, &format!("Optional answer {}", i));
        client.add_clue(&hunt_id, &question, &answer, &10u32, &false, &None, &None);
    }

    // Add 3 required clues
    for i in 0..3 {
        let question = String::from_str(&env, &format!("Required question {}", i));
        let answer = String::from_str(&env, &format!("Required answer {}", i));
        client.add_clue(&hunt_id, &question, &answer, &20u32, &true, &None, &None);
    }

    // Verify hunt has 5 total clues with 3 required
    let hunt = client.get_hunt_info(&hunt_id);
    assert_eq!(hunt.total_clues, 5, "Hunt should have 5 total clues");
    assert_eq!(hunt.required_clues, 3, "Hunt should have 3 required clues");

    // Activate the hunt - should succeed
    let result = client.try_activate_hunt(&hunt_id, &creator);
    assert!(
        result.is_ok(),
        "Activation should succeed with multiple required clues"
    );
}

#[test]
fn test_activate_hunt_all_clues_required_succeeds() {
    let env = Env::default();
    env.ledger().set_timestamp(1_700_000_000);
    env.mock_all_auths();

    let (client, creator, hunt_id) = setup_hunt(&env);

    // Add 5 required clues (all are required)
    for i in 0..5 {
        let question = String::from_str(&env, &format!("Required question {}", i));
        let answer = String::from_str(&env, &format!("Required answer {}", i));
        client.add_clue(
            &hunt_id, &question, &answer, &15u32, &true, // All are required
            &None, &None,
        );
    }

    // Verify hunt has 5 total clues with 5 required
    let hunt = client.get_hunt_info(&hunt_id);
    assert_eq!(hunt.total_clues, 5);
    assert_eq!(hunt.required_clues, 5);

    // Activate the hunt - should succeed
    let result = client.try_activate_hunt(&hunt_id, &creator);
    assert!(
        result.is_ok(),
        "Activation should succeed when all clues are required"
    );
}

#[test]
fn test_cannot_activate_hunt_with_only_required_clues_zero() {
    let env = Env::default();
    env.ledger().set_timestamp(1_700_000_000);
    env.mock_all_auths();

    let (client, creator, hunt_id) = setup_hunt(&env);

    // Hunt has 0 total clues and 0 required clues
    let hunt = client.get_hunt_info(&hunt_id);
    assert_eq!(hunt.total_clues, 0);
    assert_eq!(hunt.required_clues, 0);

    // Try to activate - should fail with NoCluesAdded error (because total_clues == 0)
    let result = client.try_activate_hunt(&hunt_id, &creator);
    assert!(
        result.is_err(),
        "Activation should fail with no clues at all"
    );
}

#[test]
fn test_required_clue_count_tracks_correctly() {
    let env = Env::default();
    env.ledger().set_timestamp(1_700_000_000);
    env.mock_all_auths();

    let (client, creator, hunt_id) = setup_hunt(&env);

    // Add clues and track required_clues count after each addition
    for i in 0..10 {
        let question = String::from_str(&env, &format!("Question {}", i));
        let answer = String::from_str(&env, &format!("Answer {}", i));
        let is_required = i % 2 == 0; // Even clues are required, odd are optional

        client.add_clue(
            &hunt_id,
            &question,
            &answer,
            &10u32,
            &is_required,
            &None,
            &None,
        );

        // Verify required_clues count
        let hunt = client.get_hunt_info(&hunt_id);
        let expected_required = ((i + 1) + 1) / 2; // 0,1,1,2,2,3,3,4,4,5 = (i+2)/2 rounded down
        assert_eq!(
            hunt.required_clues, expected_required as u32,
            "Required clues count should be {} after adding clue {}",
            expected_required, i
        );
    }

    // Final state: 10 clues (5 required, 5 optional)
    let hunt = client.get_hunt_info(&hunt_id);
    assert_eq!(hunt.total_clues, 10);
    assert_eq!(hunt.required_clues, 5);

    // Activation should succeed
    let result = client.try_activate_hunt(&hunt_id, &creator);
    assert!(
        result.is_ok(),
        "Activation should succeed with required clues present"
    );
}

#[test]
fn test_activate_hunt_boundary_one_required_clue() {
    let env = Env::default();
    env.ledger().set_timestamp(1_700_000_000);
    env.mock_all_auths();

    let (client, creator, hunt_id) = setup_hunt(&env);

    // Add one required clue
    client.add_clue(
        &hunt_id,
        &String::from_str(&env, "Sole required clue"),
        &String::from_str(&env, "answer"),
        &50u32,
        &true,
        &None,
        &None,
    );

    // Verify exactly 1 required clue
    let hunt = client.get_hunt_info(&hunt_id);
    assert_eq!(hunt.required_clues, 1);

    // Activation should succeed (at minimum boundary)
    let result = client.try_activate_hunt(&hunt_id, &creator);
    assert!(
        result.is_ok(),
        "Activation should succeed with exactly 1 required clue"
    );
}

#[test]
fn test_unauthorized_user_cannot_activate() {
    let env = Env::default();
    env.ledger().set_timestamp(1_700_000_000);
    env.mock_all_auths();

    let (client, creator, hunt_id) = setup_hunt(&env);
    let unauthorized_user = Address::generate(&env);

    // Add a required clue
    client.add_clue(
        &hunt_id,
        &String::from_str(&env, "Required"),
        &String::from_str(&env, "answer"),
        &10u32,
        &true,
        &None,
        &None,
    );

    // Unauthorized user tries to activate - should fail with Unauthorized
    let result = client.try_activate_hunt(&hunt_id, &unauthorized_user);
    assert!(
        result.is_err(),
        "Unauthorized user should not be able to activate hunt"
    );

    // Creator can activate
    let result = client.try_activate_hunt(&hunt_id, &creator);
    assert!(result.is_ok(), "Creator should be able to activate hunt");
}
