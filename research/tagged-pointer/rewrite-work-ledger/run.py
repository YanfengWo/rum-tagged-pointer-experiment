"""Deterministic work probes and result checks; no elapsed-time comparisons."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
sys.dont_write_bytecode = True
import workloads


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main(out, preflight, pilot=False):
    study = Path(__file__).resolve().parent
    repo = study.parents[2]
    env = {k: v for k, v in os.environ.items() if not k.startswith("RUM_")}
    bounded = ["python3", str(repo / "scripts/run-bounded.py"), "--timeout", "120", "--"]
    inputs = out / "inputs"
    inputs.mkdir(exist_ok=True)
    for label in workloads.CASES:
        for size in workloads.SIZES:
            source = inputs / f"{label}-{size}.rum"
            value = workloads.program(label, size)
            if source.exists():
                assert source.read_text() == value
            else:
                source.write_text(value, encoding="utf-8")
    records = []

    def execute(name, binary, source, enabled):
        local = env.copy()
        if enabled:
            local["RUM_REWRITE_WORK_LEDGER"] = "1"
        result = subprocess.run(bounded + [str(binary), "--file", str(source)], cwd=repo, env=local,
                                stdin=subprocess.DEVNULL, capture_output=True, timeout=135)
        raw = out / "raw"
        raw.mkdir(exist_ok=True)
        (raw / (name + ".stdout")).write_bytes(result.stdout)
        (raw / (name + ".stderr")).write_bytes(result.stderr)
        sentinel = b"RESOURCE_PROBE_VALUE_ALIAS_PASS" if source.name == "resource-probes.rum" else b"WORK_LEDGER_PASS"
        assert result.returncode == 0 and sentinel in result.stdout, name
        if enabled and source.name != "resource-probes.rum":
            assert b"rum-work\troot\t1\tbegin" in result.stderr and b"rum-work\troot\t1\tend" in result.stderr, name
        else:
            assert not result.stderr.strip(), name
        records.append({"name": name, "binary_sha256": sha(binary), "source": str(source), "source_sha256": sha(source),
                        "ledger_enabled": enabled, "exit_code": result.returncode})

    if preflight:
        assert not (out / "preflight.json").exists()
        baseline = Path("/var/tmp/rum-copy-build/base-final/release/rum-cli")
        prior = json.loads((out.parent / "copy-attribution/matched-build.json").read_text())
        assert sha(baseline) == prior["baseline_binary_sha256"]
        for source in sorted(inputs.glob("*.rum")):
            execute("preflight-" + source.stem, baseline, source, False)
        execute("preflight-resource", baseline, study / "resource-probes.rum", False)
        (out / "preflight.json").write_text(json.dumps({"status": "PASS", "runs": records}, indent=2) + "\n")
        print("PASS:24 baseline full-result/alias probes and resource observations; not performance data")
        return
    builds = json.loads((out / "builds.json").read_text())
    assert builds["status"] in (["RUNNING", "PASS"] if pilot else ["PASS"]) and builds["diagnostic_only"] and builds["jobs"] == 2
    assert json.loads((out / "preflight.json").read_text())["status"] == "PASS"
    record_file = out / ("pilot.json" if pilot else "runs.json")
    assert not record_file.exists()
    for item in builds["variants"][:1] if pilot else builds["variants"]:
        mode, binary = item["mode"], Path(item["binary"])
        assert sha(binary) == item["binary_sha256"]
        for source in sorted(inputs.glob("*.rum")):
            if pilot and not source.stem.endswith("-8"):
                continue
            prefix = ("pilot-" if pilot else "") + mode
            for repetition in ([1] if pilot else [1, 2]):
                execute(f'{prefix}-{source.stem}-{repetition}', binary, source, True)
            execute(f'{prefix}-{source.stem}-off', binary, source, False)
        execute(f'{"pilot-" if pilot else ""}{mode}-resource', binary, study / "resource-probes.rum", False)
        print(mode, "COUNTS_AND_OUTPUTS_PASS", flush=True)
    record_file.write_text(json.dumps({"status": "PASS", "runs": records,
        "diagnostic_only": True, "performance_gate": "DEFERRED_USER_ACTIVE_DESKTOP",
        "no_clock_measurements": True, "repetitions": 1 if pilot else 2,
        "scope": "Generated workloads, single synchronous Replace per fresh process, exact deterministic operation counts and full expected values/aliases. External application usage is permitted; these data do not establish elapsed-time acceleration."}, indent=2) + "\n")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("evidence", type=Path)
    parser.add_argument("--preflight", action="store_true")
    parser.add_argument("--pilot", action="store_true")
    args = parser.parse_args()
    assert not (args.preflight and args.pilot)
    main(args.evidence.resolve(), args.preflight, args.pilot)
