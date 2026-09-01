# I2PControl Proposal 170 Full-Support Completion Roadmap

Status: active; M095-M096, M098-M108 closed, M099 closed internally/partial, M097 and M104 closed as blocked; no dependency-ready successor

Planning origin: M094 closed planning head `630a8fd1cd4e5943fcde0b5c16f5fc1e88b5d207`.

Current corrective planning baseline: `0a5e8c9` — M108 implementation head; 158 applicable TunnelManager residual cells remain independently blocked.

Pinned external authority:

- I2P Proposal 170, `I2PControl Expansion`;
- status: `Open`;
- pinned revision: `2026-05-20`.

Canonical/internal authority:

- `plans/000-long-term-specification.md`;
- `plans/001-terminology-and-domain-model.md`;
- `plans/002-long-term-roadmap.md`;
- `plans/003-planning-process.md`;
- ADR-0001/0002/0003/0004;
- M061/M062/M063 containment authority;
- M093 current tunnel security reclosure;
- M095 machine-readable full-support matrix;
- M097-M108 closure evidence.

## 1. Purpose

Move the internal fork from truthful partial Proposal 170 support to full support against the pinned `2026-05-20` revision while keeping Proposal 170 policy concentrated under `emissary-cli/src/i2pcontrol/**` and refusing to turn conformance gaps into broad router-core or dependency redesign.

The workstream is Proposal 170 only. It is not general I2PControl parity and it is not an upstream contribution program.

## 2. Current production state

The current production state after M108 is:

- RouterInfo: 43 canonical additions / 42 available / 1 protocol-permitted neutral / 0 unavailable;
- AddressBook CRUD, subscriptions, all 13 `SetConfig` keys, and independent cross-book shadowing semantics operational under the confined AddressBook owner;
- all 12 canonical TunnelManager tunnel types have real data planes;
- all 7 canonical TunnelManager actions are implemented;
- M097 applied `TunnelLength`, `TunnelQuantity`, and typed `EncType` where applicable;
- M098 applied the bounded client proxy/outproxy/auth/privacy subset;
- M099 applied the bounded server presentation/access/filter/admission/rate subset;
- M106 applied six TCP-client `DelayOpen` cells;
- all 6 ClientServicesInfo selectors are implemented;
- API version `1` is the sole accepted Authenticate version;
- M107 fresh managed TLS material is restrictive on Unix, fails closed on managed symlink/non-regular paths, and covers `localhost`, `127.0.0.1`, and `::1`;
- M108 repaired pre-M107 managed TLS permissions on Unix before managed reads and uses create-time owner-only private-key modes;
- unsupported residual options fail before allocation rather than being persisted-and-ignored;
- full public-network/reference-router certification remains open;
- M104 remains closed as blocked because applicable TunnelManager residual cells remain.

The current M095 matrix after M106 is:

- 70 canonical option rows × 12 canonical types = 840 cells;
- 224 `apply`;
- 158 applicable `blocked_primitive`;
- 458 `not_applicable`;
- 0 `planned_apply`, unsupported, unknown, or accept-inert cells.

M093 remains the current tunnel security authority. M088's accepted lower-layer pre-accept residual and the bounded Streamr availability limitation are not reopened merely for option parity.

## 3. Corrective findings through the current baseline

### 3.1 M097 dependency finding

M097 proved that several common session/key options cannot be truthfully mapped through the currently accepted Yosemite/SAM path. It safely implemented the supported subset and closed as blocked rather than expanding the dependency/core boundary.

### 3.2 M098/M099 decomposition correction

The original M098/M099 graph treated successful M097 completion as a milestone-wide hard dependency. That was too coarse. Many client and server option cells already had exact owners inside existing I2PControl runtimes.

The corrected sequence therefore:

- let M098 implement exact client proxy/auth/privacy cells;
- let M099 implement exact server presentation/access/filter/admission/rate cells;
- transferred genuine missing-owner/session/key/LeaseSet cells to explicit residual ownership.

