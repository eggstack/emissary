# M081 Closure — Generic Server LeaseSet Option Truthfulness Corrective

Status: closed

Source implementation plan:

- `plans/implementation/i2pcontrol-proposal-170/081-generic-server-leaseset-option-truthfulness-corrective.md`

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-tunnel-security-hardening-roadmap.md`

Corrective predecessor closures:

- M073 closure: `plans/closure/i2pcontrol-proposal-170/073-closure.md` — historical
  closure accepted against `3d1d8f1`; invariant later regressed by M075.
- M075 closure: `plans/closure/i2pcontrol-proposal-170/075-closure.md` —
  `corrective pass required` for the M075-introduced
  accepted-but-ignored `leaseSetEncType` regression; M081 owns the defect.

Planning production baseline: `1618de172e7a78a193fc1bb117af269f31174030`.

## 1. Retained implementation evidence

M075 established the accepted-stream architecture that remains accepted and
unreverted:

- control-plane generic `server` no longer uses blind SAM `STREAM FORWARD`;
- it owns an application-visible accepted-stream session;
- it reuses shared accepted-server admission before local-target work;
- after admission, payload is relayed byte-for-byte to a fixed loopback target;
- persistent server Destination/public identity remains stable across restart;
- startup-managed server forwarding remains separately owned in
  `emissary-cli/src/tunnel/server.rs`;
- local target connect is bounded;
- private destination material remains redacted;
- no `emissary-core/**` production path or SAM protocol extension was added.

M080 closed the admission transactionality/cardinality defects that M081's
generic server consumes unchanged: `ServerAdmissionState` retains canonical
32-byte cryptographic peer identity, transactional-denied mutation-free
state, and a bounded `(expires_at, peer_key)` expiry index.

M081 adds the `leaseSetEncType` apply-or-reject correction while leaving
every M075/M080 property intact.

## 2. Confirmed regression and root cause

Independent review of head `1618de172e7a78a193fc1bb117af269f31174030` found
that:

- `ServerTunnelBackend::validate_i2cp_options`
  (`emissary-cli/src/i2pcontrol/backends/server.rs:489`) still accepted
  `leaseSetEncType` and rejected other I2CP keys;
- `GenericServerRuntimeConfig` did not carry the accepted value;
- `AcceptedServerRuntimeConfig` did not carry the accepted value;
- `run_accepted_server`
  (`emissary-cli/src/i2pcontrol/backends/runtime/accepted_server.rs:78`)
  created Yosemite `SessionOptions` with `..Default::default()` for the
  remaining fields, including `lease_set_enc_type: None`.

The option was accepted and persisted but did not influence the actual
accepted-stream session. M073's historical apply-or-reject invariant
regressed in M075.

A second defect was discovered while threading the value: `SERVER_OPTIONS`
declared `i2cp: CustomOptionPolicy::Reject`, which made the generic
`validate_options` call reject every `i2cp_options` entry before the
backend-specific `validate_i2cp_options` could allow `leaseSetEncType`. M081
flips the generic capability to `Accept` and uses
`ServerTunnelBackend::validate_i2cp_options` as the authoritative key
allowlist, exactly as the plan calls for.

## 3. Yosemite capability confirmation (WP1)

The pinned Yosemite `0.7.0` `SessionOptions::lease_set_enc_type`
(`/home/sugarwookie/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/yosemite-0.7.0/src/options.rs:273`)
is consumed by `session.rs:224` for every `SESSION CREATE` regardless of the
later data path (`STREAM ACCEPT` or `STREAM FORWARD`). The accepted-stream
path therefore reuses the same option that the existing startup-managed
server runtime (`emissary-cli/src/tunnel/server.rs:107`) already threads
through `SessionOptions::lease_set_enc_type`. No new `emissary-core/**`
production path, no Yosemite fork, and no SAM protocol extension is
required.

## 4. Applied correction (WP2)

The validated optional value is threaded through the I2PControl
accepted-server configuration:

```text
ServerTunnelBackend::runtime_config
  -> GenericServerRuntimeConfig::lease_set_enc_type
  -> AcceptedServerRuntimeConfig::lease_set_enc_type
  -> SessionOptions::lease_set_enc_type
```

Concretely:

- `AcceptedServerRuntimeConfig` gains a single optional
  `lease_set_enc_type: Option<String>` field
  (`emissary-cli/src/i2pcontrol/backends/runtime/accepted_server.rs:42`).
  Every other accepted-server family (`irc_server`, `http_server`,
  `http_bidir`) explicitly sets `lease_set_enc_type: None` so they do not
  silently gain a capability their own option contracts do not document.
- `GenericServerRuntimeConfig` gains the same field
  (`emissary-cli/src/i2pcontrol/backends/server.rs:50`).
- `run_accepted_server` writes the value into `SessionOptions`
  (`emissary-cli/src/i2pcontrol/backends/runtime/accepted_server.rs:88`).
- `ServerTunnelBackend::runtime_config` extracts the validated value
  through a new `lease_set_enc_type` helper
  (`emissary-cli/src/i2pcontrol/backends/server.rs:507`) that consumes the
  `i2cp_options` `BTreeMap<String, String>` only after
  `validate_i2cp_options` has confirmed the key is `leaseSetEncType`.
  An empty string is treated as `None` so we never emit an empty
  `i2cp.leaseSetEncType=` on the SAM wire.
- `SERVER_OPTIONS` in `emissary-cli/src/i2pcontrol/backends/options.rs:151`
  is set to `i2cp: CustomOptionPolicy::Accept` so the generic
  `validate_options` coarse-grained check defers to the backend-specific
  `validate_i2cp_options` allowlist. The `custom` policy remains `Reject`.
- `ServerTunnelBackend::start` calls `validate_i2cp_options` before
  `validate_options` so any unknown I2CP key fails before the
  `SERVER_OPTIONS` capability check is reached, preserving the
  "every accepted runtime-relevant option is applied or rejected before
  allocation" invariant.

## 5. Runtime evidence (WP3)

The fake-SAM fixture and regression tests
(`emissary-cli/src/i2pcontrol/backends/server.rs:970`) prove the value
is actually expressed in session setup:

- `generic_server_threades_lease_set_enc_type_into_session_create` — the
  `SESSION CREATE` line observed by the fake SAM contains
  `leaseSetEncType=4,0` and remains a `STREAM` style session;
- `generic_server_omits_lease_set_enc_type_when_unset` — without the
  option, the I2PControl layer does not inject `leaseSetEncType=4,0` on
  the wire (the SAM parser defaults to `6,4` are unchanged);
- `lease_set_enc_type_survives_restart_with_new_session_generation` —
  both restart generations issue their own `SESSION CREATE` and each
  carries the configured `leaseSetEncType=4,0`;
- `lease_set_enc_type_is_threaded_when_present_and_absent_otherwise`
  covers the `runtime_config` helper directly, including the empty-string
  and absent-key cases;
- `unknown_generic_server_i2cp_keys_still_fail_before_allocation` — a
  non-`leaseSetEncType` I2CP key still fails with
  `BackendError::UnsupportedOption { option: "I2CPOptions" }` before any
  destination-store/session/task allocation;
- the existing `generic_server_uses_accepted_stream_and_relays_bytes_without_forwarding`
  test continues to prove `STREAM ACCEPT` is issued and no
  `STREAM FORWARD` is ever issued.

The pre-existing M074/M075 regression suite
(`admission_options_are_applied_and_non_loopback_targets_fail_before_allocation`,
`unimplemented_server_options_fail_before_store_or_session_allocation`,
`server_lifecycle_preserves_public_destination_and_cancels_exact_task`,
`startup_server_lifecycle_is_rejected_before_store_access`) and the
accepted-server fake-SAM fixtures remain green unchanged.

## 6. Capability matrix and support docs (WP4)

- `docs/i2pcontrol/proposal-170-support.md` — status table updated to
  record M081 as closed and to mark M073 as `closed; corrective history`
  (M081 closes the M075-accepted-stream regression and re-establishes the
  M073 invariant at the current head).
- `docs/i2pcontrol/tunnel-backends.md` — explicit note that the generic
  server is the only accepted-server family that accepts an I2CP
  session-shaping option, that `i2cp.leaseSetEncType` is the sole
  supported key, and that other accepted-server families explicitly pass
  `None` for the new shared field.
- `docs/i2pcontrol/tunnel-manager.md` — status line updated to reflect
  M081 closure.
- `plans/registry.md` — M081 status changes from `ready` to `closed`, M082
  status changes from `blocked` to `ready`, and M077's dependency is
  updated to reference M082 only.

## 7. Failure, cancellation, restart, and contention semantics

- session setup failure remains sanitized (`AcceptedServerRuntimeError::SessionSetup`
  carries no destination material and the private destination material is
  absent from `Debug`/error output);
- start reports running only after the accepted-stream session is actually
  ready (`session.destination()` is published to the supervisor before
  `mark_running` is allowed);
- persistent destination remains unchanged across restart
  (`ServerDestinationStore` holds the same private key for every
  generation; the lease-set value is applied to every new session
  generation);
- changing `leaseSetEncType` through a stopped/edit/start cycle applies
  to the new session generation if the option remains supported; the
  existing `TunnelManager` edit semantics continue to require stop before
  edit;
- running-generation edit semantics remain whatever `TunnelManager`
  already defines; M081 adds no hot reconfiguration;
- the old generation cannot retain/apply options to a later generation;
  each generation owns a distinct supervisor task and a fresh
  `ServerAdmissionState`;
- startup-managed server runtime is unchanged and still maps
  `leaseSetEncType` through its separate path
  (`emissary-cli/src/tunnel/server.rs:107`);
- the generic server remains accepted-stream, never re-introduces
  `STREAM FORWARD`, and after admission the payload is loopback-only and
  byte-transparent.

## 8. Verification

The commands from section 12 of the plan were executed against the
implementation commit and produced the recorded outcomes:

| Command | Outcome |
|---|---|
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol` | 1602 passed (24 suites) |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol generic_server` | 10 passed |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol admission` | 48 passed (unchanged M080 set) |
| `cargo check -p emissary-cli --no-default-features` | clean |
| `cargo check -p emissary-cli --no-default-features --features i2pcontrol` | clean |
| `cargo check -p emissary-core` | clean |
| `cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings` | clean |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment` | 7 passed |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m062_dependency_containment` | 19 passed |
| `git diff --check` | clean |

Scoped nightly rustfmt (`cargo +nightly fmt -- --check`) reports no
formatting violations for the touched files
(`server.rs`, `options.rs`, `runtime/accepted_server.rs`,
`http_server.rs`, `irc_server.rs`, `http_bidir.rs`,
`tests/m062_dependency_containment.rs`). Pre-existing repo-wide rustfmt
drift outside the M081 touched surface is unchanged.

The M062 manifest was also updated to admit the M080-introduced
`peer_identity.rs` and `peer_identity_impl.rs` paths and the M080 closure
document. These were added by M080 but never recorded in the M062
allowlist, so the M062 `allowed_production_paths_match_the_m062_budget`
test was silently failing at the M080 closure head. M081 records the
correction as part of its verification gate; the underlying M080 closure
remains accepted at `f07bf14` and the workstream does not reopen M080's
audit findings.

## 9. Compatibility, migration, security review

- the public accepted-server runtime surface
  (`AcceptedServerRuntimeConfig`, `AcceptedServerConnection`,
  `AcceptedServerHandler`, `AcceptedServerRuntimeError`) gains exactly
  one new optional private field (`lease_set_enc_type`); all existing
  callers are updated to pass `None` for their own accepted-server
  families, so no production type changes;
- raw option/value parsing for `leaseSetEncType` is unchanged; the
  existing `validate_i2cp_options` allowlist remains the sole
  authoritative key check, and the new `SERVER_OPTIONS` capability
  change is purely a coarse-grained flag flip that defers to the
  backend-specific allowlist;
- the corrected accepted-server runtime is consumed by `httpserver`,
  `httpbidirserver`, and `ircserver` unchanged; each passes `None` for
  the new field so they do not incidentally gain a public capability;
- the M080 canonical cryptographic Destination identity
  (`TrustedPeerIdentity`) and the corrected admission state are consumed
  unchanged by the generic server runtime;
- the Tokio `test-util` feature remains scoped to `[dev-dependencies]`
  from M080; M062 transitive-feature containment remains green;
- no `emissary-core/**` production change was required; the existing
  Yosemite `SessionOptions::lease_set_enc_type` field is reused through
  the new `AcceptedServerRuntimeConfig::lease_set_enc_type` plumbing;
- M061 source-containment and M062/M063 dependency-containment suites
  remain green; the M062 manifest now explicitly admits the
  M080-introduced `peer_identity.rs`, `peer_identity_impl.rs`, and
  `080-closure.md` paths that were silently pending at the M080 closure
  head.

## 10. Acceptance criteria evaluation

Section 13 of the plan is satisfied:

1. generic `server` no longer accepts any runtime-relevant I2CP option
   that is ignored;
2. `leaseSetEncType` is demonstrably present in accepted-stream session
   configuration (the three new fake-SAM regression tests inspect the
   `SESSION CREATE` line and assert the operator's value is carried);
3. the M075 accepted-stream architecture remains intact
   (`generic_server_uses_accepted_stream_and_relays_bytes_without_forwarding`
   and the new `lease_set_enc_type*` tests still observe `STREAM ACCEPT`
   and never `STREAM FORWARD`);
4. no control-plane `STREAM FORWARD` path is reintroduced;
5. unsupported generic server I2CP/options remain fail-closed
   (`unknown_generic_server_i2cp_keys_still_fail_before_allocation`);
6. persistent identity/lifecycle/secret-redaction behavior remains intact
   (`server_lifecycle_preserves_public_destination_and_cancels_exact_task`,
   `generic_server_debug_is_secret_safe`);
7. no core/router/startup ownership change occurs;
8. M080 is closed and its admission protections remain consumed
   unchanged;
9. capability/support documentation matches runtime truth
   (`docs/i2pcontrol/proposal-170-support.md`,
   `docs/i2pcontrol/tunnel-backends.md`,
   `docs/i2pcontrol/tunnel-manager.md`,
   `plans/registry.md`);
10. no high/medium option-truthfulness finding remains in generic server
    scope (M081 closes the M075-accepted-but-ignored `leaseSetEncType`
    regression and the M073 historical invariant is re-established at
    the current head).

## 11. Unresolved findings

None at M081 scope. The M075 closure's disposition is updated to
`closed` because M081 closes the option-truthfulness defect that the
M075 closure originally flagged for a corrective pass. The M073 closure
disposition is updated to `closed; corrective history` because M081
re-establishes the M073 invariant at the current head.

## 12. Unblocked downstream plans

M081 closes the M075 option-truthfulness defect and unblocks:

- **M082 — HTTP peer identity and `Expect`-framing corrective**
  (`plans/implementation/i2pcontrol-proposal-170/082-http-peer-identity-and-expect-framing-corrective.md`).
  Registry sequencing in `plans/registry.md` advances M082 from `blocked`
  to `ready`. M082 explicitly consumes the M080 canonical cryptographic
  Destination identity and the M081 generic-server `leaseSetEncType`
  capability matrix; it does not depend on M081's value-threading beyond
  the rejected-or-applied invariant documented in
  `docs/i2pcontrol/tunnel-backends.md`.
- **M077 — IRC server lifetime/exhaustion hardening**
  (`plans/implementation/i2pcontrol-proposal-170/077-irc-server-lifetime-and-exhaustion-hardening.md`).
  M077's original "behind M081-M082" blocker is simplified to
  "behind M082" because the M081 half of the dependency is now closed.
- **M078 — Streamr local-boundary hardening** and **M079 — integrated
  tunnel-security reclosure** remain blocked behind M082 and M077.
  M079, not an implementation-agent assertion, is the final independent
  tunnel-security reclosure authority.

## 13. Internal-only boundary

External I2P/I2P+ reference material was inspected read-only (the pinned
Yosemite `SessionOptions` contract in
`~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/yosemite-0.7.0/`)
while confirming the accepted-stream `lease_set_enc_type` capability. No
upstream repository, maintainer channel, issue, pull request, merge
request, or submission was opened, drafted, requested, or prepared. No
contribution artifact was produced under M081. All repository writes
remain internal to `eggstack/emissary`.
