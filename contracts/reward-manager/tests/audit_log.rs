use reward_manager::storage::Storage;
use reward_manager::types::{PoolAuditEntry, PoolOperation};
use reward_manager::RewardManager;
use soroban_sdk::{contracttype, symbol_short, testutils::Address as _, Address, Env};

#[contracttype]
#[derive(Clone, Debug)]
enum LegacyPoolOperation {
    Create,
    Fund,
    Distribute,
    Refund,
    Withdraw,
}

#[contracttype]
#[derive(Clone, Debug)]
struct LegacyPoolAuditEntry {
    operation: LegacyPoolOperation,
    actor: Address,
    amount: i128,
    timestamp: u64,
}

#[test]
fn test_audit_log_keys_append_count_and_ring_buffer() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(RewardManager, ());
    let hunt_id = 42u64;
    let actor = Address::generate(&env);

    env.as_contract(&contract_id, || {
        assert_eq!(Storage::get_pool_audit_count(&env, hunt_id), 0);

        for i in 0..55u64 {
            Storage::append_audit_entry(
                &env,
                hunt_id,
                PoolAuditEntry {
                    actor: actor.clone(),
                    operation: PoolOperation::Fund,
                    timestamp: 1000 + i,
                    amount: Some(i as i128),
                },
            );
        }

        assert_eq!(Storage::get_pool_audit_count(&env, hunt_id), 55);

        // Ring buffer: entries 0–4 were overwritten by entries 50–54.
        let overwritten = Storage::get_pool_audit_entry(&env, hunt_id, 0).unwrap();
        assert_eq!(overwritten.timestamp, 1050);
        assert_eq!(overwritten.amount, Some(50));

        let oldest_kept = Storage::get_pool_audit_entry(&env, hunt_id, 5).unwrap();
        assert_eq!(oldest_kept.timestamp, 1005);
        assert_eq!(oldest_kept.amount, Some(5));

        let newest = Storage::get_pool_audit_entry(&env, hunt_id, 54).unwrap();
        assert_eq!(newest.timestamp, 1054);
        assert_eq!(newest.amount, Some(54));

        // The public query must page the retained window chronologically, not
        // expose overwritten slots a second time after the ring wraps.
        let first_page = RewardManager::get_pool_audit_log(env.clone(), hunt_id, None, Some(10u32));
        assert_eq!(first_page.total, 55);
        assert_eq!(first_page.entries.len(), 10);
        assert_eq!(first_page.entries.get(0).unwrap().timestamp, 1005);
        assert_eq!(first_page.entries.get(9).unwrap().timestamp, 1014);

        let second_page =
            RewardManager::get_pool_audit_log(env.clone(), hunt_id, Some(10u64), Some(10u32));
        assert_eq!(second_page.entries.len(), 10);
        assert_eq!(second_page.entries.get(0).unwrap().timestamp, 1015);
        assert_eq!(second_page.entries.get(9).unwrap().timestamp, 1024);

        let capped_page =
            RewardManager::get_pool_audit_log(env.clone(), hunt_id, None, Some(1_000u32));
        assert_eq!(capped_page.entries.len(), 50);
        assert_eq!(capped_page.entries.get(49).unwrap().timestamp, 1054);

        let exhausted =
            RewardManager::get_pool_audit_log(env.clone(), hunt_id, Some(50u64), Some(10u32));
        assert!(exhausted.entries.is_empty());
    });
}

#[test]
fn audit_query_handles_empty_and_zero_limit_pages() {
    let env = Env::default();
    let contract_id = env.register(RewardManager, ());
    env.as_contract(&contract_id, || {
        let empty = RewardManager::get_pool_audit_log(env.clone(), 7, None, None);
        assert_eq!(empty.total, 0);
        assert!(empty.entries.is_empty());

        let zero = RewardManager::get_pool_audit_log(env.clone(), 7, None, Some(0));
        assert_eq!(zero.total, 0);
        assert!(zero.entries.is_empty());
    });
}

#[test]
fn legacy_audit_record_is_readable_after_schema_consolidation() {
    assert_eq!(PoolOperation::Create as u32, 0);
    assert_eq!(PoolOperation::Fund as u32, 1);
    assert_eq!(PoolOperation::Distribute as u32, 2);
    assert_eq!(PoolOperation::Withdraw as u32, 3);

    let env = Env::default();
    let contract_id = env.register(RewardManager, ());
    let actor = Address::generate(&env);

    env.as_contract(&contract_id, || {
        let key = (symbol_short!("AUDL"), 9u64, 0u64);
        env.storage().persistent().set(
            &key,
            &LegacyPoolAuditEntry {
                operation: LegacyPoolOperation::Withdraw,
                actor: actor.clone(),
                amount: 777,
                timestamp: 1234,
            },
        );

        let recovered = Storage::get_pool_audit_entry(&env, 9, 0).unwrap();
        assert_eq!(recovered.actor, actor);
        assert_eq!(recovered.operation, PoolOperation::Withdraw);
        assert_eq!(recovered.amount, Some(777));
        assert_eq!(recovered.timestamp, 1234);
    });
}
