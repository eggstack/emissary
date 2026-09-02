# M113 Closure — Server Presentation, Address-Routing, and LeaseSet Residual Completion

Status: **closed as blocked**

Date: 2026-09-02

Implementation commit: `82368ea` (no production code change; closure-only)

Plan: `plans/implementation/i2pcontrol-proposal-170/113-server-presentation-address-routing-and-leaseset-residual-completion.md`

## Outcome

M113 re-froze the 21 remaining server-side residual cells and retained all of
them as explicit `blocked_primitive` responses. No `apply` or
`not_applicable` reclassification was made. The exact M095 matrix remains
`312 apply / 70 blocked_primitive / 458 not_applicable` with SHA-256
`9fea6844e0b7e28959e1169491d100ce2f81124fff790f6c10882b765b41eea9`.

The 21 retained blocked cells are:

- `AllowInternalSSL` × `httpserver`, `httpbidirserver` (2): no bounded
  server-side TLS trust/termination owner in the supported Yosemite/I2PControl
  path;
- `UniqueLocalAddressPerClient` × `httpserver`, `httpbidirserver` (2): no safe
  per-client local-address allocator in the literal-loopback server target
  model;
- `MultiHoming` × `httpserver`, `httpbidirserver` (2): no bounded
  non-request-selected multihoming/routing owner in the existing HTTP server
  data plane;
- `EncryptLeaseSet` × `server`, `httpserver`, `httpbidirserver`, `ircserver`,
  `streamrserver` (5): no supported Yosemite/SAM LeaseSet confidentiality
  serializer or key-management path;
- `OptionalLookup` × the same five server families (5): no supported
  Yosemite/SAM LeaseSet lookup-policy serializer;
- `LeaseSetClientAuths` × the same five server families (5): no supported
  LeaseSet client-authorization key store or session handoff.

M111's four `UseSSL` cells and M112's 45 proxy/plugin/profile/reduction
cells remain outside this closure's scope and retain their existing blockers.

## Requirement and evidence matrix

| Work package | Result | Evidence |
|---|---|---|
| WP1 applicability | All 21 cells received an exact blocked disposition with pinned/reference semantics | `095-full-support-matrix.toml` cell notes; `105-residual-option-audit.toml`; Yosemite 8026f5b source audit below |
| WP2 `AllowInternalSSL` | Retained blocked | Proposal local TLS presentation vs Yosemite SAM-control `ssl`; no server TLS terminator; `backends/http_server.rs` rejects before allocation |
| WP3 `UniqueLocalAddressPerClient` / `MultiHoming` | Retained blocked | Loopback confinement M090/M093; no per-client address allocator or multihoming router; request-selected LAN routing prohibited |
| WP4 LeaseSet capability check | Retained blocked — no accepted primitive | Yosemite `SessionOptions::encrypt_lease_set`, `lease_set_auth_type`, `lease_set_*_key/secret`, `lease_set_blinded_type` declared but not emitted on `SESSION CREATE` wire (`proto/session.rs` only emits `i2cp.leaseSetEncType`, `i2cp.dontPublishLeaseSet`, tunnel lengths/quantities/variance/backup and `SIGNATURE_TYPE`) |
| WP5 LeaseSet secret ownership | No second store created; existing M110 store not extended | No bounded client-auth cargo; `LeaseSetClientAuths` rejected before allocation; no secret in RPC/log/RawConfig |
| WP6 Fail-closed session activation | Preserved — failed LeaseSet start cannot downgrade | `backends/server.rs` and `http_server.rs` fail before `build_session_options`/`run_accepted_server`; no fallback to public LeaseSet |
| WP7 Matrix truthfulness | No silent downgrade; no new subsystem for parity | `m095_full_support_matrix` test still passes; 312/70/458 counts unchanged |

### Semantic re-freeze detail

