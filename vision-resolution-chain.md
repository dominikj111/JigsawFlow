# Vision: Capability Resolution Chain & Deferred Registry

> **Status**: Brainstorming / vision — written 2026-04-28
> **Relates to**: [PLAN.md layers L1–L5](PLAN.md#10-what-is-out-of-scope-for-this-document), [README — Singleton Network](README.md#singleton-network--distributed-registry)
> **Purpose**: Capture the resolution chain idea, the deferred registry design, and early thinking on capability visibility and security.

---

## The Core Idea

The `singleton-registry` (L0) is the core primitive of the JigsawFlow pattern — a typed, thread-safe service locator that is synchronous and knows nothing about networks. That is its strength: it is minimal, correct, and the stable foundation everything else composes on top of. It is not the JigsawFlow Microkernel; it is the mechanism the pattern and the Microkernel are built on.

The natural question is: what should happen when a capability is not registered locally? The answer is a layered resolution chain. Each layer is optional and independently configurable. Together they form a **capability resolution stack** that degrades gracefully at every level — and that, extended fully, is what the JigsawFlow Microkernel provides.

---

## Resolution Chain

When `registry.get(SomeCapability)` finds nothing locally, resolution proceeds outward:

```text
1. Local registry          (sync or async, in-process)
        ↓ not found
2. Local channel           (sync or async, same machine — IPC, Unix socket, shared memory)
        ↓ not found
3. Network / LAN           (sync or async, trusted local network — org intranet, dev environment)
        ↓ not found
4. Global registry         (async only — by design and by nature, internet-scale)
        ↓ not found
5. Null object fallback    (graceful degradation, logs warning)
   OR
5. Remote pull             (async only - download + register latest implementation, then retry)
```

Each step is **opt-in**. A runtime that should never talk to the internet simply does not configure steps 3–5.

**The composability principle:** The sync `singleton-registry` is the foundation. The async registry is built on top of it — not a replacement, an extension that widens the resolution scope. Steps 1–3 are available in both sync and async flavors because the async layer composes over the sync one. Step 4 is async only because internet-scale latency makes sync indefensible. This layering — quality composing upon quality — is what proves the design is right. A developer working only with local singletons uses sync directly and never pays for anything they don't need. A developer who needs wider resolution reaches for the async layer, which transparently covers the same local cases plus more.

**Microcontrollers and resource-limited devices** are a distinct target with their own constraints and are addressed separately — see the dedicated section below.

The important invariant: **L0 (`singleton-registry`) never changes**. Steps 2–5 are a wrapper layer on top of it, not modifications to it.

---

## The Deferred Registry

The proposed design is a **deferred registry**: a separate capability that wraps the local registry and extends it with network/channel lookup. It can operate in sync or async mode depending on the layer being resolved — steps 2–3 may resolve synchronously when latency is negligible; step 4 (global) is always async.

```text
Deferred registry
│
├── tryGet(token) → Promise<T | undefined>
│     1. local registry.tryGet(token)           → sync, immediate
│     2. if absent: channel/network lookup      → async
│     3. if found: local registry.register()    → cache result locally
│     4. if not found anywhere: return null object or undefined
│
└── get(token) → Promise<T>
      same chain, rejects (or returns null impl) if absent everywhere
```

The deferred registry is **itself registered as a capability** in the local registry:

```typescript
// The network registry is a JigsawFlow component like any other
registry.register(NetworkRegistry, createNetworkRegistry({ layers: [...] }));

// A component that wants network-aware resolution:
const net = registry.tryGet(NetworkRegistry);
const config = await net?.get(Config);
```

This keeps the design self-consistent: the network capability is optional and accessed through the same pattern as every other capability. A component that only needs local resolution never touches `NetworkRegistry`. A component that wants network fallback explicitly reaches for it.

### Why a separate capability, not a transparent upgrade

The tempting design is to make `registry.get()` transparently return a `Promise` if it falls back to the network. That was how CORBA and Java RMI failed: hiding the async boundary produced code that looked synchronous but behaved asynchronously, and the difference appeared only at runtime under load.

The explicit model is:

- Local registry (L0) → sync or async — the async registry is built on top of the sync one.
- Local channel / LAN (steps 2–3) → sync or async, developer's choice.
- Global registry (step 4) → async only, by design.
- The caller chooses which surface they need.

### Sync/async boundary model

The CORBA failure was about *hiding* the async boundary, not about *offering* a sync variant. These are different things. Node.js offers `fs.readFileSync` alongside `fs.readFile` — the sync version is not hidden, it is an explicit choice. The same principle applies here.

| Layer | Sync/async | Rationale |
| ----- | ---------- | --------- |
| L0 — local registry | Sync or async | Async registry is built on top of the sync one. Use sync for known-local singletons; use async when composing across resolution layers. |
| Steps 2–3 — local channel, LAN | Sync or async | Explicit developer choice via distinct API names (`tryGetSync` vs `tryGet`). Mandatory timeout — fails loudly, never silently hangs. |
| Step 4 — global registry | Async only | Internet-scale latency makes sync indefensible. Hard rule by design. |

The critical distinction from CORBA: the sync/async surface is visible at the call site through distinct API names. There is no illusion. A developer who calls `tryGetSync` across a LAN is making a conscious tradeoff; if the assumption breaks under load, the mandatory timeout surfaces the failure immediately.

### Daemon autopull — making the question moot

The most elegant answer to the sync/async tension is to avoid the choice entirely. If the daemon **proactively pushes** capability registrations into connected local runtimes before they are first needed, then by the time a component calls `registry.tryGet(Config)`, the capability is already in the local L0 registry. The lookup is sync. The async work happened in the daemon's background process before the component ran.

```text
Daemon (background)                  Local runtime
───────────────────                  ─────────────
Discovers Config owner on network
Generates local proxy                →  registry.register(Config, proxy)
                                         (already done before component starts)

Component (foreground)
  registry.tryGet(Config)            →  sync hit on L0, no network call
```

This is the **prefetch model**. Components never touch the deferred registry at all in the common case — they only see a pre-populated local registry. The deferred registry becomes the fallback for capabilities the daemon did not anticipate, not the primary resolution path.

Autopull is the preferred design for the daemon. The sync path on steps 2–3 is the escape hatch for cases where prefetch is not possible or not desired.

---

## Public vs. Private Capabilities

Not every capability should be visible to every runtime. There are at least three natural visibility levels:

**Internal** — implementation detail of a single runtime; never exported. Example: an internal caching helper, a low-level hardware abstraction in firmware.

**Org-private** — visible within a trust boundary (a team, an organisation, a secured network segment). Satisfies business logic that is proprietary or sensitive. Example: a pricing engine, an internal authentication scheme.

**Public** — visible globally through the community registry. Satisfies standard contracts (`Logger`, `Config`, `HttpClient`, `Storage`) with well-known, vetted implementations.

Each layer of the resolution chain maps naturally to one visibility level:

- L0 — always internal (in-process only)
- L1/L2 — org-private or explicitly shared within a controlled network
- L3/L4 (global) — public

A runtime exports an explicit subset of its capabilities to each layer. Nothing is exported by default. The CLI/daemon is the authority for what a runtime advertises.

---

## Security Model — Early Thinking

This section is deliberately speculative. Three models are worth understanding before designing anything.

### Model A — Linux-style permissions

Every capability has an owner (the runtime that registered it) and a set of permissions expressed as bits or roles: who can read (call), who can write (replace), who can administer (grant access to others).

The Linux model is proven but its design was shaped by file semantics and single-machine scope. Translating it to distributed capabilities across heterogeneous runtimes requires defining what "user" and "group" mean in this context. The closest mapping: a **runtime identity** (a signed public key) takes the role of user; a **namespace or org** takes the role of group.

### Model B — AWS IAM-style policies

Every action on a capability is an event. Policies describe which identities can perform which actions on which capability tokens. Evaluation is: does any policy allow this (identity, action, token) triple?

This is expressive and auditable. The cost is that policies are configuration, not code — they live outside the capability implementation and require a policy evaluation engine. More moving parts than Model A, but scales better to complex org structures.

### Model C — Capability-based security (the natural fit)

This model is less well-known but architecturally the cleanest fit. In capability-based security, a **capability** is an unforgeable token that grants the right to perform a specific action on a specific resource. You can only do something if you hold the token. You share access by passing the token. You revoke access by invalidating it.

The JigsawFlow registry contract token is already close to this model: if you hold a reference to the contract token, you can call `registry.get(token)`. Access control becomes: control who can hold which tokens.

In practice: the daemon issues **access tokens** when a runtime joins the network. An access token is a signed assertion: "runtime R is permitted to register/invoke capability C". The network registry validates the token before routing the request. Revocation is handled by the daemon invalidating previously issued tokens.

This model treats capabilities literally as capabilities in the security sense — a convergence that is probably not accidental.

### Open question: what is a "user" in JigsawFlow?

The entity that holds tokens is a runtime (a process, a firmware image, a container). Runtimes have identities (public keys, UUIDs, names). The security model's "user" is a runtime, not a human. Human-level access control (who can deploy which capability, who can issue tokens) is a layer above — an admin interface, a CI/CD system, or a governance process. That separation seems right.

---

## Null Object and Remote Pull

Two fallback strategies when nothing resolves:

**Null object** — a no-op implementation of the contract is registered automatically. It satisfies the interface (no crash), logs a warning, and returns safe defaults. This is the offline-first guarantee: the application always has something to call. The null object is defined by the contract author as part of the standard contracts catalog.

**Remote pull** — the resolution chain fetches the latest compatible implementation from a community registry, validates it (checksum, signature, trust level), and registers it locally for the lifetime of the runtime. The next lookup is a local hit. This enables the "dependency management" feel: declare needed contracts, and the runtime resolves implementations automatically — similar to `cargo` or `npm`, but at runtime rather than build time.

These two are not mutually exclusive. A runtime could use null objects for critical capabilities it must never block on, and remote pull for lower-priority utilities where downloading on first use is acceptable.

---

## How This Fits the Existing Layer Stack

The PLAN.md layer stack gains a clearer mapping:

| Layer | What it is | Resolution chain position |
| ----- | ---------- | ------------------------- |
| L0 | Local sync registry | Step 1 — in-process |
| L1 | Transport facade | Steps 2–4 transport (IPC → socket → libp2p) |
| L2 | Singleton network | Steps 2–4 discovery + proxy generation |
| L3 | Request-reply correlation | Protocol for steps 2–4 (mandatory for async path; optional for sync local-channel path) |
| L4 | Capability namespacing | Visibility scoping (internal / org-private / public) |
| L5 | RBAC / capability roles | Security model (Model A, B, or C above) |

The deferred registry is the developer-facing surface of L1+L2. L3 is its wire protocol. L4 controls what it can see. L5 controls what it is allowed to do.

---

## Capability Hierarchy Tree

The registry is flat at runtime — every capability is a peer. But the *design* of capabilities is not flat; it is hierarchical, and different roles in an organisation interact with it at different levels of abstraction.

```text
CEO / Product Owner
    └── Application capabilities  (billing, user management, reporting)
            └── Architect / Tech Lead
                    └── Quality capabilities  (observability, resilience, security posture)
                            └── Developer
                                    └── Functional capabilities  (storage, auth, HTTP, logger)
                                            └── Tester
                                                    └── Contract implementations  (mock, stub, real)
```

Each stakeholder composes at their layer using the vocabulary that makes sense to them:

- **CEO / Product Owner** — thinks in business capabilities: "what does this application do?" Expresses requirements as high-level contracts without knowing which technical capabilities satisfy them.
- **Architect** — thinks in quality attributes: "what resilience, observability, and security guarantees does this application provide?" Selects which capability implementations meet those attributes.
- **Developer** — thinks in functional components: "what contracts do I need to implement this feature?" Composes specific low-level capabilities and writes the business logic module.
- **Tester** — thinks in contract verification: "does this implementation satisfy the contract?" Swaps real implementations for controlled ones and verifies behaviour at the contract boundary.

**Concrete example — an invoicing feature:**

```text
CEO sees:
  Billing  ──────────────────────────── "can we charge customers?"

Architect adds:
  Billing
    ├── PaymentResilience               "retries, circuit breaker, idempotency"
    └── AuditCompliance                 "every transaction is logged and immutable"

Developer composes:
  Billing
    ├── PaymentGateway                  contract: charge(amount, customer) → receipt
    │     └── StripeAdapter             implementation (hot-swappable)
    ├── Ledger                          contract: record(transaction) → void
    └── AuditLog                        contract: append(entry) → void

Tester substitutes:
  Billing
    ├── PaymentGateway → StubPaymentGateway   (returns fixed receipts, never charges)
    ├── Ledger         → InMemoryLedger
    └── AuditLog       → CapturingAuditLog    (asserts on entries)
```

The Tester column is just a different set of registrations — same contracts, different implementations. No test framework, no mocking library needed.

The capability tree is also the **debugging interface**. A debugging tool navigates the tree: at the top, "which capabilities does this application expose?"; drill down, "which runtime owns this capability?"; drill further, "which concrete implementation is currently registered?"; at the leaf, "does this implementation satisfy its contract tests?". A mis-wired application is visible as a missing or unexpected node in the tree.

This hierarchy is a design and tooling concept — the runtime registry remains flat. The tree emerges from how capabilities are named, namespaced, and documented.

---

## CLI, Versioning, and Cross-Language Registration

### Versioning model

The CLI registers capability implementations by name and version:

```bash
jigsawflow register capability-rs-3.2.1       # Rust implementation, contract v3, impl 2.1
jigsawflow register capability-go-3.2.4       # Go implementation, same contract v3, impl 2.4
jigsawflow register capability-base           # null object — removes active implementation
```

The version number encodes two things:

- **Major version** — identifies the **contract**. `capability-rs-3.2.1` and `capability-go-3.2.4` both implement contract version 3. They are interchangeable at runtime; the daemon and consumers treat them as equivalent. A different major version (`capability-rs-4.0.0`) is a different contract — a different capability identity.
- **Minor and patch** — describe **implementation quality** within the same contract. A bump from `3.2.1` to `3.3.0` means the same contract is satisfied more reliably, faster, or with fewer bugs. The contract itself is unchanged; existing consumers benefit automatically on the next hot-swap.

The **language suffix** (`-rs-`, `-go-`, `-ts-`) identifies which runtime provides the implementation. Since JigsawFlow is language-agnostic, the suffix is primarily useful for the daemon and CLI to route capability processes correctly. Whether it remains explicit in the name or becomes an optional annotation is a detail to define later — the important thing is that the major version is what determines contract identity, not the language.

This gives capability authors clear guidance: if you want to add features, define a new contract (bump the major). If you want to improve quality, bump minor or patch. If you want to port to another language, keep the same major and register alongside the existing implementation.

One subtlety: the full version string (`capability-rs-3.2.1`) is **metadata for the daemon and CLI**, not a registry key. The registry itself stores one implementation per contract token — `register()` silently overwrites. When you hot-swap `capability-go-3.2.4` for `capability-rs-3.2.1`, the daemon routes the transition; the registry just sees one register call for the same contract. Two implementations of the same contract version cannot coexist in a single registry — if you need that (e.g. a canary deployment), you need two distinct contract tokens.

### Cross-language registration and the daemon as broker

A natural tension: if the CLI is written in TypeScript, can it register a Rust capability implementation? In general, no — the CLI cannot construct a Rust object. But the constraint disappears when the daemon is introduced as a **language-agnostic broker**.

The design:

1. A Rust binary implementing a capability starts up and connects to the daemon's communication channel (IPC pipe, Unix socket, message queue).
2. The daemon authenticates the binary and admits it as a capability provider.
3. The CLI sends `jigsawflow register capability-rs-3.2.1` — this is an instruction to the daemon, not a direct object instantiation.
4. The daemon informs connected runtimes that `capability-rs-3.2.1` is available on the channel.
5. Runtimes register a proxy implementation pointing at the channel endpoint.

The capability is written in Rust. The CLI is written in any language. The daemon is the translation layer. No runtime needs to know the implementation language of the capabilities it consumes.

This is the same ownership rule from PLAN.md expressed as a deployment workflow: the Rust binary owns the capability; every other runtime that uses it holds a proxy.

The daemon can itself be pluggable (this matches the separately mentioned work on an extensible Rust daemon that acts as a service manager). The communication channel it provides — RabbitMQ, Redis, IPC, REST, P2P — is itself a registered capability, hot-swappable without changing the capability implementations that use it.

---

## Scripting Language Nuance

In Node.js and Python, everything imported is already part of the application's module graph. The JigsawFlow distinction between "registered" and "not registered" is less sharp because the module loader is involved at a different level.

This suggests that for scripting runtimes:

- The **local registry** maps cleanly (explicit `register()` calls at startup)
- The **network fallback** is where the pattern pays dividends — capabilities not in the static import tree can still be resolved at runtime via the deferred registry
- **Lazy loading** fits naturally: the deferred registry can `import()` a capability module on first miss, then register and cache it locally

One important distinction from compiled languages: in Node.js, **remote pull is not just "download a binary"** — it requires running a package manager step first (`npm install` or equivalent) to place the module on disk before `import()` can load it. Remote pull for scripting runtimes therefore involves a two-phase fetch: package installation, then dynamic import. This is more complex than a compiled binary drop, but the pattern is well-established (think hot-reloading in dev servers or plugin systems in editors). The deferred registry would delegate this to a `PackageInstaller` capability — itself registered and swappable — keeping the registry logic clean.

---

## Open Questions

These are not resolved — they are the questions that need answers before L1–L5 can be specced formally.

1. **Resolution chain configuration** — where does a runtime declare which layers are active? A config file? A programmatic API at startup? A CLI command registered as a capability?

2. **Caching policy** — when the deferred registry fetches an implementation from the network and caches it locally, how long does the cache live? What triggers invalidation? Who has authority to push an invalidation signal?

3. **Null object authorship** — who defines the null object for a contract? The contract author (most authoritative), the consuming application (most flexible), or a community default? All three may be valid at different scopes.

4. **Security model selection** — Models A, B, and C are not mutually exclusive but designing for all three simultaneously is complex. A decision is needed before L5 can be specced. The capability-based model (C) feels most consistent with the JigsawFlow vocabulary, but requires the daemon to issue and validate tokens, which is a significant additional piece.

5. **Remote pull trust** — downloading and executing a remote implementation is inherently risky. The trust model (checksum, signature, community voting tier, sandboxing) needs to be designed before remote pull is usable in production. This is the same problem solved by OS package managers, container image signing, and browser extension stores — the pattern is well-understood; the JigsawFlow-specific application needs to be defined.

6. **Deferred registry as a standard contract** — should `NetworkRegistry` / `DeferredRegistry` be part of the standard contracts catalog? If yes, it is itself hot-swappable (different network backends can be registered). If no, it is a special privileged capability. The former seems more consistent with the pattern.

7. **Drain contract for in-flight network calls** — `best-practices.md` defines the drain contract for multi-runtime hot-swap: the old provider finishes in-flight work before shutting down. The deferred registry is the place where this matters most acutely: a component mid-execution holding a pending `await net.get(Config)` when the owning runtime shuts down has no local Arc to fall back on. The deferred registry needs a defined behaviour for this case — resolve from the next available provider? reject and let the caller retry? surface a `CapabilityDraining` error type? This needs an answer before L2 can be specced.

---

## Microcontrollers and Resource-Limited Devices

> **Focus area** — this section is a placeholder for dedicated investigation. The constraints of embedded and constrained targets differ enough from general-purpose runtimes that they deserve their own analysis rather than a footnote.

Constrained devices do not form a single category. At minimum, four distinct situations need separate treatment:

**1. No async, no network** — the device has no async runtime and no communication channel. Only step 1 (local sync registry) and step 5 (null object fallback) are available. This is the simplest case: the registry is a plain map, capabilities are registered at boot, the application runs. This is already well-covered by the Rust `no_std` and C bare-metal registry implementations described in [best-practices.md](best-practices.md#embedded-systems-and-iot).

**2. No async, simple networking** — the device has a communication channel (UART, serial, BLE, simple TCP) but no async runtime. Sync access to step 2 (local channel) becomes available. The channel is polled or interrupt-driven rather than event-loop driven. This is a meaningful addition: higher-level capabilities (ML inference, configuration, logging) can be proxied from a connected edge node over the sync channel, without reflashing.

**3. Async runtime, limited networking** — the device has an async executor (e.g. Embassy on Cortex-M, or MicroPython's event loop on ESP32/RP2040) but only limited network access. Steps 1–2 are fully available in both sync and async flavors. Step 3 (LAN) may be reachable depending on the network stack.

**4. Full async, full networking** — the device is more like a small embedded Linux system (Raspberry Pi, industrial SBC). The full resolution chain is potentially available. The only meaningful constraint compared to a general-purpose runtime is resource budget — memory, storage, and processing limits affect which implementations are feasible, not which steps of the chain are reachable.

The "own hardware, proxy everything else" model from [best-practices.md](best-practices.md#embedded-systems-and-iot) applies across all four cases. What changes per case is which resolution steps are reachable and whether the async surface is available. The sync registry remains the foundation in every case — even devices with no async support and no network have a valid, complete JigsawFlow application.
