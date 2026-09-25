#[cfg(test)]
mod tests {
    use crate::audit::*;
    use crate::audit_emitter::emit_audit_event;
    use soroban_sdk::{
        symbol_short,
        testutils::{Address as _, Events as _},
        xdr, Address, Env, String, Symbol, TryFromVal, Val, Vec,
    };

    /// Decodes the event topic at `index` as a [`Symbol`].
    fn symbol_at(env: &Env, topics: &Vec<Val>, index: u32) -> Symbol {
        Symbol::try_from_val(env, &topics.get(index)).unwrap()
    }

    /// Returns the recorded contract events as `(contract, topics, data)` tuples.
    ///
    /// Soroban SDK 27 only exposes recorded events in XDR form, so decode them
    /// back into SDK values for the assertions below.
    fn all_events_legacy(env: &Env) -> std::vec::Vec<(Address, Vec<Val>, Val)> {
        env.events()
            .all()
            .events()
            .iter()
            .filter_map(|event| {
                let xdr::ContractEventBody::V0(body) = &event.body else {
                    return None;
                };
                let contract = event.contract_id.as_ref()?;
                let contract_val =
                    Val::try_from_val(env, &xdr::ScVal::Address(contract.clone())).ok()?;
                let contract = Address::try_from_val(env, &contract_val).ok()?;
                let topics = body
                    .topics
                    .iter()
                    .map(|topic| Val::try_from_val(env, topic))
                    .collect::<Result<std::vec::Vec<Val>, _>>()
                    .ok()?;
                let data = Val::try_from_val(env, &body.data).ok()?;
                Some((contract, topics, data))
            })
            .collect()
    }

    #[test]
    fn test_pause_event_emission() {
        let env = Env::default();
        let admin = Address::generate(&env);

        env.mock_all_auths();

        let mut details = Vec::new(&env);
        details.push_back((symbol_short!("prev"), String::from_str(&env, "unpaused")));
        details.push_back((symbol_short!("new"), String::from_str(&env, "paused")));

        let contract = symbol_short!("HUNTY");
        emit_audit_event(&env, &admin, ACTION_PAUSE, contract, details);

        // Verify event was emitted
        let events = all_events_legacy(&env);
        assert_eq!(events.len(), 1);

        let (_contract, topics, data) = events.get(0).unwrap();
        assert_eq!(topics.len(), 3);
        assert_eq!(symbol_at(&env, &topics, 0), TOPIC_AUDIT);
        assert_eq!(symbol_at(&env, &topics, 1), ACTION_PAUSE);
        assert_eq!(Address::try_from_val(&env, &topics.get(2)).unwrap(), admin);

        let event = AuditEvent::try_from_val(&env, &data).unwrap();
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
        details.push_back((symbol_short!("target"), target.to_string()));
        details.push_back((symbol_short!("operation"), String::from_str(&env, "add")));

        let contract = symbol_short!("HUNTY");
        emit_audit_event(&env, &admin, ACTION_BLACKLIST_ADD, contract, details);

        let events = all_events_legacy(&env);
        let (_contract, _topics, data) = events.get(0).unwrap();
        let data = AuditEvent::try_from_val(&env, &data).unwrap();

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

        let events = all_events_legacy(&env);
        let (_contract, _topics, data) = events.get(0).unwrap();
        let data = AuditEvent::try_from_val(&env, &data).unwrap();

        assert!(data.timestamp >= pre_time && data.timestamp <= post_time);
        assert_eq!(data.action_type, ACTION_EMERGENCY);
    }
}
