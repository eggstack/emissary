# M152 Closure — Final Residual Proposal-170 Requalification (Safe Partial / Terminal)

Status: **closed as complete; safe partial / terminal under current policy; zero Proposal promotions; M152 is the final qualification authority for the implemented subset**

Date: `2026-09-10`

Source implementation plan:

- `plans/implementation/i2pcontrol-proposal-170/152-final-residual-proposal-170-requalification.md`
  (Status now `closed as complete` with closure link; hard dependencies satisfied by
  `plans/closure/i2pcontrol-proposal-170/153-closure.md` (M153 ancestry),
  `plans/closure/i2pcontrol-proposal-170/146-closure.md` (M146 blocked),
  `plans/closure/i2pcontrol-proposal-170/154-closure.md` (M154 disposition C, M147 path blocked),
  `plans/closure/i2pcontrol-proposal-170/155-closure.md` through
  `plans/closure/i2pcontrol-proposal-170/162-closure.md` (M155-M162 closed, zero promotions);
  zero production and zero promotion budgets observed as proven below).

Source roadmaps:

- `plans/subsystems/i2pcontrol-proposal-170-post-m154-leaseset-security-corrective-roadmap.md`;
- `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md`.

Promotion budget: **zero Proposal cells** (observed; M095 remains `336/29/475`).

Production budget: **zero production Rust/dependency/Yosemite changes**
(planning/test/evidence only; same scope as the M161 zero-production gate).

## Planning baseline

- Registration baseline `2783ad06` (registered M152 handoff; clean worktree
  before M152 work; M062 records the zero-production budget with no production
  paths, dependencies, manifests, lockfile, Yosemite, or core change).
- Last production-bearing head before and after M152: `babf9fa6` (M162
  blocked-integration implementation/closure; nine I2PControl owners). M152
  registration (`2783ad06`) and this closure touch only plans/docs/guard-test
  paths.
- Incoming M095 matrix: `336 apply / 29 blocked_primitive / 475
  not_applicable` across 840 TunnelManager option/family cells.
- M153 closed complete as the whole-surface runtime/security qualification
  ancestry; M146 closed blocked; M154 closed disposition C (M147 path blocked,
  M148 deferred behind M147); M155-M160 closed complete with zero promotions;
  M161 closed outcome B (valid but blocked legacy AES, zero production/
  promotions, no successor); M162 closed blocked integration (nine-file
  I2PControl subset, zero promotions, all fifteen LeaseSet-security cells
  remain blocked).
- Reviewed head: the working tree described in §8. No `emissary-core/**`,
  `emissary-util/**`, `emissary-cli/src/**`, manifest, lockfile, Yosemite,
  frontend, or workflow production path was created, modified, or deleted
  (see §8/§11 and the no-production-diff proof). The only Rust change is the
  bounded `m062` planning-guard extension (§6), which is test-only and
  explicitly allowed by the plan ("planning/test/evidence only").

Pinned authority (all accessed read-only; no upstream mutation, contact, or
submission occurred — see read-only attestation):

- Proposal 170 revision `2026-05-20`, status Open, SHA-256
  `f13ae00b886c5e72131bc5d5b138a371148d1faa6899a119a1dacb65a555e7dc`
  (cited from M095/M153/M155; no new Proposal fetch — M152 adds no Proposal
  policy beyond requalification);
- Java I2PControl Proposal-170 head `45bb593000408071dd376b78848fdc246dccd964`
  and Java I2P/I2PTunnel reference snapshot `2c3fd2a9532cd86ec06cb6f2b9f3f813ca752243`
  (cited from pinned M155/M161 records; no new fetch);
- I2P Encrypted LeaseSet / common-structures / I2CP / Red25519 specifications
  (cited from M155/M161 records; no new fetch — M152 implements no crypto);
- Yosemite optional exact revision `59140a2277bf296928d2e8ce39a148182eeff044`
  (unchanged; no Yosemite change in M152; Yosemite gaps cited from the frozen
  M162 §4 record);
- Emissary source at the reviewed tree (all file:line citations below read
  at HEAD plus the workdir diff closed here).

Current Proposal matrix at closure (mechanically recomputed via the
`m095_full_support_matrix` suite, dispositions unchanged):

- `336 apply / 29 blocked_primitive / 475 not_applicable` (840 cells);
- `095-full-support-matrix.toml` SHA-256
  `c67a3649d482bcb1269f73f1aa7ca70488845914ef0605d6ecebbdf5a53d23f9`
  (identical to the M153 head; every cell disposition byte identical);
