# M074 Closure — Shared Server Admission and Rate-Limit Hardening

Status: corrective pass required — independent post-M076 review invalidated the original closure; M080 owns the defects

Source implementation plan:

- `plans/implementation/i2pcontrol-proposal-170/074-server-admission-and-rate-limit-hardening.md`

Corrective successor:

- `plans/implementation/i2pcontrol-proposal-170/080-server-admission-transactionality-and-cardinality-corrective.md`

Original implementation commit:

- `3d1d8f1232a7a25dbff72ef81d63886c7b90bf75`.

## 1. Retained implementation evidence

M074 established several useful and still-required mechanisms:

- one I2PControl-owned shared admission component for `httpserver`, `ircserver`, and inbound `httpbidirserver`;
- default global ceiling 30, hard maximum 128;
- finite per-peer concurrent ceiling 8;
- peer 30/80/200 minute/hour/day reference-scale defaults;
- aggregate 50/0/0 minute/hour/day defaults;
- trusted Yosemite peer identity before handler/local-target work;
- RAII `AdmissionLease` release on normal completion/panic/cancellation/abort;
- bounded primary peer map and fail-closed table-full behavior;
- common task-group/global-capacity ownership;
- no artificial timing jitter;
- production code confined to I2PControl paths.

Those properties should be preserved by M080 unless direct corrective evidence requires a narrower change.

## 2. Original verification evidence

The original closure recorded successful package, feature-disabled/feature-enabled, core, Clippy, M061/M062 containment, scoped nightly rustfmt, and `git diff --check` runs. Paused-time tests covered global/per-peer concurrency, rate windows, expiry after successful admission, table-full behavior, and lease release.

That evidence remains useful but was incomplete for state-transition/cardinality correctness.

## 3. Independent findings invalidating closure

### HIGH — aggregate-rate rejected new peer may persist without expiry

At current head, `ServerAdmissionState::try_acquire` may create a new peer record before aggregate-rate eligibility is evaluated. If the aggregate check then denies the connection, no successful accepted-event path queues expiry for that newly inserted record.

`reap()` only reaches peers through the expiry queue. A remote attacker can therefore exhaust aggregate rate and then present fresh authenticated Destinations to accumulate zero-active unexpiring records until the peer table is full. New identities then receive `PeerStateCapacity` until runtime restart.

This directly contradicts the original closure claim that table-full behavior and expiry made the admission state churn-safe.

### MEDIUM-HIGH — expiry queue is not independently bounded

The peer map is bounded, but the append-only expiry `VecDeque` can accumulate stale/superseded entries on accepted activity and lease release. Every attacker-influenced auxiliary collection must have a finite bound tied to peer/accounting capacity.

### MEDIUM-HIGH — fixed peer capacity is incoherent with long retained windows

The default policy retains peer-day accounting while the aggregate 50/minute limit can admit fresh identities fast enough to fill a 4096-entry peer table well before day-window state naturally expires. A hard map bound protects memory but creates a predictable deny-new-peer condition.

M080 must derive representable peer capacity from enabled retained windows and aggregate arrival bounds within a documented hard memory ceiling, rejecting unsafe configurations before session allocation rather than silently accepting an incoherent runtime policy.

### LOW-MEDIUM — peer accounting uses 64-bit `DefaultHasher`

Admission should use the canonical cryptographic I2P Destination ID/hash rather than an unspecified 64-bit general-purpose hash.

## 4. Why original tests missed the defects

The original suite tested successful insertion/expiry and independent denial types, but did not assert that every denial path leaves peer/counter/expiry state unchanged. In particular it did not cover the exact transition:

```text
new peer inserted -> aggregate rate denial -> no successful expiry registration
```

It also tested that the main peer map had a bound, not that every auxiliary expiry structure was bounded or that the selected peer-table cardinality was coherent with the maximum distinct identities permitted over the longest retained default window.

## 5. Current closure disposition

M074 is not currently closed for security purposes. M080 must add transactional denial, bounded expiry/index state, canonical peer identity, and coherent capacity/retention semantics with regression tests that would have caught these findings.

M077 must not treat M074 as a satisfied hard dependency until M080 closes.

No upstream interaction is authorized. External I2P/I2P+ references remain read-only evidence only.
