"""Run an existing WSL research driver and sample Windows lock flags."""
import argparse
import ctypes
from ctypes import wintypes
from datetime import datetime, timedelta, timezone
import json
from pathlib import Path
import shlex
import subprocess
import sys
import tempfile
import time

root = Path(__file__).resolve().parent.parent
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("evidence", type=Path)
parser.add_argument("driver", help="Driver path relative to the rum base checkout")
args = parser.parse_args()
out = args.evidence.resolve()
out.relative_to((root / "evidence").resolve())
assert out.is_dir() and not (out / "lock-state.json").exists()
api = ctypes.WinDLL("wtsapi32", use_last_error=True)
api.WTSQuerySessionInformationW.argtypes = [wintypes.HANDLE, wintypes.DWORD, ctypes.c_int,
    ctypes.POINTER(ctypes.c_void_p), ctypes.POINTER(wintypes.DWORD)]
api.WTSQuerySessionInformationW.restype = wintypes.BOOL
api.WTSFreeMemory.argtypes = [ctypes.c_void_p]
sid = wintypes.DWORD()
assert ctypes.windll.kernel32.ProcessIdToSessionId(ctypes.windll.kernel32.GetCurrentProcessId(), ctypes.byref(sid))
assert sys.getwindowsversion().major >= 10


def state():
    pointer, count = ctypes.c_void_p(), wintypes.DWORD()
    assert api.WTSQuerySessionInformationW(None, sid.value, 25, ctypes.byref(pointer), ctypes.byref(count))
    try:
        assert count.value >= 20 and ctypes.c_uint32.from_address(pointer.value).value == 1
        assert ctypes.c_uint32.from_address(pointer.value + 8).value == sid.value
        flag = ctypes.c_uint32.from_address(pointer.value + 16).value
        assert flag in {0, 1}
        return {"utc": datetime.now(timezone.utc).isoformat(), "state": "unlocked" if flag else "locked"}
    finally:
        api.WTSFreeMemory(pointer)


def linux(path):
    path = str(path).replace("\\", "/")
    return "/mnt/" + path[0].lower() + path[2:]


samples = [state()]
protocol_path = out / "session-protocol.json"
protocol = json.loads(protocol_path.read_text(encoding="utf-8")) if protocol_path.exists() else {}
protocol.update(recorded_before_timing=True,
                recorded_on=datetime.now(timezone(timedelta(hours=-4))).date().isoformat(),
                client_timezone="Etc/GMT+4", windows_utc=datetime.now(timezone.utc).isoformat(),
                decision="Measure serial paired processes under the sampled session state; retain transitions.",
                limit="This is not a randomized lock/unlock causal comparison.")
protocol_path.write_text(json.dumps(protocol, indent=2) + "\n", encoding="utf-8")
command = ("cd " + shlex.quote(linux(root.parents[1]))
           + "; bash experiments/rum-tagged/scripts/run.sh python3 " + shlex.quote(args.driver)
           + " " + shlex.quote(linux(out)))
print("Initial session:", samples[0]["state"], "; starting monitored validation", flush=True)
stdout_path, stderr_path = out / "monitor-runner.stdout", out / "monitor-runner.stderr"
# Keep live logs outside out: the native driver's final copy must not overwrite
# an open stream. Direct file handles also avoid full-pipe child deadlocks.
with tempfile.TemporaryDirectory(prefix="rum-monitor-") as temporary:
    live_stdout, live_stderr = Path(temporary) / "stdout", Path(temporary) / "stderr"
    print("Live console directory:", temporary, flush=True)
    with live_stdout.open("wb") as stdout_log, live_stderr.open("wb") as stderr_log:
        process = subprocess.Popen(["wsl", "-d", "Ubuntu-22.04", "--", "bash", "-lc", command],
            stdout=stdout_log, stderr=stderr_log)
        while process.poll() is None:
            time.sleep(1)
            samples.append(state())
    stdout = live_stdout.read_bytes().decode("utf-8", errors="replace")
    stderr = live_stderr.read_bytes().decode("utf-8", errors="replace")
stdout_path.write_text(stdout, encoding="utf-8")
stderr_path.write_text(stderr, encoding="utf-8")
record = {"method": "WTSInfoEx SessionFlags sampled each second around the WSL process lifetime; Windows10+ 0=locked,1=unlocked. No account fields retained.",
          "samples": samples, "interval_seconds": 1,
          "all_samples_unlocked": all(s["state"] == "unlocked" for s in samples),
          "exit_code": process.returncode,
          "limits": "Discrete samples are not continuous coverage. Windows/WSL UTC clocks may differ; lifetime ordering uses the outer process, not cross-clock timestamp subtraction."}
(out / "lock-state.json").write_text(json.dumps(record, indent=2) + "\n", encoding="utf-8")
print(stdout)
print("Lock samples:", len(samples), "all unlocked:", record["all_samples_unlocked"])
assert process.returncode == 0
