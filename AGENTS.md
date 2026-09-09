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
- M157 closed the neutral modern type-5/no-auth Encrypted LeaseSet2 publication primitive with zero Proposal promotions;
- M158 closed the neutral standard lookup-secret/blinding and encrypted-service extended-B32 infrastructure with zero Proposal promotions;
- **no Proposal-170 successor is currently registered (M159 deferred with satisfied hard dep, amendment pending)**.

Current execution roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-post-m154-leaseset-security-corrective-roadmap.md`.

Closed plan:

- `plans/implementation/i2pcontrol-proposal-170/158-leaseset-lookup-secret-and-blinded-address-primitive.md`
  (closure: `plans/closure/i2pcontrol-proposal-170/158-closure.md`).

M158 had zero Proposal-promotion budget (observed; M095 remains `336/29/475`).

## M158 exact production budget (realized)

Only these production paths changed:

1. `emissary-core/src/crypto/els2.rs`;
2. `emissary-core/src/destination/lease_set.rs`;
3. `emissary-core/src/sam/parser.rs`;
4. `emissary-core/src/sam/session.rs`.

No new source file is authorized. All four are already individually present in M061's realized exact allowlist; M158 is a stricter active subset, not a new source-boundary waiver.

M062 records zero:

- new dependencies;
- Cargo manifest changes;
- lockfile changes;
- Yosemite changes;
- `emissary-cli/src/i2pcontrol/**` production changes.

If follow-up work needs any production path or dependency outside this budget, stop before editing and amend the owning milestone/M061/M062.

## M158 implementation constraints (realized)

### Standard lookup secret

Direct Java I2P/I2PTunnel source uses:

```text
i2cp.leaseSetSecret = Base64(UTF8(secret))
```

M158 consumes only this standard SAM/I2CP property. Proposal `OptionalLookup` plaintext mapping/persistence remains M162 work.

`sam/parser.rs` must:

- Base64-decode and UTF-8 validate the property before activation;
- remove it from the generic options map before `SamCommand` is constructed;
- move the decoded bytes into a dedicated zeroizing/non-`Debug` `LookupSecret` owned by `crypto/els2.rs` and carried inside redacted `DestinationContext`;
- continue rejecting type-5 auth/PSK/DH/legacy-AES/legacy-key companions that belong to later milestones.

Do not leave the source Base64 secret or decoded secret in generic `SamCommand`/`SamSession` option state: those types have debug-capable surfaces.

`destination/lease_set.rs` keeps the secret generation-locally inside `EncryptedPublicationConfig` and uses it on every current-day and rollover blinding derivation. There is no core secret persistence.

`crypto/red25519.rs` is already secret-capable and must remain unchanged unless a demonstrated correctness defect forces plan amendment.

### Extended encrypted-service B32

Implement in `crypto/els2.rs` only for the current type-7 -> type-11 one-byte-sigtype domain:

- exactly 35 decoded bytes / 56 I2P Base32 characters + `.b32.i2p`;
- flag bit 1 = secret required;
- flag bit 2 = client auth required;
- bits 7..3 zero; two-byte sigtype flag rejected;
- unblinded sigtype 7, blinded sigtype 11;
- 32-byte unblinded Ed25519 public key;
- IEEE CRC-32 over public-key bytes, XOR low three CRC bytes into header bytes 0..2 before Base32 encoding;
- strict length/suffix/checksum/flags/sigtype/public-key validation.

No CRC dependency: use a small exact local helper and pin against Java `net.i2p.crypto.Blinding` vectors.

M158 runtime emission sets `auth_required=false` but the codec may round-trip that public flag for M159/M160.

### Address publication

For published type-5 server destinations, `sam/session.rs` emits the canonical extended encrypted-service B32 through the existing opaque server-destination event string. Set `secret_required=true` iff the decoded secret is nonempty.

Ordinary/non-type5 destination event behavior must remain unchanged. `events.rs` is not in scope.

### Publication behavior

- absent/empty secret must preserve M157 blinded publication semantics;
- nonempty secret must feed the exact M156 daily alpha derivation;
- same destination/day/secret is deterministic;
- different secrets change blinded public/storage keys;
- UTC rollover retains the same generation-local secret;
- no failure may downgrade a secret-required generation to empty-secret or ordinary publication.

## Explicit M158 exclusions

Do not modify:

- `emissary-core/src/crypto/red25519.rs` or `crypto/mod.rs`;
- `destination/mod.rs`, `destination/session/mod.rs`, or `events.rs`;
- any `i2np/**`, `netdb/**`, `primitives/**`;
- router/tunnel/transport/frontend code;
- `emissary-cli/src/i2pcontrol/**` or `emissary-cli/src/tunnel/**`;
- Yosemite;
- Cargo manifests or `Cargo.lock`.

Do not add client-side blinded NetDB lookup/decryption, PSK/DH authorization, legacy AES/LS1, or configurable Destination `SigType` in M158.

M158 promotes **zero Proposal cells**. M095 must remain `336/29/475`. `OptionalLookup` remains blocked until M162 supplies the administrative mapping, persistent secret custody, redaction, edit/restart transactionality, and five-family integration.

## Corrected line

```text
M155 semantic/owner refreeze            [CLOSED]
  -> M156 Red25519/blinding              [CLOSED]
  -> M157 modern Encrypted LS2           [CLOSED]
  -> M158 lookup-secret/blinded address  [CLOSED]
  -> M159 PSK auth                       [DEFERRED; HARD DEP SATISFIED, AMENDMENT PENDING]
  -> M160 DH auth                        [DEFERRED]
  -> M161 legacy AES/LS1 feasibility     [DEFERRED]
  -> M162 Proposal field integration     [DEFERRED]
  -> M152 final requalification          [DEFERRED]
```

M149-M151 remain historical superseded drafts. Do not execute them directly.

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
