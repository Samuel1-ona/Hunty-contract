#![no_std]
use soroban_sdk::{contracterror, contracttype, symbol_short, Address, BytesN, Env, Symbol, Vec};

/// Current schema version for Hunty contract storage layouts.
pub const CURRENT_SCHEMA_VERSION: u32 = 3;

/// Minimum accepted upgrade timelock, in seconds.
///
/// A zero (or otherwise tiny) timelock lets an admin — or a compromised admin
/// key — propose and execute a migration in the same ledger, giving users no
/// window to react. Proposing an upgrade while the effective timelock is below
/// this bound fails with `UpgradeAuthError::InvalidTimelock`, and the timelock
/// itself can never be set below it.
pub const MIN_UPGRADE_TIMELOCK_SECONDS: u64 = 86_400; // 24 hours

pub const VERSION_KEY: Symbol = symbol_short!("SCHEMA");
pub const ROLLBACK_KEY: Symbol = symbol_short!("RBKVER");
const PROPOSAL_KEY: Symbol = symbol_short!("UPROP");
const TIMELOCK_KEY: Symbol = symbol_short!("UPTLK");
const TIMELOCK_PENDING_KEY: Symbol = symbol_short!("UPTLDP");
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
    WasmHashMismatch = 6,
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
    pub wasm_hash: BytesN<32>,
    pub proposed_at: u64,
    pub effective_at: u64,
    pub proposer: Address,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UpgradeHistoryEntry {
    pub from_version: u32,
    pub to_version: u32,
    pub wasm_hash: BytesN<32>,
    pub executed_at: u64,
    pub executor: Address,
}

