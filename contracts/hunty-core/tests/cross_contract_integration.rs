/// Three-Contract Integration Tests
/// Tests the interaction between HuntyCore, RewardManager, and NftReward
///
/// Acceptance Criteria:
/// - HuntyCore calls RewardManager.distribute
/// - RewardManager calls NftReward.mint
/// - Verify state consistency across contracts
/// - Test error propagation between contracts

use hunty_core::HuntyCore;
use nft_reward::NftReward;
use reward_manager::RewardManager;
use soroban_sdk::testutils::{Address as _, Ledger as _};
use soroban_sdk::{token, Address, Env, String};

fn setup_environment(env: &Env) -> (Address, Address, Address, Address) {
    let core_id = env.register(HuntyCore, ());
    let reward_manager_id = env.register(RewardManager, ());
    let nft_reward_id = env.register(NftReward, ());
    let token_admin = Address::generate(env);
    
    let token_contract = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_address = token_contract.address();

    // Initialize RewardManager
    env.as_contract(&reward_manager_id, || {
        RewardManager::initialize(env.clone(), token_admin.clone(), token_address.clone()).unwrap();
        RewardManager::set_nft_contract(env.clone(), nft_reward_id.clone()).unwrap();
    });

    // Initialize NftReward with RewardManager as authorized minter
    env.as_contract(&nft_reward_id, || {
        NftReward::initialize_admin(env.clone(), Address::generate(env)).unwrap();
        NftReward::set_reward_manager(env.clone(), reward_manager_id.clone()).unwrap();
    });

    (core_id, reward_manager_id, nft_reward_id, token_address)
}

fn as_core_contract<T>(env: &Env, contract_id: &Address, f: impl FnOnce(&Env) -> T) -> T {
    env.as_contract(contract_id, || f(env))
}

// ============================================================================
// Tests for HuntyCore → RewardManager → NftReward Interaction
// ============================================================================

#[test]
fn test_hunty_core_calls_reward_manager_for_xlm_distribution() {
    let env = Env::default();
    env.ledger().set_timestamp(1_700_000_000);
    env.mock_all_auths();

    let creator = Address::generate(&env);
    let player = Address::generate(&env);
    
    let (core_id, reward_manager_id, _nft_reward_id, token_address) = setup_environment(&env);

    // Setup token and fund reward manager
    let sac = token::StellarAssetClient::new(&env, &token_address);
    sac.mint(&reward_manager_id, &50_000);

    // Create and setup hunt
    let hunt_id = as_core_contract(&env, &core_id, |env| {
        let hunt_id = HuntyCore::create_hunt(
            env.clone(),
            creator.clone(),
            String::from_str(env, "XLM Reward Hunt"),
            String::from_str(env, "Testing XLM distribution"),
            None,
            None,
            0,
            None,
        )
        .unwrap();

        // Add required clue
        HuntyCore::add_clue(
            env.clone(),
            hunt_id,
            String::from_str(env, "Question 1"),
            String::from_str(env, "Answer 1"),
            10,
            true,
        )
        .unwrap();

        // Setup reward config (XLM only)
        HuntyCore::set_reward_config(
            env.clone(),
            hunt_id,
            1000,  // max_winners
            1000,  // xlm_pool
            false, // nft_enabled
            None,
        )
        .unwrap();

        // Activate hunt
        HuntyCore::activate_hunt(env.clone(), hunt_id, creator.clone()).unwrap();

        // Set reward manager
        HuntyCore::set_reward_manager(env.clone(), reward_manager_id.clone());

        hunt_id
    });

    // Setup reward pool
    env.as_contract(&reward_manager_id, || {
        RewardManager::create_reward_pool(env.clone(), creator.clone(), hunt_id, 0).unwrap();
        RewardManager::fund_reward_pool(env.clone(), creator.clone(), hunt_id, 10_000).unwrap();
    });

    // Register player and submit answers
    as_core_contract(&env, &core_id, |env| {
        HuntyCore::register_player(env.clone(), hunt_id, player.clone()).unwrap();
        HuntyCore::submit_answer(
            env.clone(),
            hunt_id,
            player.clone(),
            0,
            String::from_str(env, "Answer 1"),
        )
        .unwrap();
    });

    // Get player balance before completion
    let token_client = token::Client::new(&env, &token_address);
    let player_balance_before = token_client.balance(&player);

    // Complete hunt and claim rewards
    let result = as_core_contract(&env, &core_id, |env| {
        HuntyCore::complete_hunt(env.clone(), hunt_id, player.clone())
    });

    // Verify completion was successful
    assert!(result.is_ok(), "Hunt completion should succeed");

    // Verify player received XLM
    let player_balance_after = token_client.balance(&player);
    let xlm_per_winner = 10_000 / 1000;
    assert_eq!(
        player_balance_after - player_balance_before,
        xlm_per_winner,
        "Player should receive XLM reward"
    );

    // Verify reward pool was decremented
    env.as_contract(&reward_manager_id, || {
        let pool_balance = RewardManager::get_pool_balance(env.clone(), hunt_id);
        assert_eq!(
            pool_balance,
            10_000 - xlm_per_winner,
            "Pool balance should be decremented"
        );
    });
}

