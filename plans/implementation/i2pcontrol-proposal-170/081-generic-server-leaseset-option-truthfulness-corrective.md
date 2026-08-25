# M081 — Generic Server LeaseSet Option Truthfulness Corrective

Status: closed — implementation and closure accepted; retained at current head

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-tunnel-security-hardening-roadmap.md`.

Original implementation/closure context:

- M073: `plans/implementation/i2pcontrol-proposal-170/073-generic-tunnel-option-truthfulness-corrective.md`;
- M073 closure: `plans/closure/i2pcontrol-proposal-170/073-closure.md`;
- M075: `plans/implementation/i2pcontrol-proposal-170/075-generic-server-accepted-stream-hardening.md`;
- M075 closure: `plans/closure/i2pcontrol-proposal-170/075-closure.md`.

Planning production baseline: `1618de172e7a78a193fc1bb117af269f31174030`.

## 1. Objective

Restore the generic Proposal 170 `server` backend's apply-or-reject invariant for `leaseSetEncType` after the M075 migration from the startup forwarding runtime to the I2PControl accepted-stream runtime.

The accepted-stream migration is retained. M081 must either carry the already-supported `leaseSetEncType` value into Yosemite `SessionOptions` for the control-plane generic server, or reject that option before destination-store/session/task allocation if accepted-stream Yosemite cannot apply it truthfully.

No additional I2CP option, LeaseSet feature, identity mode, or router behavior is authorized.

## 2. Confirmed regression

M073 closed against a runtime where the generic server accepted exactly one I2CP option, `leaseSetEncType`, and the reusable server runtime mapped it into:

```text
SessionOptions.lease_set_enc_type
```

M075 then replaced the control-plane generic server's blind `STREAM FORWARD` runtime with `AcceptedServerRuntimeConfig` and `run_accepted_server` so peer-aware admission could occur before raw relay.

At the current head:

- `ServerTunnelBackend::validate_i2cp_options` still accepts `leaseSetEncType` and rejects other I2CP keys;
- `GenericServerRuntimeConfig` does not carry `leaseSetEncType`;
- `AcceptedServerRuntimeConfig` does not carry it;
- `run_accepted_server` creates Yosemite `SessionOptions` with persistent destination/publish/nickname/SAM port and defaults the remaining fields.

The option is therefore accepted and persisted but does not influence the actual accepted-stream session. This is the same recognized-but-ignored class M073 was intended to remove.

M073's historical closure can remain true for its pinned implementation commit, but the invariant regressed in M075. M075's current closure is therefore corrective-pass-required until M081 closes.

## 3. Why prior verification missed the regression

M075 tests correctly proved:

- no control-plane `STREAM FORWARD` command;
- accepted-stream session creation;
- raw byte relay;
- persistent destination stability;
- M074 admission reuse;
- loopback target confinement;
- rejection of several unsupported generic options.

They did not include a positive fixture asserting that the one still-declared supported I2CP option is present in the accepted-stream SAM `SESSION CREATE`/Yosemite configuration. The generic option tests checked rejection surfaces and policy parsing, but no end-to-end capability matrix test tied accepted `leaseSetEncType` to session setup after the runtime migration.

M081 must add that positive runtime evidence or convert the field to an explicit pre-allocation reject.

## 4. Hard invariants

- retain M075's accepted-stream generic server architecture;
- do not restore control-plane `STREAM FORWARD` merely to regain this option;
- no `emissary-core/**` production change;
- no SAM/Yosemite protocol extension;
- no startup-managed server behavior change;
- startup server continues to use its existing `leaseSetEncType` mapping independently;
- only `leaseSetEncType` is in scope among generic-server I2CP options;
- every accepted runtime-relevant option must be demonstrably applied before M081 closes;
- if Yosemite accepted-stream sessions cannot apply the option, reject it before secret-store lookup/session/task allocation;
- do not silently coerce invalid values to defaults;
- private destination material and raw option values remain absent from errors/logs/Debug;
- M080 admission behavior is reused unchanged after it closes;
- no public Proposal 170 field/alias/type is added.

## 5. Required research/check before implementation

Inspect the exact Yosemite version pinned by this repository and its `SessionOptions` contract.

Confirm:

1. whether `SessionOptions::lease_set_enc_type` is used for `Session<style::Stream>::new` regardless of whether the later data path is `STREAM ACCEPT` or `STREAM FORWARD`;
2. accepted value syntax and whether Yosemite validates it or passes it to SAM/I2CP;
3. whether any `silent_forward`/forward-only setting was accidentally coupled to the old behavior;
4. how the fake SAM fixture can observe the resulting session command/config without exposing private destination material.

The current startup runtime already provides strong local evidence that `lease_set_enc_type` is a `SessionOptions` field. Reuse that established API if it applies to accepted-stream sessions; do not build a new I2CP adapter.

## 6. Preferred production correction

If Yosemite supports the option for ordinary streaming sessions, thread one optional field through the existing I2PControl accepted-server configuration:

```text
ServerTunnelBackend::runtime_config
  -> GenericServerRuntimeConfig
  -> AcceptedServerRuntimeConfig
  -> SessionOptions::lease_set_enc_type
```

Keep the field optional and absent for HTTP/IRC server families unless their own option contracts explicitly support it. Do not give every accepted-server family a new public capability accidentally.

A clean shape is either:

- an optional backend-owned session-options subset on `AcceptedServerRuntimeConfig`; or
- one narrowly named `lease_set_enc_type: Option<String>` field.

Choose the smaller design. Do not introduce a general `HashMap<String, String>` I2CP pass-through because that recreates persist-and-ignore/security ambiguity.

If the common accepted-server struct receives the field, each caller must explicitly set `None` unless its backend has validated and owns the capability.

## 7. Fail-closed alternative

If current Yosemite cannot apply `leaseSetEncType` to an accepted-stream `Session<style::Stream>` without a new core/API/protocol change:

- remove it from the generic server's supported option set;
- reject typed/raw `leaseSetEncType` before destination-store lookup/session/task allocation;
- update capability/support documentation to state that the control-plane generic server rejects it even though the startup-managed server may continue to support it through its separate path;
- retain the accepted-stream M075 architecture.

This outcome is preferable to widening Emissary core or restoring forwarding.

## 8. Validation semantics

The implementation must preserve current input truthfulness:

- exactly the existing I2CP key spelling is recognized;
- any additional generic server I2CP key remains unsupported;
- a non-string/structurally invalid value fails before allocation according to the existing TunnelManager/domain parsing contract;
- errors identify the option/key, not the secret/private destination or full raw config;
- create/edit/get round-trip behavior remains truthful about the accepted/rejected capability.

Do not opportunistically implement `SignatureType`, `IsPrivate`, `HashCash`, `Consumer`, LeaseSet client auth, encryption keys, multihoming, or other server fields in this corrective.

## 9. Failure, cancellation, restart, and compatibility semantics

- session setup failure remains sanitized;
- start reports running only after real accepted-stream session readiness;
- persistent destination remains unchanged across restart;
- changing `leaseSetEncType` through a stopped/edit/start cycle applies to the new session generation if the option remains supported;
- running-generation edit semantics remain whatever TunnelManager already defines; do not add hot reconfiguration;
- old generation cannot retain/apply options to a later generation;
- startup-managed server runtime is unchanged;
- generic relay remains loopback-only and byte-transparent after admission.

## 10. Ordered work packages

### WP1 — Yosemite capability confirmation

Read the pinned Yosemite implementation and existing startup runtime. Record whether the accepted-stream session can use `lease_set_enc_type` without any broader dependency/core change.

### WP2 — Apply or reject

Preferred: thread the validated optional value into the accepted-stream `SessionOptions` path.

Fallback: remove support declaration and fail before allocation.

Do exactly one; no conditional silent ignore.

### WP3 — Runtime evidence

Extend the fake SAM/session fixture to prove the value is actually expressed in session setup when supported, or prove the unsupported value prevents any SAM connection/store lookup when rejected.

### WP4 — Capability matrix/docs

Update generic server option-capability evidence, support docs, M073/M075 corrective notes, and the active planning state.

## 11. Required tests

If supported/applied:

- generic server with `leaseSetEncType` reaches accepted-stream `SESSION CREATE` with the expected Yosemite/SAM configuration;
- generic server without the field uses default session configuration;
- value survives definition persistence/edit/start round-trip without appearing in diagnostics if considered sensitive;
- restart applies the configured value to the new session generation;
- non-`leaseSetEncType` I2CP keys still fail before allocation;
- accepted-stream server still issues `STREAM ACCEPT` and never `STREAM FORWARD`;
- M080 admission runs before generic handler/local target;
- startup-managed server behavior is unchanged.

If rejected instead:

- typed/raw `leaseSetEncType` rejects before store lookup, SAM connect, task reservation, or local target work;
- support/capability docs report the rejection truthfully;
- all other unsupported I2CP keys remain rejected;
- generic server without the field still starts through accepted streams.

In both outcomes:

- private destination is absent from errors/Debug;
- public destination stability remains intact;
- M061/M062/M063 containment remains green.

## 12. Verification

Run at minimum:

```text
cargo test -p emissary-cli --no-default-features --features i2pcontrol
cargo test -p emissary-cli --no-default-features --features i2pcontrol generic_server
cargo check -p emissary-cli --no-default-features
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo check -p emissary-core
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m062_dependency_containment
git diff --check
```

Use scoped nightly rustfmt for touched Rust files under existing repository convention.

## 13. Acceptance criteria

M081 may close only when:

1. generic `server` no longer accepts any runtime-relevant I2CP option that is ignored;
2. `leaseSetEncType` is either demonstrably present in accepted-stream session configuration or rejected before all allocation;
3. the M075 accepted-stream architecture remains intact;
4. no control-plane `STREAM FORWARD` path is reintroduced;
5. unsupported generic server I2CP/options remain fail-closed;
6. persistent identity/lifecycle/secret-redaction behavior remains intact;
7. no core/router/startup ownership change occurs;
8. M080 is closed and its admission protections remain consumed;
9. capability/support documentation matches runtime truth;
10. no high/medium option-truthfulness finding remains in generic server scope.

## 14. Stop conditions

Stop and create separate architecture planning if applying `leaseSetEncType` would require:

- an `emissary-core/**` production API change;
- a Yosemite fork/protocol extension;
- restoring `STREAM FORWARD` for the control-plane server;
- a generic arbitrary I2CP passthrough map;
- implementing adjacent LeaseSet/privacy features not already in scope.

External source inspection is read-only. No upstream review, issue, PR, submission, or repository write is authorized.
