/// Required Clues Validation Tests
/// Tests that hunt activation requires at least one required clue to succeed.
/// 
/// Acceptance Criteria:
/// - Create hunt, add only optional clues
/// - Attempt activation → should fail with NoRequiredClues
/// - Add one required clue → activation should succeed

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
// Tests for Required Clues Validation on Activation
// ============================================================================

#[test]
fn test_activate_hunt_with_zero_required_clues_fails() {
    let env = Env::default();
    env.ledger().set_timestamp(1_700_000_000);
    env.mock_all_auths();

    let core_id = setup_core_contract(&env);
    let creator = Address::generate(&env);

    as_core_contract(&env, &core_id, |env| {
        // Create a new hunt
        let hunt_id = HuntyCore::create_hunt(
            env.clone(),
            creator.clone(),
            String::from_str(env, "Hunt with Optional Clues Only"),
            String::from_str(env, "This hunt only has optional clues"),
            None,
            None,
            0,
            None,
        )
        .unwrap();

        // Add 5 optional clues (is_required = false)
        for i in 0..5 {
            let question = String::from_str(
                env,
                &format!("Optional question {}", i),
            );
            let answer = String::from_str(
                env,
                &format!("Optional answer {}", i),
            );
            HuntyCore::add_clue(
                env.clone(),
                hunt_id,
                question,
                answer,
                10,
                false, // is_required = false
                None,
            )
            .unwrap();
        }

        // Verify we have 5 clues but 0 required
        let hunt = HuntyCore::get_hunt(env.clone(), hunt_id).unwrap();
        assert_eq!(hunt.total_clues, 5, "Hunt should have 5 total clues");
        assert_eq!(hunt.required_clues, 0, "Hunt should have 0 required clues");

        // Attempt to activate the hunt - should fail with NoRequiredClues
        let result = HuntyCore::activate_hunt(env.clone(), hunt_id, creator.clone());
        assert!(
            result.is_err(),
            "Activation should fail when there are zero required clues"
        );
    });
}

#[test]
fn test_activate_hunt_with_one_required_clue_succeeds() {
    let env = Env::default();
    env.ledger().set_timestamp(1_700_000_000);
    env.mock_all_auths();

    let core_id = setup_core_contract(&env);
    let creator = Address::generate(&env);

    as_core_contract(&env, &core_id, |env| {
        // Create a new hunt
        let hunt_id = HuntyCore::create_hunt(
            env.clone(),
            creator.clone(),
            String::from_str(env, "Hunt with One Required Clue"),
            String::from_str(env, "This hunt has one required clue"),
            None,
            None,
            0,
            None,
        )
        .unwrap();

        // Add 3 optional clues
        for i in 0..3 {
            let question = String::from_str(
                env,
                &format!("Optional question {}", i),
            );
            let answer = String::from_str(
                env,
                &format!("Optional answer {}", i),
            );
            HuntyCore::add_clue(
                env.clone(),
                hunt_id,
                question,
                answer,
                10,
                false, // is_required = false
                None,
            )
            .unwrap();
        }

        // Add 1 required clue
        HuntyCore::add_clue(
            env.clone(),
            hunt_id,
            String::from_str(env, "Required question"),
            String::from_str(env, "Required answer"),
            20,
            true, // is_required = true
            None,
        )
        .unwrap();

        // Verify hunt has 4 total clues with 1 required
        let hunt = HuntyCore::get_hunt(env.clone(), hunt_id).unwrap();
        assert_eq!(hunt.total_clues, 4, "Hunt should have 4 total clues");
        assert_eq!(hunt.required_clues, 1, "Hunt should have 1 required clue");

        // Activate the hunt - should succeed
        let result = HuntyCore::activate_hunt(env.clone(), hunt_id, creator.clone());
        assert!(result.is_ok(), "Activation should succeed with at least one required clue");

        // Verify hunt status changed to Active
        let activated_hunt = HuntyCore::get_hunt(env.clone(), hunt_id).unwrap();
        assert_eq!(
            activated_hunt.status.to_string(),
            "Active",
            "Hunt status should be Active after successful activation"
        );
    });
}

