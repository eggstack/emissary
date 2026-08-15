# M072 — Proposal 170 Tunnel Runtime Completion Reclosure

Status: corrective pass required — M072 reclosure recorded; M073 owns the generic option finding

Planning production baseline: `a1296b018ce98d26a019bd5064dff9f4b47e0ad6`

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-tunnel-runtime-completion-roadmap.md`.

Authority:

- `plans/000-long-term-specification.md`;
- `plans/002-long-term-roadmap.md`;
- `plans/003-planning-process.md`;
- `plans/adrs/ADR-0003-proposal-170-tunnel-runtime-completion-and-filter-boundary.md`.

Hard dependencies:

- M066 IRC client/server closed;
- M067 HTTP server closed;
- M068 HTTP client/CONNECT closed;
- M069 SOCKS/SOCKS-IRC closed;
- M070 HTTP bidirectional server closed;
- M071 Streamr client/server closed.

M064/M065 must also remain accepted predecessors.

## 1. Objective

Perform an independent integrated reclosure of the newly authorized Proposal 170 tunnel-runtime completion phase.

M072 is primarily a closure/invariant milestone, not a place to invent missing family behavior. It must verify that all twelve Proposal 170 tunnel types are truthfully operational within their documented option sets, that HTTP/IRC filtering cannot be bypassed, that lifecycle/persistence/containment remain coherent, and that the broader fork has not been contaminated unnecessarily.

If M072 discovers a material family defect, it must create a new corrective implementation plan rather than absorb broad production work into reclosure.

## 2. Classification

Primary class: invariant / integration closure.

Small direct fixes are permitted only if all of the following are true:

- defect is unambiguous and low-risk;
- fix is confined to an already accepted M066-M071 I2PControl path;
- no architecture, option semantics, public behavior, or containment boundary changes;
- focused regression test can be added in the same change.

Anything larger requires M073+ corrective planning.

## 3. Integrated target state

Expected production registry at M072 entry:

- `client` — real;
- `httpclient` — real;
- `ircclient` — real;
- `socks` — real;
- `socksirc` — real;
- `connectclient` — real;
- `streamrclient` — real;
- `server` — real;
- `httpserver` — real;
- `httpbidirserver` — real;
- `ircserver` — real;
- `streamrserver` — real.

The default/test registry may remain intentionally unsupported/fake if that is part of its test contract; production composition must be exhaustive and real.

Every backend must still be able to reject an unsupported relevant option truthfully without misreporting the type itself as unimplemented.

## 4. Hard invariants to re-audit

### Protocol and domain

- exactly twelve pinned tunnel types;
- exact Proposal 170 action/type/field spellings;
- no new public capability/status metadata;
- create/edit/get/delete persistence semantics unchanged;
- existing compatibility aliases not accidentally expanded;
- unsupported option errors use existing operation-status channel.

### Ownership/lifecycle

- control-plane resources are separate from startup-managed resources;
- per-name lifecycle operations serialize deterministically;
- stop/restart/delete cannot target startup-owned tasks;
- stale generation completions cannot overwrite current state;
- destination secrets remain backend-owned/path-confined;
- server identities survive stop/restart/definition rename semantics as previously specified;
- no orphan task/listener/session remains after stop/delete/failure.

### HTTP security

- `httpserver` and `httpbidirserver` inbound paths cannot route around request sanitizer via generic server forward;
- spoofed I2P/proxy identity headers cannot reach backend as trusted input;
- request framing ambiguity fails before local service;
- response fingerprint/proxy headers are filtered;
- local target cannot be selected by remote Host/request data;
- relevant throttles/access controls operate with bounded trusted peer state;
- `httpclient`/bidirectional client path preserve anonymity header policy;
- clearnet requires configured I2P outproxy;
- CONNECT is strict and direct success occurs only after remote connection.

### IRC security

- `ircclient` and `socksirc` use the same common IRC filter;
- DCC remains blocked unless a later accepted plan explicitly changed it;
- unsupported CTCP remains blocked while ACTION/normal traffic works;
- USER/PING/PART/QUIT anonymity rewrites remain active;
- `ircserver` filters registration before local IRCd receives it;
- peer hostname comes from trusted I2P identity;
- registration/cross-protocol bounds remain enforced;
- WEBIRC remains explicitly unsupported unless separately closed.

### SOCKS/CONNECT proxy safety

- no local DNS for direct I2P;
- literal/local/private/LAN direct target restrictions hold;
- SOCKS BIND/UDP remain correctly unsupported unless separately accepted;
- auth/listener exposure policy cannot produce accidental open proxy;
- `socksirc` cannot switch to raw relay.

### Streamr resource safety

- subscriber count hard bound;
- expiry/refresh/unsubscribe semantics coherent;
- payload/task queue bound;
- no packet-driven local target redirection;
- one failing subscriber does not block/fail all;
- restart clears ephemeral subscriber state but preserves server identity.

### Containment/default behavior

- no new `emissary-core/**` production changes from M065-M071;
- M064 remains the only planned core correction in this workstream;
- non-I2PControl production path delta is individually justified and minimal;
- M061 source-containment authority updated only if an accepted exceptional path required it; otherwise unchanged;
- M062/M063 dependency ownership still passes;
- default/no-I2PControl builds do not activate specialized runtimes/dependencies/tasks;
- no hosted CI/release expansion introduced.

### Source truthfulness

- RouterInfo remains accepted 37/1/5 unless separately planned;
- M051 blocker not conflated with tunnel runtime completion;
- AddressBook SetConfig/base-method limitations accurately remain limitations if still present;
- ClientServicesInfo does not overclaim services beyond actual composed listeners/sessions.

## 5. Option-capability reclosure

M072 must produce a single integrated internal matrix covering each type and every Proposal 170 option category relevant to it.

For each option/type pair classify:

- implemented/applied;
- invalid/irrelevant and rejected;
- recognized but intentionally unsupported and rejected;
- not applicable by protocol.

The matrix must specifically flag security-sensitive options and prove none are silently accepted/ignored.

At minimum review categories:

- listen interface/port;
- target destination/host/port;
- access lists;
- proxy/outproxy credentials;
- User-Agent/Referer/Accept controls;
- server filtering/throttling controls;
- IRC fields and WEBIRC/DCC custom options;
- SOCKS command/outproxy options;
- Streamr target/control fields;
- tunnel length/variance/quantity;
- signature/encryption/LeaseSet/session options;
- custom options.

If any backend currently stores a relevant option but does not apply/reject it at start, M072 cannot close.

## 6. Integrated lifecycle scenarios

Run table-driven/focused integration tests for each type:

- create -> start -> get/status -> stop -> start -> stop -> delete;
- StartOnLoad/restart recovery where already supported by TunnelManager design;
- edit while stopped then start changed definition;
- invalid edit/start does not corrupt durable definition;
- duplicate start;
- concurrent stop/restart race;
- bind collision;
- SAM unavailable;
- local target unavailable;
- task panic/failure injection where test hooks exist;
- server secret identity preserved;
- delete cannot orphan live resource.

`All` stop/start/restart behavior must be reviewed across a heterogeneous set including specialized types.

Do not invent new public status fields for detailed backend state.

## 7. Cross-family security tests

### HTTP

Use a capture backend and direct backend invocation/fake SAM to ensure all server-type HTTP entry paths sanitize the same dangerous fixture corpus.

The bidirectional type must pass the same inbound corpus as `httpserver` and the same outbound corpus as `httpclient`.

### IRC

Run the same leak corpus through `ircclient` and `socksirc`, plus registration corpus through `ircserver`.

### Proxy routing

Run a common forbidden-target corpus against HTTP client, CONNECT, and SOCKS to ensure no backend found a different route to local/private targets or OS DNS.

### Secrets/logging

Inject recognizable sentinel passwords/private-key-like strings/Authorization headers into failures and assert they do not appear in returned status/log capture/debug output.

## 8. Containment review

Compare the final runtime-completion head against the M064 production baseline and classify every changed production path:

- I2PControl-owned new/modified path;
- M064 neutral core correction;
- accepted exceptional non-I2PControl seam with owning closure evidence;
- unauthorized path.

Unauthorized paths block closure.

Review direct dependencies added during M065-M071:

- necessity;
- feature ownership;
- default-features policy;
- lockfile impact;
- transitive activation from `default`, `ui`, `metrics`;
- alternatives/removal opportunity if a dependency ended unused.

Do not turn M072 into a general dependency slimming campaign; remove only workstream-introduced unnecessary dependency edges.

## 9. Documentation/support reconciliation

Update/review:

- `docs/i2pcontrol/proposal-170-support.md`;
- `plans/registry.md`;
- implementation README;
- tunnel runtime roadmap status;
- any support matrix used by conformance tests.

Documentation must state:

- all real tunnel types;
- exact rejected subfeatures/options (e.g. DCC, WEBIRC, SOCKS BIND/UDP if still unsupported);
- server filtering guarantees at a high level;
- outproxy/DNS/LAN safety policy;
- Streamr bounds/control semantics;
- any environmental qualification for live public-network proof;
- unrelated remaining Proposal 170 limitations such as RouterInfo unavailable rows/AddressBook gaps.

Tunnel runtime completion must not be phrased as full historical I2PControl method completeness if base methods remain out of scope.

## 10. Ordered work packages

### WP1 — registry/type/option inventory

Freeze final production registry and generate/review option capability matrix.

### WP2 — lifecycle integration matrix

Run per-type and heterogeneous `All` lifecycle/persistence/restart tests.

### WP3 — security corpus reclosure

Run common HTTP/IRC/proxy/secret/resource fixture sets across all relevant families.

### WP4 — containment/default/dependency audit

Review changed paths and feature graph against M061-M063 authority.

### WP5 — documentation/truthfulness reconciliation

Ensure runtime-complete claims do not overwrite unrelated partial Proposal 170 source/method limitations.

### WP6 — closure record

Create `plans/closure/i2pcontrol-proposal-170/072-closure.md` with requirement-to-evidence matrix and unresolved findings.

Material findings create corrective plans and block M072.

## 11. Verification commands

Use existing/focused local commands, likely including:

```text
cargo test -p emissary-cli --no-default-features --features i2pcontrol
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m062_dependency_containment
cargo check -p emissary-cli --no-default-features
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo check -p emissary-core --no-default-features
cargo check -p emissary-core
git diff --check
```

Run focused live child-process I2PControl test if it remains part of existing verification. Add bounded local/fake-SAM traffic coverage for all twelve types as needed.

Do not add a new remote CI matrix, fuzz service, soak suite, or release gate.

## 12. Acceptance criteria

M072 may close only when all are true:

1. M066-M071 closure records are accepted;
2. production registry has exactly twelve real Proposal 170 backends;
3. no declared type remains mapped to `UnsupportedTunnelBackend` in production composition;
4. persisted historical unsupported definitions remain loadable without schema migration;
5. every backend validates runtime-relevant options before allocation;
6. no security-sensitive relevant option is silently ignored;
7. per-type lifecycle create/start/stop/restart/delete is operational and truthful;
8. heterogeneous `All` lifecycle behavior is deterministic;
9. startup-managed resources remain externally owned;
10. no stale-generation/orphan-task/listener/session defect remains;
11. server destination secrets remain path-confined/redacted and restart-stable;
12. HTTP server/bidir inbound sanitizer cannot be bypassed;
13. HTTP request-framing/identity/proxy-header negative corpus passes;
14. HTTP response fingerprint/proxy filtering corpus passes;
15. HTTP client/CONNECT no-local-DNS/outproxy/LAN target corpus passes;
16. IRC client/SOCKS-IRC common leak/DCC/CTCP corpus passes;
17. IRC server registration identity/cross-protocol/timeout corpus passes;
18. SOCKS command/address/auth/open-proxy corpus passes;
19. Streamr subscription/payload/amplification/resource corpus passes;
20. secret-sentinel log/error corpus shows no credential/private material leakage;
21. feature-disabled/default builds do not activate specialized runtime work;
22. M061 source containment passes or any explicitly amended authority is itself closed/accepted;
23. M062/M063 dependency containment passes;
24. M064 remains the only planned core production correction in this runtime series unless an exceptional separately planned corrective was accepted;
25. no unexplained non-I2PControl production path remains;
26. RouterInfo support remains truthfully 37/1/5 unless separately changed by accepted plan;
27. AddressBook/base-method/ClientServicesInfo docs remain truthful and are not overclaimed by tunnel completion;
28. support documentation accurately lists all real types and remaining subfeature limitations;
29. no full-Proposal/full-I2PControl claim exceeds evidence;
30. no CI/release/fuzz/coverage/platform machinery was expanded solely for this workstream;
31. no upstream/third-party issue, PR, review, merge request, maintainer contact, branch/tag/release, contribution package, or submission was created/prepared;
32. no high- or medium-severity correctness/security/containment finding remains open;
33. closure record names any low-severity/deferred items precisely and explains why they do not invalidate runtime support;
34. registry/roadmap/implementation README agree on final lifecycle status.

## 13. Closure evidence required

`072-closure.md` must include:

- all implementation/closure commit references M064-M071;
- exact final production registry matrix;
- integrated option-capability matrix or durable pointer to machine-readable evidence;
- per-type lifecycle evidence;
- heterogeneous All-action evidence;
- HTTP security corpus outcomes;
- IRC security corpus outcomes;
- SOCKS/CONNECT routing/auth outcomes;
- Streamr resource/control outcomes;
- secret redaction evidence;
- final changed-path containment ledger from M064 baseline;
- direct dependency/feature audit;
- default/feature-disabled results;
- documentation/support-claim review;
- unresolved findings with severity;
- explicit external read-only/internal-write-only attestation;
- final disposition.

## 14. Stop conditions

M072 must stop and create a corrective plan if:

- any production type is still stubbed;
- any real backend silently ignores a relevant security option;
- any HTTP/IRC filtering bypass exists;
- any proxy family performs unintended OS DNS/local/LAN direct routing;
- Streamr state/task growth is unbounded;
- any new core/runtime path lacks accepted planning authority;
- a material dependency/default-feature contamination exists;
- any high/medium finding remains;
- documentation cannot truthfully support the intended final runtime-completion statement.
