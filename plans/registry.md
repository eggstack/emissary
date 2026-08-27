# Emissary Active Planning Registry

This file is the compact control surface for active planning.

Canonical direction:

- `plans/000-long-term-specification.md`
- `plans/001-terminology-and-domain-model.md`
- `plans/002-long-term-roadmap.md`
- `plans/003-planning-process.md`

Accepted Proposal 170 architecture decisions:

- `plans/adrs/ADR-0001-proposal-170-contract-and-stub-boundary.md`
- `plans/adrs/ADR-0002-control-plane-tunnel-runtime-ownership.md`
- `plans/adrs/ADR-0003-proposal-170-tunnel-runtime-completion-and-filter-boundary.md`
- `plans/adrs/ADR-0004-pinned-full-proposal-170-completion-boundary.md`

Pinned Proposal 170 revision: `2026-05-20` (proposal status remains Open).

## Status vocabulary

- **proposed** — document exists but is not approved for execution.
- **ready** — dependencies and interfaces are satisfied; plan may be handed off.
- **active** — implementation or closure work is in progress.
- **blocked** — a named dependency or evidence requirement prevents progress.
- **closing** — implementation landed and independent closure evidence is being gathered.
- **closed** — closure record accepted for the pinned implementation head.
- **closed internally against pinned revision** — internal closure accepted against an explicitly named revision of an open external specification; does not imply upstream review or acceptance.
- **partial Proposal 170 support** — exact supported dimensions are closed, but one or more pinned capabilities remain truthfully unavailable or unapplied.
- **corrective pass required** — a prior disposition or closure was invalidated by a material implementation, compatibility, scope, merge-integration, security, or evidence defect.
- **superseded** — replaced by another document and not executable.
- **archived** — inactive and retained for traceability.

## Active subsystem roadmaps

| Subsystem | Status | Roadmap | Current handoff | Dependencies or blockers |
|---|---|---|---|---|
| I2PControl Proposal 170 full-support completion | active; M095 closed | `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md` | M096/M097/M100-M103 | M098/M099 remain blocked on M097; M104 remains blocked on M096-M103 |
| I2PControl Proposal 170 source/truthfulness | historical partial baseline closed through M057; successor work authorized by ADR-0004 | `plans/subsystems/i2pcontrol-proposal-170-roadmap.md` | successor source work is M100-M103 under the full-support roadmap | current production matrix remains 37 available / 1 protocol-permitted neutral / 5 unavailable until owning completion milestones close |
| I2PControl Proposal 170 containment | accepted authority; pre-M091 semantics restored by M092 | `plans/subsystems/i2pcontrol-proposal-170-containment-roadmap.md` | none | M061 source boundary and M062/M063 dependency/feature rules remain authoritative; new planning paths may be added only as exact bookkeeping entries |
| I2PControl Proposal 170 tunnel runtime completion | functionally complete | `plans/subsystems/i2pcontrol-proposal-170-tunnel-runtime-completion-roadmap.md` | no data-plane handoff | all twelve registered tunnel backends remain real; full-option semantics continue under M097-M099 without redesigning the data planes |
| I2PControl Proposal 170 tunnel security hardening | production/security closed after M093; M094 planning cleanup closed | `plans/subsystems/i2pcontrol-proposal-170-tunnel-security-hardening-roadmap.md` | no security-corrective handoff | M093 remains current production/security authority; M088 lower-layer residual remains accepted unless new evidence directly reopens it |

## Current full-support completion sequence

ADR-0004 changes the intended final completeness target without rewriting the historical correctness of M051-M057, M064-M094, or their closure records.

The current production baseline remains truthful partial support. Full support means full support against the explicitly pinned Proposal 170 revision `2026-05-20`, not general I2PControl parity and not upstream acceptance.

