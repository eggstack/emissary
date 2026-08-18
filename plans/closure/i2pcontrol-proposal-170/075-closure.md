# M075 Closure — Generic Server Accepted-Stream Hardening

Status: closed; M081 closed the option-truthfulness corrective on the accepted-stream architecture

Source implementation plan:

- `plans/implementation/i2pcontrol-proposal-170/075-generic-server-accepted-stream-hardening.md`

Corrective successor:

- `plans/implementation/i2pcontrol-proposal-170/081-generic-server-leaseset-option-truthfulness-corrective.md` (closed; closure: `plans/closure/i2pcontrol-proposal-170/081-closure.md`)

Implementation commit:

- `20db126325c858b6a240d49f4bdbe436ab184a50`.

## 1. Retained implementation evidence

M075 made a security-relevant architectural improvement that remains accepted and should not be rolled back:

- control-plane generic `server` no longer uses blind SAM `STREAM FORWARD`;
- it owns an application-visible accepted-stream session;
- it reuses shared accepted-server admission before local-target work;
- after admission, payload is relayed byte-for-byte to a fixed loopback target;
- persistent server Destination/public identity remains stable across restart;
- startup-managed server forwarding remains separately owned in `emissary-cli/src/tunnel/server.rs`;
- local target connect is bounded;
- private destination material remains redacted;
- no `emissary-core/**` production path or SAM protocol extension was added.

The fake-SAM/raw-relay/lifecycle/containment evidence for those properties remains useful.

## 2. Independent finding invalidating current closure

### MEDIUM — `leaseSetEncType` is accepted but no longer applied

M073 historically closed while generic server accepted exactly one I2CP session-shaping option, `leaseSetEncType`, and the then-current server runtime passed it into Yosemite `SessionOptions`.

M075 migrated the control-plane generic server to:

```text
GenericServerRuntimeConfig
  -> AcceptedServerRuntimeConfig
  -> run_accepted_server
```

At current head:

- `validate_i2cp_options` still accepts `leaseSetEncType` and rejects other generic-server I2CP keys;
- neither generic nor accepted-server runtime configuration carries the accepted value;
- `run_accepted_server` creates `SessionOptions` without setting `lease_set_enc_type`.

The option is therefore recognized/accepted but ignored by the actual running accepted-stream session. This is a direct regression of the apply-or-reject invariant and invalidates the original M075 claim that no high/medium option-truthfulness finding remained.

## 3. Why M075 verification missed it

M075 tests strongly covered the migration shape:

- accepted-stream session rather than `STREAM FORWARD`;
- raw bidirectional relay;
- admission reuse;
- loopback target;
- restart/public Destination stability;
- unsupported-option rejection;
- secret safety and containment.

They did not include a positive session-configuration fixture proving that every still-supported generic-server I2CP option reaches Yosemite/SAM after the migration. The rejection matrix remained green while the one positive capability silently disappeared.

## 4. Corrective requirement

M081 must inspect the pinned Yosemite `SessionOptions` behavior and do exactly one of:

1. thread the validated optional `leaseSetEncType` value through the accepted-stream runtime into `SessionOptions::lease_set_enc_type`; or
2. if accepted-stream Yosemite cannot apply it without a broader core/protocol change, reject the field before destination-store/session/task allocation and update support/capability documentation.

The following are prohibited corrective shortcuts:

- restoring control-plane `STREAM FORWARD`;
- adding arbitrary I2CP pass-through;
- implementing adjacent LeaseSet/privacy features;
- widening `emissary-core/**` merely for this option.

## 5. Inherited M074 corrective dependency

M075 also consumes the shared server admission state. Independent review found defects in that state which are owned by M080. The generic accepted-stream migration remains structurally correct, but the current generic server must consume M080's corrected admission implementation before final security closure.

## 6. Current disposition

Treat M075 as `corrective pass required` for current security/option-truthfulness purposes while retaining its accepted-stream migration as the required architecture.

M081 closes the direct M075 option regression after M080. M079 later independently re-audits generic server payload transparency, admission, identity stability, and apply-or-reject option truthfulness at final head.

External sources remain read-only. No upstream review, issue, pull request, merge, or submission is authorized.