This preserved fail-closed semantics without using an external/library blocker to hold unrelated safe work hostage.

### 3.3 M104-M106 residual finding

M104 performed the bounded integrated closure attempt and stopped correctly. It established that ordinary planned implementation work had been consumed but 164 applicable cells remained blocked at that review point.

M105 audited those cells without changing production behavior and identified exactly six TCP-client `DelayOpen` cells with a contained existing owner. M106 implemented those six cells and closed with the production matrix at `224 apply / 158 blocked_primitive / 458 not_applicable`.

The remaining 158 cells still require exact primitive/semantic/architecture evidence. Neither M107 nor M108 is a residual-option successor and neither changes those dispositions.

### 3.4 Post-M106 protocol, AddressBook, and fresh managed-TLS findings

A review of baseline `06a697006b7b7733587aafed166f438561552193` against the pinned Proposal 170 text and current read-only I2P documentation identified three bounded defects with existing I2PControl-local owners:

1. Authenticate accepted API version `2`, while current I2PControl documentation specifies API `1` and Proposal 118/API 2 is rejected for backward-compatibility reasons.
2. AddressBook state validation rejected the same hostname across distinct books even though the runtime owner already had deterministic precedence and I2P naming uses first-match ordered lookup with conflicts/shadowing permitted.
3. fresh generated managed TLS material did not explicitly protect the private key with restrictive Unix permissions and the generated certificate named only `localhost`, not ordinary loopback IP identities.

M107 corrected all three at implementation head `27a0376` without changing the TunnelManager matrix or lower-layer ownership.

The same review also established continuing non-goals:

- unrelated base-method parity remains outside this Proposal 170-only workstream;
- the existence of token-expiration error `-32004` does not establish a normative token TTL;
- confined AddressBook SetConfig paths remain an accepted security restriction rather than being relaxed to reproduce Java example paths;
- no new evidence unblocks any of the 158 residual TunnelManager cells.

### 3.5 Post-M107 managed-TLS upgrade-path finding

A review of M107 implementation/closure found one narrower security gap:

- fresh managed directory/key final modes are restrictive, but an existing regular `i2pcontrol-certs/` directory or `key.pem` created before M107 is reused without tightening its Unix mode;
- the private-key temporary file is created with ordinary `OpenOptions` semantics and restricted to `0600` only after secret bytes have been written.

This was an upgrade-path confidentiality issue in the existing I2PControl TLS owner. It required no new dependency, router-core seam, protocol change, certificate parser, or TunnelManager work. M108 was therefore dependency-ready as a bounded corrective pass and subsequently closed at implementation head `0a5e8c9`.

## 4. Ownership boundary

### Preferred ownership

Proposal 170 administrative/application/runtime policy belongs under:

`emissary-cli/src/i2pcontrol/**`

This includes option validation/application, proxy/filter policy, server admission configuration, administrative file confinement, authentication/version gating, managed I2PControl TLS, matrix/audit reconciliation, and I2PControl interoperability harnesses.

### Lower-layer exception rule

A future production change outside `i2pcontrol` is allowed only when all are true:

1. the required behavior belongs to an existing lower-layer canonical owner;
2. no truthful I2PControl-local implementation exists;
3. exact paths/owners are named before implementation;
4. the seam is neutral/reusable rather than Proposal-170-shaped;
5. behavior is bounded and does not silently change unrelated router decisions;
6. M061 containment is not widened implicitly;
7. a registered successor plan explicitly authorizes it.

M102's existing network-error observation is the deliberate full-support lower-layer exception already accepted. M108 introduced no new lower-layer production path.

### Dependency rule

No plan in this roadmap automatically authorizes:

- vendoring/forking/patching Yosemite;
- replacing Yosemite with an internal duplicate SAM stack;
- adding Proposal-170-shaped APIs to `emissary-core`;
- adding dependencies merely to make matrix counts green.

A residual option requiring one of those actions remains blocked until a separately approved architecture decision changes the boundary.

