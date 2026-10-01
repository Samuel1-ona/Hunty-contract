use crate::storage::{Storage, MAX_CO_CREATORS};
use hunty_migration::{
    MigrationFramework, UpgradeAuthError, UpgradeAuthorization, UpgradeExecutedEvent,
    UpgradeHistoryEntry, UpgradeProposal, UpgradeProposedEvent,
};
use soroban_sdk::{Address, BytesN, Env, Symbol};

pub use hunty_migration::MigrationReport;

/// Per-contract migration steps for HuntyCore storage layouts.
pub struct HuntyCoreMigration;

// Upgrade lifecycle helpers are exercised by the migration framework and
// integration tests rather than this crate's entry points.
#[allow(dead_code)]
impl HuntyCoreMigration {
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

    /// Runs migrations up to `target_version`. When `dry_run` is true, no storage writes occur.
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

        let mut current = MigrationFramework::detect_version(env);
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
        }

        let from_version = current;
        let mut steps = 0u32;
        while current < target_version {
            steps += 1;
            match current {
                0 => {
                    if !dry_run {
                        Self::migrate_v0_to_v1(env);
                    }
                    current = 1;
                }
                1 => {
                    if !dry_run {
                        Self::migrate_v1_to_v2(env);
                    }
                    current = 2;
                }
                // v1 -> v2: not yet implemented.
                // Add the arm here (and the migrate_v1_to_v2 fn below) once
                // the new storage layout is defined. Until then the `_ =>`
                // catchall correctly refuses to bump the version counter.
                2 => {
                    if !dry_run {
                        Self::migrate_v2_to_v3(env);
                    }
                    current = 3;
                }
                3 => {
                    if !dry_run {
                        Self::migrate_v3_to_v4(env);
                    }
                    current = 4;
                }
                _ => {
                    return Ok(MigrationFramework::build_report(
                        env,
                        MigrationFramework::detect_version(env),
                        target_version,
                        steps,
                        dry_run,
                        false,
                        "unsupported version step",
                    ));
                }
            }
        }

        if !dry_run {
            MigrationFramework::set_version(env, current);
            UpgradeAuthorization::finalize_migration_run(env, admin, from_version, current, now);
        }

        Ok(MigrationFramework::build_report(
            env,
            MigrationFramework::detect_version(env),
            target_version,
            steps,
            dry_run,
            true,
            "migration complete",
        ))
    }

    /// Returns true if any data-transforming migration step has run since the
    /// last rollback point was saved. Rollback is refused in that case because
    /// down-migrations are not implemented.
    fn has_data_steps(from_version: u32, to_version: u32) -> bool {
        to_version > from_version
    }

    /// Restores the schema version saved before the last migration.
    pub fn rollback_migration(
        env: &Env,
        admin: &Address,
    ) -> Result<MigrationReport, UpgradeAuthError> {
        UpgradeAuthorization::require_admin(env, admin, Self::configured_admin(env))?;
        let previous =
            MigrationFramework::rollback_version(env).ok_or(UpgradeAuthError::NoProposal)?;
        let current = MigrationFramework::detect_version(env);
        if Self::has_data_steps(previous, current) {
            return Ok(MigrationFramework::build_report(
                env,
                current,
                previous,
                0,
                false,
                false,
                "rollback refused: data migrations cannot be reverted",
            ));
        }
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

    /// v0 -> v1: ensure `required_clues` is populated for legacy hunts.
    fn migrate_v0_to_v1(env: &Env) {
        let hunt_count = Storage::get_hunt_counter(env);
        for hunt_id in 1..=hunt_count {
            if let Some(mut hunt) = Storage::get_hunt(env, hunt_id) {
                if hunt.required_clues == 0 && hunt.total_clues > 0 {
                    hunt.required_clues = hunt.total_clues;
                    Storage::save_hunt(env, &hunt);
                }
            }
        }
    }

    /// v1 -> v2: no-op compatibility stub. This storage schema is still not
    /// migrated in-place, but the version gate must not fail for callers that
    /// attempt to advance through the current migration chain.
    fn migrate_v1_to_v2(_env: &Env) {}

    /// v2 -> v3: populate required clue IDs list for on-demand clue loading.
    /// Iterates all hunts and saves the list of required clue IDs in separate storage,
    /// so that `check_all_required_clues_completed` can verify completion without
    /// loading full clue data.
    fn migrate_v2_to_v3(env: &Env) {
        use soroban_sdk::Vec;
        let hunt_count = Storage::get_hunt_counter(env);
        for hunt_id in 1..=hunt_count {
            if Storage::get_hunt(env, hunt_id).is_none() {
                continue;
            }
            let clue_count = Storage::get_clue_counter(env, hunt_id);
            if clue_count == 0 {
                continue;
            }
            let all_clues = Storage::list_clues_for_hunt(env, hunt_id, 0, clue_count);
            let mut required_ids: Vec<u32> = Vec::new(env);
            for i in 0..all_clues.len() {
                // SAFETY: i is in [0, all_clues.len()) — loop bound guarantees existence
                let clue = all_clues.get(i).unwrap();
                if clue.is_required {
                    required_ids.push_back(clue.clue_id);
                }
            }
            // Only save if there are required clues to avoid unnecessary storage writes
            if !required_ids.is_empty() {
                Storage::set_required_clues(env, hunt_id, &required_ids);
            }
        }
    }

    /// v3 -> v4: add weight field to existing clues (default to 1).
    fn migrate_v3_to_v4(env: &Env) {
        let hunt_count = Storage::get_hunt_counter(env);
        for hunt_id in 1..=hunt_count {
            if Storage::get_hunt(env, hunt_id).is_none() {
                continue;
            }
            let clue_count = Storage::get_clue_counter(env, hunt_id);
            if clue_count == 0 {
                continue;
            }
            let all_clues = Storage::list_clues_for_hunt(env, hunt_id, 0, clue_count);
            for i in 0..all_clues.len() {
                // SAFETY: i is in [0, all_clues.len()) — loop bound guarantees existence
                let clue = all_clues.get(i).unwrap();
                Storage::save_clue(env, hunt_id, &clue);
            }
        }
    }

    /// v4 -> v5: move co-creator lists from instance storage to persistent
    /// storage and enforce the per-hunt cap.
    fn migrate_v4_to_v5(env: &Env) {
        let hunt_count = Storage::get_hunt_counter(env);
        for hunt_id in 1..=hunt_count {
            if Storage::get_hunt(env, hunt_id).is_none() {
                continue;
            }
            let mut co_creators = Storage::get_co_creators(env, hunt_id);
            if co_creators.len() > MAX_CO_CREATORS {
                co_creators = co_creators.slice(0..MAX_CO_CREATORS);
            }
            Storage::set_co_creators(env, hunt_id, &co_creators);
        }
    }
}
