# M103 — RouterInfo Banned-Peer Semantic Completion

Status: closed; dependency M095 closed; closure: `plans/closure/i2pcontrol-proposal-170/103-closure.md`

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md`.

Canonical requirements:

- `plans/000-long-term-specification.md`;
- `plans/003-planning-process.md`;
- ADR-0004 full-support/minimal-core boundary;
- M051 news/banned-peer source audit;
- M056 current RouterInfo 37/1/5 reclosure;
- M061 source-containment authority.

Planning baseline: `630a8fd1cd4e5943fcde0b5c16f5fc1e88b5d207` plus accepted M095 source audit when dependency-ready.

Pinned external contract: `i2p.router.netdb.bannedpeers`, Proposal 170 revision `2026-05-20`, return type `Map<String, Map<String, Object>>`.

Classification: capability / invariant / containment.

## 1. Objective

Give the Proposal 170 banned-peer selector a truthful authoritative semantic owner without introducing a new router-wide peer-ban algorithm solely to make a read-only telemetry row appear implemented.

M051 correctly left this row unavailable because no substantive canonical ban owner was identified. ADR-0004 authorizes a fresh exhaustive audit and one of two bounded completion paths:

1. expose an existing enforceable peer-ban/exclusion state through a neutral bounded inspection seam; or
2. if Emissary provably has no state in which a peer can be banned by design, codify an authoritative by-design empty result rather than an unowned fallback.

If neither path is truthful, M103 must stop and full Proposal 170 support remains blocked pending a separate maintainer architecture decision. This plan does not authorize inventing router ban behavior.

## 2. Hard semantic gate

M095 must answer all of the following before M103 becomes executable:

- Does any Emissary subsystem enforce a peer-specific exclusion/ban with duration or reason?
- Are there temporary transport/session/profile exclusions that are semantically equivalent to the proposal/i2pd banned-peer concept, or merely local connection/backoff state?
- Can a peer hash be in a globally meaningful `banned` state independent of I2PControl?
- What exact map key/value member shape is expected by the pinned proposal/reference implementation?
- What details are stable semantic requirements versus implementation-specific extra fields?
- If no ban state exists, can the repository prove that the set of banned peers is structurally empty by design rather than merely unobserved?

Do not equate HTTP/server tunnel `TotalBanTime`, rate-limit denial tables, transport retry backoff, or disconnected peers with router NetDB banned peers unless M095 proves semantic identity. These are separate ownership domains by default.

## 3. Completion path A — existing real ban/exclusion owner

If M095 identifies an existing canonical enforceable peer-ban state:

- expose only a bounded immutable snapshot containing the peer hash and exact stable details required by Proposal 170;
- use the owner that actually enforces the exclusion;
- do not duplicate the state in I2PControl;
- do not expose mutable handles, profile objects, sockets, tasks, or internal command channels;
- sort deterministically before serialization;
- bound entry count/serialized size using existing RouterInfo response limits;
- I2PControl owns JSON field names and reference-specific presentation.

Any lower-layer path must already be in M061 or receive an explicit exact containment amendment before code.

## 4. Completion path B — authoritative by-design empty state

If the exhaustive audit proves Emissary has no peer-ban facility/state at all, M103 may make `bannedpeers` available as an empty map only if all of the following are established:

1. no production writer/path can put a peer into a router-wide banned state;
2. transport retry/backoff/profile status is explicitly not the proposal's ban concept;
3. server-tunnel temporary denial tables are explicitly not router-wide bans;
4. the returned empty map represents the router's actual semantic set, not missing telemetry;
5. static/runtime tests make the proof durable enough that adding a future ban facility will fail or require updating this source disposition.

Preferred implementation in this path is entirely under `emissary-cli/src/i2pcontrol/**`: a source classification that says the canonical router capability has no banned entries by design, with a static guard tied to an explicit repository capability marker/inventory. Do not add a dummy core ban list with no writers merely to create an owner object.

A naked `serde_json::json!({})` fallback with no capability proof is not acceptable.

## 5. Completion path C — blocked architecture

If the pinned semantics require an actual ban engine and Emissary lacks one, or if existing exclusion states are too ambiguous to claim either path A or B, stop M103.

Record:

- exact missing router capability;
- why an empty set would be semantically false/ambiguous;
- why existing backoff/denial state is not equivalent;
- what substantive router behavior would have to be added.

Do not implement that behavior under M103. A new ADR/direct maintainer instruction would be required because it changes routing/security behavior rather than exposing existing state.

## 6. Preferred authorized path boundary

Path B preferred target:

- `emissary-cli/src/i2pcontrol/router_info.rs`;
- `emissary-cli/src/i2pcontrol/router_info_handler.rs`;
- source inventory/observability modules under `emissary-cli/src/i2pcontrol/**`;
- static/focused tests/docs/M095 matrix updates.

Path A, only if M095 proves an existing owner:

- the exact already-accepted M061 owner/inspection path(s), individually named in a dependency-ready revision of this plan;
- the I2PControl consumer paths above.

No broad `emissary-core/**` glob, new ban subsystem, dependency, workflow, frontend, or unrelated runtime path is authorized.

## 7. Invariants

1. `bannedpeers` represents actual router-wide banned-peer semantics only.
2. Tunnel-local temporary denial/rate limiting does not leak into router ban output by accident.
3. Empty output is permitted only with by-design-empty proof.
4. No new router ban/enforcement algorithm under this plan.
5. I2PControl owns wire/map field names.
6. Core changes, if any, are passive bounded inspection only.
7. Peer hashes/details are deterministic and bounded.
8. No secret/private peer/profile material is exposed.
9. API reads do not mutate ban/exclusion state.
10. No upstream interaction occurs.

## 8. Explicit non-goals

M103 MUST NOT:

- add peer ban/unban commands;
- add peer scoring/reputation algorithms;
- convert ordinary connection failures/backoff into bans;
- feed tunnel-server `TotalBanTime` into router NetDB state;
- change peer selection, NetDB, transport, tunnel building, or profile behavior;
- add a dummy unowned core ban table;
- alter other RouterInfo rows;
- add CI/fuzz/release machinery;
- contact upstream.

## 9. Ordered work packages

### A. Perform exhaustive semantic/source audit

Inspect current core/router/transport/profile/NetDB code and historical M051 evidence. Record all peer-specific rejection/backoff/denial concepts and why each is or is not the pinned banned-peer concept.

### B. Freeze returned map schema

Use the pinned proposal/reference implementation to establish required stable details. Avoid copying unstable implementation-specific diagnostic fields into the canonical contract unless explicitly adopted.

### C. Select one completion path

Choose A or B only from evidence. If neither is defensible, select C and stop.

### D1. Path A implementation

Add the smallest neutral bounded snapshot from the real owner, then map it in I2PControl.

### D2. Path B implementation

Add an explicit by-design-empty source classification plus durable static/runtime proof; serialize the exact empty map only through that authoritative classification.

### E. Regression guards

Ensure future introduction of a substantive ban owner cannot leave the by-design-empty classification silently stale. A guard may search an explicit capability registration or require the source inventory to be updated when an owner is added; avoid brittle string scans over arbitrary source prose.

### F. Update matrix/support docs

Mark the row available only if A or B closes. If C, leave it unavailable/blocked and M104 cannot claim full support.

## 10. Failure/restart/contention semantics

Path A:

- snapshot failure follows existing truthful RouterInfo error/unavailable behavior;
- no persistence added unless the existing ban owner already persists state;
- restart behavior follows the owner;
- bounded snapshot copied without locks crossing JSON serialization/await.

Path B:

- no runtime task/state/persistence;
- restart remains trivially empty because the router capability has no banned state;
- static capability proof is the authority, not request history.

## 11. Compatibility/migration

No public schema change. If path B is selected, clients receive an empty map that is explicitly documented as the actual Emissary router banned-peer set, not a generic `unsupported` fallback.

If a future Emissary release gains real router ban behavior, this source must migrate to path A before the by-design-empty proof/guard can pass.

## 12. Tests

Common:

- exact Proposal 170 return type/map shape;
- deterministic sorting/size bounds where non-empty;
- unknown/unowned state cannot silently serialize empty;
- tunnel-local rate denial does not appear in bannedpeers;
- transport retry/backoff does not appear unless M095 proves exact semantic equivalence;
- API read has no mutation effect;
- feature-off/default behavior unchanged.

Path A additional:

- owner insertion/removal/expiry -> snapshot correctness;
- bounded large owner state;
- restart according to owner semantics;
- no mutable/private state leak.

Path B additional:

- exhaustive capability marker/inventory asserts no router ban owner exists;
- introduction of a fixture/mock real ban owner invalidates the by-design-empty disposition;
- response is `{}` only after source capability is explicitly valid.

## 13. Verification

Path B preferred verification:

```text
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo test -p emissary-cli --no-default-features --features i2pcontrol
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m062_dependency_containment
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m063_feature_reachability
git diff --check
```

If path A changes core, also run `cargo test -p emissary-core` and exact containment/static guards for those paths.

## 14. Documentation/static guards

Update M095 with the chosen semantic path and evidence. Update source/truthfulness/support docs without rewriting M051's historical correctness.

If path B is used, documentation must say Emissary has no router-wide ban facility at this revision and therefore the authoritative banned set is empty. It must not imply support for ban management.

## 15. Acceptance and stop conditions

M103 closes only if one of these is true:

- Path A: a real canonical ban owner is exposed truthfully and minimally; or
- Path B: by-design empty semantics are proven and guarded.

M103 does not close under path C. Full-support reclosure M104 remains blocked.

Regardless of path, no new router ban algorithm, peer-selection behavior, or broad core change may land and no upstream interaction may occur.

## 16. Closure evidence required

Create `plans/closure/i2pcontrol-proposal-170/103-closure.md` containing:

- M095 semantic/source audit;
- selected path A/B/C and rationale;
- exact map-schema evidence;
- changed paths;
- source/capability owner proof;
- separation from transport backoff and tunnel-local denial state;
- bounds/restart/failure tests as applicable;
- containment results;
- updated RouterInfo matrix disposition;
- unresolved findings/blocker if path C;
- internal-only/no-upstream attestation.

## 17. Internal-only rule

All writes remain internal to `eggstack/emissary`. External I2P/i2pd/reference repositories are read-only evidence. No upstream issue/PR/review/submission/merge/contribution activity is authorized.
