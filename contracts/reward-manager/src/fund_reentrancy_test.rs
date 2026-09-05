//! Regression test for the fund_reward_pool checks-effects-interactions fix
//! (issue #870).
//!
//! `fund_reward_pool` used to transfer tokens to the contract *before*
//! updating the pool balance, with no reentrancy guard. This test wires up a
//! token whose `transfer` attempts to call straight back into
//! `fund_reward_pool` for the same hunt, to confirm the pool is never
//! double-credited.
//!
//! In practice the reentrant call traps before it reaches any reward-manager
//! logic: the Soroban host itself refuses to re-enter a contract that is
//! already on the call stack ("Contract re-entry is not allowed"), so this
//! specific attack is blocked at the platform level regardless of the guard.
//! The fix — reordering the state writes ahead of the token transfer and
//! reusing the existing `ReentrancyGuard` — is still the right change:
//! it matches `distribute_rewards`'s existing pattern in this file, costs one
//! storage write, and is the actual backstop should the call topology ever
//! change (e.g. an authorized/opt-in reentrant call path). This test
//! documents the resulting end-to-end guarantee: the attempted callback is
//! rejected and the original funding call still completes exactly once with
//! correct state.

use crate::RewardManager;
use soroban_sdk::{
    contract, contractimpl, testutils::Address as _, Address, Env, IntoVal, Symbol, Val, Vec,
};

#[contract]
pub struct ReentrantFundingToken;

#[contractimpl]
impl ReentrantFundingToken {
    /// Arms this token to attempt one reentrant `fund_reward_pool` call, with
    /// the same arguments, the next time its `transfer` is invoked.
    pub fn configure(env: Env, target: Address, funder: Address, hunt_id: u64, amount: i128) {
        env.storage()
            .instance()
            .set(&Symbol::new(&env, "target"), &target);
        env.storage()
            .instance()
            .set(&Symbol::new(&env, "funder"), &funder);
        env.storage()
            .instance()
            .set(&Symbol::new(&env, "hunt_id"), &hunt_id);
        env.storage()
            .instance()
            .set(&Symbol::new(&env, "amount"), &amount);
        env.storage()
            .instance()
            .set(&Symbol::new(&env, "armed"), &true);
    }

    /// Whether the reentrant call attempted during `transfer` was rejected.
    pub fn reentry_was_rejected(env: Env) -> bool {
        env.storage()
            .instance()
            .get(&Symbol::new(&env, "reentry_rejected"))
            .unwrap_or(false)
    }

    pub fn balance(_env: Env, _id: Address) -> i128 {
        0
    }

    pub fn transfer(env: Env, _from: Address, _to: Address, _amount: i128) {
        let armed: bool = env
            .storage()
            .instance()
            .get(&Symbol::new(&env, "armed"))
            .unwrap_or(false);
        if !armed {
            return;
        }
        // Disarm before recursing so the reentrant attempt can't itself
        // trigger another nested attempt.
        env.storage()
            .instance()
            .set(&Symbol::new(&env, "armed"), &false);

        let target: Address = env
            .storage()
            .instance()
            .get(&Symbol::new(&env, "target"))
            .unwrap();
        let funder: Address = env
            .storage()
            .instance()
            .get(&Symbol::new(&env, "funder"))
            .unwrap();
        let hunt_id: u64 = env
            .storage()
            .instance()
            .get(&Symbol::new(&env, "hunt_id"))
            .unwrap();
        let amount: i128 = env
            .storage()
            .instance()
            .get(&Symbol::new(&env, "amount"))
            .unwrap();

        let mut args: Vec<Val> = Vec::new(&env);
        args.push_back(funder.into_val(&env));
        args.push_back(hunt_id.into_val(&env));
        args.push_back(amount.into_val(&env));

        // `try_invoke_contract` lets us observe the rejection instead of
        // trapping the whole transaction, so the original (outer) funding
        // call can still complete normally afterward.
        let result: Result<
            Result<(), soroban_sdk::ConversionError>,
            Result<crate::RewardErrorCode, soroban_sdk::InvokeError>,
        > = env.try_invoke_contract(&target, &Symbol::new(&env, "fund_reward_pool"), args);

        let rejected = !matches!(result, Ok(Ok(())));
        env.storage()
            .instance()
            .set(&Symbol::new(&env, "reentry_rejected"), &rejected);
    }
}

#[test]
fn test_fund_reward_pool_rejects_reentrant_funding() {
    let env = Env::default();
    env.mock_all_auths_allowing_non_root_auth();

    let contract_id = env.register(RewardManager, ());
    let token_id = env.register(ReentrantFundingToken, ());

    let admin = Address::generate(&env);
    let creator = Address::generate(&env);

    let hunty_core = Address::generate(&env);
    env.as_contract(&contract_id, || {
        RewardManager::initialize(env.clone(), admin, token_id.clone(), hunty_core).unwrap();
        // Non-zero min_distribution_amount avoids the unrelated NFT-only-pool
        // validation rule; it plays no role in this reentrancy scenario.
        RewardManager::create_reward_pool(
            env.clone(),
            creator.clone(),
            1,
            token_id.clone(),
            10_000_000,
            0,
            true,
        )
        .unwrap();
    });

    env.as_contract(&token_id, || {
        ReentrantFundingToken::configure(
            env.clone(),
            contract_id.clone(),
            creator.clone(),
            1,
            50_000_000,
        );
    });

    env.as_contract(&contract_id, || {
        let result = RewardManager::fund_reward_pool(env.clone(), creator.clone(), 1, 50_000_000);
        assert!(result.is_ok(), "the original funding call should succeed");

        // Only the original funding should be recorded — the reentrant call
        // made during the token transfer must not have been able to credit
        // the pool a second time.
        assert_eq!(RewardManager::get_pool_balance(env.clone(), 1), 50_000_000);
        let status = RewardManager::get_reward_pool(env.clone(), 1).unwrap();
        assert_eq!(status.total_deposited, 50_000_000);
    });

    env.as_contract(&token_id, || {
        assert!(
            ReentrantFundingToken::reentry_was_rejected(env.clone()),
            "reentrant fund_reward_pool call should have been rejected"
        );
    });
}
