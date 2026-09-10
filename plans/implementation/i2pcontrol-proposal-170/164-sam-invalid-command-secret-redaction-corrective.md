# M164 — SAM Invalid-Command Secret-Redaction Corrective

Status: **deferred / unregistered**

Class: neutral SAM security hardening

Promotion budget: **zero Proposal cells**

Hard dependency: M163 closure

## 1. Purpose

Remove the remaining malformed-command secret exposure in the neutral SAM socket boundary before final Proposal-170 current-head requalification.

The current `SamSocket` parse-failure branch logs the complete UTF-8 command string when `SamCommand::parse()` returns `None`. M158-M160 introduced standard SAM/I2CP inputs that may contain lookup passwords and PSK/DH private/client key material. Valid commands are extracted/redacted before session retention, but a malformed command containing the same values can currently be echoed to the router log.

This is a logging defect, not a Proposal feature gap. M164 must remain Proposal-free and must not change accepted SAM syntax or session behavior.

## 2. Exact current behavior

Current owner:

- `emissary-core/src/sam/socket.rs`

Current failure branch:

- obtains a UTF-8 command slice;
- calls `SamCommand::parse::<R>(command)`;
- on `None`, emits a warning with the complete `%command` field.

That field may include arbitrary option values supplied by the local SAM client, including malformed `i2cp.leaseSetSecret`, `i2cp.leaseSetPrivKey`, `i2cp.leaseSetClient.psk.N`, and `i2cp.leaseSetClient.dh.N` material.

## 3. Required behavior

On SAM parse failure:

1. Never log the raw command string or raw command bytes.
2. Log only non-secret structural metadata that is already safe at the socket boundary, such as:
   - stable socket observation identifier;
   - accepted peer socket address when already available;
   - command byte length;
   - fixed event text `invalid sam command`.
3. Do not infer or log command-family names by tokenizing the rejected string; rejected input is untrusted and may itself contain secret-bearing prefixes.
4. Preserve the current stream behavior after rejection. Do not convert malformed input into an accepted command, new protocol response, panic, or connection-lifecycle semantic change unless existing behavior already requires it.
5. Invalid UTF-8 handling may continue to log the decoding error metadata, but must not emit raw bytes.
6. Valid SAM commands and their parser/session secret-redaction behavior remain byte/semantically unchanged.

## 4. Exact production path budget

Only:

- `emissary-core/src/sam/socket.rs`

This path is already an exact realized M061 neutral core owner. No new broad M061 waiver is required.

No other production file is authorized. In particular:

- no `sam/parser.rs` or `sam/session.rs` change;
- no crypto/ELS2/destination/NetDB change;
- no I2PControl production change;
- no Yosemite/dependency/manifest/lockfile change.

If another production logging site is discovered that must change to satisfy the same malformed-command path, stop and amend M164 before editing it.

## 5. Dependency/containment budget

- new direct dependencies: none;
- manifest changes: none;
- lockfile change: false;
- Yosemite change: false;
- I2PControl source change: false;
- core production change: exactly `emissary-core/src/sam/socket.rs`.

At registration M062 must record this exact one-file neutral budget. The core change must contain no Proposal/I2PControl policy terminology.

## 6. Required tests/evidence

Closure must provide evidence that:

- the invalid-command warning contains no raw command field;
- a malformed command carrying a unique lookup-secret marker cannot place that marker in captured/loggable warning fields;
- malformed PSK and DH key markers likewise do not appear;
- valid `HELLO`, naming, session and streaming/datagram command parsing remains unchanged;
- existing M158-M160 parser redaction and negative suites remain green;
- no new log line contains private/auth material;
- M095 remains exactly `336/29/475` with zero promotions.

Prefer a deterministic captured-tracing test if the existing test infrastructure supports it. A source-level guard against `%command` is useful defense in depth but is not sufficient alone if a runtime capture test can be implemented within test-only scope.

## 7. Security/architecture constraints

- Treat the entire rejected command as sensitive/untrusted.
- Do not attempt ad-hoc regex redaction of selected keys; future SAM extensions could introduce additional secrets. The safe contract is to omit rejected payload content entirely.
- Do not weaken parser validation to avoid the warning path.
- Do not change SAM bind/exposure policy in this milestone.
- Do not reopen M147/M148, M146, M161 or M162 blocked dispositions.
- Do not alter M159/M160 cryptographic semantics.

## 8. Closure and successor

M164 closes when malformed SAM input can no longer be echoed into logs and all parser/session regressions remain green.

After clean M163 + M164 closures, only M165 may be registered next.

M165 is the zero-production current-head requalification and M095 production-head authority refresh.

## 9. Stop conditions

Stop and amend if:

- another production source path is required;
- logging removal changes SAM connection/protocol behavior;
- a dependency is needed merely to redact the log;
- a material secret exposure exists outside the exact rejected-command path and requires broader remediation;
- any matrix disposition changes.
