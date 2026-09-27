# Project charter

## Purpose

**Write once. Run everywhere.** Universal Source is a cross-platform specification and runtime for portable content sources.

Content sources often depend on private host APIs, application-specific data formats, or undocumented behavior. Reusing a source then requires copying application assumptions along with its content logic. Universal Source exists to establish an explicit, testable boundary between a source and its host so that the same source can work across independent implementations.

The cross-platform objective covers Android, Android TV, iOS, iPadOS, tvOS, macOS, Windows, and Linux. These are target platforms, not a statement of current support. Progress must be demonstrated with shared sources and comparable results, not inferred from an implementation language's portability.

## What the project is

- A platform-neutral specification of observable behavior and interfaces, useful independently of a reference runtime.
- A home for reference runtime work, examples, conformance fixtures, and compatibility adapters in one monorepo.
- A small content-source contract whose initial operations are defined in the [Source API](../spec/source-api.md).
- A controlled host boundary for untrusted sources, governed by the [Host API](../spec/host-api.md).

The standard defines behavior and interfaces, not implementation language. Declarative JSON and JavaScript are the initial source-engine targets. Their inclusion does not mandate the language used to implement a runtime. Rust remains a candidate for a shared reference core; WebAssembly remains exploratory.

## What the project is not

Universal Source is not a player application, a content catalog or provider, a platform SDK disguised as a standard, or a promise that every existing source can be converted automatically. It is not tied to Android, Java/JAR, Swift, Kotlin, Rust, or any single host ecosystem.

The current v0.1 scope excludes recommendation systems, accounts, sync, DRM, subtitles, comments, danmaku, downloads, player UI, platform-specific apps, and WASM engines. Do not add these as incidental parts of foundation or runtime work. Later platform proof should use narrow test harnesses rather than expanding the product into a player.

## Compatibility inputs

TVBox, FongMi, drpy, XBPQ, and XYQ are legacy and inspiration ecosystems. They provide concrete input formats, source samples, and migration problems; they are not the identity of Universal Source.

Import existing sources where reasonably possible through explicit adapters. Report unsupported behavior and narrower platform coverage honestly. Legacy application APIs and native execution requirements must not become implicit requirements of the core standard. See the [compatibility contract](../spec/compatibility.md).

## Success and authority

Success means that independent hosts can agree on a source's observable behavior, enforce its authority boundaries, and explain incompatibilities. It does not require every host to implement every engine or every media codec.

This charter defines project purpose and scope. The [design principles](DESIGN_PRINCIPLES.md) guide choices, the [specification](../spec/README.md) defines normative behavior, the [architecture](ARCHITECTURE.md) records current reality, and the [roadmap](ROADMAP.md) orders work. Changes to project direction require an explicit decision record; changes to source contracts follow the RFC process. Neither a plan nor a context summary overrides the specification.
