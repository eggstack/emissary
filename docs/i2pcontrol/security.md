# I2PControl Security

Status: M036 authentication and publication hardening implemented; M021/M030/M032
requirements retained

This document describes the security properties and considerations for the I2PControl administrative state in Emissary.

## Authentication

I2PControl uses JSON-RPC authentication with opaque tokens:

- Passwords use the reviewed `subtle` constant-time primitive with fixed-size
  padding and explicit length handling
- Tokens are cryptographically random (32 bytes, hex-encoded)
- Tokens are stored in-memory only; no persistence
- Maximum concurrent tokens bounded at 1024
- Failed authentication is throttled per accepted TCP peer with a fixed-capacity
  monotonic-window table and bounded delay; successful authentication clears
  that peer's failure state
- Credentials are never logged or included in Debug output

See [README.md](README.md) for authentication details.

## Secret handling

### Redacted values

Sensitive tunnel options (passwords, keys) use the `OptionRedacted` wrapper:

```rust
pub struct OptionRedacted(Option<String>);
```

- `Debug` output shows `OptionRedacted(***)` instead of the value
- `Display` output shows `***` instead of the value
- The actual value is stored only where persistence is required for a future
  backend and is never logged or returned by TunnelManager responses

### Affected fields

The following `TunnelOptions` fields are redacted:

- `ssl_key` - SSL private key path
- `proxy_password` - SOCKS/HTTP proxy password
- `irc_password` - IRC server password

Canonical `ProxyPassword`, `OutproxyPassword`, and `LeaseSetClientAuths` input
is not duplicated into response-facing `rawConfig`; response serializers filter
all sensitive keys. Canonical `PrivKeyFile` is rejected as generic key-material
ingress. Errors contain field names at most, never secret values.

### Logging policy

- Complete tunnel definitions are never logged wholesale
- Individual option values are logged only at debug level with redaction
- Error messages from backends contain tunnel type but not secrets

Generic server destination private material is held only by the backend-owned
`ServerDestinationStore`. Its `StoredDestination` wrapper redacts Debug and
Display output, the server runtime configuration is intentionally not
Debuggable, and setup/forward errors are sanitized. Public destinations are
distinct from private session material and are published only after a real
Yosemite session exists.

## File system security

### Path confinement

The generation store enforces path confinement:

- Store directories must be real directories (not symlinks)
- All resolved paths must remain within the configured base path
- Symlinks in the generation directory are rejected during load
- User-provided identifiers (tunnel names) are used as BTreeMap keys, never as filesystem paths
- Server destination state uses the fixed `server-destinations/` directory;
  request values never select a path or filename

### File permissions

On Unix systems, generation files are created with restrictive permissions:

- Mode `0o600` (owner read/write only)
- Applies to both temporary and final generation files
- Non-Unix platforms rely on OS file permissions

### Atomic publication

State updates use atomic rename to prevent corruption:

1. Write to a temporary file (`.tmp-gen-NNNNNN.json`)
2. Flush and sync the file
3. Rename to the final path (`gen-NNNNNN.json`)
4. Sync the containing directory where supported
5. Update the in-memory snapshot only after the selected publication point

This establishes process-crash atomicity and prior-generation recovery. Where
directory synchronization is available and succeeds, it also establishes the
documented power-loss durability point. Platforms without an equivalent API
retain the atomicity/recovery guarantee but are not described as power-loss
durable.

If the process crashes during publication:
- Temporary files are detected and skipped on next load
- The most recent valid generation is loaded
- Corrupt generations fall back to prior valid ones

Permission-setting failure is fatal on Unix rather than best effort. Failed
publication removes its temporary file where possible and does not update the
in-memory snapshot.

The server destination store publishes bounded `current.json` and
`backup.json` files. It writes and syncs a temporary file, applies owner-only
permissions where supported, atomically rotates current to backup, publishes
the new current state, and syncs the containing directory where supported.
Corrupt current state falls back to a valid backup; corrupt state files and
irregular/symlink files fail closed.

### Symlink rejection

Symlinks in the generation directory are:
- Detected during directory scanning
- Rejected with a warning log message
- Never followed or loaded

## State integrity

### Corruption detection

Each generation file includes:
- Schema identifier (`emissary-i2pcontrol`)
- Schema version (currently 1)
- Revision number

