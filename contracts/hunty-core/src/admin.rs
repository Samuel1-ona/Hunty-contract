use soroban_sdk::{Address, Env, Symbol, Vec};
use crate::storage::{is_admin, is_paused, set_paused, set_admin, is_blacklisted, set_blacklisted};
use common::audit::*;
use common::audit_emitter::emit_audit_event;

pub const CONTRACT_NAME: Symbol = symbol_short!("HUNTY");

/// Toggle contract pause state
pub fn toggle_pause(env: &Env, admin: &Address) {
    admin.require_auth();
    assert!(is_admin(env, admin), "Unauthorized: caller is not admin");

    let current = is_paused(env);
    let new_state = !current;
    set_paused(env, new_state);

    let action = if new_state { ACTION_PAUSE } else { ACTION_UNPAUSE };
    
    let mut details = Vec::new(env);
    details.push_back((symbol_short!("prev_state"), String::from_str(env, if current { "unpaused" } else { "paused" })));
    details.push_back((symbol_short!("new_state"), String::from_str(env, if new_state { "paused" } else { "unpaused" })));

    emit_audit_event(env, admin, action, CONTRACT_NAME, details);
}

/// Transfer admin role to new address
pub fn transfer_admin(env: &Env, current_admin: &Address, new_admin: &Address) {
    current_admin.require_auth();
    assert!(is_admin(env, current_admin), "Unauthorized");

    let old_admin = get_admin(env);
    set_admin(env, new_admin);

    let mut details = Vec::new(env);
    details.push_back((symbol_short!("old_admin"), old_admin.to_string()));
    details.push_back((symbol_short!("new_admin"), new_admin.to_string()));

    emit_audit_event(env, current_admin, ACTION_ADMIN_TRANSFERRED, CONTRACT_NAME, details);
}

/// Add address to blacklist
pub fn blacklist_add(env: &Env, admin: &Address, target: &Address) {
    admin.require_auth();
    assert!(is_admin(env, admin), "Unauthorized");

    set_blacklisted(env, target, true);

    let mut details = Vec::new(env);
    details.push_back((symbol_short!("target"), target.to_string()));
    details.push_back((symbol_short!("operation"), String::from_str(env, "add")));

    emit_audit_event(env, admin, ACTION_BLACKLIST_ADD, CONTRACT_NAME, details);
}

/// Remove address from blacklist
pub fn blacklist_remove(env: &Env, admin: &Address, target: &Address) {
    admin.require_auth();
    assert!(is_admin(env, admin), "Unauthorized");

    set_blacklisted(env, target, false);

    let mut details = Vec::new(env);
    details.push_back((symbol_short!("target"), target.to_string()));
    details.push_back((symbol_short!("operation"), String::from_str(env, "remove")));

    emit_audit_event(env, admin, ACTION_BLACKLIST_REMOVE, CONTRACT_NAME, details);
}

/// Emergency stop all active hunts
pub fn emergency_stop_all(env: &Env, admin: &Address, reason: &str) {
    admin.require_auth();
    assert!(is_admin(env, admin), "Unauthorized");

    // Stop all active hunts
    let active_hunts = get_active_hunt_ids(env);
    for hunt_id in active_hunts.iter() {
        set_hunt_status(env, hunt_id, HuntStatus::EmergencyStopped);
    }

    let mut details = Vec::new(env);
    details.push_back((symbol_short!("reason"), String::from_str(env, reason)));
    details.push_back((symbol_short!("hunts_affected"), active_hunts.len().to_string()));

    emit_audit_event(env, admin, ACTION_EMERGENCY, CONTRACT_NAME, details);
}