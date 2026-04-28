# JigsawFlow: Singleton Registry — Language Implementation Plan

> **Status**: Foundation specification — written 2026-04-27  
> **Scope**: Defines the singleton registry contract that every language port must satisfy  
> **Active target**: TypeScript/Node.js (`singleton-registry` npm package, local name `singleton-registry-npm`)  
> **Reference implementation**: Rust crate `singleton-registry` v2

---

## 1. Validation of the Architecture Thinking

Before the spec: confirming that the reasoning behind the plan is sound.

### The singleton registry IS the right foundation

JigsawFlow's four invariants — offline-first components, zero direct coupling between components, all external dependencies behind facades, and runtime hot-swap — are all _enforcement mechanisms_, not structural patterns. The mechanism that enforces them is the registry. Nothing else needs to exist for the pattern to hold:

- **Component independence** is enforced because components cannot import each other; they can only call `registry.get(SomeContract)`. If the contract is absent, they degrade gracefully.
- **Facade wrapping** is enforced because even the filesystem, clock, and RNG are registered through the registry behind a contract. There is no "side door."
- **Hot-swap** is possible because the registry is the only resolution point. Replacing a registered value redirects all future lookups without touching callers.
- **Offline-first** becomes the default: a missing registration means the capability is unavailable, and every well-written JigsawFlow component handles that with a warning rather than a crash.

A DI container, event bus, module bundler, or plugin loader can all be _implemented on top of_ the registry, but the registry itself is the minimal complete primitive.

### The polyglot implication is a side effect, not a goal

Each language runtime gets its own registry. The registries are independent. They know nothing about each other. When you want a Node.js process and a Rust process to call into each other's capabilities, you add a communication channel (IPC pipe, Unix socket, gRPC, HTTP) — and that channel is itself a facade registered in each registry. The registry does not change; you just register a remote-proxy implementation of a contract instead of a local one. The polyglot story emerges from this: two microkernel instances with a bridge between them, each locally coherent. This plan covers only the registry; the communication bridge is out of scope.

### Node.js and Rust daemons managing services the same way — confirmed

Because both registries expose identical semantics (register a contract implementation, retrieve it by contract, hot-swap at runtime), application-level logic written against the pattern looks the same in both languages. The contracts are the shared vocabulary; the registries are the runtime backing. A system composed of a Rust daemon and a Node.js orchestrator is two microkernel instances, each managing its own capability set using the same pattern.

---

## 2. Core Semantics (language-agnostic)

These rules must hold in every port. They are not negotiable.

| Rule | Description |
| ------ | ------------- |
| One instance per contract | Each contract maps to exactly one registered value. No named singletons. |
| Register overwrites | Registering a contract a second time silently replaces the previous value. |
| No removal | Contracts cannot be deleted from the registry, only replaced. |
| Retrieval returns a snapshot reference | `get()` returns the current value at call time; if the registry is updated afterward, existing references are unaffected. |
| Missing contract is not a crash | Retrieval of an unregistered contract returns an error/undefined, not an exception, so callers can degrade gracefully. |
| Registry isolation | Multiple independent registry instances may coexist with zero shared state. |
| Tracing / observability | An optional callback receives events (register, get, contains, clear) for debugging. |

### Language-neutral API surface

Every port exposes these operations. Names should be idiomatic to the target language; behaviour must match exactly.

| Operation | Behaviour |
| --------- | --------- |
| `register(token, value)` | Store `value` for `token`; silently overwrites any previous value |
| `get(token)` → value or error | Retrieve value; signal absence via error/exception/Result if not registered |
| `tryGet(token)` → value or absent | Retrieve value; return null/undefined/None/Option if not registered — the primary graceful-degradation path |
| `has(token)` → bool | Return true if a value is registered for the token; no side effects |
| `setTraceCallback(fn)` | Register an observability callback fired after every operation; optional but recommended |
| `clear()` | Remove all values — test teardown only, not for production use |

> The Rust crate also exposes `register_arc` (store a pre-wrapped `Arc<T>`) and `get_cloned` (return an owned clone). These are Rust-specific conveniences; other language ports do not need them.

### Language-agnostic implementation checklist

