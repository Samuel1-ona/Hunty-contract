//! Pool freeze authority (#1077).
//!
//! `freeze_pool` accepts either the pool creator or the contract admin, but it
//! never recorded who issued the freeze. That let a pool creator immediately
//! lift an admin's incident freeze. The freezer is now tracked in
//! `RewardPoolConfig::frozen_by`, and an admin-issued freeze can only be lifted
//! by the admin.
//!
//! Written as a standalone integration target because the crate's `src/test.rs`
//! unit-test module does not currently compile (same reason as
//! `granular_pause.rs`), and because these cases only need storage seeding —
//! no HuntyCore pool creation.

use reward_manager::storage::Storage;
use reward_manager::types::{DistributionMode, RewardPoolConfig};
use reward_manager::{RewardErrorCode, RewardManager};
use soroban_sdk::testutils::Address as _;
use soroban_sdk::{Address, Env, Vec};

struct Fixture {
    env: Env,
    contract_id: Address,
    admin: Address,
    creator: Address,
    stranger: Address,
}

fn setup() -> Fixture {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let creator = Address::generate(&env);
    let stranger = Address::generate(&env);
    let token_address = Address::generate(&env);
    // `RewardManager` initializes through its 3-argument `__constructor`
    // (admin, xlm_token, hunty_core). Registering with no constructor
    // arguments panics before any freeze call can run.
    let contract_id = env.register(
        RewardManager,
        (
            admin.clone(),
            token_address.clone(),
            Address::generate(&env),
        ),
    );

    env.as_contract(&contract_id, || {
        Storage::set_admin(&env, &admin);
        Storage::set_pool_config(
            &env,
            1,
            &RewardPoolConfig {
                creator: creator.clone(),
                delegates: Vec::new(&env),
                min_distribution_amount: 0,
                time_based_tiers: Vec::new(&env),
                frozen: false,
                token_address,
                nft_contract: None,
                target_amount: 0,
                min_distribution_interval_secs: 0,
                distribution_mode: DistributionMode::Fixed,
                vesting_period_secs: 0,
                claim_deadline: 0,
                nft_royalty_bps: 0,
                nft_transferable: true,
                rank_based_tiers: Vec::new(&env),
                frozen_by: None,
            },
        );
    });

    Fixture {
        env,
        contract_id,
        admin,
        creator,
        stranger,
    }
}

fn in_contract<T>(fx: &Fixture, f: impl FnOnce(&Env) -> T) -> T {
    fx.env.as_contract(&fx.contract_id, || f(&fx.env))
}

fn freeze(fx: &Fixture, caller: &Address) -> Result<(), RewardErrorCode> {
    in_contract(fx, |env| {
        RewardManager::freeze_pool(env.clone(), caller.clone(), 1)
    })
}

fn unfreeze(fx: &Fixture, caller: &Address) -> Result<(), RewardErrorCode> {
    in_contract(fx, |env| {
        RewardManager::unfreeze_pool(env.clone(), caller.clone(), 1)
    })
}

fn is_frozen(fx: &Fixture) -> bool {
    in_contract(fx, |env| RewardManager::is_pool_frozen(env.clone(), 1))
}

fn frozen_by(fx: &Fixture) -> Option<Address> {
    in_contract(fx, |env| {
        Storage::get_pool_config(env, 1).and_then(|config| config.frozen_by)
    })
}

#[test]
fn creator_can_freeze_and_unfreeze() {
    let fx = setup();

    freeze(&fx, &fx.creator).unwrap();
    assert!(is_frozen(&fx));
    assert_eq!(frozen_by(&fx), Some(fx.creator.clone()));

    unfreeze(&fx, &fx.creator).unwrap();
    assert!(!is_frozen(&fx));
    assert_eq!(frozen_by(&fx), None);
}

#[test]
fn admin_freeze_cannot_be_lifted_by_creator() {
    let fx = setup();

    freeze(&fx, &fx.admin).unwrap();
    assert_eq!(frozen_by(&fx), Some(fx.admin.clone()));

    // The core of #1077: the creator must not be able to undo an incident freeze.
    assert_eq!(
        unfreeze(&fx, &fx.creator),
        Err(RewardErrorCode::Unauthorized)
    );
    assert!(is_frozen(&fx));
    assert_eq!(frozen_by(&fx), Some(fx.admin.clone()));

    // The admin can still lift their own freeze.
    unfreeze(&fx, &fx.admin).unwrap();
    assert!(!is_frozen(&fx));
    assert_eq!(frozen_by(&fx), None);
}

#[test]
fn admin_can_lift_a_creator_freeze() {
    let fx = setup();

    freeze(&fx, &fx.creator).unwrap();
    unfreeze(&fx, &fx.admin).unwrap();
    assert!(!is_frozen(&fx));
    assert_eq!(frozen_by(&fx), None);
}

#[test]
fn creator_refreeze_cannot_downgrade_an_admin_freeze() {
    let fx = setup();

    freeze(&fx, &fx.admin).unwrap();
    // A creator freeze call on an already-frozen pool must not take over the
    // recorded authority (that would re-open the #1077 bypass).
    freeze(&fx, &fx.creator).unwrap();
    assert_eq!(frozen_by(&fx), Some(fx.admin.clone()));
    assert_eq!(
        unfreeze(&fx, &fx.creator),
        Err(RewardErrorCode::Unauthorized)
    );
}

#[test]
fn stranger_cannot_freeze_or_unfreeze() {
    let fx = setup();

    assert_eq!(
        freeze(&fx, &fx.stranger),
        Err(RewardErrorCode::Unauthorized)
    );

    freeze(&fx, &fx.creator).unwrap();
    assert_eq!(
        unfreeze(&fx, &fx.stranger),
        Err(RewardErrorCode::Unauthorized)
    );
    assert!(is_frozen(&fx));
}

#[test]
fn unattributed_freeze_cannot_be_lifted_by_creator() {
    let fx = setup();

    // Simulate freeze state written before `frozen_by` existed: the pool is
    // frozen but no freezer is recorded. The creator must not be able to lift
    // it, because the freeze cannot be proven to be their own.
    in_contract(&fx, |env| {
        let mut config = Storage::get_pool_config(env, 1).unwrap();
        config.frozen = true;
        config.frozen_by = None;
        Storage::set_pool_config(env, 1, &config);
    });

    assert!(is_frozen(&fx));
    assert_eq!(frozen_by(&fx), None);

    assert_eq!(
        unfreeze(&fx, &fx.creator),
        Err(RewardErrorCode::Unauthorized)
    );
    assert!(is_frozen(&fx));

    // The admin can still lift an unattributed freeze.
    unfreeze(&fx, &fx.admin).unwrap();
    assert!(!is_frozen(&fx));
    assert_eq!(frozen_by(&fx), None);
}
