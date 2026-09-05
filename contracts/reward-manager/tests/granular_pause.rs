//! Granular pause for reward-manager (issue #628).
//!
//! `hunty-core` can pause registrations, answers and rewards independently.
//! reward-manager had a single `PAUSED` flag — and that flag was only ever read
//! by `emergency_withdraw` as a precondition, so calling `pause()` did not
//! actually stop funding or distribution.
//!
//! These tests pin down the split: funding and distribution now pause
//! independently, the global stop still implies both, and the gates run before
//! any money moves.
//!
//! Written as a standalone integration target because the crate's `src/test.rs`
//! unit-test module does not currently compile — see the PR description.

use reward_manager::RewardManager;
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{Address, Env};

struct Fixture {
    env: Env,
    contract_id: Address,
    admin: Address,
    stranger: Address,
}

fn setup() -> Fixture {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(RewardManager, ());
    let admin = Address::generate(&env);
    let stranger = Address::generate(&env);
    let xlm_token = Address::generate(&env);

    let hunty_core = Address::generate(&env);
    env.as_contract(&contract_id, || {
        RewardManager::initialize(env.clone(), admin.clone(), xlm_token, hunty_core).unwrap();
    });

    Fixture {
        env,
        contract_id,
        admin,
        stranger,
    }
}

fn in_contract<T>(fx: &Fixture, f: impl FnOnce(&Env) -> T) -> T {
    fx.env.as_contract(&fx.contract_id, || f(&fx.env))
}

// Each contract call needs its own auth frame, so these wrap one call each
// rather than batching several into a single `as_contract` block.
fn pause_funding(fx: &Fixture) {
    in_contract(fx, |env| {
        RewardManager::pause_funding(env.clone(), fx.admin.clone()).unwrap();
    });
}

fn unpause_funding(fx: &Fixture) {
    in_contract(fx, |env| {
        RewardManager::unpause_funding(env.clone(), fx.admin.clone()).unwrap();
    });
}

fn pause_distribution(fx: &Fixture) {
    in_contract(fx, |env| {
        RewardManager::pause_distribution(env.clone(), fx.admin.clone()).unwrap();
    });
}

fn pause_global(fx: &Fixture) {
    in_contract(fx, |env| {
        RewardManager::pause(
            env.clone(),
            fx.admin.clone(),
            soroban_sdk::String::from_str(env, "incident"),
        )
        .unwrap();
    });
}

fn unpause_global(fx: &Fixture) {
    in_contract(fx, |env| {
        RewardManager::unpause(env.clone(), fx.admin.clone()).unwrap();
    });
}

/// The funding gate runs before any token interaction, so a paused call returns
/// `FundingPaused`. When funding is allowed the call proceeds past the gate and
/// fails later for an unrelated reason — never with `FundingPaused`.
fn funding_blocked(fx: &Fixture) -> bool {
    let funder = Address::generate(&fx.env);
    let result = in_contract(fx, |env| {
        RewardManager::fund_reward_pool(env.clone(), funder.clone(), 1, 10_000_000)
    });
    matches!(
        result,
        Err(reward_manager::errors::RewardErrorCode::FundingPaused)
    )
}

fn distribution_blocked(fx: &Fixture) -> bool {
    let player = Address::generate(&fx.env);
    let result = in_contract(fx, |env| {
        RewardManager::distribute_proportional(env.clone(), 1, player.clone(), 5, 10)
    });
    matches!(
        result,
        Err(reward_manager::errors::RewardErrorCode::DistributionPaused)
    )
}

// ── Default state ──────────────────────────────────────────────────────────

#[test]
fn nothing_is_paused_by_default() {
    let fx = setup();

    let (global, funding, distribution) =
        in_contract(&fx, |env| RewardManager::get_pause_state(env.clone()));

    assert!(!global);
    assert!(!funding);
    assert!(!distribution);
    assert!(!funding_blocked(&fx));
    assert!(!distribution_blocked(&fx));
}

// ── Independence: the point of the issue ───────────────────────────────────

#[test]
fn pausing_funding_leaves_distribution_running() {
    let fx = setup();

    in_contract(&fx, |env| {
        RewardManager::pause_funding(env.clone(), fx.admin.clone()).unwrap();
    });

    let (global, funding, distribution) =
        in_contract(&fx, |env| RewardManager::get_pause_state(env.clone()));
    assert!(!global, "granular pause must not engage the global stop");
    assert!(funding);
    assert!(!distribution);

    assert!(funding_blocked(&fx));
    assert!(
        !distribution_blocked(&fx),
        "owed rewards must still be payable while funding is frozen"
    );
}

