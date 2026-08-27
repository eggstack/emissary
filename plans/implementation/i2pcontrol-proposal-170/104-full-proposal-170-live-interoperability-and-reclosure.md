# M104 — Full Proposal 170 Live Interoperability and Reclosure

Status: blocked on M096-M103

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md`.

Canonical requirements:

- `plans/000-long-term-specification.md`;
- `plans/002-long-term-roadmap.md`;
- `plans/003-planning-process.md`;
- ADR-0004 full-support completion boundary;
- retained ADR-0001/0002/0003 protocol/runtime/security boundaries;
- M061/M062/M063 containment authorities;
- M093 tunnel production/security reclosure authority;
- accepted closures for M095-M103 when dependency-ready.

Planning baseline: `630a8fd1cd4e5943fcde0b5c16f5fc1e88b5d207` plus the integrated closed M095-M103 production head.

Pinned external contract: I2P Proposal 170, status `Open`, revision `2026-05-20`.

Classification: invariant / capability / operations / closure.

## 1. Objective

Perform the independent integrated acceptance pass required before the repository may describe itself as fully supporting Proposal 170 against the pinned `2026-05-20` revision.

M104 is primarily verification, interoperability, reconciliation, and tiny corrective integration work. It must not become a catch-all implementation milestone. Material source/configuration/option/security defects discovered here create a new numbered corrective plan rather than being absorbed into M104.

## 2. Hard readiness gate

M104 cannot begin until accepted closure records exist for:

- M095 exact full-support matrix/containment budget;
- M096 AddressBook SetConfig;
- M097 common tunnel session/key options;
- M098 client/proxy/management/HTTP options;
- M099 server/access/throttle/LeaseSet options;
- M100 transit 15-second source;
- M101 router news source;
- M102 IPv4/IPv6 network-error owner;
- M103 banned-peer semantic completion.

If M103 closed blocked/path C, M104 remains blocked and must not dilute the full-support definition.

## 3. Required final conformance state

M104 must independently reconcile the M095 machine-readable matrix to the production head.

Required target:

### RouterInfo

- all 43 pinned Proposal 170 additions have truthful operational sources/semantics;
- no applicable row remains `unavailable`;
- any protocol-defined neutral behavior remains only where explicitly permitted by the pinned contract;
- news, transit 15s, v4/v6 network errors, and banned peers are specifically re-audited rather than trusted from owning closures alone.

### AddressBook

- exact CRUD for private/local/router/published books;
- exact SetSubscriptions behavior;
- all 13 SetConfig keys have the M095/M096 operational disposition;
- addressbook/subscription/config RouterInfo selectors reflect the active durable generation;
- path confinement and restart behavior remain correct.

### TunnelManager

- exactly 12 canonical tunnel types;
- exactly 7 canonical actions;
- canonical lowercase action/type behavior and response shapes exact;
- all applicable canonical option/type cells are `apply` with real runtime evidence;
- all `not_applicable` cells have rationale;
- no applicable `unsupported`/`blocked_primitive` cell remains;
- compatibility extensions such as historical `List`/capitalized actions are clearly separated and do not alter canonical behavior.

### ClientServicesInfo

- exactly 6 Proposal 170 selectors and their exact direct-presence semantics;
- live TunnelManager inventory and SAM observation remain bounded/current;
- BOB/I2CP behavior remains truthful.

### Base I2PControl scope

Unrelated base methods remain explicitly outside this Proposal 170 closure. M104 must not call their absence a Proposal 170 failure unless the pinned proposal text actually requires them.

## 4. Live interoperability requirement

M104 must supplement local/unit evidence with a focused real-network environment.

Preferred environment:

- feature-enabled Emissary launched as a real router with I2PControl TLS/authentication;
- a reseeded peer set or otherwise functional I2P network connectivity;
- one reference router implementation (Java I2P and/or i2pd) where practical for data-plane interoperability and adopted field semantics;
- local loopback services for HTTP/IRC/raw TCP/UDP targets;
- deterministic teardown and bounded timeouts.

This is an acceptance harness, not a permanent hosted CI matrix. It may be a documented manual/local script/test mode if reliable automation is impractical in the repository environment.

Do not require privileged namespaces or environmental machinery unrelated to Proposal 170. Reuse practical process/container tooling only if it adds direct interoperability evidence.

## 5. API acceptance matrix

### 5.1 Authentication/transport

Verify:

- real TLS listener;
- token authentication/version behavior required by the implemented Proposal 170 surface;
- unauthenticated mutation/read rejection;
- request ID preservation and JSON-RPC envelopes;
- bounded malformed/oversized request handling.

### 5.2 RouterInfo

Exercise every one of the 43 selectors at least through direct production composition. For dynamic source rows, induce or observe meaningful non-default state where practical.

Specifically:

- news returns the adopted real news representation;
- transit 15s is non-request-driven and responds to actual transit traffic when a test can create it;
- network error values correlate with explicit source state, including a non-No-error fixture/integration path where practical;
- banned peers follows the M103 authoritative semantic path;
- addressbook selectors reflect live administrative mutations;
- log clear affects only the I2PControl log buffer as previously accepted.

### 5.3 AddressBook

Over authenticated I2PControl:

- add/update/delete each relevant administrative book;
- SetSubscriptions and verify active downloader/durable generation behavior;
- SetConfig across all 13 keys using safe temporary administrative roots;
- restart router/service and verify persistence/configuration publication;
- test invalid/path-escape configuration and prove no partial mutation;
- verify publication/proxy/update-delay behavior using bounded local fixtures where external network dependency is unnecessary.

### 5.4 TunnelManager lifecycle

For each of 12 types:

1. `create` with a minimal valid applicable configuration;
2. `get` and verify canonical info/rawConfig without secrets;
3. `start`;
4. prove a real data-plane/session effect appropriate to the type;
5. exercise representative option set from M095, including security-sensitive options in negative/positive fixtures;
6. `restart` and prove resource generation changes while persistent identity remains stable where required;
7. `stop`;
8. `delete`;
9. verify no stale listener/session/task/secret state.

For `All`, verify start/stop/restart over a bounded mixed inventory and deterministic partial-error semantics.

### 5.5 Data-plane interoperability by family

- `client`: local TCP -> remote I2P destination -> traffic round trip.
- `server`: remote I2P client -> persistent server destination -> literal-loopback target.
- `httpclient`: direct `.i2p` HTTP; configured outproxy path if available; privacy/auth behavior.
- `connectclient`: CONNECT direct I2P and outproxy/clearnet policy.
- `socks`: SOCKS4a/SOCKS5 accepted CONNECT semantics.
- `socksirc`: SOCKS plus common IRC filter.
- `ircclient`: filtered IRC registration/traffic with DCC/unsafe CTCP still fail-closed as accepted.
- `httpserver`: remote I2P HTTP -> filtered local target; trusted identity/header/framing/response filtering remains active.
- `httpbidirserver`: inbound HTTP server plus composed outbound local proxy behavior; no third HTTP stack/outproxy violation.
- `ircserver`: bounded filtered registration and raw post-registration relay with half-close/inactivity behavior.
- `streamrclient`/`streamrserver`: actual I2P datagram subscribe/refresh/payload/unsubscribe path with bounded local UDP endpoints.

A type is not accepted merely because `start` returns success. Real data-plane evidence is required.

## 6. Server anonymity/security recheck

M104 must retain M093 as the prior deep security authority but re-run focused regression evidence at the integrated full-option head for changes that could affect security:

- trusted peer Destination parsing/canonicalization/accounting;
- application admission bounds/transactionality;
- literal-loopback target confinement;
- HTTP framing, spoofed identity, Host, fingerprint, Expect, POST/rate behavior;
- IRC registration filters, DCC/CTCP policy, half-close/inactivity;
- LeaseSet encryption/auth no-downgrade;
- access/filter-file path confinement;
- Streamr subscriber/payload/fanout bounds;
- key/password/path redaction;
- client proxy/outproxy no-local-DNS/LAN-open-proxy behavior.

M088's lower-layer pre-accept residual remains accepted unless this phase introduced a direct regression or new evidence that materially changes its severity. M104 does not reopen M091 by default.

## 7. Containment reclosure

Compare the final production head against:

- pinned upstream baseline used by M061;
- current M061 exact containment manifest;
- M062/M063 dependency/feature authority;
- M095 path budgets and M102/M103 lower-layer exceptions.

Required conclusion:

- Proposal 170 policy remains under `emissary-cli/src/i2pcontrol/**`;
- every non-I2PControl production path changed during M095-M103 has explicit owner/seam/rationale/evidence;
- no broad prefix/glob was added for convenience;
- no I2PControl-only dependency leaked into default/features;
- no vendored/git dependency or unauthorized Yosemite change exists;
- no frontend coupling.

The raw number of lower-layer changed paths is less important than exact necessity, but M104 should flag any path whose rationale no longer holds after implementation.

## 8. Authorized M104 production scope

M104 is not an implementation catch-all.

Authorized changes:

- test/interoperability harnesses;
- support/conformance documentation;
- M095 matrix final reconciliation;
- registry/roadmap/implementation README/closure bookkeeping;
- tiny directly demonstrated integration fixes entirely inside already-budgeted `emissary-cli/src/i2pcontrol/**` if they do not alter architecture/security ownership and can be evidenced within M104.

Any of the following requires a separate corrective plan:

- new/changed core behavior or path;
- new dependency;
- new persistent schema design;
- material AddressBook/TunnelManager source semantics;
- security/anonymity defect requiring more than a tiny local fix;
- new tunnel data-plane mechanism;
- unresolved applicable matrix cell.

## 9. Failure/recovery/contention acceptance

Integrated evidence must cover:

- service/router restart with durable AddressBook/TunnelManager state;
- stable server destination identities where configured/persistent;
- failed bind/start leaves definition recoverable;
- source/news/config worker failure preserves prior valid generation as specified;
- concurrent TunnelManager edit/start/stop/restart stays exact-name/generation-safe;
- concurrent server connections preserve bounded admission/rate accounting;
- stop/restart cancels sampler/news/worker/runtime tasks without stale generation effects;
- malformed requests do not partially mutate state;
- cleanup is bounded and test environment leaves no orphan child processes/listeners.

## 10. Compatibility/migration review

Verify:

- old persisted definitions/state from the pre-full-support production baseline still load or migrate deterministically;
- AddressBook configuration schema migration is explicit and recoverable;
- compatibility aliases/extensions remain compatible but are not required for canonical closure;
- canonical clients see no renamed key/action/type/status;
- default/feature-disabled Emissary behavior remains unchanged.

## 11. Verification commands

At minimum, on the final production head:

```text
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo test -p emissary-core
cargo test -p emissary-cli --no-default-features --features i2pcontrol
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m062_dependency_containment
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m063_feature_reachability
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol -- -D warnings
git diff --check
```

Run the focused live interoperability harness and record exact command/environment/outcome.

Use the repository's accepted rustfmt/toolchain policy. Do not create formatter-only churn across audited core files to satisfy unrelated nightly/stable drift.

No new hosted CI, fuzz farm, coverage gate, or broad platform matrix is required for closure.

## 12. Documentation end state

If and only if M104 closes successfully, update:

- `docs/i2pcontrol/proposal-170-support.md`;
- `docs/i2pcontrol/proposal-170-conformance.md`;
- relevant AddressBook/TunnelManager/runtime docs;
- `plans/registry.md`;
- full-support subsystem roadmap;
- implementation README.

The final support statement must include the revision pin:

> Emissary fully supports I2P Proposal 170 against the pinned 2026-05-20 revision.

Also state that Proposal 170 remains Open and a later revision requires a new delta audit.

Do not claim upstream adoption/review/merge or general full I2PControl parity.

## 13. Static final guards

Maintain or add focused guards that make the final claim auditable:

- exact matrix row/cell exhaustiveness;
- no `planned_apply`, applicable `unsupported`, `blocked_primitive`, or unknown final cell;
- exact 43 RouterInfo / 13 SetConfig / 12 types / 7 actions / 6 service selectors;
- no secret-bearing canonical get output;
- exact containment path assertions;
- Proposal 170/wire policy absent from neutral core declarations;
- by-design-empty banned-peer proof remains valid if M103 used path B;
- request-independent transit sampler ownership;
- no per-request news fetch.

Avoid brittle test-count or line-number assertions.

## 14. Acceptance and stop conditions

M104 closes only when:

- every pinned applicable Proposal 170 matrix cell has production evidence;
- live interoperability proves all twelve tunnel types carry their intended traffic;
- AddressBook all-key SetConfig and persistence are operational;
- all RouterInfo rows are truthfully operational;
- ClientServicesInfo remains exact/current;
- no high/medium correctness/security/containment defect remains;
- default/feature-disabled behavior is unchanged;
- full matrix/static/containment tests pass;
- documentation uses the revision-pinned full-support statement and no broader claim;
- no upstream interaction occurred.

M104 must stop and open a new corrective plan if it finds:

- any applicable matrix cell still unsupported/blocked;
- a materially wrong source semantic;
- a live tunnel family that starts but cannot carry traffic;
- a path/auth/identity/anonymity defect;
- unplanned core/dependency expansion;
- a substantive persistence/migration defect;
- a material change in Proposal 170 from the pinned revision.

Partial completion is not full closure. Keep support documentation partial until corrective work closes.

## 15. Closure evidence required

Create `plans/closure/i2pcontrol-proposal-170/104-closure.md` containing:

- exact integrated production head and proposal revision;
- M095-M103 closure dependency table;
- final machine-readable matrix summary and artifact hash;
- full RouterInfo 43-row evidence summary;
- AddressBook CRUD/subscriptions/13-key SetConfig evidence;
- TunnelManager 12-type/7-action/all-option applicability/runtime matrix;
- ClientServicesInfo six-selector evidence;
- live environment topology and exact interoperability commands/results;
- per-tunnel-family data-plane evidence;
- server/client security regression evidence;
- persistence/restart/failure/contention evidence;
- containment/dependency/default-feature comparison;
- exact changed-path review for M104 itself;
- verification command outcomes;
- unresolved findings with severity;
- explicit final disposition: `closed internally against pinned revision`, `corrective pass required`, or `blocked`;
- explicit statement that unrelated base I2PControl methods are outside Proposal 170 closure;
- explicit external-sources-read-only/no-upstream-write/review/submission attestation.

## 16. Internal-only rule

All work and writes remain internal to `eggstack/emissary`.

External I2P, i2pd, Java I2P, go-i2p, Yosemite, specifications, issues, commits, and pull requests may be inspected read-only only. No upstream issue, PR, review request, submission, merge/adoption request, contribution preparation, branch/tag push, release, or maintainer contact is authorized.