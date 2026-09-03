# M125 Closure — M113 Capability and Crypto-Ownership Audit

Status: **closed**

Date: 2026-09-03

Source implementation plan:

- `plans/implementation/i2pcontrol-proposal-170/125-m113-capability-crypto-ownership-audit.md`

Source M113 plan and closure:

- `plans/implementation/i2pcontrol-proposal-170/113-server-presentation-address-routing-and-leaseset-residual-completion.md`
- `plans/closure/i2pcontrol-proposal-170/113-closure.md`

Repository baseline reviewed: `97083896f6170962a8c9610d056e8fc2dd57646d`

Implementation commits or pull requests:

- `08dc475fd09ebadde3d482f474a782319ca792af` — documentation, matrix, test,
  and active-roadmap audit update; no production code.

## 1. Executive finding

M125 completed the authorized read-only audit. It corrected two M113 matrix
cells: `AllowInternalSSL` is an HTTP-client filtering option in Proposal 170
and Java configuration, not a server-role capability. The two server cells are
therefore `not_applicable`.

The other 19 M113 cells remain `blocked_primitive`. Yosemite Y005 provides
canonical API-to-SESSION-CREATE transport for several LeaseSet fields, but not
the complete Proposal 170 mode contract, `OptionalLookup`, an Emissary-owned
client-auth secret lifecycle, or end-to-end encrypted-LeaseSet construction and
interoperability. No implementation successor is dependency-ready.

## 2. Requirement-to-evidence matrix

| Requirement | Evidence | Result | Notes |
|---|---|---|---|
| Re-freeze all 21 cells | M095 matrix, M105 post-M125 delta, Proposal 170 and Java configuration references | pass | 2 reclassified to `not_applicable`; 19 remain blocked |
| `AllowInternalSSL` applicability | Proposal 170 HTTP-client option grouping; Java config `allowInternalSSL` is HTTP-client-only | pass | Server cells corrected; existing HTTP-client disposition remains unchanged |
| Unique local address semantics | Java `enableUniqueLocal`; Emissary `server.rs`, `http_server.rs`, `http_bidir.rs` loopback target validation | pass | No safe allocator or routing owner exists |
| MultiHoming semantics | Proposal/server option grouping; current HTTP server ownership | pass | No bounded non-request-selected host/interface owner exists |
| LeaseSet transport capability | Yosemite Y005 `src/options.rs` and `src/proto/session.rs` at exact pinned SHA; M124 fake-SAM reachability test | pass | Transport is not end-to-end capability evidence |
| Complete LeaseSet mode mapping | Proposal mode list versus Y005 fields and current Emissary builder | partial | No exact mapping for all ten modes; no `OptionalLookup` field |
| Client-auth key ownership | Y005 typed DH/PSK API; Emissary server destination store and raw-config redaction paths | partial | No bounded `LeaseSetClientAuths` request/key persistence or generation handoff |
| No downgrade / containment | Existing fail-before-allocation rejection and M093 loopback boundary | pass | No production behavior or weaker fallback added |
| Future readiness | Registry/roadmap audit | pass | M114 remains blocked; no successor registered |

## 3. Production implementation evidence

No production implementation was authorized or introduced. Current code still
rejects the unimplemented LeaseSet Proposal options before session allocation.
The existing `yosemite_i2pcontrol` Y005 alias is the only accepted dependency
transport surface, and it remains isolated from ordinary Yosemite.

Y005's public fields and serializer were inspected at
`/home/sugarwookie/projects/yosemite` revision
`59140a2277bf296928d2e8ce39a148182eeff044`. They cover canonical LeaseSet
session properties and mode-aware DH/PSK namespace emission. They do not own
Proposal policy, LeaseSet cryptography, NetDb publication/query behavior, or
Emissary secret persistence.

## 4. Verification executed

### Commands run

```bash
rtk cargo test -p emissary-cli --no-default-features --features i2pcontrol \
  --test m095_full_support_matrix --test m105_residual_option_audit --no-fail-fast
rtk cargo test -p emissary-cli --no-default-features --features i2pcontrol \
  --test m061_containment --test m062_dependency_containment --no-fail-fast
rtk cargo check -p emissary-cli --no-default-features --features i2pcontrol
rtk cargo clippy -p emissary-cli --no-default-features --features i2pcontrol \
  --all-targets -- -D warnings
rtk git diff --check
rtk cargo fmt --all -- --check
rtk cargo tree -p emissary-cli --no-default-features --features i2pcontrol --edges normal
```

