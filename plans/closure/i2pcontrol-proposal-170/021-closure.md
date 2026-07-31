# M021 Closure Record — TunnelManager Wire, Atomic Persistence, and Secret Boundary

Status: closed

Source implementation plan:

- `plans/implementation/i2pcontrol-proposal-170/021-tunnelmanager-wire-atomicity-and-secrets.md`

Source subsystem roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-roadmap.md`

Implementation commit:

- `55c4b0f` — `fix(i2pcontrol): close TunnelManager wire and persistence gaps`

## 1. Executive finding

M021 is complete for its bounded administrative TunnelManager scope. The
canonical lowercase action path is strict and separate from Emissary's
capitalized/`List` compatibility path. Canonical responses use the pinned
structured operation envelope and lower-case `info/rawConfig` schema. CRUD
mutations publish one complete generation, failed publication preserves the
prior state, persisted legacy secret duplication is migrated, and response,
debug, and compatibility serializers do not disclose secret values.

Unsupported tunnel types remain durable administrative definitions with inert
runtime behavior. M021 does not implement a missing data plane, import
startup-managed tunnels, or change router lifecycle ownership.

## 2. Requirement-to-evidence matrix

| Requirement | Evidence | Result |
|---|---|---|
| Exactly seven canonical actions and twelve tunnel types | `ALL_TUNNEL_ACTIONS`/`ALL_TUNNEL_TYPES` inventories, domain count tests, canonical fixture | pass |
| Exact action-specific parameters and `All` matrix | `validate_canonical_request`; canonical invalid/missing/conflict tests | pass |
| Unknown and malformed canonical options rejected | strict option inventory, scalar/container validators, focused rejection tests | pass |
| Full range and `EncryptLeaseSet` validation | bounded integer validators and ten-value enum; focused canonical tests | pass |
| CRUD success/failure envelopes | canonical operation response helpers and handler fixtures | pass |
| Exact canonical `get` info keys/types | dedicated serializer and pinned-key test; lower-case `rawConfig.name/type` fixture | pass |
| No legacy/fabricated canonical fields | canonical serializer omits `Name`, `Type`, `State`, flattened aliases, and unavailable destinations; negative fixture | pass |
| All twelve types persist and round-trip | all-type CRUD fixture plus store persistence tests | pass |
| Unsupported backends remain resource-inert | existing exhaustive unsupported registry tests and unchanged backend boundary | pass |
| One-generation create/edit/rename/delete publication | `TunnelStore::update`, production adapter wiring, revision assertions | pass |
| Rename failure preserves memory and restart state | injected publication failure test and reload assertion | pass |
| Permission failure is fail-closed | Unix permission failpoint test; generation publisher removes temp files | pass |
| Corrupt newest generation falls back | generation-store fallback test | pass |
| Secrets are not duplicated or disclosed | typed-secret migration, filtered serializers, custom `TunnelDefinition` Debug, secret tests | pass |
| Compatibility remains isolated | separate compatibility serializer and existing capitalized/`List` fixtures | pass |
| No router.toml, frontend, core, data-plane, or upstream scope | changed-file review and internal-only attestation below | pass |

## 3. Failure, restart, and contention evidence

- Production mutations serialize under the existing `Arc<tokio::sync::Mutex<_>>`
  owner. The update operation clones the complete map, checks existence and
  collisions, and calls `publish` exactly once.
- `GenerationStore::publish` validates and serializes before writing, syncs the
  temporary file, enforces `0o600` on Unix, atomically renames, and updates the
  in-memory snapshot only after rename succeeds.
- Write, sync, permission, injected publication, and rename failures remove the
  temporary file where feasible and do not update the previous snapshot.
- Loading scans newest-first and falls back to an older complete generation;
  legacy typed/raw secret duplication is normalized into one new generation.
- Readers use the existing owner lock and therefore observe a complete
  before-state or complete after-state.
- Unsupported start/restart do not persist or acquire runtime resources; stop
  remains inactive/idempotent.

## 4. Compatibility, migration, and security review

Canonical lowercase actions accept only the pinned top-level fields and
canonical option names. Capitalized actions and `List` retain their separate
historical response shapes, but all serializers filter sensitive keys. Typed
secrets (`sslKey`, proxy password, IRC password) are not duplicated into raw
configuration. Future-backend sensitive containers are persisted once and are
omitted from generic responses. `PrivKeyFile` is rejected as generic
key-material ingress.

On load, generations from the prior model move duplicated proxy/SSL/IRC secret
values into the typed redacted fields and remove the raw copy before publishing
the migrated complete generation. No encrypted-at-rest dependency or broad
configuration migration was introduced.

The canonical serializer emits neutral `offlineKeys: false` because Emissary
has no source for offline-key state. `localDestination`, `destination`, and
`destinationB32` are omitted until a truthful destination owner is supplied by
the startup/source work; no local target host or empty fabricated value is
substituted.

## 5. Documentation and static evidence

Updated:

- `docs/i2pcontrol/tunnel-manager.md`
- `docs/i2pcontrol/proposal-170-support.md`
- `docs/i2pcontrol/security.md`

The docs now contain literal canonical `get` wire fixtures, the complete
option/enum boundary, compatibility separation, secret handling, and the
unsupported-runtime boundary. The domain and handler tests assert the seven/
twelve inventories and the exact canonical `info` key set.

## 6. Verification executed

```text
cargo test -p emissary-cli --no-default-features --features i2pcontrol tunnel_manager
  139 passed
