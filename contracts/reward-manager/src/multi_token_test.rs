use crate::{RewardErrorCode, RewardManager};
use reward_interface::RewardConfig;
use soroban_sdk::{testutils::Address as _, token::StellarAssetClient, Address, Env, String};

/// Registers a mock token contract and returns (token_address, token_admin).
fn create_mock_token(env: &Env) -> (Address, Address) {
    let admin = Address::generate(env);
    let address = env
        .register_stellar_asset_contract_v2(admin.clone())
        .address();
    (address, admin)
}

/// Mints `amount` units of the token at `token_address` to `to`.
fn mint_tokens(env: &Env, token_address: &Address, to: &Address, amount: i128) {
    StellarAssetClient::new(env, token_address).mint(to, &amount);
}

#[test]
fn test_create_pool_with_xlm_token() {
    let env = Env::default();
    env.mock_all_auths_allowing_non_root_auth();

    let contract_id = env.register(RewardManager, ());
    let admin = Address::generate(&env);
    let creator = Address::generate(&env);
    let (xlm_token, _xlm_admin) = create_mock_token(&env);

    let hunty_core = Address::generate(&env);
    env.as_contract(&contract_id, || {
        // Initialize with XLM token
        RewardManager::initialize(
            env.clone(),
            admin.clone(),
            xlm_token.clone(),
            hunty_core.clone(),
        )
        .unwrap();

        // Create pool with XLM token. Zero-minimum pools are NFT-only pools
        // and must declare an NFT contract address.
        let result = RewardManager::create_reward_pool_with_nft(
            env.clone(),
            creator.clone(),
            1,
            xlm_token.clone(),
            0,
            Some(Address::generate(&env)),
            0,
            true,
        );

        assert!(result.is_ok());

        // Verify pool config has correct token address
        let config = RewardManager::get_pool_config(env.clone(), 1).unwrap();
        assert_eq!(config.token_address, xlm_token);
        assert_eq!(config.creator, creator);
    });
}

#[test]
fn test_create_pool_with_usdc_token() {
    let env = Env::default();
    env.mock_all_auths_allowing_non_root_auth();

    let contract_id = env.register(RewardManager, ());
    let admin = Address::generate(&env);
    let creator = Address::generate(&env);
    let (xlm_token, _xlm_admin) = create_mock_token(&env);
    let (usdc_token, _usdc_admin) = create_mock_token(&env); // Different token (e.g., USDC)

    let hunty_core = Address::generate(&env);
    env.as_contract(&contract_id, || {
        // Initialize with XLM token (still needed for backward compatibility)
        RewardManager::initialize(env.clone(), admin.clone(), xlm_token.clone(), hunty_core)
            .unwrap();

        // Create pool with USDC token. Zero-minimum pools are NFT-only pools
        // and must declare an NFT contract address.
        let result = RewardManager::create_reward_pool_with_nft(
            env.clone(),
            creator.clone(),
            1,
            usdc_token.clone(),
            0,
            Some(Address::generate(&env)),
            0,
            true,
        );

        assert!(result.is_ok());

        // Verify pool config has USDC token address
        let config = RewardManager::get_pool_config(env.clone(), 1).unwrap();
        assert_eq!(config.token_address, usdc_token);
    });
}

#[test]
fn test_create_multiple_pools_with_different_tokens() {
    let env = Env::default();
    env.mock_all_auths_allowing_non_root_auth();

    let contract_id = env.register(RewardManager, ());
    let admin = Address::generate(&env);
    let creator = Address::generate(&env);
    let (xlm_token, _xlm_admin) = create_mock_token(&env);
    let (usdc_token, _usdc_admin) = create_mock_token(&env);
    let (eurc_token, _eurc_admin) = create_mock_token(&env);

    let hunty_core = Address::generate(&env);
    env.as_contract(&contract_id, || {
        RewardManager::initialize(env.clone(), admin.clone(), xlm_token.clone(), hunty_core)
            .unwrap();

        // Create pool 1 with XLM
        RewardManager::create_reward_pool_with_nft(
            env.clone(),
            creator.clone(),
            1,
            xlm_token.clone(),
            0,
            Some(Address::generate(&env)),
            0,
            true,
        )
        .unwrap();

        // Create pool 2 with USDC
        RewardManager::create_reward_pool_with_nft(
            env.clone(),
            creator.clone(),
            2,
            usdc_token.clone(),
            0,
            Some(Address::generate(&env)),
            0,
            true,
        )
        .unwrap();

        // Create pool 3 with EURC
        RewardManager::create_reward_pool_with_nft(
            env.clone(),
            creator.clone(),
            3,
            eurc_token.clone(),
            0,
            Some(Address::generate(&env)),
            0,
            true,
        )
        .unwrap();

        // Verify each pool has the correct token
        let config1 = RewardManager::get_pool_config(env.clone(), 1).unwrap();
        assert_eq!(config1.token_address, xlm_token);

        let config2 = RewardManager::get_pool_config(env.clone(), 2).unwrap();
        assert_eq!(config2.token_address, usdc_token);

        let config3 = RewardManager::get_pool_config(env.clone(), 3).unwrap();
        assert_eq!(config3.token_address, eurc_token);
    });
}