#[test]
fn test_activate_hunt_after_adding_required_clue() {
    let env = Env::default();
    env.ledger().set_timestamp(1_700_000_000);
    env.mock_all_auths();

    let core_id = setup_core_contract(&env);
    let creator = Address::generate(&env);

    as_core_contract(&env, &core_id, |env| {
        // Create a new hunt
        let hunt_id = HuntyCore::create_hunt(
            env.clone(),
            creator.clone(),
            String::from_str(env, "Hunt with No Required Clues Initially"),
            String::from_str(env, "Test required clue addition flow"),
            None,
            None,
            0,
            None,
        )
        .unwrap();

        // Add 3 optional clues
        for i in 0..3 {
            let question = String::from_str(
                env,
                &format!("Optional question {}", i),
            );
            let answer = String::from_str(
                env,
                &format!("Optional answer {}", i),
            );
            HuntyCore::add_clue(
                env.clone(),
                hunt_id,
                question,
                answer,
                10,
                false,
                None,
            )
            .unwrap();
        }

        // Verify hunt has 3 clues but 0 required
        let hunt = HuntyCore::get_hunt(env.clone(), hunt_id).unwrap();
        assert_eq!(hunt.total_clues, 3);
        assert_eq!(hunt.required_clues, 0);

        // Try to activate - should fail
        let result = HuntyCore::activate_hunt(env.clone(), hunt_id, creator.clone());
        assert!(result.is_err(), "Activation should fail with only optional clues");

        // Now add a required clue
        HuntyCore::add_clue(
            env.clone(),
            hunt_id,
            String::from_str(env, "The required question"),
            String::from_str(env, "The required answer"),
            25,
            true,
            None,
        )
        .unwrap();

        // Verify hunt now has 1 required clue
        let hunt = HuntyCore::get_hunt(env.clone(), hunt_id).unwrap();
        assert_eq!(hunt.total_clues, 4);
        assert_eq!(hunt.required_clues, 1);

        // Try to activate again - should succeed
        let result = HuntyCore::activate_hunt(env.clone(), hunt_id, creator.clone());
        assert!(result.is_ok(), "Activation should succeed after adding required clue");
    });
}

#[test]
fn test_activate_hunt_with_multiple_required_clues_succeeds() {
    let env = Env::default();
    env.ledger().set_timestamp(1_700_000_000);
    env.mock_all_auths();

    let core_id = setup_core_contract(&env);
    let creator = Address::generate(&env);

    as_core_contract(&env, &core_id, |env| {
        // Create a new hunt
        let hunt_id = HuntyCore::create_hunt(
            env.clone(),
            creator.clone(),
            String::from_str(env, "Hunt with Multiple Required Clues"),
            String::from_str(env, "This hunt has multiple required clues"),
            None,
            None,
            0,
            None,
        )
        .unwrap();

        // Add 2 optional clues
        for i in 0..2 {
            let question = String::from_str(
                env,
                &format!("Optional question {}", i),
            );
            let answer = String::from_str(
                env,
                &format!("Optional answer {}", i),
            );
            HuntyCore::add_clue(
                env.clone(),
                hunt_id,
                question,
                answer,
                10,
                false,
                None,
            )
            .unwrap();
        }

        // Add 3 required clues
        for i in 0..3 {
            let question = String::from_str(
                env,
                &format!("Required question {}", i),
            );
            let answer = String::from_str(
                env,
                &format!("Required answer {}", i),
            );
            HuntyCore::add_clue(
                env.clone(),
                hunt_id,
                question,
                answer,
                20,
                true,
                None,
            )
            .unwrap();
        }

        // Verify hunt has 5 total clues with 3 required
        let hunt = HuntyCore::get_hunt(env.clone(), hunt_id).unwrap();
        assert_eq!(hunt.total_clues, 5, "Hunt should have 5 total clues");
        assert_eq!(hunt.required_clues, 3, "Hunt should have 3 required clues");

        // Activate the hunt - should succeed
        let result = HuntyCore::activate_hunt(env.clone(), hunt_id, creator.clone());
        assert!(
            result.is_ok(),
            "Activation should succeed with multiple required clues"
        );
    });
}

