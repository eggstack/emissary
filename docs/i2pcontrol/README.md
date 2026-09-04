# I2PControl for Emissary

Status: partial Proposal 170 support; M095-M096, M098-M103, and M107 closed; M097 and M104 closed as blocked; M126 historical requalification closed; M127 token-lifetime corrective closed; M128 bounded batch conformance closed; residual option blocker remains

Proposal 170 remains **Open**. This documentation is pinned to the revision
created and last updated on `2026-05-20`.

Historical invalidation and completed corrective sequence:

- `plans/closure/i2pcontrol-proposal-170/027-closure-invalidation.md`
- `plans/implementation/i2pcontrol-proposal-170/028-post-m027-status-and-addressbook-feature-isolation.md`
- `plans/implementation/i2pcontrol-proposal-170/029-in-scope-conformance-reclosure.md`
- `plans/closure/i2pcontrol-proposal-170/030-closure.md`
- `plans/closure/i2pcontrol-proposal-170/034-closure.md`
- `plans/closure/i2pcontrol-proposal-170/035-closure.md`
- `plans/closure/i2pcontrol-proposal-170/039-closure.md`

M020–M027 implementation/evidence remains retained, but M027's final disposition
is historical invalidated evidence. The post-M027 merge that revived M019 is
historical and superseded. M028 restored strict AddressBook feature/runtime
isolation. M030 corrected destination/lookup owner coherence and independently
closed the AddressBook dimension; the bounded result remains partial support.
M035 corrected the base method inventory and separated direct Proposal 170
RouterInfo requests from historical nested compatibility requests.
M039 independently reviewed the complete M031–M038 final head and formally
closed the authorized workstream as partial support.

The later M066–M071 tunnel-family sequence made all twelve production tunnel
types real. M072 and M073 are retained historical runtime-composition and
option-truthfulness closures. The current aggregate workstream is M095, which
records the exact remaining RouterInfo, SetConfig, tunnel-option, and owner/path
boundaries without overstating production support.

The authoritative aggregate handoff is
[`095-full-support-matrix.toml`](../../plans/implementation/i2pcontrol-proposal-170/095-full-support-matrix.toml),
with focused exhaustiveness coverage in
`emissary-cli/tests/m095_full_support_matrix.rs`. It records the current baseline
and later owner/path budgets; it does not promote any planned cell to production
support.

The expected bounded final status remains `partial Proposal 170 support` because the
integrated tunnel-option and live-interoperability work is not closed. M066 through M071
close the IRC, HTTP, CONNECT, SOCKS, bidirectional HTTP, and Streamr families. The
canonical 43-addition matrix currently contains 42 available selectors, one
protocol-permitted neutral selector, and no unavailable selectors. Banned peers use an
authoritative by-design-empty source because Emissary has no router-wide ban facility;
this does not imply ban management.
M100 supplies transit bandwidth through a bounded request-independent
I2PControl-owned sampler, and M101 supplies authenticated signed router news
through a bounded I2PControl-owned refresh/cache task. M102 supplies explicit
neutral v4/v6 network-error state from the existing SSU2 reachability owner;
uninitialized and firewalled families remain unavailable.

## Compile feature

I2PControl is an independent Cargo feature in `emissary-cli`. It is **not** enabled by default.

```bash
# Build without I2PControl
cargo build -p emissary-cli --no-default-features

# Build with I2PControl enabled as a compile-time feature
cargo build -p emissary-cli --no-default-features --features i2pcontrol

# Build with UI and I2PControl
cargo build -p emissary-cli --all-features
```

M028 specifically provides proof that a build without `i2pcontrol`, and a build
where the feature is compiled but runtime configuration is disabled, do not
read, write, migrate, or consult Proposal 170 AddressBook control state.
When enabled, one runtime owner is authoritative for administrative, RouterInfo,
Base32, and Base64 AddressBook views.

## Runtime enablement

Even when compiled with the `i2pcontrol` feature, the service is disabled by
default. It starts only when explicitly enabled in configuration.

```toml
[i2pcontrol]
enabled = true
bind = "127.0.0.1:7650"
password = "your-secure-password"
```

## Configuration

| Field | Type | Default | Description |
|---|---|---|---|
| `enabled` | boolean | `false` | Enable the I2PControl listener and Proposal 170 control owners |
| `bind` | string | `"127.0.0.1:7650"` | Bind address |
| `password` | string | `""` | Authentication password |
| `certificate` | string | managed | Optional TLS certificate path |
| `private_key` | string | managed | Optional TLS private-key path |

