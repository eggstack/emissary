# M156 Closure — Neutral Red25519 and Ed25519-Blinding Primitive

Status: **closed as complete; M157 hard dependency satisfied, amendment pending**

Date: `2026-09-09`

Source implementation plan:

- `plans/implementation/i2pcontrol-proposal-170/156-neutral-red25519-blinding-primitive.md`
  (Status now `closed as complete` with closure link; hard dependency on
  M155 closure satisfied by `plans/closure/i2pcontrol-proposal-170/155-closure.md`;
  zero promotion budget observed as proven below).

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-post-m154-leaseset-security-corrective-roadmap.md`.

Promotion budget: **zero Proposal cells**.

Production budget: **exact M155 §8 files only with M061/M062 authorization at
registration** (new `emissary-core/src/crypto/red25519.rs`,
`emissary-core/src/crypto/mod.rs` declaration only, direct
`curve25519-dalek` edge plus `Cargo.lock`).

## Planning baseline

- Registration baseline `1355555c4d5c69294c8f3382d06449d3dba578d8` (M155
  closure head; clean worktree before M156 registration edits).
- Incoming M095 matrix: `336 apply / 29 blocked_primitive / 475
  not_applicable` across 840 TunnelManager option/family cells.
- M153 closed complete as the current runtime/security qualification
  authority; M154 closed complete disposition C (M147 path blocked);
  M155 closed complete with zero production; M146 closed blocked.

Reviewed head:

- M156 audit work completes on the working tree described in §9. Exact
  changed production paths are `emissary-core/src/crypto/red25519.rs` (new),
  `emissary-core/src/crypto/mod.rs` (declaration only),
  `emissary-core/Cargo.toml` (direct `curve25519-dalek` edge), and
  `Cargo.lock` (direct-edge only, no version churn). Exact guard/planning
  paths are `emissary-cli/tests/m061_containment.rs`,
  `emissary-cli/tests/m062_dependency_containment.rs`,
  `061-containment-boundary.toml`, `062-dependency-containment.toml`, the
  M156 plan, this closure, registry/README/roadmap/AGENTS/docs updates, and
  the M157 deferred-status note (see §10).

Pinned authority (all accessed read-only; no upstream mutation, contact, or
submission occurred — see read-only attestation):

- Proposal 170 revision `2026-05-20`, status Open, SHA-256
  `f13ae00b886c5e72131bc5d5b138a371148d1faa6899a119a1dacb65a555e7dc`
  (cited from M095/M153/M155; no new Proposal fetch — M156 adds no Proposal
  policy);
- I2P Red25519 specification, `https://geti2p.net/en/docs/specs/red25519`,
  read-only fetch 2026-09-09 (RedDSA instantiation, conversion functions,
  sign/verify equations, ten test vectors; vectors 1–2 ingested as
  known-answer gates);
- I2P Encrypted LeaseSet specification,
  `https://geti2p.net/en/docs/specs/encryptedleaseset`, read-only fetch
  2026-09-09 (`GENERATE_ALPHA`/`BLIND_PRIVKEY`/`BLIND_PUBKEY` formulas,
  `HStar` signing construction, `SHA-256(sigtype || blinded_pubkey)`
  storage key, UTC-day rollover, secret contribution);
- I2P common-structures specification (key-certificate codes 0–11;
  Destination layout; cited from M155 records; no new fetch);
- Java I2PControl Proposal-170 head `45bb593000408071dd376b78848fdc246dccd964`
  and Java I2P snapshot `2c3fd2a9532cd86ec06cb6f2b9f3f813ca752243` (cited
  from pinned M155 records; no new fetch — M156 is neutral crypto, not
  Java interop);
- Yosemite optional exact revision `59140a2277bf296928d2e8ce39a148182eeff044`
  (unchanged; no Yosemite change in M156);
- Emissary source at the reviewed tree (all file:line citations below read
  at HEAD plus the workdir diff closed here).

Current Proposal matrix at closure (mechanically recomputed, dispositions
unchanged):