## 5. Invariants

- exact pinned names, types, actions, response shapes, and presence semantics;
- I2PControl API negotiation must not advertise a rejected/non-supported API version;
- no fabricated state or inert accepted configuration;
- every applicable `apply` cell changes real runtime behavior;
- implementation difficulty is not evidence of `not_applicable`;
- a Java-specific implementation mechanism is not automatically an Emissary architecture requirement;
- Proposal 170 policy remains in I2PControl wherever possible;
- AddressBook types remain independent administrative state with deterministic effective lookup semantics;
- startup/control-plane ownership remains distinct;
- HTTP/IRC/Streamr anonymity and resource bounds remain mandatory;
- server local targets remain confined as accepted;
- direct I2P proxy traffic never falls through to clearnet DNS;
- clearnet proxy traffic requires an explicit I2P outproxy;
- secrets and path-valued configuration remain redacted/confined;
- managed I2PControl private-key material must be restrictive before it is read or exposed through managed publication on Unix;
- unsafe managed TLS file types or unrepairable managed permission state fail server initialization;
- LeaseSet security never silently downgrades;
- feature-disabled/default Emissary gains no I2PControl-only runtime/dependency behavior;
- external sources are read-only;
- all repository writes remain internal to `eggstack/emissary`.

## 6. Explicit non-goals

This workstream MUST NOT:

- implement unrelated base methods such as `GetKeys`, `GetRate`, `RouterManager`, `NetworkSetting`, or `AdvancedSettings`;
- add non-Proposal-170 methods, tunnel types, actions, statuses, or wire fields;
- implement or negotiate rejected API 2;
- invent a token TTL without separate normative evidence;
- relax AddressBook filesystem confinement merely to reproduce reference implementation examples;
- redesign real tunnel data planes solely to share code;
- reopen M088/M091 without new independent security evidence;
- add DCC/WEBIRC/SOCKS BIND/UDP ASSOCIATE absent a pinned requirement;
- add router-wide banning merely for telemetry or server throttling;
- couple I2PControl to frontend state;
- add broad hosted CI/fuzz/coverage/release infrastructure;
- initiate or prepare upstream review/merge/submission/adoption/contact.

## 7. Target architecture

```text
                         Proposal 170 JSON-RPC
                                |
                                v
                  emissary-cli/src/i2pcontrol/**
      +-------------------------+--------------------------+
      |                         |                          |
      v                         v                          v
AddressBook config       Tunnel option policy       RouterInfo sources
and confined state       + existing real backends   + bounded local owners
      |                         |                          |
      v                         v                          v
existing downloader      existing Yosemite/SAM      existing neutral core
seam where required      interfaces only            observation seams only
```

Authentication/version gating and managed I2PControl TLS remain at the same I2PControl-local boundary. No new Proposal 170 business object belongs in router core.

## 8. Full-support matrix rule

The M095 machine-readable matrix remains production-support authority.

Final closure requires:

- RouterInfo: all 43 rows available except proposal-permitted neutral semantics;
- AddressBook: all 13 SetConfig keys operational with explicit owners and correct independent-book semantics;
- TunnelManager: every canonical option/type cell is `apply` or evidenced `not_applicable`;
- zero applicable `planned_apply`, `blocked_primitive`, unsupported, or unknown cells;
- exactly 12 tunnel types and 7 actions;
- exactly 6 ClientServicesInfo selectors;
- compatibility aliases/extensions clearly separated from canonical completion.

Parser acceptance, persistence without runtime effect, fail-before-allocation rejection, or an audit-candidate classification is not final `apply` evidence.

M108 did not change any TunnelManager matrix row or cell.

## 9. Current dependency graph

