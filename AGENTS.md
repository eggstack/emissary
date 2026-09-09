# AGENTS.md — Emissary

Rust I2P router implementation. Cargo workspace with 3 crates + 2 examples.

## Workspace layout

- `emissary-core` — I2P protocol library (async, `no_std` optional)
- `emissary-util` — Runtime impls, reseeder, NAT-PMP/IGD, metrics, TLS backends
- `emissary-cli` — **Default build target.** CLI + optional Dioxus desktop/web UI + optional I2PControl API
- `examples/` — `rust-chat`, `rust-tutorial`

## Commands

```bash
cargo build
cargo build --release
cargo build -p emissary-core
cargo test
cargo test -p emissary-core
cargo fmt
cargo clippy
cargo run -- router-ui-dev
cargo run -- router-ui-dev --native
```

I2PControl qualification:

```bash
cargo fmt --all -- --check
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo test -p emissary-cli --no-default-features --features i2pcontrol
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
```

Keep Proposal/admin/application policy within `emissary-cli/src/i2pcontrol/` wherever possible. Neutral lower-layer behavior belongs only in exact canonical owners with Proposal-free APIs. Unsupported capabilities must fail before allocation/effect rather than be accepted inertly.

## Current Proposal-170 authority

- M095: `336 apply / 29 blocked_primitive / 475 not_applicable`;
- M153: current whole-surface runtime/security qualification authority;
- M146 `UseOutproxyPlugin` ×4: closed blocked;
- M154/M147/M148 configurable Destination `SigType` ×10: closed blocked;
- M155-M159 LeaseSet-security neutral lineage: closed, zero Proposal promotions;
- **no registered successor; M160 deferred with satisfied hard dep, registration pending**.

Roadmap: `plans/subsystems/i2pcontrol-proposal-170-post-m154-leaseset-security-corrective-roadmap.md`.

Closed plan: `plans/implementation/i2pcontrol-proposal-170/159-leaseset-psk-client-authorization-primitive.md`
(closure: `plans/closure/i2pcontrol-proposal-170/159-closure.md`).

## M159 exact production budget (realized)

Only these production files changed:

1. `emissary-core/src/crypto/els2.rs`
2. `emissary-core/src/destination/lease_set.rs`
3. `emissary-core/src/sam/parser.rs`
4. `emissary-core/src/sam/session.rs`

All four are existing exact M061 owners. No new M061 path waiver was created.

M159 authorized no:

- new source file;
- dependency;
- Cargo manifest/lockfile change;
- Yosemite change;
- I2PControl production-source change;
- `crypto/mod.rs`/`red25519.rs` change;
- `destination/mod.rs`/`destination/session/mod.rs` change;
- NetDB/I2NP/primitives/event/router/tunnel/transport change.

## M159 PSK contract (realized)

Standard SAM/I2CP inputs:

```text
i2cp.leaseSetType=5
i2cp.leaseSetAuthType=2
i2cp.leaseSetPrivKey=Base64(32B)
i2cp.leaseSetClient.psk.N=[Base64(UTF8(name)) ":"] Base64(32B)
```

The base `leaseSetPrivKey` is itself an authorized PSK. Indexed entries are additional; do not require them for non-per-user PSK modes. Duplicate entries are preserved exactly as configured (pinned Java semantics).

Parser requirements (realized):

- decode every key to exactly 32 bytes before activation;
- reject sparse indexed entries followed by later indices, mixed DH entries and unsupported auth selectors;
- strip optional `name:` prefixes but do not retain names in core;
- remove base/indexed PSK values from generic debug-capable options before `SamCommand`/session retention;
- carry key material only in zeroizing/non-`Debug` neutral types;
- M158 lookup secret may coexist and remains independently secret-required.

Crypto/wire requirements (realized):

- PSK layer-1 flags `0x03`;
- fresh 32-byte auth cookie and auth salt for each regenerated outer object;
- `ELS2PSKA` HKDF-SHA256 output length 52 = key32 + IV12 + clientID8;
- one client record = clientID8 + encrypted authCookie32;
- auth cookie participates in `ELS2_L2K`; it does not alter `ELS2_L1K` input;
- randomized client-record order for multiple keys;
- no-auth M157 and lookup-secret-only M158 behavior remain compatible.

### Work bound

Pinned Java `EncryptedLeaseSet.MAX_ENCRYPTED_SIZE=4096` is the authenticated encrypted-data ceiling. Checked complete-size calculation occurs before per-client crypto. Never silently drop/truncate clients to fit.

### Publication

`LeaseSetManager` remains the sole publication/UTC-rollover/storage-verification owner. Authenticated build failure never falls back to no-auth or type 3.

Extended B32 sets `auth_required=true`; `secret_required` continues to reflect the independent M158 lookup secret. The existing opaque event address seam remains unchanged.

Core persists no PSK material. Proposal-layer key generation/persistence/names/edit/restart/Get-redaction/five-family integration remain M162.

M159 promoted **zero Proposal cells**; M095 remains exactly `336/29/475`.

## Remaining line — already corrected

```text
M159 PSK authorization             [CLOSED]
  -> M160 DH/X25519 authorization  [DEFERRED; HARD DEP SATISFIED, REGISTRATION PENDING]
  -> M161 legacy AES/LS1 gate      [DEFERRED; HARD-DEPENDS M160; ZERO PRODUCTION]
  -> M162 Proposal field integration [DEFERRED]
  -> M152 final requalification    [DEFERRED]
```

M159 closure revalidates the exact four owners with the existing X25519 dependency sufficient, so M160 may be registered directly with exact M061/M062 authorization. M160 must use the same 4096-byte O(N) bound, preserve duplicates, and explicitly reject all-zero X25519 shared secrets.

M161 runs only after M160 and cannot implement legacy LS1 inside the gate. M162 is the first milestone allowed to promote LeaseSet fields and must implement typed/redacted I2PControl state plus transactional LeaseSet-security secret custody across all five server families.

Correct modern/legacy mappings for M162:

```text
legacy AES -> i2cp.encryptLeaseSet=true
modern -> i2cp.leaseSetType=5
OptionalLookup -> i2cp.leaseSetSecret=Base64(UTF8(value))
PSK -> authType=2 + base PSK + optional indexed PSK clients
DH  -> authType=1 + base X25519 private key + optional indexed DH clients
```

Do not execute superseded M149-M151. Do not reopen M147/M148 or M146 without separate accepted plans. No cryptographic, plaintext, secret, authentication, or direct-clearnet fallback may be used to manufacture Proposal support.

Streamr remains separate from TCP tunnel helpers. Preserve its documented 16-subscriber, 60-second expiry, 1200-byte payload, 4095-byte transport-buffer, 15-second refresh, and bounded shutdown limits. Remote datagrams must never choose a local UDP destination.

Fuzz targets (requires nightly):

```bash
cd emissary-core/fuzz && cargo fuzz run <target>
```

Available: `short_tunnel_build_builder`, `i2np_message_builder`, `tunnel_data_builder`, `i2np`, `primitives`, `messages`.

## Formatting/testing quirks

`rustfmt.toml`: `imports_granularity="Crate"`, `max_width=100`, `comment_width=100`, `trailing_comma="Vertical"`, Unix newlines. Always run `cargo fmt` before committing.

The repo uses cargo-nextest; `emissary-core` supports `no_std`; crypto dependencies include pre-release crates; Yosemite I2PControl remains an exact optional fork pin.