```text
M095 exact full-support matrix + containment budget     [CLOSED]
  |
  +------------------+-------------------+------------------+
  |                  |                   |                  |
  v                  v                   v                  v
M096 AddressBook   M097 common tunnel  M100 transit 15s   M101 router news
SetConfig          session/key opts    source             source
[READY]            [READY]             [READY]          [READY]
                       |
                       +----------------------+
                       |                      |
                       v                      v
                 M098 client/proxy      M099 server/
                 management/HTTP opts   LeaseSet/access opts
                 [BLOCKED M097]         [BLOCKED M097]

M095 ----------------------------------------------+
  |                                                |
  v                                                v
M102 canonical network-error owner            M103 banned-peer semantic closure
[READY]                                         [READY]
  |                                                |
  +----------------------+-------------------------+
                         |
M096-M103 all closed ----+
                         |
                         v
                 M104 live interoperability +
                 full Proposal 170 reclosure
                 [BLOCKED M096-M103]
```

### Closed handoff — M095

Plan:

- `plans/implementation/i2pcontrol-proposal-170/095-full-support-contract-matrix-and-containment-budget.md`

Status: **closed**; closure: `plans/closure/i2pcontrol-proposal-170/095-closure.md`.

M095 was planning/matrix/containment-budget work and had no production-behavior authority. Its exact machine-readable inventory is now the prerequisite for the next ready handoffs.

Required output includes:

- all 43 Proposal 170 RouterInfo additions;
- all 13 AddressBook SetConfig keys;
- every canonical TunnelManager option crossed with all 12 tunnel types and classified from actual runtime behavior;
- all 6 ClientServicesInfo selectors;
- explicit separation of unrelated base I2PControl methods;
- exact owner/path budgets for M096-M103.

### Prewritten blocked handoffs

The following plans exist for handoff continuity but are not registered as executable work until their hard dependencies close:

| Milestone | Status | Plan | Primary target |
|---|---|---|---|
| M096 | **ready** | `plans/implementation/i2pcontrol-proposal-170/096-addressbook-setconfig-operational-completion.md` | operational semantics for all 13 SetConfig keys with confined filesystem ownership |
| M097 | **ready** | `plans/implementation/i2pcontrol-proposal-170/097-tunnel-common-session-and-key-option-completion.md` | common session/tunnel/key/persistence options through existing Yosemite/SAM primitives |
| M098 | blocked on M097 | `plans/implementation/i2pcontrol-proposal-170/098-client-proxy-management-and-http-option-completion.md` | client proxy/outproxy/auth/management/HTTP privacy options |
| M099 | blocked on M097 | `plans/implementation/i2pcontrol-proposal-170/099-server-access-throttle-and-leaseset-option-completion.md` | server access/throttle/filter/LeaseSet options while preserving M093 security boundaries |
| M100 | **ready** | `plans/implementation/i2pcontrol-proposal-170/100-routerinfo-transit-15s-source-completion.md` | request-independent I2PControl-owned transit 15-second sampler |
| M101 | **ready** | `plans/implementation/i2pcontrol-proposal-170/101-routerinfo-news-source-completion.md` | bounded real router-news source under I2PControl |
| M102 | **ready** | `plans/implementation/i2pcontrol-proposal-170/102-routerinfo-network-error-owner-completion.md` | minimal neutral v4/v6 network-error owner observation, wire mapping in I2PControl |
| M103 | **ready** | `plans/implementation/i2pcontrol-proposal-170/103-routerinfo-banned-peer-semantic-completion.md` | real ban-owner snapshot or proven by-design-empty semantics; no ban engine solely for telemetry |
| M104 | blocked on M096-M103 | `plans/implementation/i2pcontrol-proposal-170/104-full-proposal-170-live-interoperability-and-reclosure.md` | integrated matrix, live interoperability, security/containment reclosure, revision-pinned full-support decision |

## Full-support containment authority

The preferred production boundary remains `emissary-cli/src/i2pcontrol/**`.

ADR-0004 requires the completion sequence to exhaust I2PControl-local options before changing lower-layer ownership. M102 is the only currently anticipated milestone that may need a minimal neutral lower-layer state addition, and it cannot execute until M095 names exact owner/writer/path evidence.

M103 explicitly does not authorize a new router-wide peer-ban algorithm solely for the `bannedpeers` getter. If no real ban owner exists, it may close only with evidence-backed by-design-empty semantics; otherwise it remains blocked pending a separate maintainer architecture decision.

