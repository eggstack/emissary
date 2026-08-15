# I2PControl Proposal 170 Milestone M070 — Closure Status

Status: closed

Source implementation plan:

- `plans/implementation/i2pcontrol-proposal-170/070-http-bidirectional-server-composition.md`

Source subsystem roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-tunnel-runtime-completion-roadmap.md`

Planning production baseline: `a1296b018ce98d26a019bd5064dff9f4b47e0ad6`.

Implementation commit: this closing commit.

## 1. Executive finding

M070 is closed. `httpbidirserver` is a real control-plane-owned composite of
the accepted M067 HTTP server and M068 HTTP client paths. The inbound half
uses the exact M067 accepted-stream handler and the local proxy half uses the
exact M068 HTTP client handler with `outproxy = None`. A single generation
supervisor owns both child runtimes, reports ready only after both halves are
ready, tears down the sibling on partial startup/runtime failure, and retains
the one backend-owned persistent server destination identity across restart.

The local proxy uses a non-published sibling Yosemite stream session for
outbound I2P connections. It does not create or persist a second private server
destination. This is the accepted tightly scoped sibling-session semantics for
the deprecated Proposal 170 family.

## 2. Requirement-to-evidence matrix

| Requirement | Evidence | Result | Notes |
|---|---|---|---|
| M067 and M068 are closed | `plans/closure/.../067-closure.md`, `068-closure.md` | pass | Hard dependencies accepted before implementation |
| No third HTTP parser/filter/proxy path | `http_bidir.rs` imports `make_accepted_handler` and `make_no_outproxy_handler`; no parser implementation exists there | pass | Composition-only module |
| Exact M067 inbound security behavior | `http_server::make_accepted_handler` is shared by standalone and composite backends; M067 filter/backend tests pass | pass | Trusted peer, request, Host, proxy-header, throttle, and response behavior remain in M067 |
| Exact M068 outbound privacy behavior | `http_client::make_no_outproxy_handler` reuses M068 request parsing, target classification, sanitization, destination resolution, and relay | pass | Standalone HTTP-client tests remain green |
| Clearnet/outproxy disabled | `HTTP_BIDIR_SERVER_OPTIONS` and raw allowlist omit outproxy capability; `ProxyList` fails before allocation; handler parses with `None` outproxy | pass | Dedicated negative test |
| Singular persistent server identity | `ServerDestinationStore`, `SERVER_IDENTITY_KEY`, and production server-definition preparation include `HttpBidirServer` | pass | Client sibling session is non-published and ephemeral |
| Both halves required before running | composite readiness waits for accepted-server destination and client listener bind/session readiness | pass | Dedicated start/restart test |
| Partial startup is atomic | composite cancellation shuts down both child tasks when either readiness channel fails; bind-collision test passes | pass | No proxy-only running state |
| Unexpected half failure fails the composite | composite select cancels and boundedly drains the sibling, then supervisor marks `Failed` | pass | Child completion cannot leave composite reported running |
| Stop/restart exact generation | composite supervisor reserves one name/generation, rejects duplicate start, cancels both children, and removes the generation | pass | Dedicated lifecycle test |
| Identity survives restart | lifecycle test observes the same published server destination after stop/start | pass | Private material remains in backend-owned store |
| Option validation before allocation | typed union capability matrix, raw allowlist, loopback target validation, auth validation, and store lookup occur before supervisor reservation | pass | Outproxy and unsupported modes fail closed |
| Only `httpbidirserver` promoted | production registry maps only this additional type; Streamr remains unsupported/resource-free | pass | No public type/action/status/schema changes |
| StartOnLoad and server persistence seams updated | `production.rs` includes `HttpBidirServer` in identity load/prune, reconciliation, persistence, runtime-state, and preparation matches | pass | Startup-owned HTTP services untouched |
| Containment/default behavior | M061: 7 passed; M062: 19 passed; no `emissary-core/**`, manifest, startup proxy, CI, fuzz, or release changes | pass | New runtime remains under `emissary-cli/src/i2pcontrol/**` |
| Deprecated/composite/no-outproxy behavior documented | `docs/i2pcontrol/{README.md,proposal-170-support.md,tunnel-backends.md,tunnel-manager.md}` and planning records | pass | Two Streamr types remain unsupported |

## 3. Production implementation evidence

Production changes are confined to the I2PControl composition boundary:

- `emissary-cli/src/i2pcontrol/backends/http_bidir.rs` — composite option
  mapping, child-runtime readiness/failure coordination, generation-safe
  lifecycle, identity lookup, and tests;
- `emissary-cli/src/i2pcontrol/backends/http_server.rs` — exposes the existing
  M067 accepted handler seam for composition;
- `emissary-cli/src/i2pcontrol/backends/http_client.rs` — exposes the existing
  M068 handler with a structurally absent outproxy;
- `emissary-cli/src/i2pcontrol/backends/options.rs` — explicit bidirectional
  typed capability matrix;
- `emissary-cli/src/i2pcontrol/backends/registry.rs` — only the
  `httpbidirserver` production registration;
- `emissary-cli/src/i2pcontrol/production.rs` — durable identity and
  `StartOnLoad` composition seams;
- `emissary-cli/tests/m062_dependency_containment.rs` — current containment
  budget includes the M070 path and closure evidence.

No new HTTP parser, sanitizer, response filter, core session API, startup
listener adoption, public Proposal 170 field, action, status, tunnel type, or
persistence schema was added.

## 4. Verification executed

### Commands passed

```text
cargo test -p emissary-cli --no-default-features --features i2pcontrol http_bidir --lib -- --nocapture --test-threads=1
cargo test -p emissary-cli --no-default-features --features i2pcontrol http_server --lib -- --nocapture --test-threads=1
cargo test -p emissary-cli --no-default-features --features i2pcontrol http_client --lib -- --nocapture --test-threads=1
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment -- --nocapture --test-threads=1
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m062_dependency_containment -- --nocapture --test-threads=1
cargo check -p emissary-cli --no-default-features
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --lib -- -D warnings
rustfmt +nightly --check --edition 2021 --config-path rustfmt.toml emissary-cli/src/i2pcontrol/backends/http_bidir.rs emissary-cli/src/i2pcontrol/backends/http_client.rs emissary-cli/src/i2pcontrol/backends/http_server.rs
git diff --check
```

Results: M070 focused tests 4 passed; M067-focused tests 4 passed; M068-focused
tests 6 passed; M061 containment 7 passed; M062 containment 19 passed; default
and feature-enabled CLI checks passed; feature-enabled library Clippy passed;
the three composition/filter Rust files passed scoped nightly rustfmt; and
`git diff --check` passed.

The broad feature-enabled library test command was stopped after approximately
three minutes without producing a result. The required focused capability and
containment suites completed independently and are the accepted M070 evidence.

### Inherited verification limitations

These checks were executed and remain outside M070 scope:

- `cargo check -p emissary-core --no-default-features` fails on inherited
  feature-disabled `RwLock` import errors across unrelated core modules;
- `cargo fmt --all -- --check` fails on widespread pre-existing stable-toolchain
  formatting drift because this repository requires nightly rustfmt options;
- `cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings`
  fails only on the unchanged startup-owned `emissary-cli/src/proxy/socks.rs`
  `to_string`-inside-`format!` warning.

No M070-specific compile, lint, format, or diff failure remains.

## 5. Invariant review

All M070 hard invariants pass. Composition logic is I2PControl-local, the
inbound and outbound filters are reused unchanged, clearnet is rejected before
the listener/session runtime is reserved, the server identity is singular and
persistent, the two halves share one cancellation/generation owner, and the
composite cannot report running before both readiness events. Standalone M067
and M068 behavior is preserved by the narrow handler seams.

The production registry remains exhaustive. The default registry remains
resource-free unsupported for specialized types; production with the existing
server store promotes only the closed families, including M070, while Streamr
remains unsupported.

## 6. Failure and recovery review

Validation rejects unsupported typed/raw options, invalid loopback targets,
incomplete proxy authentication, non-loopback exposure without credentials,
clearnet requests without an explicit route, and outproxy configuration before
allocation. A client-listener bind failure cancels and drains the accepted
server half. A runtime exit of either child cancels and boundedly drains its
sibling and marks the composite failed. Stop is idempotent and bounded; a
duplicate start is rejected; restart is completed stop followed by a new
generation; stale completion is ignored by generation matching.

The local request path remains bounded by the M068 request/header/body limits,
and the inbound path remains bounded by M067 parser, connection, body, and
throttle limits. No lock is held across the composite lifecycle waits.

## 7. Migration and compatibility review

There is no wire, public schema, or persistence migration. Existing persisted
`httpbidirserver` definitions become startable only when their typed and raw
options fit the explicit composition matrix. The generated server identity
key/public destination fields remain backend-owned administrative state, and
private destination material is not copied into generic raw configuration.
Startup HTTP proxy/server ownership and behavior are unchanged.

## 8. Security review

The inbound path derives peer identity from SAM's trusted accepted stream and
uses M067's identity/proxy-header, framing, Host, access, and response-filter
policy. The local proxy uses M068's direct-I2P classifier and anonymity header
sanitizer. `None` is passed as the outproxy capability, and the negative test
proves a clearnet target cannot be classified as routable by the composed path.
Proxy credentials remain bounded/constant-time checked and are not included in
Debug/error output. No local DNS or startup proxy state is consulted for direct
I2P requests.

## 9. Documentation and operations

Updated support/backend/TunnelManager documentation describes the deprecated
composite, its two halves, sibling-session identity semantics, and no-outproxy
rule. The active registry, implementation handoff index, subsystem roadmap,
M070 plan, and this closure record now mark M070 closed and M071 Streamr as the
next registered ready handoff. M072 remains blocked on M071.

## 10. Unresolved findings

| Severity | Finding | Impact | Required action |
|---|---|---|---|
| low / inherited | Core no-feature check fails on unrelated `RwLock` imports | M070 has no core changes and does not consume that configuration | Preserve; address under separate core planning |
| low / inherited | Stable all-workspace formatting check reports pre-existing nightly-format drift | Formatting check cannot be used as a clean repository-wide gate | Preserve; use scoped nightly rustfmt for changed composition files |
| low / inherited | All-targets Clippy fails on unchanged startup `proxy/socks.rs` | Outside I2PControl and M070 ownership | Preserve; address under separate startup lint cleanup |
| low / evidence | Broad feature-enabled library test was stopped after a three-minute no-output hang | Focused M070/M067/M068 and containment evidence passed; no M070 failure observed | Keep broad-suite hang as a repository verification limitation; investigate separately |

No high- or medium-severity M070 filter-bypass, partial-lifecycle,
identity-duplication, outproxy-escape, persistence, or containment finding
remains open.

## 11. Roadmap disposition

Milestone closed and next dependency may proceed. M071 is dependency-ready and
is now the next registered handoff. M072 remains blocked until M071 closes.
The independent M051 RouterInfo blocker remains unchanged.

## 12. Registry updates

Applied in this change:

- M070 plan and closure are `closed`;
- the tunnel-runtime roadmap is `active; M064-M070 closed; M071 is the next
  registered handoff`;
- the active registry now names M071 as the sole dependency-ready handoff;
- the implementation README and support documentation describe M070 as real
  and Streamr as the remaining unsupported family;
- the M062 containment allowlist recognizes the M070 production path and
  closure evidence.

Internal-only attestation: external specifications and reference material were
read-only. No upstream repository, issue, pull request, review, merge,
adoption, submission, contribution package, or maintainer channel was mutated
or prepared. Repository writes remained within the authorized internal
`eggstack/emissary` repository.

Final disposition: **closed**.
