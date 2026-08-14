# M066 — IRC Client and IRC Server Tunnel Family

Status: blocked — hard dependency on M065 closure

Planning production baseline: `a1296b018ce98d26a019bd5064dff9f4b47e0ad6`

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-tunnel-runtime-completion-roadmap.md`.

Authority:

- `plans/000-long-term-specification.md`;
- `plans/003-planning-process.md`;
- `plans/adrs/ADR-0003-proposal-170-tunnel-runtime-completion-and-filter-boundary.md`.

Dependencies:

- hard: M065 accepted client/accepted-server runtime and option-capability primitives;
- interface: existing TunnelManager domain/persistence/registry remains unchanged.

## 1. Objective

Implement a single independently authored IRC anonymity/filtering layer and use it to replace the `ircclient` and `ircserver` unsupported backends with real control-plane-owned runtimes.

`ircclient` and `ircserver` are not generic byte tunnels. Completion requires filtering before potentially identifying or attacker-controlled IRC registration/control data crosses the anonymity/application boundary.

DCC and WEBIRC are explicitly not required for M066. They must fail closed rather than be silently passed through or accepted as inert configuration.

## 2. Current/reference evidence

The Java reference implementation treats IRC client traffic as line-oriented filtered traffic, not raw relay. Security-relevant behaviors include:

- command classification/allowlisting;
- filtering CTCP except safe/explicit forms;
- DCC handling because it carries IP/port information;
- rewriting USER hostname/servername fields;
- PING/PONG handling that avoids reflecting proxy/local address data;
- replacement of user-controlled PART/QUIT text;
- IRCv3 message-tag and CAP/SASL compatibility.

The reference IRC server separately filters the registration phase, imposes strict time/line/count bounds, rejects obvious wrong protocols, and replaces the client's presented host identity with a value derived from its I2P destination.

These are behavioral/security references only. Rust implementation must be independently authored.

## 3. Classification

Primary class: capability / security.

Types promoted on successful closure:

- `ircclient`;
- `ircserver`.

`socksirc` remains unsupported until M069 and must reuse this filter then.

## 4. Hard invariants

- all new IRC production logic remains under `emissary-cli/src/i2pcontrol/**`;
- no `emissary-core/**` change;
- no changes to startup IRC behavior (none is required);
- no public Proposal 170 type/field/action/status change;
- no DCC auxiliary tunnel creation;
- no WEBIRC secret support in M066;
- unknown/unsafe CTCP is blocked rather than raw-forwarded;
- server-side presented peer identity is derived from trusted accepted-stream identity, not remote IRC fields;
- local IRCd connection for `ircserver` occurs only after bounded registration material is available and accepted;
- filter state is per connection and cannot leak PING/PONG or registration state between clients;
- line and total-registration bounds are explicit constants/configured caps and tested;
- secrets such as IRC password are redacted;
- backend option validation happens before listener/session allocation;
- startup-managed resources remain untouched.

## 5. Explicit non-goals

Do not:

- implement DCC CHAT/SEND/RESUME/ACCEPT tunnels;
- implement XDCC-specific transport helpers;
- implement WEBIRC;
- add IRC UI/config beyond Proposal 170 fields;
- build a full RFC IRC parser/daemon;
- support arbitrary server-side cloak scripting;
- refactor unrelated proxy modules;
- add clearnet IRC outproxy behavior not required by Proposal 170;
- promote `socksirc` yet;
- add new core peer identity APIs.

## 6. Proposed implementation surface

Expected I2PControl-local modules:

```text
emissary-cli/src/i2pcontrol/backends/filters/irc.rs
emissary-cli/src/i2pcontrol/backends/irc_client.rs
emissary-cli/src/i2pcontrol/backends/irc_server.rs
emissary-cli/src/i2pcontrol/backends/registry.rs
```

Exact names may differ. Keep one common IRC filtering implementation consumable by M069.

## 7. Common IRC filter requirements

### 7.1 Parsing model

Use a bounded line-oriented parser over IRC's byte-oriented protocol. Do not assume UTF-8 for safety decisions; preserve valid opaque text where possible while classifying ASCII command structure.

Required parser features:

- optional IRCv3 message tags (`@...`);
- optional source prefix (`:...`);
- command token;
- bounded parameter split sufficient for security decisions;
- CRLF normalization;
- reject embedded NUL/control forms that break line semantics;
- hard maximum line length.

Do not parse message bodies more deeply than required for CTCP/DCC/security behavior.

### 7.2 Client-to-network filter

Required behavior:

- permit a documented set of normal user commands needed by contemporary clients;
- pass CAP/AUTHENTICATE/SASL-related commands required for normal registration;
- rewrite `USER` so local hostname/servername cannot be disclosed; preserve username/mode/realname where safe;
- sanitize PING forms that include an additional local/server location parameter so local/proxy address data is not sent;
- track only the minimum per-connection expected PONG state required to preserve client behavior;
- replace PART free-text reason with a fixed neutral reason;
- replace QUIT free-text reason with a fixed neutral reason;
- permit PRIVMSG/NOTICE ordinary text;
- for CTCP-delimited PRIVMSG/NOTICE, allow ACTION; block unsupported CTCP;
- detect DCC CHAT/SEND/RESUME/ACCEPT and block it deterministically;
- do not log blocked raw messages at levels/content that may expose passwords/tokens/private chat text.

Any command outside the explicitly supported safety set should fail closed unless the implementation plan's reviewed compatibility fixtures prove it is safe to pass.

### 7.3 Network-to-client filter

Required behavior:

- numeric replies pass unless a specific security rewrite is required;
- common server commands (PING, MODE, JOIN, NICK, QUIT, PART, ERROR, KICK, TOPIC, CAP, AUTHENTICATE, PROTOCTL, AWAY, ACCOUNT, CHGHOST and other reviewed safe forms) may pass;
- PONG may be rewritten using per-connection expected-PONG state;
- PRIVMSG/NOTICE ordinary text passes;
- CTCP ACTION may pass;
- DCC/unsupported CTCP is blocked;
- malformed command structure is dropped/connection-failed according to deterministic policy.

### 7.4 State isolation

PING/PONG/filter state must be scoped to one connection and bounded to a tiny fixed amount of memory. No global map keyed by nickname is needed.

## 8. `ircclient` backend

Required runtime:

```text
local IRC client
    -> control-plane local TCP listener
    -> common IRC outbound filter
    -> Yosemite stream to configured I2P destination/port
    -> common IRC inbound filter
    -> local IRC client
```

Requirements:

- validate `TargetDestination`, target port/default, listen interface/port, and IRC-specific Proposal 170 fields before allocation;
- direct target must be I2P destination/name through existing approved resolution path;
- no local DNS fallback;
- local listener exposure/auth policy follows supported Proposal 170 fields;
- connect/read/filter errors close only the connection;
- tunnel supervisor remains running for subsequent connections unless session-level fatal error occurs;
- connection count bounded;
- lifecycle uses M065 exact cancellation/generation rules.

Proposal 170 IRC-specific fields such as server/port/nick/password/channels must be mapped explicitly. If some fields represent client automation rather than tunnel forwarding and are not needed/implemented, backend start must reject the unsupported relevant field instead of ignoring it.

## 9. `ircserver` backend

Required runtime:

```text
remote I2P IRC client
    -> accepted Yosemite stream + trusted peer identity
    -> bounded registration filter
    -> configured local IRCd
    -> raw relay after accepted registration
```

### 9.1 Registration bounds

Initial target bounds should be intentionally conservative and may mirror reference scale without copying implementation:

- first/registration line timeout on order of seconds, not minutes;
- total registration completion timeout bounded;
- line length around normal IRC limits with limited compatibility headroom, never unbounded;
- pre-USER/SERVER line count bounded (e.g. low double digits).

Exact constants must be documented/tested.

### 9.2 Cross-protocol rejection

Before passing registration, reject obvious signatures for HTTP request methods and common binary protocols that should never hit an IRC endpoint. This is defense-in-depth and prevents the local IRCd from becoming a generic service probe target.

### 9.3 USER identity rewrite

When registration contains USER:

- parse required fields;
- replace the client-provided hostname with a deterministic safe representation derived from trusted remote I2P identity (prefer B32-based public identity or a deterministic keyed cloak if explicitly implemented);
- do not expose local router/server addresses;
- preserve safe username/mode/realname semantics;
- forward prior safe registration lines plus rewritten USER only after validation.

Initial M066 should prefer the simplest deterministic peer-B32 hostname behavior. A configurable cloak key/WEBIRC path is not required and must be rejected if requested.

### 9.4 Local target safety

TargetHost should default to loopback and be explicitly validated. No incoming IRC command may influence target host/port.

## 10. Option-capability contract

Before resource allocation, each backend must reject unsupported relevant fields/custom options.

Required explicit disposition table in code/tests/docs for at least:

- listen interface/port;
- target destination/port for client;
- target host/port for server;
- access/auth fields;
- IRC server/port/nick/password/channels fields;
- I2CP tunnel/session fields actually supported by Yosemite session options;
- WEBIRC/cloak custom options;
- DCC-related custom options.

Password values must never appear in rejection/error strings.

## 11. Ordered work packages

### WP1 — common parser/filter with fixtures

Implement independent filter and table-driven tests covering safe/blocked/rewrite cases before wiring networking.

Include modern IRCv3 message tags and CAP/SASL fixtures.

### WP2 — `ircclient` runtime/backend

Wire M065 client listener/session primitive, common bidirectional filter, lifecycle, option validation, and registry promotion for `ircclient` only after focused tests pass.

### WP3 — `ircserver` registration filter/runtime

Wire M065 accepted-server primitive. Prove trusted peer identity drives USER rewrite and local target receives only sanitized registration.

### WP4 — adversarial/security tests

At minimum test:

- USER with local IP/hostname;
- PING with extra IP/location;
- PART/QUIT custom identifying text;
- CTCP VERSION/TIME/etc blocked;
- DCC CHAT/SEND/RESUME/ACCEPT blocked;
- CTCP ACTION passes;
- multiple/malformed CTCP markers fail closed;
- overlong line;
- registration slowloris timeout;
- too many pre-USER lines;
- HTTP/BitTorrent-like first input to IRC server rejected;
- spoofed hostname replaced by peer-derived identity;
- concurrent clients do not share state;
- filter logs do not echo IRC password/private content.

### WP5 — lifecycle/registry/docs

Promote only `ircclient` and `ircserver` in production registry. Leave `socksirc` unsupported.

Update support documentation with DCC/WEBIRC limitations and option matrix.

## 12. Failure, cancellation, restart, contention semantics

- one malformed/blocked IRC line normally closes or drops according to explicit filter policy; it must not panic;
- connection-level failure does not stop sibling connections;
- session/listener fatal failure marks the named backend failed;
- stop cancels listener/accepted-session and every current-generation connection task;
- restart creates fresh per-connection filter state and retains server destination identity;
- duplicate start remains rejected;
- stale task completion ignored after restart;
- server local-target connect failure returns a bounded synthetic failure/close behavior and does not leak target details over I2P.

## 13. Compatibility and migration

No public/persistence schema migration.

Previously persisted `ircclient`/`ircserver` definitions become startable only if their configured options fall within the implemented capability set. Otherwise start fails truthfully and definition remains editable.

`socksirc` remains unsupported until M069.

## 14. Verification commands

Focused commands should include:

```text
cargo test -p emissary-cli --no-default-features --features i2pcontrol irc
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m062_dependency_containment
cargo check -p emissary-cli --no-default-features
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo check -p emissary-core --no-default-features
git diff --check
```

Add a bounded end-to-end fake-SAM/local TCP test proving filtered bytes cross both `ircclient` and `ircserver` paths.

No external IRC network required.

## 15. Acceptance criteria

M066 may close only when:

1. M065 is closed;
2. one common independently authored IRC filter is used by `ircclient` and available for future `socksirc`;
3. line parser has explicit size bounds and handles IRCv3 tags/prefixes needed for safety decisions;
4. `USER` client outbound local hostname/servername leakage is prevented;
5. PING/PONG address-leak case is sanitized;
6. PART/QUIT user text is neutralized per adopted policy;
7. ordinary PRIVMSG/NOTICE works;
8. CTCP ACTION works;
9. unsupported CTCP fails closed;
10. DCC CHAT/SEND/RESUME/ACCEPT fails closed with no auxiliary listener/session;
11. `ircclient` is operational through real local-listener -> I2P stream traffic;
12. `ircserver` uses accepted I2P streams, not blind forwarding;
13. server registration is filtered before local IRCd connect/write;
14. server USER hostname is derived from trusted I2P peer identity;
15. registration timeout, line limit, and pre-registration line count are enforced;
16. cross-protocol first-input cases are rejected;
17. WEBIRC/cloak options requested but not implemented are rejected before allocation;
18. passwords/raw private IRC text are not emitted by filter diagnostics;
19. per-connection state is isolated and bounded;
20. lifecycle start/stop/restart/cancellation is exact-name and generation-safe;
21. only `ircclient` and `ircserver` replace unsupported backends; `socksirc` remains unsupported;
22. no `emissary-core/**` production change;
23. no non-I2PControl production change unless individually justified/approved;
24. feature-disabled/default and containment checks remain green;
25. support docs accurately list DCC/WEBIRC limitations and option dispositions;
26. no CI/release/fuzz/coverage expansion;
27. no upstream/third-party write/review/submission/contribution preparation.

## 16. Closure evidence required

`066-closure.md` must include:

- implementation commits/changed paths;
- filter behavior matrix (pass/rewrite/drop);
- DCC/CTCP negative evidence;
- server registration bounds and cross-protocol evidence;
- trusted peer identity rewrite proof;
- client/server e2e traffic proof;
- lifecycle failure/cancellation evidence;
- option-capability matrix;
- registry mapping before/after;
- containment/default-build outcomes;
- documentation review;
- internal-only attestation;
- unresolved findings and disposition.

## 17. Stop conditions

Stop/replan if:

- normal modern IRC cannot function without allowing an unreviewed leak-prone command path;
- server filtering cannot occur before local IRCd receives registration;
- peer identity cannot be obtained through M065 accepted-stream boundary;
- implementation pressure leads to DCC auxiliary tunnel scope;
- a core change appears necessary;
- a security-sensitive field must be silently ignored to claim support.