- residual split `SigType` 10, `EncryptLeaseSet` 5, `OptionalLookup` 5,
  `LeaseSetClientAuths` 5, `UseOutproxyPlugin` 4 — identical to
  M153/M154/M155/M156/M157/M158/M159/M160/M161/M162 entry.

Full support requires `blocked_primitive == 0`. With 29 blockers, full support
is not declared. Safe-partial terminal closure is declared because every
blocker is tied to an accepted blocked/security disposition (§5) and no
remaining dependency-ready safe implementation plan exists under current
architecture/policy (§12).

## 1. Requirement-to-evidence matrix (plan §§3-5)

| Plan-required trace | Evidence |
|---|---|
| Mechanical matrix audit: exact apply/blocked/N/A totals, exact blocked identities, every `apply` row has runtime/interop evidence (not parser reachability), every N/A row has affirmative family/spec evidence, SigType/UseOutproxyPlugin/EncryptLeaseSet/OptionalLookup/LeaseSetClientAuths dispositions match closed milestones, any M162 promotion tied to complete field contract | Totals recomputed via `m095_full_support_matrix` (`840/336/29/475` == declared) plus `python3` TOML recomputation; blocked identities exactly SigType×10 (client/httpclient/ircclient/socks/socksirc/connectclient/server/httpserver/httpbidirserver/ircserver), EncryptLeaseSet×5 + OptionalLookup×5 + LeaseSetClientAuths×5 (server/httpserver/httpbidirserver/ircserver/streamrserver), UseOutproxyPlugin×4 (httpclient/socks/socksirc/connectclient); `apply` runtime evidence via full green suite (§7: 2278 passed, 40 suites — M127-M146 behavioral guards, M141-M145 promotion guards, M156-M162 lib suites, live runtime); N/A affirmative evidence via retained `m140` + `m095`/`m105` guards green; dispositions match M146 (blocked), M154-C (SigType blocked), M161-B (EncryptLeaseSet blocked), M162 (all 15 LeaseSet cells blocked, zero promotions); M162 promoted zero cells so no partial-domain promotion exists |
| Whole-surface composition M127-M162 (§4) | Green composition table below (§2): M127 finite token lifetime, M128 bounded batches, M129 fail-closed management TLS, shared session/destination + M135 quantities/LeaseSet truthfulness, Reduce→Close→IdlePolicy→NewDest lifecycle, M141 loopback confinement, M142 I2P-only egress, M143 retained Profile, M144 app-TLS separation, M145 reply-bundling, M146 blocked provider, M156 blinding, M157 type-5 publication/rollover, M158 lookup-secret/B32, M159 PSK + 4096 bound, M160 DH + all-zero/bound, M161-B legacy disposition, M162 ten-mode table + fail-closed gates + redaction + Yosemite gaps. Blocked predecessors proven fail-before-allocation (M146/M154/M161/M162 negative suites green), never treated as support |
| Security/containment audit (§5) | Held and proven (§3): Proposal/admin policy under `i2pcontrol/`; non-I2PControl paths exact in M061/M062 with neutral Proposal-free APIs (`m061`/`m062` green); M159/M160 stayed in exact four-file owner set (no fifth file; core 1159 green, byte-identical through M161/M162); no broad crypto/netdb/i2np/destination waiver; no Proposal terminology in core neutral APIs (`policy_terms` guard green); I2PControl-only deps optional/feature-owned (Yosemite `59140a2` exact pin, `m062` + lockfile guards green); no plaintext/unsecreted/unauthenticated downgrade (M162 preflight==start + core negative suites green); Red25519/lookup/PSK/DH/private material redacted (M162 redaction suites green); M162 definition+secret generations transactionally vacuous (blocked — none allocated; existing secret-store suites green); no direct-clearnet fallback (M146 + filter guards green); no high/medium finding waived (§13) |
| Stop conditions (§8 — none triggered beyond recorded dispositions) | No `apply` cell is inert/approximate (full suite green; parser-only reachability never counted — M162 §1 observed); no production/dependency change required (`git log` over production paths since `babf9fa6` empty; §11); interop consistent with local tests (live runtime green; reference gaps source-proven per M161/M162, no flag-ignored success counted); no late feature weakened containment (M156-M162 primitives byte-identical; `m061`/`m062` green); no broad waiver required (M061 unchanged); every blocker has an accepted disposition (§5); M162 promoted zero fields so no partial-domain promotion exists |

## 2. Whole-surface composition requalification