- `336 apply / 29 blocked_primitive / 475 not_applicable` (840 cells);
- residual split `SigType` 10, `EncryptLeaseSet` 5, `OptionalLookup` 5,
  `LeaseSetClientAuths` 5, `UseOutproxyPlugin` 4 — identical to M153/M154/M155
  entry.

## 1. Requirement-to-evidence matrix (plan requirements)

| Plan requirement | Evidence |
|---|---|
| derive daily `alpha` from unblinded pubkey, sig-type codes, UTC date, optional secret | `generate_alpha` (`red25519.rs`): `stA`/`stA'` BE codes, `keydata = A \|\| stA \|\| stA'`, `datestring` 8B ASCII, UTF-8 secret (empty allowed, >64B rejects), `salt = SHA-256("I2PGenerateAlpha" \|\| keydata)`, `seed = HKDF-SHA256(salt, datestring \|\| secret, "i2pblinding1", 64)`, `alpha = seed mod L` via `from_bytes_mod_order_wide`; sig-type gate (unblinded 7/11, blinded 11 only); day validation; secret UTF-8 validation; proven by determinism/secret-sensitivity/rollover tests |
| blind Ed25519 public key by Edwards-point addition | `blind_public_key`: `A' = A + [alpha]B` via `curve25519-dalek` `EdwardsPoint`; decompress + torsion-free + identity + small-order fail-closed; proven by spec vectors 1–2 `rvk` gates |
| derive matching blinded private scalar from seed/scalar plus `alpha` | `blind_private_key_ed25519` (SHA-512 + clamp + `from_bytes_mod_order` + mod-L add) and `blind_private_key_red25519` (canonical check + mod-L add); proven by spec vectors 1–2 `rsk` gates and agreement test |
| Red25519 sign with random nonce per I2P construction | `sign` (secure RNG, 80-byte `T`) + `sign_with_transcript` (test-only deterministic): `r = HStar(T, vkBytes, m)`, `R = [r]B`, `c = HStar(Rbytes, vkBytes, m)`, `S = (r + c*sk) mod L`; `MAX_MESSAGE_LEN` 65534 enforced; proven by roundtrip + randomization tests |
| verify Red25519 signatures | `verify`: `R` decompress, `S` canonical, `vk` torsion-free/non-identity, cofactor equation `((-[S]B) + R + ([c]vk)).mul_by_cofactor().is_identity()`; proven by roundtrip, wrong-key/message/tamper rejections |
| derive blinded storage-key preimage/hash inputs for M157 | `blinded_storage_key_preimage` (`0x000b BE \|\| pubkey`, 34B) + `blinded_storage_key` (`SHA-256(preimage)`); proven by preimage/hash test and rollover key-change test |
| exact 32/64-byte typed values, zeroization, no `Debug` on secrets | `Alpha`/`BlindedPrivateKey` (secret, `Zeroizing`, no `Debug`), `BlindedPublicKey`/`RedSignature` (public, `Debug` ok); HKDF PRK/OKM/IKM/seed zeroized; `sign` transcript is stack-local; proven by type inspection + `cargo clippy` clean |
| no generic runtime algorithm registry | none added; API is four narrow types plus six pure functions; `Destination::new`, SAM `DEST GENERATE`, secret-store envelope untouched |
| UTC-day boundary deterministic + vectors | `format_day_string` + `validate_day_string` + `generate_alpha` day input; day-rollover test proves new alpha/blinded/store key each UTC day |
| ordinary Ed25519 unchanged | `SigningPrivateKey`/`SigningPublicKey` untouched; regression test proves `ed25519-dalek` generate/sign/`verify_strict` still works |
| known-answer/cross-reference vectors | spec vectors 1–2 ingested (`edsk`/`edpk`/`sk`/`vk`/`alpha`/`rsk`/`rvk` + `msg` roundtrip); alpha with/without secret; agreement; sign/verify; wrong-key/message; rollover; malformed rejection; no-std; zero regression (§9) |
| no-std compilation | `cargo check -p emissary-core --no-default-features --features no_std` passes; `curve25519-dalek` is `#![no_std]` + `alloc`; new module uses `alloc`-free fixed buffers plus `zeroize/alloc` only |