#[test]
fn test_reward_manager_calls_nft_reward_for_minting() {
    let env = Env::default();
    env.ledger().set_timestamp(1_700_000_000);
    env.mock_all_auths();

    let creator = Address::generate(&env);
    let player = Address::generate(&env);
    
    let (core_id, reward_manager_id, nft_reward_id, token_address) = setup_environment(&env);

    // Setup token
    let sac = token::StellarAssetClient::new(&env, &token_address);
    sac.mint(&reward_manager_id, &50_000);

    // Create hunt with NFT rewards
    let hunt_id = as_core_contract(&env, &core_id, |env| {
        let hunt_id = HuntyCore::create_hunt(
            env.clone(),
            creator.clone(),
            String::from_str(env, "NFT Reward Hunt"),
            String::from_str(env, "Testing NFT minting"),
            None,
            None,
            0,
            None,
        )
        .unwrap();

        HuntyCore::add_clue(
            env.clone(),
            hunt_id,
            String::from_str(env, "Question"),
            String::from_str(env, "Answer"),
            10,
            true,
        )
        .unwrap();

        // Setup reward config with NFT
        HuntyCore::set_reward_config(
            env.clone(),
            hunt_id,
            100,
            1000,
            true,  // nft_enabled
            Some(nft_reward_id.clone()),
        )
        .unwrap();

        HuntyCore::activate_hunt(env.clone(), hunt_id, creator.clone()).unwrap();
        HuntyCore::set_reward_manager(env.clone(), reward_manager_id.clone());

        hunt_id
    });

    // Register player and submit answers
    as_core_contract(&env, &core_id, |env| {
        HuntyCore::register_player(env.clone(), hunt_id, player.clone()).unwrap();
        HuntyCore::submit_answer(
            env.clone(),
            hunt_id,
            player.clone(),
            0,
            String::from_str(env, "Answer"),
        )
        .unwrap();
    });

    // Complete hunt - should trigger NFT minting
    let result = as_core_contract(&env, &core_id, |env| {
        HuntyCore::complete_hunt(env.clone(), hunt_id, player.clone())
    });

    assert!(result.is_ok(), "Hunt completion should succeed and mint NFT");

    // Verify NFT was minted
    env.as_contract(&nft_reward_id, || {
        let nft_count = NftReward::get_total_supply(env.clone());
        assert_eq!(nft_count, 1, "One NFT should be minted");

        // Verify NFT ownership
        let nft_metadata = NftReward::get_nft_metadata(env.clone(), 0).unwrap();
        assert_eq!(
            nft_metadata.hunt_id, hunt_id,
            "NFT should be associated with correct hunt"
        );
        assert_eq!(
            nft_metadata.completion_player, player,
            "NFT should be owned by player"
        );
    });
}

