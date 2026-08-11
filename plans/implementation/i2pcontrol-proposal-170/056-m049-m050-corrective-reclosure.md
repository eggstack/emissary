# M056 — M049/M050 Corrective Integration Reclosure

Status: closed

Planning baseline: `970252c` — merged M053–M052 implementation/reclosure head

Source roadmap: `plans/subsystems/i2pcontrol-proposal-170-roadmap.md`

Corrects the affected final-closure claims in:

- `plans/closure/i2pcontrol-proposal-170/049-closure.md`;
- `plans/closure/i2pcontrol-proposal-170/050-closure.md`;
- `plans/closure/i2pcontrol-proposal-170/052-closure.md`.

Milestone class: corrective integration + containment reclosure

Hard dependencies:

- M054 accepted closure;
- M055 accepted closure.

Pinned authority: I2P Proposal 170 `I2PControl Expansion`, Open, revision `2026-05-20`.

## 1. Objective

Independently reclose only the portions of the RouterInfo source-completion sequence affected by the post-M052 review:

- `i2p.router.net.bw.transit.15s`;
- `i2p.router.net.error`;
- `i2p.router.net.error.v6`;
- the final source-count/integration claims that depended on those three rows.

M056 makes no production implementation changes. It validates the accepted M054/M055 dispositions at their final integrated head, reruns the bounded retained regression matrix, reconciles documentation/source accounting, and establishes the new truthful M052 successor disposition.

Do not reopen M053/M045, M046, M047, M048, M049's recent-success/queue/TBM fields, M050's status/testing fields, or M051's accepted news/ban limitation unless a direct new defect is demonstrated during this review.

## 2. Why a separate reclosure is required

M052 closed on the basis of 40 available / 1 protocol-permitted neutral / 2 unavailable. Post-closure review found that:

1. transit-15s availability depended on API request history rather than independent router traffic history;
2. v4/v6 network-error availability mapped missing source authority to the positive `No error` code.

Those findings invalidate the final three-row source accounting and the associated M049/M050 closure claims even though the broad test matrix passed. M054 and M055 are implementation/truthfulness corrections; M056 is the independent integration judgment required by `plans/003-planning-process.md`.

## 3. Scope and production budget

Production changes: **none**.

M056 may update only:

- `plans/**`;
- `docs/i2pcontrol/**` when required to reconcile final accepted behavior;
- tests only if a closure-only fixture is missing and no production change is needed.

If M056 discovers that production code must change to satisfy an acceptance criterion, stop and create another corrective implementation plan. Do not patch production under a closure milestone.

## 4. Required integrated source matrix

The final matrix is determined by the accepted M054 disposition:

### If M054 restores exact request-independent transit-15s

- 43 canonical additions total;
- 38 available;
- 1 protocol-permitted neutral;
- 4 unavailable:
  - `i2p.router.news`;
  - `i2p.router.netdb.bannedpeers`;
  - `i2p.router.net.error`;
  - `i2p.router.net.error.v6`.

### If M054 truthfully demotes transit-15s

- 43 canonical additions total;
- 37 available;
- 1 protocol-permitted neutral;
- 5 unavailable, adding `i2p.router.net.bw.transit.15s` to the list above.

M056 must derive the matrix from `PROPOSAL_170_CONTRACT` and exact production behavior rather than forcing either target count.

## 5. Invariants

1. Every row marked available has a real named production owner and exact regression evidence.
2. No source is considered available merely because its serializer returns a syntactically valid value.
3. Zero/false/empty/null are accepted only when the authoritative owner reports that state or the protocol explicitly permits the neutral value.
4. Transit-15s behavior is independent of prior RouterInfo reads if marked available.
5. Network error code `0` is not emitted when the implementation merely lacks an error owner.
6. M051 news/ban unavailability remains explicit and does not trigger new substantive subsystems.
7. No core/wire ownership boundary is broadened during closure.
8. Default/no-I2PControl behavior remains green.
9. No remote CI/release infrastructure or upstream interaction is introduced.

## 6. Work packages

### WP1 — Verify accepted corrective heads and changed paths

Record exact implementation and closure commits for M054 and M055. Compare each against `970252c` and its own authorized boundary.

Confirm:

- M054 did not touch tunnel/transport/router/NetDB data-plane paths outside its budget;
- M055 did not touch transport/SSU2 status/testing paths except where a separately accepted blocker required it;
- no production changes occurred under M056 itself.

### WP2 — Reproduce the two original semantic defects against historical behavior

Retain explicit evidence showing why the old closure was invalid:

- old request-driven transit sampling fails a no-prior-query or long-query-gap case;
- old network-error mapping can return `0` despite no production error owner.

This historical failing evidence must remain understandable after cleanup; do not delete all traces of the regression that motivated the corrective pass.

### WP3 — Validate transit-15s final disposition

If available after M054, prove at the integrated head:

- source history advances with zero RouterInfo transit-15s reads;
- a first later read obtains the reference-correct value after authoritative history exists;
- long gaps between API reads do not reset traffic history;
- startup/zero-traffic/reset behavior matches the accepted M054 closure;
- no request-local sampler remains authoritative.