#[test]
fn test_activate_hunt_all_clues_required_succeeds() {
    let env = Env::default();
    env.ledger().set_timestamp(1_700_000_000);
    env.mock_all_auths();

    let core_id = setup_core_contract(&env);
    let creator = Address::generate(&env);

    as_core_contract(&env, &core_id, |env| {
        // Create a new hunt
        let hunt_id = HuntyCore::create_hunt(
            env.clone(),
            creator.clone(),
            String::from_str(env, "Hunt with All Required Clues"),
            String::from_str(env, "All clues are required"),
            None,
            None,
            0,
            None,
        )
        .unwrap();

        // Add 5 required clues (all are required)
        for i in 0..5 {
            let question = String::from_str(
                env,
                &format!("Required question {}", i),
            );
            let answer = String::from_str(
                env,
                &format!("Required answer {}", i),
            );
            HuntyCore::add_clue(
                env.clone(),
                hunt_id,
                question,
                answer,
                15,
                true, // All are required
                None,
            )
            .unwrap();
        }

        // Verify hunt has 5 total clues with 5 required
        let hunt = HuntyCore::get_hunt(env.clone(), hunt_id).unwrap();
        assert_eq!(hunt.total_clues, 5);
        assert_eq!(hunt.required_clues, 5);

        // Activate the hunt - should succeed
        let result = HuntyCore::activate_hunt(env.clone(), hunt_id, creator.clone());
        assert!(result.is_ok(), "Activation should succeed when all clues are required");
    });
}

#[test]
fn test_cannot_activate_hunt_with_only_required_clues_zero() {
    let env = Env::default();
    env.ledger().set_timestamp(1_700_000_000);
    env.mock_all_auths();

    let core_id = setup_core_contract(&env);
    let creator = Address::generate(&env);

    as_core_contract(&env, &core_id, |env| {
        // Create a new hunt
        let hunt_id = HuntyCore::create_hunt(
            env.clone(),
            creator.clone(),
            String::from_str(env, "Empty Hunt"),
            String::from_str(env, "Hunt with no clues"),
            None,
            None,
            0,
            None,
        )
        .unwrap();

        // Hunt has 0 total clues and 0 required clues
        let hunt = HuntyCore::get_hunt(env.clone(), hunt_id).unwrap();
        assert_eq!(hunt.total_clues, 0);
        assert_eq!(hunt.required_clues, 0);

        // Try to activate - should fail with NoCluesAdded error (because total_clues == 0)
        let result = HuntyCore::activate_hunt(env.clone(), hunt_id, creator.clone());
        assert!(result.is_err(), "Activation should fail with no clues at all");
    });
}

