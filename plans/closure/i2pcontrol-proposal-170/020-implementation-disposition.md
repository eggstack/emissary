# M020 Implementation Disposition — Base I2PControl and JSON-RPC Interoperability

Status: closed for implementation; M020 closure accepted; M021 ready

Frozen implementation/test head: `71a8dc6`

Implementation commit: `71a8dc6` — `fix(i2pcontrol): restore base JSON-RPC interoperability`

Implementation plan:

- `plans/implementation/i2pcontrol-proposal-170/020-base-i2pcontrol-and-jsonrpc-interoperability.md`

Closure record:

- `plans/closure/i2pcontrol-proposal-170/020-closure.md`

## Scope disposition

M020 implemented the bounded common I2PControl and JSON-RPC interoperability surface:

- standard `Authenticate` parameters and numeric API response;
- canonical `params.Token` authentication with documented header compatibility;
- distinct authentication/API error inventory and sanitized failures;
- notification execution with response suppression;
- lossless valid request IDs and rejection of invalid JSON-RPC IDs;
- direct base RouterInfo selector compatibility after authentication metadata removal;
- focused fixtures, conformance inventory, adversarial coverage, and documentation.

No router lifecycle, tunnel data-plane, persistence, AddressBook, service inventory,
telemetry-source, dependency, CI, release, or upstream scope entered this disposition.

## Evidence disposition

The M020 closure record contains the requirement-to-evidence matrix, request/response
fixture table, invariant and failure review, compatibility and security review, and
unresolved-finding disposition. The frozen head passed:

```text
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo test -p emissary-cli --no-default-features --features i2pcontrol
cargo clippy -p emissary-cli --no-default-features --features i2pcontrol --all-targets -- -D warnings
cargo test -p emissary-cli --no-default-features --features i2pcontrol authenticate
cargo test -p emissary-cli --no-default-features --features i2pcontrol token
cargo test -p emissary-cli --no-default-features --features i2pcontrol notification
cargo test -p emissary-cli --no-default-features --features i2pcontrol request_id
cargo test -p emissary-cli --no-default-features --features i2pcontrol router_info
cargo +nightly fmt -- --check                 # from emissary-cli
git diff --check
```

Results were: package check passed; `1149` feature-gated package tests passed across
15 suites; clippy passed with `-D warnings`; focused filters passed with `10`, `22`,
`8`, `2`, and `91` tests respectively; crate-local nightly formatting passed; and
`git diff --check` passed. The root workspace formatter remains a known pre-existing
baseline limitation and is recorded in the closure record rather than widened into M020.

## Findings and successor disposition

The three M020 high-severity finding groups are resolved:

- standard authentication/token/error interoperability;
- notification and request-ID correctness;
- direct base RouterInfo compatibility after token removal.

Token expiry remains informational because the current in-memory service has no expiry
state; the named `-32004` inventory is retained without claiming unsupported behavior.

M020 is therefore closed for implementation. M021's hard dependency is satisfied and
M021 is the only dependency-ready handoff. M022 through M027 remain blocked by their
named hard dependencies. Proposal 170 and the subsystem remain open for the remaining
corrective milestones; this disposition does not close the subsystem.

## Internal-only compliance

All writes were limited to the internal `eggstack/emissary` repository. External protocol
references were read-only. No upstream issue, pull request, review request, submission
package, maintainer contact, or other upstream mutation was created or prepared.
