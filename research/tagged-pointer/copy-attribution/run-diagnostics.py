"""Run original and diagnostic copies on the exact prior semantic probe inputs."""
import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import subprocess
import sys
sys.dont_write_bytecode = True

def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

def run(out, trace, suffix):
    study = Path(__file__).resolve().parent
    repo = study.parents[2]
    previous = out.parent / "hotspot-validation"
    old = json.loads((previous / "summary.json").read_text())
    baseline = Path("/var/tmp/rum-native-dispatch-build/base/release/rum-cli")
    assert sha(baseline) == old["binary_sha256"]
    raw = out / ("diagnostic" + suffix + "-raw")
    raw.mkdir(exist_ok=False)
    env = {k: v for k, v in os.environ.items() if not k.startswith("RUM_PROFILE_")}
    env.update(RUM_TRACE_COPIES="1", RUM_COPY_STACKS="0", RUM_PROFILE_REPR="1",
               RUM_PROFILE_LEXICAL_SUBSTITUTION="1", RUM_PROFILE_LEXICAL_DEFER="1", RUM_PROFILE_BUILTIN_TIMING="1")
    records = []
    def execute(name, binary, source, stacks=False):
        child_env = env.copy()
        child_env["RUM_COPY_STACKS"] = "1" if stacks else "0"
        result = subprocess.run(["python3", str(repo / "scripts/run-bounded.py"), "--timeout", "120", "--", str(binary), "--file", str(source)], env=child_env, stdin=subprocess.DEVNULL, capture_output=True, timeout=135)
        (raw / f"{name}.stdout").write_bytes(result.stdout)
        (raw / f"{name}.stderr").write_bytes(result.stderr)
        assert result.returncode == 0 and b"RUM_HOTSPOT_PASS" in result.stdout, name
        records.append({"name": name, "binary_sha256": sha(binary), "source_sha256": sha(source), "source": str(source), "exit_code": result.returncode})
    for source in sorted((previous / "inputs").glob("*.rum")):
        assert sha(source) == old["raw_evidence_hashes"][source.relative_to(previous).as_posix()]
        for mode, binary in [("base", baseline), ("trace", trace)]:
            execute(mode + "-" + source.stem, binary, source)
    module_path = study.parent / "hotspot-validation/workloads.py"
    spec = importlib.util.spec_from_file_location("copy_probe_workloads", module_path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    source = out / ("rewrite-stack" + suffix + ".rum")
    source.write_text(module.program("rewrite-wide", 8), encoding="utf-8")
    execute("rewrite-stack", trace, source, stacks=True)
    (out / ("diagnostics" + suffix + ".json")).write_text(json.dumps({"status": "PASS", "runs": records, "diagnostic_only": True, "baseline_sha256": sha(baseline), "trace_sha256": sha(trace)}, indent=2) + "\n")
    print("PASS: 36 baseline/trace semantic probes and one small stack probe")

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("evidence", type=Path)
    parser.add_argument("--trace", type=Path, default=Path("/var/tmp/rum-copy-build/trace/release/rum-cli"))
    parser.add_argument("--suffix", default="")
    args = parser.parse_args()
    assert args.suffix in {"", "-final", "-candidate"}
    run(args.evidence.resolve(), args.trace.resolve(), args.suffix)
