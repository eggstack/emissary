# M018A Implementation Disposition — Wire Semantics and Internal-Only Corrective Pass

Status: closed for implementation; M019A ready for independent closure

Frozen implementation/test head: `a3c4f469f4877e5ff4a0bb4230da298f0b367ed2`

Implementation commit: `a3c4f46` — `fix(i2pcontrol): complete M018A corrective semantics`

Implementation plan:

- `plans/implementation/i2pcontrol-proposal-170/018a-wire-semantics-and-internal-only-corrective-pass.md`

Parent disposition:

- `plans/closure/i2pcontrol-proposal-170/018-implementation-disposition.md`

## Internal-only compliance

This pass was performed entirely in `eggstack/emissary`.

- Repository writes targeted only the Emissary checkout and its `origin` remote.
- No upstream issue, pull request, merge request, discussion, review request, patch,
  branch, tag, or comment was created or modified.
- No upstream maintainer was contacted for review, adoption, approval, or merge.
- No submission package, contribution checklist, patch series, or merge plan was prepared.
- External Proposal 170 and repository metadata were inspected read-only for internal
  verification.

The internal-only boundary in `plans/003-planning-process.md`, the Proposal 170 roadmap,
the handoff index, registry, and active M019A handoff remains normative. No current record
authorizes upstream submission or review solicitation.

## Requirement-to-evidence matrix

| Requirement | Evidence | Disposition |
|---|---|---|
| Transit total uses forwarded/transmitted bytes only | `router_info_handler.rs` returns `TransitBytes.sent`; `canonical_transit_bytes_returns_forwarded_counter_only` uses received `11` and sent `22` and asserts `22` | resolved |
| Canonical TunnelManager operational failures use structured status | `operation_error_response`; `canonical_operation_failures_use_structured_status` covers missing edit/get/start/stop/restart, duplicate create, and malformed create | resolved |
| Malformed canonical requests remain JSON-RPC validation errors | The same focused test asserts missing create fields return `INVALID_PARAMS`; existing validation suite passes | resolved |
| Compatibility TunnelManager shapes remain unchanged | Existing capitalized-action and `List` tests remain in the module; full package suite passes | resolved |
| Canonical/base/compatibility inventories are separate | `conformance_manifest.rs` now names base methods, canonical AddressBook modes, canonical seven actions, compatibility actions, and standalone base methods separately; exact 43-key and seven-action assertions remain | resolved |
| TunnelManager documentation is exact | Lowercase canonical examples, `All: true`/`Name` rule, and structured failure wording updated in directly affected docs | resolved |
| Internal-only policy is explicit and enforceable | Normative planning section plus active roadmap, registry, handoff index, M018A, and M019A rules | resolved |

## Changed-file scope

| File group | Files | Scope result |
|---|---|---|
| Production RouterInfo | `emissary-cli/src/i2pcontrol/router_info.rs`, `router_info_handler.rs` | Existing read-only DTO/test seam and selector mapping only |
| Production TunnelManager | `emissary-cli/src/i2pcontrol/tunnel_manager.rs` | Existing canonical result-envelope paths only; compatibility behavior retained |
| Tests | `emissary-cli/tests/conformance_manifest.rs` and focused module tests | Regression and classification guards only |
| Documentation | `docs/i2pcontrol/{proposal-170-conformance,proposal-170-support,router-info,tunnel-manager}.md` | Directly affected semantic and contract wording only |
| Prohibited scope | CI, release, dependencies, router/runtime redesign, tunnel data planes, upstream writes | none entered |

## Verification evidence

Passed against the frozen head:

```text
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo test -p emissary-cli --no-default-features --features i2pcontrol
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
```

Outcomes:

- package check passed;
- package tests passed: `1137 passed` across `15 suites`;
- clippy passed with no issues;
- conformance manifest passed: `56 passed`;
- focused canonical TunnelManager failure regression passed: `2 passed`;
- focused transit semantic regression passed: `2 passed`;
- configured nightly formatting check passed for all four touched Rust files;
- `git diff --check` passed.

`cargo fmt --all -- --check` remains non-zero because the repository baseline contains
pre-existing workspace-wide formatting differences and uses rustfmt options unavailable on
stable. No unrelated files were formatted; the configured nightly check is the applicable
touched-file evidence.

## Finding disposition

| Finding | Severity | Disposition |
|---|---|---|
| M018A-F1 transit double counting | high | resolved |
| M018A-F2 canonical failure envelope escape | high | resolved |
| M018A-F3 manifest classification | medium | resolved |
| M018A-F4 TunnelManager documentation | low | resolved |
| M018A-F5 internal-only governance | governance | resolved and attested |
| M018-F6 true live SAM session evidence | medium evidence decision | retained as a qualified limitation for M019A; not reopened by M018A |

No unresolved M018A high or medium implementation finding remains. M019A must independently
adjudicate the qualified SAM evidence and perform the pinned-revision final closure review.

## Disposition and dependency update

M018A is complete as a bounded implementation corrective pass. The original M018 disposition
remains historical and corrective-pass-required; it is not rewritten into passing evidence.
The superseded M019 handoff remains non-executable.

The registry and roadmap now:

- mark M018A `closing` with this frozen head and disposition;
- mark M019A `ready` because its hard implementation dependency is complete;
- retain M019A's independent reviewer requirement and the qualified SAM evidence blocker for
  that review.

The Proposal 170 subsystem is not yet finally closed. Final status remains dependent on the
distinct internal M019A review and may only be `closed internally against pinned revision`.
