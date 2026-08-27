# M097 — Tunnel Common Session and Key Option Completion

Status: ready; dependency M095 closed

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md`.

Canonical requirements:

- `plans/000-long-term-specification.md`;
- `plans/003-planning-process.md`;
- `plans/adrs/ADR-0002-control-plane-tunnel-runtime-ownership.md`;
- `plans/adrs/ADR-0003-proposal-170-tunnel-runtime-completion-and-filter-boundary.md`;
- `plans/adrs/ADR-0004-pinned-full-proposal-170-completion-boundary.md`;
- M072/M073 option-truthfulness evidence;
- M093 current tunnel production/security reclosure authority.

Planning baseline: `630a8fd1cd4e5943fcde0b5c16f5fc1e88b5d207` plus accepted M095 matrix when dependency-ready.

Pinned external contract: I2P Proposal 170 revision `2026-05-20`.

Classification: capability / infrastructure / security.

## 1. Objective

Complete the common Proposal 170 TunnelManager options that affect I2P session/tunnel construction, destination identity, cryptographic types, persistence, and cross-family session behavior, using existing Yosemite/SAM and I2PControl-owned backend primitives wherever possible.

The current system correctly rejects recognized runtime-relevant options it cannot apply. M097 changes the final target for the common option class from `reject truthfully` to `apply where applicable`, without weakening the fail-before-allocation rule during implementation.

M097 does not implement proxy-specific, HTTP-filter-specific, server-throttle/access, or LeaseSet client-authorization behavior owned by M098/M099 except where shared session/key plumbing is a hard prerequisite.

## 2. Option class owned by M097

The exact set is frozen by M095. Expected common candidates include:

- `Shared`;
- `UseSSL` only where it represents common transport/listener/session behavior rather than a proxy-specific feature;
- `TunnelLength`;
- `TunnelVariance`;
- `TunnelQuantity`;
- `TunnelBackupQuantity`;
- `SigType`;
- `EncType`;
- `CustomOptions`;
- `NewDest`;
- `PersistentClientKey`;
- `PrivKeyFile`;
- any exact common session options M095 assigns here.

M095 may move a candidate to M098 or M099 when the contract/reference semantics are clearly family-specific. M097 must follow the matrix rather than this expected list if they differ.

## 3. Required architecture

```text
TunnelManager canonical option
       |
       v
I2PControl normalized typed option
       |
       v
backend applicability validator
       |
       v
I2PControl session/destination configuration
       |
       v
