/// Three-Contract Integration Tests
/// Tests the interaction between HuntyCore, RewardManager, and NftReward
///
/// Acceptance Criteria:
/// - HuntyCore calls RewardManager.distribute
/// - RewardManager calls NftReward.mint
/// - Verify state consistency across contracts
/// - Test error propagation between contracts
use hunty_core::{HuntyCore, HuntyCoreClient};
use nft_reward::{CollectionMetadata, NftReward, NftRewardClient};
use reward_manager::{RewardManager, RewardManagerClient};
use soroban_sdk::testutils::{Address as _, Ledger as _};
use soroban_sdk::{token, Address, Env, String};

struct TestContext<'a> {
    core_id: Address,
    reward_manager_id: Address,
    nft_reward_id: Address,
    token_address: Address,
    admin: Address,
    core_client: HuntyCoreClient<'a>,
    reward_client: RewardManagerClient<'a>,
    nft_client: NftRewardClient<'a>,
}

fn setup_environment<'a>(env: &'a Env) -> TestContext<'a> {
    let core_id = env.register(HuntyCore, ());
    let reward_manager_id = env.register(RewardManager, ());
    let nft_reward_id = env.register(NftReward, ());
    let token_admin = Address::generate(env);
    let admin = Address::generate(env);

    let token_contract = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_address = token_contract.address();

    let core_client = HuntyCoreClient::new(env, &core_id);
    let reward_client = RewardManagerClient::new(env, &reward_manager_id);
    let nft_client = NftRewardClient::new(env, &nft_reward_id);

    // Initialize RewardManager
    reward_client.initialize(&admin, &token_address, &core_id);
    reward_client.set_nft_reward_contract(&admin, &nft_reward_id);

    // Initialize NftReward with RewardManager as authorized minter
    nft_client.initialize(
        &admin,
        &reward_manager_id,
        &None,
        &CollectionMetadata {
            name: String::from_str(env, "Hunty Rewards"),
            description: String::from_str(env, "Reward NFTs for completed hunts"),
            total_supply: 0,
            creator: None,
        },
    );

    // Initialize HuntyCore admin
    core_client.initialize_admin(&admin);

    TestContext {
        core_id,
        reward_manager_id,
        nft_reward_id,
        token_address,
        admin,
        core_client,
        reward_client,
        nft_client,
    }
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

    let ctx = setup_environment(&env);

    // Setup token and mint to creator
    let sac = token::StellarAssetClient::new(&env, &ctx.token_address);
    sac.mint(&creator, &50_000_000);

    // Create and setup hunt
    let hunt_id = ctx.core_client.create_hunt(
        &creator,
        &String::from_str(&env, "XLM Reward Hunt"),
        &String::from_str(&env, "Testing XLM distribution"),
        &None,
        &None,
        &0u32,
        &None,
        &None,
    );

    let clue_id = ctx.core_client.add_clue(
        &hunt_id,
        &String::from_str(&env, "Question 1"),
        &String::from_str(&env, "Answer 1"),
        &10u32,
        &true,
        &None,
        &None,
    );

    ctx.core_client.set_reward_config(
        &hunt_id,
        &1000u32,        // max_winners
        &10_000_000i128, // xlm_pool
        &false,          // nft_enabled
        &None,
    );

    ctx.core_client.activate_hunt(&hunt_id, &creator);
    ctx.core_client
        .set_reward_manager(&ctx.admin, &ctx.reward_manager_id);

    // Setup reward pool with min_distribution_amount > 0
    ctx.reward_client.create_reward_pool(
        &creator,
        &hunt_id,
        &ctx.token_address,
        &1i128,
        &0u32,
        &false,
    );
    ctx.reward_client
        .fund_reward_pool(&creator, &hunt_id, &10_000_000i128);

    // Register player and submit answer
    ctx.core_client.register_player(&hunt_id, &player);
    ctx.core_client.submit_answer(
        &hunt_id,
        &clue_id,
        &player,
        &String::from_str(&env, "Answer 1"),
        &1u64,
        &1_700_000_000u64,
    );

    // Get player balance before completion
    let token_client = token::Client::new(&env, &ctx.token_address);
    let player_balance_before = token_client.balance(&player);

    // Complete hunt and claim rewards
    ctx.core_client.complete_hunt(&hunt_id, &player);

    // Verify player received XLM
    let player_balance_after = token_client.balance(&player);
    let xlm_per_winner = 10_000_000 / 1000;
    assert_eq!(
        player_balance_after - player_balance_before,
        xlm_per_winner,
        "Player should receive XLM reward"
    );

    // Verify reward pool was decremented
    let pool_balance = ctx.reward_client.get_pool_balance(&hunt_id);
    assert_eq!(
        pool_balance,
        10_000_000 - xlm_per_winner,
        "Pool balance should be decremented"
    );
}