## 2. Formula conformance (M155 §3.1 refreeze)

| Item | Implementation | Source |
|---|---|---|
| Curve/order | `B` = Ed25519 basepoint via `EdwardsPoint::mul_base`; `L` via `Scalar` mod-L ops | ELS/Red25519 Definitions |
| Daily alpha | `stA`/`stA'` BE, `keydata`, `datestring`, UTF-8 `secret`, `HKDF(H("I2PGenerateAlpha",keydata), datestring \|\| secret, "i2pblinding1", 64)`, `alpha = seed mod L` (64B LE) | ELS `GENERATE_ALPHA` |
| Scalar conversion | type 7: `clamp(SHA-512(seed)[..32])` then `from_bytes_mod_order` (handles `>= L` clamped range); type 11: `from_canonical_bytes` strict | ELS `BLIND_PRIVKEY`; Red25519 `CONVERT_ED25519_PRIVATE` |
| Blinded public | `A + [alpha]B` Edwards addition | ELS `BLIND_PUBKEY`; Red25519 `RANDOMIZE_PUBLIC` |
| Blinded private | `(a + alpha) mod L` via `Scalar` addition | ELS `BLIND_PRIVKEY`; Red25519 `RANDOMIZE_PRIVATE` |
| Agreement | `blind_public_key == derive_public(blind_private_key)` proven by test | ELS/Red25519 design |
| Randomized signing | `T` 80 rand bytes; `HStar` = `SHA-512("I2P_Red25519H(x)" \|\| p1 \|\| p2 \|\| len_u16(m) \|\| m) mod L`; `R=[r]B`; `c=HStar(Rbytes,vkBytes,m)`; `S=(r+c*sk) mod L` | ELS Sign/Verify; Red25519 `SIGN` |
| Verification | cofactor equation with `S < L`, `R` valid point | Red25519 `VERIFY` |
| Storage key | `SHA-256(0x000b BE \|\| A')`, rotated daily | ELS Format |
| Rollover | new alpha/blinded/store key each UTC day; no mixed material (primitive is pure; publication rollover is M157 owner) | ELS daily rule |
| Secret | UTF-8 `secret` feeds `datestring \|\| secret`; wrong/missing secret yields different alpha, never falls back (pure function; no fallback path exists) | ELS `GENERATE_ALPHA` |
| Non-reopening | blinded type fixed 11; unblinded gate 7/11 only; `Destination::new`, SAM `"7"` gate, envelope v1 untouched; no DSA/ECDSA added | M154 disposition C |

Vector provenance: Red25519 spec test vectors 1–10 (each with
`edsk`/`edpk`/`sk`/`vk`/`msg`/`sig`/`alpha`/`rsk`/`rvk`/`rsig`); M156
ingests vectors 1–2 as known-answer gates for conversion (`sk`), blinding
(`rsk`/`rvk`), and agreement, plus `msg` roundtrip signing. `sig`/`rsig`
reference signatures are randomized (every Red25519 signature differs), so
they are not byte-compared; instead sign/verify roundtrip plus wrong-key
rejection proves the construction. Alpha/store-key vectors are derived
deterministically from the frozen `GENERATE_ALPHA`/storage formulas and
proven by determinism/secret-sensitivity/rollover tests.

## 3. Dependency/security review (M155 §6 freeze)

- Direct `curve25519-dalek 5.0.0-pre.6`, `default-features = false`,
  `features = ["alloc", "precomputed-tables", "zeroize"]` in
  `emissary-core/Cargo.toml`; `Cargo.lock` gains only the direct edge to
  `emissary-core` (no new version; transitive `5.0.0-pre.6` already pinned
  via `ed25519-dalek`/`x25519-dalek`).
- Why existing deps insufficient: `ed25519-dalek 3.0.0-pre.6` exposes only
  `SigningKey/VerifyingKey/sign/verify_strict`; `x25519-dalek 3.0.0-pre.6`
  exposes only X25519 DH; neither exposes Edwards-point addition, scalar
  mod-L addition/reduction, arbitrary-basepoint multiplication, or
  randomized Red25519 signing.
