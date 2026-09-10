# M163 — Blocked LeaseSet-Security State Persistence Corrective

Status: **registered / dependency-ready**

Class: correctness/security corrective; I2PControl-only

Promotion budget: **zero Proposal cells**

Hard dependency: M152 closure (`plans/closure/i2pcontrol-proposal-170/152-closure.md`)

## 1. Purpose

Correct the M162 control-plane persistence defect discovered by post-M152 review: Proposal LeaseSet-security values that remain `blocked_primitive` are currently parsed into typed `TunnelOptions` and may be durably written to the generic `TunnelStore` during Create/Edit, while runtime start later fails before allocation.

That behavior violates the workstream's stronger truthfulness rule: unsupported/blocked Proposal values must fail before allocation **and before durable effect**. A blocked capability must not create or mutate durable configuration state that implies acceptance.

M163 is intentionally narrow. It does not implement Yosemite LeaseSet key emission, legacy LS1 AES, Proposal field promotion, core cryptography, or a new LeaseSet secret store.

## 2. Current-head defect evidence

At the M152-qualified head:

- `TunnelOptions.optional_lookup` and `TunnelOptions.lease_set_client_auths` are serde-serializable typed fields.
- M162 tests explicitly assert that these secret-bearing fields persist in the definition store while remaining absent from response-facing `rawConfig`.
- `TunnelStore` serializes complete `TunnelDefinition` values into generation JSON.
- Production `TunnelManagerControl::create`/`update` persist definitions without the blocked-LeaseSet persistence gate that later rejects the same fields during Start.
- `GenerationStore::publish()` retains the current generation plus up to five prior generation files and its ordinary `cleanup()` is best-effort. Therefore publishing one sanitized generation would leave M162-era secret-bearing generations on disk and is insufficient as a secret-removal migration.

This is the behavior M163 must remove.

## 3. Exact behavioral contract

For all five server families currently carrying blocked LeaseSet-security cells (`server`, `httpserver`, `httpbidirserver`, `ircserver`, `streamrserver`):

1. Canonical Create containing any supplied `EncryptLeaseSet`, `OptionalLookup`, or `LeaseSetClientAuths` value must fail with the existing Proposal/I2PControl operation/validation error convention **before `TunnelStore::upsert`**.
2. Canonical Edit containing any supplied LeaseSet-security field must fail **before `TunnelStore::update`**, preserving the exact prior durable definition and revision.
3. Compatibility/legacy request forms that reach the same typed fields must obey the same no-effect rule; there must be no API surface where blocked values can be stored inertly.
4. Malformed LeaseSet-security inputs continue to fail at request parsing/validation before persistence, with no secret echo.
5. Existing ordinary definitions remain semantically compatible.
6. Existing M162 fail-before-runtime-allocation gates remain as defense in depth. M163 does not remove backend/session rejection just because CRUD now rejects earlier.
7. Explicit `EncryptLeaseSet="disable"` remains blocked while the field is present because the current matrix classifies the field contract as blocked as a whole; ordinary publication is represented by omitting the field. Do not silently reinterpret `disable` as successful omission unless a later accepted plan changes the matrix/domain contract.
8. No Proposal cell promotion/demotion occurs. M095 remains `336 apply / 29 blocked_primitive / 475 not_applicable`.

## 4. Existing persisted inert state — crash-resumable scrub

M163 must account for definitions already written by M162-era builds and for prior retained generation files containing those secret-bearing typed fields.

A current definition is contaminated if it contains one or more of:

- `options.encrypt_lease_set != None`;
- `options.optional_lookup.is_some()`;
- non-empty `options.lease_set_client_auths`.

The migration must clear all three fields together while preserving every unrelated definition field, name/type/ownership/start intent and safe `raw_config` state.

### 4.1 Why a single sanitized publish is forbidden

`GenerationStore` deliberately retains prior known-good generations for corruption fallback. Ordinary best-effort cleanup keeps up to five prior files. A single sanitized publish would therefore make the active snapshot clean while leaving historical LeaseSet secrets durably present. That is not sufficient closure evidence.

### 4.2 Required two-phase state machine

`TunnelStorePayload` must carry an internal serde-defaulted scrub state/epoch so legacy payloads enter the migration deterministically and a crash cannot make an incomplete purge look complete. Exact field naming is implementation-local; required logical states are:

- `legacy/unstarted`;
- `sanitized_pending_history_purge`;
- `scrub_complete`.

The load sequence is:

1. Load the newest valid generation using the existing corruption-fallback rules.
2. If the payload is legacy/unstarted **or** any blocked LeaseSet-security field is present, clone the loaded payload, clear all three blocked fields from every affected control-plane definition, set state `sanitized_pending_history_purge`, and publish a clean generation through the ordinary fully-synced `GenerationStore::publish` path.
3. Publish the same sanitized pending payload once more before destructive history cleanup. This establishes at least two newest clean generations so secret removal does not leave the store with zero corruption fallback.
4. Invoke a new narrow fail-closed `GenerationStore` history-purge operation that retains only the newest two generation files and removes all older generations. Unlike ordinary `cleanup()`, this operation is not best-effort: each candidate path is confined/non-symlink generation state, deletion failures are returned, and the directory is synced after removals.
5. Only after successful purge + directory sync, publish a `scrub_complete` clean generation. The resulting retained fallback set contains clean state only.
6. If any stage fails or the process crashes before step 5, `TunnelStore::load` must not proceed to StartOnLoad. The newest pending generation remains sanitized, and the next load recognizes `sanitized_pending_history_purge` and resumes the cleanup sequence rather than assuming success.
7. If a `scrub_complete` payload nevertheless contains a blocked field, treat it as contamination and re-enter the pending scrub path; never trust the marker over payload content.

