# Declarative reference loader

This Rust library loads and validates static declarative sources against the v0.1 draft reviewed at `e0613c5fe1e5d36f690e34c16893e45847217674`. It implements **loading only**. Phase 2 remains active: no operation dispatch, operation execution harness, grants, invocation lifecycle, or network consumers exist yet. [ADR 0002](../docs/decisions/0002-reference-runtime-language.md) selects Rust for this implementation, not for portable source packages.

## Build and validate

Run from the repository root with Rust/Cargo 1.96 or newer (tested with 1.96.0):

```sh
cargo build --manifest-path runtime/Cargo.toml --locked
cargo test --manifest-path runtime/Cargo.toml --locked
cargo fmt --manifest-path runtime/Cargo.toml --check
cargo clippy --manifest-path runtime/Cargo.toml --locked --all-targets -- -D warnings
```

Also run the independent [Python conformance checks](../conformance/README.md#setup):

```sh
.venv/bin/python -m unittest discover -s conformance -p 'test_*.py' -v
```

The crate is unpublished; its version is an implementation version, not a stable specification release. The lockfile pins the tested dependency resolution. Initial dependency acquisition needs registry access; the loader and tests do not fetch source URLs or remote schemas. No CLI, package installer, or native binding is included.

## Library boundary

`load(source_root, LoadLimits::default())` returns `Result<LoadedSource, LoadError>`. The source root is a host-provided directory. `LoadedSource` owns an immutable manifest, entry, and normalized requested-origin list, exposed by read-only accessors. Separate loads share no mutable source data. Later file changes do not alter a loaded snapshot. Accessors are host inspection of stored data, not source-operation results or permission approval; hosts must not consume stored URLs as if access were authorized.

The loader:

1. Validates the configured budgets and resolves the source root.
2. Reads a contained regular `manifest.json` within its byte budget; rejects excessive JSON depth, malformed UTF-8/JSON, duplicate members at every depth, invalid Unicode, and non-finite numbers.
3. Distinguishes unsupported version and engine strings, then validates against the [authoritative Draft 2020-12 schema](../spec/schema/manifest.schema.json), embedded at compile time without a copied schema. It supports exactly version `0.1` and the `declarative` engine.
4. Separately validates origin semantics and detects duplicates after normalization.
5. Resolves the declared entry, verifies containment and regular-file status, applies entry/combined-input budgets, and parses it with the same strict JSON rules.
6. Validates the exact declared operation keys, every stored shape, required/optional/unknown fields, map and array IDs, valid query keys, URL/header/media-type syntax, conditional cross-references, and consistent reuse of playable IDs across details.

Successful loading establishes whole-entry validity. It is not yet an invocable ready instance: grants and invocation machinery are intentionally absent. Per [A1](../docs/reviews/2026-09-28-phase-1-closure-review.md#a1--fixture-validation-is-narrower-than-semantic-conformance), an otherwise valid stored URL may be outside requested origins. Such data is not rejected merely because a future result would be denied. Validation never fetches URLs, resolves DNS, follows redirects, or starts media access.

## Filesystem and load-budget policy

The host must supply a directory whose contents and path ancestors are controlled and kept stable for the duration of `load`. Untrusted package bytes and static symlinks are validated; concurrent hostile filesystem writers are outside this loader's boundary. Canonicalizing then opening a path has a race window and is **not** an adversarial filesystem sandbox. Hosts needing that threat model must first provide a stable isolated snapshot; this unit does not implement installation or snapshot creation.

Both manifest and entry are canonicalized and checked with path-component containment, not string prefixes. In-root symlinks, including directory symlinks, are accepted; dangling links and escapes (including similarly prefixed sibling directories) fail. Root symlinks resolve to the host-selected root. Only regular files are opened; directories/devices/pipes are refused. The file is checked again after open and reads are bounded even if metadata understates its length. Filesystem identity/case rules follow the host OS. Unix symlink behavior is tested locally; Windows symlink/reparse behavior and all platform embeddings remain unproven.

| Budget | Default and hard ceiling | Scope |
| --- | --- | --- |
| Manifest bytes | 65,536 (64 KiB) | Raw manifest input, before decoding. |
| Entry bytes | 8,388,608 (8 MiB) | Raw static entry input, before decoding. |
| Package input bytes | 8,454,144 | Sum of manifest and entry bytes read, even if paths alias. |
| JSON nesting | 64 | Maximum nested object/array containers in each input. |

Hosts may lower each budget to any nonzero value up to its ceiling. Zero or values above the ceilings return `ResourceLimit`. Exact byte limits are inclusive. A depth scan ignores braces inside strings before entering the JSON parser. The package-input budget covers the entire source data consumed by this static profile (manifest plus entry); unrelated files in the directory are neither traversed nor loaded and are not counted. It is not a directory-tree or archive-size guarantee.

These are reference-runtime limits, not portable v0.1 numbers. They bound accepted input and parser recursion; they do not constitute a precise allocator quota, wall-clock deadline, or isolation from blocking host filesystem I/O. The host must provide suitable local storage. Invocation timeouts/cancellation/memory accounting remain subsequent Phase 2 work.

## Diagnostics and precedence policy

`LoadError` variants are implementation-specific host diagnostics, never Source API envelopes or new portable error codes. They carry no raw source strings, header values, paths, schema-error instances, or underlying parser/OS errors; `Display` uses fixed category descriptions. `LoadedSource` debugging also omits stored data. Hosts inspecting values directly remain responsible for confidentiality.

| Variant | Meaning in this loader |
| --- | --- |
| `InvalidManifest` | Bad root/manifest file, syntax/duplicates, schema failure, or semantic manifest failure. |
| `UnsupportedVersion` | A string `specVersion` other than `0.1`. |
| `UnsupportedEngine` | A string engine other than `declarative`, including `javascript`. |
| `UnavailableService` | Distinct reserved category; unreachable for a schema-valid static declarative source, whose required-service list must be empty. |
| `InvalidEntry` | Missing, escaping, non-regular, unreadable, malformed, or semantically invalid entry. |
| `ResourceLimit` | Invalid budget configuration, excessive input bytes/combined input, or excessive JSON depth. |

Under [D1](../docs/reviews/2026-09-28-phase-1-closure-review.md#d1--host-policy-and-simultaneous-lifecycleload-failures), this implementation uses the numbered loading order above. Limits encountered while reading/scanning take precedence over later parsing. After manifest decoding, unsupported version strings precede unsupported engine strings, then schema and semantic errors. Missing or non-string version/engine values are schema errors unless another explicit unsupported string is found first. Invalid schema/semantic data precedes entry access. Invalid host-service declarations in a declarative manifest yield `InvalidManifest`; they cannot be made valid by supplying services. No synthetic unavailable-service path or Phase 3 binding is introduced.

Multiple problems in a phase produce that phase's category, with no promised field-level order. These choices are locally tested policies, not portable multi-fault precedence. The compiled-in repository schema is trusted build input; failure to parse/compile it is a developer defect (panic), not a source load diagnostic. Tests validate that artifact independently.

## URL and syntax policies

The [Host API](../spec/host-api.md#capabilities-and-permission-grants) supplies the mandatory exact-origin rules. Parsing uses the [`url` crate](https://docs.rs/url/2.5.8/url/), followed by explicit checks; schema patterns alone are insufficient. Origin requests forbid credentials, paths (including `/`), query/fragment, wildcards, uppercase scheme/DNS names, Unicode DNS spellings, explicit default ports, zero/out-of-range/leading-zero ports, and malformed hosts. Bracketed IPv6 is parsed, compressed for comparison, and semantically duplicate origins are rejected. HTTP, HTTPS, and non-default ports remain separate authorities. The returned normalized list is requested authority only.

The following are implementation policies under [D2](../docs/reviews/2026-09-28-phase-1-closure-review.md#d2--complete-cross-implementation-url-parsing-profile), not a universal URL-service profile:

- Require an explicit `://` and nonempty authority for stored HTTP(S) URLs. Reject literal ASCII whitespace/control characters, backslashes, and any user-info delimiter, including empty credentials; percent-encoded path data is handled by the parser.
- Validate parsed DNS labels as nonempty ASCII letters/digits/hyphens, at most 63 bytes each, with no edge hyphens and at most 253 bytes excluding a single terminal dot. The parser performs IDNA handling; Unicode resource hostnames may normalize to ASCII, while manifest origins must already use ASCII. Single-label hosts are not assumed to have a public suffix.
- The authoritative manifest schema excludes trailing-dot origins; stored resource URLs may contain a single trailing DNS dot. Its future result-origin comparison needs to remain explicit in the parser policy. WHATWG numeric IPv4 forms normalize through the parser (for example `127.1` and `127.0.0.1` collide). IPv6 alternate spellings also collide. This is comparison of requested origins, not a grant expansion mechanism.
- Stored resource URLs may include case differences, explicit default ports, paths, queries, and fragments when parsed as valid HTTP(S) URLs without credentials. Stored text is preserved, not rewritten. Syntax acceptance does not grant access. The future result validator must use consistent normalization, not string-prefix checks.
- Media types use the `mime` parser and must be concrete types rather than wildcard ranges. Headers use HTTP token names, case-insensitive uniqueness, string values without CR/LF, and the six forbidden names in the Source API. No transport or cookie behavior is implemented.

The schema validator is [`jsonschema`](https://docs.rs/jsonschema/0.58.1/jsonschema/) in Draft 2020-12 mode, with network/file retrieval disabled. Parser/library behavior outside the explicit rules remains a portability surface. Record concrete disagreements for review rather than treating this library's behavior as the specification.

## Tests and remaining scope

Tests load both repository source contexts, reuse manifest fixtures, and exercise JSON/Unicode/duplicate rejection, schema-versus-semantic checks, diagnostic categories/precedence, containment and Unix symlinks, shapes and references, URLs/origins/headers, A1, independent snapshots, and limit boundaries. They do not execute any of the 34 operation cases. The independent Python suite continues to validate those authored cases as data.

Still absent: dispatch, result envelopes/grant enforcement, operation-case execution, cancellation/disposal/serialization, consumers/redirects, host-service bindings, JavaScript, legacy adapters, platform applications, and WASM. No interoperability or platform-support certification is implied. The next coherent task is dispatch plus an execution harness for the existing declarative cases; it has not started here.
