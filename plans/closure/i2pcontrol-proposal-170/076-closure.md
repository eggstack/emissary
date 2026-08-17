# M076 Closure — HTTP Server Anonymity and POST-Throttle Hardening

Status: closed

Source implementation plan:

- plans/implementation/i2pcontrol-proposal-170/076-http-server-anonymity-and-post-throttle-hardening.md

Source roadmap:

- plans/subsystems/i2pcontrol-proposal-170-tunnel-security-hardening-roadmap.md

Implementation commit:

- 3cf082e — feat(i2pcontrol): harden HTTP anonymity and POST throttling
- f454e35 — fix(i2pcontrol): strip all forwarded identity headers

## 1. Disposition

M076 is closed. The shared accepted-stream HTTP server filter now removes
attacker-controlled proxy/privacy identity and backend/provider/cache/trace
fingerprints, preserves validated response framing, bounds trusted peer
identity injection, and uses bounded fail-closed POST/PUT/PATCH accounting.
The inbound httpbidirserver path continues to compose the same
make_accepted_handler, filter, and PostLimiter objects; no second HTTP parser
or filter was introduced.

## 2. Requirement-to-evidence matrix

| Requirement | Evidence | Result |
|---|---|---|
| Java parity response filtering | filters/http.rs::RESPONSE_FINGERPRINTS includes Date, Server, X-Powered-By, X-Runtime, Proxy, and Proxy-Connection; mixed-case table test | pass |
| I2P+ non-framing anonymity denylist | Explicit lowercase table includes age/cache/provider/trace/HSTS and related fields; table-driven mixed-case test covers every entry | pass |
| Common request proxy identity cannot reach backend | Case-insensitive proxy-identity helper strips Forwarded, Via, the complete X-Forwarded-* namespace, X-Real-IP, X-Client-IP, True-Client-IP, Cloudflare/Fastly/cluster names, and proxy names; capture tests cover representative X-Forwarded-Proto/Port/Prefix variants | pass |
| Deliberate privacy header decision | Priority and Sec-GPC are adopted in REQUEST_PRIVACY; request capture test and docs record their stripping as anonymity/fingerprinting policy | pass |
| Spoofed I2P identity is replaced | Existing end-to-end local capture test verifies attacker X-I2P-* values do not survive and trusted Host/identity headers are rebuilt | pass |
| Trusted identity output is bounded | MAX_TRUSTED_DESTINATION_TEXT = 524, derived from the 391-byte reference destination key-certificate form and padded I2P Base64; over-bound identity test fails before request construction/local connect | pass |
| Response framing remains valid | Content-Length is normalized and retained; chunked Transfer-Encoding is normalized and retained; content type/cookies remain; content and chunked capture tests pass | pass |
| No application-body rewriting | Filtering is restricted to bounded request/response heads; body copy remains byte-for-byte and no body test data is rewritten | pass |
| POST limiter is fixed-size and monotonic | PostPeerKey([u8; 8]), Tokio monotonic Instant, FIFO expiry queue, and bounded HashMap; no raw peer strings or retain()/oldest eviction remain | pass |
| Limiter full-table behavior is fail-closed | Paused-time test fills all 1024 entries, denies an unseen peer, and confirms an existing peer remains throttled | pass |
| Expiry is reclaimable without full-map cleanup | Paused-time test advances the window and confirms the expiry queue reclaims entries before admitting a new peer; active-table state remains unchanged on denial | pass |
| Write rejection precedes local allocation | handle_http_stream checks the limiter before TcpStream::connect; rejected POST capture test observes 429 and no local accept | pass |
| Non-write methods do not consume POST quota | Method gate remains limited to POST/PUT/PATCH; limiter unit coverage retains independent per-peer semantics | pass |
| httpbidirserver inherits the hardened path | http_bidir.rs imports make_accepted_handler and PostLimiter from http_server.rs; existing shared-policy composition test remains green | pass |
| Error responses remain bounded and detail-free | Existing fixed status heads remain unchanged and contain no target/OS/backend detail; rejection capture tests pass | pass |
| Scope and containment | Production changes are in emissary-cli/src/i2pcontrol/**; M062 allowlist now authorizes the M075/M076 closure records; no core/router/SAM/public Proposal 170 change | pass |

## 3. Verification

Passed:

    cargo test -p emissary-cli --no-default-features --features i2pcontrol
    cargo test -p emissary-cli --no-default-features --features i2pcontrol http
    cargo check -p emissary-cli --no-default-features
    cargo check -p emissary-cli --no-default-features --features i2pcontrol
    cargo check -p emissary-core
    cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
    cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment
    cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m062_dependency_containment
    rustfmt +nightly --check --edition 2021 --config-path rustfmt.toml emissary-cli/src/i2pcontrol/backends/filters/http.rs emissary-cli/src/i2pcontrol/backends/http_server.rs emissary-cli/tests/m062_dependency_containment.rs
    git diff --check

The full feature-enabled package suite passed 1,560 tests across 24 suites.
The focused HTTP filter/server run passed 95 tests. The containment suites
passed 7 and 19 tests respectively. The required stable
cargo fmt --all -- --check was also run; it remains red on inherited
repository-wide formatting drift because this repository's rustfmt.toml uses
nightly-only options and unrelated files are not formatted by stable rustfmt.
The touched Rust files pass the repository's nightly rustfmt check.

## 4. Invariant, compatibility, and security review

- Request framing remains fail-closed: conflicting Content-Length and
  transfer-encoding are rejected, and request transfer encoding is not
  accepted.
- Response framing is selected before filtering; only validated
  Content-Length or normalized chunked framing is emitted, with close-delimited
  bodies preserved where no framing header exists.
- Set-Cookie, Content-Type, Content-Disposition, ETag/cache-control, and
  Location remain application semantics and are not in the denylist.
- Remote headers never select a local target. The fixed loopback target and
  pre-connect limiter ordering remain unchanged.
- No lock is held across request body copy, local connection, response parsing,
  or network I/O; the limiter lock covers only bounded accounting operations.
- Limiter state is ephemeral and therefore clears on process/runtime restart.
- No sleeps, jitter, aggregate Proposal 170 fields, application-body rewrite,
  public wire field, router algorithm, or deferred tunnel data plane was added.
- No high/medium finding remains in the M076 HTTP anonymity/resource scope.

## 5. Documentation and future-plan disposition

Updated:

- docs/i2pcontrol/proposal-170-support.md
- docs/i2pcontrol/tunnel-manager.md
- docs/i2pcontrol/tunnel-backends.md
- the security-hardening roadmap and active registry
- the M076 implementation plan
- the M062 dependency-containment allowlist for authorized closure metadata

M077 is now unblocked by M076, marked ready in its handoff, and registered as
the next dependency-ready plan. M078 remains blocked by the sequencing rule
until M077 closes. M079 remains blocked until M074-M078 close. Independent
source milestone M051 remains blocked by its accepted absence of substantive
news/ban owners and is unaffected by M076. A post-closure registry audit after
the X-Forwarded-* namespace correction found no additional dependency-ready
successor or status transition.

## 6. Internal-only external interaction attestation

Java I2P/I2P+ reference material and the pinned Proposal 170 source were used
as read-only behavioral evidence. No upstream repository, maintainer channel,
issue, pull request, review, merge, adoption, or submission was mutated or
requested. No upstream contribution artifact was prepared under M076. The
requested push is limited to the internal eggstack/emissary repository remote.
