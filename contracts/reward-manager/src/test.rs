#[cfg(test)]
mod test {
    use crate::errors::RewardErrorCode;
    use crate::storage::Storage;
    use crate::types::RewardConfig;
    use crate::RewardManager;
    use soroban_sdk::testutils::{Address as _, Ledger as _};
    use soroban_sdk::{token, Address, Env};

    /// Registers the RewardManager contract and a mock SAC token.
    /// Returns (contract_id, token_address, token_admin).
    fn setup(env: &Env) -> (Address, Address, Address) {
        let contract_id = env.register(RewardManager, ());
        let token_admin = Address::generate(&env);
        let token_contract = env.register_stellar_asset_contract_v2(token_admin.clone());
        let token_address = token_contract.address();
        (contract_id, token_address, token_admin)
    }

    /// Mints tokens to an address using the SAC admin.
    fn mint_tokens(env: &Env, token_address: &Address, _admin: &Address, to: &Address, amount: i128) {
        let client = token::StellarAssetClient::new(env, token_address);
        client.mint(to, &amount);
    }

    fn get_balance(env: &Env, token_address: &Address, addr: &Address) -> i128 {
        let client = token::Client::new(env, token_address);
        client.balance(addr)
    }

    fn xlm_only_config(env: &Env, amount: i128) -> RewardConfig {
        RewardConfig {
            xlm_amount: Some(amount),
            nft_contract: None,
            nft_title: soroban_sdk::String::from_str(env, ""),
            nft_description: soroban_sdk::String::from_str(env, ""),
            nft_image_uri: soroban_sdk::String::from_str(env, ""),
            nft_hunt_title: soroban_sdk::String::from_str(env, ""),
            nft_rarity: 0,
            nft_tier: 0,
        }
    }

    fn initialize_contract(env: &Env, token_address: &Address) {
        let admin = Address::generate(&env);
        RewardManager::initialize(env.clone(), admin, token_address.clone()).unwrap();
    }

    // ========== Initialization ==========

    #[test]
    fn test_initialize_sets_xlm_token() {
        let env = Env::default();
        env.mock_all_auths_allowing_non_root_auth();
        let (contract_id, token_address, _) = setup(&env);

        env.as_contract(&contract_id, || {
            initialize_contract(&env, &token_address);
            assert_eq!(Storage::get_xlm_token(&env), Some(token_address.clone()));
        });
    }

    #[test]
    fn test_initialize_cannot_be_called_twice() {
        let env = Env::default();
        env.mock_all_auths_allowing_non_root_auth();
        let (contract_id, token_address, token_admin) = setup(&env);
        let second_token = env
            .register_stellar_asset_contract_v2(token_admin)
            .address();

        env.as_contract(&contract_id, || {
            let admin = Address::generate(&env);
            RewardManager::initialize(env.clone(), admin.clone(), token_address.clone()).unwrap();
            let result = RewardManager::initialize(env.clone(), admin, second_token.clone());
            assert_eq!(result, Err(RewardErrorCode::AlreadyInitialized));
            assert_eq!(Storage::get_xlm_token(&env), Some(token_address.clone()));
        });
    }

    #[test]
    fn test_set_nft_reward_contract_admin_only() {
        let env = Env::default();
        env.mock_all_auths_allowing_non_root_auth();
        let (contract_id, token_address, _) = setup(&env);
        let admin = Address::generate(&env);
        let attacker = Address::generate(&env);
        let nft_contract = Address::generate(&env);

        env.as_contract(&contract_id, || {
            RewardManager::initialize(env.clone(), admin.clone(), token_address).unwrap();
        });
        env.mock_all_auths_allowing_non_root_auth();
        env.as_contract(&contract_id, || {
            let unauthorized =
                RewardManager::set_nft_reward_contract(env.clone(), attacker, nft_contract.clone());
            assert_eq!(unauthorized, Err(RewardErrorCode::Unauthorized));
        });
        env.mock_all_auths_allowing_non_root_auth();
        env.as_contract(&contract_id, || {
            RewardManager::set_nft_reward_contract(env.clone(), admin, nft_contract.clone()).unwrap();
            assert_eq!(Storage::get_nft_contract(&env), Some(nft_contract));
        });
    }

    // ========== create_reward_pool ==========

    #[test]
    fn test_create_reward_pool_success() {
        let env = Env::default();
        env.mock_all_auths_allowing_non_root_auth();
        let (contract_id, _, _) = setup(&env);
        let creator = Address::generate(&env);

        env.as_contract(&contract_id, || {
            let result = RewardManager::create_reward_pool(env.clone(), creator.clone(), 1, 0);
            assert!(result.is_ok());

            // Pool should now be queryable
            let status = RewardManager::get_reward_pool(env.clone(), 1);
            assert!(status.is_some());
            let status = status.unwrap();
            assert_eq!(status.creator, creator);
            assert_eq!(status.balance, 0);
            assert_eq!(status.total_deposited, 0);
            assert_eq!(status.total_distributed, 0);
            assert_eq!(status.min_distribution_amount, 0);
        });
    }

