#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Address, Env, String, Symbol, Vec};

    #[test]
    fn test_pause_event_emission() {
        let env = Env::default();
        let admin = Address::generate(&env);
        
        // Mock admin auth
        env.mock_all_auths();
        
        // Toggle pause
        toggle_pause(&env, &admin);
        
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
        
        blacklist_add(&env, &admin, &target);
        
        let events = env.events().all();
        let (_, data): (_, AuditEvent) = events.get(0).unwrap();
        
        assert_eq!(data.action_type, ACTION_BLACKLIST_ADD);
        // Verify details contain target address
        let details = data.details;
        assert_eq!(details.len(), 2);
    }

    #[test]
    fn test_emergency_event_timestamp() {
        let env = Env::default();
        let admin = Address::generate(&env);
        
        env.mock_all_auths();
        
        let pre_time = env.ledger().timestamp();
        emergency_stop_all(&env, &admin, "Security breach detected");
        let post_time = env.ledger().timestamp();
        
        let events = env.events().all();
        let (_, data): (_, AuditEvent) = events.get(0).unwrap();
        
        assert!(data.timestamp >= pre_time && data.timestamp <= post_time);
        assert_eq!(data.action_type, ACTION_EMERGENCY);
    }

    #[test]
    #[should_panic(expected = "Unauthorized")]
    fn test_unauthorized_action_no_event() {
        let env = Env::default();
        let non_admin = Address::generate(&env);
        
        // No auth mock - should panic
        toggle_pause(&env, &non_admin);
        
        // Should never reach here
        let events = env.events().all();
        assert_eq!(events.len(), 0);
    }
}