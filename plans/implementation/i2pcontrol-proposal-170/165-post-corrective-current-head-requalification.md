# M165 — Post-Corrective Current-Head Requalification

Status: **closed as complete** (closure: `plans/closure/i2pcontrol-proposal-170/165-closure.md`)

Class: zero-production runtime/security/containment requalification

Promotion budget: **zero Proposal cells**

Hard dependencies: M163 closure + M164 closure

## 1. Purpose

Requalify the actual repository head after both post-M152 correctives:

- M163 removes blocked LeaseSet-security durable/inert state and scrubs retained contaminated tunnel-store generations;
- M164 removes malformed-SAM raw-command secret logging.

M152 remains immutable historical qualification evidence for the pre-corrective head, but it cannot remain the current machine authority after production-bearing corrections. M165 refreshes the machine production-head pointer and proves that the corrected implementation still satisfies the complete Proposal-170 partial-support contract.

M165 is not an implementation milestone. It may not smuggle additional capability work into qualification.

## 2. Entry gate

M165 may be registered only after:

### M163 closure proves

- Create/Edit rejection of all supplied blocked LeaseSet-security fields before durable mutation;
- crash-resumable sanitization of M162-era inert typed fields;
- no blocked LeaseSet secret remains in any retained tunnel-store generation after successful scrub;
- no new dependency, core, Yosemite, backend/session runtime or secret-store change;
- no Proposal promotion/demotion.

### M164 closure proves

- malformed SAM commands no longer emit raw command content;
- lookup/PSK/DH secret markers cannot enter the invalid-command warning;
- no parser/session behavior change;
- exact one-file neutral core budget, no dependency/Yosemite/I2PControl production change;
- no Proposal promotion/demotion.

If either closure lands with a different disposition, amend M165 before registration.

## 3. Production budget

Zero production source changes.

Allowed changes are planning/qualification/test-evidence plus the machine authority metadata required to point at the actual production head.

No `emissary-core/src/**`, `emissary-util/src/**`, `emissary-cli/src/**`, Cargo manifest, lockfile, Yosemite, frontend, workflow or runtime behavior change is authorized by M165.

## 4. M095 authority refresh

M165 must determine the actual last production-bearing commit after M164 and update:

`plans/implementation/i2pcontrol-proposal-170/095-full-support-matrix.toml`

so `current_production_head` equals that exact commit.

Do not change cell dispositions merely to make the metadata update convenient. Expected counts remain:

- total = 840;
- apply = 336;
- blocked_primitive = 29;
- not_applicable = 475.

Residual identities must remain exactly:

- SigType ×10;
- EncryptLeaseSet ×5;
- OptionalLookup ×5;
- LeaseSetClientAuths ×5;
- UseOutproxyPlugin ×4.

Any mechanical disposition change is a stop condition requiring separate accepted planning.

## 5. Required requalification

M165 must rerun and record the current repository's accepted qualification surface, including at minimum:

1. M061 exact containment and Proposal-vocabulary separation.
2. M062 dependency containment and exact-current-registration closure state.
3. M095 machine matrix recomputation and exact residual identities.
4. M105 residual option audit.
5. M127 finite token lifetime.
6. M128 bounded JSON-RPC batch/request admission.
7. M129 fail-closed management TLS.
8. Shared session/destination ownership and M135 live quantity/LeaseSet truthfulness.
9. Reduce/Close/IdlePolicy/NewDest lifecycle composition.
10. M141 UniqueLocal confinement.
11. M142 SSLProxies/JumpList I2P-only routing.
12. M143 retained Profile behavior.
13. M144 application TLS separation.
14. M145 `shouldBundleReplyInfo` semantics.
15. M146 blocked provider behavior.
16. M154/M147/M148 SigType blocked behavior.
17. M156-M160 neutral ELS2 primitives, lookup secret/B32, PSK/DH auth, 4096-byte bounds, all-zero X25519 rejection and no-fallback behavior.
18. M161 legacy-AES outcome-B behavior.
19. M162 ten-mode typed/redacted validation and backend/session fail-before-allocation defense in depth.
20. M163 fail-before-durable-effect behavior plus historical retained-generation scrub.
21. M164 malformed-SAM secret-log hardening.

Qualification must distinguish runtime/behavioral proof from parser/persistence reachability. Inert storage is never support.

## 6. M163-specific adversarial evidence

Explicitly prove on the final head:

- blocked LeaseSet Create produces no new tunnel-store revision;
- blocked LeaseSet Edit preserves the previous definition and revision;
- all three blocked fields are covered individually and in combinations;
- malformed secret-bearing fields never echo values;
- an M162-era legacy generation with persisted LeaseSet-security fields is sanitized through the pending scrub state;
- at least two clean fallback generations exist before contaminated history is deleted;
- successful scrub leaves no blocked field/secret bytes in any retained generation file;
- interruption/failure before scrub completion prevents StartOnLoad and resumes on next load;
- sanitization does not delete unrelated definitions or mutate unrelated options;
- no LeaseSet secret is copied to `server_secret_store`, `raw_config`, logs, metrics, errors or Get output.

## 7. M164-specific adversarial evidence

Explicitly prove:

- rejected SAM UTF-8 command content is absent from warning fields;
- unique lookup-secret, PSK and DH markers sent in malformed commands never appear in captured logging;
- invalid UTF-8 logging contains no raw bytes;
- valid SAM command behavior remains unchanged;
- no ad-hoc secret-key allow/deny list is required to achieve redaction; rejected payload content is omitted wholesale.

## 8. Security/architecture review

Re-audit the post-M164 head for:

- no blocked or unsupported option accepted with durable/runtime effect;
- no plaintext/unsecreted/unauthenticated fallback;
- no direct-clearnet outproxy fallback;
- no broad M061/M062 waiver;
- no secret-bearing Proposal/SAM state in response-facing, debug or log surfaces;
- no second LeaseSet publication/rollover owner;
- no high/medium defect waived as `terminal`.

## 9. Authority disposition

If all required evidence is green and no high/medium defect remains, M165 may close as the new **safe-partial current-head qualification authority** with `336/29/475` and no registered successor.

The phrase `terminal` means only "no dependency-ready safe implementation plan under the accepted architecture/policy". It does not convert blocked cells into support and does not prevent a separately accepted future Yosemite/provider/SigType architecture plan.

If a high/medium defect remains, M165 must close blocked/incomplete or spawn an explicit corrective successor. Do not declare terminal authority around an unresolved material finding.

## 10. Historical dispositions

M165 must preserve:

- M149-M151 superseded/unregistered;
- M147 blocked and M148 blocked/deferred behind M147 under M154-C;
- M146 blocked for UseOutproxyPlugin;
- M161 outcome B for legacy AES/LS1;
- M162 zero promotions;
- M159/M160 neutral primitives as infrastructure only.

No historical closure file is rewritten.

## 11. Stop conditions

Stop and write a separate corrective plan if requalification finds:

- any M095 disposition drift;
- any `apply` row that is parser-only/inert/approximate;
- any blocked field that still causes durable/runtime effect;
- any M163 migration data-loss, retained-secret or rollback defect;
- any remaining malformed-SAM secret log exposure;
- any new high/medium security finding;
- any need for production code, dependency or Yosemite changes.