existing Yosemite/SAM/session primitive
```

Proposal 170 names, JSON values, and backend policy do not cross into `emissary-core`.

If Yosemite/SAM has no supported way to apply a required common session option, M097 must record the exact missing primitive and stop that option cell. It must not invent a Proposal-170-shaped core API or vendor/fork Yosemite under this plan.

## 4. Session/tunnel construction controls

For the tunnel length/variance/quantity/backup controls:

- establish exact inbound/outbound/reference semantics from M095;
- map values deterministically to the session options actually consumed by current Yosemite/SAM;
- validate the Proposal 170 ranges before session allocation;
- ensure an edit of a running definition follows the existing lifecycle rule: either restart is required/applied through the defined control path or the edit affects the next start; do not silently mutate only persisted config while reporting the current session as changed;
- `get` must reflect the effective persisted canonical option values without leaking internal option names.

No new router tunnel-builder algorithm is authorized. These are client/session requests into existing router mechanisms.

## 5. Signature/encryption types

`SigType` and `EncType` must map exact Proposal 170 accepted values to the signing/encryption types supported by the current destination/session machinery.

Requirements:

- use a finite allowlist derived from current cryptographic capabilities and pinned reference semantics;
- reject unsupported/unknown types before key/session allocation;
- never downgrade silently;
- preserve existing server destination identity across restart when the same persisted key/type configuration applies;
- changing an identity-affecting type must create a new destination only through explicit `NewDest`/edit semantics established by M095, not accidentally during restart;
- secrets/private key material remain outside ordinary JSON responses and logs.

## 6. Shared destinations and persistent client keys

Where `Shared` and `PersistentClientKey` apply:

- define a bounded ownership key for shared client sessions;
- sharing may occur only between definitions whose security/session identity-affecting settings are compatible;
- one tunnel definition stopping must not cancel a shared session still owned by another active definition;
- final owner release cancels the session deterministically;
- shared-session tables are bounded by the existing tunnel inventory;
- persistent client destination/key state uses the existing I2PControl-owned secret/persistence authority or a sibling backend-owned store, not arbitrary global files;
- restart restores identity where the option requires persistence.

Do not merge startup-managed clients into the control-plane shared-session authority.

## 7. New destination semantics

`NewDest` must have one unambiguous effect derived from the pinned reference behavior:

- when requested in an applicable edit/start path, retire the prior control-plane destination/key generation and create a new one at the documented lifecycle boundary;
- never rotate silently on ordinary restart if `NewDest` was not requested;
- old key material is deleted/retired according to the existing secret-store atomicity policy only after the replacement generation is safely committed;
- failures preserve a recoverable prior durable definition unless the contract explicitly commits the identity change first.

## 8. PrivKeyFile

The current canonical handler rejects `PrivKeyFile`, which is safe but incomplete for a literal option surface.

M097 may implement the option only through confined administrative semantics:

- path resolution is restricted to an I2PControl-owned key root established by the backend secret authority;
- arbitrary absolute host paths and path traversal are rejected;
- import parses and validates key material before replacing any durable identity;
- export/write behavior is implemented only if the pinned/reference semantics require it and remains within the key root;
- file permissions are restrictive;
- symlink/special-file escape is rejected where practical;
- key contents never appear in API output, logs, errors, Debug, or matrices.

If the pinned semantics require unrestricted filesystem key paths, stop and document the security conflict rather than weakening confinement silently.

## 9. CustomOptions

`CustomOptions` cannot become an unbounded escape hatch around the typed capability matrix.

M097 must:

- determine which SAM/I2CP session options the pinned Proposal 170 semantics expect to pass through;
- reject keys that would override security/identity/path/listener policy already owned by typed options;
- bound key/value lengths and entry count;
- use a deterministic allowlist or namespaced pass-through policy established by M095/reference evidence;
- apply accepted values to the actual session creation path before allocation;
- redact keys/values classified sensitive;
- ensure `get` round-trip does not expose secrets.

Unknown arbitrary key acceptance is not required merely because the container is named `CustomOptions` if doing so would bypass security invariants.

## 10. Readiness/current evidence

M097 cannot execute until M095 proves:

- exact per-type applicability for this option class;
- current Yosemite/SAM primitive availability;
- current persisted option representation and secret-store ownership;
- exact required canonical value forms;
- no new non-I2PControl path is needed for common option application.

If M095 identifies a required primitive absent from supported Yosemite/SAM, keep that matrix cell blocked and update this plan before implementation. Do not repeat the M091 pattern of crossing dependency/core boundaries while a plan is blocked.

## 11. Preferred authorized path boundary

Production changes should remain under:

- `emissary-cli/src/i2pcontrol/domain/tunnel.rs`;
- `emissary-cli/src/i2pcontrol/tunnel_manager.rs`;
- `emissary-cli/src/i2pcontrol/backends/options.rs`;
- `emissary-cli/src/i2pcontrol/backends/runtime/**` only for neutral shared-session/lifecycle helpers;
- existing real backend files under `emissary-cli/src/i2pcontrol/backends/**` that consume common session/key configuration;
- I2PControl-owned secret/persistence modules identified by M095;
- `emissary-cli/src/i2pcontrol/production.rs` only for composition of an I2PControl-owned shared/session/key authority;
- focused tests/docs/matrix updates.

No `emissary-core/**`, root dependency, `Cargo.lock`, vendored dependency, startup tunnel/proxy path, or frontend change is authorized.

## 12. Invariants

1. Every M097-owned applicable option is validated before allocation.
2. No silent ignore/downgrade.
3. No new core/router algorithm or API.
4. Yosemite remains the supported dependency source; no vendoring/forking/git override.
5. Identity/key state is backend-owned, bounded, atomic, restart-safe, and redacted.
6. Shared sessions preserve exact ownership/reference counts and cannot cancel another definition's resource.
7. Startup-managed resources remain separate.
8. Existing HTTP/IRC/server admission/filter invariants remain unchanged.
9. Default/feature-disabled builds remain unaffected.
10. No upstream interaction occurs.

## 13. Explicit non-goals

M097 MUST NOT:

- implement proxy lists/auth/outproxy policy owned by M098;
- implement HTTP privacy header policy owned by M098;
- implement server access/throttle/LeaseSet auth owned by M099;
- change RouterInfo/AddressBook sources;
- add new tunnel types/actions/fields;
- redesign tunnel data planes;
- add lower-layer transport/session APIs in `emissary-core`;
- vendor/patch Yosemite;
- add CI/fuzz/release machinery;
- contact upstream.

## 14. Ordered work packages

### A. Freeze M097 matrix subset

Extract every M097-assigned option/type cell from the M095 matrix and make it a focused test fixture. No implementation starts while any owned cell is `unknown`.

### B. Normalize canonical values

Extend typed tunnel options only where current representation cannot preserve exact semantics. Keep compatibility aliases at the handler edge.

### C. Implement common session-option translation

Build one deterministic conversion from validated typed common options to existing Yosemite/SAM session settings. Reuse it across backends instead of duplicating string maps.

### D. Implement key/destination lifecycle

Add persistent/shared/new-destination/PrivKeyFile confined behavior as assigned by M095.

### E. Integrate real backends

Each affected backend consumes the common translator and proves actual session creation uses the requested settings. Do not mark a matrix cell `apply` from unit conversion alone.

### F. Reconcile get/edit/restart

Verify persisted canonical values, restart identity expectations, running-edit behavior, and failure recovery.

## 15. Failure, cancellation, restart, and contention semantics

- Unsupported/invalid value: deterministic failure before listener/session/key allocation.
- Key import/rotation pre-commit failure: preserve prior key generation.
- Shared session creation: one creator generation; concurrent compatible owners join only after successful publication.
- Shared session failure: all current owners observe failed/stopped runtime consistently; no stale owner leaks.
- Stop: release exact owner; cancel underlying session only at zero owners.
- Restart: does not rotate persistent identity unless explicit semantics require it.
- Edit requiring session reconstruction: use existing per-name generation/lifecycle serialization; do not hold locks across network I/O or joins.

## 16. Compatibility and migration

Prefer current persisted raw/canonical option representation. If new secret/shared-state metadata is required, use an additive versioned schema with deterministic migration.

Existing definitions that omit new options retain current behavior/defaults. Definitions containing previously persisted-but-rejected options become startable only when M097 has implemented the exact applicable semantics.

## 17. Tests

At minimum:

- per-type applicability/invalid option fixtures;
- actual session option translation fixtures;
- tunnel length/variance/quantity/backup range and apply tests;
- SigType/EncType exact mapping and no-downgrade tests;
- persistent identity restart stability;
- NewDest intentional rotation and ordinary restart no-rotation;
- shared-session ownership/refcount/concurrency/stop tests;
- PrivKeyFile traversal/symlink/special-file/permission/redaction tests;
- CustomOptions bounds/allowlist/no-override tests;
- start failure before resource allocation;
- all twelve backends retain real registration;
- existing server/filter security regressions stay green.

## 18. Verification

Run focused option/backend tests plus:

```text
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo test -p emissary-cli --no-default-features --features i2pcontrol
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m062_dependency_containment
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m063_feature_reachability
git diff --check
```

No broad core suite is required because M097 is not authorized to modify core.

## 19. Documentation/static guards

Update the M095 matrix only for cells proven operational. Keep any blocked primitive explicit.

Update support/tunnel-manager docs with supported common option semantics, key/path confinement, and any intentionally `not_applicable` cells. Overall Proposal 170 remains partial until M104.

## 20. Acceptance and stop conditions

M097 closes only if:

- every M097-owned applicable matrix cell is `apply` with runtime evidence;
- no recognized behaviorally meaningful option is silently ignored;
- persistent/shared/key lifecycle is bounded and restart-safe;
- no core/dependency/vendor boundary changed;
- existing tunnel security invariants remain green;
- no high/medium finding remains;
- no upstream interaction occurred.

Stop if a required option needs an unavailable Yosemite/SAM primitive, a new core API, unrestricted filesystem key access, or a dependency provenance change. Record the exact blocker and leave the cell blocked.

## 21. Closure evidence required

Create `plans/closure/i2pcontrol-proposal-170/097-closure.md` with:

- M095 dependency/matrix subset;
- exact changed paths;
- option/type requirement-to-runtime evidence matrix;
- Yosemite/SAM primitive evidence;
- key/shared/NewDest/PrivKeyFile security and restart evidence;
- failure/contention/cancellation results;
- containment guard outcomes;
- updated M095 matrix totals;
- unresolved blocked primitives/findings;
- internal-only/no-upstream attestation.

## 22. Internal-only rule

All writes remain within `eggstack/emissary`; external specs/reference/dependency repos are read-only evidence only. No upstream review/submission/merge/contribution activity is authorized.
