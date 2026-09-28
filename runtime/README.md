# Declarative reference runtime

This Rust library loads and validates static declarative sources against the v0.1 draft reviewed at `e0613c5fe1e5d36f690e34c16893e45847217674`. It implements loading, five-operation dispatch, returned-origin checks, and **lifecycle-managed declarative instances**. The Rust harness executes all 34 authored cases through that managed path. The [Phase 2 closure review](../docs/reviews/2026-09-28-phase-2-closure-review.md) accepts this bounded static profile; network consumers are not implemented. [ADR 0002](../docs/decisions/0002-reference-runtime-language.md) selects Rust for this implementation, not for portable source packages.

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

Successful loading establishes whole-entry validity. Move the snapshot into `Instance::new(source, CallLimits::default())` to create a ready lifecycle-managed instance. The snapshot itself has no mutable lifecycle state or public dispatch method. Per [A1](../docs/reviews/2026-09-28-phase-1-closure-review.md#a1--fixture-validation-is-narrower-than-semantic-conformance), an otherwise valid stored URL may be outside requested origins. Such data is not rejected merely because a future result would be denied. Validation never fetches URLs, resolves DNS, follows redirects, or starts media access.

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

These are reference-runtime limits, not portable v0.1 numbers. They bound accepted input and parser recursion; they do not constitute a precise allocator quota, wall-clock deadline, or isolation from blocking host filesystem I/O. The host must provide suitable local storage. Invocation deadlines, cancellation, and response budgets are described below; precise allocator accounting is not implemented.

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
- The authoritative manifest schema excludes trailing-dot origins; stored resource URLs may contain a single trailing DNS dot. Result-origin comparison preserves that dot, so it does not match a dotless requested origin. This conservative D2 policy does not grant additional authority. WHATWG numeric IPv4 forms normalize through the parser (for example `127.1` and `127.0.0.1` collide). IPv6 alternate spellings also collide. This is comparison of requested origins, not a grant expansion mechanism.
- Stored resource URLs may include case differences, explicit default ports, paths, queries, and fragments when parsed as valid HTTP(S) URLs without credentials. Stored text is preserved, not rewritten. Syntax acceptance does not grant access. The result validator uses the same origin formatter as the loader, not string-prefix checks.
- Media types use the `mime` parser and must be concrete types rather than wildcard ranges. Headers use HTTP token names, case-insensitive uniqueness, string values without CR/LF, and the six forbidden names in the Source API. No transport or cookie behavior is implemented.

The schema validator is [`jsonschema`](https://docs.rs/jsonschema/0.58.1/jsonschema/) in Draft 2020-12 mode, with network/file retrieval disabled. Parser/library behavior outside the explicit rules remains a portability surface. Record concrete disagreements for review rather than treating this library's behavior as the specification.

## Ready-source dispatch and effective grants

`Instance::invoke(operation, &input, &grants, &cancellation) -> Result<serde_json::Value, LifecycleError>` is the synchronous implementation-specific boundary, not a portable Rust binding. The instance owns a successfully loaded immutable snapshot; results are owned copies. The earlier public `LoadedSource::invoke` boundary is now internal so ordinary calls cannot bypass lifecycle enforcement. Use `instance.source()` for read-only snapshot inspection and grant setup. U1 portable non-ready/disposed reporting remains deferred; the local misuse outcome is documented below.

Construct `EffectiveGrants::new(&source, &network_origins)` separately from operation input. It normalizes and validates exact origins with the loader's policy, rejects duplicates and authority beyond the manifest, and returns the host setup error `InvalidGrants` on failure. `EffectiveGrants::default()` denies all network access. Static sources have no cookie/storage authority. Grants are supplied afresh per call; no grant is cached in the snapshot. Returned URLs must match both the invoked source's requests and the effective grants, even if the grants object originated with another source.

Call ordering is declaration/vocabulary, input, lookup, result shape/identity/reference validation, returned-origin checks, then envelope delivery. Unknown vocabulary strings and undeclared operations yield `UNSUPPORTED_OPERATION` before input checks. Inputs are closed objects; IDs remain opaque, and search rejects only the specified empty/whitespace-only queries. Pages default only when omitted and accept integer JSON numbers from 1 through 9007199254740991, including `1.0`; null, booleans, strings, fractions and out-of-range values fail without coercion.

| Operation | Static behavior |
| --- | --- |
| `home` | Copies stored categories/items, including empty arrays. |
| `category` | Looks up category before page handling; unknown is `NOT_FOUND`. Page 1 returns stored items, later pages empty. |
| `search` | Exact case-sensitive lookup without trimming/normalization. Absent query or later page returns empty items. |
| `detail` | Copies the stored detail, independently of preceding calls; unknown is `NOT_FOUND`. |
| `play` | Looks up the stored resource before grants; unknown remains `NOT_FOUND` even with no grants. |

Paged results preserve the effective page and use `hasMore: false`. Shape and reference validation reuses loader type checks. Invalid constructed data yields `INVALID_RESULT`. This local structure-before-permission order does not standardize U2 or add combined-violation conformance expectations. Envelopes are constructed with exactly `ok`/`data` or `ok`/`error`; errors contain only an existing code and fixed, nonempty message without source values. No new portable errors are introduced.

Only typed returned URL fields are checked: list-item posters, detail posters, and play resource URLs. IDs, titles, descriptions and header values are not recursively interpreted as URLs. A denied URL rejects the entire result with `PERMISSION_DENIED`; nothing is stripped or fetched. An unreturned stored URL cannot deny an unrelated result, and a later empty page contains no poster to check. Origin parsing handles case/default ports/IPv6 consistently; scheme, subdomain and non-default port boundaries remain exact. Every future consumer must still recheck actual access and redirects; this implementation does not provide a consumer.

## Lifecycle and invocation policy

The instance owns its snapshot by value. It cannot be cloned or reset; share the same instance with `Arc` when several host threads need to call it. Independent loads/new instances have separate snapshots, mutexes, condition variables, and counters. Returned values never share mutable source memory. `source()` is inspection only, including after disposal; it cannot execute operations or authorize resource consumption.

`Instance::new` validates `CallLimits` and starts ready. Invalid configuration is `Err(LifecycleError::InvalidLimits)`, a host setup diagnostic. A call arriving after disposal is `Err(LifecycleError::Disposed)`, with no operation execution and no Source API envelope. This implements U1 locally without assigning a portable misuse code. Other admitted calls return one ordinary Source API envelope.

A per-instance mutex/condition-variable gate admits at most one executing call. Waiting callers retain their own borrowed input/grants on their host threads; the runtime creates no thread, async task, input copy, or unbounded queue. Admission is not FIFO and has no fairness guarantee. Waiting consumes the call's deadline. Independent instances have no shared execution lock. Standard-library synchronization suffices; no async runtime dependency is added.

`Cancellation::default()` creates a fresh sticky token; `cancel()` requests cancellation. Deliberately sharing one token cancels all calls using it, even across instances; isolation tests use independent tokens. There is no reset or retry. A queued call checks cancellation at most every 10 ms while scheduled, and wakes on instance state changes. This polling is only a waiting policy, not an artificial delay in source execution.

Execution is cooperative and synchronous. Checkpoints surround dispatch stages, returned-origin checks, JSON traversal/serialization, and final publication. Snapshot cloning, result-shape validation, URL parsing, and individual serialization work are finite operations over bounded data; they are not forcibly interrupted midway. A request is observed at the next checkpoint. Once observed, no further dispatch stage is started; the result is discarded. No cancelled result or detached worker can later access the source. There are no I/O effects to undo. Cancellation/timeout delivery and disposal can wait for the current bounded stage and OS scheduling; this is not a hard real-time latency guarantee or a sandbox for arbitrary executable code. These mechanics must be reevaluated for JavaScript.

The monotonic deadline starts on entry to `invoke`, including admission and queue time. It is checked before execution, at checkpoints, and at publication. No result is accepted at or after expiration; `TIMEOUT` replaces it. Tests use private compile-time-only execution gates and a private clock override to establish race ordering, plus a real-clock queued timeout test. No test hook or controllable delay is exposed in production.

At publication, the instance-state and cancellation-token mutexes select one outcome. Observed disposal/cancellation wins over expiration, expiration wins over the computed result, and otherwise completion wins. Cancellation arriving after this decision cannot change the owned returned envelope. The synchronous return is the only delivery channel. Admission checks disposed state first, then the outstanding-call cap. For admitted ready calls, interruption checks precede declaration checks; declared calls then undergo input-budget checks before ordinary input validation. These simultaneous-fault/race choices are D1 implementation policy, not new portable precedence. The existing declaration-before-input rule is preserved. U2 shape-before-permission checking remains local policy.

`dispose()` permanently marks the instance disposed, wakes/cancels all admitted calls, and waits for their count to reach zero. Already admitted calls return `CANCELLED`; later calls get the host misuse error. It is idempotent and returns only after execution has quiesced, including discarded late results. No cleanup callback or storage deletion occurs. The immutable snapshot remains available for host inspection until the instance is dropped; transient call values are released as calls return. `Drop` also disposes. Reuse requires a fresh load/new instance, not reactivation of the disposed object.

| Invocation policy | Default | Configurable bounds and measurement |
| --- | --- | --- |
| Deadline | 5 seconds | Nonzero `Duration`, at most 60 seconds; monotonic time from call entry. |
| Input bytes | 65,536 | Inclusive compact JSON serialization size; hosts may lower to any nonzero value. |
| Result bytes | 8,389,632 | Inclusive compact JSON envelope size; hosts may lower to any nonzero value. |
| Input nesting | 64 | Fixed maximum nested object/array containers; checked before recursive serialization. |
| Outstanding calls | 64 | Active plus waiting, per instance; hosts may lower to any nonzero value. |

Byte measurement writes to a counting sink without allocating a serialized copy. It checks large strings/keys and nesting before serialization and observes interruption throughout counting. Exceeding invocation budgets yields `RESOURCE_LIMIT`. Fallback lifecycle/limit diagnostic envelopes (each below 128 compact JSON bytes) are exempt from the configured result budget so even a one-byte budget can report failure. Ordinary dispatched envelopes are measured; an over-budget envelope becomes `RESOURCE_LIMIT`. A success is never truncated. Limits are reference-host policy, not portable v0.1 constants.

Result construction copies only selected data from the loader-bounded immutable snapshot; invocation cannot add source data or raise load ceilings. At most one call per instance constructs a result; the result is measured before delivery. The result-byte cap is not a preallocation quota: selected data can be copied before that lower cap is checked. Memory is structurally bounded by finite loaded data, bounded input traversal, bounded admission metadata, and one call constructing a result per instance, with container/parser/allocator overhead. This is not precise heap/RSS accounting. Caller-owned input, retained results, host threads, total instance count, and process-wide memory limits belong to the embedding host. No custom allocator, storage quota, or general process sandbox is implemented; this profile has no persistent storage.

Cancellation, timeout, and checked budget exhaustion leave an instance safe to reuse: execution has quiesced before returning, the snapshot is immutable, and there are no external effects. An unwinding execution panic is treated as an unsafe internal context: return a confidential `SOURCE_ERROR` unless interruption already wins, permanently dispose the instance, and cancel waiters. Tests inject that fault through the private hook. Normal Source API errors do not dispose healthy instances. Abort-mode panics, allocation failure that aborts the process, and external termination are not recoverable outcomes; the library does not change the host's process-wide panic hook. This static policy is not a promise for future script engines.

## Execution harness and remaining scope

From the repository root, run:

```sh
cargo test --manifest-path runtime/Cargo.toml --locked --test declarative_conformance -- --nocapture
```

The [harness](tests/declarative_conformance.rs) validates the fixture schema, checks the corpus count and unique IDs, and freshly loads the named source for each of the 34 cases using production code. It constructs effective grants and a fresh `Instance` with default finite call limits and an uncancelled token, invokes the managed dispatcher, checks closed envelopes, and compares against the unchanged authored expectations. It prints `PASS`/`FAIL` with every case ID and fails the test if any case fails, including setup failures. Loader/grant/instance-setup or lifecycle-misuse failures are never converted to Source API results. Cookie/storage fixture grants must be false for this profile.

The independent test comparator ignores object order, preserves array order/types/missing versus null, and compares serialized decimal numeric values so `1` and `1.0` match without collapsing adjacent large integers through floating-point conversion. Error codes are exact; message wording is not compared, but the field must be a nonempty string. Expected outcomes are never generated by the runtime. The Python suite remains independent structural validation of these data files.

Rust tests include the existing loader/schema tests, malformed-result unit checks, focused dispatcher/permission tests through managed calls, lifecycle synchronization/budget tests, the comparator test, and execution of all 34 cases. Coverage includes declaration precedence, input/default/page/lookup rules, exact search, known/unknown IDs, media/poster denial, normalized origins, per-call grants, and independent returned copies.

Still absent: precise process/allocator quotas, preemptive CPU termination, consumers/redirects, host-service bindings, JavaScript, legacy adapters, platform applications, and WASM. U1/U2, P1, and the broader D1/D2 portability surfaces remain deferred. Only local macOS execution is demonstrated. The closure review assessed these tests together with the normative requirements and implementation boundaries and accepted Phase 2 completion; it does not certify all v0.1 behavior. Phase 3 is active for a bounded JavaScript execution-binding RFC, not implemented JavaScript execution.
