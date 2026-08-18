# Emissary Active Planning Registry

This file is the compact control surface for active planning.

Canonical direction:

- `plans/000-long-term-specification.md`
- `plans/001-terminology-and-domain-model.md`
- `plans/002-long-term-roadmap.md`
- `plans/003-planning-process.md`

## Status vocabulary

- **proposed** — document exists but is not approved for execution.
- **ready** — dependencies and interfaces are satisfied; plan may be handed off.
- **active** — implementation or closure work is in progress.
- **blocked** — a named dependency or evidence requirement prevents progress.
- **closing** — implementation landed and independent closure evidence is being gathered.
- **closed** — closure record accepted for the pinned implementation head.
- **closed internally against pinned revision** — internal closure accepted against an explicitly named revision of an open external specification; does not imply upstream review or acceptance.
- **partial Proposal 170 support** — exact supported dimensions are closed, but one or more pinned source/runtime capabilities remain truthfully unavailable.
- **corrective pass required** — a prior disposition or closure was invalidated by a material implementation, compatibility, scope, or evidence defect.
- **superseded** — replaced by another document and not executable.
- **archived** — inactive and retained for traceability.

## Active subsystem roadmaps

| Subsystem | Status | Roadmap | Current handoff | Dependencies or blockers |
|---|---|---|---|---|
| I2PControl Proposal 170 source/truthfulness | partial Proposal 170 support; M057 closed | `plans/subsystems/i2pcontrol-proposal-170-roadmap.md` | no source-completion handoff | M051 remains blocked by absent substantive news/ban owners; accepted RouterInfo matrix remains 37/1/5 |
| I2PControl Proposal 170 containment | closed | `plans/subsystems/i2pcontrol-proposal-170-containment-roadmap.md` | no containment corrective handoff | M061 source containment and M062/M063 dependency containment remain accepted authorities |
| I2PControl Proposal 170 tunnel runtime completion | historical runtime completion accepted; current security closure reopened | `plans/subsystems/i2pcontrol-proposal-170-tunnel-runtime-completion-roadmap.md` | no separate runtime handoff | M072/M073 historical closure evidence remains valid for their pinned heads; M081 re-establishes the M073 generic-server option truthfulness invariant at the current head |
| I2PControl Proposal 170 tunnel security hardening | M080-M082 closed; M077-M079 corrective work remains | `plans/subsystems/i2pcontrol-proposal-170-tunnel-security-hardening-roadmap.md` | M077 — IRC server lifetime/exhaustion hardening | independent post-M076 review found M074 admission-state defects, M075 leaseSet option regression, and M076 HTTP identity/Expect defects; M080 closes the admission transactionality/cardinality defects; M081 closes the accepted-but-ignored `leaseSetEncType` regression; M082 closes the HTTP structural-Destination, Expect, and POST peer-key defects |

## Canonical scope amendment for tunnel runtimes

ADR-0003 remains the controlling scope amendment for the ten Proposal 170 tunnel data planes. ADR-0001/ADR-0002 remain historical/controlling for contract spelling, startup/control-plane separation, server secret ownership, and internal-only scope except where ADR-0003 explicitly superseded the earlier deferment of those data planes.

The preferred production boundary is `emissary-cli/src/i2pcontrol/**`. M065-M082 do not authorize a new `emissary-core/**` production path. M064 remains the narrow historical exception for an already accepted event-observation setter repair.

No upstream review, merge, submission, contribution preparation, issue/PR mutation, or maintainer contact is authorized. External I2P/I2P+ repositories and specifications are read-only evidence only. Repository writes remain internal to `eggstack/emissary`.

## Dependency-ready implementation plan

Exactly one plan is currently registered as dependency-ready:

| Handoff | Status | Plan | Objective |
|---|---|---|---|
| M077 — IRC server lifetime/exhaustion hardening | ready | `plans/implementation/i2pcontrol-proposal-170/077-irc-server-lifetime-and-exhaustion-hardening.md` | add activity-resetting post-registration idle expiry to `ircserver` without parsing or reframing normal IRC traffic, building on the M080 canonical cryptographic peer identity |

Per `plans/003-planning-process.md`, only the next dependency-ready implementation plan is registered `ready`.

## Current tunnel-security corrective sequence

The independent review of head `1618de172e7a78a193fc1bb117af269f31174030` invalidated the claim that M074-M076 leave no high/medium findings at the current head. Historical closure evidence remains useful, but current security closure is governed by the following corrective sequence.

