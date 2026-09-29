#!/usr/bin/env python3
"""
Throwaway engine-fix feasibility driver (Universal Source, Phase 3).

Answers question 1 of the task: is the Promise/async uncatchable-stop SWALLOWING
defect COMPLETELY fixable by a bounded engine change that reuses the engine's own
`JS_IsUncatchableError` guard (the pattern `promise_reaction_job` already carries)?

It NEVER touches the committed/pinned dependency. It:
  1. Locates the pinned QuickJS-NG source and SHA-verifies quickjs.c.
  2. Copies it into build/baseline/qjs and build/patched/qjs (both gitignored).
  3. Applies per-site guard edits ONLY to the patched copy, asserting exact counts.
  4. Confirms the baseline copy is byte-identical to the pinned source.
  5. Builds harness.c against each engine with ASan+UBSan and QuickJS leak-abort.
  6. Runs both and diffs: baseline SWALLOWS the swallow sites; patched TERMINATES
     every uncatchable case; ordinary cases stay CAUGHT in both; guarded sites
     stay TERMINATED in both; a fresh runtime is healthy.

No outer watchdog decides success: termination is proven by the engine returning
control to the C host with the uncatchable latched and no source continuation.
A generous outer timeout exists ONLY to fail a genuine hang; a timeout is a FAIL.
"""
import hashlib
import os
import re
import shutil
import subprocess
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
BUILD = HERE / "build"

PINNED_QUICKJS_C_SHA256 = (
    "3a6b52a225c21709ef1fd314183efd3af862e867ce4ef8caabcae67922a745b3"
)

# The exact revision this experiment is calibrated against.
PINNED_REVISION = "0fdea21ff1090084e91dad812b343d92e79ba9d9"  # QuickJS-NG 0.16.2
RQUICKJS_SYS = "rquickjs-sys-0.14.0"

SOURCES = ["quickjs.c", "libregexp.c", "libunicode.c", "dtoa.c"]

