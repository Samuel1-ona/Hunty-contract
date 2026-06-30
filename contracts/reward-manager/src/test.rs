#[cfg(test)]
mod test {
    use crate::errors::RewardErrorCode;
    use crate::storage::Storage;
    use crate::types::RewardConfig;
    use crate::RewardManager;
    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::{symbol_short, token, Address, Env, IntoVal, Symbol, Val};

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
    fn mint_tokens(
        env: &Env,
        token_address: &Address,
        _admin: &Address,
        to: &Address,
        amount: i128,
    ) {
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

    fn find_event<T: TryFromVal<Env, Val>>(env: &Env, topic: Symbol) -> Option<(Vec<Val>, T)> {
        let expected_topic = topic.into_val(env);
        let events = env.events().all();
        let mut idx = 0;
        while idx < events.len() {
            let event = events.get(idx).unwrap();
            let topics = event.1.clone();
            if topics.len() > 0 && topics.get(0).unwrap() == expected_topic {
                if let Ok(data) = T::try_from_val(env, &event.2) {
                    return Some((topics, data));
                }
            }
            idx += 1;
        }
        None
    }


    fn initialize_contract(env: &Env, token_address: &Address) {
        let admin = Address::generate(&env);
        RewardManager::initialize(env.clone(), admin, token_address.clone()).unwrap();
    }

    // ========== set_pool_tiers / get_pool_config / tier resolution ==========

    use crate::TimeBasedRewardTier as _TimeBasedRewardTier;
    use crate::resolve_tier_amount as _resolve_tier_amount;

    fn make_tier(max_secs: u64, amount: i128) -> _TimeBasedRewardTier {
        _TimeBasedRewardTier { max_completion_secs: max_secs, xlm_amount: amount }
    }

    #[test]
    fn test_resolve_tier_first_fit_at_boundary() {
        let env = Env::default();
        // Tiers: <=60s => 100, <=3600s => 50, <=86400s => 25
        let tiers = Vec::from_array(
            &env,
            [make_tier(60, 100), make_tier(3_600, 50), make_tier(86_400, 25)],
        );

        // `<=` boundary exactly matches the smallest tier
        assert_eq!(_resolve_tier_amount(&tiers, 0), Some(100));
        assert_eq!(_resolve_tier_amount(&tiers, 30), Some(100));
        assert_eq!(_resolve_tier_amount(&tiers, 60), Some(100));

        // Just past the first tier -> falls into the second tier
        assert_eq!(_resolve_tier_amount(&tiers, 61), Some(50));
        assert_eq!(_resolve_tier_amount(&tiers, 3_600), Some(50));

        // Past mid-tier -> slowest tier
        assert_eq!(_resolve_tier_amount(&tiers, 3_601), Some(25));
        assert_eq!(_resolve_tier_amount(&tiers, 86_400), Some(25));

        // Past all tiers -> last (slowest) tier is the fallback
        assert_eq!(_resolve_tier_amount(&tiers, 1_000_000), Some(25));
    }

    #[test]
    fn test_resolve_tier_empty_list_returns_none() {
        let env = Env::default();
        let tiers: Vec<_TimeBasedRewardTier> = Vec::new(&env);
        assert_eq!(_resolve_tier_amount(&tiers, 100), None);
        assert_eq!(_resolve_tier_amount(&tiers, 0), None);
    }

    #[test]
    fn test_set_pool_tiers_success() {
        let env = Env::default();
        env.mock_all_auths_allowing_non_root_auth();
        let (contract_id, _, _) = setup(&env);
        let creator = Address::generate(&env);

        env.as_contract(&contract_id, || {
            RewardManager::create_reward_pool(env.clone(), creator.clone(), 7, 0).unwrap();

            let tiers = Vec::from_array(
                &env,
                [make_tier(60, 100), make_tier(3_600, 50), make_tier(86_400, 25)],
            );
            RewardManager::set_pool_tiers(env.clone(), creator.clone(), 7, tiers).unwrap();

            let cfg = RewardManager::get_pool_config(env.clone(), 7).unwrap();
            assert_eq!(cfg.time_based_tiers.len(), 3);
            assert_eq!(cfg.time_based_tiers.get(0).unwrap().xlm_amount, 100);
            assert_eq!(cfg.time_based_tiers.get(1).unwrap().xlm_amount, 50);
            assert_eq!(cfg.time_based_tiers.get(2).unwrap().xlm_amount, 25);
        });
    }

    #[test]
    fn test_set_pool_tiers_empty_disables_tiers() {
        let env = Env::default();
        env.mock_all_auths_allowing_non_root_auth();
        let (contract_id, _, _) = setup(&env);
        let creator = Address::generate(&env);

        env.as_contract(&contract_id, || {
            RewardManager::create_reward_pool(env.clone(), creator.clone(), 7, 0).unwrap();
            // Install tiers first
            RewardManager::set_pool_tiers(
                env.clone(),
                creator.clone(),
                7,
                Vec::from_array(&env, [make_tier(60, 100)]),
            )
            .unwrap();

            // Now disable by passing empty
            let empty: Vec<_TimeBasedRewardTier> = Vec::new(&env);
            RewardManager::set_pool_tiers(env.clone(), creator.clone(), 7, empty).unwrap();

            let cfg = RewardManager::get_pool_config(env.clone(), 7).unwrap();
            assert_eq!(cfg.time_based_tiers.len(), 0);
        });
    }

    #[test]
    fn test_set_pool_tiers_rejects_out_of_order() {
        let env = Env::default();
        env.mock_all_auths_allowing_non_root_auth();
        let (contract_id, _, _) = setup(&env);
        let creator = Address::generate(&env);

        env.as_contract(&contract_id, || {
            RewardManager::create_reward_pool(env.clone(), creator.clone(), 1, 0).unwrap();
            // Out-of-order (3_000 < 60): not strictly ascending
            let tiers =
                Vec::from_array(&env, [make_tier(3_000, 100), make_tier(60, 50)]);
            let err = RewardManager::set_pool_tiers(env.clone(), creator.clone(), 1, tiers)
                .unwrap_err();
            assert_eq!(err, RewardErrorCode::InvalidConfig);
        });
    }

    #[test]
    fn test_set_pool_tiers_rejects_non_positive_amount() {
        let env = Env::default();
        env.mock_all_auths_allowing_non_root_auth();
        let (contract_id, _, _) = setup(&env);
        let creator = Address::generate(&env);

        env.as_contract(&contract_id, || {
            RewardManager::create_reward_pool(env.clone(), creator.clone(), 1, 0).unwrap();
            let tiers = Vec::from_array(
                &env,
                [make_tier(60, 100), make_tier(3_600, 0)],
            );
            let err = RewardManager::set_pool_tiers(env.clone(), creator.clone(), 1, tiers)
                .unwrap_err();
            assert_eq!(err, RewardErrorCode::InvalidConfig);
        });
    }

    #[test]
    fn test_set_pool_tiers_rejects_duplicate_bound() {
        let env = Env::default();
        env.mock_all_auths_allowing_non_root_auth();
        let (contract_id, _, _) = setup(&env);
        let creator = Address::generate(&env);

        env.as_contract(&contract_id, || {
            RewardManager::create_reward_pool(env.clone(), creator.clone(), 1, 0).unwrap();
            // Equal max_completion_secs across adjacent tiers: not strictly ascending
            let tiers = Vec::from_array(
                &env,
                [make_tier(60, 100), make_tier(60, 50)],
            );
            let err = RewardManager::set_pool_tiers(env.clone(), creator.clone(), 1, tiers)
                .unwrap_err();
            assert_eq!(err, RewardErrorCode::InvalidConfig);
        });
    }

    #[test]
    fn test_set_pool_tiers_unauthorized() {
        let env = Env::default();
        env.mock_all_auths_allowing_non_root_auth();
        let (contract_id, _, _) = setup(&env);
        let creator = Address::generate(&env);
        let attacker = Address::generate(&env);

        env.as_contract(&contract_id, || {
            RewardManager::create_reward_pool(env.clone(), creator.clone(), 1, 0).unwrap();
            let tiers = Vec::from_array(&env, [make_tier(60, 100)]);
            let err = RewardManager::set_pool_tiers(env.clone(), attacker.clone(), 1, tiers)
                .unwrap_err();
            assert_eq!(err, RewardErrorCode::Unauthorized);
        });
    }

    #[test]
    fn test_set_pool_tiers_pool_not_found() {
        let env = Env::default();
        env.mock_all_auths_allowing_non_root_auth();
        let (contract_id, _, _) = setup(&env);
        let creator = Address::generate(&env);

        env.as_contract(&contract_id, || {
            let tiers = Vec::from_array(&env, [make_tier(60, 100)]);
            let err = RewardManager::set_pool_tiers(env.clone(), creator.clone(), 99, tiers)
                .unwrap_err();
            assert_eq!(err, RewardErrorCode::PoolNotFound);
        });
    }

    #[test]
    fn test_get_pool_config_returns_none_for_unknown() {
        let env = Env::default();
        env.mock_all_auths_allowing_non_root_auth();
        let (contract_id, _, _) = setup(&env);

        env.as_contract(&contract_id, || {
            assert!(RewardManager::get_pool_config(env.clone(), 999).is_none());
        });
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
            RewardManager::set_nft_reward_contract(env.clone(), admin, nft_contract.clone())
                .unwrap();
            assert_eq!(Storage::get_nft_contract(&env), Some(nft_contract));
        });
    }

    #[test]
    fn test_set_nft_reward_contract_initial_configuration() {
        let env = Env::default();
        let (contract_id, token_address, _) = setup(&env);
        let admin = Address::generate(&env);
        let nft_contract = Address::generate(&env);

        env.mock_all_auths();
        env.as_contract(&contract_id, || {
            RewardManager::initialize(env.clone(), admin.clone(), token_address).unwrap();
        });
        env.as_contract(&contract_id, || {
            assert_eq!(Storage::get_nft_contract(&env), None);
        });
        env.mock_all_auths_allowing_non_root_auth();
        env.as_contract(&contract_id, || {
            let result = RewardManager::set_nft_reward_contract(
                env.clone(),
                admin.clone(),
                nft_contract.clone(),
            );
            assert!(result.is_ok());
        });
        env.as_contract(&contract_id, || {
            assert_eq!(Storage::get_nft_contract(&env), Some(nft_contract.clone()));
        });
    }

    #[test]
    fn test_set_nft_reward_contract_update_existing() {
        let env = Env::default();
        let (contract_id, token_address, _) = setup(&env);
        let admin = Address::generate(&env);
        let nft_contract_1 = Address::generate(&env);
        let nft_contract_2 = Address::generate(&env);

        env.mock_all_auths();
        env.as_contract(&contract_id, || {
            RewardManager::initialize(env.clone(), admin.clone(), token_address).unwrap();
        });
        env.mock_all_auths_allowing_non_root_auth();
        env.as_contract(&contract_id, || {
            RewardManager::set_nft_reward_contract(
                env.clone(),
                admin.clone(),
                nft_contract_1.clone(),
            )
            .unwrap();
        });
        env.as_contract(&contract_id, || {
            assert_eq!(Storage::get_nft_contract(&env), Some(nft_contract_1.clone()));
        });
        env.mock_all_auths_allowing_non_root_auth();
        env.as_contract(&contract_id, || {
            let result = RewardManager::set_nft_reward_contract(
                env.clone(),
                admin.clone(),
                nft_contract_2.clone(),
            );
            assert!(result.is_ok());
        });
        env.as_contract(&contract_id, || {
            assert_eq!(Storage::get_nft_contract(&env), Some(nft_contract_2.clone()));
        });
    }

    #[test]
    fn test_set_nft_reward_contract_multiple_successive_updates() {
        let env = Env::default();
        let (contract_id, token_address, _) = setup(&env);
        let admin = Address::generate(&env);
        let nft_contract_1 = Address::generate(&env);
        let nft_contract_2 = Address::generate(&env);
        let nft_contract_3 = Address::generate(&env);

        env.mock_all_auths();
        env.as_contract(&contract_id, || {
            RewardManager::initialize(env.clone(), admin.clone(), token_address).unwrap();
        });
        env.mock_all_auths_allowing_non_root_auth();
        env.as_contract(&contract_id, || {
            RewardManager::set_nft_reward_contract(
                env.clone(),
                admin.clone(),
                nft_contract_1.clone(),
            )
            .unwrap();
        });
        env.as_contract(&contract_id, || {
            assert_eq!(Storage::get_nft_contract(&env), Some(nft_contract_1.clone()));
        });
        env.mock_all_auths_allowing_non_root_auth();
        env.as_contract(&contract_id, || {
            RewardManager::set_nft_reward_contract(
                env.clone(),
                admin.clone(),
                nft_contract_2.clone(),
            )
            .unwrap();
        });
        env.as_contract(&contract_id, || {
            assert_eq!(Storage::get_nft_contract(&env), Some(nft_contract_2.clone()));
        });
        env.mock_all_auths_allowing_non_root_auth();
        env.as_contract(&contract_id, || {
            RewardManager::set_nft_reward_contract(
                env.clone(),
                admin.clone(),
                nft_contract_3.clone(),
            )
            .unwrap();
        });
        env.as_contract(&contract_id, || {
            assert_eq!(Storage::get_nft_contract(&env), Some(nft_contract_3.clone()));
        });
    }

    #[test]
    fn test_set_nft_reward_contract_unauthorized_does_not_emit() {
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
            // Attempt unauthorized update should fail
            let result =
                RewardManager::set_nft_reward_contract(env.clone(), attacker, nft_contract.clone());
            assert_eq!(result, Err(RewardErrorCode::Unauthorized));

            // NFT contract should remain unset
            assert_eq!(Storage::get_nft_contract(&env), None);
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
            RewardManager::create_reward_pool(env.clone(), creator.clone(), 42, 5_000_000).unwrap();

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

    // ========== update_pool_config ==========

    #[test]
    fn test_update_pool_config_success() {
        let env = Env::default();
        env.mock_all_auths_allowing_non_root_auth();
        let (contract_id, _, _) = setup(&env);
        let creator = Address::generate(&env);

        env.as_contract(&contract_id, || {
            RewardManager::create_reward_pool(env.clone(), creator.clone(), 1, 5_000_000).unwrap();
        });
        env.mock_all_auths_allowing_non_root_auth();
        env.as_contract(&contract_id, || {
            // Lower the minimum
            RewardManager::update_pool_config(env.clone(), creator.clone(), 1, 100).unwrap();

            let status = RewardManager::get_reward_pool(env.clone(), 1).unwrap();
            assert_eq!(status.min_distribution_amount, 100);
            // Creator field must not change
            assert_eq!(status.creator, creator);
        });
    }

    #[test]
    fn test_update_pool_config_to_zero() {
        let env = Env::default();
        env.mock_all_auths_allowing_non_root_auth();
        let (contract_id, _, _) = setup(&env);
        let creator = Address::generate(&env);

        env.as_contract(&contract_id, || {
            RewardManager::create_reward_pool(env.clone(), creator.clone(), 1, 10_000_000).unwrap();
        });
        env.mock_all_auths_allowing_non_root_auth();
        env.as_contract(&contract_id, || {
            // Remove the minimum entirely
            RewardManager::update_pool_config(env.clone(), creator.clone(), 1, 0).unwrap();

            let status = RewardManager::get_reward_pool(env.clone(), 1).unwrap();
            assert_eq!(status.min_distribution_amount, 0);
        });
    }

    #[test]
    fn test_update_pool_config_unauthorized() {
        let env = Env::default();
        env.mock_all_auths_allowing_non_root_auth();
        let (contract_id, _, _) = setup(&env);
        let creator = Address::generate(&env);
        let attacker = Address::generate(&env);

        env.as_contract(&contract_id, || {
            RewardManager::create_reward_pool(env.clone(), creator.clone(), 1, 5_000_000).unwrap();

            let result = RewardManager::update_pool_config(env.clone(), attacker.clone(), 1, 100);
            assert_eq!(result, Err(RewardErrorCode::Unauthorized));

            // Original value unchanged
            let status = RewardManager::get_reward_pool(env.clone(), 1).unwrap();
            assert_eq!(status.min_distribution_amount, 500);
        });
    }

    #[test]
    fn test_update_pool_config_pool_not_found() {
        let env = Env::default();
        env.mock_all_auths_allowing_non_root_auth();
        let (contract_id, _, _) = setup(&env);
        let creator = Address::generate(&env);

        env.as_contract(&contract_id, || {
            let result = RewardManager::update_pool_config(env.clone(), creator.clone(), 99, 100);
            assert_eq!(result, Err(RewardErrorCode::PoolNotFound));
        });
    }

    #[test]
    fn test_update_pool_config_negative_amount() {
        let env = Env::default();
        env.mock_all_auths_allowing_non_root_auth();
        let (contract_id, _, _) = setup(&env);
        let creator = Address::generate(&env);

        env.as_contract(&contract_id, || {
            RewardManager::create_reward_pool(env.clone(), creator.clone(), 1, 5_000_000).unwrap();
        });
        env.mock_all_auths_allowing_non_root_auth();
        env.as_contract(&contract_id, || {
            let result = RewardManager::update_pool_config(env.clone(), creator.clone(), 1, -1);
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

        mint_tokens(&env, &token_address, &token_admin, &creator, 100_000_000);

        env.as_contract(&contract_id, || {
            initialize_contract(&env, &token_address);
            RewardManager::create_reward_pool(env.clone(), creator.clone(), 1, 0).unwrap();
            RewardManager::fund_reward_pool(env.clone(), creator.clone(), 1, 50_000_000).unwrap();
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
    fn test_fund_reward_pool_negative_amount() {
        let env = Env::default();
        env.mock_all_auths_allowing_non_root_auth();
        let (contract_id, token_address, _) = setup(&env);
        let creator = Address::generate(&env);

        env.as_contract(&contract_id, || {
            initialize_contract(&env, &token_address);
            RewardManager::create_reward_pool(env.clone(), creator.clone(), 1, 0).unwrap();
            let result = RewardManager::fund_reward_pool(env.clone(), creator.clone(), 1, -1000);
            assert_eq!(result, Err(RewardErrorCode::InvalidAmount));
        });
    }

    #[test]
    fn test_fund_reward_pool_below_minimum_dust_attack() {
        let env = Env::default();
        env.mock_all_auths_allowing_non_root_auth();
        let (contract_id, token_address, _) = setup(&env);
        let creator = Address::generate(&env);

        env.as_contract(&contract_id, || {
            initialize_contract(&env, &token_address);
            RewardManager::create_reward_pool(env.clone(), creator.clone(), 1, 0).unwrap();
            
            // Try to fund with less than 1 XLM (10_000_000 stroops)
            let result = RewardManager::fund_reward_pool(env.clone(), creator.clone(), 1, 9_999_999);
            assert_eq!(result, Err(RewardErrorCode::BelowMinimumFunding));
            
            // Also test with very small amounts
            let result = RewardManager::fund_reward_pool(env.clone(), creator.clone(), 1, 1);
            assert_eq!(result, Err(RewardErrorCode::BelowMinimumFunding));
            
            let result = RewardManager::fund_reward_pool(env.clone(), creator.clone(), 1, 100);
            assert_eq!(result, Err(RewardErrorCode::BelowMinimumFunding));
        });
    }

    #[test]
    fn test_fund_reward_pool_exactly_minimum() {
        let env = Env::default();
        env.mock_all_auths_allowing_non_root_auth();
        let (contract_id, token_address, token_admin) = setup(&env);
        let creator = Address::generate(&env);

        // Mint exactly 1 XLM (10_000_000 stroops)
        mint_tokens(&env, &token_address, &token_admin, &creator, 10_000_000);

        env.as_contract(&contract_id, || {
            initialize_contract(&env, &token_address);
            RewardManager::create_reward_pool(env.clone(), creator.clone(), 1, 0).unwrap();
            
            // Funding with exactly 1 XLM should succeed
            let result = RewardManager::fund_reward_pool(env.clone(), creator.clone(), 1, 10_000_000);
            assert!(result.is_ok());
            assert_eq!(RewardManager::get_pool_balance(env.clone(), 1), 10_000_000);
        });
    }

    #[test]
    fn test_fund_reward_pool_exceeds_maximum_single_funding() {
        let env = Env::default();
        env.mock_all_auths_allowing_non_root_auth();
        let (contract_id, token_address, _) = setup(&env);
        let creator = Address::generate(&env);

        env.as_contract(&contract_id, || {
            initialize_contract(&env, &token_address);
            RewardManager::create_reward_pool(env.clone(), creator.clone(), 1, 0).unwrap();
            
            // Try to fund with more than 1 billion XLM (1_000_000_000 * 10_000_000 stroops)
            let max_plus_one = 1_000_000_000i128 * 10_000_000 + 1;
            let result = RewardManager::fund_reward_pool(env.clone(), creator.clone(), 1, max_plus_one);
            assert_eq!(result, Err(RewardErrorCode::ExceedsMaximumFunding));
        });
    }

    #[test]
    fn test_fund_reward_pool_exactly_maximum_single_funding() {
        let env = Env::default();
        env.mock_all_auths_allowing_non_root_auth();
        let (contract_id, token_address, token_admin) = setup(&env);
        let creator = Address::generate(&env);

        // Mint exactly 1 billion XLM
        let max_amount = 1_000_000_000i128 * 10_000_000;
        mint_tokens(&env, &token_address, &token_admin, &creator, max_amount);

        env.as_contract(&contract_id, || {
            initialize_contract(&env, &token_address);
            RewardManager::create_reward_pool(env.clone(), creator.clone(), 1, 0).unwrap();
            
            // Funding with exactly 1 billion XLM should succeed
            let result = RewardManager::fund_reward_pool(env.clone(), creator.clone(), 1, max_amount);
            assert!(result.is_ok());
            assert_eq!(RewardManager::get_pool_balance(env.clone(), 1), max_amount);
        });
    }

    #[test]
    fn test_fund_reward_pool_overflow_protection() {
        let env = Env::default();
        env.mock_all_auths_allowing_non_root_auth();
        let (contract_id, token_address, token_admin) = setup(&env);
        let creator = Address::generate(&env);

        // Mint enough tokens for two large deposits
        let large_amount = 600_000_000i128 * 10_000_000; // 600 million XLM
        mint_tokens(&env, &token_address, &token_admin, &creator, large_amount * 2);

        env.as_contract(&contract_id, || {
            initialize_contract(&env, &token_address);
            RewardManager::create_reward_pool(env.clone(), creator.clone(), 1, 0).unwrap();
            
            // First funding: 600 million XLM - should succeed
            let result = RewardManager::fund_reward_pool(env.clone(), creator.clone(), 1, large_amount);
            assert!(result.is_ok());
            assert_eq!(RewardManager::get_pool_balance(env.clone(), 1), large_amount);
            
            // Second funding: another 600 million XLM - should fail (would exceed 1 billion limit)
            let result = RewardManager::fund_reward_pool(env.clone(), creator.clone(), 1, large_amount);
            assert_eq!(result, Err(RewardErrorCode::PoolBalanceOverflow));
            
            // Balance should remain at 600 million (first deposit only)
            assert_eq!(RewardManager::get_pool_balance(env.clone(), 1), large_amount);
        });
    }

    #[test]
    fn test_fund_reward_pool_multiple_deposits_under_limit() {
        let env = Env::default();
        env.mock_all_auths_allowing_non_root_auth();
        let (contract_id, token_address, token_admin) = setup(&env);
        let creator = Address::generate(&env);

        // Mint enough for multiple deposits that approach the limit
        let deposit1 = 300_000_000i128 * 10_000_000; // 300M XLM
        let deposit2 = 400_000_000i128 * 10_000_000; // 400M XLM
        let deposit3 = 299_000_000i128 * 10_000_000; // 299M XLM (total: 999M)
        let deposit4 = 1_000_000i128 * 10_000_000;    // 1M XLM (brings to 1B)
        
        mint_tokens(&env, &token_address, &token_admin, &creator, deposit1 + deposit2 + deposit3 + deposit4 + 10_000_000);

        env.as_contract(&contract_id, || {
            initialize_contract(&env, &token_address);
            RewardManager::create_reward_pool(env.clone(), creator.clone(), 1, 0).unwrap();
            
            // First deposit: 300M XLM
            RewardManager::fund_reward_pool(env.clone(), creator.clone(), 1, deposit1).unwrap();
            assert_eq!(RewardManager::get_pool_balance(env.clone(), 1), deposit1);
            
            // Second deposit: 400M XLM (total: 700M)
            RewardManager::fund_reward_pool(env.clone(), creator.clone(), 1, deposit2).unwrap();
            assert_eq!(RewardManager::get_pool_balance(env.clone(), 1), deposit1 + deposit2);
            
            // Third deposit: 299M XLM (total: 999M, still under 1 billion)
            RewardManager::fund_reward_pool(env.clone(), creator.clone(), 1, deposit3).unwrap();
            let current_balance = deposit1 + deposit2 + deposit3;
            assert_eq!(RewardManager::get_pool_balance(env.clone(), 1), current_balance);
            
            // Adding 1M XLM brings total to 1000M (exactly 1 billion) - should succeed
            RewardManager::fund_reward_pool(env.clone(), creator.clone(), 1, deposit4).unwrap();
            assert_eq!(RewardManager::get_pool_balance(env.clone(), 1), 1_000_000_000i128 * 10_000_000);
            
            // One more XLM should fail (would exceed 1 billion limit)
            let result = RewardManager::fund_reward_pool(env.clone(), creator.clone(), 1, 10_000_000);
            assert_eq!(result, Err(RewardErrorCode::PoolBalanceOverflow));
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

    /// Verifies that `fund_reward_pool` rejects any caller who is not the pool creator.
    ///
    /// A third-party address (attacker) with sufficient token balance attempts to fund a pool
    /// they did not create. The call must return `Unauthorized` and leave the attacker's
    /// balance untouched — no tokens should be transferred.
    ///
    /// Closes #195.
    #[test]
    fn test_fund_reward_pool_unauthorized_funder() {
        let env = Env::default();
        env.mock_all_auths_allowing_non_root_auth();
        let (contract_id, token_address, token_admin) = setup(&env);
        let creator = Address::generate(&env);
        let attacker = Address::generate(&env);

        mint_tokens(&env, &token_address, &token_admin, &attacker, 100_000_000);

        env.as_contract(&contract_id, || {
            initialize_contract(&env, &token_address);
            RewardManager::create_reward_pool(env.clone(), creator.clone(), 1, 0).unwrap();

            // Non-creator tries to fund
            let result = RewardManager::fund_reward_pool(env.clone(), attacker.clone(), 1, 10_000_000);
            assert_eq!(result, Err(RewardErrorCode::Unauthorized));
        });

        // Attacker's balance unchanged — no tokens were transferred
        assert_eq!(get_balance(&env, &token_address, &attacker), 100_000_000);
    }

    #[test]
    #[should_panic]
    fn test_fund_reward_pool_requires_creator_auth() {
        let env = Env::default();
        // Do NOT mock auths here to test require_auth rejection
        let (contract_id, token_address, _) = setup(&env);
        let creator = Address::generate(&env);

        env.as_contract(&contract_id, || {
            initialize_contract(&env, &token_address);
            crate::storage::Storage::set_pool_config(&env, 1, &crate::types::RewardPoolConfig {
                creator: creator.clone(),
                min_distribution_amount: 0,
                time_based_tiers: Vec::new(&env),
            });
            let _ = RewardManager::fund_reward_pool(env.clone(), creator.clone(), 1, 10_000_000);
        });
    }

    #[test]
    fn test_fund_reward_pool_additive() {
        let env = Env::default();
        env.mock_all_auths_allowing_non_root_auth();
        let (contract_id, token_address, token_admin) = setup(&env);
        let creator = Address::generate(&env);

        mint_tokens(&env, &token_address, &token_admin, &creator, 200_000_000);

        env.as_contract(&contract_id, || {
            initialize_contract(&env, &token_address);
            RewardManager::create_reward_pool(env.clone(), creator.clone(), 1, 0).unwrap();
            RewardManager::fund_reward_pool(env.clone(), creator.clone(), 1, 50_000_000).unwrap();
            RewardManager::fund_reward_pool(env.clone(), creator.clone(), 1, 30_000_000).unwrap();
            assert_eq!(RewardManager::get_pool_balance(env.clone(), 1), 80_000_000);
        });

        assert_eq!(get_balance(&env, &token_address, &contract_id), 80_000_000);
    }

    #[test]
    fn test_fund_reward_pool_updates_total_deposited() {
        let env = Env::default();
        env.mock_all_auths_allowing_non_root_auth();
        let (contract_id, token_address, token_admin) = setup(&env);
        let creator = Address::generate(&env);

        mint_tokens(&env, &token_address, &token_admin, &creator, 100_000_000);

        env.as_contract(&contract_id, || {
            initialize_contract(&env, &token_address);
            RewardManager::create_reward_pool(env.clone(), creator.clone(), 1, 0).unwrap();
            RewardManager::fund_reward_pool(env.clone(), creator.clone(), 1, 40_000_000).unwrap();
            RewardManager::fund_reward_pool(env.clone(), creator.clone(), 1, 20_000_000).unwrap();

            let status = RewardManager::get_reward_pool(env.clone(), 1).unwrap();
            assert_eq!(status.total_deposited, 60_000_000);
            assert_eq!(status.balance, 60_000_000);
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

        mint_tokens(&env, &token_address, &token_admin, &creator, 100_000_000);

        env.as_contract(&contract_id, || {
            initialize_contract(&env, &token_address);
            RewardManager::create_reward_pool(env.clone(), creator.clone(), 1, 1_000_000).unwrap();
            RewardManager::fund_reward_pool(env.clone(), creator.clone(), 1, 80_000_000).unwrap();
            RewardManager::distribute_rewards(
                env.clone(),
                1,
                player.clone(),
                xlm_only_config(&env, 30_000_000),
            )
            .unwrap();

            let status = RewardManager::get_reward_pool(env.clone(), 1).unwrap();
            assert_eq!(status.balance, 50_000_000);
            assert_eq!(status.total_deposited, 80_000_000);
            assert_eq!(status.total_distributed, 30_000_000);
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

        mint_tokens(&env, &token_address, &token_admin, &creator, 100_000_000);

        env.as_contract(&contract_id, || {
            initialize_contract(&env, &token_address);
            RewardManager::create_reward_pool(env.clone(), creator.clone(), 1, 0).unwrap();
            RewardManager::fund_reward_pool(env.clone(), creator.clone(), 1, 50_000_000).unwrap();

            let result = RewardManager::validate_pool(env.clone(), 1, 50_000_000);
            assert!(result.is_valid);
            assert_eq!(result.balance, 50_000_000);
            assert_eq!(result.required, 50_000_000);
        });
    }

    #[test]
    fn test_validate_pool_insufficient_funds() {
        let env = Env::default();
        env.mock_all_auths_allowing_non_root_auth();
        let (contract_id, token_address, token_admin) = setup(&env);
        let creator = Address::generate(&env);

        mint_tokens(&env, &token_address, &token_admin, &creator, 10_000_000);

        env.as_contract(&contract_id, || {
            initialize_contract(&env, &token_address);
            RewardManager::create_reward_pool(env.clone(), creator.clone(), 1, 0).unwrap();
            RewardManager::fund_reward_pool(env.clone(), creator.clone(), 1, 10_000_000).unwrap();

            let result = RewardManager::validate_pool(env.clone(), 1, 50_000_000);
            assert!(!result.is_valid);
            assert_eq!(result.balance, 10_000_000);
            assert_eq!(result.required, 50_000_000);
        });
    }

    #[test]
    fn test_validate_pool_below_minimum() {
        let env = Env::default();
        env.mock_all_auths_allowing_non_root_auth();
        let (contract_id, token_address, token_admin) = setup(&env);
        let creator = Address::generate(&env);

        mint_tokens(&env, &token_address, &token_admin, &creator, 100_000_000);

        env.as_contract(&contract_id, || {
            initialize_contract(&env, &token_address);
            // Pool requires minimum 5_000_000 per distribution
            RewardManager::create_reward_pool(env.clone(), creator.clone(), 1, 5_000_000).unwrap();
            RewardManager::fund_reward_pool(env.clone(), creator.clone(), 1, 50_000_000).unwrap();

            // 200 < minimum 5_000_000 → invalid even though funds are available
            let result = RewardManager::validate_pool(env.clone(), 1, 2_000_000);
            assert!(!result.is_valid);

            // 500 == minimum → valid
            let result = RewardManager::validate_pool(env.clone(), 1, 5_000_000);
            assert!(result.is_valid);
        });
    }

    #[test]
    fn test_validate_pool_not_created() {
        let env = Env::default();
        env.mock_all_auths_allowing_non_root_auth();
        let (contract_id, _, _) = setup(&env);

        env.as_contract(&contract_id, || {
            let result = RewardManager::validate_pool(env.clone(), 99, 10_000_000);
            assert!(!result.is_valid);
            assert_eq!(result.balance, 0);
            assert_eq!(result.required, 10_000_000);
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
            RewardManager::fund_reward_pool(env.clone(), creator.clone(), 1, 50_000_000).unwrap();

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

        mint_tokens(&env, &token_address, &token_admin, &creator, 100_000_000);

        env.as_contract(&contract_id, || {
            initialize_contract(&env, &token_address);
            RewardManager::create_reward_pool(env.clone(), creator.clone(), 1, 0).unwrap();
            RewardManager::fund_reward_pool(env.clone(), creator.clone(), 1, 50_000_000).unwrap();

            let config = xlm_only_config(&env, 20_000_000);
            let result = RewardManager::distribute_rewards(env.clone(), 1, player.clone(), config);
            assert!(result.is_ok());
        });

        // Verify player received tokens
        assert_eq!(get_balance(&env, &token_address, &player), 20_000_000);
        // Verify contract balance decreased
        assert_eq!(get_balance(&env, &token_address, &contract_id), 30_000_000);

        // Verify pool balance updated
        env.as_contract(&contract_id, || {
            assert_eq!(RewardManager::get_pool_balance(env.clone(), 1), 30_000_000);
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
    fn test_rewards_distributed_event_topics_and_data() {
        let env = Env::default();
        env.mock_all_auths_allowing_non_root_auth();
        let (contract_id, token_address, token_admin) = setup(&env);
        let creator = Address::generate(&env);
        let player = Address::generate(&env);

        mint_tokens(&env, &token_address, &token_admin, &creator, 100_000_000);

        env.as_contract(&contract_id, || {
            initialize_contract(&env, &token_address);
            RewardManager::create_reward_pool(env.clone(), creator.clone(), 7, 0).unwrap();
            RewardManager::fund_reward_pool(env.clone(), creator.clone(), 7, 50_000_000).unwrap();

            let config = xlm_only_config(&env, 20_000_000);
            RewardManager::distribute_rewards(env.clone(), 7, player.clone(), config).unwrap();

            let (topics, event) = find_event::<RewardsDistributedEvent>(
                &env,
                symbol_short!("RWD_DIST"),
            )
            .expect("missing rewards distribution event");
            assert_eq!(topics.len(), 2);
            assert_eq!(topics.get(0).unwrap(), symbol_short!("RWD_DIST").into_val(&env));
            assert_eq!(topics.get(1).unwrap(), 7u64.into_val(&env));
            assert_eq!(event.hunt_id, 7);
            assert_eq!(event.player, player);
            assert_eq!(event.xlm_amount, 20_000_000);
            assert_eq!(event.nft_id, None);
        });
    }

    #[test]
    fn test_distribute_rewards_insufficient_pool() {
        let env = Env::default();
        env.mock_all_auths_allowing_non_root_auth();
        let (contract_id, token_address, token_admin) = setup(&env);
        let creator = Address::generate(&env);
        let player = Address::generate(&env);

        mint_tokens(&env, &token_address, &token_admin, &creator, 10_000_000);

        env.as_contract(&contract_id, || {
            initialize_contract(&env, &token_address);
            RewardManager::create_reward_pool(env.clone(), creator.clone(), 1, 0).unwrap();
            RewardManager::fund_reward_pool(env.clone(), creator.clone(), 1, 10_000_000).unwrap();

            // Try to distribute more than pool has
            let config = xlm_only_config(&env, 50_000_000);
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
            RewardManager::create_reward_pool(env.clone(), creator.clone(), 1, 10_000_000).unwrap();
            RewardManager::fund_reward_pool(env.clone(), creator.clone(), 1, 50_000_000).unwrap();

            // Attempt to distribute 500 — below minimum of 10_000_000
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
            RewardManager::create_reward_pool(env.clone(), creator.clone(), 1, 10_000_000).unwrap();
            RewardManager::fund_reward_pool(env.clone(), creator.clone(), 1, 50_000_000).unwrap();

            // Distribute exactly the minimum
            let config = xlm_only_config(&env, 10_000_000);
            let result = RewardManager::distribute_rewards(env.clone(), 1, player.clone(), config);
            assert!(result.is_ok());
        });

        assert_eq!(get_balance(&env, &token_address, &player), 10_000_000);
    }

    #[test]
    fn test_distribute_rewards_double_distribution() {
        let env = Env::default();
        env.mock_all_auths_allowing_non_root_auth();
        let (contract_id, token_address, token_admin) = setup(&env);
        let creator = Address::generate(&env);
        let player = Address::generate(&env);

        mint_tokens(&env, &token_address, &token_admin, &creator, 100_000_000);

        env.as_contract(&contract_id, || {
            initialize_contract(&env, &token_address);
            RewardManager::create_reward_pool(env.clone(), creator.clone(), 1, 0).unwrap();
            RewardManager::fund_reward_pool(env.clone(), creator.clone(), 1, 100_000_000).unwrap();

            // First distribution — success
            let config1 = xlm_only_config(&env, 20_000_000);
            let result1 =
                RewardManager::distribute_rewards(env.clone(), 1, player.clone(), config1);
            assert!(result1.is_ok());

            // Second distribution — blocked
            let config2 = xlm_only_config(&env, 20_000_000);
            let result2 =
                RewardManager::distribute_rewards(env.clone(), 1, player.clone(), config2);
            assert_eq!(result2, Err(RewardErrorCode::AlreadyDistributed));
        });

        // Verify player only received once
        assert_eq!(get_balance(&env, &token_address, &player), 20_000_000);
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
            let result = RewardManager::distribute_rewards(env.clone(), 1, player.clone(), config);
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
            let result = RewardManager::distribute_rewards(env.clone(), 1, player.clone(), config);
            assert_eq!(result, Err(RewardErrorCode::InvalidConfig));
        });
    }

    #[test]
fn test_nft_mint_failure_does_not_block_distribution() {
    let env = Env::default();
    env.mock_all_auths_allowing_non_root_auth();
    let (contract_id, token_address, token_admin) = setup(&env);
    let creator = Address::generate(&env);
    let player = Address::generate(&env);
    let missing_nft_contract = Address::generate(&env);

    mint_tokens(&env, &token_address, &token_admin, &creator, 100_000_000);

    env.as_contract(&contract_id, || {
        initialize_contract(&env, &token_address);
        RewardManager::create_reward_pool(env.clone(), creator.clone(), 1, 0).unwrap();
        RewardManager::fund_reward_pool(env.clone(), creator.clone(), 1, 50_000_000).unwrap();

        let config = RewardConfig {
            xlm_amount: Some(20_000_000),
            nft_contract: Some(missing_nft_contract),
            nft_title: soroban_sdk::String::from_str(&env, "NFT"),
            nft_description: soroban_sdk::String::from_str(&env, "desc"),
            nft_image_uri: soroban_sdk::String::from_str(&env, "uri"),
            nft_hunt_title: soroban_sdk::String::from_str(&env, "hunt"),
            nft_rarity: 0,
            nft_tier: 0,
        };

        // Distribution should succeed even though NFT mint fails
        let result = RewardManager::distribute_rewards(env.clone(), 1, player.clone(), config);
        assert!(result.is_ok());
    });

    // Verify XLM was distributed despite NFT failure
    assert_eq!(get_balance(&env, &token_address, &player), 20_000_000);

    // Verify distribution status shows NFT mint failure
    env.as_contract(&contract_id, || {
        let status = RewardManager::get_distribution_status(env.clone(), 1, player.clone());
        assert!(status.distributed);
        assert_eq!(status.xlm_amount, 20_000_000);
        assert_eq!(status.nft_id, None);
        assert!(status.nft_mint_failed);
    });
}

#[test]
fn test_nft_only_mint_failure_logs_and_allows_retry() {
    let env = Env::default();
    env.mock_all_auths_allowing_non_root_auth();
    let (contract_id, token_address, _) = setup(&env);
    let admin = Address::generate(&env);
    let player = Address::generate(&env);
    let missing_nft_contract = Address::generate(&env);

    env.as_contract(&contract_id, || {
        RewardManager::initialize(env.clone(), admin.clone(), token_address.clone()).unwrap();

        let config = RewardConfig {
            xlm_amount: None,
            nft_contract: Some(missing_nft_contract),
            nft_title: soroban_sdk::String::from_str(&env, "NFT"),
            nft_description: soroban_sdk::String::from_str(&env, "desc"),
            nft_image_uri: soroban_sdk::String::from_str(&env, "uri"),
            nft_hunt_title: soroban_sdk::String::from_str(&env, "hunt"),
            nft_rarity: 0,
            nft_tier: 0,
        };

        // Distribution should succeed (no XLM to block on NFT failure)
        let result = RewardManager::distribute_rewards(env.clone(), 1, player.clone(), config);
        assert!(result.is_ok());
    });

    // Verify the NFT mint failed was logged
    env.as_contract(&contract_id, || {
        let status = RewardManager::get_distribution_status(env.clone(), 1, player.clone());
        assert!(status.distributed);
        assert_eq!(status.xlm_amount, 0);
        assert_eq!(status.nft_id, None);
        assert!(status.nft_mint_failed);
    });
}

#[test]
fn test_retry_failed_nft_mint_returns_not_found_when_no_pending() {
    let env = Env::default();
    env.mock_all_auths_allowing_non_root_auth();
    let (contract_id, token_address, _) = setup(&env);
    let admin = Address::generate(&env);
    let player = Address::generate(&env);

    env.as_contract(&contract_id, || {
        RewardManager::initialize(env.clone(), admin.clone(), token_address.clone()).unwrap();

        let result = RewardManager::retry_failed_nft_mint(
            env.clone(),
            admin.clone(),
            1,
            player.clone(),
        );
        assert_eq!(result, Err(RewardErrorCode::NftMintPendingNotFound));
    });
}

#[test]
fn test_retry_failed_nft_mint_rejects_unauthorized_caller() {
    let env = Env::default();
    env.mock_all_auths_allowing_non_root_auth();
    let (contract_id, token_address, _) = setup(&env);
    let admin = Address::generate(&env);
    let attacker = Address::generate(&env);
    let player = Address::generate(&env);

    env.as_contract(&contract_id, || {
        RewardManager::initialize(env.clone(), admin.clone(), token_address.clone()).unwrap();

        let result = RewardManager::retry_failed_nft_mint(
            env.clone(),
            attacker,
            1,
            player.clone(),
        );
        assert_eq!(result, Err(RewardErrorCode::Unauthorized));
    });
}

#[test]
fn test_distribute_rewards_failed_nft_creates_pending_entry() {
    let env = Env::default();
    env.mock_all_auths_allowing_non_root_auth();
    let (contract_id, token_address, _) = setup(&env);
    let admin = Address::generate(&env);
    let player = Address::generate(&env);
    let missing_nft = Address::generate(&env);

    env.as_contract(&contract_id, || {
        RewardManager::initialize(env.clone(), admin.clone(), token_address.clone()).unwrap();

        let config = RewardConfig {
            xlm_amount: None,
            nft_contract: Some(missing_nft),
            nft_title: soroban_sdk::String::from_str(&env, "NFT"),
            nft_description: soroban_sdk::String::from_str(&env, "desc"),
            nft_image_uri: soroban_sdk::String::from_str(&env, "uri"),
            nft_hunt_title: soroban_sdk::String::from_str(&env, "hunt"),
            nft_rarity: 0,
            nft_tier: 0,
        };

        // Distribution succeeds despite NFT failure
        let result = RewardManager::distribute_rewards(env.clone(), 1, player.clone(), config);
        assert!(result.is_ok());

        // Verify pending NFT mint entry was created
        let pending = Storage::get_pending_nft_mint(&env, 1, &player);
        assert!(pending.is_some());
        assert_eq!(pending.as_ref().unwrap().hunt_id, 1);
        assert_eq!(pending.as_ref().unwrap().player, player);
    });
}

    #[test]
    fn test_distribute_rewards_not_initialized() {
        let env = Env::default();
        env.mock_all_auths_allowing_non_root_auth();
        let (contract_id, _, _) = setup(&env);
        let player = Address::generate(&env);

        env.as_contract(&contract_id, || {
            let config = xlm_only_config(&env, 10_000_000);
            let result = RewardManager::distribute_rewards(env.clone(), 1, player.clone(), config);
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

        mint_tokens(&env, &token_address, &token_admin, &creator, 300_000_000);

        env.as_contract(&contract_id, || {
            initialize_contract(&env, &token_address);
            RewardManager::create_reward_pool(env.clone(), creator.clone(), 1, 0).unwrap();
            RewardManager::fund_reward_pool(env.clone(), creator.clone(), 1, 300_000_000).unwrap();

            assert!(RewardManager::distribute_rewards(
                env.clone(),
                1,
                player1.clone(),
                xlm_only_config(&env, 100_000_000),
            )
            .is_ok());
            assert!(RewardManager::distribute_rewards(
                env.clone(),
                1,
                player2.clone(),
                xlm_only_config(&env, 100_000_000),
            )
            .is_ok());
            assert!(RewardManager::distribute_rewards(
                env.clone(),
                1,
                player3.clone(),
                xlm_only_config(&env, 100_000_000),
            )
            .is_ok());
        });

        assert_eq!(get_balance(&env, &token_address, &player1), 100_000_000);
        assert_eq!(get_balance(&env, &token_address, &player2), 100_000_000);
        assert_eq!(get_balance(&env, &token_address, &player3), 100_000_000);
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

        mint_tokens(&env, &token_address, &token_admin, &creator, 100_000_000);

        env.as_contract(&contract_id, || {
            initialize_contract(&env, &token_address);
            RewardManager::create_reward_pool(env.clone(), creator.clone(), 1, 0).unwrap();

            // Initially zero
            assert_eq!(RewardManager::get_pool_balance(env.clone(), 1), 0);

            // After funding
            RewardManager::fund_reward_pool(env.clone(), creator.clone(), 1, 80_000_000).unwrap();
            assert_eq!(RewardManager::get_pool_balance(env.clone(), 1), 80_000_000);

            // After distribution
            let config = xlm_only_config(&env, 30_000_000);
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

        mint_tokens(&env, &token_address, &token_admin, &creator, 200_000_000);

        env.as_contract(&contract_id, || {
            initialize_contract(&env, &token_address);
            RewardManager::create_reward_pool(env.clone(), creator.clone(), 1, 0).unwrap();
            RewardManager::create_reward_pool(env.clone(), creator.clone(), 2, 0).unwrap();
            RewardManager::fund_reward_pool(env.clone(), creator.clone(), 1, 50_000_000).unwrap();
            RewardManager::fund_reward_pool(env.clone(), creator.clone(), 2, 100_000_000).unwrap();
        });

        // Verify pools are separate
        env.as_contract(&contract_id, || {
            assert_eq!(RewardManager::get_pool_balance(env.clone(), 1), 5_000);
            assert_eq!(RewardManager::get_pool_balance(env.clone(), 2), 100_000_000);
        });

        // Distribute from hunt 1
        env.as_contract(&contract_id, || {
            let config = xlm_only_config(&env, 30_000_000);
            assert!(
                RewardManager::distribute_rewards(env.clone(), 1, player.clone(), config).is_ok()
            );
            assert_eq!(RewardManager::get_pool_balance(env.clone(), 1), 20_000_000);
            assert_eq!(RewardManager::get_pool_balance(env.clone(), 2), 100_000_000);
        });

        // Player can still claim from hunt 2 (separate pool)
        env.as_contract(&contract_id, || {
            let config = xlm_only_config(&env, 50_000_000);
            assert!(
                RewardManager::distribute_rewards(env.clone(), 2, player.clone(), config).is_ok()
            );
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

        mint_tokens(&env, &token_address, &token_admin, &creator, 100_000_000);

        env.as_contract(&contract_id, || {
            initialize_contract(&env, &token_address);
            RewardManager::create_reward_pool(env.clone(), creator.clone(), 1, 0).unwrap();
            RewardManager::fund_reward_pool(env.clone(), creator.clone(), 1, 50_000_000).unwrap();

            // Before distribution
            let status = RewardManager::get_distribution_status(env.clone(), 1, player.clone());
            assert!(!status.distributed);
            assert_eq!(status.xlm_amount, 0);
            assert_eq!(status.nft_id, None);

            // After distribution
            let config = xlm_only_config(&env, 20_000_000);
            RewardManager::distribute_rewards(env.clone(), 1, player.clone(), config).unwrap();

            let status = RewardManager::get_distribution_status(env.clone(), 1, player.clone());
            assert!(status.distributed);
            assert_eq!(status.xlm_amount, 20_000_000);
            assert_eq!(status.nft_id, None);
        });
    }

    #[test]
    fn test_get_distribution_status_ignores_stale_bool_flag() {
        let env = Env::default();
        let (contract_id, _, _) = setup(&env);
        let player = Address::generate(&env);

        env.as_contract(&contract_id, || {
            let record = crate::types::DistributionRecord {
                xlm_amount: 2_000,
                nft_id: None,
            };
            Storage::set_distribution_record(&env, 1, &player, &record);
            Storage::set_distributed(&env, 1, &player);

            // Simulate stale state: the record remains but the separate boolean flag disappears.
            let dist_key = (symbol_short!("DIST"), 1u64, player.clone());
            env.storage().persistent().remove(&dist_key);

            let status = RewardManager::get_distribution_status(env.clone(), 1, player.clone());
            assert!(status.distributed);
            assert_eq!(status.xlm_amount, 20_000_000);
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

        mint_tokens(&env, &token_address, &token_admin, &creator, 100_000_000);

        env.as_contract(&contract_id, || {
            initialize_contract(&env, &token_address);
            RewardManager::create_reward_pool(env.clone(), creator.clone(), 1, 0).unwrap();
            RewardManager::fund_reward_pool(env.clone(), creator.clone(), 1, 50_000_000).unwrap();

            let ok = RewardManager::distribute_rewards_legacy(
                env.clone(),
                player.clone(),
                1,
                2_000,
                false,
            );
            assert!(ok);
        });

        assert_eq!(get_balance(&env, &token_address, &player), 20_000_000);
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

        mint_tokens(&env, &token_address, &token_admin, &creator, 30_000_000);

        env.as_contract(&contract_id, || {
            initialize_contract(&env, &token_address);
            RewardManager::create_reward_pool(env.clone(), creator.clone(), 1, 0).unwrap();
            RewardManager::fund_reward_pool(env.clone(), creator.clone(), 1, 30_000_000).unwrap();

            // First distribution uses 2_000 — leaves 1_000
            RewardManager::distribute_rewards(
                env.clone(),
                1,
                player1.clone(),
                xlm_only_config(&env, 20_000_000),
            )
            .unwrap();

            // validate_pool for 2_000 now fails (only 1_000 left)
            let v = RewardManager::validate_pool(env.clone(), 1, 20_000_000);
            assert!(!v.is_valid);
            assert_eq!(v.balance, 10_000_000);

            // Attempting to over-distribute also returns InsufficientPool
            let result = RewardManager::distribute_rewards(
                env.clone(),
                1,
                player2.clone(),
                xlm_only_config(&env, 20_000_000),
            );
            assert_eq!(result, Err(RewardErrorCode::InsufficientPool));
        });

        // Only player1 received tokens
        assert_eq!(get_balance(&env, &token_address, &player1), 20_000_000);
        assert_eq!(get_balance(&env, &token_address, &player2), 0);
    }

    #[test]
    fn test_refund_pool_transfers_remaining_balance_to_creator() {
        let env = Env::default();
        env.mock_all_auths_allowing_non_root_auth();
        let (contract_id, token_address, token_admin) = setup(&env);
        let creator = Address::generate(&env);

        mint_tokens(&env, &token_address, &token_admin, &creator, 100_000_000);

        env.as_contract(&contract_id, || {
            RewardManager::initialize(env.clone(), token_admin.clone(), token_address.clone())
                .unwrap();
            RewardManager::create_reward_pool(env.clone(), creator.clone(), 77, 0).unwrap();
            RewardManager::fund_reward_pool(env.clone(), creator.clone(), 77, 60_000_000).unwrap();
            RewardManager::refund_pool(env.clone(), creator.clone(), 77).unwrap();

            assert_eq!(RewardManager::get_pool_balance(env.clone(), 77), 0);
        });

        assert_eq!(get_balance(&env, &token_address, &creator), 100_000_000);
        assert_eq!(get_balance(&env, &token_address, &contract_id), 0);
    }

    #[test]
    fn test_refund_pool_unauthorized_fails() {
        let env = Env::default();
        env.mock_all_auths_allowing_non_root_auth();
        let (contract_id, token_address, token_admin) = setup(&env);
        let creator = Address::generate(&env);
        let attacker = Address::generate(&env);

        mint_tokens(&env, &token_address, &token_admin, &creator, 100_000_000);

        env.as_contract(&contract_id, || {
            RewardManager::initialize(env.clone(), token_admin.clone(), token_address.clone())
                .unwrap();
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
        mint_tokens(&env, &token_address, &token_admin, &creator, 100_000_000);

        env.as_contract(&contract_id, || {
            RewardManager::initialize(env.clone(), admin.clone(), token_address.clone()).unwrap();
            RewardManager::create_reward_pool(env.clone(), creator.clone(), 1, 0).unwrap();
            RewardManager::fund_reward_pool(env.clone(), creator.clone(), 1, 60_000_000).unwrap();

            // Distribute to one player, leaving 4_000 unclaimed
            let player = Address::generate(&env);
            RewardManager::distribute_rewards(env.clone(), 1, player, xlm_only_config(&env, 20_000_000))
                .unwrap();

            // Admin withdraws the remaining 4_000 to recipient
            let result = RewardManager::admin_withdraw_unclaimed(
                env.clone(),
                admin.clone(),
                1,
                recipient.clone(),
                0,
            );
            assert!(result.is_ok());

            // Pool balance should now be 0
            assert_eq!(RewardManager::get_pool_balance(env.clone(), 1), 0);
        });

        // Recipient should have received 4_000
        assert_eq!(get_balance(&env, &token_address, &recipient), 40_000_000);
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
            RewardManager::fund_reward_pool(env.clone(), creator.clone(), 1, 50_000_000).unwrap();

            // Non-admin tries to withdraw
            let result = RewardManager::admin_withdraw_unclaimed(
                env.clone(),
                non_admin.clone(),
                1,
                non_admin.clone(),
                0,
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
                0,
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

        mint_tokens(&env, &token_address, &token_admin, &creator, 30_000_000);

        env.as_contract(&contract_id, || {
            RewardManager::initialize(env.clone(), admin.clone(), token_address.clone()).unwrap();
            RewardManager::create_reward_pool(env.clone(), creator.clone(), 1, 0).unwrap();
            RewardManager::fund_reward_pool(env.clone(), creator.clone(), 1, 30_000_000).unwrap();

            // Distribute all funds
            RewardManager::distribute_rewards(
                env.clone(),
                1,
                player.clone(),
                xlm_only_config(&env, 30_000_000),
            )
            .unwrap();

            // Admin tries to withdraw from an empty pool
            let result = RewardManager::admin_withdraw_unclaimed(
                env.clone(),
                admin.clone(),
                1,
                recipient.clone(),
                0,
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
                0,
            );
            assert_eq!(result, Err(RewardErrorCode::NotInitialized));
        });
    }

    #[test]
    fn test_admin_withdraw_unclaimed_never_funded() {
        let env = Env::default();
        env.mock_all_auths_allowing_non_root_auth();
        let (contract_id, token_address, _token_admin) = setup(&env);
        let admin = Address::generate(&env);
        let creator = Address::generate(&env);
        let recipient = Address::generate(&env);

        env.as_contract(&contract_id, || {
            RewardManager::initialize(env.clone(), admin.clone(), token_address.clone()).unwrap();
            // Create pool with 0 initial balance and never fund it
            RewardManager::create_reward_pool(env.clone(), creator.clone(), 1, 0).unwrap();

            // Admin tries to withdraw from a pool that was never funded
            let result = RewardManager::admin_withdraw_unclaimed(
                env.clone(),
                admin.clone(),
                1,
                recipient.clone(),
                0,
            );
            assert_eq!(result, Err(RewardErrorCode::InvalidAmount));
        });

        // Recipient received nothing
        assert_eq!(get_balance(&env, &token_address, &recipient), 0);
    }

    // ========== Authorized Contracts ==========

    #[test]
    fn test_admin_adds_authorized_contract() {
        let env = Env::default();
        env.mock_all_auths_allowing_non_root_auth();
        let (contract_id, token_address, _) = setup(&env);
        let admin = Address::generate(&env);

        env.as_contract(&contract_id, || {
            RewardManager::initialize(env.clone(), admin.clone(), token_address).unwrap();
            let authorized = Address::generate(&env);
            let result = RewardManager::add_authorized_contract(
                env.clone(),
                admin.clone(),
                authorized.clone(),
            );
            assert!(result.is_ok());
            assert!(Storage::is_authorized_contract(&env, &authorized));
        });
    }

    #[test]
    fn test_non_admin_cannot_add_authorized_contract() {
        let env = Env::default();
        env.mock_all_auths_allowing_non_root_auth();
        let (contract_id, token_address, _) = setup(&env);
        let admin = Address::generate(&env);
        let attacker = Address::generate(&env);
        let authorized = Address::generate(&env);

        env.as_contract(&contract_id, || {
            RewardManager::initialize(env.clone(), admin.clone(), token_address).unwrap();
            let result = RewardManager::add_authorized_contract(
                env.clone(),
                attacker,
                authorized.clone(),
            );
            assert_eq!(result, Err(RewardErrorCode::Unauthorized));
            assert!(!Storage::is_authorized_contract(&env, &authorized));
        });
    }

    #[test]
    fn test_authorized_contract_can_call_distribute() {
        let env = Env::default();
        env.mock_all_auths_allowing_non_root_auth();
        let (contract_id, token_address, token_admin) = setup(&env);
        let admin = Address::generate(&env);
        let creator = Address::generate(&env);
        let player = Address::generate(&env);
        let authorized = Address::generate(&env);

        mint_tokens(&env, &token_address, &token_admin, &creator, 10_000);

        // Initialize the contract and set up authorized contracts
        env.as_contract(&contract_id, || {
            RewardManager::initialize(env.clone(), admin.clone(), token_address.clone()).unwrap();
            Storage::add_authorized_contract(&env, &authorized);
            RewardManager::create_reward_pool(env.clone(), creator.clone(), 1, 0).unwrap();
            RewardManager::fund_reward_pool(env.clone(), creator.clone(), 1, 5_000).unwrap();
        });

        let mut pool_balance_before = 0i128;
        env.as_contract(&contract_id, || {
            pool_balance_before = RewardManager::get_pool_balance(env.clone(), 1);
        });
        assert!(pool_balance_before >= 2_000);

        // Call distribute_rewards from the authorized contract context
        // This simulates a cross-contract call where env.caller() == authorized
        let config = xlm_only_config(&env, 2_000);
        env.as_contract(&authorized, || {
            let mut args: Vec<Val> = Vec::new(&env);
            args.push_back((1u64).into_val(&env));
            args.push_back(player.clone().into_val(&env));
            args.push_back(config.clone().into_val(&env));

            let result = env.try_invoke_contract::<(), RewardErrorCode>(
                &contract_id,
                &Symbol::new(&env, "distribute_rewards"),
                args,
            );
            assert!(result.is_ok(), "invocation should succeed");
            let inner: Result<(), RewardErrorCode> = result.unwrap();
            assert!(inner.is_ok(), "distribute_rewards should return Ok");
        });

        // Verify player received tokens
        assert_eq!(get_balance(&env, &token_address, &player), 2_000);
    }

    #[test]
    fn test_unauthorized_contract_cannot_call_distribute() {
        let env = Env::default();
        env.mock_all_auths_allowing_non_root_auth();
        let (contract_id, token_address, token_admin) = setup(&env);
        let admin = Address::generate(&env);
        let creator = Address::generate(&env);
        let player = Address::generate(&env);
        let authorized = Address::generate(&env);
        let unauthorized = Address::generate(&env);

        mint_tokens(&env, &token_address, &token_admin, &creator, 10_000);

        env.as_contract(&contract_id, || {
            RewardManager::initialize(env.clone(), admin.clone(), token_address).unwrap();
            RewardManager::create_reward_pool(env.clone(), creator.clone(), 1, 0).unwrap();
            RewardManager::fund_reward_pool(env.clone(), creator.clone(), 1, 5_000).unwrap();

            // Configure authorized contracts — only 'authorized' is allowed
            Storage::add_authorized_contract(&env, &authorized);
        });

        // Try to distribute from an unauthorized contract context
        let config = xlm_only_config(&env, 2_000);
        env.as_contract(&unauthorized, || {
            let mut args: Vec<Val> = Vec::new(&env);
            args.push_back((1u64).into_val(&env));
            args.push_back(player.clone().into_val(&env));
            args.push_back(config.clone().into_val(&env));

            let result = env.try_invoke_contract::<(), RewardErrorCode>(
                &contract_id,
                &Symbol::new(&env, "distribute_rewards"),
                args,
            );
            // The invocation should succeed but return Unauthorized
            assert!(result.is_ok(), "invocation should succeed");
            let inner: Result<(), RewardErrorCode> = result.unwrap();
            assert_eq!(inner, Err(RewardErrorCode::Unauthorized));
        });

        // Verify player received nothing
        assert_eq!(get_balance(&env, &token_address, &player), 0);
    }

    #[test]
    fn test_admin_removes_authorized_contract() {
        let env = Env::default();
        env.mock_all_auths_allowing_non_root_auth();
        let (contract_id, token_address, _) = setup(&env);
        let admin = Address::generate(&env);
        let authorized = Address::generate(&env);

        env.as_contract(&contract_id, || {
            RewardManager::initialize(env.clone(), admin.clone(), token_address).unwrap();
            Storage::add_authorized_contract(&env, &authorized);
            assert!(Storage::is_authorized_contract(&env, &authorized));

            let result = RewardManager::remove_authorized_contract(
                env.clone(),
                admin.clone(),
                authorized.clone(),
            );
            assert!(result.is_ok());
            assert!(!Storage::is_authorized_contract(&env, &authorized));
        });
    }
}
