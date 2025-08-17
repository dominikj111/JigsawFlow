<!-- markdownlint-disable MD036 -->
<!-- markdownlint-disable MD033 -->
<!-- markdownlint-disable MD023 -->
<!-- markdownlint-disable MD041 -->

<div align="center">
  <img src="logo.png" alt="JigsawFlow Logo" width="200" height="200">
  
  **JigsawFlow Microkernel** — A Capability-Driven Architecture for Offline-First, Hot-Swappable, Language-Agnostic Applications
  
  _An enterprise-grade pattern for building modular software systems_
  
  > ⚠️ **Work in Progress**: This is an evolving architectural pattern. The specification and examples are actively being developed and refined based on real-world implementations and community feedback.
  
[![GitHub Stars](https://img.shields.io/github/stars/dominikj111/JigsawFlow?style=flat-square)](https://github.com/dominikj111/JigsawFlow)
[![Issues](https://img.shields.io/github/issues/dominikj111/JigsawFlow?style=flat-square)](https://github.com/dominikj111/JigsawFlow/issues)
[![License](https://img.shields.io/badge/license-CC0%201.0%20%2B%20CC%20BY--SA%204.0-blue?style=flat-square)](LICENSE)
![Status](https://img.shields.io/badge/status-work%20in%20progress-yellow?style=flat-square)
</div>

## Overview

JigsawFlow is a revolutionary architecture pattern that transforms how enterprise applications are built and composed. Inspired by battle-tested industrial automation systems like PLCs and SCADA architectures, JigsawFlow enables developers to construct robust applications through **compound component composition** with minimal integration overhead.

**Core Philosophy**: Following the proven PLC paradigm, JigsawFlow applications emerge through the strategic composition of specialized components, each focused on solving a specific domain problem. Just as PLC components contribute distinct capabilities to form comprehensive industrial control systems, JigsawFlow components bring focused expertise—user management, data persistence, communication protocols—that collectively shape the overall application architecture.

Unlike traditional software architecture approaches that require extensive glue code and tight coupling, JigsawFlow applications emerge organically from reusable, interface-compliant components managed through a centralized singleton registry.

---

## What Makes JigsawFlow Different?

### **Singleton Registry Architecture vs Traditional DI**

JigsawFlow uses a singleton registry pattern, not traditional dependency injection:

**Traditional DI** (Constructor Injection):

```rust
// Framework-heavy, lifecycle management
container.register::<DatabaseService, PostgresImpl>();
let db = container.resolve::<DatabaseService>();
```

**JigsawFlow Singleton Registry** (Type-Based Singleton Access):

```rust
// Type-focused, hot-swappable singletons
registry.register::<Storage>(postgres_component);
let storage = registry.get::<Storage>();
```

### **Singleton Registry vs Dependency Injection**

JigsawFlow uses a **singleton registry pattern**, not traditional dependency injection:

**Traditional DI:**

- Constructor/setter injection with framework lifecycle management
- Compile-time dependency resolution and object graph construction
- Framework controls object creation and disposal
- Complex object graph management

**JigsawFlow Singleton Registry:**

- Global singleton store accessed by trait/interface/type
- Runtime component discovery and hot-swapping
- Service locator pattern with type safety
- Write-once, read-many access pattern optimized for performance
- Thread-safe singleton replacement without application restart

**Language-Specific Implementations:**

- **Rust**: Trait-based registry (`T: Send + Sync + 'static`)
- **Java/C#**: Interface-based registry
- **TypeScript**: Type-based registry
- **Go**: Interface-based registry

This approach enables the core JigsawFlow benefits: hot-swappable components, runtime composition, and zero-restart component replacement. The registry functions as a global service locator where components register their implementations and discover the services they need.

### **Unix Microkernel Principles at Application Level**

JigsawFlow applies proven Unix design principles to application architecture:

| **Unix Principle**                 | **JigsawFlow Application**                       |
| ---------------------------------- | ------------------------------------------------ |
| "Everything is a file"             | "Everything is a capability"                     |
| Small tools that do one thing well | Small components with focused responsibilities   |
| Compose via pipes                  | Compose via singleton registry                   |
| Process independence               | Component independence with graceful degradation |
| Hot-swappable kernel modules       | Hot-swappable application components             |

### **Emergent System Composition**

The real innovation isn't just modularization—it's **emergent composition**:

- **Traditional Architecture**: Explicitly designed, components know about each other
- **JigsawFlow Architecture**: Applications emerge from available capabilities, components declare needs rather than dependencies

This creates a **userspace microkernel** where complex applications arise from simple, composable primitives—just like Unix achieved emergent complexity from basic process and file abstractions.

---

## Why JigsawFlow?

### **Industrial-Strength Foundation**

JigsawFlow draws inspiration from proven industrial automation patterns:

- **PLC Systems**: Modular, signal-driven components with standardized interfaces
- **SCADA Architecture**: Centralized control with distributed, autonomous components
- **Component-Based Software Engineering (CBSE)**: Reusable, qualified components with defined interfaces
- **Microservice Orchestration**: Loosely coupled services with clear boundaries

### **Enterprise Benefits**

- **Rapid Application Assembly**: Build complex systems by composing pre-built components
- **Zero-Restart Hot-Swapping**: Replace functionality without application downtime (Phase 1 supports DI singleton replacement; Phase 2 adds dynamic library loading via RuntimeSwap)
- **Polyglot System Architecture**: Build cross-language applications through communication components (Bluetooth, P2P, TCP/IP, UDP, Modbus) that enable JigsawFlow implementations across Java, C#, JavaScript/TypeScript, Rust, and other languages to interoperate seamlessly
- **Minimal Integration Overhead**: Singleton registry handles component coordination
- **Community-Driven Ecosystem**: Shared component repository for common functionality

---

## Core Architecture

### **Singleton Registry**

The heart of JigsawFlow is a trait/interface-based singleton registry - a global singleton store that returns instances by trait/interface/type - that:

- Registers components by their service interfaces, not concrete types
- Enables singleton replacement without application restart
- Provides language-agnostic service discovery
- Functions as a thread-safe service locator for component services
- Supports trait registry (Rust), interface registry (Java/C#), and type registry (TypeScript) patterns

### **Component vs Plugin Distinction**

JigsawFlow uses precise terminology to distinguish between architectural and deployment concepts:

**JigsawFlow Component:**

- Core architectural element that registers with the singleton registry
- Provides specific capabilities through well-defined interfaces
- Behaves like a PLC component with focused responsibilities
- Can be hot-swapped at runtime without application restart
- Examples: Storage Component, Authentication Component, Logging Component

**Plugin:**

- Deployment package that extends application functionality
- May contain one or more components along with additional resources
- Physical distribution mechanism (files, libraries, packages)
- Brings complete solutions including configs, assets, documentation
- Examples: User Management Plugin (containing Auth Component + Profile Component + UI assets)

**Relationship:** Plugins are containers that deliver components to applications. A plugin typically packages related components together with their supporting resources for easy installation and distribution.

### **Component Interface Compliance**

Every JigsawFlow component implements standardized interfaces for:

- **Interface Definition**: What services the component provides/consumes
- **Component Lifecycle**: Component registration, initialization and cleanup

### **Compound Application Assembly**

Applications are built through **additive composition**:

1. Start with minimal core (singleton registry + main thread component)
2. Add functionality by installing interface-compliant components
3. Components self-register their capabilities upon loading (components can focus on or contain multiple solutions)

### **The Requirements**

JigsawFlow components must adhere to three fundamental architectural constraints:

- **Offline-First Design**: Components must function when network connectivity is lost (WiFi down, cables cut) but may utilize network protocols when available
- **Component Independence**: Components must not directly depend on other components. For shared functionality, prefer extracting it into a separate shared component; when dependencies are unavailable, components should log and degrade gracefully
- **Facade Pattern**: All external dependencies (file I/O, environment access, system calls) must be wrapped through singleton registry facades

_See [best-practices.md](best-practices.md) for detailed implementation guidance and testing strategies._

---

## Use Cases

### **Enterprise Application Development**

Transform monolithic applications into composable, maintainable systems:

- **API Gateways**: HTTP + Authentication + Logging + Monitoring components
- **Data Processing Pipelines**: Input + Transform + Output + Persistence components
- **IoT Platforms**: Device Communication + Data Collection + Analytics components

### **Industrial Automation**

Leverage familiar PLC-style programming for software systems:

- **Process Control**: Sensor Input + Logic Processing + Actuator Output components
- **SCADA Integration**: Data Acquisition + Supervisory Control + HMI components
- **Manufacturing Execution**: Workflow + Quality Control + Reporting components

### **Desktop & GUI Applications**

Transform traditional desktop development through distributed GUI architecture:

- **Cross-Platform Applications**: GUI Rendering + Business Logic + Data Persistence components
- **Development Tools**: Code Editor + Compiler + Debugger + GUI Dashboard components
- **System Administration**: Monitoring + Configuration + GUI Interface components
- **Network-Distributed Apps**: Remote business logic + Local GUI rendering via P2P connections

### **Microservice Orchestration**

Simplify complex distributed system management:

- **Service Mesh**: Discovery + Load Balancing + Circuit Breaking components
- **Event Processing**: Message Routing + Stream Processing + State Management components
- **DevOps Automation**: CI/CD + Monitoring + Alerting + Deployment components

---

## Positioning vs. known patterns

| Pattern                          | What it is                          | Where JigsawFlow differs                                                                    |
| -------------------------------- | ----------------------------------- | ------------------------------------------------------------------------------------------- |
| Microkernel / Plug‑in            | Minimal core with plug‑ins          | You formalize plug‑in capabilities and offline‑first guarantees                             |
| Hexagonal (Ports & Adapters)     | Domain wrapped by ports             | Ports/adapters map to service interfaces; singleton registry discovers/binds                |
| Service‑oriented / Microservices | Network‑separated services          | JigsawFlow can be in‑proc or cross‑proc; components are composable without service overhead |
| Actor model                      | Isolated entities exchange messages | Your components can adopt actors internally; the bus covers inter‑component traffic         |
| OSGi/Module systems              | Runtime module lifecycles           | You keep it language‑agnostic and simpler (no classloader tricks)                           |

---

## Future Enhancements

### **Language-Agnostic Interface Standards**

Establish a comprehensive collection of standardized traits/interfaces for common functionality across all supported languages. This standardization addresses two critical areas:

**1. Singleton Registry Standards**

- Define problem-solution contracts through standardized trait/interface definitions
- Enable community-driven component development where components provide solutions to well-defined problems
- Create language-agnostic specifications that translate to idiomatic implementations in each target language
- Establish component certification and compatibility frameworks

**2. Communication Protocol Definitions**

- Standardize event-driven communication patterns across protocol boundaries
- Define message shapes and interface contracts for inter-component communication
- Enable components to send events implementing specific traits/interfaces while other components listen for compatible event types
- Support both intra-application and cross-application communication patterns

This approach mirrors industrial automation standards, where discussed contracts enable polyglot systems through well-defined, battle-tested interface specifications.

### **GUI-as-a-Service Architecture**

**Distributed GUI Rendering Capabilities**

JigsawFlow enables revolutionary GUI architecture where applications become pure business logic while GUI rendering becomes a dedicated capability component:

- **Contract-Based GUI Rendering**: Applications send declarative UI specifications via singleton registry events, eliminating the need for GUI libraries in business logic
- **Language-Agnostic GUI Services**: Backend services in Rust, Python, Go, etc. leverage unified GUI infrastructure without language-specific bindings
- **Hot-Swappable UI Components**: GUI components update independently from application logic, enabling live UI theming and layout changes
- **Network-Distributed Applications**: P2P secure connections enable GUI components to run on different machines—applications become truly distributed without installation requirements
- **WorkFlows OS Integration**: GUI service components serve as core system services, providing unified desktop experiences across all applications

**Revolutionary Distribution Model**

The most exciting aspect: applications no longer require local installation. Through secure P2P networking:

- Business logic runs on remote nodes
- GUI renders locally via service contracts
- Applications distribute dynamically across network topology
- Zero-installation application ecosystem emerges naturally

This transforms software distribution from "install and run" to "connect and compose"—applications become network-native capabilities that assemble on-demand.

### **Dynamic Loading Capabilities**

**RuntimeSwap Component Development**

- Hot-loading of dynamic libraries without application restart
- Advanced component versioning and compatibility management
- Runtime component replacement with dependency resolution
- Enhanced security models and sandboxing options for untrusted components
- Performance optimization for dynamic loading scenarios

### **Ecosystem Expansion**

**Community Infrastructure**

- Central component registry with discovery and distribution capabilities
- Community contribution frameworks with quality assurance processes
- Cross-language component bridging and interoperability testing
- Advanced orchestration and monitoring tools for complex component-based systems
- Developer tooling for component creation, testing, and debugging

**Component Governance & Security**

- **Official Component Certification**: Community-vetted components marked as "JigsawFlow Official" (similar to Docker Official Images)
- **Community Voting System**: Democratic selection of idiomatic solutions for common problems
- **Security Inspection Process**: Peer review and automated security scanning for component validation
- **Standard Solution Registry**: Community-defined canonical approaches (similar to WooCommerce in WordPress ecosystem)
- **Trust Levels**: Graduated trust system from community contributions to enterprise-certified components

---

## Architecture Comparison

| Pattern        | Coupling              | Hot-Swap   | Multithreading | Communication Types | Interface Focus   | Industrial Heritage |
| -------------- | --------------------- | ---------- | -------------- | ------------------- | ----------------- | ------------------- |
| **JigsawFlow** | Loose (DI Registry)   | ✅ Yes     | ✅ Native      | ✅ Multi-Protocol   | ✅ Trait-based    | ✅ PLC/SCADA        |
| PLC Systems    | Loose (Signal Bus)    | ⚠️ Limited | ❌ Single      | ⚠️ Signal-based     | ✅ Standard I/O   | ✅ Industrial       |
| Traditional DI | Medium (Framework)    | ❌ No      | ⚠️ Framework   | ❌ In-Process       | ⚠️ Type-based     | ❌ No               |
| Microservices  | Loose (Network)       | ✅ Yes     | ✅ Per Service | ⚠️ Network Only     | ⚠️ API-based      | ❌ No               |
| Plugin Systems | Medium (Callbacks)    | ⚠️ Limited | ⚠️ Host App    | ❌ Callbacks        | ❌ Implementation | ❌ No               |
| ECS Pattern    | Tight (Entity Tables) | ❌ No      | ⚠️ System      | ❌ Component Data   | ❌ Component      | ❌ No               |

---

## 📚 Documentation

This repository contains comprehensive documentation to help you understand and implement JigsawFlow:

### **Core Documentation**

- **[📖 README.md](README.md)** _(this document)_ - Project overview, architecture principles, and getting started guide
- **[🏗️ Best Practices](best-practices.md)** - Implementation patterns, architectural requirements, and development guidelines for building robust JigsawFlow applications
- **[💡 Implementation Examples](implementation-examples.md)** - Real-world examples demonstrating JigsawFlow patterns across different technologies and domains

### **Quick Navigation**

| Document                       | Purpose                            | Best For                    |
| ------------------------------ | ---------------------------------- | --------------------------- |
| **README.md**                  | Architecture overview & philosophy | Understanding core concepts |
| **best-practices.md**          | Implementation guidance & patterns | Building production systems |
| **implementation-examples.md** | Practical code examples            | Learning through examples   |

---

## Getting Started

### **For Developers**

1. **Explore Examples**: Review reference implementations in your preferred language _(implementations in progress)_
2. **Define Interfaces**: Create trait/interface definitions for your domain
3. **Build Components**: Implement interface-compliant components
4. **Compose Applications**: Use DI registry to assemble functionality

> **Note**: Reference implementations are currently being developed. See [implementation-examples.md](implementation-examples.md) for current progress and conceptual examples.

### **For Enterprises**

1. **Assess Current Architecture**: Identify monolithic components suitable for component-based composition
2. **Plan Migration Strategy**: Design interface boundaries and component responsibilities
3. **Pilot Implementation**: Start with non-critical system components
4. **Scale Adoption**: Expand component-based approach across application portfolio

### **For Contributors**

1. **Join Community**: Participate in architecture discussions and RFC process
2. **Develop Components**: Create reusable components for common enterprise needs
3. **Improve Tooling**: Enhance developer experience and debugging capabilities
4. **Share Knowledge**: Write tutorials, case studies, and best practices

---

## Community & Ecosystem

### **Success Metrics**

- **GitHub Engagement**: Stars, issues, and community contributions
- **Component Ecosystem**: Number of available components and their adoption
- **Enterprise Adoption**: Applications successfully built using JigsawFlow pattern
- **Cross-Language Support**: Implementation across multiple programming languages

### **Development Roadmap**

- **Q1 2025**: Core pattern specification and reference implementation _(in progress)_
- **Q2 2025**: Developer tooling and documentation _(in progress)_
- **Q3 2025**: RuntimeSwap component and dynamic loading capabilities _(planned)_
- **Q4 2025**: Central registry and community contribution platform _(planned)_

> **Current Focus**: Refining core architectural patterns and developing reference implementations based on real-world usage in production systems.

---

## Contributing

We welcome contributions from developers, architects, and industrial automation experts. Whether you're building components, improving documentation, or sharing use cases, your input helps shape the future of component-based application architecture.

**Get Involved**:

- 📖 Read our [Contributing Guidelines](CONTRIBUTING.md)
- 💬 Join discussions in [GitHub Issues](https://github.com/dominikj111/JigsawFlow/issues)
- 🔧 Submit pull requests for improvements
- 📝 Share your JigsawFlow success stories

---

## License

**Code:** JigsawFlow is released under the [CC0 1.0 Universal (Public Domain Dedication)](https://creativecommons.org/publicdomain/zero/1.0/).  
This dedicates the code to the public domain—**you can copy, modify, distribute, and use it for any purpose, without any restrictions or attribution requirements**.

**Documentation and Examples:** All documentation, tutorials, and example materials are released under the [Creative Commons Attribution-ShareAlike 4.0 International License (CC BY-SA 4.0)](https://creativecommons.org/licenses/by-sa/4.0/).  
You are free to share and adapt these materials **as long as you give appropriate credit and distribute any derivative works under the same license**.

We believe in open, collaborative development that benefits the entire software engineering community.

---

**Ready to revolutionize your application architecture?** Start exploring JigsawFlow today and discover the power of industrial-grade modular composition.