```text
M095 exact matrix + containment budget              [CLOSED]
  |
  +--> M096 AddressBook SetConfig                    [CLOSED]
  +--> M100 transit 15s                              [CLOSED]
  +--> M101 router news                              [CLOSED]
  +--> M102 network-error owner                      [CLOSED]
  +--> M103 banned-peer semantics                    [CLOSED]
  |
  +--> M097 common session/key options               [CLOSED AS BLOCKED]
         | supported: TunnelLength/TunnelQuantity/EncType
         | residual primitive blockers retained
         |
         +----------------------+----------------------+
                                |                      |
                                v                      |
M098 client/proxy/HTTP independent slice             |
[CLOSED]                                             |
  |                                                  |
  v                                                  |
M099 server/access/throttle independent slice        |
[CLOSED INTERNALLY — PARTIAL]                        |
  |                                                  |
  v                                                  |
M104 live interoperability + final reclosure         |
[CLOSED AS BLOCKED — RESIDUAL CELLS]                |
  |                                                  |
  v                                                  |
M105 residual primitive/applicability audit          |
[CLOSED — SIX LOCAL TCP CLIENT CELLS IDENTIFIED]    |
  |                                                  |
  v                                                  |
M106 DelayOpen client-listener lifecycle             |
[CLOSED — SIX TCP CLIENT FAMILIES; STREAMR EXCLUDED] |
  |
  v
M107 post-M106 conformance/fresh-TLS corrective      [CLOSED]
  |
  | legacy managed-permission gap
  v
M108 managed TLS upgrade-permission corrective       [CLOSED]
  |
  +--> no dependency-ready successor
       matrix remains 224 apply / 158 blocked / 458 N/A
```

M108 followed M107 as a security-corrective repository-baseline dependency, not as a residual-option dependency. It closed without changing the M104 residual gate, and no future plan is unblocked by this corrective pass.

## 10. Milestone status and exit conditions

### M095 — full-support matrix and containment budget

Status: closed.

Exit evidence remains the exact 43 / 13 / 12×options / 6 inventory and path budgets.

### M096 — AddressBook SetConfig

Status: closed.

All 13 pinned keys have bounded operational/metadata semantics. The sole non-I2PControl production amendment reused the existing AddressBook downloader seam rather than creating competing ownership. M107 corrected cross-book shadowing while preserving M096 confinement/transactionality; M108 does not reopen AddressBook behavior.

### M097 — common tunnel session/key options

Status: closed as blocked.

Implemented safe subset:

- `TunnelLength`;
- `TunnelQuantity`;
- typed `EncType`.

Retained blockers include `Shared`, `UseSSL`, `TunnelVariance`, `TunnelBackupQuantity`, `SigType`, `CustomOptions`, `NewDest`, `PersistentClientKey`, and `PrivKeyFile` where current accepted ownership/serialization is absent.

M097's residual stop conditions remain valid; M108 does not reopen them.

### M098 — client proxy, management, and HTTP independent slice

Status: closed.

Exact proxy/outproxy/auth/privacy behavior owned by existing I2PControl client runtimes is operational. Plugin/TLS-proxy/jump-list/client-management semantics without exact owners remain blocked.

### M099 — server access, throttle, and LeaseSet independent slice

Status: closed internally against pinned revision; partial.

Exact server presentation/access/filter/admission/rate behavior owned by accepted I2PControl server runtime is operational. LeaseSet/TLS/address-routing semantics without exact safe owners remain blocked.

### M100 — transit 15s

Status: closed.

Request-independent I2PControl-owned sampler over the authoritative cumulative transit source.

### M101 — router news

Status: closed.

Bounded signed router-news acquisition/cache using the existing SU3/certificate seam. Full public-network evidence remains part of a successful future reclosure.

### M102 — network error owner

Status: closed.

Minimal neutral v4/v6 observation in existing core owners; Proposal 170 mapping remains in I2PControl.

### M103 — banned peers

Status: closed.

Explicit by-design-empty router-wide banned-peer source after proof that Emissary has no such ban owner. No ban algorithm added.

### M104 — full live interoperability/reclosure

Status: closed as blocked.

Closure: `plans/closure/i2pcontrol-proposal-170/104-closure.md`.

