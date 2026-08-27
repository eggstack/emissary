# M095 Closure — Full-Support Contract Matrix and Containment Budget

Status: closed

Source implementation plan:

- `plans/implementation/i2pcontrol-proposal-170/095-full-support-contract-matrix-and-containment-budget.md`

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md`

Planning baseline: `630a8fd1cd4e5943fcde0b5c16f5fc1e88b5d207`.
Pre-closure implementation/planning head: `087ee59abb5494e1413d96af8f49f96b00948a7d`.
Closure head: the final internal commit containing this record; its exact identifier is
reported with the repository handoff.
Review date: 2026-08-27.

## 1. Disposition

M095 closes as a planning, machine-readable inventory, and containment-budget milestone.
It makes no production, dependency, lockfile, runtime, router, frontend, or API behavior
change. The repository remains at partial Proposal 170 support.

The authoritative matrix is
`plans/implementation/i2pcontrol-proposal-170/095-full-support-matrix.toml` and its
feature-gated exhaustiveness guard is
`emissary-cli/tests/m095_full_support_matrix.rs`.

## 2. Exact changed-path matrix

| Area | Paths |
|---|---|
| Matrix and guard | `plans/implementation/i2pcontrol-proposal-170/095-full-support-matrix.toml`; `emissary-cli/tests/m095_full_support_matrix.rs` |
| M095 planning/closure | `plans/implementation/i2pcontrol-proposal-170/095-full-support-contract-matrix-and-containment-budget.md`; `plans/closure/i2pcontrol-proposal-170/095-closure.md` |
| Dependency-ready handoffs | M096, M097, M100, M101, M102, and M103 plan files |
| Planning indexes | `plans/implementation/i2pcontrol-proposal-170/README.md`; `plans/registry.md`; `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md` |
| Support/architecture documentation | `README.md`; `AGENTS.md`; `docs/i2pcontrol/README.md`; `docs/i2pcontrol/proposal-170-support.md`; `docs/i2pcontrol/proposal-170-conformance.md`; `docs/i2pcontrol/inspection-architecture.md` |
| Existing containment bookkeeping | `emissary-cli/tests/m062_dependency_containment.rs` |

The last path adds only exact M095 planning/matrix/closure/test entries to the existing
cumulative bookkeeping guard. No M061 production boundary, dependency rule, or lockfile
authority was broadened.

## 3. Pinned proposal evidence

The official source is:

- URL: `https://i2p.net/proposals/170-i2pcontrol-expansion.txt`;
- proposal: I2PControl Expansion, Proposal 170;
- status: Open;
- created: 2026-05-20;
- last updated: 2026-05-20;
- SHA-256 of the downloaded source: `f13ae00b886c5e72131bc5d5b138a371148d1faa6899a119a1dacb65a555e7dc`.

The matrix intentionally implements the pinned text and does not incorporate later draft
edits. The proposal remains Open, so a later revision requires a separate delta audit.

## 4. RouterInfo reconciliation

The matrix contains 43 unique canonical rows sourced from
`rpc.rs::router_info_keys::PROPOSAL_170_CONTRACT`:

- 37 `available`;
- 1 protocol-permitted `neutral` (`i2p.router.clockskew`);
- 5 `unavailable`.

The unavailable rows map exactly to later work:

| Row | Current source disposition | Owner | Target |
|---|---|---|---|
| `i2p.router.net.bw.transit.15s` | unavailable | M100 | available after request-independent sampling |
| `i2p.router.news` | unavailable | M101 | available after bounded real source/cache evidence |
| `i2p.router.net.error` | unavailable | M102 | available after explicit neutral v4 state and I2PControl mapping |
| `i2p.router.net.error.v6` | unavailable | M102 | available after explicit neutral v6 state and I2PControl mapping |
| `i2p.router.netdb.bannedpeers` | unavailable | M103 | available only through a real owner or proven by-design-empty semantics |

No historical 40/1/2 disposition is carried forward. Existing M026/M030/M034/M046-M056
source and closure evidence is cited rather than rewritten. M102 is the only row budgeted
for a possible non-I2PControl production path.

## 5. AddressBook SetConfig reconciliation

The matrix contains all 13 pinned keys, with no `unknown` or `accept_inert` disposition.
The current boundary is recorded accurately: every non-empty SetConfig request is rejected
before persistence.

- Eight path-valued keys require one confined I2PControl AddressBook administrative root,
  normalized paths, safe file ownership, and transactional migration under M096.
- Four behaviorally meaningful keys (`update_delay`, `proxy_port`, `proxy_host`, and
  `should_publish`) require active runtime/persistence consumption under M096.
- `theme` is explicitly administrative metadata and may round-trip without frontend or
  router coupling.

All 13 rows are owned by M096. No arbitrary absolute path, global logger reconfiguration,
or resolver-precedence redesign is authorized.

## 6. TunnelManager applicability

