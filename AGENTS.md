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
- M154 closed disposition C and left the general M147/M148 configurable Destination `SigType` path blocked (10 cells);
- the M147 block does **not** imply modern Encrypted LeaseSet2 is blocked: the current corrective line separately evaluates the narrower type-7 Ed25519 -> type-11 Red25519 blinded-key primitive.

Current execution roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-post-m154-leaseset-security-corrective-roadmap.md`.

Sole registered handoff:

- **M155** `plans/implementation/i2pcontrol-proposal-170/155-post-m154-leaseset-security-semantic-and-owner-refreeze.md`.

M155 has zero production/dependency and zero Proposal-promotion budget. It must freeze the ten-value `EncryptLeaseSet` contract, the legacy AES/LS1 disposition, narrow Red25519/blinding formulas/dependency posture, Proposal field coupling, and exact future owners before M156 may be registered.

Deferred corrected line:

```text
M155 semantic/owner refreeze            [REGISTERED; ZERO PRODUCTION]
  -> M156 Red25519/blinding              [DEFERRED]
  -> M157 modern Encrypted LS2           [DEFERRED]
  -> M158 lookup-secret/blinded address  [DEFERRED]
  -> M159 PSK auth                       [DEFERRED]
  -> M160 DH auth                        [DEFERRED]
  -> M161 legacy AES/LS1 feasibility     [DEFERRED]
  -> M162 Proposal field integration     [DEFERRED]
  -> M152 final requalification          [DEFERRED]
```

M149-M151 remain historical drafts but are superseded for execution by M155-M162. Do not execute them directly.

Do not reopen M147/M148 without a separate explicit architecture/security decision superseding M154. Do not create a dummy outproxy provider, alias `ProxyList`, or add direct-clearnet DNS/TCP egress to resolve M146.

No draft candidate path is production authority. Broad `crypto/**`, `netdb/**`, `i2np/**`, `destination/**`, `primitives/**` or transport waivers are prohibited. Exact M061/M062 file authorization must be added only when the next milestone is explicitly registered.

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
