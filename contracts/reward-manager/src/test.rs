#[cfg(test)]
mod test {
    use crate::errors::RewardErrorCode;
    use crate::storage::Storage;
    use crate::types::RewardConfig;
    use soroban_sdk::testutils::Ledger;
    use crate::{DistributionAnalytics, RewardManager, RewardsDistributedEvent};
    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::testutils::Events as _;
    use soroban_sdk::{symbol_short, token, Address, Env, IntoVal, Symbol, TryFromVal, Val, Vec};

    /// Registers the RewardManager contract and a mock SAC token.
    /// Returns (contract_id, token_address, token_admin).
    fn setup(env: &Env) -> (Address, Address, Address) {
        let contract_id = env.register(RewardManager, ());
        let token_admin = Address::generate(&env);
        let token_contract = env.register_stellar_asset_contract_v2(token_admin.clone());
        let token_address = token_contract.address();
        (contract_id, token_address, token_admin)
    }

    /// Helper function to create a reward pool with the token address.
    /// This wraps the new create_reward_pool signature for easier test updates.
    fn create_pool_with_token(
        env: &Env,
        creator: Address,
        hunt_id: u64,
        token_address: Address,
        min_distribution_amount: i128,
    ) -> Result<(), RewardErrorCode> {
        RewardManager::create_reward_pool(
            env.clone(),
            creator,
            hunt_id,
            token_address,
            min_distribution_amount,
        )
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
        let expected_topic: Val = topic.into_val(env);
        let events: Vec<(Address, Vec<Val>, Val)> = env.events().all();
        let mut idx = 0;
        while idx < events.len() {
            let event = events.get(idx).unwrap();
            let topics = event.1.clone();
            if topics.len() > 0
                && topics.get(0).unwrap().get_payload() == expected_topic.get_payload()
            {
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

    use crate::resolve_tier_amount as _resolve_tier_amount;
    use crate::TimeBasedRewardTier as _TimeBasedRewardTier;

    fn make_tier(max_secs: u64, amount: i128) -> _TimeBasedRewardTier {
        _TimeBasedRewardTier {
            max_completion_secs: max_secs,
            xlm_amount: amount,
        }
    }

    #[test]
    fn test_resolve_tier_first_fit_at_boundary() {
        let env = Env::default();
        // Tiers: <=60s => 100, <=3600s => 50, <=86400s => 25
        let tiers = Vec::from_array(
            &env,
            [
                make_tier(60, 100),
                make_tier(3_600, 50),
                make_tier(86_400, 25),
            ],
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
        let (contract_id, token_address, _) = setup(&env);
        let creator = Address::generate(&env);

        env.as_contract(&contract_id, || {
            create_pool_with_token(&env, creator.clone(), 7, token_address.clone(), 0).unwrap();
        });

        env.as_contract(&contract_id, || {
            let tiers = Vec::from_array(
                &env,
                [
                    make_tier(60, 100),
                    make_tier(3_600, 50),
                    make_tier(86_400, 25),
                ],
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
        let (contract_id, token_address, _) = setup(&env);
        let creator = Address::generate(&env);

        env.as_contract(&contract_id, || {
            create_pool_with_token(&env, creator.clone(), 7, token_address.clone(), 0).unwrap();
        });
        env.as_contract(&contract_id, || {
            // Install tiers first
            RewardManager::set_pool_tiers(
                env.clone(),
                creator.clone(),
                7,
                Vec::from_array(&env, [make_tier(60, 100)]),
            )
            .unwrap();
        });
        env.as_contract(&contract_id, || {
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
        let (contract_id, token_address, _) = setup(&env);
        let creator = Address::generate(&env);

        env.as_contract(&contract_id, || {
            create_pool_with_token(&env, creator.clone(), 1, token_address.clone(), 0).unwrap();
            // Out-of-order (3_000 < 60): not strictly ascending
            let tiers = Vec::from_array(&env, [make_tier(3_000, 100), make_tier(60, 50)]);
            let err =
                RewardManager::set_pool_tiers(env.clone(), creator.clone(), 1, tiers).unwrap_err();
            assert_eq!(err, RewardErrorCode::InvalidConfig);
        });
    }

    #[test]
    fn test_set_pool_tiers_rejects_non_positive_amount() {
        let env = Env::default();
        env.mock_all_auths_allowing_non_root_auth();
        let (contract_id, token_address, _) = setup(&env);
        let creator = Address::generate(&env);

        env.as_contract(&contract_id, || {
            create_pool_with_token(&env, creator.clone(), 1, token_address.clone(), 0).unwrap();
            let tiers = Vec::from_array(&env, [make_tier(60, 100), make_tier(3_600, 0)]);
            let err =
                RewardManager::set_pool_tiers(env.clone(), creator.clone(), 1, tiers).unwrap_err();
            assert_eq!(err, RewardErrorCode::InvalidConfig);
        });
    }

    #[test]
    fn test_set_pool_tiers_rejects_duplicate_bound() {
        let env = Env::default();
        env.mock_all_auths_allowing_non_root_auth();
        let (contract_id, token_address, _) = setup(&env);
        let creator = Address::generate(&env);

        env.as_contract(&contract_id, || {
            create_pool_with_token(&env, creator.clone(), 1, token_address.clone(), 0).unwrap();
            // Equal max_completion_secs across adjacent tiers: not strictly ascending
            let tiers = Vec::from_array(&env, [make_tier(60, 100), make_tier(60, 50)]);
            let err =
                RewardManager::set_pool_tiers(env.clone(), creator.clone(), 1, tiers).unwrap_err();
            assert_eq!(err, RewardErrorCode::InvalidConfig);
        });
    }

    #[test]
    fn test_set_pool_tiers_unauthorized() {
        let env = Env::default();
        env.mock_all_auths_allowing_non_root_auth();
        let (contract_id, token_address, _) = setup(&env);
        let creator = Address::generate(&env);
        let attacker = Address::generate(&env);

        env.as_contract(&contract_id, || {
            create_pool_with_token(&env, creator.clone(), 1, token_address.clone(), 0).unwrap();
            let tiers = Vec::from_array(&env, [make_tier(60, 100)]);
            let err =
                RewardManager::set_pool_tiers(env.clone(), attacker.clone(), 1, tiers).unwrap_err();
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
            let err =
                RewardManager::set_pool_tiers(env.clone(), creator.clone(), 99, tiers).unwrap_err();
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
            assert_eq!(
                Storage::get_nft_contract(&env),
                Some(nft_contract_1.clone())
            );
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
            assert_eq!(
                Storage::get_nft_contract(&env),
                Some(nft_contract_2.clone())
            );
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
            assert_eq!(
                Storage::get_nft_contract(&env),
                Some(nft_contract_1.clone())
            );
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
            assert_eq!(
                Storage::get_nft_contract(&env),
                Some(nft_contract_2.clone())
            );
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
            assert_eq!(
                Storage::get_nft_contract(&env),
                Some(nft_contract_3.clone())
            );
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
        let (contract_id, token_address, _) = setup(&env);
        let creator = Address::generate(&env);

        env.as_contract(&contract_id, || {
            let result = create_pool_with_token(&env, creator.clone(), 1, token_address.clone(), 0);
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
        let (contract_id, token_address, _) = setup(&env);
        let creator = Address::generate(&env);

        env.as_contract(&contract_id, || {
            create_pool_with_token(&env, creator.clone(), 42, token_address.clone(), 5_000_000)
                .unwrap();

            let status = RewardManager::get_reward_pool(env.clone(), 42).unwrap();
            assert_eq!(status.min_distribution_amount, 5_000_000);
        });
    }

    #[test]
    fn test_create_reward_pool_duplicate_fails() {
        let env = Env::default();
        env.mock_all_auths_allowing_non_root_auth();
        let (contract_id, token_address, _) = setup(&env);
        let creator = Address::generate(&env);

        env.as_contract(&contract_id, || {
            create_pool_with_token(&env, creator.clone(), 1, token_address.clone(), 0).unwrap();

            let result = create_pool_with_token(&env, creator.clone(), 1, token_address.clone(), 0);
            assert_eq!(result, Err(RewardErrorCode::PoolAlreadyExists));
        });
    }

    #[test]
    fn test_create_reward_pool_negative_minimum_fails() {
        let env = Env::default();
        env.mock_all_auths_allowing_non_root_auth();
        let (contract_id, token_address, _) = setup(&env);
        let creator = Address::generate(&env);

        env.as_contract(&contract_id, || {
            let result =
                create_pool_with_token(&env, creator.clone(), 1, token_address.clone(), -1);
            assert_eq!(result, Err(RewardErrorCode::InvalidAmount));
        });
    }

    // ========== update_pool_config ==========

    #[test]
    fn test_update_pool_config_success() {
        let env = Env::default();
        env.mock_all_auths_allowing_non_root_auth();
        let (contract_id, token_address, _) = setup(&env);
        let creator = Address::generate(&env);

        env.as_contract(&contract_id, || {
            create_pool_with_token(&env, creator.clone(), 1, token_address.clone(), 5_000_000)
                .unwrap();
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
        let (contract_id, token_address, _) = setup(&env);
        let creator = Address::generate(&env);

        env.as_contract(&contract_id, || {
            create_pool_with_token(&env, creator.clone(), 1, token_address.clone(), 10_000_000)
                .unwrap();
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
        let (contract_id, token_address, _) = setup(&env);
        let creator = Address::generate(&env);
        let attacker = Address::generate(&env);

        env.as_contract(&contract_id, || {
            create_pool_with_token(&env, creator.clone(), 1, token_address.clone(), 5_000_000)
                .unwrap();

            let result = RewardManager::update_pool_config(env.clone(), attacker.clone(), 1, 100);
            assert_eq!(result, Err(RewardErrorCode::Unauthorized));

            // Original value unchanged
            let status = RewardManager::get_reward_pool(env.clone(), 1).unwrap();
            assert_eq!(status.min_distribution_amount, 5_000_000);
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
        let (contract_id, token_address, _) = setup(&env);
        let creator = Address::generate(&env);

        env.as_contract(&contract_id, || {
            create_pool_with_token(&env, creator.clone(), 1, token_address.clone(), 5_000_000)
                .unwrap();
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
            create_pool_with_token(&env, creator.clone(), 1, token_address.clone(), 0).unwrap();
            RewardManager::fund_reward_pool(env.clone(), creator.clone(), 1, 50_000_000).unwrap();
        });

        // Verify pool balance
        env.as_contract(&contract_id, || {
            assert_eq!(RewardManager::get_pool_balance(env.clone(), 1), 50_000_000);
        });

        // Verify tokens transferred to contract
        assert_eq!(get_balance(&env, &token_address, &contract_id), 50_000_000);
        assert_eq!(get_balance(&env, &token_address, &creator), 50_000_000);
    }

    #[test]
    fn test_fund_reward_pool_invalid_amount() {
        let env = Env::default();
        env.mock_all_auths_allowing_non_root_auth();
        let (contract_id, token_address, _) = setup(&env);
        let creator = Address::generate(&env);

        env.as_contract(&contract_id, || {
            initialize_contract(&env, &token_address);
            create_pool_with_token(&env, creator.clone(), 1, token_address.clone(), 0).unwrap();
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
            create_pool_with_token(&env, creator.clone(), 1, token_address.clone(), 0).unwrap();
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
            RewardManager::create_reward_pool(env.clone(), creator.clone(), 1, token_address.clone(), 0).unwrap();

            // Try to fund with less than 1 XLM (10_000_000 stroops)
            let result =
                RewardManager::fund_reward_pool(env.clone(), creator.clone(), 1, 9_999_999);
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
            RewardManager::create_reward_pool(env.clone(), creator.clone(), 1, token_address.clone(), 0).unwrap();

            // Funding with exactly 1 XLM should succeed
            let result =
                RewardManager::fund_reward_pool(env.clone(), creator.clone(), 1, 10_000_000);
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
            RewardManager::create_reward_pool(env.clone(), creator.clone(), 1, token_address.clone(), 0).unwrap();

            // Try to fund with more than 1 billion XLM (1_000_000_000 * 10_000_000 stroops)
            let max_plus_one = 1_000_000_000i128 * 10_000_000 + 1;
            let result =
                RewardManager::fund_reward_pool(env.clone(), creator.clone(), 1, max_plus_one);
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
            RewardManager::create_reward_pool(env.clone(), creator.clone(), 1, token_address.clone(), 0).unwrap();

            // Funding with exactly 1 billion XLM should succeed
            let result =
                RewardManager::fund_reward_pool(env.clone(), creator.clone(), 1, max_amount);
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
        mint_tokens(
            &env,
            &token_address,
            &token_admin,
            &creator,
            large_amount * 2,
        );

        env.as_contract(&contract_id, || {
            initialize_contract(&env, &token_address);
            RewardManager::create_reward_pool(env.clone(), creator.clone(), 1, token_address.clone(), 0).unwrap();

            // First funding: 600 million XLM - should succeed
            let result =
                RewardManager::fund_reward_pool(env.clone(), creator.clone(), 1, large_amount);
            assert!(result.is_ok());
            assert_eq!(
                RewardManager::get_pool_balance(env.clone(), 1),
                large_amount
            );

            // Second funding: another 600 million XLM - should fail (would exceed 1 billion limit)
            let result =
                RewardManager::fund_reward_pool(env.clone(), creator.clone(), 1, large_amount);
            assert_eq!(result, Err(RewardErrorCode::PoolBalanceOverflow));

            // Balance should remain at 600 million (first deposit only)
            assert_eq!(
                RewardManager::get_pool_balance(env.clone(), 1),
                large_amount
            );
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
        let deposit4 = 1_000_000i128 * 10_000_000; // 1M XLM (brings to 1B)

        mint_tokens(
            &env,
            &token_address,
            &token_admin,
            &creator,
            deposit1 + deposit2 + deposit3 + deposit4 + 10_000_000,
        );

        env.as_contract(&contract_id, || {
            initialize_contract(&env, &token_address);
            RewardManager::create_reward_pool(env.clone(), creator.clone(), 1, token_address.clone(), 0).unwrap();

            // First deposit: 300M XLM
            RewardManager::fund_reward_pool(env.clone(), creator.clone(), 1, deposit1).unwrap();
            assert_eq!(RewardManager::get_pool_balance(env.clone(), 1), deposit1);

            // Second deposit: 400M XLM (total: 700M)
            RewardManager::fund_reward_pool(env.clone(), creator.clone(), 1, deposit2).unwrap();
            assert_eq!(
                RewardManager::get_pool_balance(env.clone(), 1),
                deposit1 + deposit2
            );

            // Third deposit: 299M XLM (total: 999M, still under 1 billion)
            RewardManager::fund_reward_pool(env.clone(), creator.clone(), 1, deposit3).unwrap();
            let current_balance = deposit1 + deposit2 + deposit3;
            assert_eq!(
                RewardManager::get_pool_balance(env.clone(), 1),
                current_balance
            );

            // Adding 1M XLM brings total to 1000M (exactly 1 billion) - should succeed
            RewardManager::fund_reward_pool(env.clone(), creator.clone(), 1, deposit4).unwrap();
            assert_eq!(
                RewardManager::get_pool_balance(env.clone(), 1),
                1_000_000_000i128 * 10_000_000
            );

            // One more XLM should fail (would exceed 1 billion limit)
            let result =
                RewardManager::fund_reward_pool(env.clone(), creator.clone(), 1, 10_000_000);
            assert_eq!(result, Err(RewardErrorCode::PoolBalanceOverflow));
        });
    }

    #[test]
    fn test_fund_reward_pool_not_initialized() {
        let env = Env::default();
        env.mock_all_auths_allowing_non_root_auth();
        let (contract_id, token_address, _) = setup(&env);
        let creator = Address::generate(&env);

        // Pool created, but amount is below minimum funding threshold
        env.as_contract(&contract_id, || {
            create_pool_with_token(&env, creator.clone(), 1, token_address.clone(), 0).unwrap();
            let result = RewardManager::fund_reward_pool(env.clone(), creator.clone(), 1, 1000);
            assert_eq!(result, Err(RewardErrorCode::BelowMinimumFunding));
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
            create_pool_with_token(&env, creator.clone(), 1, token_address.clone(), 0).unwrap();

            // Non-creator tries to fund
            let result =
                RewardManager::fund_reward_pool(env.clone(), attacker.clone(), 1, 10_000_000);
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
            crate::storage::Storage::set_pool_config(
                &env,
                1,
                &crate::types::RewardPoolConfig {
                    creator: creator.clone(),
                    delegates: Vec::new(&env),
                    min_distribution_amount: 0,
                    time_based_tiers: Vec::new(&env),
                    frozen: false,
                    token_address: token_address.clone(),
                    nft_contract: None,
                    target_amount: 0,
                    min_distribution_interval_secs: 0,
                    distribution_mode: crate::types::DistributionMode::Fixed,
                    vesting_period_secs: 0,
                    claim_deadline: 0,
                },
            );
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
            create_pool_with_token(&env, creator.clone(), 1, token_address.clone(), 0).unwrap();
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
            create_pool_with_token(&env, creator.clone(), 1, token_address.clone(), 0).unwrap();
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
            create_pool_with_token(&env, creator.clone(), 1, token_address.clone(), 1_000_000)
                .unwrap();
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
            assert_eq!(status.min_distribution_amount, 1_000_000);
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
            create_pool_with_token(&env, creator.clone(), 1, token_address.clone(), 0).unwrap();
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
            create_pool_with_token(&env, creator.clone(), 1, token_address.clone(), 0).unwrap();
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
            create_pool_with_token(&env, creator.clone(), 1, token_address.clone(), 5_000_000)
                .unwrap();
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

        mint_tokens(&env, &token_address, &token_admin, &creator, 100_000_000);

        env.as_contract(&contract_id, || {
            initialize_contract(&env, &token_address);
            create_pool_with_token(&env, creator.clone(), 1, token_address.clone(), 0).unwrap();
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
            create_pool_with_token(&env, creator.clone(), 1, token_address.clone(), 0).unwrap();
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
            create_pool_with_token(&env, creator.clone(), 7, token_address.clone(), 0).unwrap();
            RewardManager::fund_reward_pool(env.clone(), creator.clone(), 7, 50_000_000).unwrap();

            let config = xlm_only_config(&env, 20_000_000);
            RewardManager::distribute_rewards(env.clone(), 7, player.clone(), config).unwrap();

            let (topics, event) =
                find_event::<RewardsDistributedEvent>(&env, symbol_short!("RWD_DIST"))
                    .expect("missing rewards distribution event");
            assert_eq!(topics.len(), 2);
            let t0: Val = topics.get(0).unwrap();
            let t1: Val = topics.get(1).unwrap();
            let expected_t0: Val = symbol_short!("RWD_DIST").into_val(&env);
            let expected_t1: Val = 7u64.into_val(&env);
            assert_eq!(t0.get_payload(), expected_t0.get_payload());
            assert_eq!(t1.get_payload(), expected_t1.get_payload());
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
            create_pool_with_token(&env, creator.clone(), 1, token_address.clone(), 0).unwrap();
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

        mint_tokens(&env, &token_address, &token_admin, &creator, 100_000_000);

        env.as_contract(&contract_id, || {
            initialize_contract(&env, &token_address);
            // Pool requires minimum 10_000_000 per distribution
            create_pool_with_token(&env, creator.clone(), 1, token_address.clone(), 10_000_000)
                .unwrap();
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

        mint_tokens(&env, &token_address, &token_admin, &creator, 100_000_000);

        env.as_contract(&contract_id, || {
            initialize_contract(&env, &token_address);
            create_pool_with_token(&env, creator.clone(), 1, token_address.clone(), 10_000_000)
                .unwrap();
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
            create_pool_with_token(&env, creator.clone(), 1, token_address.clone(), 0).unwrap();
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
            RewardManager::create_reward_pool(env.clone(), creator.clone(), 1, token_address.clone(), 0).unwrap();
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
            create_pool_with_token(&env, admin.clone(), 1, token_address.clone(), 0).unwrap();

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
        });
        env.as_contract(&contract_id, || {
            let result =
                RewardManager::retry_failed_nft_mint(env.clone(), admin.clone(), 1, player.clone());
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

            let result =
                RewardManager::retry_failed_nft_mint(env.clone(), attacker, 1, player.clone());
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
            create_pool_with_token(&env, admin.clone(), 1, token_address.clone(), 0).unwrap();

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
    fn test_distribute_rewards_not_initialized_pool() {
        let env = Env::default();
        env.mock_all_auths_allowing_non_root_auth();
        let (contract_id, _, _) = setup(&env);
        let player = Address::generate(&env);

        env.as_contract(&contract_id, || {
            let config = xlm_only_config(&env, 10_000_000);
            let result = RewardManager::distribute_rewards(env.clone(), 1, player.clone(), config);
            assert_eq!(result, Err(RewardErrorCode::PoolNotFound));
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
            create_pool_with_token(&env, creator.clone(), 1, token_address.clone(), 0).unwrap();
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
            assert_eq!(status.total_distributed, 300_000_000);
            assert_eq!(status.total_deposited, 300_000_000);
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
            create_pool_with_token(&env, creator.clone(), 1, token_address.clone(), 0).unwrap();

            // Initially zero
            assert_eq!(RewardManager::get_pool_balance(env.clone(), 1), 0);

            // After funding
            RewardManager::fund_reward_pool(env.clone(), creator.clone(), 1, 80_000_000).unwrap();
            assert_eq!(RewardManager::get_pool_balance(env.clone(), 1), 80_000_000);

            // After distribution
            let config = xlm_only_config(&env, 30_000_000);
            RewardManager::distribute_rewards(env.clone(), 1, player.clone(), config).unwrap();
            assert_eq!(RewardManager::get_pool_balance(env.clone(), 1), 50_000_000);
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
            create_pool_with_token(&env, creator.clone(), 1, token_address.clone(), 0).unwrap();
            create_pool_with_token(&env, creator.clone(), 2, token_address.clone(), 0).unwrap();
            RewardManager::fund_reward_pool(env.clone(), creator.clone(), 1, 50_000_000).unwrap();
            RewardManager::fund_reward_pool(env.clone(), creator.clone(), 2, 100_000_000).unwrap();
        });

        // Verify pools are separate
        env.as_contract(&contract_id, || {
            assert_eq!(RewardManager::get_pool_balance(env.clone(), 1), 50_000_000);
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
            assert_eq!(RewardManager::get_pool_balance(env.clone(), 2), 50_000_000);
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
            create_pool_with_token(&env, creator.clone(), 1, token_address.clone(), 0).unwrap();
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
                xlm_amount: 20_000_000,
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
            create_pool_with_token(&env, creator.clone(), 1, token_address.clone(), 0).unwrap();
            RewardManager::fund_reward_pool(env.clone(), creator.clone(), 1, 50_000_000).unwrap();

            let ok = RewardManager::distribute_rewards_legacy(
                env.clone(),
                player.clone(),
                1,
                20_000_000,
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
            create_pool_with_token(&env, creator.clone(), 1, token_address.clone(), 0).unwrap();
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
            create_pool_with_token(&env, creator.clone(), 77, token_address.clone(), 0).unwrap();
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
            create_pool_with_token(&env, creator.clone(), 88, token_address.clone(), 0).unwrap();
            RewardManager::fund_reward_pool(env.clone(), creator.clone(), 88, 10_000_000).unwrap();

            let result = RewardManager::refund_pool(env.clone(), attacker.clone(), 88);
            assert_eq!(result, Err(RewardErrorCode::Unauthorized));
            assert_eq!(RewardManager::get_pool_balance(env.clone(), 88), 10_000_000);
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
            create_pool_with_token(&env, creator.clone(), 1, token_address.clone(), 0).unwrap();
            RewardManager::fund_reward_pool(env.clone(), creator.clone(), 1, 60_000_000).unwrap();

            // Distribute to one player, leaving 4_000 unclaimed
            let player = Address::generate(&env);
            RewardManager::distribute_rewards(
                env.clone(),
                1,
                player,
                xlm_only_config(&env, 20_000_000),
            )
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

        mint_tokens(&env, &token_address, &token_admin, &creator, 100_000_000);

        env.as_contract(&contract_id, || {
            RewardManager::initialize(env.clone(), admin.clone(), token_address.clone()).unwrap();
            create_pool_with_token(&env, creator.clone(), 1, token_address.clone(), 0).unwrap();
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
            assert_eq!(RewardManager::get_pool_balance(env.clone(), 1), 50_000_000);
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
            create_pool_with_token(&env, creator.clone(), 1, token_address.clone(), 0).unwrap();
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
            create_pool_with_token(&env, creator.clone(), 1, token_address.clone(), 0).unwrap();

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
        let authorized = Address::generate(&env);

        env.as_contract(&contract_id, || {
            RewardManager::initialize(env.clone(), admin.clone(), token_address).unwrap();
        });
        env.as_contract(&contract_id, || {
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
            let result =
                RewardManager::add_authorized_contract(env.clone(), attacker, authorized.clone());
            assert_eq!(result, Err(RewardErrorCode::Unauthorized));
            assert!(!Storage::is_authorized_contract(&env, &authorized));
        });
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
        });
        env.as_contract(&contract_id, || {
            let result = RewardManager::remove_authorized_contract(
                env.clone(),
                admin.clone(),
                authorized.clone(),
            );
            assert!(result.is_ok(), "invocation should succeed");
            assert!(!Storage::is_authorized_contract(&env, &authorized));
        });
    }

    // Ignored: soroban-sdk 22 exposes no immediate-caller API, so the
    // authorized-contract allowlist cannot reject a foreign caller from inside
    // distribute_rewards. This test documents the intended behaviour and should
    // be re-enabled once the caller is threaded through the signature.
    #[test]
    #[ignore = "authorized-caller gate needs an immediate-caller API not present in soroban-sdk 22"]
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
            RewardManager::initialize(env.clone(), admin.clone(), token_address.clone()).unwrap();
            RewardManager::create_reward_pool(env.clone(), creator.clone(), 1, token_address.clone(), 0).unwrap();
            RewardManager::fund_reward_pool(env.clone(), creator.clone(), 1, 5_000).unwrap();
            Storage::add_authorized_contract(&env, &authorized);
        });

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
            assert!(result.is_ok(), "invocation should succeed");
            let _inner: Result<(), soroban_sdk::ConversionError> = result.unwrap();
        });
    }

    /// Test get_pool_distributions with pagination
    #[test]
    fn test_get_pool_distributions_pagination() {
        let env = Env::default();
        env.mock_all_auths_allowing_non_root_auth();
        let (contract_id, token_address, token_admin) = setup(&env);
        let creator = Address::generate(&env);
        let player1 = Address::generate(&env);
        let player2 = Address::generate(&env);
        let player3 = Address::generate(&env);

        mint_tokens(&env, &token_address, &token_admin, &creator, 30_000_000);

        env.as_contract(&contract_id, || {
            initialize_contract(&env, &token_address);
            RewardManager::create_reward_pool(env.clone(), creator.clone(), 1, token_address.clone(), 0).unwrap();
            RewardManager::fund_reward_pool(env.clone(), creator.clone(), 1, 30_000_000).unwrap();
        });

        env.as_contract(&contract_id, || {
            RewardManager::distribute_rewards(
                env.clone(),
                1,
                player1.clone(),
                xlm_only_config(&env, 10_000_000),
            )
            .unwrap();
            RewardManager::distribute_rewards(
                env.clone(),
                1,
                player2.clone(),
                xlm_only_config(&env, 10_000_000),
            )
            .unwrap();
            RewardManager::distribute_rewards(
                env.clone(),
                1,
                player3.clone(),
                xlm_only_config(&env, 10_000_000),
            )
            .unwrap();
        });

        env.as_contract(&contract_id, || {
            let count = RewardManager::get_pool_distribution_count(env.clone(), 1);
            assert_eq!(count, 3);

            let page1 = RewardManager::get_pool_distributions(env.clone(), 1, 0, 2);
            assert_eq!(page1.len(), 2);
            assert_eq!(page1.get(0).unwrap().player, player1);
            assert_eq!(page1.get(0).unwrap().xlm_amount, 10_000_000);
            assert_eq!(page1.get(1).unwrap().player, player2);
            assert_eq!(page1.get(1).unwrap().xlm_amount, 10_000_000);

            let page2 = RewardManager::get_pool_distributions(env.clone(), 1, 2, 2);
            assert_eq!(page2.len(), 1);
            assert_eq!(page2.get(0).unwrap().player, player3);
            assert_eq!(page2.get(0).unwrap().xlm_amount, 10_000_000);

            let page3 = RewardManager::get_pool_distributions(env.clone(), 1, 10, 2);
            assert_eq!(page3.len(), 0);

            let all = RewardManager::get_pool_distributions(env.clone(), 1, 0, 100);
            assert_eq!(all.len(), 3);
        });
    }

    // ========== get_pool_statistics ==========

    #[test]
    fn test_get_pool_statistics_returns_none_for_unknown_pool() {
        let env = Env::default();
        env.mock_all_auths_allowing_non_root_auth();
        let (contract_id, _, _) = setup(&env);

        env.as_contract(&contract_id, || {
            assert!(RewardManager::get_pool_statistics(env.clone(), 99).is_none());
        });
    }

    #[test]
    fn test_get_pool_statistics_after_creation() {
        let env = Env::default();
        env.mock_all_auths_allowing_non_root_auth();
        let (contract_id, token_address, _) = setup(&env);
        let creator = Address::generate(&env);

        env.as_contract(&contract_id, || {
            RewardManager::create_reward_pool(env.clone(), creator.clone(), 1, token_address.clone(), 0).unwrap();

            let stats = RewardManager::get_pool_statistics(env.clone(), 1).unwrap();
            assert_eq!(stats.total_funded, 0);
            assert_eq!(stats.total_distributed, 0);
            assert_eq!(stats.distribution_count, 0);
            assert_eq!(stats.avg_distribution, 0);
            assert_eq!(stats.last_distribution_timestamp, 0);
        });
    }

    #[test]
    fn test_get_pool_statistics_after_funding() {
        let env = Env::default();
        env.mock_all_auths_allowing_non_root_auth();
        let (contract_id, token_address, token_admin) = setup(&env);
        let creator = Address::generate(&env);

        mint_tokens(&env, &token_address, &token_admin, &creator, 100_000_000);

        env.as_contract(&contract_id, || {
            initialize_contract(&env, &token_address);
            RewardManager::create_reward_pool(env.clone(), creator.clone(), 1, token_address.clone(), 0).unwrap();
            RewardManager::fund_reward_pool(env.clone(), creator.clone(), 1, 80_000_000).unwrap();

            let stats = RewardManager::get_pool_statistics(env.clone(), 1).unwrap();
            assert_eq!(stats.total_funded, 80_000_000);
            assert_eq!(stats.total_distributed, 0);
            assert_eq!(stats.distribution_count, 0);
            assert_eq!(stats.avg_distribution, 0);
            assert_eq!(stats.last_distribution_timestamp, 0);
        });
    }

    #[test]
    fn test_get_pool_statistics_after_distributions() {
        let env = Env::default();
        env.mock_all_auths_allowing_non_root_auth();
        let (contract_id, token_address, token_admin) = setup(&env);
        let creator = Address::generate(&env);
        let player1 = Address::generate(&env);
        let player2 = Address::generate(&env);

        mint_tokens(&env, &token_address, &token_admin, &creator, 200_000_000);

        env.ledger().set_timestamp(100);

        env.as_contract(&contract_id, || {
            initialize_contract(&env, &token_address);
            RewardManager::create_reward_pool(env.clone(), creator.clone(), 1, token_address.clone(), 0).unwrap();
            RewardManager::fund_reward_pool(env.clone(), creator.clone(), 1, 100_000_000).unwrap();

            // Distribute to player1
            RewardManager::distribute_rewards(
                env.clone(),
                1,
                player1.clone(),
                xlm_only_config(&env, 30_000_000),
            )
            .unwrap();

            let stats = RewardManager::get_pool_statistics(env.clone(), 1).unwrap();
            assert_eq!(stats.total_funded, 100_000_000);
            assert_eq!(stats.total_distributed, 30_000_000);
            assert_eq!(stats.distribution_count, 1);
            assert_eq!(stats.avg_distribution, 30_000_000);
            assert!(stats.last_distribution_timestamp > 0);

            let ts_after_first = stats.last_distribution_timestamp;

            // Distribute to player2
            RewardManager::distribute_rewards(
                env.clone(),
                1,
                player2.clone(),
                xlm_only_config(&env, 20_000_000),
            )
            .unwrap();

            let stats = RewardManager::get_pool_statistics(env.clone(), 1).unwrap();
            assert_eq!(stats.total_funded, 100_000_000);
            assert_eq!(stats.total_distributed, 50_000_000);
            assert_eq!(stats.distribution_count, 2);
            assert_eq!(stats.avg_distribution, 25_000_000);
            assert!(stats.last_distribution_timestamp >= ts_after_first);
        });
    }

    #[test]
    fn test_get_pool_statistics_zero_distributions_avg() {
        let env = Env::default();
        env.mock_all_auths_allowing_non_root_auth();
        let (contract_id, token_address, token_admin) = setup(&env);
        let creator = Address::generate(&env);

        mint_tokens(&env, &token_address, &token_admin, &creator, 100_000_000);

        env.as_contract(&contract_id, || {
            initialize_contract(&env, &token_address);
            RewardManager::create_reward_pool(env.clone(), creator.clone(), 1, token_address.clone(), 0).unwrap();
            RewardManager::fund_reward_pool(env.clone(), creator.clone(), 1, 50_000_000).unwrap();

            let stats = RewardManager::get_pool_statistics(env.clone(), 1).unwrap();
            // No distributions yet: avg should be 0, not a division error
            assert_eq!(stats.distribution_count, 0);
            assert_eq!(stats.avg_distribution, 0);
        });
    }

    #[test]
    fn test_get_pool_distributions_empty() {
        let env = Env::default();
        env.mock_all_auths_allowing_non_root_auth();
        let (contract_id, token_address, _token_admin) = setup(&env);
        let creator = Address::generate(&env);

        env.as_contract(&contract_id, || {
            initialize_contract(&env, &token_address);
            RewardManager::create_reward_pool(env.clone(), creator.clone(), 1, token_address.clone(), 0).unwrap();
        });

        env.as_contract(&contract_id, || {
            // Empty pool should return 0 count and empty list
            let count = RewardManager::get_pool_distribution_count(env.clone(), 1);
            assert_eq!(count, 0);

            let distributions = RewardManager::get_pool_distributions(env.clone(), 1, 0, 10);
            assert_eq!(distributions.len(), 0);
        });
    }

    // ========== migrate_pool ==========

    /// Minimal stand-in for HuntyCore used to drive migrate_pool's eligibility
    /// check. Eligibility is set per hunt_id via `set_eligible`.
    #[soroban_sdk::contract]
    pub struct MockHuntyCore;

    #[soroban_sdk::contractimpl]
    impl MockHuntyCore {
        pub fn set_eligible(env: Env, hunt_id: u64, eligible: bool) {
            env.storage().persistent().set(&hunt_id, &eligible);
        }

        pub fn is_hunt_expired_or_cancelled(env: Env, hunt_id: u64) -> bool {
            env.storage().persistent().get(&hunt_id).unwrap_or(false)
        }
    }

    /// Registers a MockHuntyCore and marks `hunt_id` as expired/cancelled.
    fn setup_hunty_core(env: &Env, hunt_id: u64, eligible: bool) -> Address {
        let hunty_core_id = env.register(MockHuntyCore, ());
        let client = MockHuntyCoreClient::new(env, &hunty_core_id);
        client.set_eligible(&hunt_id, &eligible);
        hunty_core_id
    }

    #[test]
    fn test_migrate_pool_success() {
        let env = Env::default();
        env.mock_all_auths_allowing_non_root_auth();
        let (contract_id, token_address, token_admin) = setup(&env);
        let admin = Address::generate(&env);
        let creator = Address::generate(&env);

        mint_tokens(&env, &token_address, &token_admin, &creator, 100_000_000);
        // Source hunt (1) is expired/cancelled and therefore migratable.
        let hunty_core_id = setup_hunty_core(&env, 1, true);

        env.as_contract(&contract_id, || {
            RewardManager::initialize(env.clone(), admin.clone(), token_address.clone()).unwrap();
            // Create + fund the source and create the destination BEFORE wiring
            // HuntyCore, so pool creation does not perform hunt-existence checks.
            RewardManager::create_reward_pool(env.clone(), creator.clone(), 1, token_address.clone(), 0).unwrap();
            RewardManager::create_reward_pool(env.clone(), creator.clone(), 2, token_address.clone(), 0).unwrap();
            RewardManager::fund_reward_pool(env.clone(), creator.clone(), 1, 60_000_000).unwrap();

            Storage::set_hunty_core(&env, &hunty_core_id);

            let migrated = RewardManager::migrate_pool(env.clone(), creator.clone(), 1, 2).unwrap();
            assert_eq!(migrated, 60_000_000);

            // Source drained, destination credited.
            assert_eq!(RewardManager::get_pool_balance(env.clone(), 1), 0);
            assert_eq!(RewardManager::get_pool_balance(env.clone(), 2), 60_000_000);

            let dest = RewardManager::get_reward_pool(env.clone(), 2).unwrap();
            assert_eq!(dest.balance, 60_000_000);
            assert_eq!(dest.total_deposited, 60_000_000);
        });
    }

    #[test]
    fn test_migrate_pool_credits_existing_destination_balance() {
        let env = Env::default();
        env.mock_all_auths_allowing_non_root_auth();
        let (contract_id, token_address, token_admin) = setup(&env);
        let admin = Address::generate(&env);
        let creator = Address::generate(&env);

        mint_tokens(&env, &token_address, &token_admin, &creator, 100_000_000);
        let hunty_core_id = setup_hunty_core(&env, 1, true);

        env.as_contract(&contract_id, || {
            RewardManager::initialize(env.clone(), admin.clone(), token_address.clone()).unwrap();
            RewardManager::create_reward_pool(env.clone(), creator.clone(), 1, token_address.clone(), 0).unwrap();
            RewardManager::create_reward_pool(env.clone(), creator.clone(), 2, token_address.clone(), 0).unwrap();
            RewardManager::fund_reward_pool(env.clone(), creator.clone(), 1, 30_000_000).unwrap();
            RewardManager::fund_reward_pool(env.clone(), creator.clone(), 2, 25_000_000).unwrap();

            Storage::set_hunty_core(&env, &hunty_core_id);

            let migrated = RewardManager::migrate_pool(env.clone(), creator.clone(), 1, 2).unwrap();
            assert_eq!(migrated, 30_000_000);
            assert_eq!(RewardManager::get_pool_balance(env.clone(), 1), 0);
            // Destination keeps its own funds and gains the migrated amount.
            assert_eq!(RewardManager::get_pool_balance(env.clone(), 2), 55_000_000);

            let dest = RewardManager::get_reward_pool(env.clone(), 2).unwrap();
            assert_eq!(dest.total_deposited, 55_000_000);
        });
    }

    #[test]
    fn test_migrate_pool_source_not_eligible() {
        let env = Env::default();
        env.mock_all_auths_allowing_non_root_auth();
        let (contract_id, token_address, token_admin) = setup(&env);
        let admin = Address::generate(&env);
        let creator = Address::generate(&env);

        mint_tokens(&env, &token_address, &token_admin, &creator, 100_000_000);
        // Source hunt is NOT expired/cancelled.
        let hunty_core_id = setup_hunty_core(&env, 1, false);

        env.as_contract(&contract_id, || {
            RewardManager::initialize(env.clone(), admin.clone(), token_address.clone()).unwrap();
            RewardManager::create_reward_pool(env.clone(), creator.clone(), 1, token_address.clone(), 0).unwrap();
            RewardManager::create_reward_pool(env.clone(), creator.clone(), 2, token_address.clone(), 0).unwrap();
            RewardManager::fund_reward_pool(env.clone(), creator.clone(), 1, 60_000_000).unwrap();
            Storage::set_hunty_core(&env, &hunty_core_id);

            let result = RewardManager::migrate_pool(env.clone(), creator.clone(), 1, 2);
            assert_eq!(result, Err(RewardErrorCode::SourcePoolNotEligible));
            // Balances unchanged on failure.
            assert_eq!(RewardManager::get_pool_balance(env.clone(), 1), 60_000_000);
            assert_eq!(RewardManager::get_pool_balance(env.clone(), 2), 0);
        });
    }

    #[test]
    fn test_migrate_pool_without_hunty_core_is_not_eligible() {
        let env = Env::default();
        env.mock_all_auths_allowing_non_root_auth();
        let (contract_id, token_address, token_admin) = setup(&env);
        let admin = Address::generate(&env);
        let creator = Address::generate(&env);

        mint_tokens(&env, &token_address, &token_admin, &creator, 100_000_000);

        env.as_contract(&contract_id, || {
            RewardManager::initialize(env.clone(), admin.clone(), token_address.clone()).unwrap();
            RewardManager::create_reward_pool(env.clone(), creator.clone(), 1, token_address.clone(), 0).unwrap();
            RewardManager::create_reward_pool(env.clone(), creator.clone(), 2, token_address.clone(), 0).unwrap();
            RewardManager::fund_reward_pool(env.clone(), creator.clone(), 1, 60_000_000).unwrap();

            // HuntyCore not configured — eligibility cannot be proven.
            let result = RewardManager::migrate_pool(env.clone(), creator.clone(), 1, 2);
            assert_eq!(result, Err(RewardErrorCode::SourcePoolNotEligible));
        });
    }

    #[test]
    fn test_migrate_pool_destination_missing() {
        let env = Env::default();
        env.mock_all_auths_allowing_non_root_auth();
        let (contract_id, token_address, token_admin) = setup(&env);
        let admin = Address::generate(&env);
        let creator = Address::generate(&env);

        mint_tokens(&env, &token_address, &token_admin, &creator, 100_000_000);
        let hunty_core_id = setup_hunty_core(&env, 1, true);

        env.as_contract(&contract_id, || {
            RewardManager::initialize(env.clone(), admin.clone(), token_address.clone()).unwrap();
            RewardManager::create_reward_pool(env.clone(), creator.clone(), 1, token_address.clone(), 0).unwrap();
            RewardManager::fund_reward_pool(env.clone(), creator.clone(), 1, 60_000_000).unwrap();
            Storage::set_hunty_core(&env, &hunty_core_id);

            // Destination pool (2) was never created.
            let result = RewardManager::migrate_pool(env.clone(), creator.clone(), 1, 2);
            assert_eq!(result, Err(RewardErrorCode::DestinationPoolNotFound));
        });
    }

    #[test]
    fn test_migrate_pool_source_missing() {
        let env = Env::default();
        env.mock_all_auths_allowing_non_root_auth();
        let (contract_id, token_address, _) = setup(&env);
        let admin = Address::generate(&env);
        let creator = Address::generate(&env);

        env.as_contract(&contract_id, || {
            RewardManager::initialize(env.clone(), admin.clone(), token_address.clone()).unwrap();
            RewardManager::create_reward_pool(env.clone(), creator.clone(), 2, token_address.clone(), 0).unwrap();

            let result = RewardManager::migrate_pool(env.clone(), creator.clone(), 1, 2);
            assert_eq!(result, Err(RewardErrorCode::PoolNotFound));
        });
    }

    #[test]
    fn test_migrate_pool_different_creator_unauthorized() {
        let env = Env::default();
        env.mock_all_auths_allowing_non_root_auth();
        let (contract_id, token_address, token_admin) = setup(&env);
        let admin = Address::generate(&env);
        let creator = Address::generate(&env);
        let other = Address::generate(&env);

        mint_tokens(&env, &token_address, &token_admin, &creator, 100_000_000);

        env.as_contract(&contract_id, || {
            RewardManager::initialize(env.clone(), admin.clone(), token_address.clone()).unwrap();
            RewardManager::create_reward_pool(env.clone(), creator.clone(), 1, token_address.clone(), 0).unwrap();
            RewardManager::fund_reward_pool(env.clone(), creator.clone(), 1, 60_000_000).unwrap();
            // Destination is owned by a different creator.
            RewardManager::create_reward_pool(env.clone(), other.clone(), 2, token_address.clone(), 0).unwrap();

            let result = RewardManager::migrate_pool(env.clone(), creator.clone(), 1, 2);
            assert_eq!(result, Err(RewardErrorCode::Unauthorized));
        });
    }

    #[test]
    fn test_migrate_pool_same_hunt_rejected() {
        let env = Env::default();
        env.mock_all_auths_allowing_non_root_auth();
        let (contract_id, token_address, token_admin) = setup(&env);
        let admin = Address::generate(&env);
        let creator = Address::generate(&env);

        mint_tokens(&env, &token_address, &token_admin, &creator, 100_000_000);

        env.as_contract(&contract_id, || {
            RewardManager::initialize(env.clone(), admin.clone(), token_address.clone()).unwrap();
            RewardManager::create_reward_pool(env.clone(), creator.clone(), 1, token_address.clone(), 0).unwrap();
            RewardManager::fund_reward_pool(env.clone(), creator.clone(), 1, 60_000_000).unwrap();

            let result = RewardManager::migrate_pool(env.clone(), creator.clone(), 1, 1);
            assert_eq!(result, Err(RewardErrorCode::InvalidMigration));
        });
    }

    #[test]
    fn test_migrate_pool_zero_balance_rejected() {
        let env = Env::default();
        env.mock_all_auths_allowing_non_root_auth();
        let (contract_id, token_address, _) = setup(&env);
        let admin = Address::generate(&env);
        let creator = Address::generate(&env);
        let hunty_core_id = setup_hunty_core(&env, 1, true);

        env.as_contract(&contract_id, || {
            RewardManager::initialize(env.clone(), admin.clone(), token_address.clone()).unwrap();
            // Source created but never funded.
            RewardManager::create_reward_pool(env.clone(), creator.clone(), 1, token_address.clone(), 0).unwrap();
            RewardManager::create_reward_pool(env.clone(), creator.clone(), 2, token_address.clone(), 0).unwrap();
            Storage::set_hunty_core(&env, &hunty_core_id);

            let result = RewardManager::migrate_pool(env.clone(), creator.clone(), 1, 2);
            assert_eq!(result, Err(RewardErrorCode::InvalidMigration));
        });
    }

    // ========== get_distribution_analytics ==========

    #[test]
    fn test_get_distribution_analytics_empty_pool() {
        let env = Env::default();
        env.mock_all_auths_allowing_non_root_auth();
        let (contract_id, token_address, _token_admin) = setup(&env);
        let creator = Address::generate(&env);

        env.as_contract(&contract_id, || {
            initialize_contract(&env, &token_address);
            RewardManager::create_reward_pool(env.clone(), creator.clone(), 1, token_address.clone(), 0).unwrap();

            let analytics = RewardManager::get_distribution_analytics(
                env.clone(),
                1,
                None,
                None,
            );
            assert_eq!(analytics.count, 0);
            assert_eq!(analytics.total, 0);
            assert_eq!(analytics.average, 0);
            assert_eq!(analytics.median, 0);
            assert_eq!(analytics.min, 0);
            assert_eq!(analytics.max, 0);
        });
    }

    #[test]
    fn test_get_distribution_analytics_single_distribution() {
        let env = Env::default();
        env.mock_all_auths_allowing_non_root_auth();
        let (contract_id, token_address, token_admin) = setup(&env);
        let creator = Address::generate(&env);
        let player = Address::generate(&env);

        mint_tokens(&env, &token_address, &token_admin, &creator, 100_000_000);

        env.as_contract(&contract_id, || {
            initialize_contract(&env, &token_address);
            RewardManager::create_reward_pool(env.clone(), creator.clone(), 1, token_address.clone(), 0).unwrap();
            RewardManager::fund_reward_pool(env.clone(), creator.clone(), 1, 100_000_000).unwrap();

            RewardManager::distribute_rewards(
                env.clone(),
                1,
                player.clone(),
                xlm_only_config(&env, 50_000_000),
            )
            .unwrap();

            let analytics = RewardManager::get_distribution_analytics(
                env.clone(),
                1,
                None,
                None,
            );
            assert_eq!(analytics.count, 1);
            assert_eq!(analytics.total, 50_000_000);
            assert_eq!(analytics.average, 50_000_000);
            assert_eq!(analytics.median, 50_000_000);
            assert_eq!(analytics.min, 50_000_000);
            assert_eq!(analytics.max, 50_000_000);
        });
    }

    #[test]
    fn test_get_distribution_analytics_multiple_distributions() {
        let env = Env::default();
        env.mock_all_auths_allowing_non_root_auth();
        let (contract_id, token_address, token_admin) = setup(&env);
        let creator = Address::generate(&env);
        let player1 = Address::generate(&env);
        let player2 = Address::generate(&env);
        let player3 = Address::generate(&env);

        mint_tokens(&env, &token_address, &token_admin, &creator, 200_000_000);

        env.as_contract(&contract_id, || {
            initialize_contract(&env, &token_address);
            RewardManager::create_reward_pool(env.clone(), creator.clone(), 1, token_address.clone(), 0).unwrap();
            RewardManager::fund_reward_pool(env.clone(), creator.clone(), 1, 200_000_000).unwrap();

            // Distribute 10M, 20M, 30M — median should be 20M
            RewardManager::distribute_rewards(
                env.clone(),
                1,
                player1.clone(),
                xlm_only_config(&env, 10_000_000),
            )
            .unwrap();

            RewardManager::distribute_rewards(
                env.clone(),
                1,
                player2.clone(),
                xlm_only_config(&env, 30_000_000),
            )
            .unwrap();

            RewardManager::distribute_rewards(
                env.clone(),
                1,
                player3.clone(),
                xlm_only_config(&env, 20_000_000),
            )
            .unwrap();

            let analytics = RewardManager::get_distribution_analytics(
                env.clone(),
                1,
                None,
                None,
            );
            // Values: 10M, 20M, 30M
            assert_eq!(analytics.count, 3);
            assert_eq!(analytics.total, 60_000_000);
            assert_eq!(analytics.average, 20_000_000);  // 60M / 3
            assert_eq!(analytics.median, 20_000_000);   // middle of sorted [10M, 20M, 30M]
            assert_eq!(analytics.min, 10_000_000);
            assert_eq!(analytics.max, 30_000_000);
        });
    }

    #[test]
    fn test_get_distribution_analytics_even_count_median() {
        let env = Env::default();
        env.mock_all_auths_allowing_non_root_auth();
        let (contract_id, token_address, token_admin) = setup(&env);
        let creator = Address::generate(&env);
        let mut players = soroban_sdk::Vec::new(&env);
        for _ in 0..4 {
            players.push_back(Address::generate(&env));
        }

        mint_tokens(&env, &token_address, &token_admin, &creator, 200_000_000);

        env.as_contract(&contract_id, || {
            initialize_contract(&env, &token_address);
            RewardManager::create_reward_pool(env.clone(), creator.clone(), 1, token_address.clone(), 0).unwrap();
            RewardManager::fund_reward_pool(env.clone(), creator.clone(), 1, 200_000_000).unwrap();

            let amounts = [5_000_000i128, 20_000_000, 10_000_000, 15_000_000];
            let mut idx: u32 = 0;
            while idx < 4 {
                RewardManager::distribute_rewards(
                    env.clone(), 1, players.get(idx).unwrap().clone(),
                    xlm_only_config(&env, amounts[idx as usize]),
                ).unwrap();
                idx += 1;
            }

            let analytics = RewardManager::get_distribution_analytics(
                env.clone(), 1, None, None,
            );
            assert_eq!(analytics.count, 4);
            assert_eq!(analytics.total, 50_000_000);
            assert_eq!(analytics.average, 12_500_000);
            assert_eq!(analytics.median, 12_500_000);
            assert_eq!(analytics.min, 5_000_000);
            assert_eq!(analytics.max, 20_000_000);
        });
    }

    #[test]
    fn test_get_distribution_analytics_time_range_filter() {
        let env = Env::default();
        env.mock_all_auths_allowing_non_root_auth();
        let (contract_id, token_address, token_admin) = setup(&env);
        let creator = Address::generate(&env);
        let player1 = Address::generate(&env);
        let player2 = Address::generate(&env);
        let player3 = Address::generate(&env);

        env.ledger().set_timestamp(1_000_000);

        mint_tokens(&env, &token_address, &token_admin, &creator, 200_000_000);

        env.as_contract(&contract_id, || {
            initialize_contract(&env, &token_address);
            RewardManager::create_reward_pool(env.clone(), creator.clone(), 1, token_address.clone(), 0).unwrap();
            RewardManager::fund_reward_pool(env.clone(), creator.clone(), 1, 200_000_000).unwrap();

            RewardManager::distribute_rewards(
                env.clone(), 1, player1.clone(), xlm_only_config(&env, 10_000_000),
            ).unwrap();

            env.ledger().set_timestamp(2_000_000);

            RewardManager::distribute_rewards(
                env.clone(), 1, player2.clone(), xlm_only_config(&env, 20_000_000),
            ).unwrap();

            env.ledger().set_timestamp(3_000_000);

            RewardManager::distribute_rewards(
                env.clone(), 1, player3.clone(), xlm_only_config(&env, 30_000_000),
            ).unwrap();

            let analytics = RewardManager::get_distribution_analytics(
                env.clone(), 1, Some(2_000_000), None,
            );
            assert_eq!(analytics.count, 2);
            assert_eq!(analytics.total, 50_000_000);
            assert_eq!(analytics.min, 20_000_000);
            assert_eq!(analytics.max, 30_000_000);

            let analytics = RewardManager::get_distribution_analytics(
                env.clone(), 1, None, Some(3_000_000),
            );
            assert_eq!(analytics.count, 2);
            assert_eq!(analytics.total, 30_000_000);

            let analytics = RewardManager::get_distribution_analytics(
                env.clone(), 1, Some(1_500_000), Some(2_500_000),
            );
            assert_eq!(analytics.count, 1);
            assert_eq!(analytics.total, 20_000_000);
        });
    }

    #[test]
    fn test_get_distribution_analytics_gas_bound() {
        let env = Env::default();
        env.mock_all_auths_allowing_non_root_auth();
        let (contract_id, token_address, token_admin) = setup(&env);
        let creator = Address::generate(&env);

        mint_tokens(&env, &token_address, &token_admin, &creator, 1_000_000_000_000);

        env.as_contract(&contract_id, || {
            initialize_contract(&env, &token_address);
            RewardManager::create_reward_pool(env.clone(), creator.clone(), 1, token_address.clone(), 0).unwrap();
            RewardManager::fund_reward_pool(env.clone(), creator.clone(), 1, 1_000_000_000_000).unwrap();

            let total_dists: u32 = 100;
            let mut i: u32 = 0;
            while i < total_dists {
                let player = Address::generate(&env);
                Storage::add_pool_distribution(
                    &env,
                    1,
                    crate::types::PoolDistribution {
                        player,
                        xlm_amount: 1_000_000,
                        nft_id: None,
                        timestamp: 100,
                    },
                );
                i += 1;
            }

            let analytics = RewardManager::get_distribution_analytics(
                env.clone(), 1, None, None,
            );
            assert_eq!(analytics.count, 100);
            assert_eq!(analytics.total, 100_000_000i128);
        });
    }
}
