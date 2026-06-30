use soroban_sdk::{Address, Env, Symbol, Vec};
use common::audit::*;
use common::audit_emitter::emit_audit_event;

pub const CONTRACT_NAME: Symbol = symbol_short!("NFT");

/// Add secondary admin
pub fn add_admin(env: &Env, current_admin: &Address, new_admin: &Address) {
    current_admin.require_auth();
    assert!(is_admin(env, current_admin), "Unauthorized");

    add_admin_address(env, new_admin);

    let mut details = Vec::new(env);
    details.push_back((symbol_short!("added_admin"), new_admin.to_string()));
    details.push_back((symbol_short!("added_by"), current_admin.to_string()));

    emit_audit_event(env, current_admin, ACTION_ADMIN_ADDED, CONTRACT_NAME, details);
}

/// Remove admin
pub fn remove_admin(env: &Env, current_admin: &Address, admin_to_remove: &Address) {
    current_admin.require_auth();
    assert!(is_admin(env, current_admin), "Unauthorized");

    remove_admin_address(env, admin_to_remove);

    let mut details = Vec::new(env);
    details.push_back((symbol_short!("removed_admin"), admin_to_remove.to_string()));
    details.push_back((symbol_short!("removed_by"), current_admin.to_string()));

    emit_audit_event(env, current_admin, ACTION_ADMIN_REMOVED, CONTRACT_NAME, details);
}

/// Pause NFT minting
pub fn pause_minting(env: &Env, admin: &Address) {
    admin.require_auth();
    assert!(is_admin(env, admin), "Unauthorized");

    set_minting_paused(env, true);

    let details = Vec::new(env);
    emit_audit_event(env, admin, ACTION_PAUSE, CONTRACT_NAME, details);
}

/// Emergency freeze all NFTs
pub fn emergency_freeze(env: &Env, admin: &Address, reason: &str) {
    admin.require_auth();
    assert!(is_admin(env, admin), "Unauthorized");

    set_all_nfts_frozen(env, true);

    let mut details = Vec::new(env);
    details.push_back((symbol_short!("reason"), String::from_str(env, reason)));
    details.push_back((symbol_short!("freeze_all"), String::from_str(env, "true")));

    emit_audit_event(env, admin, ACTION_EMERGENCY, CONTRACT_NAME, details);
}