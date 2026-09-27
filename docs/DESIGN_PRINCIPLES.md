# Design principles

These principles guide decisions within the [project charter](PROJECT_CHARTER.md). They do not add fields or operations to the [specification](../spec/README.md).

## Portability over platform convenience

Define observable behavior that independent mobile, TV, and desktop hosts can reproduce. Keep implementation details and platform integration behind the host boundary. A convenient Android API, Apple framework, native ABI, or implementation language is not sufficient reason to standardize an interface.

## Compatibility through adapters

Translate legacy formats and behavior at explicit boundaries. Preserve useful source logic where possible and report unsupported behavior where not. Do not expand the core with application-specific fields or privileged escape hatches to make one legacy source appear compatible.

## Minimal sufficient abstractions

Choose the smallest contract that handles demonstrated cases clearly. Start with the existing five operations and static declarative profile. Add an abstraction only when concrete examples explain its semantics, cost, and testability across independent implementations.

## Sources are untrusted

Treat source packages, imported scripts, and returned data as untrusted. Validate before execution, enforce host-granted permissions, isolate state, and bound resource use. A manifest declaration or familiar legacy format is not a grant of authority. Adapters obey the same host boundary. Detailed requirements belong in the Host API and lifecycle documents, not duplicated here.

## Backward compatibility is deliberate

Preserve observable contracts and stable identities where possible. Evaluate old-source/new-runtime and new-source/old-runtime behavior when making changes. Follow the [versioning rules](../spec/compatibility.md); never silently reinterpret an unsupported version or declaration. The current draft may evolve, but draft status is not an excuse to hide breaking changes or claim stable support prematurely.

## Proof before standardization

Use small representative sources, deterministic positive and negative fixtures, and independent implementation evidence to test proposed behavior. Prototypes may inform the draft; one reference implementation must not silently become the definition of the standard. Record evidence and remaining gaps before treating a contract as stable. Live-site success alone is not proof of conformance.

## Avoid architecture for architecture's sake

Keep the monorepo and the active phase focused on a coherent, reviewable result. Do not add frameworks, engine formats, repositories, native apps, or speculative layers merely because they might be useful later. Rust is the reference implementation choice under [ADR 0002](decisions/0002-reference-runtime-language.md), not a source-format requirement; WASM research is not authorization to design or implement a plugin subsystem now.

## Keep evidence and status visible

Distinguish specified behavior, implemented behavior, planned work, and exploratory ideas. Update the [current architecture](ARCHITECTURE.md), [roadmap](ROADMAP.md), and relevant [decision records](decisions/README.md) when their facts change. Leave enough context for the next contributor to continue without a chat transcript.
