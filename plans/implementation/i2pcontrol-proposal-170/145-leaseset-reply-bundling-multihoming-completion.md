# M145 — LeaseSet Reply Bundling / MultiHoming Completion

Status: **closed as complete** (closure `plans/closure/i2pcontrol-proposal-170/145-closure.md`; registered for execution after M144 closure via `M145-AMEND-01` with the pinned session/bundling freeze and exact-file budget recorded here)

Registration amendment (M145-AMEND-01, 2026-09-08, pre-implementation):

- Retained target set (verified, not assumed): exactly `MultiHoming:httpserver` × 1 and
  `MultiHoming:httpbidirserver` × 1 (the two M095 `blocked_primitive` cells; all ten other
  families are `not_applicable` by HTTP-server-presentation-role evidence and must never
  inherit the new seam).
- Matrix baseline: M144 closure `334/31/475` (SHA
  `af01bc0efb4723f471e442e4cdf5dd8e70f6a9473ac5fe0be022ae9cadfa34c9`).
  Maximum promotion budget for M145 is exactly 2 cells.
- Pinned value/semantic freeze (read-only reference, verified 2026-09-08):
  Java `OutboundClientMessageOneShotJob.java` (pinned I2P/I2PTunnel snapshot
  `2c3fd2a9532cd86ec06cb6f2b9f3f813ca752243`):
  `BUNDLE_REPLY_LEASESET = "shouldBundleReplyInfo"`, defaults to true
  (`allow == null || Boolean.parseBoolean(allow)`; absent means bundle).
  Proposal `MultiHoming` maps directly: `true` → bundle, `false` → suppress,
  omitted → bundle default (no wire option, preserves current always-bundle behavior).
  Explicit `"true"` emits no wire option (same effective as omitted, distinct persisted
  string for round-trip, per M143 bulk/omitted precedent; omitted and `true` share one
  session). `false` emits wire `"false"` (suppress). All other types/values fail
  before allocation with no echo. Per-message `SendMessageOptions.getSendLeaseSet`
  has no Emissary SAM equivalent layer, so the session policy alone controls (documented,
  no invented per-message flag). Throttle is ack-gated: Java `getReplyLeaseSet(false)`
  sends only when not already acked (cache date check); Emissary equivalent is the existing
  `ActiveSession.lease_set` pending-until-ack plus 5s publish timer (bounded per-session,
  one pending `Option`, no unbounded history). Message types: garlic `DatabaseStore`
  `LeaseSet2` clove in `NewSession` (mandatory handshake) and `ExistingSession` updates.
  Emissary `SessionManager::decrypt` rejects `NewSession` without `DatabaseStore`
  ("cannot reply"), so `NewSession` retains mandatory bundling for liveness (bounded
  fail-closed adaptation, documented here); the new policy gates only `ExistingSession`
  update bundling (periodic reply-info path). No valid current LeaseSet → send without
  fabrication (punt). Expired/expiring handled by existing `LeaseSetManager` (only current
  valid LS bundled). Unpublished destinations still bundle directly to the peer (not via
  NetDB) for reply possibility, never published contrary to policy. Encrypted/blinded
  LeaseSets not yet implemented (M149-M151 deferred) → no downgrade (current LS2 only).
  No new NetDB lookup to fabricate sender info. Bounded message growth checked via existing
  garlic builder (no oversized construction/send).
- Exact-file core path budget (smallest, no broader allowance):
  - neutral policy owner: `emissary-core/src/destination/session/mod.rs`
    (`SessionManager` generation-local immutable-after-set `bundle_reply_lease_set: bool`,
    default `true`, setter clears stale pending on disable; `register_lease_set`,
    `encrypt` ES path, and `publish_local_lease_set` gate on the flag; `NewSession`
    path unchanged for handshake liveness; core-only parsing/bounds tests);
  - destination coordination: `emissary-core/src/destination/mod.rs` (already M061-allowed;
    narrow `set_bundle_reply_lease_set` forwarding bridge, no new owner);
  - SAM handoff: `emissary-core/src/sam/session.rs` (already M061-allowed as `SamSession`
    owner; neutral `shouldBundleReplyInfo` fail-safe parse `true` default + plumb to
    `Destination` before activation; no Proposal vocabulary).
  No `i2np/`, `lease_set/` (beyond already-allowed `destination/lease_set.rs` untouched),
  `netdb/`, `crypto/`, transport, tunnel-pool, Cargo/dependency, Yosemite, frontend,
  startup-tunnel or `.github/**` change. M061 must gain exactly one new file
  (`destination/session/mod.rs` with owner evidence) before closure; no prefix/glob waiver.
- I2PControl path budget (exact):
  - `emissary-cli/src/i2pcontrol/backends/runtime/session.rs` (validate `MultiHoming`
    for the two server families only, map to neutral `shouldBundleReplyInfo` via Yosemite
    generic `add_session_option`, shared-session compatibility via existing
    `additional_options_identity`);
  - `emissary-cli/src/i2pcontrol/backends/http_server.rs` (target-family `SUPPORTED`
    gate for `MultiHoming` only; server `SESSION CREATE` carries the wire option);
  - `emissary-cli/src/i2pcontrol/backends/http_bidir.rs` (same gate; server-half session
    only, client-half never inherits).
  No other backend file may accept `MultiHoming`; N/A families keep exact
  `UnsupportedOption` rejection.