#[test]
fn test_reward_manager_calls_nft_reward_for_minting() {
    let env = Env::default();
    env.ledger().set_timestamp(1_700_000_000);
    env.mock_all_auths();

    let creator = Address::generate(&env);
    let player = Address::generate(&env);

    let ctx = setup_environment(&env);

    let sac = token::StellarAssetClient::new(&env, &ctx.token_address);
    sac.mint(&creator, &50_000_000);

    let hunt_id = ctx.core_client.create_hunt(
        &creator,
        &String::from_str(&env, "NFT Reward Hunt"),
        &String::from_str(&env, "Testing NFT minting"),
        &None,
        &None,
        &0u32,
        &None,
        &None,
    );

    let clue_id = ctx.core_client.add_clue(
        &hunt_id,
        &String::from_str(&env, "Question"),
        &String::from_str(&env, "Answer"),
        &10u32,
        &true,
        &None,
        &None,
    );

    ctx.core_client.set_reward_config(
        &hunt_id,
        &100u32,
        &10_000_000i128,
        &true,
        &Some(ctx.nft_reward_id.clone()),
    );

    ctx.core_client.activate_hunt(&hunt_id, &creator);
    ctx.core_client
        .set_reward_manager(&ctx.admin, &ctx.reward_manager_id);

    ctx.reward_client.create_reward_pool_with_nft(
        &creator,
        &hunt_id,
        &ctx.token_address,
        &0i128,
        &Some(ctx.nft_reward_id.clone()),
        &0u32,
        &true,
    );
    ctx.reward_client
        .fund_reward_pool(&creator, &hunt_id, &10_000_000i128);

    ctx.core_client.register_player(&hunt_id, &player);
    ctx.core_client.submit_answer(
        &hunt_id,
        &clue_id,
        &player,
        &String::from_str(&env, "Answer"),
        &1u64,
        &1_700_000_000u64,
    );

    ctx.core_client.complete_hunt(&hunt_id, &player);

    let nft_count = ctx.nft_client.total_supply();
    assert_eq!(nft_count, 1, "One NFT should be minted");

    let nft_metadata = ctx.nft_client.get_nft_metadata(&1).unwrap();
    assert_eq!(nft_metadata.hunt_id, hunt_id);
    assert_eq!(nft_metadata.current_owner, player);
}

#[test]
fn test_xlm_and_nft_reward_distribution_combined() {
    let env = Env::default();
    env.ledger().set_timestamp(1_700_000_000);
    env.mock_all_auths();

    let creator = Address::generate(&env);
    let player = Address::generate(&env);

    let ctx = setup_environment(&env);

    let sac = token::StellarAssetClient::new(&env, &ctx.token_address);
    sac.mint(&creator, &50_000_000);

    let hunt_id = ctx.core_client.create_hunt(
        &creator,
        &String::from_str(&env, "Combined Reward Hunt"),
        &String::from_str(&env, "Testing XLM + NFT"),
        &None,
        &None,
        &0u32,
        &None,
        &None,
    );

    let clue_id = ctx.core_client.add_clue(
        &hunt_id,
        &String::from_str(&env, "Q"),
        &String::from_str(&env, "A"),
        &10u32,
        &true,
        &None,
        &None,
    );

    ctx.core_client.set_reward_config(
        &hunt_id,
        &100u32,
        &10_000_000i128,
        &true,
        &Some(ctx.nft_reward_id.clone()),
    );

    ctx.core_client.activate_hunt(&hunt_id, &creator);
    ctx.core_client
        .set_reward_manager(&ctx.admin, &ctx.reward_manager_id);

    ctx.reward_client.create_reward_pool_with_nft(
        &creator,
        &hunt_id,
        &ctx.token_address,
        &0i128,
        &Some(ctx.nft_reward_id.clone()),
        &0u32,
        &true,
    );
    ctx.reward_client
        .fund_reward_pool(&creator, &hunt_id, &10_000_000i128);

    ctx.core_client.register_player(&hunt_id, &player);
    ctx.core_client.submit_answer(
        &hunt_id,
        &clue_id,
        &player,
        &String::from_str(&env, "A"),
        &1u64,
        &1_700_000_000u64,
    );

    let token_client = token::Client::new(&env, &ctx.token_address);
    let player_balance_before = token_client.balance(&player);

    ctx.core_client.complete_hunt(&hunt_id, &player);

    let player_balance_after = token_client.balance(&player);
    let xlm_per_winner = 10_000_000 / 100;
    assert_eq!(
        player_balance_after - player_balance_before,
        xlm_per_winner,
        "Player should receive XLM"
    );

    let supply = ctx.nft_client.total_supply();
    assert_eq!(supply, 1, "NFT should be minted");
}

