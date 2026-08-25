# M086 Closure — Post-M085 Documentation and Evidence Reconciliation

Status: closed

Source implementation plan:

- `plans/implementation/i2pcontrol-proposal-170/086-post-m085-documentation-and-evidence-reconciliation-corrective.md`

Source roadmaps:

- `plans/subsystems/i2pcontrol-proposal-170-tunnel-security-hardening-roadmap.md`
- `plans/subsystems/i2pcontrol-proposal-170-roadmap.md`

M084/M085 controlling evidence:

- `plans/closure/i2pcontrol-proposal-170/084-closure.md`
- `plans/closure/i2pcontrol-proposal-170/085-closure.md`

M086 baseline: `a1b26987a203796465c5f7c43279adf3006bca28`.

Final reviewed M086 implementation head:
`aeec3e02ba5a7a13669852252ff81643d41c7529`.

The closure record is added immediately after that reviewed implementation
head as a separate planning-only commit. The reviewed head contains all
substantive M086 reconciliation changes; the closure commit adds only this
record.

## 1. Disposition

M086 is complete. It reconciled stale Proposal 170 planning/support status and
closure evidence without reopening M085 or changing production behavior.

M085 remains the current-head final runtime/security closure authority. The
tunnel runtime/security workstream is closed, and no successor tunnel-security
handoff is registered. Proposal 170 remains partial only for the separately
accepted source/truthfulness limitations, RouterInfo 37/1/5 disposition, M051
blocker, and unrelated AddressBook/base-I2PControl gaps.

## 2. Changed-document and evidence matrix

| Document or guard | Reconciliation/evidence | Result |
|---|---|---|
| `plans/registry.md` | M084/M085 remain closed; M086 is closed; no dependency-ready or successor tunnel-security handoff remains | pass |
| `plans/subsystems/i2pcontrol-proposal-170-tunnel-security-hardening-roadmap.md` | M077/M078 merge defects are resolved history; M085 remains runtime/security authority; M086 is documentation/evidence-only and closed | pass |
| `plans/implementation/i2pcontrol-proposal-170/README.md` | Historical M084/M085 sequence is past tense; M086 is closed and no active handoff remains | pass |
| `docs/i2pcontrol/proposal-170-support.md` | Trusted-peer text now documents the exact bounded decode, `parse_frame`, empty remainder, `parsed.id()`, and canonical `parsed.serialize()` boundary; M086 is listed closed | pass |
| `docs/i2pcontrol/tunnel-manager.md`, `tunnel-backends.md`, `streamr-runtime.md` | Inspected for the same stale current-state language; no contradiction required a change | pass; unchanged |
| `plans/closure/i2pcontrol-proposal-170/085-closure.md` | Explicit M086 arithmetic erratum corrects `81,920` to `83,886` and preserves chronology | pass |
| `plans/closure/i2pcontrol-proposal-170/084-closure.md` | Explicit M086 clarification records the bounded production HTTP-helper restoration and M085 acceptance | pass |
| `emissary-cli/tests/m062_dependency_containment.rs` | Added only the exact M086 implementation and closure paths to `is_authorized_planning_path` | pass |

## 3. Trusted-peer documentation correction

The user-facing support record now states that trusted peer text is:

1. bounded to `MAX_TRUSTED_DESTINATION_B64_TEXT` before decode;
2. Base64-decoded once;
3. parsed with `Destination::parse_frame`;
4. rejected when the parser remainder is non-empty;
5. reduced to the 32-byte accounting ID from `parsed.id()`; and
6. forwarded as canonical full-Destination text by Base64-encoding
   `parsed.serialize()`, rather than retaining attacker-selected text.

No implementation change was required.

## 4. Transparent historical errata

M085's original `81,920` capacity transcription remains visible and is
explicitly corrected by the appended M086 erratum:

```text
HARD_PEER_STATE_MEMORY_BUDGET = 16 * 1024 * 1024 = 16,777,216 bytes
WORST_CASE_BYTES_PER_PEER = 200
MAX_PEER_ENTRIES = 16,777,216 / 200 = 83,886 (integer division)
```

The Rust expression `(16 * 1024 * 1024) / 200` is authoritative and did not
change. The error affected only closure prose, not policy construction,
runtime behavior, or M085 tests.

M084's appended clarification records that implementation commit
`776407f51e75e0df245a304749b5981e639e9aab` restored two dropped helper
definitions in production `filters/http.rs`. The restoration reinstated the
already-intended exact-list plus `x-forwarded-*` / `x-i2p-*` prefix behavior,
introduced no new wire feature or broadened policy, and was independently
audited and accepted by M085 with no high/medium finding. No new runtime
corrective is required.

## 5. Changed-path and containment review

The final `git diff --name-only a1b26987a203796465c5f7c43279adf3006bca28..HEAD`
contains only these paths:

- `docs/i2pcontrol/proposal-170-support.md`
- `emissary-cli/tests/m062_dependency_containment.rs`
- `plans/closure/i2pcontrol-proposal-170/084-closure.md`
- `plans/closure/i2pcontrol-proposal-170/085-closure.md`
- `plans/closure/i2pcontrol-proposal-170/086-closure.md`
- `plans/implementation/i2pcontrol-proposal-170/086-post-m085-documentation-and-evidence-reconciliation-corrective.md`
- `plans/implementation/i2pcontrol-proposal-170/README.md`
- `plans/registry.md`
- `plans/subsystems/i2pcontrol-proposal-170-tunnel-security-hardening-roadmap.md`

No production source, `emissary-core/**`, manifest, dependency, feature, or
lockfile path changed. The M062 guard received no production glob, dependency
rule, or source-path exception; only two exact planning paths were added.

## 6. Verification

```text
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m062_dependency_containment  → pass (19 tests)
git diff --check                                                                                             → pass
git diff --name-only a1b26987a203796465c5f7c43279adf3006bca28..HEAD                                          → paths listed in §5 only
```

Targeted inspection confirmed that active planning/support documents no longer
claim pending M077/M078 merged-head integration, reopened M085 security
reclosure, or the pre-M083 trusted-peer boundary. The remaining `81,920` and
`Destination::parse` mentions are historical plan/evidence text or explicit
M086 corrective references, not current boundary claims.

No full runtime/security rerun was required because M086 made no production
change.

## 7. Final disposition and unblocked-plan review

- M086: **closed**.
- M085: remains **closed** and is the final runtime/security authority.
- Tunnel runtime/security: **closed**; no active or successor tunnel-security
  handoff is registered.
- Containment: M061/M062/M063 authority remains accepted; M086 changed only
  exact planning-path bookkeeping.
- Source/truthfulness: remains partial with RouterInfo 37/1/5 and M051
  unchanged.

M086 was the sole dependency-ready handoff during implementation. After its
closure, no future plan in this cleanup line remains blocked or ready to be
unblocked: there is no successor plan, and none is warranted by the evidence.
Any newly discovered production defect must receive a separate narrowly scoped
plan rather than reopening M085 or M086.

## 8. Internal-only attestation

All changes and verification were confined to the internal
`eggstack/emissary` repository. No upstream issue, pull request, review,
submission, merge request, maintainer contact, or contribution artifact was
opened, drafted, or requested. External specifications and reference sources
remain read-only evidence.
