# M062 Closure Record — I2PControl Dependency-Surface Containment

Status: closed

Source implementation plan:

- `plans/implementation/i2pcontrol-proposal-170/062-dependency-surface-containment.md`

Source subsystem roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-containment-roadmap.md`

Accepted predecessor:

- M061 closure at `77a2555` (`a0d9f2d` planning close head).

Planning baseline: `a70dd3ac82f12fbea1f8fba51e30a9e2e516650a` — merged M061 containment reclosure head.

Pinned upstream comparison baseline:
`9b43484a21d5a1291c4881cdae62a36c527f8c0f`, accessed read-only.

Implementation commit:

- M062 corrects only `Cargo.toml` and `emissary-cli/Cargo.toml`, plus the
  planning/test artifacts listed in §3.1 of the plan.

## 1. Executive disposition

M062 is closed. The direct `subtle` dependency that I2PControl authentication
uses for reviewed constant-time password comparison is now optional at the
package boundary, activated exclusively by the `i2pcontrol` feature, and no
longer owned by `[workspace.dependencies]`.

The root `Cargo.toml` no longer carries an I2PControl-only `subtle` workspace
declaration. `emissary-cli` declares `subtle` locally with `optional = true`
and `default-features = false`, and its `i2pcontrol` feature explicitly
activates `dep:subtle`. `Cargo.lock` is byte-identical to the M062 planning
baseline `a70dd3ac`.

M062 made no production Rust source, runtime, configuration, workflow, or
release change. The M061 source-boundary authority
(`061-containment-boundary.toml` plus `m061_containment.rs`) remains unchanged
and continues to pass. The accepted Proposal 170 disposition remains 43
RouterInfo rows: 37 available, 1 protocol-permitted neutral, and 5 unavailable.
M051 remains independently blocked because substantive news and banned-peer
owners are absent.

A complementary machine-readable dependency authority
(`062-dependency-containment.toml`) and a semantic static guard
(`m062_dependency_containment.rs`) are now installed. No CI/release/fuzz/
coverage/platform expansion occurred.

## 2. Requirement-to-evidence matrix

| Requirement | Evidence | Result |
|---|---|---|
| M058–M061 are accepted closed | Prior closure records and `a0d9f2d` planning head | pass |
| No independent non-I2PControl workspace consumer exists | Source/manifest inspection; only `emissary-cli` uses `subtle = { workspace = true }`; `emissary-core` declares a literal version entry, not a workspace reference | pass |
| Root `[workspace.dependencies]` no longer owns `subtle` | `Cargo.toml` byte/semantic-equivalence to pinned upstream | pass |
| `emissary-cli` declares `subtle` as `optional = true` with `default-features = false` | `emissary-cli/Cargo.toml`; semantic TOML guard | pass |
| `i2pcontrol` explicitly activates `dep:subtle` | Feature list and `cargo metadata` introspection | pass |
| `default`, `ui`, `metrics` do not activate `subtle` | Manifest + Cargo.toml semantic guard | pass |
| `Cargo.lock` is unchanged relative to `a70dd3ac` | `git diff` against baseline | pass |
| M061 source-boundary files remain byte-identical across M062 | `git diff` of `061-containment-boundary.toml` and `m061_containment.rs` | pass |
| No `emissary-cli/src/**`, `emissary-core/**`, `emissary-util/**`, example, workflow, runtime config, or release change | `git diff --name-only` against the M062 baseline | pass |
| Manifest + guard are self-consistent | `m062_dependency_containment` (8 tests passed) | pass |
| Guard is fail-closed for repeated regressions | Manual regression injection (workspace re-add, non-optional re-declaration, `ui` activation) all failed the guard | pass |
| Reviewed constant-time `subtle` primitive retained | `emissary-cli/src/i2pcontrol/auth.rs` `use subtle::ConstantTimeEq;` and `compare_passwords` body unchanged; authentication tests pass | pass |
| Accepted Proposal 170 behavior and 37/1/5 RouterInfo disposition unchanged | Invariant review; no source-path or contract change | pass |
| M051 blocker and unsupported tunnel data-plane scope unchanged | Invariant review; no source-path or contract change | pass |
| No upstream issue, pull request, review, merge, contribution preparation, or write occurred | Attestation in §8 | pass |

## 3. Exact direct ownership disposition

The post-M062 `subtle` dependency edges are:

| Owner | Manifest entry | Optional | Owning feature | Notes |
|---|---|---|---|---|
| `emissary-cli` | `subtle = { version = "2.6.1", default-features = false, optional = true }` | yes | `i2pcontrol` (via `dep:subtle`) | I2PControl authentication, constant-time password comparison |
| `emissary-core` | `subtle = { version = "2.6.1", default-features = false }` | no | n/a (literal version, not workspace) | DSA constant-time compare in `emissary-core/src/crypto/dsa.rs`; unchanged by M062 |
| Root `[workspace.dependencies]` | absent | n/a | n/a | Was the I2PControl-only leak; removed |

`emissary-util` and the `examples/*` packages have no `subtle` reference and
were not modified.

## 4. Verification executed

Passed:

```text
rtk cargo metadata --format-version 1 --no-deps
rtk cargo check -p emissary-cli --no-default-features
rtk cargo check -p emissary-cli --no-default-features --features i2pcontrol
rtk cargo build -p emissary-cli --no-default-features
rtk cargo build -p emissary-cli --no-default-features --features i2pcontrol
rtk cargo build -p emissary-cli
rtk cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m062_dependency_containment (8)
rtk cargo test -p emissary-cli --no-default-features --features i2pcontrol --lib i2pcontrol::auth (20)
rtk cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment (7)
rtk cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --tests
rtk rustup run nightly rustfmt --check emissary-cli/tests/m062_dependency_containment.rs
rtk git diff --check
rtk git diff --name-only a70dd3ac..HEAD
```

Diagnostic only (not acceptance gates):

```text
rtk cargo tree -p emissary-cli --no-default-features -e features
rtk cargo tree -p emissary-cli --no-default-features --features i2pcontrol -e features
```

Diagnostic output confirms `subtle` is not a direct dependency of
`emissary-cli` in the no-default-features build, while it appears as a
direct dependency only when `i2pcontrol` is enabled. The crate name may
still legitimately appear transitively through `rustls`, `digest`,
`poly1305`, and similar unrelated cryptographic dependencies; that
transitive presence is permitted by the plan's dependency rule and is not
treated as a regression.

Known non-blocking toolchain findings:

- Stable `cargo fmt --all -- --check` reports the repository's existing
  stable/nightly rustfmt option drift and formatting differences across
  frozen core, utility, CLI, and I2PControl files. The new M062 test was
  formatted with `rustup run nightly rustfmt` and is clean against the
  project rustfmt options. `git diff --check` passes and no broad rewrite
  was made.
- CLI clippy reports the pre-existing frozen-path warning
  `clippy::to_string_in_format_args` at `emissary-cli/src/proxy/socks.rs:543`.
  Fixing it would be a forbidden M062 production edit.

## 5. Invariant, failure, and security review

- No Proposal 170 selector, wire, administrative persistence, support
  policy, or JSON-RPC type was changed.
- The reviewed `subtle::ConstantTimeEq` / `subtle::Choice` primitive used by
  `emissary-cli/src/i2pcontrol/auth.rs::compare_passwords` remains in place.
  Authentication tests pass unchanged.
- No dependency upgrade, refresh, or version campaign was performed. The
  `subtle` version stays at `2.6.1` and continues to resolve to the same
  package as the pre-M062 graph.
- The dependency guard is deterministic and fail-closed. Manual regressions
  (re-adding `subtle` to root `[workspace.dependencies]`, removing
  `optional = true`, and adding `dep:subtle` to the `ui` feature) were each
  detected by the corresponding guard test or by Cargo's manifest parser.
- M061's source-boundary manifest and test remain byte-identical and
  continue to pass.
- M062 has no runtime state, cancellation, restart, lock, or contention
  surface. Authentication throttle, token service, and TLS behavior remain
  covered by the existing M040/M041/M012 focused suites and unchanged.
- The new rule "a dependency whose only direct consumer is code gated by
  `feature = "i2pcontrol"` must itself be optional and feature-owned" is
  installed as a durable dependency boundary. The transitive crate-name
  presence criterion is explicitly *not* an acceptance gate.

## 6. Documentation and planning evidence

- `plans/implementation/i2pcontrol-proposal-170/062-dependency-containment.toml`
  is the current dependency authority, parsed semantically by
  `m062_dependency_containment.rs`.
- `plans/implementation/i2pcontrol-proposal-170/062-dependency-surface-containment.md`
  remains the source plan; it is now closed via this record.
- `plans/implementation/i2pcontrol-proposal-170/README.md` indexes M062 as
  closed alongside the M058–M061 historical entries.
- `plans/registry.md` records M062 as closed and removes the dependency-ready
  handoff. M062 is no longer the sole dependency-ready containment handoff;
  no successor becomes ready.
- `plans/subsystems/i2pcontrol-proposal-170-containment-roadmap.md` returns
  the containment roadmap to closed: source containment closed by M061;
  dependency containment closed by M062.
- `plans/closure/i2pcontrol-proposal-170/061-closure.md` and the M058/M059/M060
  closure records were not rewritten.

## 7. Unresolved findings

No high or medium containment, security, behavior, compatibility, or scope
finding remains open. The pre-existing CLI clippy warning and repository-wide
rustfmt drift are low-severity tooling findings and do not compromise the
manifest-only dependency containment closure.

## 8. Internal-only attestation

The pinned upstream source and commit identity were accessed read-only for
comparison. No upstream repository or maintainer channel was mutated; no
upstream issue, pull request, review, merge, adoption request, submission,
contribution artifact, branch, tag, release, or feedback request was created
or prepared. The only authorized remote operation is the push of this internal
`eggstack/emissary` branch.

**Disposition: M062 closed; dependency-surface containment corrective
complete; overall Proposal 170 support remains partial; M051 remains
independently blocked; no future implementation plan becomes ready as a
result of M062.**