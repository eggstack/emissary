# I2PControl Proposal 170 RouterInfo Source-Completion Roadmap

Status: historical partial-support source line closed through M057; ADR-0004 authorizes successor completion work under M095/M100-M103

Historical planning baselines:

- `b759038` — M044 finalized reviewed head;
- `bf9c2eeb` — M045 blocked after stale startup-snapshot rollback;
- `970252c` — merged M053-M052 head with historical 40/1/2 claim;
- `cdbc3a4` — M054-M056 corrected production/reclosure state and M057 planning baseline.

Pinned external authority:

- I2P Proposal 170, `I2PControl Expansion`;
- status: `Open`;
- revision created/updated `2026-05-20`;
- existing I2PControl authentication/JSON-RPC contract;
- read-only i2pd/reference behavior where the proposal adopts or leaves semantics terse.

Canonical/internal authority:

- `plans/000-long-term-specification.md`;
- `plans/003-planning-process.md`;
- `plans/adrs/ADR-0001-proposal-170-contract-and-stub-boundary.md`;
- `plans/adrs/ADR-0004-pinned-full-proposal-170-completion-boundary.md`;
- `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md`;
- M044-M057 implementation/closure records;
- `plans/implementation/i2pcontrol-proposal-170/045-052-routerinfo-source-boundary.toml`;
- M061/M062/M063 containment authorities.

## 1. Purpose and current authority

This roadmap records the completed historical RouterInfo source/truthfulness sequence that produced the current pre-full-support production baseline.

The sequence deliberately preferred truthful unavailability over fabricated state and progressively added only bounded live observations from canonical owners. Its final accepted pre-ADR-0004 state is:

- 43 canonical Proposal 170 RouterInfo additions;
- 37 available;
- 1 protocol-permitted neutral;
- 5 unavailable:
  - `i2p.router.news`;
  - `i2p.router.netdb.bannedpeers`;
  - `i2p.router.net.bw.transit.15s`;
  - `i2p.router.net.error`;
  - `i2p.router.net.error.v6`.

M057 closed the planning-record consistency corrective for that state. M051/M054/M055/M056/M057 remain correct historical authority for the revisions they reviewed.

ADR-0004 does not rewrite or invalidate those closures. It changes the intended final end state by authorizing a new full-support completion phase. New source work is therefore owned by the successor roadmap and milestones:

- M095 — exact full-support/source/containment matrix;
- M100 — request-independent transit 15-second source;
- M101 — router news source;
- M102 — canonical IPv4/IPv6 network-error owner;
- M103 — banned-peer semantic completion;
- M104 — integrated live interoperability/full-support reclosure.

No new implementation handoff is registered from this historical roadmap. `plans/registry.md` is the active execution authority.

## 2. Historical source architecture

The durable architecture established by M045-M057 remains in force:

```text
canonical router/runtime owner
        |
        | smallest neutral bounded observation
        v
I2PControl observation/source adapter
        |
        | source disposition + wire mapping + bounds
        v
Proposal 170 RouterInfo serializer
```

Core/runtime code may expose only facts that the canonical owner is uniquely positioned to know. I2PControl owns Proposal 170 field names, source disposition, numeric/wire mapping, JSON serialization, compatibility semantics, aggregate bounds, and sanitized errors.

A serializer, adjacent metric, startup snapshot, or empty/zero default is not evidence of a live authoritative source.

## 3. Historical milestone outcomes

### M044 — prior corrective reclosure

Established a defensible baseline of 43 additions with 16 available / 1 neutral / 26 unavailable before later source-completion work.

### M045/M053 — known-peer directory

M045's startup `CoreSnapshot` source was invalid. M053 corrected it with a bounded live `ProfileStorage` inspection seam. Known peer directory/list/info remain accepted.

### M046 — active peers and finite transport limits

Added bounded live active-peer inventory/info and finite NTCP/SSU limits through neutral inspection.

### M047 — active-peer statistics

Added bounded active-peer statistics. Rich unstable reference diagnostic members were not silently elevated into contract requirements.

### M048 — tunnel pool counts/details

Added participating/exploratory/client tunnel count/detail observations through bounded tunnel inspection.

### M049/M054 — rolling metrics and transit 15s

M049's recent tunnel-success/queue observations remain accepted.

Its original transit-15s implementation was invalid because the sampler advanced only when RouterInfo was requested. M054 removed that request-history-dependent implementation and restored truthful unavailability.