M104 reached its final verification stop condition with applicable `blocked_primitive` cells. It remains the authority that the repository cannot yet claim full support.

### M105 — residual TunnelManager primitive and applicability audit

Status: **closed**.

Plan:

`plans/implementation/i2pcontrol-proposal-170/105-residual-tunnel-option-primitive-audit.md`

M105 created:

`plans/implementation/i2pcontrol-proposal-170/105-residual-option-audit.toml`

covering the 164 M104 residual cells and classifying their exact semantics, applicability, current owner/blocker, dependency capability, and security impact without production changes.

M105 closed with one dependency-ready residual successor: M106, limited to `DelayOpen` for the six TCP-style client families. Streamr `DelayOpen` remained `semantic_blocked`.

### M106 — DelayOpen client-listener lifecycle

Status: **closed**.

Plan:

`plans/implementation/i2pcontrol-proposal-170/106-delay-open-client-listener.md`

Closure:

`plans/closure/i2pcontrol-proposal-170/106-closure.md`

M106 applied lazy first-local-use session creation through the existing I2PControl listener owner for six TCP-client families. It did not authorize Streamr, Yosemite, core, util, or dependency changes. Production moved to `224 apply / 158 blocked / 458 not-applicable`.

### M107 — I2PControl conformance and managed-TLS corrective pass

Status: **closed**.

Plan:

`plans/implementation/i2pcontrol-proposal-170/107-i2pcontrol-conformance-and-managed-tls-corrective-pass.md`

Closure:

`plans/closure/i2pcontrol-proposal-170/107-closure.md`

Class: corrective capability/security.

M107 corrected exactly three post-M106 defects through existing I2PControl-local owners:

- API version `1` is the sole accepted Authenticate version and API `2` is rejected with `-32006` before token issuance;
- valid cross-book hostname shadowing is permitted while independent books, deterministic effective precedence, persistence, and M096 confinement remain intact;
- fresh managed TLS key file type/final permissions are restrictive and fresh managed certificates cover `localhost`, `127.0.0.1`, and `::1` using existing dependencies.

M107 closed at implementation head `27a0376` and changed no TunnelManager support disposition.

### M108 — managed TLS upgrade-permission corrective pass

Status: **closed**.

Plan:

`plans/implementation/i2pcontrol-proposal-170/108-managed-tls-upgrade-permission-corrective-pass.md`

Closure:

`plans/closure/i2pcontrol-proposal-170/108-closure.md`

Class: corrective capability/security.

M108 was limited to the existing I2PControl managed TLS owner and planning-state reconciliation. It:

- restrict and revalidate an existing managed `i2pcontrol-certs/` directory to `0700` on Unix before managed child material is read, or fail initialization;
- restrict and revalidate an existing regular managed private key to `0600` before key bytes are read, or fail initialization;
- create the private-key temporary file with requested Unix mode `0600` at inode creation through the standard library rather than relying on post-write chmod as the first confidentiality boundary;
- preserve valid managed certificate/key bytes across permission-only repair and restart;
- preserve explicit operator TLS ownership unchanged;
- reconcile stale pre-M108 planning state in the implementation README, registry, and this roadmap;
- leave the M095 matrix exactly `224 apply / 158 blocked_primitive / 458 not_applicable`.

Implementation head: `0a5e8c9` — `fix(i2pcontrol): repair managed TLS upgrade permissions`.

Hard dependency: M107 closure at `27a0376` and current repository baseline.

Exit conditions:

- legacy permissive managed directory/key fixtures are repaired before key reads and remain byte-stable;
- create-time private-key mode is explicitly tested/guarded without timing-sensitive observation;
- M107 symlink/type/SAN/restart behavior remains passing;
- feature-gated I2PControl, live local runtime, containment, matrix/audit, check, and clippy evidence are recorded;
- no production path outside `emissary-cli/src/i2pcontrol/**` changes;
- no Cargo/dependency/Yosemite/core/util/frontend/workflow change occurs;
- planning state no longer describes M107 as ready/current/pending;
- external research remains read-only and all repository writes remain internal.