| Handoff | Status | Plan | Dependency / blocker |
|---|---|---|---|
| M074 — shared server admission/rate hardening | closed; corrective history | `plans/implementation/i2pcontrol-proposal-170/074-server-admission-and-rate-limit-hardening.md` | M080 owns the discovered transactional/cardinality defects |
| M075 — generic server accepted-stream hardening | closed | `plans/implementation/i2pcontrol-proposal-170/075-generic-server-accepted-stream-hardening.md` | M081 closed the `leaseSetEncType` accepted-but-ignored regression; M080 repaired inherited admission state |
| M076 — HTTP anonymity/POST hardening | closed | `plans/implementation/i2pcontrol-proposal-170/076-http-server-anonymity-and-post-throttle-hardening.md` | M082 closed the valid-Destination bound, `Expect`, and POST peer-key correction |
| M080 — admission transactionality/cardinality corrective | closed | `plans/implementation/i2pcontrol-proposal-170/080-server-admission-transactionality-and-cardinality-corrective.md` | closure: `plans/closure/i2pcontrol-proposal-170/080-closure.md`; implementation commit `f07bf14acd18f3ee6dff89d993ca73f2a14a85b7` |
| M081 — generic server LeaseSet option truthfulness corrective | closed | `plans/implementation/i2pcontrol-proposal-170/081-generic-server-leaseset-option-truthfulness-corrective.md` | closure: `plans/closure/i2pcontrol-proposal-170/081-closure.md`; accepted-stream `SESSION CREATE` now carries the validated `leaseSetEncType` and the M075 accepted-but-ignored regression is closed |
| M082 — HTTP peer identity and Expect-framing corrective | closed | `plans/implementation/i2pcontrol-proposal-170/082-http-peer-identity-and-expect-framing-corrective.md` | closure: `plans/closure/i2pcontrol-proposal-170/082-closure.md`; structural trusted peer identity is consumed through the M080 `TrustedPeerIdentity` boundary, `Expect` requests are rejected with fixed `417 Expectation Failed` semantics before local target allocation, and the POST limiter is keyed by the canonical 32-byte Destination hash |
| M077 — IRC server lifetime/exhaustion hardening | ready | `plans/implementation/i2pcontrol-proposal-170/077-irc-server-lifetime-and-exhaustion-hardening.md` | next registered handoff; consumes the M080 canonical cryptographic peer identity for its activity-resetting post-registration idle expiry |
| M078 — Streamr local-boundary hardening | blocked | `plans/implementation/i2pcontrol-proposal-170/078-streamr-local-boundary-hardening.md` | M077 must close first |
| M079 — integrated tunnel-security reclosure | blocked | `plans/implementation/i2pcontrol-proposal-170/079-tunnel-security-reclosure.md` | M077-M078 must close; M079 independently re-audits the final head |

## Corrective findings controlling M080-M082

### M080 / M074 admission (closed)

`ServerAdmissionState::try_acquire` now performs every denial check before
mutating peer/expiry/aggregate state. Aggregate, peer, global, and
peer-state-capacity denials leave `state.peers.len()` and
`state.expiry_queue.len()` unchanged, covered by four dedicated regression
tests.

The expiry index is replaced with a `BTreeMap<(Instant, PeerKey), ()>`
keyed by composite `(expires_at, peer_key)` so two peers may share a
deadline without colliding and stale entries cannot accumulate beyond the
peer-map cardinality. `assert_invariants` debug-asserts run after every
`try_acquire` commit and every `AdmissionLease` drop in test builds.

Peer identity accounting uses the canonical 32-byte SHA-256 I2P
Destination hash derived from a structurally validated remote Destination,
replacing the M074 8-byte `DefaultHasher` digest.

`ServerAdmissionPolicy::new` derives `required_peer_entries` from the
strongest enabled aggregate arrival bound and the longest enabled per-peer
window; configurations whose exact retained-rate semantics would exceed
`MAX_PEER_ENTRIES = HARD_PEER_STATE_MEMORY_BUDGET / WORST_CASE_BYTES_PER_PEER`
(= 16 MiB / 200 = 83,886) or whose aggregate bound is fully unlimited
reject with `AdmissionPolicyError::IncoherentCapacity` before session/task
allocation.

Closure evidence: `plans/closure/i2pcontrol-proposal-170/080-closure.md`.

### M081 / M073-M075 generic server truthfulness (closed)

M073 historically closed while generic `server` mapped its only supported I2CP option, `leaseSetEncType`, through the old server session configuration. M075 migrated control-plane generic server to accepted streams but did not carry that option into `AcceptedServerRuntimeConfig`/Yosemite `SessionOptions`, while the backend still accepts the field.

M081 closes the regression by threading the validated optional value through:

```text
ServerTunnelBackend::runtime_config
  -> GenericServerRuntimeConfig::lease_set_enc_type
  -> AcceptedServerRuntimeConfig::lease_set_enc_type
  -> SessionOptions::lease_set_enc_type
```

`SERVER_OPTIONS` is set to `i2cp: CustomOptionPolicy::Accept` so the generic `validate_options` coarse-grained check defers to the backend-specific `validate_i2cp_options` allowlist (still `leaseSetEncType` only). Every other accepted-server family explicitly passes `None` for the new shared field. The accepted-stream architecture from M075 is preserved; control-plane `STREAM FORWARD` is not reintroduced.

Closure evidence: `plans/closure/i2pcontrol-proposal-170/081-closure.md`.

### M082 / M076 HTTP correctness (closed)

M076's 524-character trusted-Destination ceiling is based on a legacy-sized Destination assumption and can reject valid current I2P key-certificate/signature forms. M082 replaces the magic ceiling with the M080 structural `TrustedPeerIdentity::from_stream` validation so the HTTP filter consumes the same 32-byte canonical cryptographic peer identity as the shared admission state.

