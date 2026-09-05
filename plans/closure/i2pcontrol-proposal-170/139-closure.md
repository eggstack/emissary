# M139 Closure — Post-Lifecycle Integrated Requalification and Authority Rebase

Status: **closed as complete**

Date: `2026-09-05`

Source implementation plan:

- `plans/implementation/i2pcontrol-proposal-170/139-post-lifecycle-integrated-requalification-and-authority-rebase.md`

Source roadmaps:

- `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md`
- `plans/subsystems/i2pcontrol-proposal-170-post-m114-corrective-roadmap.md`
- `plans/subsystems/i2pcontrol-proposal-170-session-lifecycle-completion-roadmap.md`

## 1. Closure decision

M139 is closed as complete. It is a qualification, invariant, and planning
correction milestone; it makes no Proposal-170 capability promotion. The
current implemented-subset runtime/security authority is now this closure.
M130 remains immutable historical evidence for the M130 implementation head and
is superseded only as the current-head authority.

The final current matrix remains exactly:

- `325 apply`;
- `47 blocked_primitive`;
- `468 not_applicable`;
- `840` total cells.

No high- or medium-severity correctness or security defect was found in the
implemented subset. No corrective implementation plan is required by M139.

## 2. Reviewed commits and scope

The lifecycle implementation line reviewed by M139 is:

- `e4f217cb1459e26bf011da46b67fc2c83cd192b5` — lifecycle implementation head.

The qualification and authority-rebase changes are in:

- `e39b8dd8512e1bee0babfc6a978389c452cc8fee` — qualification guards, matrix
  authority, and active-document reconciliation.

The M139 starting point was a clean worktree at:

- `28cfa7fdecbd3761e7d7f370fce05aabd10b4bd8`.

The qualification commit changes only tests, machine-readable planning data,
planning records, documentation, and authority metadata. It changes no
production Rust source, Cargo manifest, lockfile, Yosemite source/revision,
frontend, or hosted-CI path. The changed qualification paths were:

- `AGENTS.md`;
- `docs/i2pcontrol/README.md`, `docs/i2pcontrol/proposal-170-support.md`,
  `docs/i2pcontrol/tunnel-manager.md`;
- `emissary-cli/tests/m060_containment.rs`,
  `emissary-cli/tests/m062_dependency_containment.rs`,
  `emissary-cli/tests/m126_requalification.rs`,
  `emissary-cli/tests/m127_token_lifetime.rs`,
  `emissary-cli/tests/m128_jsonrpc_batch.rs`,
  `emissary-cli/tests/m129_nonloopback_tls.rs`,
  `emissary-cli/tests/m130_post_corrective_requalification.rs`, and
  `emissary-cli/tests/m139_post_lifecycle_requalification.rs`;
- `plans/implementation/i2pcontrol-proposal-170/095-full-support-matrix.toml`;
- the M139 implementation plan and
  `plans/implementation/i2pcontrol-proposal-170/README.md`;
- `plans/registry.md`;
- all three source roadmaps listed above.

## 3. Baseline drift ledger and disposition

Before edits, the repository was checked at the clean M139 starting point and
the known guards were run individually. The failures were classified as
historical/current-head authority drift, not silently ignored production
regressions:

| Evidence | Baseline result | Disposition |
|---|---|---|
| M060 containment | Failed on later accepted core paths | Re-scoped the historical assertion to the M060 implementation range, from the pinned upstream baseline through `6085eca`; later lifecycle seams are not charged to M060. |
| M061 containment | Passed | Retained the exact manifest-backed current non-policy boundary. |
| M062 dependency containment | Passed after its exact M139 path authorization was added | Retained dependency ownership and authorized only the explicit M139 planning/test paths. |
| M095 full matrix | Passed | Kept the declared and recomputed `325/47/468` authority. |
| M105 residual audit | Passed | Kept the residual inventory unchanged. |
| M126 requalification | Failed on superseded `284/96/460` assertions | Rebased only current-head count/document checks to `325/47/468`; historical milestone evidence remains. |
| M127 token lifetime | Failed on an old production-range assertion and old counts | Scoped the production-range assertion to M127's historical range and rebased current matrix checks. |
| M128 JSON-RPC batch | Failed on the analogous historical range and old counts | Scoped the range to M128's historical implementation and rebased current matrix checks. |
| M129 non-loopback TLS | Failed on the analogous historical range and old counts | Scoped the range to M129's historical implementation and rebased current matrix checks. |
| M130 integrated requalification | Failed on old counts, active-doc wording, and its old baseline path prohibition | Preserved M130's historical range assertion and made M139 the separate current-head guard. |
| Initial M139 authority guard | Found one active M125 document sentence presenting the old matrix as current | Relabeled that statement as historical; no capability or count changed. |

