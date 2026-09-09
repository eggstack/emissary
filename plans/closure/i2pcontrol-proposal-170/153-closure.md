# M153 Closure — Post-M146 Current-Head Requalification and Authority Rebase

Status: **closed as complete**

Date: `2026-09-09`

Source implementation plan:

- `plans/implementation/i2pcontrol-proposal-170/153-post-m146-current-head-requalification-and-authority-rebase.md`
  (Status now `closed as complete` with closure link; hard dependencies on
  M146/M141-M145 closures and the `336/29/475` mechanical recomputation
  satisfied as proven below).

Source roadmaps:

- `plans/subsystems/i2pcontrol-proposal-170-post-m146-corrective-roadmap.md`;
- `plans/subsystems/i2pcontrol-proposal-170-residual-primitive-completion-roadmap.md`;
- `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md`.

## Planning baseline

- Registration baseline `a0c4a791a6a7a974d34eaf93c45330aafd11116f` (clean
  worktree verified before M153 edits; M153 implementation started at
  `516739309d284acbeb4b6e490639f2ad27f2eae6` with no uncommitted changes).
- Last production-bearing head at registration:
  `7cbd80a6d72aa07d158ba9dc74f8bbacef767be2` (M145 no-std/format follow-up).
- Incoming M095 matrix: `336 apply / 29 blocked_primitive / 475
  not_applicable` across 840 TunnelManager option/family cells.
- M146 closed blocked with zero production delta and zero promotions.

Reviewed head:

- M153 qualification work completes on the working tree described in §9. No
  `emissary-core/**`, `emissary-util/**`, `emissary-cli/src/**`, manifest,
  lockfile, Yosemite, frontend, or workflow path was created, modified, or
  deleted (see §9 and the no-production-diff proof).

Pinned authority (all accessed read-only; no external fetch was required
beyond citing the already-pinned revisions recorded in M095/M146):

- Proposal 170 revision `2026-05-20`, status Open, SHA-256
  `f13ae00b886c5e72131bc5d5b138a371148d1faa6899a119a1dacb65a555e7dc`;
- Java I2PControl Proposal-170 head `45bb593000408071dd376b78848fdc246dccd964`;
- Java I2P/I2PTunnel reference snapshot `2c3fd2a9532cd86ec06cb6f2b9f3f813ca752243`;
- Yosemite optional exact revision `59140a2277bf296928d2e8ce39a148182eeff044`.

Current Proposal matrix at closure (mechanically recomputed, dispositions
unchanged):

- `336 apply / 29 blocked_primitive / 475 not_applicable` (840 cells);
- `095-full-support-matrix.toml` SHA-256
  `c67a3649d482bcb1269f73f1aa7ca70488845914ef0605d6ecebbdf5a53d23f9`
  (changed from `81851be26af3ed1e4d3f0526d914aaae3d061224d8b78c5cc985cd586d84b20d`
  by the single-line `current_production_head` metadata correction only; every
  cell disposition byte is identical).

## 1. Executive finding

M153 is closed as complete. All four post-M139 qualification defects named in
the plan are corrected:

1. M153 supersedes M139 as the current runtime/security qualification
   authority; M139 is retained as historical whole-surface evidence only.
2. M095 `current_production_head` is corrected from the stale pre-M141
   lifecycle head `e4f217cb1459e26bf011da46b67fc2c83cd192b5` to the accepted
   post-M145 production head `7cbd80a6d72aa07d158ba9dc74f8bbacef767be2`.
3. The broad I2PControl suite no longer fails from historical milestone tests
   asserting obsolete current matrix totals/wording: 19 stale aggregate
   assertions across 11 suites were rebased to milestone-local facts (§6),
   and the new `m153_post_m146_requalification` guard owns the current
   `336/29/475` aggregate plus the exact 29-cell residual set.
4. The M145 `7cbd80a...` no-std/format follow-up is explicitly absorbed into
   accepted production evidence and requalified (no-std checks green, §7).

Zero Proposal cells changed disposition. Zero production code changed.

## 2. Matrix and production-head reconciliation (WP1)

Mechanically parsed M095 (`python3` TOML recomputation over
`tunnel_manager.options[*].cells`):

- total = 840;
- apply = 336;
- blocked_primitive = 29;
- not_applicable = 475;
- declared `current_matrix_counts` agree with recomputation.

