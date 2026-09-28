# ADR 005: View-Only Access Control

## Status

Accepted

## Context

Several query functions in HuntyCore — most notably `get_hunt_leaderboard` and
`get_hunt_leaderboard_window` (issue #398) — are intended to be callable only by
authorised parties in contexts where hunt data is commercially sensitive or the
creator wants to gate leaderboard visibility. A mechanism was needed that:

- lets individual hunt creators grant and revoke per-hunt read access to specific
  addresses without involving the contract admin
- lets the contract admin grant and revoke read access across every hunt (for
  operators, analytics pipelines, and similar infrastructure roles)
- does not require callers to submit transactions that alter state just to read data

A written design was absent when this was implemented, which contributed to an
integration defect: the access management functions (`add_view_only_access` and
friends) were at one point placed outside the `#[contractimpl]` block in a draft
of `lib.rs`, making them invisible as contract entry points without a compile-time
error. Without a spec that listed every function that must be inside the impl, that
placement was not caught until runtime integration testing.

## Decision

Two independent viewer lists are maintained:

**Per-hunt viewers** — managed by the hunt creator.

- `add_view_only_access(hunt_id, creator, viewer)` — grants `viewer` access to
  `hunt_id`. Only the hunt creator may call this; the function verifies
  `hunt.creator == creator` after `creator.require_auth()`.
- `remove_view_only_access(hunt_id, creator, viewer)` — revokes access under the
  same guard.
- `is_view_only(hunt_id, address) → bool` — read-only membership check.
- `get_view_only_list(hunt_id, offset, limit) → Vec<Address>` — returns a page
  of the list for a hunt (`limit` capped at `MAX_BATCH_SIZE`).

Storage keys: `("V", hunt_id)` keeps the ordered enumeration list, while a
per-address marker `("VOMEM", hunt_id, address)` makes `is_view_only` an O(1)
lookup that never scans the list. Both lists are bounded by
`MAX_VIEW_ONLY_ENTRIES`.

**Global viewers** — managed by the contract admin.

- `add_global_view_only(admin, viewer)` — grants viewer access to all hunts.
  Requires admin auth and verifies the caller is the stored admin.
- `remove_global_view_only(admin, viewer)` — revokes global access.
- `is_global_view_only(address) → bool` — read-only check.
- `get_global_view_only_list(offset, limit) → Vec<Address>` — returns a page of
  the global list (`limit` capped at `MAX_BATCH_SIZE`).

Storage keys: `"GV"` keeps the ordered enumeration list, while a per-address
marker `("GVOMEM", address)` makes `is_global_view_only` an O(1) lookup. The
list is bounded by `MAX_VIEW_ONLY_ENTRIES`.

**Enforcement in query functions** — callers of access-gated queries must pass
their address. The function checks `is_view_only || is_global_view_only` and
returns `Unauthorized` if neither condition is met. Functions that are
unconditionally public (e.g. `get_hunt`, `list_clues`) carry no viewer check.

All eight access-control functions must reside inside the single
`#[contractimpl] impl HuntyCore` block. Any helper that lives outside that block
is not exported as a contract entry point and will silently fail to be callable.
This requirement is enforced by the Soroban SDK's macro — only `pub fn` items
inside `#[contractimpl]` are wired into the WASM dispatch table.

## Consequences

- Hunt creators have fine-grained control over who can view their hunt's
  leaderboard and statistics without admin involvement.
- The admin can grant platform-wide read access to infrastructure accounts without
  touching every hunt individually.
- Per-hunt viewer lists are stored in instance storage as `Vec<Address>`. This is
  consistent with other small config lists in the contract, but the list is
  unbounded in code. Hunts with large viewer lists will pay increasing instance
  storage costs and slower membership checks (`O(n)` linear scan). A practical
  cap (e.g. 50 addresses per hunt) should be enforced in a follow-up.
- Global viewer list has the same `O(n)` scan characteristic. Because it is
  expected to remain very small (operator accounts only), this is acceptable.
- Storage keys `"V"` and `"GV"` are single- and two-character symbols. They do
  not collide with any other key in the current key table (see ADR 002), but they
  are short enough that a future key addition could accidentally conflict. Both
  keys should be included in the key-table documentation in `DEVELOPMENT.md`.
- The `is_view_only` and `is_global_view_only` predicate functions are public
  contract entry points. External callers can query membership without requiring
  auth. This is intentional — membership in a viewer list is not sensitive — but
  it means the list contents are observable on-chain regardless.

## Alternatives Considered

- **Role-based access (enum roles per address)**: more expressive but adds
  complexity for a feature that only needs two levels (per-hunt and global).
  Deferred until a third role is needed.
- **Persistent storage for viewer lists**: considered for consistency with player
  progress (ADR 002), but viewer lists are small, change infrequently, and are
  read together with other instance-stored config, making instance storage the
  lower-overhead choice. If lists grow large, migrating to persistent storage with
  per-entry keys (as done for NFT owner indexes) is the upgrade path.
- **No access control on query functions**: simplest option. Rejected because
  creators need a way to run private pre-launch hunts and hide leaderboards until
  a reveal moment, without making answers visible to competitors.
- **Merkle-proof or off-chain signatures**: overkill for this use case; adds
  client complexity with no meaningful security benefit over on-chain lists of this
  size.