- No bespoke curve/bignum: all arithmetic via maintained Dalek
  constant-time `Scalar`/`EdwardsPoint` (`from_bytes_mod_order`,
  `from_bytes_mod_order_wide`, `from_canonical_bytes`, `mul_base`,
  point addition, `mul_by_cofactor`, `is_torsion_free`).
- `no_std + alloc` compatible: `curve25519-dalek` is `#![no_std]` with
  `alloc`; new module uses fixed `[u8;N]` buffers, `sha2`/`hmac` (existing
  `default-features = false`), `zeroize/alloc` `Zeroizing`; proven by
  `cargo check -p emissary-core --no-default-features --features no_std`.
- HKDF: RFC-5869 extract/expand composed from existing `hmac 0.12.1` +
  `sha2 0.10.9`; no `hkdf` crate added (verified absent from `Cargo.lock`
  before and after; only `curve25519-dalek` edge added).
- Constant-time/zeroization: `Scalar`/`EdwardsPoint` Dalek constant-time
  ops; `subtle` not needed beyond Dalek internals; private
  scalars/alpha/seeds/PRK/OKM/IKM wrapped in `Zeroizing` or explicitly
  zeroized; `Alpha`/`BlindedPrivateKey` never `Debug`/logged; length
  validated before allocation (`MAX_MESSAGE_LEN`, 32/64-byte types,
  64-byte secret cap, 8-byte day).
- Domain separation: every derivation includes exact I2P domains
  (`I2PGenerateAlpha`, `i2pblinding1`, `I2P_Red25519H(x)`, `stA`/`stA'`,
  `YYYYMMDD`, `len_u16`).
- Malformed fail-closed: non-canonical scalars/points, small-order/
  non-torsion-free/identity keys, bad sig-types/days/secrets, overlong
  messages all return `Error::InvalidData` before any network or storage
  effect (primitive is pure; no allocation path exists to weaken).
- Production RNG: `sign` requires `RngCore + CryptoRng`; deterministic
  `sign_with_transcript` is documented test-only.
- Ordinary Ed25519 byte-compatibility preserved (no `crypto/mod.rs`
  signing-path change; declaration-only plus new module).
- `redjubjub` explicitly not used (Zcash RedJubjub, not I2P RedDSA;
  M154/M155 inherited rejection). No `p384`/`p521`/`dsa` additions.

## 4. Exact-path reconciliation (M061/M062)

M061 (`061-containment-boundary.toml` + `m061_containment.rs`):

- `core_owner_hooks` gains exactly `emissary-core/src/crypto/mod.rs` and
  `emissary-core/src/crypto/red25519.rs` with owner/purpose/consumer/
  sensitivity/seam/reference evidence (neutral Red25519/blinding owner;
  declaration-only seam).
- `prohibited.production_prefixes` retains `emissary-core/src/crypto/`
  (no broad waiver); `m061_containment.rs` gains narrow
  `is_authorized_m156_crypto_exception` permitting only those two exact
  files in `high_sensitivity_core_paths_are_individually_named`.
- `policy_terms_do_not_leak_into_non_policy_production_paths` passes:
  new crypto source contains no `Proposal 170`/`JsonRpc`/`jsonrpc`/
  `TunnelManagerControl`/`control-state.json` strings.

M062 (`062-dependency-containment.toml` + `m062_dependency_containment.rs`):

- Header comment updated: M155 closed, M156 sole registered with exact-file
  budget, M157-M162/M152 deferred, no broad prefix allowance.
- New `[[evidence]]` for `curve25519-dalek` (core-owned unconditional,
  exact version/features, transitive-impact none, `no_std` posture, why
  existing Dalek wrappers insufficient).
- New `is_authorized_m156_path` covering exactly `emissary-core/src/crypto/
  red25519.rs`, `emissary-core/src/crypto/mod.rs`,
  `emissary-core/Cargo.toml`, `Cargo.lock`,
  `061/062-containment TOMLs`, both containment tests, `156-closure.md`,
  M156/M157 plans, registry/README/roadmaps, AGENTS, and docs support
  files; wired into both the allowed-path and prohibited-pattern assert
  chains.