Residual blocker identities are exactly:

- 10 `SigType`: `client`, `httpclient`, `ircclient`, `socks`, `socksirc`,
  `connectclient`, `server`, `httpserver`, `httpbidirserver`, `ircserver`;
- 5 `EncryptLeaseSet`: `server`, `httpserver`, `httpbidirserver`,
  `ircserver`, `streamrserver`;
- 5 `OptionalLookup`: `server`, `httpserver`, `httpbidirserver`,
  `ircserver`, `streamrserver`;
- 5 `LeaseSetClientAuths`: `server`, `httpserver`, `httpbidirserver`,
  `ircserver`, `streamrserver`;
- 4 `UseOutproxyPlugin`: `httpclient`, `socks`, `socksirc`,
  `connectclient`.

Production-head determination:

- `git diff --name-only 7cbd80a..HEAD` (at registration head `5167393`)
  shows only docs/plans/tests paths (AGENTS.md, docs, m062 test, m146 guard,
  closures, plan/registry/roadmap files). No `emissary-*/src`, manifest,
  lockfile, Yosemite, frontend, or workflow path changed after `7cbd80a`.
- Therefore the true last production-bearing commit is
  `7cbd80a6d72aa07d158ba9dc74f8bbacef767be2`, as expected.
- M095 `current_production_head` now records exactly that commit. The M146
  closure commit is not named as a production head (it made no production
  change).
- Historical matrix hashes/counts in immutable closure records were not
  rewritten (§10 proves each closure still records its own head).

## 3. Requirement-to-evidence matrix

| Plan requirement | Evidence | Result |
|---|---|---|
| M095 mechanically remains `336/29/475` with exact 29-cell residual identity | §2 recomputation; new `m153_current_matrix_is_336_29_475_with_exact_residuals` guard (recomputed == declared == `336/29/475`; blocked set equals the exact 29-cell identity; per-option `10/5/5/5/4` split) | **pass** |
| no Proposal cell disposition changes | cell-by-cell dispositions untouched (only the file header line changed); `m095`/`m105` green; full suite green | **pass** |
| actual production head recorded correctly | §2 determination; M095 header now `7cbd80a...`; `m153` asserts it | **pass** |
| ordinary suite no longer fails solely on obsolete current aggregate counts/wording | §6 disposition table (19 stale assertions rebased); full `cargo test -p emissary-cli --features i2pcontrol`: **2249 passed, 0 failed (40 suites)** | **pass** |
| M141-M145 behavior composes on final source incl. M145 no-std after `7cbd80a...` | §4 composition evidence; `cargo check -p emissary-core --no-default-features --features no_std` pass; `cargo test -p emissary-core --lib` 1080 passed | **pass** |
| M146 blocked behavior remains fail-closed and production-neutral | `m146_outproxy_provider_blocked` 5/5 green; §4 M146 evidence | **pass** |
| M127-M129 and lifecycle security regressions green | `m127`/`m128`/`m129` suites green (security tests untouched); M139 lifecycle composition tests green; §4 | **pass** |
| M061/M062 exact containment/dependency isolation truthful, not weakened | §5 cumulative path audit; `m061`/`m062` green (23/23); no broad waiver; Y005 intact | **pass** |
| no medium/high correctness/security finding remains | §11 findings (lows only) | **pass** |
| M153 becomes current runtime/security qualification authority | §8 authority updates across registry/README/roadmaps/AGENTS/docs; `m153_authority_docs_name_m153_and_retain_partial_support` green | **pass** |
| no production Rust/dependency/Yosemite change | §9 no-production-diff proof; `git diff --check` pass | **pass** |

## 4. Whole-surface post-M145 composition requalification (WP3)

### M140 applicability

- The seven reclassified cells (`Profile:httpclient|ircclient|socks|
  socksirc|connectclient|streamrclient`, `ConnectDelay:streamrclient`)
  remain affirmative `not_applicable` at the current head (asserted by the
  rebased `m140_matrix_reconciliation_is_exact_and_contained` and the M153
  composition block).
- `Profile:client` is `apply` via the later M143 promotion (explicitly
  asserted, with M143 ownership).

### M141 UniqueLocalAddressPerClient