The stable-toolchain `cargo fmt --all -- --check` failure is unrelated
repository-wide formatting drift: it reports 28 existing `Diff in` sites. No
unrelated formatting was normalized. Full CLI clippy likewise has one known
pre-existing stable-toolchain lint at
`emissary-cli/src/i2pcontrol/backends/filters/proxy.rs:60:35`
(`chunks_exact_to_as_chunks`) under `-D warnings`; that production file was
untouched. The M139-changed test files pass targeted clippy with that unrelated
lint explicitly isolated.

## 4. Requirement evidence

### Matrix and residual truthfulness

`m139_post_lifecycle_requalification.rs` mechanically parses
`095-full-support-matrix.toml`, verifies the Proposal revision/status and
reviewed production head, enumerates all 12 tunnel types, recomputes all 840
cells, and compares both the declared and documented counts. The final matrix
SHA-256 is:

`c06236cb3eb014cbe7f04b3b065155da85720e66c6c7a106bdb7624405d5ff9d`

The pinned Proposal-170 source SHA remains:

`f13ae00b886c5e72131bc5d5b138a371148d1faa6899a119a1dacb65a555e7dc`

The exact 47 blocked cells are unchanged and are mechanically checked:

| Option | Families | Count |
|---|---|---:|
| `ConnectDelay` | `streamrclient` | 1 |
| `EncryptLeaseSet` | `server`, `httpserver`, `httpbidirserver`, `ircserver`, `streamrserver` | 5 |
| `JumpList` | `httpclient` | 1 |
| `LeaseSetClientAuths` | `server`, `httpserver`, `httpbidirserver`, `ircserver`, `streamrserver` | 5 |
| `MultiHoming` | `httpserver`, `httpbidirserver` | 2 |
| `OptionalLookup` | `server`, `httpserver`, `httpbidirserver`, `ircserver`, `streamrserver` | 5 |
| `Profile` | `client`, `httpclient`, `ircclient`, `socks`, `socksirc`, `connectclient`, `streamrclient` | 7 |
| `SSLProxies` | `httpclient` | 1 |
| `SigType` | `client`, `httpclient`, `ircclient`, `socks`, `socksirc`, `connectclient`, `server`, `httpserver`, `httpbidirserver`, `ircserver` | 10 |
| `UniqueLocalAddressPerClient` | `httpserver`, `httpbidirserver` | 2 |
| `UseOutproxyPlugin` | `httpclient`, `socks`, `socksirc`, `connectclient` | 4 |
| `UseSSL` | `httpclient`, `connectclient`, `httpserver`, `httpbidirserver` | 4 |

Promotions remain exactly the lifecycle promotions already justified by their
closures: M134 `NewDest` (6), M136 `Reduce*` (21), and M137 `Close*` (14).
M139 contributes zero promotions and zero demotions.

### Containment and dependency ownership

The M061 guard passes against its pinned upstream baseline and exact path
manifest: 37 non-policy changed source paths are authorized, with no broad
prefix allowance. Its complete current source comparison contains 101 paths,
including policy-root history; the non-policy set is exactly the 37 manifest
entries.

The M062 guard passes with the explicit M139 path set. The Yosemite I2PControl
fork remains read-only evidence at revision
`59140a2277bf296928d2e8ce39a148182eeff044`, with optional alias
`yosemite-i2pcontrol`; the ordinary Yosemite registry entry remains present and
the subtle workspace remains absent. M139 changes no dependency or upstream
pin.

### Security and runtime requalification

The M127, M128, and M129 guards continue to pass at the current head. The full
feature-enabled CLI package tests also pass, retaining evidence for:

- finite token lifetime, bounded token/throttle state, and reachable
  `TOKEN_EXPIRED`;