#[test]
fn test_state_consistency_across_contracts_after_distribution() {
    let env = Env::default();
    env.ledger().set_timestamp(1_700_000_000);
    env.mock_all_auths();

    let creator = Address::generate(&env);
    let player = Address::generate(&env);

    let ctx = setup_environment(&env);

    let sac = token::StellarAssetClient::new(&env, &ctx.token_address);
    sac.mint(&creator, &50_000_000);

    let hunt_id = ctx.core_client.create_hunt(
        &creator,
        &String::from_str(&env, "Consistency Test Hunt"),
        &String::from_str(&env, "Testing state consistency"),
        &None,
        &None,
        &0u32,
        &None,
        &None,
    );

    let clue_id = ctx.core_client.add_clue(
        &hunt_id,
        &String::from_str(&env, "Q"),
        &String::from_str(&env, "A"),
        &10u32,
        &true,
        &None,
        &None,
    );

    ctx.core_client.set_reward_config(
        &hunt_id,
        &50u32,
        &10_000_000i128,
        &true,
        &Some(ctx.nft_reward_id.clone()),
    );

    ctx.core_client.activate_hunt(&hunt_id, &creator);
    ctx.core_client
        .set_reward_manager(&ctx.admin, &ctx.reward_manager_id);

    ctx.reward_client.create_reward_pool_with_nft(
        &creator,
        &hunt_id,
        &ctx.token_address,
        &0i128,
        &Some(ctx.nft_reward_id.clone()),
        &0u32,
        &true,
    );
    ctx.reward_client
        .fund_reward_pool(&creator, &hunt_id, &10_000_000i128);

    let hunt_before = ctx.core_client.get_hunt_info(&hunt_id);
    assert_eq!(hunt_before.completed_count, 0, "No completions yet");

    let pool_balance_before = ctx.reward_client.get_pool_balance(&hunt_id);
    assert_eq!(pool_balance_before, 10_000_000, "Initial pool balance");

    ctx.core_client.register_player(&hunt_id, &player);
    ctx.core_client.submit_answer(
        &hunt_id,
        &clue_id,
        &player,
        &String::from_str(&env, "A"),
        &1u64,
        &1_700_000_000u64,
    );
    ctx.core_client.complete_hunt(&hunt_id, &player);

    let hunt_after = ctx.core_client.get_hunt_info(&hunt_id);
    assert_eq!(
        hunt_after.completed_count, 1,
        "Completion count should increment"
    );
    assert_eq!(
        hunt_after.reward_config.claimed_count, 1,
        "Claimed count should increment"
    );

    let pool_balance_after = ctx.reward_client.get_pool_balance(&hunt_id);
    let xlm_per_winner = 10_000_000 / 50;
    assert_eq!(
        pool_balance_after,
        pool_balance_before - xlm_per_winner,
        "Pool balance should be decremented by reward amount"
    );

    let supply = ctx.nft_client.total_supply();
    assert_eq!(supply, 1, "One NFT should exist");

    let metadata = ctx.nft_client.get_nft_metadata(&1).unwrap();
    assert_eq!(metadata.hunt_id, hunt_id);
    assert_eq!(metadata.current_owner, player);
}

