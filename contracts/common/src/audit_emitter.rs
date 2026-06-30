use soroban_sdk::{Address, Env, String, Symbol, Vec};
use crate::audit::{AuditEvent, TOPIC_AUDIT};

/// Emits a standardized audit event to the contract environment
pub fn emit_audit_event(
    env: &Env,
    admin_address: &Address,
    action_type: Symbol,
    contract_name: Symbol,
    details: Vec<(Symbol, String)>,
) {
    let timestamp = env.ledger().timestamp();
    
    let event = AuditEvent {
        admin_address: admin_address.clone(),
        timestamp,
        action_type: action_type.clone(),
        contract: contract_name,
        details,
    };

    // Emit with indexed topics for efficient off-chain filtering
    env.events().publish(
        (TOPIC_AUDIT, action_type, admin_address.clone()),
        event,
    );
}

/// Helper to build detail pairs
pub fn detail(env: &Env, key: Symbol, value: &str) -> (Symbol, String) {
    (key, String::from_str(env, value))
}