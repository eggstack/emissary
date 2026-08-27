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
| I2PControl Proposal 170 containment | accepted authority; M091 exception under corrective rollback | `plans/subsystems/i2pcontrol-proposal-170-containment-roadmap.md` | M092 | M061/M062/M063 semantics must be restored to the pre-M091 production/dependency boundary |
| I2PControl Proposal 170 tunnel runtime completion | functionally complete; security corrective sequence tracked separately | `plans/subsystems/i2pcontrol-proposal-170-tunnel-runtime-completion-roadmap.md` | no runtime-feature handoff | all twelve registered tunnel backends remain real; M092 is rollback/governance correction, not new tunnel capability |
| I2PControl Proposal 170 tunnel security hardening | **corrective pass required; M092 closed; M093 ready** | `plans/subsystems/i2pcontrol-proposal-170-tunnel-security-hardening-roadmap.md` | `plans/implementation/i2pcontrol-proposal-170/093-post-m092-tunnel-security-reclosure.md` | M091 implemented while blocked and its dependency authorization was applied post hoc; M092 is closed; M093 is the next dependency-ready handoff |

## Canonical scope for tunnel runtimes

ADR-0003 remains the controlling scope amendment for the Proposal 170 tunnel data planes. ADR-0001/ADR-0002 remain controlling for contract spelling, startup/control-plane separation, server secret ownership, and internal-only scope except where ADR-0003 explicitly superseded earlier data-plane deferment.

The preferred production boundary remains `emissary-cli/src/i2pcontrol/**`. M090 correctly remained inside that boundary. M092 exists because M091 crossed into root dependency state, `emissary-core`, historical containment machinery, and a full vendored Yosemite copy without the pre-implementation authorization its registered plan required.

No upstream review, merge, submission, contribution preparation, issue/PR mutation, or maintainer contact is authorized. External I2P/I2P+/Yosemite repositories and specifications are read-only evidence only. Repository writes remain internal to `eggstack/emissary`.

## Corrective trigger after M091

M090 is valid and remains closed. It normalized accepted HTTP/IRC local targets to literal loopback socket addresses and corrected IRC half-close/drain behavior without changing dependencies or core/router behavior.

M091 was registered at commit `7194fa50ac03b44fb4c08a4d4d05d5fd33ea49b3` as **blocked**. Its plan explicitly stated that no supported in-repository Yosemite/SAM transport existed and that vendoring/forking Yosemite, an unreviewed git dependency, or a magic registry was not authorized without a later maintainer directive.

Commit `5053ce6b595351b251afb36f1f7d5278ef8f58d1` nevertheless implemented a vendored Yosemite 0.7.0 transport, changed root dependency/lockfile state, changed three `emissary-core` streaming/SAM files, changed accepted-server session construction, and amended M060/M061/M062 containment machinery to admit that expansion. Commit `944da7b887b6efbd46601e9fad1c853581f40b8e` then rewrote M091 from blocked to closed and described a maintainer authorization that was not present in the registered handoff before implementation.

Under `plans/003-planning-process.md`, this is a corrective-pass trigger. Technical test success does not create retroactive implementation authority.

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
M092 authorization/dependency/containment rollback    [CLOSED]
  |
  v
