# JigsawFlow Examples

Runnable code demonstrating JigsawFlow capability-driven composition across languages.

## What's covered

Each language directory implements the same three scenarios, mirroring the article
[*From Factory Floor to Codebase*](https://dominikj111.github.io/blog/capability-driven-module-composition-in-rust/):

| Scenario | What it shows |
|---|---|
| **Capability contract** | Define a trait/interface, resolve via registry, degrade gracefully if absent |
| **Runtime swap** | Replace an implementation mid-execution; consumers unchanged |
| **Testing without mocks** | Register test doubles directly; no mock library needed |

## Language examples

- [`rust/`](rust/) — reference implementation using [`singleton-registry`](https://github.com/dominikj111/singleton-registry)

## Adding a new language

1. Create `examples/<language>/` as a self-contained project (its own `package.json`, `go.mod`, etc.)
2. Implement the three scenarios above using a singleton registry primitive for that language
3. Name the scenario files to match: `01_capability_contract`, `02_runtime_swap`, and a test for `testing_without_mocks`
4. Add a brief `README.md` covering only how to run (see [`rust/README.md`](rust/README.md) as template)
5. Add an entry to the table above
