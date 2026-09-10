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

Keep Proposal/admin/application policy within `emissary-cli/src/i2pcontrol/` wherever possible. Neutral lower-layer behavior belongs only in exact canonical owners with Proposal-free APIs. Unsupported capabilities must fail before allocation/effect **and before durable state mutation** rather than be accepted inertly. Rejected secret-bearing protocol payloads must not be logged verbatim.

## Current Proposal-170 authority

- M095: `336 apply / 29 blocked_primitive / 475 not_applicable`;
- Full Proposal-170 status remains **partial**; no full-support claim.
- M139: historical whole-surface qualification (superseded by M153 for current-head purposes);
- M141 `UniqueLocalAddressPerClient` ×2: closed complete (HTTP-server per-client loopback source);
- M142 `SSLProxies`/`JumpList` ×2: closed complete (HTTP-client bounded I2P-only selection/address-helper);
- M143 `Profile:client`: closed complete (retained neutral streaming max-window mapping);
- M153: whole-surface runtime/security qualification ancestry at `336/29/475`;
- M146 `UseOutproxyPlugin` ×4: closed blocked;
- M154/M147/M148 configurable Destination `SigType` ×10: closed blocked;
- M155-M159 LeaseSet-security neutral lineage: closed, zero Proposal promotions;
- M160 DH/X25519 neutral lineage: closed, zero Proposal promotions;
- M161 legacy AES/LS1 gate: closed outcome B (valid but blocked), zero production/promotions;
- M162 Proposal LeaseSet-security blocked integration: closed, zero promotions; all 15 LeaseSet cells remain blocked;
- M152: closed historical safe-partial qualification on the M162 head; post-closure review found blocked-state persistence and malformed-SAM logging defects, so it is not the current terminal handoff authority;
- **M163 is the sole registered/dependency-ready handoff**.

Current roadmap: `plans/subsystems/i2pcontrol-proposal-170-post-m152-blocked-state-corrective-roadmap.md`.

Registered plan: `plans/implementation/i2pcontrol-proposal-170/163-blocked-leaseset-state-persistence-corrective.md`.

Deferred successors:

- M164 `164-sam-invalid-command-secret-redaction-corrective.md` — exact neutral `emissary-core/src/sam/socket.rs` hardening after M163;
- M165 `165-post-corrective-current-head-requalification.md` — zero-production requalification after M163+M164.

## M163 exact production budget

Only these production files may change:

1. `emissary-cli/src/i2pcontrol/tunnel_manager.rs`
2. `emissary-cli/src/i2pcontrol/stores/tunnel_store.rs`
3. `emissary-cli/src/i2pcontrol/stores/generation_store.rs`

All three are inside the existing I2PControl policy root. No new non-I2PControl M061 path waiver is created.

M163 authorizes no:

- `emissary-core/**` or `emissary-util/**` production change;
- dependency/Cargo manifest/lockfile change;
- Yosemite change;
- `server_secret_store.rs` change;
- backend/session-runtime change;
- M159/M160 crypto/parser/publication change;
- Proposal promotion/demotion.

If a fourth production path or dependency is required, stop before editing and amend M163/M061/M062.

## M163 corrective contract

M162 typed/redacted LeaseSet-security inputs but allowed blocked `EncryptLeaseSet`, `OptionalLookup`, and `LeaseSetClientAuths` values to be serialized in `TunnelDefinition` and committed before Start later rejected them. M163 must reject those fields on Create/Edit before `TunnelStore` mutation while preserving M162 backend/session rejection as defense in depth.

Historical M162-era state also requires cleanup. `GenerationStore` retains prior generations, so a single sanitized publish is insufficient. M163 must use a crash-resumable three-state scrub (`legacy/unstarted -> sanitized_pending_history_purge -> scrub_complete`):

1. clear all three blocked fields from all affected definitions;
2. publish at least two newest clean fallback generations;
3. use a narrow fail-closed generation-history purge to delete all older contaminated generations and sync the directory;
4. publish scrub-complete only after successful purge;
5. on interruption/failure, fail load before StartOnLoad and resume the pending scrub next startup.

Do not change ordinary `GenerationStore::cleanup()` retention semantics globally. The secure purge is an explicit narrow operation used only by the tunnel-store migration.

M163 promotes **zero Proposal cells**; M095 remains `336/29/475`.

## M164 planned neutral SAM hardening

After M163 closes, M164 may be registered on exactly:

- `emissary-core/src/sam/socket.rs`

Current invalid-command handling logs the complete `%command` string after parser rejection. M164 must treat rejected payload content as sensitive and remove it entirely from tracing, retaining only safe structural metadata such as observation id, peer and byte length. Do not implement ad-hoc key-name redaction; do not change parser/session/connection semantics. No dependencies or Proposal promotions.

## M165 planned current-head requalification

After clean M163+M164 closures, M165 may be registered as zero-production/zero-promotion qualification. It must refresh M095 `current_production_head` to the actual last production-bearing M164 closure commit, mechanically re-evaluate `336/29/475`, and re-run whole-surface behavioral/security/containment evidence. It may become the new safe-partial current-head authority only if no high/medium defect remains.

## M160 exact production budget (realized)

Only these production files changed:

1. `emissary-core/src/crypto/els2.rs`
2. `emissary-core/src/destination/lease_set.rs`
3. `emissary-core/src/sam/parser.rs`
4. `emissary-core/src/sam/session.rs`

