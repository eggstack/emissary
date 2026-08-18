# M076 Closure — HTTP Server Anonymity and POST-Throttle Hardening

Status: closed; corrective history — retained anonymity/filter work remains accepted; M082 closed every follow-up defect

Source implementation plan:

- `plans/implementation/i2pcontrol-proposal-170/076-http-server-anonymity-and-post-throttle-hardening.md`

Corrective successor:

- `plans/implementation/i2pcontrol-proposal-170/082-http-peer-identity-and-expect-framing-corrective.md`
- `plans/closure/i2pcontrol-proposal-170/082-closure.md`

Implementation commit:

- `3cf082ef19efd490db6e5c602bebd5e0e95207cb`.

## 1. Retained implementation evidence

M076 delivered substantial security/anonymity improvements that remain required:

- Java-parity response filtering including `Date`, `Server`, `X-Powered-By`, `X-Runtime`, proxy headers;
- broader I2P+-style provider/cache/trace fingerprint stripping;
- request-side proxy identity stripping including `Forwarded`, `Via`, `X-Forwarded-*`, `X-Real-IP`, and related fields;
- spoofed `X-I2P-*` removal and trusted identity reinjection;
- preserved validated response framing;
- no application-body rewriting;
- POST/PUT/PATCH rejection before local target allocation;
- fixed-size fail-closed POST table without eviction of active/unexpired state;
- shared inbound handler/filter path for `httpserver` and `httpbidirserver`;
- fixed bounded error responses;
- I2PControl source containment.

The original package/focused HTTP/containment verification remains useful evidence for these retained properties.

## 2. Independent findings invalidating full closure

### MEDIUM — hard-coded 524-character trusted Destination ceiling is not valid for all current I2P Destination forms

M076 treated `MAX_TRUSTED_DESTINATION_TEXT = 524` as a safe maximum based on a 391-byte reference Destination assumption. Current I2P key-certificate/signature types include forms whose valid serialized Destination representation is larger than that legacy assumption.

The HTTP filter can therefore reject a structurally valid authenticated peer solely because its Destination uses a larger supported key form.

The repository already has I2PControl helpers based on:

```text
base64_decode -> emissary_core::primitives::Destination::parse -> Destination::id()
```

M082 must use structural validity/canonical identity and a bound derived from all currently supported Destination forms rather than the obsolete magic ceiling.

### MEDIUM — unsupported `Expect: 100-continue` can hold a handler until body timeout

The HTTP flow forwards sanitized request headers to the loopback backend and then waits for the remote request body before reading the backend response. `Expect` is not currently rejected.

A client sending `Expect: 100-continue` may wait for the interim response while the backend has already emitted it. Emissary waits for the body and does not read that interim response, creating a client/backend wait cycle until the body timeout.

M082 must reject all `Expect` requests before local target allocation with fixed bounded close semantics rather than expanding the HTTP state machine to support informational responses.

### LOW-MEDIUM — POST peer accounting still uses an eight-byte `DefaultHasher` key

As with shared admission, HTTP write throttling should use the canonical cryptographic Destination ID/hash from the trusted accepted peer. M082 owns this correction and must revalidate that its auxiliary expiry metadata is itself bounded.

## 3. Why original verification missed these findings

The original identity test treated a string longer than 524 as invalid without including structurally valid large current Destination fixtures. It therefore verified the constant, not the full I2P identity domain.

HTTP framing tests covered Content-Length/Transfer-Encoding, upgrades, parser bounds, proxy identity, response framing, and fingerprints, but did not exercise `Expect` semantics or prove that an expectation-waiting client cannot pin the request-body phase.

POST limiter tests verified table cardinality/churn behavior, not canonical cryptographic peer-key identity.

## 4. Inherited M074 corrective dependency

HTTP accepted streams also consume the shared server admission state. M080 owns independent M074 defects involving aggregate-rejection state poisoning, expiry-index bounds, capacity/retention coherence, and canonical peer accounting.

M082 must consume the corrected M080 trusted-peer identity boundary where practical rather than creating a separate identity model.

## 5. Corrective requirements retained for M082

M082 must:

- accept every structurally valid current I2P Destination supported by the repository parser, including large valid key-certificate/signature forms;
- reject malformed/non-Destination peer text before request construction/local target;
- retain an explicit defensible bound derived from actual supported representation rather than arbitrary 64 KiB/524-byte values;
- derive B32/B64 injected identity from the authenticated structurally valid peer;
- reject any `Expect` header before local connect, preferably with fixed `417 Expectation Failed` + `Connection: close` semantics;
- move POST peer keys to canonical Destination ID/hash;
- preserve M076 fingerprint/proxy/framing/limiter protections;
- prove inbound `httpbidirserver` inherits the same corrections.

Do not add full `100 Continue` support, a second HTTP parser, body rewriting, HTTP/2, TLS termination, or adjacent protocol features inside this corrective.

## 6. Current disposition

M076's fingerprint and POST-state architecture is retained; the independent identity/Expect/accounting-key findings are closed by M082
(`plans/closure/i2pcontrol-proposal-170/082-closure.md`). M077 is now
dependency-ready, M079 remains the final independent reclosure
authority.

External I2P/I2P+ source material remains read-only behavioral evidence. No upstream review, issue/PR mutation, merge, or submission is authorized.
