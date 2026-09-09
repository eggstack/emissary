# AGENTS.md — Emissary

Rust I2P router implementation. Cargo workspace with 3 crates + 2 examples.

## Workspace layout

- `emissary-core` — I2P protocol library (async, `no_std` optional)
- `emissary-util` — Runtime impls, reseeder, NAT-PMP/IGD, metrics, TLS backends
- `emissary-cli` — **Default build target.** CLI + optional Dioxus desktop/web UI + optional I2PControl API
- `examples/` — `rust-chat`, `rust-tutorial`

## Commands

```bash
cargo build                    # builds emissary-cli (default member)
cargo build --release
cargo build -p emissary-core   # core only
cargo test                     # all workspace tests (uses cargo-nextest)
cargo test -p emissary-core    # core tests only
cargo fmt
cargo clippy
cargo run -- router-ui-dev     # dev UI, no network
cargo run -- router-ui-dev --native  # native desktop UI
```

I2PControl (Proposal 170) tests:
```bash
cargo fmt --all -- --check
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo test -p emissary-cli --no-default-features --features i2pcontrol
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
```

I2PControl supports a large partial subset of Proposal 170. Its I2PControl-owned runtime primitives and specialized backends provide bounded local-listener, accepted-stream, and Streamr datagram lifecycle ownership plus fail-before-allocation option validation. Tunnel data-plane backends and options without a canonical Emissary owner remain explicit unsupported/unavailable responses. Keep Proposal/admin/application policy within `emissary-cli/src/i2pcontrol/` wherever possible; do not turn the administrative API into a router lifecycle or protocol implementation.

Current Proposal-170 planning authority:

- M139 closed as the historical whole-surface qualification at `325/47/468`; superseded by M153 for current-head purposes after M141-M145 production changes;
- M140 closed as the residual streaming applicability re-freeze;
- M141 closed with 2 `UniqueLocalAddressPerClient` promotions;
- M142 closed with 2 HTTP `SSLProxies`/`JumpList` promotions;
- M143 closed with 1 retained `Profile:client` promotion through neutral streaming owners;
- M144 closed with 4 application `UseSSL` promotions;
- M145 closed with 2 `MultiHoming`/reply-LeaseSet-bundling promotions; accepted production evidence includes no-std/format follow-up `7cbd80a6d72aa07d158ba9dc74f8bbacef767be2`;
- M146 closed as blocked with zero promotions: four `UseOutproxyPlugin` cells remain blocked because no real safe provider exists in current architecture;
- **M153 closed as the current runtime/security qualification authority** (`336/29/475`, zero promotions, zero production changes): `plans/closure/i2pcontrol-proposal-170/153-closure.md`;
- M154 is registered as the mandatory pre-registration SigType/security/exact-owner audit before M147;
- M147-M152 remain deferred/unregistered.

Current execution roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-post-m146-corrective-roadmap.md`.

The authoritative matrix is `plans/implementation/i2pcontrol-proposal-170/095-full-support-matrix.toml` at `336 apply / 29 blocked_primitive / 475 not_applicable` after M145, unchanged by blocked M146. Remaining blockers are exactly 10 `SigType`, 15 LeaseSet-security (`EncryptLeaseSet`/`OptionalLookup`/`LeaseSetClientAuths`), and 4 `UseOutproxyPlugin`.

M153 has zero Proposal-promotion and zero production-code/dependency budget. It exists to restore a truthful current-head integrated qualification, reconcile stale historical aggregate test assertions, correct M095 production-head metadata, and requalify M140-M146 plus earlier security/lifecycle invariants. Any production fix required during M153 is a stop condition requiring a separate corrective plan.

Do not register M147 directly after M153. M154 must first freeze the exact signature-type domain, algorithm security disposition, current generate/sign/verify/serialize/persist capability, persistence/migration implications, maintained dependencies, and exact-file M061/M062 ownership. Broad `crypto/`, `netdb/`, `i2np/`, `destination/` or `primitives/` waivers are prohibited.

M146 remains terminal blocked under the current security architecture. Do not create a dummy provider, alias `ProxyList`, or add direct-clearnet DNS/TCP egress merely to promote `UseOutproxyPlugin`. A future provider successor requires a separate explicit architecture/security decision and plan.

Streamr is intentionally separate from TCP tunnel helpers. Preserve its documented 16-subscriber, 60-second expiry, 1200-byte payload, 4095-byte transport-buffer, 15-second refresh, and bounded shutdown limits. Remote datagrams must never choose a local UDP destination.

Fuzz targets (requires nightly):
```bash
cd emissary-core/fuzz && cargo fuzz run <target>
```
Available targets: `short_tunnel_build_builder`, `i2np_message_builder`, `tunnel_data_builder`, `i2np`, `primitives`, `messages`

## Formatting

`rustfmt.toml` enforces:
- `imports_granularity = "Crate"` (grouped imports)
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
- Custom cargo profile `testnet` (release + debug=1 + assertions) — not standard
- Dioxus desktop UI requires system GTK3/WebKit libs (see `Dockerfile` for full list)
- `emissary-core` supports `no_std` (uses `spin` instead of `parking_lot`)
- `package.json` is only for vitepress docs, not the Rust project
- I2PControl feature (`i2pcontrol`) is optional, disabled by default; activates axum, TLS, JSON-RPC