cargo test -p emissary-cli --no-default-features --features i2pcontrol tunnel_store
  28 passed
cargo test -p emissary-cli --no-default-features --features i2pcontrol generation_store
  44 passed
cargo test -p emissary-cli --no-default-features --features i2pcontrol secret
  14 passed
cargo check -p emissary-cli --no-default-features --features i2pcontrol
  passed
cargo test -p emissary-cli --no-default-features --features i2pcontrol
  1165 passed across 15 suites
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
  passed
cargo +nightly fmt --all
  passed for touched implementation files; unrelated tutorial formatting was not retained
git diff --check
  passed
```

The repository-wide nightly format check still reports the pre-existing
`examples/rust-tutorial/src/main.rs` baseline difference. That unrelated file
is clean in the implementation commit; no workspace-wide formatting change was
included.

## 7. Residual findings and successor disposition

| Severity | Finding | Owner/disposition |
|---|---|---|
| informational | Startup-managed inventory, truthful runtime destinations, and any safe existing lifecycle adapters are not owned by M021 | M023, now ready |
| informational | AddressBook runtime authority remains a separate disconnected-source correction | M022, now ready |
| informational | Missing HTTP/IRC/SOCKS-IRC/CONNECT/Streamr/bidirectional data planes remain unsupported | Explicit non-goal; no successor may infer implementation authority |
| informational | Final 43-selector/method-family conformance and subsystem disposition remain open | M025–M027 |

No unresolved M021 high- or medium-severity finding remains. The subsystem
status remains `corrective pass required` until the later source and final
conformance milestones complete.

## 8. Dependency disposition

M021's hard dependency M020 is closed. M021 is closed and therefore unblocks:

- M022 — AddressBook runtime bridge and canonical source reconciliation;
- M023 — Startup tunnel inventory and ClientServicesInfo truthfulness.

M024 remains blocked on M023. M025 remains blocked on M022/M023/M024, and the
later milestones retain their existing dependency chain.

## 9. Internal-only compliance attestation

All implementation, test, documentation, closure, and planning writes targeted
the internal `eggstack/emissary` repository. The Proposal 170 page was accessed
read-only for exact wire verification. No upstream issue, pull request, review,
discussion, merge request, patch, branch, tag, submission package, maintainer
contact, review request, adoption request, or other upstream repository
mutation was created or prepared.