#[test]
fn test_xlm_and_nft_reward_distribution_combined() {
    let env = Env::default();
    env.ledger().set_timestamp(1_700_000_000);
    env.mock_all_auths();

    let creator = Address::generate(&env);
    let player = Address::generate(&env);
    
    let (core_id, reward_manager_id, nft_reward_id, token_address) = setup_environment(&env);

    // Setup token
    let sac = token::StellarAssetClient::new(&env, &token_address);
    sac.mint(&reward_manager_id, &50_000);

    // Create hunt with both XLM and NFT rewards
    let hunt_id = as_core_contract(&env, &core_id, |env| {
        let hunt_id = HuntyCore::create_hunt(
            env.clone(),
            creator.clone(),
            String::from_str(env, "Combined Reward Hunt"),
            String::from_str(env, "Testing XLM + NFT"),
            None,
            None,
            0,
            None,
        )
        .unwrap();

        HuntyCore::add_clue(
            env.clone(),
            hunt_id,
            String::from_str(env, "Q"),
            String::from_str(env, "A"),
            10,
            true,
        )
        .unwrap();

        // Setup rewards: 100 winners, 5000 XLM pool, NFT enabled
        HuntyCore::set_reward_config(
            env.clone(),
            hunt_id,
            100,
            5000,
            true,
            Some(nft_reward_id.clone()),
        )
        .unwrap();

        HuntyCore::activate_hunt(env.clone(), hunt_id, creator.clone()).unwrap();
        HuntyCore::set_reward_manager(env.clone(), reward_manager_id.clone());

        hunt_id
    });

    // Setup reward pool
    env.as_contract(&reward_manager_id, || {
        RewardManager::create_reward_pool(env.clone(), creator.clone(), hunt_id, 0).unwrap();
        RewardManager::fund_reward_pool(env.clone(), creator.clone(), hunt_id, 10_000).unwrap();
    });

    // Register and complete hunt
    as_core_contract(&env, &core_id, |env| {
        HuntyCore::register_player(env.clone(), hunt_id, player.clone()).unwrap();
        HuntyCore::submit_answer(
            env.clone(),
            hunt_id,
            player.clone(),
            0,
            String::from_str(env, "A"),
        )
        .unwrap();
    });

    let token_client = token::Client::new(&env, &token_address);
    let player_balance_before = token_client.balance(&player);

    // Complete hunt
    as_core_contract(&env, &core_id, |env| {
        HuntyCore::complete_hunt(env.clone(), hunt_id, player.clone()).unwrap()
    });

    // Verify XLM received
    let player_balance_after = token_client.balance(&player);
    let xlm_per_winner = 5000 / 100;
    assert_eq!(
        player_balance_after - player_balance_before,
        xlm_per_winner,
        "Player should receive XLM"
    );

    // Verify NFT minted
    env.as_contract(&nft_reward_id, || {
        let supply = NftReward::get_total_supply(env.clone());
        assert_eq!(supply, 1, "NFT should be minted");
    });
}

