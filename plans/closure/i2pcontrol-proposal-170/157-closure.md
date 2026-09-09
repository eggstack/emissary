# M157 Closure — Modern Encrypted LeaseSet2 Publication Primitive

Status: **closed as complete; M158 hard dependency satisfied, amendment pending**

Date: `2026-09-09`

Source implementation plan:

- `plans/implementation/i2pcontrol-proposal-170/157-modern-encrypted-leaseset2-publication-primitive.md`
  (Status now `closed as complete` with closure link; hard dependency on
  M156 closure satisfied by `plans/closure/i2pcontrol-proposal-170/156-closure.md`;
  zero promotion budget observed as proven below).

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-post-m154-leaseset-security-corrective-roadmap.md`.

Promotion budget: **zero Proposal cells**.

Production budget: **exact ten-path M157 set only**
(`emissary-core/src/crypto/els2.rs` new,
`emissary-core/src/crypto/mod.rs` declaration only,
`emissary-core/src/primitives/lease_set.rs`,
`emissary-core/src/primitives/mod.rs` exact re-export only,
`emissary-core/src/i2np/database/store.rs`,
`emissary-core/src/netdb/mod.rs`,
`emissary-core/src/destination/lease_set.rs`,
`emissary-core/src/destination/mod.rs`,
`emissary-core/src/sam/parser.rs`,
`emissary-core/src/sam/session.rs`),
with M061/M062 reconciliation in the same commit. No Cargo, lockfile,
Yosemite, or `emissary-cli/src/i2pcontrol/**` production change.

## Planning baseline

- Registration baseline `a11fd9242ba6d5450cc94ef32bdfa2f9eeb6b292` (registered
  M157 handoff; clean worktree before M157 implementation).
- Last production-bearing head before M157: `1678790cc74d4075e800425c57312da89e1169fe`
  (M156 primitive; unchanged through M157 registration commits, which touched
  only plans, TOMLs, and containment tests).
- Incoming M095 matrix: `336 apply / 29 blocked_primitive / 475
  not_applicable` across 840 TunnelManager option/family cells.
- M153 closed complete as the current runtime/security qualification
  authority; M154 closed complete disposition C (M147 path blocked);
  M155 closed complete with zero production; M156 closed complete with zero
  promotions; M146 closed blocked.

Reviewed head:

- M157 work completes on the working tree described in §9. Exact changed
  production paths are the ten registered paths above (one new file, nine
  exact-owner extensions). Exact guard/planning paths are
  `emissary-cli/tests/m061_containment.rs`,
  `emissary-cli/tests/m062_dependency_containment.rs`,
  `061-containment-boundary.toml`, `062-dependency-containment.toml`, the
  M157 plan, this closure, registry/README/roadmaps/AGENTS/docs updates, and
  the M158 deferred-status note (see §10).

Pinned authority (all accessed read-only; no upstream mutation, contact, or
submission occurred — see read-only attestation):

- Proposal 170 revision `2026-05-20`, status Open, SHA-256
  `f13ae00b886c5e72131bc5d5b138a371148d1faa6899a119a1dacb65a555e7dc`
  (cited from M095/M153/M155; no new Proposal fetch — M157 adds no Proposal
  policy);
- I2P Encrypted LeaseSet specification,
  `https://geti2p.net/en/docs/specs/encryptedleaseset`, read-only fetch
  2026-09-09 (three nested layers, layer-0/1/2 formats, `GENERATE_ALPHA` /
  `BLIND_PRIVKEY` / `BLIND_PUBKEY` formulas, credential/subcredential,
  `HKDF(salt, ikm, info, n)` schedules, `ELS2_L1K` / `ELS2_L2K` labels,
  ChaCha20 counter-1 `ENCRYPT`/`DECRYPT`, `SHA-256(sigtype || blinded key)`
  storage key, UTC-day rollover, no-auth flag `0x00`, inner type `3`);
- I2P Red25519 specification (cited from M156 records; no new fetch — M156
  remains the signing authority and `crypto/red25519.rs` is unchanged);
- I2P common-structures specification (key-certificate codes 0–11;
  Destination layout; cited from M155 records; no new fetch);
- Direct Proposal-170 PR `ServiceTunnelCreator` source (cited from pinned
  M155/M157 records; no new fetch): `i2cp.encryptLeaseSet=true` only for
  legacy AES, modern blinded/PSK/DH modes select `i2cp.leaseSetType=5`;
- Java I2PControl Proposal-170 head `45bb593000408071dd376b78848fdc246dccd964`
  and Java I2P snapshot `2c3fd2a9532cd86ec06cb6f2b9f3f813ca752243` (cited
  from pinned M155 records; no new fetch);
- Yosemite optional exact revision `59140a2277bf296928d2e8ce39a148182eeff044`
  (unchanged; no Yosemite change in M157);
- Emissary source at the reviewed tree (all file:line citations below read
  at HEAD plus the workdir diff closed here).

Current Proposal matrix at closure (mechanically recomputed, dispositions
unchanged):

- `336 apply / 29 blocked_primitive / 475 not_applicable` (840 cells);
- residual split `SigType` 10, `EncryptLeaseSet` 5, `OptionalLookup` 5,
  `LeaseSetClientAuths` 5, `UseOutproxyPlugin` 4 — identical to M153/M154/M155/M156
  entry.

## 1. Requirement-to-evidence matrix (plan requirements)

| Plan requirement | Evidence |
|---|---|
| credential/subcredential derivation | `credential` (`A \|\| 0x0007 \|\| 0x000b` under `SHA-256("credential" \|\| keydata)`), `subcredential` (`SHA-256("subcredential" \|\| credential \|\| A')`); proven by independent SHA-256 recomputation KAT |
| exact 44-byte schedules | `hkdf_sha256_44` RFC-5869 extract/expand from existing `hmac`/`sha2`, labels `ELS2_L1K`/`ELS2_L2K`, input `subcredential \|\| published_BE`; proven by pinned KAT and layer-independence test |
| no-auth layer encryption/decryption | `encrypt_inner_with_salts` (`0x03 \|\| LS2`), `encrypt_outer_with_salts` (`0x00 \|\| innerCT`), salt-prefixed ChaCha20 counter-1 via existing `ChaCha::with_iv`; deterministic-salt round-trip plus wrong-published/tamper/flag/type rejections |
| secure salts via caller RNG | fresh 32-byte salts from `RngCore + CryptoRng` in `encrypt_inner`/`encrypt_outer`/`encrypt_no_auth`; deterministic `*_with_salts` variants reserved for tests |
| UTC day conversion without new dep | `day_string_from_epoch_secs` (pure integer Gregorian) + `next_day_boundary_secs`; proven by epoch/leap-day/year-boundary KAT |
| zeroizing type-7 seed handoff | `SigningSeed` (`Zeroizing`, no `Debug`); `blinded_day_material` composes M156 `generate_alpha`/`blind_public_key`/`blind_private_key_ed25519`/`blinded_storage_key` with agreement check; no secret logging |
| no PSK/DH/secret/B32/registry | none added; successor-only inputs fail before activation at the parser gate |
| type-5 outer framing + Red25519 verify | `EncryptedLeaseSet2::parse_frame`/`parse`/`build`/`serialize`: sigtype exactly 11, flags exactly 0, bounded lengths, `RedSignature` canonicality, `0x05 \|\| prefix` verify via M156; ordinary LS2 bytes untouched |
| DatabaseStore type 5 | `DatabaseStorePayload::EncryptedLeaseSet2 { outer, raw }` + `DatabaseStoreKind::EncryptedLeaseSet2`; builder emits store byte exactly 5; raw preserved; type-3 semantics unchanged |
| NetDB type preservation | `CachedLeaseSet { raw, expires, encrypted }`; encrypted store path uses outer expiry without decrypting; lookup/flood rebuild the same kind with exact bytes; ordinary active queries never complete from type-5 payloads (re-insert + warn) |
| publication mode + rollover | `EncryptedPublicationConfig` + `LeaseSetManager::enable_encrypted_publication`/`refresh_encrypted_outer`/`handle_rollover`; current inner remains truth source; blinded key switch is atomic with outer/day/deadline; verification state cleared per rotation; stale-key replies ignored; failures retry without type-3 fallback; one owner-local timer only |
| Destination composition | `Destination::enable_encrypted_publication` bridge; `SessionManager` stays on ordinary inner; direct type-5 DSM routes with type/key match; `publish_lease_set` lifecycle preserved |
| SAM activation gates | `is_type5_requested`/`is_valid_type5_no_auth`/`validate_lease_set_type_options` in parser (fail before allocation); session re-validates and constructs the narrow seed handoff only for the valid subset; legacy flag never aliased |
| no-std | `cargo check -p emissary-core --no-default-features --features no_std` passes; new code uses `alloc`/`core` plus existing `hmac`/`sha2`/`chacha20`/`rand`/`zeroize`/`bytes`/`nom` only |
| mapping correction §3 | `i2cp.leaseSetType=5` is the sole modern selector; `i2cp.encryptLeaseSet=true` rejects type-5 activation; proven by parser accept/reject tests; carried forward for M162 |

## 2. Format conformance (plan §2 + spec)

| Item | Implementation | Source |
|---|---|---|
| Layer 0 | sigtype `11` BE, blinded key 32B, published `u32` BE, expires offset `u16` BE, flags `0x0000`, `lenOuterCiphertext` `u16` BE, outer ciphertext, 64B signature over `0x05 \|\| prefix` | ELS Format/layer-0 |
| Layer 1 no-auth | flags `0x00`, no DH/PSK section, remainder is inner ciphertext | ELS layer-1 |
| Layer 2 | type `0x03` + complete signed ordinary LS2 bytes from the canonical owner | ELS layer-2 |
| Credential | `SHA-256("credential" \|\| A \|\| 0x0007 \|\| 0x000b)` | ELS subcredentials |
| Subcredential | `SHA-256("subcredential" \|\| credential \|\| A')` | ELS subcredentials |
| Layer keys | `HKDF(salt32, subcredential \|\| published_BE, "ELS2_L1K"/"ELS2_L2K", 44)`, key `[0..32]`, IV `[32..44]` | ELS layer-1/2 encryption |
| Cipher | `salt32 \|\| ChaCha20(counter=1, key, IV, plaintext)` both layers | ELS `STREAM`/`ENCRYPT` |
| Blinding/storage | M156 `GENERATE_ALPHA` (empty secret), `BLIND_PUBKEY`/`BLIND_PRIVKEY`, `SHA-256(0x000b \|\| A')`, daily rotation | ELS blinding + M156 |
| Rollover | day from `Runtime::time_since_epoch`, next boundary deterministic, single rotation per boundary, republish still-valid inner, never stale, never mixed | ELS daily rule + plan §7 |
| Offline keys | flags nonzero rejected; transient-key path out of scope | ELS layer-0 flags |
| End-to-end | floodfill publication encrypted; established garlic/session wrapping retains ordinary inner | ELS Notes + plan §4.5 |

Vector provenance: fixed type-7 seed/public pair (Red25519 spec vector-1
`edsk`/`edpk`), fixed UTC day, fixed inner payload, fixed inner/outer salts.
Credential/subcredential recomputed independently with direct `sha2`
calls; layer schedules pinned against direct HKDF recomputation;
ciphertext pinned byte-exact under fixed salts; randomized Red25519 outer
signatures proven by verify round-trip plus wrong-key/tamper rejection
(per the randomized construction, as in M156). Day conversion pinned
against calendar KAT including leap-day and year boundary.

## 3. Dependency/security review (M155 §6 freeze)

- No new dependency. Reused: M156 `curve25519-dalek`/`red25519.rs`
  (unchanged), `hmac 0.12.1` + `sha2` (HKDF-SHA256), `chacha20` via
  `crypto::chachapoly::ChaCha::with_iv` (counter-1 seek, unchanged file),
  `rand`/`CryptoRng` (salts + randomized signing), `zeroize` (seed, blinded
  private, HKDF chaining, plaintext scrub on rejection), `bytes`/`nom`
  (existing wire owners).
- `Cargo.toml`, `emissary-core/Cargo.toml`, `Cargo.lock` unchanged in this
  commit (verified by `git diff --name-only`; M062 records the zero budget).
- No Yosemite change; no `emissary-cli/src/i2pcontrol/**` production change.
- `no_std + alloc` compatible; proven by the `no_std` check above.
- Constant-time/zeroization: Dalek ops via M156; HKDF chaining zeroized;
  `SigningSeed`/`BlindedPrivateKey` never `Debug`/logged; salts are public
  wire values; derived keys/IVs are stack-local and zeroized where retained
  (`okm`); decrypt rejects scrub plaintext on flag/type mismatch.
- Bounds before crypto: inner payload cap, outer ciphertext min/max,
  `RedSignature` canonicality, blinded-key torsion/identity checks, message
  length via M156 ceiling, length-validated before allocation.
- Domain separation: exact labels (`credential`, `subcredential`,
  `ELS2_L1K`, `ELS2_L2K`, `I2PGenerateAlpha`, `i2pblinding1`,
  `I2P_Red25519H(x)`, sigtype codes, `YYYYMMDD`, `published_BE`).
- Fail-closed: malformed lengths/flags/sigtypes/signatures/trailing data,
  expired outers (owner-enforced), wrong type/key verification replies,
  stale-key acks, successor companions, unsupported type selectors, and
  crypto/build failures (no type-3 fallback) all reject before network or
  storage effect.
- Ordinary Ed25519/LS2/DatabaseStore-type-3/NetDB-ordinary/M135/M145
  behavior preserved (see §5).

## 4. Exact-path reconciliation (M061/M062)

M061 (`061-containment-boundary.toml` + `m061_containment.rs`):

- `[allowed]` `core_owner_hooks` gains exactly `crypto/els2.rs`,
  `primitives/lease_set.rs`, `primitives/mod.rs`,
  `i2np/database/store.rs`, `netdb/mod.rs` (the five pending paths).
- Ten new `[[evidence]]` blocks: five new-path owners plus five M157
  purpose/reference extensions for `crypto/mod.rs`,
  `destination/lease_set.rs`, `destination/mod.rs`, `sam/parser.rs`,
  `sam/session.rs` (existing paths remain exact owners).
- `[registered_pending]` removed: no successor is registered after closure.
- `m061_containment.rs` pending test returns early when no milestone is
  registered (M158 stays deferred pending its amendment); the
  high-sensitivity exception helper retains the exact five-file enumeration
  (`crypto/mod.rs`, `crypto/red25519.rs`, `crypto/els2.rs`,
  `i2np/database/store.rs`, `netdb/mod.rs`) with no prefix/glob.
- `policy_terms_do_not_leak_into_non_policy_production_paths` passes: new
  core source contains no `Proposal 170`/`JsonRpc`/`jsonrpc`/
  `TunnelManagerControl`/`control-state.json` strings.

M062 (`062-dependency-containment.toml` + `m062_dependency_containment.rs`):

- Header comment updated: M155/M156/M157 closed, no registered successor,
  M158-M162/M152 deferred, zero-budget statement retained.
- `[registered_pending]` removed (zero-budget realized: no manifest,
  lockfile, Yosemite, or I2PControl-source change in this commit).
- No new `[[evidence]]` crate entry (zero promotions/dependencies).
- New `is_authorized_m157_path` covering the exact ten production paths plus
  the M157 guard/TOML/closure/plan/registry/README/roadmap/AGENTS/docs set,
  wired into both the allowed-path and prohibited-pattern assert chains.
- `m061_source_boundary_files_remain_unchanged` updated to the exact
  five-file sensitive-core exception (repairs a pre-existing name drift
  where the test sought `is_authorized_m156_crypto_exception` while the
  guard defines `is_authorized_sensitive_core_exception`).

## 5. Verification outcomes

| Command group | Result |
|---|---|
| `cargo check -p emissary-core` | **pass** |
| `cargo check -p emissary-core --no-default-features --features no_std` | **pass** |
| `cargo test -p emissary-core --lib --no-fail-fast` | **pass**: `1112 passed, 2 ignored` (1094 pre-existing + 18 new M157 tests, zero regressions) |
| `cargo check -p emissary-cli --no-default-features --features i2pcontrol` | **pass** |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --lib --no-fail-fast` | **pass**: `821 passed` |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment --test m062_dependency_containment --test m095_full_support_matrix --test m105_residual_option_audit --no-fail-fast` | **pass**: `8 + 23 + 3 + 1` across 4 suites |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m153_post_m146_requalification` | pre-existing failure only (`AGENTS.md must retain M139`; fails identically at the registration baseline — see §11) |
| M095 mechanical recomputation (`python3` TOML parse over `tunnel_manager.options[*].cells`) | **pass**: recomputed `840/336/29/475` == declared |
| M145 reply-bundling suite | **pass**: `10 passed` (end-to-end garlic path unchanged) |
| M135 lease-set desired-count tests | **pass**: `6 passed` (publication readiness still real-lease truthful) |
| `cargo clippy -p emissary-core --all-targets -- -D warnings` | **pass**: No issues found |
| `cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings` | pre-existing failures only in unrelated `proxy.rs`/`m144` paths; zero warnings from M157 files (see §11) |
| stable `rustfmt` on M157-touched Rust files | **pass** for all ten production files plus both guard tests (nightly-only repo options warn; `max_width = 100` observed) |
| `git diff --check` (staged) | **pass** |

Focused M157 suites (18 tests): `crypto::els2` (7: KAT, schedules,
round-trip, gates, bounds, UTC, day material); `primitives::lease_set`
encrypted-outer (3: round-trip, malformed rejections, expiry);
`i2np::database::store` type-5 (2: builder/round-trip, malformed + type-3
stability); `netdb` preservation (1: type-5 cache/flood identity);
`destination::lease_set` encrypted (4: blinded-key/type-5 use, verification
gating, rollover/stale-key, no-fallback); `sam::parser` type-5 gates (1:
accept + nine rejection families).

## 6. Changed paths (exact budget only)

Production (10):

- `emissary-core/src/crypto/els2.rs` (new: neutral helper + 7 tests);
- `emissary-core/src/crypto/mod.rs` (declaration only);
- `emissary-core/src/primitives/lease_set.rs` (outer type + 3 tests);
- `emissary-core/src/primitives/mod.rs` (single-type re-export);
- `emissary-core/src/i2np/database/store.rs` (type-5 payload/kind + 2 tests);
- `emissary-core/src/netdb/mod.rs` (type-preserving cache/forwarding + 1 test
  and two exact test-seam updates);
- `emissary-core/src/destination/lease_set.rs` (mode/rollover owner + 4 tests
  and five exact test-seam call updates);
- `emissary-core/src/destination/mod.rs` (narrow bridge + type-5 reply route);
- `emissary-core/src/sam/parser.rs` (pre-allocation gates + 1 test);
- `emissary-core/src/sam/session.rs` (narrow seed handoff).

Guards/ledgers (4):

- `emissary-cli/tests/m061_containment.rs` (pending-absent handling);
- `emissary-cli/tests/m062_dependency_containment.rs` (M157 helper in both
  chains + boundary-exception reconciliation);
- `plans/implementation/i2pcontrol-proposal-170/061-containment-boundary.toml`
  (five newly allowed paths + ten evidence blocks, pending removed);
- `plans/implementation/i2pcontrol-proposal-170/062-dependency-containment.toml`
  (closure bookkeeping, pending removed, zero-budget recorded).

Planning/closure/registry (7 + 1 note):

- `plans/implementation/i2pcontrol-proposal-170/157-modern-encrypted-leaseset2-publication-primitive.md`
  (Status `registered / dependency-ready` → `closed as complete` with
  closure link);
- new `plans/closure/i2pcontrol-proposal-170/157-closure.md` (this file);
- `plans/registry.md` (M157 → closed; no registered successor; M158
  deferred pending amendment);
- `plans/implementation/i2pcontrol-proposal-170/README.md` (M157 closed
  section; chain updated);
- `plans/subsystems/i2pcontrol-proposal-170-post-m154-leaseset-security-corrective-roadmap.md`
  (M157 closed; graph updated);
- `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md`
  (M157 closed; handoff updated);
- `AGENTS.md` (M157 closed entry; chain updated);
- `plans/implementation/i2pcontrol-proposal-170/158-leaseset-lookup-secret-and-blinded-address-primitive.md`
  (deferred-status note: M157 hard dependency satisfied, amendment pending;
  no registration).

Docs (3, authority wording only):

- `docs/i2pcontrol/README.md`, `docs/i2pcontrol/proposal-170-support.md`,
  `docs/i2pcontrol/tunnel-manager.md` (M157 closed authority + `336/29/475`
  wording retained; LeaseSet residual-blocked statements retained).

`061/062`, `095/105/110` semantics: `061/062` gain only the exact M157
authorization above (no broad waiver); `095/105/110` intentionally
**unchanged** (zero promotions; see §8).

## 7. Requirement-to-evidence matrix (planning process §2.5)

| Closure duty | Evidence |
|---|---|
| implementation commits | one commit landing with this closure (production + tests + M061/M062 guards + planning records); exact paths in §6; no prior partial implementation commit (registration commits touched only plans/TOMLs/tests) |
| invariant review | partial-support, no-fabrication, I2P-only egress, fail-closed option validation, containment, and Y005 invariants re-proved by green `m061`/`m062`/`m095`/`m105` guards; no invariant weakened; M147/M148 not reopened (type-11 derived-only, gates enforced); M146 untouched (no egress); no downgrade path exists (fallible crypto, no type-3 fallback, stale-key rejection) |
| failure/recovery and contention evidence | wrong-type/key verification ignored; stale-key acks ignored; crypto/build failure retries without fallback; expired/invalid inner never published; rollover is single-owner timer/state with no second task; no lock spans I/O or crypto; shutdown drops the owner-local timer; successor generations receive no predecessor key state (config is generation-local); tamper/wrong-published/flag/type/length tests prove closed rejection |
| compatibility, migration, security review | no wire change to type-3/RouterInfo/lookup protocol; type-5 is additive; no storage migration (opaque DHT keys, bounded cache); no new attack surface beyond reviewed Dalek/HMAC/ChaCha ops; all 15 LeaseSet cells stay fail-closed (matrix unchanged); legacy AES still blocked; `cargo clippy -p emissary-core -D warnings` clean; `emissary-cli -D warnings` failures are pre-existing and unrelated with zero M157-file warnings |
| documentation and operational evidence | §6 changed paths; authority docs name M157 closed and retain partial support; operational impact none beyond the new opt-in `i2cp.leaseSetType=5` publication mode (default ordinary path byte-compatible) |
| M157 acceptance (plan closure evidence) | exact changed files/dependencies (§6), KAT/reference table (§2), DatabaseStore/floodfill evidence (§5), publication/verification trace (§5), rollover/stale-key trace (§5), inner/M135/M145 composition (§5), no-downgrade/zeroization review (§3), mapping correction (§1), complete verification (§5), unchanged M095 (§8), implementation SHA (commit landing with this closure), M158 readiness (§10) |

## 8. Promotion accounting (zero)

- `110-completion-ledger.toml`: no new entry (zero promotions).
- `095-full-support-matrix.toml` / `105-residual-option-audit.toml`:
  unchanged.
- Mechanically recomputed `336/29/475` == declared; residual
  `10/5/5/5/4` unchanged.
- Infrastructure alone has zero Proposal support value (registry rule);
  no `EncryptLeaseSet`/`OptionalLookup`/`LeaseSetClientAuths` cell is
  claimed by this primitive.

## 9. Production-head determination

- Last production-bearing commit before M157: `1678790cc74d4075e800425c57312da89e1169fe`
  (M156 primitive; unchanged through M157 registration).
- M157 is production-bearing (exact §6 files); after the closure commit
  lands, the last production-bearing head becomes that commit.
- `git log 1678790..HEAD` over `emissary-core/src`,
  `emissary-cli/src`, `emissary-util/src`, all manifests and `Cargo.lock`
  at closure time contains only the staged M157 exact-budget diff
  (10 production paths; Rust production delta is `els2.rs` new plus nine
  exact-owner extensions plus guard/test exact amendments).
- `git diff --name-only` over manifests/lockfile is empty: no `Cargo.toml`,
  `emissary-core/Cargo.toml`, or `Cargo.lock` change.
- Streamr datagram limits (16-subscriber, 60s expiry, 1200-byte payload,
  4095-byte transport-buffer, 15s refresh, bounded shutdown,
  loopback-only UDP) untouched; remote datagrams never choose a local UDP
  destination.

## 10. Registry updates and future-plan unblock determination

Applied alongside this closure (see §6 for the file list):

- `157-*.md` plan: Status `registered / dependency-ready` → `closed as
  complete` with closure link;
- `plans/registry.md`: M157 → closed as complete (`336/29/475` unchanged,
  zero promotions, exact production diff); **no registered successor**;
- `110-completion-ledger.toml`: no new entry (zero promotions);
- `m061`/`m062`: exact M157 authorization only (§4); no broad waiver.

Future-plan unblock determination (as required by the tasking):

- **M157 CLOSED as complete.** Its sole hard dependency (M156 closure with
  frozen blinding primitive and zero promotions) was satisfied, and all plan
  requirements are met with exact-budget production diff and zero matrix
  promotions. No stop condition in plan §14 triggered: no outside file or
  dependency was required, no second NetDB subsystem was needed, no
  plaintext fallback was used, floodfill preservation fit the existing
  owner, M145 session bundling is untouched per the specification note, and
  reference vectors agree with local behavior.
- **M158 REMAINS deferred/unregistered (hard dependency now satisfied,
  amendment pending).** M157 closure satisfies its hard dependency, but its
  registration gate additionally requires an exact-path amendment freezing
  secret-handoff ownership, extended-B32 codec ownership, secret-store and
  restart semantics, publication/address integration seams, exact
  files/dependencies, and interop vectors/fixtures. That amendment is not
  written by M157 and must be authored at M158 registration. The M158 plan
  file gains only a deferred-status note recording the satisfied hard dep;
  it is **not** registered by this closure. Only M158 may be registered
  next, with exact M061/M062 authorization in that registration commit.
- **M159 remains deferred/unregistered**, hard-gated on M158 closure.
- **M160 remains deferred/unregistered**, hard-gated on M159 closure.
- **M161 remains deferred/unregistered**, hard-gated on M155 closure
  (satisfied) but sequenced after M160 per the roadmap; it may now be
  prepared but must not overtake M157-M160 implementation. Its A/B/C
  outcome gates M162 `EncryptLeaseSet` promotion.
- **M162 remains deferred/unregistered**, hard-gated on M160 closure plus
  M161 disposition (plus any M161-A successor closure before
  `EncryptLeaseSet` promotion). It inherits the §3 mapping correction
  (`i2cp.leaseSetType=5` modern selector; legacy flag never aliased).
- **M152 remains deferred/unregistered**, re-gated on M162 closure plus any
  M161-A successor. Its final qualification must now account for the M155
  field-coupling rule plus the M156/M157 primitive availability.
- **M149-M151 remain superseded/unregistered (do not execute).** Confirmed
  fully superseded by M155-M162; no re-gating or revival is authorized.
- **M147/M148 remain closed-blocked/deferred-behind-M147** (M154
  disposition C); this line does not reopen them. Type-11 blinding is
  explicitly not a `SigType` reopening.
- **M146 remains closed as blocked** with no successor; it is not reopened
  by M157 or by any future LeaseSet tail.
- No other future-plan status required a change. File presence alone never
  authorises production work.

## 11. Unresolved findings

- Non-blocking: `cargo test -p emissary-cli --no-default-features
  --features i2pcontrol --test m153_post_m146_requalification` reports
  `AGENTS.md must retain M139 as historical evidence`. Fails identically at
  the pre-M157 registration baseline; unrelated to M157 files. Severity:
  informational; no corrective pass required (out of M157 exact-budget
  scope; must not be smuggled into this line).
- Non-blocking: `cargo clippy -p emissary-cli --no-default-features
  --features i2pcontrol --all-targets -- -D warnings` reports pre-existing
  lints in `backends/filters/proxy.rs:60` (`chunks_exact`) and `m144`
  initializer-style paths, untouched by M157; M157 files contribute zero
  warnings. Severity: informational; no corrective pass required (out of
  M157 exact-budget scope).
- Non-blocking: `cargo fmt --all -- --check` (stable or nightly) reports
  drift in files outside the M157 budget at the registration baseline;
  M157-touched Rust files are stable-`rustfmt` clean with `max_width = 100`
  observed. Severity: informational; unrelated files were not touched.

## Internal-only / read-only-upstream attestation

- External sources (the Encrypted LeaseSet specification re-fetched
  read-only 2026-09-09; the already-pinned Proposal/Java/Yosemite revisions
  cited from existing M155/M157 records) were accessed read-only for
  evidence;
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