- M061/M062 readiness: this amendment is the required pre-registration exact-file
  freeze. Implementation must amend `061-containment-boundary.toml` (add the one
  not-yet-allowed core file with owner evidence) and `m062_dependency_containment.rs`
  (`is_authorized_m145_path`) before closure. No broad prefix waiver.

Class: infrastructure + capability / I2P message privacy and reachability

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-residual-primitive-completion-roadmap.md`.

Target cells:

- `MultiHoming:httpserver`;
- `MultiHoming:httpbidirserver`.

Historical authority:

- M131 `PB-LEASESET-BUNDLING-01`.

## 1. Objective

Implement Proposal `MultiHoming` according to the pinned Java behavior: it is the session option controlling `shouldBundleReplyInfo` / reply LeaseSet bundling on outbound client messages. It is **not** local interface multihoming, source routing, or multiple listener binding.

The milestone requires a real neutral Emissary outbound-message primitive. Passing `shouldBundleReplyInfo` through Yosemite without a router consumer is accept-inert and cannot promote either cell.

## 2. Hard dependency and registration gate

Hard dependency:

- M144 closed and current machine/containment authority reconciled.

M145 MUST NOT be registered until a current-code owner audit identifies the exact files that:

1. construct/send outbound client messages from a local destination;
2. obtain the sender's current publishable LeaseSet;
3. decide whether/when reply LeaseSet information is bundled;
4. encode the relevant garlic/I2NP message content.

This plan intentionally does not pre-authorize a directory. M061 currently treats broad `emissary-core/src/i2np/`, `lease_set/`, `netdb/`, and crypto prefixes as prohibited. Before registration, amend M145 and M061/M062 with only the exact neutral files proven necessary.

If no bounded owner exists without a broad router-message redesign, leave the cells blocked.

## 3. Pinned semantic contract

Pinned Java router evidence:

- `OutboundClientMessageOneShotJob.BUNDLE_REPLY_LEASESET = "shouldBundleReplyInfo"`;
- the sender session's option participates in deciding whether reply LeaseSet information is allowed to be bundled with outbound client traffic;
- a per-message suppression flag can override the session setting;
- the effective behavior uses the sender's real LeaseSet/reply information, not fabricated metadata.

Before coding, freeze from the pinned snapshot:

- default value when the option is absent;
- exact meaning of Proposal `MultiHoming=true/false` relative to `shouldBundleReplyInfo`;
- any frequency/freshness threshold controlling how often a LeaseSet is attached;
- message types/garlic cloves carrying the reply info;
- interaction with LeaseSet expiration, unpublished/hidden destinations, encrypted LeaseSets and per-message flags;
- behavior when no current usable LeaseSet exists.

M145 may not guess these details from the option name.

## 4. Neutral lower-layer contract

The core primitive must be Proposal-free. It should represent a destination/session policy such as `bundle_reply_lease_set: bool` consumed by the existing outbound client-message owner.

Required properties:

- policy scoped to the originating local destination/session, not router-global;
- omitted setting preserves current behavior/default;
- current usable LeaseSet obtained from the canonical destination owner;
- no fabricated, expired, unpublished or nonexistent leases are bundled;
- bounded added message size and no unbounded per-destination history;
- encrypted/authenticated LeaseSet state is not downgraded or exposed;
- no new direct NetDB lookup is performed solely to fabricate sender reply info;
- per-message suppression/override semantics match the reference if Emissary exposes an equivalent layer;
- shared I2PControl sessions have one compatible effective value or reject incompatible sharing before allocation.

## 5. I2PControl mapping

Only after the neutral consumer exists:

- validate Proposal `MultiHoming` for exactly the two target families;
- map it to the exact standard session property/neutral session config consumed by core;
- preserve exact get/edit/restart round-trip;
- include the effective value in shared-session/session-identity compatibility where relevant;
- reject unsupported families before allocation.

If Yosemite can serialize `shouldBundleReplyInfo`, reuse that accepted adapter. Do not modify Yosemite merely to avoid an I2PControl-local mapping if generic `SessionOption` is sufficient.

## 6. Initial path budget

Before registration this section MUST be amended with exact current-code files.

Expected I2PControl candidate paths:

- `emissary-cli/src/i2pcontrol/backends/options.rs`;
- accepted HTTP server backend/runtime session configuration paths;
- `domain/tunnel.rs` / `tunnel_manager.rs` only if typed extraction is needed.

Expected neutral-core owner categories, **not authorized paths**:

- outbound local-destination client-message sender;
- destination's current LeaseSet owner/snapshot;
- exact garlic/I2NP encoder touched by the existing sender, if required.

No transport peer-selection, tunnel-building, RouterInfo, frontend, startup proxy, Cargo/dependency or Yosemite source change is expected.

## 7. Security and privacy invariants

- Never bundle a LeaseSet belonging to a different local destination/session.
- Never expose a private/unpublished LeaseSet contrary to its canonical publication policy.
- Do not downgrade encrypted/blinded/authenticated LeaseSet semantics.
- Do not extend LeaseSet lifetime or advertise expired tunnels.
- Message growth remains protocol-bounded and is checked before allocation/send.
- A malicious remote destination cannot cause arbitrary local LeaseSet selection.
- No LeaseSet/private key bytes enter logs or control responses.
- Disabling bundling must actually suppress the relevant reply info, not merely persist false.
- Enabling bundling with no usable current LeaseSet sends without fabricated reply info according to reference behavior.
- Policy changes are generation/session-local and do not mutate unrelated destinations.

## 8. Failure, cancellation, restart and contention

- Policy validation occurs before server session allocation.
- LeaseSet snapshot acquisition must not hold a destination/router lock across message encryption/send.
- Failure to obtain a usable LeaseSet follows pinned fallback behavior and does not deadlock or retry unboundedly.
- Cancellation of a session prevents stale policy from affecting a successor generation.
- Restart/edit changes the value only for the accepted successor session and preserves last-known-good behavior on failure.
- Any frequency/throttle state is bounded and destination-local.

## 9. Work packages

### WP1 — exact owner/semantic freeze

Trace pinned Java bundling logic and current Emissary outbound client-message path. Amend this plan with exact file paths, default, frequency and message-construction semantics before registration.

### WP2 — neutral core policy and runtime consumer

Add the smallest destination/session policy field and consume it at the actual outbound message owner. Core-only tests must prove the resulting message includes/excludes current reply LeaseSet info as configured.

### WP3 — I2PControl session mapping

Map the two Proposal cells through existing session construction, shared-session compatibility and restart semantics.

### WP4 — privacy/freshness tests

Test no LeaseSet, current LeaseSet, expiring/expired LeaseSet, wrong-destination isolation, disabled policy, message-size boundary and cancellation.

### WP5 — matrix/containment/closure

Update only exact allowed core files, M061/M062, machine matrix/docs/registry and closure evidence.

## 10. Focused tests

Required:

1. absent option preserves frozen reference/default behavior;
2. enabled policy bundles the sender's current valid reply LeaseSet in the exact outbound message path;
3. disabled policy suppresses it;
4. no valid current LeaseSet -> no fabrication and reference-compatible send/failure behavior;
5. expired/expiring/unpublished state is handled according to the frozen contract;
6. destination A can never bundle destination B's LeaseSet under concurrency;
7. message-size bound enforced before oversized construction/send;
8. shared sessions with incompatible policy do not silently share;
9. edit/restart/cancellation isolate generations;
10. I2PControl wire-only test plus a core message-decode test proves an actual runtime difference;
11. encrypted/authenticated LeaseSet regression tests remain no-weaker even though M149-M151 are not yet implemented.

## 11. Matrix promotion budget

Maximum: **2 cells**, derived from the M144 closure baseline.

No promotion occurs until the core outbound-message effect is observable. A new neutral field with no consumer has zero promotion value.

## 12. Broad verification

The amended registered plan must add exact focused core tests for the touched owner. Minimum baseline:

```text
cargo check -p emissary-core
cargo test -p emissary-core --no-fail-fast
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo check -p emissary-cli --no-default-features
cargo check
cargo test -p emissary-cli --no-default-features --features i2pcontrol --lib --no-fail-fast
cargo test -p emissary-cli --no-default-features --features i2pcontrol --no-fail-fast
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment --test m062_dependency_containment --test m095_full_support_matrix --test m105_residual_option_audit --no-fail-fast
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
cargo fmt --all -- --check
git diff --check
```

## 13. Acceptance criteria

M145 closes as complete only when:

- exact pinned `MultiHoming`/`shouldBundleReplyInfo` semantics are frozen;
- exact neutral core files were authorized before implementation;
- actual outbound messages observably include/exclude truthful sender LeaseSet info;
- destination/privacy/freshness/message-size invariants are proven;
- core remains Proposal-free;
- no broad M061 prefix allowance was introduced;
- matrix/docs/registry and M061/M062 match actual code;
- no medium/high privacy, routing or LeaseSet truthfulness defect remains.

## 14. Stop conditions

Stop and leave cells blocked if:

- implementing bundling requires broad I2NP/garlic/NetDB redesign outside a small canonical owner;
- the current destination owner cannot expose a truthful current LeaseSet without leaking private state;
- exact reference frequency/default cannot be established;
- the only available implementation is a Yosemite/session option with no Emissary consumer;
- encrypted/private LeaseSet safety cannot be preserved.

## 15. Closure evidence required

Record amended exact paths, pinned source table, decoded outbound-message fixtures for enabled/disabled cases, LeaseSet freshness/privacy tests, shared-session/restart/cancellation evidence, matrix delta, containment diff, broad verification, implementation SHA, unresolved findings and M146 readiness. External access remains read-only.