- Only `httpserver`/`httpbidirserver` apply; no other UniqueLocal cell is
  apply (rebased `m141_matrix_promotes_exactly_two_unique_local_cells`).
- Canonical peer-hash loopback source, literal-loopback target, no
  DNS/non-loopback fallback, IPv6 unavailable-source fail-closed behavior:
  existing `m141` behavioral guards green (unchanged).

### M142 HTTP SSLProxies / JumpList

- Both `httpclient` cells apply with M142 row ownership; I2P-only bounded
  selection, bounded/deterministic last-failure, JumpList metadata-only, no
  clearnet/DNS escape or response injection: existing `m142` adversarial and
  live HTTPS/CONNECT selection guards green (unchanged).

### M143 Profile

- `bulk`/omitted preserve default; `interactive` reaches the neutral
  streaming max-window owner; deterministic generation/shared-session
  behavior; Proposal-free core; no-std green: existing `m143` behavioral
  guards plus `cargo check -p emissary-core --no-default-features --features
  no_std` green.

### M144 UseSSL

- Local-listener vs local-target TLS distinct from M129 management TLS and
  Yosemite SAM TLS; real handshake at the correct boundary; fail-closed
  trust; no plaintext fallback; redacted private material; bounded
  timeout/cancellation/generation: existing `m144` TLS behavioral guards
  green (unchanged); dependency containment (`tokio-rustls`/`rcgen` present,
  `webpki-roots`/`rustls-native-certs` absent) re-pinned by the rebased
  `m144_containment_docs_and_registry_agree`.

### M145 MultiHoming / shouldBundleReplyInfo

- `SessionManager` neutral ownership; omitted/true bundling; false
  suppression with intact `NewSession` mandatory bundling; no
  wrong/private/fabricated LeaseSet bundling: existing `m145` guards green.
- `7cbd80a...` explicitly recorded as accepted M145 production evidence in
  the plan (§1), M095 head (§2), registry, and this closure.

### M146 blocked behavior

- All four `UseOutproxyPlugin` cells remain `blocked_primitive`; supplied
  values fail before allocation; omitted flag preserves ordinary `ProxyList`
  behavior; no dummy provider; no direct-clearnet path; zero production
  delta: `m146_outproxy_provider_blocked` 5/5 green.

### Security/control-plane regression (§7 of plan)

- M127 finite token lifetime, M128 bounded batch/body/task semantics, M129
  fail-closed non-loopback management TLS: behavioral tests in those suites
  untouched and green (only their stale matrix-count companions were
  rebased).
- M135-M137/M134 composition, M061/M062 containment, secret redaction,
  feature-disabled isolation: `m061`/`m062`/adversarial/production suites
  green inside the 2249-pass full run.
- No behavioral regression was found; nothing stopped as
  corrective-required.

## 5. Containment and dependency audit (WP4)

Cumulative post-M139 production diff
(`e4f217c..7cbd80a`, i.e. M141-M145 plus the M145 follow-up):

- `emissary-cli/src/i2pcontrol/**` (11 files): `backends/client.rs`,
  `backends/connect_client.rs`, `backends/filters/http_client.rs`,
  `backends/http_bidir.rs`, `backends/http_client.rs`,
  `backends/http_server.rs`, `backends/options.rs`,
  `backends/runtime/mod.rs`, `backends/runtime/presentation_tls.rs`,
  `backends/runtime/session.rs`, `production.rs` — M141/M142/M144
  I2PControl-local production changes plus composition paths. Within the
  accepted `emissary-cli/src/i2pcontrol/**` policy home.
- Neutral core (6 files, all exact M061 owners): M143
  `emissary-core/src/sam/protocol/streaming/config.rs`,
  `.../streaming/mod.rs`, `.../streaming/stream/active.rs`,
  `emissary-core/src/sam/session.rs`; M145
  `emissary-core/src/destination/mod.rs`,
  `emissary-core/src/destination/session/mod.rs`. Each path verified present
  in `061-containment-boundary.toml` exact-file authority.
- Zero manifest/lockfile/dependency changes in range; zero Yosemite changes.
- No Proposal/I2PControl/TunnelManager/JSON-RPC vocabulary in neutral core
  production declarations (mechanical grep over the six files split at
  `#[cfg(test)]`; the single hit is a doc comment explicitly stating no
  Proposal vocabulary).