M093 post-M092 independent security reclosure         [READY]
```

M093 is the only dependency-ready tunnel-security implementation handoff. M092 has an accepted closure record at `plans/closure/i2pcontrol-proposal-170/092-closure.md`.

## Current ready handoff — M093

Plan:

- `plans/implementation/i2pcontrol-proposal-170/093-post-m092-tunnel-security-reclosure.md`

Planning baseline:

- `944da7b887b6efbd46601e9fad1c853581f40b8e`.

Known valid pre-M091 implementation/closure baseline:

- `6d631d4423c7faa761b47a84e07436bbaf5d9ad4`.

M092 has been executed and closed; see `plans/closure/i2pcontrol-proposal-170/092-closure.md`. M093 is now the only dependency-ready tunnel-security handoff.

M093 must:

- independently audit all twelve Proposal 170 tunnel backends at the corrected head;
- verify M090 remains intact;
- verify M091 production/dependency/vendor artifacts are gone;
- verify containment semantics were restored rather than weakened;
- recheck generic/HTTP/IRC/Streamr security and lifetime behavior;
- explicitly record the post-accept application-admission boundary and accepted lower-layer resource/timing residual;
- make no production code changes;
- open a new numbered corrective for any high/medium production defect.

## Former ready handoff — M092 (closed)

M092 has been executed and closed. See `plans/closure/i2pcontrol-proposal-170/092-closure.md`.

No further tunnel-security handoff is registered or ready. Subsequent registry updates follow the normal next-handoff review; future work remains in the subsystem roadmap rather than being promoted to active planning merely because M092 closed.

## Recently closed / corrective authority

| Handoff | Current disposition | Plan | Note |
|---|---|---|---|
| M083 | closed | `plans/implementation/i2pcontrol-proposal-170/083-admission-capacity-and-trusted-destination-exactness-corrective.md` | application-admission/trusted-Destination authority |
| M084 | closed | `plans/implementation/i2pcontrol-proposal-170/084-merged-head-integration-and-planning-corrective.md` | merged-head integration corrective |
| M085 | closed; historical pinned-head reclosure | `plans/implementation/i2pcontrol-proposal-170/085-merged-head-tunnel-security-reclosure.md` | historical evidence retained |
| M086 | closed; documentation/evidence only | `plans/implementation/i2pcontrol-proposal-170/086-post-m085-documentation-and-evidence-reconciliation-corrective.md` | no runtime change |
| M087 | closed | `plans/implementation/i2pcontrol-proposal-170/087-generic-server-inactivity-timeout-corrective.md` | progress-based generic relay inactivity + half-close |
| M088 | closed; Tier 3 unsupported lower-layer semantic | `plans/implementation/i2pcontrol-proposal-170/088-pre-accept-server-admission-boundary-corrective.md` | becomes current lower-layer residual disposition after M092 rollback |
| M089 | closed for pinned head | `plans/implementation/i2pcontrol-proposal-170/089-post-corrective-tunnel-security-reclosure.md` | historical reviewed-head authority only |
| M090 | **closed / retained** | `plans/implementation/i2pcontrol-proposal-170/090-server-loopback-and-irc-half-close-corrective.md` | valid resolver-free server targets and IRC half-close correction |
| M091 | **corrective pass required** | `plans/implementation/i2pcontrol-proposal-170/091-pre-accept-stream-concurrency-boundary-hardening.md` | technical implementation landed while registered plan was blocked; closure is not current authority |
| M092 | **closed** | `plans/implementation/i2pcontrol-proposal-170/092-m091-authorization-and-containment-corrective.md` | remove M091 production/dependency/vendor delta and repair history/containment |
| M093 | **ready** | `plans/implementation/i2pcontrol-proposal-170/093-post-m092-tunnel-security-reclosure.md` | independent corrected-head reclosure after M092 |

Older M064-M082 history remains in the implementation directory, closure records, and subsystem roadmaps and is not duplicated here.

## M092 scope guard

Authorized production/dependency rollback paths are exactly:

- `Cargo.toml`;
- `Cargo.lock`;
- `emissary-cli/src/i2pcontrol/backends/runtime/accepted_server.rs`;
- `emissary-core/src/sam/protocol/streaming/config.rs`;
- `emissary-core/src/sam/protocol/streaming/mod.rs`;
- `emissary-core/src/sam/session.rs`;
- removal of `vendor/yosemite/**`;
- `emissary-cli/tests/m060_containment.rs`;
- `emissary-cli/tests/m062_dependency_containment.rs`;
- `plans/implementation/i2pcontrol-proposal-170/061-containment-boundary.toml`;
- `plans/implementation/i2pcontrol-proposal-170/062-dependency-containment.toml`.

Planning/status files named by M092 may also change. No other production/core/router/startup/frontend/dependency/protocol path is authorized.

M092 must use M090 closure head `6d631d4423c7faa761b47a84e07436bbaf5d9ad4` as the reference for M091-touched production/dependency files. It must restore pre-M091 containment semantics rather than preserve M091's self-authorizing exceptions.

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

After M092, documentation must again state truthfully that application admission is post-accept and that lower-layer signed-SYN/streaming work may occur before it. Streamr's finite subscriber set remains non-Sybil-resistant and is retained as a reference-aligned specialized availability limitation.

## Containment authority

M061 remains the source-boundary authority. M062/M063 remain the dependency/feature-containment authority.

M092 must remove the M091 core/vendor/lockfile exceptions and restore the pre-M091 semantic assertions. The cumulative M062 planning allowlist may contain exact M092/M093 plan and closure paths only as planning bookkeeping; this must not broaden production globs or dependency ownership.

## Accepted unrelated Proposal 170 state

Tunnel-security work does not reopen the accepted RouterInfo matrix:

- 43 canonical additions;
- 37 available;
- 1 protocol-permitted neutral;
- 5 unavailable.

M051 remains blocked by absent substantive news/banned-peer owners. AddressBook and unrelated base-I2PControl limitations remain separate and must be documented truthfully.

## Registry maintenance rules

1. M093 is the only dependency-ready tunnel-security implementation handoff.
2. M092 is closed; see `plans/closure/i2pcontrol-proposal-170/092-closure.md`.
3. M090 remains closed and must not be reverted by M092.
4. M091 is corrective-pass-required; its technical history is retained but its post-hoc authorization/closure is not current authority.
5. M088 becomes the current lower-layer residual disposition after M092 removes the M091 transport.
6. Preserve RouterInfo 37/1/5 and M051 unless separate source-owner work changes them.
7. Preserve ADR-0003 and the preferred `emissary-cli/src/i2pcontrol/**` production boundary wherever technically possible.
8. Unsupported/underspecified runtime options fail before allocation; persist-and-ignore is forbidden.
9. External sources remain read-only; no upstream interaction is authorized.
10. All writes remain internal to `eggstack/emissary`.
