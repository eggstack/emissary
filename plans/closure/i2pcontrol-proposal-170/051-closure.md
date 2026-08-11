# I2PControl Proposal 170 Milestone M051 — Closure Status

Status: blocked

Source implementation plan:

- `plans/implementation/i2pcontrol-proposal-170/051-routerinfo-news-and-banned-peer-semantics.md`

Source subsystem roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-roadmap.md`

Repository baseline reviewed: `2b615a0`

Implementation commit:

- `5ae0f77` — `i2pcontrol: adjudicate router news and banned peers`

Closure date: 2026-08-11

Pinned contract: [Proposal 170, `I2PControl Expansion`](https://i2p.net/en/proposals/170-i2pcontrol-expansion/),
Open, created and last updated 2026-05-20.

## 1. Executive finding

M051 is formally closed as blocked for both target fields, with an accepted
semantic disposition. The pinned proposal defines `i2p.router.news` as a
string and `i2p.router.netdb.bannedpeers` as a
`Map<String, Map<String, Object>>`, but does not define an empty value as the
meaning of an implementation without the corresponding source owner.

The read-only [Java reference change](https://github.com/i2p/i2p.plugins.i2pcontrol/pull/6)
uses a substantive `NewsFeedHelper.getEntries(...)` owner for news and
`RouterContext.banlist().getEntries(...)` for bans. Its banned-peer entry shape
is a Base64 router hash mapped to an object containing `expireOn` (long),
`cause` (String), `causeCode` (String), and `transports` (Set<String> or null).
These references establish source-backed values, not capability-empty
fallbacks. Emissary has neither owner, and its production adapter therefore
continues to return sanitized `Unavailable` errors for both fields.

No news feed, release checker, ban engine, peer-blocking policy, firewall rule,
routing penalty, or core production source was added. The truthful RouterInfo
matrix remains 40 available, 1 protocol-permitted neutral, and 2 unavailable.

## 2. Requirement-to-evidence matrix

| Requirement | Evidence | Result | Notes |
|---|---|---|---|
| Pin exact Proposal 170 wire semantics | Proposal 170 RouterInfo section; `PROPOSAL_170_CONTRACT` | pass | News is `String`; banned peers are `Map<String, Map<String, Object>>`. No empty capability state is specified. |
| Read-only reference adjudication | Java reference PR #6 and `Banlist.Entry` source | pass | News calls `NewsFeedHelper`; bans call `Banlist.getEntries`; no-source semantics are not established as empty values. |
| Exact banned-peer entry shape | Reference `RouterInfoHandler` and `Banlist.Entry` | pass | Keys and value types are recorded above; Emissary does not serialize them without an owner. |
| Audit Emissary news ownership | `ProductionRouterInfoControl::router_news`; repository-wide owner search | pass | No news owner exists; production returns `UnavailableReason` with `no router news owner`. |
| Audit Emissary ban ownership | `ProductionRouterInfoControl::banned_peers`; repository-wide owner search | pass | No canonical ban owner exists; production returns `Unavailable(PeerStats)`. |
| Do not promote empty placeholders | `direct_news_uses_p170_disposition`; `direct_banned_peers_do_not_promote_fake_or_empty_values`; production adapter tests | pass | Fake or injected values cannot bypass the direct Proposal 170 unavailable rows. |
| Distinguish unavailable from source failure | `InspectionError` source/error taxonomy and request preflight | pass | Missing owner is unavailable; no fabricated empty success or source-failure fallback is used. |
| Keep source-map and documentation parity | `docs/i2pcontrol/router-info-source-map.md`, `router-info.md`, contract manifest | pass | Both rows remain unavailable with explicit owner/semantic reasons; counts remain 40/1/2. |
| Preserve containment | `git diff --name-only`, changed-path review | pass | Production changes remain in `emissary-cli/src/i2pcontrol/**`; no core or router behavior changed. |

## 3. Production implementation evidence

M051 required no new production owner because neither field has a canonical
Emissary source. The existing adapter behavior is the implementation of the
accepted unavailable disposition:

- `router_news()` returns `UnavailableReason { group: Retained, reason: "no
  router news owner" }`;
- `banned_peers()` returns `Unavailable { group: PeerStats }`;
- canonical direct requests are rejected during Proposal 170 disposition
  validation before any source query or serializer default can run;
- historical nested RouterInfo compatibility behavior remains separate and is
  not evidence for direct Proposal 170 support.

The small handler cleanup replaces an `is_some()`/`expect()` pair with
`if let Some` to keep the existing M050 network response behavior warning-free
under the required clippy configuration. The M050 network fixture was also
corrected to use the direct canonical request envelope; the behavior and wire
values are unchanged.

## 4. Verification executed

### Commands run

```bash
rtk cargo check -p emissary-core
rtk cargo check -p emissary-cli --no-default-features
rtk cargo check -p emissary-cli --no-default-features --features i2pcontrol
rtk cargo test -p emissary-core
rtk cargo test -p emissary-cli --no-default-features
rtk cargo test -p emissary-cli --no-default-features --features i2pcontrol
rtk cargo test -p emissary-cli --no-default-features --features i2pcontrol --test production_adapter production_router_info_returns_banned_peers_unavailable -- --exact
rtk cargo test -p emissary-cli --no-default-features --features i2pcontrol --test conformance_manifest
rtk cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m027_literal_fixtures
rtk cargo test -p emissary-cli --no-default-features --features i2pcontrol --test router_info_truthfulness
rtk cargo test -p emissary-cli --no-default-features --features i2pcontrol --test static_guards
rtk cargo clippy -p emissary-core --all-targets -- -D warnings
rtk cargo clippy -p emissary-cli --no-default-features --all-targets -- -D warnings
rtk cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
rtk git diff --check
rtk cargo fmt --all -- --check
```

### Results

- Core checks passed; core tests passed with 1,062 passed and 2 ignored.
- No-feature CLI checks and tests passed with 56 tests.
- Feature-enabled CLI check passed; the final full suite passed with 1,369
  tests. The first run exposed the existing M050 network fixture's incorrect
  nested request envelope; the fixture was corrected to direct mode and the
  full suite was rerun successfully.
- The focused production banned-peer test passed. Conformance, literal,
  truthfulness, and static-guard suites passed with 58, 7, 34, and 38 tests.
- Core, no-feature CLI, and feature-enabled CLI clippy passed with
  `-D warnings`. The feature clippy run was repeated after the equivalent
  `if let` cleanup described above.
- `git diff --check` passed.
- `cargo fmt --all -- --check` remains qualified by the repository's known
  stable/nightly rustfmt mismatch and reports unrelated pre-existing churn;
  no formatter churn was retained.

## 5. Invariant review

1. No news or ban subsystem was added solely for I2PControl.
2. Profiles, dial failures, reachability, firewall state, and transport
   errors are not reinterpreted as bans.
3. Empty string/map values are not emitted without an authoritative owner and
   contract-proven current-state semantics.
4. Proposal 170 policy and source disposition remain in I2PControl; no core
   production path changed.
5. Direct presence semantics, authentication, bounds, sanitized errors, and
   no-feature isolation remain unchanged.
6. The canonical request fails before assembly when either unavailable field is
   requested, so no partial or fabricated result can escape.

## 6. Failure, restart, and contention review

Unavailable fields fail deterministically before source acquisition. There is
no lock, task, persistence, network fetch, polling loop, or restart state in
M051. A future real owner would need to use the existing sanitized source
failure taxonomy; M051 does not turn source failure into an empty success.
The direct fake-value regression proves that test/configuration injection does
not bypass the unavailable contract rows.

## 7. Compatibility and migration review

No schema, migration, configuration, method inventory, selector spelling, or
legacy compatibility behavior changed. Direct Proposal 170 news and banned
peer requests retain their established unavailable behavior. Nested historical
news behavior remains separate and does not promote the canonical row.

## 8. Security review

The two selectors remain behind authenticated, bounded, read-only RouterInfo
dispatch. No sockets, keys, mutable router state, ban authority, transport
handles, or message payloads cross the control-plane boundary. Retaining
unavailability avoids exposing an invented or incomplete peer-control view.

## 9. Documentation and operations

Updated the source map, RouterInfo behavior documentation, implementation
handoff, active registry, subsystem roadmap, and M051 plan status. The exact
reference-backed adjudication is recorded here. No operator migration or
runtime action is required; future support requires a separately authorized
canonical owner plan.

## 10. Unresolved findings

| Severity | Finding | Impact | Required action |
|---|---|---|---|
| low | Stable rustfmt cannot satisfy the repository's nightly-only formatting configuration without unrelated churn | Formatting evidence remains qualified | Preserve the existing repository qualification; use the documented formatter/toolchain when available. |
| medium | Router news and banned-peer fields have no Emissary source owner | The two fields remain unavailable and RouterInfo source completion cannot be claimed | Retain the semantic disposition; create a new bounded owner-specific plan only if the router later gains those capabilities. |

The medium item is the intentional M051 stop condition, not an unreviewed
implementation defect. No high or critical finding remains.

## 11. Roadmap and future-plan disposition

M051 is formally closed as blocked with an accepted semantic limitation. The
RouterInfo dimension remains incomplete at 40 available, 1 protocol-permitted
neutral, and 2 unavailable; overall Proposal 170 status remains partial.

M052 is newly unblocked and moved from `blocked` to `ready`. Its plan
explicitly accepts M045–M051 as either closed or blocked with accepted semantic
dispositions, and requires it to report RouterInfo completion as incomplete
when a field remains semantically unavailable. No other future plan became
ready.

## 12. Registry updates

- M051 implementation plan status is `blocked`.
- This record is the accepted M051 closure record with blocked disposition.
- M052 implementation plan status is `ready` and is the sole dependency-ready
  handoff.
- The registry, subsystem roadmap, and implementation README record the
  M051/M052 transition and retain the 40/1/2 source counts.

## 13. Internal-only attestation

The official Proposal 170 page, the Java reference PR, and the Java
`Banlist.Entry` source were accessed read-only on 2026-08-11. No upstream
repository or maintainer channel was mutated; no upstream issue, review,
merge, adoption request, submission, or contribution artifact was created or
prepared. All repository changes are confined to the authorized internal
`eggstack/emissary` repository.

Disposition: **M051 closed as blocked; M052 ready; RouterInfo source completion remains incomplete.**