M108 did not unblock or constitute a future M104 reattempt.

## 11. Residual TunnelManager families

M104's pre-M106 audit baseline grouped 164 cells as:

| Family | Cells at M104 baseline |
|---|---:|
| `Shared` | 7 |
| `UseSSL` | 4 |
| `TunnelVariance`, `TunnelBackupQuantity`, `SigType`, `CustomOptions` | 40 |
| `NewDest`, `PersistentClientKey` | 14 |
| `PrivKeyFile` | 10 |
| `UseOutproxyPlugin`, `SSLProxies`, `JumpList` | 12 |
| `ConnectDelay`, `Profile`, `DelayOpen`, `Reduce*`, `Close*` | 56 |
| `AllowInternalSSL`, `UniqueLocalAddressPerClient`, `MultiHoming` | 6 |
| `EncryptLeaseSet`, `OptionalLookup`, `LeaseSetClientAuths` | 15 |

M106 removed six TCP-client `DelayOpen` cells from that blocked inventory. The authoritative current total is 158 blocked cells. The machine-readable M095/M105 artifacts, not this grouped table, remain the cell-level authority.

Implementation difficulty alone is not evidence that a cell is `not_applicable`. Conversely, a Java-specific implementation mechanism is not automatically a required Emissary architecture.

## 12. Security and anonymity requirements

All future work must preserve the security and anonymity requirements established through M108:

- trusted Yosemite-derived peer identity;
- bounded transactional server admission;
- literal-loopback server targets;
- HTTP framing/spoof/fingerprint/Expect/POST protections;
- IRC registration/DCC/CTCP/lifetime protections;
- bounded Streamr subscriber/payload/fanout state;
- secret-safe persistence/get/logging;
- restrictive managed I2PControl private-key handling before read/publication on Unix;
- explicit I2P outproxy requirement for clearnet proxy traffic;
- no local DNS fallback for direct I2P destinations;
- no silent LeaseSet security downgrade;
- cancellation/edit/restart generation isolation;
- bounded shared-session/key ownership if such work is ever authorized.

An option or compatibility path that achieves literal compatibility by weakening an accepted M093 anonymity/resource invariant is not dependency-ready.

## 13. Failure, lifecycle, and contention requirements

For future tunnel implementations:

- validate before allocation/publication where technically possible;
- failed edits preserve prior durable/runtime generations;
- per-name tunnel lifecycle remains serialized;
- timers/workers/tasks remain generation-local and cancellable;
- no lock crosses network I/O, sleeps, joins, or cancellation waits;
- bounded cleanup on stop/restart;
- blocked options continue to fail before allocation;
- no partial state presented as successful configuration.

For AddressBook behavior, the existing mutation mutex remains authoritative and failed publication must leave live/durable state unchanged. Cross-book shadowing is a lookup property over independent books, not a merged persistence model.

For managed TLS corrective work, unsafe managed file types, unrepairable permission state, or publication failures must fail server initialization rather than fall back to plaintext or continue with permissive key material. M108 repair is synchronous startup state and must not add a long-lived task or new lock.

## 14. Verification policy

Use focused unit/integration/static guards plus the existing feature-gated containment/matrix suite. Do not build a CI farm for this phase.

Current baseline commands include:

```text
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo check
cargo test -p emissary-cli --no-default-features --features i2pcontrol
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m095_full_support_matrix --test m105_residual_option_audit
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment --test m062_dependency_containment
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test i2pcontrol_live_runtime -- --nocapture
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
cargo fmt --all -- --check
git diff --check
```

M108 closure added focused legacy-directory/key permission-repair, byte-stability/restart, create-time `0600`, and retained M107 symlink/SAN evidence as specified in its plan. M095/M105 counts must not change.

The historical `m063_feature_reachability` test target is absent in the current checkout; preserve that limitation rather than inventing unrelated replacement scope.

