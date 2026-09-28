use crate::types::HuntStatus;
use crate::HuntyCore;
use soroban_sdk::testutils::{Address as _, Ledger as _};
use soroban_sdk::{Address, Env, String};

#[test]
fn reactivation_preserves_activation_timestamp_and_registration_gate() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_timestamp(1_000);

    let contract_id = env.register(HuntyCore, ());
    let creator = Address::generate(&env);
    let player = Address::generate(&env);

    let hunt_id = env.as_contract(&contract_id, || {
        let hunt_id = HuntyCore::create_hunt(
            env.clone(),
            creator.clone(),
            String::from_str(&env, "Paused lifecycle"),
            String::from_str(&env, "Reactivation keeps its original activation time"),
            None,
            None,
            0,
            None,
            None,
        )
        .unwrap();
        HuntyCore::add_clue(
            env.clone(),
            hunt_id,
            String::from_str(&env, "Question"),
            String::from_str(&env, "answer"),
            10,
            true,
            None,
            None,
        )
        .unwrap();
        HuntyCore::activate_hunt(env.clone(), hunt_id, creator.clone()).unwrap();
        hunt_id
    });

    let activated_at = env.as_contract(&contract_id, || {
        HuntyCore::get_hunt_info(env.clone(), hunt_id)
            .unwrap()
            .activated_at
    });
    assert_eq!(activated_at, 1_000);

    env.ledger().set_timestamp(2_000);
    env.as_contract(&contract_id, || {
        HuntyCore::deactivate_hunt(env.clone(), hunt_id, creator.clone()).unwrap();
    });
    assert!(env.as_contract(&contract_id, || {
        HuntyCore::register_player(env.clone(), hunt_id, player.clone()).is_err()
    }));

    env.ledger().set_timestamp(3_000);
    env.as_contract(&contract_id, || {
        HuntyCore::activate_hunt(env.clone(), hunt_id, creator.clone()).unwrap();
    });

    env.as_contract(&contract_id, || {
        let hunt = HuntyCore::get_hunt_info(env.clone(), hunt_id).unwrap();
        assert_eq!(hunt.status, HuntStatus::Active);
        assert_eq!(hunt.activated_at, activated_at);
        HuntyCore::register_player(env.clone(), hunt_id, player).unwrap();
    });
}

#[test]
fn paused_status_has_stable_wire_discriminants() {
    // The explicit values in HuntStatus are part of the on-chain wire format.
    // Keep the new state append-only so old EmergencyStopped/Archived records
    // cannot silently turn into a different state after an upgrade.
    assert_eq!(HuntStatus::Draft as u32, 0);
    assert_eq!(HuntStatus::Active as u32, 1);
    assert_eq!(HuntStatus::Completed as u32, 2);
    assert_eq!(HuntStatus::Cancelled as u32, 3);
    assert_eq!(HuntStatus::Paused as u32, 4);
    assert_eq!(HuntStatus::EmergencyStopped as u32, 5);
    assert_eq!(HuntStatus::Archived as u32, 6);
}
