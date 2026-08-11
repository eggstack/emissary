# M054 closure — transit bandwidth 15s corrective

Status: closed

Implementation commit: `eed0368` (`i2pcontrol: correct M054 transit bandwidth semantics`)

Closure date: 2026-08-11

## Scope and final disposition

M054 corrected the `i2p.router.net.bw.transit.15s` source-truthfulness defect
identified after the historical M049/M052 closure. The request-local
`TransitBandwidthSampler` was removed from `ProductionRouterInfoControl`. The
canonical source row is now explicitly unavailable with owner
`transit-bandwidth` and reason `no request-independent rolling transit owner`.

This is the accepted truthful limitation, not a failed implementation. Proposal
170 defines the field as a 15-second average transit bandwidth in bytes per
second. The pinned i2pd reference maintains that value from a dedicated
one-second transport-owner timer and calculates it from elapsed cumulative
transport bytes. Emissary's existing `EventManager` refresh interval is
configurable and is not an exact equivalent. Adding a dedicated timer,
data-plane instrumentation, or a new polling subsystem was outside the M054
budget. No existing owner could therefore provide the pinned semantics without
an approximation.

The authoritative references are [Proposal 170](https://i2p.net/en/proposals/170-i2pcontrol-expansion/)
and the read-only pinned i2pd sources
([Transports.h](https://sources.debian.org/src/i2pd/2.45.1-1/libi2pd/Transports.h),
[Transports.cpp](https://sources.debian.org/src/i2pd/2.45.1-1/libi2pd/Transports.cpp)).

## Requirement and evidence matrix

| Requirement | Evidence | Result |
|---|---|---|
| Remove the request-driven production sampler | `emissary-cli/src/i2pcontrol/production.rs` no longer contains the sampler, its mutex, or its rolling-window state | pass |
| Keep the canonical row present and truthful | `rpc.rs` declares the row `Unavailable`; the owner and reason are stable source metadata | pass |
| Never synthesize zero or stale transit data | production control returns `InspectionError::UnavailableReason`; a fake stored value cannot bypass the source disposition | pass |
| Fail direct requests before partial assembly | `router_info_truthfulness.rs` covers a direct transit request and a combined version-plus-transit request; both return `-32603` with a null result | pass |
| Preserve the other M049 fields | the RouterInfo fixture retains recent success, tunnel queue, and TBM queue success coverage | pass |
| Exercise the real authenticated runtime path | `i2pcontrol_live_runtime.rs` checks transit unavailability and combined available selectors through the TLS/authenticated child process | pass |
| Keep source accounting synchronized | conformance manifest and literal fixtures now assert 43 total, 39 available, 1 neutral, 3 unavailable | pass |
| Keep the change within the M054 boundary | changed production/API composition, tests, docs, and planning records only; no core, transport, tunnel, router, NetDB, or new timer changes | pass |

## Runtime, lifecycle, and concurrency review

There is no longer a transit rolling owner in the production adapter, so no
startup warm-up, reset, or window-boundary behavior is claimed for this field.
The removed sampler had process-local request history; deleting it removes that
misleading state. Until a compliant owner is introduced by a future authorized
plan, every request receives the explicit unavailable disposition.

The unavailable preflight occurs before source acquisition and result assembly,
so direct and combined requests cannot return a partial result. M054 adds no
timer, task, lock, await point, or shared mutable transit state. Request
cancellation, repeated requests, and concurrent requests therefore cannot
mutate transit measurement state or create a contention path.

## Verification

Successful checks:

- `cargo test -p emissary-core --no-fail-fast` — 1062 passed, 2 ignored.
- `cargo check -p emissary-core`.
- `cargo test -p emissary-core events --no-fail-fast` — 3 passed.
- `cargo check -p emissary-cli --no-default-features`.
- `cargo test -p emissary-cli --no-default-features --no-fail-fast` — 56 passed.
- `cargo check -p emissary-cli --no-default-features --features i2pcontrol`.
- `cargo test -p emissary-cli --no-default-features --features i2pcontrol --no-fail-fast` — 1369 passed.
- `cargo test -p emissary-cli --no-default-features --features i2pcontrol router_info --no-fail-fast` — 133 passed.
- `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test router_info_truthfulness` — 36 passed.
- `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test production_adapter` — 22 passed.
- `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test static_guards` — 39 passed.
- `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test conformance_manifest` — 58 passed.
- `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test i2pcontrol_live_runtime live_runtime_interoperability` — 1 passed.
- `cargo clippy -p emissary-core --all-targets -- -D warnings`.
- `cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings`.
- `git diff --check`.

`cargo fmt --all -- --check` was run with the active stable toolchain and
`cargo +nightly fmt --all -- --check` was also run with the repository's
nightly toolchain. Both report extensive pre-existing workspace-wide formatting
differences in untouched files, including core and unrelated CLI modules. The
working tree remains unformatted rather than absorbing that unrelated churn.

## Compatibility, security, and operational review

The JSON-RPC method, selector spelling, and error contract remain unchanged;
only the source disposition changes from an invalid available value to the
protocol's explicit unavailable path. No migration or persisted-state change
is required. The change reduces exposure by removing request-controlled
measurement state and does not add network access, credentials, logging of
sensitive data, background work, or router lifecycle behavior.

Documentation and source-map counts were updated in the I2PControl docs. The
no-feature build and test paths remain green. The current machine-readable
matrix is 39 available / 1 neutral / 3 unavailable; the two network-error rows
remain available until M055 addresses them.

## Planning handoff

The original M049 closure remains historical for the three retained fields; its
transit-15s claim is corrected by this closure. M054 is formally closed.

- M055 (`055-m050-network-error-truthfulness-corrective.md`) is unblocked and
  is now the sole dependency-ready implementation plan.
- M056 (`056-m049-m050-corrective-reclosure.md`) remains blocked until M055
  closes and can reconcile all 43 source rows.
- M051 remains blocked with its accepted news/ban semantic limitation because
  no authoritative owner exists.

No upstream issue, PR, review, submission, or maintainer contact was made.