- [ ] Type-keyed storage using the language's appropriate token mechanism (see Section 11)
- [ ] Thread-safe access — mutex or lock where the runtime is concurrent (see Section 11)
- [ ] Reference-safe value storage — `Arc`, `shared_ptr`, or GC-managed references (see Section 11)
- [ ] `register(token, value)` — overwrites silently; never errors
- [ ] `get(token)` — errors/throws on missing token
- [ ] `tryGet(token)` — returns absent value (null/undefined/None/Option) on missing token
- [ ] `has(token)` — boolean presence check; no side effects
- [ ] Multiple isolated registry instances with zero shared state between them
- [ ] Tracing callback fired after register, get, has, and clear
- [ ] `clear()` — removes all values; for test teardown only
- [ ] Tests: primitives, contract/interface objects, replacement semantics, registry isolation, thread safety (where applicable)

---

## 3. Language-Specific Design Decisions: TypeScript

### 3.1 The contract token problem

Rust's type system provides `TypeId` — a unique compile-time identity for every type. TypeScript interfaces vanish at runtime; they cannot serve as Map keys. This is the central design problem for the TypeScript port.

#### Decision: abstract class as contract token (primary path)

An abstract class is both a type (TypeScript compile-time) and a constructor function (JavaScript runtime). This gives us:

- A valid `Map` key (the constructor reference)
- TypeScript type inference on retrieval (no casts needed)
- A natural contract definition (mirrors a Rust trait)
- Runtime `instanceof` availability if needed

```typescript
// Contract definition — mirrors a Rust trait
abstract class Formatter {
  abstract format(title: string, body: string): string;
}

// Implementation — mirrors `impl Formatter for PlainFormatter`
class PlainFormatter extends Formatter {
  format(title: string, body: string): string {
    return `${title}: ${body}`;
  }
}

// Registration and retrieval are type-safe:
registry.register(Formatter, new PlainFormatter());
const fmt = registry.tryGet(Formatter); // inferred type: Formatter | undefined
```

**Alternative: `RegistryKey<T>` for pure-interface contracts**

When the contract is a plain TypeScript `interface` (e.g., a third-party type that cannot be extended via abstract class), a typed key object provides the same guarantees:

```typescript
interface Logger { log(msg: string): void; }
const LOGGER = new RegistryKey<Logger>('Logger');

registry.register(LOGGER, myConsoleLogger);
const logger = registry.tryGet(LOGGER); // inferred type: Logger | undefined
```

Both token forms use the same internal `Map<object, unknown>` storage. The abstract class constructor and the `RegistryKey` instance are both stable object references.

### 3.2 Thread safety

Node.js runs application code on a single event-loop thread. Worker Threads run in separate V8 isolates with their own module scope; a registry imported in a worker is an independent instance with no shared state. There is no need for locking in the default Node.js runtime model.

The TypeScript registry is therefore implemented without mutexes. If explicit Worker Thread sharing were ever needed (unlikely in practice), the isolation-by-design already satisfies the requirement differently.

### 3.3 Reference semantics vs. `Arc`

Rust's `Arc<T>` provides reference counting for safe concurrent ownership. TypeScript has garbage-collected object references: object identity is stable until all references are dropped. The behavior is the same from the consumer's perspective:

- `register()` stores the object reference.
- `get()` / `tryGet()` return the same reference (not a copy).
- After `register()` is called again with a new value, old references are still alive until their holders release them.

No `Arc` wrapper is needed. The registry stores `unknown` internally and returns typed references on retrieval.

### 3.4 Error handling

TypeScript does not have `Result<T, E>`. The primary retrieval method returns `T | undefined` rather than throwing, because the graceful-degradation idiom is the expected usage:

```typescript
const fmt = registry.tryGet(Formatter);
if (!fmt) {
  console.warn('no Formatter registered, skipping');
  return;
}
fmt.format(title, body);
```

A throwing variant `get()` is also provided for contexts where the caller knows the contract must be present and an absent registration is a programming error.

---

## 4. Full API Specification — TypeScript

### 4.1 Registry instance API

