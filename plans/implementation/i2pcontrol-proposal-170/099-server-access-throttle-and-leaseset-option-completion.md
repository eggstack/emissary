# M099 — Server Access, Throttle, and LeaseSet Option Completion

Status: blocked on M097

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md`.

Canonical requirements:

- `plans/000-long-term-specification.md`;
- `plans/003-planning-process.md`;
- ADR-0002 server secret ownership;
- ADR-0003 filtered-server/security boundary;
- ADR-0004 full-support completion boundary;
- M074-M090 server hardening authority;
- M093 current tunnel production/security reclosure authority.

Planning baseline: `630a8fd1cd4e5943fcde0b5c16f5fc1e88b5d207` plus M095/M097 closures when dependency-ready.

Pinned external contract: I2P Proposal 170 revision `2026-05-20`.

Classification: capability / security.

## 1. Objective

Complete every M095-assigned applicable Proposal 170 server-side configuration option across `server`, `httpserver`, `httpbidirserver`, `ircserver`, and `streamrserver` without weakening the existing accepted-stream admission, trusted peer identity, HTTP/IRC filtering, loopback target confinement, persistent destination identity, or bounded Streamr rules.

M099 extends option semantics around already-real backends. It is not authority to redesign the server data planes or reopen M088's lower-layer pre-accept residual.

## 2. Option classes

The exact matrix is frozen by M095. Expected groups include:

### HTTP/server presentation and access

- `WebsiteHostname`;
- `SpoofedHost`;
- `BlockAccessInProxies` where server semantics apply;
- `BlockUserAgents`;
- `UserAgents`;
- `UniqueLocalAddressPerClient`;
- `BlockReferers`;
- `MultiHoming`;
- `AccessOption`;
- `AccessList`;
- `FilterFilePath`.

### Connection/rate/POST controls

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

### LeaseSet controls

- `EncryptLeaseSet`;
- `OptionalLookup`;
- `LeaseSetClientAuths`;
- any LeaseSet/session key controls assigned by M095 that build on M097 common plumbing.

Do not duplicate M097 common `SigType`/`EncType`/tunnel-build handling except where LeaseSet-specific semantics require additional mapping.

## 3. Preserve current server admission authority

M074-M093 established the accepted application boundary:

```text
signed/validated I2P stream
    -> trusted peer Destination
    -> bounded ServerAdmissionState
    -> protocol-specific bounded filter where applicable
    -> literal-loopback local target
