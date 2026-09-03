# M125 — M113 Capability and Crypto-Ownership Audit

Status: **closed** — audit complete; two misclassified server cells corrected, 19 M113 cells remain blocked, no implementation successor is ready

Closure: `plans/closure/i2pcontrol-proposal-170/125-closure.md`

Source implementation plan:

- `plans/implementation/i2pcontrol-proposal-170/113-server-presentation-address-routing-and-leaseset-residual-completion.md`

Source closure:

- `plans/closure/i2pcontrol-proposal-170/113-closure.md`

Repository baseline: `97083896f6170962a8c9610d056e8fc2dd57646d`

Class: read-only capability / security / ownership audit

## 1. Objective

Re-audit M113's 21 residual cells after M124 adopted Yosemite Y005. Freeze the
current Proposal/reference semantics, identify the actual owner for every
candidate capability and cryptographic boundary, and decide whether a future
implementation plan is dependency-ready.

This audit does not implement Proposal 170 behavior, change router/core code, or
create a private LeaseSet serializer.

## 2. Evidence reviewed

- Proposal 170, including its HTTP-client/server option grouping and the ten
  `EncryptLeaseSet` values:
  `https://i2p.net/en/proposals/170-i2pcontrol-expansion/`;
- I2CP router/client option ownership and LeaseSet fields:
  `https://i2p.net/en/docs/specs/i2cp-overview/`;
- Java configuration semantics for `AllowInternalSSL` and
  `enableUniqueLocal`:
  `https://www.i2p.net/en/docs/specs/configuration/`;
- encrypted LeaseSet cryptographic ownership and protocol requirements:
  `https://www.i2p.net/en/docs/specs/encryptedleaseset/`;
- Emissary M095/M105/M113 evidence and the exact Y005 consumer pin;
- Yosemite Y005 `59140a2277bf296928d2e8ce39a148182eeff044`, including
  `src/options.rs` and `src/proto/session.rs`.

## 3. Audit result

| Cell family | Result | Current disposition |
|---|---|---|
| `AllowInternalSSL` × `httpserver`, `httpbidirserver` | Proposal 170 places this under HTTP client filtering; the two server cells were a matrix classification error. | `not_applicable` |
| `UniqueLocalAddressPerClient` × `httpserver`, `httpbidirserver` | Requires per-remote local-address allocation/routing outside Emissary's literal-loopback target owner. | `blocked_primitive` |
| `MultiHoming` × `httpserver`, `httpbidirserver` | Requires a bounded host/interface presentation-routing owner not present in Emissary. | `blocked_primitive` |
| `EncryptLeaseSet` × five server families | Y005 transports canonical SAM/I2CP LeaseSet fields, but there is no complete Proposal-mode mapping, client-side LeaseSet crypto/key lifecycle owner, or interoperability proof. | `blocked_primitive` |
| `OptionalLookup` × five server families | No Proposal-defined wire semantics or Yosemite/session primitive exists. | `blocked_primitive` |
| `LeaseSetClientAuths` × five server families | Y005 transports validated DH/PSK entries, but Emissary has no request schema, bounded secret owner/persistence, generation-safe handoff, or end-to-end proof. | `blocked_primitive` |

The authoritative matrix is now `284 apply / 96 blocked_primitive /
460 not_applicable`. The two-cell correction is recorded in the M105 audit
summary as a post-M125 delta; historical M113/M124 closures remain unchanged.

## 4. Capability and crypto ownership freeze

Yosemite Y005 is an accepted API-to-SESSION-CREATE transport owner. It validates
and emits `i2cp.encryptLeaseSet`, `i2cp.leaseSetAuthType`,
`i2cp.leaseSetBlindedType`, `i2cp.leaseSetType`, the canonical persistent
LeaseSet key spellings, and the selected DH/PSK client namespace. This is
dependency reachability evidence only. It is not proof that Emissary can
construct, publish, query, rotate, or interoperate with an encrypted LeaseSet.

The remaining LeaseSet work cannot be registered as an implementation plan
because the Proposal exposes mode names without defining a complete mapping to
the typed SAM fields, `OptionalLookup` has no accepted primitive, and
`LeaseSetClientAuths` needs an Emissary-owned bounded secret/key lifecycle. The
distinct router-local `i2cp.leaseSetPrivKey` is also outside Y005's typed API.
The I2CP architecture leaves LeaseSet construction, signing, key management,
and router-side consumption as separate boundaries; no current Emissary owner
provides the required end-to-end contract.

The presentation/routing cells remain blocked where their semantics would
require TLS termination/trust, host-interface allocation, or request-sensitive
non-loopback routing. M093's literal-loopback and no-SSRF boundary is unchanged.

## 5. Future-plan disposition

No M113 successor implementation plan is dependency-ready. A future plan must
first freeze an accepted contract for the ten modes and `OptionalLookup`, name
the actual crypto/key owner, define the bounded secret store and generation
handoff, prove no-downgrade behavior, and include live/reference
interoperability evidence. M114 remains blocked by the 96 applicable residual
cells. No Yosemite plan is blocked on this audit.

## 6. Closure evidence required

The closure record records the cell-by-cell dispositions, source/code evidence,
matrix delta, verification outcomes, security review, and the unchanged
internal-only external-interaction boundary.
