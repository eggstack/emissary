# M160 Closure — Encrypted LeaseSet DH (X25519) Client-Authorization Primitive

Status: **closed as complete; M161 hard dependency satisfied, registration pending**

Date: `2026-09-09`

Source implementation plan:

- `plans/implementation/i2pcontrol-proposal-170/160-leaseset-dh-client-authorization-primitive.md`
  (Status now `closed as complete` with closure link; hard dependency on
  M159 closure satisfied by `plans/closure/i2pcontrol-proposal-170/159-closure.md`;
  zero promotion budget observed as proven below).

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-post-m154-leaseset-security-corrective-roadmap.md`.

Promotion budget: **zero Proposal cells**.

Production budget: **exact four-path M160 set only**
(`emissary-core/src/crypto/els2.rs`,
`emissary-core/src/destination/lease_set.rs`,
`emissary-core/src/sam/parser.rs`,
`emissary-core/src/sam/session.rs`),
all already individually present in M061's realized exact allowlist, with
M062 current-registration bookkeeping closed in the same commit. No new
production file. No Cargo, lockfile, Yosemite, or
`emissary-cli/src/i2pcontrol/**` production change.

## Planning baseline

- Registration baseline `61cec0fefec457fbb8f633a7fdfe38ef68f941e3` (registered M160 handoff; clean worktree
  before M160 implementation).
- Last production-bearing head before M160: the M159 closure commit ancestry
  (`c0bbf9d488d7e1b51c3f511a6ebd4c1562068d24`, unchanged through the M160 registration commit, which touched
  only plans).
- Incoming M095 matrix: `336 apply / 29 blocked_primitive / 475
  not_applicable` across 840 TunnelManager option/family cells.
- M153 closed complete as the current runtime/security qualification
  authority; M154 closed complete disposition C (M147 path blocked);
  M155 closed complete with zero production; M156 closed complete with zero
  promotions; M157 closed complete with zero promotions; M158 closed
  complete with zero promotions; M159 closed complete with zero promotions;
  M146 closed blocked.

Reviewed head:

- M160 work completes on the working tree described in §9. Exact changed
  production paths are the four registered paths above (exact-owner
  extensions, no new file). Exact guard/planning paths are
  `emissary-cli/tests/m062_dependency_containment.rs` (M160 authorization
  helper only),
  `plans/implementation/i2pcontrol-proposal-170/062-dependency-containment.toml`
  (current-registration closure bookkeeping), the M160 plan (status flip),
  the M161 plan (deferred-status note only), this closure, registry/README/
  roadmaps/AGENTS/docs authority updates (see §6).

Pinned authority (all accessed read-only; no upstream mutation, contact, or
submission occurred — see read-only attestation):

- Proposal 170 revision `2026-05-20`, status Open, SHA-256
  `f13ae00b886c5e72131bc5d5b138a371148d1faa6899a119a1dacb65a555e7dc`
  (cited from M095/M153/M155; no new Proposal fetch — M160 adds no Proposal
  policy);
- Direct Java I2P/I2PTunnel standard-property contract
  `i2cp.leaseSetAuthType = "1"`, `i2cp.leaseSetPrivKey = Base64(32B X25519
  private)`, `i2cp.leaseSetClient.dh.N = [Base64(UTF8(name)) ":"] Base64(32B
  X25519 public)` (cited from the frozen M160 plan §2 records; no new fetch —
  the plan representation is consumed as the standard property);
- I2P Encrypted LeaseSet specification, DH layer-1 construction
  (`https://geti2p.net/en/docs/specs/encryptedleaseset`, read-only fetch of
  the already-pinned text for construction evidence: flags `0x01`,
  `ephemeralPublicKey || count_BE || records`, per-client `ELS2_XCA`
  schedule, cookie-bound L2; no new fetch beyond that single read — the
  frozen M160 plan §3 format is authoritative for this milestone);
- Java `net.i2p.crypto.EncryptedLeaseSet` `MAX_ENCRYPTED_SIZE = 4096`
  ceiling (cited from the frozen M160 plan §4; no new fetch — the
  4096-byte bound is enforced by checked arithmetic below);
- I2P Red25519 specification (cited from M156 records; no new fetch — M156
  remains the blinding authority and `crypto/red25519.rs` is unchanged);
- Java I2PControl Proposal-170 head `45bb593000408071dd376b78848fdc246dccd964`
  and Java I2P snapshot `2c3fd2a9532cd86ec06cb6f2b9f3f813ca752243` (cited
  from pinned M155 records; no new fetch);
- Yosemite optional exact revision `59140a2277bf296928d2e8ce39a148182eeff044`
  (unchanged; no Yosemite change in M160);
- Emissary source at the reviewed tree (all file:line citations below read
  at HEAD plus the workdir diff closed here).

Current Proposal matrix at closure (mechanically recomputed, dispositions
unchanged):

- `336 apply / 29 blocked_primitive / 475 not_applicable` (840 cells);
- residual split `SigType` 10, `EncryptLeaseSet` 5, `OptionalLookup` 5,
  `LeaseSetClientAuths` 5, `UseOutproxyPlugin` 4 — identical to
  M153/M154/M155/M156/M157/M158/M159 entry.

## 1. Requirement-to-evidence matrix (plan requirements)

| Plan requirement | Evidence |
|---|---|
| standard property `leaseSetType=5` + `leaseSetAuthType=1` | `is_valid_type5_dh` (`sam/parser.rs`): type trimmed exactly `5`, auth trimmed exactly `1`; any other auth selector fails; proven by accept + wrong-auth rejection tests |
| required base `leaseSetPrivKey` Base64 exactly 32B X25519 private | `parse_base_dh_value`: trims, rejects empty/colon-bearing input, I2P-Base64 decodes, requires exactly 32 bytes; missing base with auth 1 fails; proven by base-only accept + missing/malformed rejection tests |
| base-derived public always first authorized key | `DhPrivateKey::public` (`crypto/els2.rs`: `StaticSecret::from` + `PublicKey::from`); `extract_dh_authorization` pushes the derived public before indexed entries; proven by base-only accept test asserting the derived public |
| optional contiguous `leaseSetClient.dh.N` from zero | `extract_dh_authorization`: collects `i2cp.leaseSetClient.dh.N`, parses indices, requires exactly `0..n-1` with no gaps; zero indexed entries valid; proven by ordered multi-client accept + sparse rejection tests |
| optional single `name:` prefix stripped, names not retained | `parse_dh_entry_value`: splits on exactly one `:`, requires non-empty name/suffix, validates suffix only, never stores prefix; core types carry `[u8; 32]` publics only; proven by named-entry accept + order test and redaction tests |
| duplicate public-key bytes preserved exactly | no dedup anywhere in extraction, `DhAuthorization`, or crypto; duplicates occupy two records and the same size budget; proven by duplicate-preservation tests at parser and crypto layers (§2) |
| mixed PSK entries rejected; DH/PSK mutually exclusive | extraction rejects any `leaseSetClient` without the exact `i2cp.` DH prefix and any `i2cp.leaseSetClient.psk`/bare-`leaseSetClient` key; PSK extractor defers auth-1 requests to DH and vice versa; both-`Some` fails closed; proven by mixed-entry + mutual-exclusion tests |
| incompatible companions rejected | legacy `encryptLeaseSet=true/1`, `dontPublishLeaseSet=true/1`, non-empty `leaseSetKey`/`leaseSetPrivateKey`/`leaseSetSigningPrivateKey`/`leaseSetBlindedType` all fail; proven by negative tests |
| checked pre-allocation ceiling + exact size re-check | `MAX_DH_CLIENTS = 99` from `(4096-100)/40`; extraction rejects `>99` total keys before decoding; `dh_outer_len` uses fully checked arithmetic and `encrypt_dh_with_ephemeral` fails when `total > 4096`; checked before any per-client X25519 work; never drops entries; proven by ceiling + overflow tests |
| DH values removed from generic options | `extract_dh_authorization` removes base + every indexed DH key from `key_value_pairs` before owned-`options` construction; options retain only selectors (`leaseSetType`, `leaseSetAuthType`); proven by absence assertions plus `Debug` redaction tests |
| key material only in zeroizing/non-`Debug` types | `DhPrivateKey(Zeroizing<[u8; 32]>)` (dropped after public derivation), `DhPublicKey`/`DhAuthorization`: no `Debug`/`Display`, `from_keys` gates empty/over-ceiling; ephemeral `StaticSecret` zeroizes on drop, shared secrets/cookies/HKDF inputs/plaintext zeroized; proven by gating + redaction tests |
| explicit all-zero shared-secret rejection at ELS2 boundary | `dh_shared_or_reject` (server) + `dh_shared_client_or_reject` (client) fail closed on `shared == [0u8; 32]` before any KDF; proven by zero-peer encrypt rejection (library accepts the key, ELS2 rejects) + zero-ephemeral decrypt rejection tests |
| M158 lookup secret coexists independently | `DestinationContext` carries `lookup_secret` alongside `dh_auth`; `EncryptedPublicationConfig::{with_dh,with_secret_and_dh}` compose them; blinding still uses the secret exactly; proven by secret+DH coexistence tests at parser and publication layers |
| no persistence in core | no store, file, or static holds DH material; config is generation-local; replacement generations carry only their construction keys; proven by replacement-isolation reasoning plus no-fallback tests |
| `ELS2_XCA` HKDF-SHA256 52-byte schedule | `derive_dh_client` (`shared \|\| cpk \|\| subcredential \|\| published_BE` salted by the ephemeral public, `ELS2_XCA`, key32+IV12+ID8); proven by independent-HMAC KAT (§2) |
| fresh ephemeral X25519 keypair + fresh 32B auth cookie per regenerated object | `encrypt_dh` fills inner/outer salts + cookie from caller `CryptoRng` and generates the ephemeral pair via `StaticSecret::random_from_rng` on every call; deterministic helper exists only for tests; proven by rollover-freshness test (old vs new outer differ) |
| exact client record layout | `clientID[8] \|\| ChaCha(clientKey, clientIV, authCookie)`; proven by KAT byte assertions |
| auth-cookie-bound L2, unchanged L1 | `derive_layer_keys_with_cookie` (`authCookie \|\| subcredential \|\| published_BE`, `ELS2_L2K`); L1 still (`subcredential \|\| published_BE`, `ELS2_L1K`); proven by L2-differs/L1-equal assertions |
| flags `0x01` | `DH_LAYER1_FLAGS`; decrypt rejects any other flags byte; proven by KAT + tamper tests; no-auth `0x00` and PSK `0x03` paths untouched |
| randomized multi-client order | `encrypt_dh` Fisher-Yates shuffles record order with the caller RNG when `N>1`; deterministic helper preserves given order so tests prove order-independence; proven by forward/reverse decrypt-both tests + production double-encrypt tests |
| no-auth M157 path unchanged | `encrypt_no_auth*`/`decrypt_no_auth` untouched; DH outers rejected by `decrypt_no_auth` and vice versa; proven by cross-rejection tests + full regression suites |
| lookup-secret-only M158 path unchanged | `blinded_day_material*` untouched; secret-only publication tests retained and passing |
| PSK M159 path unchanged | `encrypt_psk*`/`decrypt_psk` untouched; DH outers rejected by `decrypt_psk` and vice versa; proven by cross-rejection tests + full regression suites |
| sole publication owner, no second scheduler | `LeaseSetManager::refresh_encrypted_outer` branches on `(psk_auth, dh_auth)` with both-`Some` failing closed; rollover/day-only paths reuse the same config; no new timer/state machine; proven by rollover tests |
| build failure never falls back | `refresh_encrypted_outer` returns `Err` before state mutation; callers retry without emitting ordinary/empty/no-auth/PSK stores (`create_database_store` returns `None` without outer); proven by corrupt-inner failure test |
| stale generations cannot publish superseded material | key-scoped verification (`register_database_store` type+key match) retained; rollover rotates once per boundary and ignores stale-key acks; proven by stale-ack test |
| extended B32 `auth_required=true` | `server_destination_address` carries `(key, secret_required, auth_required)`; DH bridge passes `auth_required=true` while `secret_required` reflects the lookup secret; ordinary path byte-for-byte unchanged; `events.rs` untouched; proven by address tests |
| no log/error/metric/event carries DH bytes | parser rejection logs name the failure shape only; `DestinationContext` custom `Debug` omits all fields; `SamCommand` `Debug`/`Display` carry session IDs only; proven by redaction tests (residual socket-echo note in §11) |
| no-std | `cargo check -p emissary-core --no-default-features --features no_std` passes; new code uses `alloc`/`core` plus existing codecs/`zeroize`/`hmac`/`sha2`/`x25519-dalek` only |

## 2. Format conformance (plan §3 + frozen references)

| Item | Implementation | Source |
|---|---|---|
| Standard properties | `leaseSetType=5`, `leaseSetAuthType=1`, `leaseSetPrivKey=Base64(32B X25519 private)`, `leaseSetClient.dh.N=[name:]Base64(32B X25519 public)`; blank/missing base with auth 1 fails | Plan §2 / Java handler records |
| Base key role | derived base public is always first logical authorized key; indexed entries additional; zero indexed valid | Plan §2 |
| Duplicate semantics | duplicates preserved exactly as configured, each consuming one 40-byte record and the 4096 budget; no dedup | Plan §2 + `6228782d` semantics commit |
| Layer-1 flags | `0x01` (per-client auth + DH scheme bits) | Plan §3 / ELS spec |
| Per-client schedule | `okm = HKDF-SHA256(ephemeralPub, shared \|\| cpk \|\| subcredential \|\| published_BE, "ELS2_XCA", 52)`; key `okm[0..32]`, IV `okm[32..44]`, ID `okm[44..52]`; `encCookie = ChaCha20(key, IV, authCookie)` | Plan §3 / ELS spec |
| Inner schedule | `HKDF-SHA256(innerSalt, authCookie \|\| subcredential \|\| published_BE, "ELS2_L2K", 44)` | Plan §3 / ELS spec |
| Outer schedule | `HKDF-SHA256(outerSalt, subcredential \|\| published_BE, "ELS2_L1K", 44)` (unchanged) | Plan §3 / M157 |
| Wire layout | `outerSalt[32] \|\| ChaCha_L1(0x01 \|\| ephemeralPub[32] \|\| count_BE[u16] \|\| records(8+32) \|\| innerSalt[32] \|\| ChaCha_L2(0x03 \|\| innerLS2))` | Plan §3 / ELS spec |
| Size bound | `32+1+32+2+40*N+32+1+inner ≤ 4096` checked with checked arithmetic before per-client X25519 work | Plan §4 / Java `MAX_ENCRYPTED_SIZE` |
| Absolute entry ceiling | `(4096-100)/40 = 99` (`MAX_DH_CLIENTS`); `u16` encoding never binding | Plan §4 |
| Freshness | fresh ephemeral keypair + auth cookie (+ inner/outer salts) per regenerated outer; randomized record order for `N>1` | Plan §3/§7 |
| Address flags | extended B32 `auth_required=true` for DH generations; `secret_required` independent | Plan §8 |

KAT provenance (fixed `crypto::els2` fixtures: type-7 key
`8a88e3dd7409f195fd52db2d3cba5d72ca6709bf1d94121bf3748801b40f6f5c`,
day `20260909`, published `1788000000`, inner 16 bytes
`deadbeef00112233445566778899aabb`, salts `0x11/0x22`, ephemeral private
`0xE7`, cookie `0x44`, base private `0xA5`, client private `0xB6`):

- subcredential recomputed through the frozen `credential`/`subcredential`
  helpers and pinned by known-answer test;
- base/client/ephemeral publics recomputed through direct `x25519-dalek`
  derivation (independent path, not the helper under test);
- shared secrets recomputed through direct `StaticSecret::diffie_hellman`
  (independent path, not the `SecretKey` seam under test);
- `ELS2_XCA` 52-byte outputs pinned against an independent local
  HMAC-SHA256 extract+expand recomputation (`independent_hkdf_52`), not
  the helper under test;
- client IDs, encrypted cookies, L1/L2 key splits, complete outer
  ciphertext layout, and `outer.len() == dh_outer_len(inner, N)` asserted
  byte-for-byte;
- outer Red25519 verification proven at the publication layer:
  `EncryptedLeaseSet2::parse` (which verifies `0x05 || prefix` via M156)
  accepts every DH outer produced by `LeaseSetManager`, and
  `decrypt_dh` recovers the exact inner bytes under each authorized
  private key.

Decoder/negative matrix (all fail closed): missing base, malformed
Base64, non-32-byte keys, sparse indices (`dh.1` without `dh.0`,
`dh.0+dh.2` without `dh.1`), mixed PSK entries, PSK/DH mutual-exclusion
violation, wrong auth selectors (`0/2/00/hello`, missing auth with base
present), legacy flag, unpublished flag, incompatible key companions,
multi-colon entries, wrong client/no match, tampered
flags/ephemeral/ID/cookie, all-zero peer public, all-zero ephemeral on the
wire, wrong published, wrong subcredential (wrong-secret equivalent),
checked-size overflow, `>4096` outer input, corrupt inner, stale
day/material acks, and all cross-mode fallbacks (`decrypt_no_auth` on DH
outer, `decrypt_psk` on DH outer, `decrypt_dh` on PSK outer,
`decrypt_dh` on no-auth outer).

Reference interoperability: no Java fixture checkout was performed
(read-only boundary). Interop evidence is the independent-HMAC KAT
recomputation of the frozen plan §3 schedule with independently derived
X25519 publics/shared secrets plus `EncryptedLeaseSet2`
signature-verify + `decrypt_dh` round-trips at the publication layer.
Deterministic reference randomness is not required by the plan when
derivation/decryption evidence is stronger; that condition is met. No
production client resolver was added.

## 3. Dependency/security review (M155 §6 freeze)

- No new dependency. Reused: existing workspace `x25519-dalek`
  (`3.0.0-pre.6`, features `reusable_secrets`/`static_secrets`/`zeroize`/
  `precomputed-tables`, already in `emissary-core/Cargo.toml`) via
  `StaticSecret::random_from_rng` (ephemeral keygen), `PublicKey::from`
  (public derivation), and the existing `crypto::SecretKey::diffie_hellman`
  seam (shared-secret bytes, all through `StaticPublicKey` wrappers, no new
  module); `hmac`/`sha2` (HKDF extract+expand), `chacha20` via frozen
  `ChaCha::with_iv` (counter-1 seek, unchanged file), `rand` RNG
  (`RngCore`/`CryptoRng` only), `zeroize` (`Zeroizing` private/cookie/
  inputs/plaintext + scrub-on-reject), `data-encoding` I2P Base64 (existing
  codecs), M156 Red25519 (unchanged), M157 ELS2/rollover owner, M158
  secret/B32 stack, M159 PSK stack (untouched), `bytes`/`nom` (existing wire
  owners). No second curve/X25519 implementation.
- `Cargo.toml`, `emissary-core/Cargo.toml`, `Cargo.lock` unchanged in this
  commit (verified by `git status --short`; M062 records the zero budget).
- No Yosemite change; no `emissary-cli/src/i2pcontrol/**` production change.
- `no_std + alloc` compatible; proven by the `no_std` check above.
- Constant-time/zeroization: Dalek ops via M156 seam; `DhPrivateKey`
  zeroized on drop with no `Debug`/`Display`; `DhPublicKey`/
  `DhAuthorization` never `Debug`-formattable; ephemeral `StaticSecret`
  zeroizes on drop; shared secrets, auth-cookie copies, HKDF inputs/outputs,
  and L1/inner plaintext buffers zeroized after use; `LookupSecret`/
  `SigningSeed`/blinded-private handling unchanged; no DH material in logs,
  errors, metrics, or events.
- Bounds before crypto: SAM framing bounds the properties; every key
  decodes to exactly 32 bytes before activation; base public derived before
  authorization construction; indexed count bounded by `MAX_DH_CLIENTS`
  before decoding; complete `dh_outer_len` computed with checked arithmetic
  before any X25519/HKDF/ChaCha work; oversize fails closed without dropping
  entries.
- Domain separation: unchanged `ELS2_L1K`/`ELS2_L2K` labels plus new
  `ELS2_XCA` label; sigtype codes, `YYYYMMDD`, CRC polynomial, flag bits
  unchanged.
- Fail-closed: malformed/sparse/mixed/over-ceiling inputs, bad
  flags/sigtypes/checksums/keys/lengths, all-zero shared secrets, expired
  inners, wrong-type/key verification replies, stale-key acks, and
  crypto/build failures all reject before network or storage effect with no
  ordinary, empty-secret, no-auth, PSK, or type-3 fallback.
- Ordinary Ed25519/LS2/DatabaseStore-type-3/NetDB-ordinary/M135/M145 SAM
  behavior preserved (see §5).

## 4. Exact-path reconciliation (M061/M062)

M061 (`061-containment-boundary.toml` + `m061_containment.rs`):

- No change required and none made. All four M160 production paths are
  already individually present in the realized exact `[allowed]` ledger
  from M060/M157; M160 is a strict active subset frozen by the M160 plan
  and the registry. `registered_pending` handling is untouched.

M062 (`062-dependency-containment.toml` + `m062_dependency_containment.rs`):

- `062-dependency-containment.toml` `[current_registration]` bookkeeping
  closed: milestone `M160 (closed complete, no registered successor)`
  realized with the exact four production paths, zero new files,
  `new_direct_dependencies=[]`, `manifest_changes=[]`,
  `lockfile_change=false`, `yosemite_change=false`,
  `i2pcontrol_source_change=false`.
- `m062_dependency_containment.rs` gains narrow `is_authorized_m160_path`
  covering exactly the four production files plus the M160 closure/plan/
  M161-plan-note/registry/README/roadmap/AGENTS/docs set, wired into both
  the allowed-path and prohibited-pattern assert chains (same pattern as the
  M159 helper; authorizes the new closure file, which no earlier helper
  covers).
- `policy_terms_do_not_leak_into_non_policy_production_paths` passes: new
  core source contains no `Proposal 170`/`JsonRpc`/`jsonrpc`/
  `TunnelManagerControl`/`control-state.json` strings.

## 5. Verification outcomes

| Command group | Result |
|---|---|
| `cargo check -p emissary-core` | **pass** |
| `cargo check -p emissary-core --no-default-features --features no_std` | **pass** |
| `cargo test -p emissary-core --lib --no-fail-fast` | **pass**: `1159 passed, 2 ignored` (1141 pre-existing + 18 new M160 tests, zero regressions) |
| `cargo check -p emissary-cli --no-default-features --features i2pcontrol` | **pass** |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --lib --no-fail-fast` | **pass**: `821 passed` |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment --test m062_dependency_containment --test m095_full_support_matrix --test m105_residual_option_audit --no-fail-fast` | **pass**: `8 + 23 + 3 + 1` across 4 suites |
| M095 mechanical recomputation (via `m095_full_support_matrix` suite) | **pass**: `840/336/29/475` == declared |
| `cargo clippy -p emissary-core --all-targets -- -D warnings` | **pass**: No issues found |
| `cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings` | pre-existing failure only in unrelated `proxy.rs` path; zero warnings from M160 files (see §11) |
| stable `rustfmt` on M160-touched Rust files | **pass** for all four production files plus the M062 guard test (repo nightly-only options warn; `max_width = 100` observed; no new nightly drift introduced — see §11) |
| `git diff --check` | **pass** |

Focused M160 suites (18 tests): `crypto::els2` (6: bounded-type gating,
checked-length ceiling, single-client KAT vs independent HMAC + independent
X25519 derivation, multi-client order-independence + production
randomization, duplicate preservation + size-boundary, full negative matrix
without fallback); `destination::lease_set` (4: DH publication decrypt +
4096 bound + cross-rejection, rollover freshness + stale-ack + type-gated
verification, secret+DH composition, build-failure no-fallback);
`sam::parser` (7: base-only accept+redaction, indexed order + name
stripping, duplicate preservation, secret coexistence, negative matrix,
debug-surface redaction, PSK/DH mutual exclusion); `sam::session` (1:
extended-B32 `auth_required` emission for both secret states).

M157/M158/M159/ordinary regressions: all pre-existing no-auth type-5,
lookup-secret type-5, PSK type-5, B32 flags/CRC, UTC-rollover,
type-preserving NetDB storage/flooding, and ordinary type-3 LS2 tests
retained and passing (1141 pre-existing core + 821 CLI lib, zero failures).

## 6. Changed paths (exact budget only)

Production (4):

- `emissary-core/src/crypto/els2.rs` (`DhPrivateKey`, `DhPublicKey`,
  `DhAuthorization`, `DH_LAYER1_FLAGS`/`ELS2_XCA`/`MAX_ENCRYPTED_DATA_LEN`
  reuse/`MAX_DH_CLIENTS`/`DH_CLIENT_RECORD_LEN`, `derive_dh_client` +
  all-zero-rejecting DH seams, `dh_outer_len`, `encrypt_dh_with_ephemeral`/
  `encrypt_dh`, `decrypt_dh`, 6 tests);
- `emissary-core/src/destination/lease_set.rs` (generation-local DH
  ownership (`with_dh`/`with_secret_and_dh`), DH branch in
  `refresh_encrypted_outer` with fresh RNG ephemeral/cookie/salts, both-
  `Some` fail-closed, 4 tests plus helper);
- `emissary-core/src/sam/parser.rs` (DH entry/base decoders,
  `extract_dh_authorization` with redaction + PSK deferral, PSK deferral on
  DH auth, `is_valid_type5_dh`, extended type-5 gate, context DH field,
  7 tests);
- `emissary-core/src/sam/session.rs` (DH threading into publication
  config, three-mode extended-B32 event emission, address test).

Guards/ledgers (2):

- `emissary-cli/tests/m062_dependency_containment.rs` (M160 authorization
  helper in both chains);
- `plans/implementation/i2pcontrol-proposal-170/062-dependency-containment.toml`
  (current-registration closure bookkeeping, zero-budget recorded).

Planning/closure/registry (8 + 1 note):

- `plans/implementation/i2pcontrol-proposal-170/160-leaseset-dh-client-authorization-primitive.md`
  (Status `registered / dependency-ready` → `closed as complete` with
  closure link);
- new `plans/closure/i2pcontrol-proposal-170/160-closure.md` (this file);
- `plans/registry.md` (M160 → closed; no registered successor; M161
  deferred with satisfied hard dep noted);
- `plans/implementation/i2pcontrol-proposal-170/README.md` (M160 closed
  section; chain updated);
- `plans/subsystems/i2pcontrol-proposal-170-post-m154-leaseset-security-corrective-roadmap.md`
  (M160 closed; graph updated);
- `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md`
  (M160 closed; handoff updated);
- `AGENTS.md` (M160 closed entry; chain updated);
- `plans/implementation/i2pcontrol-proposal-170/161-legacy-aes-ls1-feasibility-and-contract-gate.md`
  (deferred-status note: M160 hard dependency satisfied, registration
  pending; no registration).

Docs (3, authority wording only):

- `docs/i2pcontrol/README.md`, `docs/i2pcontrol/proposal-170-support.md`,
  `docs/i2pcontrol/tunnel-manager.md` (M160 closed authority + `336/29/475`
  wording retained; LeaseSet residual-blocked statements retained).

`061/062`, `095/105/110` semantics: `061` unchanged (strict subset, no
broad waiver); `062` gains only the exact M160 closure authorization
above; `095/105/110` intentionally **unchanged** (zero promotions; see
§8).

## 7. Requirement-to-evidence matrix (planning process §2.5)

| Closure duty | Evidence |
|---|---|
| implementation commits | one commit landing with this closure (production + tests + M062 guard + planning records); exact paths in §6; no prior partial implementation commit (registration commit touched only plans) |
| invariant review | partial-support, no-fabrication, I2P-only egress, fail-closed option validation, containment, and Y005 invariants re-proved by green `m061`/`m062`/`m095`/`m105` guards; no invariant weakened; M147/M148 not reopened (type-11 derived-only, gates enforced); M146 untouched (no egress); no downgrade path exists (fallible derivation, all-zero rejection, no no-auth/PSK/ordinary fallback, stale-key rejection) |
| failure/recovery and contention evidence | malformed/sparse/mixed/over-ceiling inputs reject pre-allocation; all-zero shared secrets reject at the ELS2 boundary before KDF; oversize publication fails at checked-size with retry and no fallback; wrong-type/key verification ignored; stale-key acks ignored; expired/invalid inner never published; rollover is single-owner timer/state with no second task; no lock spans I/O or crypto; shutdown drops the owner-local timer; successor generations receive no predecessor keys (config is generation-local); tamper/wrong-key/flag/type/length tests prove closed rejection |
| compatibility, migration, security review | no wire change to type-3/RouterInfo/lookup protocol; DH + auth-required B32 are additive; no storage migration (opaque DHT keys, bounded cache); no new attack surface beyond reviewed parser/codec/derivation composition; all 15 LeaseSet cells stay fail-closed (matrix unchanged); legacy AES still blocked; `cargo clippy -p emissary-core -D warnings` clean; `emissary-cli -D warnings` failure is pre-existing and unrelated with zero M160-file warnings |
| documentation and operational evidence | §6 changed paths; authority docs name M160 closed and retain partial support; operational impact none beyond the opt-in standard DH properties and the correct type-5 server address form (default ordinary path byte-compatible) |
| M160 acceptance (plan closure evidence) | exact changed files/dependencies (§6), standard-property evidence (§1), DH ephemeral/cookie/schedule known answers (§1/§2), extended-B32 auth vectors (§1/§5), DH-redaction audit (§1), rollover/recreation traces (§1/§5), no-downgrade evidence (§1/§5), unchanged M095 (§8), implementation SHA (commit landing with this closure), M161/M162 readiness (§10) |

## 8. Promotion accounting (zero)

- `110-completion-ledger.toml`: no new entry (zero promotions).
- `095-full-support-matrix.toml` / `105-residual-option-audit.toml`:
  unchanged.
- Mechanically recomputed `336/29/475` == declared; residual
  `10/5/5/5/4` unchanged.
- Infrastructure alone has zero support value (registry rule);
  no `EncryptLeaseSet`/`OptionalLookup`/`LeaseSetClientAuths` cell is
  claimed by this primitive. `LeaseSetClientAuths` remains blocked until
  M162 supplies mapping, custody, redaction, transactionality, and
  five-family integration; legacy AES remains M161.

## 9. Production-head determination

- Last production-bearing commit before M160: the M159 closure commit
  ancestry (unchanged through M160 registration at `61cec0fe`).
- M160 is production-bearing (exact §6 files); after the closure commit
  lands, the last production-bearing head becomes that commit.
- `git log 61cec0fe..HEAD` over `emissary-core/src`,
  `emissary-cli/src`, `emissary-util/src`, all manifests and `Cargo.lock`
  at closure time contains only the staged M160 exact-budget diff
  (4 production paths; Rust production delta is four exact-owner extensions
  plus the M062 guard exact amendment).
- `git diff --name-only` over manifests/lockfile is empty: no `Cargo.toml`,
  `emissary-core/Cargo.toml`, or `Cargo.lock` change. `git status --short`
  shows only the §6 paths.
- Streamr datagram limits (16-subscriber, 60s expiry, 1200-byte payload,
  4095-byte transport-buffer, 15s refresh, bounded shutdown,
  loopback-only UDP) untouched; remote datagrams never choose a local UDP
  destination.

## 10. Registry updates and future-plan unblock determination

Applied alongside this closure (see §6 for the file list):

- `160-*.md` plan: Status `registered / dependency-ready` → `closed as
  complete` with closure link;
- `plans/registry.md`: M160 → closed as complete (`336/29/475` unchanged,
  zero promotions, exact production diff); **no registered successor**;
- `110-completion-ledger.toml`: no new entry (zero promotions);
- `m061`: unchanged (strict subset; no broad waiver);
- `m062`: exact M160 closure authorization only (§4); no broad waiver.

Future-plan unblock determination (as required by the tasking):

- **M160 CLOSED as complete.** Its sole hard dependency (M159 closure with
  frozen publication/rollover owner and zero promotions) was satisfied,
  and all plan requirements are met with exact-budget production diff and
  zero matrix promotions. No stop condition in plan §12 triggered: no
  outside file or dependency was required, no I2PControl persistence
  change was needed for the neutral primitive, no NetDB subsystem was
  added, DH data leaves generic debug-capable state before activation,
  all-zero shared secrets reject at the ELS2 boundary, the extended-B32
  auth flag needed no broader naming subsystem, reference vectors agree
  with local behavior, and no failure downgrades to no-auth, PSK, or
  ordinary publication.
- **M161 REMAINS deferred/unregistered (hard dependency now satisfied,
  registration pending).** M160 closure satisfies its hard dependency, and
  the gate needs no production envelope: it is a strictly zero-production
  feasibility/contract gate over the now-frozen modern owner graph. The
  M161 plan file gains only a deferred-status note recording the satisfied
  hard dep; it is **not** registered by this closure. Only M161 may be
  registered next. No generic exact-path research milestone is required.
- **M162 remains deferred/unregistered**, hard-gated on M161 disposition
  (plus any M161-A successor closure before `EncryptLeaseSet` promotion).
  It inherits the standard-property mapping (`i2cp.leaseSetAuthType=1` +
  base/indexed DH consumer proven here, `authType=2` + base/indexed PSK
  proven in M159; Proposal `LeaseSetClientAuths` naming/persistence mapping
  still to build).
- **M152 remains deferred/unregistered**, re-gated on M162 closure plus any
  M161-A successor. Its final qualification must now account for the M155
  field-coupling rule plus the M156/M157/M158/M159/M160 primitive
  availability and the M159/M160 authenticated bounds.
- **M149-M151 remain superseded/unregistered (do not execute).** Confirmed
  fully superseded by M155-M162; no re-gating or revival is authorized.
- **M147/M148 remain closed-blocked/deferred-behind-M147** (M154
  disposition C); this line does not reopen them. Type-11 blinding is
  explicitly not a `SigType` reopening.
- **M146 remains closed as blocked** with no successor; it is not reopened
  by M160 or by any future LeaseSet tail.
- No other future-plan status required a change. File presence alone never
  authorises production work.

## 11. Unresolved findings

- Test-maintenance note (expected behavior change, applied): the pre-existing
  `type5_psk_negative_paths_rejected` wrong-auth vector used a bare base key
  with auth `1` and expected rejection. A bare base key with auth `1` is now
  a valid DH base-only request, so that vector is pinned to PSK mode with an
  indexed `psk.0` entry (which still fails closed via the mixed-entry gate).
  Severity: test-only; no code impact; documents the intended DH/PSK
  disambiguation.
- Non-blocking: `cargo test -p emissary-cli --no-default-features
  --features i2pcontrol --test m153_post_m146_requalification` reports
  `AGENTS.md must retain partial support`. Fails identically at
  the pre-M160 registration baseline; unrelated to M160 files. Severity:
  informational; no corrective pass required (out of M160 exact-budget
  scope; must not be smuggled into this line).
- Non-blocking: `cargo clippy -p emissary-cli --no-default-features
  --features i2pcontrol --all-targets -- -D warnings` reports a
  pre-existing lint in `backends/filters/proxy.rs:60` (`chunks_exact`),
  untouched by M160; M160 files contribute zero warnings. Severity:
  informational; no corrective pass required (out of M160 exact-budget
  scope).
- Non-blocking: `cargo fmt --all -- --check` (stable or nightly) reports
  drift in files outside the M160 budget at the registration baseline;
  M160-touched Rust files are stable-`rustfmt` clean with `max_width = 100`
  observed, and no new nightly drift was introduced by M160 lines
  (remaining nightly hunks are pre-existing comment reflow from a newer
  toolchain). Severity: informational; unrelated files were not touched.
- Informational: rejected-command logging in `sam/socket.rs` (outside the
  M160 budget, intentionally untouched) echoes the raw command line when
  `SamCommand::parse` fails. A *malformed*-DH `SESSION CREATE` would
  therefore echo its rejected option value to the local log; valid-DH
  sessions never hit that path (parse succeeds, nothing is logged).
  Severity: low/informational; secret hygiene for that generic socket path
  is noted for a future hardening pass and must not be smuggled into this
  line.

## Internal-only / read-only-upstream attestation

- External sources (the already-pinned Proposal/Java/Yosemite revisions and
  the Encrypted-LeaseSet/Red25519 specifications cited from existing
  M155/M156/M157 records) were accessed read-only for evidence; one
  additional read-only fetch of the already-pinned Encrypted-LeaseSet
  specification text was performed to confirm the DH wire/schedule
  construction — no Java source checkout was performed for this closure;
  DH vectors were cross-checked against an independent local HMAC+X25519
  recomputation of the frozen plan schedule plus
  `EncryptedLeaseSet2` signature verification;
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