ADR-0004 later authorizes a different architecture for M100: a bounded feature-gated I2PControl-owned periodic sampler over the existing authoritative cumulative transit counter. That new authority is not retroactively part of M049/M054.

### M050/M055 — network status/testing/error

M050's IPv6 status and v4/v6 testing observations remain accepted.

Its original v4/v6 error exposure lacked a canonical error-reason owner and mapped source absence to `0 / No error`. M055 removed that fabricated semantic and restored truthful unavailability.

ADR-0004 later authorizes M102 to add the smallest neutral explicit owner state only after M095 proves the exact writer/source/path budget. Wire-code mapping must remain in I2PControl.

### M051 — news and banned peers

M051 correctly found no authoritative router-news or banned-peer owner and refused to fabricate values.

ADR-0004 creates two new bounded successor strategies:

- M101 may implement a real bounded news source/cache under I2PControl after M095 freezes exact reference source/format semantics;
- M103 may expose a real existing enforced ban owner, or—if exhaustive evidence proves Emissary has no possible router-wide banned state—codify an authoritative by-design-empty set. M103 explicitly does not authorize a new peer-ban algorithm solely for telemetry.

M051 remains historically correct until those successor milestones independently close.

### M052/M056 — integrated accounting

M052's historical 40/1/2 final accounting was invalidated only for transit-15s and network-error claims.

M056 independently reconciled the corrected final historical/current baseline as 37 available / 1 protocol-permitted neutral / 5 unavailable.

### M057 — planning consistency

M057 changed planning records only and ensured active source documentation consistently distinguished historical `970252c` 40/1/2 evidence from the accepted post-M056 37/1/5 state.

M057 is closed and has no direct successor within this roadmap.

## 4. Durable source/truthfulness invariants

The successor full-support phase MUST retain these rules:

1. Exact Proposal 170 key names, types, and selector-by-presence behavior remain unchanged.
2. Source absence is never serialized as zero, false, empty, or `No error` unless the semantic state itself is explicitly and authoritatively that value.
3. Request frequency must not determine a router-owned rolling metric.
4. Startup one-shot snapshots do not become live source authority.
5. Core observations are neutral, bounded, passive, and read-only.
6. Core observation cannot change routing, peer selection, NetDB, transport, tunnel building, congestion, retry, timing, cryptographic, or LeaseSet behavior.
7. No private/session key material, socket, mutable runtime object, command channel, or message payload crosses inspection boundaries.
8. Collections/histories are bounded and locks do not cross `.await`, sleep, network I/O, or JSON serialization where avoidable.
9. Partial observation fails closed.
10. Proposal 170 source/wire policy remains under `emissary-cli/src/i2pcontrol/**`.
11. Lower-layer paths require explicit pre-implementation containment budgets.
12. No upstream interaction occurs.

## 5. Successor dependency boundary

The historical 37/1/5 state remains the current production truth until successor milestones close.

```text
M057 historical source line closed
   |
   v
ADR-0004 full-support target authorized
   |
   v
M095 exact source/applicability/containment audit [current ready handoff]
   |
   +--> M100 transit 15s
   +--> M101 router news
   +--> M102 v4/v6 network-error owner
   +--> M103 banned-peer semantics
             |
             v
          M104 final reclosure
```

M095-M104 are governed by `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md`, not by reopening M051-M057 implementation plans.

## 6. Containment requirements for successor work

- M100 and M101 target `emissary-cli/src/i2pcontrol/**` only.
- M102 is blocked until M095 names exact neutral canonical owner/writer paths. Existing M061 paths are a ceiling, not blanket permission.
- M103 should remain I2PControl-only if the correct semantic is a proven by-design-empty router set; if a real enforced owner exists, only the exact bounded neutral snapshot path may be considered.
- No new timers/pollers/probes belong in core merely for RouterInfo.
- No news service belongs in core merely for RouterInfo.
- No peer-ban engine may be introduced merely for RouterInfo.

## 7. Verification and closure

Historical M044-M057 closure evidence remains retained in `plans/closure/i2pcontrol-proposal-170/` and is not duplicated here.

New source work follows its own M100-M103 plans/closures and final M104 integration. Only M104 may change the top-level support statement to full support against the pinned revision.

Until then, documentation must continue to state the current 37/1/5 production matrix and identify which successor milestones are incomplete.

## 8. Internal-only rule

All work remains internal to `eggstack/emissary`.

External specifications and reference repositories are read-only evidence. No upstream issue, pull request, review, submission, merge/adoption request, contribution preparation, repository write, or maintainer contact is authorized.