/// A requested timelock reduction that has not taken effect yet.
///
/// Lowering the timelock is itself subject to the current timelock: the new
/// value only becomes active at `effective_at`, so an attacker who compromises
/// the admin key cannot shrink the warning window and exploit it in a single
/// ledger. Increases apply immediately and never create a pending change.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TimelockChange {
    pub old_timelock: u64,
    pub new_timelock: u64,
    pub requested_at: u64,
    pub effective_at: u64,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UpgradeProposedEvent {
    pub target_version: u32,
    pub wasm_hash: BytesN<32>,
    pub proposed_at: u64,
    pub effective_at: u64,
    pub proposer: Address,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UpgradeExecutedEvent {
    pub from_version: u32,
    pub to_version: u32,
    pub wasm_hash: BytesN<32>,
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
        env.storage().instance().set(&UPGRADE_ADMIN_KEY, admin);
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

    /// Returns the currently effective upgrade timelock in seconds.
    ///
    /// Lazily applies any pending timelock reduction whose delay has elapsed.
    /// Defaults to `0` when never configured; upgrade entry points reject
    /// proposals while the effective timelock is below
    /// [`MIN_UPGRADE_TIMELOCK_SECONDS`].
    pub fn get_timelock_seconds(env: &Env) -> u64 {
        Self::apply_pending_timelock_change(env, env.ledger().timestamp());
        env.storage().instance().get(&TIMELOCK_KEY).unwrap_or(0)
    }

    /// Returns the pending timelock reduction, if any, that has not yet taken
    /// effect. `None` means no change is queued (or it already applied).
    pub fn get_pending_timelock_change(env: &Env) -> Option<TimelockChange> {
        env.storage().instance().get(&TIMELOCK_PENDING_KEY)
    }

    /// Sets the upgrade timelock to `seconds`.
    ///
    /// - Values below [`MIN_UPGRADE_TIMELOCK_SECONDS`] are rejected with
    ///   `UpgradeAuthError::InvalidTimelock`.
    /// - Increases (and no-op writes) apply immediately.
    /// - Reductions are stored as a pending [`TimelockChange`] that only takes
    ///   effect after the *current* timelock has elapsed from the request, so
    ///   users always keep at least the previously configured warning window.
    ///
    /// `now` must be the current ledger timestamp of the calling contract.
    pub fn set_timelock_seconds(env: &Env, now: u64, seconds: u64) -> Result<(), UpgradeAuthError> {
        if seconds < MIN_UPGRADE_TIMELOCK_SECONDS {
            return Err(UpgradeAuthError::InvalidTimelock);
        }
        let current = Self::get_timelock_seconds(env);
        if seconds < current {
            // Reduction: queue it behind the current timelock.
            env.storage().instance().set(
                &TIMELOCK_PENDING_KEY,
                &TimelockChange {
                    old_timelock: current,
                    new_timelock: seconds,
                    requested_at: now,
                    effective_at: now.saturating_add(current),
                },
            );
        } else {
            // Increase or no-op: apply immediately and drop any stale pending
            // reduction so it cannot resurrect after the raise.
            env.storage().instance().remove(&TIMELOCK_PENDING_KEY);
            env.storage().instance().set(&TIMELOCK_KEY, &seconds);
        }
        Ok(())
    }

    /// Applies the pending timelock reduction once its delay has elapsed.
    /// Returns the newly applied timelock, or `None` if nothing was applied.
    pub fn apply_pending_timelock_change(env: &Env, now: u64) -> Option<u64> {
        let change: TimelockChange = env.storage().instance().get(&TIMELOCK_PENDING_KEY)?;
        if now < change.effective_at {
            return None;
        }
        env.storage()
            .instance()
            .set(&TIMELOCK_KEY, &change.new_timelock);
        env.storage().instance().remove(&TIMELOCK_PENDING_KEY);
        Some(change.new_timelock)
    }

    pub fn get_proposal(env: &Env) -> Option<UpgradeProposal> {
        env.storage().instance().get(&PROPOSAL_KEY)
    }

    /// Proposes an upgrade to `target_version`, enforceable after the
    /// effective timelock elapses.
    ///
    /// Fails with `UpgradeAuthError::InvalidTimelock` unless the effective
    /// timelock is at least [`MIN_UPGRADE_TIMELOCK_SECONDS`], so a migration
    /// can never be proposed and executed in the same ledger.
    pub fn propose_upgrade(
        env: &Env,
        proposer: &Address,
        target_version: u32,
        wasm_hash: BytesN<32>,
        now: u64,
    ) -> Result<UpgradeProposal, UpgradeAuthError> {
        let timelock = Self::get_timelock_seconds(env);
        if timelock < MIN_UPGRADE_TIMELOCK_SECONDS {
            return Err(UpgradeAuthError::InvalidTimelock);
        }
        let proposal = UpgradeProposal {
            target_version,
            wasm_hash,
            proposed_at: now,
            effective_at: now.saturating_add(timelock),
            proposer: proposer.clone(),
        };
        env.storage().instance().set(&PROPOSAL_KEY, &proposal);
        Ok(proposal)
    }

    pub fn validate_upgrade(
        env: &Env,
        wasm_hash: &BytesN<32>,
        now: u64,
    ) -> Result<UpgradeProposal, UpgradeAuthError> {
        let timelock = Self::get_timelock_seconds(env);
        if timelock < MIN_UPGRADE_TIMELOCK_SECONDS {
            return Err(UpgradeAuthError::InvalidTimelock);
        }
        let proposal = Self::get_proposal(env).ok_or(UpgradeAuthError::NoProposal)?;
        if proposal.wasm_hash != *wasm_hash {
            return Err(UpgradeAuthError::WasmHashMismatch);
        }
        if now < proposal.effective_at {
            return Err(UpgradeAuthError::TimelockPending);
        }
        Ok(proposal)
    }

    pub fn clear_proposal(env: &Env) {
        env.storage().instance().remove(&PROPOSAL_KEY);
    }

    pub fn validate_execution(
        env: &Env,
        target_version: u32,
        now: u64,
    ) -> Result<UpgradeProposal, UpgradeAuthError> {
        // Defense in depth: refuse execution if the effective timelock has
        // fallen below the minimum (e.g. legacy state predating the guard).
        let timelock = Self::get_timelock_seconds(env);
        if timelock < MIN_UPGRADE_TIMELOCK_SECONDS {
            return Err(UpgradeAuthError::InvalidTimelock);
        }
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
        let count: u32 = env.storage().persistent().get(&HIST_COUNT_KEY).unwrap_or(0);
        let key = (symbol_short!("UPHIS"), count);
        env.storage().persistent().set(&key, entry);
        env.storage()
            .persistent()
            .set(&HIST_COUNT_KEY, &(count + 1));
    }

    pub fn get_history(env: &Env, offset: u32, limit: u32) -> Vec<UpgradeHistoryEntry> {
        let count: u32 = env.storage().persistent().get(&HIST_COUNT_KEY).unwrap_or(0);
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
        env.storage().persistent().get(&HIST_COUNT_KEY).unwrap_or(0)
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

    pub fn finalize_upgrade_run(
        env: &Env,
        executor: &Address,
        from_version: u32,
        to_version: u32,
        wasm_hash: &BytesN<32>,
        now: u64,
    ) {
        Self::clear_proposal(env);
        Self::record_execution(
            env,
            &UpgradeHistoryEntry {
                from_version,
                to_version,
                wasm_hash: wasm_hash.clone(),
                executed_at: now,
                executor: executor.clone(),
            },
        );
    }

    pub fn finalize_migration_run(
        env: &Env,
        executor: &Address,
        from_version: u32,
        to_version: u32,
        now: u64,
    ) {
        let default_hash = BytesN::from_array(env, &[0u8; 32]);
        Self::finalize_upgrade_run(env, executor, from_version, to_version, &default_hash, now);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::testutils::{Address as _, Ledger as _};
    use soroban_sdk::{contract, contractimpl};

    /// Bare contract used to provide a storage context in unit tests.
    #[contract]
    pub struct TestContract;

    #[contractimpl]
    impl TestContract {}

    /// Initial ledger timestamp used across the tests below.
    const BASE_TS: u64 = 1_700_000_000;
    /// Arbitrary upgrade target version used when proposing.
    const TARGET: u32 = 4;

    /// Sets up an `Env` with a registered test contract and returns both, so
    /// storage-backed helpers can run inside `env.as_contract`.
    fn setup() -> (Env, Address) {
        let env = Env::default();
        let contract_id = env.register(TestContract, ());
        env.ledger().set_timestamp(BASE_TS);
        (env, contract_id)
    }

    fn set_time(env: &Env, ts: u64) {
        env.ledger().set_timestamp(ts);
    }

    // -----------------------------------------------------------------------
    // Acceptance criterion 1: enforce a non-zero minimum timelock
    // -----------------------------------------------------------------------

    /// Fresh storage defaults the timelock to 0. Proposing an upgrade must be
    /// rejected until the admin configures a compliant timelock, so a
    /// migration can never be proposed and executed in the same ledger.
    fn dummy_hash(env: &Env) -> BytesN<32> {
        BytesN::from_array(env, &[1u8; 32])
    }

    #[test]
    fn default_timelock_zero_rejects_proposal() {
        let (env, contract_id) = setup();
        let proposer = Address::generate(&env);
        let hash = dummy_hash(&env);
        env.as_contract(&contract_id, || {
            assert_eq!(UpgradeAuthorization::get_timelock_seconds(&env), 0);
            assert_eq!(
                UpgradeAuthorization::propose_upgrade(&env, &proposer, TARGET, hash, BASE_TS),
                Err(UpgradeAuthError::InvalidTimelock)
            );
            assert!(UpgradeAuthorization::get_proposal(&env).is_none());
        });
    }

    /// The reported attack path: prepare (and run) a migration in the same
    /// ledger while the timelock is still 0. Execution must be refused.
    #[test]
    fn default_timelock_blocks_same_ledger_migration_run() {
        let (env, contract_id) = setup();
        env.mock_all_auths();
        let admin = Address::generate(&env);
        env.as_contract(&contract_id, || {
            assert_eq!(
                UpgradeAuthorization::prepare_migration_run(
                    &env,
                    &admin,
                    Some(admin.clone()),
                    TARGET,
                    false,
                    BASE_TS,
                ),
                Err(UpgradeAuthError::InvalidTimelock)
            );
        });
    }

    /// The timelock itself can never be configured below the minimum, and a
    /// rejected write leaves both the effective and pending state untouched.
    #[test]
    fn timelock_below_minimum_rejected_and_unchanged() {
        let (env, contract_id) = setup();
        env.as_contract(&contract_id, || {
            assert_eq!(
                UpgradeAuthorization::set_timelock_seconds(&env, BASE_TS, 0),
                Err(UpgradeAuthError::InvalidTimelock)
            );
            assert_eq!(
                UpgradeAuthorization::set_timelock_seconds(
                    &env,
                    BASE_TS,
                    MIN_UPGRADE_TIMELOCK_SECONDS - 1
                ),
                Err(UpgradeAuthError::InvalidTimelock)
            );
            assert_eq!(UpgradeAuthorization::get_timelock_seconds(&env), 0);
            assert!(UpgradeAuthorization::get_pending_timelock_change(&env).is_none());
        });
    }

    /// Exactly at the minimum the timelock is accepted: proposals are created
    /// with a full warning window and cannot execute before it elapses.
    #[test]
    fn timelock_at_minimum_allows_proposal_and_delayed_execution() {
        let (env, contract_id) = setup();
        let proposer = Address::generate(&env);
        let hash = dummy_hash(&env);
        env.as_contract(&contract_id, || {
            UpgradeAuthorization::set_timelock_seconds(&env, BASE_TS, MIN_UPGRADE_TIMELOCK_SECONDS)
                .unwrap();

            let proposal = UpgradeAuthorization::propose_upgrade(
                &env,
                &proposer,
                TARGET,
                hash.clone(),
                BASE_TS,
            )
            .unwrap();
            assert_eq!(
                proposal.effective_at,
                BASE_TS + MIN_UPGRADE_TIMELOCK_SECONDS
            );

            set_time(&env, proposal.effective_at - 1);
            assert_eq!(
                UpgradeAuthorization::validate_execution(&env, TARGET, proposal.effective_at - 1),
                Err(UpgradeAuthError::TimelockPending)
            );

            set_time(&env, proposal.effective_at);
            assert!(
                UpgradeAuthorization::validate_execution(&env, TARGET, proposal.effective_at)
                    .is_ok()
            );
        });
    }

    // -----------------------------------------------------------------------
    // Acceptance criterion 2: reductions are subject to the current timelock
    // -----------------------------------------------------------------------

    /// A reduction must not take effect immediately: the old timelock keeps
    /// governing proposals until `requested_at + current_timelock`, so users
    /// never lose the warning window they were promised.
    #[test]
    fn timelock_reduction_is_delayed_by_current_timelock() {
        let (env, contract_id) = setup();
        let initial: u64 = 100_000;
        let reduced: u64 = 90_000;
        env.as_contract(&contract_id, || {
            UpgradeAuthorization::set_timelock_seconds(&env, BASE_TS, initial).unwrap();
        });

        // Request a reduction once the initial window has passed.
        let request_at = BASE_TS + initial + 10;
        set_time(&env, request_at);
        env.as_contract(&contract_id, || {
            UpgradeAuthorization::set_timelock_seconds(&env, request_at, reduced).unwrap();

            // Old timelock still applies while the change is pending.
            let pending = UpgradeAuthorization::get_pending_timelock_change(&env).unwrap();
            assert_eq!(pending.old_timelock, initial);
            assert_eq!(pending.new_timelock, reduced);
            assert_eq!(pending.effective_at, request_at + initial);
            assert_eq!(UpgradeAuthorization::get_timelock_seconds(&env), initial);

            // Proposals made meanwhile still use the old (longer) window.
            let proposer = Address::generate(&env);
            let hash = dummy_hash(&env);
            let proposal =
                UpgradeAuthorization::propose_upgrade(&env, &proposer, TARGET, hash, request_at)
                    .unwrap();
            assert_eq!(proposal.effective_at, request_at + initial);
        });

        // The reduction only lands after the full current-timelock delay...
        set_time(&env, request_at + initial - 1);
        env.as_contract(&contract_id, || {
            assert_eq!(UpgradeAuthorization::get_timelock_seconds(&env), initial);
        });
        set_time(&env, request_at + initial);
        env.as_contract(&contract_id, || {
            assert_eq!(UpgradeAuthorization::get_timelock_seconds(&env), reduced);
            assert!(UpgradeAuthorization::get_pending_timelock_change(&env).is_none());
        });

        // ...and a further reduction is delayed by the new timelock in turn.
        let second_request = request_at + initial + 5;
        set_time(&env, second_request);
        env.as_contract(&contract_id, || {
            UpgradeAuthorization::set_timelock_seconds(
                &env,
                second_request,
                MIN_UPGRADE_TIMELOCK_SECONDS,
            )
            .unwrap();
            let pending = UpgradeAuthorization::get_pending_timelock_change(&env).unwrap();
            assert_eq!(pending.old_timelock, reduced);
            assert_eq!(pending.effective_at, second_request + reduced);
            assert_eq!(UpgradeAuthorization::get_timelock_seconds(&env), reduced);
        });
    }

    /// Raising the timelock is not delayed, and cancels any pending reduction
    /// so it cannot resurrect after the raise.
    #[test]
    fn timelock_increase_applies_immediately_and_cancels_pending_reduction() {
        let (env, contract_id) = setup();
        let initial: u64 = 100_000;
        env.as_contract(&contract_id, || {
            UpgradeAuthorization::set_timelock_seconds(&env, BASE_TS, initial).unwrap();
        });

        let request_at = BASE_TS + initial + 10;
        set_time(&env, request_at);
        env.as_contract(&contract_id, || {
            // Queue a reduction, then supersede it with an increase.
            UpgradeAuthorization::set_timelock_seconds(&env, request_at, 90_000).unwrap();
            assert!(UpgradeAuthorization::get_pending_timelock_change(&env).is_some());

            UpgradeAuthorization::set_timelock_seconds(&env, request_at, initial * 2).unwrap();
            assert_eq!(
                UpgradeAuthorization::get_timelock_seconds(&env),
                initial * 2
            );
            assert!(UpgradeAuthorization::get_pending_timelock_change(&env).is_none());
        });

        // The cancelled reduction must not apply later.
        set_time(&env, request_at + initial * 3);
        env.as_contract(&contract_id, || {
            assert_eq!(
                UpgradeAuthorization::get_timelock_seconds(&env),
                initial * 2
            );
            assert!(UpgradeAuthorization::get_pending_timelock_change(&env).is_none());
        });
    }

    /// Once a pending reduction matures, new proposals are bound by the new
    /// (shorter) window — which itself can never go below the minimum.
    #[test]
    fn proposal_after_reduction_applies_new_window() {
        let (env, contract_id) = setup();
        let initial: u64 = 100_000;
        let reduced: u64 = MIN_UPGRADE_TIMELOCK_SECONDS * 2;
        env.as_contract(&contract_id, || {
            UpgradeAuthorization::set_timelock_seconds(&env, BASE_TS, initial).unwrap();
        });

        let request_at = BASE_TS + initial + 10;
        set_time(&env, request_at);
        env.as_contract(&contract_id, || {
            UpgradeAuthorization::set_timelock_seconds(&env, request_at, reduced).unwrap();
        });

        let applied_at = request_at + initial;
        set_time(&env, applied_at);
        env.as_contract(&contract_id, || {
            let proposer = Address::generate(&env);
            let hash = dummy_hash(&env);
            let proposal =
                UpgradeAuthorization::propose_upgrade(&env, &proposer, TARGET, hash, applied_at)
                    .unwrap();
            assert_eq!(proposal.effective_at, applied_at + reduced);
        });
    }

    // -----------------------------------------------------------------------
    // Issue #1060: WASM upgrade proposal and execution security tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_1_authorized_proposal_succeeds() {
        let (env, contract_id) = setup();
        let admin = Address::generate(&env);
        let hash_a = BytesN::from_array(&env, &[10u8; 32]);
        env.as_contract(&contract_id, || {
            UpgradeAuthorization::set_upgrade_admin(&env, &admin);
            UpgradeAuthorization::set_timelock_seconds(&env, BASE_TS, MIN_UPGRADE_TIMELOCK_SECONDS)
                .unwrap();

            let proposal = UpgradeAuthorization::propose_upgrade(
                &env,
                &admin,
                TARGET,
                hash_a.clone(),
                BASE_TS,
            )
            .unwrap();
            assert_eq!(proposal.target_version, TARGET);
            assert_eq!(proposal.wasm_hash, hash_a);
            assert_eq!(proposal.proposer, admin);
            assert_eq!(
                proposal.effective_at,
                BASE_TS + MIN_UPGRADE_TIMELOCK_SECONDS
            );
        });
    }

    #[test]
    fn test_2_unauthorized_proposal_fails() {
        let (env, contract_id) = setup();
        env.mock_all_auths();
        let admin = Address::generate(&env);
        let non_admin = Address::generate(&env);
        env.as_contract(&contract_id, || {
            UpgradeAuthorization::set_upgrade_admin(&env, &admin);
            UpgradeAuthorization::set_timelock_seconds(&env, BASE_TS, MIN_UPGRADE_TIMELOCK_SECONDS)
                .unwrap();

            assert_eq!(
                UpgradeAuthorization::require_admin(&env, &non_admin, Some(admin)),
                Err(UpgradeAuthError::Unauthorized)
            );
        });
    }

    #[test]
    fn test_3_timelock_enforced_before_expiration() {
        let (env, contract_id) = setup();
        let admin = Address::generate(&env);
        let hash_a = BytesN::from_array(&env, &[0xAA; 32]);
        env.as_contract(&contract_id, || {
            UpgradeAuthorization::set_upgrade_admin(&env, &admin);
            UpgradeAuthorization::set_timelock_seconds(&env, BASE_TS, MIN_UPGRADE_TIMELOCK_SECONDS)
                .unwrap();

            let proposal = UpgradeAuthorization::propose_upgrade(
                &env,
                &admin,
                TARGET,
                hash_a.clone(),
                BASE_TS,
            )
            .unwrap();

            // 1 second before effective_at: must fail with TimelockPending
            set_time(&env, proposal.effective_at - 1);
            assert_eq!(
                UpgradeAuthorization::validate_upgrade(&env, &hash_a, proposal.effective_at - 1),
                Err(UpgradeAuthError::TimelockPending)
            );
        });
    }

    #[test]
    fn test_4_hash_mismatch_fails() {
        let (env, contract_id) = setup();
        let admin = Address::generate(&env);
        let hash_a = BytesN::from_array(&env, &[0xAA; 32]);
        let hash_b = BytesN::from_array(&env, &[0xBB; 32]);
        env.as_contract(&contract_id, || {
            UpgradeAuthorization::set_upgrade_admin(&env, &admin);
            UpgradeAuthorization::set_timelock_seconds(&env, BASE_TS, MIN_UPGRADE_TIMELOCK_SECONDS)
                .unwrap();

            let proposal =
                UpgradeAuthorization::propose_upgrade(&env, &admin, TARGET, hash_a, BASE_TS)
                    .unwrap();

            set_time(&env, proposal.effective_at);
            // Attempting upgrade with hash_b must fail with WasmHashMismatch
            assert_eq!(
                UpgradeAuthorization::validate_upgrade(&env, &hash_b, proposal.effective_at),
                Err(UpgradeAuthError::WasmHashMismatch)
            );
        });
    }

    #[test]
    fn test_5_successful_upgrade_validation() {
        let (env, contract_id) = setup();
        let admin = Address::generate(&env);
        let hash_a = BytesN::from_array(&env, &[0xAA; 32]);
        env.as_contract(&contract_id, || {
            UpgradeAuthorization::set_upgrade_admin(&env, &admin);
            UpgradeAuthorization::set_timelock_seconds(&env, BASE_TS, MIN_UPGRADE_TIMELOCK_SECONDS)
                .unwrap();

            let proposal = UpgradeAuthorization::propose_upgrade(
                &env,
                &admin,
                TARGET,
                hash_a.clone(),
                BASE_TS,
            )
            .unwrap();

            set_time(&env, proposal.effective_at);
            let validated =
                UpgradeAuthorization::validate_upgrade(&env, &hash_a, proposal.effective_at)
                    .unwrap();
            assert_eq!(validated.wasm_hash, hash_a);
        });
    }

    #[test]
    fn test_6_replay_execution_fails() {
        let (env, contract_id) = setup();
        let admin = Address::generate(&env);
        let hash_a = BytesN::from_array(&env, &[0xAA; 32]);
        env.as_contract(&contract_id, || {
            UpgradeAuthorization::set_upgrade_admin(&env, &admin);
            UpgradeAuthorization::set_timelock_seconds(&env, BASE_TS, MIN_UPGRADE_TIMELOCK_SECONDS)
                .unwrap();

            let proposal = UpgradeAuthorization::propose_upgrade(
                &env,
                &admin,
                TARGET,
                hash_a.clone(),
                BASE_TS,
            )
            .unwrap();

            set_time(&env, proposal.effective_at);
            assert!(
                UpgradeAuthorization::validate_upgrade(&env, &hash_a, proposal.effective_at)
                    .is_ok()
            );

            // Finalize upgrade run (clears proposal and records history)
            UpgradeAuthorization::finalize_upgrade_run(
                &env,
                &admin,
                1,
                TARGET,
                &hash_a,
                proposal.effective_at,
            );

            // Replay attempt: proposal no longer exists
            assert_eq!(
                UpgradeAuthorization::validate_upgrade(&env, &hash_a, proposal.effective_at),
                Err(UpgradeAuthError::NoProposal)
            );
        });
    }

    #[test]
    fn test_7_upgrade_history_records_exact_wasm_hash() {
        let (env, contract_id) = setup();
        let admin = Address::generate(&env);
        let hash_a = BytesN::from_array(&env, &[0xAA; 32]);
        env.as_contract(&contract_id, || {
            UpgradeAuthorization::set_upgrade_admin(&env, &admin);
            UpgradeAuthorization::set_timelock_seconds(&env, BASE_TS, MIN_UPGRADE_TIMELOCK_SECONDS)
                .unwrap();

            let proposal = UpgradeAuthorization::propose_upgrade(
                &env,
                &admin,
                TARGET,
                hash_a.clone(),
                BASE_TS,
            )
            .unwrap();

            set_time(&env, proposal.effective_at);
            UpgradeAuthorization::finalize_upgrade_run(
                &env,
                &admin,
                1,
                TARGET,
                &hash_a,
                proposal.effective_at,
            );

            let history = UpgradeAuthorization::get_history(&env, 0, 10);
            assert_eq!(history.len(), 1);
            let entry = history.get(0).unwrap();
            assert_eq!(entry.from_version, 1);
            assert_eq!(entry.to_version, TARGET);
            assert_eq!(entry.wasm_hash, hash_a);
            assert_eq!(entry.executed_at, proposal.effective_at);
            assert_eq!(entry.executor, admin);
        });
    }
}