    #[test]
    fn test_create_reward_pool_with_minimum() {
        let env = Env::default();
        env.mock_all_auths_allowing_non_root_auth();
        let (contract_id, _, _) = setup(&env);
        let creator = Address::generate(&env);

        env.as_contract(&contract_id, || {
            RewardManager::create_reward_pool(env.clone(), creator.clone(), 42, 500).unwrap();

            let status = RewardManager::get_reward_pool(env.clone(), 42).unwrap();
            assert_eq!(status.min_distribution_amount, 500);
        });
    }

    #[test]
    fn test_create_reward_pool_duplicate_fails() {
        let env = Env::default();
        env.mock_all_auths_allowing_non_root_auth();
        let (contract_id, _, _) = setup(&env);
        let creator = Address::generate(&env);

        env.as_contract(&contract_id, || {
            RewardManager::create_reward_pool(env.clone(), creator.clone(), 1, 0).unwrap();

            let result = RewardManager::create_reward_pool(env.clone(), creator.clone(), 1, 0);
            assert_eq!(result, Err(RewardErrorCode::PoolAlreadyExists));
        });
    }

    #[test]
    fn test_create_reward_pool_negative_minimum_fails() {
        let env = Env::default();
        env.mock_all_auths_allowing_non_root_auth();
        let (contract_id, _, _) = setup(&env);
        let creator = Address::generate(&env);

        env.as_contract(&contract_id, || {
            let result = RewardManager::create_reward_pool(env.clone(), creator.clone(), 1, -1);
            assert_eq!(result, Err(RewardErrorCode::InvalidAmount));
        });
    }

    // ========== fund_reward_pool ==========

    #[test]
    fn test_fund_reward_pool() {
        let env = Env::default();
        env.mock_all_auths_allowing_non_root_auth();
        let (contract_id, token_address, token_admin) = setup(&env);
        let creator = Address::generate(&env);

        mint_tokens(&env, &token_address, &token_admin, &creator, 10_000);

        env.as_contract(&contract_id, || {
            initialize_contract(&env, &token_address);
            RewardManager::create_reward_pool(env.clone(), creator.clone(), 1, 0).unwrap();
            RewardManager::fund_reward_pool(env.clone(), creator.clone(), 1, 5_000).unwrap();
        });

        // Verify pool balance
        env.as_contract(&contract_id, || {
            assert_eq!(RewardManager::get_pool_balance(env.clone(), 1), 5_000);
        });

        // Verify tokens transferred to contract
        assert_eq!(get_balance(&env, &token_address, &contract_id), 5_000);
        assert_eq!(get_balance(&env, &token_address, &creator), 5_000);
    }

    #[test]
    fn test_fund_reward_pool_invalid_amount() {
        let env = Env::default();
        env.mock_all_auths_allowing_non_root_auth();
        let (contract_id, token_address, _) = setup(&env);
        let creator = Address::generate(&env);

        env.as_contract(&contract_id, || {
            initialize_contract(&env, &token_address);
            RewardManager::create_reward_pool(env.clone(), creator.clone(), 1, 0).unwrap();
            let result = RewardManager::fund_reward_pool(env.clone(), creator.clone(), 1, 0);
            assert_eq!(result, Err(RewardErrorCode::InvalidAmount));
        });
    }

    #[test]
    fn test_fund_reward_pool_not_initialized() {
        let env = Env::default();
        env.mock_all_auths_allowing_non_root_auth();
        let (contract_id, _, _) = setup(&env);
        let creator = Address::generate(&env);

        // Pool created, but XLM token not initialized
        env.as_contract(&contract_id, || {
            RewardManager::create_reward_pool(env.clone(), creator.clone(), 1, 0).unwrap();
            let result = RewardManager::fund_reward_pool(env.clone(), creator.clone(), 1, 1000);
            assert_eq!(result, Err(RewardErrorCode::NotInitialized));
        });
    }

    #[test]
    fn test_fund_reward_pool_not_created() {
        let env = Env::default();
        env.mock_all_auths_allowing_non_root_auth();
        let (contract_id, token_address, _) = setup(&env);
        let funder = Address::generate(&env);

        env.as_contract(&contract_id, || {
            initialize_contract(&env, &token_address);
            // Skip create_reward_pool — should fail with PoolNotFound
            let result = RewardManager::fund_reward_pool(env.clone(), funder.clone(), 1, 1000);
            assert_eq!(result, Err(RewardErrorCode::PoolNotFound));
        });
    }

    #[test]
    fn test_fund_reward_pool_unauthorized_funder() {
        let env = Env::default();
        env.mock_all_auths_allowing_non_root_auth();
        let (contract_id, token_address, token_admin) = setup(&env);
        let creator = Address::generate(&env);
        let attacker = Address::generate(&env);

        mint_tokens(&env, &token_address, &token_admin, &attacker, 10_000);

        env.as_contract(&contract_id, || {
            initialize_contract(&env, &token_address);
            RewardManager::create_reward_pool(env.clone(), creator.clone(), 1, 0).unwrap();

            // Non-creator tries to fund
            let result = RewardManager::fund_reward_pool(env.clone(), attacker.clone(), 1, 1_000);
            assert_eq!(result, Err(RewardErrorCode::Unauthorized));
        });

        // Attacker's balance unchanged
        assert_eq!(get_balance(&env, &token_address, &attacker), 10_000);
    }

