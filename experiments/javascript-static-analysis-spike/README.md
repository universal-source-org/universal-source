# JavaScript static-analysis compatibility spike

**Non-production evidence.** The original Boa-only unit concluded **Outcome B — static preflight blocker**. Boa's public module AST loses the distinction between an absent import-attribute clause and `with {}`. Both Boa and pinned QuickJS accept those sources, but RFC 0001 forbids attributes. The passing negative controls reproduce this gap; they do not approve those sources. See the [original review](../../docs/reviews/2026-09-28-javascript-static-analysis-spike.md). The subsequent [attribute-clause review](../../docs/reviews/2026-09-28-javascript-import-attributes-spike.md) records **Outcome A — attribute-clause preflight viable** using supplemental Oxc 0.152.0 clause-preserving AST data. The original negative controls remain unchanged; the hybrid closes their false-acceptance path.

This independent unpublished crate tests Boa Parser/AST/Interner **0.22.0**, revision `337a3668a0dc86dd401ea20906e782249a64a228`, against unchanged rquickjs/core/sys **0.14.0**, revision `d7ef5eeae702fea24c03643064de454f1c1dd4b0`, with QuickJS-NG **0.16.2**, revision `0fdea21ff1090084e91dad812b343d92e79ba9d9`. It does not depend on `boa_engine`. Supplemental Oxc parser/AST/allocator/span are pinned to **0.152.0**, revision `dfbc0d1ea752f021ba4a68e4703b3a9031082a46`. Parser use is independent of engine choice; none is selected for production.

## What runs

- [analysis.rs](src/analysis.rs): deliberately incomplete public-AST inspection, semantic names/specifiers, recursive syntax visitor, diagnostic locations and observable traversal bounds. No regex validator or source rewriting.
- [corpus.rs](src/corpus.rs): 62 fixed sources with separate syntax-acceptance and RFC-violation expectations, including two explicitly marked attribute-presence gaps.
- [tests.rs](src/tests.rs): the original 15 tests, exact-byte/hash comparison, parser/QuickJS matrix, structural export classification, escaped names, Unicode distinctions, nested syntax, TLA, diagnostics and negative controls.
- [attributes.rs](src/attributes.rs) and [attribute_tests.rs](src/attribute_tests.rs): supplemental clause-presence check and nine tests, including 24 import/re-export matrix rows, 40 trivia variants, 15 lexical/ASI controls, source spans, parser disagreement and prior-corpus regression. Both parsers read identical bytes; the check rejects parser errors or ordered graph-edge summary disagreement. It is not a full multi-parser validation policy.

Immutable UTF-8 fixture bytes go into `Source::from_bytes` and unchanged into `Module::declare`. The matrix deliberately compiles rejected fixtures independently to compare syntax acceptance; that is diagnostic evidence, not a production pipeline that executes rejected sources. The fixed helper loader has no filesystem/network path. It ignores import attributes only for the compiler-acceptance probe, not as RFC permission. Compilation causes no pending jobs. Only the decoded-export-name witness evaluates declaration-only modules to read actual namespace names; no operation is invoked. The previous native pre-evaluation capture strategy is untouched.

## Run

From the repository root, prebuild and run under a 60-second external process-group timeout (Python standard library; a timeout kills Cargo and its child processes):

```sh
cargo test --manifest-path experiments/javascript-static-analysis-spike/Cargo.toml --locked --no-run
python3 - <<'PY'
import os, signal, subprocess
command = ['cargo', 'test', '--manifest-path',
           'experiments/javascript-static-analysis-spike/Cargo.toml',
           '--locked', '--', '--test-threads=1', '--nocapture']
p = subprocess.Popen(command, start_new_session=True)
try:
    raise SystemExit(p.wait(timeout=60))
except subprocess.TimeoutExpired:
    os.killpg(p.pid, signal.SIGKILL)
    p.wait()
    raise SystemExit('FAIL: external 60-second timeout')
PY
cargo fmt --manifest-path experiments/javascript-static-analysis-spike/Cargo.toml --check
cargo clippy --manifest-path experiments/javascript-static-analysis-spike/Cargo.toml --locked --all-targets -- -D warnings
```

Observed: **24 tests pass** (15 original + 9 supplemental). The original matrix remains **58 both accept / 1 Boa-only / 0 QuickJS-only / 3 both reject**. The Boa-only `import.source()` case is outside the RFC and caught as dynamic import. Empty attribute clauses remain invisible in the Boa AST but are now rejected using Oxc clause presence. Full current validation is in the [completed plan](../../docs/plans/completed/2026-09-28-javascript-import-attributes-spike.md).

Local evidence: Rust/Cargo 1.96.0, macOS 27.0 (26A428), arm64. The manifest pins direct dependencies; the lockfile records 103 external packages: 32 added for Oxc, with none of the prior 71 package/version pairs removed or upgraded. Building rquickjs still compiles vendored QuickJS C code and uses its existing native build requirements. No production dependency changed, no extra platform support is claimed.

## Limits

This is not a complete RFC validator or ES2023 edition gate. No graph/path containment, sandbox, dynamic-constructor lockdown, service, production capture integration or conformance implementation is included. Runtime `eval`/`Function` denial is not replaced by a static syntax ban. Source-size admission and post-parse expression counts do not prove parser stack/heap/time bounds. Fixture string conversion intentionally fails on unpaired UTF-16 surrogates; a production specifier/diagnostic path would need explicit rejection/encoding behavior.

Oxc preserves clause **presence** via `Option<WithClause>` and parent declaration ownership, not substring matching. Its clause span covers the braces and contents, excluding the keyword. The next engine-feasibility unit is only a focused dynamic JavaScript compilation suppression proof under RFC §9. Do not select a parser/engine or begin production execution.
