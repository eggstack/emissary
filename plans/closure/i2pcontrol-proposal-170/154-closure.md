# M154 Closure — M147 Signature Domain, Security, and Exact-Owner Re-freeze

Status: **closed as complete; disposition C — M147 path closed as blocked**

Date: `2026-09-09`

Source implementation plan:

- `plans/implementation/i2pcontrol-proposal-170/154-m147-signature-domain-security-and-owner-refreeze.md`
  (Status now `closed as complete` with closure link; hard dependency on
  M153 closure satisfied by `plans/closure/i2pcontrol-proposal-170/153-closure.md`;
  zero production and zero promotion budgets observed as proven below).

Source roadmaps:

- `plans/subsystems/i2pcontrol-proposal-170-post-m146-corrective-roadmap.md`;
- `plans/subsystems/i2pcontrol-proposal-170-residual-primitive-completion-roadmap.md`;
- `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md`.

## Planning baseline

- Registration baseline `32086c3af43dd242e8c1a592a540c22f7934339f` (clean
  worktree verified before M154 edits; `git status --short` empty).
- Last production-bearing head: `7cbd80a6d72aa07d158ba9dc74f8bbacef767be2`
  (M145 no-std/format follow-up; `git log 7cbd80a..HEAD` over
  `emissary-core/src`, `emissary-cli/src`, `emissary-util/src`, all manifests
  and `Cargo.lock` is empty at both entry and closure).
- Incoming M095 matrix: `336 apply / 29 blocked_primitive / 475
  not_applicable` across 840 TunnelManager option/family cells.
- M153 closed complete as the current runtime/security qualification
  authority; M146 closed blocked with zero production delta.

Reviewed head:

- M154 audit work completes on the working tree described in §9. No
  `emissary-core/**`, `emissary-util/**`, `emissary-cli/src/**`, manifest,
  lockfile, Yosemite, frontend, or workflow path was created, modified, or
  deleted (see §9 and the no-production-diff proof).

Pinned authority (all accessed read-only; no upstream mutation, contact, or
submission occurred — see read-only attestation):

- Proposal 170 revision `2026-05-20`, status Open, SHA-256
  `f13ae00b886c5e72131bc5d5b138a371148d1faa6899a119a1dacb65a555e7dc`
  (cited from M095/M153; live text re-fetched read-only 2026-09-09 to confirm
  the `SigType - signing key type` option carries no enumerated value domain
  of its own — the domain is therefore taken from the I2P key-certificate
  specification plus the Java/go-i2p SAM reference names below);
- Java I2PControl Proposal-170 head `45bb593000408071dd376b78848fdc246dccd964`
  (cited from pinned records; no new fetch);
- Java I2P/I2PTunnel reference snapshot `2c3fd2a9532cd86ec06cb6f2b9f3f813ca752243`
  (cited from pinned records; no new fetch);
- I2P common-structures specification, `https://geti2p.net/spec/common-structures`,
  read-only fetch 2026-09-09 (key-certificate signing-type codes 0–11 with
  Usage column, reserved GOST codes 9–10 per Proposal 134, reserved MLDSA
  codes 12–20 per Proposal 169; frozen as Table 1 below);
- SAM `SIGNATURE_TYPE` reference names observed read-only via go-i2p `sam3`
  constants (`DSA_SHA1`, `ECDSA_SHA256_P256`, `ECDSA_SHA384_P384`,
  `ECDSA_SHA512_P521`, `EdDSA_SHA512_Ed25519`);
- Yosemite optional exact revision `59140a2277bf296928d2e8ce39a148182eeff044`
  (exact transport capability only, not signing-support evidence).

Current Proposal matrix at closure (mechanically recomputed, dispositions
unchanged):

- `336 apply / 29 blocked_primitive / 475 not_applicable` (840 cells);
- residual split `SigType` 10, `EncryptLeaseSet` 5, `OptionalLookup` 5,
  `LeaseSetClientAuths` 5, `UseOutproxyPlugin` 4 — identical to M153 entry.

## 1. Executive finding

M154 is closed as complete with **disposition C: close the M147 path
blocked**. The ten `SigType` cells remain `blocked_primitive`; no crypto is
implemented, no successor is registered, and no production, dependency,
matrix, or containment change occurred.