    #[test]
    fn test_fund_reward_pool_additive() {
        let env = Env::default();
        env.mock_all_auths_allowing_non_root_auth();
        let (contract_id, token_address, token_admin) = setup(&env);
        let creator = Address::generate(&env);

        mint_tokens(&env, &token_address, &token_admin, &creator, 20_000);

        env.as_contract(&contract_id, || {
            initialize_contract(&env, &token_address);
            RewardManager::create_reward_pool(env.clone(), creator.clone(), 1, 0).unwrap();
            RewardManager::fund_reward_pool(env.clone(), creator.clone(), 1, 5_000).unwrap();
            RewardManager::fund_reward_pool(env.clone(), creator.clone(), 1, 3_000).unwrap();
            assert_eq!(RewardManager::get_pool_balance(env.clone(), 1), 8_000);
        });

        assert_eq!(get_balance(&env, &token_address, &contract_id), 8_000);
    }

    #[test]
    fn test_fund_reward_pool_updates_total_deposited() {
        let env = Env::default();
        env.mock_all_auths_allowing_non_root_auth();
        let (contract_id, token_address, token_admin) = setup(&env);
        let creator = Address::generate(&env);

        mint_tokens(&env, &token_address, &token_admin, &creator, 10_000);

        env.as_contract(&contract_id, || {
            initialize_contract(&env, &token_address);
            RewardManager::create_reward_pool(env.clone(), creator.clone(), 1, 0).unwrap();
            RewardManager::fund_reward_pool(env.clone(), creator.clone(), 1, 4_000).unwrap();
            RewardManager::fund_reward_pool(env.clone(), creator.clone(), 1, 2_000).unwrap();

            let status = RewardManager::get_reward_pool(env.clone(), 1).unwrap();
            assert_eq!(status.total_deposited, 6_000);
            assert_eq!(status.balance, 6_000);
        });
    }

    // ========== get_reward_pool ==========

    #[test]
    fn test_get_reward_pool_none_before_creation() {
        let env = Env::default();
        env.mock_all_auths_allowing_non_root_auth();
        let (contract_id, _, _) = setup(&env);

        env.as_contract(&contract_id, || {
            assert!(RewardManager::get_reward_pool(env.clone(), 99).is_none());
        });
    }

    #[test]
    fn test_get_reward_pool_tracks_all_fields() {
        let env = Env::default();
        env.mock_all_auths_allowing_non_root_auth();
        let (contract_id, token_address, token_admin) = setup(&env);
        let creator = Address::generate(&env);
        let player = Address::generate(&env);

        mint_tokens(&env, &token_address, &token_admin, &creator, 10_000);

        env.as_contract(&contract_id, || {
            initialize_contract(&env, &token_address);
            RewardManager::create_reward_pool(env.clone(), creator.clone(), 1, 100).unwrap();
            RewardManager::fund_reward_pool(env.clone(), creator.clone(), 1, 8_000).unwrap();
            RewardManager::distribute_rewards(
                env.clone(),
                1,
                player.clone(),
                xlm_only_config(&env, 3_000),
            )
            .unwrap();

            let status = RewardManager::get_reward_pool(env.clone(), 1).unwrap();
            assert_eq!(status.balance, 5_000);
            assert_eq!(status.total_deposited, 8_000);
            assert_eq!(status.total_distributed, 3_000);
            assert_eq!(status.creator, creator);
            assert_eq!(status.min_distribution_amount, 100);
        });
    }

    // ========== validate_pool ==========

    #[test]
    fn test_validate_pool_valid() {
        let env = Env::default();
        env.mock_all_auths_allowing_non_root_auth();
        let (contract_id, token_address, token_admin) = setup(&env);
        let creator = Address::generate(&env);

        mint_tokens(&env, &token_address, &token_admin, &creator, 10_000);

        env.as_contract(&contract_id, || {
            initialize_contract(&env, &token_address);
            RewardManager::create_reward_pool(env.clone(), creator.clone(), 1, 0).unwrap();
            RewardManager::fund_reward_pool(env.clone(), creator.clone(), 1, 5_000).unwrap();

            let result = RewardManager::validate_pool(env.clone(), 1, 5_000);
            assert!(result.is_valid);
            assert_eq!(result.balance, 5_000);
            assert_eq!(result.required, 5_000);
        });
    }

    #[test]
    fn test_validate_pool_insufficient_funds() {
        let env = Env::default();
        env.mock_all_auths_allowing_non_root_auth();
        let (contract_id, token_address, token_admin) = setup(&env);
        let creator = Address::generate(&env);

        mint_tokens(&env, &token_address, &token_admin, &creator, 1_000);

        env.as_contract(&contract_id, || {
            initialize_contract(&env, &token_address);
            RewardManager::create_reward_pool(env.clone(), creator.clone(), 1, 0).unwrap();
            RewardManager::fund_reward_pool(env.clone(), creator.clone(), 1, 1_000).unwrap();

            let result = RewardManager::validate_pool(env.clone(), 1, 5_000);
            assert!(!result.is_valid);
            assert_eq!(result.balance, 1_000);
            assert_eq!(result.required, 5_000);
        });
    }