| Lineage | Disposition requalified on the actual head |
|---|---|
| M127 finite token lifetime | Behavioral suite green (token expiry distinct from unknown, atomic removal, standard expired-token error); part of full 2278-pass run |
| M128 bounded JSON-RPC admission/batches | Behavioral suite green (bounded batch/body/task semantics); part of full run |
| M129 fail-closed management TLS | Behavioral suite green (non-loopback managed TLS fails closed); part of full run |
| Shared session/destination ownership + M135 live quantity/LeaseSet truthfulness | Retained guards green inside full run; no second session/destination owner added by M156-M162 |
| Reduce → Close → IdlePolicy → NewDest lifecycle | M134/M136/M137 composition guards green inside full run; deterministic lifecycle preserved |
| M141 UniqueLocal loopback confinement | 5/6 behavioral guards green (per-client `127.<hash>` / `fd`+hash source bind, literal-loopback targets only, no DNS/fallback); 1/6 docs-agreement guard repaired in this closure (AGENTS/registry/README drift, §7) — behavior intact |
| M142 SSLProxies/JumpList I2P-only egress | 13/14 behavioral guards green (bounded I2P-only selection, last-failure avoidance, metadata-only JumpList, no clearnet/DNS escape); 1/14 docs guard repaired (§7) |
| M143 retained Profile behavior | 13/14 behavioral guards green (bulk/omitted→128, interactive→16, neutral streaming owner, Proposal-free core, no-std green); 1/14 docs guard repaired (§7) |
| M144 application TLS separation | Full suite green (listener vs target TLS distinct from M129/Yosemite SAM TLS, real handshake at correct boundary, fail-closed trust, redacted material, bounded timeout/cancellation/generation) |
| M145 reply-LeaseSet bundling privacy/liveness | Suite green (omitted/true bundles, false suppresses `ExistingSession` with `NewSession` retained, no wrong/private/fabricated bundling) |
| M146 blocked provider | 5/5 blocked-behavior guards green (all four cells `blocked_primitive`, fail before allocation, no dummy provider, no direct-clearnet path, zero production delta) |
| M156 Red25519/blinding | Closed neutral primitive requalified: lib suites green, no-std green, `curve25519-dalek` exact-version direct edge, constant-time ops, zero promotions |
| M157 modern type-5 encrypted-LS2 publication/storage verification/UTC rollover | Closed primitive requalified: `LeaseSetManager` remains sole publication/rollover/verification owner (no second scheduler), type-preserving NetDB cache/flood/lookup, lib + NetDB suites green, zero promotions |
| M158 lookup-secret/blinded-address + secret redaction | Closed primitive requalified: secret-aware blinding, extended-B32 flags (`secret_required` independent), redaction boundary (no secret in logs/Get/rawConfig), suites green, zero promotions |
| M159 PSK client authorization + 4096-byte bounded-work contract | Closed primitive requalified: flags `0x03`, fresh cookie/salt, `ELS2PSKA` 52-byte schedule, duplicate-preserving records, checked complete-size before per-client work, 4096 ceiling, zero promotions; wrong-key/tamper/size-bound negative suites green |
| M160 DH/X25519 client authorization + all-zero/bounded-work contract | Closed primitive requalified: flags `0x01`, fresh ephemeral keypair + cookie per object, `ELS2_XCA` 52-byte schedule, explicit all-zero shared-secret rejection, duplicate-preserving randomized records, 4096 ceiling, zero promotions; negative suites green |
| M161 legacy AES disposition | Outcome B requalified: valid LS1 `i2cp.encryptLeaseSet=true` contract but blocked (disproportionate/unsafe resurrection of deprecated insecure LS1); no successor; all five `EncryptLeaseSet` cells remain blocked; SAM gates (legacy-with-type-5 rejects, OLD `1` rejects, inert-ordinary without encryption) green |
| M162 Proposal field validation, ten-mode mapping, secret transactionality, no-downgrade | Blocked integration requalified: executable ten-mode table, typed/redacted domain, five-family fail-before-allocation gates (closing `httpserver`/`streamr` inert gaps), session-layer defense-in-depth with ordinary-only wire fixture, control-plane persistence/redaction hardening; custody deferred (would be inert without Yosemite base-key path) with design frozen; Yosemite base/duplicate/bound gaps + M161-B keep all 15 cells blocked; zero promotions; 14 M162 lib tests green |

Adversarial/failure evidence: malformed Proposal shapes fail at control plane before persistence (INVALID_PARAMS, no echo); typed+raw LeaseSet presence fails identically in preflight and start before store/runtime/wire work on all five families; oversize/sparse/mixed/over-ceiling type-5 inputs reject at frozen core gates; legacy flag either rejects with type 5 or stays inert without encrypting; no LS1 fallback added; ordinary Ed25519/LS2/DatabaseStore-type-3/NetDB-ordinary/M135/M145 SAM behavior preserved (core 1159 + CLI lib 835 green).