No single bounded neutral signature-suite primitive (disposition A) can cover
the pinned required domain, and no honest split (disposition B) can unblock
M148, because:

1. the pinned destination-capable domain is `{0, 1, 2, 3, 7, 11}` (§2) —
   six suites with materially independent dependencies, key/signature
   lengths (private 20–32 bytes for the small suites, up to 66 bytes for
   P521; signatures 40–132 bytes), and security semantics;
2. type 0 (DSA-SHA1/SHA-1) generation is **rejected by project security
   policy** — minting new SHA-1/1024-bit-DSA destination identities is
   irresponsible, while verification of existing remote identities must stay
   for NetDB interop. A configurable `SigType` field cannot truthfully ship
   with a policy-refused value silently narrowed out (§2, §4);
3. types 1–3 are spec-deprecated legacy ECDSA; only type 1 even verifies
   today, none generates/signs, and types 2–3 need wholly new audited
   dependencies plus a variable-length config/storage migration (§4, §5);
4. type 11 (RedDSA) has **no maintained I2P-suitable Rust primitive**
   identified — `redjubjub` is Zcash-bound, not an I2P RedDSA suite — so the
   required domain cannot be implemented safely within current dependency
   policy (§4);
5. the structural owners assume fixed Ed25519 sizes end to end
   (`config.signing_key: Option<[u8; 32]>`, `Destination::new` hardcoded
   32/32 shape, secret-store envelope v1 with no suite field and type-blind
   validation, SAM parser string-`"7"` gate, Yosemite hardcoded 7 under
   separate ADR-0005 authority) — variable-suite support is a broad
   crypto/config/storage migration, which is an explicit M154 stop condition
   (§5, §6).

Building a modern-only subset as infrastructure with no capability consumer
that can ever satisfy M148 would violate the sizing rule against
infrastructure with no consumer and the prohibition on fabricating support
by narrowing the domain. M154 therefore advances no implementation and
registers nothing.

## 2. Exact SigType domain table (WP1)

Table 1 freezes the pinned contract. Lengths are public/private/signature
bytes from the I2P common-structures tables fetched read-only 2026-09-09.
"Destination use" is the spec Usage column for key certificates.

| Code | Canonical name | Pub | Priv | Sig | Destination use | Disposition |
|---|---|---:|---:|---:|---|---|
| 0 | `DSA_SHA1` | 128 | 20 | 40 | NULL-cert or key-cert; deprecated for RIs, discouraged for Destinations | `legacy_compatibility_only` (verify retained) + `rejected_by_policy` (generation) |
| 1 | `ECDSA_SHA256_P256` | 64 | 32 | 64 | key-cert; deprecated older Destinations | `legacy_compatibility_only` |
| 2 | `ECDSA_SHA384_P384` | 96 | 48 | 96 | key-cert; deprecated, rarely used | `legacy_compatibility_only` |
| 3 | `ECDSA_SHA512_P521` | 132 | 66 | 132 | key-cert; deprecated, rarely used | `legacy_compatibility_only` |
| 4 | `RSA_SHA256_2048` | 256 | 512 | 256 | offline only; never RIs or Destinations | `not_required_by_pinned_contract` |
| 5 | `RSA_SHA384_3072` | 384 | 768 | 384 | offline only; never RIs or Destinations | `not_required_by_pinned_contract` |
| 6 | `RSA_SHA512_4096` | 512 | 1024 | 512 | offline only; never RIs or Destinations | `not_required_by_pinned_contract` |
| 7 | `EdDSA_SHA512_Ed25519` | 32 | 32 | 64 | recent RIs and Destinations | `required_and_acceptable` |
| 8 | `EdDSA_SHA512_Ed25519ph` | 32 | 32 | 64 | offline only; never RIs or Destinations | `not_required_by_pinned_contract` |
| 9 | reserved (GOST, Prop 134) | 64 | — | — | reserved, no stable contract | `not_required_by_pinned_contract` |
| 10 | reserved (GOST, Prop 134) | 128 | — | — | reserved, no stable contract | `not_required_by_pinned_contract` |
| 11 | `RedDSA_SHA512_Ed25519` | 32 | 32 | 64 | Destinations + encrypted LeaseSets only, never RIs | `required_and_acceptable`, primitive missing (§4) |
| 12–20 | reserved (MLDSA, Prop 169) | — | — | — | reserved, no stable contract | `not_required_by_pinned_contract` |
| 65280–65535 | experimental / future expansion | — | — | — | reserved | `not_required_by_pinned_contract` |

