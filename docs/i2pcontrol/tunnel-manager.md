# I2PControl TunnelManager

Status: M080-M085 server security corrective sequence closed;
M085 merged-head independent reclosure accepted;
tunnel runtime/security line complete against the pinned Proposal 170 revision
and current internal fork head;
lifecycle reconciliation remains closed against the pinned Proposal 170 revision

This document describes the Proposal 170 TunnelManager API handler in Emissary.
Wire/CRUD/persistence evidence is distinct from runtime data-plane support.

## Overview

The TunnelManager handler implements the `TunnelManager` JSON-RPC method for all declared tunnel types. It provides:

- Canonical CRUD operations (create, edit, get, delete) for tunnel definitions
- Compatibility `List` and capitalized action values
- Lifecycle dispatch (start, stop, restart) through the backend registry
- Ownership enforcement for startup-managed tunnels
- Real control-plane lifecycle for all twelve tunnel families, including
  bounded Streamr datagram producer/consumer runtimes

Production inventory is the deterministic union of startup-configured generic
client/server definitions and persisted control-plane definitions. Startup
definitions are read-only observations and are not copied into the persistent
generation store. A duplicate name across the two sources fails closed during
I2PControl initialization; create and rename reject startup-owned names.

Startup-managed generic client/server managers remain externally owned and are
never adopted by I2PControl. Control-plane-created generic `client` definitions
use an independent Yosemite streaming session and an I2PControl-owned,
per-name supervisor with readiness, cancellation, restart, and failure cleanup.

Control-plane-created generic `server` definitions use the I2PControl-owned
Yosemite accepted-stream runtime through the same bounded per-name ownership
model. Shared peer-aware admission runs before any local-target connection,
then the admitted stream is relayed byte-for-byte to a fixed loopback target;
the generic backend does not parse an application protocol or issue SAM
`STREAM FORWARD`.
The first successful start allocates a stable internal identity and stores its
persistent destination below `server-destinations/` in the I2PControl state
root. The key is never accepted as `PrivKeyFile`, copied into `rawConfig`, or
returned by `get`. The actual public destination is available to
`ClientServicesInfo` only after the backend has established the session.
Startup-managed server definitions remain externally owned.

M065 also provides internal-only runtime seams for specialized backends:
the client seam owns one outbound Yosemite session, a validated local listener,
and bounded per-connection tasks; the accepted-server seam owns one persistent
session and passes the SAM-derived public peer identity plus stream to a
protocol handler before any local target connection. M066 consumes those seams
for the IRC family and M067 consumes them for HTTP. `ircclient` uses one bounded
line-oriented filter for both traffic directions; `ircserver` filters
registration before connecting to loopback, bounds the local connect to five
seconds, and expires registered-idle streams after ten minutes while resetting
the deadline on traffic; `httpserver` normalizes bounded
HTTP headers before connecting to loopback and filters response fingerprints.
Streamr uses one owner loop per runtime. `streamrserver` accepts one-byte
subscribe/refresh and unsubscribe controls from authenticated Yosemite peer
destinations, expires subscriptions after 60 seconds, caps the set at 10, and
fans out payloads of at most 1200 bytes. Its local UDP source is loopback-only;
non-loopback `TargetHost`, `Host`, `ReachableBy`, and typed `ListenInterface`
values are rejected before resource allocation, and unexpected non-loopback
packet sources are dropped. Subscriber destination text is capped at 524 bytes.
`streamrclient` refreshes every 15 seconds and forwards payloads only to its
administrator-configured loopback UDP target; remote payloads cannot change
that target. Yosemite's 4095-byte datagram receive ceiling is the transport
buffer bound; no general UDP tunnel is introduced.

After the durable definition and server-identity stores load, `StartOnLoad` is
reconciled only for control-plane-owned `client`, `httpclient`, `connectclient`,
`ircclient`, `socks`, `socksirc`, `server`, `httpserver`, `httpbidirserver`, and
`ircserver`
definitions. Each start is isolated; a failed definition remains stopped and
does not prevent the service or other eligible definitions from starting.
Unsupported and startup-managed definitions are never auto-started.

## Actions

The seven canonical actions are lowercase:

