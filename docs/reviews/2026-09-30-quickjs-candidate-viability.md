# QuickJS-NG candidate cost and selection readiness

Date: 2026-09-30. Baseline: `06b8ea646d028cda7f869ff7f3875b8acbb7146e`,
`main`, clean tree, no pre-existing active plan.

**Outcome: Conditional continue.** Retain QuickJS-NG as a candidate, but advance
only to a **complete patch-set maintenance acceptance gate** before investing in
other selection gates. Its architecture has moved materially beyond a nearly
unmodified upstream interpreter with a few host checks. It now requires engine
failure-semantic maintenance, a permanent restricted-runtime policy layer, an
independent syntax-validation boundary, and a carefully owned Rust/C binding.
That can still be a defensible small-runtime architecture, but the evidence does
not establish that this project can sustainably own the whole arrangement.

Terminal allocation/heap failure is treated as **engine-level feasible**, as
specified by this task. The decision is whether to carry its cost, not whether
to attempt another fix. Neither past experimental investment nor absence of a
proved alternative is a sufficient reason to keep investing. No engine selection,
maintained fork, parser choice, RFC acceptance or production integration follows.

## Evidence boundary and decision basis

The [charter](../PROJECT_CHARTER.md), [principles](../DESIGN_PRINCIPLES.md) and
proposed [RFC 0001](../../spec/rfcs/0001-javascript-execution-binding.md)
supply the constraints: portable execution, explicit authority, small auditable
boundaries and unchanged source semantics. The [previous consolidated review](2026-09-29-consolidated-engine-feasibility.md)
is the starting assessment, qualified by the [integrated experiment](2026-09-30-quickjs-integrated-resource-failure-spike.md)
and [public-API follow-up](2026-09-30-quickjs-public-api-failure-classification.md).
Only component evidence needed for the cost questions below was followed; the
historical archive was not reread wholesale.

There is an evidence-location mismatch that must not become invented provenance.
At this HEAD, the checked-in reviews, experiment driver and tests end at the
public-API Outcome B. They contain the async patch but no terminal-OOM engine
patch, site audit or successful execution record for it. The task supplies the
later feasibility conclusion. This review accepts that conclusion as a premise;
it cannot independently report the missing patch's lines, functions, digest,
cleanup proof or combined test results. This is an **artifact/audit gap**, not a
recommendation to repeat terminal-OOM feasibility. The current public-API
configuration remains demonstrably insufficient; the intended patched candidate
is a different configuration. Historical negative results remain intact.

The inspectable pin is rquickjs/core/sys 0.14.0 and QuickJS-NG 0.16.2 at
`0fdea21ff1090084e91dad812b343d92e79ba9d9`. Prior upstream observations are dated
2026-09-29; this review makes no fresh claim about upstream releases, security
status, platform availability or acceptance of any patch. No network research
or executable experiment is necessary for this repository-evidence assessment.

Labels below distinguish **fact** (recorded local evidence), **premise** (task
input), **gate** (not closed), **cost** (continuing work if adopted) and **risk**
(a plausible failure not demonstrated as inevitable).

## Engine changes: small text is not the maintenance boundary

| Patch responsibility | Size and intrusion | Failure mode and recurring audit |
| --- | --- | --- |
| Promise/async terminal propagation — **fact** | [Async patch review](2026-09-29-quickjs-async-termination-fix-spike.md): about 40 added lines, 12 edits across 11 functions in `quickjs.c`; ten conversion guards plus two async-generator caller-propagation edits. No new functions, signatures or structures in this patch. All 36 `JS_GetException` sites were classified. | Missed rejection conversion lets source continue after a trusted stop. The void-helper/caller case initially failed to return control; mechanical guard insertion was insufficient. Re-audit exception extraction/conversion, resume callers, pending exceptions, reference ownership and cleanup on every upgrade; preserve ordinary catches/rejections and leak-free fresh-runtime retirement. |
| Allocation/heap terminal classification and propagation — **required cost; feasibility premise** | A second semantic responsibility beyond propagating already-uncatchable Error objects. The public-API evidence establishes heap rejection before allocator notification and an OOM null fallback that cannot be marked. Actual patch size, edited functions/files, runtime fields/API changes and overlap with the async patch are **not available at this baseline**; no total line count or low-intrusion claim is justified. | A lost origin/terminal state permits catch/finally, capability action or success; overclassification can make ordinary/null/stack exceptions terminal. Failure while constructing the exception must not erase the trusted reason. Audit allocator and engine-ceiling refusal, exception creation/replacement/consumption, catch dispatch, Promise propagation and cleanup against the actual patch. These are obligations to inspect, not a proposed patch design. |
| Combined patch set — **gate** | The integration driver applies only the existing async edits. Its single-engine linkage proof is useful but cannot attest to the intended additional terminal-OOM change. Count unique changed sites and patch ordering when the complete artifact is available; two responsibilities need not mean two independent patch files. | Independent successes can compose incorrectly if one path consumes/replaces the other's pending exception or loses the host reason. Require linked source/patch identity and the existing combined failure matrix. Do not assume C-harness evidence or a clean rebase proves the Rust path. |