`Expect: 100-continue` was forwarded while Emissary waits for the request body before reading the local response, creating a client/backend wait cycle until body timeout. M082 now rejects every request carrying an `Expect` header (single, duplicate, mixed-case `100-Continue`, or unknown expectation tokens) with a fixed `417 Expectation Failed` response and `Connection: close` before any local target connection, releasing the M080 admission lease on handler return.

The HTTP POST limiter previously keyed peers with an 8-byte `DefaultHasher` digest of the textual peer string. M082 moves the key to the canonical 32-byte Destination hash derived from `TrustedPeerIdentity::canonical_id()`, leaving the `MAX_THROTTLE_ENTRIES = 1024` bound and the "active/unexpired entries are never evicted" rule intact.

Closure evidence: `plans/closure/i2pcontrol-proposal-170/082-closure.md`.

## Durable tunnel security boundary

- accepted server families use authenticated Yosemite peer identity before application handler/local target work;
- global and per-peer concurrency plus peer/aggregate rate state must be bounded and denial must not leave attacker-owned state;
- no active/throttled identity is evicted merely to admit a new identity;
- every attacker-influenced auxiliary collection, including expiry indexes, is hard bounded;
- generic control-plane `server` remains accepted-stream/raw-relay, not blind `STREAM FORWARD`;
- every runtime-relevant option is applied or rejected before allocation;
- HTTP request identity/proxy spoofing is stripped; trusted I2P identity is structurally valid and bounded; response `Date`/server/provider/cache/trace fingerprints are stripped;
- unsupported HTTP expectations fail before local allocation unless a separately accepted plan implements full informational-response semantics;
- IRC registration filtering remains before local connect and M077 adds activity-resetting post-registration idle expiry;
- Streamr remains bounded and M078 makes local UDP ingress/output loopback-only with reference-aligned fanout;
- timing hardening removes controllable saturation/occupancy/fingerprint signals rather than adding artificial sleeps or jitter;
- local target selection remains fixed/administrator-owned; remote payloads do not select LAN/localhost routes beyond each tunnel's explicit safe policy;
- private destination material never enters logs/errors/Debug/API output.

## Containment authority

Accepted containment authorities remain:

- `plans/implementation/i2pcontrol-proposal-170/061-containment-boundary.toml` plus `m061_containment.rs` for source paths;
- `plans/implementation/i2pcontrol-proposal-170/062-dependency-containment.toml` plus the M063-strengthened `m062_dependency_containment.rs` for direct dependency ownership and transitive local-feature activation.

M062/M063 durable dependency rule: a direct dependency whose only direct consumer is `feature = "i2pcontrol"` code must be optional and feature-owned by `i2pcontrol`; unrelated local features must not activate it directly or indirectly.

M080 additionally rechecks whether Tokio `test-util`, introduced for paused-time tests, can be confined to test/dev activation without production feature widening. This is a narrow dependency-containment hygiene item, not general workspace cleanup.

## Accepted unrelated Proposal 170 state

The RouterInfo source matrix remains exactly:

- 43 canonical Proposal 170 RouterInfo additions;
- 37 available;
- 1 protocol-permitted neutral;
- 5 unavailable: transit 15s, news, banned peers, and both v4/v6 network-error rows.

M051 remains blocked by absent substantive news/ban owners. Tunnel security hardening does not create those owners or reopen transit/error source decisions.

AddressBook `SetConfig`, unrelated base-I2PControl limitations, and environmental qualification remain separate and must continue to be documented truthfully.

## Verification policy

Keep verification local/package-scoped and proportional:

- focused backend/filter/runtime tests;
- fake/local SAM and local TCP/UDP capture services;
- structurally valid I2P Destination fixtures;
- Tokio paused-time limiter/expiry/idle tests;
- feature-disabled and feature-enabled `cargo check`;
- M061/M062/M063 containment suites;
- Clippy and scoped repository-accepted nightly rustfmt for touched files;
- `git diff --check`.

Do not add hosted CI jobs, release machinery, coverage gates, generalized fuzz infrastructure, soak farms, broad platform matrices, public-network deanonymization experiments, or upstream contribution machinery for this workstream.

## Registry maintenance rules

1. M077 is the sole dependency-ready handoff.
2. M078 must remain blocked until M077 closes.
3. M079 remains behind M077 and M078.
4. M079, not an implementation-agent assertion, is the final independent tunnel-security reclosure authority.
5. Any high/medium finding discovered by M079 creates another narrow corrective plan; it may not be hidden inside closure.
6. Preserve ADR-0003 scope: no adjacent tunnel/protocol features.
7. No M082 plan may add a new `emissary-core/**` production path without stopping and creating separate architecture/corrective planning.
8. Preserve RouterInfo 37/1/5 and the M051 blocker unless separate source-owner work changes them.
9. Unsupported/underspecified options fail before allocation; persist-and-ignore is forbidden.
10. No artificial response jitter/fixed delays substitute for bounded resource ownership.
11. External sources are read-only only; no upstream interaction is authorized.
12. All repository writes remain internal to `eggstack/emissary`.
