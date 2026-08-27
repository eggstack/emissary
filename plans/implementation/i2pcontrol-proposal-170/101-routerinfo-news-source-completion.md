# M101 — RouterInfo News Source Completion

Status: ready; dependency M095 closed

Source roadmap:

- `plans/subsystems/i2pcontrol-proposal-170-full-support-completion-roadmap.md`.

Canonical requirements:

- `plans/000-long-term-specification.md`;
- `plans/003-planning-process.md`;
- ADR-0004 full-support completion boundary;
- M051 historical blocked news/banned-peer source audit;
- M056 current RouterInfo 37/1/5 reclosure authority.

Planning baseline: `630a8fd1cd4e5943fcde0b5c16f5fc1e88b5d207` plus M095 closure when dependency-ready.

Pinned external contract: `i2p.router.news`, Proposal 170 revision `2026-05-20`, return type `String`.

Classification: capability / security / operations.

## 1. Objective

Implement a real, bounded RouterInfo news source owned by the optional I2PControl subsystem, replacing the current truthful unavailability of `i2p.router.news` without adding a router-core news subsystem or allowing untrusted remote content to become an unbounded parser/cache/logging path.

M051 correctly refused to fabricate news because Emissary had no authoritative source. ADR-0004 now authorizes a dedicated I2PControl-local source as part of full Proposal 170 completion.

## 2. Source semantics must be frozen first

M095 must establish from the pinned proposal/reference implementation:

- what `String` represents: raw news document, concatenated entries, rendered text/HTML, XML-derived content, or another exact form;
- canonical source location(s) and transport expectations;
- authenticity/signature requirements, if any;
- refresh cadence and cache behavior expected by the adopted implementation;
- whether a missing/stale source is an operation error, empty news set, retained prior generation, or another specified behavior.

M101 must not choose a convenient public web page if that differs from the reference router-news semantics.

If M095 cannot establish a source/format with adequate authority, M101 remains blocked rather than returning arbitrary I2P website content.

## 3. Ownership architecture

```text
configured/adopted router-news source
              |
              v
I2PControl-owned bounded fetch/reader
              |
       authenticity/format checks
              |
              v
      complete immutable generation
              |
      bounded in-memory/durable cache
              |
              v
RouterInfo `i2p.router.news` serializer
```

No production code under `emissary-core/**` is needed or authorized.

If the reference source is a local file rather than network content, use the same I2PControl administrative-root confinement principles as M096. If it is network content, use a supported existing HTTP/network client path without creating a new general router downloader.

## 4. Privacy/network behavior

A background news fetch is network-visible behavior. Because it exists solely when I2PControl is enabled, M101 must make that ownership explicit.

Requirements:

- no fetch in default/feature-disabled execution;
- bounded refresh cadence with jitter only if reference/security evidence justifies it; do not add elaborate privacy machinery speculatively;
- finite connect/read/body timeouts;
- finite redirect policy;
- finite body/decompressed size;
- no arbitrary caller-controlled source URL through RouterInfo requests;
- source changes, if configurable at all, occur only through authenticated static/admin configuration outside the getter and are path/URL validated;
- RouterInfo requests read a cache/snapshot and do not trigger one network fetch per request;
- failures do not create unbounded retry loops.

If source access requires I2P routing/outproxy behavior, use the existing I2P application boundary rather than local DNS/clearnet shortcuts inconsistent with the adopted source.

## 5. Authenticity and format handling

News is untrusted external input even when retrieved from a canonical source.

M101 must:

- verify any signature/hash metadata required by the adopted format using existing appropriate primitives where available;
- reject malformed/truncated/oversized documents;
- bound entry count, title/body lengths, nested structure, and decompression ratio where applicable;
- avoid executing/rendering script or active content because I2PControl returns data, not a browser view;
- normalize/serialize exactly the Proposal 170 return semantic without injecting local paths, fetch errors, tokens, headers, or source credentials into the result;
- keep parser complexity proportional to the pinned format.

Do not copy GPL reference implementation code line-for-line; independently implement the documented format/behavior.

## 6. Cache/generation semantics

Use complete-generation publication:

1. fetch/read candidate within strict bounds;
2. authenticate/validate/parse completely;
3. convert to the canonical returned string form;
4. publish atomically as the current generation;
5. retain only bounded current/metadata state.

On refresh failure, retain the prior valid generation until a documented staleness threshold. After the threshold, follow M095's pinned failure semantics rather than pretending old news is fresh forever.

Persistence across router restart is optional unless M095/reference behavior requires it. If durable caching is used, it must be versioned/path-confined/atomic and must not make stale data indistinguishable from a freshly validated generation.

## 7. Preferred authorized path boundary

Target production changes under `emissary-cli/src/i2pcontrol/**`, likely:

- a dedicated `news`/`news_runtime` module;
- `production.rs`/`server.rs` only to compose one bounded source handle/task;
- `router_info.rs` / `router_info_handler.rs` to consume the source and change this one disposition;
- I2PControl-local persistence only if M095 requires durable cache;
- focused tests/docs/M095 matrix updates.