- `m061_source_boundary_files_remain_unchanged` updated to permit the exact
  M156 `m061_containment.rs` exception while asserting the guard still
  records only the two exact crypto files and retains broad-prefix
  prohibition.

## 5. Verification outcomes

| Command group | Result |
|---|---|
| `cargo check -p emissary-core` | **pass** |
| `cargo check -p emissary-core --no-default-features --features no_std` | **pass** |
| `cargo test -p emissary-core --lib --no-fail-fast` | **pass**: `1094 passed, 2 ignored` (1080 pre-existing + 14 new `crypto::red25519` tests, zero regressions) |
| `cargo check -p emissary-cli --no-default-features --features i2pcontrol` | **pass** |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment --test m062_dependency_containment --test m095_full_support_matrix --test m105_residual_option_audit --test m153_post_m146_requalification --no-fail-fast` | **pass**: `37 passed (5 suites, 0 failed)` |
| M095 mechanical recomputation (`python3` TOML parse over `tunnel_manager.options[*].cells`) | **pass**: recomputed `840/336/29/475` == declared; residual split `SigType 10 blocked (+2 N/A-shaped) / Encrypt 5 / Optional 5 / Auth 5 / Outproxy 4` |
| `cargo clippy -p emissary-core --all-targets -- -D warnings` | **pass**: No issues found |
| `cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings` | pre-existing failures only in unrelated `proxy.rs`/`m144` paths; zero warnings from M156 files (see §7) |
| `git diff --check` (staged + unstaged) | **pass** |
| `rustfmt --check` on M156-touched Rust files | **pass** for `red25519.rs`, `m061/m062` guards (stable `rustfmt`; repo nightly-only options warn but `max_width = 100` observed) |

Focused `crypto::red25519` suite (14 tests): conversion vector 1; `rsk`
vectors 1–2; `rvk` vectors 1–2; agreement; sign/verify roundtrip;
randomization difference + dual verify; wrong-key/message/tamper rejection;
alpha determinism + secret sensitivity + day change; rollover blinded/store
key change; bad sig-type/day/secret rejection; storage preimage/hash;
malformed point/scalar/identity/signature-length/message-length rejection;
ordinary Ed25519 regression.

## 6. Changed paths (exact budget only)

- `emissary-core/src/crypto/red25519.rs` (new, 789 lines: neutral
  primitive + 14 focused tests);
- `emissary-core/src/crypto/mod.rs` (one-line `pub mod red25519;`
  declaration only);
- `emissary-core/Cargo.toml` (one-line direct `curve25519-dalek`
  `5.0.0-pre.6` edge);
- `Cargo.lock` (one-line direct-edge addition under `emissary-core`;
  no version churn);
- `emissary-cli/tests/m061_containment.rs` (narrow exact-file exception
  helper);
- `emissary-cli/tests/m062_dependency_containment.rs` (M156 helper in both
  chains + M061-boundary reconciliation for the exact test amendment);
- `plans/implementation/i2pcontrol-proposal-170/061-containment-boundary.toml`
  (two allowed paths + two evidence blocks);
- `plans/implementation/i2pcontrol-proposal-170/062-dependency-containment.toml`
  (M156 registration comment + `curve25519-dalek` evidence);
- `plans/implementation/i2pcontrol-proposal-170/156-neutral-red25519-blinding-primitive.md`
  (Status `registered / dependency-ready` → `closed as complete` with
  closure link);
- `plans/implementation/i2pcontrol-proposal-170/157-modern-encrypted-leaseset2-publication-primitive.md`
  (deferred-status note: M156 hard dependency satisfied, amendment pending;
  no registration);
- new `plans/closure/i2pcontrol-proposal-170/156-closure.md` (this file);
- `plans/registry.md` (M156 → closed; no registered successor; M157
  deferred with satisfied hard dep noted);
- `plans/implementation/i2pcontrol-proposal-170/README.md` (M156 closed
  section; chain updated);
- `plans/subsystems/i2pcontrol-proposal-170-post-m154-leaseset-security-corrective-roadmap.md`
  (M156 closed; graph updated);
- `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md`
  (M156 closed; chain/handoff updated);
- `AGENTS.md` (M156 closed entry; deferred line updated; broad-prefix rule
  retained);
- `docs/i2pcontrol/README.md`, `docs/i2pcontrol/proposal-170-support.md`,
  `docs/i2pcontrol/tunnel-manager.md` (M156 closed authority + `336/29/475`
  wording retained; LeaseSet residual-blocked statements retained).

`061-containment-boundary.toml`, `062-dependency-containment.toml`,
`095-full-support-matrix.toml`, `105-residual-option-audit.toml`, and
`110-completion-ledger.toml` semantics: `061/062` gain only the exact M156
authorization above (no broad waiver); `095/105/110` intentionally
**unchanged** (zero promotions; see §8).

## 7. Requirement-to-evidence matrix (planning process §2.5)

| Closure duty | Evidence |
|---|---|
| implementation commits | one commit landing with this closure (production + tests + M061/M062 guards + planning records); exact paths in §6; no prior partial implementation commit |
| invariant review | partial-support, no-fabrication, I2P-only egress, fail-closed option validation, containment, and Y005 invariants re-proved by green `m061`/`m062`/`m095`/`m105`/`m153` guards; no invariant weakened; M147/M148 not reopened (type-11 derived-only, gates enforced); M146 untouched (no egress); no downgrade path exists (pure fallible crypto, no publication fallback) |
| failure/recovery and contention evidence | pure fallible functions: every malformed input returns `InvalidData` before allocation-heavy work; no runtime tasks, locks, channels, or storage to contend; overlong messages fail before hashing; secret/alpha zeroized; wrong-key/message/tamper tests prove closed rejection; ordinary Ed25519 regression proves no collateral failure |
| compatibility, migration, security review | no wire/protocol/storage/dependency-version change ⇒ no migration impact; no new attack surface beyond reviewed Dalek ops; all 15 LeaseSet cells stay fail-closed (matrix unchanged); legacy AES still blocked (C); `cargo clippy -p emissary-core -D warnings` clean; `emissary-cli -D warnings` failures are pre-existing in `backends/filters/proxy.rs:60` (`chunks_exact`) and `m144` initializer-style lints, untouched by M156 and proven unrelated by zero M156-file warnings |
| documentation and operational evidence | §6 changed paths; authority docs name M156 closed and retain partial support; operational impact none (library primitive, no runtime wiring) |
| M156 acceptance (plan closure evidence) | exact changed files/dependencies (§6), vector provenance/results (§2), no-std/security review (§3), M061/M062 reconciliation (§4), zero matrix delta (§5), implementation SHA (commit landing with this closure), M157 readiness (§8) |

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

- Last production-bearing commit before M156: `7cbd80a6d72aa07d158ba9dc74f8bbacef767be2`
  (M145 no-std/format follow-up; unchanged through M153/M154/M155
  planning closures).
- M156 is production-bearing (exact §6 files); after the closure commit
  lands, the last production-bearing head becomes that commit.
- `git log 7cbd80a..HEAD` over `emissary-core/src`,
  `emissary-cli/src`, `emissary-util/src`, all manifests and `Cargo.lock`
  at closure time contains only the staged M156 exact-budget diff
  (8 paths, `889 insertions, 10 deletions` including tests/plans; Rust
  production delta is `red25519.rs` new + `mod.rs`/`Cargo.toml`/`Cargo.lock`
  one-liners plus guard/test exact amendments).
- Streamr datagram limits (16-subscriber, 60s expiry, 1200-byte payload,
  4095-byte transport buffer, 15s refresh, bounded shutdown,
  loopback-only UDP) untouched; remote datagrams never choose a local UDP
  destination.

## 10. Registry updates and future-plan unblock determination

Applied alongside this closure (see §6 for the file list):

- `156-*.md` plan: Status `registered / dependency-ready` → `closed as
  complete` with closure link;
- `plans/registry.md`: M156 → closed as complete (`336/29/475` unchanged,
  zero promotions, exact production diff); **no registered successor**
  (M157 remains deferred pending its exact-path amendment — see below);
- `110-completion-ledger.toml`: no new entry (zero promotions);
- `m061`/`m062`: exact M156 authorization only (§4); no broad waiver.

Future-plan unblock determination (as required by the tasking):

- **M156 CLOSED as complete.** Its sole hard dependency (M155 closure with
  frozen formulas/vectors, exact files/APIs, dependency/no-std/security
  review, and type-7-sufficiency confirmation) was satisfied, and all plan
  requirements are met with exact-budget production diff and zero matrix
  promotions. No stop condition in plan §Stop triggered: maintained
  constant-time dependency exists (`curve25519-dalek 5.0.0-pre.6` direct
  edge), vectors agree with spec behavior (vectors 1–2 `sk`/`rsk`/`rvk`
  gates pass; randomized `sig`/`rsig` proven via roundtrip, not byte
  comparison, per the construction), and implementation required no
  persistent `SigType` generalization.
- **M157 REMAINS deferred/unregistered (hard dependency now satisfied,
  amendment pending).** M156 closure satisfies its hard dependency, but its
  registration gate additionally requires an exact-path amendment freezing
  layer formats, DatabaseStore type-5 representation, blinded DHT/storage
  key wiring, publication/storage-verification owner, UTC-day
  rollover/republish owner, exact files/dependencies, and interop
  vectors/fixtures. That amendment is not written by M156 and must be
  authored at M157 registration. M157 plan file gains only a
  deferred-status note recording the satisfied hard dep; it is **not**
  registered by this closure. Only M157 may be registered next, with exact
  M061/M062 authorization in that registration commit.
- **M158 remains deferred/unregistered**, hard-gated on M157 closure. The
  conditional M158 `OptionalLookup` promotion remains **unauthorized**
  until M155's full-contract proof gate is met at M158 closure (default is
  defer to M162).
- **M159 remains deferred/unregistered**, hard-gated on M158 closure.
- **M160 remains deferred/unregistered**, hard-gated on M159 closure.
- **M161 remains deferred/unregistered**, hard-gated on M155 closure
  (satisfied) but sequenced after M160 per the roadmap; it may now be
  prepared but must not overtake M156-M160 implementation. Its A/B/C
  outcome gates M162 `EncryptLeaseSet` promotion.
- **M162 remains deferred/unregistered**, hard-gated on M160 closure plus
  M161 disposition (plus any M161-A successor closure before
  `EncryptLeaseSet` promotion).
- **M152 remains deferred/unregistered**, re-gated on M162 closure plus any
  M161-A successor. Its final qualification must now account for the M155
  field-coupling rule plus the M156 primitive availability.
- **M149-M151 remain superseded/unregistered (do not execute).** Confirmed
  fully superseded by M155-M162; no re-gating or revival is authorized.
- **M147/M148 remain closed-blocked/deferred-behind-M147** (M154
  disposition C); this line does not reopen them. Type-11 blinding is
  explicitly not a `SigType` reopening (§2).
- **M146 remains closed as blocked** with no successor; it is not reopened
  by M156 or by any future LeaseSet tail.
- No other future-plan status required a change. File presence alone never
  authorises production work.

## 11. Unresolved findings

- None blocking. Non-blocking note: `cargo clippy -p emissary-cli
  --no-default-features --features i2pcontrol --all-targets -- -D warnings`
  reports pre-existing lints in `backends/filters/proxy.rs:60` and
  `m144_presentation_usessl.rs:202-203`, untouched by M156; M156 files
  contribute zero warnings. Severity: informational; no corrective pass
  required (out of M156 exact-budget scope; must not be smuggled into this
  line).

## Internal-only / read-only-upstream attestation

- External sources (the Red25519 and EncryptedLeaseSet specifications
  re-fetched read-only 2026-09-09; the already-pinned Proposal/Java/
  Yosemite revisions cited from existing M155 records) were accessed
  read-only for evidence;
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