#[test]
fn test_error_propagation_insufficient_pool_balance() {
    let env = Env::default();
    env.ledger().set_timestamp(1_700_000_000);
    env.mock_all_auths();

    let creator = Address::generate(&env);
    let player = Address::generate(&env);

    let ctx = setup_environment(&env);

    let sac = token::StellarAssetClient::new(&env, &ctx.token_address);
    sac.mint(&creator, &50_000_000);

    let hunt_id = ctx.core_client.create_hunt(
        &creator,
        &String::from_str(&env, "Insufficient Pool Hunt"),
        &String::from_str(&env, "Testing error propagation"),
        &None,
        &None,
        &0u32,
        &None,
        &None,
    );

    let clue_id = ctx.core_client.add_clue(
        &hunt_id,
        &String::from_str(&env, "Q"),
        &String::from_str(&env, "A"),
        &10u32,
        &true,
        &None,
        &None,
    );

    // Each winner expects 20_000_000 (200_000_000 / 10)
    ctx.core_client
        .set_reward_config(&hunt_id, &10u32, &200_000_000i128, &false, &None);

    ctx.core_client.activate_hunt(&hunt_id, &creator);
    ctx.core_client
        .set_reward_manager(&ctx.admin, &ctx.reward_manager_id);

    ctx.reward_client.create_reward_pool(
        &creator,
        &hunt_id,
        &ctx.token_address,
        &1i128,
        &0u32,
        &false,
    );
    // Only fund 10_000_000 (< 20_000_000 needed per winner)
    ctx.reward_client
        .fund_reward_pool(&creator, &hunt_id, &10_000_000i128);

    ctx.core_client.register_player(&hunt_id, &player);
    ctx.core_client.submit_answer(
        &hunt_id,
        &clue_id,
        &player,
        &String::from_str(&env, "A"),
        &1u64,
        &1_700_000_000u64,
    );

    let result = ctx.core_client.try_complete_hunt(&hunt_id, &player);
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

    let ctx = setup_environment(&env);

    let sac = token::StellarAssetClient::new(&env, &ctx.token_address);
    sac.mint(&creator, &50_000_000);

    let hunt_id = ctx.core_client.create_hunt(
        &creator,
        &String::from_str(&env, "Invalid NFT Hunt"),
        &String::from_str(&env, "Testing invalid NFT config"),
        &None,
        &None,
        &0u32,
        &None,
        &None,
    );

    let clue_id = ctx.core_client.add_clue(
        &hunt_id,
        &String::from_str(&env, "Q"),
        &String::from_str(&env, "A"),
        &10u32,
        &true,
        &None,
        &None,
    );

    ctx.core_client.set_reward_config(
        &hunt_id,
        &10u32,
        &10_000_000i128,
        &true,
        &Some(ctx.nft_reward_id.clone()),
    );

    ctx.core_client.activate_hunt(&hunt_id, &creator);
    ctx.core_client
        .set_reward_manager(&ctx.admin, &ctx.reward_manager_id);

    // Corrupt the NFT rarity after activation to test error propagation during completion
    env.as_contract(&ctx.core_id, || {
        let mut hunt = hunty_core::Storage::get_hunt(&env, hunt_id).unwrap();
        hunt.reward_config.nft_rarity = 99;
        hunty_core::Storage::save_hunt(&env, &hunt);
    });

    ctx.reward_client.create_reward_pool(
        &creator,
        &hunt_id,
        &ctx.token_address,
        &1i128,
        &0u32,
        &false,
    );
    ctx.reward_client
        .fund_reward_pool(&creator, &hunt_id, &10_000_000i128);

    ctx.core_client.register_player(&hunt_id, &player);
    ctx.core_client.submit_answer(
        &hunt_id,
        &clue_id,
        &player,
        &String::from_str(&env, "A"),
        &1u64,
        &1_700_000_000u64,
    );

    let result = ctx.core_client.try_complete_hunt(&hunt_id, &player);
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

    let ctx = setup_environment(&env);

    let sac = token::StellarAssetClient::new(&env, &ctx.token_address);
    sac.mint(&creator, &50_000_000);

    let hunt_id = ctx.core_client.create_hunt(
        &creator,
        &String::from_str(&env, "Double Claim Hunt"),
        &String::from_str(&env, "Testing double claim prevention"),
        &None,
        &None,
        &0u32,
        &None,
        &None,
    );

    let clue_id = ctx.core_client.add_clue(
        &hunt_id,
        &String::from_str(&env, "Q"),
        &String::from_str(&env, "A"),
        &10u32,
        &true,
        &None,
        &None,
    );

    ctx.core_client.set_reward_config(
        &hunt_id,
        &10u32,
        &10_000_000i128,
        &true,
        &Some(ctx.nft_reward_id.clone()),
    );

    ctx.core_client.activate_hunt(&hunt_id, &creator);
    ctx.core_client
        .set_reward_manager(&ctx.admin, &ctx.reward_manager_id);

    ctx.reward_client.create_reward_pool_with_nft(
        &creator,
        &hunt_id,
        &ctx.token_address,
        &0i128,
        &Some(ctx.nft_reward_id.clone()),
        &0u32,
        &true,
    );
    ctx.reward_client
        .fund_reward_pool(&creator, &hunt_id, &10_000_000i128);

    ctx.core_client.register_player(&hunt_id, &player);
    ctx.core_client.submit_answer(
        &hunt_id,
        &clue_id,
        &player,
        &String::from_str(&env, "A"),
        &1u64,
        &1_700_000_000u64,
    );

    ctx.core_client.complete_hunt(&hunt_id, &player);

    let result2 = ctx.core_client.try_complete_hunt(&hunt_id, &player);
    assert!(
        result2.is_err(),
        "Second completion should fail (reward already claimed)"
    );

    let supply = ctx.nft_client.total_supply();
    assert_eq!(supply, 1, "Only one NFT should exist");
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

    let ctx = setup_environment(&env);

    let sac = token::StellarAssetClient::new(&env, &ctx.token_address);
    sac.mint(&creator, &100_000_000);

    let hunt_id = ctx.core_client.create_hunt(
        &creator,
        &String::from_str(&env, "Multi-Player Hunt"),
        &String::from_str(&env, "Testing multiple player rewards"),
        &None,
        &None,
        &0u32,
        &None,
        &None,
    );

    let clue_id = ctx.core_client.add_clue(
        &hunt_id,
        &String::from_str(&env, "Q"),
        &String::from_str(&env, "A"),
        &10u32,
        &true,
        &None,
        &None,
    );

    ctx.core_client.set_reward_config(
        &hunt_id,
        &3u32,
        &30_000_000i128,
        &true,
        &Some(ctx.nft_reward_id.clone()),
    );

    ctx.core_client.activate_hunt(&hunt_id, &creator);
    ctx.core_client
        .set_reward_manager(&ctx.admin, &ctx.reward_manager_id);

    ctx.reward_client.create_reward_pool_with_nft(
        &creator,
        &hunt_id,
        &ctx.token_address,
        &0i128,
        &Some(ctx.nft_reward_id.clone()),
        &0u32,
        &true,
    );
    ctx.reward_client
        .fund_reward_pool(&creator, &hunt_id, &30_000_000i128);

    ctx.core_client.register_player(&hunt_id, &player1);
    ctx.core_client.register_player(&hunt_id, &player2);
    ctx.core_client.register_player(&hunt_id, &player3);

    ctx.core_client.submit_answer(
        &hunt_id,
        &clue_id,
        &player1,
        &String::from_str(&env, "A"),
        &1u64,
        &1_700_000_000u64,
    );
    ctx.core_client.submit_answer(
        &hunt_id,
        &clue_id,
        &player2,
        &String::from_str(&env, "A"),
        &2u64,
        &1_700_000_000u64,
    );
    ctx.core_client.submit_answer(
        &hunt_id,
        &clue_id,
        &player3,
        &String::from_str(&env, "A"),
        &3u64,
        &1_700_000_000u64,
    );

    let token_client = token::Client::new(&env, &ctx.token_address);
    let rewards_per_player = 30_000_000 / 3;

    ctx.core_client.complete_hunt(&hunt_id, &player1);
    ctx.core_client.complete_hunt(&hunt_id, &player2);
    ctx.core_client.complete_hunt(&hunt_id, &player3);

    let balance1 = token_client.balance(&player1);
    let balance2 = token_client.balance(&player2);
    let balance3 = token_client.balance(&player3);

    assert_eq!(balance1, rewards_per_player);
    assert_eq!(balance2, rewards_per_player);
    assert_eq!(balance3, rewards_per_player);

    let pool_balance = ctx.reward_client.get_pool_balance(&hunt_id);
    assert_eq!(pool_balance, 30_000_000 - (rewards_per_player * 3));

    let supply = ctx.nft_client.total_supply();
    assert_eq!(supply, 3, "Three NFTs should be minted");
}

