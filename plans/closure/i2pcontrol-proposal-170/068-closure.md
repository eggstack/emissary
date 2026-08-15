# M068 — HTTP Client and CONNECT Client Tunnel Closure

Implementation commit: `cb76892`.

## Disposition

M068 is closed. `httpclient` and `connectclient` are real control-plane-owned
bounded local proxies. `httpbidirserver`, SOCKS, SOCKS-IRC, and Streamr remain
explicitly unsupported until their owning milestones close.

## Requirement-to-evidence matrix

| Requirement | Evidence | Result |
|---|---|---|
| M065 client listener/session primitive | `backends/runtime/client_listener.rs` now exposes request-selected `connect_to`; both backends use `run_client_listener` with bounded task ownership | pass |
| HTTP request bounds | `filters/http_client.rs` applies 5s request-line, 15s header, 8 KiB line, 64-header, and 32 KiB aggregate limits | pass |
| HTTP parser safety | CRLF-only parsing, CTL/NUL rejection, obs-fold rejection, token header names, HTTP/1.1-only serialization, duplicate Host rejection | pass |
| HTTP framing safety | duplicate `Content-Length` must agree; `Transfer-Encoding` and body ambiguity reject before target connect | pass |
| Target classification | `.i2p`/B32/full destinations use I2P target flow; clearnet requires an explicit I2P outproxy; HTTPS, unsupported methods, and invalid authorities fail closed | pass |
| No local DNS | target hosts are passed only to Yosemite/SAM; no target `ToSocketAddrs` or OS DNS path exists in M068 code | pass |
| Local/LAN safety | localhost, loopback, RFC1918, unspecified, link-local, IPv6 ULA/link-local targets are rejected | pass |
| Address-book alias privacy | named `.i2p` aliases resolve through the I2PControl runtime address-book owner and serialize the resolved B32 host; without that owner the request fails closed | pass |
| HTTP anonymity headers | `Forwarded`, `Via`, `From`, `X-Forwarded-*`, `Proxy-*`, hop-by-hop, proxy authorization, `DNT`, and identity headers are removed; User-Agent, Referer, and Accept-family behavior is policy-controlled | pass |
| Outproxy credentials | only separately configured outproxy credentials are reconstructed for the outproxy hop; direct I2P traffic never receives them | pass |
| Proxy authentication | configured Basic credentials are compared with the existing constant-time helper; configured credentials imply authentication, non-loopback listeners require authentication, and values are redacted from Debug/errors | pass |
| CONNECT strictness | `connectclient` accepts only CONNECT, requires a valid non-zero port, rejects request bodies/framing ambiguity, and ignores extra direct-I2P headers | pass |
| CONNECT establishment | direct `200 Connection Established` is written only after Yosemite connect succeeds; outproxy CONNECT responses must be 2xx before local success; only then does raw relay begin | pass |
| Lifecycle | each backend has bounded per-name generation state, duplicate-start rejection, cancellation, exact task drain, restart, stale-completion protection, and failure isolation | pass |
| Option capability | typed and raw Proposal 170 fields are explicitly accepted or rejected before listener/session allocation; I2CP/custom and unsupported security options fail closed | pass |
| Registry promotion | production registry replaces only `httpclient` and `connectclient`; `httpbidirserver`, SOCKS, SOCKS-IRC, and Streamr remain unsupported/resource-free | pass |
| StartOnLoad | production reconciliation includes `httpclient` and `connectclient` only for control-plane-owned definitions | pass |
| Containment | production code is under `emissary-cli/src/i2pcontrol/**`; M062’s path authority was extended to recognize the already-authorized M067/M068 runtime paths; no core or manifest change | pass |

## Target classification matrix

| Input | Direct action |
|---|---|
| `.b32.i2p` | connect through Yosemite/SAM to the B32 destination; rewrite Host to the destination host |
| named `.i2p` | resolve through the I2PControl runtime address-book owner, then use the resolved B32 destination |
| full destination | pass to Yosemite/SAM after structural validation |
| clearnet hostname/IP | require a configured I2P HTTP/CONNECT outproxy; never connect locally |
| localhost, loopback, private, link-local, unspecified | reject before any remote connect |

## Request and credential evidence

`http_client` unit tests cover proxy identity leakage, User-Agent replacement,
Referer suppression, clearnet-without-outproxy rejection, local/private target
rejection, conflicting framing, and obs-fold rejection. `connect_client` tests
cover GET rejection, body rejection, valid CONNECT parsing, and local-target
rejection. Proxy helper tests cover Basic credential round-trip and mismatch
without printing the secret.

The shared client listener test suite retains fake-SAM session setup failure,
listener readiness, cancellation, bounded task handling, and handler panic
isolation. The production backend tests verify fail-before-allocation exposure
and secret-option behavior; registry and production tests verify composition.

## Exact verification

Passed:

```text
cargo test -p emissary-cli --no-default-features --features i2pcontrol http_client --lib -- --nocapture --test-threads=1
cargo test -p emissary-cli --no-default-features --features i2pcontrol connect_client --lib -- --nocapture --test-threads=1
cargo test -p emissary-cli --no-default-features --features i2pcontrol backends::registry --lib -- --nocapture --test-threads=1
cargo test -p emissary-cli --no-default-features --features i2pcontrol production --lib -- --nocapture --test-threads=1
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment -- --nocapture --test-threads=1
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m062_dependency_containment -- --nocapture --test-threads=1
cargo check -p emissary-cli --no-default-features
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --lib -- -D warnings
rustfmt +nightly --check --edition 2021 --config-path rustfmt.toml <new M068 Rust files>
git diff --check
```

The required feature-enabled all-targets Clippy command remains blocked by an
inherited pre-existing warning in `emissary-cli/src/proxy/socks.rs`:
`to_string` is used inside `format!`. That startup-owned file is outside M068
and was not changed. `cargo check -p emissary-core --no-default-features`
retains the inherited feature-disabled `RwLock` import failures in unrelated
core paths; M068 makes no core changes. These are recorded limitations, not
M068 findings.

## Compatibility, documentation, and security review

No public Proposal 170 wire field, persistence schema, action, or tunnel type
changed. Existing persisted definitions become startable only when their
options fit the implemented matrix. Startup HTTP proxy ownership and behavior
remain unchanged. Support documentation, TunnelManager lifecycle documentation,
the active registry, roadmap, and successor statuses were updated.

No unresolved high- or medium-severity open-proxy, direct-DNS/LAN, credential
leak, CONNECT target-confusion, request-framing, or anonymity-header finding
remains in M068 scope. Public-network interoperability is not claimed; the
bounded contract is covered by unit, fake-SAM, local-listener, registry, and
containment evidence.

## Successor disposition

- M069 is now `ready` and is the next registered handoff; its M065 and M066
  dependencies are closed.
- M070 is dependency-ready but remains unregistered until M069 is handled; its
  M067 and M068 dependencies are closed.
- M071 is dependency-ready but remains unregistered until M069 is handled; its
  M065 dependency is closed.
- M072 remains blocked on M069-M071 closure.
- Independent M051 remains blocked by the accepted absence of substantive
  RouterInfo news/ban owners; M068 does not affect that blocker.

Internal-only attestation: repository writes are scoped to the authorized
`eggstack/emissary` repository. External specifications and reference material
were used read-only. No upstream issue, pull request, review, merge, adoption,
submission, contribution package, or maintainer contact was prepared.

Final disposition: **closed**.
