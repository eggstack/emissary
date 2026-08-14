# M069 — SOCKS and SOCKS-IRC Tunnels

Status: blocked — dependency-ready but not the next registered handoff

Planning production baseline: `a1296b018ce98d26a019bd5064dff9f4b47e0ad6`

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-tunnel-runtime-completion-roadmap.md`.

Authority:

- `plans/000-long-term-specification.md`;
- `plans/003-planning-process.md`;
- `plans/adrs/ADR-0003-proposal-170-tunnel-runtime-completion-and-filter-boundary.md`.

Dependencies:

- hard: M065 client listener/session and option-capability primitives;
- hard/interface: M066 common IRC filter for `socksirc`.

## 1. Objective

Implement a bounded control-plane SOCKS frontend for direct I2P TCP CONNECT and compose that exact frontend with M066's accepted IRC filter to make `socksirc` operational.

The initial SOCKS scope is deliberately narrow: SOCKS4a and SOCKS5 TCP CONNECT. BIND, UDP ASSOCIATE, and protocol-specific resolve extensions remain unsupported unless the pinned Proposal 170 contract proves they are mandatory for the declared tunnel type.

## 2. Security model

SOCKS is a generic application tunnel and therefore cannot sanitize the application payload. Its security obligations are at negotiation/routing/exposure boundaries:

- do not perform accidental clearnet/local DNS;
- do not permit arbitrary direct LAN/localhost targets;
- do not become an unauthenticated non-loopback open proxy;
- bound negotiation and connection resources;
- route non-I2P only through explicitly configured I2P outproxy behavior;
- return protocol-correct failure rather than raw-close where practical;
- for `socksirc`, ensure payload cannot bypass the common IRC filter.

## 3. Current/reference evidence

The existing startup SOCKS service already demonstrates SOCKS5 domain CONNECT over Yosemite. The Java reference supports a wider SOCKS surface and explicitly treats literal-IP requests as potentially unsafe for anonymity. Java's SOCKS-IRC type is composition: SOCKS negotiation followed by the same IRC inbound/outbound filters used by IRC client tunnels.

M069 adopts those design principles but independently implements only the bounded functionality required here.

## 4. Classification

Primary class: capability / security.

Types promoted on closure:

- `socks`;
- `socksirc`.

## 5. Hard invariants

- new production code under `emissary-cli/src/i2pcontrol/**`;
- no `emissary-core/**` change;
- SOCKS4a/SOCKS5 negotiation bounded in time/bytes;
- direct I2P routing uses domain/I2P destination forms, not OS DNS;
- literal IP direct targets fail closed except any tightly scoped protocol-internal bind reply values;
- clearnet requires explicit I2P SOCKS/outproxy configuration;
- localhost/private/link-local targets fail closed in direct mode;
- non-loopback listener follows explicit authentication/exposure policy;
- credentials redacted;
- BIND/UDP ASSOCIATE unsupported and return correct failure without allocating I2P resources;
- `socksirc` uses M066 common IRC filter code path, not duplicated tables or raw relay;
- DCC remains blocked in `socksirc` through the common filter;
- unsupported relevant options reject before listener/session allocation;
- no startup SOCKS listener/task adoption.

## 6. Explicit non-goals

Do not:

- implement SOCKS5 BIND;
- implement UDP ASSOCIATE;
- implement torsocks/Tor RESOLVE extensions unless explicitly required by pinned Proposal 170;
- implement arbitrary DNS proxying;
- implement general clearnet outproxy selection beyond declared/configured I2P outproxy behavior;
- add DCC support;
- fork the IRC filter;
- refactor `emissary-cli/src/proxy/socks/**` into shared lifecycle authority;
- add a third-party SOCKS dependency solely for convenience if the bounded negotiation can be implemented safely with existing dependencies/std/Tokio.

## 7. Proposed implementation surface

Expected modules:

```text
emissary-cli/src/i2pcontrol/backends/socks.rs
emissary-cli/src/i2pcontrol/backends/socks_irc.rs
emissary-cli/src/i2pcontrol/backends/registry.rs
```

A single negotiation/target result type should feed both normal SOCKS and SOCKS-IRC connection handlers.

## 8. SOCKS4a requirements

Support TCP CONNECT only.

Required behavior:

- parse VN/CD/DSTPORT/DSTIP/USERID/domain with explicit maximum lengths and initial timeout;
- require SOCKS4a domain form for I2P targets rather than resolving arbitrary IPv4 destination;
- reject CONNECT to port zero;
- reject malformed/no-NUL-termination within bounds;
- ignore USERID for identity unless Proposal 170 explicitly maps proxy auth to SOCKS4 semantics; do not log it as trusted user identity;
- send protocol-correct success only after I2P connection established;
- send bounded failure otherwise.

If plain SOCKS4 literal IPv4 is requested, fail closed rather than local/network connect.

## 9. SOCKS5 requirements

### 9.1 Method negotiation

Implement:

- `NO AUTHENTICATION REQUIRED` only when listener policy permits;
- username/password when configured/safety policy requires;
- `NO ACCEPTABLE METHODS` for unsupported method sets.

Bound method count and negotiation timeout.

Credential comparison should use an appropriate constant-time path already available to `i2pcontrol` when comparing secrets.

### 9.2 Request parsing

Support command:

- CONNECT.

Reject with correct reply:

- BIND;
- UDP ASSOCIATE;
- unknown commands.

Address types:

- DOMAINNAME supported for `.i2p`/B32 and configured outproxy destinations;
- IPv4/IPv6 literal requests fail closed for direct I2P routing rather than being locally resolved/connected;
- zero/invalid port rejects.

Bound domain length per protocol and total negotiation bytes.

### 9.3 Connection reply

Do not claim success before remote I2P/outproxy connection succeeds.

Use a neutral bind address/port in local success reply; do not expose local routable interface information unnecessarily.

## 10. Target routing

Direct I2P target:

- `.i2p`/`.b32.i2p`/safe destination form resolved through approved I2P path;
- Yosemite stream opened to requested destination port;
- no OS DNS.

Clearnet target:

- only if explicitly configured Proposal 170 outproxy behavior exists;
- outproxy itself must be an I2P destination/name or otherwise an explicitly reviewed safe application endpoint;
- target hostname is conveyed to outproxy according to its SOCKS semantics;
- no direct local TCP connection to clearnet target.

Forbidden target:

- localhost;
- loopback literals;
- RFC1918/private/link-local/multicast/unspecified literal ranges;
- router console/internal application addresses in direct mode.

## 11. Listener exposure/authentication

Audit Proposal 170 SOCKS proxy auth/listen fields.

Required policy:

- safe loopback default;
- configured username/password support if specified by Proposal 170;
- non-loopback exposure must not silently be unauthenticated under project security policy;
- auth failure response protocol-correct and secret-free;
- bounded concurrent negotiation/connection count;
- no long sleep while holding scarce global runtime capacity merely to delay password guessing.

## 12. `socks` payload relay

After successful negotiation/remote connect, payload is raw bidirectional relay because SOCKS intentionally transports arbitrary application data.

Document clearly that SOCKS itself cannot prevent application-layer deanonymization by unsafe protocols. That is why `socksirc` exists separately.

Cancellation closes both sides and connection accounting.

## 13. `socksirc` composition

Required path:

```text
local IRC client
    -> exact M069 SOCKS negotiation
    -> Yosemite stream
    -> M066 common IRC outbound/inbound filter
    -> remote IRC service
```

Requirements:

- no alternate raw copy path after negotiation;
- same per-connection IRC filter state semantics as `ircclient`;
- same CTCP/DCC policy;
- same log redaction;
- SOCKS target may select IRC destination dynamically, but application payload remains filtered;
- target port validated;
- filter construction failure closes connection before raw relay.

A static/code-level guard/test should make it difficult for future refactor to replace the filtered path with generic `copy_bidirectional` in `socksirc`.

## 14. Option-capability matrix

Explicitly disposition:

- listen interface/port;
- proxy username/password;
- outproxy list/type/credentials;
- I2CP session/tunnel options supported by Yosemite;
- SOCKS-specific custom options;
- IRC-specific options inherited/relevant to `socksirc`;
- unsupported BIND/UDP/DNS-resolve features.

Security-sensitive recognized options reject before allocation if unimplemented.

## 15. Ordered work packages

### WP1 — negotiation parser/tests

Implement standalone bounded SOCKS4a and SOCKS5 negotiation state machines with table-driven byte fixtures.

### WP2 — normal `socks` backend

Wire M065 listener/session runtime and safe target routing; add auth/exposure and e2e fake-SAM tests.

### WP3 — `socksirc` composition

Wire exact negotiation result into M066 filter path. Add regression proving raw DCC/CTCP payload cannot bypass filter.

### WP4 — adversarial/resource tests

Exercise malformed negotiation, unsupported commands/types, literal-IP routing, auth, slow negotiation, connection caps, cancellation, and outproxy failures.

### WP5 — registry/docs/closure

Promote both types. Document raw SOCKS application-risk boundary and SOCKS-IRC filtering guarantee.

## 16. Required tests

At minimum:

- valid SOCKS4a `.i2p` CONNECT;
- plain SOCKS4 IPv4 rejected;
- overlong USERID/domain bounded;
- valid SOCKS5 no-auth loopback CONNECT;
- username/password success/failure;
- method list with no supported method rejected;
- BIND rejected before remote allocation;
- UDP ASSOCIATE rejected before remote allocation;
- IPv4/IPv6 literal direct target rejected;
- domain `.i2p` connects without local DNS;
- clearnet without outproxy rejected;
- local/private target rejected;
- success reply only after remote connect;
- negotiation timeout/connection cap;
- raw `socks` payload relay works;
- `socksirc` normal IRC works;
- `socksirc` DCC and unsupported CTCP blocked exactly as M066;
- `socksirc` code/test path never switches to raw relay;
- cancellation and restart cleanup.

## 17. Failure, cancellation, restart, contention semantics

Negotiation failure closes only the connection with protocol reply where safe.

Remote connect/outproxy failure produces SOCKS connection failure and no success reply.

Listener/session fatal failure marks named backend failed.

Stop cancels pending negotiations and active relays for exact generation.

Restart creates fresh client session/listener/auth state.

Connection cap accounting must recover on every parser/auth/connect/filter exit path.

## 18. Compatibility and migration

No wire/persistence schema change.

Persisted SOCKS/SOCKS-IRC definitions become startable only within implemented option set.

Existing startup SOCKS proxy remains unchanged.

## 19. Verification commands

Minimum:

```text
cargo test -p emissary-cli --no-default-features --features i2pcontrol socks
cargo test -p emissary-cli --no-default-features --features i2pcontrol socks_irc
cargo test -p emissary-cli --no-default-features --features i2pcontrol irc
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m062_dependency_containment
cargo check -p emissary-cli --no-default-features
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo check -p emissary-core --no-default-features
git diff --check
```

No external SOCKS server/network required.

## 20. Acceptance criteria

M069 may close only when:

1. M065 and M066 are closed;
2. SOCKS4a CONNECT works for safe I2P domain targets;
3. plain SOCKS4 literal-IP direct routing fails closed;
4. SOCKS5 method negotiation is bounded and correct;
5. configured username/password authentication works and secrets are redacted;
6. SOCKS5 CONNECT domain target works;
7. BIND returns unsupported without remote resource allocation;
8. UDP ASSOCIATE returns unsupported without remote resource allocation;
9. literal IPv4/IPv6 direct targets do not trigger local connect/DNS;
10. `.i2p` routing uses I2P mechanisms only;
11. clearnet requires explicit I2P outproxy;
12. localhost/private/link-local/unsafe targets fail closed;
13. success reply is not sent before remote connection succeeds;
14. negotiation and active connection counts/time are bounded;
15. normal `socks` payload relay works after safe negotiation;
16. `socksirc` reuses the M066 common IRC filter implementation;
17. no raw/unfiltered `socksirc` relay path exists;
18. DCC/unsupported CTCP remain blocked in SOCKS-IRC;
19. relevant unimplemented options reject before allocation;
20. lifecycle stop/restart/cancellation is exact and generation-safe;
21. only `socks`/`socksirc` replace their unsupported backends;
22. no `emissary-core/**` production change;
23. no unjustified non-I2PControl production change;
24. feature-disabled/default and containment checks pass;
25. docs accurately state SOCKS command/address support, auth/outproxy policy, and raw-protocol anonymity limitation;
26. no CI/release/fuzz/coverage/platform expansion;
27. no upstream/third-party write/review/submission/contribution preparation;
28. no high/medium open-proxy, DNS leak, auth bypass, unfiltered SOCKS-IRC, or resource-exhaustion finding remains.

## 21. Closure evidence required

`069-closure.md` must include:

- implementation commits/paths;
- SOCKS4a/5 negotiation matrix;
- command/address-type rejection evidence;
- target routing/no-local-DNS evidence;
- auth/outproxy evidence;
- SOCKS-IRC filter-reuse/static-path evidence;
- DCC/CTCP negative evidence;
- lifecycle/resource-bound results;
- option-capability matrix;
- registry/docs;
- containment/default-build outcomes;
- internal-only attestation;
- findings/disposition.

## 22. Stop conditions

Stop/replan if:

- correct direct routing requires literal IP/local DNS behavior;
- SOCKS-IRC cannot reuse M066 filter without duplicating it;
- UDP/BIND scope becomes necessary to satisfy an explicit pinned contract requirement not accounted for here;
- a core change is needed;
- non-loopback exposure cannot be made safe/truthful;
- a security-sensitive option would be accepted but ignored.
