# Proposal 170 Implementation Handoffs

Status: partial Proposal 170 production support; all twelve tunnel runtimes real; production tunnel security closed by M093; M095-M096 and M100-M102 closed; M097 closed as blocked; M103 ready; M098/M099/M104 remain dependency-blocked

This directory contains bounded internal implementation and closure handoffs for the I2PControl Proposal 170 subsystem.

Authoritative planning references:

- `plans/000-long-term-specification.md`
- `plans/002-long-term-roadmap.md`
- `plans/003-planning-process.md`
- `plans/adrs/ADR-0001-proposal-170-contract-and-stub-boundary.md`
- `plans/adrs/ADR-0002-control-plane-tunnel-runtime-ownership.md`
- `plans/adrs/ADR-0003-proposal-170-tunnel-runtime-completion-and-filter-boundary.md`
- `plans/adrs/ADR-0004-pinned-full-proposal-170-completion-boundary.md`
- `plans/subsystems/i2pcontrol-proposal-170-roadmap.md`
- `plans/subsystems/i2pcontrol-proposal-170-containment-roadmap.md`
- `plans/subsystems/i2pcontrol-proposal-170-tunnel-runtime-completion-roadmap.md`
- `plans/subsystems/i2pcontrol-proposal-170-tunnel-security-hardening-roadmap.md`
- `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md`
- `plans/implementation/i2pcontrol-proposal-170/061-containment-boundary.toml`
- `plans/implementation/i2pcontrol-proposal-170/062-dependency-containment.toml`
- `plans/registry.md`

Pinned Proposal 170 revision: `2026-05-20` (proposal remains Open).

Full-support planning baseline: `630a8fd1cd4e5943fcde0b5c16f5fc1e88b5d207` — M094 closed planning head before ADR-0004/M095-M104 planning.

## Internal-only rule

All work is internal to `eggstack/emissary`. External specifications, I2P/I2P+/i2pd/Java I2P/Yosemite source, issues, commits, and pull requests are read-only evidence.

No plan authorizes upstream submission, review request, maintainer contact, contribution preparation, merge/adoption request, issue/PR mutation, branch/tag push, or repository write outside this fork.

## Scope and containment

The preferred production boundary remains `emissary-cli/src/i2pcontrol/**`.

The durable containment rules are:

- M061 owns the exact non-I2PControl source boundary;
- M062/M063 own I2PControl-only dependency and feature reachability;
- Proposal 170 business/admin/application/source policy stays under `i2pcontrol` wherever technically possible;
- lower-layer changes are exceptional and must be named by exact path/owner before implementation;
- a new standalone crate is not required merely for aesthetic isolation;
- no broad core/router refactor is authorized for API convenience;
- no hosted CI/fuzz/coverage/release expansion is required by default.

M091 remains the cautionary boundary case: while its plan was registered blocked, an unauthorized vendored Yosemite/core/dependency implementation landed. M092 removed that expansion and restored the accepted containment semantics. M093 independently reclosed the corrected production head. Those dispositions remain current authority.

## Current production state before M095 implementation

The current production state remains partial, despite the new full-support plans:

- RouterInfo: 43 canonical Proposal 170 additions / 41 available / 1 protocol-permitted neutral / 1 unavailable;
- AddressBook CRUD, `SetSubscriptions`, and all thirteen `SetConfig` keys operational within the confined AddressBook owner;
- exactly 12 Proposal 170 tunnel types have real backends;
- exactly 7 canonical TunnelManager actions are implemented;
- applicable but not-yet-implemented runtime options fail before allocation rather than being silently ignored;
- all 6 ClientServicesInfo selectors are implemented;
- full live/reseeded/reference-router interoperability certification remains open.

ADR-0004 changes the intended end state, not the truthfulness of this current baseline.

## Current full-support completion sequence

Only dependency-ready plans registered in `plans/registry.md` are executable. Future plan files below are prewritten for continuity and remain blocked until their hard dependencies close.

