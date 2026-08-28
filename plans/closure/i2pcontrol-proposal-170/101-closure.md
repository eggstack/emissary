# M101 Closure — RouterInfo News Source Completion

Status: closed

Source implementation plan:

- `plans/implementation/i2pcontrol-proposal-170/101-routerinfo-news-source-completion.md`

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md`

Implementation commit: the internal commit containing this record.
Closure commit: the internal commit containing this record.
Review date: 2026-08-28.

## 1. Disposition

M101 closes as implemented. `i2p.router.news` now has an explicit
I2PControl-owned source which fetches the pinned I2P router news feed through
the configured local HTTP proxy, authenticates the SU3 envelope, validates the
Atom/I2P metadata and XHTML content, renders the reference HTML shape, and
publishes complete generations to a bounded in-memory snapshot. No
`emissary-core` path, dependency, lockfile, workflow, frontend, or router-wide
news subsystem was added.

The authoritative matrix is
`plans/implementation/i2pcontrol-proposal-170/095-full-support-matrix.toml`.
Its current RouterInfo disposition is 43 rows: 39 available, 1
protocol-permitted neutral, and 3 unavailable.

## 2. Source and format evidence

M095 froze the source semantics against the pinned Proposal 170 revision and
read-only reference evidence:

- Proposal 170 defines `i2p.router.news` as a `String` containing all router
  news entries: <https://i2p.net/proposals/170-i2pcontrol-expansion.txt>.
- The reference I2PControl implementation invokes `NewsFeedHelper` for the
  news response: <https://github.com/i2p/i2p.plugins.i2pcontrol/pull/6>.
- The reference helper returns the rendered `newsentry`/`newscontent` HTML
  form, not the raw Atom document or a clearnet page.
- The reference `NewsFetcher` uses the canonical `news.su3` source through the
  local HTTP proxy; the adopted default URL is the fixed
  `tc73...b32.i2p/news.su3` URL in `news.rs`.
- The adopted feed is `XML_GZ` SU3 content of kind `NewsFeed`, signed with
  RSA-4096/SHA-512. The three reference news certificates are pinned as
  read-only trust anchors: Echelon, Hankhill19580, and zzz.
- The reference refresh default is 36 hours. Emissary retries failed refreshes
  after 15 minutes and treats a generation older than 7 days as stale.

The Rust implementation is independent of the GPL reference implementation;
only its documented wire/source semantics were adopted.

## 3. Requirement/evidence matrix

| Requirement | Evidence | Result |
|---|---|---|
| Real adopted source | `RouterNewsSource::start` uses only the fixed I2P URL and the configured local HTTP proxy. | pass |
| Authenticity | `Su3::parse_news` requires `NewsFeed` + `XML_GZ` + RSA-4096/SHA-512, verifies the signed message against the three pinned certificates, and gzip-decompresses only bounded content. | pass |
| Returned-string semantics | `render_news` independently parses the Atom/I2P feed and emits the reference `newsentry`, date, author, link, and `newscontent` HTML shape. | pass |
| Input bounds | 2 MiB compressed body, 4 MiB decompressed XML, 128 entries, 32 XML nesting depth, 4096 XML nodes, 64 KiB fields/content, and 1 MiB rendered output. | pass |
| Active-content safety | XML declarations/DOCTYPE-like sections, scripts, event/style/src attributes, unsafe `javascript:`/`data:` links, malformed XML, and invalid timestamps are rejected. | pass |
| Complete publication | Fetch, SU3 verification, decompression, XML validation, and rendering complete before one atomic snapshot replacement. | pass |
| Failure/staleness | Failed refreshes retain the last valid generation for at most 7 days; no generation and stale generation return sanitized unavailable errors. | pass |
| Cadence/retry | One background owner refreshes immediately, then every 36 hours on success or every 15 minutes after failure; RouterInfo reads never trigger fetches. | pass |
| Cancellation/restart | The source owns one abortable Tokio task; dropping the source aborts it. Restart reconstructs an empty source and cannot present unvalidated prior data. | pass |
| Feature boundary | Source creation is inside the feature-gated I2PControl startup path; feature-disabled builds do not compile or start the fetch owner. | pass |
| Containment | The only non-I2PControl production seam is the existing `emissary-util` SU3/certificate parser and exact pinned certificate assets; no core path changed. | pass |

## 4. Exact changed paths

Production and trust-anchor paths:

- `emissary-cli/src/i2pcontrol/news.rs`
- `emissary-cli/src/i2pcontrol/mod.rs`
- `emissary-cli/src/i2pcontrol/production.rs`
- `emissary-cli/src/i2pcontrol/router_info.rs`
- `emissary-cli/src/i2pcontrol/router_info_handler.rs`
- `emissary-cli/src/i2pcontrol/rpc.rs`
- `emissary-cli/src/i2pcontrol/server.rs`
- `emissary-cli/src/main.rs`
- `emissary-util/src/certificates.rs`
- `emissary-util/src/su3.rs`
- `emissary-util/assets/certificates/news/echelon_at_mail.i2p.crt`
- `emissary-util/assets/certificates/news/hankhill19580_at_gmail.com.crt`
- `emissary-util/assets/certificates/news/zzz_at_mail.i2p.crt`

Focused tests and evidence paths:

- `emissary-cli/tests/conformance_manifest.rs`
- `emissary-cli/tests/m027_literal_fixtures.rs`
- `emissary-cli/tests/m062_dependency_containment.rs`
- `emissary-cli/tests/m095_full_support_matrix.rs`
- `emissary-cli/tests/static_guards.rs`
- `plans/implementation/i2pcontrol-proposal-170/095-full-support-matrix.toml`
- `plans/implementation/i2pcontrol-proposal-170/101-routerinfo-news-source-completion.md`
- `plans/implementation/i2pcontrol-proposal-170/README.md`
- `plans/registry.md`
- `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md`
- current I2PControl support/conformance/source-map/RouterInfo docs and `AGENTS.md`
- `plans/closure/i2pcontrol-proposal-170/101-closure.md`

No new Cargo dependency or lockfile change was required.

## 5. Verification outcomes

| Command | Outcome |
|---|---|
| `cargo check -p emissary-cli --no-default-features` | pass |
| `cargo check -p emissary-cli --no-default-features --features i2pcontrol` | pass |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol` | pass after updating the former unavailable-news expectations |
| `cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings` | pass |
| focused news, SU3, RouterInfo handler, matrix, and static-guard tests | pass |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment` | pass; 7 tests |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m062_dependency_containment` | pass; 19 tests |
| `cargo test -p emissary-cli --no-default-features --test m061_containment` | pass; 0 feature-gated tests |
| `cargo test -p emissary-cli --no-default-features --test m062_dependency_containment` | pass; 0 feature-gated tests |
| `cargo test -p emissary-cli --no-default-features --test m063_feature_reachability` | unavailable test target; this checkout has no M063 test binary |
| `git diff --check` | pass |

