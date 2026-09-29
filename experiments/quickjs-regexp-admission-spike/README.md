# QuickJS RegExp admission route probes

Evidence for the [RegExp compilation remedy review](../../docs/reviews/2026-09-29-quickjs-regexp-admission-review.md). It maps where the pinned engine compiles RegExp patterns, which of those routes a source-visible replacement can intercept, and whether pattern text exists at a measurable point before the non-polling compiler. It is **not** an admission implementation, facade, parser selection, policy value or RFC change. The preserved blocker lives unchanged in [quickjs-resource-spike](../quickjs-resource-spike/README.md).

Baseline: `8f6cafa67b7e53d95d54d60200acac852fd78ac1`. Phase 3 remains active; RFC 0001 remains proposed.

## Pins and boundaries

- rquickjs/core/sys `0.14.0` (`d7ef5eeae702fea24c03643064de454f1c1dd4b0`); QuickJS-NG `0.16.2` (`0fdea21ff1090084e91dad812b343d92e79ba9d9`). Defaults disabled; `std`/`loader` only.
- Boa parser/AST/interner `0.22.0` (lexer validates literal bodies with `regress` `0.12.0`) and Oxc `0.152.0`, exactly as in the static-analysis spike. The lockfile equals that spike's lockfile after renaming only the root package. No new dependency, no Boa engine.
- Each realm: fresh Runtime/Context, 8 MiB heap limit, 128 KiB stack setting, a host-only counting interrupt handler that requests interruption on every armed callback. Route probes use a bare full-intrinsic realm, so they observe pristine native paths.
- `forbid(unsafe_code)`: zero unsafe blocks, functions or impls. No private engine API, patch, network, credentials or source rewriting. `PROBE_LIMIT` (4,096 UTF-16 units) is a demonstration threshold, not a proposed host value.

## Run

The QuickJS inputs here compile in about 0.1 s at most; the 320,007-unit reproducer is only measured, never handed to QuickJS. The established guard is still used:

```sh
cargo test --manifest-path experiments/quickjs-regexp-admission-spike/Cargo.toml --locked --no-run
python3 - <<'PY'
import os, signal, subprocess
command = ['cargo', 'test', '--manifest-path',
           'experiments/quickjs-regexp-admission-spike/Cargo.toml', '--locked',
           '--', '--test-threads=1', '--nocapture']
process = subprocess.Popen(command, start_new_session=True)
try:
    result = process.wait(timeout=60)
except subprocess.TimeoutExpired:
    os.killpg(process.pid, signal.SIGKILL)
    process.wait()
    raise SystemExit('FAIL: external 60-second timeout')
raise SystemExit(result)
PY
cargo fmt --manifest-path experiments/quickjs-regexp-admission-spike/Cargo.toml --check
cargo clippy --manifest-path experiments/quickjs-regexp-admission-spike/Cargo.toml --locked --all-targets -- -D warnings
```

## Eight tests

| Test | Observation |
| --- | --- |
| `literal_compiles_at_declaration_on_the_non_polling_path` | Control: armed `/^(a+)+$/` matching is interrupted. A 20,007-unit forward-reference **literal** compiles during compile-only `Module::declare` (~0.1 s) with **zero** callbacks. An invalid tail pays the same cost, then fails. |
| `every_literal_position_is_compiled_at_declaration_and_seen_by_boa` | Eleven positions: entry/helper top level, unreachable `if (false)`, nested function/arrow, class method and static field, template substitution, statement after `if`, default parameter, generator, async function, object/array. `/(/` fails QuickJS declaration in every position; Boa's visitor finds every `/ok+/g`; Oxc validation agrees; Oxc raw mode does not validate patterns. |
| `division_regexp_ambiguity_escapes_and_flags_match_quickjs` | 14 lexical fixtures (division chains, `/=`, `({}) / 2`, block-then-literal, `if (x) /re/`, escaped slash, slash in class, Unicode escapes, all ES2023 flags, non-ASCII, comments/templates containing slashes). QuickJS accepts all; Boa's literal list equals the expected list; bodies are verbatim in unchanged bytes. |
| `oversized_literal_is_measured_and_rejected_before_quickjs` | 80,007 and 320,007-unit literals are enumerated with exact UTF-16 length and rejected by the probe threshold without QuickJS. Boa takes 44 / ~550 ms (superlinear on this family, debug build; cause not diagnosed); Oxc raw 0–2 ms; Oxc validated 12 / ~50 ms. |
| `replaced_global_regexp_is_bypassed_by_builtin_compile_routes` | Negative control: global `RegExp` and `RegExp.prototype.constructor` replaced by a refusing function. 20 routes classified; see below. |
| `source_coercion_completes_before_native_compilation` | Order `IsRegExp → ToPrimitive(pattern, string) → ToString(flags) → SyntaxError`. Pinned QuickJS reads `newTarget.prototype` only after compile and implements `RegExpCreate` via its full constructor; both differ from the ES2023 algorithm order. |
| `native_global_match_loops_reach_the_interrupt_hook` | Global `replace`/`split`/`match`/`matchAll` over 200,000 short matches are interrupted at the first armed callback. Observation, not a matching interval bound. |
| `post_coercion_length_check_stops_reproducer_before_compile` | Ordering witness: the exact 320,007-unit reproducer text is an ordinary coerced string before any compiler entry; a length check rejects it in under 1 ms with zero callbacks. |

Route classes with the invalid pattern `"("`:

| Class | Routes |
| --- | --- |
| Reaches replaced global (`facade`) | `new RegExp`, `RegExp()`, `Reflect.construct`, bound constructor, `class extends RegExp`, `(/x/).constructor`, species that resolves to the replacement. |
| Native compile, bypassing the replacement | `String.prototype.match`/`matchAll`/`search` with a non-RegExp argument; `RegExp.prototype.compile`; `RegExp.prototype[Symbol.split]`/`[Symbol.matchAll]` on a generic receiver whose species defaults to the internal constructor. |
| No compilation of source text | String `split`/`replace`/`replaceAll`; `exec`; `Symbol.replace`; genuine receiver defaulting to the internal constructor (reuses its internal source even when `source` is overridden). |

Timings are local debug-profile observations (macOS 27.0 arm64, Rust 1.96.0), not portable constants.
