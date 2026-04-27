<!-- markdownlint-disable MD024 -->

# JigsawFlow Best Practices

Implementation patterns and techniques for building robust, efficient JigsawFlow applications.

---

## Core Architectural Requirements

### Offline-First Design

All JigsawFlow components must be designed to function independently without external dependencies or network connectivity.

**Mandatory Requirements:**

- Components must operate fully when disconnected from networks or external services
- Implement graceful degradation when external resources are unavailable
- Cache essential data locally to maintain functionality during outages
- Design for eventual consistency rather than real-time synchronization
- Provide meaningful functionality even with stale or limited data

**Benefits:**

- Enhanced reliability in unstable network environments
- Improved user experience during connectivity issues
- Simplified testing without external service dependencies
- Better performance through local data access
- Increased system resilience and fault tolerance

### Component Independence Principle

Components must never directly depend on other components. Any shared functionality must be unified into a single component.

**Strict Requirements:**

- **No Inter-Component Dependencies**: Components cannot directly reference or import other components
  **Shared Functionality Guidance**: If components need shared functionality, prefer extracting it into a separate shared component or refactor boundaries. Avoid tight coupling; prefer interfaces and graceful degradation
  **Communication Flexibility**: Use the singleton registry, event-driven messaging, channels, IPC, or other appropriate mechanisms—no single method is prescribed
- **Interface-Based Coupling**: Components depend only on traits/interfaces, never concrete implementations

**Enforcement Guidelines:**

- Analyze component boundaries during design to prevent dependency coupling
- Refactor shared code into common components rather than creating dependencies
- Use composition over inheritance to avoid component coupling
- Design components as completely self-contained elements

### Facade Pattern for External Dependencies

All language standard utilities and environmental access must be wrapped through facade functions registered in the singleton registry.

**Mandatory Wrapping:**

- **File System Access**: Wrap all file I/O operations through singleton registry facades
- **Network Operations**: Abstract all network calls behind interface facades
- **Environment Variables**: Access system environment through singleton registry functions
- **Time/Date Functions**: Wrap time operations for consistent testing and mocking
- **Random Number Generation**: Abstract randomness for deterministic testing
- **System Calls**: Wrap all OS-specific operations through facades

**Testing Benefits:**

- **Complete Mockability**: Every external dependency can be mocked for testing
- **Deterministic Testing**: Control all external inputs for reproducible test results
- **Isolated Unit Tests**: Test module logic without external system dependencies
- **Simplified Test Setup**: No complex mocking frameworks required
- **Natural Service Substitution**: Singleton registry provides seamless mock substitution

**Implementation Pattern:**

1. Create facade interfaces for all external operations
2. Register concrete implementations in singleton registry during normal operation
3. Register mock implementations during testing
4. Components access external functionality only through singleton registry facades

## Core Implementation Patterns

### Lazy Loading & Caching

Any singleton should implement its own efficient resource management through lazy initialization and intelligent caching strategies.

Future: a thin abstraction will standardize lazy initialization and caching patterns across languages.

#### Lightweight Initialization Pattern

Create component structures that defer expensive operations until first access.

Pre-calculation can be triggered via a deliberate first access to warm caches.

**Pattern Overview:**

- `::new()` or equivalent constructor creates minimal structure
- Expensive resources (configuration files, database connections, remote data) load on demand
- Results cached at component level for subsequent access
- Particularly effective for configuration components requiring remote downloads

**Benefits:**

- Faster application startup times
- Reduced memory footprint during initialization
- Improved fault tolerance (failures occur at access time, not startup)
- Better resource utilization in multi-component applications

**Use Cases:**

- Configuration components downloading from remote sources
- Database connection pools
- External API clients
- Large data structure initialization

#### Caching Strategies

**Component-Level Caching:**

- Cache expensive computations within component boundaries
- Implement cache invalidation for dynamic data
- Consider memory constraints in long-running applications

---

### Builder Pattern Integration

Leverage the singleton registry's function registration capabilities to implement flexible object construction patterns.

#### Factory Function Registration

Register builder functions as first-class citizens in the singleton registry:

**Simple Builders:**

- Register factory functions that construct objects with default configurations
- Enable parameterized construction through closure capture

**Complex Multi-Step Builders:**

- Break construction into discrete, testable steps
- Register intermediate builder stages as separate functions
- Enable builder composition through singleton registry coordination

#### Builder Pattern Benefits

**Flexibility:**

- Construct objects with varying configurations
- Support optional parameters and default values
- Enable runtime configuration of object construction