Required destination-capable domain for a truthful configurable tunnel
`SigType`: **{0, 1, 2, 3, 7, 11}**. RSA/Ed25519ph/GOST/MLDSA/experimental
codes are excluded with explicit spec reasons (offline-only or reserved),
not by silent narrowing: unknown/reserved values must continue to fail
before allocation, exactly as today.

Reference evidence per row: spec key-certificate table (codes, lengths,
Usage); Proposal 170 text (`SigType - signing key type`, no value list);
go-i2p `sam3` `SIGNATURE_TYPE=<name>` constants for 0–3, 7; M121 Outcome C
(singleton type-7 domain is a fixed field, not configurable support).

Emissary's local `SigningKeyKind::try_from` (`emissary-core/src/crypto/mod.rs:101-112`)
recognises only 0/1/7 for parsing — it is a verifier-side convenience, not
capability evidence (per plan §4, capability is proven in §3, and only
type 7 passes all five layers).

## 3. Current Emissary capability inventory (WP2)

Proven at generate / sign / verify / serialize / persist layers, not from
enum constants. Every row was read at the cited source lines on the reviewed
tree.

| Suite | Generate | Sign | Verify | Serialize | Persist | Evidence |
|---|---|---|---|---|---|---|
| 7 Ed25519 | yes | yes | yes | yes | yes | `SigningPrivateKey::{random,from_bytes,sign}` (`crypto/mod.rs:532-551`, single-variant enum `:527-530`, fixed 32→64); `Destination::new` writes `0x0007` cert (`primitives/destination.rs:129-166`); `LeaseSet2::serialize` appends `sign()` output (`primitives/lease_set.rs:396-426`); SAM `DEST GENERATE` handler generates + returns PRIV/PUB (`sam/pending/connection.rs:418-458`); secret stores round-trip base64 (`client_secret_store.rs`, `server_secret_store.rs`) |
| 0 DSA-SHA1 | no | no | yes (remote only) | parse only | no | verify via vendored `crypto/dsa.rs:147-176` (SHA-1, bignum, no signing API); parse via NULL cert (`destination.rs:184-191`) and `SigningPublicKey::dsa_sha1` (`crypto/mod.rs:624-626`); generation impossible — no `SigningPrivateKey` variant, `from_bytes` rejects non-32 (`crypto/mod.rs:539-544`) |
| 1 P256 | no | no | yes (remote only) | parse only | no | verify via `p256` crate (`crypto/mod.rs:614-621,637-642`); parse via key-cert branch (`destination.rs:200-204`) and offline signatures (`offline_signature.rs:74-80`); no private-key type, no `sign()` arm |
| 2 P384 | no | no | no | reject | no | `SigningKeyKind::try_from` returns `Err` (`crypto/mod.rs:109`); `Destination::parse_frame` returns `UnsupportedSigningKey` (`destination.rs:220-223`) |
| 3 P521 | no | no | no | reject | no | same as type 2 |
| 4–6 RSA | no | no | no | reject (destinations) | no | no core RSA dependency; `rsa 0.9.10` exists only under `emissary-util` (`emissary-util/Cargo.toml:23`), unreachable from `emissary-core` signing paths |
| 8 Ed25519ph | no | no | no | reject | no | no pre-hash signing API; key-cert parse rejects (`destination.rs:220-223`) |
| 11 RedDSA | no | no | no | reject | no | no RedDSA code anywhere in tree (`rg RedDSA` empty outside this closure) |

Hardcoded Ed25519 assumptions that M147 would have had to change (each an
exact file, no broad prefix):

- `emissary-core/src/crypto/mod.rs:527-580` — single-variant
  `SigningPrivateKey::Ed25519`, `random`/`from_bytes`/`sign`/`signature_len`
  (64)/`From<[u8; 32]>`/`AsRef<[u8]>`;