Files with unknown schema or version are rejected.

### Corruption recovery

On load, the store:
1. Scans generation files newest-first
2. Tries each file
3. Falls back to the previous valid generation on failure
4. Returns an actionable error if all generations are corrupt

A diagnostic is emitted for each failed generation without exposing payload secrets.

### Oversized state rejection

Each store has a configurable maximum size limit. State that exceeds this limit is rejected before any files are written.

## Startup safety

### Bounded automatic task launch

- `StartOnLoad` is stored as durable intent in tunnel definitions
- Post-load reconciliation launches tasks only for control-plane-owned generic
  `client` and `server` definitions
- Unsupported and startup-managed definitions are never launched or adopted
- Each eligible definition is started independently, with bounded inventory and
  sanitized failure handling
- Runtime inspection comes from the supervisor/backend, never from persisted
  intent alone

### Runtime resolver owner coherence

When I2PControl is disabled, administrative state remains independent from the
runtime resolver and existing runtime files are untouched. When enabled, the
single runtime AddressBook owner is intentionally authoritative for lookup:
- Private, local, router, and published books remain stored separately
- Base32 and Base64 lookup use the owner and never fall through to stale legacy files
- Published entries are structurally validated full destinations
- Legacy destination import/repair is bounded, filename-confined, and symlink-safe

### Configuration isolation

Proposal 170 state is stored separately from:
- `router.toml` - Router configuration
- Runtime address book (`<base>/addressbook/`)
- Server private key paths
- Frontend configuration files

AddressBook setter isolation is stricter:

- `SetSubscriptions` accepts only bounded HTTP/HTTPS URLs and sends one typed
  replacement command to the active downloader;
- the command channel has capacity one, with at most one in-flight refresh and
  one newest pending generation;
- durable state is committed by the runtime manager before success is returned;
- unavailable manager/channel state fails explicitly and cannot claim deferred
  success;
- all Proposal 170 `SetConfig` keys are classified, with arbitrary path keys
  rejected as invalid parameters and every other non-empty key rejected as
  unsupported;
- legacy configuration metadata is cleared during enabled migration and never
  controls filesystem, proxy, scheduler, publication, or logging behavior.

## Compilation features

### Feature gating

All I2PControl code is gated behind the `i2pcontrol` Cargo feature:

```bash
# Build without I2PControl (default)
cargo build -p emissary-cli

# Build with I2PControl
cargo build -p emissary-cli --no-default-features --features i2pcontrol
```

### Core crate isolation

The `emissary-core` crate contains:
- No I2PControl or administrative persistence dependencies
- No JSON-RPC handling
- No HTTP server code

This ensures the core router remains independent of administrative concerns.

## Known limitations

### Platform-specific durability

Files are synced before rename and containing directories are synced after
rename on Unix-like platforms that expose the directory through the standard
filesystem API. On platforms without equivalent directory synchronization,
success means process-crash atomicity and prior-generation recovery only; it is
not an unqualified power-loss durability claim.

### Platform-specific permissions

File permission enforcement is Unix-only. On other platforms:
- Permission bits are not set
- OS-level file permissions apply
- The same filesystem confinement rules still apply

### No encryption at rest

Tunnel definitions containing proxy passwords are stored as plaintext JSON files. Operators should ensure the state directory has appropriate file system permissions.

## Testing

### Security tests

The following security properties are verified by tests:

- `symlink_in_directory_is_rejected` - Symlinks in generation directory are skipped
- `generation_files_have_restrictive_permissions` - Files are created with 0o600 on Unix
- `validate_confined_path_rejects_escape` - Paths escaping base are rejected
- `validate_confined_path_accepts_within_base` - Paths within base are accepted
- `option_redacted_debug_redacts` - Debug output redacts secrets
- `option_redacted_display_redacts` - Display output redacts secrets

### Static guards

Compile-time guards ensure:
- All 12 tunnel types are registered in `ALL_TUNNEL_TYPES`
- All 8 tunnel actions are registered in `ALL_TUNNEL_ACTIONS`
- No tunnel type is accidentally omitted
- The complete Proposal 170 AddressBook configuration-key inventory has one
  explicit disposition.

### No-side-effect tests

Tests verify that unsupported backends:
- Do not call `tokio::spawn`
- Do not allocate listeners or sockets
- Report `Unsupported` state consistently
