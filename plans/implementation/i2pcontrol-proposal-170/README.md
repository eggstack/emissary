# Proposal 170 Implementation Handoffs

Status: partial Proposal 170 support; corrective sequence closed

This directory contains bounded internal implementation and closure handoffs for
the I2PControl Proposal 170 subsystem.

Authoritative direction:

- `plans/003-planning-process.md`
- `plans/adrs/ADR-0001-proposal-170-contract-and-stub-boundary.md`
- `plans/adrs/ADR-0002-control-plane-tunnel-runtime-ownership.md`
- `plans/subsystems/i2pcontrol-proposal-170-roadmap.md`
- `plans/closure/i2pcontrol-proposal-170/039-closure-invalidation.md`

Pinned external authority:

- Proposal 170 `I2PControl Expansion`, Open, created/updated `2026-05-20`
- existing I2PControl authentication and JSON-RPC contract

## Internal-only rule

All handoffs are internal to `eggstack/emissary`.

No plan authorizes:

- upstream issues, pull requests, reviews, discussions, submissions, adoption,
  or merge requests;
- pushing branches, commits, tags, patches, artifacts, or releases to an
  upstream remote;
- maintainer outreach or contribution-package preparation;
- connector/API writes against upstream or third-party repositories.

External specifications and reference implementations may be inspected
read-only. Violation is a stop condition and invalidates affected evidence.

## Current handoff

The corrective sequence is closed through M044. No implementation plan is
currently dependency-ready; deferred RouterInfo sources and unsupported tunnel
families remain outside this roadmap.

The sequence corrected the startup-managed generic server cancellation-owner
regression without changing control-plane ownership, core behavior, protocol,
or tunnel families.

## Corrective sequence

| Handoff | Status | Plan | Hard dependency |
|---|---|---|---|
| M040 — Startup server cancellation-owner correction | closed | `040-startup-server-cancellation-correction.md` | M039 invalidation recorded |
| M041 — Authentication throttle source/accounting correction | closed | `041-auth-throttle-source-accounting.md` | M040 closed |
| M042 — AddressBook subscription commit boundary | closed | `042-addressbook-subscription-commit-boundary.md` | M041 closed |
| M043 — Corrective runtime regression validation | closed | `043-corrective-runtime-regression-validation.md` | M040–M042 closed |
| M044 — Corrective final-head reclosure | closed | `044-corrective-final-head-reclosure.md` | M043 closed |

Only the registry may advance a successor to `ready` after its hard dependency
and closure evidence are accepted.

## Why M039 is invalidated

`plans/closure/i2pcontrol-proposal-170/039-closure-invalidation.md` records three
demonstrated defects:

1. the startup server manager drops its watch sender before entering the reusable
   runtime, allowing immediate self-cancellation;
2. failed-auth throttling is keyed by full `SocketAddr` and split across
   read/sleep/write, permitting ephemeral-port and concurrent-attempt bypass;
3. `SetSubscriptions` can commit durably and then return failure if refresh
   scheduling becomes unavailable.

M039's final status is non-controlling until M044. Unaffected M020–M039 evidence
remains retained.

## Retained capability boundary

The corrective sequence does not reopen:

- generic control-plane client/server backend architecture;
- per-name lifecycle supervision and fixed server-secret ownership;
- startup/control-plane ownership separation;
- explicit unsupported status for ten tunnel families;
- RouterInfo's 16 available / 1 neutral / 26 unavailable matrix;
- AddressBook entry owner coherence and feature isolation;
- direct/base compatibility inventories;
- constant-time password comparison;
- publication confinement/recovery/durability qualification;
- bounded passive SAM observation;
- internal-only/no-upstream governance.

## Runtime tunnel boundary

Under ADR-0002:

- generic `client` and `server` are the only real backends authorized by this
  roadmap;
- startup-managed client/server tunnels remain externally owned and read-only;
- control-plane-created definitions are supervised separately by I2PControl;
- HTTP, HTTP server/bidirectional server, IRC, SOCKS-IRC, CONNECT, Streamr, and
  other missing types remain explicit unsupported backends;