## 3. Security/containment audit

- Proposal/admin policy remains under `emissary-cli/src/i2pcontrol/**` wherever possible: M162 nine production files are all inside the M061 policy root; M152 adds no production path.
- Every non-I2PControl production path introduced by the line is exact in M061/M062 with a neutral owner/rationale: M159/M160 four core files are already realized M061 owners (`crypto/els2.rs`, `destination/lease_set.rs`, `sam/parser.rs`, `sam/session.rs`); no fifth file; `m061_containment` 8/8 green; M061 unchanged by M152.
- M159/M160 stayed inside their registered exact neutral owner sets: proven by `git log` (core production head unchanged since `1629b0a5` through M161/M162/M152 registrations) and 1159 core tests green with zero regressions.
- No broad crypto/netdb/i2np/destination/primitives/event/router/tunnel/transport waiver exists: M061 has no prefix allowance (mechanical rule + `m061` green); M062 records only exact-path authorizations.
- No Proposal terminology leaked into core neutral APIs: `policy_terms_do_not_leak_into_non_policy_production_paths` green (new Proposal strings live only under `i2pcontrol/`).
- I2PControl-only direct dependencies remain optional/feature-owned: `subtle` + `yosemite-i2pcontrol` optional under `i2pcontrol` feature; `curve25519-dalek` unconditional core-owned (neutral primitive, not I2PControl-gated) per dependency rule; `m062` 23/23 green; no manifest/lockfile change in M152.
- Yosemite remains exact optional authority `59140a2277bf296928d2e8ce39a148182eeff044` (unchanged; lockfile/pin guards green; no Yosemite change in M152).
- No plaintext/unsecreted/unauthenticated downgrade in LeaseSet security modes: M162 gates fail before allocation; ordinary wire emits no LeaseSet keys (negative fixture green); type-5 mismatches fail closed at preserved core gates.
- Red25519/lookup-secret/PSK/DH/private-key material is not logged or response-facing: `OptionRedacted`/entry redaction + `TunnelDefinition` raw-keys-only Debug + backend field-name-only errors + Get omission + redacted `CompatibilityKey` (identity values in equality only); M162 redaction suites green.
- M162 definition + LeaseSet-security secret generations are transactionally consistent: vacuously satisfied (blocked — no secret generations allocated, so no split-brain/stale-republish/destroy-on-failed-start path exists); existing stage/commit/discard + current/backup discipline untouched and covered by retained secret-store tests.
- No direct-clearnet fallback for M146: no provider/network path added; clearnet-without-outproxy still fails via filter; `m146` 5/5 green.
- No high/medium security finding is waived to improve completion status: only lows/infos recorded (§13); residual 29 blockers stay fail-closed.

## 4. Terminal blocker inventory (all 29 cells blocked_primitive)

| Cluster | Cells | Accepted disposition | Architecture/security decision required to resume |
|---|---|---|---|
| `SigType` ×10 (client/httpclient/ircclient/socks/socksirc/connectclient/server/httpserver/httpbidirserver/ircserver) | 10 | M154 disposition C / M147 path closed as blocked: required destination-capable domain {0,1,2,3,7,11} cannot be truthfully shipped (type-0 generation refused by policy; types 1-3 deprecated legacy with missing signers; type 11 has no maintained I2P-suitable Rust primitive; fixed-Ed25519 owners end to end) | Separate accepted architecture/security plan supplying a maintained RedDSA primitive + legacy-suite generation policy + versioned storage migration + exact-file M061/M062 authorization; cannot be hidden inside LeaseSet work (type-11 blinding stays derived-only) |
| `EncryptLeaseSet` ×5 (server/httpserver/httpbidirserver/ircserver/streamrserver) | 5 | M161 outcome B (valid but blocked legacy AES) + M162 Yosemite base/duplicate/bound gaps: legacy `encrypted (aes)` is a real LS1 contract but resurrecting deprecated insecure LS1 (ElGamal-256B, destination-pubkey IV, flag-less leases, keyring sharing, broken revocation) is disproportionate/unsafe; modern modes blocked by Yosemite gaps below; no partial-enum apply | Legacy: separate accepted plan blessing LS1 resurrection (new LS1 wire/DatabaseStore/floodfill/keyring/SAM subsystem) — explicitly rejected as unsafe under current policy. Modern: Yosemite typed `leaseSetPrivKey` amendment + duplicate/bound dispositions (below) via separate accepted plan; then complete-field promotion proof per M162 rules |
| `OptionalLookup` ×5 (same five server families) | 5 | M162 blocked: valid lookup uses include PSK/DH lookup variants that are Yosemite-inexpressible (base gap) plus persistent custody absent | Yosemite typed base-key amendment + I2PControl custody/transaction owner (frozen design in M162 §5) via separate accepted plan; full-contract proof (mapping, M158 publication, custody, transactions, B32, interop, no downgrade) |
| `LeaseSetClientAuths` ×5 (same five) | 5 | M162 blocked: Yosemite base gap (six of eight modern modes inexpressible) + duplicate gap (Yosemite rejects duplicates that core preserves per pinned Java semantics) + 16-vs-99 bound gap (valid 17-99-entry lists would reject) + custody absent | Same Yosemite amendment (base field + duplicate/bound dispositions) + custody owner via separate accepted plan; full PSK/DH list-contract proof (parsing/formatting, 4096 bound, custody/redaction, edit/restart, interop, no inert/sparse-truncation/drop) |
| `UseOutproxyPlugin` ×4 (httpclient/socks/socksirc/connectclient) | 4 | M146 blocked: no real bounded I2P-routed local outproxy provider within budget; direct-clearnet provider prohibited; registry-only/dummy provider has zero support value | Separate accepted provider plan with a real I2P-routed provider (bounded registry, timeout/cancellation, filter preservation, credential redaction, provider-first ordering, I2P-only fallback) |