### Results

All commands except the stable formatter check passed. The matrix guard reports
`284 / 96 / 460`; containment and residual-audit suites pass; feature
compilation and clippy pass; the diff check is clean. The enabled dependency
tree retains ordinary registry Yosemite plus exactly the optional Y005 git
alias.

The repository's stable rustfmt remains unable to verify the full workspace
because of pre-existing nightly-only rustfmt settings and unrelated formatting
drift. No formatter rewrite was made; this is a low-severity baseline tooling
finding, not an M125 code failure.

## 5. Invariant review

- M093 literal-loopback target confinement and no request-selected LAN/clearnet
  routing are unchanged.
- No TLS terminator, multihoming subsystem, private LeaseSet serializer,
  router/core API, or dependency replacement was added.
- Unsupported LeaseSet options continue to fail before allocation; no public or
  unauthenticated fallback is introduced.
- Y005 remains optional and I2PControl-only; ordinary Yosemite and default
  builds remain unchanged.
- No Proposal capability is claimed from serializer reachability alone.

## 6. Failure and recovery review

No runtime state changed. Existing malformed/unsupported-option tests continue
to prove fail-before-wire/allocation behavior. Existing generation, rollback,
restart, secret-store, admission, HTTP, IRC, and Streamr tests remain the
authoritative runtime evidence. Because no new session owner was added, no new
cancellation or contention path was created.

## 7. Migration and compatibility review

The two-cell matrix correction changes planning classification only; it changes
no stored definition, RPC schema, runtime behavior, or wire format. Historical
M113/M124 closures remain intact. Current counts move from `284 / 98 / 458` to
`284 / 96 / 460` solely by reclassifying the two server-role
`AllowInternalSSL` cells.

## 8. Security review

The audit confirms that Y005's typed serializer validates bounds, canonical
property names, mode/type consistency, selected DH/PSK namespaces, duplicate
entries, and redacted debug output. Those properties do not establish the
missing Emissary secret owner or actual encrypted-LeaseSet construction.

No key material was added to planning fixtures. Existing RPC/raw-config
redaction and server destination confinement remain unchanged. The unresolved
LeaseSet capability is retained as blocked rather than approximated or silently
downgraded.

## 9. Documentation and operations

Updated internal planning evidence:

- M095 current matrix and M105 post-M125 audit delta;
- M125 plan and this closure;
- active registry and Proposal 170 roadmaps/support documentation.

No upstream issue, pull request, review, release, or maintainer interaction was
performed.

## 10. Unresolved findings

| Severity | Finding | Impact | Required action |
|---|---|---|---|
| medium | 19 applicable M113 presentation/routing/LeaseSet cells remain blocked | Full Proposal 170 support and M114 final reclosure remain unavailable | Register a successor only after exact semantics, owner, secret lifecycle, no-downgrade proof, and interoperability evidence are accepted |
| low | Stable rustfmt cannot verify the workspace's nightly-only configuration | Full formatter check is unavailable in this environment | Use the repository's compatible nightly formatter/toolchain in a future verification pass |

No new high-severity security finding was identified.

## 11. Roadmap disposition

The M113 capability/crypto audit is closed. M113 remains closed as blocked,
with M125 superseding only the two incorrect `AllowInternalSSL` server-cell
classifications. No M113 implementation successor is dependency-ready. M114
remains blocked by the current 96 applicable residual cells.

## 12. Registry updates

`plans/registry.md` and both active Proposal 170 roadmaps now record:

- M125 closed;
- current matrix `284 / 96 / 460`;
- 19 remaining M113 cells;
- no successor implementation plan registered;
- M114 still blocked.

## External-interaction attestation

Proposal, I2CP, encrypted-LeaseSet, Java configuration, and Yosemite sources
were accessed read-only. No upstream repository or maintainer channel was
mutated, and no upstream review, merge, adoption, submission, contribution
artifact, release, or contact was requested or prepared.
