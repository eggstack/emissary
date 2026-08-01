# M027 Implementation Disposition — Exact Conformance and Independent Reclosure

Status: closed for implementation; M027 closure accepted; final disposition is
`partial Proposal 170 support`

Source implementation plan:

- `plans/implementation/i2pcontrol-proposal-170/027-proposal-170-conformance-and-reclosure.md`

Pinned authority:

- Proposal 170, `I2PControl Expansion`, Open, created and last updated
  `2026-05-20`: <https://i2p.net/en/proposals/170-i2pcontrol-expansion/>
- Existing I2PControl JSON-RPC/authentication contract:
  <https://i2p.net/en/docs/api/i2pcontrol/>

Reviewer:

- Codex internal closure reviewer, distinct from the `wr3n-ai` implementation
  author of the M026 source audit and prior corrective commits.
- The reviewer re-read the pinned external pages read-only, inspected
  production code and tests, and independently ran the required local checks.

## Disposition

M027 added the missing canonical literal fixture corpus and performed the final
claim/source/security/scope audit. No production feature or tunnel data plane
was added. The exact Proposal 170 wire surfaces and every claimed available
source/runtime/persistence dimension are supported by repository evidence.

The final status is intentionally partial because 26 of the 43 pinned
RouterInfo additions remain unavailable without a bounded authoritative owner.
Missing HTTP, IRC, SOCKS-IRC, CONNECT, Streamr, and bidirectional tunnel data
planes remain explicit unsupported runtime behavior under ADR-0001.

## M027 changes

- Added `emissary-cli/tests/m027_literal_fixtures.rs`, a separately named
  canonical fixture suite containing literal base/auth, JSON-RPC, AddressBook,
  TunnelManager, ClientServicesInfo, and RouterInfo examples.
- Added a literal 43-selector/type manifest and exact 16/1/26 source partition
  guard to the canonical fixture suite.
- Kept `emissary-cli/tests/golden_fixtures.rs` explicitly compatibility/parser
  evidence; it is not counted as canonical fixture evidence.
- Reconciled I2PControl support, conformance, method, security, source-map,
  implementation, roadmap, and registry documents.
- Added the M027 closure record.

## Requirement evidence

| Requirement | Evidence | Result |
|---|---|---|
| Literal canonical fixture corpus | `m027_literal_fixtures.rs` | pass |
| Existing I2PControl auth/protected RouterInfo | `server::handle_authenticate`, `authenticate_protected_request`; server auth tests and literal base fixture | pass |
| JSON-RPC notifications and IDs | `rpc::parse_request`, `server::handle_jsonrpc`; notification/ID tests | pass |
| Exact 43-key RouterInfo matrix | `rpc::router_info_keys::PROPOSAL_170_CONTRACT`; literal manifest and source-map tests | pass |
| Canonical AddressBook request/result shapes | `address_book::handle_canonical_address_book`; AddressBook handler tests and literal fixtures | pass |
| Canonical TunnelManager actions/types/get shape | `tunnel_manager::handle_tunnel_manager`, `tunnel_definition_to_get_result`; TunnelManager tests and literal fixtures | pass |
| ClientServicesInfo selectors/shapes | `client_services::handle_client_services_info`; selector/lifecycle/SAM tests and literal fixtures | pass |
| Unsupported tunnel lifecycle behavior | exhaustive backend registry and unsupported backend tests | pass; runtime unsupported by design |
| Source/runtime/persistence separation | M020–M026 dispositions, production adapters, source map, and M027 closure matrix | pass |
| Security and sanitized failure behavior | `tests/adversarial.rs`, `tests/static_guards.rs`, security tests, handler negative tests | pass |
| Restart/failure/contention/recovery behavior | generation-store, AddressBook, startup inventory, SAM, proxy-exit, and RouterInfo tests cited in closure | pass |

## Exact verification

The final CLI package check, test, and clippy commands passed. The complete
feature suite passed with 1,213 tests. The core checks passed because M024
touched the permitted SAM observation seam; core tests passed with 1,066 tests
and 2 ignored.

The stable workspace formatter check remains a pre-existing configuration
mismatch (`rustfmt.toml` uses nightly-only options) and reports unrelated
formatting differences across the workspace. The touched CLI package was
checked with nightly rustfmt and passed.

## Scope and external-interaction attestation

The corrected baseline was `a3c4f469f4877e5ff4a0bb4230da298f0b367ed2`.
All prior production changes classify as `i2pcontrol local`, `composition-only
seam`, `address-book owner seam`, `existing tunnel manager seam`, `SAM
observation seam`, or `bounded RouterInfo owner snapshot`; no unexpected
production scope was introduced by M027. M027 itself changes tests,
documentation, and planning records only.

External specifications were accessed read-only. No upstream or third-party
repository, issue, pull request, review, maintainer channel, submission,
adoption request, merge solicitation, or contribution artifact was created or
prepared. Repository writes remained within `eggstack/emissary`.

Frozen M027 implementation/test head: `ef5fadf`.