- existing HTTP/SOCKS startup services are not Proposal 170 I2PTunnel backends;
- no corrective plan may add core behavior or adopt startup tasks.

## Production budgets

### M040

Authorized production path:

- `emissary-cli/src/tunnel/server.rs`

Only cancellation-sender lifetime and directly related focused tests are in
scope.

### M041

Authorized production paths:

- `emissary-cli/src/i2pcontrol/auth.rs`
- `emissary-cli/src/i2pcontrol/server.rs`

Only source-IP normalization and atomic failed-auth reservation are in scope.

### M042

Authorized production paths:

- `emissary-cli/src/address_book.rs`
- narrowly related `emissary-cli/src/i2pcontrol/address_book_runtime.rs`
- `emissary-cli/src/i2pcontrol/address_book.rs` only if wire translation is
  directly affected

Only subscription mutation linearization and refresh-result semantics are in
scope.

### M043/M044

No production changes. A material defect requires a new implementation plan.

## Prohibited throughout

- new tunnel data planes;
- startup task adoption/control;
- new RouterInfo sources or fabricated values;
- router, transport, streaming, LeaseSet, cryptographic, routing, or tunnel
  algorithm changes;
- frontend work;
- repository-wide crate/service refactors;
- arbitrary request-selected paths;
- persistent accounts, proxy trust, distributed bans, or firewall integration;
- general AddressBook scheduler/event bus/second authority;
- `.github/workflows/**`, remote CI, release/publishing, coverage, fuzz, soak,
  platform matrices, or generated evidence bundles;
- upstream activity.

## Handoff discipline

Each implementation milestone must:

1. inspect the accepted dependency head;
2. add a failing regression for the demonstrated defect;
3. preserve unrelated retained evidence;
4. remain within its production budget;
5. run focused tests before the bounded broad matrix;
6. create an implementation disposition and independent closure record;
7. freeze the implementation/test head;
8. report unresolved findings with severity;
9. leave final subsystem status to M044;
10. attest that no upstream interaction occurred.

A code commit, compilation result, or broad test count is not closure by itself.

## Corrective stop rules

### M040

Stop rather than expose startup cancellation to I2PControl, modify core, alter
server identity, or refactor tunnel supervision broadly.

### M041

Stop rather than add accounts, persistent bans, proxy-header trust, global rate
limiting, firewall integration, or token/password redesign.

### M042

Stop rather than wait synchronously for downloads, add a scheduler/event bus,
create a second owner, or expose arbitrary paths/configuration.

### M043/M044

Validation and closure do not patch production. A defect requires a new plan.

## Verification rule

Normal bounded matrix:

```bash
cargo check -p emissary-cli --no-default-features
cargo test -p emissary-cli --no-default-features
cargo clippy -p emissary-cli --no-default-features --all-targets -- -D warnings
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo test -p emissary-cli --no-default-features --features i2pcontrol
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
cargo check -p emissary-core
cargo test -p emissary-core sam
git diff --check
```

Each plan adds focused commands. Use targeted formatting. Remote CI, release,
coverage, fuzz, soak, network farms, and generated evidence bundles are not
required.

## Corrective sequence disposition

The M039 invalidation was resolved by the serialized M040–M044 corrective
sequence. M040–M043 are closed with accepted evidence records; M044 is the
final independent reclosure. No deferred RouterInfo source or unsupported
tunnel-family plan became dependency-ready.

## Final-status rule

M044 may select:

- `partial Proposal 170 support` only if every implemented/claimed dimension is
  exact, operational, bounded, and evidenced;
- `corrective pass required` if any high/medium defect remains;
- `blocked` if the final head or required evidence cannot be reviewed.

Full completion is unavailable under this roadmap because ten tunnel families
and 26 RouterInfo additions remain unsupported/unavailable.

No status implies upstream review, acceptance, certification, adoption, or
merge.