- `emissary-core/src/primitives/destination.rs:61,129-166` —
  `KEY_KIND_EDDSA_SHA512_ED25519`, `Destination::new` fixed 32/32 shape;
- `emissary-core/src/sam/parser.rs:867-885` — `DEST GENERATE` accepts only
  the literal string `"7"`, rejects absent/other (tests `:1516-1525`);
- `emissary-core/src/sam/pending/connection.rs:426-451` — handler builds
  `Destination::new` + fixed PRIV blob (`destination || elgamal[256] ||
  ed_priv[32]`);
- `emissary-core/src/config.rs:274` — `signing_key: Option<[u8; 32]>`;
- `emissary-core/src/router/mod.rs:206-210` — router key construction from
  that fixed 32-byte option (router-identity path, must remain untouched);
- `emissary-core/src/primitives/lease_set.rs:386-393` — `serialized_len`
  hardcodes `+ 64` signature;
- `emissary-core/src/primitives/router_info.rs:249,262` — 64-byte signature
  slot (router path, must remain untouched);
- `emissary-cli/src/i2pcontrol/backends/options.rs:215-221` — M121 blanket
  `SigType` reject before allocation (all values incl. `"7"`, no echo);
- `emissary-cli/src/i2pcontrol/backends/runtime/session.rs:1145-1151` —
  unreachable `sig_type`→`signature_type: u16` mapping (defence in depth;
  omitted defaults to 7 as router behaviour, not Proposal support);
- `emissary-cli/src/i2pcontrol/client_secret_store.rs:539-544` —
  `parse_signature_type` accepts exactly `"7"`.

## 4. Dependency and security review (WP3)

- `emissary-core/Cargo.toml`: `ed25519-dalek 3.0.0-pre.6`
  (`alloc,rand_core,fast,zeroize`), `p256 0.13.2`
  (`alloc,ecdsa,ecdsa-core` — verification only, no signer), `sha1 0.10.6`
  (DSA-SHA1 verification only), `sha2 0.10.9`, `subtle 2.6.1`, `zeroize
  1.8.1`, all `default-features = false` for `no_std + alloc`. No
  `rsa`/`dsa`-signing/`p384`/`p521`/`ed448`/`gost`/`reddsa` direct dependency
  exists in core. Workspace pins `ed25519-dalek`/`x25519-dalek 3.0.0-pre.6`,
  `sha2 0.10.9` (`Cargo.toml:27,35,44`).
- `rsa 0.9.10` is in `Cargo.lock` solely via `emissary-util`
  (`emissary-util/Cargo.toml:23`, reseeding/TLS-adjacent use). It is not
  consumable by `emissary-core` signing without a new direct dependency,
  exact-file M062 authorisation, and a variable-length key architecture —
  none of which M154 grants. Its presence proves nothing about destination
  signing capability.
- Per-suite dependency findings:
  - Type 0 generation: would need a DSA signing implementation over the
    vendored bignum verifier or the upstream `dsa` crate (unreviewed,
    non-constant-time bignum heritage, SHA-1). **Rejected**: no safe
    maintained path is accepted, and policy forbids minting SHA-1
    identities regardless.
  - Type 1 signing: `p256` with signer/`rfc6979` features is the maintained
    candidate, but enabling signing expands the trusted API (deterministic
    nonces, side-channel posture, zeroization of P256 scalars) and still
    leaves five other required suites missing — a single-suite addition
    cannot satisfy the configurable field.
  - Types 2–3: `p384`/`p521` (RustCrypto, maintained) are candidates on
    paper only; each needs no-std/alloc proof, zeroization review,
    known-answer fixtures, and the §5 storage migration. Independent
    dependencies with independent audits — the textbook case that fails the
    "one bounded primitive" test for disposition A.
  - Type 11: **no maintained I2P RedDSA candidate identified**. Upstream
    `redjubjub` implements Zcash RedJubjub, not the I2P
    RedDSA-SHA512-Ed25519 suite; adopting or forking it as I2P RedDSA would
    be a new cryptographic design decision, not a dependency bump.
  - Types 9–10 (GOST): reserved with no stable pinned contract; the few
    upstream GOST crates lack the audit standing this project requires for
    identity keys. Excluded as `not_required`; would additionally be
    policy-sensitive if ever required.
