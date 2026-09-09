# M158 Closure — LeaseSet Lookup-Secret and Blinded-Address Primitive

Status: **closed as complete; M159 hard dependency satisfied, amendment pending**

Date: `2026-09-09`

Source implementation plan:

- `plans/implementation/i2pcontrol-proposal-170/158-leaseset-lookup-secret-and-blinded-address-primitive.md`
  (Status now `closed as complete` with closure link; hard dependency on
  M157 closure satisfied by `plans/closure/i2pcontrol-proposal-170/157-closure.md`;
  zero promotion budget observed as proven below).

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-post-m154-leaseset-security-corrective-roadmap.md`.

Promotion budget: **zero Proposal cells**.

Production budget: **exact four-path M158 set only**
(`emissary-core/src/crypto/els2.rs`,
`emissary-core/src/destination/lease_set.rs`,
`emissary-core/src/sam/parser.rs`,
`emissary-core/src/sam/session.rs`),
all already individually present in M061's realized exact allowlist, with
M062 current-registration bookkeeping closed in the same commit. No new
production file. No Cargo, lockfile, Yosemite, or
`emissary-cli/src/i2pcontrol/**` production change.

## Planning baseline

- Registration baseline `4dad28f9` (registered M158 handoff; clean worktree
  before M158 implementation).
- Last production-bearing head before M158: the M157 closure commit ancestry
  (`a18fba7fa307d195f2a6e7c2cae57c554b07eba8` research head cited by the
  M158 plan; unchanged through M158 registration commits, which touched
  only plans, TOMLs, and containment tests).
- Incoming M095 matrix: `336 apply / 29 blocked_primitive / 475
  not_applicable` across 840 TunnelManager option/family cells.
- M153 closed complete as the current runtime/security qualification
  authority; M154 closed complete disposition C (M147 path blocked);
  M155 closed complete with zero production; M156 closed complete with zero
  promotions; M157 closed complete with zero promotions; M146 closed
  blocked.

Reviewed head:

- M158 work completes on the working tree described in §9. Exact changed
  production paths are the four registered paths above (exact-owner
  extensions, no new file). Exact guard/planning paths are
  `emissary-cli/tests/m062_dependency_containment.rs` (M158 authorization
  helper only),
  `plans/implementation/i2pcontrol-proposal-170/062-dependency-containment.toml`
  (current-registration closure bookkeeping), the M158 plan (status flip),
  the M159 plan (deferred-status note only), this closure, registry/README/
  roadmaps/AGENTS/docs authority updates (see §6).

Pinned authority (all accessed read-only; no upstream mutation, contact, or
submission occurred — see read-only attestation):

- Proposal 170 revision `2026-05-20`, status Open, SHA-256
  `f13ae00b886c5e72131bc5d5b138a371148d1faa6899a119a1dacb65a555e7dc`
  (cited from M095/M153/M155; no new Proposal fetch — M158 adds no Proposal
  policy);
- Direct Java I2P/I2PTunnel standard-property contract
  `i2cp.leaseSetSecret = Base64(UTF8(secret))` (cited from the frozen M158
  plan §2.1 records: `TunnelConfig.setBlindedPassword`,
  `RequestLeaseSetMessageHandler`, `ClientMessageEventListener`; no new
  fetch — the plan representation is consumed as the standard property);
- I2P Encrypted LeaseSet specification, blinding calculations and
  encrypted-service Base32 address sections
  (`https://geti2p.net/en/docs/specs/encryptedleaseset`, cited from
  M155/M157 records; no new fetch — the frozen M158 plan §2.3 format is
  authoritative for this milestone);
- Java `net.i2p.crypto.Blinding` encode/decode contract (cited from the
  frozen M158 plan §2.3/§8 records; no new fetch — byte-exact vectors below
  are pinned against the plan-frozen format plus an independent local
  recomputation, see §2);
- I2P Red25519 specification (cited from M156 records; no new fetch — M156
  remains the blinding authority and `crypto/red25519.rs` is unchanged);
- Java I2PControl Proposal-170 head `45bb593000408071dd376b78848fdc246dccd964`
  and Java I2P snapshot `2c3fd2a9532cd86ec06cb6f2b9f3f813ca752243` (cited
  from pinned M155 records; no new fetch);
- Yosemite optional exact revision `59140a2277bf296928d2e8ce39a148182eeff044`
  (unchanged; no Yosemite change in M158);
- Emissary source at the reviewed tree (all file:line citations below read
  at HEAD plus the workdir diff closed here).

Current Proposal matrix at closure (mechanically recomputed, dispositions
unchanged):

- `336 apply / 29 blocked_primitive / 475 not_applicable` (840 cells);
- residual split `SigType` 10, `EncryptLeaseSet` 5, `OptionalLookup` 5,
  `LeaseSetClientAuths` 5, `UseOutproxyPlugin` 4 — identical to
  M153/M154/M155/M156/M157 entry.

## 1. Requirement-to-evidence matrix (plan requirements)

| Plan requirement | Evidence |
|---|---|
| standard property Base64(UTF-8) before activation | `parse_lookup_secret_value` (`sam/parser.rs`): blank → empty secret; otherwise I2P-Base64 decode then UTF-8 validate; malformed Base64/non-UTF-8 rejects before `SamCommand` construction; proven by accept + malformed rejection tests |
| no new semantic length limit, no truncation | parser imposes no length bound; `LookupSecret::from_bytes` stores valid UTF-8 of any framed length without truncation; overlong inputs fail closed at derivation (see below); proven by 65-byte stored-without-truncation test |
| dedicated zeroizing/non-`Debug` secret type | `LookupSecret` (`crypto/els2.rs`): `Zeroizing<Vec<u8>>`, no `Debug`/`Display`, UTF-8 gating constructor that scrubs on failure, `empty`/`as_bytes`/`is_empty`/`len`; proven by gating test |
| secret removed from generic option state | extraction via `key_value_pairs.remove("i2cp.leaseSetSecret")` before `options` map construction on both `SESSION CREATE` paths; options map retains no secret in either form; proven by absence assertions plus `Debug` redaction tests |
| secret travels in redacted `DestinationContext` | `lookup_secret` field added; custom `Debug` still `finish_non_exhaustive` (omits all fields); `PartialEq` extended; both TRANSIENT and persistent constructors carry it; `ConnectionKind::Session`/`PendingSamSession` move it untouched (no file change needed outside budget) |
| no log/error/metric/event carries secret bytes | parser rejection logs name the failure shape only, never the value; no new log carries secret material; `Debug` redaction tests prove command/context surfaces are clean (residual socket-echo note in §11) |
| secret-aware daily blinding | `blinded_day_material_with_secret` composes frozen M156 `generate_alpha` with `secret.as_bytes()` plus agreement check; empty secret delegates exactly to the unchanged `blinded_day_material`; proven by equivalence + direct-alpha known-answer tests |
| same triple deterministic; different secrets differ | determinism re-derivation test; secret-vs-empty and secret-vs-secret inequality tests on blinded public and storage keys |
| generation-local ownership, rollover-safe | `EncryptedPublicationConfig::{new,with_secret}` owns seed/pubkey/secret for one generation; `refresh_encrypted_outer` and `refresh_encrypted_day_only` both derive through the secret-aware helper; rollover keeps the same config; proven by rollover-retains-secret test |
| no downgrade on failure | derivation/build failure returns `Err` before state mutation; callers retry without emitting ordinary or empty-secret stores (`create_database_store` returns `None` without outer); proven by overlong-secret and corrupt-inner failure tests |
| no secret persistence in core | no store, file, or static holds the secret; replacement generations carry only their construction secret; proven by replacement-does-not-inherit test |
| extended-B32 type-7 → type-11 codec | `encode_encrypted_service_b32`/`decode_encrypted_service_b32` (`crypto/els2.rs`): flags bit 1 secret / bit 2 auth, reserved/two-byte rejection, sigtypes exactly 7/11, IEEE CRC-32 XOR header, 35-byte/56-char shape, M156-gate pubkey validation; proven by pinned vectors + adversarial suite (§2) |
| local CRC helper, no new dependency | `crc32_ieee` bitwise helper; pinned against `"123456789" → 0xCBF43926`; no manifest/lockfile change |
| codec round-trips auth flag for M159/M160 | combined-flags vector round-trips; runtime emission fixes `auth_required=false` |
| server-destination event emits extended B32 | `server_destination_address` helper + activation wiring in `sam/session.rs`; `secret_required=true` iff secret nonempty; ordinary path byte-for-byte unchanged; `events.rs` untouched; proven by address-selection tests |
| type-5 auth/PSK/DH/legacy companions still rejected | `is_valid_type5_no_auth` extended only for valid secrets; legacy flag, auth-type nonzero, key companions, per-client entries, unpublished-type-5, and bad type selectors still reject; existing rejection tests retained and passing |
| no-std | `cargo check -p emissary-core --no-default-features --features no_std` passes; new code uses `alloc`/`core` plus existing codecs/`zeroize` only |

## 2. Format conformance (plan §2 + frozen references)

| Item | Implementation | Source |
|---|---|---|
| Standard property | `Base64(UTF8(secret))`, blank → empty, malformed → reject pre-allocation | Plan §2.1 / Java I2PTunnel records |
| Alpha with secret | `GENERATE_ALPHA(A, YYYYMMDD, secret_utf8_bytes)` via frozen `red25519::generate_alpha` | M156 primitive, unchanged file |
| B32 payload | `flags \|\| 0x07 \|\| 0x0b \|\| 32B pubkey` (35 bytes) | Plan §2.3 / Java `Blinding` records |
| B32 flags | bit 1 secret-required, bit 2 auth-required, bit 0 and bits 7..3 reject | Plan §2.3 |
| B32 checksum | `crc = CRC-32(pubkey)`; `wire[0..3] = (flags,7,11) XOR crc[0..3]` | Plan §2.3 / `java.util.zip.CRC32` |
| B32 text | I2P-Base32 56 chars + `.b32.i2p`; ASCII case-insensitive decode | Plan §2.3 |
| B32 validation | exact suffix/length, reverse-XOR header, sigtypes 7/11, M156-gate pubkey check, ordinary-B32/trailing/checksum rejections | Plan §8 |

Vector provenance: fixture type-7 public key
`8a88e3dd7409f195fd52db2d3cba5d72ca6709bf1d94121bf3748801b40f6f5c`
(from the M157 ELS2 fixture suite), CRC `0xe11647f0` over the key,
XORed header `f0/40/1d` (plain), pinned labels independently recomputed
with `binascii.crc32` plus RFC-4648 Base32 (lowercased I2P alphabet):

- plain:
  `6bab3cui4poxicprsx6vfwznhs5f24wkm4e36hmucin7g5eiag2a6324.b32.i2p`;
- secret-required:
  `6jab3cui4poxicprsx6vfwznhs5f24wkm4e36hmucin7g5eiag2a6324.b32.i2p`;
- auth-required:
  `6rab3cui4poxicprsx6vfwznhs5f24wkm4e36hmucin7g5eiag2a6324.b32.i2p`.

No upstream fetch was performed for these vectors (internal-only
boundary); the independent Python recomputation of the frozen plan format
plus the IEEE check value is the cross-check. Decoder adversarial cases
(ordinary-B32, suffix, Base32, length, reserved/two-byte flags, wrong
sigtypes, zero key, header/key corruption, trailing bytes, uppercase
acceptance) all behave as specified.

## 3. Dependency/security review (M155 §6 freeze)

- No new dependency. Reused: M156 `red25519.rs` (unchanged),
  `crypto::{base32_decode, base32_encode, base64_decode}` (existing I2P
  codecs), M157 ELS2/rollover owner, `zeroize` (`Zeroizing` secret +
  scrub-on-reject), `hmac`/`sha2` via the frozen primitive, `bytes`/`nom`
  (existing wire owners).
- `Cargo.toml`, `emissary-core/Cargo.toml`, `Cargo.lock` unchanged in this
  commit (verified by `git status --short`; M062 records the zero budget).
- No Yosemite change; no `emissary-cli/src/i2pcontrol/**` production change.
- `no_std + alloc` compatible; proven by the `no_std` check above.
- Constant-time/zeroization: Dalek ops via M156; `LookupSecret` zeroized on
  drop and scrubbed on rejected construction; `SigningSeed`/blinded
  private handling unchanged; derived alpha/keys remain one-shot locals;
  no secret material in logs, errors, metrics, or events.
- Bounds before crypto: SAM framing bounds the property; UTF-8 validated
  before activation; 65-byte-and-longer secrets fail closed at derivation
  through the frozen 64-byte primitive work bound without truncation and
  without downgrade (no new semantic limit invented at the parser layer).
- Domain separation: unchanged M156/M157 labels, sigtype codes,
  `YYYYMMDD`, CRC polynomial, flag bits.
- Fail-closed: malformed Base64/UTF-8, invalid companions, bad
  flags/sigtypes/checksums/keys/lengths, expired inners, wrong type/key
  verification replies, stale-key acks, and crypto/build failures all
  reject before network or storage effect with no ordinary or
  empty-secret fallback.
- Ordinary Ed25519/LS2/DatabaseStore-type-3/NetDB-ordinary/M135/M145 SAM
  behavior preserved (see §5).

## 4. Exact-path reconciliation (M061/M062)

M061 (`061-containment-boundary.toml` + `m061_containment.rs`):

- No change required and none made. All four M158 production paths are
  already individually present in the realized exact `[allowed]` ledger
  from M060/M157; M158 is a strict active subset frozen by the M158 plan
  and the registry. `registered_pending` handling is untouched.

M062 (`062-dependency-containment.toml` + `m062_dependency_containment.rs`):

- `062-dependency-containment.toml` `[current_registration]` bookkeeping
  closed: milestone `M158` realized with the exact four production paths,
  zero new files, `new_direct_dependencies=[]`, `manifest_changes=[]`,
  `lockfile_change=false`, `yosemite_change=false`,
  `i2pcontrol_source_change=false`.
- `m062_dependency_containment.rs` gains narrow `is_authorized_m158_path`
  covering exactly the four production files plus the M158 closure/plan/
  registry/README/roadmap/AGENTS/docs set, wired into both the
  allowed-path and prohibited-pattern assert chains (same pattern as the
  M157 helper; authorizes the new closure file, which no earlier helper
  covers).
- `policy_terms_do_not_leak_into_non_policy_production_paths` passes: new
  core source contains no `Proposal 170`/`JsonRpc`/`jsonrpc`/
  `TunnelManagerControl`/`control-state.json` strings.

## 5. Verification outcomes

| Command group | Result |
|---|---|
| `cargo check -p emissary-core` | **pass** |
| `cargo check -p emissary-core --no-default-features --features no_std` | **pass** |
| `cargo test -p emissary-core --lib --no-fail-fast` | **pass**: `1124 passed, 2 ignored` (1112 pre-existing + 12 new M158 tests, zero regressions) |
| `cargo check -p emissary-cli --no-default-features --features i2pcontrol` | **pass** |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --lib --no-fail-fast` | **pass**: `821 passed` |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment --test m062_dependency_containment --test m095_full_support_matrix --test m105_residual_option_audit --no-fail-fast` | **pass**: `8 + 23 + 3 + 1` across 4 suites |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m153_post_m146_requalification` | pre-existing failure only (`AGENTS.md must retain M139`; fails identically at the registration baseline — see §11) |
| M095 mechanical recomputation (via `m095_full_support_matrix` suite) | **pass**: `840/336/29/475` == declared |
| `cargo clippy -p emissary-core --all-targets -- -D warnings` | **pass**: No issues found |
| `cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings` | pre-existing failure only in unrelated `proxy.rs` path; zero warnings from M158 files (see §11) |
| stable `rustfmt` on M158-touched Rust files | **pass** for all four production files plus the M062 guard test (repo nightly-only options warn; `max_width = 100` observed; no new nightly drift introduced — see §11) |
| `git diff --check` | **pass** |

Focused M158 suites (12 tests): `crypto::els2` (5: secret gating,
secret-aware derivation/determinism/sensitivity/overlong fail-closed,
CRC check value, pinned extended-B32 vectors, adversarial decoder);
`destination::lease_set` (3: secret-derived publication key + decrypt,
rollover retains secret, overlong-secret no-fallback + replacement
isolation); `sam::parser` (2 updated/extended gate tests + redaction and
absent/empty-secret tests); `sam::session` (2: ordinary address unchanged,
extended emission with both flag states).

## 6. Changed paths (exact budget only)

Production (4):

- `emissary-core/src/crypto/els2.rs` (+462/−9: `LookupSecret`,
  `blinded_day_material_with_secret`, CRC helper, extended-B32
  codec/metadata, 5 tests);
- `emissary-core/src/destination/lease_set.rs` (+191/−10:
  generation-local secret ownership, secret-aware current-day/rollover
  derivation, 3 tests plus helper);
- `emissary-core/src/sam/parser.rs` (+154/−10: standard-property
  decode/validate/extract/remove, extended type-5 gate, context secret
  field, redaction tests);
- `emissary-core/src/sam/session.rs` (+95/−7: secret threading into
  publication config, extended-B32 event emission helper, address tests).

Guards/ledgers (2):

- `emissary-cli/tests/m062_dependency_containment.rs` (M158 authorization
  helper in both chains);
- `plans/implementation/i2pcontrol-proposal-170/062-dependency-containment.toml`
  (current-registration closure bookkeeping, zero-budget recorded).

Planning/closure/registry (8 + 1 note):

- `plans/implementation/i2pcontrol-proposal-170/158-leaseset-lookup-secret-and-blinded-address-primitive.md`
  (Status `registered / dependency-ready` → `closed as complete` with
  closure link);
- new `plans/closure/i2pcontrol-proposal-170/158-closure.md` (this file);
- `plans/registry.md` (M158 → closed; no registered successor; M159
  deferred with satisfied hard dep noted);
- `plans/implementation/i2pcontrol-proposal-170/README.md` (M158 closed
  section; chain updated);
- `plans/subsystems/i2pcontrol-proposal-170-post-m154-leaseset-security-corrective-roadmap.md`
  (M158 closed; graph updated);
- `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md`
  (M158 closed; handoff updated);
- `AGENTS.md` (M158 closed entry; chain updated);
- `plans/implementation/i2pcontrol-proposal-170/159-leaseset-psk-client-authorization-primitive.md`
  (deferred-status note: M158 hard dependency satisfied, amendment pending;
  no registration).

Docs (3, authority wording only):

- `docs/i2pcontrol/README.md`, `docs/i2pcontrol/proposal-170-support.md`,
  `docs/i2pcontrol/tunnel-manager.md` (M158 closed authority + `336/29/475`
  wording retained; LeaseSet residual-blocked statements retained).

`061/062`, `095/105/110` semantics: `061` unchanged (strict subset, no
broad waiver); `062` gains only the exact M158 closure authorization
above; `095/105/110` intentionally **unchanged** (zero promotions; see
§8).

## 7. Requirement-to-evidence matrix (planning process §2.5)

| Closure duty | Evidence |
|---|---|
| implementation commits | one commit landing with this closure (production + tests + M062 guard + planning records); exact paths in §6; no prior partial implementation commit (registration commits touched only plans/TOMLs/tests) |
| invariant review | partial-support, no-fabrication, I2P-only egress, fail-closed option validation, containment, and Y005 invariants re-proved by green `m061`/`m062`/`m095`/`m105` guards; no invariant weakened; M147/M148 not reopened (type-11 derived-only, gates enforced); M146 untouched (no egress); no downgrade path exists (fallible derivation, no empty-secret/ordinary fallback, stale-key rejection) |
| failure/recovery and contention evidence | malformed secrets reject pre-allocation; overlong secrets fail at derivation with retry and no fallback; wrong-type/key verification ignored; stale-key acks ignored; expired/invalid inner never published; rollover is single-owner timer/state with no second task; no lock spans I/O or crypto; shutdown drops the owner-local timer; successor generations receive no predecessor secret (config is generation-local); tamper/wrong-secret/flag/type/length tests prove closed rejection |
| compatibility, migration, security review | no wire change to type-3/RouterInfo/lookup protocol; secret + extended-B32 are additive; no storage migration (opaque DHT keys, bounded cache); no new attack surface beyond reviewed parser/codec/derivation composition; all 15 LeaseSet cells stay fail-closed (matrix unchanged); legacy AES still blocked; `cargo clippy -p emissary-core -D warnings` clean; `emissary-cli -D warnings` failure is pre-existing and unrelated with zero M158-file warnings |
| documentation and operational evidence | §6 changed paths; authority docs name M158 closed and retain partial support; operational impact none beyond the opt-in standard secret property and the corrected type-5 server address form (default ordinary path byte-compatible) |
| M158 acceptance (plan closure evidence) | exact changed files/dependencies (§6), standard-property evidence (§1), secret alpha/blinded/store-key known answers (§1), extended-B32 vectors and adversarial results (§2), secret-redaction audit (§1), rollover/recreation traces (§1/§5), no-downgrade evidence (§1/§5), unchanged M095 (§8), implementation SHA (commit landing with this closure), M159 readiness (§10) |

## 8. Promotion accounting (zero)

- `110-completion-ledger.toml`: no new entry (zero promotions).
- `095-full-support-matrix.toml` / `105-residual-option-audit.toml`:
  unchanged.
- Mechanically recomputed `336/29/475` == declared; residual
  `10/5/5/5/4` unchanged.
- Infrastructure alone has zero support value (registry rule);
  no `EncryptLeaseSet`/`OptionalLookup`/`LeaseSetClientAuths` cell is
  claimed by this primitive. `OptionalLookup` remains blocked until M162
  supplies mapping, custody, redaction, transactionality, and five-family
  integration.

## 9. Production-head determination

- Last production-bearing commit before M158: the M157 closure commit
  ancestry (unchanged through M158 registration at `4dad28f9`).
- M158 is production-bearing (exact §6 files); after the closure commit
  lands, the last production-bearing head becomes that commit.
- `git log 4dad28f9..HEAD` over `emissary-core/src`,
  `emissary-cli/src`, `emissary-util/src`, all manifests and `Cargo.lock`
  at closure time contains only the staged M158 exact-budget diff
  (4 production paths, `902 insertions, 36 deletions`; Rust production
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

- `158-*.md` plan: Status `registered / dependency-ready` → `closed as
  complete` with closure link;
- `plans/registry.md`: M158 → closed as complete (`336/29/475` unchanged,
  zero promotions, exact production diff); **no registered successor**;
- `110-completion-ledger.toml`: no new entry (zero promotions);
- `m061`: unchanged (strict subset; no broad waiver);
- `m062`: exact M158 closure authorization only (§4); no broad waiver.

Future-plan unblock determination (as required by the tasking):

- **M158 CLOSED as complete.** Its sole hard dependency (M157 closure with
  frozen publication/rollover owner and zero promotions) was satisfied,
  and all plan requirements are met with exact-budget production diff and
  zero matrix promotions. No stop condition in plan §14 triggered: no
  outside file or dependency was required, no I2PControl persistence
  change was needed for the neutral primitive, no NetDB subsystem was
  added, secret data leaves generic debug-capable state before activation,
  the extended-B32 format needed no broader naming subsystem, reference
  vectors agree with local behavior, and no failure downgrades to
  empty-secret or ordinary publication.
- **M159 REMAINS deferred/unregistered (hard dependency now satisfied,
  amendment pending).** M158 closure satisfies its hard dependency, but
  its registration gate additionally requires an exact-path amendment
  freezing PSK layer-1 format/ownership, per-client work limits, secret
  custody boundaries, publication/auth integration seams, exact
  files/dependencies, and interop vectors/fixtures. That amendment is not
  written by M158 and must be authored at M159 registration. The M159 plan
  file gains only a deferred-status note recording the satisfied hard dep;
  it is **not** registered by this closure. Only M159 may be registered
  next, with exact M061/M062 authorization in that registration commit.
- **M160 remains deferred/unregistered**, hard-gated on M159 closure.
- **M161 remains deferred/unregistered**, hard-gated on M155 closure
  (satisfied) but sequenced after M160 per the roadmap; it may now be
  prepared but must not overtake M158-M160 implementation. Its A/B/C
  outcome gates M162 `EncryptLeaseSet` promotion.
- **M162 remains deferred/unregistered**, hard-gated on M160 closure plus
  M161 disposition (plus any M161-A successor closure before
  `EncryptLeaseSet` promotion). It inherits the standard-property mapping
  (`i2cp.leaseSetSecret` Base64(UTF-8) consumer proven here; Proposal
  `OptionalLookup` plaintext mapping still to build).
- **M152 remains deferred/unregistered**, re-gated on M162 closure plus any
  M161-A successor. Its final qualification must now account for the M155
  field-coupling rule plus the M156/M157/M158 primitive availability.
- **M149-M151 remain superseded/unregistered (do not execute).** Confirmed
  fully superseded by M155-M162; no re-gating or revival is authorized.
- **M147/M148 remain closed-blocked/deferred-behind-M147** (M154
  disposition C); this line does not reopen them. Type-11 blinding is
  explicitly not a `SigType` reopening.
- **M146 remains closed as blocked** with no successor; it is not reopened
  by M158 or by any future LeaseSet tail.
- No other future-plan status required a change. File presence alone never
  authorises production work.

## 11. Unresolved findings

- Non-blocking: `cargo test -p emissary-cli --no-default-features
  --features i2pcontrol --test m153_post_m146_requalification` reports
  `AGENTS.md must retain M139 as historical evidence`. Fails identically at
  the pre-M158 registration baseline; unrelated to M158 files. Severity:
  informational; no corrective pass required (out of M158 exact-budget
  scope; must not be smuggled into this line).
- Non-blocking: `cargo clippy -p emissary-cli --no-default-features
  --features i2pcontrol --all-targets -- -D warnings` reports a
  pre-existing lint in `backends/filters/proxy.rs:60` (`chunks_exact`),
  untouched by M158; M158 files contribute zero warnings. Severity:
  informational; no corrective pass required (out of M158 exact-budget
  scope).
- Non-blocking: `cargo fmt --all -- --check` (stable or nightly) reports
  drift in files outside the M158 budget at the registration baseline;
  M158-touched Rust files are stable-`rustfmt` clean with `max_width = 100`
  observed, and no new nightly drift was introduced by M158 lines
  (remaining nightly hunks are pre-existing comment reflow from a newer
  toolchain). Severity: informational; unrelated files were not touched.
- Informational: rejected-command logging in `sam/socket.rs` (outside the
  M158 budget, intentionally untouched) echoes the raw command line when
  `SamCommand::parse` fails. A *malformed*-secret `SESSION CREATE` would
  therefore echo its rejected option value to the local log; valid-secret
  sessions never hit that path (parse succeeds, nothing is logged).
  Severity: low/informational; secret hygiene for that generic socket path
  is noted for a future hardening pass and must not be smuggled into this
  line.
- Informational: the frozen M156 primitive bounds derivation secrets to 64
  bytes while the standard property has no semantic limit. M158 reconciles
  this without truncation or downgrade: the parser is length-agnostic and
  stores any framed valid UTF-8 secret exactly, while overlong secrets fail
  closed at blinding derivation (retry, no empty-secret or ordinary
  fallback), as proven by test. A future milestone may revisit the bound
  only through a plan amendment.

## Internal-only / read-only-upstream attestation

- External sources (the already-pinned Proposal/Java/Yosemite revisions and
  the Encrypted-LeaseSet/Red25519 specifications cited from existing
  M155/M156/M157 records) were accessed read-only for evidence; no new
  upstream fetch, and no Java source checkout, was performed for this
  closure — byte-exact address vectors were cross-checked against an
  independent local recomputation of the frozen plan format;
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
