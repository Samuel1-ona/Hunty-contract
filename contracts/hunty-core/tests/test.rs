use hunty_core::{HuntyCore, HuntyCoreClient};
use reward_manager::RewardManager;
use soroban_sdk::testutils::{Address as _, Ledger as _};
use soroban_sdk::{token, Address, Env, String};

fn setup_reward_manager(env: &Env) -> (Address, Address) {
    let reward_manager_id = env.register(RewardManager, ());
    let token_admin = Address::generate(env);
    let token_contract = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_address = token_contract.address();

    env.as_contract(&reward_manager_id, || {
        RewardManager::initialize(env.clone(), token_admin.clone(), token_address.clone()).unwrap();
    });

    (reward_manager_id, token_address)
}

#[test]
fn test_cancel_hunt_with_reward_pool_refund() {
    let env = Env::default();
    env.ledger().set_timestamp(1_700_000_000);
    env.mock_all_auths();

    let creator = Address::generate(&env);
    let admin = Address::generate(&env);
    let question = String::from_str(&env, "Valid question");
    let answer = String::from_str(&env, "a");

    let core_id = env.register(HuntyCore, ());
    let (reward_manager_id, token_address) = setup_reward_manager(&env);

    let client = HuntyCoreClient::new(&env, &core_id);

    // Initialize admin
    client.initialize_admin(&admin);

    // Mint tokens to creator
    let sac = token::StellarAssetClient::new(&env, &token_address);
    sac.mint(&creator, &5_000);

    // Create hunt, add clue, activate, and set reward manager
    let hunt_id = client.create_hunt(
        &creator,
        &String::from_str(&env, "Integration Refund Hunt"),
        &String::from_str(&env, "Testing refund on cancel"),
        &None,
        &None,
    );
    client.add_clue(&hunt_id, &question, &answer, &1, &true, &1);
    client.activate_hunt(&hunt_id, &creator);
    client.set_reward_manager(&admin, &reward_manager_id);

    // Create reward pool on reward manager
    env.as_contract(&reward_manager_id, || {
        RewardManager::create_reward_pool(env.clone(), creator.clone(), hunt_id, 0).unwrap();
        RewardManager::fund_reward_pool(env.clone(), creator.clone(), hunt_id, 5_000).unwrap();
    });

    env.as_contract(&reward_manager_id, || {
        assert_eq!(RewardManager::get_pool_balance(env.clone(), hunt_id), 5_000);
    });

    // Cancel the hunt — should trigger cross-contract refund_pool call
    client.cancel_hunt(&hunt_id, &creator);

    env.as_contract(&reward_manager_id, || {
        assert_eq!(RewardManager::get_pool_balance(env.clone(), hunt_id), 0);
    });

    let token_client = token::Client::new(&env, &token_address);
    assert_eq!(token_client.balance(&creator), 5_000);
    assert_eq!(token_client.balance(&reward_manager_id), 0);
}