#[test]
fn pausing_distribution_leaves_funding_open() {
    let fx = setup();

    in_contract(&fx, |env| {
        RewardManager::pause_distribution(env.clone(), fx.admin.clone()).unwrap();
    });

    let (global, funding, distribution) =
        in_contract(&fx, |env| RewardManager::get_pause_state(env.clone()));
    assert!(!global);
    assert!(!funding);
    assert!(distribution);

    assert!(distribution_blocked(&fx));
    assert!(
        !funding_blocked(&fx),
        "creators must still be able to top a pool up while distribution is halted"
    );
}

// ── Global stop still implies both ─────────────────────────────────────────

#[test]
fn global_pause_implies_both() {
    let fx = setup();

    in_contract(&fx, |env| {
        RewardManager::pause(
            env.clone(),
            fx.admin.clone(),
            soroban_sdk::String::from_str(env, "incident"),
        )
        .unwrap();
    });

    let (global, funding, distribution) =
        in_contract(&fx, |env| RewardManager::get_pause_state(env.clone()));
    assert!(global);
    assert!(funding, "global stop must imply funding paused");
    assert!(distribution, "global stop must imply distribution paused");

    assert!(funding_blocked(&fx));
    assert!(distribution_blocked(&fx));
}

#[test]
fn global_pause_now_actually_blocks_money_movement() {
    // Regression guard for the bug found while implementing #628: `pause()` set
    // a flag that only `emergency_withdraw` ever read, so funding and
    // distribution continued while the contract reported itself paused.
    let fx = setup();

    assert!(!funding_blocked(&fx));
    assert!(!distribution_blocked(&fx));

    in_contract(&fx, |env| {
        RewardManager::pause(
            env.clone(),
            fx.admin.clone(),
            soroban_sdk::String::from_str(env, "incident"),
        )
        .unwrap();
    });

    assert!(funding_blocked(&fx));
    assert!(distribution_blocked(&fx));
}

#[test]
fn unpause_does_not_clear_a_granular_flag() {
    let fx = setup();

    pause_distribution(&fx);
    pause_global(&fx);
    unpause_global(&fx);

    let (global, funding, distribution) =
        in_contract(&fx, |env| RewardManager::get_pause_state(env.clone()));
    assert!(!global, "the emergency stop was lifted");
    assert!(!funding);
    assert!(
        distribution,
        "lifting the global stop must not silently resume a separately paused half"
    );
    assert!(distribution_blocked(&fx));
}

#[test]
fn raw_flags_distinguish_granular_from_global() {
    let fx = setup();

    pause_funding(&fx);
    pause_global(&fx);

    // Effective state says both halves are paused...
    let (_, funding, distribution) =
        in_contract(&fx, |env| RewardManager::get_pause_state(env.clone()));
    assert!(funding && distribution);

    // ...but only funding was paused in its own right.
    let (raw_funding, raw_distribution) =
        in_contract(&fx, |env| RewardManager::get_raw_pause_flags(env.clone()));
    assert!(raw_funding);
    assert!(!raw_distribution);
}

// ── Resuming ───────────────────────────────────────────────────────────────

#[test]
fn unpause_funding_restores_funding_only() {
    let fx = setup();

    pause_funding(&fx);
    pause_distribution(&fx);
    unpause_funding(&fx);

    assert!(!funding_blocked(&fx));
    assert!(distribution_blocked(&fx));
}

#[test]
fn granular_pause_is_idempotent() {
    let fx = setup();

    pause_funding(&fx);
    pause_funding(&fx);
    assert!(funding_blocked(&fx));

    unpause_funding(&fx);
    unpause_funding(&fx);
    assert!(!funding_blocked(&fx));
}

// ── Authorization ──────────────────────────────────────────────────────────

#[test]
fn only_the_admin_may_pause() {
    let fx = setup();

    for result in [
        in_contract(&fx, |env| {
            RewardManager::pause_funding(env.clone(), fx.stranger.clone())
        }),
        in_contract(&fx, |env| {
            RewardManager::pause_distribution(env.clone(), fx.stranger.clone())
        }),
        in_contract(&fx, |env| {
            RewardManager::unpause_funding(env.clone(), fx.stranger.clone())
        }),
        in_contract(&fx, |env| {
            RewardManager::unpause_distribution(env.clone(), fx.stranger.clone())
        }),
    ] {
        assert_eq!(
            result,
            Err(reward_manager::errors::RewardErrorCode::Unauthorized)
        );
    }

    // Nothing was paused by the rejected attempts.
    let (global, funding, distribution) =
        in_contract(&fx, |env| RewardManager::get_pause_state(env.clone()));
    assert!(!global && !funding && !distribution);
}
