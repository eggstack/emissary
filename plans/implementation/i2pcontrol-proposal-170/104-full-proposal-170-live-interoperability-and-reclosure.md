# M104 — Full Proposal 170 Live Interoperability and Reclosure

Status: **closed as blocked — residual TunnelManager primitives remain unresolved**

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
- accepted closures for M095-M103;
- revised M098/M099 corrective dependency decomposition.

Pinned external contract: I2P Proposal 170, status `Open`, revision `2026-05-20`.

Classification: invariant / capability / operations / closure.

## 1. Objective

Perform the independent integrated acceptance pass required before the repository may describe itself as fully supporting Proposal 170 against the pinned `2026-05-20` revision.

M104 is verification, interoperability, reconciliation, and at most tiny directly demonstrated integration repair inside already-authorized I2PControl paths. It is not a catch-all implementation milestone.

## 2. Corrected readiness gate

The original M104 gate depended on M097-M103 at milestone granularity. M097 has since closed as blocked, and M098/M099 were corrected to extract work that does not actually depend on those unresolved primitives.

M104 now requires all of the following:

- M095 closure accepted;
- M096 closure accepted;
- M100 closure accepted;
- M101 closure accepted;
- M102 closure accepted;
- M103 closure accepted;
- revised M098 independent client/proxy/HTTP slice closed;
- revised M099 independent server/access/throttle slice closed;
- the final M095 TunnelManager matrix contains **no applicable `planned_apply` or `blocked_primitive` cell**;
- every residual cell transferred out of M098/M099 has subsequently been resolved by a separately registered bounded plan and closure;
- no unresolved M097 shared-session, session-wire, destination/key-lifecycle, private-key-import, or LeaseSet/session-security blocker remains.

Therefore M104 reached its blocked verification stop condition. M098/M099
progress alone cannot make it executable while any residual blocked option
exists.

This preserves the original full-support definition rather than weakening it to fit current dependency limitations.

## 3. Required final conformance state

### RouterInfo

- all 43 pinned Proposal 170 additions have truthful operational sources/semantics;
- no applicable row is unavailable;
- protocol-defined neutral behavior appears only where explicitly permitted by the pinned contract;
- news, transit 15s, v4/v6 network errors, and banned peers are re-audited from production composition.

Current owning milestones M100-M103 are closed, but M104 independently verifies their integrated behavior.

### AddressBook

- CRUD for the four administrative books;
- `SetSubscriptions` behavior;
- all 13 `SetConfig` keys operational under the M096 confined owner;
- RouterInfo AddressBook selectors reflect active durable state;
- restart, publication, downloader, and path-confinement semantics remain correct.

### TunnelManager

- exactly 12 canonical tunnel types;
- exactly 7 canonical actions;
- exact lowercase canonical action/type behavior and response shapes;
- every applicable canonical option/type cell is `apply` with real runtime evidence;
- every `not_applicable` cell has pinned semantic rationale;
- no applicable `planned_apply`, `unsupported`, `blocked_primitive`, or unknown cell remains;
- compatibility extensions remain explicitly separate from canonical behavior.

### ClientServicesInfo

- exactly 6 Proposal 170 selectors with direct-presence semantics;
- live TunnelManager inventory and bounded SAM observation remain current;
- BOB/I2CP behavior remains truthful.

### Base I2PControl scope

Unrelated base methods remain outside this Proposal 170 closure. Their absence is not a Proposal 170 failure unless the pinned revision actually requires them.

## 4. Live interoperability environment

M104 must supplement local/unit evidence with focused real-network evidence.

Preferred environment:

- feature-enabled Emissary as a real router with I2PControl TLS/authentication;
- reseeded or otherwise functional I2P connectivity;
- Java I2P and/or i2pd as a read-only reference/data-plane interoperability peer where practical;
- loopback HTTP/IRC/raw TCP/UDP fixtures;
- deterministic bounded teardown.

This is an acceptance harness, not a permanent hosted CI matrix. Do not introduce privileged namespace machinery or broad infrastructure unrelated to Proposal 170.

## 5. API acceptance

### Authentication/transport

Verify real TLS, authentication/token behavior, unauthorized rejection, request-ID preservation, notification semantics, JSON-RPC envelopes, and bounded malformed/oversized request handling.

### RouterInfo

Exercise all 43 selectors through real production composition. Dynamic rows must be driven or observed in meaningful state where practical. News must use the adopted signed `.i2p` source path; transit 15s must remain request-independent; network error state must correspond to explicit source observations; banned peers must retain M103's authoritative semantic proof.

### AddressBook

Exercise authenticated CRUD, subscriptions, all 13 SetConfig keys, restart persistence, invalid/path-escape failure, publication/proxy/update-delay behavior, and no-partial-mutation guarantees.

### TunnelManager lifecycle

For each of the 12 types:

1. create;
2. canonical get/rawConfig verification without secrets;
3. start;
4. prove actual family-appropriate data-plane/session effect;
5. exercise representative canonical options including security-sensitive positive/negative cases;
6. restart and verify generation/persistent identity behavior;
7. stop;
8. delete;
9. verify no stale listener/session/task/secret state.

Verify `All` start/stop/restart over a bounded mixed inventory.

### Data-plane interoperability

Required family evidence remains:

- raw client and server streaming;
- `httpclient` direct I2P and configured outproxy policy;
- `connectclient` CONNECT routing;
- `socks` SOCKS4a/SOCKS5 CONNECT;
- `socksirc` plus IRC filter;
- `ircclient` filtering;
- `httpserver` filtered inbound HTTP;
- `httpbidirserver` composed inbound/outbound behavior;
- `ircserver` filtered registration and bounded relay;
- `streamrclient`/`streamrserver` real I2P datagram subscribe/refresh/payload/unsubscribe.