```typescript
interface Registry {
  /**
   * Register a value for the given contract token.
   * Overwrites any previously registered value for that token.
   */
  register<T>(token: Token<T>, value: T): void;

  /**
   * Retrieve the registered value for the given token.
   * Returns undefined if no value is registered.
   */
  tryGet<T>(token: Token<T>): T | undefined;

  /**
   * Retrieve the registered value, throwing RegistryError if absent.
   * Use when the contract must be present (programming-error guard).
   */
  get<T>(token: Token<T>): T;

  /**
   * Check whether a contract has a registered value.
   */
  has<T>(token: Token<T>): boolean;

  /**
   * Register a tracing callback.
   * Called synchronously after each register/get/has/clear operation.
   */
  setTraceCallback(cb: (event: RegistryEvent) => void): void;

  /** Remove the tracing callback. */
  clearTraceCallback(): void;

  /**
   * Remove all registered values.
   * Intended for test teardown only.
   */
  clear(): void;
}
```

### 4.2 Token types

```typescript
/** Abstract class constructor — the primary contract token type. */
type AbstractConstructor<T> = abstract new (...args: never[]) => T;

/** Concrete class constructor — also accepted as a token. */
type ConcreteConstructor<T> = new (...args: never[]) => T;

/** Typed key object — for interface contracts that cannot use abstract classes. */
class RegistryKey<T> {
  constructor(public readonly description: string);
  toString(): string; // returns `RegistryKey(${description})`
}

/** Union of all valid token types. */
type Token<T> =
  | AbstractConstructor<T>
  | ConcreteConstructor<T>
  | RegistryKey<T>;
```

### 4.3 Error type

```typescript
type RegistryErrorKind = 'TypeNotFound' | 'TypeMismatch';

class RegistryError extends Error {
  constructor(
    public readonly kind: RegistryErrorKind,
    public readonly tokenDescription: string,
  );
  message: string; // `[${kind}] ${tokenDescription}`
}
```

### 4.4 Tracing events

```typescript
type RegistryEvent =
  | { type: 'register'; tokenDescription: string }
  | { type: 'get';      tokenDescription: string; found: boolean }
  | { type: 'has';      tokenDescription: string; found: boolean }
  | { type: 'clear' };
```

### 4.5 Factory function

```typescript
/** Create a new isolated registry instance. */
function createRegistry(): Registry;
```

No default export. Each consumer calls `createRegistry()` and owns its instance. This replaces the Rust `define_registry!(name)` macro: the isolation unit is the object reference rather than a static module-level storage block.

---

## 5. Package Structure

```text
singleton-registry-npm/
├── package.json
├── tsconfig.json
├── tsconfig.build.json
├── src/
│   ├── index.ts          — public API re-exports
│   ├── registry.ts       — Registry class implementation
│   ├── registry-key.ts   — RegistryKey<T> class
│   ├── registry-error.ts — RegistryError class
│   └── types.ts          — Token<T>, RegistryEvent, shared type definitions
├── tests/
│   ├── basic.test.ts           — register, get, tryGet, has for primitive and object values
│   ├── contracts.test.ts       — abstract class tokens, interface + RegistryKey tokens
│   ├── replacement.test.ts     — overwrite semantics, reference stability after replacement
│   ├── isolation.test.ts       — multiple independent registries, no cross-instance leakage
│   ├── tracing.test.ts         — event callbacks, all event types, callback removal
│   └── errors.test.ts          — RegistryError on get() miss, correct kind/message
├── examples/
│   ├── 01_capability_contract.ts
│   ├── 02_runtime_swap.ts
│   └── 03_testing_without_mocks.ts
└── README.md
```

### package.json key fields

```json
{
  "name": "singleton-registry",
  "version": "1.0.0",
  "description": "Type-safe singleton registry for TypeScript/Node.js — foundation for JigsawFlow microkernel architecture",
  "main": "dist/index.js",
  "types": "dist/index.d.ts",
  "exports": {
    ".": {
      "import": "./dist/index.js",
      "types": "./dist/index.d.ts"
    }
  },
  "type": "module",
  "scripts": {
    "build": "tsc -p tsconfig.build.json",
    "test": "node --experimental-vm-modules node_modules/jest/bin/jest.js",
    "test:watch": "...",
    "typecheck": "tsc --noEmit"
  },
  "license": "BSD-3-Clause"
}
```

