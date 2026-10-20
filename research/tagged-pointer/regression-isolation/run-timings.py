"""Serial retained-sample and unchanged-suite controls; no runtime edits."""
import argparse
from collections import Counter
from datetime import datetime, timezone
import hashlib
import json
import os
from pathlib import Path
import platform
import subprocess
import sys
sys.dont_write_bytecode = True
import control_cases
import pipeline_cases

MODES = ["base-a", "candidate-a", "base-b", "candidate-b"]


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main(out, preflight):
    study = Path(__file__).resolve().parent
    repo = study.parents[2]
    builds = json.loads((out / "build-controls.json").read_text())
    assert builds["status"] == "PASS" and builds["baseline_source_restored"]
    by_name = {v["mode"]: v for v in builds["builds"]}
    # Two copies per implementation make simultaneous A/A controls available.
    # Equality is audited, not assumed, before treating modes as aliases.
    assert builds["base_replay_byte_identical"] and builds["candidate_replay_byte_identical"]
    binaries = {mode: by_name[("candidate" if "candidate" in mode else "base") + ("-0" if mode.endswith("a") else "-1")]
                for mode in MODES}
    for item in binaries.values():
        assert sha(Path(item["binary"])) == item["sha256"]
    env = {k: v for k, v in os.environ.items() if not k.startswith("RUM_")}
    env["RUM_CORE_MICRO_RETAINED_MODULE_GC_BATCHES"] = "3"
    sources = {"control": (control_cases.make_source(), control_cases.CASES, "ISO_CONTROL_PASS"),
               "pipeline": (pipeline_cases.make_source(), pipeline_cases.CASES, "ISO_PIPELINE_PASS")}
    for name, (content, labels, sentinel) in sources.items():
        path = out / (name + "-corpus.rum")
        if path.exists():
            assert path.read_text() == content
        else:
            path.write_text(content)
    bounded = ["python3", str(repo / "scripts/run-bounded.py"), "--timeout", "600", "--"]
    if preflight:
        assert not (out / "preflight-status.txt").exists()
        for mode in ["base-a", "candidate-a"]:
            for name, (_, _, sentinel) in sources.items():
                result = subprocess.run(bounded + [binaries[mode]["binary"], "--file", str(out / (name + "-corpus.rum"))],
                                        stdin=subprocess.DEVNULL, capture_output=True, env=env, timeout=615)
                (out / f"preflight-{name}-{mode}.stdout").write_bytes(result.stdout)
                (out / f"preflight-{name}-{mode}.stderr").write_bytes(result.stderr)
                assert result.returncode == 0 and sentinel.encode() in result.stdout and not result.stderr.strip(), (name, mode)
        (out / "preflight-status.txt").write_text("PREFLIGHT_PASS\n")
        print("PASS: complete expected results and aliases on both implementations; not A/B data")
        return
    assert (out / "preflight-status.txt").read_text().strip() == "PREFLIGHT_PASS"
    assert not (out / "timing-plan.json").exists()
    # Four-treatment Williams design repeated twice: positions and each
    # ordered within-block predecessor pair balanced, including A/A controls.
    indices = [[0, 1, 3, 2], [1, 2, 0, 3], [2, 3, 1, 0], [3, 0, 2, 1]]
    orders = [[MODES[i] for i in order] for order in indices + list(reversed(indices))]
    assert all(Counter(order[p] for order in orders) == Counter({m: 2 for m in MODES}) for p in range(4))
    predecessors = Counter((order[i], order[i + 1]) for order in orders for i in range(3))
    assert len(predecessors) == 12 and set(predecessors.values()) == {2}
    boundaries = Counter((orders[i][-1], orders[i + 1][0]) for i in range(len(orders) - 1))
    cpu = min(os.sched_getaffinity(0))
    script = repo / "scripts/core-language-microbench.sh"
    pin = json.loads((study.parent / "expression-census/corpus/PIN.json").read_text())
    assert sha(script) == pin["upstream_key_source_hashes"]["scripts/core-language-microbench.sh"]
    plan = {"recorded_on": "2026-10-19", "modes": MODES, "orders": orders, "blocks": len(orders),
            "binaries": binaries, "cpu": cpu, "kernel": platform.release(), "machine": platform.machine(),
            "aslr": Path("/proc/sys/kernel/randomize_va_space").read_text().strip(),
            "cpu_model": next(line.split(":", 1)[1].strip() for line in Path("/proc/cpuinfo").read_text().splitlines() if line.startswith("model name")),
            "suite_script_sha256": sha(script), "scale": 5, "reps": 9, "retained_reps": 11,
            "within_block_predecessor_counts": {a + " -> " + b: n for (a, b), n in sorted(predecessors.items())},
            "block_boundary_predecessor_counts": {a + " -> " + b: n for (a, b), n in sorted(boundaries.items())},
            "sources": {k: {"sha256": sha(out / (k + "-corpus.rum")), "labels": v[1], "sentinel": v[2]} for k, v in sources.items()},
            "started_utc": datetime.now(timezone.utc).isoformat(),
            "method": "Eight serial Williams blocks, fixed CPU, no concurrent project builds. Full unchanged 112-label min9 suite, standalone controls, synthetic end-to-end pipelines. Retained corpus primary is process median; paired implementation ratio uses geometric mean of its two mode medians. Eight independent process blocks, not 176 independent repetitions. All RUM_* cleared then retained GC batches=3.",
            "limits": "Same host/toolchain, synthetic workloads, no real application distribution. Williams predecessor balance holds within blocks; seven boundary transitions are separately recorded and not balanced. A/A samples expose but do not subtract noise. Source/target replay reproducibility rules out observed repeated-build variation here, not compiled-code layout effects or general compiler nondeterminism."}
    (out / "timing-plan.json").write_text(json.dumps(plan, indent=2) + "\n")
    bounded += ["taskset", "-c", str(cpu)]
    with (out / "timing-progress.tsv").open("w", buffering=1) as journal:
        journal.write("block\tposition\tmode\tstatus\tutc\n")
        for block, order in enumerate(orders, 1):
            for position, mode in enumerate(order, 1):
                journal.write(f'{block}\t{position}\t{mode}\tstarted\t{datetime.now(timezone.utc).isoformat()}\n')
                binary = binaries[mode]["binary"]
                with (out / f"suite-{block:02d}-{mode}.log").open("w") as log:
                    subprocess.run(bounded + ["bash", str(script), "--rum-cli", binary, "--output-dir", str(out / f"suite-{block:02d}-{mode}"),
                                             "--reps", "9", "--scale", "5", "--timeout", "480"], cwd=repo, env=env, stdin=subprocess.DEVNULL, stdout=log, stderr=subprocess.STDOUT, check=True, timeout=615)
                for name, (_, _, sentinel) in sources.items():
                    result = subprocess.run(bounded + [binary, "--file", str(out / (name + "-corpus.rum"))],
                                            cwd=repo, env=env, stdin=subprocess.DEVNULL, capture_output=True, timeout=615)
                    (out / f"{name}-{block:02d}-{mode}.stdout").write_bytes(result.stdout)
                    (out / f"{name}-{block:02d}-{mode}.stderr").write_bytes(result.stderr)
                    assert result.returncode == 0 and sentinel.encode() in result.stdout and not result.stderr.strip(), (block, mode, name)
                journal.write(f'{block}\t{position}\t{mode}\tpassed\t{datetime.now(timezone.utc).isoformat()}\n')
            print("block", block, "PASS", flush=True)
    assert all(sha(Path(v["binary"])) == v["sha256"] for v in binaries.values())
    (out / "timing-status.txt").write_text("TIMINGS_PASS\n")
    print("PASS: 32 complete suites, 32 standalone control processes, 32 pipeline processes")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("evidence", type=Path)
    parser.add_argument("--preflight", action="store_true")
    args = parser.parse_args()
    main(args.evidence.resolve(), args.preflight)
