# M113 — Server Presentation, Address-Routing, and LeaseSet Residual Completion

Status: **closed as blocked** — 21 server presentation/routing/LeaseSet cells remain blocked with exact primitive evidence; no apply or not_applicable reclassification

Closure record: `plans/closure/i2pcontrol-proposal-170/113-closure.md`

Historical status: proposed / blocked — roadmap-defined; not registry-ready until M110/M111 establish final key/session ownership and exact LeaseSet/presentation primitives are demonstrated

Class: capability / server security / LeaseSet ownership

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md`

Source evidence:

- M099 closure: `plans/closure/i2pcontrol-proposal-170/099-closure.md`
- M105 audit: `plans/implementation/i2pcontrol-proposal-170/105-residual-option-audit.toml`
- M093 tunnel security closure.

Pinned authority: I2P Proposal 170 revision `2026-05-20`, status Open.

External sources are read-only. All writes remain internal to `eggstack/emissary`.

## 1. Objective

Resolve the remaining server-side residual families:

- `AllowInternalSSL`, `UniqueLocalAddressPerClient`, `MultiHoming` — 6 M105 presentation/routing cells;
- `EncryptLeaseSet`, `OptionalLookup`, `LeaseSetClientAuths` — 15 M105 LeaseSet cells.

The current maximum target is 21 blocked cells, subject to exact M095/M105 re-freeze at execution time.

M113 must preserve the accepted M093 server boundary: public I2P identity and remote peer identity may influence I2P protocol behavior and bounded presentation, but request-controlled configuration may not expand local target routing beyond the existing safe ownership model merely for Java parity.

## 2. Hard blockers and readiness

M113 remains blocked until:

1. M110 closes or otherwise establishes the accepted secret/key persistence model relevant to client-auth/LeaseSet credentials;
2. M111 closes or records the final accepted Yosemite session capability surface;
3. read-only dependency/current-code evidence identifies an exact public primitive for encrypted/authenticated LeaseSet creation and client-auth key handoff, or an explicitly accepted neutral existing-owner seam is registered;
4. the six presentation/routing cells have direct pinned/reference semantics separated from Java local-interface implementation details.

If LeaseSet semantics are unavailable through the accepted dependency, those cells remain blocked. This plan does not authorize a private LeaseSet serializer or Yosemite fork.

## 3. Invariants

M113 MUST preserve:

- literal-loopback/no-resolver local target confinement from M090/M093 unless a separately accepted architecture decision changes that security boundary;
- no request-selected LAN/clearnet target routing;
- no direct-clearnet fallback;
- trusted peer identity only from authenticated Yosemite stream/datagram identity;
- no LeaseSet encryption/authentication downgrade;
- client-auth secrets never appear in RPC results, `raw_config`, logs, debug, RouterInfo, or ordinary definition persistence;
- secret state is bounded, validated, owner-only on Unix and fail-closed on unsafe file types;
- exact recipient/auth cardinality limits;
- failed LeaseSet/session setup never publishes an unencrypted or unauthenticated replacement;
- server admission/rate/filter state remains transactional and bounded;
- HTTP/IRC spoof/framing/DCC/CTCP protections remain non-bypassable;
- generation-local stop/restart/edit semantics;
- no frontend/router-global multihoming subsystem created for parity;
- feature-disabled/default behavior unchanged.

## 4. Explicit non-goals

M113 MUST NOT:

- relax loopback target confinement simply because Java I2PTunnel supports broader local addresses;
- create arbitrary request-selected local interface binding/routing;
- implement a general host multihoming or network namespace subsystem;
- build a LeaseSet serializer/key-exchange stack outside accepted Yosemite/core owners;
- add router-global banning/access control;
- vendor/fork Yosemite or create raw SAM/I2CP handling;
- add dependencies solely for matrix completion;
- modify `emissary-core` unless a separately registered neutral-owner plan explicitly authorizes exact paths;
- alter proxy/client residuals from M112;
- implement unrelated I2PControl/base methods or frontend controls;
- prepare/request upstream changes.

## 5. Expected production paths

Preferred scope:

- `emissary-cli/src/i2pcontrol/domain/tunnel.rs`;
- `emissary-cli/src/i2pcontrol/backends/options.rs`;
- server-family backends under `emissary-cli/src/i2pcontrol/backends/**`;
- accepted-server/session runtime helpers under `emissary-cli/src/i2pcontrol/backends/runtime/**`;
- I2PControl secret store if M110 establishes one reusable for LeaseSet auth material;
- `emissary-cli/src/i2pcontrol/tunnel_manager.rs`;
- M095/M105 artifacts and focused tests.

No non-I2PControl production path is pre-authorized. If a neutral lower-layer owner is truly required, stop and register an exact-path plan before implementation.

## 6. Work packages

### WP1 — Re-freeze exact semantics/applicability

For each of the 21 cells:

- cite Proposal/reference behavior;
- distinguish remote I2P behavior from Java local networking implementation;
- name the current Emissary owner or exact missing primitive;
- record security/anonymity impact;
- identify whether a safe implementation exists inside the accepted boundary.

Do not use difficulty as evidence for `not_applicable`.

### WP2 — `AllowInternalSSL`

Determine the exact contract meaning. It must not be confused with the I2PControl administrative HTTPS listener.

If it governs whether encrypted/TLS-like traffic is accepted/presented through an HTTP server path, implement only at the existing HTTP server filter/presentation boundary with no certificate interception and no local-target expansion.

If correct semantics require Java-specific local SSL handling unavailable in Emissary, retain a truthful blocker or evidence-backed applicability disposition rather than adding a TLS termination stack.

### WP3 — `UniqueLocalAddressPerClient` / `MultiHoming`

Treat these as high-risk routing/presentation options.

- establish whether Proposal 170 requires a portable externally visible behavior or simply exposes Java I2PTunnel local-address machinery;
- never derive a LAN/local destination from remote peer-controlled data;
- never allocate arbitrary host addresses or interfaces;
- if an implementation is possible using only synthetic presentation headers/identity at an existing loopback service boundary, keep it local and bounded;
- if semantics require real per-client local IP/interface allocation or non-loopback routing, stop and require a separate security/architecture decision rather than weakening M093.

### WP4 — LeaseSet capability check

Before code changes, prove one exact accepted primitive for:

- encrypted LeaseSet activation requested by `EncryptLeaseSet`;
- optional lookup behavior if it changes destination/session resolution;
- client authorization list/key handoff for `LeaseSetClientAuths`.

The primitive must be part of an accepted public Yosemite or existing neutral Emissary owner. No approximate application-layer flag counts.

### WP5 — LeaseSet secret ownership

If client-auth material is supported, reuse or extend the M110 confined secret owner rather than creating a second incompatible store.

- canonical bounded authorization entry format;
- maximum count and size;
- structural validation before activation;
- atomic/owner-only persistence if persistence is contractually required;
- RPC/log redaction;
- edit/restart rollback;
- no key material in M095/M105 or docs fixtures.

### WP6 — Fail-closed session activation

LeaseSet security options must be part of the server session generation before publication/forwarding.

If requested encryption/authentication cannot be established, start/restart fails. Do not retry with default/public LeaseSet security and do not report a running tunnel.

Editing security settings requires a new session generation and must not expose a transient weaker generation.

### WP7 — Matrix truthfulness

Move cells to `apply` only with actual session/presentation behavior. Evidence-backed `not_applicable` is allowed only where the pinned contract/reference shows a Java-specific mechanism without a portable requirement. All other unresolved cells remain blocked.

## 7. Failure, cancellation, restart, contention

Security configuration and secret validation precede session allocation/publication. Generation replacement is serialized per tunnel name but no lock crosses Yosemite/network I/O or filesystem sync.

A failed encrypted/authenticated LeaseSet start leaves no weaker live/public generation. A failed edit preserves the prior accepted generation when existing TunnelManager transaction semantics permit; otherwise it stops truthfully rather than downgrading.

Client-auth maps and presentation state are bounded and generation-local. Remote peers cannot create persistent configuration entries.

## 8. Compatibility and migration

Existing blocked options have no operational migration guarantee. New semantics apply only after validation. Existing server destination secrets remain in their established owner; M113 must not migrate them gratuitously.

No router.toml, frontend, global network-interface, or router-core storage migration.

## 9. Focused tests

At minimum:

- every implemented LeaseSet option reaches a real dependency/session primitive;
- requested encryption/client auth never falls back to weaker/default LeaseSet behavior;
- malformed/oversized/duplicate auth entries fail before allocation and never leak;
- edit/restart security transition is generation-safe;
- non-loopback/request-selected local routing remains rejected;
- `UniqueLocalAddressPerClient`/`MultiHoming` cannot be used for SSRF/LAN access or remote-controlled local address selection;
- HTTP/IRC identity/filter tests remain passing;
- exact matrix/applicability dispositions are regression-guarded.

## 10. Broad verification

Run the active roadmap's standard feature, containment, M095/M105, live-runtime, check, clippy, fmt-attempt and diff checks, plus focused M093 server/HTTP/IRC/admission/secret-store suites.

No new CI/fuzz/release infrastructure.

## 11. Acceptance criteria

M113 closes only when:

1. every target cell is `apply`, evidence-backed `not_applicable`, or explicitly remains blocked with an exact primitive reason;
2. every applied LeaseSet security option changes the actual server session/LeaseSet behavior;
3. no silent security downgrade is possible;
4. local target confinement and trusted peer identity remain intact;
5. no router-global multihoming/TLS/LeaseSet subsystem was added for parity;
6. secret ownership is bounded and redacted;
7. matrix/audit deltas and residual count are exact;
8. M061/M062/M093 security/containment evidence remains green;
9. closure decides whether M114 final reclosure is dependency-ready.

## 12. Stop conditions

Stop if:

- accepted Yosemite/current neutral owners cannot express encrypted/client-auth LeaseSet semantics;
- implementation requires a private SAM/I2CP/LeaseSet serializer, dependency fork/vendor, or Proposal-shaped core API;
- presentation options require non-loopback request-selected routing or a new host-interface subsystem;
- a security option can only be approximated by metadata;
- any failure path would downgrade LeaseSet security.

## 13. Closure evidence

Require cell-by-cell matrix, dependency/session proof, secret-store and no-downgrade tests, local-routing/anonymity review, generation failure/restart evidence, full verification outcomes, unresolved findings, M114 readiness decision, and internal-only attestation.

## 14. Internal-only boundary

No upstream issue/PR/review/submission/merge/adoption request, dependency contribution preparation, branch/tag push, release, or maintainer contact is authorized. External sources are read-only evidence only.