#[test]
fn test_state_consistency_across_contracts_after_distribution() {
    let env = Env::default();
    env.ledger().set_timestamp(1_700_000_000);
    env.mock_all_auths();

    let creator = Address::generate(&env);
    let player = Address::generate(&env);
    
    let (core_id, reward_manager_id, nft_reward_id, token_address) = setup_environment(&env);

    let sac = token::StellarAssetClient::new(&env, &token_address);
    sac.mint(&reward_manager_id, &50_000);

    let hunt_id = as_core_contract(&env, &core_id, |env| {
        let hunt_id = HuntyCore::create_hunt(
            env.clone(),
            creator.clone(),
            String::from_str(env, "Consistency Test Hunt"),
            String::from_str(env, "Testing state consistency"),
            None,
            None,
            0,
            None,
        )
        .unwrap();

        HuntyCore::add_clue(
            env.clone(),
            hunt_id,
            String::from_str(env, "Q"),
            String::from_str(env, "A"),
            10,
            true,
        )
        .unwrap();

        HuntyCore::set_reward_config(
            env.clone(),
            hunt_id,
            50,
            2500,
            true,
            Some(nft_reward_id.clone()),
        )
        .unwrap();

        HuntyCore::activate_hunt(env.clone(), hunt_id, creator.clone()).unwrap();
        HuntyCore::set_reward_manager(env.clone(), reward_manager_id.clone());

        hunt_id
    });

    env.as_contract(&reward_manager_id, || {
        RewardManager::create_reward_pool(env.clone(), creator.clone(), hunt_id, 0).unwrap();
        RewardManager::fund_reward_pool(env.clone(), creator.clone(), hunt_id, 5000).unwrap();
    });

    // Before completion
    let hunt_before = as_core_contract(&env, &core_id, |env| {
        HuntyCore::get_hunt(env.clone(), hunt_id).unwrap()
    });
    assert_eq!(hunt_before.completed_count, 0, "No completions yet");

    let pool_balance_before = env.as_contract(&reward_manager_id, |env| {
        RewardManager::get_pool_balance(env.clone(), hunt_id)
    });
    assert_eq!(pool_balance_before, 5000, "Initial pool balance");

    // Register, answer, and complete
    as_core_contract(&env, &core_id, |env| {
        HuntyCore::register_player(env.clone(), hunt_id, player.clone()).unwrap();
        HuntyCore::submit_answer(
            env.clone(),
            hunt_id,
            player.clone(),
            0,
            String::from_str(env, "A"),
        )
        .unwrap();
        HuntyCore::complete_hunt(env.clone(), hunt_id, player.clone()).unwrap();
    });

    // After completion - verify HuntyCore state
    let hunt_after = as_core_contract(&env, &core_id, |env| {
        HuntyCore::get_hunt(env.clone(), hunt_id).unwrap()
    });
    assert_eq!(
        hunt_after.completed_count, 1,
        "Completion count should increment"
    );
    assert_eq!(
        hunt_after.reward_config.claimed_count, 1,
        "Claimed count should increment"
    );

    // Verify RewardManager state
    let pool_balance_after = env.as_contract(&reward_manager_id, |env| {
        RewardManager::get_pool_balance(env.clone(), hunt_id)
    });
    assert_eq!(
        pool_balance_after,
        pool_balance_before - (2500 / 50),
        "Pool balance should be decremented by reward amount"
    );

    // Verify NftReward state
    env.as_contract(&nft_reward_id, || {
        let supply = NftReward::get_total_supply(env.clone());
        assert_eq!(supply, 1, "One NFT should exist");

        let metadata = NftReward::get_nft_metadata(env.clone(), 0).unwrap();
        assert_eq!(metadata.hunt_id, hunt_id);
        assert_eq!(metadata.completion_player, player);
    });
}

#[test]
fn test_error_propagation_insufficient_pool_balance() {
    let env = Env::default();
    env.ledger().set_timestamp(1_700_000_000);
    env.mock_all_auths();

    let creator = Address::generate(&env);
    let player = Address::generate(&env);
    
    let (core_id, reward_manager_id, _nft_reward_id, token_address) = setup_environment(&env);

    let sac = token::StellarAssetClient::new(&env, &token_address);
    // Fund with insufficient amount
    sac.mint(&reward_manager_id, &100);

    let hunt_id = as_core_contract(&env, &core_id, |env| {
        let hunt_id = HuntyCore::create_hunt(
            env.clone(),
            creator.clone(),
            String::from_str(env, "Insufficient Pool Hunt"),
            String::from_str(env, "Testing error propagation"),
            None,
            None,
            0,
            None,
        )
        .unwrap();

        HuntyCore::add_clue(
            env.clone(),
            hunt_id,
            String::from_str(env, "Q"),
            String::from_str(env, "A"),
            10,
            true,
        )
        .unwrap();

        // Request 5000 XLM reward pool
        HuntyCore::set_reward_config(
            env.clone(),
            hunt_id,
            10,
            5000,
            false,
            None,
        )
        .unwrap();

        HuntyCore::activate_hunt(env.clone(), hunt_id, creator.clone()).unwrap();
        HuntyCore::set_reward_manager(env.clone(), reward_manager_id.clone());

        hunt_id
    });

    // Fund pool with only 50 (less than 5000 requested)
    env.as_contract(&reward_manager_id, || {
        RewardManager::create_reward_pool(env.clone(), creator.clone(), hunt_id, 0).unwrap();
        RewardManager::fund_reward_pool(env.clone(), creator.clone(), hunt_id, 50).unwrap();
    });

    as_core_contract(&env, &core_id, |env| {
        HuntyCore::register_player(env.clone(), hunt_id, player.clone()).unwrap();
        HuntyCore::submit_answer(
            env.clone(),
            hunt_id,
            player.clone(),
            0,
            String::from_str(env, "A"),
        )
        .unwrap();
    });

    // Attempt to complete hunt - should fail due to insufficient pool
    let result = as_core_contract(&env, &core_id, |env| {
        HuntyCore::complete_hunt(env.clone(), hunt_id, player.clone())
    });

    // Verify error was propagated
    assert!(
        result.is_err(),
        "Completion should fail when pool balance is insufficient"
    );
}