# ---------------------------------------------------------------------------
# Per-site guard edits. Each reuses the engine's existing
# `if (JS_IsUncatchableError(ctx->rt->current_exception))` check and routes the
# latched uncatchable back to the host through that site's EXISTING cleanup
# (a shared fail/done label where one exists, else the site's own owned values),
# WITHOUT calling JS_GetException first (so the pending uncatchable is preserved).
# ---------------------------------------------------------------------------
PATCHES = [
    # 1. js_promise_constructor: executor throw -> propagate via `fail:`
    dict(
        name="js_promise_constructor (executor)",
        count=1,
        old=(
            "        JSValue ret2, error;\n"
            "        error = JS_GetException(ctx);\n"
            "        ret2 = JS_Call(ctx, args[1], JS_UNDEFINED, 1, vc(&error));\n"
        ),
        new=(
            "        JSValue ret2, error;\n"
            "        if (unlikely(JS_IsUncatchableError(ctx->rt->current_exception)))\n"
            "            goto fail;\n"
            "        error = JS_GetException(ctx);\n"
            "        ret2 = JS_Call(ctx, args[1], JS_UNDEFINED, 1, vc(&error));\n"
        ),
    ),
    # 2. js_promise_resolve_function_call: thenable `then` getter throw
    dict(
        name="js_promise_resolve_function_call (fail_reject)",
        count=1,
        old=(
            "    fail_reject:\n"
            "        error = JS_GetException(ctx);\n"
            "        fulfill_or_reject_promise(ctx, s->promise, error, true);\n"
        ),
        new=(
            "    fail_reject:\n"
            "        if (unlikely(JS_IsUncatchableError(ctx->rt->current_exception)))\n"
            "            return JS_EXCEPTION;\n"
            "        error = JS_GetException(ctx);\n"
            "        fulfill_or_reject_promise(ctx, s->promise, error, true);\n"
        ),
    ),
    # 3. js_promise_resolve_thenable_job: thenable `then` call throw.
    #    Skip only the reject conversion; the existing tail already frees
    #    args[0]/args[1] and returns res (== JS_EXCEPTION) unchanged.
    dict(
        name="js_promise_resolve_thenable_job",
        count=1,
        old=(
            "    if (JS_IsException(res)) {\n"
            "        JSValue error = JS_GetException(ctx);\n"
            "        res = JS_Call(ctx, args[1], JS_UNDEFINED, 1, vc(&error));\n"
            "        JS_FreeValue(ctx, error);\n"
            "    }\n"
        ),
        new=(
            "    if (JS_IsException(res)) {\n"
            "        if (!JS_IsUncatchableError(ctx->rt->current_exception)) {\n"
            "            JSValue error = JS_GetException(ctx);\n"
            "            res = JS_Call(ctx, args[1], JS_UNDEFINED, 1, vc(&error));\n"
            "            JS_FreeValue(ctx, error);\n"
            "        }\n"
            "    }\n"
        ),
    ),
    # 4 + 5. js_promise_all AND js_promise_race share this identical fail_reject
    #        block -> propagate via each function's own `fail:` label.
    dict(
        name="js_promise_all / js_promise_race (fail_reject)",
        count=2,
        old=(
            "    fail_reject:\n"
            "        error = JS_GetException(ctx);\n"
            "        ret = JS_Call(ctx, resolving_funcs[1], JS_UNDEFINED, 1, vc(&error));\n"
        ),
        new=(
            "    fail_reject:\n"
            "        if (unlikely(JS_IsUncatchableError(ctx->rt->current_exception)))\n"
            "            goto fail;\n"
            "        error = JS_GetException(ctx);\n"
            "        ret = JS_Call(ctx, resolving_funcs[1], JS_UNDEFINED, 1, vc(&error));\n"
        ),
    ),
    # 6. js_promise_try: callback throw -> free owned funcs + result_promise.
    dict(
        name="js_promise_try",
        count=1,
        old=(
            "    if (JS_IsException(ret)) {\n"
            "        is_reject = 1;\n"
            "        ret = JS_GetException(ctx);\n"
            "    }\n"
        ),
        new=(
            "    if (JS_IsException(ret)) {\n"
            "        if (unlikely(JS_IsUncatchableError(ctx->rt->current_exception))) {\n"
            "            JS_FreeValue(ctx, resolving_funcs[0]);\n"
            "            JS_FreeValue(ctx, resolving_funcs[1]);\n"
            "            JS_FreeValue(ctx, result_promise);\n"
            "            return JS_EXCEPTION;\n"
            "        }\n"
            "        is_reject = 1;\n"
            "        ret = JS_GetException(ctx);\n"
            "    }\n"
        ),
    ),
    # 7. js_async_generator_resume_next: body resume throw (void fn -> return).
    dict(
        name="js_async_generator_resume_next",
        count=1,
        old=(
            "            if (JS_IsException(func_ret)) {\n"
            "                value = JS_GetException(ctx);\n"
            "                js_async_generator_complete(ctx, s);\n"
            "                js_async_generator_reject(ctx, s, value);\n"
            "                JS_FreeValue(ctx, value);\n"
            "            } else if (JS_VALUE_GET_TAG(func_ret) == JS_TAG_INT) {\n"
        ),
        new=(
            "            if (JS_IsException(func_ret)) {\n"
            "                if (unlikely(JS_IsUncatchableError(ctx->rt->current_exception)))\n"
            "                    return;\n"
            "                value = JS_GetException(ctx);\n"
            "                js_async_generator_complete(ctx, s);\n"
            "                js_async_generator_reject(ctx, s, value);\n"
            "                JS_FreeValue(ctx, value);\n"
            "            } else if (JS_VALUE_GET_TAG(func_ret) == JS_TAG_INT) {\n"
        ),
    ),
    # 7b. js_async_generator_next: resume_next is a VOID helper; on a latched
    #     uncatchable it now returns with the exception pending, so its caller
    #     must propagate it (the one-line site-7 guard alone is NOT sufficient
    #     here -- this is why the async-generator route needs caller cooperation).
    dict(
        name="js_async_generator_next (propagate resume_next)",
        count=1,
        old=(
            "    if (s->state != JS_ASYNC_GENERATOR_STATE_EXECUTING) {\n"
            "        js_async_generator_resume_next(ctx, s);\n"
            "    }\n"
            "    return promise;\n"
        ),
        new=(
            "    if (s->state != JS_ASYNC_GENERATOR_STATE_EXECUTING) {\n"
            "        js_async_generator_resume_next(ctx, s);\n"
            "        if (unlikely(JS_IsUncatchableError(ctx->rt->current_exception))) {\n"
            "            JS_FreeValue(ctx, promise);\n"
            "            return JS_EXCEPTION;\n"
            "        }\n"
            "    }\n"
            "    return promise;\n"
        ),
    ),
    # 7c. js_async_generator_resolve_function: the await-resolution caller of the
    #     void resume_next; propagate so promise_reaction_job's existing guard
    #     (quickjs.c:55584) carries the uncatchable back to the host.
    dict(
        name="js_async_generator_resolve_function (propagate resume_next)",
        count=1,
        old=(
            "        js_async_generator_resume_next(ctx, s);\n"
            "    }\n"
            "    return JS_UNDEFINED;\n"
        ),
        new=(
            "        js_async_generator_resume_next(ctx, s);\n"
            "        if (unlikely(JS_IsUncatchableError(ctx->rt->current_exception)))\n"
            "            return JS_EXCEPTION;\n"
            "    }\n"
            "    return JS_UNDEFINED;\n"
        ),
    ),
    # 8. js_async_generator_completed_return: mirror the existing `return -1`
    #    error path exactly (promise == JS_EXCEPTION, value not owned here).
    dict(
        name="js_async_generator_completed_return",
        count=1,
        old=(
            "    if (JS_IsException(promise)) {\n"
            "        JSValue err = JS_GetException(ctx);\n"
            "        promise = js_promise_resolve(ctx, ctx->promise_ctor, 1, vc(&err),\n"
            "                                     /*is_reject*/1);\n"
        ),
        new=(
            "    if (JS_IsException(promise)) {\n"
            "        JSValue err;\n"
            "        if (unlikely(JS_IsUncatchableError(ctx->rt->current_exception)))\n"
            "            return -1;\n"
            "        err = JS_GetException(ctx);\n"
            "        promise = js_promise_resolve(ctx, ctx->promise_ctor, 1, vc(&err),\n"
            "                                     /*is_reject*/1);\n"
        ),
    ),
    # 9a. js_async_from_sync_iterator_next: reject: label (next/getter throw).
    dict(
        name="js_async_from_sync_iterator_next (reject)",
        count=1,
        old=(
            "    reject:\n"
            "        err = JS_GetException(ctx);\n"
            "        is_reject = 1;\n"
            "    done_resolve:\n"
        ),
        new=(
            "    reject:\n"
            "        if (unlikely(JS_IsUncatchableError(ctx->rt->current_exception))) {\n"
            "            JS_FreeValue(ctx, resolving_funcs[0]);\n"
            "            JS_FreeValue(ctx, resolving_funcs[1]);\n"
            "            JS_FreeValue(ctx, promise);\n"
            "            return JS_EXCEPTION;\n"
            "        }\n"
            "        err = JS_GetException(ctx);\n"
            "        is_reject = 1;\n"
            "    done_resolve:\n"
        ),
    ),
    # 9b. js_async_from_sync_iterator_next: sync IteratorClose during missing
    #     `throw` method (method here is undefined/null; no method free needed).
    dict(
        name="js_async_from_sync_iterator_next (IteratorClose)",
        count=1,
        old=(
            "            } else if (JS_IteratorClose(ctx, s->sync_iter, false)) {\n"
            "                err = JS_GetException(ctx);\n"
            "                is_reject = 1;\n"
        ),
        new=(
            "            } else if (JS_IteratorClose(ctx, s->sync_iter, false)) {\n"
            "                if (unlikely(JS_IsUncatchableError(ctx->rt->current_exception))) {\n"
            "                    JS_FreeValue(ctx, resolving_funcs[0]);\n"
            "                    JS_FreeValue(ctx, resolving_funcs[1]);\n"
            "                    JS_FreeValue(ctx, promise);\n"
            "                    return JS_EXCEPTION;\n"
            "                }\n"
            "                err = JS_GetException(ctx);\n"
            "                is_reject = 1;\n"
        ),
    ),
]

