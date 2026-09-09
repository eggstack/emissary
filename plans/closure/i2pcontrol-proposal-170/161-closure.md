# M161 Closure — Legacy AES / LeaseSet1 Feasibility and Contract Gate (Outcome B)

Status: **closed as complete, outcome B; no implementation successor; M162 hard dependencies satisfied, registration pending**

Date: `2026-09-09`

Source implementation plan:

- `plans/implementation/i2pcontrol-proposal-170/161-legacy-aes-ls1-feasibility-and-contract-gate.md`
  (Status now `closed as complete` with closure link; hard dependency on
  M160 closure satisfied by `plans/closure/i2pcontrol-proposal-170/160-closure.md`;
  zero production and zero promotion budgets observed as proven below).

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-post-m154-leaseset-security-corrective-roadmap.md`.

Promotion budget: **zero Proposal cells**.

Production budget: **zero production Rust/dependency/Yosemite changes**.

## Planning baseline

- Registration baseline `e11b277` (registered M161 zero-production gate; clean
  worktree before M161 gate work).
- Last production-bearing head before M161: `1629b0a5` (M160 DH
  implementation/closure; unchanged through M161 registration, which touched
  only plans).
- Incoming M095 matrix: `336 apply / 29 blocked_primitive / 475
  not_applicable` across 840 TunnelManager option/family cells.
- M153 closed complete as the current runtime/security qualification
  authority; M154 closed complete disposition C (M147 path blocked);
  M155 closed complete with zero production; M156 closed complete with zero
  promotions; M157 closed complete with zero promotions; M158 closed
  complete with zero promotions; M159 closed complete with zero promotions;
  M160 closed complete with zero promotions; M146 closed blocked.

Reviewed head:

- M161 gate work completes on the working tree described in §12. No
  `emissary-core/**`, `emissary-util/**`, `emissary-cli/src/**`, manifest,
  lockfile, Yosemite, frontend, or workflow path was created, modified, or
  deleted for production purposes (see §12 and the no-production-diff proof).
  The only Rust change in this closure is the bounded `m062` planning-guard
  extension (§7), which is test-only and explicitly allowed by the plan
  ("Authorized changes are planning/test/evidence only").

Pinned authority (all accessed read-only; no upstream mutation, contact, or
submission occurred — see read-only attestation):

- Proposal 170 revision `2026-05-20`, status Open, SHA-256
  `f13ae00b886c5e72131bc5d5b138a371148d1faa6899a119a1dacb65a555e7dc`
  (cited from M095/M153/M155; live text re-fetched read-only 2026-09-09 to
  confirm the ten-value `EncryptLeaseSet` domain including
  `encrypted (aes)` — the Proposal lists the ten strings with no per-mode
  I2CP property table, so per-mode wire mapping is frozen from the
  I2CP/ELS2/Yosemite/Java sources below);
- Java I2PControl Proposal-170 head `45bb593000408071dd376b78848fdc246dccd964`
  and Java I2P snapshot `2c3fd2a9532cd86ec06cb6f2b9f3f813ca752243` (cited
  from pinned M155 records; no new fetch; direct `ServiceTunnelCreator`
  mapping cited from the frozen M157 §3 record: `i2cp.encryptLeaseSet=true`
  only for legacy `ENCRYPT_LEASE_SET_AES`, modern blinded/PSK/DH modes
  select `i2cp.leaseSetType=5`);
- I2P Encrypted LeaseSet specification,
  `https://geti2p.net/en/docs/specs/encryptedleaseset`, read-only fetch
  2026-09-09 for this closure (three nested layers; layer-1 flags `0x00`
  no-auth / `0x01` DH / `0x03` PSK; `ELS2_XCA` / `ELS2PSKA` 52-byte
  schedules; `ELS2_L1K` / `ELS2_L2K` 44-byte schedules; ChaCha20 counter-1
  `ENCRYPT`; blinded DHT storage key; UTC-day rollover; 56-char extended
  B32; "ChaCha20 was selected over AES" note; no legacy-AES LS1 path in
  the modern format);
- I2P common-structures specification,
  `https://geti2p.net/spec/common-structures`, read-only fetch 2026-09-09
  for this closure (legacy `LeaseSet` LS1: ElGamal/AES+SessionTag heritage,
  256-byte encryption key, 44-byte `Lease`, revocation signing key
  unimplemented, "Destination public key ... currently unused except for
  the IV for LeaseSet encryption, which is deprecated"; modern `LeaseSet2` /
  `Lease2` 40-byte compact leases; key-certificate codes 0–11);
- I2P Client Protocol (I2CP) specification,
  `https://geti2p.net/en/docs/specs/i2cp`, read-only fetch 2026-09-09 for
  this closure (`i2cp.encryptLeaseSet` since 0.7.1 "Encrypt the lease";
  `CreateLeaseSetMessage` DEPRECATED "Cannot be used for LeaseSet2 ...
  or encrypted LeaseSets. Use CreateLeaseSet2Message"; `CreateLeaseSet2`
  type 1 `LeaseSet` deprecated, type 3 `LeaseSet2`, type 5
  `EncryptedLeaseSet`; 0.9.39 CreateLeaseSet2, 0.9.41 EncryptedLeaseSet
  options, 0.9.43 BlindingInfo);
- Java `net.i2p.data.LeaseSet` API documentation (API 0.9.70),
  `https://i2p.github.io/i2p.i2p/net/i2p/data/LeaseSet.html`, read-only
  fetch 2026-09-09 for this closure ("Support encryption and decryption
  with a supplied key. Only the gateways and tunnel IDs in the individual
  leases are encrypted. WARNING: Encryption is poorly designed and probably
  insecure. Not recommended. Encrypted leases are not indicated as such.
  The only way to tell a lease is encrypted is to determine that the listed
  gateways do not exist. Routers wishing to decrypt a leaseset must have
  the desthash and key in their keyring."; `encrypt(SessionKey)` "Encrypt
  the gateway and tunnel ID of each lease, leaving the expire dates
  unchanged. This adds an extra dummy lease, because AES data must be
  padded to 16 bytes. The fact that it is encrypted is not stored
  anywhere. Must be called after all the leases are in place, but before
  sign().");
- Java `net.i2p.data.LeaseSet` reference source (I2PPlus mirror of the
  reference implementation),
  `https://raw.githubusercontent.com/I2PPlus/i2pplus/master/core/java/src/net/i2p/data/LeaseSet.java`,
  read-only fetch 2026-09-09 for this closure (`DATA_LEN = Hash.HASH_LENGTH
  + 4` (36); `IV_LEN = 16`; `encryp` packs `{gateway Hash, tunnelId}` pairs,
  pads to a 16-byte multiple with random data, IV = first 16 bytes of the
  Destination public key, AES encrypt, pads to a 36-byte multiple, appends
  one dummy lease and replaces all gateway/tunnel IDs; `decrypt` reverses
  via the `keyRing` SessionKey keyed by destination hash; `isEncrypted`
  decrypts on first call only when a keyring entry exists);
- Java `net.i2p.data.EncryptedLeaseSet` API evidence (cited from reference
  documentation excerpts, read-only 2026-09-09): `encrypt(SessionKey)`
  override documents `skey` as **ignored** — the modern encrypted-LS2 path
  does not consume a legacy session key;
- i2pd tunnel documentation excerpts (read-only websearch 2026-09-09):
  `i2cp.leaseSetType` OLD `1` Deprecated, STANDARD `3` Default, ENCRYPTED
  `5` Encrypted LeaseSet; `i2cp.leaseSetEncType` ELGAMAL `0` Legacy,
  ECIES_X25519_AEAD `4` Default;
- Yosemite optional exact revision `59140a2277bf296928d2e8ce39a148182eeff044`
  (unchanged; no Yosemite change in M161; typed LeaseSet wire fields
  verified read-only in the checked-out fork at
  `~/.cargo/git/checkouts/yosemite-b3f22cc17f665e22/59140a2/src/options.rs`
  (`encrypt_lease_set: bool` emits `i2cp.encryptLeaseSet=true`;
  `lease_set_type: 1..=255` default `1`; `lease_set_key` documented "For
  encrypted leasesets. Base 64 SessionKey") and
  `src/proto/session.rs:270-330` (deterministic typed lease-option
  serialization), see §1);
- Emissary source at the reviewed tree (all file:line citations below read
  at HEAD plus the workdir diff closed here).

Current Proposal matrix at closure (mechanically recomputed via the
`m095_full_support_matrix` suite, dispositions unchanged):

- `336 apply / 29 blocked_primitive / 475 not_applicable` (840 cells);
- residual split `SigType` 10, `EncryptLeaseSet` 5, `OptionalLookup` 5,
  `LeaseSetClientAuths` 5, `UseOutproxyPlugin` 4 — identical to
  M153/M154/M155/M156/M157/M158/M159/M160 entry.

## 1. Requirement-to-evidence matrix (plan required evidence)

| Plan-required trace | Evidence |
|---|---|
| Proposal PR `encrypted (aes)` mapping (`i2cp.encryptLeaseSet=true`, not type 5) | Live Proposal text lists exactly `encrypted (aes)` among the ten `EncryptLeaseSet` strings with no per-mode table; frozen M157 §3 direct-`ServiceTunnelCreator` record establishes `i2cp.encryptLeaseSet=true` only for `ENCRYPT_LEASE_SET_AES` and `i2cp.leaseSetType=5` for modern blinded/PSK/DH modes; Yosemite typed options keep the two selectors independent (`encrypt_lease_set` bool vs `lease_set_type 1..=255`); Emissary `tunnel_manager.rs:1734-1751` validates the exact ten-string domain including `encrypted (aes)` without mapping it |
| Java I2PTunnel/I2CP LS1-vs-LS2 selection on the pinned snapshot | I2CP spec: `CreateLeaseSetMessage` deprecated for LS2/encrypted sets, `CreateLeaseSet2` type 1 (LS1) deprecated, types 3/5 current; version notes 0.9.39 (CreateLeaseSet2) and 0.9.41 (EncryptedLeaseSet options); pinned M155 records cite `RequestLeaseSetMessageHandler.requiresLS2()` forcing LS2 when supported with `_ls2Type` selection — M161 freezes the structural consequence (below) rather than re-claiming a fresh Java line it did not re-fetch |
| Java `RequestLeaseSetMessageHandler.requiresLS2()` behavior and `_ls2Type` selection | Cited from pinned M155/M157 records (no new fetch): the handler prefers/requires LS2 when supported. Structural corroboration in this closure: I2CP `CreateLeaseSet2` supersedes `CreateLeaseSet`; type-1 LS1 is the deprecated member of the `1/3/5` family (i2pd docs); modern destinations (Ed25519 + ECIES_X25519) have no LS1 publication path in the current stack |
| Legacy `LeaseSet.encrypt(SessionKey)` wire behavior | Javadoc + raw source (fetched read-only for this closure): AES over packed `{gateway Hash (32B), tunnelId (4B)}` pairs padded to 16B, IV = first 16B of Destination public key (deprecated use), one extra dummy lease appended with post-encryption padding to 36B multiples, expire dates unchanged, no on-wire encrypted flag, client-side encryption with router-side keyring decryption (`keyRing.get(destinationHash)`), "poorly designed and probably insecure. Not recommended", revocation-of-encrypted explodes |
| `LeaseSet2` / EncryptedLeaseSet behavior when the legacy flag/key are present | `EncryptedLeaseSet.encrypt(SessionKey)` documents `skey` as ignored; ELS spec layer-1 uses flags `0x00/0x01/0x03` with ChaCha20/HKDF (`ELS2_L1K/L2K/XCA/PSKA`), never a single AES `SessionKey` call — "ChaCha20 was selected over AES"; aliasing `encryptLeaseSet=true` to type 5 would therefore be a silent reinterpretation, explicitly forbidden by the plan |
| Current router/client capability negotiation that could still select LS1 | i2pd `leaseSetType` OLD `1` Deprecated / STANDARD `3` / ENCRYPTED `5`; Java `CreateLeaseSetMessage` + type-1 deprecated; common-structures LS1 revocation unimplemented; no current encrypted-service negotiation selects LS1 for modern Ed25519/ECIES destinations — the only LS1 selector left is the deprecated type-1 path |
| Current Emissary LS1 construction, DatabaseStore, publication, lookup and client-consumption capability, if any | §3 owner inventory: **none** — no LS1 `LeaseSet` struct, no `encrypt(SessionKey)` helper, no ElGamal-256B LS1 builder, no type-1 payload variant, no LS1 publication/lookup/decryption path; the 44B `parse_frame_lease` helper exists but is unused by any LS1 owner (LS2 exclusively uses 40B `parse_frame_lease2`); SAM either rejects the legacy flag with type 5 or accepts it inertly without encrypting (§3) |
| Whether a pinned reference Java service configured through the Proposal PR with `encrypted (aes)` actually publishes a usable encrypted object against an LS2-capable router | No live-router exercise was buildable in this zero-production gate (see §4). Source-level determination: the value maps to the deprecated LS1 AES path whose wire object is a type-1 `LeaseSet` with keyring-gated, flag-less AES leases — not a type-5 ELS2 object — while the current handler/spec family prefers LS2 and the modern encrypted path ignores the session key. A "success" consisting of the flag being ignored and an ordinary LS2 being published would violate the plan invariant ("no reference test that succeeds only because the flag is ignored counts as support") and is therefore not support |
| Whether the Proposal value is operationally coherent or a retained configuration surface | Operationally coherent **as a legacy-LS1 contract** (real AES wire behavior with real Java implementation), but incoherent **as a modern encrypted-service value**: it cannot produce a type-5 ELS2 object and cannot survive LS2-preferring negotiation. Hence outcome B (valid but blocked), not C (reference-incoherent/dead): the reference operation exists, but supporting it here would require broad resurrection of a deprecated, insecure protocol (see §5) |

Do not infer support from GUI/property persistence alone: observed —
Emissary `tunnel_manager.rs` validates/persists the `encrypted (aes)`
string and Yosemite can serialize `i2cp.encryptLeaseSet=true`, but neither
constitutes an encrypted LS1 data plane (§3).

## 2. Published DatabaseStore type/object and lookup/decryption evidence (not controller configuration)

- A legacy-`encrypted (aes)` publication, if it existed, would have to emit
  an I2NP DatabaseStore of LS1 type (OLD `1` family) containing a
  `Destination` + 256-byte ElGamal encryption key + revocation signing key
  + count + 44-byte `Lease` entries (one dummy-padded) + signature, with
  gateway/tunnel IDs AES-encrypted under a `SessionKey` distributed
  out-of-band through a destination-hash keyring. None of this exists in
  Emissary (§3): `DatabaseStoreKind`/`DatabaseStorePayload` implement only
  `RouterInfo`, `LeaseSet2`, and `EncryptedLeaseSet2`
  (`emissary-core/src/i2np/database/store.rs:149-192,376-500`); `StoreType`
  parses the LS1 type code but has **no payload variant** for it, so no
  LS1 object can be built, parsed into a payload, cached, flooded, or
  answered on lookup.
- Floodfill/lookup behavior for such an object is therefore unimplemented by
  construction: `netdb/mod.rs` handles only `LeaseSet2` (ordinary queries
  via `query_lease_set`) and `EncryptedLeaseSet2` (type-preserving
  cache/flood/lookup from M157); there is no LS1 store/verify/lookup path
  and no keyring-gated AES lease decryption anywhere in the tree (full-tree
  search for `keyRing`, `isEncrypted`, LS1 `encrypt(SessionKey)` finds only
  the unrelated tunnel-AES/ChaCha helpers).
- SAM-level behavior pins the negative: `validate_lease_set_type_options`
  (`sam/parser.rs:759-786`) accepts absent/`3` as ordinary, gates `5`
  through the exact no-auth/PSK/DH companions, and rejects every other
  `leaseSetType` (including OLD `1`); each of `is_valid_type5_no_auth`,
  `is_valid_type5_psk`, `is_valid_type5_dh` rejects
  `i2cp.encryptLeaseSet=true/1` before activation (proven by the
  `type5_*_negative_paths_rejected` suites). With no type-5 selector the
  legacy flag falls through to the ordinary path where it is accepted
  **inertly** — no AES encryption occurs. Per the plan invariant, inert
  acceptance is not support.

## 3. Current Emissary owner inventory (read at the reviewed tree)

| Capability | Exact current owner | Status |
|---|---|---|
| Proposal `EncryptLeaseSet` string domain | `emissary-cli/src/i2pcontrol/tunnel_manager.rs:1734-1751` (`MODES` with exactly `encrypted (aes)`; unknown rejects) | validation/persistence only; no runtime mapping for legacy AES |
| Standard SAM session-option transport | Yosemite `options.rs:159-186,639-760` (typed `encrypt_lease_set`, `lease_set_type 1..=255`, `leaseSetKey/Secret/PrivKey` companions, bounded client-auth namespaces); `proto/session.rs:270-330` (deterministic serialization) | reachable serializer only; not a data plane |
| SAM pre-allocation gates | `emissary-core/src/sam/parser.rs:64-66` (`is_type5_requested`), `:705-786` (`is_valid_type5_no_auth/psk/dh`, `validate_lease_set_type_options`); `session.rs` narrow type-5 composition | legacy flag rejected with type 5, inert without it; OLD `1` rejected |
| Ordinary LS2 parse/serialize/sign | `emissary-core/src/primitives/lease_set.rs:44-130` (`LeaseSet2Header`), `:251-460` (`LeaseSet2`: 40B `parse_frame_lease2` leases only, X25519-family keys only, nonzero ≤16 leases, type-3 framing) | no LS1 struct; 44B `parse_frame_lease` helper exists but has no LS1 owner |
| Encrypted LS2 outer/wire | `primitives/lease_set.rs` (`EncryptedLeaseSet2`), `crypto/els2.rs` (M156-M160 schedules, no AES), `crypto/red25519.rs` (M156, unchanged) | modern type-5 only; legacy AES absent |
| DatabaseStore build/parse | `emissary-core/src/i2np/database/store.rs:39-81` (`StoreType` codes incl. LS1), `:149-500` (payload/kind only for `RouterInfo`/`LeaseSet2`/`EncryptedLeaseSet2`), `:571-794` (parse only LS2/ELS2 payloads) | LS1 code parsed, no LS1 payload path |
| Floodfill cache/lookup/flood | `emissary-core/src/netdb/mod.rs` (M157 type-preserving cache; ordinary `QueryKind::LeaseSet` + encrypted handling; `query_lease_set` returns `LeaseSet2`) | no LS1/keyring path |
| Publication/rollover/verification | `emissary-core/src/destination/lease_set.rs` (`LeaseSetManager`: ordinary LS2 + modern type-5 ELS2 only, sole owner, UTC rollover, key-scoped verification) | no LS1 generation, no second scheduler |
| Session/end-to-end composition | `destination/mod.rs`, `destination/session/mod.rs` (ordinary inner LS2 retained for garlic/session wrapping per M157 §4.5) | unchanged |
| Key material | `crypto/mod.rs` (X25519/ECIES, Ed25519; no ElGamal-256B LS1 keygen, no SessionKey-ring) | no LS1 crypto owner |

Implementing legacy AES would therefore require a **new** LS1
wire/publication/lookup subsystem at minimum: LS1 `LeaseSet`
(Destination + 256B ElGamal key + revocation key + 44B leases + dummy-lease
AES framing with destination-pubkey IV) construction/parsing/signing;
type-1 DatabaseStore payload + floodfill/lookup/verification paths;
destination-hash keyring generation/distribution/persistence with
transactional secret custody; SAM `leaseSetType=1` + `leaseSetKey`
activation semantics; and client-side keyring decryption for consumption —
all against a protocol the reference itself marks deprecated and insecure.
That is a broad legacy-protocol resurrection, explicitly forbidden inside
this gate.

## 4. Reference-runtime exercise (if buildable)

A minimal live reference-router exercise (configure a pinned Java service
with `encrypted (aes)` via the Proposal PR path and observe the published
DatabaseStore type/object plus lookup/decryption behavior) was **not
buildable** within this zero-production gate: it would require checking
out and building the pinned Java router/I2PTunnel snapshot, provisioning a
live floodfill network identity, and driving I2CP publication end to end —
work that itself needs new harness/dependency/network effect outside the
plan envelope ("Authorized changes are planning/test/evidence only";
"a minimal reference-runtime exercise **if buildable**").

The gate instead relies on the stronger source-level combination above:
live Proposal text (ten strings, no per-mode table) + frozen
`ServiceTunnelCreator` mapping + I2CP spec deprecation/version history +
common-structures LS1-vs-LS2 structures + verbatim `LeaseSet.encrypt`
Javadoc and raw AES/keyring source + `EncryptedLeaseSet.encrypt` ignored-key
evidence + ELS-spec ChaCha-over-AES selection + i2pd OLD-deprecated table +
checked-out Yosemite typed options + complete Emissary owner inventory with
negative SAM/DatabaseStore/NetDB/publication gates. Under the plan rule
that flag-ignored "success" never counts as support, this combination
decisively establishes outcome B without a live network.

## 5. Outcome determination: B — valid contract but disproportionate or unsafe under current architecture

M161 closes **outcome B**:

- The legacy-AES contract is **valid**: `LeaseSet.encrypt(SessionKey)` is a
  real, implemented Java operation with exact wire behavior (AES over
  gateway/tunnel IDs, destination-pubkey IV, dummy-lease padding, keyring
  distribution, flag-less leases). It is not a phantom or contradictory
  reference value, so outcome C (reference-incoherent/dead operational
  value) is rejected: claiming the reference operation does not exist would
  be false.
- Supporting it in Emissary is **disproportionate**: it needs an entire new
  LS1 subsystem (§3) — wire, DatabaseStore, floodfill/lookup/verification,
  keyring custody, SAM activation, client consumption — for a deprecated
  type-1 path while the modern type-5 owner graph (M156-M160) is frozen and
  complete.
- Supporting it is **unsafe under current architecture/security policy**:
  the reference itself warns the design is "poorly designed and probably
  insecure" and "Not recommended"; the construction reuses the Destination
  public key as an AES IV (deprecated), marks encryption nowhere on the
  wire (opacity failure), breaks revocation ("Revocation of an encrypted
  leaseset will explode"), and depends on out-of-band keyring sharing with
  no modern authentication. Reintroducing ElGamal-256B LS1/AES publication
  would widen attack surface against the explicit M155/M161 invariants (no
  broad legacy resurrection, no aliasing to modern modes, no plaintext
  fallback, no changes to M156-M160 primitives).
- Outcome A (coherent and bounded successor) is rejected: freezing an LS1
  implementation successor would bless resurrection of a deprecated,
  insecure protocol as the price of completing a ten-value admin enum. The
  truthful disposition is to keep the field blocked rather than schedule
  insecure work.

Consequences (frozen):

- Legacy `encrypted (aes)` stays **valid but blocked** under current
  architecture/security policy.
- Modern ELS2 work (M156-M160) remains valid and untouched.
- All five `EncryptLeaseSet` cells remain **blocked**, because the field's
  complete valid enum domain is not implemented (no partial-enum promotion).
- **No implementation successor is created.** M161 must not implement LS1
  inside the gate, and no separate LS1 successor is frozen by this closure.
- M162 may proceed with modern field integration, but `EncryptLeaseSet`
  must remain blocked across all five server families even when all modern
  type-5 modes work (M162 promotion rule inherited).

## 6. Dependency/security review

- No new dependency. No Cargo, lockfile, Yosemite, or
  `emissary-cli/src/i2pcontrol/**` production change (proven by §12).
- `no_std + alloc` posture unchanged (no production code touched; proven by
  the `no_std` check in §8).
- Security invariants held: no reinterpretation of legacy AES as modern
  type-5 ELS2; no alias from `i2cp.encryptLeaseSet=true` to no-auth/PSK/DH
  modes (SAM gates preserved and green); no plaintext fallback (ordinary
  path never fabricates encryption; type-5 mismatches fail closed); no
  broad legacy-protocol resurrection (no LS1 code added); no changes to
  M156-M160 modern primitives (byte-identical); no flag-ignored test
  counted as support.
- Fail-closed posture preserved: legacy flag with type 5 rejects before
  allocation; OLD type `1` rejects; malformed/sparse/mixed/over-ceiling
  type-5 inputs still reject (M157-M160 negative suites green with zero
  regressions: 1159 core + 821 CLI lib + 35 guards).
- Ordinary Ed25519/LS2/DatabaseStore-type-3/NetDB-ordinary/M135/M145 SAM
  behavior preserved (same suites green).

## 7. Exact-path reconciliation (M061/M062)

M061 (`061-containment-boundary.toml` + `m061_containment.rs`):

- No change required and none made. M161 is zero-production; no new owner
  is realized and no pending registration remains after closure.

M062 (`062-dependency-containment.toml` + `m062_dependency_containment.rs`):

- `062-dependency-containment.toml` `[current_registration]` bookkeeping
  closed: milestone `M161 (closed complete, outcome B, no registered
  successor)` realized with zero production paths, zero new files,
  `new_direct_dependencies=[]`, `manifest_changes=[]`,
  `lockfile_change=false`, `yosemite_change=false`,
  `i2pcontrol_source_change=false`.
- `m062_dependency_containment.rs` gains narrow `is_authorized_m161_path`
  covering exactly the zero-production closure set (AGENTS, 3 docs, guard
  test itself, `161-closure.md`, `062-dependency-containment.toml`,
  `161-*.md` plan, `162-*.md` deferred-status note, README, registry, both
  roadmaps), wired into both the allowed-path and prohibited-pattern assert
  chains (same pattern as the M155/M160 helpers; authorizes the new closure
  file, which no earlier helper covers).
- `policy_terms_do_not_leak_into_non_policy_production_paths` passes: no
  production source was touched.

## 8. Verification outcomes

| Command group | Result |
|---|---|
| `cargo check -p emissary-core` | **pass** |
| `cargo check -p emissary-core --no-default-features --features no_std` | **pass** |
| `cargo test -p emissary-core --lib --no-fail-fast` | **pass**: `1159 passed, 2 ignored` (identical to the M160 head; zero new tests, zero regressions — expected for a zero-production gate) |
| `cargo check -p emissary-cli --no-default-features --features i2pcontrol` | **pass** |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --lib --no-fail-fast` | **pass**: `821 passed` (identical to the M160 head; zero regressions) |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment --test m062_dependency_containment --test m095_full_support_matrix --test m105_residual_option_audit --no-fail-fast` | **pass**: `8 + 23 + 3 + 1 = 35` across 4 suites (includes the new `is_authorized_m161_path` wiring in both chains) |
| M095 mechanical recomputation (via `m095_full_support_matrix` suite) | **pass**: `840/336/29/475` == declared; residual `10/5/5/5/4` unchanged |
| `cargo clippy -p emissary-core --all-targets -- -D warnings` | **pass**: No issues found |
| `cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings` | pre-existing failure only in unrelated `proxy.rs:60` (`chunks_exact`); zero warnings from M161 files (see §14) |
| stable `rustfmt` on M161-touched Rust files | **pass** for the M062 guard test (`rustfmt --check` clean; repo nightly-only options warn; `max_width = 100` observed; no new nightly drift — see §14) |
| `cargo fmt --all -- --check` (whole tree) | pre-existing drift only in files outside the M161 budget (see §14); unrelated files untouched |
| `git diff --check` | **pass** |

## 9. Changed paths (exact zero-production budget)

Guards/ledgers (2):

- `emissary-cli/tests/m062_dependency_containment.rs` (M161 authorization
  helper in both chains);
- `plans/implementation/i2pcontrol-proposal-170/062-dependency-containment.toml`
  (current-registration closure bookkeeping, zero-budget recorded).

Planning/closure/registry (8 + 1 note):

- `plans/implementation/i2pcontrol-proposal-170/161-legacy-aes-ls1-feasibility-and-contract-gate.md`
  (Status `registered / dependency-ready` → `closed as complete` with
  closure link and outcome-B record);
- new `plans/closure/i2pcontrol-proposal-170/161-closure.md` (this file);
- `plans/implementation/i2pcontrol-proposal-170/162-proposal-leaseset-security-field-integration.md`
  (deferred-status note: M160 closed + M161 outcome B; hard deps satisfied;
  registration pending exact-path amendment; no registration);
- `plans/registry.md` (M161 → closed outcome B; no registered successor;
  only M162 may be registered next);
- `plans/implementation/i2pcontrol-proposal-170/README.md` (M161 closed
  outcome-B section; chain updated);
- `plans/subsystems/i2pcontrol-proposal-170-post-m154-leaseset-security-corrective-roadmap.md`
  (M161 closed outcome B; graph updated);
- `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md`
  (M161 closed outcome B; handoff updated);
- `AGENTS.md` (M161 closed outcome-B entry; chain updated).

Docs (3, authority wording only):

- `docs/i2pcontrol/README.md`, `docs/i2pcontrol/proposal-170-support.md`,
  `docs/i2pcontrol/tunnel-manager.md` (M161 closed outcome-B authority +
  `336/29/475` wording retained; LeaseSet residual-blocked statements
  retained).

`061/062`, `095/105/110` semantics: `061` unchanged (zero production; no
broad waiver); `062` gains only the exact M161 closure authorization
above; `095/105/110` intentionally **unchanged** (zero promotions; see
§11).

## 10. Requirement-to-evidence matrix (planning process §2.5)

| Closure duty | Evidence |
|---|---|
| implementation commits | two commits: registration (`e11b277`, plans-only) plus the commit landing with this closure (guard test + M062 TOML + planning records + this closure; exact paths in §9); no production commit exists for M161 by design |
| invariant review | partial-support, no-fabrication, I2P-only egress, fail-closed option validation, containment, and Y005 invariants re-proved by green `m061`/`m062`/`m095`/`m105` guards; no invariant weakened; M147/M148 not reopened (no SigType change; type-11 stays derived-only); M146 untouched (no egress); no downgrade path exists (legacy flag either rejects with type 5 or stays inert without encrypting; no LS1 fallback was added) |
| failure/recovery and contention evidence | no new runtime exists to fail; preserved evidence: type-5 legacy-companion rejections before allocation, OLD-`1` rejection, inert-ordinary handling without encryption, M157-M160 negative matrices green, no second scheduler/state machine added, no lock spans changed, shutdown paths unchanged |
| compatibility, migration, security review | no wire/protocol/storage/dependency change ⇒ no migration impact; no new attack surface (no production code); all 15 LeaseSet cells stay fail-closed (matrix unchanged); legacy AES stays valid-but-blocked with no silent aliasing (§5); `cargo clippy -p emissary-core -D warnings` clean; `emissary-cli -D warnings` failure is pre-existing and unrelated with zero M161-file warnings |
| documentation and operational evidence | §9 changed paths; authority docs name M161 closed outcome B and retain partial support; operational impact none (default ordinary path byte-compatible; type-5 modern path untouched) |
| M161 acceptance (plan closure evidence) | outcome B with exact source/runtime evidence (§1), no successor (§5), unchanged matrix (§8/§11), zero production diff (§12), M061/M062 guards green (§7/§8), implementation SHAs (registration + closure commits), M162 readiness with `EncryptLeaseSet` still blocked (§13) |

## 11. Promotion accounting (zero)

- `110-completion-ledger.toml`: no new entry (zero promotions).
- `095-full-support-matrix.toml` / `105-residual-option-audit.toml`:
  unchanged.
- Mechanically recomputed `336/29/475` == declared; residual
  `10/5/5/5/4` unchanged.
- Infrastructure alone has zero support value (registry rule); the gate
  itself claims no cell. `EncryptLeaseSet` remains blocked on all five
  server families because the valid `encrypted (aes)` enum value is not
  implemented — even after M162 completes modern modes, unless a future
  accepted plan revisits this disposition.

## 12. Production-head determination

- Last production-bearing commit before and after M161: `1629b0a5` (M160
  implementation/closure). M161 registration (`e11b277`) and this closure
  commit touch only plans/docs/guard-test paths.
- `git log e11b277..HEAD` over `emissary-core/src`,
  `emissary-cli/src`, `emissary-util/src`, all manifests and `Cargo.lock`
  at closure time is empty (verified before writing this closure; the only
  Rust delta in the closure commit is the `m062` guard-test helper).
- `git diff --name-only` over manifests/lockfile is empty: no `Cargo.toml`,
  `emissary-core/Cargo.toml`, or `Cargo.lock` change. `git status --short`
  shows only the §9 paths.
- Streamr datagram limits (16-subscriber, 60s expiry, 1200-byte payload,
  4095-byte transport-buffer, 15s refresh, bounded shutdown,
  loopback-only UDP) untouched; remote datagrams never choose a local UDP
  destination.

## 13. Registry updates and future-plan unblock determination

Applied alongside this closure (see §9 for the file list):

- `161-*.md` plan: Status `registered / dependency-ready` → `closed as
  complete` with closure link and outcome-B record;
- `plans/registry.md`: M161 → closed outcome B (`336/29/475` unchanged,
  zero promotions, zero production diff); **no registered successor**;
- `110-completion-ledger.toml`: no new entry (zero promotions);
- `m061`: unchanged (zero production; no broad waiver);
- `m062`: exact M161 closure authorization only (§7); no broad waiver.

Future-plan unblock determination (as required by the tasking):

- **M161 CLOSED outcome B.** Its sole hard dependency (M160 closure with
  frozen modern publication/rollover owner and zero promotions) was
  satisfied at registration, and all plan requirements are met with a
  zero-production diff and zero matrix promotions. No stop condition
  beyond the recorded outcome triggered: no outside file or dependency was
  required (the LS1 subsystem that outcome A would need is precisely what
  makes B disproportionate), no I2PControl persistence change was needed
  for the gate, no NetDB subsystem was added, the legacy flag stays out of
  modern type-5 activation, and no failure downgrades to plaintext or
  ordinary publication.
- **M162 REMAINS deferred/unregistered (hard dependencies now satisfied,
  registration pending exact-path amendment).** M160 closure plus M161
  outcome B satisfy its disposition gate for modern field integration, and
  no M161-A successor exists to block it. But M162 is **not** registered
  by this closure: its entry gate additionally requires revalidation of
  the exact I2PControl owner set and secret-store schema against the
  then-current head with M061/M062 authorization in that registration
  commit. The M162 plan file gains only a deferred-status note recording
  satisfied hard deps; it is **not** registered here. Only M162 may be
  registered next. When M162 closes, `EncryptLeaseSet` must still remain
  blocked on all five families per §5.
- **M152 remains deferred/unregistered**, re-gated on M162 closure. Its
  final qualification must now account for the M155 field-coupling rule
  plus the M156/M157/M158/M159/M160 primitive availability, the M161
  outcome-B `EncryptLeaseSet`-blocked disposition, and M162
  secret-transactionality evidence.
- **M149-M151 remain superseded/unregistered (do not execute).** Confirmed
  fully superseded by M155-M162; no re-gating or revival is authorized.
- **M147/M148 remain closed-blocked/deferred-behind-M147** (M154
  disposition C); this line does not reopen them. Type-11 blinding remains
  explicitly not a `SigType` reopening.
- **M146 remains closed as blocked** with no successor; it is not reopened
  by M161 or by any future LeaseSet tail.
- No other future-plan status required a change. File presence alone never
  authorises production work.

## 14. Unresolved findings

- Non-blocking: `cargo test -p emissary-cli --no-default-features
  --features i2pcontrol --test m153_post_m146_requalification` was not run
  in the M161 verification set (same scope as the M160 closure: the four
  `m061`/`m062`/`m095`/`m105` suites). The known pre-existing
  `AGENTS.md`/`proxy.rs` findings below fail identically at the
  pre-M161 registration baseline; unrelated to M161 files. Severity:
  informational; no corrective pass required (out of M161 exact-budget
  scope; must not be smuggled into this line).
- Non-blocking: `cargo clippy -p emissary-cli --no-default-features
  --features i2pcontrol --all-targets -- -D warnings` reports a
  pre-existing lint in `backends/filters/proxy.rs:60` (`chunks_exact`),
  untouched by M161; M161 files contribute zero warnings. Severity:
  informational; no corrective pass required (out of M161 exact-budget
  scope).
- Non-blocking: `cargo fmt --all -- --check` (stable or nightly) reports
  drift in files outside the M161 budget at the registration baseline
  (e.g. `address_book_runtime.rs` reflow); the M161-touched Rust file
  (`m062_dependency_containment.rs`) is stable-`rustfmt` clean with
  `max_width = 100` observed, and no new nightly drift was introduced by
  M161 lines (remaining nightly hunks are pre-existing comment reflow from
  a newer toolchain). Severity: informational; unrelated files were not
  touched.

## Internal-only / read-only-upstream attestation

- External sources (the already-pinned Proposal/Java/Yosemite revisions
  cited from existing M155/M156/M157 records; the Encrypted-LeaseSet,
  common-structures, and I2CP specifications plus the `LeaseSet` Javadoc
  and raw `LeaseSet.java` source re-fetched read-only 2026-09-09 for this
  closure; i2pd tunnel-documentation excerpts read-only; the checked-out
  Yosemite fork read-only for wire evidence) were accessed read-only for
  evidence; no Java source checkout was built and no live reference router
  was provisioned for this zero-production gate;
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
