use reward_manager::storage::Storage;
use reward_manager::types::{PoolAuditEntry, PoolOperation};
use reward_manager::RewardManager;
use soroban_sdk::{testutils::Address as _, Address, Env};

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
    });
}