```

M099 options must compose with this boundary.

- `MaxConcurrentConns` and rate limits should reuse/extend the existing bounded application admission state rather than create parallel unbounded maps.
- client-scoped accounting uses the trusted cryptographic Destination ID, never attacker-supplied headers/hostnames.
- HTTP POST accounting remains keyed by trusted peer identity.
- access lists are evaluated against canonical I2P peer identity where that is the pinned meaning.
- no option may cause the local target connection to occur before the currently required validation/filter/admission stage.

## 4. Access controls

### 4.1 AccessOption/AccessList

Implement the exact allow/deny semantics established by M095/reference behavior.

Requirements:

- bounded number/size of entries;
- canonical Destination/hash parsing where peer identity is the subject;
- deterministic precedence;
- validation before server publication/listen/session allocation when possible;
- runtime lookup is bounded and does not perform DNS/network I/O;
- errors/logging do not disclose full private configuration unnecessarily.

### 4.2 FilterFilePath

If the option points to an access/filter file, use a server/I2PControl-owned confined configuration root:

- reject arbitrary absolute paths/traversal/symlink escape/special files;
- bound file size and entry count;
- parse a complete generation before publication;
- retain prior valid generation on reload failure;
- do not watch arbitrary filesystem trees unless the pinned semantics require reload; if reload is required, use one bounded owner task.

### 4.3 User-agent/referer controls

HTTP server filtering must apply these options in the existing bounded header parser before local target forwarding. The options must not reintroduce spoofable proxy/I2P identity headers or ambiguous framing.

### 4.4 WebsiteHostname/SpoofedHost/MultiHoming

Preserve exact Host/vhost semantics without request-controlled local target selection. MultiHoming is a presentation/routing policy for the accepted HTTP tunnel role, not permission for a remote peer to select arbitrary host/LAN backends.

`UniqueLocalAddressPerClient` may be implemented only if M095 establishes a safe, meaningful Emissary equivalent. It must not expose or allocate arbitrary host addresses or create a local side channel. If the reference semantic cannot be reproduced safely through the current loopback target model, keep the cell blocked and stop for architecture review.

## 5. Rate/throttle controls

Map all applicable connection/request/POST controls onto the existing bounded admission/throttle structures.

Requirements:

- finite configured maxima and periods;
- bounded per-peer cardinality/history;
- bounded global counters/windows;
- monotonic time;
- transactional admission where a rejected request cannot consume capacity permanently;
- expiry/cleanup bounded by existing limits;
- no lock held across local connect, protocol body relay, sleeps, or network I/O;
- `TotalBanTime` in this backend context is an application-tunnel temporary denial interval only if that is the pinned reference meaning. It must not silently become a router-wide peer-ban subsystem or feed RouterInfo `bannedpeers` unless M103 independently establishes identical canonical semantics.

This separation is critical: server-tunnel throttling and router peer bans are different ownership domains.

## 6. LeaseSet encryption/authentication

Implement the pinned `EncryptLeaseSet`, `OptionalLookup`, and `LeaseSetClientAuths` semantics through existing Yosemite/SAM/session options and M097 key/session plumbing.

Requirements:

- exact finite mode mapping from the canonical strings validated by the handler;
- no silent downgrade from encrypted/authenticated to public LeaseSet;
- client auth entries parsed/validated/bounded before destination/session publication;
- secrets/auth keys redacted from logs/errors/get output;
- persistent server destination identity remains stable across restart when config is unchanged;
- changing LeaseSet security settings follows the existing safe restart/identity semantics;
- failure occurs before publication where possible and leaves the durable definition recoverable.

If Yosemite/SAM lacks a required LeaseSet option, stop that cell rather than modify `emissary-core` under M099.

## 7. Backend applicability

- generic `server`: access/rate/session/LeaseSet options that make sense for raw accepted streams; no HTTP header options.
- `httpserver`: HTTP presentation/access/rate/POST + applicable LeaseSet/session options.
- `httpbidirserver`: inbound server behavior reuses `httpserver`; outbound side remains M098/ADR-0003 composition.
- `ircserver`: connection/access/session controls only; HTTP options not applicable.
- `streamrserver`: datagram/session controls that fit Streamr; TCP/HTTP admission options not applicable unless M095 proves otherwise.

No option may be accepted simply because it exists in the raw config.

## 8. Preferred authorized path boundary

Target changes stay under existing I2PControl server backends/filters/runtime, especially:

- `emissary-cli/src/i2pcontrol/backends/options.rs`;
- generic `server.rs`;
- `http_server.rs`, `http_bidir.rs`, `irc_server.rs`, `streamr.rs`;
- `backends/runtime/admission.rs` and other accepted bounded server runtime helpers;
- existing HTTP filter modules;
- I2PControl-owned secret/filter-file persistence identified by M095/M097;
- tunnel handler/domain only for exact typed extraction/serialization;
- focused tests/docs/M095 matrix updates.

No new `emissary-core/**`, startup server adoption, root dependency/vendor, frontend, or workflow change is authorized.

## 9. Invariants

1. Trusted peer identity remains Yosemite-derived and canonicalized.
2. Application admission remains bounded and post-accept as documented by M093.
3. HTTP/IRC filters remain mandatory/non-bypassable.
4. Local server targets remain literal loopback/confined.
5. Rate/access state is bounded and generation-local.
6. Tunnel temporary bans do not become router-wide peer bans.
7. LeaseSet security never silently downgrades.
8. Secrets/path values remain confined/redacted.
9. Every accepted applicable option changes real runtime behavior.
10. No upstream interaction occurs.

## 10. Explicit non-goals

M099 MUST NOT:

- reopen pre-accept Yosemite/stream concurrency work from M091;
- add router-wide peer banning;
- add DCC/WEBIRC;
- add request-selected LAN/backend routing;
- weaken Host/framing/spoof protections;
- implement client proxy options owned by M098;
- add new core LeaseSet/tunnel APIs;
- vendor/patch Yosemite;
- add CI/release machinery;
- interact upstream.

## 11. Ordered work packages

A. Freeze M099 option/type matrix subset.

B. Extend existing admission/throttle configuration with exact Proposal 170 fields and bounded validation.

C. Integrate access-list/filter-file semantics into the existing pre-local-target security path.

D. Integrate HTTP presentation/privacy controls into the accepted HTTP parser/filter.

E. Implement LeaseSet encryption/auth/lookup mapping via current Yosemite/SAM and M097 plumbing.

F. Reconcile edit/restart/persistence/get behavior and update M095 matrix/support docs.

## 12. Failure/cancellation/restart/contention semantics

- invalid access/rate/LeaseSet config fails before allocation/publication;
- filter-file reload failure retains prior complete generation;
- rate windows and temporary denials are cancelled/dropped with the owning tunnel generation;
- restart reconstructs bounded admission/filter state from durable config, not transient peer history;
- concurrent connections use transactional capacity accounting already established by M080/M083;
- concurrent edit/lifecycle follows current per-name generation ownership;
- no lock crosses network I/O/sleeps/joins.

## 13. Compatibility/migration

Prefer existing tunnel raw/canonical config schema. Add versioned secret/filter metadata only if unavoidable. Existing definitions without these options retain current secure defaults.

Do not expose `LeaseSetClientAuths`, passwords, private keys, or filter-file contents through ordinary `get`.

## 14. Tests

At minimum:

- full M099 applicability fixtures;
- allow/deny list canonical peer matching;
- filter path traversal/symlink/special-file/size bounds;
- MaxConcurrentConns and all period/rate counters;
- POST limits keyed by trusted peer identity;
- temporary-denial expiry and bounded cardinality;
- Host/UserAgent/Referer/MultiHoming behavior without spoof/framing regression;
- LeaseSet mode mapping and no downgrade;
- client auth secret redaction;
- restart identity stability;
- httpbidir inbound composition;
- IRC/Streamr negative applicability tests;
- existing M074-M093 security regressions.

## 15. Verification

Run focused server/filter/admission tests plus:

```text
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo test -p emissary-cli --no-default-features --features i2pcontrol
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m062_dependency_containment
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m063_feature_reachability
git diff --check
```

## 16. Documentation/static guards

Update M095 cells only with runtime evidence. Document tunnel-vs-router ban separation explicitly. Keep overall support partial until M104.

Add guards that secret/path-bearing server options cannot leak through canonical `get` and every M099-owned option is classified across the five server families.

## 17. Acceptance and stop conditions

M099 closes only if every applicable M099 cell is operational, current server security invariants remain green, and no new core/dependency boundary is crossed.

Stop if parity would require router-wide banning, unsafe local address allocation, unrestricted filesystem access, missing Yosemite LeaseSet primitives, or changes to lower-layer pre-accept behavior.

## 18. Closure evidence required

Create `plans/closure/i2pcontrol-proposal-170/099-closure.md` containing:

- M095/M097 dependency evidence;
- exact changed paths;
- per-option/per-server-type runtime matrix;
- access/filter/throttle/LeaseSet security evidence;
- trusted-peer/admission regression evidence;
- restart/failure/contention evidence;
- containment results;
- updated M095 matrix;
- unresolved blockers/findings;
- internal-only/no-upstream attestation.

## 19. Internal-only rule

All writes remain internal to `eggstack/emissary`; external sources are read-only. No upstream issue/PR/review/submission/merge/contribution activity is authorized.