# Proposal 170 Support Status

Status: partial Proposal 170 support; M093 production/security reclosure and M094 planning reconciliation closed; M095 full-support matrix closed

Proposal 170 remains Open. This status is pinned to the `2026-05-20` revision.

Historical invalidation:

- `plans/closure/i2pcontrol-proposal-170/027-closure-invalidation.md`
- `plans/closure/i2pcontrol-proposal-170/039-closure-invalidation.md` (resolved by M040–M044)

Current roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-roadmap.md`
- tunnel-runtime completion: `plans/subsystems/i2pcontrol-proposal-170-tunnel-runtime-completion-roadmap.md`
- full-support completion: `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md`
- authoritative aggregate matrix: `plans/implementation/i2pcontrol-proposal-170/095-full-support-matrix.toml`

Tunnel-runtime reclosure:

- M072: `plans/closure/i2pcontrol-proposal-170/072-closure.md` — accepted after
  the generic option corrective;
- M073: `plans/closure/i2pcontrol-proposal-170/073-closure.md` — closed;
- M074: `plans/closure/i2pcontrol-proposal-170/074-closure.md` — closed;
- M075: `plans/closure/i2pcontrol-proposal-170/075-closure.md` — closed;
- M076: `plans/closure/i2pcontrol-proposal-170/076-closure.md` — closed;
- M077: `plans/closure/i2pcontrol-proposal-170/077-closure.md` — implementation
  and closure present; merged-head integration reconciled by M084;
- M078: `plans/closure/i2pcontrol-proposal-170/078-closure.md` — implementation
  and closure present; merged-head containment bookkeeping reconciled by M084;
- M079: `plans/closure/i2pcontrol-proposal-170/079-closure.md` — historical
  closure of the older M077/M078 branch lineage; not current-head certification
  and superseded by M085 for the merged head;
- M084: `plans/closure/i2pcontrol-proposal-170/084-closure.md` — closed;
- M085: `plans/closure/i2pcontrol-proposal-170/085-closure.md` — closed;
  current-head final reclosure authority.
- M086: `plans/closure/i2pcontrol-proposal-170/086-closure.md` — closed;
  documentation/evidence reconciliation only; it does not reopen M085.

Closed handoffs:

- M028 closed for implementation: `plans/implementation/i2pcontrol-proposal-170/028-post-m027-status-and-addressbook-feature-isolation.md`
- M029 closed: `plans/implementation/i2pcontrol-proposal-170/029-in-scope-conformance-reclosure.md`; closure:
  `plans/closure/i2pcontrol-proposal-170/029-closure.md`
- M030 closed: `plans/implementation/i2pcontrol-proposal-170/030-addressbook-destination-owner-coherence.md`; closure:
  `plans/closure/i2pcontrol-proposal-170/030-closure.md`
- M031 closed: `plans/implementation/i2pcontrol-proposal-170/031-client-tunnel-runtime-backend.md`; closure:
  `plans/closure/i2pcontrol-proposal-170/031-closure.md`
- M032 closed: `plans/implementation/i2pcontrol-proposal-170/032-server-tunnel-runtime-backend.md`; closure:
  `plans/closure/i2pcontrol-proposal-170/032-closure.md`
- M033 closed: `plans/implementation/i2pcontrol-proposal-170/033-tunnel-lifecycle-reconciliation.md`; closure:
  `plans/closure/i2pcontrol-proposal-170/033-closure.md`
- M034 closed: `plans/implementation/i2pcontrol-proposal-170/034-addressbook-setter-truthfulness.md`; closure:
  `plans/closure/i2pcontrol-proposal-170/034-closure.md`; disposition:
  `plans/closure/i2pcontrol-proposal-170/034-implementation-disposition.md`
- M035 closed: `plans/implementation/i2pcontrol-proposal-170/035-base-compatibility-and-selector-overlap.md`; closure:
  `plans/closure/i2pcontrol-proposal-170/035-closure.md`; disposition:
  `plans/closure/i2pcontrol-proposal-170/035-implementation-disposition.md`
- M036 closed: `plans/implementation/i2pcontrol-proposal-170/036-auth-and-publication-hardening.md`; closure:
  `plans/closure/i2pcontrol-proposal-170/036-closure.md`
- M037 closed: `plans/implementation/i2pcontrol-proposal-170/037-containment-boundary-reduction.md`; closure:
  `plans/closure/i2pcontrol-proposal-170/037-closure.md`
- M038 closed: `plans/implementation/i2pcontrol-proposal-170/038-live-runtime-interoperability.md`; closure:
  `plans/closure/i2pcontrol-proposal-170/038-closure.md`; disposition:
  `plans/closure/i2pcontrol-proposal-170/038-implementation-disposition.md`
- M039 closed: `plans/implementation/i2pcontrol-proposal-170/039-operational-reclosure.md`; closure:
  `plans/closure/i2pcontrol-proposal-170/039-closure.md`
- M039's final disposition was invalidated by
  `plans/closure/i2pcontrol-proposal-170/039-closure-invalidation.md` and
  corrected by M040–M044:
  - M040: `plans/closure/i2pcontrol-proposal-170/040-closure.md`
  - M041: `plans/closure/i2pcontrol-proposal-170/041-closure.md`
  - M042: `plans/closure/i2pcontrol-proposal-170/042-closure.md`
  - M043: `plans/closure/i2pcontrol-proposal-170/043-closure.md`
  - M044: `plans/closure/i2pcontrol-proposal-170/044-closure.md`

M019 is superseded and non-controlling. M020–M027 remain retained corrective
evidence, while M027's final disposition is historical invalidated evidence.
M028's implementation disposition and closure record contain the
feature-isolation correction evidence; M030's implementation disposition and
final-head closure are now controlling for AddressBook destination coherence.

## Status model

Support is reported separately as:

| Dimension | Meaning |
|---|---|
| Wire | exact public request/response names, casing, presence semantics, and JSON types |
| Source | truthful current Emissary source exists |
| Runtime | a real backend performs the requested operation |
| Persistence | process-crash atomic/recoverable mutation; qualified power-loss durability |
| Feature isolation | disabled/default execution is unaffected by the administrative feature |
| Evidence | literal external-contract, failure, restart, composition, and transition proof exists |

Parser acceptance, compatibility aliases, stored administrative definitions,
unavailable sources, and unsupported runtime stubs are not full operational
implementation.

## Current overall disposition

The repository remains partial Proposal 170 support. M044 reviewed the earlier
corrected final head and accepted that source/method disposition; the later
tunnel-runtime security sequence added bounded admission, generic accepted
server relay, HTTP anonymity, IRC lifetime, and Streamr local-boundary
correctives. M085 independently audited the actual post-M084 merged head and
closed the tunnel runtime/security line with no high or medium finding
remaining; the unrelated RouterInfo source limitations remain.
M039 remains a historical invalidated closure.

M028 owns the status/feature-boundary correction. M030 owns the destination
authority, lookup precedence, bounded import/repair, and independent final-head
review. M034 owns live subscription replacement, bounded refresh control, and
truthful configuration rejection. M035 owns the base method inventory and the
mode-specific RouterInfo compatibility boundary. M038 owns production-composition
and live child-process interoperability evidence. M040–M043 own the corrective
implementation and regression evidence; M044 owns the accepted independent
final-head review. M053 owns the live ProfileStorage correction for M045
known-peer sources. M054 owns the transit-15s truthfulness correction for M049.
M055 owns the network-error truthfulness correction for M050. M056 owns the
accepted integrated reclosure and final 43-row source audit.

Expected final disposition under the authorized scope remains
`partial Proposal 170 support` because five of the 43 RouterInfo additions lack
bounded authoritative sources. M066 adds real filtered `ircclient` and
`ircserver`, M067 adds filtered accepted-stream `httpserver`, M068 adds real
`httpclient` and `connectclient`, M069 adds bounded `socks` and filtered
`socksirc`, and M070 adds the deprecated composed `httpbidirserver`; the
unavailable rows are router news, banned peers, transit-15s, and v4/v6
network-error.

## M038 live-runtime evidence

The live evidence is layered rather than presented as a universal network
certification:

- unit and fixture evidence remains in the retained M020–M037 suites;
- production-composition evidence launches the feature-enabled `emissary-cli`
  binary with its real TLS/authentication, AddressBook, RouterInfo,
  ClientServicesInfo, and TunnelManager owners;
- the child-process scenario is
  `emissary-cli/tests/i2pcontrol_live_runtime.rs` and exercises restart,
  persistence, bind failure/recovery, unsupported types, startup ownership,
  malformed requests, and bounded cleanup;
- local I2P data-plane formation is not claimed: this loopback run has no
  reseeded peer set, so client/server traffic and public server destination
  identity stability remain explicitly blocked at the formation boundary;
- subscription replacement is reported as unavailable when no HTTP downloader
  is composed, matching the existing documented `-32603` failure rather than
  claiming a successful refresh.

The run uses temporary loopback state, process-local credentials, generated
runtime-only destination material, and temporary service TLS material. No
secrets or generated state are committed.

## Base method and RouterInfo compatibility

The exact method support inventory is maintained in
`rpc.rs::methods::SUPPORT_INVENTORY`. Implemented base methods are
`Authenticate` and `RouterInfo`; `AddressBook`, `TunnelManager`, and
`ClientServicesInfo` are Proposal 170 methods; and `SetSubscriptions` and
`SetConfig` remain shipped compatibility aliases. `GetKeys`, `GetRate`,
`RouterManager`, `NetworkSetting`, and `AdvancedSettings` return standard
`METHOD_NOT_FOUND` errors and are not claimed as implemented.

RouterInfo direct requests use Proposal 170 presence/source semantics. The
historical nested `Selector` form accepts only base selectors, uses truthy
boolean selection, and retains legacy serializers. The three exact overlaps
(`i2p.router.news`, `i2p.router.addressbook.subscriptions`, and
`i2p.router.addressbook.config`) are explicitly table-driven and tested.

## Retained implementation

Retained candidate evidence includes:

- HTTPS I2PControl service with bounded request and connection handling;
- standard I2PControl authentication, token placement, error codes, JSON-RPC
  notification execution, and strict request IDs;
- exact Proposal 170 direct method/selector/action/type parsing;
- literal external-contract fixtures;
- typed twelve-tunnel administrative inventory and exhaustive backend registry;
- exact TunnelManager result shapes, validation, atomic persistence, and secret
  handling;
- operational control-plane-owned generic `client` and `server` runtime
  backends with startup ownership isolation; server identities are persistent,
  fixed-path, and redacted;
- startup-managed tunnel inventory and proxy lifecycle observation;
- bounded recoverable SAM observation;
- exact 43-key RouterInfo matrix and explicit unavailable behavior;
- enabled-mode runtime AddressBook authority.
- M065 bounded I2PControl-owned client-listener and accepted-server runtime seams;
- M065 deterministic backend option-capability validation with secret-safe errors;
- M066 bounded IRC parsing/filtering, filtered `ircclient`, and registration-filtered
  `ircserver` runtimes with trusted peer-derived registration identity;
- M067 bounded HTTP request normalization and response filtering for `httpserver`.
- M068 bounded HTTP client and strict CONNECT client proxies with I2P-only
  target routing, explicit outproxy handling, proxy authentication, and
  generation-safe local listener lifecycle.

M028 must not reimplement or broaden these areas.

## Base I2PControl and JSON-RPC

Retained status: method-level implementation complete in M020.

Behavior includes:

- `Authenticate` with `API` and `Password`;
- numeric `API` response and opaque `Token`;
- standard `params.Token` protected requests;
- compatibility-only header token with conflict rejection;
- distinct I2PControl authentication/version errors;
- notification execution with response suppression;
- explicit-null request IDs and strict invalid-ID rejection;
- direct base RouterInfo compatibility.

M029 reran the focused evidence after M028; the exact command outcomes are
recorded in `plans/closure/i2pcontrol-proposal-170/029-closure.md`.

## TunnelManager

Retained status: wire/persistence correction complete in M021, startup/source
correction retained from M023, and the generic control-plane `client` and
`server` runtime backends are operational from M031/M032.

Retained behavior:

- seven lowercase canonical actions;
- twelve exact tunnel types;
- exact `status`, `results`, `info`, and nested `rawConfig` shapes;
- strict action/option/type/range validation;
- one-publication create/edit/rename/delete;
- prior-state preservation on publication failure;
- restrictive permissions and temporary cleanup where supported;
- secret-safe persistence and response serialization;
- startup-owned name collision and mutation rejection;
- deterministic resource-free unsupported lifecycle behavior.

### Streamr tunnel runtime boundary

The generic `client` and `server` types are the real control-plane lifecycle
backends at this stage. They reuse the existing Yosemite streaming data planes
behind I2PControl-owned, per-name supervisors. Startup-managed client and
server definitions remain externally managed and reject administrative
lifecycle operations.

`streamrserver` owns a persistent repliable-datagram identity and a
loopback-only local UDP source. `streamrclient` refreshes a bounded subscription
every 15 seconds and forwards payloads only to its configured loopback UDP
target. The producer caps subscribers at 10, expires entries after 60 seconds,
and caps payloads at 1200 bytes; destination text is bounded at 524 bytes.
Control packets are exactly one byte (`0` subscribe/refresh, `1` unsubscribe);
malformed or unknown controls do not create state. Non-loopback local-address
configuration is rejected before allocation, and unexpected non-loopback local
UDP sources are ignored. Yosemite exposes trusted peer destination identity but
not inbound port metadata, so Emissary uses the trusted destination plus the
fixed configured session port tuple and makes no core API change.

### SOCKS and SOCKS-IRC runtime boundary

`socks` accepts bounded SOCKS4a and SOCKS5 TCP CONNECT negotiation. SOCKS4
literal IPv4, SOCKS5 literal IPv4/IPv6, BIND, UDP ASSOCIATE, arbitrary DNS
resolution, localhost/private targets, and unsupported options fail closed
before a Yosemite connection is opened. I2P domain targets resolve only through
the approved I2P/address-book path. Clearnet targets require one explicitly
configured I2P-hosted SOCKS5 outproxy; Emissary never opens a local clearnet
socket. Loopback is the safe default listener, and non-loopback exposure
requires configured username/password authentication. After establishment,
`socks` intentionally relays arbitrary application bytes, so SOCKS alone does
not provide application-layer anonymity.

`socksirc` uses the exact same negotiation, target routing, lifecycle, and
authentication path, then enters the M066 stateful IRC filter in both
directions. It has no raw relay alternative; unsupported CTCP and DCC remain
blocked exactly as they are for `ircclient`.

### HTTP server runtime boundary

`httpserver` is operational only through the I2PControl-owned accepted-stream
runtime. It reads and bounds the request line/header block before opening the
loopback target, rejects ambiguous Content-Length/Transfer-Encoding framing,
obs-fold, upgrades, `Expect` headers (single, duplicate, mixed-case
`100-Continue`, or unknown expectation tokens), proxy identity/privacy
headers, and spoofed `X-I2P-*` headers, then injects
peer identity derived from the accepted SAM stream. Configured Host rewriting,
access lists, proxy/referer/User-Agent policy, peer-aware admission (30 global
connections by default, 8 per peer, and bounded peer/aggregate minute/hour/day
rates), and peer-keyed POST/PUT/PATCH throttling are applied before local
connection. The inbound half of `httpbidirserver` consumes the same policy.
Local response headers are parsed and identifying server/proxy/provider/cache/
trace headers are removed before the bounded response body is streamed back;
validated Content-Length and chunked framing remain intact, while application
headers are preserved. Trusted peer identity injection is bounded to the
structurally validated I2P Destination representation. TLS, compression, custom
options, arbitrary target hosts, and unsupported Proposal 170 modes reject
before destination/session allocation.

`TrustedPeerIdentity` is structurally validated at the accepted-stream
boundary. Trusted peer text is bounded to
`MAX_TRUSTED_DESTINATION_B64_TEXT` (1024) before decoding, Base64-decoded once,
and parsed with `emissary_core::primitives::Destination::parse_frame`. The
parser remainder must be empty or the identity is rejected. The 32-byte
accounting ID is derived from `parsed.id()`, and protocol handlers receive the
canonical full-Destination text produced by Base64-encoding
`parsed.serialize()` rather than attacker-selected input text. `Expect`
rejections emit a fixed `417 Expectation Failed` response with
`Connection: close` and no local target connection, so a client that waits for
a `100 Continue` cannot pin a handler until body timeout.

### IRC tunnel runtime boundary

`ircclient` and `ircserver` are operational only through the I2PControl-owned
filtered paths. IRC lines are bounded and parsed without assuming UTF-8;
client `USER` hostnames, PING/PONG tokens, PART reasons, and QUIT reasons are
sanitized. Ordinary PRIVMSG/NOTICE and CTCP ACTION are supported. Unsupported
CTCP, including DCC CHAT/SEND/RESUME/ACCEPT, is dropped; no auxiliary DCC
listener or session is created. WEBIRC and configurable cloak options are
rejected before allocation. IRC automation fields (`ircServer`, `ircPort`,
`ircNick`, `ircPassword`, and `ircChannels`) are likewise rejected because
M066 does not synthesize registration or channel behavior.

`ircserver` accepts only bounded registration material, rejects obvious
HTTP/Binary protocol probes, rewrites `USER` using the trusted accepted I2P
peer identity, and connects only to loopback. It does not forward registration
to the local IRCd until NICK and sanitized USER have been accepted. The
post-registration path is a raw IRC stream as specified by M066. Its local IRCd
connect is bounded to five seconds, and the accepted stream expires after ten
minutes of inactivity; successful traffic in either direction resets that
deadline, so active IRC sessions are not capped by total lifetime. Accepted
streams use the same bounded peer-aware admission policy as `httpserver`, and
the client side remains filtered for the future `socksirc` composition.

## AddressBook

Retained enabled-mode status: M022 established one runtime/durable authority for
private, local, router, and published books and normal lookup publication. M034
replaces the former inert subscription/configuration setter behavior.

M034 additionally proves:

- `SetSubscriptions` reaches the active downloader through one bounded typed
  command seam and publishes complete generations durably;
- restart restores the last accepted source set;
- pre-commit unavailability preserves the prior generation, while post-commit
  refresh-worker unavailability cannot turn the completed mutation into an
  error response;
- URL/count/aggregate bounds are enforced before mutation;
- every pinned `SetConfig` key has an explicit path/unsupported disposition;
- non-empty configuration requests never persist or report success;
- disabled/default execution still does not construct or consult the control
  command seam.

TunnelManager lifecycle reconciliation is operational for all twelve
control-plane tunnel families. `StartOnLoad` is honored only for persisted
control-plane definitions after durable state loads; startup-managed definitions
remain externally managed.

M028-corrected defect:

- the control owner had not been isolated from no-feature and runtime-disabled
  execution;
- normal startup could read retained control state and rebuild legacy lookup
  from it;
- normal downloads could update control state even when no I2PControl service
  was active.

M028 result:

- no-feature and runtime-disabled execution use legacy address files only and
  do not touch control state;
- enabled execution constructs one control owner and preserves M022 behavior;
- disabling preserves but ignores control-state files;
- re-enabling restores them;
- no second authority or schema migration is introduced;
- `serde_json` returned to feature ownership because no independent
  unconditional consumer requires it.

## ClientServicesInfo

Retained behavior:

| Selector | Retained source/runtime behavior |
|---|---|
| `I2PTunnel` | bounded startup/control-plane inventory with actual destination provenance |
| `HTTPProxy` | actual listener state and inactive publication on task exit |
| `SOCKS` | actual listener state and inactive publication on task exit |
| `SAM` | bounded active-session source with incomplete/recovery semantics |
| `BOB` | exact `false` |
| `I2CP` | actual listener state |

M028 does not alter these sources. M029 revalidated them.

## RouterInfo

Current source matrix:

- 37 available;
- 1 protocol-permitted neutral;
- 5 unavailable.

Available selectors have bounded current owners. Clock skew uses `null` only
when the protocol permits it. Unavailable selectors—including both network-error
selectors, for which Emissary has no canonical error owner—fail with sanitized
errors before assembly and never return fabricated zero, false, empty, partial,
or semantically adjacent values.

M026 found no additional in-scope authoritative source. M028/M029 do not repeat
that audit or authorize new telemetry/core inspection.

## Persistence and security

Retained strengths:

- versioned complete generations;
- deterministic serialization;
- write/sync/rename publication;
- prior-generation fallback;
- bounded retention and response size;
- authentication before protected work;
- request and collection bounds;
- redacted diagnostics and response filtering;
- explicit resource-free unsupported backends.

M028 additionally proved that disabled/default AddressBook execution cannot be
influenced by stale, corrupt, or attacker-planted Proposal 170 control state.

## Corrective sequence

| Milestone | Status | Scope |
|---|---|---|
| M020–M027 | retained evidence | base/wire/persistence/source corrections and literal review |
| M028 | closed for implementation | post-M027 status repair and AddressBook compile/runtime feature isolation |
| M029 | historical invalidated closure | retained non-AddressBook evidence |
| M030 | closed; partial Proposal 170 support | AddressBook destination/lookup coherence and final-head review |
| M034 | closed | AddressBook setter truthfulness and runtime subscription control |
| M040 | closed | startup server cancellation-owner correction |
| M041 | closed | source-IP throttle identity and atomic reservation |
| M042 | closed | AddressBook post-commit refresh boundary |
| M043 | closed | corrective runtime regression validation |
| M044 | closed; partial Proposal 170 support | corrected final-head independent reclosure |
| M053 | closed | live ProfileStorage correction for M045 known-peer sources |
| M054 | closed | transit-15s truthfulness correction; explicit unavailable disposition |
| M055 | closed | network-error truthfulness correction; explicit unavailable dispositions |
| M056 | closed | integrated reclosure; final 37 available / 1 neutral / 5 unavailable audit |
| M066–M071 | closed | real IRC, HTTP, CONNECT, SOCKS, bidirectional HTTP, and Streamr tunnel families |
| M072 | closed after M073 | integrated twelve-type runtime reclosure |
| M073 | closed; corrective history | generic client/server option apply-or-reject corrective; M081 closes the M075-accepted-stream regression that re-introduced the accepted-but-ignored `leaseSetEncType` |
| M074 | closed; corrective history | shared peer-aware server admission and rate-limit hardening; M080 closes the discovered transactional/cardinality defects |
| M080 | closed; corrective history | server admission transactionality and cardinality corrective; canonical cryptographic peer identity; bounded expiry index; remaining current-head capacity/expiry defects closed by M083 |
| M075 | closed | generic server accepted-stream raw relay hardening |
| M076 | closed | HTTP anonymity/POST-throttle hardening |
| M081 | closed | generic server `leaseSetEncType` apply-or-reject corrective; accepted-stream `SESSION CREATE` now carries the validated value |
| M082 | closed; corrective history | HTTP peer identity, `Expect` rejection, and POST cryptographic peer-key corrective; inherited trusted-Destination exactness closed by M083 |
| M083 | closed | admission capacity semantics, inactive-peer expiry-index invariant, and exact/canonical trusted Destination corrective |
| M077 | implementation/closure present; merged-head integration reconciled by M084 | IRC lifetime and exhaustion hardening; consumes the now-closed shared admission/trusted-peer boundary |
| M078 | implementation/closure present; merged-head containment bookkeeping reconciled by M084 | Streamr loopback-only local-boundary hardening |
| M079 | historical closure only; current-head certification superseded by M085 | integrated tunnel-security reclosure before the later M083 merge |
| M084 | closed | merged-head integration/planning corrective (test fixture, M062 bookkeeping, status reconciliation); see `plans/closure/i2pcontrol-proposal-170/084-closure.md` |
| M085 | closed | independently audited actual post-M084 merged head; tunnel runtime/security line complete against the pinned Proposal 170 revision and current internal fork head; see `plans/closure/i2pcontrol-proposal-170/085-closure.md` |
| M086 | closed | documentation/evidence reconciliation only; trusted-peer documentation and closure errata corrected without reopening runtime/security; see `plans/closure/i2pcontrol-proposal-170/086-closure.md` |

## Final-status rule

M030 selected:

- `partial Proposal 170 support` when all implemented/claimed dimensions pass
  but one or more sources/runtimes remain unavailable;
- `closed internally against pinned revision` only if every source/runtime
  dimension is actually available and evidenced;
- `corrective pass required` for unresolved high/medium defects;
- `blocked` when the proposal changed or required evidence cannot be obtained.

Under the current scope, `partial Proposal 170 support` is the expected honest
result. Explicit errors and unsupported stubs are not full operational support.

## Internal-only boundary

All work is internal to `eggstack/emissary`.

No plan authorizes upstream issues, pull requests, reviews, discussions,
submissions, patches, maintainer outreach, contribution preparation, adoption
requests, or merge activity. External specifications and reference sources may
be inspected read-only solely for internal correctness.
