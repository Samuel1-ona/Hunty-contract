use crate::storage::Storage;
use hunty_migration::{
    MigrationFramework, UpgradeAuthorization, UpgradeAuthError, UpgradeExecutedEvent,
    UpgradeHistoryEntry, UpgradeProposal, UpgradeProposedEvent, CURRENT_SCHEMA_VERSION,
};
use soroban_sdk::{Address, Env, Symbol};

pub use hunty_migration::MigrationReport;

pub struct RewardManagerMigration;

impl RewardManagerMigration {
    pub fn get_schema_version(env: &Env) -> u32 {
        MigrationFramework::detect_version(env)
    }

    pub fn initialize_schema(env: &Env) {
        MigrationFramework::init_version_on_deploy(env);
    }

    fn configured_admin(env: &Env) -> Option<Address> {
        Storage::get_admin(env)
    }

    pub fn propose_upgrade(
        env: &Env,
        admin: &Address,
        target_version: u32,
    ) -> Result<UpgradeProposal, UpgradeAuthError> {
        UpgradeAuthorization::require_admin(env, admin, Self::configured_admin(env))?;
        let now = env.ledger().timestamp();
        Ok(UpgradeAuthorization::propose_upgrade(env, admin, target_version, now))
    }

    pub fn set_upgrade_timelock(
        env: &Env,
        admin: &Address,
        delay_seconds: u64,
    ) -> Result<(), UpgradeAuthError> {
        UpgradeAuthorization::require_admin(env, admin, Self::configured_admin(env))?;
        UpgradeAuthorization::set_timelock_seconds(env, delay_seconds);
        Ok(())
    }

    pub fn get_upgrade_proposal(env: &Env) -> Option<UpgradeProposal> {
        UpgradeAuthorization::get_proposal(env)
    }

    pub fn get_upgrade_timelock(env: &Env) -> u64 {
        UpgradeAuthorization::get_timelock_seconds(env)
    }

    pub fn get_upgrade_history(env: &Env, offset: u32, limit: u32) -> soroban_sdk::Vec<UpgradeHistoryEntry> {
        UpgradeAuthorization::get_history(env, offset, limit)
    }

    pub fn run_migration(
        env: &Env,
        admin: &Address,
        target_version: u32,
        dry_run: bool,
    ) -> Result<MigrationReport, UpgradeAuthError> {
        let now = env.ledger().timestamp();
        UpgradeAuthorization::prepare_migration_run(
            env,
            admin,
            Self::configured_admin(env),
            target_version,
            dry_run,
            now,
        )?;

        let current = MigrationFramework::detect_version(env);
        if current >= target_version {
            return Ok(MigrationFramework::build_report(
                env,
                current,
                target_version,
                0,
                dry_run,
                true,
                "already at target",
            ));
        }
        if !dry_run {
            MigrationFramework::save_rollback_point(env, current);
            MigrationFramework::set_version(env, CURRENT_SCHEMA_VERSION);
            UpgradeAuthorization::finalize_migration_run(
                env,
                admin,
                current,
                CURRENT_SCHEMA_VERSION,
                now,
            );
        }
        Ok(MigrationFramework::build_report(
            env,
            current,
            target_version,
            1,
            dry_run,
            true,
            "reward-manager migration complete",
        ))
    }

    pub fn rollback_migration(env: &Env, admin: &Address) -> Result<MigrationReport, UpgradeAuthError> {
        UpgradeAuthorization::require_admin(env, admin, Self::configured_admin(env))?;
        let previous = MigrationFramework::rollback_version(env).ok_or(UpgradeAuthError::NoProposal)?;
        let current = MigrationFramework::detect_version(env);
        MigrationFramework::set_version(env, previous);
        MigrationFramework::clear_rollback(env);
        Ok(MigrationFramework::build_report(
            env,
            current,
            previous,
            1,
            false,
            true,
            "rolled back",
        ))
    }

    pub fn upgrade_proposed_event(proposal: &UpgradeProposal) -> UpgradeProposedEvent {
        UpgradeProposedEvent {
            target_version: proposal.target_version,
            proposed_at: proposal.proposed_at,
            effective_at: proposal.effective_at,
            proposer: proposal.proposer.clone(),
        }
    }

    pub fn upgrade_executed_event(
        from_version: u32,
        to_version: u32,
        executed_at: u64,
        executor: Address,
    ) -> UpgradeExecutedEvent {
        UpgradeExecutedEvent {
            from_version,
            to_version,
            executed_at,
            executor,
        }
    }

    pub fn upgrade_proposed_topic(env: &Env) -> (Symbol,) {
        (Symbol::new(env, "UpgradeProposed"),)
    }

    pub fn upgrade_executed_topic(env: &Env) -> (Symbol,) {
        (Symbol::new(env, "UpgradeExecuted"),)
    }
}
