# M122 Closure — Corrected Yosemite LeaseSet Pin Adoption

Status: **closed**

Plan: `plans/implementation/i2pcontrol-proposal-170/122-corrected-yosemite-leaseset-pin-adoption.md`

Implementation commit: `548c174` (`feat(i2pcontrol): adopt Y004 LeaseSet pin with adapter reachability (M122)`)

Closure date: 2026-09-03

Proposal authority: I2P Proposal 170, pinned revision `2026-05-20`, status Open.

## Disposition

M122 is closed. The optional `yosemite-i2pcontrol` alias advances from the
Y002 implementation revision `8026f5b424fc178d683e63555335f8b33e0aba04` to the
exact reviewed Y004 implementation commit
`c2db73dba35dd9392947af5c74df29b0b556775f`, and the corrected generic LeaseSet
API is proven reachable from an I2PControl-only test path at a fake SAM
endpoint. M122 is dependency/adaptation plumbing only: no Proposal 170 cell is
promoted, and the M095 matrix remains exactly `284 apply / 98
blocked_primitive / 458 not_applicable`.

Authoritative matrix: `plans/implementation/i2pcontrol-proposal-170/095-full-support-matrix.toml`
(unchanged by M122; `apply = 284`, `blocked_primitive = 98`,
`not_applicable = 458`).

## Hard readiness gates (plan §2 — all satisfied before promotion)

1. Yosemite Y004 is closed with an exact implementation commit SHA explicitly
   suitable for consumer pinning: `c2db73dba35dd9392947af5c74df29b0b556775f`
   (`eggstack/yosemite` closure
   `plans/closure/004-y003-leaseset-wire-semantics-corrective.md`). The Yosemite
   registry records Y004 as closed and states the Yosemite-side blocker for a
   future Emissary exact-revision adoption plan is removed.
2. Y004 closure has no open high/medium protocol/security finding in its
   LeaseSet wire surface: its closure records "No in-scope Y004 finding remains
   open." Clippy and format failures there are explicitly dispositioned as
   baseline-only with no new Y004 diagnostic remaining.
3. The exact Y004 diff from the Y002 baseline was reviewed: production changes
   are limited to `src/options.rs`, `src/proto/session.rs`, and `src/lib.rs`
   (public type re-exports). No Emissary/Proposal-specific production code, no
   dependency/release work, no unrelated runtime expansion. The diff contains
   no `Proposal170`, `I2PControl`, `TunnelManager`, or `Emissary` concept
   (verified by case-insensitive search over the production diff: zero
   matches). No `Cargo.toml`/`Cargo.lock` change in Yosemite.
4. M121 is closed (`21f4070`; closure
   `plans/closure/i2pcontrol-proposal-170/121-closure.md`); the Emissary
   semantic baseline `284 / 98 / 458` was frozen before dependency adoption.
5. ADR-0005 remains accepted and unchanged with respect to optional
   exact-revision aliasing.

No broader Emissary API redesign was required: Y004 is additive over Y002 for
every field Emissary consumes (new `lease_set_client_auths` defaults to empty;
all other corrected fields keep their names with tightened validation), so the
existing I2PControl call sites compile unchanged.

## WP1 — Y004 exact-revision review

Yosemite history from the Y002 pin to the Y004 implementation commit:

- `9ac7d9a` Y003 LeaseSet surface (defective; never consumed by Emissary);
- `94d7455` Y003 SHA record;
- `8024b9d`/`e04a2f3`/`d67c0eb` Y004 planning;
- `c2db73d` Y004 implementation (pinned).

`git diff 8026f5b..c2db73d --stat` touches only `src/options.rs`,
`src/proto/session.rs`, `src/lib.rs`, and Yosemite planning/registry/roadmap
docs. The corrected semantics now owned by the dependency:

- `lease_set_private_key` → `i2cp.leaseSetPrivateKey` (was the distinct
  `i2cp.leaseSetPrivKey`);
- `lease_set_signing_private_key` → `i2cp.leaseSetSigningPrivateKey` (was the
  truncated `i2cp.leaseSetSigningPrivKey`);
- per-client authorization is mode-aware: `i2cp.leaseSetClient.dh.<n>` /
  `i2cp.leaseSetClient.psk.<n>` with deterministic contiguous per-mode
  numbering and constructed `b64name:b64key` values (replaces the opaque
  single-token representation);