- no-std posture: every candidate adds `alloc`-gated bignum/curve code to
  the `no_std` core build; M154 adds none (zero-dependency budget observed).
- Secret handling for any future work is frozen as a requirement, not
  implemented: no `Debug`/log exposure of private bytes (current Ed25519
  type derives only `Clone`, `:526`), `zeroize` on secret buffers,
  length-validation before allocation, no Ed25519 fallback, cert-type must
  match key material. These become M147 preconditions if the path ever
  reopens under a new architecture decision — they are not M154 deliverables.

## 5. Persistence and migration audit (WP4)

- Transient format: `Destination::new` emits padding + signing pubkey +
  `05 0004 0007 0000` (`destination.rs:137-154`). Current private-destination
  bytes (`destination || elgamal_priv[256] || ed_priv[32]`,
  `connection.rs:434-451`) do **not** self-describe the suite beyond the
  embedded key certificate; the 32-byte trailer length is assumed, not
  encoded.
- Persistent formats: client store envelope `{version: 1, entries:
  name → {private_key: base64, import_reference?}}`
  (`client_secret_store.rs:453-518`); server store `identity → base64
  destination` map with atomic publish (`server_secret_store.rs`). Both
  validate base64/shape only (`validate_private_key`, `validate_destination`)
  and are **type-blind**: a non-Ed25519 blob cannot be distinguished, and
  the v1 schema has no suite field.
- Answers to the plan §7 questions: (1) suite is encoded only in the
  Destination key certificate, not in the private-key trailer or store
  envelope; (2) stored bytes round-trip only suite 7 (and parse 0/1 for
  public destinations) — nothing else; (3) existing Ed25519 destinations
  remain byte-compatible (round-trip tests `destination.rs:339-352,363-392`
  green); (4) any additional suite requires a **versioned storage format**
  (or a self-describing private-key container), because trailer lengths vary
  20–1024 bytes and the fixed `[u8; 32]` config/store assumptions break;
  (5) malformed/wrong-type/truncated imports today fail only generic
  base64/shape checks — suite-aware rejection does not exist and must be
  designed; (6) a suite change is a **new destination identity** (identity
  hash covers the full KeysAndCert), requiring regeneration plus the
  existing staged secret-transaction restart semantics — in-place relabelling
  must fail without mutating last-known-good identity.
- No migration exists or is authorised: automatic rewrite of stored
  identities is prohibited, and M154 designs none (disposition C advances no
  storage change).

## 6. Exact owner and path graph (WP5)

Candidate production files M147 *would* have required — recorded as
planning metadata only, **not executable authority** (plan §9; no M061/M062
amendment accompanies this closure since no successor registers):

Core (neutral, would each have needed exact-file authorisation):

1. `emissary-core/src/crypto/mod.rs` — `SigningPrivateKey` enum + all
   methods, `SigningKeyKind::try_from`, `SigningPublicKey::{from_bytes,
   verify, signature_len}`;
2. `emissary-core/src/crypto/dsa.rs` — Knochen: verification only; any DSA
   signing ambition would touch this file (rejected, §4);
3. `emissary-core/src/primitives/destination.rs` — cert consts, `new`,
   `parse_frame`, `serialized_len`, length accessors;
4. `emissary-core/src/primitives/offline_signature.rs` — transient-key
   branches (currently 7/1 only);
5. `emissary-core/src/primitives/lease_set.rs` — `LeaseSet2::serialize`
   signing call site + `serialized_len` 64-byte slot;
6. `emissary-core/src/sam/parser.rs` — `DEST GENERATE` gate (+ `SESSION
   CREATE SIGNATURE_TYPE` handling);
7. `emissary-core/src/sam/pending/connection.rs` — `DEST GENERATE` handler
   and PRIV blob layout;
8. `emissary-core/src/config.rs` — fixed `signing_key: Option<[u8; 32]>`
   (migration required);
