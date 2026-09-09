# M155 Closure — Post-M154 LeaseSet-Security Semantic and Exact-Owner Re-freeze

Status: **closed as complete; M156 unblocked → registered / dependency-ready**

Date: `2026-09-09`

Source implementation plan:

- `plans/implementation/i2pcontrol-proposal-170/155-post-m154-leaseset-security-semantic-and-owner-refreeze.md`
  (Status now `closed as complete` with closure link; hard dependency on
  M154 closure satisfied by `plans/closure/i2pcontrol-proposal-170/154-closure.md`;
  zero production and zero promotion budgets observed as proven below).

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-post-m154-leaseset-security-corrective-roadmap.md`.

Promotion budget: **zero Proposal cells**.

Production budget: **zero production Rust/dependency/Yosemite changes**.

## Planning baseline

- Registration baseline `6fbc9ffcaeff426736b6951581111c4bdf126fca` (clean
  worktree verified before M155 closure edits; `git status --short` empty).
- Plan baseline cited at registration: `5a9754cdd2226ed23336feb61e6fa77118d2939e`
  (M154 closure head). All M155-line planning commits
  `0705a49..6fbc9ff` (M155 plan, post-M154 roadmap, M156-M162 drafts,
  M149-M151 supersedes, M152 re-gate, registry/README/AGENTS/roadmap alignment)
  are docs/plans-only; `git log 7cbd80a..HEAD` over `emissary-core/src`,
  `emissary-cli/src`, `emissary-util/src`, all manifests and `Cargo.lock`
  is empty at both entry and closure (see §9).
- Last production-bearing head: `7cbd80a6d72aa07d158ba9dc74f8bbacef767be2`
  (M145 no-std/format follow-up; unchanged since M153/M154).
- Incoming M095 matrix: `336 apply / 29 blocked_primitive / 475
  not_applicable` across 840 TunnelManager option/family cells.
- M153 closed complete as the current runtime/security qualification
  authority; M154 closed complete disposition C (M147 path blocked);
  M146 closed blocked.

Reviewed head:

- M155 audit work completes on the working tree described in §9. No
  `emissary-core/**`, `emissary-util/**`, `emissary-cli/src/**`, manifest,
  lockfile, Yosemite, frontend, or workflow path was created, modified, or
  deleted for production purposes (see §9 and the no-production-diff proof).
  The only Rust change in this closure is the bounded `m062` planning-guard
  repair (§9), which is test-only and explicitly allowed by plan §6
  ("Planning-only paths may be added to M062 now").

Pinned authority (all accessed read-only; no upstream mutation, contact, or
submission occurred — see read-only attestation):

- Proposal 170 revision `2026-05-20`, status Open, SHA-256
  `f13ae00b886c5e72131bc5d5b138a371148d1faa6899a119a1dacb65a555e7dc`
  (cited from M095/M153; live text re-fetched read-only 2026-09-09 to confirm
  the ten-value `EncryptLeaseSet` domain and the `OptionalLookup` /
  `LeaseSetClientAuths` option names — the Proposal lists the ten strings
  with no per-mode I2CP property table, so per-mode wire mapping is frozen
  from the I2CP/ELS2/Yosemite sources below, with provenance marked
  per-row in §1);
- Java I2PControl Proposal-170 head `45bb593000408071dd376b78848fdc246dccd964`
  (cited from pinned records; PR file diff not re-fetched — GitHub PR UI
  requires auth for file fragments; head hash and ten-string domain cited
  from existing records plus the live Proposal text above);
- Java I2P/I2PTunnel reference snapshot `2c3fd2a9532cd86ec06cb6f2b9f3f813ca752243`
  (cited from pinned records; no new fetch; direct Java
  `requiresLS2()` / `LeaseSet.encrypt(SessionKey)` / `LeaseSet2.encrypt`
  line-trace is delegated to M161 live-runtime exercise — M155 freezes the
  structural LS1-vs-LS2 incoherence from specs below and chooses disposition
  C, see §2);
- I2P common-structures specification, `https://geti2p.net/spec/common-structures`,
  read-only fetch 2026-09-09 (LeaseSet LS1 vs LeaseSet2 vs Encrypted LS2
  structures; key-certificate codes 0–11; Destination layout);
- I2P Encrypted LeaseSet specification, `https://geti2p.net/en/docs/specs/encryptedleaseset`,
  read-only fetch 2026-09-09 (three nested layers, blinding derivation,
  Red25519 signing, ChaCha20/HKDF/X25519 layer encryption, DH/PSK per-client
  auth, blinded DHT key, UTC-day rollover, extended B32 format);
- I2P Red25519 specification, `https://geti2p.net/en/docs/specs/red25519`,
  read-only fetch 2026-09-09 (RedDSA instantiation, conversion functions,
  sign/verify equations, ten test vectors);
- I2P Proposal 170 text, `https://i2p.net/proposals/170-i2pcontrol-expansion.txt`,
  read-only fetch 2026-09-09 (ten `EncryptLeaseSet` strings verbatim);
- i2pd tunnel docs (read-only websearch excerpts 2026-09-09):
  `i2cp.leaseSetType` 1/3/5 (OLD/STANDARD/ENCRYPTED), `i2cp.leaseSetEncType`,
  `i2cp.leaseSetPrivKey`, `i2cp.leaseSetAuthType` 0/1/2;
- Yosemite optional exact revision `59140a2277bf296928d2e8ce39a148182eeff044`
  (exact transport capability only; typed LeaseSet wire fields verified
  read-only in the checked-out fork at
  `~/.cargo/git/checkouts/yosemite-b3f22cc17f665e22/59140a2/src/options.rs`
  and `src/proto/session.rs`, see §1/§5);
- Emissary source at the reviewed tree (all file:line citations below read
  at HEAD).

Current Proposal matrix at closure (mechanically recomputed, dispositions
unchanged):

- `336 apply / 29 blocked_primitive / 475 not_applicable` (840 cells);
- residual split `SigType` 10, `EncryptLeaseSet` 5, `OptionalLookup` 5,
  `LeaseSetClientAuths` 5, `UseOutproxyPlugin` 4 — identical to M153/M154 entry.

## 1. Exact ten-mode table (plan §2.1)

Proposal source: the live Proposal text lists exactly these ten strings under
`EncryptLeaseSet` (verified 2026-09-09). Emissary already validates exactly
this domain in `emissary-cli/src/i2pcontrol/tunnel_manager.rs:1734-1751`
(`MODES` array; unknown values reject with `INVALID_PARAMS`, proven by
`canonical_validation_rejects_unknown_and_malformed_known_fields`
with `"EncryptLeaseSet": "not-a-mode"` at `tunnel_manager.rs:2145`).
No string is added, removed, or reclassified by M155.

Per-mode I2CP mapping below is frozen from the EncryptedLeaseSet spec (layer
formats, auth types, secret handling), the i2pd I2CP option docs
(`leaseSetType`/`leaseSetAuthType`/`leaseSetKey`/`leaseSetSecret`/
`leaseSetPrivKey`/`leaseSetClient.*`), and Yosemite typed wire evidence
(`options.rs` + `proto/session.rs` lease-option serialization). The Proposal
PR head is cited for the ten-string domain, not for a per-mode property
table (the Proposal text itself carries no per-mode table). Rows marked
`spec-derived` are therefore M155-frozen planning authority for M161/M162,
not claims of live Java-router interop (interop is M157-M162 closure
evidence).

| # | Proposal `EncryptLeaseSet` string | `i2cp.encryptLeaseSet` | `i2cp.leaseSetType` | `i2cp.leaseSetSecret` (OptionalLookup) | `i2cp.leaseSetAuthType` | `i2cp.leaseSetKey` / `PrivKey` / per-client props | Dest signing constraint | Actual wire/storage format | Provenance |
|---|---|---|---|---|---|---|---|---|---|
| 1 | `disable` | absent/false | `1` (default; no ELS2) | absent | `0` | none | type 7 (existing) | ordinary LS2, unblinded DHT key | Proposal text + Yosemite defaults (`encrypt_lease_set:false`, `lease_set_type:1`, `auth:0`) |
| 2 | `encrypted (aes)` | `true`, **without** type-5 ELS2 | not `5` (legacy LS1 path) | absent | n/a (LS1) | legacy `SessionKey` (`leaseSetKey` LS1 sense) | any persistent SigType (LS1) | legacy LS1 `LeaseSet.encrypt(SessionKey)` object — **not** type-5 ELS2; disposition C, see §2 | Proposal string + i2pd `leaseSetType` 1/3/5 docs + common-structures LS1 vs LS2 + M161 delegation |
| 3 | `blinded` | `true` | `5` | absent | `0` | none beyond blinded key | unblinded type 7 → blinded type 11 (derived, not persistent) | type-5 ELS2, layer-1 flags `0x00` (everybody), no auth section; blinded DHT key | ELS spec layers + blinding derivation + Yosemite `leaseSetType=5` |
| 4 | `blinded with lookup password` | `true` | `5` | **required** (`leaseSetSecret` = base64 UTF-8 secret) | `0` | secret feeds `GENERATE_ALPHA` | type 7 → 11 + secret | type-5 ELS2 + secret-derived alpha/store key; wrong/missing secret never falls back | ELS spec `GENERATE_ALPHA(destination,date,secret)` + Yosemite `leaseSetSecret` |
| 5 | `encrypted (psk)` | `true` | `5` | absent | `2` (PSK) | `leaseSetKey`/`PrivKey` layer keys + `leaseSetClient.psk.N` entries | type 7 → 11 | type-5 ELS2, layer-1 PSK block (`authSalt` + `clients` + `authClient` 40B each) | ELS spec PSK auth + Yosemite `authType=2` + `leaseSetClient.psk.N` |
| 6 | `encrypted with lookup password (psk)` | `true` | `5` | **required** | `2` | secret + PSK entries | type 7 → 11 + secret | type-5 ELS2, secret alpha + PSK layer-1 | ELS spec (secret + PSK compose) + Yosemite secret + PSK namespace |
| 7 | `encrypted with per-user key (psk)` | `true` | `5` | absent | `2` | per-user PSK entries (`psk.N`) | type 7 → 11 | type-5 ELS2, PSK per-client cookies | ELS spec PSK + Yosemite PSK namespace |
| 8 | `encrypted with lookup password and per-user key (psk)` | `true` | `5` | **required** | `2` | secret + per-user PSK entries | type 7 → 11 + secret | type-5 ELS2, secret alpha + PSK per-client | ELS spec composition |
| 9 | `encrypted with per-user key (dh)` | `true` | `5` | absent | `1` (DH) | per-user DH entries (`leaseSetClient.dh.N` = `b64name:b64key` X25519) + server `esk`/`epk` | type 7 → 11 | type-5 ELS2, layer-1 DH block (`epk` + `clients` + `authClient`) | ELS spec DH auth + Yosemite `authType=1` + `leaseSetClient.dh.N` |
| 10 | `encrypted with lookup password and per-user key (dh)` | `true` | `5` | **required** | `1` | secret + per-user DH entries | type 7 → 11 + secret | type-5 ELS2, secret alpha + DH per-client | ELS spec composition |

Yosemite wire proof (read-only, checked-out fork):

- `options.rs:159-186` reserves `i2cp.encryptLeaseSet`, `leaseSetAuthType`,
  `leaseSetBlindedType`, `leaseSetType`, `leaseSetKey`, `leaseSetPrivateKey`,
  `leaseSetSecret`, `leaseSetSigningPrivateKey` plus `leaseSetPrivKey` /
  `leaseSetSigningPrivKey` aliases and `leaseSetClient.dh./psk./Auth` namespaces;
- `options.rs:325-424` defines `LeaseSetClientAuthMode::{Dh,Psk}` and
  `LeaseSetClientAuth::{dh,psk}` with I2P-base64 32-byte key validation,
  `wire_key_prefix` (`i2cp.leaseSetClient.dh.` / `.psk.`), `wire_value`
  (`b64name:b64key`), redacted `Debug`;
- `options.rs:639-760` documents `lease_set_auth_type` (`0..=2`),
  `lease_set_key`/`private_key`/`secret`/`signing_private_key`,
  `lease_set_type` (`1..=255`), `encrypt_lease_set`, bounded
  `lease_set_client_auths` (mode must match selected nonzero auth type,
  `lease_set_type == 5` required);
- `proto/session.rs:270-330` serializes exactly `i2cp.encryptLeaseSet=true`,
  `leaseSetAuthType`, `leaseSetBlindedType`, `leaseSetType`,
  `leaseSetKey`/`PrivateKey`/`Secret`/`SigningPrivateKey`, and the selected
  `leaseSetClient.{dh,psk}.N` namespace sorted deterministically;
- Emissary M124 reachability tests prove the typed path without Proposal
  mapping: `backends/runtime/session.rs:1697-1755`
  (`m124_y005_coherent_leaseset_wire_is_reachable_at_fake_sam`) asserts
  `encryptLeaseSet=true`, `authType=1/2`, `blindedType=10`, `type=5`,
  `leaseSetKey/Secret/PrivateKey/SigningPrivateKey`, `dh.0`/`psk.0`
  namespaces and canonical-spelling rejections.

Required destination signing constraint for rows 3–10: unblinded signing key
is type 7 Ed25519 (existing Emissary capability); blinded key is type 11
Red25519 derived internally per §3. No other SigType is required or allowed
for the modern path (§3 explicitly states this does not reopen M147/M148).

## 2. Legacy AES disposition (plan §2.2): **C — unresolved, delegated to M161**

M155 chooses disposition **C**: leave `EncryptLeaseSet` blocked and stop any
plan that assumes full ten-value completion; M161 must exercise the live
reference and return A/B/C with runtime evidence before `EncryptLeaseSet`
promotion is ever considered. M155 does not silently reclassify or remove
`encrypted (aes)`.

Direct source evidence (read-only):

- Proposal PR maps this mode to legacy `i2cp.encryptLeaseSet` behavior
  without type-5 ELS2 (ten-string domain verified in Proposal text; per-mode
  legacy sense verified in i2pd `leaseSetType` 1/3/5 docs: `1` OLD deprecated,
  `3` STANDARD default, `5` ENCRYPTED; legacy AES is the non-`5` member).
- Current Java `requiresLS2()` behavior forces LS2 when supported (cited from
  pinned records; live Java line-trace delegated to M161 per plan §2.2 —
  M155 does not claim a fresh Java source line it did not re-fetch).
- `LeaseSet.encrypt(SessionKey)` is legacy LS1 behavior (ElGamal/AES+SessionTag
  heritage; Destination public key field noted as deprecated IV use in
  common-structures `Destination` notes); `LeaseSet2.encrypt(SessionKey)` is
  the unsupported/incorrect reinterpretation (ELS spec uses ChaCha20/HKDF
  nested layers, not a single AES call) — structural incoherence frozen from
  specs, not inferred.
- Whether an actual current I2PTunnel service using this value publishes a
  valid encrypted object is **not proven** in M155 (no live-router exercise);
  hence C rather than B. M161 must directly trace and, where practical,
  exercise the pinned snapshot (`2c3fd2a…`) I2PTunnel/I2CP LS1-vs-LS2 selection
  and record whether the mode publishes usable encrypted state against an
  LS2-capable router.
- Implementing it in Emissary would require adding a legacy LeaseSet1
  publication stack: Emissary has **no** LS1 construction/parsing/publication
  owner — `emissary-core/src/primitives/lease_set.rs` implements only
  `LeaseSet2`/`LeaseSet2Header` (structs at `:45-60`, `:248-258`; no `LeaseSet`
  LS1 struct exists); `emissary-core/src/i2np/database/store.rs:39-81`
  parses `EncryptedLeaseSet`/`MetaLeaseSet` store-type codes but
  `DatabaseStoreKind` (`:342-380`) implements only `RouterInfo` and `LeaseSet2`
  payloads (serialize `:407-439`, parse `:571-605` — no LS1 or Encrypted-LS2
  payload variant). A legacy-AES implementation would therefore be a new
  LS1 wire/publication/lookup subsystem, not a flag on the modern path. No
  ad-hoc reinterpretation of legacy AES as type-5 ELS2 is permitted.

Security invariants for the delegation: no silent aliasing to a modern mode,
no plaintext fallback, no broad legacy resurrection without a dedicated
implementation plan, no changes to M156-M160 primitives (M161 plan §Security
inherited).

## 3. Narrow Red25519 requirement (plan §2.3)

Frozen: modern Encrypted LS2 uses the **already-supported type-7 Ed25519
Destination** and derives a **type-11 Red25519 blinded key** without making
type 11 a persistent user-selectable Destination `SigType`. This primitive
does not reopen M147/M148 and does not authorize DSA/P256/P384/P521
generation. M154 disposition C stands.

### 3.1 Formula table (EncryptedLeaseSet + Red25519 specs, read-only fetch 2026-09-09)

| Item | Frozen formula / rule | Source |
|---|---|---|
| Curve/order | `B` = Ed25519 basepoint; `L` = `2^252 + 27742317777372353535851937790883648493` | ELS §Blinding Definitions; Red25519 §Definitions |
| Daily alpha derivation | `stA` = `0x0007`/`0x000b` BE, `stA'` = `0x000b` BE; `keydata = A \|\| stA \|\| stA'`; `datestring` = 8B ASCII `YYYYMMDD` UTC; `secret` = UTF-8 (possibly empty); `seed = HKDF(H("I2PGenerateAlpha",keydata), datestring \|\| secret, "i2pblinding1", 64)`; `alpha = seed mod L` (seed as 64B LE) | ELS §Blinding Calculations `GENERATE_ALPHA` |
| Ed25519 scalar conversion | `seed = privkey_bytes`; `a = clamp(left_half(SHA512(seed)))` (Ed25519 type 7); `a = privkey` directly for Red25519 type 11 | ELS `BLIND_PRIVKEY`; Red25519 `CONVERT_ED25519_PRIVATE` (`s[0]&=248`, `s[31]=(s[31]&63)\|64`) |
| Blinded public key | `A' = BLIND_PUBKEY(A,alpha) = A + DERIVE_PUBLIC(alpha) = A + [alpha]B` (Edwards-point addition; on-curve prime-order subgroup) | ELS `BLIND_PUBKEY`; Red25519 `RANDOMIZE_PUBLIC(vk,alpha) = vk + [alpha]B` |
| Blinded private scalar | `a' = BLIND_PRIVKEY(a,alpha) = (a + alpha) mod L` | ELS `BLIND_PRIVKEY`; Red25519 `RANDOMIZE_PRIVATE(sk,alpha) = (sk+alpha) mod L` |
| Agreement | `BLIND_PUBKEY(pubkey,alpha) == DERIVE_PUBLIC(BLIND_PRIVKEY(privkey,alpha))` | ELS Definitions; Red25519 Design |
| Red25519 randomized signing | `T = 80 rand bytes`; `r = H*(T \|\| vkBytes \|\| m)`; `R=[r]B`; `c=H*(Rbytes \|\| vkBytes \|\| m)`; `S=(r + c*sk) mod L`; sig = `Rbytes \|\| S`; every signature differs even for same key/message | ELS §Sign/Verify; Red25519 `SIGN(sk,m)` with `HStar` (`SHA-512("I2P_Red25519H(x)" \|\| prefix1 \|\| prefix2 \|\| len_u16(m) \|\| m) mod L`) |
| Verification | Ed25519-form with cofactor check: `(-[S]B) + R + ([c]vk)` times cofactor is identity; `S < L`, `R` valid point | Red25519 `VERIFY` |
| Blinded DHT storage key | `SHA-256(sigtype \|\| blinded_pubkey)`; rotated daily (new alpha/store key each UTC day) | ELS §Format ("DHT storage location is SHA-256(sig type \|\| blinded public key), and rotated daily") |
| UTC-day rollover | new `alpha`/blinded keys each UTC day; republish under new store key through canonical publication owner; never mix old/new material | ELS "A new secret alpha and blinded keys must be generated each day (UTC)" |
| Optional blinding secret | `secret` UTF-8 contributes to `GENERATE_ALPHA` seed (`datestring \|\| secret`); extended B32 + `leaseSetSecret` carry it; wrong/missing secret never falls back | ELS `GENERATE_ALPHA(destination,date,secret)` + §Encrypted LS with Base32 Addresses |
| Subcredential binding | `credential = H("credential", A \|\| stA \|\| stA')`; `subcredential = H("subcredential", credential \|\| A')`; layer keys bind `subcredential \|\| publishedTimestamp` (+ `authCookie` for layer 2) | ELS §Derivation of subcredentials, Layer 1/2 encryption |
| Layer encryption | `keys = HKDF(salt, input, "ELS2_L1K"/"ELS2_L2K", 44)`; `key=keys[0:31]`, `IV=keys[32:43]`; `ciphertext = salt \|\| ENCRYPT(ChaCha20)`; DH per-client `okm = HKDF(epk, sharedSecret \|\| cpk \|\| subcred \|\| ts, "ELS2_XCA", 52)`; PSK `okm = HKDF(authSalt, psk \|\| subcred \|\| ts, "ELS2PSKA", 52)`; `clientID=okm[44:51]`, `cookie=ENCRYPT(clientKey,clientIV,authCookie)` | ELS §§Layer 1/2 encryption, Per-client authorization |
| Extended blinded address | `data = ((flags \|\| unblinded_sigtype \|\| blinded_sigtype) XOR checksum) \|\| 32B pubkey`; `address = Base32Encode(data) + ".b32.i2p"`; 56 chars (35B) vs 52 (32B); 5 unused bits zero | ELS §Encrypted LS with Base32 Addresses; naming spec / proposal 149 |

Cross-reference vector source: Red25519 spec **Test vectors 1–10** (each with
`edsk`/`edpk`/`sk`/`vk`/`msg`/`sig`/`alpha`/`rsk`/`rvk`/`rsig`; e.g. vector 1
`edsk=01…01`, `edpk=8a88e3dd…`, `sk=58e86efb…`, `alpha=ae9ba9cb…`,
`rsk=8bb85f3c…`, `rvk=6fe12873…`). M156 must ingest at least vector 1 as a
known-answer gate plus alpha/store-key vectors from the ELS derivation.

Explicit non-reopening: type 11 remains a **derived blinded-key type only**.
`Destination::new` stays type-7-only; SAM `DEST GENERATE` stays `"7"`-gated;
secret-store envelope stays v1/type-blind; no DSA/ECDSA generation is added.
Any claim otherwise requires a separate architecture/security decision
superseding M154, not M156-M162.

## 4. Field coupling (plan §2.4)

| `EncryptLeaseSet` mode | Consumes `OptionalLookup` (`leaseSetSecret`) | Consumes `LeaseSetClientAuths` PSK | Consumes `LeaseSetClientAuths` DH |
|---|---|---|---|
| `disable` | no | no | no |
| `encrypted (aes)` | no (legacy path; M161-owned) | no | no |
| `blinded` | no | no | no |
| `blinded with lookup password` | **yes (required)** | no | no |
| `encrypted (psk)` | no | **yes** | no |
| `encrypted with lookup password (psk)` | **yes** | **yes** | no |
| `encrypted with per-user key (psk)` | no | **yes** | no |
| `encrypted with lookup password and per-user key (psk)` | **yes** | **yes** | no |
| `encrypted with per-user key (dh)` | no | no | **yes** |
| `encrypted with lookup password and per-user key (dh)` | **yes** | no | **yes** |

Determination: `OptionalLookup` and `LeaseSetClientAuths` **can** truthfully
promote independently at M162 once their own complete semantics are
operational (secret-derived blinding with restart-safe redacted custody and
reference lookup interop; bounded PSK/DH authorization with per-mode
namespaces, restart behavior, and authorized/unauthorized interop), even if
`EncryptLeaseSet` remains blocked because the legacy AES enum value is
unavailable. Rationale: the blocked `EncryptLeaseSet` cells record an
incomplete ten-value domain, not missing lookup/auth primitives; holding
lookup/auth hostage to a dead legacy value would conflate independent
contracts. Default promotion staging is **defer to M162**; M155 authorizes an
exception of up to 5 `OptionalLookup` promotions at M158 **only** with
full-contract proof (secret alpha/store-key vectors, extended-B32 interop,
restart/rotation, no-downgrade) — otherwise M158 stays zero-promotion.
`EncryptLeaseSet` promotes (up to 5) only when **every valid Proposal value**
is operational for that family (plan promotion rule; ceilings `346/19/475`
without legacy AES, `351/14/475` with it — ceilings, not claims).

## 5. Current Emissary owner audit (plan §3)

Read at the reviewed tree. Each row classifies the future change per plan §3
(reusable / new-neutral-in-existing-owner / I2PControl-only / prohibited).

| Capability | Exact current owner + functions | Status | Future-change class |
|---|---|---|---|
| Type-7 signing private/public keys | `emissary-core/src/crypto/mod.rs:527-580` (`SigningPrivateKey::Ed25519`, `random`, `from_bytes` (32B-only), `sign`, `public`, `signature_len` 64, `From<[u8;32]>`, `AsRef`); `:584-652` (`SigningPublicKey::{Ed25519,P256,DsaSha1}`, `from_bytes` 32B, `verify` strict Ed25519 / P256 / DSA, `signature_len`); `SigningKeyKind::try_from` `:101-112` (0/1/7 parse-only) | generate/sign/verify/serialize/persist for type 7 only | **reusable existing primitive** (M156 input) |
| Ordinary LeaseSet2 parse/serialize/sign | `emissary-core/src/primitives/lease_set.rs:45-100` (`LeaseSet2Header::parse_frame`), `:248-426` (`LeaseSet2::{parse,parse_frame,serialize,random}`; `serialized_len` `:386-393` hardcodes `+64` sig; `sign()` call `:396-426`) | ordinary LS2 only; no type-5 outer/inner, no blinding | **new neutral primitive in existing exact owner** (M157 extends this file; no broad `primitives/**`) |
| Destination LeaseSet construction/publication | `emissary-core/src/destination/lease_set.rs:56-120` (timeouts, `RetryKind`, `PublishState`), full publisher through `:3358` (inbound-lease collection, `DatabaseStoreBuilder` publish, storage verification); `destination/mod.rs`, `destination/session/mod.rs` (session/consumer threading) | ordinary LS2 publication + storage verification only; no blinded key, no rollover on UTC-day, no encrypted layers | **new neutral primitive in existing exact owners** (M157/M158 extend `destination/lease_set.rs`; no second NetDB subsystem) |
| DatabaseStore parsing/building/store-type encoding | `emissary-core/src/i2np/database/store.rs:39-81` (`StoreType::{RouterInfo,LeaseSet,LeaseSet2,EncryptedLeaseSet,MetaLeaseSet}`, `from_u8`/`as_u8` with `(store_type>>1)&0x7` mapping); `:342-439` (`DatabaseStoreKind::{RouterInfo,LeaseSet2}` only + `new`/serialize); `:571-633` (parse `LeaseSet2` only) | type code parses Encrypted/Meta but **no payload variant** exists | **new neutral primitive in existing exact owner** (M157 adds Encrypted payload repr; exact file only) |
| LeaseSet storage-verification lookup key | `destination/lease_set.rs:65-78` (`STORAGE_VERIFICATION_*` timeouts), publish/verify state machine; `netdb/mod.rs:122-175` (`lease_sets` map), `:441-480` (`on_lease_set_store`), `:579-640` (`on_lease_set_lookup`), `:1314-1380` (`query_lease_set`) | unblinded destination-hash key only; no blinded `SHA-256(sigtype\|\|A')` key | **new neutral primitive in existing exact owners** (M157 switches key derivation for encrypted mode) |
| Destination/server secret stores + generation transactionality | `emissary-cli/src/i2pcontrol/client_secret_store.rs` (envelope v1, `validate_private_key`, `parse_signature_type:539-544` exactly `"7"`, base64/shape-only, type-blind); `server_secret_store.rs` (identity→destination map, atomic publish, `validate_destination`); `stores/generation_store.rs`, `stores/tunnel_store.rs` (generation/staging) | type-7-only, type-blind validation, no suite field; generation-transactional for ordinary identities | **I2PControl-only policy/storage mapping** for secret/PSK/DH custody (M158-M160 extend these exact files; no versioned migration for SigType — M147 stays blocked) |
| Standard SAM/Yosemite session-option transport | `emissary-cli/src/i2pcontrol/backends/runtime/session.rs:980-1128` (`build_session_options`: `SessionOptions` construction `:1009-1015`, `lease_set_enc_type :1022-1030`, `apply_session_wire_options :1032`); `:1132-1200` (`apply_session_wire_options`, `sig_type→u16 :1145-1151` unreachable mapping); Yosemite `options.rs` + `proto/session.rs:270-330` (typed lease-option serialization, §1) | generic `leaseSetEncType` + Yosemite typed LS2 fields reachable (M124 tests `:1697-1755`); **no** Proposal ten-mode mapping | **reusable existing primitive** (transport stays; M162 adds Proposal mapping in I2PControl only) |
| X25519 DH | `crypto/mod.rs:143-233` (`StaticPublicKey::X25519`), `:237-363` (`StaticPrivateKey::{random,from_bytes,diffie_hellman→[u8;32]}` via `x25519-dalek 3.0.0-pre.6` `reusable_secrets,static_secrets,zeroize,precomputed-tables`); `:366-437` (`EphemeralPrivateKey/ReusableSecret` DH) | maintained constant-time X25519 available | **reusable** (M160 reuses; no second curve impl) |
| HKDF/HMAC/SHA256 | `crypto/hmac.rs:24-53` (`Hmac` over `hmac 0.12.1` + `sha2`), `crypto/sha256.rs:24-67` (`Sha256` over `sha2 0.10.9`), `crypto/noise.rs:19-82`, `siphash.rs:52-71`, SSU2 `HKDFSSU2DataKeys` HMAC-chaining (`transport/ssu2/...:810-825,897-909`) | HMAC-SHA256 + SHA-256 available; **no `hkdf` crate** in `Cargo.lock` (verified — `hkdf` absent) | **new neutral HKDF-SHA256 in existing exact owner** (build RFC-5869 extract/expand from existing `hmac`+`sha2`, or exact `hkdf 0.12.4` only if M156 registration proves the hand-rolled composition unsafe — default is no new dep) |
| ChaCha20 + secure RNG | `crypto/chachapoly.rs:70-210` (`ChaChaPoly` over `chacha20poly1305 0.10.1 alloc`, `ChaCha` over `chacha20 0.9.1 zeroize`); `rand 0.10.0 alloc` + `rand_core::CryptoRng` (`crypto/mod.rs:28,253,382`), `zeroize 1.8.1` | ChaCha20/Poly1305 + CSRNG + zeroization available | **reusable** (M157-M160 reuse; nonces unique-per-key per ELS STREAM) |
| Extended `.b32.i2p` | `crypto/mod.rs:45-79` (`I2P_BASE64` `-~`, `I2P_BASE32` `a-z2-7`, `base32_encode/decode` ordinary only) | ordinary B32 only; **no** 56-char extended blinded-address codec (flags/sigtypes/checksum/pubkey) | **new neutral primitive in existing exact owner** (M158 adds codec; exact file only, no generic resolver) |

Prohibited/unnecessary broad rewrites (explicitly **not** required): any
`crypto/**`, `netdb/**`, `i2np/**`, `destination/**`, `primitives/**` prefix
allowance; second NetDB query engine; generic runtime algorithm registry;
transport/RouterInfo/frontend/I2PControl-policy changes for neutral crypto;
router-identity paths (`primitives/router_identity.rs`,
`primitives/router_info.rs:249,262`, `router/mod.rs:206-210`); Yosemite fork
change (stays `59140a2`); direct-clearnet/DNS egress.

## 6. Dependency/security freeze (plan §4)

Priority order applied (plan §4.1–4.5):

1. **Direct use of already-transitive maintained curve25519 at exact
   compatible version.** `Cargo.lock` pins `curve25519-dalek 5.0.0-pre.6`
   (via `ed25519-dalek 3.0.0-pre.6` + `x25519-dalek 3.0.0-pre.6`, both
   workspace `3.0.0-pre.6` `default-features=false`). Existing
   `ed25519-dalek` exposes only `SigningKey/VerifyingKey/sign/verify_strict`
   and `x25519-dalek` exposes only X25519 DH — neither exposes Edwards-point
   addition, scalar `mod L` addition/reduction, basepoint multiplication for
   arbitrary scalars, or Red25519 randomized signing. A new **direct**
   `curve25519-dalek 5.0.0-pre.6` dependency is therefore required for M156
   (exact candidate below). No bespoke bignum/curve code is permitted.
2. `no_std + alloc` compatibility: `curve25519-dalek` with
   `default-features=false, features=["alloc","precomputed-tables","zeroize"]`
   matches the existing core posture (`ed25519-dalek alloc,rand_core,fast,
   zeroize`; `x25519-dalek reusable_secrets,static_secrets,zeroize,
   precomputed-tables`; `subtle 2.6.1`, `zeroize 1.8.1`, `sha2 0.10.9`,
   `hmac 0.12.1` all `default-features=false`); M156 tests must prove
   `cargo check -p emissary-core --no-default-features --features no_std`.
3. Audited constant-time scalar/point ops: `curve25519-dalek` (Dalek,
   maintained, constant-time `Scalar`/`EdwardsPoint`, `multiplyByCofactor`
   verification) + `subtle 2.6.1` + `zeroize 1.8.1`; private scalars/alpha/
   seeds never `Debug`/logged, length-validated before allocation.
4. No generic hazmat exposure to I2PControl policy: neutral API exposes typed
   `[u8;32]`/`[u8;64]` blinded keys/signatures with domain-separated derivation
   (`stA/stA'`, `YYYYMMDD`, `I2PGenerateAlpha`/`i2pblinding1`, `credential`/
   `subcredential` personalizations); no `SigType` registry, no Proposal
   strings in core.
5. No bespoke crypto: all arithmetic/KDF/cipher/DH from maintained crates.

Exact candidate if M156 registration confirms the need (planning metadata
only — **no dependency is added in M155**):

- `curve25519-dalek 5.0.0-pre.6`, `default-features = false`,
  `features = ["alloc", "precomputed-tables", "zeroize"]`;
  transitive impact: already in lock via ed25519/x25519-dalek at the identical
  version — direct declaration adds no new version, only a direct edge;
  license: MIT OR Apache-2.0 (Dalek; to be re-pinned from crates.io at
  registration); maintenance: Dalek maintained, audited constant-time;
  why existing deps insufficient: see (1) above; `no_std` proof and
  side-channel/zeroization review are M156 registration gates.
- HKDF: default is RFC-5869 composition from existing `hmac 0.12.1` +
  `sha2 0.10.9` (no new crate); fallback `hkdf 0.12.4` (same hash, audited)
  only with explicit registration evidence.
- All other primitives reuse existing direct deps (table §5). `redjubjub`
  (Zcash RedJubjub, not I2P RedDSA) is explicitly **rejected** as a substitute
  (M154 §4 inherited). No `p384`/`p521`/`dsa-signing`/`rsa`/`gost` additions.

## 7. Corrected successor decomposition (plan §5) — validated unchanged

```text
M155 semantic/owner refreeze                 [CLOSED by this record; ZERO PRODUCTION]
  |
  v
M156 narrow Red25519 + Ed25519 blinding      [REGISTERED next; neutral primitive; zero promotion]
  |
  v
M157 modern Encrypted LS2 publication        [DEFERRED; ZERO PROMOTION]
  |
  v
M158 lookup-secret + blinded-address support [DEFERRED]
  |
  v
M159 PSK client-authorization primitive      [DEFERRED; ZERO PROMOTION]
  |
  v
M160 DH client-authorization primitive       [DEFERRED; ZERO PROMOTION]
  |
  +--> M161 legacy AES/LS1 feasibility       [DEFERRED; ZERO PRODUCTION; may execute after M160]
  |
  v
M162 Proposal LeaseSet-field integration     [DEFERRED; CONDITIONAL PROMOTIONS]
  |
  v
M152 final residual requalification          [DEFERRED; ZERO PROMOTION]
```

No smaller or safer split was found. PSK (M159) and DH (M160) stay separate
(materially different key semantics, CPU bounds, and review surfaces);
lookup-secret/addressing (M158) stays separate from publication (M157)
(address codec + secret custody vs layer crypto); legacy AES stays on its own
feasibility branch (M161) so it cannot contaminate modern ELS2 work. M162
entry still requires M156-M160 closures plus explicit M161 A/B/C (and any
M161-A successor closure before `EncryptLeaseSet` promotion). M152 stays
re-gated on M162 plus any M161-A successor.

M149-M151 decision: **fully superseded** (already marked
`superseded / unregistered; do not execute` with corrective pointers; this
closure confirms no production/dependency/promotion authority remains in them
and no re-gating is needed — the M155-M162 line replaces their ordering
assumptions).

## 8. Exact-path freeze (plan §6)

Broad prefixes remain forbidden. Planning-only entries for M156-M162 drafts
already exist in `062-dependency-containment.toml:91-98,104` as non-executable
planning metadata (presence confers no production authority). **No production
M061/M062 amendment accompanies this closure** — the exact files below are
planning metadata only and become executable only when each successor is
explicitly registered with M061/M062 authorization in that registration
commit (M154 §9 ceremony inherited).

**Exact proposed production files for M156** (to be authorized at M156
registration, not by this closure):

1. `emissary-core/src/crypto/red25519.rs` (new neutral module: alpha
   derivation, `BLIND_PRIVKEY`/`BLIND_PUBKEY`, Red25519 sign/verify, storage-key
   preimage inputs, typed 32/64-byte values, zeroization, no `Debug` on secrets,
   UTC-day inputs, test-only deterministic-RNG injection);
2. `emissary-core/src/crypto/mod.rs` (module declaration + Proposal-free
   re-export only; no other crypto-file change);
3. `emissary-core/Cargo.toml` + `Cargo.lock` (direct `curve25519-dalek`
   edge from §6 only; no other version churn).

**Exact candidate files for M157-M160** (planning candidates; each successor
registration must re-freeze and authorize only what it actually requires):

- M157: `emissary-core/src/primitives/lease_set.rs` (type-5 outer/inner
  structures); `emissary-core/src/i2np/database/store.rs` (Encrypted payload
  repr + store-type handling); `emissary-core/src/destination/lease_set.rs`
  (blinded publication/storage-verification/rollover); plus M156's
  `crypto/red25519.rs` (reuse).
- M158: M156/M157 owners above plus extended-B32 codec in
  `emissary-core/src/crypto/mod.rs` (or the new `red25519.rs` if registration
  places the codec there — one exact file, to be frozen at registration) and
  secret custody in `emissary-cli/src/i2pcontrol/server_secret_store.rs`
  (I2PControl-only; redacted, generation-transactional).
- M159: `emissary-core/src/primitives/lease_set.rs` (PSK layer-1 block) +
  `crypto/chachapoly.rs` + `crypto/hmac.rs`/`sha256.rs` (HKDF/ ChaCha reuse) +
  `destination/lease_set.rs` (authorized publication) +
  `emissary-cli/src/i2pcontrol/server_secret_store.rs` (persistent PSK custody).
- M160: same pattern as M159 plus `emissary-core/src/crypto/mod.rs`
  X25519 reuse (`StaticPrivateKey`/`EphemeralPrivateKey`); no second curve impl.
- M161: zero production (no files).
- M162: exact `emissary-cli/src/i2pcontrol/**` backend/options/secret-store/
  session-generation paths plus tests/matrix/docs (to be frozen at registration;
  any new lower-layer requirement stops M162 and creates a separate neutral plan).

## 9. Verification outcomes

| Command group | Result |
|---|---|
| `cargo check -p emissary-core` | **pass** |
| `cargo check -p emissary-core --no-default-features --features no_std` | **pass** |
| `cargo test -p emissary-core --lib --no-fail-fast` | **pass**: `1080 passed, 2 ignored` |
| `cargo check -p emissary-cli --no-default-features --features i2pcontrol` | **pass** |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment --test m062_dependency_containment --test m095_full_support_matrix --test m105_residual_option_audit --test m153_post_m146_requalification --no-fail-fast` | **pass**: `37 passed (5 suites, 0 failed)` — includes the §10 test-only repairs for pre-existing M154 drift (`154-closure.md` uncovered) and post-M154 supersede drift (stale post-M146 active list, missing AGENTS M139) |
| M095 mechanical recomputation (`python3` TOML parse over `tunnel_manager.options[*].cells`) | **pass**: recomputed `840/336/29/475` == declared; residual split `10/5/5/5/4` (SigType 10 incl. 2 N/A-shaped families, Encrypt/Optional/Auth 5 each on servers, Outproxy 4) |
| Production-head determination (`git log 7cbd80a..HEAD` over all production paths) | **pass**: empty — last production-bearing commit remains `7cbd80a...` |
| `git diff --check` | **pass** |

### Production-path diff (bounded)

`git status --short` over `emissary-core/src`, `emissary-cli/src`,
`emissary-util/src`, all manifests, and `Cargo.lock` is empty except the
test-only `m062`/`m153` guard repairs (§10): M155 makes zero production changes.
`110-completion-ledger.toml` gains no entry (zero promotions). `061`
unchanged. Streamr datagram limits (16-subscriber, 60s expiry, 1200-byte
payload, 4095-byte transport buffer, 15s refresh, bounded shutdown,
loopback-only UDP) untouched; remote datagrams never choose a local UDP
destination.

## 10. Changed paths (exact budget only)

- `plans/implementation/i2pcontrol-proposal-170/155-post-m154-leaseset-security-semantic-and-owner-refreeze.md`
  (Status `registered / dependency-ready` → `closed as complete` with closure link);
- `plans/implementation/i2pcontrol-proposal-170/156-neutral-red25519-blinding-primitive.md`
  (Status `deferred / unregistered` → `registered / dependency-ready` on the
  satisfied M155 hard dependency; §12);
- new `plans/closure/i2pcontrol-proposal-170/155-closure.md` (this file);
- `emissary-cli/tests/m062_dependency_containment.rs` (test-only planning-guard
  repair: add `is_authorized_m154_path` covering `154-closure.md` and
  `is_authorized_m155_path` covering `155-closure.md` + this closure's
  planning/docs/test set; wire both into the permitted and prohibited-pattern
  assert chains. Repairs the pre-existing M154 drift where `154-closure.md`
  failed `allowed_production_paths_match_the_m062_budget` because no helper
  covered it. No production path, dependency, or lockfile change; no broad
  waiver);
- `emissary-cli/tests/m153_post_m146_requalification.rs` (test-only stale-authority
  repair: active docs list tracks the superseded post-M146 roadmap → the active
  post-M154 roadmap. Repairs drift from `bee898b` where the post-M146 file went
  historical and lost the `partial` wording, failing
  `m153_authority_docs_name_m153_and_retain_partial_support`. No behavior change;
  historical file untouched);
- `plans/registry.md` (M155 → closed as complete; M156 → registered /
  dependency-ready; execution chain, registration rules, lineage updated);
- `plans/implementation/i2pcontrol-proposal-170/README.md` (M155 closed
  section; M156 registered; chain updated);
- `plans/subsystems/i2pcontrol-proposal-170-post-m154-leaseset-security-corrective-roadmap.md`
  (M155 closed; M156 registered; §§3–4 and graph updated);
- `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md`
  (M155 closed; M156 registered; chain/status updated);
- `AGENTS.md` (M155 closed entry with closure link; M156 registered as sole
  handoff; deferred line updated; M139 retained as historical ancestry to keep
  the `m153` authority-docs guard green after the M155-line compaction);
- `docs/i2pcontrol/README.md`, `docs/i2pcontrol/proposal-170-support.md`,
  `docs/i2pcontrol/tunnel-manager.md` (M155 closed authority + `336/29/475`
  wording retained; `EncryptLeaseSet`/`OptionalLookup`/`LeaseSetClientAuths`
  residual-blocked statements retained).

`061-containment-boundary.toml`, `062-dependency-containment.toml`,
`095-full-support-matrix.toml`, `105-residual-option-audit.toml`, and
`110-completion-ledger.toml` are intentionally **unchanged** (zero promotions,
zero executable path/dependency authorisation; §8 candidates are planning
metadata only). `git diff --check` passes.

## 11. Requirement-to-evidence matrix (planning process §2.5)

Covered in §9 (plan requirements) plus:

| Closure duty | Evidence |
|---|---|
| implementation commits | none — planning/audit-only milestone; the commit landing with this closure carries tests/plans/docs only (§9) |
| invariant review | partial-support, no-fabrication, I2P-only egress, fail-closed option validation, containment, and Y005 invariants re-proved by the green `m061`/`m062`/`m095`/`m105`/`m153` guards; no invariant weakened |
| failure/recovery and contention evidence | fail-before-allocation `EncryptLeaseSet` enum gate (`tunnel_manager.rs:1734-1751`), `LeaseSetClientAuths` shape gate (`:1728-1733`), Yosemite typed validation + M124 pre-wire rejection tests (`session.rs:1764-1850`), atomic secret-store publish paths unchanged and unexercised by this plan |
| compatibility, migration, security review | no wire/protocol/storage/dependency change ⇒ no migration impact; no new attack surface (no production code); all 15 LeaseSet cells stay fail-closed; legacy AES kept blocked (C) with no silent aliasing; Red25519/secret material handling frozen as M156 preconditions, not implemented |
| documentation and operational evidence | §10 changed paths; authority docs name M155 closed / M156 registered and retain partial support; operational impact none |
| M155 acceptance (plan §9) | handoff-ready: ten-mode table (§1), legacy AES disposition C (§2), Red25519 formulas + vector source (§3), owner/capability table (§5), dependency review (§6), field-coupling table + promotion rule (§4), exact M156 files + M157-M160 candidates (§8), M149-M151 superseded confirmation (§7), unchanged `336/29/475` (§9), zero-diff proof (§9), registry/roadmap/M062 reconciliation (§10/§12) |

## 12. Registry updates and future-plan unblock determination

Applied alongside this closure (see §10 for the file list):

- `155-*.md` plan: Status `registered / dependency-ready` → `closed as
  complete` with closure link;
- `plans/registry.md`: M155 → closed as complete (`336/29/475` unchanged,
  zero promotions, zero production diff); M156 → **registered /
  dependency-ready** (sole handoff); execution chain and registration rules
  updated (only M156 may begin; exact M061/M062 authorization required at
  registration);
- `110-completion-ledger.toml`: no new entry (zero promotions);
- `m061`/`m062`/`m153`: `061` unchanged; `m062` gains only the test-only
  `is_authorized_m154_path` / `is_authorized_m155_path` planning-guard repair
  (no new production paths, no new dependencies); `m153` active-docs list
  re-tracks the post-M154 roadmap after the post-M146 supersede (no behavior
  change, historical file untouched).

Future-plan unblock determination (as required by the tasking):

- **M155 CLOSED as complete.** Its sole hard dependency (M154 closure) was
  satisfied, and all §11 acceptance criteria are met with zero production
  diff and zero matrix promotions. No stop condition in plan §8 triggered:
  type-7→11 blinding is sufficient (no SigType reopen), a maintained
  constant-time primitive path exists (transitive `curve25519-dalek`
  exact-version direct use), the type-5 ELS2 format is frozen from specs, no
  broad rewrite is required (exact files bounded in §8), and field coupling
  is frozen with truthful promotion criteria.
- **M156 UNBLOCKED → registered / dependency-ready.** Its hard dependency
  (M155 closure with frozen formulas/vectors, exact files/APIs, dependency/
  no-std/security review, and type-7-sufficiency confirmation) is satisfied
  by this record. At registration, M061/M062 must authorize only the exact
  §8 M156 files plus the direct `curve25519-dalek` edge. No other gate blocks it.
- **M157 remains deferred/unregistered**, hard-gated on M156 closure plus the
  §8 exact-path amendment (layer formats, DatabaseStore type-5, blinded key,
  publication/verification/rollover owners, vectors). M155 creates no shortcut.
- **M158 remains deferred/unregistered**, hard-gated on M157 closure. The
  conditional M158 `OptionalLookup` promotion remains **unauthorized** until
  M155's full-contract proof gate is met at M158 closure (default is defer to
  M162).
- **M159 remains deferred/unregistered**, hard-gated on M158 closure.
- **M160 remains deferred/unregistered**, hard-gated on M159 closure.
- **M161 remains deferred/unregistered**, hard-gated on M155 closure (satisfied)
  but sequenced after M160 per the roadmap; it may now be prepared but must not
  overtake M156-M160 implementation. Its A/B/C outcome gates M162
  `EncryptLeaseSet` promotion.
- **M162 remains deferred/unregistered**, hard-gated on M160 closure plus M161
  disposition (plus any M161-A successor closure before `EncryptLeaseSet`
  promotion).
- **M152 remains deferred/unregistered**, re-gated on M162 closure plus any
  M161-A successor. Its final qualification must now account for the M155
  field-coupling rule (independent lookup/auth promotion allowed; ten-value
  `EncryptLeaseSet` completeness required).
- **M149-M151 remain superseded/unregistered (do not execute).** Confirmed
  fully superseded by M155-M162; no re-gating or revival is authorized. Their
  files stay as historical drafts with corrective pointers.
- **M147/M148 remain closed-blocked/deferred-behind-M147** (M154 disposition C);
  this line does not reopen them. Type-11 blinding is explicitly not a SigType
  reopening (§3).
- **M146 remains closed as blocked** with no successor; it is not reopened
  by M155 or by any future LeaseSet tail.
- No other future-plan status required a change. File presence alone never
  authorises production work.

## Internal-only / read-only-upstream attestation

- External sources (the Proposal 170 text, the I2P common-structures,
  EncryptedLeaseSet and Red25519 specifications re-fetched read-only
  2026-09-09; i2pd/Yosemite docs excerpts read-only; the already-pinned Java
  I2PControl/I2PTunnel snapshots and Yosemite revision cited from existing
  records; the checked-out Yosemite fork read-only for wire evidence) were
  accessed read-only for evidence;
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
