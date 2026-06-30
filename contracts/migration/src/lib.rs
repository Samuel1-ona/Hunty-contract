#![no_std]
use soroban_sdk::{contracterror, contracttype, symbol_short, Address, Env, Symbol, Vec};

/// Current schema version for Hunty contract storage layouts.
pub const CURRENT_SCHEMA_VERSION: u32 = 3;

pub const VERSION_KEY: Symbol = symbol_short!("SCHEMA");
pub const ROLLBACK_KEY: Symbol = symbol_short!("RBKVER");
const PROPOSAL_KEY: Symbol = symbol_short!("UPROP");
const TIMELOCK_KEY: Symbol = symbol_short!("UPTLK");
const HIST_COUNT_KEY: Symbol = symbol_short!("UPHCT");
const UPGRADE_ADMIN_KEY: Symbol = symbol_short!("UPADM");

#[contracterror]
#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum UpgradeAuthError {
    Unauthorized = 1,
    NoProposal = 2,
    TimelockPending = 3,
    VersionMismatch = 4,
    InvalidTimelock = 5,
}

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

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UpgradeProposal {
    pub target_version: u32,
    pub proposed_at: u64,
    pub effective_at: u64,
    pub proposer: Address,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UpgradeHistoryEntry {
    pub from_version: u32,
    pub to_version: u32,
    pub executed_at: u64,
    pub executor: Address,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UpgradeProposedEvent {
    pub target_version: u32,
    pub proposed_at: u64,
    pub effective_at: u64,
    pub proposer: Address,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UpgradeExecutedEvent {
    pub from_version: u32,
    pub to_version: u32,
    pub executed_at: u64,
    pub executor: Address,
}

pub struct MigrationFramework;

impl MigrationFramework {
    pub fn detect_version(env: &Env) -> u32 {
        env.storage().instance().get(&VERSION_KEY).unwrap_or(0)
    }

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

pub struct UpgradeAuthorization;

impl UpgradeAuthorization {
    pub fn set_upgrade_admin(env: &Env, admin: &Address) {
        env.storage()
            .instance()
            .set(&UPGRADE_ADMIN_KEY, admin);
    }

    pub fn get_upgrade_admin(env: &Env) -> Option<Address> {
        env.storage().instance().get(&UPGRADE_ADMIN_KEY)
    }

    pub fn require_admin(
        env: &Env,
        caller: &Address,
        configured_admin: Option<Address>,
    ) -> Result<(), UpgradeAuthError> {
        caller.require_auth();
        let admin = configured_admin
            .or_else(|| Self::get_upgrade_admin(env))
            .ok_or(UpgradeAuthError::Unauthorized)?;
        if admin != *caller {
            return Err(UpgradeAuthError::Unauthorized);
        }
        Ok(())
    }

    pub fn get_timelock_seconds(env: &Env) -> u64 {
        env.storage().instance().get(&TIMELOCK_KEY).unwrap_or(0)
    }

    pub fn set_timelock_seconds(env: &Env, seconds: u64) {
        env.storage().instance().set(&TIMELOCK_KEY, &seconds);
    }

    pub fn get_proposal(env: &Env) -> Option<UpgradeProposal> {
        env.storage().instance().get(&PROPOSAL_KEY)
    }

    pub fn propose_upgrade(
        env: &Env,
        proposer: &Address,
        target_version: u32,
        now: u64,
    ) -> UpgradeProposal {
        let timelock = Self::get_timelock_seconds(env);
        let proposal = UpgradeProposal {
            target_version,
            proposed_at: now,
            effective_at: now.saturating_add(timelock),
            proposer: proposer.clone(),
        };
        env.storage().instance().set(&PROPOSAL_KEY, &proposal);
        proposal
    }

    pub fn clear_proposal(env: &Env) {
        env.storage().instance().remove(&PROPOSAL_KEY);
    }

    pub fn validate_execution(
        env: &Env,
        target_version: u32,
        now: u64,
    ) -> Result<UpgradeProposal, UpgradeAuthError> {
        let proposal = Self::get_proposal(env).ok_or(UpgradeAuthError::NoProposal)?;
        if proposal.target_version != target_version {
            return Err(UpgradeAuthError::VersionMismatch);
        }
        if now < proposal.effective_at {
            return Err(UpgradeAuthError::TimelockPending);
        }
        Ok(proposal)
    }

    pub fn record_execution(env: &Env, entry: &UpgradeHistoryEntry) {
        let count: u32 = env
            .storage()
            .persistent()
            .get(&HIST_COUNT_KEY)
            .unwrap_or(0);
        let key = (symbol_short!("UPHIS"), count);
        env.storage().persistent().set(&key, entry);
        env.storage()
            .persistent()
            .set(&HIST_COUNT_KEY, &(count + 1));
    }

    pub fn get_history(env: &Env, offset: u32, limit: u32) -> Vec<UpgradeHistoryEntry> {
        let count: u32 = env
            .storage()
            .persistent()
            .get(&HIST_COUNT_KEY)
            .unwrap_or(0);
        if offset >= count {
            return Vec::new(env);
        }
        let end = offset.saturating_add(limit).min(count);
        let mut entries = Vec::new(env);
        for i in offset..end {
            let key = (symbol_short!("UPHIS"), i);
            if let Some(entry) = env.storage().persistent().get(&key) {
                entries.push_back(entry);
            }
        }
        entries
    }

    pub fn history_count(env: &Env) -> u32 {
        env.storage()
            .persistent()
            .get(&HIST_COUNT_KEY)
            .unwrap_or(0)
    }

    pub fn prepare_migration_run(
        env: &Env,
        admin: &Address,
        configured_admin: Option<Address>,
        target_version: u32,
        dry_run: bool,
        now: u64,
    ) -> Result<(), UpgradeAuthError> {
        Self::require_admin(env, admin, configured_admin)?;
        if !dry_run {
            Self::validate_execution(env, target_version, now)?;
        }
        Ok(())
    }

    pub fn finalize_migration_run(
        env: &Env,
        executor: &Address,
        from_version: u32,
        to_version: u32,
        now: u64,
    ) {
        Self::clear_proposal(env);
        Self::record_execution(
            env,
            &UpgradeHistoryEntry {
                from_version,
                to_version,
                executed_at: now,
                executor: executor.clone(),
            },
        );
    }
}