- `AllowInternalSSL`: Java I2PTunnel local HTTPS presentation between HTTP
  server/client and internal target. In Emissary this would require a bounded
  server-side TLS termination/trust owner. The accepted Yosemite
  `SessionOptions.ssl` controls SAM-control TLS, not Proposal presentation TLS
  (M111's `UseSSL` distinction applies identically here). No such owner exists;
  adding a TLS terminator solely for matrix parity is prohibited by M113 §4 and
  the M093 loopback/target invariant.

- `UniqueLocalAddressPerClient` / `MultiHoming`: Java exposes per-client local
  address selection and multihomed host mapping. Portable contract requires
  externally visible I2P behavior; Emissary's server target model is
  literal-loopback (`127.0.0.1`/`localhost`/`::1`) validated by
  `normalize_loopback_target`. Allocating arbitrary host interfaces or
  request-selected LAN addresses would weaken SSRF/loopback confinement and
  require a new host-network subsystem, both explicitly prohibited
  (`no router-global multihoming subsystem`).

- `EncryptLeaseSet` / `OptionalLookup` / `LeaseSetClientAuths`: Proposal
  encrypted/blinded LeaseSet modes (`disable`, `encrypted (aes)`, `blinded`,
  `encrypted (psk/dh)` variants, lookup-password variants) and per-client
  authorization entries. Yosemite 8026f5b declares `encrypt_lease_set`,
  `lease_set_auth_type` (0 = none, 1 = DH, 2 = PSK), `lease_set_blinded_type`,
  `lease_set_key`, `lease_set_private_key`, `lease_set_secret`,
  `lease_set_signing_private_key`, `lease_set_type` but serializes none of them
  on `SESSION CREATE` (see `yosemite/proto/session.rs::create_session`:
  only `i2cp.leaseSetEncType`, tunnel fields, and `additional_options` are
  emitted; `encrypt_lease_set` is absent). `EncType` (`leaseSetEncType`) is
  already applied separately via M097/M111; `EncryptLeaseSet` mode and the
  auth/secret/key handoff have no wire primitive, and no M110-reusable secret
  store was established for blinded/PSK/DH material. A private serializer or
  Yosemite fork would be required, which M113 §2 and ADR-0005 prohibit.

All three families therefore satisfy the plan's stop conditions and are
truthfully retained as `blocked_primitive` with exact primitive reasons.

## Security and containment review

- Literal-loopback confinement unchanged; non-loopback `TargetHost`/`Host` still
  rejected before allocation in `server.rs`, `http_server.rs`, `http_bidir.rs`,
  `irc_server.rs`; Streamr local UDP targets remain loopback-only.
- No request-selected LAN/clearnet routing, no direct-clearnet fallback, no
  Yosemite/LS serializer added, no core/util/Cargo dependency change.
- Trusted peer identity only from authenticated Yosemite stream/datagram
  identity; `LeaseSetClientAuths` secrets never appear in RPC, `rawConfig`,
  logs, debug, RouterInfo, or persistence.
- Server admission/rate/filter state remains transactional and bounded;
  HTTP/IRC spoof/framing/DCC/CTCP protections remain non-bypassable.
- Generation-local stop/restart/edit semantics preserved.
- No frontend/router-global multihoming subsystem, no TLS termination stack,
  no LeaseSet key-exchange stack created for parity.
- Feature-disabled/default behavior unchanged.

## Verification

The following checks passed during closure:

- `cargo check -p emissary-cli --no-default-features --features i2pcontrol`;
- `cargo check -p emissary-cli --no-default-features`;
- `cargo check` (workspace);
- `cargo test -p emissary-cli --no-default-features --features i2pcontrol --lib` — 639+ tests;
- `cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m095_full_support_matrix --test m105_residual_option_audit --test m062_dependency_containment --test m060_containment`;
- `cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings`;
- `git diff --check`.

`cargo fmt --all -- --check` was attempted; the stable formatter reports the
repository's pre-existing nightly-vs-stable rustfmt drift on unrelated files.
No broad rewrite was applied, consistent with prior closures (M111/M112).

No new I2PControl production path was introduced, so no new unit test was
required. Existing server HTTP/IRC/admission/secret-store suites remain green
and are authoritative for M113's blocked-before-allocation contract.

## Unresolved findings and future handoff

There are no new high- or medium-severity findings. M113 is closed as blocked,
not complete Proposal 170 support.

- Remaining blocked count stays `70 = 4 (M111 UseSSL) + 45 (M112) + 21 (M113)`.
  No future plan became applicable from this closure; the Yosemite Y003
  encrypted-LeaseSet dependency remains unsatisfied because the exact
  LeaseSet/auth primitive is still absent at `8026f5b`.
- M114 remains `proposed / blocked` until zero applicable residual cells and a
  successful live/reference interoperability reclosure. M113 did not alter
  RouterInfo (42/1/0), AddressBook, ClientServicesInfo, or `emissary-core`
  tunnel-pool variance/backup (M118) behavior.
- A future M113 retry would require: (a) a Yosemite or neutral Emissary
  LeaseSet primitive that is publicly emitted on `SESSION CREATE` and
  fail-closed, plus a bounded/validated owner-only client-auth secret store
  reused from M110; or (b) a separately accepted ADR/security decision that
  creates a bounded presentation/routing owner without weakening loopback/SSRF
  boundaries. Neither is authorized by this closure.

The registry, roadmap, matrix, audit, ledger, and documentation now record
this decision and the current `312 / 70 / 458` counts.

## Attestation

This closure is internal to `eggstack/emissary`. External Proposal 170,
reference-router, Yosemite, and documentation sources were read-only evidence.
No upstream issue, PR, review, release, or maintainer activity was performed.
