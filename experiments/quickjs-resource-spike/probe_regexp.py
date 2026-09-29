"""Local counterexample runner; outer kills are reported as failures, never engine stops."""
import os
from pathlib import Path
import signal
import subprocess
import sys

binary = Path(__file__).resolve().parent / "target/debug/regexp_compile"
for count in (1000, 4000, 16000, 64000):
    process = subprocess.Popen(
        [str(binary), str(count)], stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT, text=True, start_new_session=True,
    )
    try:
        output, _ = process.communicate(timeout=5)
    except subprocess.TimeoutExpired:
        os.killpg(process.pid, signal.SIGKILL)
        output, _ = process.communicate()
        print(output, end="", flush=True)
        print("EXTERNAL SAFETY TIMEOUT (5 s): no engine-level stop or retirement proven.")
        # ENTER must precede the kill; startup delays alone are not a reproducer.
        if "ENTER regexp compile:" not in output or "INTERRUPT_CALLBACK" in output:
            raise SystemExit("Unexpected evidence; inspect the probe before drawing a conclusion.")
        raise SystemExit(2)
    print(output, end="", flush=True)
    if process.returncode:
        raise SystemExit(process.returncode)
print("Counterexample returned without polling; no engine interruption was demonstrated.")
sys.exit(0)
