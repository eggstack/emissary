# M082 Closure — HTTP Peer Identity and Expect-Framing Corrective

Status: closed

Source implementation plan:

- `plans/implementation/i2pcontrol-proposal-170/082-http-peer-identity-and-expect-framing-corrective.md`

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-tunnel-security-hardening-roadmap.md`

Corrective predecessor closures:

- M076 closure: `plans/closure/i2pcontrol-proposal-170/076-closure.md` —
  `corrective pass required`; M082 owns the structural-Destination bound,
  `Expect`, and POST peer-key defects.
- M080 closure: `plans/closure/i2pcontrol-proposal-170/080-closure.md` —
  canonical `TrustedPeerIdentity` boundary that M082 consumes.
- M081 closure: `plans/closure/i2pcontrol-proposal-170/081-closure.md` —
  sequencing gate that unblocked the M082 dependency-ready handoff.

Planning production baseline: `1618de172e7a78a193fc1bb117af269f31174030`.

Implementation commit: recorded in the M082 implementation commit
message (`feat(i2pcontrol): implement M082 HTTP peer identity and Expect-framing corrective`).

## 1. Retained implementation evidence

M076 established the HTTP anonymity and POST-throttle architecture that
remains accepted and unreverted:

- Java-parity response filtering including `Date`, `Server`, `X-Powered-By`,
  `X-Runtime`, and proxy headers;
- broader I2P+-style provider/cache/trace fingerprint stripping;
- request-side proxy identity stripping including `Forwarded`, `Via`,
  `X-Forwarded-*`, `X-Real-IP`, `CF-Connecting-IP`, `True-Client-IP`,
  `Fastly-Client-IP`, `X-Client-IP`, `X-Cluster-Client-IP`, `Priority`,
  `Sec-GPC`;
- spoofed `X-I2P-*` removal and trusted identity reinjection;
- preserved validated response framing (Content-Length / chunked);
- no application-body rewriting;
- POST/PUT/PATCH rejection before local target allocation;
- fixed-size fail-closed POST table without eviction of active/unexpired
  state;
- shared inbound handler/filter path for `httpserver` and
  `httpbidirserver`;
- fixed bounded error responses;
- I2PControl source containment.

M080 established the canonical 32-byte cryptographic peer-identity helper
that M082 reuses unchanged:

- `TrustedPeerIdentity::from_stream` is the sole ingress for remote
  identity;
- the 32-byte SHA-256 hash of the parsed Destination is the only key
  used by security accounting;
- `MAX_TRUSTED_DESTINATION_B64_TEXT = 1024` is the textual ingress bound.

M082 adds the `Expect` rejection and POST-key correction while leaving
every M076/M080 property intact.

## 2. Confirmed regression and root cause

Independent review of head `1618de172e7a78a193fc1bb117af269f31174030`
found that:

1. `emissary-cli/src/i2pcontrol/backends/filters/http.rs:27` declared
   `MAX_TRUSTED_DESTINATION_TEXT = 524` as a magic textual ceiling on the
   accepted remote Destination. The constant is derived from a 391-byte
   reference Destination assumption but is not actually bounded by the
   repository `Destination::parse`; valid current key-certificate forms
   (EdDSA-SHA512-Ed25519 / EcDSA-SHA256-P256) serialize to exactly 391
   bytes and encode to 524 base64 characters, so any future larger
   supported key form would be silently rejected. The structural
   validation that already exists at the M080
   `TrustedPeerIdentity::from_stream` boundary is the authoritative
   source of identity validity; HTTP must consume that boundary rather
   than maintain a second magic-length check.
2. `emissary-cli/src/i2pcontrol/backends/filters/http.rs` did not
   inspect or reject `Expect` request headers. The accepted-stream flow
   forwards the sanitized request head to the loopback target and then
   copies the remote body before reading the local response. A remote
   client sending `Expect: 100-continue` may legitimately pause until it
   sees a 100 interim response from the backend, which the backend may
   emit immediately while Emissary is still blocked on the body. Without
   a relay implementation that request can pin an accepted handler slot
   until `BODY_TIMEOUT = 60s`.
3. `emissary-cli/src/i2pcontrol/backends/http_server.rs:76` defined
   `PostPeerKey([u8; 8])` and derived it from an eight-byte
   `DefaultHasher` digest of the textual peer string. The shared
   admission state was already keyed by the canonical 32-byte
   Destination hash from M080; HTTP write throttling should use the same
   identity surface rather than a weaker 64-bit digest.

## 3. Yosemite capability confirmation (WP1)

The pinned Yosemite `0.7.0` `SessionOptions` and `Stream` surface
(`/home/sugarwookie/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/yosemite-0.7.0/src/`)
continues to expose `Stream::remote_destination()` as the validated
I2P destination text. `TrustedPeerIdentity::from_stream` already
consumes this surface; M082 reuses it through a new
`read_and_sanitize_request(..., &TrustedPeerIdentity, ...)` parameter
without changing Yosemite usage or adding a new I2PControl-only
dependency. No new `emissary-core/**` production path, no Yosemite
fork, and no SAM protocol extension is required.

The repository's `emissary_core::primitives::Destination::parse`
supports the 387-byte null-certificate form and the 391-byte
key-certificate form (EdDSA-SHA512-Ed25519 or
EcDSA-SHA256-P256). Both forms are accepted by the M080 structural
validator that M082 now consumes. P521/RSA-2048/3072/4096 forms are
not currently supported by the repository parser, so M082 makes no
claim about them.

## 4. Applied corrections

### 4.1 Structural peer identity through HTTP filter (WP1+WP2)

`emissary-cli/src/i2pcontrol/backends/filters/http.rs` now consumes
`&TrustedPeerIdentity` instead of `&str`:

```text
TrustedPeerIdentity::from_stream (& Stream)
  -> validated destination text (bounded by MAX_TRUSTED_DESTINATION_B64_TEXT)
  -> canonical_id [u8; 32]
read_and_sanitize_request(reader, peer: &TrustedPeerIdentity, policy)
  -> peer.destination() -> X-I2P-DestB64 header
  -> base32_for_destination(peer.destination()) -> X-I2P-DestB32 header
```

The `MAX_TRUSTED_DESTINATION_TEXT = 524` constant is removed. The
`validate_trusted_destination` helper is removed because the M080
`TrustedPeerIdentity::from_stream` boundary already enforces the
textual ingress bound (`MAX_TRUSTED_DESTINATION_B64_TEXT = 1024`) plus
base64/control/whitespace/structural validation. Any non-structurally
valid remote destination is rejected upstream before any handler or
filter work, so HTTP cannot observe an invalid identity.

The injected `X-I2P-DestB64` and `X-I2P-DestB32` headers are bounded
by the structurally validated remote Destination plus base32 expansion
of the canonical 32-byte hash. No new public Proposal 170 field,
alias, method, or tunnel type is introduced.

### 4.2 `Expect` fail-closed handling (WP3)

A new categorical error type `RequestSanitizeError` replaces the prior
`io::ErrorKind` mapping at the HTTP filter boundary. The variant
`RequestSanitizeError::ExpectUnsupported` carries the fixed
`417 Expectation Failed` status line used by every Expect rejection.

```text
HTTP request headers -> read_and_sanitize_request
  -> if any `Expect` header (case-insensitive, single, duplicate,
     mixed-case `100-Continue`, unknown expectation token):
       RequestSanitizeError::ExpectUnsupported
       -> 417 Expectation Failed + Connection: close + Content-Length: 0
       -> handle_http_stream returns without TcpStream::connect
```

`handle_http_stream` (`emissary-cli/src/i2pcontrol/backends/http_server.rs:453`)
maps the categorical error to its fixed status line and writes the
bounded response through `send_error`. No request body is read after
the rejection; no local target connection is opened. The error path
releases the M080 `AdmissionLease` on handler return exactly as before.

### 4.3 POST peer key canonicalization (WP4)

`PostLimiter` (`emissary-cli/src/i2pcontrol/backends/http_server.rs:69`)
is updated:

```text
PostPeerKey [u8; 8]   DefaultHasher(peer_text)  -- removed
PostPeerKey [u8; 32]  peer.canonical_id()       -- applied
```

`PostPeerKey::from_peer(&TrustedPeerIdentity)` derives the fixed-size
key from `peer.canonical_id()` — the same 32-byte SHA-256 Destination
hash consumed by `ServerAdmissionState`. The auxiliary expiry
`VecDeque<PostExpiry>` is unchanged in shape but inherits the
bounded-entries invariant from `MAX_THROTTLE_ENTRIES = 1024` plus the
M076 fail-closed "active/unexpired entries are never evicted" rule.
No raw peer Destination is stored solely for throttling.

## 5. Regression tests (WP5+WP6)

The new tests live next to the corrected code. The plan's required
matrix is satisfied:

- `accepts_canonical_trusted_peer_from_stream_helper`
  (`filters/http.rs`) — a 387-byte null-certificate Destination is
  accepted; injected `X-I2P-DestB64` matches the canonical text;
- `accepts_largest_supported_key_certificate_destination`
  (`filters/http.rs`) — a 391-byte EdDSA-SHA512-Ed25519
  key-certificate Destination (the largest supported form, 524
  base64 characters) is accepted and emits both `X-I2P-DestB64` and
  `X-I2P-DestB32` headers;
- `rejects_over_bound_destination_text_before_request_construction`
  (`filters/http.rs`) — text longer than
  `MAX_TRUSTED_DESTINATION_B64_TEXT = 1024` is rejected by the
  upstream `TrustedPeerIdentity::from_destination_text` validator
  and never reaches the filter;
- `rejects_malformed_destination_text_before_request_construction`
  (`filters/http.rs`) — empty text, placeholder strings
  (`"peer-destination"`), and non-base64 inputs
  (`"not valid base64!@#"`) all fail upstream validation;
- `rejects_expect_100_continue_before_local_allocation`
  (`filters/http.rs`) — `Expect: 100-continue` returns
  `RequestSanitizeError::ExpectUnsupported` and the
  `417 Expectation Failed` status line;
- `rejects_mixed_case_expect_continuation_token`
  (`filters/http.rs`) — `Expect: 100-Continue` is rejected identically;
- `rejects_unknown_expectation_token`
  (`filters/http.rs`) — `Expect: 102-processing` is rejected;
- `rejects_duplicate_expect_headers`
  (`filters/http.rs`) — two `Expect: 100-continue` headers are rejected;
- `plain_post_without_expect_still_forwards`
  (`filters/http.rs`) — ordinary POST without `Expect` still produces
  a `SanitizedRequest` with the expected method and content length;
- `normalizes_absolute_target_and_removes_spoofed_identity`
  (`filters/http.rs`) — accepted Destination identity reaches the
  reinjected `X-I2P-DestB64` header unchanged, attacker-supplied
  `x-i2p-destb64` is removed;
- `rejects_smuggling_and_malformed_headers`
  (`filters/http.rs`) — conflicting CL, CL/TE, obs-fold, and missing
  colon still fail closed;
- `applies_proxy_referer_user_agent_and_access_policy`
  (`filters/http.rs`) — proxy/referer/User-Agent/access policy still
  applies with structurally valid peer;
- `removes_proxy_identity_and_adopted_request_privacy_headers`
  (`filters/http.rs`) — every M076 denylisted request header still
  fails closed;
- `preserves_valid_chunked_response_framing`,
  `response_filter_removes_fingerprints_and_hop_by_hop_headers`,
  `removes_every_adopted_response_fingerprint_case_insensitively`
  (`filters/http.rs`) — M076 response fingerprint and framing
  protections remain green;
- `post_limiter_is_bounded_and_peer_keyed`,
  `post_limiter_keys_distinct_peers_independently`,
  `post_limiter_denies_churn_without_evicting_active_entries`,
  `post_limiter_counts_only_write_methods`
  (`http_server.rs`) — distinct peer identities produce distinct
  32-byte keys; the 1024-entry table remains fail-closed under
  identity churn without active-state eviction; expired entries
  remain reclaimable; repeat write counts throttle the same peer;
- `rejected_post_does_not_connect_to_local_backend`
  (`http_server.rs`) — a rejected write never opens a local target
  connection;
- `expect_request_is_rejected_with_417_before_local_allocation`
  (`http_server.rs`) — end-to-end proof that an `Expect` request
  emits `417 Expectation Failed`, never reaches the loopback
  target, and returns within the request-handling window (the
  150 ms local listener accept timeout expires with no connection);
- `end_to_end_path_sanitizes_before_local_connect_and_filters_response`
  (`http_server.rs`) — the full sanitization + proxy stripping +
  Host rewrite + response fingerprint removal path remains correct
  with the canonical trusted peer identity.

The pre-existing M074/M076/M080 regression suites
(`http_server_options_*`, `admission_*`,
`generic_server_uses_accepted_stream_and_relays_bytes_without_forwarding`,
`accepted_peer_identity_reaches_handler_before_local_target`,
`malformed_remote_destination_is_rejected_before_handler_invocation`,
`malformed_destination_text_is_rejected_before_admission`,
`expiry_index_live_entries_remain_bounded_under_repeated_acquire_drop`,
`repeated_acquire_drop_for_one_peer_does_not_grow_expiry_index`,
`lease_drop_releases_active_count_exactly_once`, …) all pass
unchanged against the corrected implementation.

`make_accepted_handler` is the single seam consumed by both
`httpserver` and the inbound half of `httpbidirserver`. The corrected
filter/limiter behavior is consumed unchanged by
`HttpBidirServerTunnelBackend::run_composite` via the same seam, so
the `httpbidirserver` inbound path automatically inherits the
structural-Destination acceptance, `Expect` rejection, and
canonical-peer POST key without code duplication. The M076
`make_accepted_handler` seam and its `Arc<dyn Fn(AcceptedServerConnection)
-> BoxFuture<'static, ()> + Send + Sync>` signature are preserved.

## 6. Failure, cancellation, restart, and contention semantics

- unsupported expectation returns before local `TcpStream::connect`
  and before any application bytes leave the accepted-stream session;
- no request body is read after the Expect rejection — the handler
  drops the lease and the reader is shut down through ordinary
  `remote_write.shutdown` semantics within the rejection response;
- the M080 admission lease is released exactly once when the
  handler returns; the `expect_request_is_rejected_with_417_before_local_allocation`
  regression covers this path explicitly;
- structurally invalid / over-bound Destination text fails before
  request construction, local target, and any limiter state;
- local backend failure remains `502 Bad Gateway` without target
  detail;
- request timeout behavior remains bounded by `REQUEST_LINE_TIMEOUT`,
  `HEADER_TIMEOUT`, and `BODY_TIMEOUT`;
- restart does not persist POST limiter state — a fresh
  `HttpServerRuntimeSupervisor` constructs a fresh `PostLimiter`;
- `httpbidirserver` consumes the corrected path through the shared
  `make_accepted_handler`; no second filter/limiter exists;
- one malformed/unsupported request cannot fail the whole tunnel
  runtime — handler errors are caught by the existing
  `AssertUnwindSafe(...).catch_unwind()` wrapper in
  `HttpServerRuntimeSupervisor::start`.

## 7. Verification

The commands from section 12 of the plan were executed against the
implementation commit and produced the recorded outcomes:

| Command | Outcome |
|---|---|
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol` | 1622 passed (24 suites) |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol http` | 115 passed |
| `cargo check -p emissary-cli --no-default-features` | clean |
| `cargo check -p emissary-cli --no-default-features --features i2pcontrol` | clean |
| `cargo check -p emissary-core` | clean |
| `cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings` | clean |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment` | 7 passed |
| `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m062_dependency_containment` | 19 passed |
| `git diff --check` | clean |

The M062 manifest was also extended to admit the M081 closure document
that M081 added but never recorded in the M062 allowlist
(`is_authorized_planning_path`). M082 records the correction as part
of its verification gate; the underlying M081 closure remains
accepted at `cd41d28` and the workstream does not reopen the M081
audit findings.

The post-fix `assert_invariants` debug-assert in
`ServerAdmissionState` continues to fire after every `try_acquire`
commit and every `AdmissionLease` drop in test builds, providing an
in-process regression gate for the bounded state machine M082
consumes unchanged.

## 8. Compatibility, migration, security review

- the public HTTP filter surface (`HttpServerPolicy`,
  `AccessOption`, `SanitizedRequest`, `SanitizedResponse`, `Header`,
  `read_and_sanitize_request`, `read_and_filter_response`,
  `copy_body`, `copy_response_body`) gains one new error type
  (`RequestSanitizeError`) and one new categorical error variant
  (`ExpectUnsupported`); callers that previously matched on
  `io::ErrorKind` for sanitization errors are updated to match the
  categorical variant;
- the public `httpserver` backend surface
  (`HttpServerTunnelBackend`, `HttpServerConfig`, `PostLimiter`,
  `PostLimiterState`) is unchanged externally; the new
  `PostPeerKey` is a 32-byte newtype instead of an 8-byte newtype and
  keys are derived from `TrustedPeerIdentity::canonical_id()` rather
  than a `DefaultHasher` digest;
- the M080 canonical cryptographic Destination identity
  (`TrustedPeerIdentity`) is consumed unchanged by `read_and_sanitize_request`,
  `PostLimiter::allow`, and `HttpServerRuntimeSupervisor`;
- the inbound `httpbidirserver` half consumes the same corrected path
  through `make_accepted_handler`; no production type changes;
- no `emissary-core/**` production change was required; the
  repository's existing `Destination::parse` and `Destination::id`
  helpers are reused through `TrustedPeerIdentity::from_destination_text`;
- M061 source-containment and M062/M063 dependency-containment suites
  remain green; the M062 manifest now explicitly admits the M081
  closure document and the planned M082 closure document;
- private destination material is never logged or written to error
  messages — `TrustedPeerIdentity::fmt::Debug` redacts both the
  textual destination and the canonical ID, and
  `RequestSanitizeError::status_line` returns only the fixed status
  line;
- the Tokio `test-util` feature remains scoped to `[dev-dependencies]`
  from M080; M062 transitive-feature containment remains green;
- no full `100 Continue` relay implementation is introduced;
  informational-response semantics remain explicitly out of scope
  per the M082 plan section 14 stop conditions.

## 9. Documentation updates

- `docs/i2pcontrol/proposal-170-support.md` — status line updated to
  record M082 closure; HTTP server runtime boundary expanded to
  document the structural `TrustedPeerIdentity` consumption, the
  `Expect` 417 rejection, and the canonical POST peer key;
  M082 row added to the closed-handoff table; M077 row updated to
  reflect the closed M082 dependency.
- `docs/i2pcontrol/tunnel-backends.md` — status line updated to
  reflect M082 closure as part of the closed M080-M082 security
  sequence.
- `docs/i2pcontrol/tunnel-manager.md` — status line updated to
  reflect M082 closure.

## 10. Acceptance criteria evaluation

Section 13 of the plan is satisfied:

1. no valid currently supported I2P Destination is rejected by an
   obsolete magic ceiling — `accepts_largest_supported_key_certificate_destination`
   proves the 391-byte / 524-char maximum is accepted;
2. trusted peer identity is structurally validated and bounded using
   repository-supported `Destination::parse` semantics — every HTTP
   handler call now consumes `&TrustedPeerIdentity`, the filter no
   longer performs its own textual validation, and
   `rejects_malformed_destination_text_before_request_construction`
   proves the upstream validator rejects non-Destination text;
3. injected `X-I2P-DestB64` and `X-I2P-DestB32` are derived from the
   authenticated canonical identity — `accepts_canonical_trusted_peer_from_stream_helper`
   and `normalizes_absolute_target_and_removes_spoofed_identity` prove
   it;
4. every request containing `Expect` fails before local target
   allocation with bounded fixed semantics — the four
   `rejects_expect*` tests + `expect_request_is_rejected_with_417_before_local_allocation`
   prove both the categorical rejection and the end-to-end
   no-local-connection behaviour;
5. no `100-continue` client/backend wait cycle can pin a handler to
   body timeout — `expect_request_is_rejected_with_417_before_local_allocation`
   exercises the path against a 150 ms local listener accept timeout
   and observes no connection;
6. POST limiter keys use canonical cryptographic Destination
   identity rather than 64-bit `DefaultHasher` —
   `PostPeerKey([u8; 32])` derived from
   `TrustedPeerIdentity::canonical_id()`, covered by
   `post_limiter_keys_distinct_peers_independently`,
   `post_limiter_denies_churn_without_evicting_active_entries`,
   and `post_limiter_counts_only_write_methods`;
7. POST limiter/map/expiry state remains hard bounded and churn-safe
   — `MAX_THROTTLE_ENTRIES = 1024` plus the unchanged
   `VecDeque<PostExpiry>` reaper plus the
   `post_limiter_denies_churn_without_evicting_active_entries` test;
8. M076 fingerprint/proxy/framing protections remain green —
   `response_filter_removes_fingerprints_and_hop_by_hop_headers`,
   `removes_every_adopted_response_fingerprint_case_insensitively`,
   `removes_proxy_identity_and_adopted_request_privacy_headers`,
   `preserves_valid_chunked_response_framing`, and the pre-existing
   smuggling/malformed-header suite all pass unchanged;
9. `httpbidirserver` consumes the same corrected path — `make_accepted_handler`
   is the only seam that
   `HttpBidirServerTunnelBackend::run_composite` invokes, and the
   inbound handler tests cover the shared path;
10. production changes remain inside I2PControl and containment tests
    pass — `emissary-cli/src/i2pcontrol/backends/filters/http.rs`,
    `emissary-cli/src/i2pcontrol/backends/http_server.rs`, and
    `emissary-cli/src/i2pcontrol/backends/runtime/peer_identity_impl.rs`
    are the only production paths touched, plus the M062 manifest
    extension in `emissary-cli/tests/m062_dependency_containment.rs`;
    M061 source-containment and M062/M063 dependency-containment
    suites are green;
11. no high/medium HTTP correctness/anonymity/resource finding
    remains in M082 scope — see "Unresolved findings" below.

## 11. Unresolved findings

None at M082 scope. The M076 closure's disposition is updated to
`closed` because M082 closes the structural-Destination bound,
`Expect` rejection, and POST peer-key defects that the M076 closure
originally flagged for a corrective pass.

## 12. Unblocked downstream plans

M082 closes the M076 corrective sequence and unblocks the remaining
tunnel-security workstream:

- **M077 — IRC server lifetime/exhaustion hardening**
  (`plans/implementation/i2pcontrol-proposal-170/077-irc-server-lifetime-and-exhaustion-hardening.md`).
  Registry sequencing in `plans/registry.md` advances M077 from
  `blocked` to `ready`. M077 consumes the M080 canonical
  cryptographic peer identity unchanged for its IRC post-registration
  idle expiry.
- **M078 — Streamr local-boundary hardening**
  (`plans/implementation/i2pcontrol-proposal-170/078-streamr-local-boundary-hardening.md`).
  M078 remains behind M077.
- **M079 — integrated tunnel-security reclosure**
  (`plans/implementation/i2pcontrol-proposal-170/079-tunnel-security-reclosure.md`).
  M079, not an implementation-agent assertion, remains the final
  independent tunnel-security reclosure authority. M079 may now
  audit the corrected head without the M082 outstanding defects.

## 13. Internal-only boundary

External I2P/I2P+ reference material was inspected read-only (the
pinned Yosemite `SessionOptions` contract and the I2P Base64/identity
helpers in `emissary_core::crypto`) while confirming the canonical
trusted peer identity boundary. No upstream repository, maintainer
channel, issue, pull request, merge request, or submission was
opened, drafted, requested, or prepared. No contribution artifact
was produced under M082. All repository writes remain internal to
`eggstack/emissary`.