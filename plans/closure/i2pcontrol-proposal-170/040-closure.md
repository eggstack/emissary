# I2PControl Proposal 170 Milestone M040 — Closure Status

Status: closed

Source implementation plan:

- `plans/implementation/i2pcontrol-proposal-170/040-startup-server-cancellation-correction.md`

Implementation disposition:

- `plans/closure/i2pcontrol-proposal-170/040-implementation-disposition.md`

Implementation/test head: `c316487`

## 1. Finding

M040 corrected the startup server regression caused by dropping the only watch
sender before the runtime started. The original `ServerTunnelManager` path now
reaches session creation, publishes the observed destination, reaches
`STREAM FORWARD`, and remains alive until its owning task is stopped.

## 2. Requirement-to-evidence matrix

| Requirement | Evidence | Result |
|---|---|---|
| Sender lifetime | `emissary-cli/src/tunnel/server.rs`; named keepalive | pass |
| Startup SAM sequence | `startup_manager_reaches_forward_and_keeps_runtime_alive` | pass: HELLO, SESSION CREATE, destination observation, and STREAM FORWARD |
| Runtime liveness | same regression before explicit task abort | pass |
| Reusable cancellation contract | `closed_cancellation_sender_still_cancels_reusable_runtime` | pass |
| Startup ownership | startup manager remains read-only and has no control handle | pass |
| Control-plane retention | production composition and M033 lifecycle suites | pass |
| Scope | one production file; no core or protocol changes | pass |

## 3. Residual findings

No high or medium correctness, security, ownership, compatibility, containment,
or evidence finding remains in this milestone. The stable/nightly rustfmt option
mismatch remains a repository tooling qualification recorded by M043.

## 4. Future-plan disposition

M041 is dependency-ready and was advanced to its accepted implementation head
`f7a9b37`. M042 remains dependent on M041 and is handled by the subsequent
corrective sequence.

## 5. Internal-only attestation

External Proposal 170 material was read only. No upstream interaction or
contribution preparation occurred.
