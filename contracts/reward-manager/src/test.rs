#[cfg(test)]
mod test {
    use crate::storage::Storage;
    use crate::types::RewardConfig;
    use crate::RewardManager;
    use soroban_sdk::testutils::{Address as _, Ledger as _};
    use soroban_sdk::{token, Address, Env};

    /// Registers the RewardManager contract and a mock SAC token.
    /// Returns (contract_id, token_address, token_admin).
    fn setup(env: &Env) -> (Address, Address, Address) {
        let contract_id = env.register(RewardManager, ());
        let token_admin = Address::generate(env);
        let token_contract = env.register_stellar_asset_contract_v2(token_admin.clone());
        let token_address = token_contract.address();
        (contract_id, token_address, token_admin)
    }

    /// Mints tokens to an address using the SAC admin.
    fn mint_tokens(env: &Env, token_address: &Address, admin: &Address, to: &Address, amount: i128) {
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
        }
    }

    #[test]
    fn test_initialize_sets_xlm_token() {
        let env = Env::default();
        let (contract_id, token_address, _) = setup(&env);

        env.as_contract(&contract_id, || {
            RewardManager::initialize(env.clone(), token_address.clone());
            assert_eq!(Storage::get_xlm_token(&env), Some(token_address.clone()));
        });
    }

    #[test]
    fn test_fund_reward_pool() {
        let env = Env::default();
        env.mock_all_auths();
        let (contract_id, token_address, token_admin) = setup(&env);
        let funder = Address::generate(&env);

        // Mint tokens to funder
        mint_tokens(&env, &token_address, &token_admin, &funder, 10_000);

        env.as_contract(&contract_id, || {
            RewardManager::initialize(env.clone(), token_address.clone());
            RewardManager::fund_reward_pool(env.clone(), funder.clone(), 1, 5_000).unwrap();
        });

        // Verify pool balance
        env.as_contract(&contract_id, || {
            assert_eq!(RewardManager::get_pool_balance(env.clone(), 1), 5_000);
        });

        // Verify tokens transferred to contract
        assert_eq!(get_balance(&env, &token_address, &contract_id), 5_000);
        assert_eq!(get_balance(&env, &token_address, &funder), 5_000);
    }

    #[test]
    fn test_fund_reward_pool_invalid_amount() {
        let env = Env::default();
        env.mock_all_auths();
        let (contract_id, token_address, _) = setup(&env);
        let funder = Address::generate(&env);

        env.as_contract(&contract_id, || {
            RewardManager::initialize(env.clone(), token_address.clone());
            let result = RewardManager::fund_reward_pool(env.clone(), funder.clone(), 1, 0);
            assert_eq!(result, Err(crate::errors::RewardErrorCode::InvalidAmount));
        });
    }

    #[test]
    fn test_fund_reward_pool_not_initialized() {
        let env = Env::default();
        env.mock_all_auths();
        let (contract_id, _, _) = setup(&env);
        let funder = Address::generate(&env);

        env.as_contract(&contract_id, || {
            let result = RewardManager::fund_reward_pool(env.clone(), funder.clone(), 1, 1000);
            assert_eq!(result, Err(crate::errors::RewardErrorCode::NotInitialized));
        });
    }

    #[test]
    fn test_distribute_rewards_success() {
        let env = Env::default();
        env.mock_all_auths();
        let (contract_id, token_address, token_admin) = setup(&env);
        let funder = Address::generate(&env);
        let player = Address::generate(&env);

        // Mint and fund
        mint_tokens(&env, &token_address, &token_admin, &funder, 10_000);

        env.as_contract(&contract_id, || {
            RewardManager::initialize(env.clone(), token_address.clone());
            RewardManager::fund_reward_pool(env.clone(), funder.clone(), 1, 5_000).unwrap();

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
            assert!(RewardManager::is_reward_distributed(
                env.clone(),
                1,
                player.clone()
            ));
        });
    }

    #[test]
    fn test_distribute_rewards_insufficient_pool() {
        let env = Env::default();
        env.mock_all_auths();
        let (contract_id, token_address, token_admin) = setup(&env);
        let funder = Address::generate(&env);
        let player = Address::generate(&env);

        mint_tokens(&env, &token_address, &token_admin, &funder, 1_000);

        env.as_contract(&contract_id, || {
            RewardManager::initialize(env.clone(), token_address.clone());
            RewardManager::fund_reward_pool(env.clone(), funder.clone(), 1, 1_000).unwrap();

            // Try to distribute more than pool has
            let config = xlm_only_config(&env, 5_000);
            let result = RewardManager::distribute_rewards(env.clone(), 1, player.clone(), config);
            assert!(result.is_err());
        });

        // Verify player didn't receive tokens
        assert_eq!(get_balance(&env, &token_address, &player), 0);
    }

    #[test]
    fn test_distribute_rewards_double_distribution() {
        let env = Env::default();
        env.mock_all_auths();
        let (contract_id, token_address, token_admin) = setup(&env);
        let funder = Address::generate(&env);
        let player = Address::generate(&env);

        mint_tokens(&env, &token_address, &token_admin, &funder, 10_000);

        env.as_contract(&contract_id, || {
            RewardManager::initialize(env.clone(), token_address.clone());
            RewardManager::fund_reward_pool(env.clone(), funder.clone(), 1, 10_000).unwrap();

            // First distribution — success
            let config1 = xlm_only_config(&env, 2_000);
            let result1 =
                RewardManager::distribute_rewards(env.clone(), 1, player.clone(), config1);
            assert!(result1.is_ok());

            // Second distribution — blocked
            let config2 = xlm_only_config(&env, 2_000);
            let result2 =
                RewardManager::distribute_rewards(env.clone(), 1, player.clone(), config2);
            assert!(result2.is_err());
        });

        // Verify player only received once
        assert_eq!(get_balance(&env, &token_address, &player), 2_000);
    }

    #[test]
    fn test_distribute_rewards_invalid_config() {
        let env = Env::default();
        env.mock_all_auths();
        let (contract_id, token_address, _) = setup(&env);
        let player = Address::generate(&env);

        env.as_contract(&contract_id, || {
            RewardManager::initialize(env.clone(), token_address.clone());

            // Empty config (no XLM, no NFT)
            let config = RewardConfig {
                xlm_amount: None,
                nft_contract: None,
                nft_title: soroban_sdk::String::from_str(&env, ""),
                nft_description: soroban_sdk::String::from_str(&env, ""),
                nft_image_uri: soroban_sdk::String::from_str(&env, ""),
            };
            let result =
                RewardManager::distribute_rewards(env.clone(), 1, player.clone(), config);
            assert_eq!(result, Err(crate::errors::RewardErrorCode::InvalidConfig));
        });
    }

    #[test]
    fn test_distribute_rewards_invalid_amount() {
        let env = Env::default();
        env.mock_all_auths();
        let (contract_id, token_address, _) = setup(&env);
        let player = Address::generate(&env);

        env.as_contract(&contract_id, || {
            RewardManager::initialize(env.clone(), token_address.clone());

            // Config with zero XLM amount is invalid (no reward types)
            let config = RewardConfig {
                xlm_amount: Some(0),
                nft_contract: None,
                nft_title: soroban_sdk::String::from_str(&env, ""),
                nft_description: soroban_sdk::String::from_str(&env, ""),
                nft_image_uri: soroban_sdk::String::from_str(&env, ""),
            };
            let result =
                RewardManager::distribute_rewards(env.clone(), 1, player.clone(), config);
            assert_eq!(result, Err(crate::errors::RewardErrorCode::InvalidConfig));
        });
    }

    #[test]
    fn test_distribute_rewards_not_initialized() {
        let env = Env::default();
        env.mock_all_auths();
        let (contract_id, _, _) = setup(&env);
        let player = Address::generate(&env);

        env.as_contract(&contract_id, || {
            let config = xlm_only_config(&env, 1_000);
            let result =
                RewardManager::distribute_rewards(env.clone(), 1, player.clone(), config);
            assert_eq!(result, Err(crate::errors::RewardErrorCode::NotInitialized));
        });
    }

    #[test]
    fn test_distribute_rewards_multiple_players() {
        let env = Env::default();
        env.mock_all_auths();
        let (contract_id, token_address, token_admin) = setup(&env);
        let funder = Address::generate(&env);
        let player1 = Address::generate(&env);
        let player2 = Address::generate(&env);
        let player3 = Address::generate(&env);

        mint_tokens(&env, &token_address, &token_admin, &funder, 30_000);

        env.as_contract(&contract_id, || {
            RewardManager::initialize(env.clone(), token_address.clone());
            RewardManager::fund_reward_pool(env.clone(), funder.clone(), 1, 30_000).unwrap();

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
        });
    }

    #[test]
    fn test_get_pool_balance_after_fund_and_distribute() {
        let env = Env::default();
        env.mock_all_auths();
        let (contract_id, token_address, token_admin) = setup(&env);
        let funder = Address::generate(&env);
        let player = Address::generate(&env);

        mint_tokens(&env, &token_address, &token_admin, &funder, 10_000);

        env.as_contract(&contract_id, || {
            RewardManager::initialize(env.clone(), token_address.clone());

            // Initially zero
            assert_eq!(RewardManager::get_pool_balance(env.clone(), 1), 0);

            // After funding
            RewardManager::fund_reward_pool(env.clone(), funder.clone(), 1, 8_000).unwrap();
            assert_eq!(RewardManager::get_pool_balance(env.clone(), 1), 8_000);

            // After distribution
            let config = xlm_only_config(&env, 3_000);
            RewardManager::distribute_rewards(env.clone(), 1, player.clone(), config).unwrap();
            assert_eq!(RewardManager::get_pool_balance(env.clone(), 1), 5_000);
        });
    }

    #[test]
    fn test_fund_reward_pool_additive() {
        let env = Env::default();
        env.mock_all_auths();
        let (contract_id, token_address, token_admin) = setup(&env);
        let funder = Address::generate(&env);

        mint_tokens(&env, &token_address, &token_admin, &funder, 20_000);

        env.as_contract(&contract_id, || {
            RewardManager::initialize(env.clone(), token_address.clone());
            RewardManager::fund_reward_pool(env.clone(), funder.clone(), 1, 5_000).unwrap();
        });

        env.mock_all_auths();
        env.as_contract(&contract_id, || {
            RewardManager::fund_reward_pool(env.clone(), funder.clone(), 1, 3_000).unwrap();
            assert_eq!(RewardManager::get_pool_balance(env.clone(), 1), 8_000);
        });

        assert_eq!(get_balance(&env, &token_address, &contract_id), 8_000);
    }

    #[test]
    fn test_separate_hunt_pools() {
        let env = Env::default();
        env.mock_all_auths();
        let (contract_id, token_address, token_admin) = setup(&env);
        let funder = Address::generate(&env);
        let player = Address::generate(&env);

        mint_tokens(&env, &token_address, &token_admin, &funder, 20_000);

        // Initialize and fund hunt 1
        env.as_contract(&contract_id, || {
            RewardManager::initialize(env.clone(), token_address.clone());
            RewardManager::fund_reward_pool(env.clone(), funder.clone(), 1, 5_000).unwrap();
        });

        // Fund hunt 2
        env.mock_all_auths();
        env.as_contract(&contract_id, || {
            RewardManager::fund_reward_pool(env.clone(), funder.clone(), 2, 10_000).unwrap();
        });

        // Verify pools are separate
        env.as_contract(&contract_id, || {
            assert_eq!(RewardManager::get_pool_balance(env.clone(), 1), 5_000);
            assert_eq!(RewardManager::get_pool_balance(env.clone(), 2), 10_000);
        });

        // Distribute from hunt 1
        env.mock_all_auths();
        env.as_contract(&contract_id, || {
            let config = xlm_only_config(&env, 3_000);
            assert!(RewardManager::distribute_rewards(
                env.clone(), 1, player.clone(), config
            )
            .is_ok());
            assert_eq!(RewardManager::get_pool_balance(env.clone(), 1), 2_000);
            assert_eq!(RewardManager::get_pool_balance(env.clone(), 2), 10_000);
        });

        // Player can still claim from hunt 2
        env.mock_all_auths();
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
        env.mock_all_auths();
        let (contract_id, token_address, token_admin) = setup(&env);
        let funder = Address::generate(&env);
        let player = Address::generate(&env);

        mint_tokens(&env, &token_address, &token_admin, &funder, 10_000);

        env.as_contract(&contract_id, || {
            RewardManager::initialize(env.clone(), token_address.clone());
            RewardManager::fund_reward_pool(env.clone(), funder.clone(), 1, 5_000).unwrap();

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
        env.mock_all_auths();
        let (contract_id, token_address, token_admin) = setup(&env);
        let funder = Address::generate(&env);
        let player = Address::generate(&env);

        mint_tokens(&env, &token_address, &token_admin, &funder, 10_000);

        env.as_contract(&contract_id, || {
            RewardManager::initialize(env.clone(), token_address.clone());
            RewardManager::fund_reward_pool(env.clone(), funder.clone(), 1, 5_000).unwrap();

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
}
