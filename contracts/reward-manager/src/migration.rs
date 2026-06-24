use hunty_migration::{MigrationFramework, CURRENT_SCHEMA_VERSION};
use soroban_sdk::{Address, Env};

pub use hunty_migration::MigrationReport;

pub struct RewardManagerMigration;

impl RewardManagerMigration {
    pub fn get_schema_version(env: &Env) -> u32 {
        MigrationFramework::detect_version(env)
    }

    pub fn initialize_schema(env: &Env) {
        MigrationFramework::init_version_on_deploy(env);
    }

    pub fn run_migration(env: &Env, target_version: u32, dry_run: bool) -> MigrationReport {
        let current = MigrationFramework::detect_version(env);
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
            MigrationFramework::set_version(env, CURRENT_SCHEMA_VERSION);
        }
        MigrationFramework::build_report(
            env,
            current,
            target_version,
            1,
            dry_run,
            true,
            "reward-manager migration complete",
        )
    }

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
}
