# M122 — Corrected Yosemite LeaseSet Pin Adoption

Status: **proposed / blocked on Yosemite Y004 closure (M121 gate satisfied by `plans/closure/i2pcontrol-proposal-170/121-closure.md`)**

Class: dependency integration / corrective containment

Planning baseline: `feafc6a1d9650887015a01f87bf21b57a4e92085`

Consumes:

- `eggstack/yosemite` Y004 `plans/implementation/004-y003-leaseset-wire-semantics-corrective.md` once closed;
- ADR-0005 exact I2PControl-only fork dependency boundary.

Corrects the dependency state left after Y003 was implemented with defective LeaseSet wire semantics while Emissary correctly remained pinned to Y002.

Source roadmaps:

- `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md`;
- `plans/subsystems/i2pcontrol-proposal-170-post-m114-corrective-roadmap.md`.

## 1. Objective

Advance the optional `yosemite-i2pcontrol` dependency from the currently accepted Y002 implementation revision `8026f5b424fc178d683e63555335f8b33e0aba04` to the exact reviewed Y004 implementation commit, while preserving the ADR-0005 feature/containment boundary and proving the corrected generic LeaseSet API is reachable from I2PControl without promoting any Proposal 170 cell.

M122 is dependency/adaptation plumbing only. It does not implement encrypted/authenticated LeaseSet behavior in Emissary and does not reopen M113 by itself.

## 2. Hard readiness gates

M122 MUST NOT be promoted until all are true:

1. Yosemite Y004 is closed with an exact implementation commit SHA explicitly suitable for consumer pinning;
2. Y004 closure has no open high/medium protocol/security finding in its LeaseSet wire surface;
3. the exact Y004 diff from Y002/Y003 baseline has been reviewed and contains no Emissary/Proposal-specific production code, dependency/release work, or unrelated runtime expansion;
4. M121 is closed so the current Emissary matrix/semantic baseline is known before dependency adoption;
5. ADR-0005 remains accepted and unchanged with respect to optional exact-revision aliasing.

If Y004 changes public types in a way that requires a broader Emissary API redesign, stop and revise M122 rather than hiding the expansion in adapter code.

## 3. Current state

Current `emissary-cli/Cargo.toml` intentionally contains two Yosemite dependencies:

- ordinary workspace `yosemite = 0.7.0` for non-I2PControl code;
- optional `yosemite-i2pcontrol` git alias at exact Y002 revision, activated only by feature `i2pcontrol`.

This isolation successfully prevented defective Y003 from entering Emissary. M122 must preserve that property exactly.

## 4. Required production/dependency changes

Expected paths:

- `emissary-cli/Cargo.toml` — change only the exact `rev` of `yosemite-i2pcontrol`;
- `Cargo.lock` — exact source/revision update;
- `emissary-cli/src/i2pcontrol/**` only where needed to compile against Y004's corrected typed LeaseSet/client-auth API or to add bounded adapter reachability tests;
- `emissary-cli/tests/m062_dependency_containment.rs`;
- M062 dependency metadata and planning/closure docs.

No root `[patch.crates-io]`, workspace Yosemite replacement, path dependency, vendoring, floating branch/tag, or ordinary non-I2PControl Yosemite import migration is permitted.

No `emissary-core/**`, `emissary-util/**`, startup manager, frontend, workflow, or release production change is authorized.

## 5. Adapter boundary

I2PControl may import corrected Y004 public types through the existing Rust crate alias `yosemite_i2pcontrol` only below `emissary-cli/src/i2pcontrol/**`.

M122 must prove reachability of the corrected generic API without yet claiming runtime LeaseSet support. Suitable tests include constructing validated Y004 session options and observing byte-for-byte `SESSION CREATE` output at a local fake SAM endpoint for:

- canonical private/signing LeaseSet property names;
- one DH client-auth entry;
- one PSK client-auth entry;
- representative corrected type-domain values.

These are dependency reachability tests, not M095 `apply` evidence.

Do not add Proposal `EncryptLeaseSet`/`LeaseSetClientAuths` mapping unless a later registered M113-successor plan owns the semantics and router effect.

## 6. Invariants

M122 MUST preserve:

- default/non-I2PControl dependency graph continues using registry Yosemite only;
- enabling `i2pcontrol` adds the exact internal fork package only through the optional alias;
- no global patch/vendor/path/floating dependency;
- exact fork revision in manifest and lockfile;
- all fork imports under I2PControl paths;
- Y001/Y002 session-wire and destination-generation behavior remains available;
- no raw SAM serializer in Emissary;
- no Proposal matrix promotion from dependency reachability alone;
- no secret values in adapter test failures/logs/debug;
- no upstream activity.

## 7. Explicit non-goals

M122 does not:

- implement router-side encrypted/blinded/authenticated LeaseSet behavior;
- add LeaseSet secret persistence;
- decide `OptionalLookup` semantics;
- implement M113 presentation options;
- change `UseSSL`, proxy, reduction, or lifecycle semantics;
- change core tunnel pools;
- perform final interoperability certification.

## 8. Work packages

### WP1 — review Y004 exact revision

Compare the Y004 implementation commit against Y002/Y003 and closure evidence. Record changed production files and verify no unrelated scope.

### WP2 — exact dependency pin

Update only the alias revision and lock source. Preserve ordinary Yosemite declarations.

### WP3 — compile-time/API adaptation

Make the minimum I2PControl-only source adjustments required by corrected public Y004 types. If no production consumer should exist until the M113 successor, keep adaptation in focused tests/helpers and do not introduce dead Proposal mapping.

### WP4 — fake-SAM corrected-wire reachability

Exercise actual Yosemite session creation from Emissary tests and assert canonical Y004 keys/values. Ensure tests do not contain real secrets and failure messages remain redacted.

### WP5 — dependency containment

Update M062 manifest/tree/import/static tests to prove enabled/disabled reachability and exact revision. Verify no fork URL in feature-disabled normal tree.

### WP6 — closure/readiness audit

Write M122 closure and decide whether a new M113-successor LeaseSet plan can be made ready. Do not auto-promote it unless the required **router-side** primitive is also accepted/planned.

## 9. Failure and rollback

A dependency/API build failure produces no runtime migration. There is no persisted-data migration in M122.

If Y004 is discovered to contain a new medium/high defect during integration, keep Emissary pinned to Y002, mark M122 blocked/corrective, and create a new Yosemite corrective rather than pinning a known-bad revision.

## 10. Focused tests

Required:

- feature-disabled dependency tree contains registry Yosemite and not `eggstack/yosemite`;
- feature-enabled tree contains registry Yosemite plus exactly the pinned Y004 fork revision;
- all `yosemite_i2pcontrol` imports remain below I2PControl paths;
- no `[patch]`, `vendor`, path or floating Yosemite source exists;
- fake SAM observes corrected `i2cp.leaseSetPrivateKey` / `i2cp.leaseSetSigningPrivateKey` spellings when exercised through Y004;
- fake SAM observes canonical DH/PSK client-auth prefixes and deterministic numbering;
- malformed corrected API values reject before wire;
- Y001 variance/backup/custom and Y002 signature-generation adapter regressions remain green;
- M095 counts unchanged by M122.

## 11. Broad verification

```text
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo check -p emissary-cli --no-default-features
cargo check
cargo test -p emissary-cli --no-default-features --features i2pcontrol --lib --no-fail-fast
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment --test m062_dependency_containment --test m095_full_support_matrix --test m105_residual_option_audit --no-fail-fast
cargo tree -p emissary-cli --no-default-features --edges normal
cargo tree -p emissary-cli --no-default-features --features i2pcontrol --edges normal
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
cargo fmt --all -- --check
git diff --check
```

## 12. Matrix/documentation rules

M122 is infrastructure only. It MUST NOT promote M113 LeaseSet cells. Update dependency metadata, registry, corrective roadmap, M122 closure, and docs that identify the exact internal Yosemite revision.

Any M095 change in this pass is a stop condition unless it is solely reconciling an already-decided M121 disposition that landed before the M122 baseline.

## 13. Acceptance criteria

M122 closes only when:

1. Emissary pins the exact reviewed Y004 implementation revision through the existing optional alias;
2. ordinary/default Yosemite provenance remains unchanged;
3. corrected Y004 LeaseSet API/wire is demonstrably reachable from an I2PControl-only test path;
4. no Proposal LeaseSet cell is promoted without router runtime evidence;
5. dependency/import/static containment is green;
6. broad verification is recorded;
7. closure identifies the next LeaseSet/router primitive gate precisely.

## 14. Stop conditions

Stop if:

- Y004 closure is not clean;
- adoption requires a workspace/global dependency replacement;
- non-I2PControl code must import the fork;
- a raw SAM serializer is proposed;
- router-side LeaseSet implementation begins inside this dependency plan;
- Y004 API cannot be adapted without materially broader scope.

## 15. External-interaction boundary

Read-only inspection of external/upstream sources is allowed. Writes are internal to `eggstack/emissary` and the separately plan-authorized `eggstack/yosemite` fork only. M122 itself writes only `eggstack/emissary`. No upstream issue, PR, review, release, submission, merge/adoption request, contribution package, or maintainer contact is authorized.

## 16. Closure evidence required

Record exact Y004 SHA and reviewed diff, manifest/lock changes, enabled/disabled dependency trees, import containment, corrected fake-SAM wire evidence, broad verification, M095 non-change, unresolved findings, implementation SHA, and the exact future M113/router prerequisite.