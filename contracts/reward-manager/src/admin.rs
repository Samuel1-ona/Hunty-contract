use soroban_sdk::{Address, Env, Symbol, Vec};
use common::audit::*;
use common::audit_emitter::emit_audit_event;

pub const CONTRACT_NAME: Symbol = symbol_short!("REWARD");

/// Pause reward distributions
pub fn pause_rewards(env: &Env, admin: &Address) {
    admin.require_auth();
    assert!(is_admin(env, admin), "Unauthorized");

    set_rewards_paused(env, true);

    let details = Vec::new(env);
    emit_audit_event(env, admin, ACTION_PAUSE, CONTRACT_NAME, details);
}

/// Unpause reward distributions
pub fn unpause_rewards(env: &Env, admin: &Address) {
    admin.require_auth();
    assert!(is_admin(env, admin), "Unauthorized");

    set_rewards_paused(env, false);

    let details = Vec::new(env);
    emit_audit_event(env, admin, ACTION_UNPAUSE, CONTRACT_NAME, details);
}

/// Emergency withdrawal of reward pool
pub fn emergency_withdraw(env: &Env, admin: &Address, recipient: &Address, amount: i128) {
    admin.require_auth();
    assert!(is_admin(env, admin), "Unauthorized");

    // Perform withdrawal
    transfer_reward_pool(env, recipient, amount);

    let mut details = Vec::new(env);
    details.push_back((symbol_short!("recipient"), recipient.to_string()));
    details.push_back((symbol_short!("amount"), amount.to_string()));
    details.push_back((symbol_short!("token"), String::from_str(env, "XLM")));

    emit_audit_event(env, admin, ACTION_EMERGENCY_WITHDRAW, CONTRACT_NAME, details);
}