    #[test]
    fn test_validate_pool_below_minimum() {
        let env = Env::default();
        env.mock_all_auths_allowing_non_root_auth();
        let (contract_id, token_address, token_admin) = setup(&env);
        let creator = Address::generate(&env);

        mint_tokens(&env, &token_address, &token_admin, &creator, 10_000);

        env.as_contract(&contract_id, || {
            initialize_contract(&env, &token_address);
            // Pool requires minimum 500 per distribution
            RewardManager::create_reward_pool(env.clone(), creator.clone(), 1, 500).unwrap();
            RewardManager::fund_reward_pool(env.clone(), creator.clone(), 1, 5_000).unwrap();

            // 200 < minimum 500 → invalid even though funds are available
            let result = RewardManager::validate_pool(env.clone(), 1, 200);
            assert!(!result.is_valid);

            // 500 == minimum → valid
            let result = RewardManager::validate_pool(env.clone(), 1, 500);
            assert!(result.is_valid);
        });
    }

    #[test]
    fn test_validate_pool_not_created() {
        let env = Env::default();
        env.mock_all_auths_allowing_non_root_auth();
        let (contract_id, _, _) = setup(&env);

        env.as_contract(&contract_id, || {
            let result = RewardManager::validate_pool(env.clone(), 99, 1_000);
            assert!(!result.is_valid);
            assert_eq!(result.balance, 0);
        });
    }

    #[test]
    fn test_validate_pool_zero_required_fails() {
        let env = Env::default();
        env.mock_all_auths_allowing_non_root_auth();
        let (contract_id, token_address, token_admin) = setup(&env);
        let creator = Address::generate(&env);

        mint_tokens(&env, &token_address, &token_admin, &creator, 5_000);

        env.as_contract(&contract_id, || {
            initialize_contract(&env, &token_address);
            RewardManager::create_reward_pool(env.clone(), creator.clone(), 1, 0).unwrap();
            RewardManager::fund_reward_pool(env.clone(), creator.clone(), 1, 5_000).unwrap();

            // required = 0 is not a valid distribution
            let result = RewardManager::validate_pool(env.clone(), 1, 0);
            assert!(!result.is_valid);
        });
    }

    // ========== distribute_rewards ==========

    #[test]
    fn test_distribute_rewards_success() {
        let env = Env::default();
        env.mock_all_auths_allowing_non_root_auth();
        let (contract_id, token_address, token_admin) = setup(&env);
        let creator = Address::generate(&env);
        let player = Address::generate(&env);

        mint_tokens(&env, &token_address, &token_admin, &creator, 10_000);

        env.as_contract(&contract_id, || {
            initialize_contract(&env, &token_address);
            RewardManager::create_reward_pool(env.clone(), creator.clone(), 1, 0).unwrap();
            RewardManager::fund_reward_pool(env.clone(), creator.clone(), 1, 5_000).unwrap();

            let config = xlm_only_config(&env, 2_000);
            let result = RewardManager::distribute_rewards(env.clone(), 1, player.clone(), config);
            assert!(result.is_ok());
        });

        // Verify player received tokens
        assert_eq!(get_balance(&env, &token_address, &player), 2_000);
        // Verify contract balance decreased
        assert_eq!(get_balance(&env, &token_address, &contract_id), 3_000);

        // Verify pool balance updated
        env.as_contract(&contract_id, || {
            assert_eq!(RewardManager::get_pool_balance(env.clone(), 1), 3_000);
        });

        // Verify distribution tracked
        env.as_contract(&contract_id, || {
            assert!(RewardManager::is_reward_distributed(env.clone(), 1, player.clone()));
        });
    }

    #[test]
    fn test_distribute_rewards_insufficient_pool() {
        let env = Env::default();
        env.mock_all_auths_allowing_non_root_auth();
        let (contract_id, token_address, token_admin) = setup(&env);
        let creator = Address::generate(&env);
        let player = Address::generate(&env);

        mint_tokens(&env, &token_address, &token_admin, &creator, 1_000);

        env.as_contract(&contract_id, || {
            initialize_contract(&env, &token_address);
            RewardManager::create_reward_pool(env.clone(), creator.clone(), 1, 0).unwrap();
            RewardManager::fund_reward_pool(env.clone(), creator.clone(), 1, 1_000).unwrap();

            // Try to distribute more than pool has
            let config = xlm_only_config(&env, 5_000);
            let result = RewardManager::distribute_rewards(env.clone(), 1, player.clone(), config);
            assert_eq!(result, Err(RewardErrorCode::InsufficientPool));
        });

        // Verify player didn't receive tokens
        assert_eq!(get_balance(&env, &token_address, &player), 0);
    }

    #[test]
    fn test_distribute_rewards_below_minimum() {
        let env = Env::default();
        env.mock_all_auths_allowing_non_root_auth();
        let (contract_id, token_address, token_admin) = setup(&env);
        let creator = Address::generate(&env);
        let player = Address::generate(&env);

        mint_tokens(&env, &token_address, &token_admin, &creator, 5_000);

        env.as_contract(&contract_id, || {
            initialize_contract(&env, &token_address);
            // Pool requires minimum 1_000 per distribution
            RewardManager::create_reward_pool(env.clone(), creator.clone(), 1, 1_000).unwrap();
            RewardManager::fund_reward_pool(env.clone(), creator.clone(), 1, 5_000).unwrap();

            // Attempt to distribute 500 — below minimum of 1_000
            let config = xlm_only_config(&env, 500);
            let result = RewardManager::distribute_rewards(env.clone(), 1, player.clone(), config);
            assert_eq!(result, Err(RewardErrorCode::BelowMinimumAmount));
        });

        assert_eq!(get_balance(&env, &token_address, &player), 0);
    }