**Testability:**

- Mock individual construction steps
- Test builder logic independently from constructed objects
- Verify construction parameters through builder inspection

**Maintainability:**

- Separate construction logic from object implementation
- Enable construction pattern reuse across components
- Support construction pattern evolution without breaking existing code

---

## Component Design Patterns

### Single Responsibility Components

Design components with focused, well-defined purposes following the PLC component paradigm:

**Guidelines:**

- Each component should solve one specific domain problem
- Avoid feature creep within component boundaries
- Prefer component composition over monolithic component design
- Design for replaceability and hot-swapping

### Interface-First Development

Define component contracts before implementation:

**Process:**

1. Define trait/interface specifications for component capabilities
2. Establish input/output contracts and data formats
3. Explore generic trait/interface collections—components may interoperate in ways not originally anticipated
4. Implement component logic to satisfy interface contracts
5. Register component capabilities through singleton registry

**Benefits:**

- Clear component boundaries and responsibilities
- Enhanced testability through interface mocking
- Simplified component replacement and versioning
- Better cross-language compatibility

### Dependency Declaration

Explicitly declare component dependencies through the singleton registry:

**Patterns:**

- Request dependencies by trait/interface, not concrete types
- Handle missing dependencies gracefully with fallback strategies
- Minimize dependency coupling through interface abstraction
- Support optional dependencies for enhanced functionality

---

## Performance Optimization

### Component Loading Strategies

**Static Loading (Phase 1):**

- Load all required components during application initialization
- Optimize for predictable startup performance
- Suitable for applications with known component requirements

**Dynamic Loading (Phase 2 - RuntimeSwap):**

- Load components on-demand based on runtime requirements
- Optimize for memory usage and startup time
- Handle loading failures and fallback scenarios

### Communication Optimization

**Intra-Application Communication:**

- Prefer direct singleton registry access for same-process components
- Minimize serialization overhead for local communication
- Use efficient data structures for high-frequency interactions

**Cross-Application Communication:**

- Choose appropriate protocols based on performance requirements
- Implement connection pooling for network-based communication
- Consider message batching for high-throughput scenarios

### Memory Management

These are generic recommendations for software development, not tied to the JigsawFlow pattern

**Component Lifecycle:**

- Implement proper cleanup in component shutdown procedures
- Release expensive resources when components are replaced
- Monitor memory usage in long-running applications

**Resource Sharing:**

- Use singleton registry for sharing expensive resources across components
- Consider resource pooling for frequently created/destroyed objects

---

## Error Handling & Resilience

### Graceful Degradation

Design components to handle dependency failures gracefully:

**Strategies:**

- Implement fallback functionality when dependencies are unavailable
- Provide reduced functionality rather than complete failure
- Log dependency issues for monitoring and debugging

### Component Isolation

Prevent component failures from cascading across the application:

**Techniques:**

- Implement error boundaries around component operations
- Isolate component state to prevent cross-contamination

- Use circuit breaker patterns (if appropriate) to detect and prevent cascading failures when external dependencies become unavailable; this allows the system to recover more quickly from outages and reduces the load on the failed dependency during the outage

### Hot-Swap Safety

Ensure safe component replacement during runtime:

**Guidelines:**

- Implement proper cleanup procedures before component replacement
- Handle in-flight operations during component swapping
- Validate new component compatibility before activation
- Provide rollback mechanisms for failed component updates

**Distributed hot-swap — drain contract:**

Within a single runtime, the reference-counting model (Arc in Rust, GC in TypeScript/JS) guarantees that in-flight callers holding a reference to the old implementation finish naturally before the old object is released. Across runtimes this guarantee does not apply automatically — the old capability provider process could be shut down while remote callers still have in-flight requests.

In a multi-runtime setup, the component lifecycle must include an explicit drain step:

1. New provider registers on the singleton network (new requests route to it immediately)
2. Old provider receives a "draining" signal from the CLI/daemon — it stops accepting new requests but finishes in-flight work
3. Old provider signals completion and shuts down

The drain signal is a lifecycle event orchestrated by the daemon, not a registry primitive. The registry's role ends at step 1.

---

## Multi-Runtime Considerations

The singleton registry API is intentionally identical across all supported languages. This is a feature for single-runtime JigsawFlow apps — all components share one registry and see the same registered values. In multi-runtime setups it creates a non-obvious trap.

### The ownership rule

**Each capability has exactly one owning runtime.** The owner is the source of truth; it is the only runtime that registers the real implementation. All other runtimes that need that capability register a *proxy* implementation of the same contract — a local object that forwards calls to the owner over the transport layer.

