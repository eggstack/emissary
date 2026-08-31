# M106 — DelayOpen Client-Listener Lifecycle

Status: **ready**

Class: capability / infrastructure

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md`

Predecessor and audit authority:

- M105 audit: `plans/implementation/i2pcontrol-proposal-170/105-residual-option-audit.toml`
- M105 closure: `plans/closure/i2pcontrol-proposal-170/105-closure.md`

## Objective

Implement the exact Proposal 170/Java `DelayOpen` behavior for the six TCP-style
client families whose existing I2PControl client-listener owner can express it:
`client`, `httpclient`, `ircclient`, `socks`, `socksirc`, and `connectclient`.

Do not apply this handoff to `streamrclient`. M105 found no pinned or reference
definition of a first-local-client-socket event for Streamr's UDP/session loop.

## Readiness and invariants

The current listener creates `Session::new` before binding its local listener.
The existing owner can instead bind first and lazily create exactly one
generation-local Yosemite session on the first accepted local connection.

The implementation MUST preserve:

- fail-before-allocation validation for every other unsupported option;
- literal-loopback and existing local-listener bounds;
- per-name generation ownership, cancellation, edit/restart isolation, and
  bounded shutdown;
- one session per non-shared tunnel generation and no identity changes caused by
  `DelayOpen`;
- no session creation, SAM connection, or tunnel build while a delayed listener
  is idle;
- deterministic error behavior if cancellation wins while lazy session creation
  is in progress.

## Required production paths

Changes are confined to:

- `emissary-cli/src/i2pcontrol/domain/tunnel.rs`
- `emissary-cli/src/i2pcontrol/tunnel_manager.rs`
- `emissary-cli/src/i2pcontrol/backends/runtime/client_listener.rs`
- the six existing client-family composition files under
  `emissary-cli/src/i2pcontrol/backends/` listed above

No Cargo manifest, lockfile, Yosemite source, core/util crate, frontend, or
workflow change is authorized.

## Work packages

1. Add a typed `DelayOpen` option with lossless get/edit persistence and the
   existing pre-allocation validation path. Ensure only the six listed tunnel
   types advertise it as optional; Streamr remains explicitly rejected or
   separately classified.
2. Extend the existing client-listener runtime with a bounded lazy session
   owner. Bind and report the local listener first; initialize the session once
   at the first accepted local connection; share only that generation's session
   with bounded handlers.
3. Thread the typed value through the six existing backend composition seams.
   Do not duplicate a session owner in individual protocol backends.
4. Add focused fake-SAM and lifecycle tests for no eager `HELLO`/`SESSION CREATE`,
   first-connection creation, concurrent first connections, cancellation before
   first connection, cancellation during setup, failed setup recovery, and
   edit/restart generation isolation.

## Failure and lifecycle semantics

The local listener must still fail synchronously at start for bind/configuration
errors. A delayed session setup failure fails the triggering connection and
transitions the generation to the existing failed state without leaking tasks.
Stop, edit, and restart cancel a pending lazy initialization and drain bounded
handlers within the existing timeout. A later generation must never reuse a
session, destination, timer, or error from the prior generation.

## Compatibility and security

The new option changes only when the client session is allocated. Existing
definitions without `DelayOpen` retain eager startup behavior. No wire format,
destination/key format, persistence location, TLS trust rule, proxy fallback,
DNS behavior, or server target boundary changes.

## Verification

Run:

```text
cargo fmt --all -- --check
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m105_residual_option_audit
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment --test m062_dependency_containment
cargo test -p emissary-cli --no-default-features --features i2pcontrol
cargo check
git diff --check
```

Closure must include the updated M095 matrix cell dispositions, exact runtime
evidence for all six families, the Streamr exclusion, containment results, and a
recount before any M104 reattempt is considered.

## Stop conditions

Stop and return to planning if exact first-use semantics cannot be maintained
for all six families, if lazy initialization requires a new lower-layer owner,
if concurrent first connections cannot be bounded/cancellation-safe, or if the
implementation would alter Streamr behavior or any M093 anonymity boundary.
