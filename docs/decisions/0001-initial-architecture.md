# ADR 0001: Initial architecture

- Status: Accepted
- Date: 2026-09-27
- Basis: existing v0.1 draft and the repository governance foundation
- Related: [charter](../PROJECT_CHARTER.md), [current architecture](../ARCHITECTURE.md), [specification](../../spec/README.md)

## Context

Universal Source seeks portable content sources across mobile, TV, and desktop hosts. Existing source ecosystems provide valuable compatibility cases but often depend on application-specific APIs and execution environments. The repository already contains a draft specification, manifest schema, RFC template, and static declarative example. There is no runtime implementation yet.

This record captures current decisions so later work does not have to recover them from conversations. It does not redesign or finalize the v0.1 specification.

## Decision

1. Keep specification, runtime work, examples, conformance work, and project documentation in a **single monorepo**.
2. Maintain a **platform-neutral specification** that defines behavior and interfaces independently of a reference runtime and its implementation language.
3. Define **exactly five initial source operations**: `home`, `category`, `search`, `detail`, and `play`. This is the operation vocabulary; a source may declare a subset under the existing manifest rules.
4. Target **declarative JSON and JavaScript** as the initial source engines. The static declarative format is specified; JavaScript binding details remain open. Neither engine is implemented yet.
5. Treat **Rust as a candidate for a shared reference runtime core**, not a selected implementation, required dependency, source engine, or part of the standard. Record the actual implementation choice when there is evidence to make it.
6. Keep **WASM exploratory and deferred**. It is not a v0.1 engine, and no WASM architecture or ABI is adopted here.
7. Put **legacy compatibility in adapters**. TVBox, FongMi, drpy, XBPQ, and XYQ are inputs and inspiration, not the core identity. Native or application-specific requirements do not become portable standard requirements.

## Alternatives considered

- Adopting one legacy ecosystem as the core would inherit its platform and application assumptions.
- Requiring one implementation language or native ABI would confuse a portable behavioral contract with one runtime's design.
- Splitting repositories or introducing WASM now would add coordination and design work before the initial contract has execution evidence.

These alternatives remain unnecessary for the current foundation. This record does not claim that an implementation benchmark or multi-platform runtime comparison has already been performed.

## Consequences

The specification can be read, reviewed, and implemented independently. The first runtime can stay small and test the existing static profile. Conformance evidence, rather than the reference runtime's incidental behavior, must guide standardization.

Adapter coverage will be explicit and may be partial. The project will need further decisions for implementation language, JavaScript execution, host-service profiles, and platform integrations. Planned engine names and empty directory scaffolding do not justify claims of implemented support. Contract changes still require the RFC and versioning process.