- reference-backed numeric domains: auth `0..=2`, blinded type `0..=65535`,
  LeaseSet type `1..=255`;
- bounded validation with fail-before-wire behavior, redacted `Debug`, and
  default wire unchanged when LeaseSet features are unused.

## WP2 — exact dependency pin

Changed paths:

- `emissary-cli/Cargo.toml` — only the `rev` of the existing optional
  `yosemite-i2pcontrol` alias: `8026f5b…` → `c2db73d…`. Ordinary workspace
  `yosemite = { workspace = true }` (registry 0.7.0) is untouched; no
  `[patch.crates-io]`, workspace replacement, path dependency, vendoring, or
  floating branch/tag.
- `Cargo.lock` — the two-line source/revision update for the fork package
  instance; the ordinary registry `yosemite` instance is unchanged.
- `plans/implementation/i2pcontrol-proposal-170/062-dependency-containment.toml`
  — fork revision in both `[direct_dependencies]` and `[fork_dependency]`
  advanced to Y004; the planning comment now records M122 as the landed Y004
  adoption instead of a future plan.

No `emissary-core/**`, `emissary-util/**`, startup manager, frontend,
workflow, or release production change.

## WP3 — compile-time/API adaptation

No production consumer change was required: all existing I2PControl
`SessionOptions` construction sites use `..Default::default()` and compile
unchanged against Y004's additive API. No Proposal `EncryptLeaseSet` /
`LeaseSetClientAuths` mapping was added; that translation remains owned by a
future registered M113-successor plan. Adaptation is confined to focused
tests/helpers under `emissary-cli/src/i2pcontrol/**`.

## WP4 — fake-SAM corrected-wire reachability

New deterministic tests in
`emissary-cli/src/i2pcontrol/backends/runtime/session.rs` (all
I2PControl-only, all fixture keying material mirrors Yosemite Y004's own
public test vectors — never router keying material):

- `m122_y004_corrected_leaseset_wire_is_reachable_at_fake_sam`: builds
  validated Y004 `SessionOptions` directly (`encrypt_lease_set=true`,
  `lease_set_auth_type=1`, `lease_set_blinded_type=10`, `lease_set_type=3`,
  canonical key/secret/private/signing fixtures, one DH entry for `alice`, one
  PSK entry for `bob`) and drives a real `Session::<style::Stream>::new`
  against a local fake SAM endpoint. The observed `SESSION CREATE` carries
  `i2cp.encryptLeaseSet=true`, `i2cp.leaseSetAuthType=1`,
  `i2cp.leaseSetBlindedType=10`, `i2cp.leaseSetType=3`,
  `i2cp.leaseSetPrivateKey=…`, `i2cp.leaseSetSigningPrivateKey=…`, exactly one
  `i2cp.leaseSetClient.dh.` entry (`dh.0=YWxpY2U=:…`) and exactly one
  `i2cp.leaseSetClient.psk.` entry (`psk.0=Ym9i:…`); the Y003 misspellings
  (`leaseSetPrivKey`, `leaseSetSigningPrivKey`, `leaseSetClientAuth`) are
  asserted absent. These are dependency reachability assertions, not M095
  `apply` evidence.
- `m122_y004_malformed_leaseset_values_reject_before_wire`: opaque fragments,
  empty/control-byte names, non-I2P-base64 and unused-bit key violations,
  same-mode duplicates, generic-path collisions on every canonical typed key
  and numbered client-auth namespace, and numeric-domain violations
  (`auth_type=3`, `blinded_type=65536`, `lease_set_type=0`, malformed key)
  all reject. Numeric-domain cases assert
  `Err(Error::Protocol(ProtocolError::InvalidOption))` against a closed SAM
  port, proving validation precedes the TCP connect (a post-connect failure
  would surface as I/O instead).
- `m122_y004_leaseset_material_is_redacted_from_debug`: `SessionOptions` and
  `LeaseSetClientAuth` `Debug` output contains `<redacted>` and none of the
  fixture key/secret values.

Y001 variance/backup/custom and Y002 signature-generation adapter regressions
(`generic_session_wire_adapter_*`, `client_secret_store` `DEST GENERATE`
plumbing, `compatibility_key_*`) remain green and untouched.

