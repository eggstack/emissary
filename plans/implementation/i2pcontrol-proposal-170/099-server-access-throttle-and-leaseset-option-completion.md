# M099 — Server Access, Throttle, and LeaseSet Option Completion

Status: **closed internally against pinned revision; partial with explicit residual blockers**

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md`.

Canonical requirements:

- `plans/000-long-term-specification.md`;
- `plans/003-planning-process.md`;
- ADR-0002 server secret ownership;
- ADR-0003 filtered-server/security boundary;
- ADR-0004 full-support completion boundary;
- M074-M090 server hardening authority;
- M093 current tunnel production/security reclosure authority;
- M095 full-support matrix;
- `plans/closure/i2pcontrol-proposal-170/097-closure.md`;
- revised M098 dependency decomposition and its closure when available.

Planning baseline: current `master` after M103 closure (`30cd8bcc9728c286b418cfb534d4f19c6b1eb4f5`) plus revised M098 planning.

Pinned external contract: I2P Proposal 170 revision `2026-05-20`.

Classification: capability / security / corrective dependency decomposition.

## 1. Corrective context

The original M099 plan made successful M097 completion a hard dependency for the entire server option family. M097 has now closed **as blocked**, and its closure demonstrates that this dependency was too coarse.

Most server HTTP presentation, access-control, filter, connection-admission, rate, and POST-throttle options compose with the already-accepted M074-M093 server runtime. They do not inherently require the unresolved M097 shared-session, client-key, or Yosemite `SESSION CREATE` serializer primitives.

The LeaseSet/session-security tail does depend on lower-layer/session capabilities and remains blocked where those primitives are absent. This revision therefore splits M099 at the option-cell boundary instead of blocking all server work behind M097.

M099 is serialized behind M098 only because both passes update the shared M095 matrix, option validator, HTTP/filter surfaces, and planning control files. This is an integration-order constraint, not a semantic dependency between client and server capabilities.

Why prior planning missed this: M098/M099 were prewritten before M097 execution, so the dependency graph was milestone-oriented. M097 closure now provides concrete per-primitive evidence that permits a safe split.

## 2. Objective

Implement every M099-assigned Proposal 170 server option cell that can be given exact runtime semantics using the existing accepted server admission/filter/runtime owners, while transferring genuinely M097-dependent or otherwise unsafe/unowned cells to explicit residual blockers before production code begins.

This milestone does not redesign server data planes, does not reopen M088's lower-layer pre-accept residual, and does not claim full Proposal 170 support while residual blocked cells exist.

## 3. Dependency model

### Hard

- M095 full-support matrix exists and remains authoritative after per-cell reconciliation.
- M074-M093 accepted server runtime/security invariants remain current.
- M098 closure must exist before M099 begins, solely to serialize shared matrix/filter/option edits and consume its final cell ownership.

### Interface/evidence

- M097 closure is stable evidence for unresolved session/key/Yosemite primitives.

Successful M097 completion is **not** a hard prerequisite for the independent M099 slice.

## 4. Mandatory pre-code cell reconciliation

Before production edits, audit every M099-owned `planned_apply` cell and any server-role cells transferred from M098 against current server runtime ownership and M097 closure evidence.

Each applicable server cell MUST become exactly one of:

1. `apply` — already operational and evidenced;
2. `planned_apply` owned by M099 — exact runtime behavior is implementable within existing I2PControl server/filter/admission owners;
3. `blocked_primitive` — exact behavior requires a named unavailable primitive or a security-relevant owner that does not exist; ownership is transferred to the residual blocked line;
4. `not_applicable` — supported by pinned-contract/reference semantics and per-type rationale.

No production implementation starts until the matrix and matrix guard encode this split.

Expected independent candidates include HTTP presentation/filter policy, access lists, confined filter-file loading, accepted-connection ceilings, per-peer/aggregate rate windows, POST controls, and tunnel-local temporary denial. LeaseSet encryption/authentication options are expected residual candidates unless the supported Yosemite/SAM path is demonstrably available at execution time.

## 5. Server HTTP presentation and filter family

Audit and, where applicable, implement:

- `WebsiteHostname`;
- `SpoofedHost`;
- `BlockAccessInProxies`;
- `BlockUserAgents`;
- `UserAgents`;
- `BlockReferers`;
- `MultiHoming`;
- server-role `AllowInternalSSL` if transferred from M098.

Requirements:

- reuse the existing bounded HTTP request/response filter;
- preserve framing, Host, Expect, proxy/I2P identity, fingerprint, and response-sanitization protections from M076/M082/M090/M093;
- presentation options may not allow a remote peer to select arbitrary local/LAN backends;
- `MultiHoming` must remain a bounded HTTP presentation/routing policy, not request-controlled backend routing;
- `AllowInternalSSL` must not become arbitrary TLS trust bypass;
- no second HTTP parser/filter stack.

`UniqueLocalAddressPerClient` requires a separate exact semantic check in the pre-code audit. It remains M099-owned only if a safe meaningful equivalent exists within the literal-loopback local-target model. Do not allocate arbitrary local addresses or invent an approximation to make the matrix green.

## 6. Access-control family

Audit and implement where exact:

- `AccessOption`;
- `AccessList`;
- `FilterFilePath`.

Requirements:

- canonical peer identity comes from the trusted Yosemite-derived Destination, never attacker-controlled HTTP headers;
- lists are bounded, canonicalized, and deterministic;
- validation occurs before publication/listen/session allocation when possible;
- runtime lookup performs no DNS or network I/O;
- `FilterFilePath` is confined beneath an I2PControl/server administrative root and rejects absolute paths, traversal, symlink escape, special files, oversized files, and oversized entry sets;
- a complete filter generation is parsed before publication;
- reload failure retains the prior valid generation;
- no arbitrary filesystem watch hierarchy is introduced.

## 7. Admission/rate/throttle family

Audit and implement through the existing bounded admission/throttle state:

- `MaxConcurrentConns`;
- `ClientPerMinute`;
- `ClientPerHour`;
- `ClientPerDay`;
- `TotalInPerMinute`;
- `TotalInPerHour`;
- `TotalInPerDay`;
- `PostLimit`;
- `PostLimitTime`;
- `PerClientPeriod`;
- `TotalPeriod`;
- `TotalBanTime`.

Requirements:

- finite configured maxima and periods;
- bounded per-peer and global history/cardinality;
- monotonic time;
- trusted cryptographic peer identity for client-scoped accounting;
- transactional admission so rejected connections do not permanently consume capacity;
- no lock across target connect, body relay, sleeps, or network I/O;
- generation-local cleanup on stop/restart;
- `TotalBanTime` is tunnel-local temporary denial only and MUST NOT become a router-wide peer-ban source or feed RouterInfo `bannedpeers`.

## 8. LeaseSet/session-security residual family

Audit:

- `EncryptLeaseSet`;
- `OptionalLookup`;
- `LeaseSetClientAuths`;
- any M099 server-role cell whose exact semantics require a common M097 session/key primitive.

A LeaseSet cell remains M099-owned only if the existing supported Yosemite/SAM/session path can perform the exact requested behavior without silent downgrade.

Otherwise it is transferred to an explicit residual `blocked_primitive` disposition before server production work begins.

Rules:

- no silent encrypted/authenticated-to-public downgrade;
- auth/key material remains bounded and redacted;
- changing LeaseSet security follows accepted restart/identity semantics;
- no new core LeaseSet API;
- no vendored/forked/patched Yosemite;
- no dependency change under this plan.

## 9. Backend applicability

Apply only matrix-authorized behavior to:

- `server`;
- `httpserver`;
- inbound/server role of `httpbidirserver`;
- `ircserver`;
- `streamrserver` only for options the pinned matrix proves applicable.

HTTP-only fields must not leak into raw/IRC/Streamr behavior. Streamr remains under its bounded datagram contract.

## 10. Authorized production boundary

Preferred changes are restricted to existing I2PControl server/filter/runtime paths, especially:

- `emissary-cli/src/i2pcontrol/backends/options.rs`;
- `backends/server.rs`;
- `backends/http_server.rs`;
- `backends/http_bidir.rs`;
- `backends/irc_server.rs`;
- `backends/streamr.rs` only for proven applicable fields;
- `backends/runtime/admission.rs` and existing bounded runtime helpers;
- existing HTTP filter modules;
- I2PControl-owned confined filter/access-file state if required;
- tunnel handler/domain only for exact typed extraction/serialization;
- focused tests, docs, M095 matrix, registry, and roadmap records.

No new `emissary-core/**`, `emissary-util/**`, Cargo dependency/lockfile, startup server redesign, frontend, or workflow change is authorized.

## 11. Invariants

1. Trusted peer identity remains Yosemite-derived and canonicalized.
2. Application admission remains bounded and post-accept as documented by M093.
3. HTTP/IRC filters remain mandatory and non-bypassable.
4. Local server targets remain literal loopback/confined.
5. Rate/access state is bounded and generation-local.
6. Tunnel-local temporary denial does not become router-wide peer banning.
7. LeaseSet security never silently downgrades.
8. Secrets and path values remain confined/redacted.
9. Every accepted applicable option changes real runtime behavior.
10. M097 blocked primitives remain blocked rather than approximated.
11. No upstream interaction occurs.

## 12. Explicit non-goals

M099 MUST NOT:

- reopen M088/M091 lower-layer pre-accept work;
- add router-wide peer banning;
- add DCC/WEBIRC;
- add request-selected LAN/backend routing;
- weaken Host/framing/spoof protections;
- implement client proxy options owned by M098;
- add new core LeaseSet/tunnel APIs;
- vendor, fork, patch, or replace Yosemite;
- introduce unrestricted filesystem access;
- add CI/release machinery;
- interact upstream.

## 13. Ordered work packages

A. Consume M098 closure and reconcile all server-role option ownership in the M095 matrix.

B. Implement HTTP presentation/filter options proven independent.

C. Implement bounded access-list/filter-file semantics.

D. Extend existing admission/throttle configuration with exact independent Proposal 170 controls.

E. Re-audit LeaseSet/session-security cells and transfer unresolved primitives rather than approximating them.

F. Reconcile edit/restart/persistence/get behavior, docs, matrix, and final residual-blocker ledger.

G. Determine whether any dependency-ready follow-on plan exists. Do not activate M104 while any applicable residual cell is blocked.

## 14. Failure, cancellation, restart, and contention

- invalid access/rate/filter config fails before allocation/publication;
- filter-file reload failure retains the prior complete generation;
- rate windows and temporary denials disappear with the owning tunnel generation;
- restart reconstructs configuration but not transient peer history;
- concurrent connections retain transactional capacity accounting;
- concurrent edit/lifecycle uses current per-name generation ownership;
- no lock crosses network I/O, sleeps, joins, or cancellation waits;
- residual blocked cells fail before allocation.

## 15. Compatibility and migration

Prefer the existing tunnel canonical/raw config schema. Existing definitions without new independent options retain current secure defaults.

Add versioned I2PControl-owned filter metadata only if unavoidable and bounded. Any broader persistent schema need requires a separate plan.

Do not expose passwords, private keys, LeaseSet client-auth secrets, or filter-file contents through ordinary canonical `get`.

## 16. Tests

At minimum:

- exhaustive revised M099 option/type applicability fixtures;
- regression guard proving no M099-owned applicable cell remains ambiguously dependent on M097;
- Host/presentation/User-Agent/Referer/proxy-access behavior without spoof/framing regressions;
- allow/deny canonical peer matching;
- filter path traversal/symlink/special-file/size bounds;
- `MaxConcurrentConns` and every admitted period/rate counter;
- POST limits keyed by trusted peer identity;
- temporary-denial expiry and bounded cardinality;
- restart/generation cleanup;
- httpbidir inbound composition;
- IRC/Streamr negative-applicability tests;
- LeaseSet no-downgrade negative tests for cells left blocked;
- retained M074-M093 server security regressions;
- feature-off/default behavior.

## 17. Verification

Run focused server/filter/admission tests plus:

```text
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo test -p emissary-cli --no-default-features --features i2pcontrol
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m062_dependency_containment
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
git diff --check
```

If the historical `m063_feature_reachability` target remains absent, record it as a repository test-inventory limitation rather than creating unrelated scope.

Do not create formatter-only churn across audited core files for the existing nightly/stable rustfmt drift.

## 18. Documentation/static guards

Update the M095 matrix before code and at closure. Update tunnel-manager/support/security docs only for proven runtime behavior.

Static guards must ensure:

- every M099-owned applicable cell is `apply` at closure;
- transferred residual cells name exact blockers and fail before allocation;
- secret/path-bearing values do not leak through canonical `get`;
- tunnel-local denial never feeds RouterInfo banned peers;
- no new lower-layer/dependency path is introduced.

Overall Proposal 170 status remains partial.

## 19. Acceptance and stop conditions

M099 closes when:

- M098 closure has been consumed and server-role matrix ownership is reconciled;
- every applicable cell still owned by M099 is operational `apply` with runtime evidence;
- every excluded cell is explicitly `not_applicable` or transferred to a named residual blocker;
- M093 security invariants remain green;
- no unauthorized core/dependency expansion occurred;
- the registry truthfully identifies the next blocker rather than advancing M104 prematurely.

Stop on any option requiring router-wide banning, unsafe local address allocation, unrestricted filesystem access, missing Yosemite/SAM LeaseSet/session primitives, new lower-layer core behavior, or dependency changes.

Closing revised M099 does **not** mean the full TunnelManager option matrix is complete while any residual blocked cell remains.

## 20. Closure evidence required

Create `plans/closure/i2pcontrol-proposal-170/099-closure.md` containing:

- M095/M097/M098 dependency evidence;
- before/after server cell ownership and counts;
- exact changed paths;
- per-option/per-server-type runtime evidence;
- access/filter/throttle security evidence;
- trusted-peer/admission regression evidence;
- LeaseSet residual/no-downgrade evidence;
- restart/failure/contention evidence;
- containment results;
- updated M095 matrix;
- residual blocked cells and exact primitives;
- next-plan/M104 disposition;
- unresolved findings;
- internal-only/no-upstream attestation.

## 21. Internal-only rule

All writes remain internal to `eggstack/emissary`; external sources are read-only. No upstream issue, PR, review, submission, merge, contribution preparation, adoption request, or maintainer contact is authorized.