# Defect-reproduction set: these MUST swallow in the pristine baseline (proving
# the reported defect reproduces) and MUST terminate after the patch. If any of
# these does not swallow in baseline, the harness does not reproduce the defect.
MUST_FLIP = {
    "executor__uncatchable",
    "resolve_thenable_getter__uncatchable",
    "thenable_then_call__uncatchable",
    "promise_all_iter__uncatchable",
    "promise_race_iter__uncatchable",
    "promise_try__uncatchable",
    "async_gen_resume__uncatchable",
    "async_gen_await_resume__uncatchable",
    "finally_no_success__uncatchable",
    "regexp_admission_gate__uncatchable",
}
# Shapes already safe in the baseline (they route through the two pre-existing
# guards: promise_reaction_job / js_async_function_resume). They must stay
# TERMINATED after the patch (no regression).
ALREADY_SAFE = {
    "await_thenable_getter__uncatchable",
    "for_await_sync_iter__uncatchable",
    "interrupt_executor__uncatchable",
    "reaction_job_guard__uncatchable",
    "async_func_guard__uncatchable",
}


def find_pinned_src() -> Path:
    override = os.environ.get("QUICKJS_SRC")
    if override:
        p = Path(override)
        if (p / "quickjs.c").is_file():
            return p
    home = Path.home()
    roots = list((home / ".cargo/registry/src").glob(f"*/{RQUICKJS_SYS}/quickjs"))
    for r in roots:
        if (r / "quickjs.c").is_file():
            return r
    sys.exit("Could not locate the pinned QuickJS-NG source (set QUICKJS_SRC).")


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def prepare_tree(src: Path, dest: Path) -> None:
    qjs = dest / "qjs"
    if qjs.exists():
        shutil.rmtree(qjs)
    qjs.mkdir(parents=True)
    for f in src.iterdir():
        if f.suffix in (".c", ".h"):
            shutil.copy2(f, qjs / f.name)


