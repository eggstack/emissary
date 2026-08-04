# I2PControl Proposal 170 Milestone M028 — Closure

Status: closed for implementation; M029 ready

## Scope and baseline

M028 reviewed and corrected the repository baseline
`03a384aec495232e64468dcf61d60dd2bab5cfe0`.

Implementation/test commit and frozen head:

- `a65eecb` — `fix: isolate I2PControl address book state`

M028 is a bounded corrective pass, not final Proposal 170 subsystem closure.
The subsystem remains `corrective pass required` until the independent M029
review selects the truthful final disposition. M029 is now `ready` because all
M028 activation conditions are satisfied.

## Closure decision

M028 is formally closed. The post-M027 planning regression is repaired in the
active registry/roadmap/support documents, M019 is historical superseded
evidence only, and the Proposal 170 AddressBook owner is isolated behind both
the compile-time feature and runtime configuration.

The final bounded expectation remains `partial Proposal 170 support`: the
retained RouterInfo matrix has 16 available, 1 neutral, and 26 unavailable
sources, and missing tunnel data planes remain explicit unsupported runtimes.

## Evidence summary

The implementation disposition at
`plans/closure/i2pcontrol-proposal-170/028-implementation-disposition.md`
contains the exact changed-file inventory, requirement matrix, verification
results, and failure/security/compatibility review. The decisive transition
evidence is:

- no-feature and runtime-disabled construction uses only legacy AddressBook
  sources and leaves pre-existing control-state bytes untouched;
- enabled construction creates exactly one control owner and supplies only a
  dedicated mutation handle to I2PControl;
- enabled state is immediately visible through ordinary lookup;
- disabled restart ignores but does not delete control state;
- re-enabled restart restores the retained current generation without a second
  authority;
- `serde_json` is optional and owned by the `i2pcontrol` feature.

The full feature suite passed with 1,219 tests; the no-feature suite passed with
54 tests. Strict package clippy passed in both modes and all touched Rust files
passed the configured nightly rustfmt check.

## Invariant and scope attestations

- Canonical Proposal 170 and existing I2PControl wire behavior was unchanged.
- The 16/1/26 RouterInfo source matrix was unchanged.
- Missing tunnel data planes remain explicit unsupported/resource-free stubs.
- No router, transport, NetDB, peer-selection, streaming, LeaseSet,
  cryptographic, frontend, resolver-policy, SAM, or core implementation was
  added.
- No schema migration, new dependency, generic framework, CI/release work,
  generated evidence bundle, or upstream activity was introduced.
- No high/medium M028 correctness or security finding remains unresolved.

## Future-plan unblocking

M029 was the only named downstream plan and was hard-blocked on a closed M028
with a frozen implementation/test head. It is therefore moved from `blocked`
to `ready` in the registry, roadmap, implementation README, and affected
support/conformance documentation. No other future plan is registered or can
be unblocked by this closure.

M029 must remain a distinct review by a different agent/run. It must inspect
the actual post-M028 repository head, refetch the pinned external contract
read-only, revalidate retained M020–M027 evidence, and choose the final
`partial Proposal 170 support` or other authorized disposition.

## Internal-only attestation

This closure records internal repository evidence only. No upstream or
third-party issue, review, pull request, submission, adoption request, merge
request, maintainer outreach, or contribution artifact was created or prepared.