- No broad `crypto/**`, `netdb/**`, `i2np/**`, `destination/**`,
  `primitives/**`, or transport waiver exists in M061 (mechanical grep).
- No new direct dependency escaped the i2pcontrol feature boundary
  (`m062` 23/23 green, including the new `is_authorized_m153_path` budget).
- Yosemite remains exact optional Y005 (`59140a2...`, `m062` + `m126`
  lockfile/pin guards green).
- M061/M062 were not broadened: the only M062 change adds the
  test/planning/docs M153 path budget (`is_authorized_m153_path` wired into
  both the production-path and prohibited-pattern assert chains), mirroring
  the M146 precedent. `061` unchanged.

## 6. Historical-test versus current-head test model (WP2)

WP1 classified every full-suite failure as stale-historical: 19 failing
assertions across 11 suites, all obsolete current-aggregate counts/wording
(`325`/`327`/`329`/`330`/`334`-era pins vs the legitimate `336/29/475`
head). No behavioral failure was found.

### Stale-test disposition table

| Suite / test | Stale pin | Disposition |
|---|---|---|
| `m126 current_matrix_is_mechanically_requalified` | recomputed `325/47/468` | **historical invariant**: durable router/SetConfig/type checks kept; tunnel counts replaced by declared==recomputed self-consistency (840 cells, no `planned_apply`); M126 closure `284/96/460` pinned as history |
| `m126 active_support_docs_agree_with_the_current_partial_claim` | active docs state `325/47/468` | **historical invariant**: durable partial + no-full-support kept; current counts delegated to M153/registry; M126 closure history pinned |
| `m127 proposal_matrix_unchanged_by_token_lifetime` | `325/47/468` | **historical invariant**: M127 closure `284/96/460` pinned; current matrix self-consistency only |
| `m128 proposal_matrix_unchanged_by_batch_conformance` | `325/47/468` | **historical invariant**: M128 closure `284/96/460` pinned; self-consistency only |
| `m129 proposal_matrix_unchanged_by_tls_fail_closed` | `325/47/468` | **historical invariant**: M129 closure `284/96/460` pinned; self-consistency only |
| `m130 proposal_matrix_is_mechanically_recomputed` | `325/47/468` recomputed + declared | **historical invariant**: M130 closure `284/96/460` pinned; router/SetConfig/selector shape kept; self-consistency only |
| `m130 active_authority_retains_partial_support_and_m130_lineage` | planning docs state `325/47/468`; README names M130 | **historical invariant + obsolete-duplicate repair**: counts delegated to M153; M130/M127-M129 lineage scoped to registry + post-M114 roadmap where it actually lives (the README never named M130, verified at HEAD — latent unsatisfiable assert); README checked for current M153 authority instead; `M127-M129` range shorthand accepted |
| `m139 current_matrix_is_exhaustive_and_residuals_are_exact` | current `325/47/468` + 47-cell set + `e4f217` head | **historical invariant**: M139 closure evidence pinned (`325/47/468`, `e4f217`); current blocked proven a strict subset of the historical 47 (later promotions only removed cells); declared blocked-count self-consistency |
| `m140 m140_matrix_reconciliation_is_exact_and_contained` | current `325/40/475` + exact 7-cell removal | **historical invariant**: M140 closure/map `325/40/475` pinned as milestone-local authority; durable facts asserted at current head (7 cells still N/A; `Profile:client` apply via M143; current blocked ⊆ 47-cell pre-set) |
| `m141 m141_matrix_promotes_exactly_two_unique_local_cells` | declared + recomputed `327/38/475` | **historical invariant**: M141 closure `327/38/475` pinned; durable facts (2 cells apply, row ownership, no other UniqueLocal apply) kept; aggregate dropped |
| `m141 m141_residual_inventory_subtracts_two_cells` | `len == 38` + stale spot-checks (MultiHoming/Profile/UseSSL later promoted) | **historical invariant**: len pin dropped (owned by M153); spot-checks replaced with still-blocked residuals; 2 promoted cells asserted absent |
| `m142` matrix + residual (2 tests) | `329/36/475`, `len == 36`, stale spot-checks | **historical invariant**: M142 closure `329/36/475` pinned; durable SSLProxies/JumpList apply + ownership kept; stale spot-checks replaced |
| `m143` matrix + residual + docs (3 tests) | `330/35/475` (×2) + docs state `330/35/475` | **historical invariant**: M143 closure `330/35/475` pinned; durable `Profile:client` apply + ownership kept; docs test now pins closure + M143/Profile lineage (OR-logic, matching M141/M142) |
| `m144` matrix + residual + containment-docs (3 tests) | `334/31/475` (×2 incl. docs) + `len == 31` | **historical invariant**: M144 closure `334/31/475` pinned; durable 4×UseSSL apply + ownership kept; dependency-containment facts kept; stale spot-check (MultiHoming) replaced |

