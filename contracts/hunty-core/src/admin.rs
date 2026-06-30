use soroban_sdk::{Address, Env, Symbol, Vec, symbol_short, String};
use crate::storage::Storage;
use crate::types::HuntStatus;
use common::audit::*;
use common::audit_emitter::emit_audit_event;

pub const CONTRACT_NAME: Symbol = symbol_short!("HUNTY");

/// Toggle contract pause state
pub fn toggle_pause(env: &Env, admin: &Address) {
    admin.require_auth();
    assert!(Storage::is_admin(env, admin), "Unauthorized: caller is not admin");

    let current = Storage::is_paused(env);
    let new_state = !current;
    Storage::set_paused(env, new_state);

    let action = if new_state { ACTION_PAUSE } else { ACTION_UNPAUSE };
    
    let mut details = Vec::new(env);
    details.push_back((symbol_short!("prev"), String::from_str(env, if current { "unpaused" } else { "paused" })));
    details.push_back((symbol_short!("new"), String::from_str(env, if new_state { "paused" } else { "unpaused" })));

    emit_audit_event(env, admin, action, CONTRACT_NAME, details);
}

/// Transfer admin role to new address
pub fn transfer_admin(env: &Env, current_admin: &Address, new_admin: &Address) {
    current_admin.require_auth();
    assert!(Storage::is_admin(env, current_admin), "Unauthorized");

    let old_admin = Storage::get_admin(env);
    Storage::set_admin(env, new_admin);

    let mut details = Vec::new(env);
    details.push_back((symbol_short!("old_admin"), old_admin.unwrap().to_string()));
    details.push_back((symbol_short!("new_admin"), new_admin.to_string()));

    emit_audit_event(env, current_admin, ACTION_ADMIN_TRANSFERRED, CONTRACT_NAME, details);
}

/// Add address to blacklist
pub fn blacklist_add(env: &Env, admin: &Address, target: &Address) {
    admin.require_auth();
    assert!(Storage::is_admin(env, admin), "Unauthorized");

    Storage::set_blacklisted(env, target, true);

    let mut details = Vec::new(env);
    details.push_back((symbol_short!("target"), target.to_string()));
    details.push_back((symbol_short!("operation"), String::from_str(env, "add")));

    emit_audit_event(env, admin, ACTION_BLACKLIST_ADD, CONTRACT_NAME, details);
}

/// Remove address from blacklist
pub fn blacklist_remove(env: &Env, admin: &Address, target: &Address) {
    admin.require_auth();
    assert!(Storage::is_admin(env, admin), "Unauthorized");

    Storage::set_blacklisted(env, target, false);

    let mut details = Vec::new(env);
    details.push_back((symbol_short!("target"), target.to_string()));
    details.push_back((symbol_short!("operation"), String::from_str(env, "remove")));

    emit_audit_event(env, admin, ACTION_BLACKLIST_REMOVE, CONTRACT_NAME, details);
}

/// Emergency stop all active hunts
pub fn emergency_stop_all(env: &Env, admin: &Address, reason: &str) {
    admin.require_auth();
    assert!(Storage::is_admin(env, admin), "Unauthorized");

    // Stop all active hunts
    let active_hunts = Storage::get_active_hunt_ids(env);
    for hunt_id in active_hunts.iter() {
        Storage::set_hunt_status(env, hunt_id, HuntStatus::EmergencyStopped);
    }

    let mut details = Vec::new(env);
    details.push_back((symbol_short!("reason"), String::from_str(env, reason)));

    let mut buf = [0u8; 11];
    let mut val = active_hunts.len();
    let mut idx = 11;
    if val == 0 {
        idx -= 1;
        buf[idx] = b'0';
    } else {
        while val > 0 {
            idx -= 1;
            buf[idx] = b'0' + (val % 10) as u8;
            val /= 10;
        }
    }
    let len_str = core::str::from_utf8(&buf[idx..]).unwrap();
    let count_str = String::from_str(env, len_str);
    details.push_back((symbol_short!("affected"), count_str));

    emit_audit_event(env, admin, ACTION_EMERGENCY, CONTRACT_NAME, details);
}