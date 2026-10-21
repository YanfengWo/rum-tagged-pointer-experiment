"""Untimed launch-mode checks for the diagnostic executable."""
import json
import os
from pathlib import Path
import subprocess
import sys

out = Path(sys.argv[1]).resolve()
study = Path(__file__).resolve().parent
repo = study.parents[2]
builds = json.loads((out / "builds.json").read_text())
assert builds["status"] == "PASS"
binary = builds["variants"][0]["binary"]
source = out / "mode-smoke.rum"
source.write_text('Print["MODE_SMOKE_PASS"];\n')
env = {k: v for k, v in os.environ.items() if not k.startswith("RUM_")}
records = []
for label, mode in [("unset", None), ("base", "base"), ("matcher", "matcher"),
                    ("depth", "depth"), ("combined", "combined"), ("empty", ""),
                    ("unknown", "other"), ("spaced", " matcher")]:
    configured = dict(env)
    if mode is not None:
        configured["RUM_REWRITE_SHORTCUTS_MODE"] = mode
    result = subprocess.run(["python3", str(repo / "scripts/run-bounded.py"), "--timeout", "30",
                             "--", binary, "--file", str(source)], env=configured,
                            stdin=subprocess.DEVNULL, capture_output=True, timeout=45)
    (out / f"mode-smoke-{label}.stdout").write_bytes(result.stdout)
    (out / f"mode-smoke-{label}.stderr").write_bytes(result.stderr)
    valid = label in {"unset", "base", "matcher", "depth", "combined"}
    if valid:
        assert result.returncode == 0 and b"MODE_SMOKE_PASS" in result.stdout and not result.stderr.strip()
    else:
        assert result.returncode != 0 and b"RUM_REWRITE_SHORTCUTS_MODE" in result.stderr
        assert b"MODE_SMOKE_PASS" not in result.stdout
    records.append({"label": label, "valid": valid, "exit_code": result.returncode})
record = {"status": "PASS", "checks": records,
          "limits": "Independent-process mode handling; no test mutates the selected mode within a process."}
(out / "mode-smoke.json").write_text(json.dumps(record, indent=2) + "\n")
print("Eight launch-mode checks PASS")
