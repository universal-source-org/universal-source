# Current architecture

This is a snapshot of the repository's current state, not a proposed implementation design. The [specification](../spec/README.md) remains authoritative for contract details; the [roadmap](ROADMAP.md) owns sequencing and future exit criteria.

## Present artifacts and implementation status

| Area | Current status | Evidence and limits |
| --- | --- | --- |
| Repository context | Present documentation | Charter, principles, roadmap, decisions, and contribution workflow in this monorepo. |
| Source contract | Specified as an experimental draft | [Source API](../spec/source-api.md) defines exactly five operations, results, errors, and static declarative dispatch. No executing engine exists. |
| Manifest | Schema and prose present | [Schema](../spec/schema/manifest.schema.json) covers structure; semantic validation still requires the prose contract. No checked-in validator or automated validation command exists. |
| Host boundary | Partially specified | [Host API](../spec/host-api.md) defines service responsibilities and permission boundaries. Exact service signatures and algorithm profiles remain open. |
| Lifecycle and versioning | Specified as draft contracts | [Lifecycle](../spec/lifecycle.md) and [compatibility](../spec/compatibility.md) define expected behavior; no runtime enforces it yet. |
| Declarative example | Static data present | [Minimal source](../examples/json/minimal/README.md) demonstrates all five operations with a placeholder media URL. It is not an executable conformance suite. |
| Reference runtime | Planned, not implemented | No runtime source code, executable, build configuration, or selected implementation language. |
| JavaScript execution | Target named; binding and implementation pending | The manifest accepts `javascript`, but no interoperable module/async binding or engine implementation exists. |
| Legacy adapters | Planned, not implemented | The compatibility document defines their boundary; no importer or adapter exists. |
| Platform integrations | Planned proof, not implemented | No Android, Apple, desktop, or TV runtime integration is implemented. |
| WASM | Exploratory only | No engine value, ABI, module loader, or WASM runtime architecture is defined or implemented. |

There are currently **no implemented executable components**. Documentation, schema, and example data are the existing foundation. Recognizing an engine name in a manifest does not mean the engine runs.

## Contract boundaries that exist in the draft

A source consists of a manifest and entry file. The manifest identifies its exact specification version, engine, operations, required host services, and requested permissions. The runtime is responsible for validation, isolation, invocation, and result validation on behalf of a host. The host grants authority and consumes results. These are specified responsibilities, not instantiated software components.

The initial operation vocabulary is `home`, `category`, `search`, `detail`, and `play`; individual sources may declare a subset under the existing manifest rules. The declarative profile describes static JSON lookups. It does not include a network scraping language. JavaScript is the other initial engine target, with its execution binding still open.

The host-service vocabulary covers HTTP, cookies, storage, HTML parsing, JSON, crypto, URL utilities, and logging. Required services and requested permissions are distinct. Returned media and poster URLs remain subject to the specified network boundary. This summary does not change the normative rules.

## Repository layout

| Path | Current role |
| --- | --- |
| `spec/` | Draft contracts, manifest schema, and RFC template. |
| `examples/json/minimal/` | The one existing declarative example. |
| `runtime/` | Reserved area for future implementation. |
| `conformance/` | Reserved area for fixtures and cross-implementation checks; no checked-in suite yet. |
| `docs/` | Durable project context, decisions, and task plans. |

Local scaffolding may contain empty runtime, language-binding, or conformance directories. Git does not preserve empty directories by itself; their names are neither implementations nor decisions to adopt a language or ABI. The plan directories have explicit `.gitkeep` files so their workflow locations survive cloning.

## Decision boundary

[ADR 0001](decisions/0001-initial-architecture.md) records the monorepo, platform-neutral contract, initial engines, and adapter boundary. Rust is a candidate for a future shared reference core, not a selected dependency or standard requirement. WASM has no current architecture beyond being deferred research. Specific runtime structure, dependency choices, and platform bindings need evidence and a scoped decision when their roadmap phase is active.
