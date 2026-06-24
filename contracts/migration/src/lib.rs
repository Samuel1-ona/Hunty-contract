#![no_std]
use soroban_sdk::{contracttype, symbol_short, Env, Symbol};

/// Current schema version for Hunty contract storage layouts.
pub const CURRENT_SCHEMA_VERSION: u32 = 2;

pub const VERSION_KEY: Symbol = symbol_short!("SCHEMA");
pub const ROLLBACK_KEY: Symbol = symbol_short!("RBKVER");

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MigrationReport {
    pub from_version: u32,
    pub to_version: u32,
    pub steps_applied: u32,
    pub dry_run: bool,
    pub succeeded: bool,
    pub message: soroban_sdk::String,
}

/// Shared migration utilities used by all Hunty contracts.
pub struct MigrationFramework;

impl MigrationFramework {
    /// Reads the stored schema version, defaulting to 0 when uninitialized.
    pub fn detect_version(env: &Env) -> u32 {
        env.storage()
            .instance()
            .get(&VERSION_KEY)
            .unwrap_or(0)
    }

    /// Sets the schema version on first initialization.
    pub fn init_version_on_deploy(env: &Env) {
        if !env.storage().instance().has(&VERSION_KEY) {
            env.storage()
                .instance()
                .set(&VERSION_KEY, &CURRENT_SCHEMA_VERSION);
        }
    }

    pub fn set_version(env: &Env, version: u32) {
        env.storage().instance().set(&VERSION_KEY, &version);
    }

    pub fn save_rollback_point(env: &Env, version: u32) {
        env.storage().instance().set(&ROLLBACK_KEY, &version);
    }

    pub fn rollback_version(env: &Env) -> Option<u32> {
        env.storage().instance().get(&ROLLBACK_KEY)
    }

    pub fn clear_rollback(env: &Env) {
        env.storage().instance().remove(&ROLLBACK_KEY);
    }

    pub fn build_report(
        env: &Env,
        from: u32,
        to: u32,
        steps: u32,
        dry_run: bool,
        succeeded: bool,
        message: &str,
    ) -> MigrationReport {
        MigrationReport {
            from_version: from,
            to_version: to,
            steps_applied: steps,
            dry_run,
            succeeded,
            message: soroban_sdk::String::from_str(env, message),
        }
    }
}