## WP5 — dependency containment

- `emissary-cli/tests/m062_dependency_containment.rs`: expected fork revision
  advanced to Y004; new `is_authorized_m121_path` (covers the previously
  unregistered `121-closure.md`, which made the M062 budget test fail on the
  M121 closure head) and `is_authorized_m122_path` (Cargo/lock, I2PControl
  session tests, 062 TOML, 122 plan/closure, README, registry, both roadmaps,
  tunnel-manager doc) helpers wired into both the allowlist and the
  prohibited-pattern assertions.
- Provenance evidence (recorded post-implementation):
  - feature-disabled `cargo tree --edges normal`: registry `yosemite v0.7.0`
    only; no `github.com/eggstack/yosemite` string;
  - I2PControl-enabled tree: registry `yosemite v0.7.0` plus exactly
    `yosemite v0.7.0 (https://github.com/eggstack/yosemite?rev=c2db73d…#c2db73d…)`;
  - all 16 `yosemite_i2pcontrol` import sites remain below
    `emissary-cli/src/i2pcontrol/` (single `rg -l` enumeration);
  - no `[patch]`, `vendor`, path, or floating Yosemite source in either
    manifest or lockfile.

## WP6 — closure/readiness audit

M095 is byte-identical before and after M122 (`apply = 284`,
`blocked_primitive = 98`, `not_applicable = 458`); the focused matrix/audit
guards (`m095_full_support_matrix`, `m105_residual_option_audit`) pass
unchanged, as required for infrastructure-only work.

## Requirement-to-evidence matrix

| Requirement | Evidence and outcome |
|---|---|
| Exact reviewed Y004 pin | `emissary-cli/Cargo.toml` rev `c2db73d…`; `Cargo.lock` fork source with matching rev+hash; WP1 diff review above |
| Ordinary provenance unchanged | Workspace `yosemite 0.7.0` registry declaration untouched; disabled tree has no fork URL |
| Feature/containment boundary | Alias remains `optional = true`, owned only by `i2pcontrol` via `dep:yosemite-i2pcontrol`; M062 guards pass (23 tests) |
| Corrected API reachable | `m122_y004_corrected_leaseset_wire_is_reachable_at_fake_sam` observes canonical keys/values at a fake SAM endpoint |
| No Proposal promotion | No `EncryptLeaseSet`/`LeaseSetClientAuths` mapping added; M095 counts unchanged; matrix file untouched |
| Malformed fails before wire | `m122_y004_malformed_leaseset_values_reject_before_wire` (constructor + pre-connect `InvalidOption` + namespace-collision rejection) |
| Secret hygiene | `m122_y004_leaseset_material_is_redacted_from_debug`; rejection strings name options, never values |
| No raw SAM serializer | Tests drive `Session::new`; no command-string construction in Emissary |

## Broad verification

From the repository root at the implementation commit:

| Command | Outcome |
|---|---|
| `cargo check -p emissary-cli --no-default-features --features i2pcontrol` | PASS |
| `cargo check -p emissary-cli --no-default-features` | PASS |
| `cargo check` | PASS |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --lib --no-fail-fast` | PASS — 704 tests (701 pre-M122 + 3 new) |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --no-fail-fast` | PASS — 1893 tests across 26 suites |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment --test m062_dependency_containment --test m095_full_support_matrix --test m105_residual_option_audit --no-fail-fast` | PASS — 33 tests |
| `cargo tree -p emissary-cli --no-default-features --edges normal` | PASS — registry Yosemite only, no fork URL |
| `cargo tree -p emissary-cli --no-default-features --features i2pcontrol --edges normal` | PASS — registry plus exact Y004 fork revision |
| `cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings` | PASS — no issues |
| `cargo fmt --all -- --check` | Pre-existing stable/nightly toolchain drift only (M121 recorded 591 stable-fmt diffs post-change; same drift class, no bulk rewrite retained; M122's new test lines are verified clean under `cargo +nightly fmt -p emissary-cli -- --check` — remaining session.rs hunks are all pre-existing) |
| `git diff --check` | PASS |

## Failure, cancellation, restart, contention review

- A dependency/API build failure produces no runtime migration; there is no
  persisted-data migration in M122.
- Invalid LeaseSet options fail inside `SessionController::new` before TCP
  connect, or inside bounded constructors before any session object exists;
  no partial client-auth sequence can be emitted (Yosemite validates the full
  collection before serializing).
- No new timer, task, lock, generation owner, or session-sharing path is
  added. Production `SessionOptions` never carry non-empty
  `lease_set_client_auths` (no Proposal mapping exists), so the
  `compatibility_key` sharing identity is behaviorally unchanged; the new
  non-empty auth collections exist only inside the M122 tests. Any future
  M113 successor that maps client auths MUST re-audit compatibility identity
  for key-material-differentiated sessions (current `Debug`-based identity
  redacts key material by design).
- Cancellation/restart semantics are unchanged; the fake-SAM tests are
  generation-local to their Tokio test tasks.

## Compatibility, migration, and security review

- Feature-disabled/default builds use ordinary registry Yosemite only —
  byte/provenance-equivalent with respect to the ordinary dependency.
- I2PControl-enabled builds carry both package instances (accepted ADR-0005
  consequence). Default `SessionOptions` wire is unchanged when LeaseSet
  features are unused (Y004 invariant, covered by Yosemite's own regression
  tests and Emissary's untouched Y001/Y002 adapter tests).
- No secret, private key, proxy credential, or custom-option value enters
  diagnostics; rejection strings name only the option (asserted by the
  redaction test).
- No clearnet fallback, proxy-boundary weakening, loopback/SSRF relaxation,
  LeaseSet downgrade, `Shared`-session ownership change, server/startup
  behavior change, or frontend coupling.
- M061 source-boundary guards pass unchanged (all touched production paths
  remain under the I2PControl policy root).

## Documentation and operational evidence

- M062 manifest/TOML and this closure record the exact landed Y004 revision.
- `plans/registry.md`, implementation `README.md`, the post-M114 corrective
  roadmap, the full-support completion roadmap (M113 current-result note),
  and `docs/i2pcontrol/tunnel-manager.md` (accepted-revision paragraph) are
  updated to the Y004 pin with the explicit statement that the 21 M113 cells
  remain blocked for want of a router-side encrypted-LeaseSet owner and
  Proposal mapping — not for want of SAM transport.
- Historical closures (M111/M113/M114/M117/M121) are untouched; this record
  supersedes only the dependency-pin disposition.

## Unresolved findings

1. Low (tooling, pre-existing): stable/nightly rustfmt drift across the repo;
   recorded, not rewritten.
2. None (high/medium): no open M122 security, containment, correctness, or
   lifecycle finding. The M062 budget test's miss of `121-closure.md`
   authorization (failing on the M121 closure head) is corrected by this
   pass's `is_authorized_m121_path` helper; it was a planning-test
   allowlist gap, not a production containment breach.

## Future-plan unblocking decision

- **M122 unblocks no implementation handoff by itself.** Per the corrective
  roadmap's LeaseSet rule, a new M113-successor neutral LeaseSet plan must
  not be registered until a focused read-only capability/crypto-ownership
  audit freezes the exact LeaseSet type(s), existing vs missing cryptographic
  primitives, server-only canonical core owner, key/secret lifecycle, SAM
  option mapping, no-downgrade rule, bounded client-auth handling, NetDb
  publication/query semantics, and minimal production paths. That audit is
  **authorized** by this closure (it requires the M122-closed
  transport + the M121-frozen `284 / 98 / 458` baseline) but is **not yet
  performed or registered**: no new implementation plan file is created here,
  and no residual cell changes disposition.
- No final reclosure is ready: 98 applicable `blocked_primitive` cells remain
  (4 UseSSL + 10 SigType + 63 client + 21 server).
- Yosemite registers no later plan blocked on Y004; no consumer-side change
  beyond this pin was made by Y004's closure.

## Internal-only attestation

External specifications, Java/I2P reference snapshots, and Yosemite sources,
history, and closure records were inspected read-only. No upstream
repository, issue, pull request, review, maintainer channel, release
artifact, or external branch/tag was mutated or requested. No Yosemite
repository write occurred under this plan; the only repository writes are the
internal `eggstack/emissary` implementation (`548c174`) and planning closure
in this pass. No upstream contribution artifact was prepared.