9. `emissary-core/src/destination/mod.rs`,
   `emissary-core/src/destination/session/mod.rs`,
   `emissary-core/src/destination/lease_set.rs` — suite-aware
   generation/import threading (consumers of already-signed blobs today).

I2PControl (Proposal mapping, M148 — not M147 — territory, listed for
boundary clarity, unchanged):

10. `emissary-cli/src/i2pcontrol/backends/options.rs` — M121 reject;
11. `emissary-cli/src/i2pcontrol/backends/runtime/session.rs` — wire mapping;
12. `emissary-cli/src/i2pcontrol/client_secret_store.rs`,
    `emissary-cli/src/i2pcontrol/server_secret_store.rs` — generation,
    envelope, type-blind validation;
13. `emissary-cli/src/i2pcontrol/domain/tunnel.rs`,
    `emissary-cli/src/i2pcontrol/tunnel_manager.rs` — `signature_type`
    plumbing/validation.

Explicitly prohibited adjacents (do not need modification; any future claim
otherwise requires a fresh plan amendment before coding):
`emissary-core/src/primitives/router_identity.rs`,
`emissary-core/src/primitives/router_info.rs` (router-identity signing
paths — destination work must never migrate router identity),
`emissary-core/src/router/mod.rs` (router key construction),
`emissary-core/src/crypto/{aes,chachapoly,hmac,noise,sha256,siphash}.rs`,
`emissary-core/src/transport/**` (incl. `ssu2/message ED25519_SIGNATURE_LEN`),
`emissary-core/src/netdb/**`, `emissary-core/src/i2np/**`,
`emissary-core/src/tunnel/**`, streaming owners (`sam/protocol/streaming/**`,
`sam/session.rs`, `sam/socket.rs`, `sam/mod.rs`), observation hooks
(`events.rs`, `inspection.rs`, `lib.rs`, `router/context.rs`), CLI
composition files, and the Yosemite fork (separate ADR-0005 authority).
No `crypto/**`, `netdb/**`, `i2np/**`, `destination/**`, or `primitives/**`
prefix allowance is created — M061/M062 are byte-identical before and after
M154 (see §9).

## 7. Verification outcomes

