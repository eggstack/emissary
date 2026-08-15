# M067 — HTTP Server Tunnel Closure

Status: closed

Implementation commit: `4512966867e3804fac06cb71d70cd4662adbbeb9`.

## Disposition

M067 is closed. `httpserver` is a real control-plane-owned accepted-stream
backend. `httpclient`, `connectclient`, and `httpbidirserver` remain explicit
unsupported backends; no neighboring family was promoted by this work.

## Requirement-to-evidence matrix

| Requirement | Evidence | Result |
|---|---|---|
| M065 accepted-stream path | `backends/http_server.rs` calls `run_accepted_server`; no generic `STREAM FORWARD` path is used | pass |
| Request bounds | `backends/filters/http.rs`: 5s request-line timeout, 15s header timeout, 8 KiB line limit, 64-header limit, 32 KiB aggregate limit | pass |
| Header/parser safety | strict CRLF, token field names, no obs-fold, control/NUL rejection, absolute/origin target normalization, HTTP/1.0/1.1 only | pass |
| Framing safety | duplicate Content-Length must agree; overflow/non-numeric values reject; Transfer-Encoding and upgrade reject before target connect | pass |
| Trusted identity | all supported `X-I2P-*` variants are removed; B64/B32 values are injected only from `TrustedPeerIdentity` | pass |
| Proxy identity safety | Forwarded/Via/X-Forwarded*/Proxy headers are stripped; `BlockAccessInProxies` rejects them before local connect | pass |
| Host/target safety | target address is definition-owned and loopback-only; configured WebsiteHostname/SpoofedHost controls Host and cannot select target | pass |
| Access/filter policy | access allow/deny list, referer, User-Agent, and proxy policy are evaluated before local connect | pass |
| Bounded throttling | `MaxConcurrentConns` is limited to M065’s 128-task ceiling; POST/PUT/PATCH state is peer-keyed, expiry-based, and capped at 1024 entries | pass |
| Request/body streaming | sanitized headers are serialized before local connect; body uses bounded Content-Length streaming and a 60s inactivity/failsafe timeout | pass |
| Response filtering | response headers are bounded/validated; Server, X-Powered-By, X-Runtime, Proxy, Proxy-Connection and hop-by-hop headers are removed | pass |
| Upgrade behavior | HTTP upgrade/WebSocket requests are rejected deterministically; no raw upgrade bypass exists | pass |
| Local failure handling | connect/response failures return bounded generic 502 responses without target details | pass |
| Lifecycle | per-name generation supervisor owns start/stop, accepted tasks, cancellation, duplicate-start rejection, and bounded shutdown | pass |
| Persistent identity | existing backend-owned server destination store and production preparation/persistence seams now include `httpserver`; private material remains redacted | pass |
| Option capability | supported and rejected canonical HTTP/security/I2CP/custom options are checked before destination lookup/session allocation | pass |
| Registry containment | only `httpserver` changes from unsupported; `httpclient`, `connectclient`, and `httpbidirserver` remain unsupported/resource-free | pass |
| Source containment | HTTP production logic is under `emissary-cli/src/i2pcontrol/**`; no `emissary-core/**`, manifest, startup proxy, CI, fuzz, or release change | pass |

## Filter and adversarial evidence

The filter suite covers:

- multiple casing and spoofed `X-I2P-DestB64`/identity headers;
- Forwarded/X-Forwarded/Proxy handling;
- duplicate/conflicting Content-Length and Transfer-Encoding ambiguity;
- obs-fold, missing-colon, malformed CRLF, control characters, and bounded input;
- absolute-target normalization and configured Host rewriting;
- access, referer, User-Agent, and proxy policy;
- response fingerprint stripping and preserved response framing.

The HTTP backend suite adds a local capture-server path. It asserts that the
local service receives the normalized request only after filtering and that the
remote peer receives a response with the server fingerprint removed. Unsupported
target/security options fail in configuration validation before destination
lookup.

## Exact verification

The following commands passed against the implementation commit:

```text
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo check -p emissary-core --no-default-features
cargo test -p emissary-cli --no-default-features --features i2pcontrol backends::filters::http --lib -- --nocapture --test-threads=1
cargo test -p emissary-cli --no-default-features --features i2pcontrol backends::http_server --lib -- --nocapture --test-threads=1
cargo test -p emissary-cli --no-default-features --features i2pcontrol backends::registry --lib -- --nocapture --test-threads=1
cargo test -p emissary-cli --no-default-features --features i2pcontrol production --lib -- --nocapture --test-threads=1
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment -- --nocapture --test-threads=1
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m062_dependency_containment -- --nocapture --test-threads=1
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --lib -- -D warnings
git diff --check
```

The stable and nightly `cargo fmt --all -- --check` commands report inherited
formatting drift in unrelated pre-existing files; the changed HTTP files were
formatted with the repository’s nightly rustfmt. The feature-disabled CLI
package check passed. `cargo check -p emissary-core --no-default-features`
retains the inherited feature-disabled `RwLock` import failure in unrelated
core paths; M067 adds no core or manifest changes.

## Compatibility, security, and operational review

No public Proposal 170 field or persistence schema changed. Existing persisted
`httpserver` definitions start only when their option set fits the implemented
matrix; unsupported modes remain persisted but fail closed at start. Runtime
throttle state is ephemeral and restart clears it. Server destination identity
survives stop/restart through the existing secret store. No local target host
can be selected from request Host, absolute-form authority, or peer input.

No unresolved high- or medium-severity request-smuggling, identity-spoofing,
SSRF/open-proxy, or resource-exhaustion finding remains. Public-network
interoperability was not claimed; fake-SAM and local capture evidence covers the
bounded administrative/runtime contract.

Internal-only attestation: all repository writes are scoped to
`eggstack/emissary`. No upstream or third-party issue, review, merge,
submission, contribution package, or maintainer contact was prepared.

## Successor disposition

M068, M069, and M071 are dependency-ready because M065 and the M066 dependency
where required are closed. Per the one-next-handoff registry rule, M068 is now
registered `ready`; M069 and M071 remain blocked only because they are not the
next registered handoff. M070 remains blocked on M068 as well as M067, and M072
remains blocked on all runtime-family closures. The independent M051
RouterInfo blocker is unchanged.

Final disposition: **closed**.
