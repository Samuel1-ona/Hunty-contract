#[cfg(test)]
mod tests {
    use crate::audit::*;
    use crate::audit_emitter::emit_audit_event;
    use soroban_sdk::{testutils::Address as _, Address, Env, String, Symbol, Vec, symbol_short};

    #[test]
    fn test_pause_event_emission() {
        let env = Env::default();
        let admin = Address::generate(&env);

        env.mock_all_auths();

        let mut details = Vec::new(&env);
        details.push_back((
            symbol_short!("prev"),
            String::from_str(&env, "unpaused"),
        ));
        details.push_back((
            symbol_short!("new"),
            String::from_str(&env, "paused"),
        ));

        let contract = symbol_short!("HUNTY");
        emit_audit_event(&env, &admin, ACTION_PAUSE, contract, details);

        // Verify event was emitted
        let events = env.events().all();
        assert_eq!(events.len(), 1);

        let (topics, data): ((Symbol, Symbol, Address), AuditEvent) = events.get(0).unwrap();
        assert_eq!(topics.0, TOPIC_AUDIT);
        assert_eq!(topics.1, ACTION_PAUSE);
        assert_eq!(topics.2, admin);

        let event: AuditEvent = data;
        assert_eq!(event.admin_address, admin);
        assert!(event.timestamp > 0);
        assert_eq!(event.action_type, ACTION_PAUSE);
    }

    #[test]
    fn test_blacklist_event_with_details() {
        let env = Env::default();
        let admin = Address::generate(&env);
        let target = Address::generate(&env);

        env.mock_all_auths();

        let mut details = Vec::new(&env);
        details.push_back((
            symbol_short!("target"),
            target.to_string(),
        ));
        details.push_back((
            symbol_short!("operation"),
            String::from_str(&env, "add"),
        ));

        let contract = symbol_short!("HUNTY");
        emit_audit_event(
            &env,
            &admin,
            ACTION_BLACKLIST_ADD,
            contract,
            details,
        );

        let events = env.events().all();
        let (_, data): (_, AuditEvent) = events.get(0).unwrap();

        assert_eq!(data.action_type, ACTION_BLACKLIST_ADD);
        // Verify details contain target address
        let event_details = data.details;
        assert_eq!(event_details.len(), 2);
    }

    #[test]
    fn test_emergency_event_timestamp() {
        let env = Env::default();
        let admin = Address::generate(&env);

        env.mock_all_auths();

        let mut details = Vec::new(&env);
        details.push_back((
            symbol_short!("reason"),
            String::from_str(&env, "Security breach detected"),
        ));

        let pre_time = env.ledger().timestamp();
        let contract = symbol_short!("HUNTY");
        emit_audit_event(&env, &admin, ACTION_EMERGENCY, contract, details);
        let post_time = env.ledger().timestamp();

        let events = env.events().all();
        let (_, data): (_, AuditEvent) = events.get(0).unwrap();

        assert!(data.timestamp >= pre_time && data.timestamp <= post_time);
        assert_eq!(data.action_type, ACTION_EMERGENCY);
    }
}
