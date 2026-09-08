# M141 — HTTP Unique Local Source Address Completion

Status: **deferred / unregistered; hard-depends on M140 closure**

Class: capability / local-network confinement

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-residual-primitive-completion-roadmap.md`.

Target cells:

- `UniqueLocalAddressPerClient:httpserver`;
- `UniqueLocalAddressPerClient:httpbidirserver`.

Historical blocker authority:

- M131 `PB-LOCAL-SOURCE-ADDRESS-01`.

## 1. Objective

Implement the exact per-client local source-address behavior for the two HTTP accepted-server families while preserving the existing literal-loopback target confinement and keeping the implementation entirely inside the I2PControl-owned accepted-stream HTTP data plane.

The capability is complete only when an accepted I2P peer's already validated canonical Destination identity changes the **source address of the local loopback TCP connection** in the reference-compatible way. Merely persisting or returning the option is not support.

## 2. Hard dependency and readiness

Hard dependency:

- M140 closed with a reconciled current M095 baseline.

The plan becomes registration-ready only if current code still provides:

- structurally validated `TrustedPeerIdentity` before local target connection;
- a canonical 32-byte Destination hash through `TrustedPeerIdentity::canonical_id()`;
- `httpserver` and `httpbidirserver` sharing the existing HTTP accepted-handler path;
- literal-loopback-only target normalization for the affected server path.

If those seams materially changed after this plan was written, amend M141 before registration.

## 3. Pinned semantic contract

Pinned Java `I2PTunnelServer` behavior for a loopback target and the unique-local option:

- IPv4 loopback target: construct local source `127.<hash[0]>.<hash[1]>.<hash[2]>` from the remote I2P Destination hash and source-bind port 0 before connect;
- IPv6 loopback target: construct a unique-local IPv6 address beginning with `fd` and copy the reference-defined Destination-hash bytes, then source-bind port 0 before connect;
- non-loopback target: reference falls back to an ordinary connection, but M141 must **not** use this to broaden Emissary's stronger existing loopback-only target confinement;
- disabled option: preserve existing ordinary loopback connect behavior.

The remote identity used for derivation must be the canonical validated Destination hash, never untrusted SAM text, HTTP headers, a username, or a general-purpose rehash of attacker-controlled strings.

## 4. Containment and initial path budget

Expected production paths:

- `emissary-cli/src/i2pcontrol/backends/http_server.rs`;
- `emissary-cli/src/i2pcontrol/backends/http_bidir.rs` only if the composite must explicitly pass/validate the option;
- `emissary-cli/src/i2pcontrol/backends/options.rs`;
- `emissary-cli/src/i2pcontrol/backends/runtime/peer_identity_impl.rs` only if a small neutral helper exposure is genuinely required;
- `emissary-cli/src/i2pcontrol/tunnel_manager.rs` or `domain/tunnel.rs` only if typed extraction is required for exact validation/round-trip rather than safely consuming the canonical raw field.

Tests/docs/matrix paths as needed.

No `emissary-core/**`, Cargo/dependency, Yosemite, NetDB, transport, crypto, startup tunnel, frontend or `.github/**` production change is authorized.

If a platform/socket limitation requires a core or dependency change, stop and amend the plan rather than broadening scope during implementation.

## 5. Invariants

- Target host remains literal loopback under the existing `normalize_loopback_target` policy.
- The option cannot make an accepted I2P peer choose a destination address or port.
- Derivation uses canonical authenticated peer identity already established before the handler.
- No DNS lookup is added.
- No direct-clearnet path is added.
- Failed source bind/connect is bounded and closes only that accepted connection; it does not corrupt tunnel generation state.
- Source address state is per connection and not stored in an unbounded peer map.
- No peer Destination/hash enters logs or errors beyond existing sanitized identity policy.
- `false` is an explicit disabled value; it does not fail merely because `true` has special behavior.
- `httpbidirserver` reuses the same server-side implementation; it must not grow a second source-address algorithm.
- Unsupported families remain fail-before-allocation / N/A according to M095.

## 6. Required implementation shape

Prefer a small I2PControl-local helper that accepts:

- target `IpAddr` already proven loopback;
- canonical peer `[u8; 32]`;
- address family;

and returns the reference-compatible local source `IpAddr`.

Connection establishment should use `tokio::net::TcpSocket` (or an equivalently bounded existing primitive) so the local address can be bound before `connect`. Preserve the existing connect timeout.

Do not bind a listener. Do not allocate a persistent address registry. Do not probe arbitrary interfaces.

### IPv4

Prove byte-for-byte compatibility with Java's `127 + first three hash bytes` derivation, including `.0`, `.255`, repeated addresses, and collision behavior. The function is deterministic; collisions are allowed if the reference allows them.

### IPv6

Freeze the exact Java byte layout before coding. Do not assume an IPv6 ULA prefix or hash-byte count from memory. If the platform rejects binding the derived local address because it is not assigned, record the behavior and determine whether the Java/reference design relies on OS support not available to Emissary. Do not silently fall back to another source address while claiming support.

If exact IPv6 behavior cannot operate portably, closure must decide whether the Proposal option's supported runtime domain can truthfully remain IPv4-only. If the pinned contract requires both families, leave the cells blocked rather than partial-promote them.

## 7. Validation and persistence

Before any accepted-stream allocation/use:

- `UniqueLocalAddressPerClient` must be boolean when supplied;
- it is accepted only for the two M095 target families;
- malformed types fail at TunnelManager validation;
- edit/restart validation occurs before replacing a running last-known-good generation.

Keep get/round-trip spelling exact. If the existing raw-config path already provides exact typed boolean preservation, do not add duplicate domain fields solely for convenience.

## 8. Failure, cancellation, restart and contention

- Source-bind/connect inherits the existing local target connect timeout.
- Cancellation of the tunnel generation must cancel/drain accepted tasks under existing bounded task-group semantics.
- A failed bind/connect produces no retry loop unless the existing handler already has a bounded retry policy.
- Restart/edit failure preserves the control-plane's existing transactional/last-known-good semantics.
- No mutex/parking_lot guard may be held across socket bind/connect or accepted-stream relay.

## 9. Focused tests

Required deterministic tests:

1. IPv4 derivation from a fixed canonical Destination hash matches the pinned Java byte layout.
2. Different peer hashes produce reference-compatible deterministic addresses.
3. Same peer hash repeats the same address with no retained state.
4. Disabled option uses the ordinary local connect path.
5. Option true reaches both `httpserver` and composed `httpbidirserver` through one shared helper/path.
6. Non-target families reject supplied option before allocation/effect.
7. Malformed/noncanonical SAM remote Destination is rejected before derivation/connect.
8. HTTP headers cannot influence the derived source address.
9. Target remains literal loopback; hostile configured hostnames/addresses are rejected as before.
10. Bind/connect failure is bounded and does not mark a stale generation running.
11. If IPv6 is supported, exact IPv6 derivation and a platform-capable source-bind fixture are covered; if not, closure records the exact blocker and makes no partial support claim.
12. `Get`/persistence/restart round-trip preserves exact boolean semantics without leaking peer data.

Include a local integration fixture whose target listener records `peer_addr()` and proves the observable source address when the OS supports the derived bind.

## 10. Broad verification

```text
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo check -p emissary-cli --no-default-features
cargo test -p emissary-cli --no-default-features --features i2pcontrol --lib --no-fail-fast
cargo test -p emissary-cli --no-default-features --features i2pcontrol --no-fail-fast
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment --test m062_dependency_containment --test m095_full_support_matrix --test m105_residual_option_audit --no-fail-fast
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
cargo fmt --all -- --check
git diff --check
```

## 11. Matrix promotion budget

Maximum promotion budget: **2 cells**.

Promote each cell only if the exact observable source-bind effect is proven end-to-end for the required runtime domain. If a shared lower primitive lands but one family does not consume it, promote only the proven cell(s) and record the residual.

Counts must be calculated from the M140-closure baseline; this plan must not hard-code a guessed post-M140 matrix.

## 12. Acceptance criteria

M141 closes as complete only when:

- both target family gates are exact;
- enabled behavior source-binds the reference-derived address before local loopback connect;
- disabled behavior is unchanged;
- target confinement and trusted-peer boundary are unchanged or stricter;
- no persistent/unbounded peer-address state exists;
- deterministic and observable integration tests pass;
- actual changed paths stay within the authorized I2PControl-local budget;
- M061/M062 exact bookkeeping is updated for actual changed paths without broadening unrelated allowances;
- M095/M105/docs/registry counts match promoted cells;
- no high/medium correctness or local-network security issue remains.

## 13. Stop conditions

Stop and leave cells blocked if:

- exact Java derivation cannot be established;
- source binding requires granting arbitrary local-interface access or weakening loopback target confinement;
- peer identity would have to be inferred from untrusted application data;
- required IPv6 semantics cannot be implemented truthfully for the Proposal-required runtime domain;
- implementation requires core/router/transport changes outside the accepted path budget.

## 14. Closure evidence required

Record exact reference source locations, derivation test vectors, OS/runtime source-bind evidence, changed paths, pre/post M095 rows/counts, validation-before-allocation evidence, target-confinement review, cancellation/restart evidence, focused/broad verification, implementation SHA, unresolved platform limitations, and M142 readiness.

External reference access must be attested read-only; no upstream mutation/contact/submission is authorized.