**Zero runtime dependencies.** Test tooling only in `devDependencies`.

---

## 6. Internal Implementation Notes

### Storage

```typescript
// Inside registry.ts
private storage = new Map<object, unknown>();
```

The `Map` key is the token object itself (constructor reference or `RegistryKey` instance). JavaScript `Map` uses reference equality (`===`) for object keys — stable and correct.

### Token description (for error messages and tracing)

Each token must resolve to a human-readable string for events and errors:

```typescript
function describeToken(token: Token<unknown>): string {
  if (token instanceof RegistryKey) return token.toString();
  // Abstract / concrete class: use .name property
  return (token as Function).name ?? 'unknown';
}
```

### Retrieval with type assertion

The internal storage is `Map<object, unknown>`. On `get()` / `tryGet()`, the value is returned as-is with a TypeScript `as T` cast. The type system (via the `Token<T>` generic) guarantees that what was registered under that token has type `T`. There is no runtime downcast (no `instanceof`), matching how Rust's `Any::downcast` is implied-correct by the TypeId key.

### Tracing: invoke outside the get/set hot path

The trace callback must be called after the operation completes, not while holding any internal lock (not applicable in single-threaded Node.js, but good convention for future Worker Thread consideration and for not blocking the caller):

```typescript
const result = this.storage.get(key);
this.traceCallback?.({ type: 'get', tokenDescription: desc, found: result !== undefined });
return result as T | undefined;
```

---

## 7. Node.js Examples to Implement

The three Node.js examples exactly mirror the three Rust examples in `examples/rust/`. Same concepts, TypeScript idioms.

### 7.1 `01_capability_contract.ts` — mirrors `01_capability_contract.rs`

```typescript
import { createRegistry } from 'singleton-registry';

abstract class Formatter {
  abstract format(title: string, body: string): string;
}

const app = createRegistry();

function generateReport(title: string, body: string): void {
  const fmt = app.tryGet(Formatter);
  if (!fmt) {
    console.warn('warn: no Formatter registered, skipping');
    return;
  }
  console.log(fmt.format(title, body));
}

class PlainFormatter extends Formatter {
  format(title: string, body: string): string {
    return `${title}: ${body}`;
  }
}

// No Formatter registered yet — degrades gracefully.
generateReport('Q0', 'This will be skipped.');

// Composition lives here — the only place that picks concrete types.
app.register(Formatter, new PlainFormatter());
generateReport('Q1', 'Revenue up 12%.');
```

Expected output:

```text
warn: no Formatter registered, skipping
Q1: Revenue up 12%.
```

### 7.2 `02_runtime_swap.ts` — mirrors `02_runtime_swap.rs`

```typescript
import { createRegistry } from 'singleton-registry';

abstract class Formatter {
  abstract format(title: string, body: string): string;
}

const app = createRegistry();

function generateReport(title: string, body: string): void {
  const fmt = app.tryGet(Formatter);
  if (!fmt) { console.warn('no Formatter registered, skipping'); return; }
  console.log(fmt.format(title, body));
}

class PlainFormatter extends Formatter {
  format(title: string, body: string): string { return `${title}: ${body}`; }
}

class MarkdownFormatter extends Formatter {
  format(title: string, body: string): string { return `# ${title}\n\n${body}`; }
}

// First call: plain text.
app.register(Formatter, new PlainFormatter());
generateReport('Q1', 'Revenue up 12%.');

// Swap — generateReport never changes.
app.register(Formatter, new MarkdownFormatter());
generateReport('Q2', 'Projections look strong.');
```

Expected output:

```text
Q1: Revenue up 12%.
# Q2

Projections look strong.
```

### 7.3 `03_testing_without_mocks.ts` — mirrors `testing_without_mocks.rs`

```typescript
import { createRegistry } from 'singleton-registry';

// Isolated registry per test suite — zero shared state.
const testApp = createRegistry();

abstract class Formatter {
  abstract format(title: string, body: string): string;
}

