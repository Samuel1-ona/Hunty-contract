use soroban_sdk::testutils::{Address as _, Ledger as _, Register as _};
use soroban_sdk::{token, Address, Env, String};
use hunty_core::HuntyCore;
use reward_manager::RewardManager;

fn setup_reward_manager(env: &Env) -> (Address, Address) {
    let reward_manager_id = env.register(RewardManager, ());
    let token_admin = Address::generate(env);
    let token_contract = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_address = token_contract.address();

    env.as_contract(&reward_manager_id, || {
        RewardManager::initialize(env.clone(), token_admin.clone(), token_address.clone())
            .unwrap();
    });

    (reward_manager_id, token_address)
}

fn as_core_contract<T>(env: &Env, contract_id: &Address, f: impl FnOnce(&Env) -> T) -> T {
    env.as_contract(contract_id, || f(env))
}

#[test]
fn test_cancel_hunt_with_reward_pool_refund() {
    let env = Env::default();
    env.ledger().set_timestamp(1_700_000_000);

    let creator = Address::generate(&env);
    let question = String::from_str(&env, "Valid question");
    let answer = String::from_str(&env, "a");

    let core_id = env.register_contract(None, HuntyCore);
    let (reward_manager_id, token_address) = setup_reward_manager(&env);

    // Mint tokens to creator
    let sac = token::StellarAssetClient::new(&env, &token_address);
    sac.mint(&creator, &5_000);

    env.mock_all_auths();

    // Create hunt, add clue, activate, and set reward manager
    let hunt_id = as_core_contract(&env, &core_id, |env| {
        let hunt_id = HuntyCore::create_hunt(
            env.clone(),
            creator.clone(),
            String::from_str(env, "Integration Refund Hunt"),
            String::from_str(env, "Testing refund on cancel"),
            None,
            None,
        )
        .unwrap();
        HuntyCore::add_clue(env.clone(), hunt_id, question, answer, 1, true).unwrap();
        HuntyCore::activate_hunt(env.clone(), hunt_id, creator.clone()).unwrap();
        HuntyCore::set_reward_manager(env.clone(), reward_manager_id.clone()).unwrap();
        hunt_id
    });

    // Create reward pool on reward manager
    env.as_contract(&reward_manager_id, || {
        RewardManager::create_reward_pool(env.clone(), creator.clone(), hunt_id, 0).unwrap();
    });

    // Fund the reward pool
    env.mock_all_auths();
    env.as_contract(&reward_manager_id, || {
        RewardManager::fund_reward_pool(env.clone(), creator.clone(), hunt_id, 5_000).unwrap();
    });

    // Verify pool is funded
    env.as_contract(&reward_manager_id, || {
        let balance = RewardManager::get_pool_balance(env.clone(), hunt_id);
        assert_eq!(balance, 5_000, "Pool should be funded before cancel");
    });

    // Cancel the hunt — should trigger cross-contract refund_pool call
    env.mock_all_auths();
    as_core_contract(&env, &core_id, |env| {
        HuntyCore::cancel_hunt(env.clone(), hunt_id, creator.clone()).unwrap();
    });

    // Assert pool balance is drained
    env.as_contract(&reward_manager_id, || {
        assert_eq!(
            RewardManager::get_pool_balance(env.clone(), hunt_id),
            0,
            "Pool balance should be 0 after refund"
        );
    });

    // Assert tokens were transferred back to creator
    let token_client = token::Client::new(&env, &token_address);
    assert_eq!(
        token_client.balance(&creator),
        5_000,
        "Creator should have the full amount refunded"
    );
    assert_eq!(
        token_client.balance(&reward_manager_id),
        0,
        "Reward manager contract should hold no tokens after refund"
    );
}
