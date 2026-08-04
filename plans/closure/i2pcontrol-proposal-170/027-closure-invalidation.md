# M027 Closure Invalidation — Post-Merge Status and Feature Boundary

Status: corrective pass required

Date: 2026-08-04

Current repository head reviewed: `03a384aec495232e64468dcf61d60dd2bab5cfe0`

Affected closure:

- `plans/closure/i2pcontrol-proposal-170/027-closure.md`

Affected historical closure revived by merge:

- `plans/closure/i2pcontrol-proposal-170/019-closure.md`

Corrective implementation handoff:

- `plans/implementation/i2pcontrol-proposal-170/028-post-m027-status-and-addressbook-feature-isolation.md`

Blocked independent reclosure:

- `plans/implementation/i2pcontrol-proposal-170/029-in-scope-conformance-reclosure.md`

## 1. Disposition

M027's method-level and source-classification evidence is retained, but its final
subsystem disposition is invalidated pending M028 and M029.

Two later findings prevent the repository from remaining closed:

1. the post-M027 merge `03a384a` restored superseded M019 status and planning
   language over M027's controlling `partial Proposal 170 support` result; and
2. M022's runtime AddressBook control-state owner is constructed and used by
   ordinary address-book execution even when I2PControl is not enabled, so the
   Proposal 170 adapter is not isolated behind its advertised feature/runtime
   boundary.

The first finding is a high-severity claim/governance defect. The second is a
medium-severity compatibility and scope defect because disabled I2PControl can
still read, publish, and persist `addressbook/control-state.json`, rebuild the
normal lookup indexes from that state, and retain `serde_json` as an
unconditional CLI dependency.

No evidence was found that the post-M027 merge reverted production Rust code.
The retained M020–M027 wire, persistence, SAM, startup-inventory, source-matrix,
and literal-fixture evidence remains candidate evidence for M029.

## 2. Why prior verification missed the defects

M027 reviewed the M020–M026 implementation head before the later merge that
revived M019. It therefore could not verify the final repository status after
that merge.

M022 and M027 verified behavior with the `i2pcontrol` feature enabled. They did
not include a negative composition test proving that:

- a build without `i2pcontrol` never reads or writes Proposal 170 control state;
- a build with the feature compiled but `[i2pcontrol].enabled = false` preserves
  legacy AddressBook behavior and does not activate the control-state owner;
- disabling I2PControl after prior use leaves control-state files untouched and
  excludes them from runtime resolution until I2PControl is re-enabled; and
- `serde_json` remains optional and feature-owned.

The absence of these negative cases allowed a narrow API adapter to become an
always-active runtime authority.

## 3. Retained evidence

The following M027 conclusions remain retained unless M028 changes the affected
production path:

- standard I2PControl authentication and `params.Token` interoperability;
- JSON-RPC notification and request-ID correctness;
- exact Proposal 170 method names, action names, selector names, response
  shapes, and literal fixtures;
- atomic TunnelManager publication and secret-safe response behavior;
- explicit unsupported behavior for missing tunnel data planes;
- startup-managed inventory and proxy exit observation;
- recoverable bounded SAM observation;
- exact RouterInfo source counts: 16 available, 1 protocol-permitted neutral,
  and 26 unavailable;
- no fabricated RouterInfo values;
- internal-only/no-upstream compliance.

Retained evidence is not final closure evidence until M029 reviews the actual
post-M028 head.

## 4. Invalidated claims

The following claims must not appear as current status before M029:

- `closed against the pinned 2026-05-20 Proposal 170 revision`;
- M019 as the current or controlling final closure;
- zero unresolved high/medium findings;
- I2PControl as fully isolated from default/disabled AddressBook execution;
- unqualified Proposal 170 completion.

The truthful interim status is `corrective pass required`.

## 5. Scope of correction

M028 owns only:

- restoring M027/M020–M027 chronology and status authority;
- marking M019 superseded and non-executable again;
- isolating Proposal 170 AddressBook control state behind both the compile-time
  feature and runtime enablement boundary;
- restoring the pre-M022 disabled-mode AddressBook persistence and lookup path;
- preserving the enabled-mode single-authority behavior already established by
  M022;
- restoring `serde_json` to optional feature ownership if no other unconditional
  CLI consumer requires it;
- focused regressions and directly affected documentation.

M028 must not:

- implement any missing tunnel data plane;
- add RouterInfo telemetry, samplers, polling, peer classifications, or NetDB
  inspection;
- redesign the resolver, downloader, router, SAM, transport, streaming, or
  frontend architecture;
- rewrite unrelated planning history;
- add CI, release, coverage, fuzz, soak, or generated-evidence machinery;
- initiate or prepare upstream review, submission, adoption, or merge.

## 6. Final status boundary

M029 may reclose the corrected line only after reviewing the actual M028 head.

Expected final status remains `partial Proposal 170 support` while the frozen
source matrix contains 26 unavailable RouterInfo additions and missing tunnel
data planes remain explicit unsupported stubs. A bounded in-scope closure means
that every implemented/claimed dimension is exact and truthful; it does not
convert unavailable sources or unsupported data planes into operational
coverage.

An unqualified full-support claim requires a separate maintainer decision to
expand scope and is not authorized by M028 or M029.

## 7. Internal-only attestation

External Proposal 170 and I2PControl sources may be inspected read-only.

No plan in this corrective pass authorizes an upstream issue, pull request,
review request, discussion, patch submission, branch push, maintainer contact,
contribution package, adoption request, or merge solicitation. All repository
writes must remain in `eggstack/emissary`.
