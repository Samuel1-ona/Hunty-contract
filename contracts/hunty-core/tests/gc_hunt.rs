//! Storage garbage collection for terminal hunts (issue #446).
//!
//! A cancelled hunt used to keep every clue, player-progress, team, leaderboard
//! and bookkeeping entry it ever wrote. Nothing referenced them; nothing removed
//! them. These tests pin down the four acceptance criteria:
//!
//!   1. all storage keys for a hunt are identified,
//!   2. `gc_hunt` removes all related data,
//!   3. only cancelled/archived hunts may be collected,
//!   4. storage reclaimed is reported.
//!
//! Written as a standalone integration target because the crate's `src/test.rs`
//! unit-test module does not currently compile — see the PR description.

use hunty_core::types::HuntStatus;
use hunty_core::HuntyCore;
use soroban_sdk::testutils::{Address as _, Ledger as _};
use soroban_sdk::{Address, Env, String};

fn as_core<T>(env: &Env, contract_id: &Address, f: impl FnOnce(&Env) -> T) -> T {
    env.as_contract(contract_id, || f(env))
}

struct Fixture {
    env: Env,
    contract_id: Address,
    creator: Address,
    admin: Address,
    players: [Address; 3],
    hunt_id: u64,
}

/// Builds a hunt with clues, registered players and recorded progress, so the
/// sweep has a realistic key surface to clear rather than a bare hunt row.
fn setup_active_hunt() -> Fixture {
    let env = Env::default();
    env.ledger().set_timestamp(1_700_000_000);
    env.mock_all_auths();

    let contract_id = env.register(HuntyCore, ());
    let creator = Address::generate(&env);
    let admin = Address::generate(&env);
    let players = [
        Address::generate(&env),
        Address::generate(&env),
        Address::generate(&env),
    ];

    as_core(&env, &contract_id, |env| {
        HuntyCore::initialize_admin(env.clone(), admin.clone()).unwrap();
    });

    let hunt_id = as_core(&env, &contract_id, |env| {
        HuntyCore::create_hunt(
            env.clone(),
            creator.clone(),
            String::from_str(env, "Collectible Hunt"),
            String::from_str(env, "A hunt that will be cancelled"),
            None,
            None,
            0,
            None,
            None,
        )
        .unwrap()
    });

    // Three clues, all required.
    as_core(&env, &contract_id, |env| {
        for i in 0..3 {
            HuntyCore::add_clue(
                env.clone(),
                hunt_id,
                String::from_str(env, "Question"),
                String::from_str(env, "answer"),
                10,
                true,
                None,
                None,
            )
            .unwrap();
            let _ = i;
        }
        HuntyCore::activate_hunt(env.clone(), hunt_id, creator.clone()).unwrap();
    });

    // Register all three players so player entries, exist-markers and counters exist.
    for player in players.iter() {
        as_core(&env, &contract_id, |env| {
            HuntyCore::register_player(env.clone(), hunt_id, player.clone()).unwrap();
        });
    }

    Fixture {
        env,
        contract_id,
        creator,
        admin,
        players,
        hunt_id,
    }
}

fn cancel(fx: &Fixture) {
    as_core(&fx.env, &fx.contract_id, |env| {
        HuntyCore::cancel_hunt(env.clone(), fx.hunt_id, fx.creator.clone()).unwrap();
    });
}

// ── Criterion 1 & 4: identify keys and report what was reclaimed ────────────

#[test]
fn footprint_reports_entries_before_collection() {
    let fx = setup_active_hunt();

    let footprint = as_core(&fx.env, &fx.contract_id, |env| {
        HuntyCore::get_hunt_storage_footprint(env.clone(), fx.hunt_id)
    });

    assert_eq!(footprint.hunt_id, fx.hunt_id);
    assert_eq!(footprint.players_swept, 3, "three players were registered");
    assert_eq!(footprint.clues_swept, 3, "three clues were added");
    assert!(
        footprint.total_removed > 0,
        "an active hunt must own storage entries"
    );
    assert_eq!(
        footprint.total_removed,
        footprint.persistent_removed + footprint.instance_removed,
        "total must be the sum of both tiers"
    );
}