```text
Rust runtime (owner)          Node.js runtime
──────────────────────        ──────────────────────
Config (real impl)            Config → RustConfigProxy
Storage (real impl)           Storage → RustStorageProxy
                              HttpServer (real impl)
```

The proxy is explicit about what it does: it goes over the wire. There is no illusion of local state.

### The trap: same contract registered independently in two runtimes

If both Rust and Node.js register their own `Config` without one delegating to the other, they hold two unrelated objects. Updates in one runtime are invisible to the other. No error is raised — the registries are working correctly. The bug is the architecture.

The registry API looking identical in all languages makes this mistake easy to make. The rule — one owner, others use proxies — must be a deliberate architectural decision, not something the registry enforces automatically.

### Local vs. network capabilities

The proxy pattern is also the right model when you want *both* a local and a remote version of the same capability (for example, a local `Config` for offline operation and an authoritative remote `Config` for synchronization). Register them under different contract tokens. The resolution policy — which one to use when — lives in the component that consumes them, not in the registry.

### The singleton network (future)

The singleton network layer, described in the Future Enhancements section of the README, automates proxy generation. When `registry.get(Config)` finds nothing locally, the network layer discovers which runtime owns `Config` and returns a transparent proxy. The ownership rule still applies; the difference is that proxies are generated automatically rather than written manually. The async boundary remains explicit: network-resolved capabilities return a `Promise`/`Future` rather than a synchronous value.

### Embedded Systems and IoT

The L0 registry is just a map and a mutex — no OS, no allocator, no network required. A minimal implementation in C or Rust `no_std` fits in a few kilobytes of flash. This makes JigsawFlow directly applicable to microcontrollers, RTOS targets, and IoT devices, with a unique advantage: **the device can extend its own capabilities at runtime without reflashing**.

#### The core idea: own hardware, proxy everything else

Firmware registers only the capabilities bound to the silicon it runs on — GPIO control, ADC reads, sensor drivers, actuator outputs. Any capability not baked into the firmware (ML inference, remote configuration, structured logging, OTA orchestration) is registered as a network proxy from a connected edge node or development machine. The device's application code calls `registry.get(MLInference)` — it does not know or care whether the implementation runs on-chip or over a serial/IP link.

```text
Embedded device (owner)        Edge node / development machine
────────────────────────       ────────────────────────────────
GPIO        (real impl)        GPIO        → DeviceGPIOProxy
ADC         (real impl)        ADC         → DeviceADCProxy
Sensor      (real impl)        Sensor      → DeviceSensorProxy
                               MLInference (real impl)
                               Config      (real impl)
                               Logger      (real impl)
```

#### Prototyping workflow

1. Flash firmware once — it registers GPIO, ADC, Sensor and nothing else
2. Connect device to development network (serial, USB-CDC, Wi-Fi, BLE)
3. Register higher-level capabilities as network proxies from the host — no reflash
4. Iterate on business logic at host speed; swap proxy implementations freely
5. As capabilities stabilise, embed them in firmware; application code never changes

This decouples the firmware release cycle from the product iteration cycle. The firmware is stable hardware glue; business logic evolves independently.

#### Language options for constrained targets

- **C with string keys** — suitable for bare-metal and RTOS environments (FreeRTOS, Zephyr, bare Cortex-M). No RTTI available; the token is a `const char *` capability name (or its hash). The contract is a struct of function pointers (vtable pattern). Store vtable structs in static memory — the registry does not own or free them. Use `pthread_mutex_t`, C11 `mtx_t`, or the RTOS mutex primitive for thread safety; on a single-core bare-metal target, an interrupt-disable pair is sufficient.

- **Rust `no_std`** — all JigsawFlow semantics are preserved. Replace `std::collections::HashMap` with `heapless::FnvIndexMap` (fixed-capacity, no heap), and `std::sync::Mutex` with the RTOS primitive (e.g. `cortex_m::interrupt::free` or an RTOS-provided mutex). The `TypeId`-based token mechanism works unchanged in `no_std` since `core::any::TypeId` is available.

- **MicroPython** — for higher-level embedded platforms (ESP32, RP2040 with MicroPython). Python classes are first-class objects and valid dictionary keys; the Python port of the registry applies directly with no adaptation needed.

#### What changes on constrained targets