- bounded request bodies, connection tasks, and JSON-RPC batches, with
  per-element authentication and notification suppression;
- TLS-only dispatch, loopback-managed TLS, explicit non-loopback TLS material,
  validation before listener/filesystem effects, and no plaintext fallback;
- secret/token/password/private-destination redaction;
- local-target confinement and no direct-clearnet fallback.

The lifecycle composition guard exercises deterministic, no-wall-clock paths
for:

- M137 reduction followed by later close;
- M134 fake-SAM proven resume, including one rotation and committed-state
  restart reuse without replay;
- manual stop, restart, and failure paths not being labeled `IdlePolicy` and
  not rotating `NewDest`.

The guard also checks that the active docs and all authority indexes agree on
M139, `partial`, and `325/47/468`, while retaining M130 as historical and not
claiming full Proposal support.

### Broad verification

All required behavioral and static suites completed as follows:

| Command group | Result |
|---|---|
| Core check, tests, and all-target clippy | pass; 1,136 passed, 2 ignored; clippy clean |
| Workspace and feature-disabled CLI checks | pass |
| I2PControl CLI library tests | pass; 805 tests |
| Requested live/adversarial/persistence/client-service/router-info suites | pass; 221 tests across 9 suites |
| Full feature-enabled CLI package tests | pass; 2,149 tests across 32 suites |
| M139 static aggregate | pass; 90 tests across 11 suites |
| M062 plus M139 closure guards | pass; 26 tests |
| `git diff --check` | pass |
| `cargo fmt --all -- --check` | known pre-existing stable/nightly drift; 28 diff sites, recorded above |
| Full CLI clippy with `-D warnings` | known pre-existing proxy lint only, recorded above |

No test was skipped because of missing runtime infrastructure. The known
formatting and proxy-lint conditions do not originate in M139 and do not mask a
M139 failure.

## 5. Documentation and authority reconciliation

The following active records now agree that M139 is closed, is the current
implemented-subset runtime/security qualification authority, and that the
current matrix is `325/47/468` with support still partial:

- `AGENTS.md`;
- `plans/registry.md`;
- `plans/implementation/i2pcontrol-proposal-170/README.md`;
- both full-support and post-M114 corrective roadmaps;
- the session-lifecycle roadmap;
- `docs/i2pcontrol/README.md`;
- `docs/i2pcontrol/proposal-170-support.md`;
- `docs/i2pcontrol/tunnel-manager.md`.

Historical `284/96/460`, `284/88/468`, and related pre-lifecycle counts remain
where they describe their original milestone or closure. They are no longer
presented as the active current matrix. M130's closure file was not rewritten.

No residual successor plan was registered. The remaining 47 cells are
explicitly tracked as residual capability work, and future cluster selection is
a separate planning decision rather than an automatic consequence of M139
closure.

## 6. Future-plan unblock determination

M139 does not unblock any future implementation plan:

- M131 remains the residual applicability/primitive authority and is still
  blocked for the remaining primitive clusters;
- M111, M112, and M113 retain their historical blocked dispositions;
- the lifecycle roadmap is complete through M134/M135/M136/M137, with M139 now
  providing the post-lifecycle qualification authority;
- no plan was found whose gate is satisfied by this requalification alone and
  whose status should therefore be changed.

Accordingly, no future-plan status was changed beyond closing M139 itself. The
47-cell residual ledger is not promoted to an active successor by this closure.

## 7. Unresolved conditions and handoff

| Condition | Classification | Handoff |
|---|---|---|
| 47 `blocked_primitive` cells | Intentional Proposal capability residual | Future planning may select a separate cluster; no M139 defect. |
| Stable/nightly rustfmt mismatch | Existing repository/toolchain drift | Address independently; no M139 formatting rewrite. |
| Proxy `chunks_exact_to_as_chunks` lint | Existing pre-M139 production lint | Address independently; M139 did not touch the file. |

There are no unresolved M139 correctness, security, containment, dependency,
or migration findings. M139 changes no public wire/schema contract, persisted
state, or dependency, so no migration or rollback operation is required.

M139 is therefore formally closed as complete, with the closure record serving
as the current authority for the implemented Proposal-170 subset.