#[test]
fn footprint_is_read_only() {
    let fx = setup_active_hunt();

    let before = as_core(&fx.env, &fx.contract_id, |env| {
        HuntyCore::get_hunt_storage_footprint(env.clone(), fx.hunt_id)
    });
    let after = as_core(&fx.env, &fx.contract_id, |env| {
        HuntyCore::get_hunt_storage_footprint(env.clone(), fx.hunt_id)
    });

    assert_eq!(before.total_removed, after.total_removed);

    // The hunt itself must still be readable.
    let hunt = as_core(&fx.env, &fx.contract_id, |env| {
        HuntyCore::get_hunt_info(env.clone(), fx.hunt_id)
    });
    assert!(hunt.is_ok());
}

// ── Criterion 2: gc_hunt removes all related data ──────────────────────────

#[test]
fn gc_removes_every_entry_the_hunt_owned() {
    let fx = setup_active_hunt();

    let before = as_core(&fx.env, &fx.contract_id, |env| {
        HuntyCore::get_hunt_storage_footprint(env.clone(), fx.hunt_id)
    });

    cancel(&fx);

    let report = as_core(&fx.env, &fx.contract_id, |env| {
        HuntyCore::gc_hunt(env.clone(), fx.hunt_id, fx.creator.clone()).unwrap()
    });

    assert!(
        report.total_removed >= before.total_removed,
        "sweep removed {} entries but the footprint before cancelling was {}",
        report.total_removed,
        before.total_removed
    );

    // Nothing is left to collect.
    let after = as_core(&fx.env, &fx.contract_id, |env| {
        HuntyCore::get_hunt_storage_footprint(env.clone(), fx.hunt_id)
    });
    assert_eq!(after.total_removed, 0, "sweep must leave nothing behind");
}

#[test]
fn gc_removes_the_hunt_record_itself() {
    let fx = setup_active_hunt();
    cancel(&fx);

    as_core(&fx.env, &fx.contract_id, |env| {
        HuntyCore::gc_hunt(env.clone(), fx.hunt_id, fx.creator.clone()).unwrap()
    });

    let hunt = as_core(&fx.env, &fx.contract_id, |env| {
        HuntyCore::get_hunt_info(env.clone(), fx.hunt_id)
    });
    assert!(hunt.is_err(), "the hunt row must be gone after collection");
}

#[test]
fn gc_does_not_touch_a_sibling_hunt() {
    let fx = setup_active_hunt();

    // A second hunt that must survive untouched.
    let other_id = as_core(&fx.env, &fx.contract_id, |env| {
        HuntyCore::create_hunt(
            env.clone(),
            fx.creator.clone(),
            String::from_str(env, "Survivor"),
            String::from_str(env, "Must not be collected"),
            None,
            None,
            0,
            None,
            None,
        )
        .unwrap()
    });
    as_core(&fx.env, &fx.contract_id, |env| {
        HuntyCore::add_clue(
            env.clone(),
            other_id,
            String::from_str(env, "Q"),
            String::from_str(env, "a"),
            10,
            true,
            None,
            None,
        )
        .unwrap();
    });

    let other_before = as_core(&fx.env, &fx.contract_id, |env| {
        HuntyCore::get_hunt_storage_footprint(env.clone(), other_id)
    });

    cancel(&fx);
    as_core(&fx.env, &fx.contract_id, |env| {
        HuntyCore::gc_hunt(env.clone(), fx.hunt_id, fx.creator.clone()).unwrap()
    });

    let other_after = as_core(&fx.env, &fx.contract_id, |env| {
        HuntyCore::get_hunt_storage_footprint(env.clone(), other_id)
    });

    assert_eq!(
        other_before.total_removed, other_after.total_removed,
        "collecting one hunt must not disturb another"
    );
    assert!(as_core(&fx.env, &fx.contract_id, |env| {
        HuntyCore::get_hunt_info(env.clone(), other_id)
    })
    .is_ok());
}

