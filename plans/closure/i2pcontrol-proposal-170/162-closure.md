# M162 Closure — Proposal LeaseSet-Security Field Integration (Blocked; Zero Promotions)

Status: **closed as complete; zero Proposal promotions; all fifteen LeaseSet-security cells remain blocked_primitive; M152 hard dependency satisfied, registration pending**

Date: `2026-09-09`

Source implementation plan:

- `plans/implementation/i2pcontrol-proposal-170/162-proposal-leaseset-security-field-integration.md`
  (Status now `closed as complete` with closure link; hard dependencies on
  M160 closure and M161 outcome B satisfied by
  `plans/closure/i2pcontrol-proposal-170/160-closure.md` and
  `plans/closure/i2pcontrol-proposal-170/161-closure.md`;
  conditional promotion budget observed as zero as proven below).

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-post-m154-leaseset-security-corrective-roadmap.md`.

Promotion budget: **zero Proposal cells** (conditional `OptionalLookup` +
`LeaseSetClientAuths` up to 10 only on full-contract proof; `EncryptLeaseSet`
held blocked per M161-B; proof not met — see §5).

Production budget: **nine I2PControl paths (strict subset of the registered
ten-file envelope)** — typed/redacted domain, common fail-before-allocation
validation with the executable ten-mode table, canonical session-translation
defensive gate, five-family backend consistency, and control-plane
persistence/redaction hardening. No new file. No dependency. No manifest,
lockfile, Yosemite, core, or second secret-store change. `server_secret_store.rs`
is intentionally unchanged (subset; see §5).

## Planning baseline

- Registration baseline `631c8fb9` (registered M162 handoff; clean worktree
  before M162 work; M062 records the ten-file I2PControl-only envelope with
  zero new dependency/manifest/lock/Yosemite/core change).
- Last production-bearing head before M162: `1629b0a5` (M160 DH
  implementation/closure; unchanged through M161 registration/closure, which
  touched only plans/docs/guard-test paths, and through M162 registration,
  which touched only plans).
- Incoming M095 matrix: `336 apply / 29 blocked_primitive / 475
  not_applicable` across 840 TunnelManager option/family cells.
- M153 closed complete as the current runtime/security qualification
  authority; M154 closed disposition C (M147 path blocked);
  M155-M160 closed complete with zero promotions; M161 closed outcome B
  (valid but blocked legacy AES, zero production/promotions); M146 closed
  blocked.
- Reviewed head: the working tree described in §9. Exact changed production
  paths are the nine registered I2PControl owners below (typed extensions +
  validation gates + tests only; no wire-emission, custody, or NetDB change).
  Exact guard/planning paths are
  `emissary-cli/tests/m062_dependency_containment.rs` (M162 authorization
  helper in both chains),
  `plans/implementation/i2pcontrol-proposal-170/062-dependency-containment.toml`
  (current-registration closure bookkeeping), the M162 plan (status flip),
  this closure, registry/README/roadmaps/AGENTS/docs authority updates (see
  §9).

Pinned authority (all accessed read-only; no upstream mutation, contact, or
submission occurred — see read-only attestation):

- Proposal 170 revision `2026-05-20`, status Open, SHA-256
  `f13ae00b886c5e72131bc5d5b138a371148d1faa6899a119a1dacb65a555e7dc`
  (cited from M095/M153/M155; no new Proposal fetch — M162 adds no Proposal
  policy beyond the frozen registration mapping);
- Direct Java I2P/I2PTunnel standard-property contract from the frozen M162
  registration (`ServiceTunnelCreator` mapping): legacy
  `i2cp.encryptLeaseSet=true` only for `encrypted (aes)`; modern
  `i2cp.leaseSetType=5`; `i2cp.leaseSetSecret=Base64(UTF8(secret))`;
  `i2cp.leaseSetAuthType=1/2` + `i2cp.leaseSetPrivKey=Base64(32B)` base +
  `i2cp.leaseSetClient.dh.N/psk.N=[Base64(UTF8(name)) ":"]Base64(32B)`
  (cited from the frozen plan; no new fetch);
- I2P Encrypted LeaseSet specification DH/PSK layer-1 construction (flags
  `0x01`/`0x03`, `ELS2_XCA`/`ELS2PSKA` 52-byte schedules, 4096-byte
  `MAX_ENCRYPTED_SIZE` ceiling; cited from M159/M160 records; no new fetch);
- Java `net.i2p.data.LeaseSet`/`EncryptedLeaseSet` legacy-AES disposition
  (cited from M161 closure §1; no new fetch — M161-B is inherited);
- Yosemite optional exact revision `59140a2277bf296928d2e8ce39a148182eeff044`
  (unchanged; no Yosemite change in M162; typed LeaseSet wire fields verified
  read-only in the checked-out fork at
  `~/.cargo/git/checkouts/yosemite-b3f22cc17f665e22/59140a2/src/options.rs`
  and `src/proto/session.rs` — see §4);
- Emissary source at the reviewed tree (all file:line citations below read
  at HEAD plus the workdir diff closed here).

Current Proposal matrix at closure (mechanically recomputed via the
`m095_full_support_matrix` suite, dispositions unchanged):

- `336 apply / 29 blocked_primitive / 475 not_applicable` (840 cells);
- residual split `SigType` 10, `EncryptLeaseSet` 5, `OptionalLookup` 5,
  `LeaseSetClientAuths` 5, `UseOutproxyPlugin` 4 — identical to
  M153/M154/M155/M156/M157/M158/M159/M160/M161 entry.

## 1. Requirement-to-evidence matrix (plan required evidence)

| Plan-required trace | Evidence |
|---|---|
| Ten-mode machine table before any production edit, recording spelling, legacy flag, type/auth, primitive, lookup/indexed coupling, key interpretation, base role, B32 flags, generation/import, create/edit/restart, DatabaseStore/storage key, M161 dependency | `LEASE_SET_SECURITY_MODES` (`backends/options.rs`): ten rows in Proposal wire order with all required columns; executable acceptance authority iterated by `m162_ten_mode_table_is_exact_and_blocked` (spellings, legacy/type/auth split, M161 dependency, B32 derivation, store types) |
| Typed domain, not rawConfig; redacted/non-Debug secrets; Get returns only safe representation | `EncryptLeaseSetMode` enum (exact ten spellings, serde round-trip), `LeaseSetClientAuthEntry` (custom redacted Debug/Display omitting names+keys), `TunnelOptions.encrypt_lease_set/optional_lookup/lease_set_client_auths` (redacted, serde-persistent); `extract_tunnel_options` parses strictly (mode exact, lookup bounds, per-entry Name/Key + 32-byte Base64 via core decoder, duplicates preserved syntactically); `extract_raw_config` + `is_typed_secret_key` exclude `OptionalLookup`/`LeaseSetClientAuths` from `raw_config`; `insert_typed_canonical_options` returns only the safe mode string in Get; `SENSITIVE_OPTION_KEYS` omits both secrets from Get/compat; proven by `m162_*` domain + tunnel-manager tests (§8) |
| Dedicated server LeaseSet-security secret state keyed by stable identity, with lookup/base/per-user/generation metadata, no raw secret serialization, stage/commit/discard + current/backup discipline; prefer extending `server_secret_store.rs`; new file requires amendment | **Deferred by design (subset; no new custody allocated).** Rationale in §5: allocating persistent base/lookup/per-user generations without a Yosemite base-key emission path would be accepted-inertly infrastructure (plan stop condition). Schema design is frozen in this closure for a future Yosemite-amended successor (separate typed secret record keyed by `__emissary_server_destination_identity`, generation/revision metadata, reuse of existing stage/commit/discard + current/backup atomic-publish discipline, no `raw_config` secrets, redacted Get, transactional ordering §1-7); `server_secret_store.rs` is therefore intentionally unchanged in this commit |
| Transaction ordering 1-7 (validate before allocation; prepare off to side; persist atomically or keep last-good; commit generation only with secret revision; cancel stale before republish; failed start preserves last-good; delete cleans secrets) | **Vacuously satisfied (blocked).** Every LeaseSet-security presence fails in `validate_common_options`/`validate_raw_options`/`build_session_options` before any secret-store lookup, runtime reservation, or SAM wire work; no secret generations are allocated, so no split-brain, stale-republish, or destroy-on-failed-start path exists; delete removes no LeaseSet secrets because none were ever committed. Proven by preflight==start agreement tests across all five families (§8) |
| Expected owner set (ten-file freeze) with M061/M062 authorization; `stores/tunnel_store.rs` needs no change; lower-layer core change is stop/split | Realized nine-file strict subset (see §7/§9): `domain/tunnel.rs`, `backends/options.rs`, `backends/runtime/session.rs`, `backends/server.rs`, `backends/http_server.rs`, `backends/http_bidir.rs`, `backends/irc_server.rs`, `backends/streamr.rs`, `tunnel_manager.rs`. `server_secret_store.rs` unchanged (subset allowed per roadmap containment rules). `stores/tunnel_store.rs` unchanged (typed fields persist through existing generic upsert — proven by tunnel-manager round-trip test). No core/Yosemite/dependency/manifest change (proven by §12) |
| Promotion rules: OptionalLookup (exact mapping, M158 publication, custody, transactions, B32, interop, no downgrade); LeaseSetClientAuths (exact parsing/formatting, 4096 bound, custody/redaction, edit/restart, interop, no inert/sparse-truncation/drop); EncryptLeaseSet (every valid enum value operational; legacy AES valid-but-unsupported keeps all five blocked; no partial-enum apply) | **Zero promotions (see §5/§11).** OptionalLookup cannot promote: valid lookup uses include PSK/DH lookup variants that are Yosemite-inexpressible (base gap) plus custody absent. LeaseSetClientAuths cannot promote: Yosemite base gap + duplicate rejection (core preserves duplicates) + 16-vs-99 bound gap (valid 17-99-entry Proposal lists would reject) plus custody absent. EncryptLeaseSet cannot promote: legacy AES valid-but-blocked per M161-B plus all modern gaps above; no partial-enum apply is claimed |
| Security/transactionality invariants (no secret in logs/errors/metrics/Get/rawConfig; names not in unrelated diagnostics; fail before allocation; no downgrade; httpbidir reuses canonical owner; no stale republish; no Debug/Display material except non-reversible identity; sharing distinguishes generations without logging material) | Held and proven: `OptionRedacted`/entry redaction + `TunnelDefinition` raw-keys-only Debug + backend error name-only messages + Get omission + session `CompatibilityKey` redacted Debug/Display (identity values in equality only) + httpbidir reuses canonical server destination/session owner (no second owner added) + preflight==start gates + ordinary wire emits no LeaseSet keys (§8) |
| Required tests: ten modes × five families (create/edit/restart/stop/start/process restart, missing/extra lookup, PSK/DH structures, base-key modes, malformed lengths/duplicates/sparse, store failure/cancellation/rollback, redaction, B32, wire fixtures, interop, deletion/cleanup, fail-before-allocation, no modern encrypt flag, legacy never maps to type 5, M157-M160 regressions) | Executed as blocked-closure evidence (§8): table-driven ten-mode × five-family rejection tests (typed + raw, preflight==start, no echo); strict Proposal parsing tests (malformed Base64/lengths/controls/too-many fail at control plane INVALID_PARAMS before persistence); redaction tests (domain/Debug/Get/compat/session-key); ordinary-wire fixture test (no encrypt/type/auth/secret/client/privkey emission, no legacy aliasing); store-failure/rollback N/A (no secret generations allocated — vacuously satisfied); B32/interop/deletion N/A (blocked — no publication/address change); full M157-M160 + ordinary regression suites green (1159 core + 835 CLI lib + 35 guards) |

Do not infer support from GUI/property persistence alone: observed —
`tunnel_manager.rs` persists the typed mode/secrets and Yosemite can
serialize *some* LeaseSet fields, but no Proposal LeaseSet-security presence
reaches Yosemite/core publication in M162 (all fail before allocation); the
M124 Y005 reachability test remains an I2PControl-only typed-API
reachability proof without any Proposal mapping (its comment now names M162
as the blocked Proposal owner).

## 2. Published DatabaseStore type/object and lookup/decryption evidence

- No Proposal LeaseSet-security presence produces a DatabaseStore in M162:
  every mode (including explicit `disable`, which must be omitted for
  ordinary publication) fails in `validate_common_options` (typed),
  per-backend `validate_raw_options` (raw), and `build_session_options`
  (defense-in-depth) before any secret-store lookup, runtime reservation, or
  SAM `SESSION CREATE` emission. The only SAM wire fixture in this closure is
  the negative ordinary wire (§8): no `i2cp.encryptLeaseSet`,
  `i2cp.leaseSetType`, `i2cp.leaseSetAuthType`, `i2cp.leaseSetSecret`,
  `i2cp.leaseSetClient.*`, `i2cp.leaseSetPrivKey`, or
  `i2cp.leaseSetPrivateKey` emission, and no legacy-to-type-5 aliasing.
- Per-mode publication/storage-key record (ten-mode table, for a future
  Yosemite-amended successor; not operational here): `disable` → ordinary
  `LeaseSet2` type 3 with destination-hash DHT key (field omitted);
  `encrypted (aes)` → legacy `LeaseSet` type 1 with keyring-gated AES leases
  (M161-B, unimplemented); all eight modern modes → `EncryptedLeaseSet2`
  type 5 with blinded DHT storage key and UTC-day rollover (frozen M157
  owner; `LeaseSetManager` remains the sole publication/rollover/verification
  owner — no second scheduler was added).
- Floodfill/lookup/decryption behavior is therefore unchanged from the
  M157-M160 head: ordinary `LeaseSet2` queries and type-preserving
  `EncryptedLeaseSet2` cache/flood/lookup only; no LS1/keyring path was added
  and no Proposal-driven type-5 publication was enabled.

## 3. Current owner inventory (read at the reviewed tree)

| Capability | Exact current owner | Status in M162 |
|---|---|---|
| Typed Proposal LeaseSet-security domain | `emissary-cli/src/i2pcontrol/domain/tunnel.rs` (`EncryptLeaseSetMode`, `LeaseSetClientAuthEntry`, `TunnelOptions` extensions) | exact parsing/redaction/persistence; always blocked at backends |
| Ten-mode executable table + common fail-before-allocation gate | `emissary-cli/src/i2pcontrol/backends/options.rs` (`LEASE_SET_SECURITY_MODES`, `validate_lease_set_security`, `present_runtime_fields`/`is_common_runtime_field` wiring) | ten rows; every typed presence rejects with field name only |
| Canonical session-translation defensive gate | `emissary-cli/src/i2pcontrol/backends/runtime/session.rs` (`reject_leaseset_raw_presence` in `build_session_options`) | typed (via common) + raw both reject before SAM wire; ordinary wire emits no LeaseSet keys |
| `server` family gate | `emissary-cli/src/i2pcontrol/backends/server.rs` (`validate_raw_options` generic-unknown rejection + `validate_common_options` + `build_session_options` dummy-destination preflight) | typed+raw reject; preflight==start proven |
| `httpserver` family gate | `emissary-cli/src/i2pcontrol/backends/http_server.rs` (`REJECTED` now includes `OptionalLookup`; typed via common) | previously-inert `OptionalLookup` gap closed; all three reject |
| `httpbidirserver` family gate | `emissary-cli/src/i2pcontrol/backends/http_bidir.rs` (generic-unknown rejection + common) | all three reject; reuses canonical server destination/session owner (no second crypto owner) |
| `ircserver` family gate | `emissary-cli/src/i2pcontrol/backends/irc_server.rs` (generic-unknown rejection + common) | all three reject; preflight==start proven |
| `streamrserver` family gate | `emissary-cli/src/i2pcontrol/backends/streamr.rs` (explicit M162 raw rejection + common) | previously-inert raw gap closed; bounded Streamr limits untouched (16-subscriber, 60s expiry, 1200-byte payload, 4095-byte buffer, 15s refresh, bounded shutdown, loopback-only UDP) |
| Control-plane persistence/redaction/merge | `emissary-cli/src/i2pcontrol/tunnel_manager.rs` (strict `extract_tunnel_options`, `is_typed_secret_key` raw exclusion, `merge_tunnel_options`, safe mode-only Get insertion, `SENSITIVE_OPTION_KEYS`) | secrets never in `raw_config`/Get/logs; mode string safe-returned; malformed shapes fail at control plane before persistence |
| LeaseSet-security secret custody | `emissary-cli/src/i2pcontrol/server_secret_store.rs` | **unchanged (subset)** — schema/transaction design frozen in §5 for a future Yosemite-amended successor; no inert custody allocated here |
| Definition persistence | `stores/tunnel_store.rs` (`TunnelStore::upsert`) | unchanged (typed fields persist through existing generic upsert; proven by round-trip test) |

Explicitly untouched without amendment (verified by §12): any
`emissary-core/**`, `emissary-util/**`, Yosemite, manifest/lockfile/
dependency, second secret-store file, any `emissary-cli/src/**` path outside
`i2pcontrol/`, NetDB/query/decryption behavior.

## 4. Yosemite gap analysis (stop condition met; no Yosemite change made)

Checked-out Yosemite `59140a2` read-only:

- `src/options.rs`: typed `lease_set_type` (`1..=255`), `lease_set_auth_type`
  (`0..=2`), `lease_set_secret` (bounded Base64), `lease_set_client_auths`
  (`Vec<LeaseSetClientAuth>`, `MAX_LEASE_SET_CLIENT_AUTHS=16`,
  duplicate-free per mode, `is_valid_i2p_base64(key,32)`), `encrypt_lease_set`
  bool, `lease_set_key`/`lease_set_private_key`/`lease_set_signing_private_key`
  (bounded Base64). **No typed `leaseSetPrivKey` base-key field exists.**
  `is_reserved_session_option_key` reserves `i2cp.leaseSetPrivKey` (and
  truncated aliases) on the generic `add_session_option` path, so the base
  cannot be smuggled generically either.
- `src/proto/session.rs`: serializes typed `encrypt_lease_set` →
  `i2cp.encryptLeaseSet=true`; `lease_set_auth_type`/`lease_set_blinded_type`/
  `lease_set_type` when non-default; `lease_set_key` → `i2cp.leaseSetKey`;
  `lease_set_private_key` → `i2cp.leaseSetPrivateKey` (distinct from the
  core-required `leaseSetPrivKey`); `lease_set_secret` →
  `i2cp.leaseSetSecret`; per-mode `leaseSetClient.dh.N/psk.N` with
  deterministic numbering. **No `i2cp.leaseSetPrivKey` emission exists.**
- Core (`sam/parser.rs`, frozen M159/M160) requires the base
  `i2cp.leaseSetPrivKey` (exactly 32B) for every PSK (`auth 2`) and DH
  (`auth 1`) type-5 session and rejects `i2cp.leaseSetPrivateKey`/
  `i2cp.leaseSetKey` companions for those modes; extraction removes
  base/indexed values from generic state into zeroizing/non-`Debug` types.
  A Yosemite `type5+auth1/2` session without the base therefore fails closed
  at the core parser (`missing base key`); per-user entries alone cannot
  activate PSK/DH publication.

Consequences (frozen; each independently keeps its field blocked per the
promotion rules):

1. **Base-key gap (PSK ×4 modes + DH ×2 modes):** six of eight modern modes
   cannot be operational through the frozen `M124 typed path` composition
   seam. Emitting them would require a Yosemite typed `leaseSetPrivKey`
   addition — explicitly out of scope without amendment. Storing base keys
   in I2PControl without a wire path would be accepted-inertly (plan stop
   condition); M162 therefore allocates no base-key custody.
2. **Duplicate gap (per-user ×4 modes):** Yosemite `add_lease_set_client_auth`
   rejects duplicate names per mode (`duplicate-free`), while core/M159/M160
   preserve duplicate key bytes exactly (pinned Java semantics, each consuming
   one 40-byte record and the 4096 budget). A syntactically valid Proposal
   list with duplicates would be rejected by Yosemite — closure must keep the
   field blocked rather than claim full support (promotion rule).
3. **Bound gap (per-user ×4 modes):** Yosemite caps 16 entries;
   M159/M160 enforce `(4096-100)/40 = 99` via checked complete-size before
   any per-client X25519 work. Valid 17-99-entry Proposal lists would be
   rejected by Yosemite — same blocked determination.
4. **Legacy AES (×1 mode):** M161 outcome B inherited — valid LS1
   `i2cp.encryptLeaseSet=true` contract but disproportionate/unsafe to
   resurrect; all five `EncryptLeaseSet` cells stay blocked (no partial-enum
   apply).
5. **Blinded ×2 modes:** Yosemite-expressible (`type5+auth0` with/without
   secret) but field-level blocked: `OptionalLookup` cannot promote without
   its PSK/DH lookup uses, and `EncryptLeaseSet` cannot partially apply one
   value; lookup custody/transaction owner is additionally absent.

No live reference-router exercise was built (internal-only boundary; no new
harness/dependency/network effect inside the I2PControl-only envelope — same
scope rationale as the M161 gate). The determination above is source-level
and decisive under the plan rule that flag-ignored/inert "success" never
counts as support: the Yosemite gaps are proven by the checked-out typed
API + reservation lists + serialization code against the frozen core
extraction/gate code, not by the absence of trying.

## 5. Outcome determination: blocked integration with truthful fail-closed hardening

M162 closes as a **blocked integration**:

- The modern/legacy field-to-property mappings are valid and frozen (ten-mode
  table); the neutral M156-M160 primitives remain valid and untouched.
- Full Proposal field support is **blocked** under the frozen
  core/Yosemite/architecture envelope by §4 gaps 1-4 plus the absent
  lookup/base/per-user custody/transaction owner. Supporting PSK/DH would
  require a Yosemite typed base-key amendment (plus duplicate/bound
  disposition) via a separate accepted plan; supporting legacy AES would
  require the disproportionate/unsafe LS1 resurrection already rejected in
  M161-B.
- The correct disposition is therefore to keep **all fifteen cells blocked**
  (`EncryptLeaseSet` ×5, `OptionalLookup` ×5, `LeaseSetClientAuths` ×5)
  rather than schedule insecure work, broaden the M159/M160 4096-byte
  O(N) bounds, alias legacy AES to type 5, or accept any mode inertly.
- What M162 *did* implement (nine-file subset) is the truthful blocked
  posture the plan requires short of promotion: exact typed/redacted domain
  (no rawConfig secrets), the executable ten-mode table, fail-before-allocation
  gates on every family (closing the `httpserver`/`streamr` inert gaps),
  session-layer defense-in-depth with an ordinary-only wire fixture, and
  control-plane persistence/redaction hardening. No downgrade path exists:
  invalid modes fail before listener/session/publication allocation; legacy
  flag with type 5 still rejects at the core gates; OLD type `1` still
  rejects; oversize/sparse/mixed inputs still reject; ordinary Ed25519/LS2
  behavior is byte-preserved.

Deferred custody/transaction design (for a future Yosemite-amended successor;
not allocated here): extend `server_secret_store.rs` with a separate typed
`LeaseSetSecurity` record keyed by the same stable
`__emissary_server_destination_identity` (lookup secret +
generated/imported base PSK or X25519 private + per-user `{name,key}` +
generation/revision metadata), reusing the existing stage/commit/discard +
current/backup atomic-publish discipline; create/edit/restart follow plan
§1-7 ordering (validate full mode/table + key lengths before allocation;
prepare off to side; persist atomically or keep last-good; commit
definition/runtime generation only with its secret revision; cancel stale
before republish; failed start preserves last-good; delete cleans through the
same deletion transaction as destination secrets); `Get` stays
redacted/omitted for secrets; `stores/tunnel_store.rs` needs no change
(typed definition fields persist through existing upsert).

## 6. Dependency/security review

- No new dependency. Reused: `serde`/`serde_json` (typed fields), existing
  Base64 codecs (core `base64_decode` for strict 32-byte Proposal Key
  validation at the control plane), `rand`/`zeroize` posture unchanged (no new
  keygen was added — blocked), and the already-pinned Yosemite
  `SessionOptions`/`LeaseSetClientAuth` types (read-only gap analysis only).
  No manifest/lockfile/Yosemite change (proven by §12).
- `no_std + alloc` posture unchanged (no core touched; proven by the `no_std`
  check in §8).
- Security invariants held: raw lookup secrets, base keys (none allocated),
  and per-user keys never appear in logs/errors/metrics/Get/rawConfig except
  the safe `EncryptLeaseSet` mode string in Get (non-secret); user names never
  appear in diagnostics (entry/options Debug redacts both names and keys;
  backend errors carry field names only); unsupported/invalid combinations
  fail before allocation on every family; no downgrade to ordinary,
  unsecreted, or unauthenticated publication (ordinary wire emits no LeaseSet
  keys; type-5 mismatches fail closed at the preserved core gates); no broad
  legacy resurrection (no LS1 code); no changes to M156-M160 primitives
  (byte-identical — 1159 core tests green); httpbidirserver reuses the
  canonical server destination/session owner (no second crypto owner); stale
  generations cannot republish (none allocated; supervisor/generation
  discipline untouched); shared-session compatibility distinguishes settings
  without logging material (existing redacted `CompatibilityKey` preserved and
  covered by retained tests).
- Fail-closed posture preserved: typed+raw LeaseSet presence rejects before
  secret-store lookup, runtime reservation, and SAM wire work on all five
  families (preflight==start proven); malformed Proposal shapes fail at the
  control plane before persistence (INVALID_PARAMS, no echo); OLD type `1`,
  legacy-with-type-5, sparse/mixed/over-ceiling type-5 inputs still reject at
  the frozen core gates (M157-M160 negative suites green).
- Ordinary Ed25519/LS2/DatabaseStore-type-3/NetDB-ordinary/M135/M145 SAM
  behavior preserved (same suites green).

## 7. Exact-path reconciliation (M061/M062)

M061 (`061-containment-boundary.toml` + `m061_containment.rs`):

- No change required and none made. All nine M162 production paths are inside
  the M061 policy root (`emissary-cli/src/i2pcontrol/`), which the M061 guard
  excludes from the exact-allowlist check. No new M061 path waiver was
  created. `registered_pending` handling is untouched.

M062 (`062-dependency-containment.toml` + `m062_dependency_containment.rs`):

- `062-dependency-containment.toml` `[current_registration]` bookkeeping
  closed: milestone `M162 (closed complete, zero promotions, blocked
  integration)` realized with the exact nine production paths (strict subset
  of the registered ten; `server_secret_store.rs` intentionally unchanged),
  zero new files, `new_direct_dependencies=[]`, `manifest_changes=[]`,
  `lockfile_change=false`, `yosemite_change=false`,
  `i2pcontrol_source_change=true` (nine I2PControl owners only).
- `m062_dependency_containment.rs` gains narrow `is_authorized_m162_path`
  covering exactly the nine production files plus the M162 closure/plan/
  registry/README/roadmap/AGENTS/docs set, wired into both the allowed-path
  and prohibited-pattern assert chains (same pattern as the M160/M161
  helpers; authorizes the new closure file, which no earlier helper covers).
- `policy_terms_do_not_leak_into_non_policy_production_paths` passes: new
  I2PControl source contains no out-of-policy production paths; core remains
  Proposal-free (new Proposal strings live only under `i2pcontrol/`).

## 8. Verification outcomes

| Command group | Result |
|---|---|
| `cargo check -p emissary-core` | **pass** |
| `cargo check -p emissary-core --no-default-features --features no_std` | **pass** |
| `cargo test -p emissary-core --lib --no-fail-fast` | **pass**: `1159 passed, 2 ignored` (identical to the M160/M161 head; zero regressions — expected: no core touched) |
| `cargo check -p emissary-cli --no-default-features --features i2pcontrol` | **pass** |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --lib --no-fail-fast` | **pass**: `835 passed` (821 pre-existing + 14 new M162 tests, zero regressions) |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment --test m062_dependency_containment --test m095_full_support_matrix --test m105_residual_option_audit --no-fail-fast` | **pass**: `8 + 23 + 3 + 1 = 35` across 4 suites (includes the new `is_authorized_m162_path` wiring in both chains) |
| M095 mechanical recomputation (via `m095_full_support_matrix` suite) | **pass**: `840/336/29/475` == declared; residual `10/5/5/5/4` unchanged |
| `cargo clippy -p emissary-core --all-targets -- -D warnings` | **pass**: No issues found |
| `cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings` | pre-existing failure only in unrelated `proxy.rs:60` (`chunks_exact`); zero warnings from M162 files (one new test lint fixed before closure; see §14) |
| stable `rustfmt` on M162-touched Rust files | **pass** for all nine production files plus the M062 guard test (`rustfmt --check` clean; repo nightly-only options warn; `max_width = 100` observed; no new nightly drift — see §14) |
| `cargo fmt --all -- --check` (whole tree) | pre-existing drift only in files outside the M162 budget (see §14); unrelated files untouched (reverted after an accidental `--all` run) |
| `git diff --check` | **pass** |

Focused M162 suites (14 tests): `domain::tunnel` (3: exact ten spellings +
serde round-trip + rejection of aliases/case/whitespace; entry redaction of
names+keys; options round-trip/redaction/default-empty); `backends::options`
(2: ten-row executable table authority — spellings, legacy/type/auth split,
M161 dependency, B32 derivation, store types; all-typed-presence rejection
across six families with no echo + ordinary-path Ok); `backends::runtime::
session` (2: typed+raw rejection across five server families with no echo +
ordinary-path build with no LeaseSet emission; ordinary SAM wire fixture with
no encrypt/type/auth/secret/client/privkey emission and no legacy aliasing);
`backends::server/http_server/http_bidir/irc_server/streamr` (5: per-family
typed+raw preflight==start rejection with no echo, ordinary Ok, Streamr
bounded limits preserved); `tunnel_manager` (2: secrets never enter
`raw_config`/Get/Debug with safe mode-string return + round-trip; malformed
Proposal shapes fail INVALID_PARAMS before persistence with no echo).

M157/M158/M159/M160/ordinary regressions: all pre-existing no-auth type-5,
lookup-secret type-5, PSK/DH type-5, B32 flags/CRC, UTC-rollover,
type-preserving NetDB storage/flooding, and ordinary type-3 LS2 tests
retained and passing (1159 pre-existing core + 821 pre-existing CLI lib, zero
failures).

## 9. Changed paths (exact budget only)

Production (9; strict subset of the registered ten — `server_secret_store.rs`
intentionally unchanged):

- `emissary-cli/src/i2pcontrol/domain/tunnel.rs` (typed/redacted
  `EncryptLeaseSetMode` + `ALL_ENCRYPT_LEASE_SET_MODES` +
  `LeaseSetClientAuthEntry` + `TunnelOptions` extensions + 3 tests);
- `emissary-cli/src/i2pcontrol/backends/options.rs` (executable
  `LEASE_SET_SECURITY_MODES` ten-mode table + `LeaseSetLookupRequirement`/
  `LeaseSetIndexedRequirement` + `validate_lease_set_security` + common wiring
  + `present_runtime_fields`/`is_common_runtime_field` coverage + 2 tests);
- `emissary-cli/src/i2pcontrol/backends/runtime/session.rs`
  (`reject_leaseset_raw_presence` defense-in-depth in `build_session_options`
  + 2 tests including the ordinary-wire fixture);
- `emissary-cli/src/i2pcontrol/backends/server.rs` (preflight==start
  LeaseSet rejection test; raw gate already exact);
- `emissary-cli/src/i2pcontrol/backends/http_server.rs` (`OptionalLookup`
  added to `REJECTED` — closing the previously-inert gap — + M162
  typed+raw rejection test);
- `emissary-cli/src/i2pcontrol/backends/http_bidir.rs` (M162 typed+raw
  rejection test; reuses canonical server owner, no second crypto state);
- `emissary-cli/src/i2pcontrol/backends/irc_server.rs` (M162 typed+raw
  rejection test);
- `emissary-cli/src/i2pcontrol/backends/streamr.rs` (explicit M162 raw
  rejection — closing the previously-inert gap — + typed rejection test
  within bounded Streamr limits);
- `emissary-cli/src/i2pcontrol/tunnel_manager.rs` (strict typed extraction
  with 32-byte Proposal Key validation, `is_typed_secret_key` raw exclusion
  for both secrets, merge coverage, safe mode-only Get insertion,
  `SENSITIVE_OPTION_KEYS` redaction for `OptionalLookup`, + 2 tests).

Guards/ledgers (2):

- `emissary-cli/tests/m062_dependency_containment.rs` (M162 authorization
  helper in both chains);
- `plans/implementation/i2pcontrol-proposal-170/062-dependency-containment.toml`
  (current-registration closure bookkeeping, nine-path subset recorded).

Planning/closure/registry (8):

- `plans/implementation/i2pcontrol-proposal-170/162-proposal-leaseset-security-field-integration.md`
  (Status `registered / dependency-ready` → `closed as complete` with
  closure link and blocked-integration record);
- new `plans/closure/i2pcontrol-proposal-170/162-closure.md` (this file);
- `plans/registry.md` (M162 → closed blocked, zero promotions; no registered
  successor; M152 hard dependency satisfied, registration pending);
- `plans/implementation/i2pcontrol-proposal-170/README.md` (M162 closed
  blocked section; chain updated);
- `plans/subsystems/i2pcontrol-proposal-170-post-m154-leaseset-security-corrective-roadmap.md`
  (M162 closed blocked; graph updated);
- `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md`
  (M162 closed blocked; handoff updated);
- `AGENTS.md` (M162 closed blocked entry; chain updated).

Docs (3, authority wording only):

- `docs/i2pcontrol/README.md`, `docs/i2pcontrol/proposal-170-support.md`,
  `docs/i2pcontrol/tunnel-manager.md` (M162 closed blocked authority +
  `336/29/475` wording retained; LeaseSet residual-blocked statements
  retained).

`061/062`, `095/105/110` semantics: `061` unchanged (policy-root subset, no
broad waiver); `062` gains only the exact M162 closure authorization above;
`095/105/110` intentionally **unchanged** (zero promotions; see §11).

## 10. Requirement-to-evidence matrix (planning process §2.5)

| Closure duty | Evidence |
|---|---|
| implementation commits | one commit landing with this closure (production + tests + M062 guard + planning records + this closure; exact paths in §9); no prior partial implementation commit (registration commit touched only plans) |
| invariant review | partial-support, no-fabrication, I2P-only egress, fail-closed option validation, containment, and Y005 invariants re-proved by green `m061`/`m062`/`m095`/`m105` guards; no invariant weakened; M147/M148 not reopened (no SigType change; type-11 stays derived-only); M146 untouched (no egress); no downgrade path exists (all LeaseSet presence fails before allocation; legacy flag either rejects with type 5 at the frozen core gates or stays rejected at the I2PControl gates without encrypting; no LS1 fallback was added; ordinary wire emits no LeaseSet keys) |
| failure/recovery and contention evidence | malformed Proposal shapes fail at the control plane before persistence (INVALID_PARAMS, no echo); typed+raw presence fails identically in preflight and start before store/runtime/wire work; oversize/sparse/mixed/over-ceiling type-5 inputs still reject at the frozen core gates; no secret generations exist to roll back, so store-failure/cancellation/rollback is vacuously satisfied (staging discipline untouched and covered by retained `server_secret_store` tests); no second scheduler/state machine added; no lock spans changed; shutdown paths unchanged; ordinary definitions restart byte-compatibly |
| compatibility, migration, security review | no wire/protocol/storage/dependency change ⇒ no migration impact (definition JSON gains three optional typed keys, all absent-by-default; `stores/tunnel_store.rs` upsert unchanged); no new attack surface (validation/redaction gates only; no keygen, custody, NetDB, or SAM construction added); all 15 LeaseSet cells stay fail-closed (matrix unchanged); legacy AES stays valid-but-blocked with no silent aliasing (§4-5); `cargo clippy -p emissary-core -D warnings` clean; `emissary-cli -D warnings` failure is pre-existing and unrelated with zero M162-file warnings |
| documentation and operational evidence | §9 changed paths; authority docs name M162 closed blocked and retain partial support; operational impact none beyond truthful blocked errors (previously-inert `httpserver`/`streamr` inputs now fail closed) and redacted Get (previously-plaintext `OptionalLookup` in Get/rawConfig is now omitted) |
| M162 acceptance (plan closure evidence) | final ten-mode machine table (§1), exact changed paths/dependencies (§9), M061/M062 authorization (§7), secret-store/transaction design with deferred rationale (§1/§5), standard-property wire fixtures — negative ordinary wire plus Yosemite gap analysis (§2/§4) — all redaction/failure results (§8), per-field/per-family promotion decisions (§11), unchanged M095 (§8/§11), implementation SHA (commit landing with this closure), residual blockers and M152 readiness (§13) |

## 11. Promotion accounting (zero)

- `110-completion-ledger.toml`: no new entry (zero promotions).
- `095-full-support-matrix.toml` / `105-residual-option-audit.toml`:
  unchanged.
- Mechanically recomputed `336/29/475` == declared; residual
  `10/5/5/5/4` unchanged.
- Per-field/per-family decisions (all blocked_primitive, five server families
  each): `EncryptLeaseSet` — blocked (M161-B legacy + §4 gaps 1-4; no
  partial-enum apply); `OptionalLookup` — blocked (valid lookup uses include
  Yosemite-inexpressible PSK/DH lookup variants + custody absent);
  `LeaseSetClientAuths` — blocked (Yosemite base/duplicate/bound gaps +
  custody absent). Clients remain not_applicable (unchanged).
- Infrastructure alone has zero support value (registry rule); the typed
  domain/table/gates in this closure claim no cell. A stored mode string,
  redacted secret, or Yosemite-serializable subset alone is explicitly not
  support.

## 12. Production-head determination

- Last production-bearing commit before M162: `1629b0a5` (M160
  implementation/closure). M161 registration (`e11b2779`), M161 closure
  (`a05e0f06`), and M162 registration (`631c8fb9`) touch only
  plans/docs/guard-test paths.
- M162 is production-bearing (exact §9 files); after the closure commit
  lands, the last production-bearing head becomes that commit.
- `git log 631c8fb9..HEAD` over `emissary-core/src`,
  `emissary-cli/src`, `emissary-util/src`, all manifests and `Cargo.lock`
  at closure time contains only the staged M162 exact-budget diff
  (nine I2PControl owners plus the M062 guard exact amendment).
- `git diff --name-only` over manifests/lockfile is empty: no `Cargo.toml`,
  `emissary-cli/Cargo.toml`, `emissary-core/Cargo.toml`, or `Cargo.lock`
  change. No `emissary-core/**`, `emissary-util/**`, Yosemite, frontend, or
  workflow production path was created, modified, or deleted
  (`git status --short` shows only the §9 paths).
- Streamr datagram limits (16-subscriber, 60s expiry, 1200-byte payload,
  4095-byte transport-buffer, 15s refresh, bounded shutdown,
  loopback-only UDP) untouched; remote datagrams never choose a local UDP
  destination.

## 13. Registry updates and future-plan unblock determination

Applied alongside this closure (see §9 for the file list):

- `162-*.md` plan: Status `registered / dependency-ready` → `closed as
  complete` with closure link and blocked-integration record;
- `plans/registry.md`: M162 → closed blocked (`336/29/475` unchanged, zero
  promotions, nine-path I2PControl diff); **no registered successor**;
- `110-completion-ledger.toml`: no new entry (zero promotions);
- `m061`: unchanged (policy-root subset; no broad waiver);
- `m062`: exact M162 closure authorization only (§7); no broad waiver.

Future-plan unblock determination (as required by the tasking):

- **M162 CLOSED (blocked integration).** Its hard dependencies (M160 closure
  with frozen modern publication/rollover owner and zero promotions; M161
  outcome B with `EncryptLeaseSet` held blocked and no successor) were
  satisfied at registration, and all plan requirements are met as a blocked
  closure with exact-budget hardening and zero matrix promotions. Stop
  conditions triggered and dispositioned (not bypassed): Yosemite base-key /
  duplicate / bound gaps (§4 gaps 1-3) plus inherited M161-B legacy block
  (§4 gap 4) make full-domain promotion impossible without a Yosemite
  amendment and without insecure LS1 resurrection; no outside file or
  dependency was required (the Yosemite change a full integration would need
  is precisely what keeps this closure blocked); no I2PControl custody was
  allocated inertly; no NetDB subsystem was added; modern modes never set the
  legacy flag and legacy never maps to type 5; no failure downgrades to
  plaintext or ordinary publication.
- **M152 REMAINS deferred/unregistered (hard dependency now satisfied,
  registration pending).** M162 closure satisfies its hard dependency for
  final whole-surface requalification, which must now account for the M155
  field-coupling rule plus the M156/M157/M158/M159/M160 primitive
  availability, the M161 outcome-B `EncryptLeaseSet`-blocked disposition, and
  the M162 blocked-integration evidence (typed domain, ten-mode table,
  fail-closed gates, redaction, Yosemite gaps, zero promotions). But M152 is
  **not** registered by this closure: its entry gate requires its own
  registration commit. Only M152 may be registered next.
- **M152 readiness:** ready for registration as a zero-production/
  zero-promotion requalification of the actual head; it must not smuggle
  Yosemite, LS1, SigType, outproxy, or custody implementation into
  qualification.
- **A future Yosemite-amended LeaseSet successor is NOT registered here.**
  If maintainers later accept a Yosemite typed `leaseSetPrivKey` (plus
  duplicate/bound dispositions) amendment, that work needs its own accepted
  implementation plan with exact-path/dependency authorization; file presence
  of the ten-mode table alone never authorises production work.
- **M149-M151 remain superseded/unregistered (do not execute).** Confirmed
  fully superseded by M155-M162; no re-gating or revival is authorized.
- **M147/M148 remain closed-blocked/deferred-behind-M147** (M154
  disposition C); this line does not reopen them. Type-11 blinding remains
  explicitly not a `SigType` reopening.
- **M146 remains closed as blocked** with no successor; it is not reopened
  by M162 or by any future LeaseSet tail.
- No other future-plan status required a change. File presence alone never
  authorises production work.

## 14. Unresolved findings

- Non-blocking: `cargo test -p emissary-cli --no-default-features
  --features i2pcontrol --test m153_post_m146_requalification` was not run
  in the M162 verification set (same scope as the M160/M161 closures: the
  four `m061`/`m062`/`m095`/`m105` suites plus lib + core). The known
  pre-existing `AGENTS.md`/`proxy.rs` findings below fail identically at
  the pre-M162 registration baseline; unrelated to M162 files. Severity:
  informational; no corrective pass required (out of M162 exact-budget
  scope; must not be smuggled into this line).
- Non-blocking: `cargo clippy -p emissary-cli --no-default-features
  --features i2pcontrol --all-targets -- -D warnings` reports a
  pre-existing lint in `backends/filters/proxy.rs:60` (`chunks_exact`),
  untouched by M162; M162 files contribute zero warnings (one new-test lint
  was fixed before closure). Severity: informational; no corrective pass
  required (out of M162 exact-budget scope).
- Non-blocking: `cargo fmt --all -- --check` (stable or nightly) reports
  drift in files outside the M162 budget at the registration baseline
  (e.g. `address_book_runtime.rs` reflow); an accidental `cargo fmt --all`
  run in this workstream touched unrelated files and was fully reverted
  before closure. The M162-touched Rust files are stable-`rustfmt` clean
  with `max_width = 100` observed, and no new nightly drift was introduced
  by M162 lines (remaining nightly hunks are pre-existing comment reflow
  from a newer toolchain). Severity: informational; unrelated files were not
  touched in the closure commit.

## Internal-only / read-only-upstream attestation

- External sources (the already-pinned Proposal/Java/Yosemite revisions
  cited from existing M155/M156/M157/M160/M161 records; the checked-out
  Yosemite fork read-only for wire/validation evidence at
  `~/.cargo/git/checkouts/yosemite-b3f22cc17f665e22/59140a2/src/options.rs`
  and `src/proto/session.rs`; no new Proposal/Java fetch) were accessed
  read-only for evidence; no Java source checkout was built and no live
  reference router was provisioned for this blocked-integration closure;
- No upstream or third-party repository, issue, pull request, discussion,
  review, or maintainer channel was opened, drafted, updated, commented on,
  or contacted;
- No commit, branch, tag, patch, release, or artifact was pushed to any
  upstream remote before this closure commit. The commit/push performed with
  this closure targets the authorized internal repository
  (`eggstack/emissary` origin) on the current branch only, as explicitly
  directed for this internal workstream;
- No upstream review, approval, feedback, adoption, or merge was requested;
- No upstream contribution package, patch series, or submission checklist
  was prepared;
- Violation would invalidate this closure per `plans/003-planning-process.md`
  §11; no such violation occurred.

(End of file)
