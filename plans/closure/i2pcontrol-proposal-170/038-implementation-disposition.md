# M038 Implementation Disposition — Live-Runtime Proposal 170 Interoperability

Status: implemented; closure accepted

Source implementation plan:

- `plans/implementation/i2pcontrol-proposal-170/038-live-runtime-interoperability.md`

Corrective dependencies:

- `plans/closure/i2pcontrol-proposal-170/038a-implementation-disposition.md`
- `plans/closure/i2pcontrol-proposal-170/038b-implementation-disposition.md`

Frozen implementation/evidence head: `a5864d2`

## Implementation

M038 adds one bounded feature-gated Rust integration test,
`emissary-cli/tests/i2pcontrol_live_runtime.rs`. It launches the real
feature-enabled `emissary-cli` child process, uses its production HTTPS/TLS
JSON-RPC stack and production control owners, and exercises authentication,
notifications and IDs, AddressBook mutation/list agreement/deletion/durable
state, available and unavailable RouterInfo, ClientServicesInfo, generic
TunnelManager client/server definitions, bind failure and recovery,
unsupported types, startup ownership, malformed input, restart, and bounded
cleanup.

The scenario uses only loopback and temporary state. Credentials are
process-local, the destination is generated at runtime, the service's
development TLS material remains temporary, and child diagnostics are checked
for password leakage without emitting raw logs. No fixture contains a token,
password, private key, destination, or generated state.

The only production corrections discovered during validation are the separate
M038A SAM call-shape correction and M038B test/import guard repairs. They do
not add capability or change the Proposal 170 wire contract.

## Evidence disposition

| Evidence layer | Result | Scope |
|---|---|---|
| Retained unit/fixture evidence | pass | M020–M037 contract, source, persistence, boundary, and failure tests remain green |
| Production-composition check | pass | feature-enabled binary and real I2PControl owners start and authenticate over TLS |
| Live child-process run | pass | one bounded run; all administrative, restart, recovery, isolation, and unsupported phases passed |
| Generic client/server data-plane traffic | qualified blocker | no local reseeded peer set; listener readiness and formation are not claimed as traffic success |
| Subscription refresh | qualified blocker | no HTTP downloader is composed in this loopback config; the documented `-32603` unavailable response was accepted |

The two qualified blockers are environment/composition limits, not fabricated
successes or unresolved correctness defects. The test still proves the
administrative lifecycle's deterministic bind failure, correction/restart
path, persistence, ownership boundaries, and process isolation.

## Verification executed

```text
cargo check -p emissary-cli --no-default-features
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo test -p emissary-core sam --no-fail-fast
cargo test -p emissary-cli --no-default-features
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test i2pcontrol_live_runtime -- --nocapture
cargo test -p emissary-cli --no-default-features --features i2pcontrol
cargo clippy -p emissary-cli --no-default-features --all-targets -- -D warnings
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
git diff --check
```

Results: checks and clippy passed; 149 focused SAM tests, 54 no-feature CLI
tests, 1 live-runtime test, and 1,325 feature-enabled CLI tests passed. The
repository-wide `cargo +nightly fmt --all -- --check` remains red on the
pre-existing formatter drift across unrelated files; the changed test files
were checked/formatted with nightly rustfmt and no unrelated formatter output
was retained.

## Scope and security

No CI, privileged namespace, external router, public I2P endpoint, remote
service, new tunnel family, startup-task adoption, or core behavior was added.
The test has fixed startup/request/cleanup deadlines, loopback-only binding,
runtime-only secrets, bounded malformed input, and child kill/wait fallback.
The unsupported and startup-managed paths remain explicit and resource-free.

## Internal-only attestation

M038 is accepted against the repository's pinned Proposal 170 revision. No
upstream issue, pull request, review, submission, maintainer channel, or
third-party connector was mutated.