Promotion ceilings from `336/29/475` (ceilings, not claims): OptionalLookup + LeaseSetClientAuths complete with EncryptLeaseSet blocked → at most `346/19/475`; complete EncryptLeaseSet enum too → at most `351/14/475`. SigType ×10 and UseOutproxyPlugin ×4 remain independent blockers in both ceilings.

## 5. Outcome determination: safe partial / terminal

M152 closes as **safe partial / terminal under current policy**:

- The mechanically recomputed matrix has 29 applicable blockers, so full support for the pinned Proposal revision is not declared.
- All safe implementation work is complete: M127-M145 safe residuals applied (11 cells); M146/M154/M161/M162 truthfully blocked with hardening (no inert acceptance); M155-M160 neutral primitives complete with zero promotions.
- Every remaining blocker has an explicit accepted architecture/security disposition (§4) with no remaining dependency-ready safe implementation plan under current architecture/policy (§12).
- M152 is therefore the final qualification authority for the implemented subset. The full-support roadmap stays explicitly partial at the pinned revision.
- Historical closures remain immutable. External reference access remains read-only.

## 6. Exact-path reconciliation (M061/M062)

M061 (`061-containment-boundary.toml` + `m061_containment.rs`):

- No change required and none made. M152 is zero-production with planning/test/evidence-only paths; all touched planning/docs paths are outside the production allowlist check or inside the policy-root/docs set. No new M061 path waiver was created.

M062 (`062-dependency-containment.toml` + `m062_dependency_containment.rs`):

- `062-dependency-containment.toml` `[current_registration]` bookkeeping closed: milestone `M152 (closed complete, safe partial / terminal)` realized with zero production paths, zero new files, `new_direct_dependencies=[]`, `manifest_changes=[]`, `lockfile_change=false`, `yosemite_change=false`, `i2pcontrol_source_change=false`.
- `m062_dependency_containment.rs` gains narrow `is_authorized_m152_path` covering exactly the twelve M152 closure paths (AGENTS, 3 docs, guard test itself, `152-closure.md`, `062-dependency-containment.toml`, `152-*.md` plan, implementation README, registry, both roadmaps), wired into both the allowed-path and prohibited-pattern assert chains (same pattern as the M161/M162 helpers; authorizes the new closure file, which no earlier helper covers).
- `policy_terms_do_not_leak_into_non_policy_production_paths` passes: no production source was touched; new Proposal strings live only in planning/docs/test paths.

## 7. Verification outcomes