### Security notes

- Default binding is loopback only.
- Non-loopback binding requires explicit configuration and produces a warning.
- Empty password is rejected when the service is enabled.
- Existing configurations without `[i2pcontrol]` remain valid.
- Authentication, token placement, secret persistence, and response redaction
  were corrected in M020/M021 and remain retained evidence.
- Disabled/default AddressBook isolation was corrected by M028 and independently
  revalidated in the M030 final-head review.

## HTTPS certificate behavior

I2PControl is served over HTTPS.

1. When `certificate` and `private_key` are configured, those files are loaded.
2. Otherwise, a managed self-signed certificate is generated under
   `<base_path>/i2pcontrol-certs/`.
3. Managed material is generated only when I2PControl starts, is written
   atomically, remains stable across restart, and is regenerated when invalid.
   The generated certificate covers `localhost`, `127.0.0.1`, and `::1`; on
   Unix, the managed private key is owner-only (`0600`) and the managed
   directory is owner-only (`0700`) when created. Managed symlinks and other
   non-regular paths fail closed. Explicit certificate and key paths retain
   their operator-owned behavior.

There is no plaintext HTTP fallback.

## Authentication

`Authenticate` accepts API version `1` in `API` and a `Password`, returns an
opaque string `Token` and numeric `API`, and protected requests put the token
in `params.Token`. Rejected API version `2` returns `-32006` and does not issue
a token.

```json
{
  "jsonrpc": "2.0",
  "method": "Authenticate",
  "params": {
    "API": 1,
    "Password": "your-password"
  },
  "id": 1
}
```