#[test]
fn test_cross_contract_call_failure_recovery() {
    let env = Env::default();
    env.ledger().set_timestamp(1_700_000_000);
    env.mock_all_auths();

    let creator = Address::generate(&env);
    let player = Address::generate(&env);
    let admin = Address::generate(&env);

    let core_id = env.register(HuntyCore, ());
    let reward_manager_id = env.register(RewardManager, ());
    let token_admin = Address::generate(&env);

    let token_contract = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_address = token_contract.address();

    let core_client = HuntyCoreClient::new(&env, &core_id);
    let reward_client = RewardManagerClient::new(&env, &reward_manager_id);

    reward_client.initialize(&admin, &token_address, &core_id);
    core_client.initialize_admin(&admin);

    let sac = token::StellarAssetClient::new(&env, &token_address);
    sac.mint(&creator, &50_000_000);

    let hunt_id = core_client.create_hunt(
        &creator,
        &String::from_str(&env, "Failure Recovery Hunt"),
        &String::from_str(&env, "Testing failure recovery"),
        &None,
        &None,
        &0u32,
        &None,
        &None,
    );

    let clue_id = core_client.add_clue(
        &hunt_id,
        &String::from_str(&env, "Q"),
        &String::from_str(&env, "A"),
        &10u32,
        &true,
        &None,
        &None,
    );

    core_client.set_reward_config(&hunt_id, &10u32, &10_000_000i128, &false, &None);

    core_client.activate_hunt(&hunt_id, &creator);
    core_client.set_reward_manager(&admin, &reward_manager_id);

    reward_client.create_reward_pool(&creator, &hunt_id, &token_address, &1i128, &0u32, &false);
    reward_client.fund_reward_pool(&creator, &hunt_id, &10_000_000i128);

    core_client.register_player(&hunt_id, &player);
    core_client.submit_answer(
        &hunt_id,
        &clue_id,
        &player,
        &String::from_str(&env, "A"),
        &1u64,
        &1_700_000_000u64,
    );

    core_client.complete_hunt(&hunt_id, &player);

    let token_client = token::Client::new(&env, &token_address);
    let player_balance = token_client.balance(&player);
    assert!(player_balance > 0, "Player should receive XLM reward");
}