| Handoff | Status | Scope |
|---|---|---|
| M095 | **closed** | exact machine-readable full-support matrix and owner/containment budgets; no production behavior |
| M096 | **closed** | operational AddressBook SetConfig for all 13 pinned keys with path confinement/persistence |
| M097 | **blocked; closed as blocked** | common tunnel session/tunnel/key/persistence options; supported length/quantity/encryption plumbing landed, remaining cells await bounded primitives |
| M098 | blocked on M097 | client proxy/outproxy/auth/management/HTTP privacy option completion |
| M099 | blocked on M097 | server access/filter/throttle/LeaseSet option completion |
| M100 | **closed** | request-independent transit 15-second RouterInfo source |
| M101 | **closed** | bounded real router-news source; closure: `plans/closure/i2pcontrol-proposal-170/101-closure.md` |
| M102 | **closed** | minimal neutral IPv4/IPv6 network-error owner observation; wire mapping stays in I2PControl; closure: `plans/closure/i2pcontrol-proposal-170/102-closure.md` |
| M103 | **ready** | real banned-peer owner or proven by-design-empty semantics; no new ban algorithm solely for telemetry |
| M104 | blocked on M097-M103 | integrated live interoperability, final matrix/security/containment reclosure, revision-pinned full-support decision |

Plans:

- `095-full-support-contract-matrix-and-containment-budget.md`
- `096-addressbook-setconfig-operational-completion.md`
- `097-tunnel-common-session-and-key-option-completion.md`
- `098-client-proxy-management-and-http-option-completion.md`
- `099-server-access-throttle-and-leaseset-option-completion.md`
- `100-routerinfo-transit-15s-source-completion.md`
- `101-routerinfo-news-source-completion.md`
- `102-routerinfo-network-error-owner-completion.md`
- `103-routerinfo-banned-peer-semantic-completion.md`
- `104-full-proposal-170-live-interoperability-and-reclosure.md`

### M095 closure and the next ready handoffs

M095 created `095-full-support-matrix.toml` covering:

The matrix is the authoritative cross-domain planning inventory and is guarded
by `emissary-cli/tests/m095_full_support_matrix.rs`.

- all 43 RouterInfo additions;
- all 13 SetConfig keys;
- every canonical TunnelManager option crossed with all 12 tunnel types and classified from actual runtime semantics;
- all 6 ClientServicesInfo selectors;
- unrelated base I2PControl methods explicitly marked outside Proposal 170 scope;
- exact M096-M103 owner/path budgets.

No M096-M103 production work should start from assumptions in the prewritten plans if M095 changes their option/source/applicability model. Reconcile the affected plan first.

## Full-support design rules

### AddressBook

M096 must turn the current exact-but-rejected SetConfig inventory into operational semantics. Path-valued settings remain confined to one I2PControl/AddressBook administrative root. Authentication does not authorize arbitrary host filesystem paths or global logger reconfiguration. Runtime resolver precedence must not be redesigned incidentally.

### Tunnel options

M097-M099 change the final option target from `apply-or-reject` to `apply every applicable cell`, while preserving fail-before-allocation as the safe intermediate rule. M097's closure records the remaining Yosemite/SAM and control-plane key-lifecycle blockers explicitly; M098 and M099 remain blocked on that milestone.

Common session/key options should map through existing Yosemite/SAM/session primitives. Client proxy/privacy options stay in the existing client/filter runtime. Server access/throttle/LeaseSet options compose with the accepted M093 admission/filter/identity boundary.

A missing Yosemite/SAM primitive is a stop condition. No plan authorizes vendoring/forking Yosemite or adding a Proposal-170-shaped core API for convenience.

### RouterInfo transit/news

M100/M101 are intentionally I2PControl-local:

- transit 15s uses a bounded feature-gated sampler over the existing authoritative cumulative transit counter and is independent of RouterInfo request frequency;
- news uses an adopted real bounded source/format/cache and is not an arbitrary public web substitute or per-request fetch.

### RouterInfo network errors