abstract class Sink {
  abstract write(content: string): void;
}

class PlainFormatter extends Formatter {
  format(title: string, body: string): string { return `${title}: ${body}`; }
}

class CapturingSink extends Sink {
  private captured: string[] = [];
  write(content: string): void { this.captured.push(content); }
  lines(): string[] { return [...this.captured]; }
}

function generateReport(title: string, body: string): void {
  const fmt = testApp.tryGet(Formatter);
  if (!fmt) { console.warn('no Formatter'); return; }
  const sink = testApp.tryGet(Sink);
  if (!sink) { console.warn('no Sink'); return; }
  sink.write(fmt.format(title, body));
}

// Test: report includes title
const sink = new CapturingSink();
testApp.register(Sink, sink);
testApp.register(Formatter, new PlainFormatter());

generateReport('Q1', 'Revenue up 12%.');

console.assert(sink.lines()[0].includes('Q1'), 'Expected Q1 in captured output');
console.log('Test passed:', sink.lines());
```

---

## 8. Test Specification

Each test file maps to one aspect of the registry's contract:

### `basic.test.ts`

- Register a string, retrieve it — values match
- Register a number, retrieve it
- Register a plain object, retrieve it
- `has()` returns false before register, true after
- `tryGet()` returns undefined before register

### `contracts.test.ts`

- Register implementation of abstract class contract, retrieve via contract token — returns correct instance
- Call method through retrieved reference — correct behavior
- Register `RegistryKey<Logger>`-based value, retrieve it — returns correct instance
- Two different abstract class contracts are independent slots
- Two different `RegistryKey` instances with same description string are independent slots (identity, not equality)

### `replacement.test.ts`

- Register value A, then value B under same token — `tryGet()` returns B
- Reference obtained before replacement still points to old value (A) — replacement does not mutate in-place
- Three consecutive replacements — last one wins

### `isolation.test.ts`

- Two registry instances, same token, different values — no cross-contamination
- `clear()` on one registry does not affect the other
- Creating 100 independent registries — all independent

### `tracing.test.ts`

- `setTraceCallback` receives `register` event after `register()`
- Receives `get` event with `found: true` after successful `tryGet()`
- Receives `get` event with `found: false` after `tryGet()` miss
- Receives `has` event with correct `found` value
- Receives `clear` event after `clear()`
- After `clearTraceCallback()`, no more events fired
- Replacing the callback — only new callback receives events

### `errors.test.ts`

- `get()` throws `RegistryError` when token not registered
- Error `kind` is `'TypeNotFound'`
- Error `tokenDescription` matches the token's name / description
- Error message is human-readable

---

## 9. Implementation Checklist

- [ ] `RegistryKey<T>` class with stable object identity
- [ ] `Token<T>` type accepting abstract class constructors, concrete constructors, and `RegistryKey`
- [ ] `createRegistry()` factory returning isolated `Registry` instance
- [ ] `register(token, value)` — stores value, emits `register` event
- [ ] `tryGet(token)` — returns `T | undefined`, emits `get` event
- [ ] `get(token)` — returns `T` or throws `RegistryError`, emits `get` event
- [ ] `has(token)` — returns boolean, emits `has` event
- [ ] `setTraceCallback` / `clearTraceCallback`
- [ ] `clear()` — test teardown; emits `clear` event
- [ ] `RegistryError` with `kind` and `tokenDescription` fields
- [ ] `describeToken()` helper using `.name` for constructors, `.toString()` for `RegistryKey`
- [ ] Public re-exports from `index.ts`
- [ ] TypeScript strict mode (`"strict": true` in tsconfig)
- [ ] ESM-first output (`"type": "module"` in package.json)
- [ ] Zero runtime dependencies
- [ ] All tests passing
- [ ] Three Node.js examples runnable with `npx ts-node` or `tsx`

---

## 10. What Is Out of Scope for This Document

The following topics belong to future documents. Each layer builds on the local registry (L0) specified here.

### Layer map

```text
L5  RBAC / capability roles        — who can register or invoke which capability
L4  Capability namespacing         — scoping which runtimes see which capabilities
L3  Request-reply correlation      — promise wrapper over the shared channel (correlation IDs)
L2  Singleton network              — distributed discovery, auto-proxy generation, ownership sync
L1  Transport                      — TCP/localhost → Unix socket → libp2p (P2P)
L0  Local singleton registry       ← THIS DOCUMENT
```

Each layer is independently replaceable. L0 is a stable primitive; upper layers change without touching it.

### L1 — Transport

The communication facade is registered in the local registry like any other capability. Swapping the transport (TCP → IPC → libp2p) is a singleton hot-swap, not a code change.

### L2 — Singleton network

The network-aware registry layer. When `registry.get(SomeCapability)` finds nothing locally, L2 queries connected runtimes and returns a transparent proxy. Key design decisions for the implementing document:

- **Ownership rule**: each capability has exactly one owning runtime; re-registration on the network follows last-write-wins with CLI/daemon authority
- **Distributed drain contract**: CLI signals old provider to drain in-flight work before shutdown; new provider takes over immediately on registration
- **Async boundary**: network-resolved capabilities return `Promise`/`Future`; local capabilities remain synchronous — the distinction is explicit, not hidden

### L3 — Request-reply correlation

Every cross-runtime call carries a correlation ID generated by the requester. The promise wrapper maps incoming responses to pending requests by ID. JSON-RPC 2.0 specifies this pattern exactly and is the recommended starting point.

### L4 — Capability namespacing

Each application exports an explicit subset of capabilities to the shared channel. This scoping document defines which runtimes see which capabilities in a multi-application environment.

### L5 — RBAC / capability roles

Capabilities carry identity and permissions. Only authorised runtimes can register or invoke a given capability. This is capability-based security applied to the distributed registry.

### Also out of scope

- **Event bus / reactive components**: implemented on top of L0 (register an event emitter as a capability)
- **JigsawFlow standard contracts**: a catalog of common contract tokens (Logger, Config, Storage, Http, ...) — belongs in a separate `@jigsawflow/contracts` package
- **Service loader / component discovery**: dynamic loading of component modules resolved through the registry

---

## 11. Cross-Language Reference

### Rust ↔ TypeScript feature map

| Feature | Rust | TypeScript |
| --------- | ------ | ----------- |
| Contract definition | `trait Formatter: Send + Sync` | `abstract class Formatter` |
| Implementation | `impl Formatter for PlainFormatter` | `class PlainFormatter extends Formatter` |
| Registry creation | `define_registry!(app)` | `const app = createRegistry()` |
| Register | `app::register(Arc::new(PlainFormatter) as Arc<dyn Formatter>)` | `app.register(Formatter, new PlainFormatter())` |
| Retrieve (optional) | `app::get_cloned::<Arc<dyn Formatter>>()` → `Result` | `app.tryGet(Formatter)` → `T \| undefined` |
| Retrieve (required) | `app::get::<Arc<dyn Formatter>>().unwrap()` | `app.get(Formatter)` → `T` or throws |
| Check presence | `app::contains::<Arc<dyn Formatter>>()` | `app.has(Formatter)` |
| Graceful degrade | `match ... { Err(_) => warn }` | `if (!fmt) { console.warn(...) }` |
| Swap at runtime | call `register` again | call `register` again |
| Test isolation | `define_registry!(test_app)` in test module | `const testApp = createRegistry()` in test file |
| Token type | `TypeId` (implicit, from generic) | constructor or `RegistryKey<T>` (explicit) |
| Reference safety | `Arc<T>` reference counting | JavaScript GC object references |
| Thread safety | `Mutex<HashMap>` | Not needed (single event loop) |

---

### Token design by language

The most language-specific decision in any port is how to identify a contract at runtime — the "token". Each language solves this differently.

| Language | Token mechanism | Registration example | Notes |
| -------- | --------------- | -------------------- | ----- |
| **Rust** | `TypeId` (implicit, from generic `T`) | `app::register(Arc::new(PlainFormatter) as Arc<dyn Formatter>)` | Compile-time identity; zero runtime overhead |
| **C** | `const char *` string key — no RTTI available | `registry_register("Formatter", &plain_formatter_vtable)` | Contract is a struct of function pointers (vtable pattern). String token is the only portable mechanism. ⚠ Embedded/bare-metal target — store vtables in static memory; the registry does not own or free them |
| **C++** | `std::type_index` (from `<typeindex>`) | `registry.register_value(std::make_shared<PlainFormatter>())` | `std::type_index(typeid(T))` is unique per concrete type; use `static_pointer_cast<T>` on retrieval |
| **TypeScript / JS** | Abstract class constructor or `RegistryKey<T>` object | `registry.register(Formatter, new PlainFormatter())` | Interfaces vanish at runtime; class constructor is the stable key |
| **Java** | `Class<T>` — `Formatter.class` | `registry.register(Formatter.class, new PlainFormatter())` | First-class at runtime. Generic parameters are erased (`List<String>.class` doesn't exist) — use a `RegistryKey<T>` wrapper for parameterised types |
| **Kotlin** | `KClass<T>` — `Formatter::class` | `registry.register(Formatter::class, PlainFormatter())` | Runs on JVM; reified inline functions (`inline fun <reified T>`) eliminate explicit `::class` at call sites. Parameterised types still need a `RegistryKey<T>` wrapper (same erasure as Java) |
| **C# / .NET** | `Type` — `typeof(IFormatter)` | `registry.Register(typeof(IFormatter), new PlainFormatter())` | Generics are reified, so `typeof(List<string>)` works without a key wrapper |
| **Python** | Class / ABC object — `Formatter` | `registry.register(Formatter, PlainFormatter())` | Classes are first-class objects and valid dict keys; straightforward |
| **PHP** | `::class` string — `Formatter::class` | `$registry->register(Formatter::class, new PlainFormatter())` | Resolves to a fully-qualified class name string; valid array key |
| **Swift** | Protocol/class metatype — `Formatter.self`; keyed by `ObjectIdentifier` | `registry.register(Formatter.self, value: PlainFormatter())` | `ObjectIdentifier(metatype)` is `Hashable` and stable. Primary target: iOS/macOS/server-side Vapor |
| **Go** | `reflect.Type` **or** explicit `RegistryKey[T]` | `registry.Register(reflect.TypeOf((*Formatter)(nil)).Elem(), &PlainFormatter{})` | ⚠ Go interfaces are structural — two identical interface definitions in different packages share the same `reflect.Type`. Prefer an explicit `RegistryKey[T]` to avoid accidental collisions |

---

### Thread safety by language

Rust and TypeScript each have a natural model (ownership + `Mutex`, and single event loop respectively). Other target languages are genuinely concurrent and require explicit synchronisation inside the registry implementation.

| Language | Concurrency model | Required synchronisation |
| -------- | ----------------- | ------------------------ |
| **Rust** | Ownership + `Send/Sync` bounds | `Mutex<HashMap<TypeId, Arc<dyn Any + Send + Sync>>>` — already in the reference implementation |
| **C** | Preemptive threads (POSIX) or RTOS tasks | POSIX: `pthread_mutex_t`; C11: `mtx_t`; FreeRTOS: `SemaphoreHandle_t`; Zephyr: `k_mutex`. Bare-metal single-core: no locking needed |
| **C++** | Preemptive threads, shared memory | `std::shared_mutex` — `shared_lock` for reads, `unique_lock` for register; storage is `std::unordered_map<std::type_index, std::shared_ptr<void>>` |
| **TypeScript / Node.js** | Single event loop; Worker Threads are isolated | None required for the default runtime; Worker Thread isolation is structural |
| **Java** | Preemptive threads, shared heap | `ConcurrentHashMap` for reads; `synchronized` block or `ReentrantReadWriteLock` for replace-on-register |
| **Kotlin** | JVM threads + coroutines | Threads: same as Java (`ConcurrentHashMap`, `ReentrantReadWriteLock`). Coroutines: `kotlinx.coroutines.sync.Mutex` for suspension-safe locking |
| **C# / .NET** | Preemptive threads, shared heap | `ConcurrentDictionary<Type, object>` covers most cases; `ReaderWriterLockSlim` for higher read throughput |
| **Python** | GIL (CPython) + threads/asyncio | `threading.Lock` around register/get; GIL alone is not sufficient for compound read-modify-write on replace |
| **PHP** | Shared-nothing per request (FPM) | No locking needed under FPM. Swoole/ReactPHP coroutine runtimes need a mutex or channel-based guard |
| **Swift** | Preemptive threads + Swift concurrency | `DispatchQueue` (serial) or `os_unfair_lock` for low-level use; Swift 5.5+ actor model makes the registry an `actor` — the compiler enforces exclusive access |
| **Go** | Goroutines, shared memory | `sync.RWMutex` — `RLock` for reads, `Lock` for register; keeps hot read path lock-free under concurrent reads |

---

### Reference counting by language

`get()` must return a copy of the reference, not a pointer into internal storage. This ensures a replacement does not invalidate outstanding references held by callers.

| Language | Reference mechanism | Caller's copy |
| -------- | ------------------- | ------------- |
| **Rust** | `Arc<T>` | `.clone()` increments ref-count; old `Arc` stays alive until all holders drop it |
| **C** | Manual — no automatic lifetime | No automatic ref-counting. Use an explicit ownership contract or `retain`/`release` callback pair. On embedded targets, prefer static vtable structs with no heap involvement |
| **C++** | `std::shared_ptr<T>` | Copy constructor increments ref-count; `std::static_pointer_cast<T>` provides the typed copy |
| **TypeScript / JS** | GC object reference | Return the reference directly — GC keeps the object alive while any reference exists |
| **Java** | GC object reference | Return the reference directly — GC handles lifetime |
| **Kotlin** | JVM GC object reference | Return the reference directly — JVM GC handles lifetime |
| **C# / .NET** | GC object reference | Return the reference directly — GC handles lifetime |
| **Python** | GC object reference (CPython: ref-counted + cycle collector) | Return the object directly — ref-count keeps it alive |
| **PHP** | GC object reference (FPM: per-request scope) | Return the object directly; FPM scopes objects to the request lifecycle |
| **Swift** | ARC (Automatic Reference Counting) — like Rust's `Arc<T>` but implicit | Return the reference directly; ARC increments the retain count automatically on assignment |
| **Go** | Pointer + GC | Return the pointer — GC keeps the pointed-to value alive as long as any reference exists |

---

### Performance considerations

These apply to all language ports:

- **Minimise lock hold time** — acquire the lock only for the map insert or lookup; release before invoking the trace callback.
- **Prefer read-write locks for read-heavy workloads** — Rust `Mutex`, Go `sync.RWMutex`, Java `ReentrantReadWriteLock`, and .NET `ReaderWriterLockSlim` all support shared reads; use the shared-read variant when reads vastly outnumber writes.
- **Invoke trace callbacks outside the lock** — calling user-supplied code while holding the registry lock creates deadlock risk (if the callback re-enters the registry) and unnecessary contention.
- **Cache retrieved references locally for hot paths** — components that retrieve the same capability in a tight loop should obtain one reference and hold it for the loop's duration; the registry is designed for infrequent lookups (setup and hot-swap), not per-call resolution.

---

### Embedded and constrained environments

An L0 registry has no mandatory runtime dependencies — it is a map and a mutex. A C or Rust `no_std` implementation fits in a few kilobytes of flash. The embedded value proposition comes from the layer stack: firmware registers only the hardware-bound capabilities it owns (GPIO, ADC, sensors, actuators); all higher-level capabilities arrive as network proxy registrations from a connected edge node or development machine — **without reflashing**. The calling code is identical whether the capability runs on-chip or over the wire.

Prototyping workflow:

1. Flash firmware once — registers GPIO, ADC, Sensor
2. Connect device to development network
3. Register MLInference, Config, Logger as network proxies from the host — no reflash
4. Iterate on business logic at host speed
5. Move stabilised capabilities into firmware; calling code never changes

Language options for constrained targets: C with string keys (bare-metal/RTOS), Rust `no_std` with `heapless` collections, MicroPython for higher-level boards (ESP32, RP2040).

See [best-practices.md — Embedded Systems and IoT](best-practices.md#embedded-systems-and-iot) for the full ownership model, language options, and what changes on single-core bare-metal targets.
