# M022 Closure — AddressBook Runtime Bridge and Canonical Source Reconciliation

Status: closed internally against pinned revision

Frozen implementation revision: the commit containing this closure record.

Implementation disposition: `plans/closure/i2pcontrol-proposal-170/022-implementation-disposition.md`

## Scope result

M022 is closed for its scoped implementation. Model A was selected: the running
router's `AddressBookManager` is the sole mutable/durable authority, and
I2PControl holds a purpose-specific `AddressBookHandle`. Canonical mutations,
compatibility actions, normal runtime lookup, restart reconstruction, and
one-time legacy migration now use that authority. RouterInfo subscription and
configuration selectors use the pinned object shape with truthful `path: null`
metadata where no path-backed source exists.

The final Proposal 170 43-selector matrix is intentionally not closed here; it
remains M025–M027 work. The subsystem roadmap therefore remains
`corrective pass required` while M023 and later handoffs execute.

## Acceptance evidence

- Runtime add/replace/delete and restart behavior: address-book owner tests.
- Invalid destination and hostname rejection: canonical AddressBook handler
  tests.
- Four-book isolation, exact subscription/config objects, and result envelopes:
  canonical wire and selector fixtures.
- Persistence failure rollback and migration collision rejection: focused owner
  and production-adapter tests.
- Production composition and fail-closed startup: `production_composition`
  suite; no fake or disconnected adapter is accepted at server startup.

## Verification record

The following package-scoped checks passed after the implementation changes:

```text
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo test -p emissary-cli --no-default-features --features i2pcontrol
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
cargo +nightly fmt --all -- --check   [only pre-existing tutorial formatting differs]
```

The feature-gated test command completed with 388 library tests, 431 binary
tests, and all integration suites passing. Focused production composition and
legacy migration tests also passed after the final test additions. The nightly
workspace format check reports only the pre-existing `examples/rust-tutorial`
match-arm formatting difference; all touched files were formatted explicitly,
and that unrelated baseline file was not changed.

## Dependency disposition

- M023 remains `ready`: its hard dependency M021 is closed and it does not
  require M022's address-book implementation.
- M024 remains `blocked` on M023.
- M025 remains `blocked` on M020, M022, M023, and M024.
- M026 remains `blocked` on M025.
- M027 remains `blocked` on M020–M026.

Only M023 is listed as dependency-ready after this closure. The AddressBook
disconnected-shadow finding is resolved for M022 and is retained in the
registry only as a resolved finding with M025's final source-matrix ownership
noted.

No remote CI, upstream contribution, or external write was performed.
