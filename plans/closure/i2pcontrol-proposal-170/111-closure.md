# M111 Closure — SAM Session-Wire Option Completion

Status: **closed**

Closure date: 2026-09-02

Plan: `plans/implementation/i2pcontrol-proposal-170/111-sam-session-wire-option-completion.md`

Implementation head: `f46917b0668439dce601185341c7bea581709b41`

Proposal authority: I2P Proposal 170, pinned revision `2026-05-20`, status Open.

## Disposition

M111 completed the 40 applicable `SessionWire` cells. The four applicable `UseSSL`
cells remain explicitly `blocked_primitive`; this is a truthful partial closure, not a
claim of full Proposal 170 support.

The exact matrix transition is:

Authoritative matrix: `plans/implementation/i2pcontrol-proposal-170/095-full-support-matrix.toml`

Post-closure matrix SHA-256: `02cc97be871ffb4e89381b1710fcec8dab04edddc08697b10f57b2eb6ff179e5`

| State | Apply | Blocked primitive | Not applicable |
|---|---:|---:|---:|
| Before M111 (post-M116) | 248 | 134 | 458 |
| After M111 | 288 | 94 | 458 |
| Delta | +40 | -40 | 0 |

The 40 promoted cells are the ten non-Streamr tunnel families (`client`, `httpclient`,
`ircclient`, `socks`, `socksirc`, `connectclient`, `server`, `httpserver`,
`httpbidirserver`, and `ircserver`) crossed with each of:

- `TunnelVariance`;
- `TunnelBackupQuantity`;
- `SigType`;
- `CustomOptions`.

The unchanged blocked cells are exactly `UseSSL:httpclient`,
`UseSSL:connectclient`, `UseSSL:httpserver`, and `UseSSL:httpbidirserver`.

## Semantic and dependency freeze

M117's accepted ADR-0005 dependency boundary is the optional `yosemite-i2pcontrol`
alias at exact fork revision `8026f5b424fc178d683e63555335f8b33e0aba04`. M111 uses its
public `SessionOptions` interface and the real Yosemite `SESSION CREATE` serializer;
it does not construct SAM commands independently. The relevant public fields and
methods are `inbound_len_variance`, `outbound_len_variance`,
`inbound_backup_quantity`, `outbound_backup_quantity`, `signature_type`, and
`add_session_option`. M118 is the accepted neutral Emissary owner that consumes
variance and backup values in tunnel-pool construction.

The accepted runtime policy is:

- variance is bounded to `-2..=2` and is applied symmetrically to inbound/outbound
  session settings;
- backup quantity is bounded to `0..=3` and is applied separately to inbound/outbound
  standby capacity;
- `SigType` accepts only the canonical string `"7"`. The Emissary SAM parser and
  destination generation support that router type; other values fail before session
  or destination allocation and never silently downgrade;
- `CustomOptions` is bounded to 32 entries, requires the `i2cp.` namespace, limits keys
  to 64 bytes and values to 256 bytes, uses Yosemite's safe token validation, rejects
  case-insensitive duplicates, and rejects typed/reserved options such as
  `i2cp.leaseSetEncType`;
- all validation occurs before session/listener allocation, and the exact session
  settings—including custom key/value identity—participate in shared-session
  compatibility without exposing custom values or private keys through diagnostics.

`UseSSL` is not mapped to Yosemite `SessionOptions.ssl`. Yosemite's field controls TLS
on the SAM control connection, while Proposal 170's applicable `UseSSL` cells require
local application/session presentation TLS. No accepted Emissary owner currently
provides that effect, so mapping the similarly named field would be semantically false.

## Evidence and changed ownership

The production path is the existing I2PControl session builder in
`emissary-cli/src/i2pcontrol/backends/runtime/session.rs`, composed by the existing
I2PControl backends. `options.rs` performs common strict validation; the session builder
constructs Yosemite `SessionOptions`; the dependency serializer emits the wire. No
`emissary-core` production code was changed by M111.

The implementation commit changes only the existing I2PControl option/session seams,
their regression tests, planning evidence, documentation, and the containment guards
needed to recognize M118's already-authorized `emissary-core/src/config.rs` seam:

- `emissary-cli/src/i2pcontrol/backends/options.rs`;
- `emissary-cli/src/i2pcontrol/backends/runtime/session.rs`;
- `emissary-cli/src/i2pcontrol/client_secret_store.rs`;
- `emissary-cli/src/i2pcontrol/backends/http_server.rs`;
- `emissary-cli/tests/m095_full_support_matrix.rs`;
- `emissary-cli/tests/m105_residual_option_audit.rs`;
- `emissary-cli/tests/m060_containment.rs` and `emissary-cli/tests/m062_dependency_containment.rs`;
- `docs/i2pcontrol/tunnel-manager.md`;
- the M095/M105/M110/README/registry/roadmap planning artifacts.

The session adapter regression opens a real local TCP fake SAM endpoint, invokes Yosemite
session creation, and asserts the serialized variance, backup, signature, and custom
options. Additional tests cover strict bounds, unsupported signature rejection, reserved
custom options, exact custom-value compatibility identity, destination generation, and
UseSSL fail-closed behavior. Existing M110/M116 generation cancellation, transactional
edit/restart, rollback, shared-session isolation, Streamr boundaries, and secret-redaction
tests remain in the passing suite; M111 did not add a second lifecycle owner.

## Verification

Passed:

- `cargo check -p emissary-cli --no-default-features`;
- `cargo check` (workspace);
- `cargo check -p emissary-cli --no-default-features --features i2pcontrol`;
- `cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings`;
- `cargo test -p emissary-cli --no-default-features --features i2pcontrol --lib` — 675 passed;
- `cargo test -p emissary-cli --no-default-features --features i2pcontrol --no-fail-fast` — 1835 passed across 26 suites;
- M060/M062 containment, M095 matrix, M105 residual-audit, and focused session/options suites;
- `git diff --check`.

`cargo fmt --all -- --check` was attempted. It reports the repository's pre-existing
stable/nightly rustfmt configuration drift across unrelated files; formatting was not
run as a bulk rewrite and no unrelated formatting churn was introduced. The changed
Rust files were kept consistent with the repository's existing formatting style.

## Failure, restart, contention, and security review

Validation rejects unsupported or malformed values before Yosemite session construction
or listener allocation. Yosemite remains the sole SAM implementation. Session-affecting
settings are included in the compatibility key, with exact length-prefixed custom
key/value identity kept separate from redacted diagnostic formatting. Existing generation
ownership ensures edits build a new generation only after candidate validation and old
generation cancellation; failed allocation preserves truthful prior state. No lock is
held across dependency I/O, and unrelated shared sessions are not terminated by a failed
M111 edit.

No raw private key material or custom option values are added to ordinary debug/display
output. Streamr remains outside this session-wire promotion and retains its documented
bounded datagram, subscriber, expiry, refresh, payload, buffer, and shutdown limits.
M061/M062 containment and M093 tunnel-security boundaries remain satisfied.

Known unresolved scope is limited to the four explicit `UseSSL` cells and the later
M112/M113 residuals. No high- or medium-severity M111 finding remains open. The rustfmt
toolchain mismatch is a low-severity repository tooling issue, not a runtime correctness
finding.

## Future-plan readiness audit

M111 unblocks M112. M110's session owner and M111's final session-wire configuration
surface are now frozen, the seven M116-transferred client `NewDest` cells are included,
and M112 is updated to `Status: ready` with its required execution-time semantic
re-freeze. M112 owns 69 remaining blocked cells.

M113 remains `proposed / blocked` on server presentation, address-routing, LeaseSet
encryption, and client-authorization primitives; M111 does not supply those primitives.
M114 remains `proposed / blocked` until M112/M113 are closed as applicable and the
authoritative matrix has zero unresolved applicable cells. No other future plan became
dependency-ready from M111. The current blocked partition is 4 M111 `UseSSL` cells,
69 M112 cells, and 21 M113 cells, totaling 94.

The next registry-ready handoff is therefore:

`plans/implementation/i2pcontrol-proposal-170/112-client-proxy-and-session-lifecycle-residual-completion.md`

## Internal-only attestation

All external specifications and dependency sources were treated as read-only evidence.
No upstream repository, issue, pull request, maintainer channel, release, or contribution
workflow was mutated. The implementation and closure are internal to `eggstack/emissary`.
