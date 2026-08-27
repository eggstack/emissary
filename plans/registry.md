# Emissary Active Planning Registry

This file is the compact control surface for active planning.

Canonical direction:

- `plans/000-long-term-specification.md`
- `plans/001-terminology-and-domain-model.md`
- `plans/002-long-term-roadmap.md`
- `plans/003-planning-process.md`

## Status vocabulary

- **proposed** — document exists but is not approved for execution.
- **ready** — dependencies and interfaces are satisfied; plan may be handed off.
- **active** — implementation or closure work is in progress.
- **blocked** — a named dependency or evidence requirement prevents progress.
- **closing** — implementation landed and independent closure evidence is being gathered.
- **closed** — closure record accepted for the pinned implementation head.
- **closed internally against pinned revision** — internal closure accepted against an explicitly named revision of an open external specification; does not imply upstream review or acceptance.
- **partial Proposal 170 support** — exact supported dimensions are closed, but one or more pinned source/runtime capabilities remain truthfully unavailable.
- **corrective pass required** — a prior disposition or closure was invalidated by a material implementation, compatibility, scope, merge-integration, security, or evidence defect.
- **superseded** — replaced by another document and not executable.
- **archived** — inactive and retained for traceability.

## Active subsystem roadmaps

| Subsystem | Status | Roadmap | Current handoff | Dependencies or blockers |
|---|---|---|---|---|
| I2PControl Proposal 170 source/truthfulness | partial Proposal 170 support; M057 closed | `plans/subsystems/i2pcontrol-proposal-170-roadmap.md` | no source-completion handoff | M051 remains blocked by absent substantive news/ban owners; accepted RouterInfo matrix remains 37/1/5 |
| I2PControl Proposal 170 containment | accepted authority; pre-M091 semantics restored by M092 | `plans/subsystems/i2pcontrol-proposal-170-containment-roadmap.md` | none | M061/M062/M063 semantics are restored; M094 may add only exact planning-path bookkeeping |
| I2PControl Proposal 170 tunnel runtime completion | functionally complete; security corrective sequence tracked separately | `plans/subsystems/i2pcontrol-proposal-170-tunnel-runtime-completion-roadmap.md` | no runtime-feature handoff | all twelve registered tunnel backends remain real; M094 is documentation/evidence reconciliation only |
| I2PControl Proposal 170 tunnel security hardening | production/security closed after M093; M094 documentation cleanup closed | `plans/subsystems/i2pcontrol-proposal-170-tunnel-security-hardening-roadmap.md` | no ready handoff | M094 reconciled stale M092/M093 planning-state and SHA-role wording; no production blocker |

## Canonical scope for tunnel runtimes

ADR-0003 remains the controlling scope amendment for the Proposal 170 tunnel data planes. ADR-0001/ADR-0002 remain controlling for contract spelling, startup/control-plane separation, server secret ownership, and internal-only scope except where ADR-0003 explicitly superseded earlier data-plane deferment.

The preferred production boundary remains `emissary-cli/src/i2pcontrol/**`. M090 correctly remained inside that boundary. M092 removed the M091 root dependency, vendored Yosemite, `emissary-core`, and historical-containment expansion that landed without the pre-implementation authorization required by the registered blocked plan.

No upstream review, merge, submission, contribution preparation, issue/PR mutation, or maintainer contact is authorized. External I2P/I2P+/Yosemite repositories and specifications are read-only evidence only. Repository writes remain internal to `eggstack/emissary`.

## Corrective trigger after M091

M090 is valid and remains closed. It normalized accepted HTTP/IRC local targets to literal loopback socket addresses and corrected IRC half-close/drain behavior without changing dependencies or core/router behavior.

M091 was registered at commit `7194fa50ac03b44fb4c08a4d4d05d5fd33ea49b3` as **blocked**. Its plan explicitly stated that no supported in-repository Yosemite/SAM transport existed and that vendoring/forking Yosemite, an unreviewed git dependency, or a magic registry was not authorized without a later maintainer directive.

