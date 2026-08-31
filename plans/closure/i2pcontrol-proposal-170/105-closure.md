# M105 Closure — Residual TunnelManager Option Primitive and Applicability Audit

Status: **closed**

Source implementation plan:

- `plans/implementation/i2pcontrol-proposal-170/105-residual-tunnel-option-primitive-audit.md`

Review date: 2026-08-31

## 1. Disposition

M105 completed its authorized evidence-only audit. It reconciled every
applicable residual TunnelManager cell from M104/M095, recorded the exact
semantic and ownership blocker for each cell, and registered one bounded,
dependency-ready successor. M105 introduced no production behavior and did
not reduce the production blocker count.

The final disposition is **closed**. Proposal 170 support remains partial
against the pinned revision, and M104 remains closed as blocked.

## 2. Exact heads and input evidence

| Evidence | Exact value |
|---|---|
| M104 reviewed production head | `cf5f5192c97d9e9963d9128f772436625a38a6c6` |
| M105 planning head at freeze | `43c1f50a9658137281a565157de9ae5cef1785b8` |
| M105 audit/status/handoff commit | `395f43d` (`plans(i2pcontrol): close M105 residual option audit`) |
| M095 matrix SHA-256 | `fcc7d21dd886cd96ac614507abba5e3cfc806cee942bb09eb387e1a60078ac` |
| M095 embedded production-head metadata | `19bdf6cee79466260b3d05bcf8e17a3ed51a6c11` |
| Proposal 170 revision audited | `2026-05-20` |

The authoritative machine-readable deliverable is
`plans/implementation/i2pcontrol-proposal-170/105-residual-option-audit.toml`.
Its SHA-256 is:

`d1719946ef0aa10d7855683feaa9426a6c9d67134dee67fdba8e1c86e57e1eed`

The artifact is pinned to the M104-reviewed M095 input. Its 70 canonical
option rows across 12 canonical tunnel types produce 840 cells: 218
`apply`, 164 `blocked_primitive`, and 458 `not_applicable`. There are no
`planned_apply`, unknown, unsupported, or accept-inert cells. M095 support
dispositions are unchanged.

## 3. Coverage and disposition accounting

The artifact contains exactly one record for each of the 164 unique M095
`blocked_primitive` cells. The M105 static guard derives the input set from
M095, rejects duplicates, and requires exact set equality.

| Audit disposition | Cells | Result |
|---|---:|---|
| `i2pcontrol_local_candidate` | 6 | bounded successor-ready `DelayOpen` slice |
| `dependency_blocked` | 101 | missing Yosemite/SAM/dependency wire or primitive |
| `architecture_decision_required` | 45 | no safe canonical owner under current architecture |
| `semantic_blocked` | 12 | Proposal/reference semantics are unresolved or speculative |
| `neutral_owner_candidate` | 0 | none established |
| `not_applicable_candidate` | 0 | none established |

Residual-family totals are:

| Family | Cells | Audit result |
|---|---:|---|
| `Shared` | 7 | architecture decision required |
| `UseSSL` | 4 | dependency blocked by SAM session wire |
| `TunnelVariance` / `TunnelBackupQuantity` / `SigType` / `CustomOptions` | 40 | dependency blocked by SAM session wire/primitive coverage |
| `NewDest` / `PersistentClientKey` | 14 | architecture decision required |
| `PrivKeyFile` | 10 | architecture decision required |
| `UseOutproxyPlugin` / `SSLProxies` / `JumpList` | 12 | architecture or semantic blocked |
| `ConnectDelay` / `Profile` / `DelayOpen` / `Reduce*` / `Close*` | 56 | dependency, semantic, or local candidate; no silent approximation |
| `AllowInternalSSL` / `UniqueLocalAddressPerClient` / `MultiHoming` | 6 | architecture decision required |
| `EncryptLeaseSet` / `OptionalLookup` / `LeaseSetClientAuths` | 15 | dependency blocked by LeaseSet serializer/key handoff |

The six local candidates are exactly `DelayOpen` for `client`, `httpclient`,
`ircclient`, `socks`, `socksirc`, and `connectclient`. `DelayOpen` for
`streamrclient` remains semantic-blocked because Streamr has no equivalent
first-local-client-socket boundary in the pinned contract and applying the
TCP meaning would be speculative.

## 4. Requirement-to-evidence matrix