#[test]
fn test_issue_833_cross_contract_nft_image_uri_verification() {
    let env = Env::default();
    env.ledger().set_timestamp(1_700_000_000);
    env.mock_all_auths();

    let creator = Address::generate(&env);
    let player = Address::generate(&env);

    let ctx = setup_environment(&env);

    let sac = token::StellarAssetClient::new(&env, &ctx.token_address);
    sac.mint(&creator, &50_000_000);

    let expected_uri = String::from_str(
        &env,
        "ipfs://bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi",
    );

    let hunt_id = ctx.core_client.create_hunt(
        &creator,
        &String::from_str(&env, "NFT URI Cross Contract Hunt"),
        &String::from_str(&env, "Testing non-empty NFT image URI propagation"),
        &None,
        &None,
        &0u32,
        &None,
        &None,
    );

    let clue_id = ctx.core_client.add_clue(
        &hunt_id,
        &String::from_str(&env, "Question"),
        &String::from_str(&env, "Answer"),
        &10u32,
        &true,
        &None,
        &None,
    );

    // Configure hunt with valid nft_image_uri inside contract environment
    env.as_contract(&ctx.core_id, || {
        let mut hunt = hunty_core::Storage::get_hunt(&env, hunt_id).unwrap();
        hunt.reward_config = hunty_core::types::RewardConfig::new(
            &env,
            0,
            true, // nft_enabled
            Some(ctx.nft_reward_id.clone()),
            1,
            0,
            0,
            Some(expected_uri.clone()),
        );
        hunty_core::Storage::save_hunt(&env, &hunt);
    });

    ctx.core_client.activate_hunt(&hunt_id, &creator);
    ctx.core_client
        .set_reward_manager(&ctx.admin, &ctx.reward_manager_id);

    ctx.reward_client.create_reward_pool_with_nft(
        &creator,
        &hunt_id,
        &ctx.token_address,
        &0i128,
        &Some(ctx.nft_reward_id.clone()),
        &0u32,
        &true,
    );
    ctx.reward_client
        .fund_reward_pool(&creator, &hunt_id, &10_000_000i128);

    ctx.core_client.register_player(&hunt_id, &player);
    ctx.core_client.submit_answer(
        &hunt_id,
        &clue_id,
        &player,
        &String::from_str(&env, "Answer"),
        &1u64,
        &1_700_000_000u64,
    );

    ctx.core_client.complete_hunt(&hunt_id, &player);

    let nft_count = ctx.nft_client.total_supply();
    assert_eq!(nft_count, 1, "One NFT should be minted");

    let nft_metadata = ctx.nft_client.get_nft_metadata(&1).unwrap();
    assert_eq!(
        nft_metadata.image_uri, expected_uri,
        "NFT image URI must match configured non-empty URI"
    );
}