| Command group | Result |
|---|---|
| `cargo check -p emissary-core` | **pass** |
| `cargo check -p emissary-core --no-default-features --features no_std` | **pass** |
| `cargo test -p emissary-core --lib --no-fail-fast` | **pass**: `1159 passed, 2 ignored` (identical to M160/M161/M162 head; zero regressions — expected: no core touched) |
| `cargo check -p emissary-cli --no-default-features --features i2pcontrol` | **pass** |
| `cargo check -p emissary-cli --no-default-features` | **pass** |
| `cargo check` | **pass** |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --lib --no-fail-fast` | **pass**: `835 passed` (821 pre-existing + 14 M162 tests, zero regressions) |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --no-fail-fast` | **pass**: `2278 passed, 0 failed (40 suites)` — includes live runtime (1), all behavioral suites, and all docs-agreement guards after the §7 repair below |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test i2pcontrol_live_runtime -- --nocapture` | **pass**: `1 passed` (live-runtime production proof; also included in the 2278 above) |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment --test m062_dependency_containment --test m095_full_support_matrix --test m105_residual_option_audit --no-fail-fast` | **pass**: `8 + 23 + 3 + 1 = 35` across 4 suites (includes the new `is_authorized_m152_path` wiring in both chains) |
| M095 mechanical recomputation (via `m095_full_support_matrix` suite + `python3` TOML recomputation) | **pass**: `840/336/29/475` == declared; residual `10/5/5/5/4` with exact identities (§1) |
| Durable M156-M162 tests (authenticated ELS2 wrong-key/tamper/size-bound + M162 persistence rollback/redaction) | **pass**: all retained suites green inside the 1159 core + 835 CLI lib + 2278 full runs (zero regressions) |
| `cargo clippy -p emissary-core --all-targets -- -D warnings` | **pass**: No issues found |
| `cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings` | pre-existing failure only in unrelated `proxy.rs:60` (`chunks_exact`); zero warnings from M152 files (M152 touches one test file, clippy-clean; see §13) |
| stable `rustfmt` on M152-touched Rust files | **pass** for `m062_dependency_containment.rs` (`rustfmt --check` clean; repo nightly-only options warn; `max_width = 100` observed; no new nightly drift — see §13) |
| `cargo fmt --all -- --check` (whole tree) | pre-existing drift only (330 `Diff in` sites) in files outside the M152 budget; unrelated files untouched |
| `git diff --check` | **pass** |

Docs-agreement repair performed inside this closure (planning/docs-only, within budget): before M152 edits, the full suite reported 7 docs-only failures (`m126`/`m130`/`m139`/`m141`/`m142`/`m143`/`m153` authority/docs guards — all behavioral companions green) caused by AGENTS.md/registry.md/implementation-README compaction during the M155-M162 line (lost `partial` wording and M130/M139/M141/M142/M143 lineage that the historical guards pin). No behavioral regression was found. M152 restores the required lineage + `partial` wording + M152 final authority in exactly the §8 docs set (AGENTS, registry, README, both roadmaps, 3 docs) without touching historical closures or behavioral tests. After repair: all 7 suites green (`4 + 17 + 3 + 6 + 14 + 14 + 3 = 61` tests) and the full `2278-pass` run above is clean. This follows the M153 §6 precedent (historical invariant + current-head guard; repair active docs, never rewrite history).

## 8. Changed paths (exact budget only)

Guards/ledgers (2):

- `emissary-cli/tests/m062_dependency_containment.rs` (M152 authorization
  helper in both chains);
- `plans/implementation/i2pcontrol-proposal-170/062-dependency-containment.toml`
  (current-registration closure bookkeeping, zero-budget recorded).

Planning/closure/registry (8):

- `plans/implementation/i2pcontrol-proposal-170/152-final-residual-proposal-170-requalification.md`
  (Status `registered / dependency-ready` → `closed as complete` with
  closure link and safe-partial record);
- new `plans/closure/i2pcontrol-proposal-170/152-closure.md` (this file);
- `plans/registry.md` (M152 → closed safe partial / terminal, zero
  promotions; no registered successor; historical lineage retained for guard
  agreement; chain/rules updated);
- `plans/implementation/i2pcontrol-proposal-170/README.md` (M152 closed
  section; lineage/chain updated);
- `plans/subsystems/i2pcontrol-proposal-170-post-m154-leaseset-security-corrective-roadmap.md`
  (M152 closed; graph updated);
- `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md`
  (M152 closed as final authority; full-support stays explicitly partial;
  handoff updated);
- `AGENTS.md` (M152 closed entry; historical lineage + partial wording
  restored; chain updated).

Docs (3, authority wording only):

- `docs/i2pcontrol/README.md`, `docs/i2pcontrol/proposal-170-support.md`,
  `docs/i2pcontrol/tunnel-manager.md` (M152 closed authority +
  `336/29/475` wording retained; residual-blocked statements retained).

`061/062`, `095/105/110` semantics: `061` unchanged (zero production; no
broad waiver); `062` gains only the exact M152 closure authorization above;
`095/105/110` intentionally **unchanged** (zero promotions).

## 9. Requirement-to-evidence matrix (planning process §2.5)

