"""Serial eight-treatment Williams validation; no builds or instrumentation."""
import argparse
from collections import Counter
from datetime import datetime, timezone
import hashlib
import json
import os
from pathlib import Path
import platform
import shutil
import subprocess
import sys
sys.dont_write_bytecode = True
import workloads
import shape_workloads


def all_sources(timed=True):
    return dict(workloads.sources(timed), shape=(shape_workloads.source(timed), shape_workloads.LABELS, "ISO_SHAPE_PASS"))


IMPLEMENTATIONS = ["base", "matcher", "depth", "combined"]
MODES = [mode + "-" + alias for alias in ("a", "b") for mode in IMPLEMENTATIONS]


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def utc():
    return datetime.now(timezone.utc).isoformat()


def write_once(path, content):
    if path.exists():
        assert path.read_text(encoding="utf-8") == content, path
    else:
        path.write_text(content, encoding="utf-8")


def assert_binary(path, digest):
    assert path.is_file() and os.access(path, os.X_OK), path
    assert sha(path) == digest, path
    assert b"RUM_REWRITE_WORK_LEDGER" not in path.read_bytes(), path


def context(out):
    study = Path(__file__).resolve().parent
    repo = study.parents[2]
    builds = json.loads((out / "builds.json").read_text(encoding="utf-8"))
    assert builds["status"] == "PASS" and builds["prior_binaries_preserved"]
    assert builds["default_checkout_changed"] is False and builds["instrumentation_present"] is False
    variants = {item["mode"]: item for item in builds["variants"]}
    assert set(variants) == set(IMPLEMENTATIONS)
    for item in variants.values():
        assert_binary(Path(item["binary"]), item["binary_sha256"])
    source_root = Path(builds["source_root"])
    assert "rewrite_work_ledger" not in (source_root / "crates/rum-core/src/lib.rs").read_text(encoding="utf-8")
    script = repo / "scripts/core-language-microbench.sh"
    pin = json.loads((study.parent / "expression-census/corpus/PIN.json").read_text(encoding="utf-8"))
    assert sha(script) == pin["upstream_key_source_hashes"]["scripts/core-language-microbench.sh"]
    env = {k: v for k, v in os.environ.items() if not k.startswith("RUM_")}
    env["RUM_CORE_MICRO_RETAINED_MODULE_GC_BATCHES"] = "3"
    for timed in (False, True):
        for name, (content, _, _) in all_sources(timed).items():
            suffix = "-corpus.rum" if timed else "-preflight-corpus.rum"
            write_once(out / (name + suffix), content)
    write_once(out / "semantic-corpus.rum", workloads.semantic_source())
    bounded = ["python3", str(repo / "scripts/run-bounded.py"), "--timeout", "600", "--"]
    return repo, builds, variants, script, env, bounded


def execute(corpus, binary, prefix, repo, env, bounded, sentinel):
    result = subprocess.run(bounded + [str(binary), "--file", str(corpus)], cwd=repo, env=env,
                            stdin=subprocess.DEVNULL, capture_output=True, timeout=615)
    Path(str(prefix) + ".stdout").write_bytes(result.stdout)
    Path(str(prefix) + ".stderr").write_bytes(result.stderr)
    assert result.returncode == 0 and sentinel.encode() in result.stdout and not result.stderr.strip(), prefix
    assert b"rum-work\t" not in result.stdout + result.stderr, prefix