If unavailable after M054, prove canonical requests fail deterministically and no stale sampler value is emitted.

### WP4 — Validate network-error final disposition

Prove at the integrated head:

- both direct error selectors are unavailable without partial result;
- combined requests with valid status/testing fields still fail atomically;
- status.v6 and testing v4/v6 remain operational and exact;
- no source-count fixture or documentation still labels the error rows available;
- dead error-only core scaffolding is absent when M055 closure required its removal.

### WP5 — Retained RouterInfo and live-process regression

Rerun the bounded broad RouterInfo suite covering:

- M045/M053 live known-peer source;
- M046 active peers/finite limits;
- M047 active-peer stats;
- M048 tunnel details/counts;
- M049 recent success and queues;
- M050 status/testing;
- M051 news/ban unavailable behavior;
- exact direct-presence and compatibility behavior;
- authentication/TLS production composition;
- source failure, bounds, and no-partial-result semantics.

Use the existing child-process test where it provides useful end-to-end evidence. Do not add an elaborate traffic generator or network harness solely to make transit-15s nonzero in a live child process; deterministic owner-level and production-adapter evidence from M054 is sufficient for the rolling semantic, while the child process should verify the final selector disposition/response path.

### WP6 — Reconcile planning and documentation

Update:

- `plans/registry.md`;
- subsystem roadmap;
- implementation README;
- RouterInfo source map/support/conformance docs;
- M049/M050/M052 status annotations as needed.

Retain the original closure records as historical evidence. M056's closure supersedes only the invalidated findings and final source-count claim; it must not erase prior chronology.

## 7. Failure, restart, cancellation, and contention review

M056 adds no runtime behavior. The closure must explicitly verify the accepted M054/M055 semantics:

- request cancellation cannot reset/advance transit history;
- restart behavior for transit rolling state is documented and bounded;
- unavailable network-error requests perform no source mutation or wait;
- no new locks span await/serialization/network I/O;
- observation failure cannot perturb router data-plane behavior.

## 8. Compatibility, migration, and security review

No schema, config, persistence, authentication, TLS, AddressBook, tunnel-manager, or compatibility migration is expected. Any unexpected migration or public API expansion is a blocker requiring another plan.

Re-audit that core observation types expose no keys, sockets, channels, mutable sessions/tunnels/router owners, or Proposal 170 wire terminology.

External Proposal 170/reference sources remain read-only evidence. No upstream issue, PR, review, submission, merge request, adoption request, maintainer contact, or contribution artifact is authorized.

## 9. Verification

Run at minimum:

```bash
cargo check -p emissary-core
cargo test -p emissary-core --no-fail-fast
cargo clippy -p emissary-core --all-targets -- -D warnings
cargo check -p emissary-cli --no-default-features
cargo test -p emissary-cli --no-default-features
cargo clippy -p emissary-cli --no-default-features --all-targets -- -D warnings
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test conformance_manifest --no-fail-fast
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m027_literal_fixtures --no-fail-fast
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test router_info_truthfulness --no-fail-fast
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test production_composition --no-fail-fast
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test i2pcontrol_live_runtime -- --nocapture
cargo test -p emissary-cli --no-default-features --features i2pcontrol
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
git diff --check
```

Record the existing rustfmt qualification accurately; do not retain unrelated formatting churn or add CI/verification infrastructure.

## 10. Acceptance criteria

M056 may close only when:

- M054 and M055 have accepted independent closures;
- the final three corrected selectors match those closures in production, contract manifest, fixtures, and docs;
- every available field in the 43-row matrix has a named authoritative production owner;
- source counts match actual contract rows exactly;
- the transit semantic regression cannot recur unnoticed;
- missing network-error ownership cannot be serialized as code `0`;
- M049's unaffected three fields and M050's valid three fields remain green;
- retained M045–M048 and M051 behavior remains unchanged;
- no production path changed under M056;
- no high/critical or unaccepted medium truthfulness/security/containment finding remains;
- the closure explicitly states that RouterInfo source completion remains partial because at least news/bans and both network-error rows are unavailable;
- overall Proposal 170 remains partial for unrelated previously accepted unsupported dimensions.

## 11. Stop conditions

Stop and create another corrective plan if:

- M054/M055 closure evidence is inconsistent with the integrated head;
- another available field lacks a real source owner;
- child-process/focused regression exposes a new semantic defect;
- production code must change to complete this review;
- source accounting cannot be reconciled without hiding an unavailable field;
- a broader core refactor or new runtime subsystem appears necessary.

## 12. Closure evidence required

The M056 closure record must contain:

- exact corrective implementation/closure heads;
- historical defect reproductions;
- requirement-to-evidence matrix for all three affected selectors;
- final 43-row source-count audit;
- changed-path containment review;
- focused and broad command outcomes;
- failure/restart/contention review;
- compatibility/security review;
- residual limitations with severity;
- explicit supersession of only the invalidated M049/M050/M052 findings;
- internal-only/no-upstream attestation.

No later RouterInfo source-completion handoff becomes ready merely because M056 closes; news/bans and any intentionally unavailable transit/error rows require separately authorized substantive-owner work.
