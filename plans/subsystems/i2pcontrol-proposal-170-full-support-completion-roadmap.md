# I2PControl Proposal 170 Full-Support Completion Roadmap

Status: **active / partial; M153 closed; M154 registered**

Pinned Proposal authority:

- I2P Proposal 170 revision `2026-05-20`, status Open.

Current machine authority:

- M095 matrix: `336 apply / 29 blocked_primitive / 475 not_applicable` across 840 TunnelManager option/family cells.

Current execution authority:

- `plans/subsystems/i2pcontrol-proposal-170-post-m146-corrective-roadmap.md`.

Historical residual roadmap through M146:

- `plans/subsystems/i2pcontrol-proposal-170-residual-primitive-completion-roadmap.md`.

Current qualification state:

- M139 is the historical whole-surface integrated runtime/security qualification, superseded by M153 for current-head purposes because it predates M141-M145 production changes;
- M153 is closed as the current current-head qualification authority before any cryptographic residual work;
- M140 remains the accepted residual streaming applicability authority;
- M141-M145 are accepted capability closures;
- M146 is accepted blocked evidence for `UseOutproxyPlugin`.

All external specification/reference activity remains read-only. Repository writes remain internal to `eggstack/emissary`; Yosemite writes require separate ADR-0005/Yosemite planning authority.

## 1. Purpose

Move the internal fork as far toward exact Proposal-170 support as can be done truthfully while keeping Proposal-specific policy under `emissary-cli/src/i2pcontrol/**` wherever possible and preserving the security-reviewed Emissary core boundary.

Full support means real externally observable behavior. Parser acceptance, persistence, serializer reachability, dormant fields, defaults, aliases or approximate semantics do not count.

This is not a general base-I2PControl parity program, router redesign, frontend project or upstream contribution program.

## 2. Canonical authority

Read in order:

1. `plans/000-long-term-specification.md`;
2. `plans/001-terminology-and-domain-model.md`;
3. `plans/002-long-term-roadmap.md`;
4. `plans/003-planning-process.md`;
5. ADR-0001 through ADR-0005;
6. M061/M062 containment;
7. M093 tunnel security;
8. M095 full-support matrix;
9. M105 residual audit;
10. M110 completion ledger;
11. accepted historical closures;
12. the current subsystem/corrective roadmap;
13. `plans/registry.md`;
14. the one registered implementation plan.

Historical closure records remain immutable. Later corrective milestones supersede only explicit current-head authority/metadata.

## 3. Current implemented surface

Current accepted Proposal-170 support includes:

- RouterInfo: 43 additions / 42 available / 1 protocol-permitted neutral / 0 unavailable;
- AddressBook CRUD, subscriptions, all 13 SetConfig keys and deterministic precedence;
- all 12 canonical TunnelManager data planes and seven canonical actions for the claimed subset;
- all six ClientServicesInfo selectors;
- API-1 auth/version/token behavior required by the extension surface;
- M127 finite token lifetime;
- M128 bounded JSON-RPC batch conformance;
- M129 fail-closed non-loopback management TLS;
- shared-session/destination ownership and exact optional Yosemite boundary;
- neutral variance/backup and live tunnel-quantity/LeaseSet desired-count behavior;
- M136 all 21 applicable `Reduce*` cells;
- M137 all 14 applicable `Close`/`CloseTime` cells plus authoritative idle termination cause;
- M134 six applicable non-Streamr TCP `NewDest` cells;
- M141 two `UniqueLocalAddressPerClient` HTTP-server cells;
- M142 HTTP-client `SSLProxies` and `JumpList`;
- M143 retained `Profile:client` runtime behavior;
- M144 four application `UseSSL` cells;
- M145 two `MultiHoming` / `shouldBundleReplyInfo` cells.

M140 additionally established seven affirmative N/A corrections without promotions.

M146 promoted nothing and correctly left four `UseOutproxyPlugin` cells blocked.

Current matrix:

- `336 apply`;
- `29 blocked_primitive`;
- `475 not_applicable`.

Full Proposal 170 support is **not** claimed.

## 4. Current residual inventory