    #[test]
    fn test_distribute_rewards_meets_minimum() {
        let env = Env::default();
        env.mock_all_auths_allowing_non_root_auth();
        let (contract_id, token_address, token_admin) = setup(&env);
        let creator = Address::generate(&env);
        let player = Address::generate(&env);

        mint_tokens(&env, &token_address, &token_admin, &creator, 5_000);

        env.as_contract(&contract_id, || {
            initialize_contract(&env, &token_address);
            RewardManager::create_reward_pool(env.clone(), creator.clone(), 1, 1_000).unwrap();
            RewardManager::fund_reward_pool(env.clone(), creator.clone(), 1, 5_000).unwrap();

            // Distribute exactly the minimum
            let config = xlm_only_config(&env, 1_000);
            let result = RewardManager::distribute_rewards(env.clone(), 1, player.clone(), config);
            assert!(result.is_ok());
        });

        assert_eq!(get_balance(&env, &token_address, &player), 1_000);
    }

    #[test]
    fn test_distribute_rewards_double_distribution() {
        let env = Env::default();
        env.mock_all_auths_allowing_non_root_auth();
        let (contract_id, token_address, token_admin) = setup(&env);
        let creator = Address::generate(&env);
        let player = Address::generate(&env);

        mint_tokens(&env, &token_address, &token_admin, &creator, 10_000);

        env.as_contract(&contract_id, || {
            initialize_contract(&env, &token_address);
            RewardManager::create_reward_pool(env.clone(), creator.clone(), 1, 0).unwrap();
            RewardManager::fund_reward_pool(env.clone(), creator.clone(), 1, 10_000).unwrap();

            // First distribution — success
            let config1 = xlm_only_config(&env, 2_000);
            let result1 =
                RewardManager::distribute_rewards(env.clone(), 1, player.clone(), config1);
            assert!(result1.is_ok());

            // Second distribution — blocked
            let config2 = xlm_only_config(&env, 2_000);
            let result2 =
                RewardManager::distribute_rewards(env.clone(), 1, player.clone(), config2);
            assert_eq!(result2, Err(RewardErrorCode::AlreadyDistributed));
        });

        // Verify player only received once
        assert_eq!(get_balance(&env, &token_address, &player), 2_000);
    }

    #[test]
    fn test_distribute_rewards_invalid_config() {
        let env = Env::default();
        env.mock_all_auths_allowing_non_root_auth();
        let (contract_id, token_address, _) = setup(&env);
        let player = Address::generate(&env);

        env.as_contract(&contract_id, || {
            initialize_contract(&env, &token_address);

            // Empty config (no XLM, no NFT)
            let config = RewardConfig {
                xlm_amount: None,
                nft_contract: None,
                nft_title: soroban_sdk::String::from_str(&env, ""),
                nft_description: soroban_sdk::String::from_str(&env, ""),
                nft_image_uri: soroban_sdk::String::from_str(&env, ""),
                nft_hunt_title: soroban_sdk::String::from_str(&env, ""),
                nft_rarity: 0,
                nft_tier: 0,
            };
            let result =
                RewardManager::distribute_rewards(env.clone(), 1, player.clone(), config);
            assert_eq!(result, Err(RewardErrorCode::InvalidConfig));
        });
    }

    #[test]
    fn test_distribute_rewards_invalid_amount() {
        let env = Env::default();
        env.mock_all_auths_allowing_non_root_auth();
        let (contract_id, token_address, _) = setup(&env);
        let player = Address::generate(&env);

        env.as_contract(&contract_id, || {
            initialize_contract(&env, &token_address);

            // Config with zero XLM amount is invalid (has_xlm returns false → InvalidConfig)
            let config = RewardConfig {
                xlm_amount: Some(0),
                nft_contract: None,
                nft_title: soroban_sdk::String::from_str(&env, ""),
                nft_description: soroban_sdk::String::from_str(&env, ""),
                nft_image_uri: soroban_sdk::String::from_str(&env, ""),
                nft_hunt_title: soroban_sdk::String::from_str(&env, ""),
                nft_rarity: 0,
                nft_tier: 0,
            };
            let result =
                RewardManager::distribute_rewards(env.clone(), 1, player.clone(), config);
            assert_eq!(result, Err(RewardErrorCode::InvalidConfig));
        });
    }

    #[test]
    fn test_distribute_rewards_not_initialized() {
        let env = Env::default();
        env.mock_all_auths_allowing_non_root_auth();
        let (contract_id, _, _) = setup(&env);
        let player = Address::generate(&env);

        env.as_contract(&contract_id, || {
            let config = xlm_only_config(&env, 1_000);
            let result =
                RewardManager::distribute_rewards(env.clone(), 1, player.clone(), config);
            assert_eq!(result, Err(RewardErrorCode::NotInitialized));
        });
    }