No `#[ignore]`, broad feature gate, loose `>=` count check, or security-assertion deletion was used. Every removed aggregate pin is covered equally or more strongly by the new M153 guard (`336/29/475` + exact 29-cell set) or by retained cell-level facts.

### New M153 current-head guard

New `emissary-cli/tests/m153_post_m146_requalification.rs` (3 tests) owns:

- `m153_current_matrix_is_336_29_475_with_exact_residuals`: recomputed ==
  declared == `336/29/475` over 840 cells; `current_production_head ==
  7cbd80a...`; exact 29-cell residual identity with the `10/5/5/4`… split
  (`SigType` 10, `EncryptLeaseSet`/`OptionalLookup`/`LeaseSetClientAuths` 5
  each, `UseOutproxyPlugin` 4); M141-M145 promotion composition (11 cells
  apply); M140 N/A composition (7 cells); M146 blocked composition (4 cells).
- `m153_authority_docs_name_m153_and_retain_partial_support`: AGENTS,
  registry, implementation README, post-M146 roadmap, docs README, support
  doc name M153 + partial + no full-support; authority docs state
  `336/29/475`; AGENTS retains M139 as history.
- `m153_historical_closures_remain_immutable`: M139-M146 closures pin
  `325/47/468`, `325/40/475`, `327/38/475`, `329/36/475`, `330/35/475`,
  `334/31/475`, `336/29/475` (×2); M126-M130 closures pin `284/96/460`.

## 7. Verification outcomes