Dependency additions are discouraged. Prefer dependencies already present in `emissary-cli`. If a genuinely new I2PControl-only direct dependency is necessary for the adopted signed format, stop and create an explicit M062/M063-compatible dependency amendment rather than adding it incidentally.

No `emissary-core/**`, root dependency, startup router behavior, frontend, workflow, or release path is authorized.

## 8. Invariants

1. News has a real adopted source; no fabricated empty/random web content.
2. RouterInfo requests do not drive fetch cadence.
3. Fetch/parser/cache state is bounded.
4. External input is fully validated before publication.
5. Prior good generation survives refresh failure according to bounded staleness policy.
6. Feature-disabled/default builds fetch nothing.
7. No router-core news state/service.
8. No source credentials/local paths leak through API/logs.
9. Proposal 170 return type/spelling remains exact.
10. No upstream interaction occurs.

## 9. Explicit non-goals

M101 MUST NOT:

- build a general-purpose RSS/news framework;
- add frontend rendering;
- add browser/webview dependencies;
- add router-core fetch scheduling;
- allow unauthenticated/API-request source URL changes;
- add persistent cache if not needed for correctness;
- alter RouterInfo rows other than news;
- add hosted network CI solely for news;
- contact or submit to upstream.

## 10. Ordered work packages

### A. Freeze source/format/authenticity semantics

Use M095/reference evidence to document canonical source and returned string semantics before implementation.

### B. Implement bounded source acquisition

Use existing network/file primitives with strict timeout/redirect/size/path bounds.

### C. Implement independent parser/authenticator

Keep the parser narrow and format-specific. Build hostile/malformed fixtures from the documented format rather than copied implementation code.

### D. Implement generation/cache owner

One optional task/handle owns refresh and snapshot publication. RouterInfo reads only the current immutable bounded value/status.

### E. Integrate RouterInfo

Change only the news row from unavailable after a valid source generation can be produced according to pinned semantics.

### F. Update matrix/support evidence

Record source ownership, staleness/failure behavior, and feature-enabled network implications.

## 11. Failure, cancellation, restart, and contention semantics

- fetch timeout/DNS/connect/TLS/format/signature failure: no candidate publication;
- refresh failure: retain prior valid generation within staleness policy;
- no prior valid generation: RouterInfo follows pinned error/unavailable behavior, not fabricated empty success;
- stop: cancel one refresh task and close active fetch where possible;
- restart: reconstruct source configuration; optionally load durable cache only if validated and clearly timestamped/versioned;
- concurrent RouterInfo calls read snapshots without serializing behind a network fetch;
- no lock held across fetch, sleep, parser work that can be done outside the critical section, or cancellation join.

## 12. Compatibility/migration

No public schema change. The requested field transitions from current explicit unavailable behavior to the exact Proposal 170 String when a valid source exists.

Any durable cache/config is additive I2PControl state and must not affect default router behavior.

## 13. Security tests

At minimum:

- oversized body/document rejection;
- malformed/truncated format;
- signature/authenticity failures if applicable;
- redirect limit and unsafe redirect cases;
- decompression/entry/string bounds if applicable;
- source failure retaining prior generation;
- stale generation behavior;
- repeated refresh failure bounded backoff/cadence;
- no per-request fetch amplification;
- cancellation/restart;
- source credential/path/log redaction;
- feature-off no network access.

## 14. Verification

Run focused news/RouterInfo tests plus:

```text
cargo check -p emissary-cli --no-default-features --features i2pcontrol
cargo test -p emissary-cli --no-default-features --features i2pcontrol
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m061_containment
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m062_dependency_containment
cargo test -p emissary-cli --no-default-features --features i2pcontrol --test m063_feature_reachability
git diff --check
```

If M101 requires a new dependency, it cannot close until the separate dependency containment amendment is accepted.

## 15. Documentation/static guards

Update M095 news row only after real source evidence. Update support/conformance docs with source/cadence/staleness behavior while overall support remains partial.

Add a guard/test that RouterInfo handler itself does not perform source network I/O and that maximum published news size is finite.

## 16. Acceptance and stop conditions

M101 closes only if:

- canonical source/format semantics are established;
- valid real news content can be produced and bounded;
- source failure/staleness remains truthful;
- no core path changed;
- feature-off network behavior is unchanged;
- no high/medium parser/privacy/security finding remains;
- no upstream interaction occurred.

Stop if the source cannot be authenticated/defined adequately, if parity requires a new router-wide downloader, or if only an arbitrary public-web substitute is available.

## 17. Closure evidence required

Create `plans/closure/i2pcontrol-proposal-170/101-closure.md` containing:

- M095 source/format dependency evidence;
- exact changed paths/dependencies;
- source authenticity and returned-string evidence;
- parser/fetch/cache bounds;
- failure/staleness/cancellation/restart evidence;
- feature-off/containment results;
- updated RouterInfo matrix totals;
- unresolved findings;
- explicit external-read-only/internal-write-only attestation.

## 18. Internal-only rule

All writes remain internal to `eggstack/emissary`. External I2P/reference repositories and feeds are read-only correctness inputs. No upstream issue/PR/review/submission/merge/contribution activity is authorized.
