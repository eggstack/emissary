# M043 Implementation Disposition — Corrective Runtime Regression Validation

Status: implemented; closure accepted

Source plan:

- `plans/implementation/i2pcontrol-proposal-170/043-corrective-runtime-regression-validation.md`

Corrective evidence head: `342420e` — `test: widen addressbook readiness bound`

## 1. Evidence completed

M043 added no production behavior. It added only a test-only bounded readiness
wait so the existing concurrent AddressBook regression cannot race manager
startup. The combined head directly exercises the four paths omitted by the
invalidated M039 evidence:

- original startup `ServerTunnelManager`: HELLO, SESSION CREATE, destination
  observation, separate `STREAM FORWARD`, and post-readiness liveness;
- same-IP authentication across changing ports;
- atomic concurrent failure reservation;
- post-commit refresh-worker closure with successful mutation response.

## 2. Verification outcomes

| Command/group | Outcome |
|---|---|
| no-feature check, test, clippy | pass; 56 CLI tests |
| conformance manifest | pass; 58 tests |
| golden fixtures | pass; 44 tests |
| M027 literal fixtures | pass; 7 tests |
| production adapter | pass; 20 tests |
| production composition | pass; 8 tests |
| M033 lifecycle | pass; 3 tests |
| M037 containment | pass; 4 tests |
| live runtime with `--nocapture` | pass; 1 test, with no-peer/no-downloader qualifications retained |
| feature-enabled package suite | pass; no failures, including 454 library tests and all integration targets |
| feature-enabled all-target clippy | pass with `-D warnings` |
| core check and SAM tests | pass; 145 unit SAM-filtered tests and 4 SAM integration tests |
| core clippy | pass with `-D warnings` |
| `git diff --check` | pass |

The stable formatter still reports the repository's known nightly-only option
mismatch; no unrelated formatter changes are retained.

## 3. Containment and residual findings

The corrective production paths are limited to
`emissary-cli/src/tunnel/server.rs`,
`emissary-cli/src/i2pcontrol/auth.rs`,
`emissary-cli/src/i2pcontrol/server.rs`, and
`emissary-cli/src/address_book.rs`. Documentation changes are limited to the
affected security and AddressBook descriptions. No core, tunnel-family,
RouterInfo, frontend, workflow, release, or remote infrastructure change was
introduced by M040–M043.

No unresolved high or medium defect remains. M043 does not select the final
Proposal 170 status.

## 4. Internal-only attestation

Validation was local and internal-only. No upstream repository or maintainer
channel was mutated, and no contribution artifact was prepared.
