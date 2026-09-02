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

I2PControl supports the Proposal 170 contract. Its I2PControl-owned runtime primitives and
specialized backends provide bounded local-listener, accepted-stream, and Streamr datagram
lifecycle ownership plus fail-before-allocation option validation. Tunnel data-plane backends
and inspection sources without a canonical Emissary owner remain explicit unsupported/unavailable
responses. Keep changes within `emissary-cli/src/i2pcontrol/` and its composition seams; do not
turn the administrative API into a router lifecycle or protocol implementation.

The current Proposal 170 baseline remains partial: RouterInfo is 43 additions with
42 available, 1 protocol-permitted neutral, and 0 unavailable; AddressBook all 13
SetConfig keys are operational; and unapplied runtime options (including 4 UseSSL,
45 client proxy/reduction, and 21 server LeaseSet/presentation cells documented in
M111-M113 closures) fail before allocation. The authoritative completion inventory is
`plans/implementation/i2pcontrol-proposal-170/095-full-support-matrix.toml` (312
apply / 70 blocked_primitive / 458 not_applicable after M113).

Streamr is intentionally separate from TCP tunnel helpers. Preserve its documented
16-subscriber, 60-second expiry, 1200-byte payload, 4095-byte transport-buffer, 15-second refresh,
and bounded shutdown limits. Remote datagrams must never choose a local UDP destination.

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