The async patch has source SHA and replacement-count guards, sanitizer and
QuickJS leak-abort evidence, and ordinary-error controls. These reduce audit
uncertainty; they do not supply a maintainer. The [integrated guide](../../experiments/quickjs-integrated-resource-failure-spike/README.md)
and [driver](../../experiments/quickjs-integrated-resource-failure-spike/run.py)
verify one experiment-local `rquickjs-sys`, compiled C source and exported symbols.
They are experimental provenance machinery, not a release/update pipeline.

The previous recommendation explicitly required reconsideration if resource work
needed another substantial engine change. That trigger is now engaged at the
semantic level: allocation failure is below ordinary host throws and async
conversion. This review performs that reconsideration. Even assuming the missing
patch is short and correct, maintenance now spans both **creating a trustworthy
terminal failure** and **preserving it across execution machinery**. Treating the
sum as the old 40-line compatibility adjustment would understate the obligation.
No GC/compiler rewrite or general engine fork is evidenced or authorized here.

Both patches are indefinite carry costs unless equivalent upstream behavior is
adopted and requalified. Upstreaming could remove local diff maintenance; it would
not remove the terminal invariant, regression corpus or security-update duty.
Neither upstream acceptance nor a permanently frozen pin is an acceptable assumed
exit strategy.

## Host policy, dependency and binding cost ledger

