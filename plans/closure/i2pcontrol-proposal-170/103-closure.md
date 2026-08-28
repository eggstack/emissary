# M103 Closure — RouterInfo Banned-Peer Semantic Completion

Status: closed

Source implementation plan:

- `plans/implementation/i2pcontrol-proposal-170/103-routerinfo-banned-peer-semantic-completion.md`

Source subsystem roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md`

Repository baseline reviewed: `a58c7af69d68027a370296322fec1fbd9185036e` (pre-M103
implementation baseline)

Implementation commits:

- implementation commit containing the M103 code, matrix, tests, and planning updates
  (recorded by the surrounding git history)

Review date: 2026-08-28.

## 1. Executive finding

M103 closes on completion path B. The exhaustive source audit found no
Emissary-owned router-wide peer-ban state or writer. The existing peer profile
failure counters, SAM incoming blacklist, tunnel-local admission/rate denial
tables, HTTP/server `TotalBanTime`, transport retry/backoff state, and received
`TerminationReason::Banned` events are separate ownership domains and are not
the Proposal 170 NetDB banned-peer concept.

The production I2PControl adapter now has an explicit
`BannedPeerSource::ByDesignEmpty` capability marker. Its snapshot is therefore
authoritative and empty, and the canonical `i2p.router.netdb.bannedpeers`
selector serializes the exact Proposal 170 map type as `{}`. No ban engine,
peer scoring, enforcement, persistence, or ban-management API was introduced.

The current RouterInfo matrix is 43 total: 42 available, 1
protocol-permitted neutral, and 0 unavailable. This is still partial Proposal
170 support because M097 remains blocked and live full-contract
interoperability/reclosure remains M104 scope.

The pinned [Proposal 170 revision](https://i2p.net/en/proposals/170-i2pcontrol-expansion/)
defines `i2p.router.netdb.bannedpeers` as
`Map<String, Map<String, Object>>`. The read-only
[reference implementation discussion](https://github.com/i2p/i2p.plugins.i2pcontrol/pull/6)
was consulted for the stable-map approach; no upstream or third-party write or
contact was made.

## 2. Requirement-to-evidence matrix

| Requirement | Evidence | Result | Notes |
|---|---|---|---|
| M095 hard gate completed before implementation | M095 matrix, M051 evidence, source audit, and this record | pass | Historical M051/M095 closures remain unchanged. |
| Exhaustive audit distinguishes ban semantics | `emissary-core` profile/SAM/transport sources and `emissary-cli` admission/tunnel sources | pass | No router-wide local ban writer or owner exists. |
| Exact canonical selector and map type | `rpc::router_info_keys::PROPOSAL_170_CONTRACT`; Proposal 170 revision; literal manifest tests | pass | Key is `i2p.router.netdb.bannedpeers`; type is map-of-maps. |
| Stable map member shape | `serialize_banned_peers`; handler unit test | pass | Non-empty entries use peer ID keys and the adopted stable `reason` and `expiresAt` members. |
| Empty result is authoritative, not an unowned fallback | `BannedPeerSource::ByDesignEmpty`; production adapter; static marker test | pass | The explicit enum/source classification is the authority. |
| Future real ban owners cannot silently retain the empty proof | exhaustive `BannedPeerSource` match plus static production-source guard | pass | A new source variant requires an explicit `snapshot` decision and test update. |
| Canonical direct request behavior | RouterInfo handler direct-map tests and conformance suites | pass | Only requested keys are emitted; source is queried once for the relevant request. |
| Compatibility behavior is preserved | Existing legacy `i2p.router.peers.banned` serializer and handler tests | pass | The legacy array form was not redefined. |
| Deterministic and bounded output | `BTreeMap`, 10,000-entry limit, serialized response-size check | pass | Duplicate IDs and oversized results fail closed. |
| No mutation, restart, or persistence behavior | production adapter/static read-only tests and source review | pass | Path B has no runtime task, state, file, or migration. |
| Tunnel-local denial/backoff is excluded | source audit and explicit non-goal/invariant documentation | pass | No `TotalBanTime`, admission history, or transport backoff is mapped. |
| Containment budget remains valid | M061 and M062 suites; exact planning-path allowlist | pass | No core source or dependency change was added. |

## 3. Production implementation evidence

The implementation is confined to the I2PControl layer and its existing
composition seams:

- `emissary-cli/src/i2pcontrol/router_info.rs` defines the explicit
  `ByDesignEmpty` source classification and exhaustive empty snapshot.
- `emissary-cli/src/i2pcontrol/production.rs` wires that classification into
  the production `RouterInfoControl` adapter.
- `emissary-cli/src/i2pcontrol/router_info_handler.rs` adds the canonical
  Proposal 170 map serializer, deterministic ordering, duplicate detection,
  entry/byte bounds, and direct-request coverage. The existing legacy banned
  peer array path remains intact.
- `emissary-cli/src/i2pcontrol/rpc.rs` marks the canonical row available under
  the `router-ban-empty-marker` owner and retains the exact map-of-maps type.
- The M095 matrix, source map, support/conformance documentation, static guards,
  production adapter tests, and literal fixtures all record the same result.

No `emissary-core/**` path, dependency, lockfile, router algorithm, transport
decision, or persistent schema changed.

## 4. Verification executed

### Commands run

The feature-on commands used an isolated temporary Cargo home and target to
avoid an unrelated workspace process holding the shared package-cache lock;
the registry was reused read-only and Cargo was offline.

```bash
CARGO_HOME=/tmp/emissary-m103-cargo-home CARGO_NET_OFFLINE=true CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=target-m103 cargo test -p emissary-cli --no-default-features --features i2pcontrol --tests --no-fail-fast
CARGO_HOME=/tmp/emissary-m103-cargo-home CARGO_NET_OFFLINE=true CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=target-m103 cargo test -p emissary-cli --no-default-features --features i2pcontrol --lib --no-fail-fast
CARGO_HOME=/tmp/emissary-m103-cargo-home CARGO_NET_OFFLINE=true CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=target-m103 cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m062_dependency_containment --no-fail-fast
CARGO_HOME=/tmp/emissary-m103-cargo-home CARGO_NET_OFFLINE=true CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=target-m103 cargo test -p emissary-cli --no-default-features --features i2pcontrol --test static_guards_m007 --no-fail-fast
CARGO_HOME=/tmp/emissary-m103-cargo-home CARGO_NET_OFFLINE=true CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=target-m103 cargo test -p emissary-cli --no-default-features --no-fail-fast
CARGO_HOME=/tmp/emissary-m103-cargo-home CARGO_NET_OFFLINE=true CARGO_BUILD_JOBS=1 CARGO_TARGET_DIR=target-m103 cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
cargo fmt --all -- --check
git diff --check
```

The focused source/production/conformance suites were also run with the
feature enabled, including `router_info_truthfulness`, `production_adapter`,
`conformance_manifest`, `m027_literal_fixtures`, `m095_full_support_matrix`,
`m061_containment`, `m062_dependency_containment`, and `static_guards`.

### Results

| Command/result | Outcome |
|---|---|
| Full feature-on `--tests` run | pass; all listed unit and integration targets passed, including 631 library tests, live runtime, 36 RouterInfo truthfulness tests, 23 production adapter tests, 42 static guards, 18 M007 guards, and 19 M062 guards |
| Feature-on library run | pass; 631 passed, 0 failed |
| Feature-off/default behavior | pass; 56 passed, 0 failed; feature-owned integration targets correctly contained 0 tests |
| M062 dependency containment | pass; 19 passed, 0 failed |
| M007/static guards | pass; 18 passed, 0 failed |
| Feature-on clippy with `-D warnings` | pass; no warnings |
| `git diff --check` | pass |
| `cargo fmt --all -- --check` | qualified tooling failure; stable rustfmt rejects the repository's nightly-only options and reports unrelated pre-existing drift. No formatter churn was accepted. |
| M063 named target | not run; this checkout has no `m063_feature_reachability` Cargo test target. Existing M063 rules are exercised through available containment and feature-disabled checks. |

An initial feature-on run found and corrected one stale 41/1/1 assertion and
one static-guard collision caused by the word `capability` in an owner label.
The complete rerun passed after those corrections. The temporary generated
`target-m103/` directory was removed after verification.

## 5. Invariant review

1. `bannedpeers` is sourced only from the explicit router-wide semantic source
   classification.
2. Tunnel-local temporary denial and rate limiting cannot enter this result.
3. Empty output is backed by an explicit by-design-empty source marker.
4. No router ban or enforcement algorithm was added.
5. I2PControl owns the JSON map and member names.
6. No lower-layer inspection seam was needed.
7. Non-empty DTO serialization is deterministic and bounded, even though the
   production by-design-empty source currently yields no entries.
8. No private keys, profiles, sockets, or secret peer material is exposed.
9. Reads do not mutate router or control-plane state.
10. No upstream interaction occurred.

## 6. Failure and recovery review

Path B has no background task, mutable ban cache, persistence, restart repair,
or cancellation race. A process restart remains empty because there is no
router-wide ban facility to restore. The production source cannot fail by
request history because it returns an owned empty snapshot from an explicit
classification.

The generic DTO path remains truthful for future/fake sources: an unknown or
unowned source is still unavailable rather than silently converted to empty.
For a substantive snapshot, duplicate peer IDs and entry/response-size
overflow fail closed. JSON serialization occurs after the bounded owned
snapshot is returned, so no lock crosses serialization or an await point.
Malformed, unauthorized, mixed-mode, and unknown-selector requests continue
through the existing parser, authentication, selector validation, and
sanitized JSON-RPC error paths.

## 7. Migration and compatibility review

No database, configuration, lockfile, or wire migration is required. The
canonical field moves from truthful unavailable status to a truthful empty
map. The legacy `i2p.router.peers.banned` array behavior is unchanged. The
empty map does not imply ban management or support for adding/removing bans.

If a future release introduces real router-wide ban behavior, the explicit
source classification must migrate to a bounded Path A owner before that
behavior is exposed; the current by-design-empty proof cannot remain silently
authoritative.

## 8. Security review

Existing I2PControl authentication and authorization are unchanged. The
result contains no secrets and introduces no network, filesystem, privilege,
destination, or routing behavior. Entry and serialized-response bounds prevent
unbounded map construction or output. No denial, retry, or profile data is
reinterpreted as a ban, avoiding accidental administrative disclosure and
security-policy changes.

## 9. Documentation and operations

Updated evidence and control surfaces:

- `AGENTS.md` current matrix note;
- `docs/i2pcontrol/README.md`;
- `docs/i2pcontrol/proposal-170-conformance.md`;
- `docs/i2pcontrol/proposal-170-support.md`;
- `docs/i2pcontrol/router-info-source-map.md`;
- `docs/i2pcontrol/router-info.md`;
- `plans/implementation/i2pcontrol-proposal-170/095-full-support-matrix.toml`;
- `plans/implementation/i2pcontrol-proposal-170/README.md`;
- `plans/registry.md`;
- `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md`;
- this closure record.

The static marker guard, source-count assertions, literal fixtures, production
adapter test, and M061/M062 containment rules provide the operational and
maintenance guardrail. Operators should read `{}` as Emissary's authoritative
empty router-wide banned set at this revision, not as a ban-management
interface.

## 10. Unresolved findings

| Severity | Finding | Impact | Required action |
|---|---|---|---|
| low | Stable rustfmt cannot satisfy the repository's nightly-only configuration in this environment. | Formatting gate is not independently green; the failure is unrelated to M103 semantics. | Resolve in a dedicated toolchain/formatting change if desired. |
| low | The named `m063_feature_reachability` test target is absent from this checkout. | That exact historical command cannot be executed verbatim. | Preserve the limitation; do not create unrelated M103 scope. |

No critical, high, or medium correctness, security, compatibility, containment,
or source-truthfulness finding remains.

## 11. Roadmap disposition

M103 is closed. Its completion does not unblock a future milestone:

- M098 and M099 remain blocked on M097.
- M104 remains blocked on M097–M103 because M097 is still blocked, even though
  M103 is now closed.
- No future plan status is advanced by this closure.

M104 must still perform integrated live/reference-router interoperability and
final matrix, security, and containment reclosure. Full Proposal 170 support is
not claimed by M103.

## 12. Registry updates

The implementation state was reconciled in the same work:

- M103 is marked closed and linked to this record in the implementation index,
  registry, and full-support roadmap.
- M095 current counts and the canonical source map now read 42 available / 1
  neutral / 0 unavailable, with M103 owning the empty-map row.
- The registry dependency graph marks M097 blocked, M098/M099 blocked on M097,
  M103 closed, and M104 still blocked on M097–M103.
- The M062 exact planning-path allowlist includes the M103 closure record (and
  the pre-existing M102 closure path required by the same baseline guard).
- Historical M051, M095, and M102 closure records were not rewritten.

**Disposition: closed.**
