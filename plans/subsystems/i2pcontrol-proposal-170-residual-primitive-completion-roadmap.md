# I2PControl Proposal 170 Residual Primitive Completion Roadmap

Status: **historical through M146; post-M146 execution superseded by the corrective roadmap**

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md`.

Successor execution roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-post-m146-corrective-roadmap.md`.

This document remains the historical subsystem authority for the M140-M146 residual sequence. It no longer defines the next executable handoff after M146.

Planning baseline for this historical line:

- M139-qualified matrix: `325 apply / 47 blocked_primitive / 468 not_applicable`;
- Proposal 170 revision `2026-05-20`, status Open;
- Java I2PControl reference head `45bb593000408071dd376b78848fdc246dccd964`;
- Java I2P/I2PTunnel snapshot `2c3fd2a9532cd86ec06cb6f2b9f3f813ca752243`;
- Yosemite optional exact revision `59140a2277bf296928d2e8ce39a148182eeff044`.

All external reference activity was read-only.

## 1. Purpose of the M140-M146 line

The residual line was ordered to:

1. correct false applicability blockers before writing code;
2. exhaust I2PControl-local/application-layer capabilities first;
3. introduce only small neutral core seams where unavoidable;
4. defer cryptographic destination/LeaseSet work until simpler residuals were resolved;
5. stop rather than fabricate support when a real primitive did not exist.

The containment objective throughout was to keep Proposal-specific policy under `emissary-cli/src/i2pcontrol/**` and limit neutral core changes to exact M061/M062 owners.

## 2. Closed milestones

### M140 — residual streaming applicability re-freeze

Disposition: **closed as complete**, zero promotions.

Result:

- six constructor-overridden/UDP `Profile` cells -> affirmative `not_applicable`;
- `ConnectDelay:streamrclient` -> affirmative `not_applicable`;
- `Profile:client` retained as the only Profile blocker;
- matrix `325/47/468 -> 325/40/475`.

### M141 — UniqueLocalAddressPerClient

Disposition: **closed as complete**, 2 promotions.

Result:

- `httpserver`, `httpbidirserver` use canonical peer-hash-derived loopback source binding;
- target confinement remains literal loopback;
- no DNS/non-loopback fallback;
- matrix `327/38/475`.

Production ownership remained I2PControl-local.

### M142 — HTTP SSLProxies + JumpList

Disposition: **closed as complete**, 2 promotions.

Result:

- HTTP-client-only SSL outproxy selection and bounded last-failure behavior;
- JumpList remains address-helper/error-response metadata and is never fetched;
- I2P-only egress preserved;
- matrix `329/36/475`.

Production ownership remained I2PControl-local.

### M143 — retained Profile:client

Disposition: **closed as complete**, 1 promotion.

Result:

- exact `bulk`/`interactive` mapping to neutral `i2p.streaming.maxWindowSize` behavior;
- neutral streaming owner consumes the effective max-window value;
- only plain `client` applies; six other client families remain N/A per M140;
- matrix `330/35/475`.

This milestone introduced a narrowly authorized neutral core streaming seam.

### M144 — application presentation UseSSL

Disposition: **closed as complete**, 4 promotions.

Result:

- `httpclient`/`connectclient`: local TLS listener termination before parsing;
- `httpserver`/`httpbidirserver`: verified TLS to the literal-loopback local target;
- distinct from M129 management TLS and Yosemite SAM TLS;
- no plaintext fallback;
- matrix `334/31/475`.

Production ownership remained I2PControl-local.

### M145 — MultiHoming / shouldBundleReplyInfo

Disposition: **closed as complete**, 2 promotions.

Result:

- neutral `SessionManager` reply LeaseSet bundling policy;
- omitted/true preserves bundling;
- false suppresses ExistingSession update bundling while retaining mandatory NewSession handshake bundling;
- only `httpserver`/`httpbidirserver` apply;
- matrix `336/29/475`.

Accepted production evidence includes follow-up commit `7cbd80a6d72aa07d158ba9dc74f8bbacef767be2`, which fixes a no-std `String` import and formatting after the original M145 closure commit.

### M146 — UseOutproxyPlugin feasibility

Disposition: **closed as blocked**, zero promotions.

Result:

- all four applicable cells remain `blocked_primitive`;
- no real bounded local provider exists in current Emissary architecture;
- registry/dummy provider alone has zero support value;
- aliasing ordinary `ProxyList` would be accept-inert;
- direct OS DNS/TCP provider violates the accepted no-direct-clearnet invariant;
- matrix remains `336/29/475`.

M146 made no production change.

## 3. Historical line outcome

The M140-M146 sequence reduced the residual inventory from 47 blocked cells to 29 through:

- seven evidence-backed N/A corrections;
- eleven real runtime promotions;
- four correctly retained UseOutproxyPlugin blockers.

Current residuals after this line:

| Cluster | Cells |
|---|---:|
| `SigType` | 10 |
| `EncryptLeaseSet` | 5 |
| `OptionalLookup` | 5 |
| `LeaseSetClientAuths` | 5 |
| `UseOutproxyPlugin` | 4 |
| **Total** | **29** |

## 4. Containment outcome

The line preserved the intended boundary:

- M141, M142 and M144 stayed under `emissary-cli/src/i2pcontrol/**` for production behavior;
- M143 used only exact neutral SAM streaming owners;
- M145 used only exact neutral destination/session and SAM owners;
- M146 made no production change;
- no broad crypto/NetDB/I2NP/transport waiver was introduced;
- Yosemite remained exact optional Y005;
- no direct-clearnet fallback was introduced.

M061/M062 remain the exact path/dependency authority.

## 5. Why execution moved to a corrective roadmap

After M146, the remaining work is qualitatively different and the repository has a qualification/readiness gap:

- M139 predates M141-M145 production work and is no longer a current-head integrated qualification;
- M095 production-head metadata is stale;
- historical milestone tests contain obsolete aggregate-count assertions that make broad testing red despite accepted later capability changes;
- M145 required a small post-closure no-std correction;
- M147 explicitly requires a dedicated signature-domain/security/exact-owner audit before registration.

The successor roadmap therefore inserts:

```text
M153 current-head requalification
  -> M154 signature-domain/security owner re-freeze
  -> M147/M148 SigType infrastructure/capability
  -> M149/M150/M151 LeaseSet security
  -> M152 final whole-surface requalification
```

See `plans/subsystems/i2pcontrol-proposal-170-post-m146-corrective-roadmap.md`.

## 6. M146 terminal blocker

M146 remains closed blocked and is not implicitly reopened by the crypto/LeaseSet tail.

A future `UseOutproxyPlugin` successor requires a separate architecture/security decision that identifies a genuinely distinct safe provider. The project must not weaken the no-direct-clearnet invariant or create a dummy provider merely to remove four blockers.

If those four cells remain blocked through final M152, full Proposal support remains partial even if all other safe residual work closes.

## 7. Historical closure rule

All M140-M146 closure files remain immutable. Later corrective qualification may supersede current-head authority/metadata, but it does not rewrite historical milestone evidence.

## 8. Post-line qualification authority (M153 closed)

M153 (`plans/closure/i2pcontrol-proposal-170/153-closure.md`) is closed as complete with zero promotions and zero production changes. It supersedes M139 as the current runtime/security qualification authority at `336/29/475`, corrects M095 production-head metadata to `7cbd80a...`, and repairs historical-vs-current test authority. This roadmap remains the immutable historical record of the M140-M146 execution line; current execution continues under `plans/subsystems/i2pcontrol-proposal-170-post-m146-corrective-roadmap.md` with M154 registered.

## 9. Post-line signature audit authority (M154 closed, disposition C)

M154 (`plans/closure/i2pcontrol-proposal-170/154-closure.md`) is closed as complete with zero promotions and zero production changes. It freezes the destination-capable SigType domain as `{0, 1, 2, 3, 7, 11}`, records type-0 generation as rejected by security policy, types 1–3 as legacy-only, type 11 as missing a maintained primitive, and Ed25519-only end-to-end capability. Disposition C closes the M147 path as blocked: no bounded single primitive (A) and no honest split (B) can satisfy the configurable field. The ten `SigType` cells are terminal blockers under current security/dependency policy, alongside M146's four `UseOutproxyPlugin` cells. M148 remains deferred behind the unsatisfiable M147 gate; M149–M152 remain deferred (M149–M151 need explicit re-gating). Line history above is otherwise immutable.
