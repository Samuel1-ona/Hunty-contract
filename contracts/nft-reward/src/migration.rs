use crate::storage::Storage;
use crate::METADATA_SCHEMA_VERSION;
use hunty_migration::MigrationFramework;
use soroban_sdk::{Address, Env};

pub use hunty_migration::MigrationReport;

/// Per-contract migration steps for NftReward storage layouts.
pub struct NftRewardMigration;

impl NftRewardMigration {
    pub fn get_schema_version(env: &Env) -> u32 {
        MigrationFramework::detect_version(env)
    }

    pub fn initialize_schema(env: &Env) {
        MigrationFramework::init_version_on_deploy(env);
    }

    /// Runs migrations up to `target_version`. When `dry_run` is true,
    /// no storage writes occur.
    pub fn run_migration(env: &Env, target_version: u32, dry_run: bool) -> MigrationReport {
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

    /// v0 -> v1: retroactively set metadata version key on legacy NFTs.
    fn migrate_v0_to_v1(env: &Env) {
        let total = Storage::get_nft_counter(env);
        for nft_id in 1..=total {
            // Skip NFTs that already have an explicit version key.
            if Storage::has_nft_version_key(env, nft_id) {
                continue;
            }
            Storage::set_nft_version(env, nft_id, METADATA_SCHEMA_VERSION);
        }
    }

    /// v1 -> v2: placeholder for future metadata layout changes.
    fn migrate_v1_to_v2(_env: &Env) {}
}
