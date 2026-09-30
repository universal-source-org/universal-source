# QuickJS complete patch-set maintenance acceptance

Date: 2026-09-30. Reviewed baseline:
`6fe23ddefc3274d8986f87164cfbca0730e33679`, `main`, initially clean, no active plan.

**Outcome: Fail — maintenance acceptance is not established at this baseline.**
The complete terminal-failure artifact cannot be identified, its successful
execution evidence cannot be bound to an exact combined source configuration,
and accountable maintenance/security ownership has not been accepted. Universal
Source cannot reasonably accept an unidentified, unowned patch set. Stop default
QuickJS selection investment; retain it as an option in the comparison, with
its existing evidence preserved. This is a failed maintenance gate, not a claim
that QuickJS is technically impossible or permanently unmaintainable.

**Only next task: end-to-end candidate architecture cost comparison under the
same RFC.** Do not implement another QuickJS fix, redo terminal-OOM feasibility,
start another selection gate or launch alternative-engine/IPC experiments.

## Authority and evidence standard

The [candidate-cost review](2026-09-30-quickjs-candidate-viability.md#only-next-task)
made continuation conditional on an auditable full artifact, explicit ownership
and a sustainable update path. This review evaluates those conditions rather
than repeating its full cost ledger. The [charter](../PROJECT_CHARTER.md),
[principles](../DESIGN_PRINCIPLES.md), [CONTRIBUTING](../../CONTRIBUTING.md) and
proposed [RFC 0001](../../spec/rfcs/0001-javascript-execution-binding.md) remain
unchanged. Phase 3 stays active; no engine selection ADR or RFC acceptance occurs.

Terminal allocation/heap failure is **feasible by the task premise**. Feasibility
is neither disputed nor used as a substitute for source provenance. An absent
artifact is evidence that this gate fails, not permission to reconstruct the
missing implementation or to infer its size, cleanup behavior or test results.

Evidence routes followed: the candidate review, [async patch audit](2026-09-29-quickjs-async-termination-fix-spike.md),
[public-API boundary](2026-09-30-quickjs-public-api-failure-classification.md), then
their linked drivers, [integration review](2026-09-30-quickjs-integrated-resource-failure-spike.md)
and guides for precise artifact, ownership and sanitizer questions. Historical
reviews are not rewritten. Upstream findings from 2026-09-29 remain dated;
no live upstream/security status, upstream promise or new platform proof is claimed.

## Artifact search and what was actually found

The read-only search covered repository source/path inventories, local branches
and reachable experiment history, the one registered worktree, relevant ignored
experiment build/target artifacts, and eleven distinct local app snapshot trees.
Snapshot path inventories revealed no terminal-OOM experiment/patch and contained
only one version each of the two existing patch/build drivers. No remote fetch,
external filesystem hunt, reflog recovery, patch application or rebuild occurred.
This establishes absence from the inspected repository evidence, not from every
possible machine or unpublished external location.

| Inspected surface | Result |
| --- | --- |
| Current tracked experiments and local history | Async fix introduced in `3c3ffa4`, integration in `004606e`, public-API diagnostics in `06b8ea6`. No terminal-OOM implementation or successful combined-run artifact identified. |
| Authored experiment text and patch/terminal/OOM path searches | No additional terminal-OOM patch, driver or success record identified. Existing public-API tests assert catch/finally continuation and the async-only engine digest. |
| All 18 local `quickjs.c` copies under `experiments/`, including ignored targets | Every digest is either the original pin or the async-only patched digest below. No third source configuration. |
| Existing C baseline/patched trees | Thirty corresponding C/header files; only `quickjs.c` differs. All thirty baseline files match the installed pinned registry copy. |
| Integrated baseline/patched provenance records | Each recorded source, archive and executable hash matches the corresponding existing file: six checks total. Both records say `sanitized: false`. Each executable exports exactly one of each of the three driver witness symbols. |
| Current integration `build/sys/quickjs/quickjs.c` | Original baseline digest, while `target/patched` contains the async digest. The driver reuses `build/sys` for baseline and patched runs. A directory name or resolved Cargo path alone does not attest to the engine in a previously built executable. |
| Maintenance/security records | No committed maintainer acceptance, CODEOWNERS or engine-update workflow found. [SECURITY.md](../../SECURITY.md) is a tracked zero-byte file. Ordinary Git authorship is not acceptance of engine/security maintenance. |

The generic ignored `build/provenance.json` also names only the async digest and
has no executable hash. More detailed `patched-provenance.json` and
`baseline-provenance.json` identify their binaries, but the current driver writes
these records **before** test/fmt/clippy execution. A matching provenance JSON is
not a successful test attestation. Dated reviews supply the existing execution
claims; none claims a terminal-OOM success on a new digest. The presence of a
`target/patched-sanitized` source file likewise proves neither successful linking
nor a sanitizer run.

## Exact inspectable provenance

The durable patch artifact is the `PATCHES` list in
[build_and_run.py](../../experiments/quickjs-async-termination-fix-spike/build_and_run.py),
not a committed engine fork. The generated C copies were compared without
reapplying those rules. Rule anchors were inspected through Python AST data,
without running the driver. Known engine revision: QuickJS-NG 0.16.2,
`0fdea21ff1090084e91dad812b343d92e79ba9d9`; rquickjs/core/sys 0.14.0,
`d7ef5eeae702fea24c03643064de454f1c1dd4b0`. Installed sys VCS metadata matches that
binding revision. The following SHA-256 values were checked in this audit.

| Object | SHA-256 |
| --- | --- |
| Original `quickjs.c` | `3a6b52a225c21709ef1fd314183efd3af862e867ce4ef8caabcae67922a745b3` |
| Async-only `quickjs.c` | `9a926c4ed02517c84ecfbc9d925b6ceb782b3119f6d05a3d58280c66363e1f3f` |
| Async driver / patch recipe | `3711cab84f268cdaadc4c3d59c91f14421dd7ff6a6c7eaf4f0ea2a6fde7197e3` |
| C harness | `80ef7b3f04a2220864719173960878b807410fdf1884c332da775dce312c4b5d` |
| [Integration driver](../../experiments/quickjs-integrated-resource-failure-spike/run.py) | `c7c661c4f4ec0325f42cfd9eac0d1db1e95dd0a6d9a99c6fd70387a2a9859d0d` |
| [Integration lockfile](../../experiments/quickjs-integrated-resource-failure-spike/Cargo.lock) | `19f53e5d454aa5e01c112dc4c7f043b5b30e991da2a0f517a6e58240e6a4a73f` |
| Pinned public `quickjs.h` | `979127138da79cad5ddc7effa1afb13e8b02f744da2aa36844d4cb9d8cb92de9` |
| Bundled QuickJS MIT license file | `96f73f9d2a16c21a36b418f06073be26e7d6d5e7c1bc99756b21a4f2c74ef171` |
| Existing patched Rust executable | `51e075bd64bffe34e53e4685a5106c74486b43dc46d9577270643b3d4edac96e` |
| Existing patched `libquickjs.a` | `62769d5f9a129167b090847e51f80b9da9d1b69ee72f7d3eafaa887fc97c8d58` |
| Existing baseline Rust executable | `d9ba8906812db4520b354daaaab2aaab5fbf15de96986275e839fc3b266835ac` |
| Existing baseline `libquickjs.a` | `8bd0f810430cfcbc3843389fab0458f83e5f9ce011e2c76ecbf4df10fb5677b8` |

The two detailed records are under
`experiments/quickjs-integrated-resource-failure-spike/build/`. Each names its
`target/{baseline,patched}/debug/deps/universal_source_quickjs_integrated_resource_failure_spike-483fca2de0df130f`
executable and corresponding
`debug/build/rquickjs-sys-780dbfd26adc67d1/out/libquickjs.a` within that target.
These paths describe existing local outputs, not portable artifact identifiers.

The repository commit plus recipe and harness identify the durable experimental
inputs. Local generated binaries are supporting observations, not committed
release artifacts or a clean-room reproducibility proof. A hash of `quickjs.c`
alone is not a whole engine/header/build-source identity. The thirty-file
comparison above improves this audit but does not create a complete source-bundle
manifest, toolchain attestation or production acquisition pipeline.

**Terminal-OOM artifact, combined digest and successful-run configuration:
unavailable.** There is no honest exact full-set count or patch ordering to report.
The known subset below must not be presented as the complete candidate patch set.

## Actual sites, order and responsibility boundaries

The inspectable async subset is **one changed engine file, eleven functions,
twelve sites, eleven ordered replacement rules** (one rule matches two sites).
Read-only unified diff: **43 added / 4 removed lines, twelve hunks**. These are
inventory counts, not a labor estimate or proof of low maintenance cost. No
header, struct, function signature or new function is added by this subset.

The table follows the actual recipe's application order. Lines identify the
start of each original matching anchor in the verified baseline `quickjs.c`;
they are not stable identifiers across upgrades.

| Rule | Function / baseline anchor line | Route and ownership/propagation obligation |
| --- | --- | --- |
| 1 | `js_promise_constructor`, 55915 | Executor exception: guard before extraction; use existing `fail` cleanup for argument functions and object. |
| 2 | `js_promise_resolve_function_call`, 55804 | Then-getter failure: preserve pending exception and return `JS_EXCEPTION`; do not settle a rejection. |
| 3 | `js_promise_resolve_thenable_job`, 55685 | Then-call failure: skip rejection conversion; preserve existing argument frees and return exceptional `res`. |
| 4, first match | `js_promise_all`, 56240 | Iterator/element failure: use existing `fail`/`done` cleanup, not rejection dispatch. |
| 4, second match | `js_promise_race`, 56381 | Same replacement, separate function's cleanup. Both matches required. |
| 5 | `js_promise_try`, 56107 | Free both resolving functions and result Promise before exceptional return. Test overcoverage; this later feature does not expand RFC ES2023. |
| 6 | `js_async_generator_resume_next`, 21890 | Void helper returns before consuming exception; queued request remains generator-owned for retirement. Requires rules 7 and 8. |
| 7 | `js_async_generator_next`, 22007 | Propagate helper's pending exception, free locally owned Promise. |
| 8 | `js_async_generator_resolve_function`, 21969 | Propagate helper's pending exception to the already-guarded reaction path. |
| 9 | `js_async_generator_completed_return`, 21811 | Return `-1` without exception extraction, matching existing exceptional-Promise ownership state. |
| 10 | `js_async_from_sync_iterator_next`, 56769 | `reject` path: free resolving functions and Promise, return exceptional result. |
| 11 | `js_async_from_sync_iterator_next`, 56743 | `IteratorClose` failure: same owned-value release and terminal propagation. |

Actual known order is **verified baseline → the above async recipe → C harness
or experiment-local sys build**. The Rust driver imports the same recipe; it does
not layer a second terminal-OOM patch. No order between the missing OOM change
and this recipe is established. A future complete audit must identify overlaps,
unique sites and before/after hashes at each layer; it must not infer that the
two responsibilities are independent just because they have different names.

| Responsibility | Actual evidence | Acceptance finding |
| --- | --- | --- |
| Create trusted terminal allocation/heap failure | Feasibility premise accepted. Public-API diagnostics distinguish allocator refusal from pre-allocator engine heap rejection and null Error-allocation fallback. No implementing engine artifact found. | Cannot audit edited allocator/error/interpreter state, runtime fields, public ABI changes, overwrite/clear rules or cleanup dependencies. Required scope is unknown, not zero. |
| Preserve terminal state through Promise/async and cleanup | The recipe above preserves an **already uncatchable Error** before exception-to-rejection conversion; dated C evidence covers cleanup and fresh runtime health. | Enumerable and concentrated for this subset. It does not create OOM origin/state or prove propagation of the missing implementation's representation. |
| Bind terminal state to host outcome and retirement | Integrated allocator latch, job handling, RegExp gate, §6 transfer and no-drain retirement exist experimentally. Public-API tests still record OOM continuation on the known digest. | Host action/publication suppression is not proof of no source continuation. Full combined success cannot be inferred from separate component tests. |

The maintained audit surface exceeds changed sites: all exception extraction,
replacement/clearing, catch dispatch, async resume callers, job failure handling,
allocator/heap entry points, Error-allocation fallback and native cleanup. The
prior 36-site conversion audit is a pin-specific starting point. Excluded dynamic
import and resource-management syntax must stay excluded by preflight; a new
language feature can make an unpatched route reachable. §6, hardening, admission
and lifetime regression responsibilities remain as in the candidate-cost ledger.

## Ownership model: required roles are not assigned people

The following is a concrete **required operating model**, recorded for any future
reconsideration. Defining it in a review does not establish an operational team,
accepted SLA, enabled CI or authority to publish.

| Responsibility | Required accountable role and record | Current reality |
| --- | --- | --- |
| Engine patch maintenance | Named maintainer accepts complete patch inventory, rebase work and native exception/allocator semantics; acceptance links to the exact supported configuration. | No accepted appointment or full inventory. |
| Independent change review | Named competent reviewer signs site/cleanup and binding qualification findings, including unresolved risks and exclusions. | No standing reviewer commitment. This agent audit is not such a commitment. |
| Dependency/security intake | Assigned person monitors engine, binding and relevant parser dependencies, owns incoming reports, triage and tracking until resolved. | No assigned coverage, reporting channel or response policy in the empty SECURITY file. |
| Build/release qualification | Assigned owner verifies single-engine artifacts and qualification records, blocks unsupported configurations and can suspend unsafe adoption. | Experimental drivers exist; no maintained release pipeline or accepted owner. |

One person may hold multiple operating roles if capacity is explicitly accepted;
independent review still needs another competent reviewer. No number of engineer
days is derived from lines or tests. The project must explicitly agree monitoring
frequency and response expectations with available capacity before accepting the
duty. This review assigns neither a person nor a deadline. The absence of that
agreement is a gate failure, not a field the agent may fill with a guess.

## Required update, security and upstream procedure

This procedure is defined but **not operationally accepted or executed**. It does
not authorize engine edits or upstream messages in this task.

1. **Open an update record.** Record reason (release, security fix, binding or
   profile change), old/new source revisions, license/source acquisition,
   patch layers, binding/parser lock differences, target/toolchain/features,
   accountable owner and reviewer. Keep the previous qualified tuple immutable.
   A rquickjs update must expose any bundled engine change explicitly.
2. **Review upstream delta before rebasing.** Read touched allocator/error,
   interpreter, Promise/async, module, object/string and RegExp paths; enumerate
   new sites as well as old anchors. Classify each exception-to-rejection or
   state-clearing route, cleanup ownership and excluded feature. A clean textual
   apply is not an audit. An anchor/hash/count mismatch stops qualification;
   do not refresh expected hashes or mechanically resolve conflicts as approval.
3. **Rebase a complete identified set in a separately authorized change.**
   Record each layer's inputs/output digest and overlap, ABI/header changes,
   ordinary-error behavior and the trusted terminal-state invariant. Remove a
   local patch only after upstream equivalence is established through the same
   behavioral boundary, not on a release note or issue closure alone.
4. **Qualify the coupled configuration.** Run the regression obligations below
   against the exact built artifact; preserve negative controls separately.
   Attach commands, corpus revision/hash, mode, target, exit status, logs and
   binary digest. Reviewer signs findings only after successful evidence exists;
   source-copy provenance written before testing is insufficient.
5. **Accept or suspend.** An accepted update records supported configurations,
   residual gaps and rollback tuple. On failure, stop promotion and keep the
   last qualified tuple only if it remains security-acceptable. Never roll back
   to a known affected revision just to recover green tests; if no safe qualified
   configuration exists, suspend JavaScript adoption/use and reopen candidate
   suitability. There is currently no production JS deployment to roll back.

Security intake must cover QuickJS-NG source/release/advisory changes, rquickjs
including sys build/bindgen/allocator changes, and parser dependencies involved
in admission. Read fixes without CVE labels as well. For each relevant report,
record affected configurations, reachability under the actual profile,
consequence, mitigation, decision owner and disposition. “Hidden global” alone
is not proof an internal engine defect is unreachable. Choose update or narrowly
reviewed backport, then apply the same qualification requirements. Missing
capacity or inability to qualify a necessary fix requires suspension, not an
indefinitely frozen vulnerable pin. No current vulnerability assessment was made.

For upstream submission, an authorized maintainer would prepare exact minimal
patches, public-API reproducers/regressions, ordinary-semantics and ownership
arguments, license/provenance and the profile boundary. Track upstream response
and equivalent implementations separately for creation and propagation of
terminal failures. No submission is made or acceptance promised here. Rejection,
indefinite non-response or incompatible semantics leaves local carry with the
project; it requires explicit renewed capacity/architecture assessment. If that
cannot be accepted, suspend default investment and compare architectures rather
than silently expanding the fork.

## Single-engine build and regression qualification

A future accepted tuple must bind engine source bundle/header/license digests,
ordered patch content and output digests, rquickjs/core/sys and parser locks,
hardening/admission/FFI/corpus revision, toolchains, target, C/Rust flags and
features. Retain clean acquisition instructions and per-mode isolated outputs;
source reproducibility is distinct from byte-identical binary reproducibility.
Prevent ambiguous reuse of a mutable `build/sys` or stale target fingerprint as
evidence for another mode. Record the exact archive and final executable used.

The existing driver checks one resolved sys crate, compiled C digests, and one
exported definition each of `JS_NewRuntime2`, `JS_ExecutePendingJob` and
`JS_SetUncatchableError`. Those are useful witnesses, freshly rechecked on the
existing binaries. A maintained qualification must additionally inspect linked
archives/dependencies or a link map to exclude a second engine hidden by symbol
visibility, and match headers/generated ABI/configuration to the linked library.
It must retain complete source identity and bind successful results to that
artifact. These stronger qualifications are requirements, not completed proofs.

| Regression obligation on a relevant update | Existing basis and remaining qualification |
| --- | --- |
| Terminal creation and origin | Require existing terminal-OOM evidence tied to exact combined source: allocator and pre-allocator heap refusal, Error-allocation failure/null fallback, trusted reason, no catch/finally/effect/success continuation. Artifact and combined success are absent here; do not reinterpret the public-API negatives as passing terminal proof. |
| Terminal propagation and cleanup | Async matrix, both native-gate/deadline mechanisms, caller propagation and fresh-runtime health; preserve ordinary null/forged Error/RangeError/stack controls, exceptional returns and reference cleanup. Re-audit new routes. |
| Host/binding composition | Setup/module initialization, direct call/jobs, inbound/outbound §6 enumeration/string/copy failures, allocator ownership, late delivery, helper identities, one-job pumping and no final drain. No source reentry or unwind across C. |
| Permanent host policies | RegExp compile-route/native recovery/coercion and literal admission; hardener inventories and dynamic-code routes; full restricted syntax/parser agreement needed to justify excluded engine paths. Do not casually replace the existing oracles with updated snapshots. |
| Memory-safety evidence | Dated C ASan/UBSan plus QuickJS leak-abort results apply to the C async harness. Rust integrated sanitizer linking failed on local Darwin arm64; no full Rust/C sanitizer success exists. A future qualification must provide a working supported instrumented integration or an explicitly reviewed equivalent assurance argument with residual limits; this review grants no waiver. |
| Target qualification | Record same engine/binding/patch configuration and representative execution by named OS/architecture/device and build mode. Apple mobile/TV and wider platform proof remain open. A sanitized source copy, compile-only result or simulator cannot be labeled device/runtime proof. |

Shared engine/semantic changes require the appropriate coupled regression corpus,
not just the changed site's unit test. That describes later authorized upgrade
qualification; it does not justify rerunning all Phase 3 for this read-only audit.

## Acceptance decision and next task

| Acceptance condition | Finding |
| --- | --- |
| Full terminal-OOM plus async artifact and exact source-bound success | **Fail:** missing; only async subset and public-API negative configuration identified. |
| Complete bounded changed-site/cleanup/ABI/order inventory | **Fail:** known async subset is concentrated; full set cannot be enumerated. Unknown scope cannot be certified acceptable. |
| Accepted maintainer/reviewer and security/update responsibility | **Fail:** required roles/procedure are now explicit, but no accepted people/capacity or operational coverage exists. |
| Sustainable single-engine qualification and security fallback | **Not established:** experimental witnesses are useful, but complete provenance, successful combined evidence and operational acceptance are absent. |
| Cross-platform and integrated sanitizer evidence | **Open:** preserved limits, neither silently waived nor treated as new platform failure. |

This is **Fail**, rather than “technical scope accepted pending paperwork”:
the full technical scope itself cannot be audited, and the ownership condition
that justified the previous Conditional continue is unsatisfied. No claim is
made that the missing patch is necessarily large or that future ownership is
impossible. The present evidence is insufficient to accept the obligation or to
keep QuickJS the default destination for more selection work.

The **only next task** is an **end-to-end candidate architecture cost comparison
under the same RFC**: compare complete prospective termination/resource, host
policy, preflight, binding, target qualification and security/update ownership
models, retaining QuickJS as one option and reusing existing test oracles. Do not
start V8/JSC/Boa experiments, design helper-process/IPC architecture or weaken the
portable contract. No alternative is selected by this failure. Finding an
external artifact later would be new comparison evidence, not an automatic gate
pass or authorization to resume implementation.

## Validation and scope

Fresh checks were read-only artifact inventory, SHA-256 comparisons, C/header
differences, AST rule/anchor counts, source diff review, existing binary symbol
counts and six provenance-file hash matches. The `git diff --no-index` exit 1
means the expected source difference, not a failed engine test. No engine driver,
patch application, build, execution/sanitizer test or upstream check was run.

The [completed plan](../plans/completed/2026-09-30-quickjs-maintenance-acceptance.md)
records Markdown/link/anchor, whitespace, complete-diff and status checks. Only
this review, its plan and affected context documents change. All artifacts,
ignored generated output, historical evidence, locks, spec/RFC and ADRs remain
unchanged. No production JS, maintained fork, engine selection, publication or
repository-setting change is authorized.