#[test]
fn test_error_propagation_invalid_nft_config() {
    let env = Env::default();
    env.ledger().set_timestamp(1_700_000_000);
    env.mock_all_auths();

    let creator = Address::generate(&env);
    let player = Address::generate(&env);
    
    let (core_id, reward_manager_id, _nft_reward_id, token_address) = setup_environment(&env);

    let sac = token::StellarAssetClient::new(&env, &token_address);
    sac.mint(&reward_manager_id, &50_000);

    // Try to setup hunt with NFT but no NFT contract
    let hunt_id = as_core_contract(&env, &core_id, |env| {
        let hunt_id = HuntyCore::create_hunt(
            env.clone(),
            creator.clone(),
            String::from_str(env, "Invalid NFT Hunt"),
            String::from_str(env, "Testing invalid NFT config"),
            None,
            None,
            0,
            None,
        )
        .unwrap();

        HuntyCore::add_clue(
            env.clone(),
            hunt_id,
            String::from_str(env, "Q"),
            String::from_str(env, "A"),
            10,
            true,
        )
        .unwrap();

        // Enable NFT but don't provide contract address
        HuntyCore::set_reward_config(
            env.clone(),
            hunt_id,
            10,
            1000,
            true,
            None,  // No NFT contract provided
        )
        .unwrap();

        HuntyCore::activate_hunt(env.clone(), hunt_id, creator.clone()).unwrap();
        HuntyCore::set_reward_manager(env.clone(), reward_manager_id.clone());

        hunt_id
    });

    env.as_contract(&reward_manager_id, || {
        RewardManager::create_reward_pool(env.clone(), creator.clone(), hunt_id, 0).unwrap();
        RewardManager::fund_reward_pool(env.clone(), creator.clone(), hunt_id, 5000).unwrap();
    });

    as_core_contract(&env, &core_id, |env| {
        HuntyCore::register_player(env.clone(), hunt_id, player.clone()).unwrap();
        HuntyCore::submit_answer(
            env.clone(),
            hunt_id,
            player.clone(),
            0,
            String::from_str(env, "A"),
        )
        .unwrap();
    });

    // Attempt to complete hunt - should fail
    let result = as_core_contract(&env, &core_id, |env| {
        HuntyCore::complete_hunt(env.clone(), hunt_id, player.clone())
    });

    assert!(
        result.is_err(),
        "Completion should fail with invalid NFT configuration"
    );
}

#[test]
fn test_reward_already_claimed_prevents_double_distribution() {
    let env = Env::default();
    env.ledger().set_timestamp(1_700_000_000);
    env.mock_all_auths();

    let creator = Address::generate(&env);
    let player = Address::generate(&env);
    
    let (core_id, reward_manager_id, nft_reward_id, token_address) = setup_environment(&env);

    let sac = token::StellarAssetClient::new(&env, &token_address);
    sac.mint(&reward_manager_id, &50_000);

    let hunt_id = as_core_contract(&env, &core_id, |env| {
        let hunt_id = HuntyCore::create_hunt(
            env.clone(),
            creator.clone(),
            String::from_str(env, "Double Claim Hunt"),
            String::from_str(env, "Testing double claim prevention"),
            None,
            None,
            0,
            None,
        )
        .unwrap();

        HuntyCore::add_clue(
            env.clone(),
            hunt_id,
            String::from_str(env, "Q"),
            String::from_str(env, "A"),
            10,
            true,
        )
        .unwrap();

        HuntyCore::set_reward_config(
            env.clone(),
            hunt_id,
            10,
            1000,
            true,
            Some(nft_reward_id.clone()),
        )
        .unwrap();

        HuntyCore::activate_hunt(env.clone(), hunt_id, creator.clone()).unwrap();
        HuntyCore::set_reward_manager(env.clone(), reward_manager_id.clone());

        hunt_id
    });

    env.as_contract(&reward_manager_id, || {
        RewardManager::create_reward_pool(env.clone(), creator.clone(), hunt_id, 0).unwrap();
        RewardManager::fund_reward_pool(env.clone(), creator.clone(), hunt_id, 5000).unwrap();
    });

    as_core_contract(&env, &core_id, |env| {
        HuntyCore::register_player(env.clone(), hunt_id, player.clone()).unwrap();
        HuntyCore::submit_answer(
            env.clone(),
            hunt_id,
            player.clone(),
            0,
            String::from_str(env, "A"),
        )
        .unwrap();
    });

    // First completion - should succeed
    let result1 = as_core_contract(&env, &core_id, |env| {
        HuntyCore::complete_hunt(env.clone(), hunt_id, player.clone())
    });
    assert!(result1.is_ok(), "First completion should succeed");

    // Second completion attempt - should fail
    let result2 = as_core_contract(&env, &core_id, |env| {
        HuntyCore::complete_hunt(env.clone(), hunt_id, player.clone())
    });
    assert!(
        result2.is_err(),
        "Second completion should fail (reward already claimed)"
    );

    // Verify only one NFT was minted
    env.as_contract(&nft_reward_id, || {
        let supply = NftReward::get_total_supply(env.clone());
        assert_eq!(supply, 1, "Only one NFT should exist");
    });
}