    #[test]
    fn test_distribute_rewards_multiple_players() {
        let env = Env::default();
        env.mock_all_auths_allowing_non_root_auth();
        let (contract_id, token_address, token_admin) = setup(&env);
        let creator = Address::generate(&env);
        let player1 = Address::generate(&env);
        let player2 = Address::generate(&env);
        let player3 = Address::generate(&env);

        mint_tokens(&env, &token_address, &token_admin, &creator, 30_000);

        env.as_contract(&contract_id, || {
            initialize_contract(&env, &token_address);
            RewardManager::create_reward_pool(env.clone(), creator.clone(), 1, 0).unwrap();
            RewardManager::fund_reward_pool(env.clone(), creator.clone(), 1, 30_000).unwrap();

            assert!(RewardManager::distribute_rewards(
                env.clone(),
                1,
                player1.clone(),
                xlm_only_config(&env, 10_000),
            )
            .is_ok());
            assert!(RewardManager::distribute_rewards(
                env.clone(),
                1,
                player2.clone(),
                xlm_only_config(&env, 10_000),
            )
            .is_ok());
            assert!(RewardManager::distribute_rewards(
                env.clone(),
                1,
                player3.clone(),
                xlm_only_config(&env, 10_000),
            )
            .is_ok());
        });

        assert_eq!(get_balance(&env, &token_address, &player1), 10_000);
        assert_eq!(get_balance(&env, &token_address, &player2), 10_000);
        assert_eq!(get_balance(&env, &token_address, &player3), 10_000);
        assert_eq!(get_balance(&env, &token_address, &contract_id), 0);

        env.as_contract(&contract_id, || {
            assert_eq!(RewardManager::get_pool_balance(env.clone(), 1), 0);

            // total_distributed should reflect all three distributions
            let status = RewardManager::get_reward_pool(env.clone(), 1).unwrap();
            assert_eq!(status.total_distributed, 30_000);
            assert_eq!(status.total_deposited, 30_000);
        });
    }

    #[test]
    fn test_get_pool_balance_after_fund_and_distribute() {
        let env = Env::default();
        env.mock_all_auths_allowing_non_root_auth();
        let (contract_id, token_address, token_admin) = setup(&env);
        let creator = Address::generate(&env);
        let player = Address::generate(&env);

        mint_tokens(&env, &token_address, &token_admin, &creator, 10_000);

        env.as_contract(&contract_id, || {
            initialize_contract(&env, &token_address);
            RewardManager::create_reward_pool(env.clone(), creator.clone(), 1, 0).unwrap();

            // Initially zero
            assert_eq!(RewardManager::get_pool_balance(env.clone(), 1), 0);

            // After funding
            RewardManager::fund_reward_pool(env.clone(), creator.clone(), 1, 8_000).unwrap();
            assert_eq!(RewardManager::get_pool_balance(env.clone(), 1), 8_000);

            // After distribution
            let config = xlm_only_config(&env, 3_000);
            RewardManager::distribute_rewards(env.clone(), 1, player.clone(), config).unwrap();
            assert_eq!(RewardManager::get_pool_balance(env.clone(), 1), 5_000);
        });
    }

    #[test]
    fn test_separate_hunt_pools() {
        let env = Env::default();
        env.mock_all_auths_allowing_non_root_auth();
        let (contract_id, token_address, token_admin) = setup(&env);
        let creator = Address::generate(&env);
        let player = Address::generate(&env);

        mint_tokens(&env, &token_address, &token_admin, &creator, 20_000);

        env.as_contract(&contract_id, || {
            initialize_contract(&env, &token_address);
            RewardManager::create_reward_pool(env.clone(), creator.clone(), 1, 0).unwrap();
            RewardManager::create_reward_pool(env.clone(), creator.clone(), 2, 0).unwrap();
            RewardManager::fund_reward_pool(env.clone(), creator.clone(), 1, 5_000).unwrap();
            RewardManager::fund_reward_pool(env.clone(), creator.clone(), 2, 10_000).unwrap();
        });

        // Verify pools are separate
        env.as_contract(&contract_id, || {
            assert_eq!(RewardManager::get_pool_balance(env.clone(), 1), 5_000);
            assert_eq!(RewardManager::get_pool_balance(env.clone(), 2), 10_000);
        });

        // Distribute from hunt 1
        env.as_contract(&contract_id, || {
            let config = xlm_only_config(&env, 3_000);
            assert!(RewardManager::distribute_rewards(
                env.clone(), 1, player.clone(), config
            )
            .is_ok());
            assert_eq!(RewardManager::get_pool_balance(env.clone(), 1), 2_000);
            assert_eq!(RewardManager::get_pool_balance(env.clone(), 2), 10_000);
        });

        // Player can still claim from hunt 2 (separate pool)
        env.as_contract(&contract_id, || {
            let config = xlm_only_config(&env, 5_000);
            assert!(RewardManager::distribute_rewards(
                env.clone(), 2, player.clone(), config
            )
            .is_ok());
            assert_eq!(RewardManager::get_pool_balance(env.clone(), 2), 5_000);
        });
    }

    #[test]
    fn test_get_distribution_status() {
        let env = Env::default();
        env.mock_all_auths_allowing_non_root_auth();
        let (contract_id, token_address, token_admin) = setup(&env);
        let creator = Address::generate(&env);
        let player = Address::generate(&env);

        mint_tokens(&env, &token_address, &token_admin, &creator, 10_000);

        env.as_contract(&contract_id, || {
            initialize_contract(&env, &token_address);
            RewardManager::create_reward_pool(env.clone(), creator.clone(), 1, 0).unwrap();
            RewardManager::fund_reward_pool(env.clone(), creator.clone(), 1, 5_000).unwrap();

            // Before distribution
            let status = RewardManager::get_distribution_status(env.clone(), 1, player.clone());
            assert!(!status.distributed);
            assert_eq!(status.xlm_amount, 0);
            assert_eq!(status.nft_id, None);

            // After distribution
            let config = xlm_only_config(&env, 2_000);
            RewardManager::distribute_rewards(env.clone(), 1, player.clone(), config).unwrap();

            let status = RewardManager::get_distribution_status(env.clone(), 1, player.clone());
            assert!(status.distributed);
            assert_eq!(status.xlm_amount, 2_000);
            assert_eq!(status.nft_id, None);
        });
    }