def apply_patches(qjs: Path) -> None:
    path = qjs / "quickjs.c"
    text = path.read_text()
    for p in PATCHES:
        found = text.count(p["old"])
        if found != p["count"]:
            sys.exit(
                f"PATCH ANCHOR MISMATCH for {p['name']!r}: "
                f"expected {p['count']} occurrence(s), found {found}. "
                f"The pinned source may differ from the calibrated revision."
            )
        text = text.replace(p["old"], p["new"])
    path.write_text(text)


def build(qjs: Path, out: Path) -> None:
    cc = os.environ.get("CC", "cc")
    cmd = [
        cc,
        "-D_GNU_SOURCE",
        "-DENABLE_DUMPS",
        "-fsanitize=address,undefined",
        "-fno-omit-frame-pointer",
        "-fno-sanitize-recover=all",
        "-g",
        "-O1",
        f"-I{qjs}",
        str(HERE / "harness.c"),
    ] + [str(qjs / s) for s in SOURCES] + ["-lm", "-o", str(out)]
    subprocess.run(cmd, check=True)


def run(binary: Path) -> dict:
    env = dict(os.environ)
    # Deterministic ASan/UBSan aborts; QuickJS leak-abort handled in-harness.
    env["ASAN_OPTIONS"] = "abort_on_error=1:detect_leaks=0"
    env["UBSAN_OPTIONS"] = "print_stacktrace=1:halt_on_error=1"
    try:
        proc = subprocess.run(
            [str(binary)], capture_output=True, text=True, timeout=120, env=env,
            start_new_session=True,
        )
    except subprocess.TimeoutExpired:
        sys.exit("HANG: engine did not return control in 120 s (outer timeout = FAIL).")
    out = proc.stdout
    print(out, end="")
    if proc.stderr:
        print(proc.stderr, end="", file=sys.stderr)
    if proc.returncode != 0:
        sys.exit(
            f"engine harness {binary.name} exited {proc.returncode} "
            f"(sanitizer or QuickJS leak-abort; inspect output above)."
        )
    verdicts, freed = {}, set()
    for line in out.splitlines():
        m = re.match(r"CASE (\S+) => (\S+)", line)
        if m:
            verdicts[m.group(1)] = m.group(2)
        m = re.match(r"FREED (\S+)", line)
        if m:
            freed.add(m.group(1))
        if line == "HEALTH ok":
            verdicts["__health__"] = "ok"
    # Every case that printed a verdict must also have printed FREED (no leak-abort).
    for name in list(verdicts):
        if name.startswith("__"):
            continue
        if name not in freed:
            sys.exit(f"LEAK: case {name!r} aborted during JS_FreeRuntime (leak check).")
    return verdicts