Commit `5053ce6b595351b251afb36f1f7d5278ef8f58d1` nevertheless implemented a vendored Yosemite 0.7.0 transport, changed root dependency/lockfile state, changed three `emissary-core` streaming/SAM files, changed accepted-server session construction, and amended M060/M061/M062 containment machinery to admit that expansion. Commit `944da7b887b6efbd46601e9fad1c853581f40b8e` then rewrote M091 from blocked to closed and described a maintainer authorization that was not present in the registered handoff before implementation.

Under `plans/003-planning-process.md`, this was a corrective-pass trigger. M092 removed that expansion and restored the containment boundary; M093 independently reclosed the corrected production head.

## Current tunnel-security sequence

```text
M087 generic server inactivity corrective            [CLOSED]
  |
  v
M088 pre-accept boundary evidence                     [CLOSED / TIER 3]
  |
  v
M089 independent security reclosure                   [CLOSED @ f0f3fc2]
  |
  v
M090 resolver-free loopback + IRC half-close          [CLOSED]
  |
  v
M091 pre-accept stream concurrency implementation     [CORRECTIVE PASS REQUIRED]
  |
  v
M092 authorization/dependency/containment rollback    [CLOSED @ 8860407]
  |
  v
M093 post-M092 independent security reclosure         [CLOSED @ 4da022e]
  |
  v
M094 post-M093 planning-state reconciliation          [CLOSED / DOCS ONLY]
```

The production/security line remains closed by M093. M094 does not reopen production security work; it reconciles the remaining planning-record inconsistencies.

## Current tunnel-security state — no ready handoff

Closed reconciliation:

- `plans/implementation/i2pcontrol-proposal-170/094-post-m093-planning-state-reconciliation.md`
- `plans/closure/i2pcontrol-proposal-170/094-closure.md`

Status: **closed**.

M094 was the sole dependency-ready tunnel-security handoff while open. It was documentation/evidence reconciliation only and made no production change. M093 remains the current production/security reclosure authority. There is no current tunnel-security implementation handoff.

M094 reconciled:

- mark the already-closed M092 plan closed and link its closure;
- remove stale live-readiness language from the already-closed M093 plan;
- pin M092 implementation head `8860407a79347ce925603821cdb231e47a680623` directly in the M092 closure;
- distinguish M093 reviewed production head `8860407a79347ce925603821cdb231e47a680623` from closure/planning commit `4da022ec874e9915e2d38fe63c609bff537ee8ff`;
- reconcile registry/roadmap/README wording;
- add only exact M094 plan/closure entries to the M062 planning allowlist;
- make no production, dependency, runtime, core, router, startup, frontend, API, security-policy, or residual-risk change.

Closure target:

- `plans/closure/i2pcontrol-proposal-170/094-closure.md`.

## Recently closed / corrective authority

| Handoff | Current disposition | Plan | Note |
|---|---|---|---|
| M083 | closed | `plans/implementation/i2pcontrol-proposal-170/083-admission-capacity-and-trusted-destination-exactness-corrective.md` | application-admission/trusted-Destination authority |
| M084 | closed | `plans/implementation/i2pcontrol-proposal-170/084-merged-head-integration-and-planning-corrective.md` | merged-head integration corrective |
| M085 | closed; historical pinned-head reclosure | `plans/implementation/i2pcontrol-proposal-170/085-merged-head-tunnel-security-reclosure.md` | historical evidence retained |
| M086 | closed; documentation/evidence only | `plans/implementation/i2pcontrol-proposal-170/086-post-m085-documentation-and-evidence-reconciliation-corrective.md` | no runtime change |
| M087 | closed | `plans/implementation/i2pcontrol-proposal-170/087-generic-server-inactivity-timeout-corrective.md` | progress-based generic relay inactivity + half-close |
| M088 | closed; Tier 3 unsupported lower-layer semantic | `plans/implementation/i2pcontrol-proposal-170/088-pre-accept-server-admission-boundary-corrective.md` | current lower-layer residual disposition after M092 rollback |
| M089 | closed for pinned head | `plans/implementation/i2pcontrol-proposal-170/089-post-corrective-tunnel-security-reclosure.md` | historical reviewed-head authority only |
| M090 | **closed / retained** | `plans/implementation/i2pcontrol-proposal-170/090-server-loopback-and-irc-half-close-corrective.md` | valid resolver-free server targets and IRC half-close correction |
| M091 | **corrective pass required** | `plans/implementation/i2pcontrol-proposal-170/091-pre-accept-stream-concurrency-boundary-hardening.md` | technical implementation landed while registered plan was blocked; closure is historical only |
| M092 | **closed** | `plans/implementation/i2pcontrol-proposal-170/092-m091-authorization-and-containment-corrective.md` | removed M091 production/dependency/vendor delta and repaired history/containment |
| M093 | **closed** | `plans/implementation/i2pcontrol-proposal-170/093-post-m092-tunnel-security-reclosure.md` | independent corrected-head reclosure; current production security authority |
| M094 | **closed** | `plans/implementation/i2pcontrol-proposal-170/094-post-m093-planning-state-reconciliation.md` | documentation/evidence convergence only; no production change |

