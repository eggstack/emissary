# M146 — Outproxy Provider / UseOutproxyPlugin Completion

Status: **deferred / unregistered; hard-depends on M145 closure**

Class: infrastructure + capability / proxy routing

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-residual-primitive-completion-roadmap.md`.

Target cells:

- `UseOutproxyPlugin:httpclient`;
- `UseOutproxyPlugin:socks`;
- `UseOutproxyPlugin:socksirc`;
- `UseOutproxyPlugin:connectclient`.

Historical authority:

- M131 `PB-OUTPROXY-PLUGIN-01`.

## 1. Objective

Implement Proposal `UseOutproxyPlugin` only if Emissary has a real, bounded local outproxy-provider abstraction with at least one usable provider whose traffic remains routed through I2P. A registry, parser flag, callback interface, or empty provider list alone is infrastructure and has zero support value.

Pinned Java behavior uses the application `ClientAppManager`/`Outproxy` provider boundary (property `i2ptunnel.useLocalOutproxy`) as an alternative to configured ordinary outproxy destinations. M146 must implement the corresponding capability without introducing a direct-clearnet escape path.

## 2. Hard dependency and readiness

Hard dependency:

- M145 closed and current matrix/containment authority reconciled.

Before registration, perform a current-code inventory and answer:

1. Is there already an in-process service/provider that can act as a local outproxy while preserving I2P egress?
2. Which request classes/protocols are covered for HTTP, CONNECT, SOCKS and SOCKS-IRC?
3. Can the provider be invoked through an I2PControl-local trait/registry without modifying router core?
4. What does Java do when the option is true but no provider is registered?
5. How are provider failures/fallback to configured I2P outproxies ordered?

If there is no real provider and adding one would require a general clearnet networking subsystem, M146 must remain blocked. Do not create a dummy provider to promote cells.

## 3. Ownership and initial path budget

Preferred production ownership is entirely I2PControl-local:

- new or existing bounded provider registry under `emissary-cli/src/i2pcontrol/backends/runtime/`;
- `http_client.rs`, `connect_client.rs`, `socks.rs`, `socks_irc.rs` only as consumers;
- shared proxy filtering/routing helpers under existing `backends/filters/` where appropriate;
- `backends/options.rs` and domain/tunnel parsing only for exact option validation/round-trip.

No `emissary-core/**`, Cargo/dependency, Yosemite, legacy startup proxy, frontend, NetDB, transport or `.github/**` production change is pre-authorized.

If a genuine provider needs a non-I2PControl owner, amend the plan before registration and justify why that owner is neutral and independently useful. A direct OS TCP/DNS provider is prohibited for this workstream.

## 4. Provider contract

A provider interface must be small, bounded and protocol-explicit. It should expose only what the pinned tunnel family needs, such as opening/handling an outproxy request over an already accepted safe routing boundary.

Required properties:

- provider names/IDs are bounded and not attacker-created at unbounded cardinality;
- registration occurs at composition/startup, not per request;
- provider lookup is deterministic and lock-free across network I/O;
- provider calls have explicit timeout/cancellation;
- provider cannot return an arbitrary unvalidated direct-clearnet socket to bypass I2P policy;
- protocol capabilities are explicit so SOCKS/HTTP/CONNECT behavior is not guessed;
- failures are sanitized and cannot leak credentials/destinations;
- provider state is bounded and shutdown-aware.

At least one production provider must satisfy this contract before any Proposal cell promotes.

## 5. Reference fallback semantics

Freeze from pinned Java before coding:

- default of `UseOutproxyPlugin`;
- selection order between local plugin/provider and configured I2P outproxy list;
- behavior when provider is absent, declines a request, or fails after selection;
- whether HTTP, HTTPS/CONNECT and SOCKS use the same provider API;
- whether SOCKS-IRC inherits SOCKS provider behavior;
- credential handling and whether provider-specific credentials exist.

Fallback, if any, may only reach another accepted I2P-routed outproxy mechanism. Never fall back to public DNS/direct TCP.

## 6. Validation, persistence and sharing

- option must be boolean and family-gated before allocation;
- `false` preserves existing configured-outproxy behavior;
- `true` with no required provider follows the pinned deterministic error/fallback behavior;
- edit/restart is transactional;
- shared client sessions remain compatible only where provider policy does not conflict;
- provider identity/config is not serialized into Proposal responses unless the Proposal explicitly defines it.

## 7. Failure, cancellation, restart and contention

- no registry lock spans provider work;
- request/provider timeouts are bounded;
- cancellation of a tunnel generation cancels provider work owned by that generation;
- stale provider completions cannot write successor state;
- provider failure cannot trigger an unbounded retry/fallback cycle;
- registry capacity is fixed/bounded and duplicate registration semantics are deterministic;
- shutdown drains only bounded tasks.

## 8. Work packages

### WP1 — provider feasibility/reference freeze

Identify an actual production provider path and exact Java selection/fallback semantics. If none exists within safe scope, stop blocked before adding infrastructure.

### WP2 — bounded provider interface/registry

Implement the minimum I2PControl-local abstraction and static/bounded registration. Add infrastructure tests but do not alter M095 yet.

### WP3 — real provider integration

Register at least one real I2P-routed provider and prove it can serve the reference request classes without direct clearnet egress.

### WP4 — four family consumers

Map `UseOutproxyPlugin` into HTTP/CONNECT/SOCKS/SOCKS-IRC, reusing common selection behavior and preserving protocol-specific filters.

### WP5 — adversarial/fallback tests

Exercise no-provider, decline, failure, timeout, cancellation, duplicate/provider-capacity and no-clearnet-fallback cases.

### WP6 — matrix/containment/closure

Promote only families with a real provider effect and reconcile M061/M062/M095/M105/docs/registry.

## 9. Focused tests

Required:

- false -> existing outproxy path unchanged;
- true + provider -> provider path observably selected;
- true + no provider -> exact reference-compatible error/fallback;
- provider decline/failure never opens direct clearnet connection;
- configured I2P fallback, if reference permits, stays I2P-only;
- provider registry bounded and duplicate behavior deterministic;
- no lock across provider call;
- timeout/cancellation terminate work;
- HTTP/CONNECT/SOCKS protocol family selection exact;
- SOCKS-IRC inherits only the safe SOCKS path and retains IRC filtering;
- credentials remain redacted;
- restart drops stale generation/provider work;
- provider-only infrastructure with no real provider does not change matrix.

## 10. Matrix promotion budget

Maximum: **4 cells**, independently proven from the M145 closure baseline.

No cell promotes unless at least one real provider services that family's request path with exact behavior.

## 11. Verification

```text
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo check -p emissary-cli --no-default-features
cargo check
cargo test -p emissary-cli --no-default-features --features i2pcontrol --lib --no-fail-fast
cargo test -p emissary-cli --no-default-features --features i2pcontrol --no-fail-fast
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test i2pcontrol_live_runtime -- --nocapture
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment --test m062_dependency_containment --test m095_full_support_matrix --test m105_residual_option_audit --no-fail-fast
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
cargo fmt --all -- --check
git diff --check
```

## 12. Acceptance criteria

M146 closes complete only when:

- pinned provider selection/fallback semantics are frozen;
- a real bounded production provider exists;
- all promoted family paths observably use it;
- no direct-clearnet fallback/DNS path exists;
- registry/task/state bounds and cancellation are proven;
- production changes remain I2PControl-local unless explicitly amended;
- machine matrix/docs/registry and containment evidence match runtime behavior;
- no medium/high proxy-routing/security defect remains.

## 13. Stop conditions

Stop and leave cells blocked if:

- no real safe provider exists within bounded scope;
- implementation would require general direct-clearnet networking;
- a registry-only/dummy provider is the only feasible result;
- Java selection/fallback semantics cannot be established;
- protocol-specific filters would be bypassed by the provider boundary.

## 14. Closure evidence required

Record provider/reference table, real provider identity and routing proof, no-clearnet tests, registry bounds, timeout/cancellation/fallback evidence, changed paths, matrix deltas, containment updates, verification, implementation SHA, unresolved findings and M147 readiness. External access remains read-only.