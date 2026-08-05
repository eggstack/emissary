# I2PControl Proposal 170 M039 Closure Invalidation

Status: corrective pass required

Invalidated closure:

- `plans/closure/i2pcontrol-proposal-170/039-closure.md`

Corrective owners:

- M040, `plans/implementation/i2pcontrol-proposal-170/040-startup-server-cancellation-correction.md`
- M041, `plans/implementation/i2pcontrol-proposal-170/041-auth-throttle-source-accounting.md`
- M042, `plans/implementation/i2pcontrol-proposal-170/042-addressbook-subscription-commit-boundary.md`
- M043, `plans/implementation/i2pcontrol-proposal-170/043-corrective-runtime-regression-validation.md`
- M044, `plans/implementation/i2pcontrol-proposal-170/044-corrective-final-head-reclosure.md`

Repository head reviewed:

- `563e093ba1e65b4edc31104e3045c8b5a665e8ed`

Review date: 2026-08-05

## 1. Decision

M039's final subsystem disposition is invalidated. Its retained evidence remains
usable where the defects below do not directly contradict it, but its claim that
no high- or medium-severity correctness, security, compatibility, ownership,
containment, or evidence defect remained is no longer controlling.

The current subsystem status is `corrective pass required`.

The expected bounded status after correction remains `partial Proposal 170
support`. This invalidation does not authorize unavailable RouterInfo sources,
new tunnel data planes, startup-task adoption, router changes, or broader CI.

## 2. Newly demonstrated defects

### 2.1 Startup-managed generic server tunnels can cancel immediately

`ServerTunnelManager::server_event_loop` creates a watch channel as
`let (_, cancellation) = tokio::sync::watch::channel(false)`. The sender is
therefore dropped before `run_single_server` begins its startup selection.

`run_single_server` treats `cancellation.changed()` completion as cancellation.
A closed watch channel is immediately ready, so an existing startup-configured
server tunnel can exit before creating its SAM session, publishing its
destination, or installing `STREAM FORWARD`.

This is a high-severity regression outside the I2PControl module. It contradicts
M032/M039 claims that the reusable server runtime preserved startup behavior and
that external production changes were behavior-preserving.

### 2.2 Failed-authentication throttling is keyed by ephemeral port

`AuthThrottle` stores failures by full `SocketAddr`. A reconnect from the same
host normally uses a new source port and therefore receives a fresh throttle
entry. The accumulated delay can be bypassed through ordinary reconnects.

The delay is also calculated before the failure is recorded. Concurrent invalid
attempts may observe the same prior count and receive the same lower delay.

This invalidates the M036/M039 claim that failed authentication is effectively
bounded per peer. The reviewed constant-time password comparison and unrelated
publication evidence remain retained.

### 2.3 AddressBook subscription failure semantics remain ambiguous

The live AddressBook command path durably commits and activates a replacement
subscription set before it attempts to enqueue refresh work. If the refresh
worker is unavailable after that commit, the request can return an error even
though the requested mutation has already occurred.

A retrying client cannot distinguish no mutation from a completed mutation with
a failed follow-up refresh notification. This invalidates the narrow M034/M039
claim that the setter has one truthful success/failure boundary. The correction
does not reopen AddressBook entry ownership, destination coherence, or the
explicit unsupported `SetConfig` disposition.

### 2.4 Prior verification did not cover the failing paths

The M038 live fixture configures a startup client tunnel but no startup server
tunnel. Its server checks exercise control-plane-created definitions and do not
run the original `ServerTunnelManager` startup path.

Authentication tests verify bounded capacity and delay values but do not assert
that the same IP across different ephemeral ports shares one throttle identity,
nor that concurrent failures reserve distinct counts before sleeping.

AddressBook tests prove live manager control and persistence but do not force a
refresh-worker failure after durable subscription commit.

The broad passing test count therefore did not exercise the exact regressions.

## 3. Retained evidence

Unless a corrective milestone exposes a direct additional defect, retain:

- exact Proposal 170 wire names, casing, types, direct-presence semantics, and
  compatibility inventories;
- generic control-plane client and server backend ownership, per-name
  supervision, generation fencing, fixed server-secret paths, and lifecycle
  serialization;
- explicit resource-free unsupported disposition for the other ten tunnel
  families;
- startup-managed inventory and administrative mutation rejection;
- AddressBook entry ownership, full-destination coherence, disabled-mode
  isolation, and explicit non-empty `SetConfig` rejection;
- RouterInfo's 16 available / 1 neutral / 26 unavailable source matrix;
- ClientServicesInfo and bounded SAM observation behavior;
- constant-time password comparison;
- publication confinement, permissions, recovery, and directory-sync
  qualification;
- M037 containment reduction except for the startup-server behavior regression
  in the reused CLI adapter;
- internal-only/no-upstream compliance.

M040–M044 must not reimplement or broaden these retained dimensions.

## 4. Required corrective boundary

### M040

Correct only the startup server cancellation-owner lifetime and add direct
regression evidence that the original `ServerTunnelManager` reaches SAM session
creation, destination observation, and forwarding.

Production changes outside `i2pcontrol/**` are limited to
`emissary-cli/src/tunnel/server.rs`. No core or control-plane redesign is
authorized.

### M041

Normalize failed-authentication identity to source IP and make failure-count
reservation atomic before delay. Preserve existing wire errors, token behavior,
connection bounds, and password comparison.

### M042

Define one truthful `SetSubscriptions` operation boundary. A response must not
report failure after the replacement set has durably committed. Refresh remains
bounded follow-up work owned by the existing manager; no scheduler or second
owner is authorized.

### M043

Run focused regressions and the existing bounded matrix against the combined
corrective head. It must include the original startup server path, ephemeral-port
throttle bypass, concurrent failure accounting, and post-commit refresh-worker
failure semantics.

### M044

Independently review the final corrective head and select the truthful subsystem
status. M044 is documentation/review only and may not patch production defects.

## 5. Scope retained

The corrective sequence does not authorize:

- HTTP, HTTP server/bidirectional server, IRC, SOCKS-IRC, CONNECT, Streamr, or
  other missing tunnel data planes;
- control or adoption of startup-managed tasks;
- new RouterInfo sources or fabricated values;
- router, transport, streaming, LeaseSet, cryptographic, routing, or
  tunnel-building changes;
- frontend work;
- broad crate extraction or service frameworks;
- remote CI matrices, release automation, coverage gates, fuzzing, soak tests,
  or generated evidence systems;
- upstream issues, pull requests, reviews, submissions, adoption requests,
  merges, maintainer contact, or contribution preparation.

## 6. Closure consequence

Until M040–M044 close:

- M039 is historical invalidated evidence;
- `partial Proposal 170 support` is not the current controlling subsystem
  status;
- the registry must show `corrective pass required` and M040 as the only
  dependency-ready handoff;
- M032, M034, and M036 retain unaffected evidence but their contradicted closure
  claims are non-controlling;
- no deferred RouterInfo or tunnel-family work is unblocked.

## 7. Internal-only attestation

This invalidation concerns internal repository correctness only. No upstream or
third-party issue, pull request, review, submission, adoption request, merge
request, maintainer outreach, or contribution artifact is authorized.