The canonical inventory has 70 unique Proposal 170 option keys and exactly 12 canonical
tunnel types. Every option has one explicit cell for every type: 840 cells total.
Compatibility action spellings, `List`, and internal `i2p.tunnel.*` spellings are listed
separately and do not satisfy canonical cells.

| Cell disposition | Count | Meaning |
|---|---:|---|
| `apply` | 82 | already consumed by current runtime/backend behavior |
| `planned_apply` | 293 | applicable and assigned to M097, M098, or M099 |
| `not_applicable` | 455 | explicit cell rationale records why the option does not apply |
| `blocked_primitive` | 10 | applicable `PrivKeyFile` cells blocked on M097 |

The blocked primitive is exact: a confined private-key import plus atomic handoff into the
backend-owned key store. It is not a generic request for core access. No parser, raw-config
round-trip, or compatibility alias is counted as runtime completion. Every option also
records its JSON value type and whether it is security-sensitive, secret-bearing,
path-bearing, or identity/key-affecting.

The matrix preserves the current fail-before-allocation rule for unapplied runtime options.
Streamr cells retain their separate bounded datagram/session contract, including the
16-subscriber, 60-second expiry, 1200-byte payload, 4095-byte transport-buffer, 15-second
refresh, and bounded shutdown constraints.

## 7. ClientServicesInfo and method scope

The matrix records exactly six selectors: `I2PTunnel`, `HTTPProxy`, `SOCKS`, `SAM`, `BOB`,
and `I2CP`, with the existing production ownership/evidence boundary retained.

Broader/base methods are explicitly marked `outside_proposal_170_scope`, including
`Authenticate`, base `RouterInfo` (whose Proposal 170 additions are separately inventoried),
`GetKeys`, `GetRate`, `RouterManager`, `NetworkSetting`, `AdvancedSettings`, and the shipped
`SetSubscriptions`/`SetConfig` compatibility aliases. M095 does not expand general API parity.

## 8. M096–M103 owner/path budgets

- M096: `emissary-cli/src/i2pcontrol/address_book.rs`,
  `address_book_runtime.rs`, and `domain/address_book.rs`; one durable confined config owner.
- M097: I2PControl tunnel option/domain/runtime paths only; existing Yosemite/SAM/session
  primitives; no core or dependency expansion.
- M098: I2PControl client/proxy/HTTP/filter paths only; no clearnet bypass, proxy exposure,
  or filtering regression.
- M099: I2PControl server/admission/HTTP/IRC/Streamr paths only; retain M093 security and
  keep tunnel-local denial separate from router bans.
- M100: I2PControl-local bounded sampler over the existing cumulative transit counter; no
  core edits or request-driven sampling.
- M101: I2PControl-local bounded real news source/cache; no arbitrary public-web substitute
  or core news subsystem.
- M102: candidates are limited to the existing M061-approved neutral observation paths
  (`emissary-core/src/events.rs`, `inspection.rs`, `transport/mod.rs`) plus the named
  I2PControl adapters. The matrix marks this as the only anticipated non-I2PControl change;
  M102 must narrow candidates to exact writer/owner files before code.
- M103: I2PControl source/handler/observability paths for an exhaustive audit; no ban engine
  is authorized. A future real ban owner must be exposed, or by-design-empty semantics must
  be proven and guarded.

## 9. Verification outcomes

| Check | Outcome |
|---|---|
| M095 matrix static guard | pass; 1 test |
| RouterInfo duplicate/count/disposition checks | pass; 43 unique, 37/1/5 baseline |
| SetConfig duplicate/exhaustiveness checks | pass; 13 unique |
| TunnelManager option/type cell checks | pass; 70 × 12 = 840 explicit cells, no unknown disposition |
| ClientServicesInfo/method-scope checks | pass; 6 selectors and explicit outside-scope methods |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m062_dependency_containment` | pass; 19 tests |
| `git diff --check` | pass |

The CI-equivalent check and lint jobs also passed locally: core `no_std` and `std`
checks, CLI without UI, both examples, and clippy for core, CLI, and util with all
features. The focused Rust files pass nightly rustfmt. The repository-wide CI fmt
jobs remain a pre-existing toolchain mismatch: current nightly requests broad
rewrites in untouched files across all three crates; M095 does not reformat or alter
production paths to conceal that baseline condition.

The final closure handoff updates the two pending entries with the locally executed
results before commit.

## 10. Findings and containment attestation

- High severity: none.
- Medium severity: none.
- Low severity: none introduced by M095.
- Accepted limitations: current partial RouterInfo source state, rejected non-empty SetConfig,
  unapplied runtime options, and the named M097 private-key primitive remain explicit.
- No production/dependency/lockfile path changed. No runtime behavior, security conclusion,
  or historical closure was rewritten.

All work remained internal to `eggstack/emissary`. The official proposal and other external
references were read-only evidence. No upstream issue, pull request, review, submission,
merge request, contribution artifact, maintainer contact, or external repository write was
opened, drafted, requested, or pushed.

**Disposition: closed.**