- **Fixed capacity** — dynamic allocation is often unavailable on bare-metal. `register()` should return an error (or assert) when the backing store is full; document the maximum number of registered capabilities at compile time.
- **No-heap vtables** — in C, place capability vtable structs in `static const` memory. Passing a stack pointer into the registry is a bug; the registry stores a pointer but does not extend lifetime.
- **Interrupt-safe locking** — on single-core bare-metal, the only concurrency source is interrupt handlers. An interrupt-disable/enable pair around register/get is simpler and lower-overhead than a mutex.
- **Drain contract still applies** — when a firmware capability is updated (e.g. a sensor driver is hot-swapped to a new calibration), the edge-node drain protocol still governs in-flight RPCs. The firmware holds the hardware capability; the daemon signals it to finish in-flight work before the new implementation is activated.

---

## Testing Strategies

### Component Testing

**Isolation Testing:**

- Test components in isolation using mocked dependencies
- Verify interface contract compliance
- Test error handling and edge cases

**Integration Testing:**

- Test component interactions through singleton registry
- Verify communication patterns and data flow
- Test component replacement and hot-swapping scenarios

### Builder Pattern Testing

**Factory Function Testing:**

- Test builder functions independently
- Verify constructed object properties and behavior
- Test builder parameter validation and error handling

**Construction Process Testing:**

- Test multi-step construction workflows
- Verify intermediate state handling
- Test construction failure recovery

---

## Security Considerations

### Component Sandboxing

**Trust Boundaries:**

- Establish clear trust levels for different component sources
- Implement sandboxing for untrusted components (future enhancement)
- Validate component signatures and integrity

### Communication Security

**Protocol Security:**

- Use encrypted communication channels for sensitive data
- Implement authentication and authorization for cross-application communication
- Validate message integrity and prevent tampering

### Resource Access Control

**Capability-Based Security:**

- Limit component access to required resources only
- Implement fine-grained permission systems
- Monitor and log resource access patterns

---

## Monitoring & Debugging

### Component Observability

**Logging Strategies:**

- Implement structured logging within components
- Include component identity and version information in logs
- Use correlation IDs for cross-component operation tracking

**Metrics Collection:**

- Monitor component performance and resource usage
- Track component loading and replacement events
- Measure communication latency and throughput

### Debugging Techniques

**Component Inspection:**

- Implement component state inspection capabilities
- Provide debugging interfaces for component introspection
- Support runtime component configuration changes

**Communication Debugging:**

- Log inter-component communication for troubleshooting
- Implement message tracing across component boundaries
- Provide tools for communication pattern visualization

---

## Migration Strategies

### The JigsawFlow Migration Advantage

JigsawFlow's greatest competitive advantage is **seamless legacy integration**. Unlike architectural patterns that require complete rewrites or extensive refactoring, JigsawFlow enables gradual adoption through interface wrapping—you can begin using JigsawFlow principles immediately without touching existing code.

**Key Migration Principle**: Any existing system, service, or application can become a JigsawFlow component by simply wrapping it with a trait/interface that registers with the singleton registry. The existing code remains completely unchanged.

### Interface Wrapping Pattern

**The Simplest Migration Path:**

1. **Identify Existing Functionality**: Choose any existing service, library, or application module
2. **Define Component Interface**: Create a trait/interface that represents the functionality you want to expose
3. **Create Wrapper Component**: Implement the interface by delegating calls to your existing code
4. **Register with Singleton**: Add the wrapper component to your singleton registry
5. **Begin Using Component**: Other parts of your application can now access this functionality through the registry

**Zero Risk Migration**: Your existing code never changes. The wrapper simply provides a JigsawFlow-compatible interface to functionality that already works.

### Legacy System Integration

**Incremental Adoption Strategy:**

- **Start with Non-Critical Systems**: Begin with components that have minimal dependencies and clear boundaries
- **Interface-First Approach**: Define clean interfaces for existing functionality without modifying the underlying implementation
- **Side-by-Side Operation**: JigsawFlow components operate alongside existing systems—no forced migration timeline
- **Gradual Capability Enhancement**: Add new features as native JigsawFlow components while legacy systems continue unchanged

**Wrapper Components in Practice:**

- **Database Access**: Wrap existing ORM or database layers behind storage interface traits
- **External APIs**: Wrap HTTP clients or SDK calls behind service interface traits  
- **Authentication Systems**: Wrap existing auth libraries behind authentication interface traits
- **Logging Infrastructure**: Wrap existing logging systems behind standardized logging interface traits
- **Configuration Management**: Wrap existing config systems behind configuration interface traits

**Benefits of Wrapper Approach:**

