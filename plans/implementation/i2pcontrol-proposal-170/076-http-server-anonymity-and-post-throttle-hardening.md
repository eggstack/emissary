# M076 — HTTP Server Anonymity and POST-Throttle Hardening

Status: blocked — hard dependencies M073 and M074

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-tunnel-security-hardening-roadmap.md`.

Inherited implementation:

- M067 HTTP server;
- M070 HTTP bidirectional composition.

Planning production baseline: `04e0c2e5a35888e6fec8fd0b6aef80437174e3b0`.

Read-only reference evidence:

- Java I2P `I2PTunnelHTTPServer` at `i2p/i2p.i2p@498488b0`;
- Java `ConnThrottler` peer/aggregate behavior;
- I2P+ `I2PTunnelHTTPServer` at `I2PPlus/i2pplus@5cd400e7`.

## 1. Objective

Close the remaining HTTP anonymity and abuse-control gaps without replacing the accepted M067 parser architecture.

Required corrections:

1. expand request-side proxy/privacy header removal where the local backend could otherwise trust attacker-supplied network identity;
2. bring response fingerprint filtering to at least Java I2P parity and adopt the stronger I2P+ anonymity-oriented denylist where framing/application semantics remain safe;
3. replace the current POST limiter's eviction-bypassable state management with bounded fail-closed peer accounting;
4. ensure `httpbidirserver` inherits the same filter/limiter behavior automatically through composition.

## 2. Confirmed defects

### Response fingerprinting

Current Emissary strips `Server`, `X-Powered-By`, `X-Runtime`, `Proxy`, and `Proxy-Connection` but not `Date`.

Java I2P strips `Date` in addition to those headers. I2P+ strips a much broader set including cache/provider/trace metadata with explicit anonymity implications. A backend-originated `Date` reveals the local application/server clock; provider/cache/trace headers may expose the hosting layer or enable cross-site correlation.

### Request proxy identity

Current filtering removes several forwarding headers but does not comprehensively cover common reverse-proxy identity names such as `X-Real-IP`. A loopback application may trust such a header precisely because the connection originated locally.

### POST limiter churn

The current `PostLimiter` bounds the peer map but, after the bound is exceeded, removes the oldest entry. A destination-rotating attacker can therefore force removal of active accounting state and re-enter with reset counters.

## 3. Hard invariants

- reuse `backends/filters/http.rs`; no second HTTP parser/filter;
- `httpbidirserver` must consume the same server filter and limiter objects;
- request framing behavior accepted by M067 remains fail-closed;
- remote headers cannot impersonate local/reverse-proxy network identity;
- trusted `X-I2P-*` values are rebuilt only from authenticated peer identity;
- response filtering happens before any local response header bytes reach the I2P peer;
- header filtering does not inspect or rewrite arbitrary application body content;
- Content-Length/chunked framing remains valid after filtering;
- limiter state is bounded and monotonic-time based;
- no active limiter record is churn-evicted to admit a new untrusted peer;
- no lock across body copy/network I/O;
- no random response jitter;
- no local backend details in generated error responses.

## 4. Request-side anonymity/proxy denylist

Audit the current request removal set and add explicit case-insensitive handling for at least:

- `X-Real-IP`;
- `X-Client-IP`;
- `True-Client-IP`;
- `CF-Connecting-IP`;
- `Fastly-Client-IP`;
- `X-Cluster-Client-IP`;
- existing `Forwarded`, `Via`, `X-Forwarded-*`, `Proxy`, and `Proxy-Connection` names.

`BlockAccessInProxies` may inspect relevant markers before stripping, but stripped input must never survive into the local request.

Review I2P+ handling of `Priority` and `Sec-GPC`. These are privacy/fingerprinting rather than trusted-network identity. Adopt their removal only if it is consistent with the existing Emissary HTTP anonymity policy; record the decision in closure tests/docs.

Do not introduce a general arbitrary request-header blocklist wire option.

## 5. Response fingerprint policy

### 5.1 Mandatory Java parity

Always strip, case-insensitively:

- `Date`;
- `Server`;
- `X-Powered-By`;
- `X-Runtime`;
- `Proxy`;
- `Proxy-Connection`.

`Date` is a blocker for M076 closure.

### 5.2 I2P+ anonymity set

Independently adopt the non-framing I2P+ response denylist unless a header is demonstrated necessary for correctness:

- `Age`;
- `Alt-Svc`;
- `Expires`;
- `Pragma`;
- `Referer` when incorrectly emitted as a response header;
- `Strict-Transport-Security` for this non-TLS tunnel boundary;
- `Via`;
- `X-Cache`;
- `X-Cache-Hits`;
- `X-Cloud-Trace-Context`;
- `X-ContextID`;
- `X-Goog-Generation`;
- `X-Goog-Hash`;
- `X-Guploader-UploadID`;
- `X-Hacker`;
- `X-Nananana`;
- `X-Pantheon-Styx-Hostname`;
- `X-Served-By`;
- `X-Styx-Req-ID`.

This list is a behavioral security policy, not copied code. Keep it as a small explicit lowercase set with table-driven tests.

Do not strip application-semantic headers such as `Set-Cookie`, `Content-Type`, `Content-Disposition`, ETag/cache-control, or Location merely because they may contain application data. Broader application-content privacy policy is out of scope.

### 5.3 Framing

Continue to remove hop-by-hop/nominated headers through the existing logic. The expanded fingerprint denylist must not remove the selected Content-Length/Transfer-Encoding framing headers in a way that changes body interpretation.

## 6. Trusted identity output bounds

The HTTP filter currently injects trusted B64/B32 identity headers. Revisit the accepted remote-destination textual bound and ensure generated trusted header bytes are themselves bounded to a reference-valid I2P destination size plus fixed encoding overhead.

Do not retain an arbitrary 64 KiB identity solely because the parser accepts that string. Derive/document the safe maximum from the I2P/Yosemite destination representation or fail closed before building the local header block. The bound must not reject ordinary valid reference destinations.

## 7. POST limiter hardening

Keep the existing `PostLimit`/`PostLimitTime` per-peer semantics unless M073 establishes a different pinned mapping.

Required state behavior:

- peer key should be fixed-size/digested for accounting;
- per-peer check is O(1) or amortized O(1);
- expiry is lazy/bucketed, not a full-map `retain()` on every request;
- map has an explicit bound;
- if the bound contains active/unexpired entries, a POST/PUT/PATCH from an unseen peer is denied rather than evicting an existing entry;
- expired entries may be reclaimed;
- restart clears ephemeral limiter state;
- rejected write requests never open/connect the local target;
- GET/HEAD/other non-write methods do not consume POST quota.

Do not add a new Proposal 170 `TotalPostLimit` field. If a safe aggregate POST fuse is desired, it must be an internal defense justified in the implementation/closure without changing the wire contract. The shared M074 aggregate connection-rate controls already limit identity-churn admission and are the required first layer.

## 8. Error/timing policy

Use fixed, bounded locally generated error heads/bodies with no target address, OS error, stack text, or backend timing metadata.

Do not add delays to normalize 403/408/429/5xx timing. The primary defense is pre-local validation plus M074 fairness/rate control. Artificial sleeps create an attacker-controlled resource cost and are prohibited by this plan.

## 9. Ordered work packages

### WP1 — Header policy tables

Expand request identity/proxy removal and response fingerprint denylist with table-driven unit tests.

### WP2 — Trusted identity bounds

Bound injected peer identity headers against reference-valid destination representation.

### WP3 — POST limiter replacement

Replace scan/eviction behavior with bounded fail-closed state and monotonic expiry.

### WP4 — HTTP server integration

Wire the hardened limiter/filter through `httpserver`; verify `httpbidirserver` inherits the same objects through existing composition.

### WP5 — Adversarial local capture tests

Capture exact bytes at the loopback backend and at the fake I2P peer boundary.

### WP6 — Documentation/closure

Document stripped header classes, limiter full-table behavior, and deliberate non-goals.

## 10. Required tests

At minimum:

- backend response `Date` never reaches I2P;
- every mandatory Java-parity response header is stripped with mixed casing;
- each adopted I2P+ fingerprint header is stripped;
- `Content-Length`, valid chunked framing, cookies, content type, and application headers remain correct;
- request `X-Real-IP`, `CF-Connecting-IP`, `True-Client-IP`, and existing proxy identity headers do not reach backend;
- attacker-supplied `X-I2P-*` still cannot survive trusted reinjection;
- trusted identity header generation rejects an over-bound remote identity before local connect;
- POST quota blocks the configured peer at the threshold;
- unrelated peer retains independent quota;
- full limiter table denies new peer write request without evicting active state;
- expired entry is reclaimable with paused time;
- limiter check does not perform O(n) full-table cleanup per request (test through observable helper/state behavior, not a fragile microbenchmark);
- rejected POST does not connect backend;
- `httpbidirserver` response/request policy is identical to `httpserver` inbound path;
- generated error responses contain no backend target/OS detail.

## 11. Verification

```text
cargo test -p emissary-cli --no-default-features --features i2pcontrol
cargo check -p emissary-cli --no-default-features
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo check -p emissary-core
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
git diff --check
```

Run focused HTTP filter/backend tests separately for closure evidence.

## 12. Acceptance criteria

M076 may close only when:

1. `Date` and all Java-parity fingerprint headers are removed;
2. the adopted I2P+ anonymity denylist is tested and documented;
3. common reverse-proxy identity headers cannot be spoofed into the loopback application;
4. trusted identity injection is itself bounded;
5. POST limiter state cannot be reset through active-entry eviction/churn;
6. no rejected request reaches the local target;
7. `httpbidirserver` inherits the same hardened server path without code duplication;
8. HTTP framing/body semantics remain correct;
9. all production changes remain under `i2pcontrol`;
10. no high/medium HTTP anonymity/resource finding remains.

## 13. Stop conditions

Stop if the proposed denylist would require application-body rewriting, if preserving HTTP framing would require a second parser, or if a requested behavior would add new public Proposal 170 fields.

Closure must attest external reference access was read-only and no upstream activity occurred.