#[test]
fn test_multiple_players_rewards_consistency() {
    let env = Env::default();
    env.ledger().set_timestamp(1_700_000_000);
    env.mock_all_auths();

    let creator = Address::generate(&env);
    let player1 = Address::generate(&env);
    let player2 = Address::generate(&env);
    let player3 = Address::generate(&env);
    
    let (core_id, reward_manager_id, nft_reward_id, token_address) = setup_environment(&env);

    let sac = token::StellarAssetClient::new(&env, &token_address);
    sac.mint(&reward_manager_id, &100_000);

    let hunt_id = as_core_contract(&env, &core_id, |env| {
        let hunt_id = HuntyCore::create_hunt(
            env.clone(),
            creator.clone(),
            String::from_str(env, "Multi-Player Hunt"),
            String::from_str(env, "Testing multiple player rewards"),
            None,
            None,
            0,
            None,
        )
        .unwrap();

        HuntyCore::add_clue(
            env.clone(),
            hunt_id,
            String::from_str(env, "Q"),
            String::from_str(env, "A"),
            10,
            true,
        )
        .unwrap();

        HuntyCore::set_reward_config(
            env.clone(),
            hunt_id,
            3,
            3000,
            true,
            Some(nft_reward_id.clone()),
        )
        .unwrap();

        HuntyCore::activate_hunt(env.clone(), hunt_id, creator.clone()).unwrap();
        HuntyCore::set_reward_manager(env.clone(), reward_manager_id.clone());

        hunt_id
    });

    env.as_contract(&reward_manager_id, || {
        RewardManager::create_reward_pool(env.clone(), creator.clone(), hunt_id, 0).unwrap();
        RewardManager::fund_reward_pool(env.clone(), creator.clone(), hunt_id, 10_000).unwrap();
    });

    // Register all 3 players
    as_core_contract(&env, &core_id, |env| {
        HuntyCore::register_player(env.clone(), hunt_id, player1.clone()).unwrap();
        HuntyCore::register_player(env.clone(), hunt_id, player2.clone()).unwrap();
        HuntyCore::register_player(env.clone(), hunt_id, player3.clone()).unwrap();

        // All answer correctly
        HuntyCore::submit_answer(
            env.clone(),
            hunt_id,
            player1.clone(),
            0,
            String::from_str(env, "A"),
        )
        .unwrap();
        HuntyCore::submit_answer(
            env.clone(),
            hunt_id,
            player2.clone(),
            0,
            String::from_str(env, "A"),
        )
        .unwrap();
        HuntyCore::submit_answer(
            env.clone(),
            hunt_id,
            player3.clone(),
            0,
            String::from_str(env, "A"),
        )
        .unwrap();
    });

    let token_client = token::Client::new(&env, &token_address);
    let rewards_per_player = 3000 / 3;

    // All 3 players complete hunt
    as_core_contract(&env, &core_id, |env| {
        HuntyCore::complete_hunt(env.clone(), hunt_id, player1.clone()).unwrap();
        HuntyCore::complete_hunt(env.clone(), hunt_id, player2.clone()).unwrap();
        HuntyCore::complete_hunt(env.clone(), hunt_id, player3.clone()).unwrap();
    });

    // Verify all players received rewards
    let balance1 = token_client.balance(&player1);
    let balance2 = token_client.balance(&player2);
    let balance3 = token_client.balance(&player3);

    assert_eq!(balance1, rewards_per_player, "Player 1 should receive reward");
    assert_eq!(balance2, rewards_per_player, "Player 2 should receive reward");
    assert_eq!(balance3, rewards_per_player, "Player 3 should receive reward");

    // Verify pool is depleted
    let pool_balance = env.as_contract(&reward_manager_id, |env| {
        RewardManager::get_pool_balance(env.clone(), hunt_id)
    });
    assert_eq!(
        pool_balance,
        10_000 - (rewards_per_player * 3),
        "Pool should be depleted correctly"
    );

    // Verify 3 NFTs were minted
    env.as_contract(&nft_reward_id, || {
        let supply = NftReward::get_total_supply(env.clone());
        assert_eq!(supply, 3, "Three NFTs should be minted");
    });
}