#[test]
fn test_required_clue_count_tracks_correctly() {
    let env = Env::default();
    env.ledger().set_timestamp(1_700_000_000);
    env.mock_all_auths();

    let core_id = setup_core_contract(&env);
    let creator = Address::generate(&env);

    as_core_contract(&env, &core_id, |env| {
        // Create a new hunt
        let hunt_id = HuntyCore::create_hunt(
            env.clone(),
            creator.clone(),
            String::from_str(env, "Hunt for Tracking"),
            String::from_str(env, "Track required clue count"),
            None,
            None,
            0,
            None,
        )
        .unwrap();

        // Add clues and track required_clues count after each addition
        for i in 0..10 {
            let question = String::from_str(
                env,
                &format!("Question {}", i),
            );
            let answer = String::from_str(
                env,
                &format!("Answer {}", i),
            );
            let is_required = i % 2 == 0; // Even clues are required, odd are optional

            HuntyCore::add_clue(
                env.clone(),
                hunt_id,
                question,
                answer,
                10,
                is_required,
                None,
            )
            .unwrap();

            // Verify required_clues count
            let hunt = HuntyCore::get_hunt(env.clone(), hunt_id).unwrap();
            let expected_required = ((i + 1) + 1) / 2; // 0,1,1,2,2,3,3,4,4,5 = (i+2)/2 rounded down
            assert_eq!(
                hunt.required_clues, expected_required as u32,
                "Required clues count should be {} after adding clue {}",
                expected_required,
                i
            );
        }

        // Final state: 10 clues (5 required, 5 optional)
        let hunt = HuntyCore::get_hunt(env.clone(), hunt_id).unwrap();
        assert_eq!(hunt.total_clues, 10);
        assert_eq!(hunt.required_clues, 5);

        // Activation should succeed
        let result = HuntyCore::activate_hunt(env.clone(), hunt_id, creator.clone());
        assert!(result.is_ok(), "Activation should succeed with required clues present");
    });
}

#[test]
fn test_activate_hunt_boundary_one_required_clue() {
    let env = Env::default();
    env.ledger().set_timestamp(1_700_000_000);
    env.mock_all_auths();

    let core_id = setup_core_contract(&env);
    let creator = Address::generate(&env);

    as_core_contract(&env, &core_id, |env| {
        // Create hunt with exactly one required clue (boundary case)
        let hunt_id = HuntyCore::create_hunt(
            env.clone(),
            creator.clone(),
            String::from_str(env, "Boundary Hunt"),
            String::from_str(env, "Exactly one required clue"),
            None,
            None,
            0,
            None,
        )
        .unwrap();

        // Add one required clue
        HuntyCore::add_clue(
            env.clone(),
            hunt_id,
            String::from_str(env, "Sole required clue"),
            String::from_str(env, "answer"),
            50,
            true,
            None,
        )
        .unwrap();

        // Verify exactly 1 required clue
        let hunt = HuntyCore::get_hunt(env.clone(), hunt_id).unwrap();
        assert_eq!(hunt.required_clues, 1);

        // Activation should succeed (at minimum boundary)
        let result = HuntyCore::activate_hunt(env.clone(), hunt_id, creator.clone());
        assert!(
            result.is_ok(),
            "Activation should succeed with exactly 1 required clue"
        );
    });
}

#[test]
fn test_unauthorized_user_cannot_activate() {
    let env = Env::default();
    env.ledger().set_timestamp(1_700_000_000);
    env.mock_all_auths();

    let core_id = setup_core_contract(&env);
    let creator = Address::generate(&env);
    let unauthorized_user = Address::generate(&env);

    as_core_contract(&env, &core_id, |env| {
        // Create hunt as creator
        let hunt_id = HuntyCore::create_hunt(
            env.clone(),
            creator.clone(),
            String::from_str(env, "Protected Hunt"),
            String::from_str(env, "Only creator can activate"),
            None,
            None,
            0,
            None,
        )
        .unwrap();

        // Add a required clue
        HuntyCore::add_clue(
            env.clone(),
            hunt_id,
            String::from_str(env, "Required"),
            String::from_str(env, "answer"),
            10,
            true,
            None,
        )
        .unwrap();

        // Unauthorized user tries to activate - should fail with Unauthorized
        let result =
            HuntyCore::activate_hunt(env.clone(), hunt_id, unauthorized_user.clone());
        assert!(
            result.is_err(),
            "Unauthorized user should not be able to activate hunt"
        );

        // Creator can activate
        let result = HuntyCore::activate_hunt(env.clone(), hunt_id, creator.clone());
        assert!(result.is_ok(), "Creator should be able to activate hunt");
    });
}