| Surface | Evidence-backed extent | Long-term cost, open gate and failure risk |
| --- | --- | --- |
| RegExp admission | [Implementation evidence](2026-09-29-quickjs-regexp-admission-impl-spike.md): 30 dynamic routes, exact-byte literal preflight across imports, native constructor hidden, clone readmission, species/subclass/generic-receiver handling. Constructor plus `compile`, String `match`/`matchAll`/`search`, generic `@@split`/`@@matchAll` need coverage. | **Permanent for this candidate architecture.** Audit every native compile entry and recovered-constructor path; preserve single coercion and ES2023 order. This is a semantic facade, not one length check. New flags/routes or changed fast paths can bypass admission. The current `v` acceptance remains an edition gap. |
| RegExp and other native-work limits | Candidate 4,096-unit pattern cap measured roughly 81–86 ms worst local debug compile; matching work also depends on subject size. Integrated evidence records finite work between polls in `Array.join`. | Publish finite pattern/input/graph/job/heap policies and calibrate on target builds. Those timings are not portable ceilings; heap and §6 budgets do not bound every native loop. **Gate:** remaining admitted-work coverage. **Risk:** a future short-pattern complexity defect or other native loop requires new policy/engine work. No such future patch is assumed cheap. |
| Independent parser/preflight | [Attribute evidence](2026-09-28-javascript-import-attributes-spike.md): Boa 0.22.0 structural analysis plus Oxc 0.152.0 clause presence; Oxc addition brought four direct dependencies and 32 lockfile packages (103 external total in that experiment). RegExp preflight uses raw literal measurement before validating parsers/compiler. | **Permanent obligation**, but two parsers are not selected or necessarily permanent. Maintain exact-byte identity, grammar/early-error coverage, clause ownership, regex/flag agreement, memory/recursion/time bounds and fail-closed diagnostics. Parser disagreement can reject valid packages or miss forbidden syntax. Oxc parsing alone was not proved complete early-error validation. No parser consolidation savings are credited yet. |
| Restricted runtime and compiler denial | [Globals](2026-09-28-quickjs-global-surface-spike.md) and [compiler denial](2026-09-28-quickjs-dynamic-code-suppression-spike.md): 38 global names, 72-object retained-intrinsic inventory, Symbol facade, stack/clock/locale/random restrictions and 80 dynamic-code routes. Setup precedes all package evaluation. | **Permanent host layer.** Upgrade inventories by review, never snapshot refresh alone; audit hidden prototype/constructor edges, removed aliases, helper injection and late intrinsic installation. Bounded graph walks are evidence, not proof against engine memory corruption. Deleting globals does not establish syntax exclusion. |
| §6 transfer | [Complete values](2026-09-28-quickjs-complete-value-spike.md): positive class admission, descriptors without executing source, strict Unicode, dense arrays, cycle/expanded-copy budgets, helper identity registry and safe own-data construction. Four audited unsafe blocks in the class/complete-value path. | **Permanent binding obligation.** Re-audit class semantics, ownership/free rules, descriptors, string buffers, context lifetimes and helper identities. Public C APIs reduce layout dependence but do not eliminate implementation-semantic review. Enumeration/UTF-16 extraction may allocate before output budgets apply. These four blocks are not the total runtime unsafe surface. |
| Additional Rust/C and allocator boundary | Admission adds two public-API unsafe calls. The integrated [FFI](../../experiments/quickjs-integrated-resource-failure-spike/src/ffi.rs) handles jobs/limits/diagnostics, while its [allocator](../../experiments/quickjs-integrated-resource-failure-spike/src/allocator.rs) implements unsafe allocation ownership and accounting. | Own null/realloc/overflow behavior, correct freeing, exception ownership, no callback reentry/unwinding across C and one engine ABI/configuration. Distinguish negative job result from empty queue. Engine-accounted bytes, adapter estimates, Rust/host allocations and process RSS are different. Integrated Rust sanitizer linking failed locally; C sanitizer success does not cover the whole binding. |
| Lifecycle and capability gates | [Lifecycle proof](2026-09-28-quickjs-lifecycle-spike.md): fresh Runtime/Context, owner-thread execution, explicit one-job pumping, no final drain, rooted-value release, scope/generation checks and late-delivery rejection. | **Permanent Model B cost:** repeated setup/hardening/module initialization and copied data, explicit root/error/resolver ownership and service revocation. Startup/memory/throughput on target devices remain unmeasured. Enabling rquickjs futures/parallel scheduling is not a free feature toggle; it changes the audited boundary. Exact services remain separate later work. |
| Module capture and containment | [Capture proof](2026-09-28-quickjs-module-capture-spike.md): private native dependency captures linked exports before package bodies; no public standalone link-only return is used. Reflection cannot replace declaration analysis. | Capture-order/private-name/identity audits continue on engine and binding updates. **Gate:** canonical exact-case contained paths, symlink aliases/cycles, stable bytes, graph budgets and source denial of host-private modules. Phase 2's static loader and fixture resolvers do not close this gate. This is substantial binding work, not another proven engine patch. |
| Platform and distribution | Only local Darwin arm64 experiment execution is recorded. | **Gate:** same engine/patch/binding/header/flags arrangement on charter targets, especially Apple mobile/TV device embedding and Android/Windows/Linux. Signing, target toolchains, ABI/stack margins and representative runtime/FFI failures need evidence. No-JIT architecture is helpful, not device/distribution proof; simulators/builds alone do not prove device behavior. No platform blocker is newly established here. |

The candidate therefore carries two engine failure-semantic responsibilities,
one interdependent hardening/admission layer, one independent preflight boundary
(current evidence uses two parser families), an audited native binding and a
multi-target qualification obligation. It is not merely “QuickJS + rquickjs”.
The costs interact: parser exclusion makes unpatched async routes unreachable;
hardening protects the RegExp facade; allocation handling protects §6 and guard
construction; lifecycle policy prevents leftover work and stale capability use.
Passing these parts independently is weaker than preserving their composition.

