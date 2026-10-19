"""Balanced A/A and native Sort remeasurement using unchanged saved binaries."""
import argparse
import csv
from datetime import datetime, timezone
import hashlib
import json
import os
from pathlib import Path
import platform
import subprocess

MODES = ["base-a", "api", "base-b", "sort-api", "sort-24"]
FOCUS = ["sort-symbol-default-width4000", "sort-symbol-default-width16000",
         "association-part-key-read-width4000", "association-part-key-read-width32000"]
def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

def main(out, previous, build):
    study = Path(__file__).resolve().parent
    repo = study.parents[2]
    assert platform.system() == "Linux"
    assert not out.exists(), "Use a fresh evidence directory; do not overwrite a prior study"
    out.mkdir(parents=True)
    metadata = {}
    script = repo / "scripts/core-language-microbench.sh"
    pin = json.loads((previous / "summary.json").read_text())
    assert pin["status"] == "PASS"
    expected_script = json.loads((study.parent / "expression-census/corpus/PIN.json").read_text())[
        "upstream_key_source_hashes"]["scripts/core-language-microbench.sh"]
    assert sha(script) == expected_script
    for mode in MODES:
        original = "base" if mode.startswith("base-") else mode
        binary = build / original / "release/rum-cli"
        expected = (previous / f"{original}-binary.sha256").read_text().split()[0]
        assert sha(binary) == expected, mode
        metadata[mode] = {"binary": str(binary), "sha256": expected}
    assert metadata["base-a"] == metadata["base-b"]
    orders = [MODES[n:] + MODES[:n] for n in range(5)]
    orders += [list(reversed(order)) for order in orders.copy()]
    cpu = min(os.sched_getaffinity(0))
    env = os.environ.copy()
    for name in list(env):
        if name.startswith("RUM_PROFILE_"):
            env[name] = "0"
    env.update(RUM_PROFILE_REPR="0", RUM_PROFILE_ALLOC="0")
    plan = {"recorded_on": "2026-10-19", "modes": MODES, "orders": orders,
            "blocks": 10, "cpu": cpu, "scale": 5, "reps": 9, "binaries": metadata,
            "script_sha256": expected_script, "focus_labels": FOCUS,
            "purpose": "Same exact binary A/A noise, API-only controls and native safe Symbol24 Sort replication",
            "limits": "Same host, existing build artifacts; no independent rebuild, new compiler, real-application corpus or causal proof of code-layout effects. Timings use original per-label minimum of repetitions and original GC convention.",
            "comparison": "Within each block, candidate / sqrt(base-a * base-b); also base-b/base-a. All labels retained; no equal-label whole-application mean. Focus labels fixed before measurement.",
            "screen": "Exploratory repeatable direction: >=8/10 block ratios same direction, median >=5% away from 1 and median absolute log effect exceeds label A/A 90th-percentile absolute log noise. This screen is not a significance test or a no-regression proof.",
            "kernel": platform.release(), "machine": platform.machine(),
            "cpu_model": next((line.split(":", 1)[1].strip() for line in Path("/proc/cpuinfo").read_text().splitlines() if line.startswith("model name")), "unavailable"),
            "aslr": Path("/proc/sys/kernel/randomize_va_space").read_text().strip(),
            "started_utc": datetime.now(timezone.utc).isoformat()}
    (out / "plan.json").write_text(json.dumps(plan, indent=2) + "\n")
    with (out / "progress.tsv").open("w", buffering=1) as progress:
        progress.write("block\tposition\tmode\tstatus\n")
        for block, order in enumerate(orders, 1):
            for position, mode in enumerate(order, 1):
                target = out / f"block-{block:02d}-{mode}"
                command = ["python3", str(repo / "scripts/run-bounded.py"), "--timeout", "900", "--",
                           "taskset", "-c", str(cpu), "bash", str(script), "--rum-cli", metadata[mode]["binary"],
                           "--output-dir", str(target), "--reps", "9", "--scale", "5", "--timeout", "600"]
                progress.write(f"{block}\t{position}\t{mode}\tstarted\n")
                with (out / f"block-{block:02d}-{mode}.log").open("w") as log:
                    subprocess.run(command, cwd=repo, env=env, stdin=subprocess.DEVNULL,
                                   stdout=log, stderr=subprocess.STDOUT, check=True)
                with (target / "timings.tsv").open() as source:
                    rows = list(csv.DictReader(source, delimiter="\t"))
                assert len(rows) == 112 and len({r["label"] for r in rows}) == 112
                progress.write(f"{block}\t{position}\t{mode}\tpassed\n")
    for mode in MODES:
        assert sha(Path(metadata[mode]["binary"])) == metadata[mode]["sha256"]
    (out / "status.txt").write_text("TIMINGS_PASS\n")
    print("PASS: 50 serial full-suite runs, including 20 identical-binary controls")

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("evidence", type=Path)
    parser.add_argument("previous", type=Path)
    parser.add_argument("--build", type=Path, default=Path("/var/tmp/rum-native-dispatch-build"))
    args = parser.parse_args()
    main(args.evidence.resolve(), args.previous.resolve(), args.build.resolve())