#[test]
fn test_cross_contract_call_failure_recovery() {
    let env = Env::default();
    env.ledger().set_timestamp(1_700_000_000);
    env.mock_all_auths();

    let creator = Address::generate(&env);
    let player = Address::generate(&env);
    
    // Setup without NFT reward to cause failure
    let core_id = env.register(HuntyCore, ());
    let reward_manager_id = env.register(RewardManager, ());
    let token_admin = Address::generate(&env);
    
    let token_contract = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_address = token_contract.address();

    // Initialize RewardManager without setting NFT contract
    env.as_contract(&reward_manager_id, || {
        RewardManager::initialize(env.clone(), token_admin.clone(), token_address.clone()).unwrap();
        // Intentionally NOT setting NFT contract
    });

    let sac = token::StellarAssetClient::new(&env, &token_address);
    sac.mint(&reward_manager_id, &50_000);

    // Create hunt expecting NFT rewards but manager has no NFT contract
    let hunt_id = as_core_contract(&env, &core_id, |env| {
        let hunt_id = HuntyCore::create_hunt(
            env.clone(),
            creator.clone(),
            String::from_str(env, "Failure Recovery Hunt"),
            String::from_str(env, "Testing failure recovery"),
            None,
            None,
            0,
            None,
        )
        .unwrap();

        HuntyCore::add_clue(
            env.clone(),
            hunt_id,
            String::from_str(env, "Q"),
            String::from_str(env, "A"),
            10,
            true,
        )
        .unwrap();

        // XLM only rewards (will work)
        HuntyCore::set_reward_config(
            env.clone(),
            hunt_id,
            10,
            1000,
            false,  // NFT disabled
            None,
        )
        .unwrap();

        HuntyCore::activate_hunt(env.clone(), hunt_id, creator.clone()).unwrap();
        HuntyCore::set_reward_manager(env.clone(), reward_manager_id.clone());

        hunt_id
    });

    env.as_contract(&reward_manager_id, || {
        RewardManager::create_reward_pool(env.clone(), creator.clone(), hunt_id, 0).unwrap();
        RewardManager::fund_reward_pool(env.clone(), creator.clone(), hunt_id, 5000).unwrap();
    });

    as_core_contract(&env, &core_id, |env| {
        HuntyCore::register_player(env.clone(), hunt_id, player.clone()).unwrap();
        HuntyCore::submit_answer(
            env.clone(),
            hunt_id,
            player.clone(),
            0,
            String::from_str(env, "A"),
        )
        .unwrap();
    });

    // Should succeed with XLM-only rewards
    let result = as_core_contract(&env, &core_id, |env| {
        HuntyCore::complete_hunt(env.clone(), hunt_id, player.clone())
    });

    assert!(result.is_ok(), "XLM-only distribution should succeed");

    let token_client = token::Client::new(&env, &token_address);
    let player_balance = token_client.balance(&player);
    assert!(player_balance > 0, "Player should receive XLM reward");
}