This migration may intentionally discard older generic tunnel-store rollback generations because those files may contain prohibited secret state. It must first create the two clean fallback generations above. It must not delete whole tunnel definitions merely to remove the blocked fields.

The purge API is generic plumbing but is authorized only as an exact I2PControl store primitive. Do not weaken ordinary `GenerationStore::cleanup()` or change its retention policy for other stores.

## 5. Exact production path budget

Only these production paths are authorized:

1. `emissary-cli/src/i2pcontrol/tunnel_manager.rs`
   - request-level Create/Edit rejection before durable mutation;
   - tests proving no persisted effect and no secret echo.
2. `emissary-cli/src/i2pcontrol/stores/tunnel_store.rs`
   - internal scrub-state marker;
   - blocked-field sanitization;
   - crash-resumable load sequencing;
   - migration tests.
3. `emissary-cli/src/i2pcontrol/stores/generation_store.rs`
   - narrow fail-closed history-purge primitive retaining the newest clean fallback generations;
   - confined deletion + directory-sync semantics and focused tests.

No other production source path is authorized by M163.

In particular M163 authorizes no change to:

- `emissary-core/**`;
- `emissary-util/**`;
- Yosemite or any Cargo manifest/lockfile;
- `server_secret_store.rs`;
- backend/session runtime implementations;
- the M159/M160 crypto/parser/publication owners;
- NetDB/I2NP/primitives/events/router/tunnel/transport code;
- Proposal matrix dispositions.

If a fourth production path is required, stop and amend M163/M061/M062 before editing it.

## 6. Dependency/containment budget

- new direct dependencies: none;
- manifest changes: none;
- lockfile change: false;
- Yosemite change: false;
- core change: false;
- I2PControl production change: exactly the three files in §5.

All three files are inside the existing M061 I2PControl policy root. No new non-policy M061 exception is required. M062 must record exactly this three-file active budget. Do not introduce prefix/broad-directory waivers.

## 7. Required tests

At minimum add/adjust tests proving:

- Create with each blocked field individually fails before store publication;
- Create with all three fields fails before store publication;
- Edit of an existing ordinary definition with each blocked field fails and leaves serialized definition + revision unchanged;
- rejected values are absent from response/log/debug strings;
- malformed values still fail at INVALID_PARAMS before persistence;
- a synthetic pre-M163 legacy generation containing all three fields loads into pending scrub, creates clean fallback generations, purges all older contaminated generations, and finishes `scrub_complete` with no blocked field or secret bytes in any remaining generation file;
- pending-scrub restart after a simulated interruption resumes purge rather than considering migration complete;
- purge/delete/directory-sync failure fails load before StartOnLoad and is retryable without secret echo;
- `scrub_complete` plus reintroduced blocked fields re-enters scrub;
- sanitization preserves unrelated options, name/type/start intent and safe raw config;
- at least two clean generation files exist before contaminated history is removed;
- ordinary `GenerationStore::cleanup()` behavior is unchanged outside the explicit purge call;
- ordinary tunnel-store load/upsert/update/remove and corruption fallback regressions remain green;
- M162 backend/session rejection tests remain green as defense in depth;
- M095 matrix recomputes unchanged at `336/29/475`.

Test-only failure injection may be added inside the two store files if needed to deterministically cover purge interruption/error behavior; it must not alter production semantics.

## 8. Security review requirements

Closure must explicitly audit:

- no LeaseSet secret bytes/names in Debug/Display/errors/logs/metrics/Get/rawConfig;
- no blocked Create/Edit performs a durable store write;
- no sanitization path copies secrets into a second persistence location;
- after a successful scrub, no retained tunnel-store generation contains the blocked fields or secret bytes;
- interrupted/failed purge never proceeds to runtime reconciliation;
- the narrow purge primitive validates/confines generation paths and syncs the directory;
- the migration preserves at least two clean fallback generations before deleting contaminated history;
- ordinary generation retention semantics for other stores are unchanged;
- no downgrade enables a blocked LeaseSet mode at runtime;
- no change reopens M147/M148, M146, or superseded M149-M151.

## 9. Closure and successors

M163 closes only when both new blocked-state persistence and historical retained-generation contamination are removed safely.

On clean closure, only M164 may be registered next.

M164 is the neutral SAM rejected-command secret-redaction corrective. M165, not M163, is the later zero-production current-head requalification that refreshes M095 production-head authority after both production-bearing correctives.

M163 must not edit M095 dispositions or declare terminal qualification.

## 10. Stop conditions

Stop before implementation/closure if:

- safe history cleanup requires deleting whole tunnel definitions rather than sanitizing only blocked fields;
- fewer than two clean fallback generations can be established before contaminated-history deletion;
- history cleanup requires changing the ordinary retention policy globally rather than using a narrow explicit purge;
- migration requires a new secret store or broader production owner;
- any currently `apply` capability regresses;
- a blocked LeaseSet field becomes accepted at runtime;
- a Proposal cell would need promotion/demotion;
- a new dependency/Yosemite/core change becomes necessary.

Any such finding requires a new explicit amendment rather than silent scope growth.
