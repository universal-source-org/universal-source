# Universal Source

**Write once. Run everywhere.**

**A cross-platform specification and runtime for portable content sources.**

[简体中文](./README.zh-CN.md) · [Specification v0.1 draft](./spec/README.md) · [Minimal JSON source](./examples/json/minimal/README.md)

Universal Source aims to let one content source run across **Android, Android TV, iOS, iPadOS, tvOS, macOS, Windows, and Linux**. This is the project goal, not a claim of current platform support. The repository contains the reviewed v0.1 draft, examples, conformance data, and a [Rust declarative runtime](./runtime/README.md) with loading, independent operation dispatch, returned-origin checks, and execution of 34 conformance cases. Broader lifecycle enforcement remains unfinished.

## The contract

**The standard defines behavior and interfaces, not implementation language.** A source exposes a small set of operations:

| Operation | Purpose |
| --- | --- |
| `home` | List entry categories and content. |
| `category` | List a page of content in a category. |
| `search` | Find a page of content matching a query. |
| `detail` | Describe content and its playable items. |
| `play` | Resolve a playable item to a media resource. |

The host supplies controlled services for HTTP, cookies, storage, HTML parsing, JSON, crypto, URL utilities, and logging. A source declares its operations, required host services, and requested permissions. Declarations do not grant access by themselves.

```text
Declarative JSON or JavaScript source
                  ↓
     Universal Source Specification
                  ↓
          Conforming runtime
                  ↓
       Mobile / TV / Desktop host
```

The specification is useful independently from the reference runtime. Implementations may use any language or platform that can preserve the contract.

## v0.1 scope and status

This is an **experimental v0.1 draft**, not a release or a production compatibility promise. It defines a manifest, five source operations, shared data and error rules, host capability boundaries, a lifecycle, and a minimal static declarative format. JavaScript is an intended source engine; its execution binding and detailed host service profiles still need specification work before interoperable runtime implementations can be claimed.

The architectural direction is:

1. A platform-neutral specification.
2. Declarative JSON for simple sources, starting with static data.
3. JavaScript for sources needing custom logic.
4. Compatibility adapters for existing ecosystems.

The reference runtime uses Rust under [ADR 0002](./docs/decisions/0002-reference-runtime-language.md), without making Rust part of the standard. WebAssembly may be explored later; it is not a v0.1 engine and is not implemented here.

v0.1 excludes recommendation systems, accounts, sync, DRM, subtitles, comments, danmaku, downloads, player UI, and platform-specific apps. The first version deliberately avoids a general scraping language, native plugins, and application-specific APIs.

## Existing ecosystems

TVBox, FongMi, drpy, XBPQ, and XYQ are legacy and inspiration ecosystems. They provide useful compatibility cases, but do not define this project's identity or core API. Importers and adapters should reuse existing sources where reasonably possible and report unsupported behavior explicitly. Android APIs, Java/JAR loading, and legacy data conventions belong in compatibility layers, not in the new standard. No adapters are implemented or compatibility guarantees made yet.

## Repository

This project stays in one monorepo:

| Directory | Purpose |
| --- | --- |
| [`spec/`](./spec/README.md) | Independent specification, manifest schema, and RFCs. |
| [`runtime/`](./runtime/README.md) | Rust static loader, dispatcher, permission checks, and execution tests. |
| [`examples/`](./examples/json/minimal/README.md) | Small sources illustrating the contract. |
| [`conformance/`](./conformance/README.md) | Manifest validation and authored declarative fixtures executed by the Rust harness. |
| [`docs/`](./docs/PROJECT_CHARTER.md) | Project charter, principles, current architecture, roadmap, decisions, and task plans. |

Start with the [specification index](./spec/README.md), then read the [example manifest and source](./examples/json/minimal/README.md). The example uses a placeholder media URL and needs no network access to inspect. The [execution harness](./conformance/declarative/README.md#structural-validation-and-runtime-execution) runs its authored calls without network access or playback.

## Contributing

Start with [CONTRIBUTING.md](./CONTRIBUTING.md) and [AGENTS.md](./AGENTS.md) for the repository workflow and context map. The [roadmap](./docs/ROADMAP.md) identifies the active phase and its exit criteria.

Propose contract changes using the [RFC template](./spec/rfcs/0000-template.md). Explain the concrete use case, portable behavior, migration impact, and how independent implementations could test agreement. Small examples and documented legacy incompatibilities are more useful than speculative abstractions.