The repository-wide nightly/stable rustfmt mismatch is a tooling issue. Run the formatter check and record its outcome, but do not rewrite audited core files solely for formatter churn.

## 15. Risks and deferred work

### Residual Yosemite/SAM capability gaps

Risk: full support may remain blocked because the accepted dependency cannot express exact pinned session/LeaseSet semantics.

Response: retain exact missing-wire/dependency classifications. No automatic vendor/fork path follows M108.

### Applicability mistakes

Risk: some blocked cells may reflect Java I2PTunnel implementation assumptions rather than true Proposal 170 applicability to a given Emissary tunnel family.

Response: require affirmative Proposal/reference evidence before changing any disposition. Difficulty alone is insufficient.

### Client identity/shared-session ownership

Risk: shared sessions or persistent/new client identities can create cross-tunnel ownership, identity rotation, secret persistence, and restart hazards.

Response: distinguish a bounded I2PControl control-plane owner from a router-wide destination/key subsystem. The latter requires an architecture decision.

### Proxy/TLS/plugin semantics

Risk: mechanically copying Java plugin/TLS machinery could introduce direct-clearnet fallback or trust bypass.

Response: audit contract behavior separately from Java implementation mechanism and preserve explicit I2P outproxy/trust boundaries. M107/M108 managed-TLS work is only the I2PControl administrative listener certificate/key owner, not TunnelManager `UseSSL`/`SSLProxies` support.

### Managed TLS upgrade permissions

Risk: legacy managed files were created under process umask and may be broader than the M107 fresh-material final modes. Standard-library path-based checks also do not establish a new general-purpose defense against an attacker with arbitrary write authority inside the router base directory.

Response: M108 tightens and revalidates the managed directory before child reads, tightens/revalidates the managed key before key reads, creates secret temporary files with `0600` from inception, and fails startup if those controls cannot be established. The operator-controlled router base directory remains the explicit trust boundary; adding a new `openat`/ACL filesystem abstraction is outside M108.

### Server option security

Risk: TLS/local-address/multihoming options could weaken M093 target/anonymity boundaries.

Response: no request-selected LAN routing or unsafe address allocation is authorized by compatibility pressure.

### Existing base API compatibility

Risk: Proposal 170's compatibility section says existing I2PControl applications should continue to work, while this fork intentionally does not implement every pre-existing base method.

Response: this roadmap remains Proposal 170-only and explicitly does not expand into general base-method parity. M107 corrected the objectively wrong API-version advertisement but did not implement unrelated methods. Any future base-parity work requires a separately authorized non-Prop-170 roadmap.

### Token expiration

Risk: long-lived bearer tokens increase the impact of token disclosure.

Response: current documentation exposes an expired-token error but this review established no normative expiration interval. M108 did not invent such a policy. A future security-hardening decision may define one separately if maintainers want it.

### AddressBook filesystem examples

Risk: Java/reference examples use relative paths that may escape a dedicated administrative root.

Response: M096 confinement remains controlling. Compatibility pressure does not authorize arbitrary authenticated filesystem writes.

### Live-network evidence

Risk: local verification may not have a functional I2P network/reference router.

Response: full support remains blocked rather than presenting process-local success as network interoperability.

## 16. Full-support exit condition

This roadmap closes only when a future M104 reattempt can truthfully record:

> Emissary fully supports I2P Proposal 170 against the pinned 2026-05-20 revision.

That claim requires zero applicable blocked/unsupported/planned TunnelManager cells and focused live interoperability. It does not mean general I2PControl parity or upstream acceptance.

M107 and M108 are corrective prerequisites over implemented control-plane behavior; neither can itself satisfy the residual full-support exit condition.

## 17. Internal-only boundary

All work and writes remain internal to `eggstack/emissary`. External specifications, Yosemite, Java I2P, i2pd, I2P+, issues, commits, and pull requests are read-only evidence only.

No upstream issue/PR/review/submission/merge/adoption request, contribution preparation, branch/tag push, release, or maintainer contact is authorized by this roadmap.