M061 remains the exact source-path containment authority. M062/M063 remain the dependency/feature-containment authority. Planning-path additions to the M062 cumulative allowlist are bookkeeping only and must not broaden production globs, dependency ownership, lockfile authority, or core paths.

## Current production/security authority retained

M090 remains valid production work. M091 remains corrective-pass-required technical history and is not current authorization. M092 removed the unauthorized M091 vendor/core/dependency expansion and restored containment. M093 independently reclosed the corrected production head. M094 reconciled planning records only.

```text
M087 generic server inactivity corrective            [CLOSED]
  |
  v
M088 pre-accept boundary evidence                     [CLOSED / TIER 3]
  |
  v
M089 independent security reclosure                   [CLOSED]
  |
  v
M090 resolver-free loopback + IRC half-close          [CLOSED / RETAINED]
  |
  v
M091 pre-accept stream concurrency implementation     [CORRECTIVE PASS REQUIRED]
  |
  v
M092 authorization/dependency/containment rollback    [CLOSED @ 8860407]
  |
  v
M093 post-M092 independent security reclosure         [CLOSED; CURRENT SECURITY AUTHORITY]
  |
  v
M094 planning-state reconciliation                    [CLOSED / DOCS ONLY]
```

Retained durable security invariants include:

- exact Proposal 170 tunnel spelling/actions/types and all twelve real backends;
- Yosemite-derived trusted peer identity;
- bounded transactional application admission;
- HTTP spoof/fingerprint/framing/Expect/POST protections;
- literal-loopback HTTP/IRC server targets;
- bounded generic/IRC lifetimes and half-close behavior;
- bounded Streamr subscriber/expiry/refresh/control/payload/fanout state;
- generation-local runtime ownership and restart behavior;
- backend-owned persistent server identities and redacted secrets;
- fail-before-allocation for unapplied runtime-relevant options until their owning full-support option milestone closes.

M088's lower-layer signed-SYN/pre-accept resource/timing residual remains accepted. Streamr's finite subscriber set remains non-Sybil-resistant and is retained as a specialized availability limitation. The full-support phase does not reopen either merely for option/source parity.

## Current Proposal 170 production state

Until the relevant new milestone closes, the current support statements remain:

- RouterInfo: 43 canonical additions / 37 available / 1 protocol-permitted neutral / 5 unavailable;
- AddressBook CRUD and SetSubscriptions operational; non-empty SetConfig still rejected truthfully;
- all 12 TunnelManager types and 7 canonical actions real/operational within their current option-capability sets;
- applicable-but-unimplemented runtime options still fail before allocation rather than being persisted-and-ignored;
- all 6 ClientServicesInfo selectors implemented;
- full public-network/reference-router certification not yet complete.

The five current unavailable RouterInfo rows are assigned to M100-M103. Historical M051/M054/M055/M056 conclusions remain correct for the production revisions they reviewed and are superseded only as new source milestones close.

## Registry maintenance rules

1. M095 is closed; M096, M097, and M100-M103 are the ready/executable full-support handoffs.
2. M098/M099 remain blocked on M097 and M104 remains blocked on M096-M103; existence of a plan file is not execution authority.
3. Do not alter the current partial-support claim merely because planning exists.
4. M093 remains the current tunnel production/security reclosure authority until a later integrated reclosure closes.
5. M061/M062/M063 containment remains authoritative; exact planning bookkeeping does not authorize new production paths.
6. New Proposal 170 business/admin/runtime option policy remains under `emissary-cli/src/i2pcontrol/**` wherever technically possible.
7. M102 lower-layer work requires exact pre-implementation owner/writer/path evidence from M095.
8. M103 must not introduce peer-ban behavior solely for telemetry.
9. Unrelated base I2PControl methods remain outside this Proposal 170 phase.
10. Proposal 170 is pinned to revision `2026-05-20`; a later draft revision requires a separate delta audit.
11. External sources remain read-only. No upstream review, merge, submission, contribution preparation, issue/PR mutation, adoption request, or maintainer contact is authorized.
12. All repository writes remain internal to `eggstack/emissary`.
