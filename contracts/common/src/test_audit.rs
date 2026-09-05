#[cfg(test)]
mod tests {
    use crate::audit::*;
    use crate::audit_emitter::emit_audit_event;
    use soroban_sdk::{
        contract, contractimpl, symbol_short,
        testutils::{Address as _, Events as _},
        Address, Env, String, Symbol, TryFromVal, Vec,
    };

    #[contract]
    struct TestContract;

    #[contractimpl]
    impl TestContract {}

    #[test]
    fn test_pause_event_emission() {
        let env = Env::default();
        let contract_id = env.register(TestContract, ());
        let admin = Address::generate(&env);

        env.mock_all_auths();

        let mut details = Vec::new(&env);
        details.push_back((symbol_short!("prev"), String::from_str(&env, "unpaused")));
        details.push_back((symbol_short!("new"), String::from_str(&env, "paused")));

        let contract = symbol_short!("HUNTY");
        env.as_contract(&contract_id, || {
            emit_audit_event(&env, &admin, ACTION_PAUSE, contract, details);
        });

        // Verify event was emitted
        let events = env.events().all();
        assert_eq!(events.len(), 1);

        let (_contract_id, topics, data_val) = events.get(0).unwrap();
        assert_eq!(
            Symbol::try_from_val(&env, &topics.get(0).unwrap()).unwrap(),
            TOPIC_AUDIT
        );
        assert_eq!(
            Symbol::try_from_val(&env, &topics.get(1).unwrap()).unwrap(),
            ACTION_PAUSE
        );
        assert_eq!(
            Address::try_from_val(&env, &topics.get(2).unwrap()).unwrap(),
            admin
        );

        let event = AuditEvent::try_from_val(&env, &data_val).unwrap();
        assert_eq!(event.admin_address, admin);
        assert_eq!(event.timestamp, env.ledger().timestamp());
        assert_eq!(event.action_type, ACTION_PAUSE);
    }

    #[test]
    fn test_blacklist_event_with_details() {
        let env = Env::default();
        let contract_id = env.register(TestContract, ());
        let admin = Address::generate(&env);
        let target = Address::generate(&env);

        env.mock_all_auths();

        let mut details = Vec::new(&env);
        details.push_back((symbol_short!("target"), target.to_string()));
        details.push_back((symbol_short!("operation"), String::from_str(&env, "add")));

        let contract = symbol_short!("HUNTY");
        env.as_contract(&contract_id, || {
            emit_audit_event(&env, &admin, ACTION_BLACKLIST_ADD, contract, details);
        });

        let events = env.events().all();
        let (_contract_id, _topics, data_val) = events.get(0).unwrap();
        let data = AuditEvent::try_from_val(&env, &data_val).unwrap();

        assert_eq!(data.action_type, ACTION_BLACKLIST_ADD);
        // Verify details contain target address
        let event_details = data.details;
        assert_eq!(event_details.len(), 2);
    }

    #[test]
    fn test_emergency_event_timestamp() {
        let env = Env::default();
        let contract_id = env.register(TestContract, ());
        let admin = Address::generate(&env);

        env.mock_all_auths();

        let mut details = Vec::new(&env);
        details.push_back((
            symbol_short!("reason"),
            String::from_str(&env, "Security breach detected"),
        ));

        let pre_time = env.ledger().timestamp();
        let contract = symbol_short!("HUNTY");
        env.as_contract(&contract_id, || {
            emit_audit_event(&env, &admin, ACTION_EMERGENCY, contract, details);
        });
        let post_time = env.ledger().timestamp();

        let events = env.events().all();
        let (_contract_id, _topics, data_val) = events.get(0).unwrap();
        let data = AuditEvent::try_from_val(&env, &data_val).unwrap();

        assert!(data.timestamp >= pre_time && data.timestamp <= post_time);
        assert_eq!(data.action_type, ACTION_EMERGENCY);
    }
}