Success:

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "Token": "hex-encoded-token",
    "API": 1
  }
}
```

Protected request:

```json
{
  "jsonrpc": "2.0",
  "method": "RouterInfo",
  "params": {
    "Token": "hex-encoded-token",
    "i2p.router.version": true
  },
  "id": 2
}
```

`X-I2PControl-Token` is compatibility-only. When both token forms are present,
they must match.

Authentication failures use the I2PControl-specific error inventory:

- invalid password: `-32001`;
- missing token: `-32002`;
- unknown token: `-32003`;
- expired token: `-32004` on first use after expiry;
- missing API version: `-32005`;
- unsupported API version: `-32006`.

Tokens are cryptographically random, bounded (1024 live entries, deterministic
oldest eviction), in-memory only, and invalidated on process restart. Every
issued token has finite monotonic in-process validity of one day
(`TOKEN_LIFETIME`; reference-compatible). The first protected use after
expiry atomically removes the token and returns `-32004`; later uses of the
same value return `-32003` and clients must re-authenticate. Presented
credentials over 256 bytes fail as unknown without echo or unbounded
allocation. M127 supersedes only M126's affected authentication-lifetime
qualification claim; M126 history is otherwise unchanged.

JSON-RPC notifications execute validation and side effects but suppress the
response. An explicit `id: null` remains a request ID rather than a
notification.

## JSON-RPC batch requests

The HTTP body may contain either one JSON-RPC request object or one
non-empty batch array of request entries (M128). Batch behavior:

- at most `MAX_BATCH_ELEMENTS = 32` entries per batch (at most half the
  64-request concurrent in-flight budget; the 1 MiB body cap applies
  independently);
- an empty array is a single invalid-request error (`-32600`, null ID);
- an over-cap batch is a single invalid-request error and executes zero
  elements;
- each entry is validated with the exact single-request rules: a non-object
  entry (scalar, null, nested array) contributes a per-entry
  invalid-request error with null ID without disturbing valid siblings,
  while named-object params remain required per entry;
- authentication is strictly per element with the same valid/expired/unknown
  semantics as single requests; an `Authenticate` entry never propagates
  its token to siblings, and request-wide `X-I2PControl-Token` header
  compatibility is reconciled per element exactly as for single dispatch;
- responses preserve input order in one JSON array; structurally valid
  notifications execute but contribute no element, so an all-notification
  batch with no invalid entries emits no JSON-RPC body (`204 No Content`);
- elements dispatch sequentially under the single held request permit and
  deadline; no task is spawned per element;
- batching is not a transaction: mutations committed by earlier elements
  are not rolled back because a later element fails or the client
  disconnects.

## Current retained implementation

Retained implementation includes:

- feature-gated HTTPS serving with bounded connections, bodies, and requests;
- standard authentication and JSON-RPC behavior;
- exact Proposal 170 method/selector/action/type parsers and literal fixtures;
- durable generation stores and atomic TunnelManager mutation;
- bounded Streamr producer/consumer datagram backends with fixed local UDP targets;
- M065 bounded I2PControl-owned client-listener and accepted-server runtime primitives;
- M065 backend-local option capability validation that rejects unsupported runtime options before
  listener/session allocation and redacts option values;
- startup-managed tunnel inventory and service lifecycle observation;
- bounded recoverable SAM observation;
- RouterInfo source classification and no-fabrication behavior;
- enabled-mode runtime AddressBook authority.
- reviewed constant-time authentication, bounded failed-login throttling, and
  bounded publication with prior-generation recovery;

M028 does not reopen these areas except for the AddressBook activation boundary.

## Corrective sequence

| Milestone | Status | Scope |
|---|---|---|
| M020–M027 | retained evidence | base/wire/persistence/source corrections and literal review |
| M028 | closed for implementation | status chronology and AddressBook control-state isolation |
| M029 | historical invalidated closure | retained non-AddressBook evidence |
| M030 | closed; partial Proposal 170 support | AddressBook destination/lookup coherence and final-head review accepted |
| M053 | closed | corrected M045 with live ProfileStorage source |
| M054 | closed | corrected M049 transit-15s truthfulness; explicit unavailable disposition |
| M055 | closed | corrected M050 network-error truthfulness; explicit unavailable dispositions |
| M056 | closed | integrated M049/M050/M052 reclosure; historical final 37/1/5 source matrix |
| M100/M101 | closed | request-independent transit-15s sampler and bounded signed news source |
| M102 | closed | explicit neutral v4/v6 network-error owner; wire mapping remains in I2PControl |

## Support dimensions

Claims are separated into:

- **Wire** — exact names, casing, presence rules, response fields, and JSON types.
- **Source** — a truthful current Emissary source exists.
- **Runtime** — a real backend performs the operation.
- **Persistence** — mutation is process-crash atomic with prior-generation
  recovery; where directory synchronization is supported and succeeds, the
  documented power-loss durability point is also reached.
- **Feature isolation** — disabled/default execution is unaffected by the administrative feature.
- **Evidence** — literal, failure, restart, composition, and transition proof exists.

Compatibility aliases, unavailable fields, stored definitions, and unsupported
backend stubs are not operational coverage.

## RouterInfo source status

The retained matrix contains:

- 42 available selectors;
- 1 protocol-permitted neutral selector;
- 0 unavailable selectors.

Unavailable selectors, if introduced by a future source regression, fail explicitly and
are never substituted with zero, false, empty, or semantically adjacent values. The
authoritative banned-peer empty map is a distinct by-design capability result.

## Streamr tunnel data plane

M071 provides bounded `streamrclient` and `streamrserver` runtimes. The server
keeps a persistent Yosemite repliable-datagram identity, receives administrator-
bound loopback UDP payloads, and fans them out to at most 10 subscribed destinations.
The client sends a one-byte subscribe/refresh (`0`) every 15 seconds, attempts a
best-effort unsubscribe (`1`) during bounded shutdown, and forwards received
payloads only to its configured loopback UDP target. Non-loopback local UDP
addresses are rejected before allocation. Subscriptions expire after
60 seconds without refresh and payloads are capped at 1200 bytes (Yosemite's
4095-byte receive ceiling is retained as the transport buffer bound).

Yosemite exposes the authenticated remote destination but not inbound datagram
port metadata. Emissary therefore keys subscriptions by that trusted destination
and uses the configured session port tuple; no core/router API change was needed.

## AddressBook enabled/disabled boundary

M028 implementation evidence now covers:

- without the compile-time feature, Proposal 170 control state is absent;
- with the feature compiled but runtime-disabled, control state is not read or
  written;
- enabled mode constructs one control owner shared with normal lookup;
- disabling after prior use preserves but ignores control-state files;
- re-enabling restores the retained state;
- ordinary legacy address files and downloads remain authoritative while the
  control plane is inactive.

M030 independently reviewed this corrected implementation head. The accepted
subsystem status is `partial Proposal 170 support`; enabled AddressBook
administrative, RouterInfo, Base32, and Base64 views now share one validated
full-destination owner.

## No frontend controls

I2PControl does not add frontend controls, views, or frontend-owned state.

## Internal-only boundary

All work is internal to `eggstack/emissary`.

No plan authorizes upstream issues, pull requests, reviews, discussions,
submissions, patches, maintainer outreach, contribution preparation, adoption
requests, or merge activity. External specifications and source trees may be
inspected read-only solely for internal correctness.
