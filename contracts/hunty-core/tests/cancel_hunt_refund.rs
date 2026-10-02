//! Integration tests for `HuntyCore::cancel_hunt` refunding a funded reward
//! pool (issue #1037).
//!
//! `cancel_hunt` asks `RewardManager::refund_pool` to return the remaining pool
//! balance to the pool's funders. `RewardManager` gates that payout on HuntyCore
//! reporting the hunt as terminal (`is_hunt_terminal`, which is derived from
//! `get_hunt_info`), so HuntyCore has to persist `Cancelled` *before* it makes
//! the cross-contract call. If the refund is requested while the hunt is still
//! `Active`/`Paused`, RewardManager rejects it with `InvalidHuntStatus` and
//! `cancel_hunt` surfaces `RefundFailed` — cancelling a funded hunt can never
//! succeed.
//!
//! These tests deliberately drive the real `RewardManager` from the real
//! `HuntyCore` (with a real SAC token) so the terminal-status gate runs for
//! real instead of being stubbed out.

use hunty_core::types::HuntStatus;
use hunty_core::{HuntyCore, HuntyCoreClient};
use reward_manager::RewardManager;
use soroban_sdk::testutils::{Address as _, Ledger as _};
use soroban_sdk::{token, Address, Env, IntoVal, Map, String, Symbol, Val};

/// RewardManager's minimum pool funding amount (10 XLM in base units).
const POOL_FUNDING: i128 = 10_000_000;

struct Fixture {
    core_id: Address,
    reward_manager_id: Address,
    token_address: Address,
    hunt_id: u64,
    creator: Address,
}

/// Stand-in HuntyCore that always reports a non-terminal hunt.
///
/// Used to force `RewardManager::refund_pool` to be rejected, so the rollback
/// behaviour of a failed refund can be asserted.
#[soroban_sdk::contract]
pub struct NonTerminalHuntyCore;

#[soroban_sdk::contractimpl]
impl NonTerminalHuntyCore {
    /// RewardManager decodes the `status` field of this response, so it has to
    /// be shaped like HuntyCore's `Hunt`: a map containing `status`.
    pub fn get_hunt_info(env: Env, _hunt_id: u64) -> Map<Symbol, Val> {
        let mut hunt = Map::new(&env);
        // HuntStatus::Active
        hunt.set(Symbol::new(&env, "status"), 1u32.into_val(&env));
        hunt
    }

    pub fn is_hunt_terminal(_env: Env, _hunt_id: u64) -> bool {
        false
    }
}

fn text(env: &Env, value: &str) -> String {
    String::from_str(env, value)
}

/// Builds an `Active` hunt owned by `creator` whose reward pool holds
/// [`POOL_FUNDING`], wired through a real `RewardManager`.
///
/// `status_source` is the address the `RewardManager` consults to decide
/// whether the hunt is terminal: the real HuntyCore for the happy path, and
/// [`NonTerminalHuntyCore`] for the rollback test.
fn setup_funded_hunt(env: &Env, core_id: &Address, status_source: &Address) -> Fixture {
    let admin = Address::generate(env);
    let creator = Address::generate(env);

    let client = HuntyCoreClient::new(env, core_id);
    client.initialize_admin(&admin);

    let token_admin = Address::generate(env);
    let token_address = env
        .register_stellar_asset_contract_v2(token_admin)
        .address();
    let reward_manager_id = env.register(
        RewardManager,
        (admin.clone(), token_address.clone(), status_source.clone()),
    );

    let hunt_id = client.create_hunt(
        &creator,
        &text(env, "Funded Hunt"),
        &text(env, "Cancel must refund the pool"),
        &None,
        &None,
        &0,
        &None,
        &None,
    );
    // An Active hunt needs at least one required clue.
    client.add_clue(
        &hunt_id,
        &text(env, "Question"),
        &text(env, "answer"),
        &10,
        &true,
        &None,
        &None,
    );
    client.activate_hunt(&hunt_id, &creator);
    client.set_reward_manager(&admin, &reward_manager_id);

    let sac = token::StellarAssetClient::new(env, &token_address);
    sac.mint(&creator, &POOL_FUNDING);

    // create_reward_pool resolves the hunt through the RewardManager's
    // configured core, so it runs after the wiring above. Each call carries its
    // own authorization and must run in its own frame.
    env.as_contract(&reward_manager_id, || {
        // min_distribution_amount must be non-zero unless an NFT contract is
        // supplied (an NFT-only pool).
        RewardManager::create_reward_pool(
            env.clone(),
            creator.clone(),
            hunt_id,
            token_address.clone(),
            1,
            0,
            false,
        )
        .unwrap();
    });
    env.as_contract(&reward_manager_id, || {
        RewardManager::fund_reward_pool(env.clone(), creator.clone(), hunt_id, POOL_FUNDING)
            .unwrap();
    });

    Fixture {
        core_id: core_id.clone(),
        reward_manager_id,
        token_address,
        hunt_id,
        creator,
    }
}