// ── Criterion 3: only cancelled/archived hunts ─────────────────────────────

#[test]
fn gc_rejects_an_active_hunt() {
    let fx = setup_active_hunt();

    let result = as_core(&fx.env, &fx.contract_id, |env| {
        HuntyCore::gc_hunt(env.clone(), fx.hunt_id, fx.creator.clone())
    });

    assert!(
        result.is_err(),
        "collecting a live hunt would destroy player progress"
    );

    // And the hunt is untouched.
    assert!(as_core(&fx.env, &fx.contract_id, |env| {
        HuntyCore::get_hunt_info(env.clone(), fx.hunt_id)
    })
    .is_ok());
}

#[test]
fn gc_accepts_an_archived_hunt() {
    let fx = setup_active_hunt();
    cancel(&fx);

    as_core(&fx.env, &fx.contract_id, |env| {
        HuntyCore::archive_hunt(env.clone(), fx.hunt_id, fx.creator.clone()).unwrap();
    });

    let status = as_core(&fx.env, &fx.contract_id, |env| {
        HuntyCore::get_hunt_info(env.clone(), fx.hunt_id)
            .unwrap()
            .status
    });
    assert_eq!(status, HuntStatus::Archived);

    let report = as_core(&fx.env, &fx.contract_id, |env| {
        HuntyCore::gc_hunt(env.clone(), fx.hunt_id, fx.creator.clone()).unwrap()
    });
    assert!(report.total_removed > 0);
}

#[test]
fn gc_reports_not_found_for_an_unknown_hunt() {
    let fx = setup_active_hunt();

    let result = as_core(&fx.env, &fx.contract_id, |env| {
        HuntyCore::gc_hunt(env.clone(), 9_999, fx.creator.clone())
    });
    assert!(result.is_err());
}

// ── Authorization ──────────────────────────────────────────────────────────

#[test]
fn gc_allows_the_admin() {
    let fx = setup_active_hunt();
    cancel(&fx);

    let report = as_core(&fx.env, &fx.contract_id, |env| {
        HuntyCore::gc_hunt(env.clone(), fx.hunt_id, fx.admin.clone()).unwrap()
    });
    assert!(report.total_removed > 0);
}

#[test]
fn gc_rejects_an_unrelated_caller() {
    let fx = setup_active_hunt();
    cancel(&fx);

    let stranger = fx.players[0].clone();
    let result = as_core(&fx.env, &fx.contract_id, |env| {
        HuntyCore::gc_hunt(env.clone(), fx.hunt_id, stranger)
    });
    assert!(result.is_err(), "only the creator or admin may collect");

    // The hunt survives a rejected attempt.
    assert!(as_core(&fx.env, &fx.contract_id, |env| {
        HuntyCore::get_hunt_info(env.clone(), fx.hunt_id)
    })
    .is_ok());
}

// ── Idempotency ────────────────────────────────────────────────────────────

#[test]
fn gc_is_idempotent() {
    let fx = setup_active_hunt();
    cancel(&fx);

    let first = as_core(&fx.env, &fx.contract_id, |env| {
        HuntyCore::gc_hunt(env.clone(), fx.hunt_id, fx.creator.clone()).unwrap()
    });
    assert!(first.total_removed > 0);

    // The hunt row is gone, so a second call reports HuntNotFound rather than
    // failing in some other way — an interrupted sweep is safe to retry.
    let second = as_core(&fx.env, &fx.contract_id, |env| {
        HuntyCore::gc_hunt(env.clone(), fx.hunt_id, fx.creator.clone())
    });
    assert!(second.is_err());

    let footprint = as_core(&fx.env, &fx.contract_id, |env| {
        HuntyCore::get_hunt_storage_footprint(env.clone(), fx.hunt_id)
    });
    assert_eq!(footprint.total_removed, 0);
}