## Upgrade, security and ownership model

Adoption would require the following recurring work, even if source patches apply
cleanly. This is a proposed acceptance bar, not an implemented process or a claim
that anyone has accepted ownership.

| Trigger | Review/test surface required |
| --- | --- |
| QuickJS-NG update or security backport | Complete patch-site and new-path audit; terminal/OOM/ordinary-error matrix; Promise/async cleanup; compile-route complexity/admission audit; hardener inventory and hidden identities; module ordering; descriptor/string/class semantics; combined resource/lifecycle regression and native sanitizers/leak checks. |
| rquickjs/core/sys update or feature change | Match engine headers, generated bindings, C flags, allocator API and exception/job semantics; verify one patched engine actually linked; recheck rooted handles, owner-thread lifetime, FFI and build scripts. A binding update can change the bundled engine even if local code is unchanged. |
| Boa/Oxc or replacement preflight dependency update | Baseline syntax and early-error corpus, attributes, raw regex measurement before expensive validation, original-byte/graph agreement, resource bounds and rejection of routes excluded from engine patches. Correct false refusals without silently narrowing the portable contract. |
| Host bootstrap, service/helper or profile change | Compile-denial and native-identity escape corpus, helper registry, permission/lifetime gates, no late intrinsic restoration and failure during setup/transfer. A profile expansion reopens previously excluded engine sites. |
| Compiler/OS/architecture/build configuration change | Target ABI/linkage, stack/allocator behavior, representative stop/retirement/value tests, supported sanitizer coverage and resource calibration. Record device versus simulator and debug versus release. |

Provenance must cover engine revision and license, reviewable patch content and
order/digests, binding/parser locks, acquisition/build instructions and target
configuration. Existing source hashes are a useful start; deterministic patch
application is not binary reproducibility. Generated ignored experiment trees
cannot be the only durable description of a maintained patch.

An accountable maintainer and a competent reviewer must explicitly accept
exception/allocator/FFI audit responsibility and upstream monitoring. A written
update/security procedure must cover upstream fixes without CVE labels, triage,
a bounded response expectation, backport/rebase choice, qualification and rollback
or suspension when an update cannot be made safe. This review does not invent a
person, staff allocation or response SLA. No measured annual effort or update
history exists; line counts cannot honestly be translated into person-days.

Prefer upstream submission of minimal changes and regressions; do not depend on
acceptance. An acceptable fallback needs capacity to carry and update the complete
set. If upstream rejects the semantic approach, the project must explicitly
reassess carrying it. No upstream message or submission is sent in this task.

## Distance from the near-upstream ideal

| Ideal assumption | Actual candidate | Cost judgment |
| --- | --- | --- |
| Native termination behavior meets host needs | Async propagation patch plus terminal allocation/heap handling | Material deviation in security-sensitive failure semantics; not a cosmetic compatibility patch. |
| A few public limit settings suffice | Admission facade, native-work policy, engine/host accounting and target calibration | Substantial recurring policy and semantic audit surface. |
| Compiler is the sole syntax authority | Separate profile preflight with experimentally combined Boa/Oxc checks | Additional dependency, resource and consistency boundary; strategy unresolved. |
| Safe binding mostly supplies the boundary | Public-C FFI, non-executing custom transfer, private capture ordering and manual retirement | Concentrated but demanding ownership; convenience APIs cannot substitute blindly. |
| Portable interpreter implies portable embedding | Common C/Rust route is plausible; target build/device evidence absent | Significant unclosed integration gate, not a demonstrated failure. |

Some cost comes from the contract regardless of engine: Model B, §6 copying,
capability revocation, module containment and the restricted language profile.
The incremental QuickJS burden is the local terminal semantics, the particular
RegExp facade, graph hardening and FFI/capture mechanisms, and their coupled
upgrade audits. A switch would retain the contract obligations but might remove
some mechanisms; the existing corpus is reusable for comparing that claim.

The favorable prospective case is a common interpreter path whose known async
changes are enumerable, whose exercised public-API boundaries are explicit, and
whose policy surfaces have concrete adversarial oracles. This supports one bounded
maintenance decision. It does **not** establish low total cost, superior security,
small device footprint or domination of alternatives. The old alternative-engine
assessment supplies context only; no fresh head-to-head comparison is claimed.