All four are existing exact M061 owners. No new M061 path waiver was created.

M160 authorized no:

- new source file;
- dependency;
- Cargo manifest/lockfile change;
- Yosemite change;
- I2PControl production-source change;
- `crypto/mod.rs`/`red25519.rs` change;
- `destination/mod.rs`/`destination/session/mod.rs` change;
- NetDB/I2NP/primitives/event/router/tunnel/transport change.

If a fifth production path or dependency was required, the milestone would have stopped before editing to amend M160/M061/M062. None was required.

## M160 DH contract (realized)

Standard SAM/I2CP inputs:

```text
i2cp.leaseSetType=5
i2cp.leaseSetAuthType=1
i2cp.leaseSetPrivKey=Base64(32B X25519 private)
i2cp.leaseSetClient.dh.N=[Base64(UTF8(name)) ":"] Base64(32B X25519 public)
```

The public key derived from the base private key is always in the authorized set. Indexed entries are additional; do not require them for non-per-user DH modes. Duplicate entries are preserved exactly as configured (pinned Java semantics). DH and PSK modes are mutually exclusive.

Parser requirements:

- decode every key to exactly 32 bytes before activation;
- reject sparse indexed entries followed by later indices, mixed PSK entries and unsupported auth selectors;
- strip optional `name:` prefixes but do not retain names in core;
- remove base/indexed DH values from generic debug-capable options before `SamCommand`/session retention;
- carry key material only in zeroizing/non-`Debug` neutral types;
- M158 lookup secret may coexist and remains independently secret-required.

Crypto/wire requirements:

- DH layer-1 flags `0x01`;
- a fresh ephemeral X25519 keypair and fresh 32-byte auth cookie for each regenerated outer object;
- `ELS2_XCA` HKDF-SHA256 output length 52 = key32 + IV12 + clientID8, salted by the ephemeral public key over `shared || cpk || subcredential || published_BE`;
- explicit all-zero shared-secret rejection at the ELS2 boundary;
- one client record = clientID8 + encrypted authCookie32;
- auth cookie participates in `ELS2_L2K`; it does not alter `ELS2_L1K` input;
- randomized client-record order for multiple keys;
- no-auth M157, lookup-secret-only M158, and PSK M159 behavior must remain compatible.

### Work bound

Use pinned Java `EncryptedLeaseSet.MAX_ENCRYPTED_SIZE=4096` as the authenticated encrypted-data ceiling. Checked complete-size calculation must occur before any per-client X25519 work. Never silently drop/truncate clients to fit.

### Publication

`LeaseSetManager` remains the sole publication/UTC-rollover/storage-verification owner. Do not add another scheduler or publication state machine. Authenticated build failure must never fall back to no-auth, PSK, or type 3.

Extended B32 sets `auth_required=true`; `secret_required` continues to reflect the independent M158 lookup secret. The existing opaque event address seam remains unchanged.

Core persists no DH material. Proposal-layer key generation/persistence/names/edit/restart/Get-redaction/five-family integration remained M162 and is still blocked at the Proposal layer.

M160 promoted **zero Proposal cells**; M095 remains exactly `336/29/475`.

## Remaining line — corrected

```text
M159 PSK authorization                       [CLOSED]
  -> M160 DH/X25519 authorization            [CLOSED]
  -> M161 legacy AES/LS1 gate                [CLOSED; OUTCOME B]
  -> M162 Proposal field integration         [CLOSED; BLOCKED INTEGRATION]
  -> M152 historical requalification         [CLOSED; SAFE PARTIAL]
  -> M163 blocked-state persistence/history  [REGISTERED]
  -> M164 SAM invalid-command redaction      [DEFERRED]
  -> M165 current-head requalification       [DEFERRED]
```

Correct modern/legacy mappings retained from M162:

```text
legacy AES -> i2cp.encryptLeaseSet=true
modern -> i2cp.leaseSetType=5
OptionalLookup -> i2cp.leaseSetSecret=Base64(UTF8(value))
PSK -> authType=2 + base PSK + optional indexed PSK clients
DH  -> authType=1 + base X25519 private key + optional indexed DH clients
```

Do not execute superseded M149-M151. Do not reopen M147/M148 or M146 without separate accepted plans. No cryptographic, plaintext, secret, authentication, durable-inert, diagnostic, or direct-clearnet fallback may be used to manufacture Proposal support.

Streamr remains separate from TCP tunnel helpers. Preserve its documented 16-subscriber, 60-second expiry, 1200-byte payload, 4095-byte transport-buffer, 15-second refresh, and bounded shutdown limits. Remote datagrams must never choose a local UDP destination.

Fuzz targets (requires nightly):

```bash
cd emissary-core/fuzz && cargo fuzz run <target>
```

Available: `short_tunnel_build_builder`, `i2np_message_builder`, `tunnel_data_builder`, `i2np`, `primitives`, `messages`.

## Formatting/testing quirks

`rustfmt.toml`: `imports_granularity="Crate"`, `max_width=100`, `comment_width=100`, `trailing_comma="Vertical"`, Unix newlines. Always run `cargo fmt` before committing.

The repo uses cargo-nextest; `emissary-core` supports `no_std`; crypto dependencies include pre-release crates; Yosemite I2PControl remains an exact optional fork pin.