| M105 requirement | Evidence | Outcome |
|---|---|---|
| Exact 164-cell reconciliation | M095 SHA pin, TOML records, `m105_residual_option_audit` | pass |
| Exact Proposal semantics and applicability | per-cell summaries/evidence refs; pinned revision | pass; unresolved cases remain explicitly blocked |
| Existing I2PControl owner/path review | per-cell owner and exact production paths | pass |
| Yosemite/SAM primitive and wire review | per-cell primitive/wire field; local Yosemite source review | pass; missing capabilities remain dependency-blocked |
| Key, secret, persistence, and shared-session review | per-cell implications and architecture dispositions | pass; no unsafe storage or handoff was invented |
| Proxy/TLS/presentation/routing review | per-cell ownership/blocker records | pass; absent safe owners remain architecture-blocked |
| Lifecycle and concurrency review | six bounded `DelayOpen` candidates; Streamr separation recorded | pass; only M106 is promoted |
| LeaseSet/security review | per-cell serializer/key implications | pass; unsupported behavior remains fail-closed |
| Successor grouping | M106 plan and registry/roadmap/README handoff | pass; exactly one successor registered |

## 5. External read-only sources consulted

The following were read only as evidence; no external repository, issue, pull
request, service, or source was modified:

- [Proposal 170 — I2PControl Expansion](https://i2p.net/proposals/170-i2pcontrol-expansion.txt)
- [I2P Configuration specification](https://www.i2p.net/en/docs/specs/configuration/)
- [I2CP overview](https://i2p.net/en/docs/specs/i2cp-overview/)
- [Streaming API](https://i2p.net/en/docs/api/streaming/)
- [I2PControl implementation pull request](https://github.com/i2p/i2p.plugins.i2pcontrol/pull/6)
- [Yosemite documentation](https://docs.rs/yosemite/latest/yosemite/)

The accepted local Yosemite 0.7.0 source was also inspected. Its declared
session options exceed what its SAM `SESSION CREATE` serializer emits; its
stream style has no equivalent `DelayOpen` lifecycle primitive. These are
dependency evidence, not authorization to patch, fork, vendor, or replace the
dependency.

## 6. Verification outcomes

| Command | Outcome |
|---|---|
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m105_residual_option_audit -- --nocapture` | pass; 1 test |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m095_full_support_matrix` | pass; 1 test |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment --test m062_dependency_containment` | pass; 26 tests across 2 suites |
| `cargo check -p emissary-cli --no-default-features --features i2pcontrol` | pass |
| `cargo check` | pass |
| `git diff --check` | pass |
| `cargo fmt --all -- --check` | known pre-existing stable/nightly rustfmt option mismatch; no unrelated churn retained |

The M105 guard also confirms the required fields, allowed dispositions, exact
candidate paths, dependency evidence, and explicit unresolved semantic wording
for every record.

## 7. Containment, dependency, and security review

Changed paths are limited to the M105 audit/closure planning records, the
M106 planning handoff, the implementation registry/roadmap/README, and the
narrow M105 guard plus its M062 allowlist registration. No production source,
`Cargo.toml`, `Cargo.lock`, dependency, core/util code, frontend, workflow, or
external source changed. No dependency or feature was added.

No blocked option is accepted, allocated, or reported as applied. The audit
preserves fail-before-allocation behavior and the existing M093 anonymity,
secret/path, literal-loopback, TLS, Streamr, and cancellation boundaries.
Potential destination/key, private-key, proxy, TLS, routing, LeaseSet, and
shared-session changes remain architecture-gated rather than being approximated
inside the administrative API.

## 8. Future-plan disposition

M105 unblocks exactly one future plan:

`plans/implementation/i2pcontrol-proposal-170/106-delay-open-client-listener.md`

M106 is **ready** and is limited to a lazy first-local-use `DelayOpen`
implementation for the six existing TCP-style client families. It has an
existing I2PControl runtime owner and a bounded path budget. It must not widen
into Streamr, tunnel data-plane semantics, Yosemite changes, core/util changes,
or dependency work.

No other future plan is unblocked. The remaining 158 residual cells stay
deferred or blocked pending new dependency, semantic, or architecture evidence.
M104 must not be reattempted until the applicable blocker inventory is actually
resolved; M105's classification alone does not satisfy the full-support gate.

## 9. Internal-only attestation and final disposition

All writes remain inside this repository. Proposal 170, Java I2P, i2pd, I2P+,
Yosemite, and GitHub material were read-only evidence. No upstream issue,
pull request, review, adoption request, release, maintainer contact, or
external service was modified.

M105 is therefore **closed**. Proposal 170 support remains partial with the
authoritative production inventory at 218 `apply`, 164 `blocked_primitive`,
and 458 `not_applicable` cells.