#[test]
fn test_invalid_token_contract_rejected() {
    let env = Env::default();
    env.mock_all_auths_allowing_non_root_auth();

    let contract_id = env.register(RewardManager, ());
    let admin = Address::generate(&env);
    let creator = Address::generate(&env);
    let (xlm_token, _xlm_admin) = create_mock_token(&env);
    let invalid_token = Address::generate(&env); // Not a token contract

    let hunty_core = Address::generate(&env);
    env.as_contract(&contract_id, || {
        RewardManager::initialize(env.clone(), admin.clone(), xlm_token.clone(), hunty_core)
            .unwrap();

        // Try to create pool with invalid token contract
        let result = RewardManager::create_reward_pool(
            env.clone(),
            creator.clone(),
            1,
            invalid_token,
            1,
            0,
            true,
        );

        // Should fail with InvalidTokenContract error
        assert_eq!(result, Err(RewardErrorCode::InvalidTokenContract));
    });
}

#[test]
fn test_fund_pool_uses_correct_token() {
    let env = Env::default();
    env.mock_all_auths_allowing_non_root_auth();

    let contract_id = env.register(RewardManager, ());
    let admin = Address::generate(&env);
    let creator = Address::generate(&env);
    let (xlm_token, _xlm_admin) = create_mock_token(&env);
    let (usdc_token, _usdc_admin) = create_mock_token(&env);

    // The creator needs a USDC balance before the pool can be funded.
    mint_tokens(&env, &usdc_token, &creator, 50_000_000);

    let hunty_core = Address::generate(&env);
    env.as_contract(&contract_id, || {
        RewardManager::initialize(env.clone(), admin.clone(), xlm_token.clone(), hunty_core)
            .unwrap();

        // Create pool with USDC
        RewardManager::create_reward_pool_with_nft(
            env.clone(),
            creator.clone(),
            1,
            usdc_token.clone(),
            0,
            Some(Address::generate(&env)),
            0,
            true,
        )
        .unwrap();

        // Fund the pool
        let fund_amount = 50_000_000i128;
        let result = RewardManager::fund_reward_pool(env.clone(), creator.clone(), 1, fund_amount);

        assert!(result.is_ok());

        // Verify pool balance
        let pool_status = RewardManager::get_reward_pool(env.clone(), 1).unwrap();
        assert_eq!(pool_status.balance, fund_amount);
        assert_eq!(pool_status.total_deposited, fund_amount);
    });
}

#[test]
fn test_distribute_rewards_uses_pool_token() {
    let env = Env::default();
    env.mock_all_auths_allowing_non_root_auth();

    let contract_id = env.register(RewardManager, ());
    let admin = Address::generate(&env);
    let creator = Address::generate(&env);
    let player = Address::generate(&env);
    let (xlm_token, _xlm_admin) = create_mock_token(&env);
    let (usdc_token, _usdc_admin) = create_mock_token(&env);

    // The creator needs a USDC balance before the pool can be funded.
    mint_tokens(&env, &usdc_token, &creator, 100_000_000);

    let hunty_core = Address::generate(&env);
    env.as_contract(&contract_id, || {
        RewardManager::initialize(env.clone(), admin.clone(), xlm_token.clone(), hunty_core)
            .unwrap();

        // Create pool with USDC
        RewardManager::create_reward_pool_with_nft(
            env.clone(),
            creator.clone(),
            1,
            usdc_token.clone(),
            0,
            Some(Address::generate(&env)),
            0,
            true,
        )
        .unwrap();

        // Fund the pool
        RewardManager::fund_reward_pool(env.clone(), creator.clone(), 1, 100_000_000).unwrap();

        // Distribute rewards
        let reward_config = RewardConfig {
            xlm_amount: Some(10_000_000),
            nft_contract: None,
            nft_title: String::from_str(&env, ""),
            nft_description: String::from_str(&env, ""),
            nft_image_uri: String::from_str(&env, ""),
            nft_hunt_title: String::from_str(&env, ""),
            nft_rarity: 0,
            nft_tier: 0,
            completion_rank: 0,
        };

        let result =
            RewardManager::distribute_rewards(env.clone(), 1, player.clone(), reward_config);

        assert!(result.is_ok());

        // Verify pool balance decreased
        let pool_status = RewardManager::get_reward_pool(env.clone(), 1).unwrap();
        assert_eq!(pool_status.balance, 90_000_000);
        assert_eq!(pool_status.total_distributed, 10_000_000);
    });
}

#[test]
fn test_refund_pool_uses_correct_token() {
    let env = Env::default();
    env.mock_all_auths_allowing_non_root_auth();

    let contract_id = env.register(RewardManager, ());
    let admin = Address::generate(&env);
    let creator = Address::generate(&env);
    let (xlm_token, _xlm_admin) = create_mock_token(&env);
    let (usdc_token, _usdc_admin) = create_mock_token(&env);

    // The creator needs a USDC balance before the pool can be funded.
    mint_tokens(&env, &usdc_token, &creator, 50_000_000);

    let hunty_core = Address::generate(&env);
    env.as_contract(&contract_id, || {
        RewardManager::initialize(env.clone(), admin.clone(), xlm_token.clone(), hunty_core)
            .unwrap();

        // Create and fund pool with USDC
        RewardManager::create_reward_pool_with_nft(
            env.clone(),
            creator.clone(),
            1,
            usdc_token.clone(),
            0,
            Some(Address::generate(&env)),
            0,
            true,
        )
        .unwrap();

        RewardManager::fund_reward_pool(env.clone(), creator.clone(), 1, 50_000_000).unwrap();

        // Refund the pool
        let result = RewardManager::refund_pool(env.clone(), creator.clone(), 1);

        assert!(result.is_ok());

        // Verify pool balance is zero
        let pool_status = RewardManager::get_reward_pool(env.clone(), 1).unwrap();
        assert_eq!(pool_status.balance, 0);
    });
}
