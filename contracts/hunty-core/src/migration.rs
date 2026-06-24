use crate::storage::Storage;
use hunty_migration::MigrationFramework;
use soroban_sdk::{Address, Env};

pub use hunty_migration::MigrationReport;

/// Per-contract migration steps for HuntyCore storage layouts.
pub struct HuntyCoreMigration;

impl HuntyCoreMigration {
    pub fn get_schema_version(env: &Env) -> u32 {
        MigrationFramework::detect_version(env)
    }

    pub fn initialize_schema(env: &Env) {
        MigrationFramework::init_version_on_deploy(env);
    }

    /// Runs migrations up to `target_version`. When `dry_run` is true, no storage writes occur.
    pub fn run_migration(env: &Env, target_version: u32, dry_run: bool) -> MigrationReport {
        let mut current = MigrationFramework::detect_version(env);
        if current >= target_version {
            return MigrationFramework::build_report(
                env,
                current,
                target_version,
                0,
                dry_run,
                true,
                "already at target",
            );
        }

        if !dry_run {
            MigrationFramework::save_rollback_point(env, current);
        }

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
                _ => {
                    return MigrationFramework::build_report(
                        env,
                        MigrationFramework::detect_version(env),
                        target_version,
                        steps,
                        dry_run,
                        false,
                        "unsupported version step",
                    );
                }
            }
        }

        if !dry_run {
            MigrationFramework::set_version(env, current);
        }

        MigrationFramework::build_report(
            env,
            MigrationFramework::detect_version(env),
            target_version,
            steps,
            dry_run,
            true,
            "migration complete",
        )
    }

    /// Restores the schema version saved before the last migration.
    pub fn rollback_migration(env: &Env, admin: Address) -> Option<MigrationReport> {
        admin.require_auth();
        let previous = MigrationFramework::rollback_version(env)?;
        let current = MigrationFramework::detect_version(env);
        MigrationFramework::set_version(env, previous);
        MigrationFramework::clear_rollback(env);
        Some(MigrationFramework::build_report(
            env,
            current,
            previous,
            1,
            false,
            true,
            "rolled back",
        ))
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

    /// v1 -> v2: placeholder for future layout changes (no-op for now).
    fn migrate_v1_to_v2(_env: &Env) {}
}
