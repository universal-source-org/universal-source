#!/usr/bin/env python3
"""Build one experiment-local sys crate; reuse only the previously audited patch.

No registry or prior experiment is modified. C sources (not Rust std/dependencies)
are instrumented in --sanitize mode. Execution watchdogs are failures, not proof.
"""
import argparse
import importlib.util
import json
import os
import re
from pathlib import Path
import shutil
import signal
import subprocess

HERE = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location(
    "async_patch", HERE.parent / "quickjs-async-termination-fix-spike/build_and_run.py"
)
patch = importlib.util.module_from_spec(spec)
spec.loader.exec_module(patch)


def run(command, env, timeout=None):
    print('+', ' '.join(map(str, command)), flush=True)
    p = subprocess.Popen(command, cwd=HERE, env=env, start_new_session=True)
    try:
        result = p.wait(timeout=timeout)
    except subprocess.TimeoutExpired:
        os.killpg(p.pid, signal.SIGKILL)
        p.wait()
        raise SystemExit('FAIL: external execution watchdog')
    if result:
        raise SystemExit(result)


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument('--baseline', action='store_true')
    parser.add_argument('--sanitize', action='store_true')
    parser.add_argument('--prepare', action='store_true')
    args = parser.parse_args()
    source = patch.find_pinned_src().parent
    assert patch.sha256(source / 'quickjs/quickjs.c') == patch.PINNED_QUICKJS_C_SHA256
    dest = HERE / 'build/sys'
    if dest.exists():
        shutil.rmtree(dest)
    shutil.copytree(source, dest)
    if not args.baseline:
        patch.apply_patches(dest / 'quickjs')
    changed = []
    for original in source.rglob('*'):
        if original.is_file():
            relative = original.relative_to(source)
            if original.read_bytes() != (dest / relative).read_bytes():
                changed.append(str(relative))
    assert changed == ([] if args.baseline else ['quickjs/quickjs.c']), changed
    digest = patch.sha256(dest / 'quickjs/quickjs.c')
    env = dict(os.environ)
    mode = 'baseline' if args.baseline else 'patched'
    target = HERE / 'target' / (mode + ('-sanitized' if args.sanitize else ''))
    env.update(CARGO_TARGET_DIR=str(target), INTEGRATED_ENGINE_MODE=mode,
               INTEGRATED_ENGINE_SHA=digest, CFLAGS='-UNDEBUG')
    if args.sanitize:
        env['CFLAGS'] += ' -fsanitize=address,undefined -fno-sanitize-recover=all -fno-omit-frame-pointer'
        env['RUSTFLAGS'] = '-C linker=clang -C link-arg=-fsanitize=address,undefined'
        env['ASAN_OPTIONS'] = 'detect_leaks=0:abort_on_error=1'
        env['UBSAN_OPTIONS'] = 'halt_on_error=1:print_stacktrace=1'
    metadata = json.loads(subprocess.check_output(
        ['cargo', 'metadata', '--offline', '--locked', '--format-version=1'], cwd=HERE, env=env))
    sys_crates = [p for p in metadata['packages'] if p['name'] == 'rquickjs-sys']
    assert len(sys_crates) == 1
    assert Path(sys_crates[0]['manifest_path']).resolve() == dest / 'Cargo.toml'
    print('PROVENANCE', mode, digest, 'one rquickjs-sys 0.14.0 at', dest, flush=True)
    if args.prepare:
        return
    build = subprocess.run(['cargo', 'test', '--offline', '--locked', '--no-run',
                            '--message-format=json'], cwd=HERE, env=env,
                           stdout=subprocess.PIPE, text=True, check=True)
    (HERE / 'build' / (mode + '-cargo-build.jsonl')).write_text(build.stdout)
    emitted = [json.loads(line) for line in build.stdout.splitlines() if line.startswith('{')]
    binaries = [Path(row['executable']) for row in emitted
                if row.get('reason') == 'compiler-artifact' and row.get('executable')]
    assert len(binaries) == 1, binaries
    symbols = subprocess.check_output(['nm', str(binaries[0])], text=True)
    for symbol in ('JS_NewRuntime2', 'JS_ExecutePendingJob', 'JS_SetUncatchableError'):
        assert len(re.findall(r' T _?' + symbol + r'$', symbols, re.M)) == 1, symbol
    compiled = list(target.glob('debug/build/rquickjs-sys-*/out/quickjs.c'))
    assert compiled and all(patch.sha256(p) == digest for p in compiled)
    # Cargo may retain older fingerprints; binaries emitted by the current build
    # are selected via Cargo, and each suite asserts the mode+source digest.
    report = dict(mode=mode, source_sha256=digest, changed=changed,
                  compiled_sources=[str(p.relative_to(HERE)) for p in compiled],
                  sys_package=sys_crates[0]['id'], sanitized=args.sanitize,
                  executable_sha256=patch.sha256(binaries[0]),
                  executable=str(binaries[0].relative_to(HERE)),
                  archives={str(p.relative_to(HERE)): patch.sha256(p)
                            for p in target.glob('debug/build/rquickjs-sys-*/out/libquickjs.a')})
    (HERE / 'build' / (mode + ('-sanitized' if args.sanitize else '') + '-provenance.json')).write_text(json.dumps(report, indent=2) + '\n')
    command = ['cargo', 'test', '--offline', '--locked']
    if args.baseline:
        command += ['baseline_swallow_negative_control']
    command += ['--', '--test-threads=1', '--nocapture']
    run(command, env, 90)
    run(['cargo', 'fmt', '--check'], env)
    run(['cargo', 'clippy', '--offline', '--locked', '--all-targets', '--', '-D', 'warnings'], env)


if __name__ == '__main__':
    main()