    #[test]
    fn test_distribute_rewards_legacy() {
        let env = Env::default();
        env.mock_all_auths_allowing_non_root_auth();
        let (contract_id, token_address, token_admin) = setup(&env);
        let creator = Address::generate(&env);
        let player = Address::generate(&env);

        mint_tokens(&env, &token_address, &token_admin, &creator, 10_000);

        env.as_contract(&contract_id, || {
            initialize_contract(&env, &token_address);
            RewardManager::create_reward_pool(env.clone(), creator.clone(), 1, 0).unwrap();
            RewardManager::fund_reward_pool(env.clone(), creator.clone(), 1, 5_000).unwrap();

            let ok = RewardManager::distribute_rewards_legacy(
                env.clone(),
                player.clone(),
                1,
                2_000,
                false,
            );
            assert!(ok);
        });

        assert_eq!(get_balance(&env, &token_address, &player), 2_000);
    }

    #[test]
    fn test_over_distribution_prevented() {
        // Verify that validate_pool correctly identifies when a pool would be over-spent
        let env = Env::default();
        env.mock_all_auths_allowing_non_root_auth();
        let (contract_id, token_address, token_admin) = setup(&env);
        let creator = Address::generate(&env);
        let player1 = Address::generate(&env);
        let player2 = Address::generate(&env);

        mint_tokens(&env, &token_address, &token_admin, &creator, 3_000);

        env.as_contract(&contract_id, || {
            initialize_contract(&env, &token_address);
            RewardManager::create_reward_pool(env.clone(), creator.clone(), 1, 0).unwrap();
            RewardManager::fund_reward_pool(env.clone(), creator.clone(), 1, 3_000).unwrap();

            // First distribution uses 2_000 — leaves 1_000
            RewardManager::distribute_rewards(
                env.clone(), 1, player1.clone(), xlm_only_config(&env, 2_000),
            )
            .unwrap();

            // validate_pool for 2_000 now fails (only 1_000 left)
            let v = RewardManager::validate_pool(env.clone(), 1, 2_000);
            assert!(!v.is_valid);
            assert_eq!(v.balance, 1_000);

            // Attempting to over-distribute also returns InsufficientPool
            let result = RewardManager::distribute_rewards(
                env.clone(), 1, player2.clone(), xlm_only_config(&env, 2_000),
            );
            assert_eq!(result, Err(RewardErrorCode::InsufficientPool));
        });

        // Only player1 received tokens
        assert_eq!(get_balance(&env, &token_address, &player1), 2_000);
        assert_eq!(get_balance(&env, &token_address, &player2), 0);
    }

    #[test]
    fn test_refund_pool_transfers_remaining_balance_to_creator() {
        let env = Env::default();
        env.mock_all_auths_allowing_non_root_auth();
        let (contract_id, token_address, token_admin) = setup(&env);
        let admin = Address::generate(&env);
        let creator = Address::generate(&env);

        mint_tokens(&env, &token_address, &token_admin, &creator, 10_000);

        env.as_contract(&contract_id, || {
            RewardManager::initialize(env.clone(), token_admin.clone(), token_address.clone());
            RewardManager::create_reward_pool(env.clone(), creator.clone(), 77, 0).unwrap();
            RewardManager::fund_reward_pool(env.clone(), creator.clone(), 77, 6_000).unwrap();
            RewardManager::refund_pool(env.clone(), creator.clone(), 77).unwrap();

            assert_eq!(RewardManager::get_pool_balance(env.clone(), 77), 0);
        });

        assert_eq!(get_balance(&env, &token_address, &creator), 10_000);
        assert_eq!(get_balance(&env, &token_address, &contract_id), 0);
    }

    #[test]
    fn test_refund_pool_unauthorized_fails() {
        let env = Env::default();
        env.mock_all_auths_allowing_non_root_auth();
        let (contract_id, token_address, token_admin) = setup(&env);
        let admin = Address::generate(&env);
        let creator = Address::generate(&env);
        let attacker = Address::generate(&env);

        mint_tokens(&env, &token_address, &token_admin, &creator, 10_000);

        env.as_contract(&contract_id, || {
            RewardManager::initialize(env.clone(), token_admin.clone(), token_address.clone());
            RewardManager::create_reward_pool(env.clone(), creator.clone(), 88, 0).unwrap();
            RewardManager::fund_reward_pool(env.clone(), creator.clone(), 88, 1_500).unwrap();

            let result = RewardManager::refund_pool(env.clone(), attacker.clone(), 88);
            assert_eq!(result, Err(RewardErrorCode::Unauthorized));
            assert_eq!(RewardManager::get_pool_balance(env.clone(), 88), 1_500);
        });
    }

    // ========== admin_withdraw_unclaimed ==========