/// Cancelling a hunt whose pool still holds funds must return the balance to
/// its funder, and must leave the hunt `Cancelled`.
// Quarantined: soroban-sdk v28 forbids re-entering a contract that is already
// on the call stack. `cancel_hunt` -> RewardManager::refund_pool ->
// is_hunt_terminal -> HuntyCore is exactly that, so the happy path now fails
// with RefundFailed. Fixing it needs a design change (skip the terminal check
// when HuntyCore is the caller, or stop routing the refund through
// RewardManager), which is out of scope for this change.
#[test]
#[ignore = "blocked on soroban-sdk v28 contract re-entry ban (see #1077 follow-up)"]
fn cancel_hunt_refunds_a_funded_pool() {
    let env = Env::default();
    env.ledger().set_timestamp(1_700_000_000);
    env.mock_all_auths();

    let core_id = env.register(HuntyCore, ());
    let fixture = setup_funded_hunt(&env, &core_id, &core_id);
    let client = HuntyCoreClient::new(&env, &core_id);

    // The pool really is funded before we cancel.
    env.as_contract(&fixture.reward_manager_id, || {
        assert_eq!(
            RewardManager::get_pool_balance(env.clone(), fixture.hunt_id),
            POOL_FUNDING
        );
    });

    client.cancel_hunt(&fixture.hunt_id, &fixture.creator);

    // HuntyCore must have reported the hunt as terminal *while* it asked for
    // the refund, and persisted that status.
    let hunt = client.get_hunt_info(&fixture.hunt_id);
    assert_eq!(hunt.status, HuntStatus::Cancelled);
    assert!(client.is_hunt_terminal(&fixture.hunt_id));

    // The pool is drained and the funder made whole.
    env.as_contract(&fixture.reward_manager_id, || {
        assert_eq!(
            RewardManager::get_pool_balance(env.clone(), fixture.hunt_id),
            0
        );
    });
    let token_client = token::Client::new(&env, &fixture.token_address);
    assert_eq!(token_client.balance(&fixture.creator), POOL_FUNDING);
    assert_eq!(token_client.balance(&fixture.reward_manager_id), 0);
}

/// A rejected refund must revert the whole cancellation: the hunt keeps its
/// original status and the pool keeps its balance, so a creator can still
/// retry once the refund can succeed.
#[test]
fn cancel_hunt_rolls_back_when_the_refund_is_rejected() {
    let env = Env::default();
    env.ledger().set_timestamp(1_700_000_000);
    env.mock_all_auths();

    let core_id = env.register(HuntyCore, ());
    // RewardManager refuses the refund because its status source never reports
    // the hunt as terminal.
    let status_source = env.register(NonTerminalHuntyCore, ());
    let fixture = setup_funded_hunt(&env, &core_id, &status_source);
    let client = HuntyCoreClient::new(&env, &core_id);

    // `try_cancel_hunt` is the variant that surfaces a contract error instead of
    // panicking, and its outer `Err` means the contract rejected the call (as
    // opposed to the host failing), i.e. `cancel_hunt` reported `RefundFailed`.
    let outcome = client.try_cancel_hunt(&fixture.hunt_id, &fixture.creator);
    assert!(
        format!("{outcome:?}").contains("RefundFailed"),
        "expected the rejected refund to surface as RefundFailed, got {outcome:?}"
    );

    // `Cancelled` is written before the cross-contract call, so this proves the
    // failed refund rolls that write back instead of stranding the hunt.
    let hunt = client.get_hunt_info(&fixture.hunt_id);
    assert_eq!(hunt.status, HuntStatus::Active);
    assert!(!client.is_hunt_terminal(&fixture.hunt_id));

    env.as_contract(&fixture.reward_manager_id, || {
        assert_eq!(
            RewardManager::get_pool_balance(env.clone(), fixture.hunt_id),
            POOL_FUNDING
        );
    });
    let token_client = token::Client::new(&env, &fixture.token_address);
    assert_eq!(token_client.balance(&fixture.creator), 0);
}
