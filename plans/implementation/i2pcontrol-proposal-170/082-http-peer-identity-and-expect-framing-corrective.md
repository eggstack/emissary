# M082 — HTTP Peer Identity and Expect-Framing Corrective

Status: ready — M080 and M081 closed; current registered handoff

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-tunnel-security-hardening-roadmap.md`.

Original implementation/closure:

- M076: `plans/implementation/i2pcontrol-proposal-170/076-http-server-anonymity-and-post-throttle-hardening.md`;
- M076 closure: `plans/closure/i2pcontrol-proposal-170/076-closure.md`.

Planning production baseline: `1618de172e7a78a193fc1bb117af269f31174030`.

## 1. Objective

Correct two HTTP server correctness/security regressions found after M076 closure while preserving the successful anonymity/fingerprint hardening:

1. replace the hard-coded 524-character trusted Destination bound with structural validation that accepts all currently valid I2P Destination forms and remains explicitly bounded;
2. fail safely on `Expect: 100-continue` rather than allowing a client/backend deadlock that occupies an accepted-server slot until body timeout.

Also replace the HTTP POST limiter's 64-bit `DefaultHasher` peer key with the same canonical cryptographic I2P Destination identity established by M080.

The M076 response fingerprint denylist, request proxy-identity stripping, framing validation, loopback target confinement, and fail-closed limiter behavior are retained.

## 2. Independent findings that reopen M076

### 2.1 MEDIUM — 524-byte textual Destination ceiling rejects valid identities

M076 added `MAX_TRUSTED_DESTINATION_TEXT = 524` based on a 391-byte Destination assumption. Current I2P supports signature public keys larger than the legacy 128-byte signing-key area, including P521 and RSA variants. Their key certificates carry excess key material and valid serialized Destinations can exceed the assumed 391-byte representation.

The HTTP filter therefore may reject a structurally valid authenticated I2P peer solely because its Destination uses a larger supported signature type.

The repository already contains structural full-Destination validation under I2PControl using:

```text
base64_decode -> emissary_core::primitives::Destination::parse
```

and derives the canonical Destination ID from the parsed object. M082 should reuse the same semantic boundary instead of maintaining a second magic length assumption.

### 2.2 MEDIUM — `Expect: 100-continue` can create a bounded but attacker-useful deadlock

Current HTTP flow:

```text
read request head
-> forward sanitized request head to local backend
-> synchronously copy complete remote request body
-> only then read local response
```

The request sanitizer does not reject or remove `Expect`.

For `Expect: 100-continue`, a compliant remote client may wait for an interim `100 Continue` response before sending its body. The local backend may emit that interim response immediately, but Emissary is not reading the local response yet because it is blocked waiting for the remote body. The connection can therefore remain pinned until the body timeout.

This is not an HTTP request-smuggling defect, but it is an avoidable resource-exhaustion/timing state on a server type whose anonymity hardening explicitly aims to remove attacker-controlled occupancy signals.

### 2.3 LOW-MEDIUM — POST limiter still uses `DefaultHasher`/64-bit identity

`PostLimiter` keys authenticated peer text with an eight-byte `DefaultHasher` output. As with M080 admission accounting, use the canonical cryptographic Destination ID instead. There is no reason for HTTP write-throttle identity to have weaker collision properties than the authenticated I2P identity already available.

## 3. Why prior verification missed these findings

M076 tested an arbitrary over-524 string and treated rejection as evidence of a safe bound; it did not include valid Destination fixtures for the largest supported I2P signature/key-certificate forms.

M076 request-framing tests focused on `Content-Length`/`Transfer-Encoding`, upgrades, header limits, and proxy identity. They did not test HTTP/1.1 expectation semantics or the request-body/local-response ordering for `100 Continue`.

POST limiter tests verified map capacity/churn behavior, but not the strength/canonicality of the peer key.

M082 must add realistic valid-Destination fixtures and an end-to-end `Expect` deadlock regression.

## 4. Hard invariants

- production changes stay under `emissary-cli/src/i2pcontrol/**`;
- no new HTTP dependency/parser framework;
- no application body rewriting or content inspection;
- request framing remains fail-closed for conflicting/ambiguous length and transfer coding;
- local target connection remains loopback-only and administrator-selected;
- spoofed I2P/proxy identity headers remain stripped before local forwarding;
- trusted identity metadata is derived only from the accepted Yosemite peer;
- `Date`, server/framework/provider/cache/trace fingerprint stripping remains intact;
- `httpbidirserver` continues to reuse the exact same inbound HTTP handler/filter/limiter path;
- no lock is held across network I/O;
- no random response delay/jitter;
- errors remain fixed/bounded and do not disclose backend/OS/target detail;
- no HTTP/2, websocket/upgrade, CONNECT-server, TLS termination, or proxy feature expansion;
- M080 canonical peer identity helper/representation is reused rather than duplicated where practical.

## 5. Required Destination validation correction

### 5.1 Structural validity is authoritative

Replace the 524-character magic ceiling as the validity rule.

The trusted remote Destination must be:

- non-empty;
- valid I2P Base64 according to the repository's decoder;
- structurally parseable by the repository's `Destination` primitive;
- within an explicit defensive textual/decoded upper bound derived from the maximum supported Destination representation in the pinned repository, not one legacy key form.

Prefer centralizing validation at or immediately after `TrustedPeerIdentity::from_stream` if M080 establishes a canonical parsed/hash identity there. HTTP should consume a trusted already-validated identity rather than invent another parser boundary.

If retaining a textual bound in HTTP for header construction, derive/document it from the actual maximum parsed Destination supported by current Emissary/I2P key types plus Base64 expansion. Add tests for the maximum valid fixture and one-byte-over/structurally-invalid cases.

Do not restore the arbitrary 64 KiB HTTP metadata allowance.

### 5.2 Preserve exact injected metadata semantics

Continue injecting only bounded trusted values:

- `X-I2P-DestB64` from the validated canonical/full Destination text;
- `X-I2P-DestB32` from the canonical Destination ID/hash.

Do not inject the full peer into additional headers. Do not log the full Destination.

If the canonical representation normalizes Base64 text, document whether the header uses Yosemite's validated text or a re-encoded canonical form. Either is acceptable if it preserves identity and remains bounded; avoid unnecessary encode/decode churn.

## 6. Required `Expect` policy

This milestone does not implement a general informational-response state machine.

Before local target connection/forwarding, inspect `Expect` request headers case-insensitively.

Required policy:

- no `Expect` header -> existing behavior;
- exactly `100-continue` (case-insensitive token semantics) -> reject before local target allocation with a fixed bounded response, preferably `417 Expectation Failed`;
- any other/combined expectation -> reject before local target allocation with the same fixed bounded policy;
- multiple `Expect` headers are treated as unsupported rather than merged into ambiguous application behavior.

Do not silently strip `Expect` and then wait for a body from a client that may still be waiting for `100 Continue`; explicit rejection is safer and deterministic.

Do not add a `100 Continue` relay implementation in this corrective. Supporting it correctly would require reading and relaying interim backend responses concurrently with request-body acquisition and substantially widens the HTTP state machine.

The fixed error must use `Connection: close`, zero/bounded body, and no local target/backend detail.

## 7. POST limiter peer-key correction

Replace `PostPeerKey([u8; 8])`/`DefaultHasher` with the canonical fixed-size Destination hash from M080.

Requirements:

- no additional parse of attacker-controlled application headers;
- key comes from the accepted trusted peer identity only;
- map cardinality remains 1024 unless a separate measured reason requires changing it;
- fail-closed full-table semantics and non-eviction of active/unexpired state remain unchanged;
- expiry bookkeeping remains bounded for this limiter; if independent review discovers the same stale-expiry growth pattern as M080, correct it here rather than carrying an unbounded auxiliary queue into M079;
- no raw peer Destination is stored solely for throttling.

## 8. HTTP framing/reply revalidation

While touching the request parser, re-run and preserve existing protections:

- request-line/header total/line/count limits;
- CRLF-only headers and obs-fold rejection;
- header-name/value validation;
- duplicate Host rejection;
- conflicting Content-Length rejection;
- request Transfer-Encoding rejection;
- HTTP upgrade rejection;
- connection-nominated hop-by-hop stripping;
- proxy/request identity stripping;
- absolute-target normalization;
- Host replacement;
- response CL/TE ambiguity rejection;
- validated response framing re-emission;
- response fingerprint denylist.

No new permissive branch should be introduced to accommodate `Expect`.

## 9. Failure, cancellation, restart, and contention semantics

- unsupported expectation returns before local `TcpStream::connect`;
- no request body is read after sending the fixed expectation rejection beyond ordinary connection close semantics;
- the accepted-server admission lease is released when the handler returns;
- malformed/invalid Destination fails before request construction/local target;
- local backend failure remains 502 without target detail;
- request timeout behavior remains bounded;
- restart does not persist POST limiter state;
- `httpbidirserver` inherits every correction through `make_accepted_handler`/shared filter and does not instantiate a second policy;
- one malformed/unsupported request cannot fail the whole tunnel runtime.

## 10. Ordered work packages

### WP1 — Real Destination fixtures and shared identity consumption

Use structurally valid I2P Destination fixtures spanning ordinary and largest-current supported key/certificate forms. Reuse M080's canonical identity representation.

### WP2 — Remove magic validity ceiling

Replace 524-as-validity with structural parse plus a defensible maximum representation bound. Keep injected headers bounded.

### WP3 — Expect fail-closed handling

Reject any request containing `Expect` before local connect, with fixed 417/close semantics. Add end-to-end no-local-allocation proof.

### WP4 — POST peer key

Move the limiter to canonical Destination hash keys and revalidate expiry/map bounds.

### WP5 — Shared httpbidir coverage and docs

Prove the inbound bidirectional server uses the corrected path and update support/security documentation.

## 11. Required regression tests

At minimum:

- a valid common Ed25519/X25519 or repository-default Destination is accepted;
- a valid P521-style Destination larger than the previous 524-text threshold is accepted if supported by the repository parser;
- valid RSA-2048/3072/4096 Destination fixtures are accepted where the current repository reports those key types as supported/parseable;
- malformed Base64 and structurally invalid Destination text reject before local connect;
- the largest supported valid Destination fits the documented defensive bound;
- an over-bound/invalid representation rejects without allocating local target or limiter state;
- injected `X-I2P-DestB64` and B32 correspond to the authenticated parsed identity;
- attacker-supplied `X-I2P-*` remains removed/replaced;
- `Expect: 100-continue` returns fixed 417 and no local connection occurs;
- mixed-case `Expect: 100-Continue` is also rejected;
- unknown expectation token is rejected;
- multiple Expect headers are rejected;
- a client that sends headers with Expect and then waits never holds the handler until body timeout;
- ordinary POST without Expect still forwards body normally;
- POST limiter uses canonical distinct Destination IDs and preserves per-peer throttling;
- full POST table remains fail-closed without active-state eviction;
- POST expiry auxiliary state remains bounded under repeated same-peer writes;
- response `Date`, `Via`, cache/provider/trace headers remain stripped;
- response CL/chunked framing remains valid;
- `httpbidirserver` inbound path inherits Destination/Expect/limiter behavior;
- all rejection/error paths release the M080 admission lease.

## 12. Verification

Run at minimum:

```text
cargo test -p emissary-cli --no-default-features --features i2pcontrol
cargo test -p emissary-cli --no-default-features --features i2pcontrol http
cargo check -p emissary-cli --no-default-features
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo check -p emissary-core
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m062_dependency_containment
git diff --check
```

Run focused fake-SAM/local HTTP tests for valid large Destinations and Expect/no-target-allocation separately for closure evidence.

## 13. Acceptance criteria

M082 may close only when:

1. no valid currently supported I2P Destination is rejected solely by the obsolete 524-character assumption;
2. trusted peer identity is structurally validated and explicitly bounded using repository-supported Destination semantics;
3. HTTP injected identity metadata is derived from that authenticated canonical identity;
4. every request containing `Expect` fails before local target allocation with bounded fixed semantics;
5. no `100-continue` client/backend wait cycle can pin a handler to body timeout;
6. POST limiter keys use canonical cryptographic Destination identity rather than 64-bit `DefaultHasher`;
7. POST limiter/map/expiry state remains hard bounded and churn-safe;
8. M076 fingerprint/proxy/framing protections remain green;
9. `httpbidirserver` consumes the same corrected path;
10. production changes remain inside I2PControl and containment tests pass;
11. no high/medium HTTP correctness/anonymity/resource finding remains in M082 scope.

## 14. Stop conditions

Stop and create separate architecture planning if:

- accepting all valid current Destinations requires changing `emissary-core::Destination` parsing;
- correct `100 Continue` support is deemed required instead of explicit rejection;
- a new HTTP parser/framework or generalized concurrent request/response state machine becomes necessary;
- the correction requires body inspection/rewrite, TLS termination, HTTP/2, or other adjacent protocol expansion;
- M080 does not yield a stable canonical peer identity representation that HTTP can consume without duplicating unsafe logic.

External I2P/I2P+ reference access is read-only only. No upstream write/review/submission activity is authorized.