def main() -> int:
    src = find_pinned_src()
    got = sha256(src / "quickjs.c")
    if got != PINNED_QUICKJS_C_SHA256:
        sys.exit(
            f"Pinned quickjs.c SHA mismatch.\n  expected {PINNED_QUICKJS_C_SHA256}\n"
            f"  got      {got}\nThis experiment is calibrated to revision "
            f"{PINNED_REVISION}."
        )
    print(f"pinned source: {src}")
    print(f"quickjs.c sha256 verified: {got}")

    BUILD.mkdir(exist_ok=True)
    baseline_qjs = BUILD / "baseline" / "qjs"
    patched_qjs = BUILD / "patched" / "qjs"

    prepare_tree(src, BUILD / "baseline")
    prepare_tree(src, BUILD / "patched")

    # Baseline copy MUST equal the pinned source (no accidental edits).
    if sha256(baseline_qjs / "quickjs.c") != PINNED_QUICKJS_C_SHA256:
        sys.exit("Baseline copy diverged from pinned source.")

    apply_patches(patched_qjs)
    total = sum(p["count"] for p in PATCHES)
    print(f"applied {len(PATCHES)} guard rules ({total} replacements) to patched copy")

    baseline_bin = BUILD / "baseline" / "engine_test"
    patched_bin = BUILD / "patched" / "engine_test"
    print("building baseline engine...")
    build(baseline_qjs, baseline_bin)
    print("building patched engine...")
    build(patched_qjs, patched_bin)

    print("\n--- baseline run ---")
    base = run(baseline_bin)
    print("\n--- patched run ---")
    patch = run(patched_bin)

    print("\n=== verdict diff ===")
    failures = []
    all_names = sorted(set(base) | set(patch))
    for name in all_names:
        if name.startswith("__"):
            continue
        b, p = base.get(name, "MISSING"), patch.get(name, "MISSING")
        note = ""
        if name.endswith("__ordinary"):
            if not (b == "CAUGHT" and p == "CAUGHT"):
                failures.append(f"{name}: expected CAUGHT in both, got {b}->{p}")
                note = "  <-- FAIL"
        else:
            # Universal hard requirement: the patched engine terminates every
            # uncatchable/guard route (control returns to host, no continuation).
            if p != "TERMINATED":
                failures.append(f"{name}: patched must TERMINATE, got {p}")
                note = "  <-- FAIL"
            if name in MUST_FLIP and b != "SWALLOWED":
                failures.append(f"{name}: baseline must reproduce SWALLOWED, got {b}")
                note = "  <-- FAIL"
            elif name in ALREADY_SAFE and b != "TERMINATED":
                failures.append(f"{name}: baseline expected TERMINATED, got {b}")
                note = "  <-- FAIL"
            elif name.startswith("deadline_"):
                note = f"  (baseline {b}; deadline mechanism)"
        print(f"  {name:44s} {b:11s} -> {p:11s}{note}")

    for label, v in (("baseline", base), ("patched", patch)):
        if v.get("__health__") != "ok":
            failures.append(f"{label}: fresh-runtime HEALTH check failed")

    print()
    if failures:
        print("RESULT: FAIL")
        for f in failures:
            print(f"  - {f}")
        return 1
    print("RESULT: PASS")
    print("  - patched engine terminates every uncatchable route")
    print("  - ordinary exceptions remain catchable/rejecting")
    print("  - already-correct guard sites unregressed")
    print("  - fresh runtime healthy; no sanitizer or leak abort")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
