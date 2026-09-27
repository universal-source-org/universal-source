# Universal Source

**A portable specification for interoperable content sources across runtimes and platforms.**

[简体中文](./README.zh-CN.md)

Universal Source is an open project exploring a common contract for describing and executing content sources without binding them to a single application, runtime, or device ecosystem.

Today, many source definitions are tightly coupled to a particular host: they depend on private JavaScript APIs, application-specific JSON shapes, implicit runtime behavior, or undocumented conventions. That makes reuse difficult and turns every client into its own compatibility island.

Universal Source aims to make the **source** portable.

## What we want to achieve

A Universal Source should be able to express **what a source needs to do** while leaving **how it is executed** to conforming runtimes.

The project is being designed around a few principles:

- **Portable** — one source definition should not belong to one app.
- **Runtime-independent** — JavaScript, Rust, Go, Swift, Kotlin, and other implementations should be able to conform to the same contract.
- **Backward-conscious** — existing source ecosystems matter; migration and compatibility should be treated as first-class problems.
- **Capability-based** — sources declare what they need instead of assuming a specific host environment.
- **Testable** — compatibility should be demonstrated against real-world source samples, not only described on paper.
- **Minimal core, extensible edge** — standardize the smallest stable contract and allow capabilities to evolve without constantly breaking the core.

## The problem

A typical content source may need to:

1. expose metadata,
2. search,
3. list or discover content,
4. resolve detail pages,
5. resolve playable resources,
6. make network requests,
7. parse structured or unstructured responses,
8. persist small amounts of state,
9. optionally execute controlled scripting.

Different applications often model these operations differently. Even when two ecosystems both use JavaScript or JSON, their APIs are usually incompatible.

Universal Source is an attempt to define a shared boundary between:

```text
Source Definition
      ↓
Universal Source Contract
      ↓
Conforming Runtime
      ↓
App / TV / Mobile / Desktop / Server
```

The goal is not to force every implementation to use the same programming language. The goal is to make them agree on the same observable contract.

## Project status

Universal Source is currently in the **early design stage**.

The specification, execution model, capability system, compatibility strategy, and real-world corpus are still being defined. APIs and terminology may change significantly before the first stable version.

This repository is the public home of that work.

## Scope

The initial work focuses on:

- source metadata and manifests,
- request / response primitives,
- search and discovery,
- detail and resource resolution,
- runtime capabilities and permissions,
- deterministic error semantics,
- compatibility with existing JavaScript- and JSON-based source formats,
- a real-world compatibility corpus.

Non-goals will be documented explicitly as the design becomes more concrete.

## A standard must earn compatibility

A specification is only useful if implementations can agree on its behavior.

Universal Source therefore intends to treat real-world compatibility as part of the standardization process:

```text
existing sources
      ↓
compatibility corpus
      ↓
conformance tests
      ↓
multiple runtimes
      ↓
same observable behavior
```

The long-term objective is simple:

> Define once. Implement anywhere. Run sources across ecosystems.

## Repository direction

This repository will initially host the core project work while the design is still young. As stable boundaries emerge, specifications, conformance suites, runtimes, SDKs, and tooling may be split into dedicated repositories.

Nothing is being split merely for appearance.

## Contributing

The project is at its most valuable stage for criticism.

If you have experience building source systems, parsers, plugin runtimes, media clients, scraping infrastructure, sandboxed execution environments, or cross-platform SDKs, discussions and concrete compatibility cases are especially welcome.

Before proposing a new abstraction, we want to answer three questions:

1. What real-world problem does it solve?
2. Can different runtimes implement it consistently?
3. Can existing sources migrate to it without unreasonable friction?

## Name

**Universal Source** describes the intended boundary: a source format and execution contract that can outlive any single host application.

---

Universal Source is experimental. The project does not yet claim stable compatibility, production readiness, or a finalized standard.
