# M126 Closure — Post-M125 Operational, Security, and Spec Requalification

Status: **closed**

Date: 2026-09-03

Source implementation plan:

- `plans/implementation/i2pcontrol-proposal-170/126-post-m125-operational-security-and-spec-requalification.md`

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-post-m114-corrective-roadmap.md`

Reviewed implementation head: `0ece72d` (`test(i2pcontrol): add M126 qualification evidence`).

Pinned authority:

- Proposal 170 revision `2026-05-20`, status Open;
- base I2PControl API-1 authentication and JSON-RPC contract;
- Proposal inventory SHA-256 `f13ae00b886c5e72131bc5d5b138a371148d1faa6899a119a1dacb65a555e7dc`.

## 1. Executive disposition

M126 is closed. The current implemented subset is operationally and security-qualified against
the pinned Proposal 170 revision on the reviewed implementation head. No current-head production,
authentication, TLS, persistence, lifecycle, source-truthfulness, or containment defect was
found. The active support status remains partial: `284 apply / 96 blocked_primitive / 460
not_applicable`.

M126 did not implement or promote a blocked cell. No M127+ corrective plan was required, and no
future residual implementation became dependency-ready. The final full-support line remains
blocked until the residual cells acquire genuine canonical owners and receive a later, separately
numbered final reclosure.

## 2. Requirement-to-evidence matrix

| Work package / requirement | Current-head evidence | Result |
|---|---|---|
| WP1: pinned inventory and matrix | `m126_requalification::current_matrix_is_mechanically_requalified`; M095/M105 guards; pinned source hash and exact 43 RouterInfo, 13 SetConfig, 12 tunnel-type inventory | pass |
| WP2: production-owner traces | `init_server` requires the runtime AddressBook owner, loads production AddressBook/TunnelManager stores, installs production adapters, and `main.rs` supplies the shared composition handles; live runtime test exercises the child binary | pass |
| WP3: auth/TLS/JSON-RPC/resources | `i2pcontrol_live_runtime::live_runtime_interoperability`; server/auth tests; adversarial suite; real TLS child composition; all six protected methods reject missing/invalid credentials and conflicting header/parameter tokens | pass |
| WP4: AddressBook | live Add/Lookup/Delete and restart round trip; production CRUD/persistence tests; runtime confinement, symlink/type, malformed hostname/destination, subscription, precedence, and concurrency tests | pass |
| WP5: TunnelManager | live production create/start failure/edit/restart/delete and startup ownership checks; production lifecycle/persistence tests; M123 cancellation tests; backend admission, framing, local-target, IRC, Streamr, secret, and unsupported-option suites | pass |
| WP6: RouterInfo and ClientServicesInfo | live request-time selector checks; RouterInfo truthfulness suite; shared tunnel/source tests; service-registry live-state and disabled-service tests | pass |
| WP7: containment/dependency boundary | M061/M062 guards; M126 production-composition guard; exact optional Yosemite Y005 alias and ordinary registry Yosemite tree checks | pass |
| WP8: active authority | active registry, roadmap, implementation README, `AGENTS.md`, and I2PControl docs now state M126 closed and partial `284 / 96 / 460` support; historical records remain unchanged | pass |

## 3. Normative surface and residual requalification

The M126 guard mechanically recomputes the active matrix from
`095-full-support-matrix.toml`:

- RouterInfo: 43 additions, 42 available, 1 protocol-permitted neutral, 0 unavailable;
- AddressBook SetConfig: 13 keys, with 12 operational keys and the contract-defined `theme`
  administrative metadata key;
- TunnelManager: 12 canonical tunnel types and 284 `apply`, 96 `blocked_primitive`, and 460
  `not_applicable` option/type cells.

The M105 residual audit and M095 current matrix agree on the exact blocked set after M125's two
server-role `AllowInternalSSL` applicability corrections. The 96 residuals remain:

- 4 `UseSSL` cells;
- 10 `SigType` cells;
- 63 client proxy/profile/reduction/lifecycle cells, including 18 `Close`/`CloseTime`/`NewDest`;
- 19 server presentation/routing/LeaseSet cells.

The residual audit found no newly available neutral authoritative owner. Yosemite Y005 remains
transport capability only; Emissary still has no accepted encrypted/authenticated LeaseSet
construction, NetDb, client-auth key-lifecycle, per-client-address, or multihoming owner.
Blocked options therefore remain fail-before-allocation and were not promoted by M126.

## 4. Production-owner and security review

The request path remains:

```text
TLS HTTP POST -> JSON-RPC parse -> API-1/token boundary -> domain handler
  -> production adapter -> shared runtime/store owner -> committed observation/mutation
  -> sanitized JSON-RPC result or error