| Cluster | Blocked cells | Current disposition |
|---|---:|---|
| `SigType` | 10 | neutral destination signature-suite infrastructure missing |
| `EncryptLeaseSet` | 5 | encrypted LeaseSet publication/runtime missing |
| `OptionalLookup` | 5 | blinded/secret lookup missing |
| `LeaseSetClientAuths` | 5 | authenticated client access missing |
| `UseOutproxyPlugin` | 4 | M146 terminal blocked under current provider/security architecture |
| **Total** | **29** | |

M095 machine authority wins over prose summaries.

## 5. Ownership and containment

Proposal/admin/application policy belongs under `emissary-cli/src/i2pcontrol/**` wherever a truthful implementation can live there.

A non-I2PControl production change is permitted only when:

1. behavior belongs to an existing canonical lower-layer owner;
2. no truthful I2PControl-local implementation exists;
3. exact files are named before implementation;
4. the seam is neutral and Proposal-free;
5. unrelated router behavior is unchanged;
6. M061/M062 are amended explicitly;
7. a registered plan authorizes the change.

Accepted lower-layer exceptions remain narrow and historical:

- M135 neutral tunnel-pool/destination/LeaseSet desired-target primitive;
- M136-M137 neutral SAM idle lifecycle owner;
- M143 neutral streaming max-window owner;
- M145 neutral destination/session reply LeaseSet bundling owner.

No broad `crypto/`, `netdb/`, `i2np/`, `destination/`, `primitives/` or transport prefix waiver is permitted for future residual work.

Yosemite remains the sole accepted SAM implementation and exact optional Y005 pin unless separately superseded.

## 6. Cross-cutting security invariants

All remaining work preserves:

- exact pinned names/types/actions/presence semantics;
- unsupported supplied values fail before allocation/effect;
- no accept-inert or fabricated support;
- no direct-I2P-to-clearnet DNS/TCP fallback;
- literal-loopback/local-target confinement where already required;
- trusted peer identity boundaries;
- bounded state/tasks/queues/timers/retries;
- generation-local cancellation and transactional edit/restart;
- no lock across unrelated network/filesystem/crypto work;
- secret/private-key/password/auth redaction;
- no signature-suite fallback;
- no encrypted/authenticated LeaseSet downgrade to plaintext/public/unauthenticated behavior;
- feature-disabled dependency isolation;
- no unrelated frontend/base-method/deferred-tunnel work;
- external interaction read-only/internal-only.

## 7. Historical residual execution through M146

```text
M139 integrated lifecycle-head requalification     [CLOSED — historical current-head authority]
  |
  v
M140 residual applicability re-freeze               [CLOSED — 325/40/475]
  |
  v
M141 UniqueLocalAddressPerClient                    [CLOSED — 327/38/475]
  |
  v
M142 SSLProxies + JumpList                          [CLOSED — 329/36/475]
  |
  v
M143 Profile:client                                 [CLOSED — 330/35/475]
  |
  v
M144 application UseSSL                             [CLOSED — 334/31/475]
  |
  v
M145 MultiHoming / reply LeaseSet bundling          [CLOSED — 336/29/475]
  |
  +--> 7cbd80a no-std/format follow-up              [accepted production follow-up]
  |
  v
M146 UseOutproxyPlugin feasibility                  [CLOSED AS BLOCKED — 336/29/475]
```

The historical residual roadmap is closed/superseded for execution after M146.

## 8. Current corrective execution

Current roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-post-m146-corrective-roadmap.md`.

Dependency graph:

```text
M153 post-M146 current-head requalification         [CLOSED — zero promotion]
  |
  v
M154 M147 signature-domain/security owner re-freeze [REGISTERED — zero promotion]
  |
  v
M147 neutral destination signature primitive        [DEFERRED — zero promotion]
  |
  v
M148 Proposal SigType                               [DEFERRED — up to 10]
  |
  v
M149 EncryptLeaseSet                                [DEFERRED — up to 5]
  |
  v
M150 OptionalLookup                                 [DEFERRED — up to 5]
  |
  v
M151 LeaseSetClientAuths                            [DEFERRED — up to 5]
  |
  v