M102 was the only currently anticipated full-support milestone requiring a small neutral lower-layer owner addition. Its explicit healthy and symmetric-NAT observations remain Proposal-170-agnostic in core; I2PControl owns integer wire mapping, and uninitialized/firewalled families remain unavailable.

### RouterInfo banned peers

M103 must not create router ban behavior solely for telemetry. It may expose a real existing enforced ban owner, or—if exhaustive evidence proves Emissary has no possible router-wide banned state—codify authoritative by-design-empty semantics. If neither is truthful, full-support work stays blocked pending a separate maintainer architecture decision.

### Final interoperability

M104 must prove all twelve tunnel types carry real intended traffic in a functional/reseeded environment and use a reference router where practical. Start-success alone is not data-plane completion. M104 remains a focused acceptance harness, not authority for a permanent hosted CI farm.

## Retained tunnel-security authority

The production/security line remains closed by M093. M094 reconciled planning records only.

Historical runtime/security sequence:

| Handoff | Current disposition | Scope |
|---|---|---|
| M064 | closed | baseline feature-disabled corrective |
| M065 | closed | I2PControl client/accepted-server runtime primitives |
| M066 | closed | IRC client/server family |
| M067 | closed | HTTP server family |
| M068 | closed | HTTP client + CONNECT |
| M069 | closed | SOCKS + SOCKS-IRC |
| M070 | closed | HTTP bidirectional composition |
| M071 | closed | Streamr client/server |
| M072 | historical runtime reclosure | integrated twelve-type runtime audit |
| M073 | closed | generic option truthfulness corrective |
| M074 | closed | shared server admission/rate hardening |
| M075 | closed; later lifetime corrective M087 | generic accepted-stream migration |
| M076 | closed | HTTP anonymity/POST hardening |
| M077 | closed | IRC connect/idle hardening |
| M078 | closed | Streamr local-boundary/fanout hardening |
| M079 | closed historical record | integrated tunnel-security reclosure before later correctives |
| M080 | closed | admission transactionality/cardinality |
| M081 | closed | generic LeaseSet option truthfulness |
| M082 | closed | HTTP peer identity/Expect/POST key corrective |
| M083 | closed | admission capacity + exact trusted Destination |
| M084 | closed | merged-head integration/planning corrective |
| M085 | closed for pinned head | independent merged-head security reclosure |
| M086 | closed docs/evidence only | post-M085 reconciliation |
| M087 | closed | generic-server inactivity/half-close corrective |
| M088 | closed; Tier 3 residual | lower-layer/pre-accept resource/timing boundary |
| M089 | closed for pinned head | post-corrective security reclosure |
| M090 | **closed / retained** | resolver-free loopback + IRC half-close correction |
| M091 | **corrective pass required** | unauthorized lower-layer concurrency/vendor/core expansion while registered blocked |
| M092 | **closed** | rollback M091 production/dependency/vendor delta and restore containment/history |
| M093 | **closed; current production security authority** | independent corrected-head tunnel-security reclosure |
| M094 | **closed; docs only** | post-M093 planning-state/SHA reconciliation |

Retained security invariants include Yosemite-derived trusted peer identity, canonical Destination accounting, bounded transactional application admission, HTTP framing/spoof/fingerprint/Expect/POST protections, literal-loopback server targets, bounded generic/IRC lifetimes and half-close, bounded Streamr state, generation-local lifecycle ownership, backend-owned persistent identities/secrets, and redaction.

M088's pre-accept lower-layer residual and Streamr's non-Sybil-resistant finite subscriber-set limitation remain accepted unless a new independently demonstrated defect creates a separate corrective plan.

## Closure rule for the active phase

Until M104 closes, support documentation remains partial and must enumerate remaining gaps.

M104 may use the final statement only if every applicable matrix cell and live evidence supports it:

> Emissary fully supports I2P Proposal 170 against the pinned 2026-05-20 revision.

The proposal remains Open; later draft changes require a new delta audit. This statement must not be expanded into general full I2PControl parity or upstream acceptance.