| Command group | Result |
|---|---|
| `cargo check -p emissary-core` | **pass** |
| `cargo check -p emissary-core --no-default-features --features no_std` | **pass** |
| `cargo test -p emissary-core --lib --no-fail-fast` | **pass**: `1080 passed, 2 ignored` |
| `cargo check -p emissary-cli --no-default-features --features i2pcontrol` | **pass** |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment --test m062_dependency_containment --test m095_full_support_matrix --test m105_residual_option_audit --test m153_post_m146_requalification --no-fail-fast` | **pass**: `37 passed (5 suites, 0 failed)` |
| M095 mechanical recomputation (`python3` TOML parse over `tunnel_manager.options[*].cells`) | **pass**: recomputed `840/336/29/475` == declared; residual split `10/5/5/5/4` |
| Production-head determination (`git log 7cbd80a..HEAD` over all production paths) | **pass**: empty — last production-bearing commit remains `7cbd80a...` |
| `git diff --check` | **pass** |

No new crypto dependency was added (`Cargo.toml`/`Cargo.lock` untouched).
`cargo clippy`/`cargo fmt --check` were not re-run as evidence: M154 touches
no Rust file, so the M153-recorded pre-existing drift findings stand without
re-measurement (recorded as info in §11 rather than re-claimed).

### Production-path diff (bounded)

`git status --short` over `emissary-core/src`, `emissary-cli/src`,
`emissary-util/src`, all manifests, and `Cargo.lock` is empty: M154 made
zero production changes. `110-completion-ledger.toml` gains no entry (zero
promotions). `061`/`062` unchanged. Streamr datagram limits (16-subscriber,
60s expiry, 1200-byte payload, 4095-byte transport buffer, 15s refresh,
bounded shutdown, loopback-only UDP) untouched; remote datagrams never
choose a local UDP destination.

## 8. Requirement-to-evidence matrix (planning process §2.5)

Covered in §7 (plan requirements) plus:

| Closure duty | Evidence |
|---|---|
| implementation commits | none — planning/audit-only milestone; the commit landing with this closure carries tests/plans/docs only (§9) |
| invariant review | partial-support, no-fabrication, I2P-only egress, fail-closed option validation, containment, and Y005 invariants re-proved by the green `m061`/`m062`/`m095`/`m105`/`m153` guards; no invariant weakened |
| failure/recovery and contention evidence | fail-before-allocation `SigType` rejection at three independent gates (`options.rs:219`, `session.rs:1145`, `client_secret_store.rs:539`, `parser.rs:867`) with no-echo errors; atomic secret-store publish paths unchanged and unexercised by this plan |
| compatibility, migration, security review | no wire/protocol/storage/dependency change ⇒ no migration impact; no new attack surface (no production code); all ten `SigType` cells stay fail-closed; DSA-SHA1 verification retained for interop, generation refused by policy (§2, §4) |
| documentation and operational evidence | §10 changed paths; authority docs name M154 closed / M147 path blocked; operational impact none |
| M147 disposition A/B/C | **C**, with no ambiguity (§1): A fails the one-primitive test, B cannot unblock M148 (policy-refused type 0, primitive-less type 11) |

## 9. Changed paths (exact budget only)

- `plans/implementation/i2pcontrol-proposal-170/154-m147-signature-domain-security-and-owner-refreeze.md`
  (Status `registered / dependency-ready` → `closed as complete` with closure link);
- `plans/implementation/i2pcontrol-proposal-170/147-neutral-destination-signature-suite-primitive.md`
  (Status `deferred / unregistered` → `closed as blocked` (path blocked by M154 disposition C) with closure link);
- new `plans/closure/i2pcontrol-proposal-170/154-closure.md` (this file);
- `plans/registry.md` (M154 → closed as complete with disposition C; M147 →
  closed as blocked (path); M148 stays deferred/unregistered behind the
  unsatisfiable M147 gate; M149–M152 stay deferred/unregistered with
  re-gating debt recorded; stale duplicated M154 prose repaired; execution
  chain, registration rules, lineage updated);
- `plans/implementation/i2pcontrol-proposal-170/README.md` (M154 closed
  section; M147 path blocked; chain updated);
- `plans/subsystems/i2pcontrol-proposal-170-post-m146-corrective-roadmap.md`
  (M154 closed; M147 path blocked; §§6–7/11–12 and graph updated);
- `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md`
  (M154 closed; disposition C; terminal-blocker semantics now include the ten
  `SigType` cells; registration discipline updated);
- `plans/subsystems/i2pcontrol-proposal-170-residual-primitive-completion-roadmap.md`
  (new §8 note records M154 closure as post-line authority; line history
  otherwise immutable);
- `AGENTS.md` (M154 closed entry with closure link and disposition C; M147
  path blocked; M148–M152 deferred);
- `docs/i2pcontrol/README.md`, `docs/i2pcontrol/proposal-170-support.md`,
  `docs/i2pcontrol/tunnel-manager.md` (M154 closed authority + `336/29/475`
  wording retained; `SigType` terminal-blocked statement).

`061-containment-boundary.toml`, `062-dependency-containment.toml`,
`095-full-support-matrix.toml`, `105-residual-option-audit.toml`, and
`110-completion-ledger.toml` are intentionally **unchanged** (zero promotions,
zero executable path/dependency authorisation). `git diff --check` passes.

## 10. Documentation and operations

Machine authorities: `095-full-support-matrix.toml` (`336/29/475`, residuals
identical to the M153 head), `110-completion-ledger.toml` (no new entry;
zero promotions), `061`/`062` (unchanged; candidate §6 paths are planning
metadata only and confer no executable authority), `m095` still expects
`336/29/475`, `m105` residual audit green, `m153` current-head guard green.
Support docs, `AGENTS.md`, registry, implementation README, and all three
roadmaps agree on `336/29/475`, partial support, M153 current qualification
authority, M146 blocked, M154 closed (disposition C), M147 path blocked, and
the 29-cell residual inventory. Operational impact: none; absent values
preserve prior behavior.

## 11. Unresolved findings

| Severity | Finding | Impact | Required action |
|---|---|---|---|
| low | `cargo fmt --all -- --check` / `cargo +nightly fmt` drift and the pre-existing `chunks_exact` clippy lint in untouched `backends/filters/proxy.rs:60` (M153 §11) were not re-measured — M154 touches no Rust file | none on behavior | record; future Rust-touching milestone re-measures |
| low | Remaining 29 blocked cells (10 SigType now terminal-blocked via M147-C, 15 LeaseSet crypto/lookup/auth deferred, 4 UseOutproxyPlugin terminal-blocked via M146) | partial Proposal 170 support remains | M148–M152 deferred chain; M152 must close at best safe-partial |
| low | Stale `062-dependency-containment.toml` header comment still names M153 as sole registered successor / M154 as deferred (pre-existing staleness, same as at M153 close) | planning-prose only; `m062` guards green (comments are not executable authority) | record; a future M062-touching milestone may refresh the comment |
| info | M147's template §2 gate ("M154 must have amended this plan with … exact M061/M062 production authorization") is satisfied by deliberate non-authorisation: the exact candidate list is frozen in §6 above and explicitly left non-executable under disposition C | no production effect; re-opening the path needs a new architecture/security decision and plan | recorded here and in the M147 plan header |
| info | M149–M152 gates reference M148 closure, which is now permanently unsatisfiable via the SigType chain; LeaseSet-security work for the Ed25519 suite may still be conceptually possible but needs explicit re-gating | planning debt, no code impact | a future planning action (not M154) must amend M149–M152 gates before any LeaseSet-security registration; M154 does not rewrite them |

No high/medium correctness defect remains.

## 12. Registry updates and future-plan unblock determination

Applied alongside this closure (see §9 for the file list):

- `154-*.md` plan: Status `registered / dependency-ready` → `closed as
  complete` with closure link;
- `147-*.md` plan: Status `deferred / unregistered` → `closed as blocked`
  (path blocked by M154 disposition C) with closure link; its template
  production/dependency budget is explicitly **not** authorised;
- `plans/registry.md`: M154 → closed as complete (`336/29/475` unchanged,
  zero promotions, zero production diff); M147 → closed as blocked (path);
  stale duplicated M154 prose repaired; execution chain, registration rules
  (no plan currently registered), and lineage updated;
- `110-completion-ledger.toml`: no new entry (zero promotions);
- `m061`/`m062`: unchanged (no new paths, no new dependencies).

Future-plan unblock determination (as required by the tasking):

- **M154 CLOSED as complete.** Its sole hard dependency (M153 closure) was
  satisfied, and all §12 acceptance criteria are met with zero production
  diff and zero matrix promotions.
- **M147 CLOSED as blocked (path).** Disposition C leaves the ten SigType
  cells blocked under current security/dependency policy. Re-opening
  requires a separate explicit architecture/security decision and plan (e.g.
  a maintained RedDSA primitive appearing, plus a policy decision covering
  legacy-suite generation and a versioned storage migration) — it cannot be
  hidden inside M148–M152.
- **M148 remains deferred/unregistered**, hard-gated on M147 closure-complete,
  which will not occur on this path. It is recorded as blocked-behind-M147,
  not as closed: its Proposal-mapping audit never executes.
- **M149–M151 remain deferred/unregistered.** Their LeaseSet-security targets
  are conceptually independent of non-Ed25519 suites, but their registered
  gates chain through M148 closure; they need explicit re-gating by a future
  planning action before any of them can register. M154 creates no such
  gate and authorises no LeaseSet production work.
- **M152 remains deferred/unregistered.** Its final qualification must now
  account for terminal `SigType` blockers (10) alongside M146's four: the
  best truthful terminal state of this workstream is **safe partial**, never
  full support, unless both terminal sets are resolved by separately
  accepted successors.
- **M146 remains closed as blocked** with no successor; it is not reopened
  by M154 or by any future LeaseSet tail.
- No other future-plan status required a change. File presence alone never
  authorises production work.

## Internal-only / read-only-upstream attestation

- External sources (the Proposal 170 text, the I2P common-structures
  specification, and go-i2p SAM `SIGNATURE_TYPE` constants cited above; the
  already-pinned Java I2PControl/I2PTunnel snapshots and Yosemite revision
  cited from existing records) were accessed read-only for evidence;
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