M152 final whole-surface requalification            [DEFERRED — zero promotion]
```

Only M154 is registered.

## 9. M153 corrective qualification (closed)

M153 closed as complete (`plans/closure/i2pcontrol-proposal-170/153-closure.md`) and is the current runtime/security qualification authority. It established the new current-head qualification before crypto work by:

- mechanically preserving `336/29/475` and exact 29-cell residual identity;
- recording the actual last production-bearing head, expected `7cbd80a6d72aa07d158ba9dc74f8bbacef767be2` unless audit proves otherwise;
- correcting stale M095 production-head metadata;
- repairing historical-vs-current test authority so broad I2PControl testing is not red merely because later accepted milestones changed aggregate counts;
- requalifying M127-M145 behavior plus M146 fail-closed blocked behavior;
- re-proving M061/M062 containment/dependency isolation;
- changing no production Rust/dependency/Yosemite code.

A production defect stops M153 and requires a separate corrective plan.

## 10. M154 pre-crypto gate

M154 is mandatory before M147 registration. It freezes:

- exact Proposal/reference SigType domain;
- per-algorithm security disposition;
- actual current generate/sign/verify/serialize/persist support;
- exact maintained Rust primitives/dependencies needed;
- persistent destination/import/export compatibility;
- exact file-by-file core owners and M061/M062 path budget.

M154 must amend/register M147, split it, or leave SigType blocked. It performs no production work and no matrix promotion.

## 11. Cryptographic residual rules

### M147/M148 SigType

Infrastructure completion requires actual destination key generation, certificate encoding, signing, verification and persistence. M147 has zero promotion budget. M148 may promote only suites/families whose full runtime identity path exists with no fallback.

### M149 EncryptLeaseSet

Requires real encrypted LeaseSet semantics end-to-end. Serializer/type reachability alone is insufficient. Plaintext downgrade is prohibited.

### M150 OptionalLookup

Requires actual blinded/secret lookup semantics at the canonical NetDB/destination owner with no public lookup fallback or secret leakage.

### M151 LeaseSetClientAuths

Requires real authorization modes and unauthorized-client rejection with bounded key/auth state and no unauthenticated downgrade.

Each security-sensitive plan must be amended with exact reference semantics and exact paths before registration.

## 12. M146 terminal blocker policy

M146 remains closed blocked.

The four `UseOutproxyPlugin` cells cannot be promoted under the current architecture because:

- no genuinely distinct real local provider exists;
- a dummy registry/provider is infrastructure with zero support value;
- aliasing configured `ProxyList` collapses semantics;
- direct-clearnet OS networking violates the accepted security boundary.

The crypto/LeaseSet chain does not implicitly reopen M146.

A future provider successor requires a separate architecture/security decision and plan. Until then, those four cells remain explicit blockers.

## 13. Final completion semantics

M152 decides the final truthfulness state from the then-current machine matrix.

### Full Proposal completion

Allowed only if:

- all 840 cells are `apply` or affirmative `not_applicable`;
- `blocked_primitive == 0`;
- every apply cell has real runtime evidence;
- all security/containment qualification passes;
- no high/medium Proposal-scoped defect remains.

### Safe partial terminal completion

If the cryptographic tail closes but M146 remains blocked, M152 may close the current safe residual workstream as **partial / terminal under current security policy**. It must:

- retain the exact four blockers;
- explicitly refuse a full-support claim;
- document that a future architecture/security decision is required for those cells;
- keep all implemented applicable cells requalified.

No roadmap may redefine four blocked cells as complete merely to close the workstream.

## 14. Verification policy

Future milestones require, as applicable:

- `cargo check` for touched core/CLI configurations;
- no-std checks for touched core modules;
- core/CLI unit and integration suites;
- M061/M062 containment/dependency guards;
- M095/M105 matrix/residual guards;
- M127-M129 security regressions;
- lifecycle regressions;
- milestone-specific adversarial/runtime/reference fixtures;
- clippy and rustfmt evidence;
- `git diff --check`.

Known unrelated formatter/lint drift must be recorded, not normalized opportunistically.

## 15. Registration discipline

Per `plans/003-planning-process.md`:

- only M153 is registered;
- M154 remains deferred until M153 closes cleanly;
- M147 cannot be registered without M154 disposition;
- M148-M152 remain deferred behind hard dependencies;
- M146 remains closed blocked;
- file presence/M062 planning bookkeeping never grants production authority;
- material path/dependency/architecture deviations require amendment before coding.

## 16. Final success rule

The roadmap succeeds when the repository reaches a fully requalified truthful terminal state for the pinned Proposal revision:

- either zero applicable blockers and full support;
- or a safe partial terminal state with exact unavoidable blockers explicitly preserved under current security policy.

In neither case may containment/security be weakened or support fabricated to improve counts.