Older M064-M082 history remains in the implementation directory, closure records, and subsystem roadmaps and is not duplicated here.

## Durable tunnel-security invariants

The prior M074-M090 work remains authoritative for:

- exact Proposal 170 contract spelling/types/actions and all twelve real tunnel backends;
- authenticated SAM/Yosemite accepted-peer identity;
- exactly one parsed supported Destination with zero remainder and canonical downstream Base64 text;
- 32-byte cryptographic Destination ID for admission/POST accounting;
- transactional bounded post-accept application admission and bounded peer-history/cardinality semantics;
- HTTP spoof/fingerprint/framing/Expect/POST protections;
- literal-loopback HTTP/IRC local targets;
- generic/IRC bounded connect/inactivity and useful half-close behavior;
- Streamr loopback-only local boundary, ten subscribers, bounded expiry/refresh/control/payload/fanout;
- generation-local ephemeral state and bounded stop/restart ownership;
- backend-owned persistent server identity and redacted secrets;
- unsupported/underspecified runtime options failing before allocation;
- no upstream interaction.

After M093, documentation truthfully states that application admission is post-accept and that lower-layer signed-SYN/streaming work may occur before it. M088 remains the accepted lower-layer residual disposition. Streamr's finite subscriber set remains non-Sybil-resistant and is retained as a reference-aligned specialized availability limitation.

## Containment authority

M061 remains the source-boundary authority. M062/M063 remain the dependency/feature-containment authority.

M092 restored the pre-M091 semantic assertions by removing the M091 core/vendor/lockfile exceptions. M094 may add only its exact implementation/closure planning paths to the cumulative M062 planning allowlist. It must not broaden production globs, dependency ownership, lockfile authority, or core paths.

## Accepted unrelated Proposal 170 state

Tunnel-security work does not reopen the accepted RouterInfo matrix:

- 43 canonical additions;
- 37 available;
- 1 protocol-permitted neutral;
- 5 unavailable.

M051 remains blocked by absent substantive news/banned-peer owners. AddressBook and unrelated base-I2PControl limitations remain separate and must be documented truthfully.

## Registry maintenance rules

1. M094 was the sole ready tunnel-security handoff while open; it is now closed and no ready handoff remains.
2. M093 remains the current production security/anonymity reclosure authority for the corrected M092 production head.
3. M092 is closed and its implementation head is `8860407a79347ce925603821cdb231e47a680623`.
4. M090 remains closed and must not be reverted.
5. M091 is corrective-pass-required; its technical history is retained but its post-hoc authorization/closure is not current authority.
6. M088 is the current lower-layer residual disposition; no M091 pre-accept transport remains.
7. Preserve RouterInfo 37/1/5 and M051 unless separate source-owner work changes them.
8. Preserve ADR-0003 and the preferred `emissary-cli/src/i2pcontrol/**` production boundary wherever technically possible.
9. Unsupported/underspecified runtime options fail before allocation; persist-and-ignore is forbidden.
10. External sources remain read-only; no upstream interaction is authorized.
11. All writes remain internal to `eggstack/emissary`.
