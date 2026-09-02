# ADR-0005 — Internal Yosemite Fork Dependency Boundary

Status: **accepted**

Date: 2026-09-02

Applies to: I2PControl Proposal 170 completion work in `eggstack/emissary` and the authorized internal dependency fork `eggstack/yosemite`.

Supersedes only the narrow dependency-fork prohibition established by M092/M111 where it would prevent use of the maintainer-authorized `eggstack/yosemite` fork. All other M092 containment and no-upstream rules remain authoritative.

## 1. Context

M092 removed an unauthorized Yosemite/vendor/core expansion and restored the rule that Proposal 170 work should remain under `emissary-cli/src/i2pcontrol/**` unless an exact neutral lower-layer owner is required.

M111 subsequently identified a hard dependency gap: Yosemite 0.7.0 already exposes several relevant fields in `SessionOptions`, but its `SESSION CREATE` serializer does not emit tunnel variance or backup quantity and hardcodes `SIGNATURE_TYPE=7`; its Router API also hardcodes signature type 7 for `DEST GENERATE`.

The maintainer has now explicitly authorized use of the internal fork `https://github.com/eggstack/yosemite` so required generic Yosemite changes can remain outside the heavily audited Emissary tree and then be consumed by this fork.

At authorization time, `eggstack/yosemite:master` was exactly upstream Yosemite 0.7.0 commit `d0fe71da214b212790773be12a93162ae71f3e03`, providing a clean audit baseline.

Emissary currently consumes ordinary Yosemite as an unconditional workspace dependency. Replacing that dependency globally with the fork would expose non-I2PControl startup/tunnel code to the fork unnecessarily.

## 2. Drivers

- keep Proposal 170 policy/business logic concentrated in `emissary-cli/src/i2pcontrol/**`;
- avoid copying or reimplementing SAM commands in I2PControl;
- avoid vendoring Yosemite into Emissary;
- preserve audited/default Emissary behavior outside I2PControl;
- make fork provenance exact and reproducible;
- allow only generic Yosemite/SAM APIs, not Proposal-shaped dependency code;
- keep all work internal with no upstream submission/review/merge activity.

## 3. Options considered

### A. Wait for a future crates.io Yosemite release

Rejected as the only path. It leaves the work indefinitely blocked despite an authorized internal fork and does not improve containment.

### B. Replace the workspace Yosemite dependency with `eggstack/yosemite`

Rejected. This unnecessarily changes the dependency used by ordinary Emissary startup/tunnel paths and broadens the review surface.

### C. Use `[patch.crates-io]` for Yosemite

Rejected. A workspace patch would also redirect non-I2PControl consumers and make the fork global.

### D. Vendor/path-copy Yosemite into Emissary

Rejected. This recreates the M091/M092 contamination pattern and couples dependency source to the Emissary tree.

### E. Add an I2PControl-only aliased dependency pinned to an exact `eggstack/yosemite` revision

Accepted.

## 4. Decision

Emissary MAY consume `eggstack/yosemite` only through an **I2PControl-owned optional package alias** pinned to an exact immutable commit revision.

The expected Cargo shape is equivalent to:

```toml
# existing non-I2PControl dependency remains unchanged
yosemite = { workspace = true }

# i2pcontrol-only internal fork dependency
yosemite-i2pcontrol = {
  package = "yosemite",
  git = "https://github.com/eggstack/yosemite",
  rev = "<closed-yosemite-commit>",
  features = ["async-extra"],
  optional = true,
}
```

and the `i2pcontrol` feature explicitly activates `dep:yosemite-i2pcontrol`.

Production imports under `emissary-cli/src/i2pcontrol/**` may use the aliased crate (`yosemite_i2pcontrol` in Rust identifier form). Non-I2PControl code MUST continue using the existing workspace `yosemite` dependency.

The fork MUST be pinned by `rev`, never a branch/tag-only floating reference.

No root `[patch]`, local path override, vendored copy, or replacement of the workspace Yosemite dependency is authorized.

## 5. Yosemite-side boundary

The authorized fork may implement generic Yosemite capabilities such as:

- truthful `SessionOptions` to `SESSION CREATE` serialization;
- bounded/injection-safe generic SAM/I2CP option transport;
- signature-aware `DEST GENERATE` API;
- reference-proven generic LeaseSet session-option serialization.

It MUST NOT contain `Proposal170`, I2PControl, TunnelManager, Emissary persistence, support-matrix, or Emissary backend concepts.

The Yosemite fork follows its own `plans/` governance and registry. Emissary plans consume only closed exact Yosemite commits.

## 6. Emissary lower-layer boundary

The Yosemite fork solves only the SAM client/API side. Emissary's SAM server/router remains responsible for actually honoring session options.

Any required Emissary production change outside I2PControl must still satisfy the existing lower-layer exception rule:

1. existing canonical owner;
2. neutral/non-Proposal-shaped API;
3. exact paths planned before implementation;
4. no unrelated router behavior change;
5. M061/M062 containment updated;
6. separate registered implementation plan.

ADR-0005 does not authorize general core modification by itself.

## 7. Security and reliability consequences

Positive consequences:

- the fork delta remains independently auditable against Yosemite 0.7.0;
- default/non-I2PControl Emissary dependency provenance remains unchanged;
- Proposal 170 dependency code is feature-owned and optional;
- exact-revision pinning prevents branch drift;
- no raw SAM duplication is introduced.

Costs:

- `Cargo.lock` may contain two Yosemite package instances from different sources;
- I2PControl builds carry the fork-specific package in addition to the ordinary workspace package;
- fork updates require explicit revision review and lockfile change.

Those costs are accepted because they materially improve containment.

## 8. Compatibility

Feature-disabled/default Emissary behavior MUST remain byte/provenance-equivalent with respect to the ordinary Yosemite dependency.

Enabling `i2pcontrol` may add the exact pinned fork package. No migration of startup configuration or ordinary tunnel behavior is permitted.

## 9. Verification

The adoption plan MUST prove:

- `cargo tree`/metadata shows the git-fork package only when the I2PControl feature is enabled;
- ordinary workspace Yosemite remains crates.io/version-owned as before;
- all I2PControl fork imports are under `emissary-cli/src/i2pcontrol/**`;
- no `[patch]`, `vendor/**`, or path Yosemite exists;
- lockfile source/revision is exact;
- M062 direct-dependency ownership rules remain satisfied.

## 10. External-interaction boundary

The maintainer directive authorizes writes to the internal forks `eggstack/emissary` and `eggstack/yosemite` for this workstream.

It does **not** authorize any write to upstream Yosemite, upstream Emissary, I2P upstream repositories, issues, pull requests, reviews, discussions, releases, or maintainer channels. Upstream/external sources remain read-only evidence.

No upstream contribution package or merge/review request may be prepared under this ADR.
