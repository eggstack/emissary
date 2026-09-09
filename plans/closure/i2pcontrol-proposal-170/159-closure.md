# M159 Closure — Encrypted LeaseSet PSK Client-Authorization Primitive

Status: **closed as complete; M160 hard dependency satisfied, registration pending**

Date: `2026-09-09`

Source implementation plan:

- `plans/implementation/i2pcontrol-proposal-170/159-leaseset-psk-client-authorization-primitive.md`
  (Status now `closed as complete` with closure link; hard dependency on
  M158 closure satisfied by `plans/closure/i2pcontrol-proposal-170/158-closure.md`;
  zero promotion budget observed as proven below).

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-post-m154-leaseset-security-corrective-roadmap.md`.

Promotion budget: **zero Proposal cells**.

Production budget: **exact four-path M159 set only**
(`emissary-core/src/crypto/els2.rs`,
`emissary-core/src/destination/lease_set.rs`,
`emissary-core/src/sam/parser.rs`,
`emissary-core/src/sam/session.rs`),
all already individually present in M061's realized exact allowlist, with
M062 current-registration bookkeeping closed in the same commit. No new
production file. No Cargo, lockfile, Yosemite, or
`emissary-cli/src/i2pcontrol/**` production change.

## Planning baseline

- Registration baseline `6228782d2d1dea3eda6456aaf4f74615ec86de7d` (registered M159 handoff; clean worktree
  before M159 implementation).
- Last production-bearing head before M159: the M158 closure commit ancestry
  (`57473dc4d729e1a298471b4e20fc062df19f7aeb`, unchanged through M159 registration commits, which touched
  only plans, TOMLs, and containment tests).
- Incoming M095 matrix: `336 apply / 29 blocked_primitive / 475
  not_applicable` across 840 TunnelManager option/family cells.
- M153 closed complete as the current runtime/security qualification
  authority; M154 closed complete disposition C (M147 path blocked);
  M155 closed complete with zero production; M156 closed complete with zero
  promotions; M157 closed complete with zero promotions; M158 closed
  complete with zero promotions; M146 closed blocked.

Reviewed head:

- M159 work completes on the working tree described in §9. Exact changed
  production paths are the four registered paths above (exact-owner
  extensions, no new file). Exact guard/planning paths are
  `emissary-cli/tests/m062_dependency_containment.rs` (M159 authorization
  helper only),
  `plans/implementation/i2pcontrol-proposal-170/062-dependency-containment.toml`
  (current-registration closure bookkeeping), the M159 plan (status flip),
  the M160 plan (deferred-status note only), this closure, registry/README/
  roadmaps/AGENTS/docs authority updates (see §6).

Pinned authority (all accessed read-only; no upstream mutation, contact, or
submission occurred — see read-only attestation):

- Proposal 170 revision `2026-05-20`, status Open, SHA-256
  `f13ae00b886c5e72131bc5d5b138a371148d1faa6899a119a1dacb65a555e7dc`
  (cited from M095/M153/M155; no new Proposal fetch — M159 adds no Proposal
  policy);
- Direct Java I2P/I2PTunnel standard-property contract
  `i2cp.leaseSetAuthType = "2"`, `i2cp.leaseSetPrivKey = Base64(32B)`,
  `i2cp.leaseSetClient.psk.N = [Base64(UTF8(name)) ":"] Base64(32B)`
  (cited from the frozen M159 plan §2.1 records:
  `RequestLeaseSetMessageHandler`, `ServiceTunnelCreator`; no new fetch —
  the plan representation is consumed as the standard property);
- I2P Encrypted LeaseSet specification, PSK layer-1 construction
  (`https://geti2p.net/en/docs/specs/encryptedleaseset`, cited from
  M155/M157 records; no new fetch — the frozen M159 plan §2.2 format is
  authoritative for this milestone);
- Java `net.i2p.crypto.EncryptedLeaseSet` `MAX_ENCRYPTED_SIZE = 4096`
  ceiling (cited from the frozen M159 plan §2.3; no new fetch — the
  4096-byte bound is enforced by checked arithmetic below);
- I2P Red25519 specification (cited from M156 records; no new fetch — M156
  remains the blinding authority and `crypto/red25519.rs` is unchanged);
- Java I2PControl Proposal-170 head `45bb593000408071dd376b78848fdc246dccd964`
  and Java I2P snapshot `2c3fd2a9532cd86ec06cb6f2b9f3f813ca752243` (cited
  from pinned M155 records; no new fetch);
- Yosemite optional exact revision `59140a2277bf296928d2e8ce39a148182eeff044`
  (unchanged; no Yosemite change in M159);
- Emissary source at the reviewed tree (all file:line citations below read
  at HEAD plus the workdir diff closed here).

Current Proposal matrix at closure (mechanically recomputed, dispositions
unchanged):

- `336 apply / 29 blocked_primitive / 475 not_applicable` (840 cells);
- residual split `SigType` 10, `EncryptLeaseSet` 5, `OptionalLookup` 5,
  `LeaseSetClientAuths` 5, `UseOutproxyPlugin` 4 — identical to
  M153/M154/M155/M156/M157/M158 entry.

## 1. Requirement-to-evidence matrix (plan requirements)

| Plan requirement | Evidence |
|---|---|
| standard property `leaseSetType=5` + `leaseSetAuthType=2` | `is_valid_type5_psk` (`sam/parser.rs`): type trimmed exactly `5`, auth trimmed exactly `2`; any other auth selector fails; proven by accept + wrong-auth rejection tests |
| required base `leaseSetPrivKey` Base64 exactly 32B | `parse_base_psk_value`: trims, rejects empty/colon-bearing input, I2P-Base64 decodes, requires exactly 32 bytes; missing base with auth 2 fails; proven by base-only accept + missing/malformed rejection tests |
| optional contiguous `leaseSetClient.psk.N` from zero | `extract_psk_authorization`: collects `i2cp.leaseSetClient.psk.N`, parses indices, requires exactly `0..n-1` with no gaps; zero indexed entries valid; proven by ordered multi-client accept + sparse rejection tests |
| optional single `name:` prefix stripped, names not retained | `parse_psk_entry_value`: splits on exactly one `:`, requires non-empty name/suffix, validates suffix only, never stores prefix; core types carry `[u8; 32]` keys only; proven by named-entry accept + order test and redaction tests |
| duplicate PSK bytes preserved exactly | no dedup anywhere in extraction, `PskAuthorization`, or crypto; duplicates occupy two records and the same size budget; proven by duplicate-preservation tests at parser and crypto layers (§2) |
| mixed DH entries rejected | extraction rejects any `leaseSetClient` without the exact `i2cp.` PSK prefix and any `i2cp.leaseSetClient.dh`/bare-`leaseSetClient` key; proven by mixed-entry rejection test |
| incompatible companions rejected | legacy `encryptLeaseSet=true/1`, `dontPublishLeaseSet=true/1`, non-empty `leaseSetKey`/`leaseSetPrivateKey`/`leaseSetSigningPrivateKey`/`leaseSetBlindedType` all fail; proven by negative tests |
| checked pre-allocation ceiling + exact size re-check | `MAX_PSK_CLIENTS = 99` from `(4096-100)/40`; extraction rejects `>99` total keys before decoding; `psk_outer_len` uses fully checked arithmetic and `encrypt_psk_with_salts` fails when `total > 4096`; never drops entries; proven by ceiling + overflow tests |
| PSK values removed from generic options | `extract_psk_authorization` removes base + every indexed PSK key from `key_value_pairs` before owned-`options` construction; options retain only selectors (`leaseSetType`, `leaseSetAuthType`); proven by absence assertions plus `Debug` redaction tests |
| key material only in zeroizing/non-`Debug` types | `PskKey(Zeroizing<[u8; 32]>)`, `PskAuthorization(Vec<PskKey>)`: no `Debug`/`Display`, `from_keys` gates empty/over-ceiling; derived keys/IVs/cookies/plaintext zeroized; proven by gating + redaction tests |
| M158 lookup secret coexists independently | `DestinationContext` carries both `lookup_secret` and `psk_auth`; `EncryptedPublicationConfig::{with_psk,with_secret_and_psk}` compose them; blinding still uses the secret exactly; proven by secret+PSK coexistence tests at parser and publication layers |
| no persistence in core | no store, file, or static holds PSKs; config is generation-local; replacement generations carry only their construction keys; proven by replacement-isolation reasoning plus no-fallback tests |
| `ELS2PSKA` HKDF-SHA256 52-byte schedule | `hkdf_sha256_52` + `derive_psk_client` (`psk \|\| subcredential \|\| published_BE`, `ELS2PSKA`, key32+IV12+ID8); proven by independent-HMAC KAT (§2) |
| fresh 32B auth cookie + auth salt per regenerated object | `encrypt_psk` fills inner/outer/auth salts + cookie from caller `CryptoRng` on every call; deterministic helper exists only for tests; proven by rollover-freshness test (old vs new outer differ) |
| exact client record layout | `clientID[8] \|\| ChaCha(clientKey, clientIV, authCookie)`; proven by KAT byte assertions |
| auth-cookie-bound L2, unchanged L1 | `derive_layer_keys_with_cookie` (`authCookie \|\| subcredential \|\| published_BE`, `ELS2_L2K`); L1 still (`subcredential \|\| published_BE`, `ELS2_L1K`); proven by L2-differs/L1-equal assertions |
| flags `0x03` | `PSK_LAYER1_FLAGS`; decrypt rejects any other flags byte; proven by KAT + tamper tests; no-auth `0x00` path untouched |
| randomized multi-client order | `encrypt_psk` Fisher-Yates shuffles record order with the caller RNG when `N>1`; deterministic helper preserves given order so tests prove order-independence; proven by forward/reverse decrypt-both tests + production double-encrypt tests |
| no-auth M157 path unchanged | `encrypt_no_auth*`/`decrypt_no_auth` untouched; PSK outers rejected by `decrypt_no_auth` and vice versa; proven by cross-rejection tests + full regression suites |
| lookup-secret-only M158 path unchanged | `blinded_day_material*` untouched; secret-only publication tests retained and passing |
| sole publication owner, no second scheduler | `LeaseSetManager::refresh_encrypted_outer` branches on `config.psk_auth`; rollover/day-only paths reuse the same config; no new timer/state machine; proven by rollover tests |
| build failure never falls back | `refresh_encrypted_outer` returns `Err` before state mutation; callers retry without emitting ordinary/empty/no-auth stores (`create_database_store` returns `None` without outer); proven by corrupt-inner failure test |
| stale generations cannot publish superseded material | key-scoped verification (`register_database_store` type+key match) retained; rollover rotates once per boundary and ignores stale-key acks; proven by stale-ack test |
| extended B32 `auth_required=true` | `server_destination_address` carries `(key, secret_required, auth_required)`; PSK bridge passes `auth_required=true` while `secret_required` reflects the lookup secret; ordinary path byte-for-byte unchanged; `events.rs` untouched; proven by address tests |
| no log/error/metric/event carries PSK bytes | parser rejection logs name the failure shape only; `DestinationContext` custom `Debug` omits all fields; `SamCommand` `Debug`/`Display` carry session IDs only; proven by redaction tests (residual socket-echo note in §11) |
| no-std | `cargo check -p emissary-core --no-default-features --features no_std` passes; new code uses `alloc`/`core` plus existing codecs/`zeroize`/`hmac`/`sha2` only |

## 2. Format conformance (plan §2 + frozen references)

| Item | Implementation | Source |
|---|---|---|
| Standard properties | `leaseSetType=5`, `leaseSetAuthType=2`, `leaseSetPrivKey=Base64(32B)`, `leaseSetClient.psk.N=[name:]Base64(32B)`; blank/missing base with auth 2 fails | Plan §2.1 / Java handler records |
| Base key role | base is always first logical authorized key; indexed entries additional; zero indexed valid | Plan §2.1 |
| Duplicate semantics | duplicates preserved exactly as configured, each consuming one 40-byte record and the 4096 budget; no dedup | Plan §2.1 + `68608bcc` semantics commit |
| Layer-1 flags | `0x03` (per-client auth + PSK scheme bits) | Plan §2.2 / ELS spec |
| Per-client schedule | `okm = HKDF-SHA256(authSalt, psk \|\| subcredential \|\| published_BE, "ELS2PSKA", 52)`; key `okm[0..32]`, IV `okm[32..44]`, ID `okm[44..52]`; `encCookie = ChaCha20(key, IV, authCookie)` | Plan §2.2 |
| Inner schedule | `HKDF-SHA256(innerSalt, authCookie \|\| subcredential \|\| published_BE, "ELS2_L2K", 44)` | Plan §2.2 |
| Outer schedule | `HKDF-SHA256(outerSalt, subcredential \|\| published_BE, "ELS2_L1K", 44)` (unchanged) | Plan §2.2 / M157 |
| Wire layout | `outerSalt[32] \|\| ChaCha_L1(0x03 \|\| authSalt[32] \|\| count_BE[u16] \|\| records(8+32) \|\| innerSalt[32] \|\| ChaCha_L2(0x03 \|\| innerLS2))` | Plan §2.2 |
| Size bound | `32+1+32+2+40*N+32+1+inner ≤ 4096` checked with checked arithmetic before per-client crypto | Plan §2.3 / Java `MAX_ENCRYPTED_SIZE` |
| Absolute entry ceiling | `(4096-100)/40 = 99` (`MAX_PSK_CLIENTS`); `u16` encoding never binding | Plan §2.3 |
| Freshness | fresh auth cookie + auth salt (+ inner/outer salts) per regenerated outer; randomized record order for `N>1` | Plan §2.2/§7 |
| Address flags | extended B32 `auth_required=true` for PSK generations; `secret_required` independent | Plan §8 |

KAT provenance (fixed `crypto::els2` fixtures: type-7 key
`8a88e3dd7409f195fd52db2d3cba5d72ca6709bf1d94121bf3748801b40f6f5c`,
day `20260909`, published `1788000000`, inner 16 bytes
`deadbeef00112233445566778899aabb`, salts `0x11/0x22/0x33`, cookie
`0x44`, PSKs `0xA1/0xB2`):

- subcredential recomputed through the frozen `credential`/`subcredential`
  helpers and pinned by known-answer test;
- `ELS2PSKA` 52-byte outputs pinned against an independent local
  HMAC-SHA256 extract+expand recomputation (`independent_hkdf_52`), not
  the helper under test;
- client IDs, encrypted cookies, L1/L2 key splits, complete outer
  ciphertext layout, and `outer.len() == psk_outer_len(inner, N)` asserted
  byte-for-byte;
- outer Red25519 verification proven at the publication layer:
  `EncryptedLeaseSet2::parse` (which verifies `0x05 || prefix` via M156)
  accepts every PSK outer produced by `LeaseSetManager`, and
  `decrypt_psk` recovers the exact inner bytes under each authorized PSK.

Decoder/negative matrix (all fail closed): missing base, malformed
Base64, non-32-byte keys, sparse indices (`psk.1` without `psk.0`,
`psk.0+psk.2` without `psk.1`), mixed DH entries, wrong auth selectors
(`0/1/00/hello`, missing auth with base present), legacy flag,
unpublished flag, incompatible key companions, multi-colon entries,
wrong PSK/no client match, tampered ID/cookie/auth-salt/flags, wrong
published, wrong subcredential (wrong-secret equivalent), checked-size
overflow, `>4096` outer input, corrupt inner, stale day/material acks,
and both cross-mode fallbacks (`decrypt_no_auth` on PSK outer,
`decrypt_psk` on no-auth outer).

Reference interoperability: no Java fixture checkout was performed
(read-only boundary). Interop evidence is the independent-HMAC KAT
recomputation of the frozen plan §2.2 schedule plus
`EncryptedLeaseSet2` signature-verify + `decrypt_psk` round-trips at the
publication layer. Deterministic reference randomness is not required by
the plan when derivation/decryption evidence is stronger; that condition
is met. No production client resolver was added.

## 3. Dependency/security review (M155 §6 freeze)

- No new dependency. Reused: `hmac`/`sha2` (HKDF extract+expand),
  `chacha20` via frozen `ChaCha::with_iv` (counter-1 seek, unchanged
  file), `rand` RNG (`RngCore`/`CryptoRng` only), `zeroize`
  (`Zeroizing` keys/cookies/inputs/plaintext + scrub-on-reject),
  `data-encoding` I2P Base64 (existing codecs), M156 Red25519 (unchanged),
  M157 ELS2/rollover owner, M158 secret/B32 stack, `bytes`/`nom`
  (existing wire owners).
- `Cargo.toml`, `emissary-core/Cargo.toml`, `Cargo.lock` unchanged in this
  commit (verified by `git status --short`; M062 records the zero budget).
- No Yosemite change; no `emissary-cli/src/i2pcontrol/**` production change.
- `no_std + alloc` compatible; proven by the `no_std` check above.
- Constant-time/zeroization: Dalek ops via M156; `PskKey`/`PskAuthorization`
  zeroized on drop with no `Debug`/`Display`; derived client keys/IVs,
  auth cookie copies, HKDF inputs/outputs, and L1/inner plaintext buffers
  zeroized after use; `LookupSecret`/`SigningSeed`/blinded-private
  handling unchanged; no PSK material in logs, errors, metrics, or events.
- Bounds before crypto: SAM framing bounds the properties; every key
  decodes to exactly 32 bytes before activation; indexed count bounded by
  `MAX_PSK_CLIENTS` before decoding; complete `psk_outer_len` computed
  with checked arithmetic before any HKDF/ChaCha work; oversize fails
  closed without dropping entries.
- Domain separation: unchanged `ELS2_L1K`/`ELS2_L2K` labels plus new
  `ELS2PSKA` label; sigtype codes, `YYYYMMDD`, CRC polynomial, flag bits
  unchanged.
- Fail-closed: malformed/sparse/mixed/over-ceiling inputs, bad
  flags/sigtypes/checksums/keys/lengths, expired inners, wrong-type/key
  verification replies, stale-key acks, and crypto/build failures all
  reject before network or storage effect with no ordinary, empty-secret,
  no-auth, or type-3 fallback.
- Ordinary Ed25519/LS2/DatabaseStore-type-3/NetDB-ordinary/M135/M145 SAM
  behavior preserved (see §5).

## 4. Exact-path reconciliation (M061/M062)

M061 (`061-containment-boundary.toml` + `m061_containment.rs`):

- No change required and none made. All four M159 production paths are
  already individually present in the realized exact `[allowed]` ledger
  from M060/M157; M159 is a strict active subset frozen by the M159 plan
  and the registry. `registered_pending` handling is untouched.

M062 (`062-dependency-containment.toml` + `m062_dependency_containment.rs`):

- `062-dependency-containment.toml` `[current_registration]` bookkeeping
  closed: milestone `M159` realized with the exact four production paths,
  zero new files, `new_direct_dependencies=[]`, `manifest_changes=[]`,
  `lockfile_change=false`, `yosemite_change=false`,
  `i2pcontrol_source_change=false`.
- `m062_dependency_containment.rs` gains narrow `is_authorized_m159_path`
  covering exactly the four production files plus the M159 closure/plan/
  registry/README/roadmap/AGENTS/docs set, wired into both the
  allowed-path and prohibited-pattern assert chains (same pattern as the
  M158 helper; authorizes the new closure file, which no earlier helper
  covers).
- `policy_terms_do_not_leak_into_non_policy_production_paths` passes: new
  core source contains no `Proposal 170`/`JsonRpc`/`jsonrpc`/
  `TunnelManagerControl`/`control-state.json` strings.

## 5. Verification outcomes

| Command group | Result |
|---|---|
| `cargo check -p emissary-core` | **pass** |
| `cargo check -p emissary-core --no-default-features --features no_std` | **pass** |
| `cargo test -p emissary-core --lib --no-fail-fast` | **pass**: `1141 passed, 2 ignored` (1124 pre-existing + 17 new M159 tests, zero regressions) |
| `cargo check -p emissary-cli --no-default-features --features i2pcontrol` | **pass** |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --lib --no-fail-fast` | **pass**: `821 passed` |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment --test m062_dependency_containment --test m095_full_support_matrix --test m105_residual_option_audit --no-fail-fast` | **pass**: `8 + 23 + 3 + 1` across 4 suites |
| M095 mechanical recomputation (via `m095_full_support_matrix` suite) | **pass**: `840/336/29/475` == declared |
| `cargo clippy -p emissary-core --all-targets -- -D warnings` | **pass**: No issues found |
| `cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings` | pre-existing failure only in unrelated `proxy.rs` path; zero warnings from M159 files (see §11) |
| stable `rustfmt` on M159-touched Rust files | **pass** for all four production files plus the M062 guard test (repo nightly-only options warn; `max_width = 100` observed; no new nightly drift introduced — see §11) |
| `git diff --check` | **pass** |

Focused M159 suites (17 tests): `crypto::els2` (6: bounded-type gating,
checked-length ceiling, single-client KAT vs independent HMAC, multi-client
order-independence + production randomization, duplicate preservation +
size-boundary, full negative matrix without fallback);
`destination::lease_set` (4: PSK publication decrypt + 4096 bound +
cross-rejection, rollover freshness + stale-ack + type-gated verification,
secret+PSK composition, build-failure no-fallback);
`sam::parser` (6: base-only accept+redaction, indexed order + name
stripping, duplicate preservation, secret coexistence, negative matrix,
debug-surface redaction); `sam::session` (1: extended-B32 `auth_required`
emission for both secret states).

M157/M158/ordinary regressions: all pre-existing no-auth type-5,
lookup-secret type-5, B32 flags/CRC, UTC-rollover, type-preserving NetDB
storage/flooding, and ordinary type-3 LS2 tests retained and passing
(1124 pre-existing core + 821 CLI lib, zero failures).

## 6. Changed paths (exact budget only)

Production (4):

- `emissary-core/src/crypto/els2.rs` (+~700/−12: `PskKey`,
  `PskAuthorization`, `PSK_LAYER1_FLAGS`/`ELS2PSKA`/`MAX_ENCRYPTED_DATA_LEN`/
  `MAX_PSK_CLIENTS`, generic HKDF + 52-byte schedule, cookie-bound L2,
  `psk_outer_len`, `encrypt_psk_with_salts`/`encrypt_psk`,
  `decrypt_psk`, 6 tests);
- `emissary-core/src/destination/lease_set.rs` (+~250/−12:
  generation-local PSK ownership (`with_psk`/`with_secret_and_psk`),
  PSK branch in `refresh_encrypted_outer` with fresh RNG cookie/salts,
  4 tests plus helper);
- `emissary-core/src/sam/parser.rs` (+~530/−10: PSK entry/base decoders,
  `extract_psk_authorization` with redaction, `is_valid_type5_psk`,
  extended type-5 gate, context PSK field, 6 tests);
- `emissary-core/src/sam/session.rs` (+~85/−7: PSK threading into
  publication config, three-flag extended-B32 event emission, address test).

Guards/ledgers (2):

- `emissary-cli/tests/m062_dependency_containment.rs` (M159 authorization
  helper in both chains);
- `plans/implementation/i2pcontrol-proposal-170/062-dependency-containment.toml`
  (current-registration closure bookkeeping, zero-budget recorded).

Planning/closure/registry (8 + 1 note):

- `plans/implementation/i2pcontrol-proposal-170/159-leaseset-psk-client-authorization-primitive.md`
  (Status `registered / dependency-ready` → `closed as complete` with
  closure link);
- new `plans/closure/i2pcontrol-proposal-170/159-closure.md` (this file);
- `plans/registry.md` (M159 → closed; no registered successor; M160
  deferred with satisfied hard dep noted);
- `plans/implementation/i2pcontrol-proposal-170/README.md` (M159 closed
  section; chain updated);
- `plans/subsystems/i2pcontrol-proposal-170-post-m154-leaseset-security-corrective-roadmap.md`
  (M159 closed; graph updated);
- `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md`
  (M159 closed; handoff updated);
- `AGENTS.md` (M159 closed entry; chain updated; duplicate-preservation
  wording corrected — see §11);
- `plans/implementation/i2pcontrol-proposal-170/160-leaseset-dh-client-authorization-primitive.md`
  (deferred-status note: M159 hard dependency satisfied, registration
  pending; no registration).

Docs (3, authority wording only):

- `docs/i2pcontrol/README.md`, `docs/i2pcontrol/proposal-170-support.md`,
  `docs/i2pcontrol/tunnel-manager.md` (M159 closed authority + `336/29/475`
  wording retained; LeaseSet residual-blocked statements retained).

`061/062`, `095/105/110` semantics: `061` unchanged (strict subset, no
broad waiver); `062` gains only the exact M159 closure authorization
above; `095/105/110` intentionally **unchanged** (zero promotions; see
§8).

## 7. Requirement-to-evidence matrix (planning process §2.5)

| Closure duty | Evidence |
|---|---|
| implementation commits | one commit landing with this closure (production + tests + M062 guard + planning records); exact paths in §6; no prior partial implementation commit (registration commits touched only plans/TOMLs/tests) |
| invariant review | partial-support, no-fabrication, I2P-only egress, fail-closed option validation, containment, and Y005 invariants re-proved by green `m061`/`m062`/`m095`/`m105` guards; no invariant weakened; M147/M148 not reopened (type-11 derived-only, gates enforced); M146 untouched (no egress); no downgrade path exists (fallible derivation, no no-auth/ordinary fallback, stale-key rejection) |
| failure/recovery and contention evidence | malformed/sparse/mixed/over-ceiling inputs reject pre-allocation; oversize publication fails at checked-size with retry and no fallback; wrong-type/key verification ignored; stale-key acks ignored; expired/invalid inner never published; rollover is single-owner timer/state with no second task; no lock spans I/O or crypto; shutdown drops the owner-local timer; successor generations receive no predecessor keys (config is generation-local); tamper/wrong-PSK/flag/type/length tests prove closed rejection |
| compatibility, migration, security review | no wire change to type-3/RouterInfo/lookup protocol; PSK + auth-required B32 are additive; no storage migration (opaque DHT keys, bounded cache); no new attack surface beyond reviewed parser/codec/derivation composition; all 15 LeaseSet cells stay fail-closed (matrix unchanged); legacy AES still blocked; `cargo clippy -p emissary-core -D warnings` clean; `emissary-cli -D warnings` failure is pre-existing and unrelated with zero M159-file warnings |
| documentation and operational evidence | §6 changed paths; authority docs name M159 closed and retain partial support; operational impact none beyond the opt-in standard PSK properties and the corrected type-5 server address form (default ordinary path byte-compatible) |
| M159 acceptance (plan closure evidence) | exact changed files/dependencies (§6), standard-property evidence (§1), PSK alpha/cookie/schedule known answers (§1/§2), extended-B32 auth vectors (§1/§5), PSK-redaction audit (§1), rollover/recreation traces (§1/§5), no-downgrade evidence (§1/§5), unchanged M095 (§8), implementation SHA (commit landing with this closure), M160 readiness (§10) |

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
  five-family integration; DH remains M160.

## 9. Production-head determination

- Last production-bearing commit before M159: the M158 closure commit
  ancestry (unchanged through M159 registration at `6228782d`).
- M159 is production-bearing (exact §6 files); after the closure commit
  lands, the last production-bearing head becomes that commit.
- `git log 6228782d..HEAD` over `emissary-core/src`,
  `emissary-cli/src`, `emissary-util/src`, all manifests and `Cargo.lock`
  at closure time contains only the staged M159 exact-budget diff
  (4 production paths, `1572 insertions, 37 deletions`; Rust production
  delta is four exact-owner extensions plus the M062 guard exact
  amendment).
- `git diff --name-only` over manifests/lockfile is empty: no `Cargo.toml`,
  `emissary-core/Cargo.toml`, or `Cargo.lock` change. `git status --short`
  shows only the §6 paths.
- Streamr datagram limits (16-subscriber, 60s expiry, 1200-byte payload,
  4095-byte transport-buffer, 15s refresh, bounded shutdown,
  loopback-only UDP) untouched; remote datagrams never choose a local UDP
  destination.

## 10. Registry updates and future-plan unblock determination

Applied alongside this closure (see §6 for the file list):

- `159-*.md` plan: Status `registered / dependency-ready` → `closed as
  complete` with closure link;
- `plans/registry.md`: M159 → closed as complete (`336/29/475` unchanged,
  zero promotions, exact production diff); **no registered successor**;
- `110-completion-ledger.toml`: no new entry (zero promotions);
- `m061`: unchanged (strict subset; no broad waiver);
- `m062`: exact M159 closure authorization only (§4); no broad waiver.

Future-plan unblock determination (as required by the tasking):

- **M159 CLOSED as complete.** Its sole hard dependency (M158 closure with
  frozen publication/rollover owner and zero promotions) was satisfied,
  and all plan requirements are met with exact-budget production diff and
  zero matrix promotions. No stop condition in plan §12 triggered: no
  outside file or dependency was required, no I2PControl persistence
  change was needed for the neutral primitive, no NetDB subsystem was
  added, PSK data leaves generic debug-capable state before activation,
  the extended-B32 auth flag needed no broader naming subsystem, reference
  vectors agree with local behavior, and no failure downgrades to
  no-auth or ordinary publication.
- **M160 REMAINS deferred/unregistered (hard dependency now satisfied,
  registration pending).** M159 closure satisfies its hard dependency, and
  the pre-frozen envelope revalidates cleanly: the same four exact files
  (`els2.rs`, `lease_set.rs`, `parser.rs`, `session.rs`) are the realized
  M159 owners with no fifth path required, and the existing `x25519-dalek`
  workspace dependency (already in `emissary-core/Cargo.toml`) remains
  sufficient with no manifest/lock change. The M160 plan file gains only a
  deferred-status note recording the satisfied hard dep and the
  revalidated envelope; it is **not** registered by this closure. Only
  M160 may be registered next, with exact M061/M062 authorization in that
  registration commit, plus explicit all-zero shared-secret rejection at
  the ELS2 boundary. No generic exact-path research milestone is required.
- **M161 remains deferred/unregistered**, hard-gated on M160 closure
  (sequenced after M160 per the roadmap; M155 disposition already
  satisfied). It may now be prepared but must not overtake M159-M160
  implementation. Its A/B/C outcome gates M162 `EncryptLeaseSet`
  promotion.
- **M162 remains deferred/unregistered**, hard-gated on M160 closure plus
  M161 disposition (plus any M161-A successor closure before
  `EncryptLeaseSet` promotion). It inherits the standard-property mapping
  (`i2cp.leaseSetAuthType=2` + base/indexed PSK consumer proven here;
  Proposal `LeaseSetClientAuths` naming/persistence mapping still to
  build).
- **M152 remains deferred/unregistered**, re-gated on M162 closure plus any
  M161-A successor. Its final qualification must now account for the M155
  field-coupling rule plus the M156/M157/M158/M159 primitive availability
  and the M159/M160 authenticated bounds.
- **M149-M151 remain superseded/unregistered (do not execute).** Confirmed
  fully superseded by M155-M162; no re-gating or revival is authorized.
- **M147/M148 remain closed-blocked/deferred-behind-M147** (M154
  disposition C); this line does not reopen them. Type-11 blinding is
  explicitly not a `SigType` reopening.
- **M146 remains closed as blocked** with no successor; it is not reopened
  by M159 or by any future LeaseSet tail.
- No other future-plan status required a change. File presence alone never
  authorises production work.

## 11. Unresolved findings

- Clarification (planning-record correction, applied): `AGENTS.md` summarized
  the M159 parser contract as rejecting "duplicate PSK bytes", while the
  frozen M159 plan §2.1/§5.2 and the `68608bcc` semantics commit require
  preserving duplicate configured entries exactly as the pinned Java
  runtime does (each duplicate consumes one record and the 4096 budget).
  Implementation follows the plan (duplicates preserved; see §1/§2 tests).
  `AGENTS.md` wording is corrected in this commit to match the realized
  contract. Severity: planning-wording only; no code impact.
- Non-blocking: `cargo test -p emissary-cli --no-default-features
  --features i2pcontrol --test m153_post_m146_requalification` reports
  `AGENTS.md must retain M139 as historical evidence`. Fails identically at
  the pre-M159 registration baseline; unrelated to M159 files. Severity:
  informational; no corrective pass required (out of M159 exact-budget
  scope; must not be smuggled into this line).
- Non-blocking: `cargo clippy -p emissary-cli --no-default-features
  --features i2pcontrol --all-targets -- -D warnings` reports a
  pre-existing lint in `backends/filters/proxy.rs:60` (`chunks_exact`),
  untouched by M159; M159 files contribute zero warnings. Severity:
  informational; no corrective pass required (out of M159 exact-budget
  scope).
- Non-blocking: `cargo fmt --all -- --check` (stable or nightly) reports
  drift in files outside the M159 budget at the registration baseline;
  M159-touched Rust files are stable-`rustfmt` clean with `max_width = 100`
  observed, and no new nightly drift was introduced by M159 lines
  (remaining nightly hunks are pre-existing comment reflow from a newer
  toolchain). Severity: informational; unrelated files were not touched.
- Informational: rejected-command logging in `sam/socket.rs` (outside the
  M159 budget, intentionally untouched) echoes the raw command line when
  `SamCommand::parse` fails. A *malformed*-PSK `SESSION CREATE` would
  therefore echo its rejected option value to the local log; valid-PSK
  sessions never hit that path (parse succeeds, nothing is logged).
  Severity: low/informational; secret hygiene for that generic socket path
  is noted for a future hardening pass and must not be smuggled into this
  line.

## Internal-only / read-only-upstream attestation

- External sources (the already-pinned Proposal/Java/Yosemite revisions and
  the Encrypted-LeaseSet/Red25519 specifications cited from existing
  M155/M156/M157 records) were accessed read-only for evidence; no new
  upstream fetch, and no Java source checkout, was performed for this
  closure — PSK vectors were cross-checked against an independent local
  HMAC recomputation of the frozen plan schedule plus
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