| Command group | Result |
|---|---|
| `cargo check -p emissary-core` | **pass** |
| `cargo check -p emissary-core --no-default-features --features no_std` | **pass** (M145 no-std requalified on final source) |
| `cargo test -p emissary-core --lib --no-fail-fast` | **pass**: `1080 passed, 2 ignored` |
| `cargo check -p emissary-cli --no-default-features --features i2pcontrol` | **pass** |
| `cargo check -p emissary-cli --no-default-features` | **pass** |
| `cargo check` | **pass** |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --lib --no-fail-fast` | **pass**: `821 passed` |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --no-fail-fast` | **pass**: `2249 passed (40 suites, 0 failed)` |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test i2pcontrol_live_runtime -- --nocapture` | **pass**: `1 passed` (live-runtime production proof) |
| targeted `m061/m062/m095/m105/m153` (+ rebased `m126-m146`) | **pass**: `155 passed (18 suites, 0 failed)` |
| `cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings` | **evidence only**: sole error is the known pre-existing `chunks_exact` lint in untouched `backends/filters/proxy.rs:60` (same as M144-M146); all 13 M153-touched files clippy-clean |
| `cargo fmt --all -- --check` (stable) | **evidence only**: 419 repo-wide diffs of pre-existing stable/nightly drift in untouched files |
| `cargo +nightly fmt --all -- --check` | **evidence only**: 58 dirty files, all untouched by M153; all 13 M153-touched `.rs` files individually nightly-clean (drift inside touched files normalized, no unrelated file touched) |
| `git diff --check` | **pass** |

### Production-path diff (bounded)

`git status --short` over `emissary-core/src`, `emissary-cli/src`,
`emissary-util/src`, all manifests, and `Cargo.lock` is empty: M153 made
zero production changes. `110-completion-ledger.toml` gains no entry (zero
promotions). `061` unchanged. Streamr datagram limits (16-subscriber, 60s
expiry, 1200-byte payload, 4095-byte transport buffer, 15s refresh, bounded
shutdown, loopback-only UDP) untouched; remote datagrams never choose a
local UDP destination.

## 8. Requirement-to-evidence matrix (planning process §2.5)

Covered in §3 (plan requirements) plus:

| Closure duty | Evidence |
|---|---|
| implementation commits | this M153 working tree (test/planning/docs-only; commit lands with this closure) |
| invariant review | partial-support, no-fabrication, I2P-only egress, fail-closed TLS/auth/batch, containment, and Y005 invariants re-proved (§4-§5); no invariant weakened |
| failure/recovery and contention evidence | deterministic lifecycle guards (`m139` composition), live-runtime proof, adversarial suite — all green in the 2249-pass run |
| compatibility, migration, security review | no wire/protocol/storage/dependency change ⇒ no migration impact; no new attack surface (no production code); blocked cells stay fail-closed |
| documentation and operational evidence | §10 changed paths; `m153` authority-docs guard; operational impact none |

## 9. Changed paths (exact budget only)

- `plans/implementation/i2pcontrol-proposal-170/153-post-m146-current-head-requalification-and-authority-rebase.md`
  (Status `registered / dependency-ready` → `closed as complete` with closure link);
- `plans/implementation/i2pcontrol-proposal-170/154-m147-signature-domain-security-and-owner-refreeze.md`
  (Status `deferred / unregistered` → `registered / dependency-ready` on the satisfied M153 hard dependency; §12);
- new `plans/closure/i2pcontrol-proposal-170/153-closure.md` (this file);
- new `emissary-cli/tests/m153_post_m146_requalification.rs` (3 current-head guards);
- `emissary-cli/tests/m062_dependency_containment.rs` (exact
  `is_authorized_m153_path` for the M153 test/planning/docs budget, wired
  into both assert chains);
- rebased historical suites (aggregate pins → closure/cell-local facts, plus
  nightly-fmt normalization inside touched files only):
  `emissary-cli/tests/m126_requalification.rs`,
  `emissary-cli/tests/m127_token_lifetime.rs`,
  `emissary-cli/tests/m128_jsonrpc_batch.rs`,
  `emissary-cli/tests/m129_nonloopback_tls.rs`,
  `emissary-cli/tests/m130_post_corrective_requalification.rs`,
  `emissary-cli/tests/m139_post_lifecycle_requalification.rs`,
  `emissary-cli/tests/m140_residual_streaming_applicability.rs`,
  `emissary-cli/tests/m141_unique_local_source.rs`,
  `emissary-cli/tests/m142_httpclient_sslproxies_jumplist.rs`,
  `emissary-cli/tests/m143_streaming_profile.rs`,
  `emissary-cli/tests/m144_presentation_usessl.rs`;
- `plans/implementation/i2pcontrol-proposal-170/095-full-support-matrix.toml`
  (one-line `current_production_head` correction `e4f217c...` →
  `7cbd80a...`; zero cell changes);
- `plans/registry.md` (M153 closed; M153 → current authority, M139 →
  historical; M154 registered; execution chain, registration rules, lineage
  table updated);
- `plans/implementation/i2pcontrol-proposal-170/README.md` (M153 closed
  section; M154 registered; authority/chain updated);
- `plans/subsystems/i2pcontrol-proposal-170-post-m146-corrective-roadmap.md`
  (M153 closed; M154 registered; registration discipline updated);
- `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md`
  (M153 closed/current authority; M154 registered; chain/status updated);
- `plans/subsystems/i2pcontrol-proposal-170-residual-primitive-completion-roadmap.md`
  (new §8 records M153 closure as post-line authority; line history
  otherwise immutable);
- `AGENTS.md` (M139 historical superseded-by-M153 wording; M153 closed
  current-authority entry with closure link; M154 registered);
- `docs/i2pcontrol/README.md`, `docs/i2pcontrol/proposal-170-support.md`,
  `docs/i2pcontrol/tunnel-manager.md` (M153 authority + `336/29/475`
  wording; `tunnel-manager` also records M154 registered; `tunnel-backends`
  unchanged — its M146 blocked statement remains true).

`git diff --check` passes.

## 10. Documentation and operations

Machine authorities: `095-full-support-matrix.toml` (`336/29/475`, SHA
`c67a3649d482bcb1269f73f1aa7ca70488845914ef0605d6ecebbdf5a53d23f9`;
counts/residuals identical to the M146 head, header pointer corrected),
`110-completion-ledger.toml` (no new entry; zero promotions), `061`
(unchanged), `m095` still expects `336/29/475`, `m105` residual audit green,
`m062` gains `is_authorized_m153_path`, new `m153` guard owns the
aggregate/residual/authority/immutability checks. Support docs, `AGENTS.md`,
registry, implementation README, and all three roadmaps agree on
`336/29/475`, partial support, M153 current authority (M139 historical),
M146 blocked, M154 registered, and the 29-cell residual inventory.
Operational impact: none; absent values preserve prior behavior.

## 11. Unresolved findings

| Severity | Finding | Impact | Required action |
|---|---|---|---|
| low | `cargo fmt --all -- --check` (stable) reports 419 repo-wide diffs; `cargo +nightly fmt --all -- --check` reports 58 dirty files | none on behavior; evidence-only | record, do not normalize unrelated source; all 13 M153-touched files are individually nightly-clean |
| low | `cargo clippy --all-targets` reports the pre-existing `chunks_exact` lint in untouched `backends/filters/proxy.rs:60` | none on M153 files (clean) | record, separate corrective if needed (same finding as M144-M146) |
| low | Remaining 29 blocked cells (10 SigType, 15 LeaseSet crypto/lookup/auth, 4 UseOutproxyPlugin) | partial Proposal 170 support remains | deferred M154/M147-M152 chain (M154 now registered); M146 stays terminal-blocked under current policy |
| info | `m130` README-lineage assert was latently unsatisfiable (README never named M130, verified at HEAD) and previously masked by the earlier count failure | no behavior impact; lineage now scoped to registry + post-M114 roadmap where it lives | recorded in §6; no further action |
| info | M095 file SHA changed (`81851be...` → `c67a3649...`) from the one-line header-pointer correction | counts/residuals byte-identical; no test pins the file hash | documented; no action |

No high/medium correctness defect remains.

## 12. Registry updates and future-plan unblock determination

Applied alongside this closure (see §9 for the file list):

- `153-*.md` plan: Status `registered / dependency-ready` → `closed as
  complete` with closure link;
- `plans/registry.md`: M153 → closed as complete (`336/29/475` unchanged);
  qualification authority M139 → historical, M153 → current; execution chain
  updated; M154 → **registered / dependency-ready**; M147-M152 stay
  deferred/unregistered; lineage table gains M153;
- `plans/implementation/.../README.md` and both active roadmaps: M153
  closed/current authority; M154 registered; counts/inventory/registration
  discipline updated;
- residual roadmap: new §8 notes M153 as post-line authority without
  rewriting line history;
- `110-completion-ledger.toml`: no new entry (zero promotions);
- `m062`: gains `is_authorized_m153_path`; `m061`: unchanged.

Future-plan unblock determination (as required by the tasking):

- **M154 UNBLOCKED → registered / dependency-ready.** Its sole hard
  dependency (M153 closure) is satisfied by this record. Its zero-production,
  zero-promotion audit may proceed. No other gate blocks it.
- **M147 remains deferred/unregistered.** It hard-depends on M154 closure
  (or an M154 split/amend disposition), which does not exist yet. M153 does
  not register M147 directly, per the M147 pre-registration gate and the
  roadmap discipline.
- **M148-M152 remain deferred/unregistered** behind the M147 chain.
- **M146 remains closed as blocked** with no successor; it is not reopened
  by M153 or by the future crypto/LeaseSet tail.
- No other future-plan status required a change. File presence alone never
  authorizes production work.

## Internal-only / read-only-upstream attestation

- External sources (the pinned Proposal text, Java I2PControl/I2PTunnel
  snapshots, Yosemite revision — all cited from existing pinned records; no
  new external fetch was performed in this session) were accessed read-only
  for evidence;
- No upstream or third-party repository, issue, pull request, discussion,
  review, or maintainer channel was opened, drafted, updated, commented on,
  or contacted;
- No commit, branch, tag, patch, release, or artifact was pushed to any
  upstream remote before this closure commit. The commit/push performed with
  this closure targets the authorized internal repository
  (`eggstack/emissary` origin) on the current branch only, as explicitly
  directed for this internal workstream;
- No upstream review, approval, feedback, adoption, or merge was requested;
- No upstream contribution package, patch series, or submission checklist
  was prepared;
- Violation would invalidate this closure per `plans/003-planning-process.md`
  §11; no such violation occurred.

(End of file)
