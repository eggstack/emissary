# M043 — Corrective Runtime Regression Validation

Status: blocked

Hard dependencies:

- M040 closed
- M041 closed
- M042 closed

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-roadmap.md`

Corrective authority:

- `plans/closure/i2pcontrol-proposal-170/039-closure-invalidation.md`

Applicable governance:

- `plans/003-planning-process.md`
- M038 live-runtime validation plan and closure

## 1. Bounded objective

Validate the combined M040–M042 corrective head with regressions that exercise
the exact paths missed by M032/M034/M036/M038, then run the existing bounded
Proposal 170 verification matrix.

M043 is evidence work. It may add narrowly required tests and test-only seams,
but it must not repair production defects. Any material production defect found
during M043 blocks closure and requires a new corrective implementation plan.

## 2. Required evidence gaps to close

M043 must directly prove:

1. the original startup `ServerTunnelManager` retains its runtime and reaches
   SAM session creation, destination observation, and `STREAM FORWARD`;
2. same-IP authentication failures across changing ephemeral ports share one
   throttle identity;
3. concurrent invalid-password attempts reserve failure counts atomically
   before delay;
4. `SetSubscriptions` does not return failure after durable commit when refresh
   scheduling becomes unavailable;
5. the retained M031–M039 wire, ownership, lifecycle, feature-isolation,
   containment, and unsupported-capability evidence still passes.

## 3. Scope boundary

### Authorized changes

- focused tests under `emissary-cli/src/**` or `emissary-cli/tests/**`;
- test-only helpers immediately adjacent to the corrected production owner;
- M043 implementation disposition and closure;
- directly affected evidence/documentation wording.

### Prohibited changes

- production behavior changes;
- new tunnel backends or RouterInfo sources;
- startup task adoption/control;
- router/core behavior changes;
- new remote test infrastructure;
- CI, release, coverage, fuzz, soak, or platform-matrix expansion;
- upstream interaction.

A production change discovered as necessary is a stop condition. Record the
defect and create a new plan rather than hiding the fix inside validation.

## 4. Required test architecture

### 4.1 Startup server regression

Use a bounded local fake SAM endpoint and the actual startup
`ServerTunnelManager` path, not the I2PControl server backend.

The fake must record the minimum authoritative sequence:

- HELLO negotiation;
- SESSION CREATE;
- destination publication;
- STREAM FORWARD.

The test must prove the runtime remains alive after readiness until the test
explicitly aborts its owner. No I2P peer set or external network is required.

Where deterministic and low-risk, extend the existing child-process fixture
with a startup server definition and assert that the startup inventory receives
a public destination. This is optional if the direct manager regression already
exercises the exact production path and the child fixture would add unstable
network timing.

### 4.2 Authentication regressions

Add or retain tests at two levels:

- pure throttle state tests for IP normalization and concurrent reservation;
- one handler/server-level test proving distinct source ports on one IP do not
  reset the throttle and successful auth clears the shared IP state.

Tests should avoid long real sleeps by inspecting returned delays/counts where
possible. One bounded wall-clock assertion may be used only if deterministic.

### 4.3 AddressBook regression

Exercise the live manager command path with a controlled refresh worker failure
after durable commit. Assert exact operation result, durable state, active state,
and restart state.

Also prove a pre-receipt/pre-commit failure returns error without mutation.

### 4.4 Retained contract matrix

Re-run exact literal/conformance fixtures, production adapters, production
composition, lifecycle tests, containment guards, no-feature tests, feature
suite, and focused core SAM tests required by the existing passive hook.

## 5. Ordered work packages

### WP1 — Freeze corrective implementation heads

Record the accepted M040, M041, and M042 implementation/test SHAs. Confirm that
all three closure records are accepted and no unresolved high/medium finding is
carried into M043.

### WP2 — Run exact-path regressions

Run the four required regression groups independently. Preserve full command
lines and outcomes in the implementation disposition.

### WP3 — Run bounded broad matrix

Run the matrix in Section 8. Do not add remote workflow infrastructure.

### WP4 — Inspect changed-path containment

Compare the final corrective production head against `563e093` and classify all
changed production paths. Expected production scope is limited to:

- `emissary-cli/src/tunnel/server.rs`;
- `emissary-cli/src/i2pcontrol/auth.rs`;
- `emissary-cli/src/i2pcontrol/server.rs`;
- `emissary-cli/src/address_book.rs`;
- at most narrowly related `i2pcontrol/address_book*.rs` files.

Any new `emissary-core/**`, tunnel-family, RouterInfo, frontend, workflow, or
release production change is a blocker unless separately authorized.

### WP5 — Review residual claims

Verify that documentation does not claim:

- startup server traffic formation against a live reseeded network unless
  actually tested;
- distributed/persistent/proxy-aware authentication throttling;
- synchronous successful subscription downloads;
- full Proposal 170 support;
- upstream review or acceptance.

## 6. Failure and cleanup semantics

- Fake SAM listeners bind loopback ephemeral ports only.
- Child/test tasks have explicit bounded shutdown or abort cleanup.
- Temporary state directories are removed by test ownership.
- No credential, token, destination private key, or unrestricted path is logged
  or committed.
- A failing regression blocks M043; it is not weakened or marked qualified
  without a documented environmental impossibility.
- Environment limitations may qualify traffic formation, but not the exact
  startup-manager/session/forward command sequence.

## 7. Acceptance matrix

| Dimension | Required evidence |
|---|---|
| Startup server preservation | Actual startup manager reaches session create, destination observation, and stream forward; task stays alive |
| Control-plane server retention | Existing backend lifecycle/identity tests pass unchanged |
| Auth source identity | Same IP across ports shares state; different IPs remain separate |
| Auth concurrency | Counts/delays reserved atomically before sleep |
| AddressBook mutation boundary | Post-commit refresh scheduling failure returns mutation success; pre-commit failure does not mutate |
| Proposal 170 wire | Conformance and literal fixtures pass |
| Tunnel lifecycle | M033 lifecycle tests pass |
| Feature isolation | No-feature CLI suite passes |
| Containment | M037 path/dependency guards pass; no new core behavior |
| Live composition | Existing bounded child-process scenario passes, with documented environmental qualifications |
| Unsupported capabilities | Ten tunnel families and 26 RouterInfo sources remain explicit/unfabricated |
| Internal-only boundary | No upstream writes or contribution preparation |

## 8. Verification commands

At minimum, using exact accepted package names/targets:

```bash
cargo check -p emissary-cli --no-default-features
cargo test -p emissary-cli --no-default-features
cargo clippy -p emissary-cli --no-default-features --all-targets -- -D warnings

cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test conformance_manifest
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test golden_fixtures
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m027_literal_fixtures
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test production_adapter
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test production_composition
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m033_tunnel_lifecycle
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m037_containment
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test i2pcontrol_live_runtime -- --nocapture
cargo test -p emissary-cli --no-default-features --features i2pcontrol
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings

cargo check -p emissary-core
cargo test -p emissary-core sam
cargo clippy -p emissary-core --all-targets -- -D warnings

git diff --check
```

Add exact focused M040–M042 regression commands to the disposition. Formatting
remains targeted because repository-wide stable rustfmt limitations are known.

## 9. Documentation and planning updates

On successful validation:

- create `plans/closure/i2pcontrol-proposal-170/043-implementation-disposition.md`;
- create an independent M043 closure record;
- update registry to mark M043 closed and M044 ready;
- do not remove or rewrite `039-closure-invalidation.md`;
- do not select the final subsystem status in M043.

## 10. Acceptance criteria

M043 may close only when:

- every exact-path regression passes against the combined corrective head;
- the startup server regression exercises the original manager path;
- the auth tests cover ephemeral-port churn and concurrent reservation;
- the AddressBook test covers worker failure after commit;
- the bounded broad matrix passes or records only pre-existing tooling
  limitations unrelated to the corrective code;
- changed paths remain within the corrective budgets;
- no high/medium defect remains;
- no production patch is included in M043;
- M044 is the only successor marked ready.

## 11. Stop conditions

Stop and register a new corrective implementation plan if:

- any production change is needed;
- startup server behavior still exits before forwarding;
- auth bypass remains possible through port churn or concurrency;
- AddressBook can still return failure after commit;
- feature-disabled behavior changes;
- new core behavior appears;
- a missing tunnel family or RouterInfo source is pulled into scope;
- upstream interaction is proposed or performed.

## 12. Closure evidence required

The M043 disposition and closure must include:

- accepted M040–M042 heads;
- exact focused regression commands/outcomes;
- fake SAM command-sequence evidence;
- authentication source/concurrency evidence;
- AddressBook pre/post-commit evidence;
- full bounded matrix outcomes;
- changed-path classification;
- environmental qualifications without fabricated success;
- unresolved findings with severity;
- internal-only/no-upstream attestation.