A successful `start` response is not sufficient evidence.

## 6. Security reclosure

Retain M093 as prior deep authority but rerun focused regression evidence at the final option head for:

- trusted peer Destination identity;
- admission transactionality/cardinality;
- literal-loopback target confinement;
- HTTP framing/spoof/Host/fingerprint/Expect/POST/rate behavior;
- IRC registration/DCC/CTCP/half-close/inactivity behavior;
- LeaseSet encryption/auth no-downgrade;
- access/filter path confinement;
- Streamr subscriber/payload/fanout bounds;
- key/password/path redaction;
- client proxy/outproxy no-local-DNS/open-proxy behavior.

M088's accepted lower-layer residual remains accepted unless new evidence directly changes it. M104 does not reopen M091 by default.

## 7. Containment reclosure

Compare the final production head against M061/M062/M063 and the M095 budgets.

Required conclusion:

- Proposal 170 policy remains under `emissary-cli/src/i2pcontrol/**` wherever technically possible;
- every non-I2PControl production path has an exact owner/rationale;
- no broad prefix/glob was added for convenience;
- no I2PControl-only dependency leaked into default features;
- no vendored/git/forked Yosemite or unauthorized dependency substitution exists;
- no frontend coupling exists.

## 8. Authorized M104 changes

Authorized:

- focused test/interoperability harnesses;
- support/conformance documentation;
- final M095 matrix reconciliation;
- registry/roadmap/README/closure bookkeeping;
- tiny directly demonstrated integration fixes entirely inside an already-budgeted I2PControl path when they do not change architecture/security ownership.

Requires a new numbered corrective plan:

- any unresolved applicable matrix cell;
- new or changed core behavior;
- new dependency;
- persistent schema design;
- material AddressBook/TunnelManager semantics;
- security/anonymity defect beyond a tiny local integration fix;
- new tunnel data-plane mechanism.

## 9. Failure/recovery/contention acceptance

Integrated evidence must cover restart/persistence, stable configured server identities, bind/start recovery, worker/source failure preserving prior valid state, concurrent per-name lifecycle serialization, bounded server admission/rate accounting, task cancellation across stop/restart, no partial mutation on malformed input, and deterministic cleanup with no orphan listeners/processes.

## 10. Compatibility/migration

Verify pre-full-support persisted definitions/state load or migrate deterministically; AddressBook configuration migration remains recoverable; compatibility aliases do not redefine canonical closure; canonical clients see no renamed key/action/type/status; feature-disabled/default Emissary remains unchanged.

## 11. Verification commands

At minimum:

```text
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo test -p emissary-core
cargo test -p emissary-cli --no-default-features --features i2pcontrol
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m062_dependency_containment
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
git diff --check
```

Run and record the focused live interoperability environment separately.

If the historical `m063_feature_reachability` target remains absent, preserve that limitation rather than creating unrelated scope. Do not create formatter-only core churn for the existing nightly/stable rustfmt mismatch.

## 12. Final documentation state

Only if M104 closes successfully may support documentation state:

> Emissary fully supports I2P Proposal 170 against the pinned 2026-05-20 revision.

Also state that Proposal 170 remains Open and later revisions require a new delta audit. Do not claim upstream adoption/review/merge or general I2PControl parity.

## 13. Static final guards

Require:

- exact matrix exhaustiveness;
- no applicable `planned_apply`, `blocked_primitive`, `unsupported`, or unknown cell;
- exact 43 RouterInfo / 13 SetConfig / 12 types / 7 actions / 6 service selectors;
- no secret-bearing canonical get output;
- exact containment assertions;
- Proposal 170 wire policy absent from neutral core declarations;
- M103 by-design-empty proof still valid;
- request-independent transit sampler;
- no per-request news fetch.

Avoid brittle line-number/test-count assertions.

## 14. Acceptance and stop conditions

M104 closes only when every pinned applicable matrix cell has production evidence, all twelve tunnel families carry intended traffic, AddressBook/RouterInfo/ClientServicesInfo remain exact, no high/medium correctness/security/containment defect remains, default behavior is unchanged, and live interoperability is recorded.

M104 MUST stop and create a new corrective plan if any residual TunnelManager cell remains blocked, a source semantic is materially wrong, a tunnel starts but cannot carry traffic, an anonymity/path/auth/identity defect is found, unplanned lower-layer/dependency expansion appears, or the pinned proposal changed materially.

Partial completion is not full closure.

## 15. Closure evidence required

Create `plans/closure/i2pcontrol-proposal-170/104-closure.md` containing:

- exact production head and proposal revision;
- M095-M103 plus revised M098/M099 and all residual-option closure dependency table;
- final machine-readable matrix summary/hash;
- RouterInfo 43-row evidence;
- AddressBook CRUD/subscriptions/13-key evidence;
- TunnelManager 12-type/7-action/all-option evidence;
- ClientServicesInfo six-selector evidence;
- live environment topology and commands/results;
- per-family data-plane evidence;
- security regression evidence;
- persistence/restart/failure/contention evidence;
- containment/default-feature comparison;
- M104 exact changed-path review;
- verification outcomes;
- unresolved findings with severity;
- final disposition: `closed internally against pinned revision`, `corrective pass required`, or `blocked`;
- explicit base-method out-of-scope statement;
- external-sources-read-only/no-upstream-write/review/submission attestation.

## 16. Internal-only rule

All work and writes remain internal to `eggstack/emissary`. External I2P/i2pd/Java I2P/Yosemite/specification sources are read-only evidence. No upstream issue, PR, review request, submission, merge/adoption request, contribution preparation, branch/tag push, release, or maintainer contact is authorized.