    #[test]
    fn test_admin_withdraw_unclaimed_success() {
        let env = Env::default();
        env.mock_all_auths_allowing_non_root_auth();
        let (contract_id, token_address, token_admin) = setup(&env);
        let admin = Address::generate(&env);
        let creator = Address::generate(&env);
        let recipient = Address::generate(&env);

        // Fund creator and mint tokens
        mint_tokens(&env, &token_address, &token_admin, &creator, 10_000);

        env.as_contract(&contract_id, || {
            RewardManager::initialize(env.clone(), admin.clone(), token_address.clone()).unwrap();
            RewardManager::create_reward_pool(env.clone(), creator.clone(), 1, 0).unwrap();
            RewardManager::fund_reward_pool(env.clone(), creator.clone(), 1, 6_000).unwrap();

            // Distribute to one player, leaving 4_000 unclaimed
            let player = Address::generate(&env);
            RewardManager::distribute_rewards(env.clone(), 1, player, xlm_only_config(&env, 2_000)).unwrap();

            // Admin withdraws the remaining 4_000 to recipient
            let result = RewardManager::admin_withdraw_unclaimed(
                env.clone(),
                admin.clone(),
                1,
                recipient.clone(),
            );
            assert!(result.is_ok());

            // Pool balance should now be 0
            assert_eq!(RewardManager::get_pool_balance(env.clone(), 1), 0);
        });

        // Recipient should have received 4_000
        assert_eq!(get_balance(&env, &token_address, &recipient), 4_000);
    }

    #[test]
    fn test_admin_withdraw_unclaimed_unauthorized() {
        let env = Env::default();
        env.mock_all_auths_allowing_non_root_auth();
        let (contract_id, token_address, token_admin) = setup(&env);
        let admin = Address::generate(&env);
        let non_admin = Address::generate(&env);
        let creator = Address::generate(&env);

        mint_tokens(&env, &token_address, &token_admin, &creator, 5_000);

        env.as_contract(&contract_id, || {
            RewardManager::initialize(env.clone(), admin.clone(), token_address.clone()).unwrap();
            RewardManager::create_reward_pool(env.clone(), creator.clone(), 1, 0).unwrap();
            RewardManager::fund_reward_pool(env.clone(), creator.clone(), 1, 5_000).unwrap();

            // Non-admin tries to withdraw
            let result = RewardManager::admin_withdraw_unclaimed(
                env.clone(),
                non_admin.clone(),
                1,
                non_admin.clone(),
            );
            assert_eq!(result, Err(RewardErrorCode::Unauthorized));

            // Pool balance unchanged
            assert_eq!(RewardManager::get_pool_balance(env.clone(), 1), 5_000);
        });
    }

    #[test]
    fn test_admin_withdraw_unclaimed_pool_not_found() {
        let env = Env::default();
        env.mock_all_auths_allowing_non_root_auth();
        let (contract_id, token_address, _) = setup(&env);
        let admin = Address::generate(&env);
        let recipient = Address::generate(&env);

        env.as_contract(&contract_id, || {
            RewardManager::initialize(env.clone(), admin.clone(), token_address.clone()).unwrap();

            // No pool created for hunt_id 99
            let result = RewardManager::admin_withdraw_unclaimed(
                env.clone(),
                admin.clone(),
                99,
                recipient.clone(),
            );
            assert_eq!(result, Err(RewardErrorCode::PoolNotFound));
        });
    }

    #[test]
    fn test_admin_withdraw_unclaimed_empty_pool() {
        let env = Env::default();
        env.mock_all_auths_allowing_non_root_auth();
        let (contract_id, token_address, token_admin) = setup(&env);
        let admin = Address::generate(&env);
        let creator = Address::generate(&env);
        let player = Address::generate(&env);
        let recipient = Address::generate(&env);

        mint_tokens(&env, &token_address, &token_admin, &creator, 3_000);

        env.as_contract(&contract_id, || {
            RewardManager::initialize(env.clone(), admin.clone(), token_address.clone()).unwrap();
            RewardManager::create_reward_pool(env.clone(), creator.clone(), 1, 0).unwrap();
            RewardManager::fund_reward_pool(env.clone(), creator.clone(), 1, 3_000).unwrap();

            // Distribute all funds
            RewardManager::distribute_rewards(
                env.clone(), 1, player.clone(), xlm_only_config(&env, 3_000),
            )
            .unwrap();

            // Admin tries to withdraw from an empty pool
            let result = RewardManager::admin_withdraw_unclaimed(
                env.clone(),
                admin.clone(),
                1,
                recipient.clone(),
            );
            assert_eq!(result, Err(RewardErrorCode::InvalidAmount));
        });

        // Recipient received nothing
        assert_eq!(get_balance(&env, &token_address, &recipient), 0);
    }

    #[test]
    fn test_admin_withdraw_unclaimed_not_initialized() {
        let env = Env::default();
        env.mock_all_auths_allowing_non_root_auth();
        let (contract_id, _, _) = setup(&env);
        let admin = Address::generate(&env);
        let recipient = Address::generate(&env);

        env.as_contract(&contract_id, || {
            // Contract not initialized — no admin set
            let result = RewardManager::admin_withdraw_unclaimed(
                env.clone(),
                admin.clone(),
                1,
                recipient.clone(),
            );
            assert_eq!(result, Err(RewardErrorCode::NotInitialized));
        });
    }
}
