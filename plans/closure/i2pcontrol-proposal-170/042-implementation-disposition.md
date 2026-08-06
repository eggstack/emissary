# M042 Implementation Disposition — AddressBook Subscription Commit Boundary

Status: implemented; closure accepted

Source plan:

- `plans/implementation/i2pcontrol-proposal-170/042-addressbook-subscription-commit-boundary.md`

Implementation/test head: `ef30155` — `fix: make subscription refresh post-commit`

## Finding and correction

The live command path now commits and activates a replacement subscription set
through the existing AddressBook owner before scheduling refresh. The durable
owner publication and active-set update are the mutation linearization point.
If the bounded refresh sender is closed afterward, the manager emits a
sanitized warning and still returns the committed mutation result. No scheduler,
second owner, arbitrary path, or synchronous download transaction was added.

## Verification

- `set_subscriptions_worker_failure_after_commit_returns_success` — pass;
- `set_subscriptions_queue_failure_preserves_prior_state` — pass;
- active replacement, concurrent replacement, and restart restoration tests — pass;
- AddressBook production adapter, golden fixtures, and live-runtime evidence — pass;
- no-feature CLI tests — pass, 56 tests;
- feature-enabled AddressBook and full package tests — pass;
- feature-enabled all-target clippy with `-D warnings` — pass;
- `git diff --check` — pass.

The production change is limited to `emissary-cli/src/address_book.rs`; the
documentation now states that refresh is bounded follow-up work. The later
`03907c3` commit only makes the test helper wait for manager readiness and is
classified under M043 test evidence.

## Internal-only attestation

No upstream repository, maintainer channel, review, merge, submission, or
contribution artifact was used or prepared.