| Closure duty | Evidence |
|---|---|
| implementation commits | two commits: registration (`2783ad06`, plans-only) plus the commit landing with this closure (guard test + M062 TOML + planning records + this closure; exact paths in §8); no production commit exists for M152 by design |
| invariant review | partial-support, no-fabrication, I2P-only egress, fail-closed option validation/TLS/auth/batch, containment, and Y005 invariants re-proved by green `m061`/`m062`/`m095`/`m105`/`m126-m146`/`m153` guards inside the 2278-pass run; no invariant weakened; M147/M148 not reopened (no SigType change; type-11 stays derived-only); M146 untouched (no egress); no downgrade path exists (all LeaseSet presence fails before allocation; legacy flag either rejects with type 5 at frozen core gates or stays rejected at I2PControl gates without encrypting; no LS1 fallback added; ordinary wire emits no LeaseSet keys) |
| failure/recovery and contention evidence | malformed Proposal shapes fail at control plane before persistence (INVALID_PARAMS, no echo); typed+raw presence fails identically in preflight and start before store/runtime/wire work on all five families; oversize/sparse/mixed/over-ceiling type-5 inputs still reject at frozen core gates; no secret generations exist to roll back (blocked), so store-failure/cancellation/rollback is vacuously satisfied (staging discipline untouched and covered by retained secret-store tests); no second scheduler/state machine added; no lock spans changed; shutdown paths unchanged; ordinary definitions restart byte-compatibly; live runtime + adversarial suites green |
| compatibility, migration, security review | no wire/protocol/storage/dependency change ⇒ no migration impact (no definition JSON change; stores unchanged); no new attack surface (no production code; validation/redaction gates only); all 29 LeaseSet/SigType/outproxy cells stay fail-closed (matrix unchanged); legacy AES stays valid-but-blocked with no silent aliasing (§4); `cargo clippy -p emissary-core -D warnings` clean; `emissary-cli -D warnings` failure is pre-existing and unrelated with zero M152-file warnings |
| documentation and operational evidence | §8 changed paths; authority docs name M152 closed safe partial / terminal and retain partial support with M153 ancestry and M139/M141-M145 lineage; operational impact none beyond the truthful blocked errors and redacted Get already established by M162 (M152 itself changes no runtime behavior) |
| M152 acceptance (plan closure evidence) | final HEAD + M095 hash/counts/blocked identities (above), safe-partial disposition with terminal blockers + resume decisions (§§4-5), verification outputs (§7), M127-M162 composition/adversarial table (§2), M061/M062 evidence (§§3/6), live/reference interop (live runtime green; reference gaps source-proven per M161/M162, no flag-ignored success counted), active documentation reconciliation (§§7-8), unresolved blockers (§§4/13), external read-only attestation (below) |

## 10. Promotion accounting (zero)

- `110-completion-ledger.toml`: no new entry (zero promotions).
- `095-full-support-matrix.toml` / `105-residual-option-audit.toml`:
  unchanged (SHA `c67a3649d482bcb1269f73f1aa7ca70488845914ef0605d6ecebbdf5a53d23f9`).
- Mechanically recomputed `336/29/475` == declared; residual
  `10/5/5/5/4` with exact identities (§1).
- Per-field/per-family decisions (all blocked_primitive, five server families
  each unless noted): `SigType` — blocked (M154-C, 10 families);
  `EncryptLeaseSet` — blocked (M161-B + M162 gaps, 5 families; no
  partial-enum apply); `OptionalLookup` — blocked (M162 Yosemite base gap +
  custody absent, 5 families); `LeaseSetClientAuths` — blocked (M162
  base/duplicate/bound gaps + custody absent, 5 families);
  `UseOutproxyPlugin` — blocked (M146, 4 client/proxy families). Clients
  remain not_applicable where applicable (unchanged).
- Infrastructure alone has zero support value (registry rule); neutral
  M156-M160 primitives and the M162 typed domain/table/gates claim no cell.
  A stored mode string, redacted secret, or Yosemite-serializable subset alone
  is explicitly not support.

## 11. Production-head determination

- Last production-bearing commit before and after M152: `babf9fa6` (M162
  implementation/closure). M152 registration (`2783ad06`) and this closure
  commit touch only plans/docs/guard-test paths.
- `git log 2783ad06..HEAD` over `emissary-core/src`,
  `emissary-cli/src`, `emissary-util/src`, all manifests and `Cargo.lock`
  at closure time is empty except the `m062` guard-test helper (test-only;
  no production path). No `emissary-core/**`, `emissary-util/**`, Yosemite,
  frontend, or workflow production path was created, modified, or deleted
  (`git status --short` shows only the §8 paths).
- `git diff --name-only` over manifests/lockfile is empty: no `Cargo.toml`,
  `emissary-cli/Cargo.toml`, `emissary-core/Cargo.toml`, or `Cargo.lock`
  change.
