# ADR 006: Reward Pool Sponsorship

## Status

Accepted

## Context

`fund_reward_pool` took a `funder: Address` parameter but then required
`funder == pool_config.creator`, so the parameter could only ever hold one
value and carried no real information. This foreclosed sponsorship — a brand
funding a community hunt, a DAO topping up a pool, several people pooling a
prize — even though the signature suggested it was supported. Today, each of
those parties would have to send tokens to the creator off-chain and trust
them to forward it into the pool.

It was also inconsistent with `refund_pool`, which sent the pool's entire
balance to the creator. If third-party funding were allowed without changing
`refund_pool`, a sponsor's contribution would be refundable to someone else
entirely.

(Filed as issue #869.)

## Decision

- **Sponsorship is enabled.** `fund_reward_pool` no longer restricts the
  `funder` to the pool creator — any address may fund a pool, and must
  authorize the call itself (`funder.require_auth()`).
- Each funder's cumulative, not-yet-refunded contribution is tracked
  per-pool (`Storage::get/set_pool_funder_contribution`, and a
  `Storage::get/set_pool_funders` list of distinct contributors), bounded by
  `MAX_FUNDERS_PER_POOL` (50) distinct funders per pool.
- **`refund_pool` pays out pro rata.** It still can only be triggered by the
  pool creator (now enforced via `creator.require_auth()`, which was
  previously missing entirely), but the current balance is split across the
  tracked funders in proportion to their share of total contributions,
  rather than being sent wholesale to the creator. A pool funded by a single
  address — the common, pre-existing case — simply gets its whole balance
  back, so this is behavior-preserving for the majority of pools.
- Integer-division remainder from the pro-rata split is assigned to the last
  funder in iteration order, so the full balance is always paid out and
  never left stranded in the contract; this rounding dust is at most
  `funders - 1` base units, spread across payouts that are already
  proportional.
- **`migrate_pool`** re-keys a fully expired/cancelled source pool's balance
  into a destination pool the same creator owns. The migrated lump sum is
  attributed to the creator as a funder contribution on the destination
  (since it's the creator's own decision to move it), and the source pool's
  funder ledger is cleared — otherwise a later refund on either pool could
  double-count or misattribute contributions that already moved.

## Consequences

- Positive: hunts can be crowd-funded or sponsored without an off-chain
  trust relationship with the creator.
- Positive: `refund_pool` can no longer send a sponsor's money to the
  creator (or vice versa), closing the gap the audit flagged.
- Positive: `refund_pool` gained the `creator.require_auth()` check it was
  missing, so a refund (now a multi-party payout) can't be triggered by an
  arbitrary caller.
- Negative: refunding a sponsored pool now iterates its funder list and
  performs one token transfer per funder with a nonzero share, instead of a
  single transfer. This is bounded by `MAX_FUNDERS_PER_POOL` (50).
- Negative: a new `TooManyFunders` error can reject funding from a
  51st distinct sponsor on the same pool; existing funders can still top up.

## Alternatives Considered

- **Restrict funding to the creator, drop the `funder` parameter**: the
  simpler option from the audit. Rejected because sponsorship is a natural
  fit for the product (community hunts, DAO-funded prizes) and the
  parameter already implied it was supported.
- **Track only a running total, refund the full balance to whoever calls
  `refund_pool`**: rejected — it doesn't solve the underlying problem of a
  sponsor's contribution being payable to someone else.
- **Give sponsors their own `withdraw_my_contribution` entry point** instead
  of a creator-triggered pro-rata refund: deferred. It would let a sponsor
  pull funds mid-hunt, which conflicts with the existing model where the
  creator controls when a pool is wound down (via `refund_pool`) versus kept
  live for distributions.