def main(out, preflight):
    repo, builds, variants, script, env, bounded = context(out)
    preflight_record = {"builds_sha256": sha(out / "builds.json"),
                        "sources": {p.name: sha(p) for p in out.glob("*-corpus.rum")}}
    if preflight:
        assert not (out / "preflight-status.txt").exists()
        for mode in IMPLEMENTATIONS:
            execute(out / "semantic-corpus.rum", variants[mode]["binary"], out / f"preflight-semantic-{mode}",
                    repo, env, bounded, "ISO_SEMANTICS_PASS")
            for name, (_, _, sentinel) in all_sources(False).items():
                execute(out / (name + "-preflight-corpus.rum"), variants[mode]["binary"], out / f"preflight-{name}-{mode}",
                        repo, env, bounded, sentinel)
        (out / "preflight-record.json").write_text(json.dumps(preflight_record, indent=2) + "\n", encoding="utf-8")
        (out / "preflight-status.txt").write_text("PREFLIGHT_PASS\n", encoding="utf-8")
        print("PASS: four implementations, full values, aliases and side-effect order; no timings")
        return
    assert (out / "preflight-status.txt").read_text(encoding="utf-8").strip() == "PREFLIGHT_PASS"
    assert json.loads((out / "preflight-record.json").read_text(encoding="utf-8")) == preflight_record
    assert not (out / "timing-plan.json").exists()
    controls = Path(builds["target_root"]) / "control-binaries"
    controls.mkdir(exist_ok=False)
    binaries = {}
    for mode in MODES:
        implementation = mode.rsplit("-", 1)[0]
        original = variants[implementation]
        binary = controls / mode
        shutil.copy2(original["binary"], binary)
        assert_binary(binary, original["binary_sha256"])
        binaries[mode] = {"binary": str(binary), "sha256": original["binary_sha256"], "implementation": implementation}
    first = [0, 1, 7, 2, 6, 3, 5, 4]
    orders = [[MODES[(index + row) % 8] for index in first] for row in range(8)]
    assert all(Counter(order[p] for order in orders) == Counter(MODES) for p in range(8))
    predecessors = Counter((order[i], order[i + 1]) for order in orders for i in range(7))
    assert len(predecessors) == 56 and set(predecessors.values()) == {1}
    boundaries = Counter((orders[i][-1], orders[i + 1][0]) for i in range(7))
    cpu = min(os.sched_getaffinity(0))
    plan = {"recorded_on": "2026-10-20", "modes": MODES, "orders": orders, "blocks": 8,
            "binaries": binaries, "builds_sha256": sha(out / "builds.json"), "cpu": cpu,
            "kernel": platform.release(), "machine": platform.machine(),
            "aslr": Path("/proc/sys/kernel/randomize_va_space").read_text().strip(),
            "cpu_model": next(line.split(":", 1)[1].strip() for line in Path("/proc/cpuinfo").read_text().splitlines() if line.startswith("model name")),
            "suite_script_sha256": sha(script), "scale": 5, "reps": 9, "retained_reps": 11,
            "within_block_predecessor_counts": {a + " -> " + b: n for (a, b), n in sorted(predecessors.items())},
            "block_boundary_predecessor_counts": {a + " -> " + b: n for (a, b), n in sorted(boundaries.items())},
            "sources": {k: {"sha256": sha(out / (k + "-corpus.rum")), "labels": v[1], "sentinel": v[2],
                            "preflight_sha256": sha(out / (k + "-preflight-corpus.rum"))} for k, v in all_sources().items()},
            "semantic_source_sha256": sha(out / "semantic-corpus.rum"), "started_utc": utc(),
            "method": "Eight serial balanced Williams blocks; four uninstrumented implementations with byte-identical A/A copies each; fixed CPU/environment; unchanged 112-label scale5/min9 suite; 11 retained samples per focus/pipeline/shape case. Each process median is one observation; independent units are eight process blocks. Full per-label gate; no regression exemption or A/A noise subtraction.",
            "limits": "Same machine/toolchain, synthetic programs. Within-block predecessor balance excludes seven recorded block boundaries. Byte-identical controls cannot establish compilation-layout causality. No project builds/uploads should run during timing; other host activity and thermal/frequency drift remain possible. Resource observation numbers may differ; value checks do not prove all resource-limit equivalence."}
    (out / "timing-plan.json").write_text(json.dumps(plan, indent=2) + "\n", encoding="utf-8")
    bounded += ["taskset", "-c", str(cpu)]
    with (out / "timing-progress.tsv").open("w", encoding="utf-8", buffering=1) as journal:
        journal.write("block\tposition\tmode\tstatus\tutc\n")
        for block, order in enumerate(orders, 1):
            for position, mode in enumerate(order, 1):
                journal.write(f"{block}\t{position}\t{mode}\tstarted\t{utc()}\n")
                binary = binaries[mode]["binary"]
                with (out / f"suite-{block:02d}-{mode}.log").open("w", encoding="utf-8") as log:
                    subprocess.run(bounded + ["bash", str(script), "--rum-cli", binary, "--output-dir", str(out / f"suite-{block:02d}-{mode}"),
                                             "--reps", "9", "--scale", "5", "--timeout", "480"], cwd=repo, env=env,
                                   stdin=subprocess.DEVNULL, stdout=log, stderr=subprocess.STDOUT, check=True, timeout=615)
                for name, (_, _, sentinel) in all_sources().items():
                    execute(out / (name + "-corpus.rum"), binary, out / f"{name}-{block:02d}-{mode}", repo, env, bounded, sentinel)
                journal.write(f"{block}\t{position}\t{mode}\tpassed\t{utc()}\n")
            print("block", block, "PASS", flush=True)
    for item in binaries.values():
        assert_binary(Path(item["binary"]), item["sha256"])
    assert sha(out / "builds.json") == plan["builds_sha256"] and sha(script) == plan["suite_script_sha256"]
    for name, item in plan["sources"].items():
        assert sha(out / (name + "-corpus.rum")) == item["sha256"]
    (out / "timing-status.txt").write_text("TIMINGS_PASS\n", encoding="utf-8")
    print("PASS: 64 complete suites and 192 focus/pipeline/shape processes")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("evidence", type=Path)
    parser.add_argument("--preflight", action="store_true")
    args = parser.parse_args()
    main(args.evidence.resolve(), args.preflight)