| Action | Description |
|---|---|
| `create` | Create a new tunnel definition |
| `edit` | Update an existing tunnel definition |
| `get` | Retrieve a tunnel definition by name |
| `delete` | Remove a tunnel definition |
| `start` | Start a tunnel through its backend |
| `stop` | Stop a tunnel through its backend |
| `restart` | Restart a tunnel (stop then start) |

Capitalized values and `List` are compatibility extensions, not Proposal 170
actions. `List` is excluded from the canonical action manifest.

### Action restrictions

`All` is accepted only for the lowercase canonical `start`, `stop`, and `restart` actions. It is rejected with an error for `create`, `edit`, `get`, and `delete`.

## Tunnel types

All 12 Proposal 170 tunnel types are accepted:

| Type | Category |
|---|---|
| `client` | Client |
| `httpclient` | Client |
| `ircclient` | Client |
| `socks` | Client |
| `socksirc` | Client |
| `connectclient` | Client |
| `streamrclient` | Client |
| `server` | Server |
| `httpserver` | Server |
| `httpbidirserver` | Server |
| `ircserver` | Server |
| `streamrserver` | Server |

## Request format

All requests follow the JSON-RPC 2.0 envelope:

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "TunnelManager",
  "params": {
    "Action": "<action>",
    "Type": "<type>",
    "Name": "<name>",
    ...
  }
}
```

### Required fields

- `Action` - One of the seven lowercase canonical action strings
- `Name` - Required for `create`, `edit`, `delete`, and single-tunnel `get`, `start`, `stop`, or `restart`
- `Type` - Required for `create`

### Optional fields

- `All` - Boolean, valid only for lowercase `start`, `stop`, and `restart`
- `NewName` - String, valid only for `Edit` (atomic rename)
- Tunnel option fields (see Tunnel options)

## CRUD operations

### Create

Creates a new tunnel definition with the specified type and name.

```json
{
  "Action": "create",
  "Type": "socks",
  "Name": "my-proxy",
  "i2p.tunnel.listenPort": 1080
}
```

- `Type` and `Name` are required
- Duplicate names return a Proposal 170 status error
- Control-plane ownership is assigned automatically
- `StartOnLoad` is stored durably and is applied during post-load reconciliation
  for eligible control-plane `client` and `server` definitions

### Edit

Updates an existing tunnel definition. Preserves omitted fields.

```json
{
  "Action": "edit",
  "Name": "my-proxy",
  "NewName": "renamed-proxy",
  "i2p.tunnel.listenPort": 2080
}
```

- `Name` is required
- `NewName` performs an atomic rename with collision detection
- Stopped control-plane renames preserve backend state (including server
  destination identity); edits and renames while a tunnel is starting, running,
  or stopping are rejected
- Startup-managed definitions are rejected

### Get

Retrieves a tunnel definition by name.

```json
{
  "Action": "get",
  "Name": "my-proxy"
}
```

The canonical response has no legacy `Name`, `Type`, `State`, flattened
`i2p.tunnel.*`, or fabricated destination fields:

```json
{
  "jsonrpc": "2.0",
  "result": {
    "status": "success - options for my-proxy",
    "info": {
      "client": true,
      "status": "stopped",
      "persistentClientKey": false,
      "offlineKeys": false,
      "targetDestination": "exampleDestinationString",
      "rawConfig": {
        "name": "my-proxy",
        "type": "client",
        "Port": 7656,
        "TargetDestination": "exampleDestinationString",
        "StartOnLoad": false
      }
    }
  },
  "id": 1
}
```

Canonical `get` returns `{status, info}`. Canonical `create`, `edit`, lifecycle,
and `delete` return a structured `{status}` result, with `results` where the
operation returns a result list. Valid canonical operations that fail during
lookup, ownership, persistence, or backend dispatch still return
`result.status: "error - ..."`; malformed requests remain JSON-RPC errors.
Compatibility requests retain their historical response shapes.

### Delete

Removes a tunnel definition.

```json
{
  "Action": "delete",
  "Name": "my-proxy"
}
```

- Startup-managed definitions are rejected
- A running eligible tunnel is stopped and awaited before its definition is
  removed; failed stop preserves durable state
- Delete of absent name is a successful no-op

### Compatibility List

Returns all tunnel definitions as an array.

```json
{
  "Action": "List"
}
```

## Lifecycle operations

### Start

```json
{
  "Action": "start",
  "Name": "my-proxy"
}
```

Dispatches through the backend registry. Generic control-plane `client`
definitions bind and establish their independent session before successful
start returns. Canonical statuses are translated at
the handler boundary and are tied to the requested action/name:
- `success - starting tunnel <name>` for supported backends
- `error - start tunnel <name> not implemented` for unsupported backends
- Ownership error for startup-managed definitions

### Stop

```json
{
  "Action": "stop",
  "Name": "my-proxy"
}
```

For unsupported backends, stop is safe and idempotent.

### Restart

```json
{
  "Action": "restart",
  "Name": "my-proxy"
}
```

Composed as exact stop completion followed by a reload of the latest durable
definition and start through the backend registry. Per-name lifecycle locks
prevent overlapping generations; unrelated names remain independently usable.

### All behavior

`All` snapshots the bounded deterministic inventory, skips startup-managed
definitions, and dispatches each remaining name in sorted order. Unsupported
types produce their explicit per-item not-implemented result without resource
allocation. One item failing does not prevent later items from being attempted.

```json
{
  "Action": "start",
  "All": true
}
```

- Bounded serial dispatch over all definitions
- Startup-managed definitions are skipped
- Maximum 1000 targets
- Returns aggregated status

## Ownership model

| Ownership | Mutations | Lifecycle | Source |
|---|---|---|---|
| `ControlPlane` | Allowed | Allowed | Created via `Create` action |
| `StartupManaged` | Rejected | Rejected | Existing startup configuration |

## Tunnel options

The following matrix is the canonical Proposal 170 option inventory. Fields in
the second row are parsed into typed Emissary options; fields in the third row
are accepted, validated where the proposal gives a bound/type, and retained in
`rawConfig` for lossless round-trip. Retention is not runtime support: applicable
client cells below are accepted only when their backend changes real behavior.

M097 applies the common session controls that current Yosemite/SAM can carry on the
actual session-creation path. `TunnelLength` (0–3), `TunnelQuantity` (1–6), and
typed `EncType` values are validated before allocation and map to both inbound and
outbound session settings. `EncType` accepts current core-supported values 3–7,
with at most two distinct values and at most one ML-KEM variant.

M111 adds the generic session-wire controls supplied by the accepted Yosemite fork:
`TunnelVariance` (−2–2) and `TunnelBackupQuantity` (0–3) reach inbound/outbound
`SESSION CREATE` options and are consumed by the neutral tunnel-pool owner;
`CustomOptions` accepts at most 32
validated `i2cp.*` token pairs (64-byte keys, 256-byte values), rejects duplicate
or typed/reserved keys, and reaches the same Yosemite session serializer. `SigType`
was promoted by M111 for the router-native type 7 and was demoted by M121
(Outcome C) to blocked for all ten applicable families: configurable signing
semantics require non-Ed25519 key generation/signing Emissary cannot produce
end-to-end, so a singleton domain is inert rather than support. Any supplied
`SigType` value fails before allocation. All are
validated before session/listener allocation and participate in shared-session
compatibility.

`UseSSL` remains fail-closed: Proposal 170's local application/session TLS meaning
is distinct from Yosemite's SAM control-connection TLS field and has no accepted
Emissary owner. M131 also confirms that `SSLProxies` and `JumpList` are
HTTP-client-only reference behaviors; their non-HTTP cells are not applicable.
M112 added bounded `ConnectDelay` behavior
to the six streaming client listeners; M121 demotes M112's `Close`, `CloseTime`,
and `NewDest` cells to blocked (§5.2) because reference closeOnIdle observes
I2P-session activity while the local owner can only count accepted TCP handler
tasks and Yosemite exposes no session-activity observation primitive. M137
closes that gap router-side: `Close`/`CloseTime` are applied through the
canonical SAM idle owner (standard `i2cp.close*`, close-before-reduce,
canonical teardown, neutral reason); `NewDest` remains blocked and any supplied
value fails before allocation. `ConnectDelay`
(0–60,000 ms) remains applied via the generation-local outbound connector.
Shared definitions must carry compatible close/reduce policy to share one
underlying session; activity from any member resets the common idle clock and
idle close tears down the shared generation once.
M113 re-validated the remaining server presentation/routing and LeaseSet cells and
closed them as blocked. M125 corrected the two server-role `AllowInternalSSL`
cells to `not_applicable`, because Proposal 170 places that option under HTTP
client filtering. `UniqueLocalAddressPerClient` still has no bounded per-client
source-address owner without weakening M093 loopback confinement, while
`MultiHoming` maps to `shouldBundleReplyInfo`/LeaseSet reply bundling and has no
neutral server-session owner. `EncryptLeaseSet`, `OptionalLookup`, and
`LeaseSetClientAuths` have typed Y005 SAM fields but still lack Emissary
construction, key-custody, lookup/publication and session-handoff owners.
The underlying SAM transport is no longer the blocker: Yosemite Y004's canonical
generic fields were adopted by M122, and Yosemite Y005 (`59140a2`, adopted by
M124) adds cross-field auth/type consistency validation. The dependency now
serializes corrected generic fields, proven reachable by I2PControl adapter
tests, but that is transport reachability only; no Proposal runtime path maps
them and no downgrade is permitted, so the remaining 19 cells fail before
allocation. M131 closes the residual ledger at `284 apply / 88 blocked /
468 not_applicable`; no dependency-ready M132+ handoff exists.

| Disposition | Proposal 170 fields |
|---|---|
| Parsed and round-tripped | `Description`, `StartOnLoad`, `TargetDestination`, `Destination`, `TargetPort`, `ReachableBy`, `Port`, `TargetHost`, `Host` |
| Applied by accepted client proxy/filter runtimes | `ProxyList`, `ProxyAuth`, `ProxyUsername`, `ProxyPassword`, `OutproxyAuth`, `OutproxyUsername`, `OutproxyPassword`, `OutproxyType`, `AllowUserAgent`, `AllowReferer`, `AllowAccept` |
| Applied by six streaming client lifecycle owners | `ConnectDelay` (0–60,000 ms) |
| Applied by seven client idle owners (M136) | `Reduce` (boolean master switch), `ReduceCount` (1–6, default 1), `ReduceTime` (ms, minimum 300000, default 1200000) via standard `i2cp.reduce*` and the live-quantity primitive; `ReduceTime`/`ReduceCount` without `Reduce=true` fail before allocation |
| Applied by seven client idle owners (M137) | `Close` (boolean master switch), `CloseTime` (ms, minimum 300000, default 1800000) via standard `i2cp.close*` with close-before-reduce ordering and canonical teardown; `CloseTime` without `Close=true` fails before allocation |
| Rejected before allocation as residual client blockers | `UseOutproxyPlugin`, HTTP `SSLProxies`/`JumpList`, `Profile`, `NewDest`, Streamr `ConnectDelay` and other retained Streamr lifecycle cells; `SigType` for all applicable families |
| Applied by server runtimes | `WebsiteHostname`, `SpoofedHost`, `BlockAccessInProxies`, `BlockUserAgents`, `UserAgents`, `BlockReferers`, `AllowUserAgent`, `AllowReferer`, `AllowAccept`, `AccessOption`, `AccessList`, `FilterFilePath`, `MaxConcurrentConns`, `ClientPerMinute`, `ClientPerHour`, `ClientPerDay`, `TotalInPerMinute`, `TotalInPerHour`, `TotalInPerDay`, `PostLimit`, `PostLimitTime`, `PerClientPeriod`, `TotalPeriod`, `TotalBanTime` |
| Rejected before allocation as residual server blockers | `UniqueLocalAddressPerClient`, `MultiHoming`, `OptionalLookup`, `EncryptLeaseSet`, `LeaseSetClientAuths` |
| Validated and retained without an accepted runtime path | `TunnelLength` (0–3), `TunnelVariance` (−2–2), `TunnelQuantity` (1–6), `TunnelBackupQuantity` (0–3), `Shared`, `UseSSL`, `SigType`, `EncType`, `CustomOptions`, `PersistentClientKey`, `PrivKeyFile`, `LeaseSetClientAuths` |

`PrivKeyFile` is part of the pinned input inventory and is retained as a redacted
canonical path value, but start is rejected until confined import and atomic
handoff into an I2PControl-owned key store exist. New `OutproxyPassword` values
are held in the same typed redacted boundary as `ProxyPassword`; legacy raw
compatibility values remain response-redacted. Proxy credentials are bounded, separated between the local
listener and the configured I2P outproxy, and never serialized in canonical
`get` output. `UseOutproxyPlugin`, HTTP `SSLProxies`/`JumpList`, and `Profile`
fail before listener/session allocation because no exact Emissary-owned
primitive exists for them. The Reduce family is applied by M136: idle SAM
sessions decrease to the configured quantity through the live-quantity
primitive and restore on activity; malformed or server-family values fail
before allocation. The Close family is applied by M137: idle SAM sessions
tear down through the canonical session/pool path with a neutral
`IdlePolicy`/`Requested`/`Failure`/`Unknown` reason; malformed or
server-family values fail before allocation.
Streamr `DelayOpen` and `NewDest` are not applicable by affirmative reference gates; its other retained
lifecycle cells remain explicit blocked responses because generic reference
setters do not establish a datagram runtime owner, except Reduce and Close
which now use the shared datagram session owner. `AllowInternalSSL` is not
applicable to the accepted HTTP-only outbound client; server-role cells belong
to M099.

No field in this matrix starts a tunnel, creates a data-plane session, or
reports a fabricated runtime state. Unsupported backends return an explicit
status/error while preserving durable definitions.

### General

| Field | Type | Description |
|---|---|---|
| `description` | string | Tunnel description |
| `i2p.tunnel.startOnLoad` | boolean | Start on router load (stored, not executed) |

### Client options

| Field | Type | Description |
|---|---|---|
| `i2p.tunnel.clientDest` | string | Target destination |
| `i2p.tunnel.clientDestPort` | integer | Target port (0-65535) |
| `i2p.tunnel.listenInterface` | string | Listen interface |
| `i2p.tunnel.listenPort` | integer | Listen port (0-65535) |
| `i2p.tunnel.accessList` | string | Comma-separated destinations |
| `i2p.tunnel.allowplaintext` | boolean | Allow plaintext to I2P |

### Server options

| Field | Type | Description |
|---|---|---|
| `i2p.tunnel.serverHostingDestination` | string | Base64 RouterInfo |
| `i2p.tunnel.isPrivate` | boolean | Hidden service |
| `i2p.tunnel.hashcashProofsRequired` | integer | Hash cash level |
| `i2p.tunnel.signatureType` | string | Signature type |
| `i2p.tunnel.consumer` | string | Consumer |

### HTTP options

| Field | Type | Description |
|---|---|---|
| `i2p.tunnel.sslCertificate` | string | SSL certificate path |
| `i2p.tunnel.sslKey` | string (redacted) | SSL key path |
| `i2p.tunnel.httpHost` | string | HTTP host |

The `httpserver` backend additionally supports loopback-only `TargetHost`/`Host`,
`TargetPort`/`Port`, `WebsiteHostname`/`SpoofedHost`, access-list and
referer/User-Agent policy, bounded `MaxConcurrentConns`, peer/aggregate
`ClientPerMinute`/`ClientPerHour`/`ClientPerDay` and
`TotalInPerMinute`/`TotalInPerHour`/`TotalInPerDay` admission, and peer-keyed
`PostLimit`/`PostLimitTime`, confined newline-delimited `FilterFilePath`
access generations, and bounded `PerClientPeriod`/`TotalPeriod`/`TotalBanTime`.
Absent admission values default to 30 global
connections, 8 concurrent connections per peer, peer rates 30/80/200 per
minute/hour/day, and aggregate rates 50 per minute and unlimited per hour/day.
It rejects TLS termination, compression/custom options, proxy/outproxy
settings, `UniqueLocalAddressPerClient`, `MultiHoming`, and LeaseSet security
options before session allocation (M125 corrected the two server-role
`AllowInternalSSL` cells to not applicable; 19 M113 cells remain blocked with
exact primitive evidence and are not silently ignored). Request proxy identity and privacy headers are stripped, trusted
peer identity injection is bounded to the 524-byte reference destination
representation, and response fingerprint/provider/cache/trace headers are
removed before forwarding. Content-Length and valid chunked framing are
re-emitted in normalized form; application headers such as cookies and
Content-Type are preserved. `PostLimit`/`PostLimitTime` uses fixed-size hashed
peer accounting with lazy expiry and denies unseen peers when its 1024-entry
table is full; it never evicts active state. The same server filter and
limiter are consumed by the inbound `httpbidirserver` composition.

### Proxy options

| Field | Type | Description |
|---|---|---|
| `i2p.tunnel.proxyUsername` | string | Proxy username |
| `i2p.tunnel.proxyPassword` | string (redacted) | Proxy password |

### IRC options

| Field | Type | Description |
|---|---|---|
| `i2p.tunnel.ircServer` | string | IRC server address |
| `i2p.tunnel.ircPort` | integer | IRC port (0-65535) |
| `i2p.tunnel.ircNick` | string | IRC nick |
| `i2p.tunnel.ircPassword` | string (redacted) | IRC password |
| `i2p.tunnel.ircChannels` | string | Comma-separated channels |

These fields are stored for Proposal 170 round-tripping but rejected by the
M066 runtime before allocation. M066 forwards an explicitly configured I2P
destination and does not synthesize IRC registration or channel automation.
`ircclient` accepts only `TargetDestination`, `TargetPort`, `ReachableBy`, and
`Port`; `ircserver` accepts loopback `TargetHost`/`Host` plus `TargetPort` or
`Port`, and uses the same bounded peer-aware admission policy as `httpserver`.
The HTTP and IRC accepted-server families share the 30/8 concurrency defaults
and the peer/aggregate minute/hour/day controls described above. I2CP and
custom options, WEBIRC/cloak options, access/auth fields, and DCC-related
options are rejected. Unsupported CTCP and DCC payloads are blocked by the
common filter.

### Streamr options

| Field | Type | Description |
|---|---|---|
| `TargetDestination` | string | Remote Streamr producer destination for `streamrclient` |
| `i2p.tunnel.streamrTarget` | string | Alias for the remote Streamr producer destination |
| `TargetHost` / `Host` | string | Loopback-only local UDP target/source IP; defaults to `127.0.0.1` |
| `TargetPort` | integer | Local UDP target port for `streamrclient`; I2P destination port for `streamrserver` |
| `Port` | integer | Required local UDP source port for `streamrserver`; optional I2P source port for `streamrclient` |
| `ReachableBy` | string | Loopback-only typed local IP fallback when `TargetHost`/`Host` is absent |

Streamr rejects I2CP/custom option maps and recognized tunnel length, quantity,
variance, signature, and encryption options before session or UDP allocation.
The server destination identity is generated and retained by the backend-owned
`server-destinations/` store; subscriber state is intentionally ephemeral and is
cleared on restart.

### I2CP and custom options

- `i2cp` - Object with string key-value pairs
- `i2p.tunnel.customOptions` - Object with string key-value pairs

Canonical requests use the Proposal 170 option names above. Unknown
top-level canonical keys and compatibility-only `i2p.tunnel.*` aliases are
rejected. `CustomOptions` is the canonical arbitrary option container;
`LeaseSetClientAuths` is an array of objects and is persisted but omitted from
generic responses.

## Security

- Secret fields (`ssl_key`, `proxy_password`, `irc_password`) are redacted in Debug/Display
- Names are validated for length (max 1024) and control characters
- Descriptions are validated for length (max 4096)
- Port values are validated for u16 range
- Error messages do not expose Rust type names or internal state
- Secret values are stored once where future runtime use requires them and are
  omitted from canonical and compatibility responses
- Canonical `PrivKeyFile` input is rejected because generic raw configuration
  is not a key-material ingress path

## Static guards

- No `std::fs`, `tokio::fs`, or `std::net` imports in handler code
- No `tokio::spawn` calls
- No `dioxus` or UI module imports
- No `router.toml` mutations
- Unsupported backends allocate zero network/session/task resources

## Tests

The handler includes focused tests covering:

- Action and type parsing validation
- CRUD operations for all 12 tunnel types
- Edit with rename, field preservation, and collision detection
- Lifecycle operations through unsupported and fake backends
- `All` enforcement and mixed-target behavior
- Ownership rejection for startup-managed definitions
- Concurrent and race condition determinism
- Secret redaction in Debug/Display
- Static guards for no resource allocation
- Exact canonical `info` keys and lower-case `rawConfig.name/type`
- Unknown/malformed option rejection and `EncryptLeaseSet` enum boundaries
- Secret omission from responses and one-publication rename failure behavior
