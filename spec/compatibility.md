# Compatibility and versioning

Status: v0.1 draft. Portability means agreement on observable behavior, not a requirement to share runtime code or implementation language.

## Version contract

`specVersion` identifies the source contract, not the source's own release number or a runtime version. This draft uses the exact string `"0.1"`. A runtime MUST explicitly support the requested version and engine or refuse loading. It MUST NOT compare version strings lexically, infer support from a prefix, or assume a later runtime version can execute every earlier source.

The draft can change before it is finalized. During this period, implementation reports SHOULD identify the repository commit they target as well as `specVersion`. There is no stable v0.1 release or certification yet. Once a specification version is finalized, incompatible changes MUST receive a new specification version. Before 1.0, that means a new minor version such as `0.2`; after 1.0, breaking contract changes require a new major version. Editorial corrections MUST NOT change required behavior.

Adding manifest fields, operations, or capability names requires an explicit versioning decision because this draft rejects unknown values. Future proposals MUST explain old-source/new-runtime and new-source/old-runtime behavior using the [RFC template](./rfcs/0000-template.md). No vendor extension namespace or automatic fallback negotiation is defined in v0.1.

## Engines and implementations

`declarative` and `javascript` are source formats, not implementation-language mandates. A host MAY implement only one engine, but MUST state that scope and reject unsupported engines. A shared Rust core is a possible future implementation, not a normative dependency. WebAssembly is deferred and MUST NOT be accepted as a v0.1 manifest engine.

The static declarative profile has specified dispatch behavior. JavaScript execution and detailed host-service bindings remain incomplete. Claims of support MUST distinguish manifest validation, static declarative behavior, source API behavior, and any experimental engine/service binding. Recognizing a manifest is not proof that its source can run.

## Legacy adapters

TVBox, FongMi, drpy, XBPQ, and XYQ are legacy and inspiration ecosystems. Their application conventions, native APIs, script environments, and data encodings MUST NOT become implicit core requirements.

Compatibility work belongs in separate adapter modules within this monorepo, for example future modules under `runtime/compatibility/`. These modules do not exist yet. An adapter MAY:

- Translate a compatible rule set into a declarative or JavaScript source.
- Supply a controlled legacy execution environment behind the five source operations.
- Report that a source requires unsupported behavior and cannot be imported.

An adapter MUST map legacy IDs, lists, details, resource resolution, and errors into the core contract. It MUST declare required host services and permissions. It MUST NOT use legacy behavior to bypass permission checks or expose Java/JAR loading, Android APIs, or host-native handles as portable source results.

Some legacy sources will depend on native code, application-specific state, executable JARs, or undocumented behavior. Supporting them may require a restricted adapter with narrower platform coverage; that does not make the original source portable. Java/JAR is not a new standard engine. Full automatic conversion is not promised.

An importer SHOULD provide a report of supported behavior, transformations, unresolved permissions, and unsupported features. It MUST NOT silently drop unsupported behavior while claiming full compatibility. Imported code MUST be treated as untrusted. Uncertain network requirements MUST NOT become wildcard access grants.

## Conformance direction

Future fixtures belong under `conformance/`. They SHOULD compare observable results across independent implementations and cover:

- Valid and invalid manifests, unsupported versions/engines, and entry containment.
- The five operations, defaults, empty lists, unknown IDs, and malformed results.
- Static declarative exact lookup, cross-references, and pages past the end.
- Denied services and permissions, redirects, cookie/storage isolation, and resource URLs.
- Cancellation, deadlines, serialization, and instance disposal.
- Adapter mappings and explicitly rejected legacy behaviors.

Live-site availability, media codec support, and UI appearance are not core conformance criteria. Deterministic fixtures SHOULD avoid third-party services. Legacy samples MUST be shareable with appropriate permission and MUST NOT contain credentials.

The current example is illustrative, not a conformance suite. No runtime, adapter, or platform support is certified by this foundation.
