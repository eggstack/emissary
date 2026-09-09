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

I2PControl (Proposal 170) tests:
```bash
cargo fmt --all -- --check
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo test -p emissary-cli --no-default-features --features i2pcontrol
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
```

I2PControl supports a large partial subset of Proposal 170. Keep Proposal/admin/application policy within `emissary-cli/src/i2pcontrol/` wherever possible; neutral lower-layer behavior belongs only in exact canonical owners with Proposal-free APIs. Unsupported capabilities must fail before allocation rather than being accepted inertly.

## Current Proposal-170 authority

- authoritative matrix: `336 apply / 29 blocked_primitive / 475 not_applicable`;
- M153 is the current whole-surface runtime/security qualification authority;
- M146 `UseOutproxyPlugin` remains closed blocked (4 cells);
- M154/M147/M148 configurable Destination `SigType` remains blocked (10 cells);
- M155 closed the corrected LeaseSet-security semantic/owner re-freeze;
- M156 closed the neutral Red25519/Ed25519 blinding primitive with zero Proposal promotions;
- **M157 is the sole registered/dependency-ready handoff**.

Current execution roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-post-m154-leaseset-security-corrective-roadmap.md`.

Registered plan:

- `plans/implementation/i2pcontrol-proposal-170/157-modern-encrypted-leaseset2-publication-primitive.md`.

M157 implements neutral modern type-5 Encrypted LeaseSet2 publication only. It has zero Proposal promotion budget.

## M157 exact production budget

Only these production paths may change:

1. `emissary-core/src/crypto/els2.rs` — new;
2. `emissary-core/src/crypto/mod.rs` — declaration/re-export only;
3. `emissary-core/src/primitives/lease_set.rs`;
4. `emissary-core/src/primitives/mod.rs` — exact type re-export only;
5. `emissary-core/src/i2np/database/store.rs`;
6. `emissary-core/src/netdb/mod.rs`;
7. `emissary-core/src/destination/lease_set.rs`;
8. `emissary-core/src/destination/mod.rs`;
9. `emissary-core/src/sam/parser.rs`;
10. `emissary-core/src/sam/session.rs`.

No other production file is authorized. No Cargo manifest, dependency, lockfile, Yosemite, or `emissary-cli/src/i2pcontrol/**` production change is authorized.

M061 contains a guarded `[registered_pending]` M157 ledger. The first M157 production commit must atomically move every newly changed path into M061's ordinary `[allowed]`/`[[evidence]]` realized-diff ledger. Existing paths already in `[allowed]` remain exact owners; do not add a broad prefix/glob.

M062 records the same path set and explicitly records zero dependency/manifest/lock/Yosemite/I2PControl-source change.

## M157 key implementation constraints

- Standard modern activation is `i2cp.leaseSetType=5`.
- Direct Proposal-170 PR source sets `i2cp.encryptLeaseSet=true` **only** for legacy `encrypted (aes)`; do not alias that legacy flag to modern type 5.
- M157 implements no client auth and no lookup secret. Type-5 requests containing successor-only secret/auth/PSK/DH companions must fail before activation.
- Reuse the ordinary signed inner LeaseSet2 produced by `SamSession`; do not create a second inner-LS builder.
- `LeaseSetManager` owns encrypted public/floodfill publication and UTC-day blinded-key rollover through its existing bounded state machine.
- `NetDb` must preserve type 3 vs type 5 when caching/flooding/answering. It currently always re-emits cached LeaseSets as type 3; M157 must fix exactly that owner rather than add a new NetDB subsystem.
- Keep `destination/session/mod.rs` unchanged. The ELS2 specification permits authenticated end-to-end clients to receive ordinary inner LS2 inside wrapped garlic while floodfill publication remains encrypted.
- Reuse M156 `crypto/red25519.rs` unchanged unless a demonstrated correctness defect forces a plan amendment.
- Reuse existing ChaCha20/HMAC/SHA256/RNG/zeroize; no new dependency.
- No plaintext type-3 fallback for a destination activated in type-5 mode.
- M157 must leave M095 exactly `336/29/475`.

If implementation needs any file/dependency outside the registered budget, stop before editing and amend M157/M061/M062.

## Deferred corrected line

```text
M155 semantic/owner refreeze            [CLOSED]
  -> M156 Red25519/blinding              [CLOSED]
  -> M157 modern Encrypted LS2           [REGISTERED]
  -> M158 lookup-secret/blinded address  [DEFERRED]
  -> M159 PSK auth                       [DEFERRED]
  -> M160 DH auth                        [DEFERRED]
  -> M161 legacy AES/LS1 feasibility     [DEFERRED]
  -> M162 Proposal field integration     [DEFERRED]
  -> M152 final requalification          [DEFERRED]
```

M149-M151 remain historical drafts but are superseded for execution by M155-M162. Do not execute them directly.

Do not reopen M147/M148 without a separate explicit architecture/security decision superseding M154. Do not create a dummy outproxy provider, alias `ProxyList`, or add direct-clearnet DNS/TCP egress to resolve M146.

No cryptographic suite fallback or plaintext/unsecreted/unauthenticated LeaseSet downgrade may be used to manufacture Proposal support. Yosemite remains the accepted exact optional I2PControl pin unless separately superseded.

Streamr is intentionally separate from TCP tunnel helpers. Preserve its documented 16-subscriber, 60-second expiry, 1200-byte payload, 4095-byte transport-buffer, 15-second refresh, and bounded shutdown limits. Remote datagrams must never choose a local UDP destination.

Fuzz targets (requires nightly):
```bash
cd emissary-core/fuzz && cargo fuzz run <target>
```
Available targets: `short_tunnel_build_builder`, `i2np_message_builder`, `tunnel_data_builder`, `i2np`, `primitives`, `messages`

## Formatting

`rustfmt.toml` enforces:
- `imports_granularity = "Crate"`
- `max_width = 100`, `comment_width = 100`
- `trailing_comma = "Vertical"`, `newline_style = "Unix"`

Always run `cargo fmt` before committing.

## Testing

- Uses `cargo-nextest` (config at `.config/nextest.toml`)
- Default profile: 5s slow-timeout, JUnit output to `junit.xml`
- Slow profile (`--profile tests-slow`): 1s period, 2 retries, no fail-fast

## Key quirks

- Crypto crates are pre-release (`ed25519-dalek 3.0.0-pre.6`, `ml-kem 0.3.0-rc.0`, etc.)
- Two async runtimes: tokio (default), smol (opt-in via `emissary-util`)
- Custom cargo profile `testnet` (release + debug=1 + assertions)
- Dioxus desktop UI requires system GTK3/WebKit libs
- `emissary-core` supports `no_std`
- `package.json` is docs-only
- I2PControl feature (`i2pcontrol`) is optional and disabled by default
