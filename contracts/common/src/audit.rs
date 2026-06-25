use soroban_sdk::{contracttype, Address, String, Symbol, Vec};

/// Core audit event emitted for every admin action
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuditEvent {
    pub admin_address: Address,
    pub timestamp: u64,
    pub action_type: Symbol,
    pub contract: Symbol,
    pub details: Vec<(Symbol, String)>,
}

/// Standardized action type symbols
pub const ACTION_PAUSE: Symbol = symbol_short!("PAUSE");
pub const ACTION_UNPAUSE: Symbol = symbol_short!("UNPAUSE");
pub const ACTION_ADMIN_ADDED: Symbol = symbol_short!("ADM_ADD");
pub const ACTION_ADMIN_REMOVED: Symbol = symbol_short!("ADM_REM");
pub const ACTION_ADMIN_TRANSFERRED: Symbol = symbol_short!("ADM_TRF");
pub const ACTION_BLACKLIST_ADD: Symbol = symbol_short!("BLK_ADD");
pub const ACTION_BLACKLIST_REMOVE: Symbol = symbol_short!("BLK_REM");
pub const ACTION_EMERGENCY: Symbol = symbol_short!("EMERGENCY");
pub const ACTION_EMERGENCY_WITHDRAW: Symbol = symbol_short!("EMG_WD");
pub const ACTION_EMERGENCY_PAUSE_ALL: Symbol = symbol_short!("EMG_PAU");

/// Topics for event filtering
pub const TOPIC_AUDIT: Symbol = symbol_short!("AUDIT");
pub const TOPIC_ADMIN: Symbol = symbol_short!("ADMIN");
pub const TOPIC_BLACKLIST: Symbol = symbol_short!("BLKLST");
pub const TOPIC_EMERGENCY: Symbol = symbol_short!("EMRG");