- Streamr datagram limits (16-subscriber, 60s expiry, 1200-byte payload,
  4095-byte transport-buffer, 15s refresh, bounded shutdown,
  loopback-only UDP) untouched; remote datagrams never choose a local UDP
  destination.

## 12. Registry updates and future-plan unblock determination

Applied alongside this closure (see §8 for the file list):

- `152-*.md` plan: Status `registered / dependency-ready` → `closed as
  complete` with closure link and safe-partial record;
- `plans/registry.md`: M152 → closed safe partial / terminal
  (`336/29/475` unchanged, zero promotions, zero production diff); **no
  registered successor**; historical lineage retained; chain/rules updated;
- `110-completion-ledger.toml`: no new entry (zero promotions);
- `m061`: unchanged (zero production; no broad waiver);
- `m062`: exact M152 closure authorization only (§6); no broad waiver.

Future-plan unblock determination (as required by the tasking):

- **M152 CLOSED safe partial / terminal.** Its entry gate (M153 ancestry;
  M146 still blocked; M147/M148 still blocked via M154-C; M155-M158 closed;
  M159/M160 closed with zero promotions and exact M061/M062 outcomes; M161
  outcome B with no successor; M162 closed blocked with zero promotions and
  all fifteen LeaseSet-security cells blocked; no other Proposal capability
  plan registered) was satisfied at registration, and all plan requirements
  are met with a zero-production diff and zero matrix promotions. No stop
  condition beyond the recorded safe-partial disposition triggered: no
  `apply` cell is inert, no outside file or dependency was required, interop
  is consistent with local tests, no late feature weakened containment, no
  broad waiver was needed, every blocker has an accepted disposition, and
  M162 promoted zero fields.
- **No successor is registered by this closure.** M152 is the final
  qualification authority for the implemented subset; the full-support
  roadmap stays explicitly partial at the pinned revision. Only a separately
  accepted architecture/security plan may register future work.
- **M149-M151 remain superseded/unregistered (do not execute).** Confirmed
  fully superseded by M155-M162; no re-gating or revival is authorized.
- **M147/M148 remain closed-blocked/deferred-behind-M147** (M154
  disposition C); this line does not reopen them. Type-11 blinding remains
  explicitly not a `SigType` reopening.
- **M146 remains closed as blocked** with no successor; it is not reopened
  by M152 or by any LeaseSet tail.
- **A future Yosemite-amended LeaseSet successor is NOT registered here.**
  If maintainers later accept a Yosemite typed `leaseSetPrivKey` (plus
  duplicate/bound dispositions) amendment, that work needs its own accepted
  implementation plan with exact-path/dependency authorization; file presence
  of the ten-mode table alone never authorises production work. The deferred
  custody/transaction design stays frozen in M162 §5.
- No other future-plan status required a change. File presence alone never
  authorises production work.

## 13. Unresolved findings

- Non-blocking: `cargo clippy -p emissary-cli --no-default-features
  --features i2pcontrol --all-targets -- -D warnings` reports a
  pre-existing lint in `backends/filters/proxy.rs:60` (`chunks_exact`),
  untouched by M152; M152 files contribute zero warnings. Severity:
  informational; no corrective pass required (out of M152 exact-budget
  scope; same finding as M153/M160/M161/M162).
- Non-blocking: `cargo fmt --all -- --check` reports pre-existing drift
  (330 `Diff in` sites) in files outside the M152 budget at the registration
  baseline; the M152-touched Rust file
  (`m062_dependency_containment.rs`) is `rustfmt --check` clean with
  `max_width = 100` observed, and no new nightly drift was introduced by
  M152 lines. Severity: informational; unrelated files were not touched.
- Non-blocking: 29 residual blocked cells (§4) keep full Proposal-170 status
  partial by design. Severity: terminal-blocked under current policy (each
  with an accepted disposition and a named resume decision); no corrective
  pass is authorized inside this line.
- Informational: pre-repair full-suite docs drift (7 historical
  authority/docs guards failing on AGENTS/registry/README compaction;
  behavior green) was repaired inside this closure within the
  planning/docs-only budget per the M153 §6 precedent; historical closures
  were not rewritten and no behavioral test was weakened. Post-repair full
  suite is `2278 passed, 0 failed (40 suites)`.

No high/medium correctness defect remains.

## Internal-only / read-only-upstream attestation

- External sources (the already-pinned Proposal/Java/Yosemite revisions
  cited from existing M095/M153/M155/M161/M162 records; no new
  Proposal/Java fetch; no live reference router provisioned for this
  zero-production requalification) were accessed read-only for evidence;
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
