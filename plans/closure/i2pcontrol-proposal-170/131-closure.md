# M131 closure — residual applicability and primitive-architecture re-freeze

Status: `closed as blocked`

Date: `2026-09-04`

## Authority and scope

M131 executed from the clean registered-plan head
`a02e9bbae4d04bce1ca3c892bc6ef4df99c6b894`. The production-behavior baseline
is the M130 closure head `a68094e128d2b92f0fd5b350e38512ef6b65cb6b`. This
milestone changed planning evidence, documentation and focused matrix guards
only: no runtime Rust, dependency, Cargo-feature or Yosemite change was made.

The pinned Proposal is revision `2026-05-20`, status `Open`, SHA-256
`f13ae00b886c5e72131bc5d5b138a371148d1faa6899a119a1dacb65a555e7dc`.
Read-only reference snapshots were Java I2PControl PR #6
`45bb593000408071dd376b78848fdc246dccd964`, Java I2P/I2PTunnel
`2c3fd2a9532cd86ec06cb6f2b9f3f813ca752243`, and Yosemite Y005
`59140a2277bf296928d2e8ce39a148182eeff044`.

## Delivered authority

- `131-residual-primitive-map.toml` contains exactly 96 unique starting
  cells, each with final disposition, applicability evidence, blocker,
  canonical owner, primitive cluster, path budget, security class, validation
  behavior and historical evidence.
- `095-full-support-matrix.toml` is reconciled to
  `284 apply / 88 blocked_primitive / 468 not_applicable`; its SHA-256 is
  `f038521da9cc685bd38dd502f02dcc81f53586c3a8dd83eb3ba5a8827f589f79`.
- `105-residual-option-audit.toml` and `110-completion-ledger.toml` record
  the same counts, changed-cell set, hash and closure link.
- The registered plan, implementation README, registry, completion roadmap,
  Proposal support documentation and TunnelManager documentation are closed
  and point to this authority.

## Requirement/evidence matrix

| Requirement | Result |
|---|---|
| Enumerate all 96 starting cells | Pass: artifact has 96 unique blocked starting records mechanically derived from M095. |
| Preserve cell identity | Pass: every record names canonical option and tunnel family; no grouped record substitutes for a cell. |
| Applicability correction | Pass: only eight cells changed, all from affirmative pinned/reference evidence. |
| Apply prohibition | Pass: 284 `apply` cells remain; artifact records zero promotions. |
| M095/M105/M110 reconciliation | Pass: all three ledgers carry post-M131 counts/set/hash/closure references. |
| Streamr review | Pass: generic setter ambiguities remain blocked; affirmative `DelayOpen` and `NewDest` exclusions are not applicable. |
| Profile/lifecycle semantics | Pass: streaming-window and I2P-session activity owners are distinguished from TCP handler counts. |
| MultiHoming semantics | Pass: frozen as `shouldBundleReplyInfo`/LeaseSet reply bundling, not host-interface multihoming. |
| UseSSL/SigType semantics | Pass: TLS role conflict is retained conservatively; destination signing remains blocked on full crypto/key ownership. |
| LeaseSet ownership | Pass: Y005 typed SAM fields are transport-only; blockers now begin at Emissary construction, key custody, lookup, publication and handoff. |
| Future dependency graph | Pass: artifact has ordered edges, named neutral owners, path-budget IDs and stop conditions. |
| Future-plan status audit | Pass: M114 and historical closures retain their statuses; no M132+ plan existed to unblock or update. |
| Containment/security | Pass: internal planning-only writes; no secrets, upstream writes, runtime paths or dependencies changed. |

## Final disposition

Starting authority: `284 apply / 96 blocked_primitive / 460 not_applicable`.

Final authority: `284 apply / 88 blocked_primitive / 468 not_applicable`.

The eight reclassifications are:

- `SSLProxies:socks`, `SSLProxies:socksirc`,
  `SSLProxies:connectclient`: the pinned Java parser supports SSLProxies
  only for HTTP clients, whose concrete I2PTunnel owner is HTTP SSL-outproxy
  handling;
- `JumpList:socks`, `JumpList:socksirc`,
  `JumpList:connectclient`: the parser gates HTTP address lookup to HTTP
  clients, whose concrete owner is HTTP jump-server/address-helper handling;
- `DelayOpen:streamrclient`: the parser explicitly disables it for Streamr
  and the UDP owner has no local client-socket opening event;
- `NewDest:streamrclient`: the parser excludes Streamr from persistent-key
  and resume handling, and the UDP owner has no streaming-session resume owner.

No retained cell was promoted. The four `UseSSL` cells remain
`httpclient`, `connectclient`, `httpserver` and `httpbidirserver`; their
reference type-gate conflict is recorded without weakening the conservative
blocked classification.

## Primitive clusters and readiness

HTTP outproxy applicability is closed by reference gates, while the HTTP cells
remain blocked on their actual TLS-outproxy/address-helper owners. Presentation
TLS, destination signing, outproxy-provider integration, streaming profile,
session reduction/idle-close/resume, local source address, LeaseSet bundling and
LeaseSet crypto/lookup/auth remain `not_ready`. Y005 typed fields do not
provide Emissary LeaseSet construction, key generation/storage, lookup/decrypt
policy, publication, authorization or session handoff.

Each retained cluster has a named path-budget ID in the artifact. A future plan
must first authorize a neutral owner, exact observable effect, security model,
deterministic tests, failure/restart behavior and any dependency change. Until
then, unsupported values remain fail-before-allocation.

No M132 handoff is dependency-ready: applicability corrections are not
implementation readiness, and no retained cluster has the complete owner,
effect, security/interoperability evidence and dependency authorization required
by the planning process. M114 remains historically closed as blocked. No future
plan status required a change.

## Invariants, failure and recovery

M131 preserves fail-before-allocation for unsupported values, no direct-clearnet
fallback, explicit I2P-routed outproxy boundaries, literal-loopback server
confinement, secret/key/path redaction, shared-session compatibility, generation
isolation, cancellation, restart atomicity and bounded Streamr limits. No lock,
task, timer or queue was introduced.

If the Proposal revision, reference head, production baseline or matrix hash
changes, this closure must be reopened and the 96-cell inventory regenerated.
If a future implementation cannot prove its exact effect or fails security or
interoperability tests, the cell remains blocked and last-known-good state is
retained. A concurrent successor requires a new registry entry and dependency
audit.

## Verification

Passed:

```text
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m095_full_support_matrix --test m105_residual_option_audit --no-fail-fast
```

Result: `3 passed`.

Also passed: the M061/M062 containment suite (`30 passed`), both requested
feature/no-feature `cargo check` commands, the workspace `cargo check`, the
I2PControl all-target clippy command (`No issues found`), and `git diff --check`.
The modified Rust test files are individually rustfmt-clean under stable
rustfmt; stable emits the expected warnings that this repository's configured
nightly-only options are unavailable.

The remaining required command was:

```text
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment --test m062_dependency_containment --no-fail-fast
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo check -p emissary-cli --no-default-features
cargo check
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
git diff --check
cargo fmt --all -- --check
```

`cargo fmt --all -- --check` is an evidence check only; pre-existing rustfmt
drift was reported across unrelated production files (including existing
`emissary-cli/src/i2pcontrol/**` and `emissary-util/**` files). It was not
normalized, so the full-repository command exits nonzero for that pre-existing
drift; no M131 production source was changed.

M131 is formally closed as a truthful residual architecture re-freeze. The
Proposal-170 status remains partial. External Proposal, Java, I2P, Yosemite and
upstream Emissary sources were read-only evidence.
