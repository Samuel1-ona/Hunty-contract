use crate::storage::Storage;
use hunty_migration::{
    MigrationFramework, UpgradeAuthError, UpgradeAuthorization, UpgradeExecutedEvent,
    UpgradeHistoryEntry, UpgradeProposal, UpgradeProposedEvent, CURRENT_SCHEMA_VERSION,
};
use soroban_sdk::{Address, BytesN, Env, Symbol};

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
        wasm_hash: BytesN<32>,
    ) -> Result<UpgradeProposal, UpgradeAuthError> {
        UpgradeAuthorization::require_admin(env, admin, Self::configured_admin(env))?;
        let now = env.ledger().timestamp();
        UpgradeAuthorization::propose_upgrade(env, admin, target_version, wasm_hash, now)
    }

    pub fn upgrade(
        env: &Env,
        admin: &Address,
        new_wasm_hash: BytesN<32>,
    ) -> Result<(), UpgradeAuthError> {
        UpgradeAuthorization::require_admin(env, admin, Self::configured_admin(env))?;
        let now = env.ledger().timestamp();
        let proposal = UpgradeAuthorization::validate_upgrade(env, &new_wasm_hash, now)?;
        let from_version = MigrationFramework::detect_version(env);
        let to_version = proposal.target_version;

        MigrationFramework::set_version(env, to_version);
        UpgradeAuthorization::finalize_upgrade_run(
            env,
            admin,
            from_version,
            to_version,
            &new_wasm_hash,
            now,
        );

        let event = Self::upgrade_executed_event(
            from_version,
            to_version,
            &new_wasm_hash,
            now,
            admin.clone(),
        );
        env.events()
            .publish(Self::upgrade_executed_topic(env), event);

        env.deployer().update_current_contract_wasm(new_wasm_hash);
        Ok(())
    }

    /// Sets the upgrade timelock to `delay_seconds`.
    ///
    /// Rejects values below `MIN_UPGRADE_TIMELOCK_SECONDS` (24 hours). Timelock
    /// reductions only take effect after the current timelock elapses, so users
    /// keep the warning window they were promised.
    pub fn set_upgrade_timelock(
        env: &Env,
        admin: &Address,
        delay_seconds: u64,
    ) -> Result<(), UpgradeAuthError> {
        UpgradeAuthorization::require_admin(env, admin, Self::configured_admin(env))?;
        let now = env.ledger().timestamp();
        UpgradeAuthorization::set_timelock_seconds(env, now, delay_seconds)
    }

    pub fn get_upgrade_proposal(env: &Env) -> Option<UpgradeProposal> {
        UpgradeAuthorization::get_proposal(env)
    }

    pub fn get_upgrade_timelock(env: &Env) -> u64 {
        UpgradeAuthorization::get_timelock_seconds(env)
    }

    /// Returns the pending timelock reduction, if one is queued.
    #[allow(dead_code)]
    pub fn get_pending_timelock_change(env: &Env) -> Option<hunty_migration::TimelockChange> {
        UpgradeAuthorization::get_pending_timelock_change(env)
    }

    pub fn get_upgrade_history(
        env: &Env,
        offset: u32,
        limit: u32,
    ) -> soroban_sdk::Vec<UpgradeHistoryEntry> {
        UpgradeAuthorization::get_history(env, offset, limit)
    }

    pub fn run_migration(
        env: &Env,
        admin: &Address,
        target_version: u32,
        dry_run: bool,
    ) -> Result<MigrationReport, UpgradeAuthError> {
        let now = env.ledger().timestamp();

        // Reject target_version above the current schema version, as we have no
        // implementation to reach a future version.
        if target_version > CURRENT_SCHEMA_VERSION {
            return Err(UpgradeAuthError::VersionMismatch);
        }

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
            // CRITICAL: Write the target_version, not CURRENT_SCHEMA_VERSION.
            // The caller explicitly requested a migration to target_version;
            // jumping to CURRENT_SCHEMA_VERSION skips intermediate migration steps
            // and contradicts the reported to_version in the event.
            MigrationFramework::set_version(env, target_version);
            UpgradeAuthorization::finalize_migration_run(env, admin, current, target_version, now);
        }
        // Report the actual version we wrote, not the target_version parameter.
        let written_version = if dry_run { current } else { target_version };
        Ok(MigrationFramework::build_report(
            env,
            current,
            written_version,
            if current < target_version { 1 } else { 0 },
            dry_run,
            true,
            "reward-manager migration complete",
        ))
    }

    pub fn rollback_migration(
        env: &Env,
        admin: &Address,
    ) -> Result<MigrationReport, UpgradeAuthError> {
        UpgradeAuthorization::require_admin(env, admin, Self::configured_admin(env))?;
        let previous =
            MigrationFramework::rollback_version(env).ok_or(UpgradeAuthError::NoProposal)?;
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
            wasm_hash: proposal.wasm_hash.clone(),
            proposed_at: proposal.proposed_at,
            effective_at: proposal.effective_at,
            proposer: proposal.proposer.clone(),
        }
    }

    pub fn upgrade_executed_event(
        from_version: u32,
        to_version: u32,
        wasm_hash: &BytesN<32>,
        executed_at: u64,
        executor: Address,
    ) -> UpgradeExecutedEvent {
        UpgradeExecutedEvent {
            from_version,
            to_version,
            wasm_hash: wasm_hash.clone(),
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