## Conditions and selection-readiness disposition

**Continue conditionally, not by default.** The total burden is significant but
still organized around a restricted profile and identifiable boundaries. There
is no evidence requiring a compiler/GC redesign or portable-contract relaxation.
That favors resolving whether the project can own this bounded architecture
before discarding it. Conversely, the missing full patch audit, unaccepted
maintenance responsibility and unresolved platform/parser gates rule out an
unqualified Continue or a selection ADR.

Continuation requires an auditable complete patch set, sustainable ownership and
an update/upstream strategy. Remaining parser, module, resource-composition and
platform gates still must pass before selection. Permanent host obligations are
not promised to disappear after a successful gate. Engine diffs may eventually
retire upstream; their regression obligations remain.

Reconsider the candidate architecture if the complete patch set needs broad
compiler/GC/language changes, new invasive APIs without a bounded audit, repeated
standard-library algorithm emulation beyond the inventoried facade, unsafe
resource recovery, an unserviceable security pin, or a platform/profile gate
cannot pass without weakening the contract. Lack of a capable willing maintainer
is itself a failed viability condition. Unknowns must not be converted into
“small future work” to keep QuickJS the default.

| Pre-selection gate | Current disposition |
| --- | --- |
| Complete patch-set maintenance acceptance | **Only next gate.** Actual terminal-OOM artifact/provenance and combined scope are not auditable at this HEAD; ownership/security/update acceptance is pending. |
| Integrated resource/failure closure | Terminal-OOM feasibility is accepted as the task premise; checked-in public-API configuration remains negative. Complete candidate composition/remaining native-work and host-allocation coverage are not declared closed here. |
| Complete module/path containment | Open; capture mechanism alone is insufficient. |
| Final parser/preflight strategy | Open; no parser selected, full ES2023/early-error/resource agreement unproved. |
| Cross-platform / Apple embedding | Open; no charter-wide/device claim. |

## Only next task

Conduct one **QuickJS complete patch-set maintenance acceptance review**, the
existing patch ownership/update/integration selection gate brought forward. Its
question is: **can this project sustainably own the complete terminal-failure
patch set and its coupled host/binding upgrade obligations?**

Review the already-proved terminal-OOM artifact alongside the existing async
patch, rather than implementing or expanding either. Required outputs are:

1. Durable source provenance and exact combined diff/site/ownership inventory,
   including the terminal-OOM patch's actual size, intrusion and failure-mode
   audit; connect existing execution evidence to that precise configuration.
2. Explicit acceptance by an accountable maintainer/reviewer, an upstream plan
   with a non-acceptance fallback, and a concrete security/rebase/update procedure
   covering the trigger matrix above. No imaginary staffing commitment.
3. A reviewable single-engine build/provenance and regression qualification plan,
   including the known integrated sanitizer gap and later target qualification;
   distinguish plans from completed proofs. No production adoption in this gate.
4. A pass/fail maintenance decision: pass allows separately scoped remaining
   selection work; missing artifact, ownership or a bounded update path keeps the
   condition unsatisfied and requires candidate architecture reconsideration.

Do not turn missing documentation into a new OOM spike, begin another gate, or
choose its implementation here. If this maintenance gate fails, the appropriate
reassessment level is an **end-to-end candidate architecture cost comparison**
under the same RFC (termination, native work, parser, binding, platforms and
updates), not an isolated engine-speed contest. That is a failure disposition,
not a second task or authorization for alternative-engine/IPC experiments.

## Validation and preserved scope

This is a documentation-only candidate decision, not a new experimental closure.
Recorded test counts and timings above are inherited evidence, not rerun results.
The [completed plan](../plans/completed/2026-09-30-quickjs-candidate-viability.md)
records scoped Markdown/link/anchor, whitespace, complete-diff and status checks.
No runtime, patch, parser, lockfile, historical review, specification, ADR or
conformance fixture changes. No full Phase 3 rebuild, fresh upstream/security
research or platform test is claimed. No push, tag, release or settings change.
