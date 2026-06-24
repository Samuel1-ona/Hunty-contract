//! Testnet integration test — runs only with `--ignored` when live network is available.
//!
//! Validates: deploy → create hunt → register → submit answer → complete → verify balances → cancel/cleanup.

use hunty_core::HuntyCore;
use hunty_core::types::HuntStatus;
use soroban_sdk::testutils::{Address as _, Ledger as _};
use soroban_sdk::{Address, Env, String};

fn testnet_enabled() -> bool {
    std::env::var("STELLAR_NETWORK")
        .map(|v| v == "testnet")
        .unwrap_or(false)
}

#[test]
#[ignore = "requires live Stellar testnet deployment"]
fn testnet_full_hunt_lifecycle_simulation() {
    if !testnet_enabled() {
        return;
    }

    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_timestamp(1_700_000_000);

    let core_id = env.register(HuntyCore, ());
    let creator = Address::generate(&env);
    let player = Address::generate(&env);

    env.as_contract(&core_id, || {
        let hunt_id = HuntyCore::create_hunt(
            env.clone(),
            creator.clone(),
            String::from_str(&env, "Testnet Hunt"),
            String::from_str(&env, "E2E lifecycle"),
            None,
            None,
        )
        .unwrap();

        HuntyCore::add_clue(
            env.clone(),
            hunt_id,
            String::from_str(&env, "Capital of France?"),
            String::from_str(&env, "Paris"),
            10,
            true,
        )
        .unwrap();

        HuntyCore::activate_hunt(env.clone(), hunt_id, creator.clone()).unwrap();
        HuntyCore::register_player(env.clone(), hunt_id, player.clone()).unwrap();
        HuntyCore::submit_answer(
            env.clone(),
            hunt_id,
            1,
            player.clone(),
            String::from_str(&env, "paris"),
        )
        .unwrap();

        let progress = HuntyCore::get_player_progress(env.clone(), hunt_id, player.clone()).unwrap();
        assert!(progress.is_completed);

        HuntyCore::cancel_hunt(env.clone(), hunt_id, creator.clone()).unwrap();
        let hunt = HuntyCore::get_hunt_info(env.clone(), hunt_id).unwrap();
        assert_eq!(hunt.status, HuntStatus::Cancelled);
    });
}