- **Immediate ROI**: Start gaining JigsawFlow benefits (hot-swapping, testability, modularity) without rewriting code
- **Risk Mitigation**: Existing functionality continues working exactly as before
- **Team Adoption**: Developers can learn JigsawFlow patterns without pressure to rewrite stable systems
- **Business Continuity**: No disruption to existing business processes or system stability

### Advanced Migration Patterns

**Strangler Fig Pattern:**

Once wrapper components are established, gradually replace internal implementations:

1. **External Interface Remains**: The trait/interface stays the same
2. **Internal Implementation Evolves**: Replace wrapped legacy code with native JigsawFlow implementations piece by piece
3. **Transparent to Consumers**: Other components see no difference during the transition
4. **Rollback Safety**: Can revert to wrapped legacy implementation if needed

**Component Extraction Process:**

```text
Monolithic Application
    ↓ (extract interface)
Monolith + JigsawFlow Wrapper Component
    ↓ (gradual internal replacement)  
Monolith + Native JigsawFlow Component
    ↓ (remove monolith dependency)
Pure JigsawFlow Component
```

**Multi-Phase Evolution:**

- **Phase 1**: Interface wrapping (immediate benefits, zero risk)
- **Phase 2**: Partial native implementation (improved performance, better integration)
- **Phase 3**: Full native implementation (maximum JigsawFlow benefits)
- **Phase 4**: Cross-language components (polyglot advantages)

### Monolith Decomposition

**Boundary Identification Without Rewriting:**

- **Interface Discovery**: Analyze existing system calls to identify natural component boundaries
- **Wrapper-First Decomposition**: Create interfaces around existing modules before extraction
- **Gradual Boundary Enforcement**: Use interfaces to enforce separation without immediate code changes
- **Data Access Patterns**: Identify shared data access for eventual component ownership

**Safe Decomposition Strategy:**

1. **Wrap Before Extracting**: Always create interface wrappers before moving code
2. **Test Interface Contracts**: Verify wrapped functionality works through new interfaces  
3. **Extract with Confidence**: Move code to separate components only after interface validation
4. **Maintain Compatibility**: Keep wrapper interfaces during transition for rollback safety

**Data Migration Considerations:**

- **Shared Database Access**: Wrapper components can share existing database connections initially
- **Gradual Data Ownership**: Transfer data responsibility to appropriate components over time
- **Transaction Boundaries**: Maintain existing transaction patterns during migration
- **Consistency Strategies**: Implement eventual consistency as components become truly independent

### Enterprise Migration Timeline

**Typical Migration Journey:**

#### Week 1-2: Interface Wrapping

- Identify 3-5 key system capabilities
- Create wrapper components with clean interfaces
- Register with singleton registry
- Begin using registry-based access patterns

#### Month 1-3: Expansion and Testing

- Wrap additional system capabilities
- Implement hot-swapping for wrapped components
- Validate component independence and testability
- Train team on JigsawFlow patterns

#### Month 3-6: Native Implementation

- Replace wrapper internals with native JigsawFlow implementations
- Improve performance and integration capabilities
- Add component-specific enhancements unavailable in legacy systems

#### Month 6+: Advanced Features

- Implement cross-language components
- Add dynamic loading and RuntimeSwap capabilities
- Build component ecosystem and reuse across applications

**Success Metrics:**

- **Development Velocity**: Faster feature development through component reuse
- **System Reliability**: Improved fault isolation and graceful degradation  
- **Team Productivity**: Reduced integration overhead and testing complexity
- **Technical Debt**: Gradual modernization without disrupting existing functionality

### Migration Risk Assessment

**Low Risk Migration Elements:**

- ✅ Interface wrapping existing functionality
- ✅ Side-by-side component operation
- ✅ Gradual adoption with rollback capability
- ✅ No changes to existing working code

**Medium Risk Migration Elements:**

- ⚠️ Replacing wrapper internals with native implementations
- ⚠️ Changing data access patterns
- ⚠️ Modifying team development workflows

**High Risk Migration Elements:**

- ❌ Complete system rewrites
- ❌ Big-bang migrations
- ❌ Forced timeline adoption

**JigsawFlow Minimizes Risk**: The wrapper-first approach ensures that high-risk migration elements are optional optimizations rather than required changes.

---

> ⚠️ **Work in Progress**: This document provides practical guidance for implementing JigsawFlow applications effectively and is actively being developed and refined based on real-world implementations and community feedback. As the pattern evolves and the community grows, these practices will continue to be refined based on real-world experience and feedback.