```

`init_server` rejects missing runtime AddressBook ownership, loads the real AddressBook and
TunnelManager stores before returning a server, shares one production TunnelManager with
RouterInfo/ClientServicesInfo, and never substitutes test adapters. Fake adapters remain limited
to explicit test constructors. The M126 static guard checks these composition seams and the live
test launches the feature-enabled CLI binary rather than an in-process fake state.

Authentication accepts API version 1 only, issues opaque bounded tokens, rejects missing,
malformed, unknown, and conflicting credentials, and removes the token before domain validation.
Batch arrays are rejected as invalid requests before protected dispatch; notifications execute
through the same authentication/validation path and suppress the response. IDs remain preserved.
The real child-process listener is TLS-only; plain HTTP does not reach JSON-RPC dispatch. Body,
connection, in-flight request, handshake, deadline, and failed-authentication bounds remain
active. Passwords, tokens, destinations, private keys, filesystem internals, and debug values are
not exposed by the exercised responses or child diagnostics.

AddressBook mutations use the runtime owner and durable store rather than a second I2PControl-only
resolver truth source. Confinement, atomic generation persistence, restart recovery, subscription
state, cross-book precedence, symlink/type rejection, and concurrent mutation coverage remain
green. Tunnel definitions use the shared production registry and lifecycle owner; failed starts,
edits, restarts, cancellation boundaries, duplicate/collision cases, startup ownership, resource
admission, and bounded data-plane policies remain covered. RouterInfo and ClientServicesInfo use
request-time authoritative inspection/registry sources; unavailable observations remain explicit
errors or the single protocol-permitted neutral value.

## 5. Containment and dependency evidence

No M126 production source outside the accepted I2PControl boundary was added. The only changed
production-adjacent paths are tests and active guidance. The M062 guard was corrected to classify
M125 and M126 closure/planning evidence as evidence paths instead of reporting the historical
allowlist as a false production regression. M061/M062 continue to reject unexplained core/util
or non-I2PControl policy expansion.

The `yosemite-i2pcontrol` dependency remains optional, exact-pinned to Y005
`59140a2277bf296928d2e8ce39a148182eeff044`, and activated only by `i2pcontrol`. Ordinary
workspace Yosemite remains the registry package; no global patch, path replacement, vendoring, or
upstream repository mutation occurred.

## 6. Verification executed

All commands were run from the reviewed Emissary checkout. Exit status 0 means pass.

| Command | Result |
|---|---|
| `cargo check -p emissary-cli --no-default-features --features i2pcontrol` | pass |
| `cargo check -p emissary-cli --no-default-features` | pass |
| `cargo check` | pass |
| `cargo test -p emissary-core` | pass |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --lib --no-fail-fast` | pass; 709 tests |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --no-fail-fast` | pass; complete package suite, including 1 live production-runtime test |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment --test m062_dependency_containment --test m095_full_support_matrix --test m105_residual_option_audit --test m126_requalification --no-fail-fast` | pass; 37 tests |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test i2pcontrol_live_runtime -- --nocapture` | pass; 1 real child-process/TLS runtime test |
| `cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings` | pass |
| `cargo fmt --all -- --check` | recorded non-zero; installed stable formatter cannot apply the repository's nightly-only settings and reports pre-existing unrelated drift |
| `git diff --check` | pass |

The stable formatter result is a pre-existing low-severity tooling limitation. No unrelated
formatter churn was introduced. The focused M061/M062/M095/M105/M126 gate passed after active
documentation reconciliation, and the live runtime test passed with the M126 auth/batch/conflict
coverage.

## 7. Unresolved findings

| Severity | Finding | Disposition |
|---|---|---|
| deferred capability | 96 applicable Proposal cells remain `blocked_primitive` | Truthfully retained; no implementation owner is dependency-ready and no full-support claim is made |
| low | Stable rustfmt cannot verify nightly-only repository formatting settings | Recorded tooling limitation; no M126 production defect |
| high/medium Proposal-scoped defect | None found | No M127+ corrective plan registered |

## 8. Roadmap and future-plan readiness

M126 is closed in the implementation plan, active registry, post-M114 roadmap, implementation
README, and I2PControl support/conformance documentation. The post-M114 corrective workstream
remains active only because the final residual capability line is incomplete; it has no
dependency-ready successor. M114/full-support completion remains blocked by the 96 residual
cells. No future plan status was advanced because no hard dependency became closed with a newly
implementable owner.

Historical closure records, including M120's superseded cancellation claim and M121/M125 matrix
history, were not rewritten. No speculative M127+ plan was registered.

## 9. Internal-only external-interaction attestation

The pinned Proposal 170, I2PControl, Java configuration, and Yosemite sources were treated as
read-only evidence. No upstream repository or maintainer channel was mutated. No upstream issue,
pull request, review, merge/adoption request, release, submission, contribution artifact, or
maintainer contact was created, requested, or prepared.
