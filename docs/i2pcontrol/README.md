# I2PControl for Emissary

Status: partial Proposal 170 support

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

The expected bounded final status remains `partial Proposal 170 support` while
27 RouterInfo selectors lack bounded authoritative sources and missing tunnel
data planes remain explicit unsupported runtimes. The canonical 43-addition
matrix currently contains 39 available selectors, one protocol-permitted
neutral selector, and three unavailable selectors; the unavailable rows are
router news, banned peers, and transit bandwidth over 15 seconds.

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

There is no plaintext HTTP fallback.

## Authentication

`Authenticate` accepts `API` and `Password`, returns an opaque string `Token`
and numeric `API`, and protected requests put the token in `params.Token`.

```json
{
  "jsonrpc": "2.0",
  "method": "Authenticate",
  "params": {
    "API": 2,
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
    "API": 2
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
- expired token: `-32004` when applicable;
- missing API version: `-32005`;
- unsupported API version: `-32006`.

Tokens are cryptographically random, bounded, in-memory only, and invalidated
on process restart.

JSON-RPC notifications execute validation and side effects but suppress the
response. An explicit `id: null` remains a request ID rather than a
notification.

## Current retained implementation

Retained implementation includes:

- feature-gated HTTPS serving with bounded connections, bodies, and requests;
- standard authentication and JSON-RPC behavior;
- exact Proposal 170 method/selector/action/type parsers and literal fixtures;
- durable generation stores and atomic TunnelManager mutation;
- explicit unsupported backends for missing tunnel data planes;
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

- 39 available selectors;
- 1 protocol-permitted neutral selector;
- 3 unavailable selectors.

Unavailable selectors fail explicitly and are never substituted with zero,
false, empty, or semantically adjacent values.

## Missing tunnel data planes

This workstream does not implement missing HTTP, IRC, SOCKS-IRC, CONNECT,
Streamr, bidirectional, or other listener/destination/LeaseSet/traffic paths.
Their definitions may parse and persist, but start/restart must fail explicitly,
stop must remain safe, and no unsupported type may report running or allocate a
runtime resource.

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