`cargo fmt --all -- --check` remains qualified by the repository's documented
nightly/stable formatter drift in untouched files. Changed Rust files were
formatted with the available formatter, and formatter-only changes outside
the M101 budget were removed.

## 6. Future-plan disposition

M101 does not unblock a new dependency branch. M102 and M103 remain ready and
independent. M098 and M099 remain blocked on M097. M104 remains blocked on
M097-M103 because it still requires the remaining capability milestones and
integrated live interoperability/reclosure evidence. The registry and roadmap
were updated accordingly; no future plan was advanced incorrectly.

## 7. Unresolved findings and limitations

- The canonical `.i2p` feed could not be live-fetched in this local
  verification environment because no running I2P HTTP proxy was available.
  Deterministic format/rendering, trust-anchor loading, failure, staleness,
  and containment behavior are covered locally; live feed interoperability
  remains part of M104.
- The requested M063 test target is absent from this checkout; the existing
  M062 semantic feature/dependency guard and feature-disabled builds passed.
- No high- or medium-severity correctness, security, privacy, or containment
  finding remains from this milestone.

## 8. Internal-only attestation

All repository writes, commits, and the push are limited to the internal
`eggstack/emissary` repository. The official Proposal 170 and I2P reference
material were read-only evidence only. No upstream issue, pull request,
review, submission, merge, contribution artifact, maintainer contact, or
external repository write was created.

**Disposition: closed.**
