"""Check the recursive study's gates, paired samples and requested-byte data."""
import argparse
import csv
import hashlib
import json
from pathlib import Path
import re
import statistics as stats

parser = argparse.ArgumentParser()
parser.add_argument("evidence", type=Path)
parser.add_argument("--require-miri", action="store_true")
args = parser.parse_args()
out = args.evidence
study = Path(__file__).resolve().parent
repo = study.parents[2]
assert (out / "status.txt").read_text(encoding="utf-8").strip() == "PASS"
checks = {}
for name, expected in [("native.log", 16), ("native-all-features.log", 16)] + ([("miri-stacked.log", 13), ("miri-tree.log", 13),
    ("fixture-miri-stacked.log", 1), ("fixture-miri-tree.log", 1)] if args.require_miri else []):
    text = (out / name).read_text(encoding="utf-8")
    groups = re.findall(r"test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored", text)
    assert groups and sum(int(g[0]) for g in groups) == expected, name
    assert all(g[1:] == ("0", "0") for g in groups), name
    checks[name] = expected
assert not (out / "format.log").read_text(encoding="utf-8").strip()

datasets = ["wide-integer", "wide-string", "balanced", "shared-dag", "deep-chain"]
operations = ["scan", "clone_drop", "cow", "bridge_roundtrip", "last_drop", "iterative_release"]
representations = ["original", "safe", "compact"]
expected = {(d, op, rep, r) for d in datasets for op in operations for rep in representations
            if not (rep == "original" and op == "bridge_roundtrip") for r in range(10)}
runs = []
checksums = {}
for number in range(1, 4):
    samples = {}
    with (out / f"run-{number}.tsv").open(encoding="utf-8", newline="") as source:
        for row in csv.DictReader(source, delimiter="\t"):
            d, op, rep = row["dataset"], row["operation"], row["representation"]
            key = (d, op, rep, int(row["round"]))
            assert key not in samples
            samples[key] = int(row["nanoseconds"])
            assert samples[key] > 0
            value = int(row["checksum"])
            if op == "scan":
                assert value == checksums.setdefault(d, value), key
            else:
                assert value == {"clone_drop": 5000, "cow": 10, "bridge_roundtrip": 3,
                                 "last_drop": 5, "iterative_release": 5}[op], key
    assert set(samples) == expected, number
    runs.append(samples)

results = []
for d in datasets:
    for op in operations:
        reps = [r for r in representations if op != "bridge_roundtrip" or r != "original"]
        medians = {rep: stats.median(s[d, op, rep, r] for s in runs for r in range(1, 10)) / 1e6 for rep in reps}
        ratios = {rep: [stats.median(s[d, op, "compact", r] / s[d, op, rep, r] for r in range(1, 10)) for s in runs]
                  for rep in reps if rep != "compact"}
        results.append({"dataset": d, "operation": op, "median_ms": medians,
            "paired_median_compact_ratios": {rep: stats.median(s[d, op, "compact", r] / s[d, op, rep, r]
                for s in runs for r in range(1, 10)) for rep in ratios},
            "per_run_paired_ratios": ratios,
            "sample_ms_range": {rep: [min(s[d, op, rep, r] for s in runs for r in range(1, 10)) / 1e6,
                max(s[d, op, rep, r] for s in runs for r in range(1, 10)) / 1e6] for rep in reps}})

def allocation_rows(name):
    with (out / name).open(encoding="utf-8", newline="") as source:
        rows = list(csv.DictReader(source, delimiter="\t"))
    assert len(rows) == 15
    assert {(r["dataset"], r["representation"]) for r in rows} == {(d, rep) for d in datasets for rep in representations}
    for row in rows:
        for field in list(row)[2:]: row[field] = int(row[field])
        assert row["remaining_bytes"] == 0 and row["retained_requested_bytes"] > 0
        assert row["peak_requested_bytes"] >= row["retained_requested_bytes"]
        assert row["root_stack_bytes"] == {"original": 32, "safe": 16, "compact": 8}[row["representation"]]
    return rows

allocations = allocation_rows("allocations.tsv")
if args.require_miri:
    for model in ["stacked", "tree"]:
        allocation_rows(f"allocator-miri-{model}.tsv")
        text = (out / f"allocator-miri-{model}.log").read_text(encoding="utf-8")
        assert "error:" not in text.lower()
        checks[f"allocator-miri-{model}"] = "15 reduced-fixture accounting windows returned to baseline"

baseline = json.loads((study.parent / "evidence/baseline-summary.json").read_text(encoding="utf-8"))
for path, digest in baseline["source_hashes"].items():
    assert hashlib.sha256((repo / path).read_bytes()).hexdigest() == digest, path
source_paths = [p for p in study.rglob("*") if p.is_file() and "target" not in p.parts
                and "__pycache__" not in p.parts and p.suffix in {".rs", ".toml", ".lock"}]
source_paths += [study.parent / "compact-expr/src/lib.rs", study.parent / "compact-expr/src/borrowed.rs"]
summary = {"status": "PASS", "recorded_on": "2026-10-10", "checks": checks,
    "upstream_commit": baseline["upstream"]["upstream_commit"], "upstream_key_source_hashes_unchanged": True,
    "previous_workspace_result": {"passed": 2238, "ignored": 7, "rerun_for_this_external_only_stage": False,
                                  "source_stage": "representation-study"},
    "results": results, "allocations": allocations,
    "methodology": {"runs": 3, "measured_rounds_per_run": 9, "samples_per_label": 27,
        "warmup_rounds": 1, "scan_passes": 10, "clone_drop_passes": 5000, "cow_passes": 10,
        "bridge_roundtrip_passes": 3, "last_release_passes": 5,
        "counter_build_separate_from_timing": True, "CPU_affinity": "first allowed CPU",
        "external_contention_ruled_out": False, "RSS_measured": False,
        "allocation_metric": "live bytes requested from System, excluding allocator metadata and source baseline",
        "original_memory_setup": "fresh original fixture factory", "wrapper_memory_setup": "bridge from retained original fixture",
        "scan_checksum_is_full_traversal": True, "other_checksums_are_iteration_counts": True,
        "roundtrip_equality_checked_before_timings": True},
    "source_hashes": {p.relative_to(study.parent).as_posix(): hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted(source_paths)},
    "raw_evidence_hashes": {p.name: hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted(out.iterdir())
                           if p.is_file() and p.name != "summary.json"},
    "kernel_integration_implemented": False, "full_soundness_proof_completed": False,
    "whole_evaluator_speedup_demonstrated": False}
(out / "summary.json").write_text(json.dumps(summary, indent=2) + "\n", encoding="utf-8", newline="\n")
for row in results:
    print(row["dataset"], row["operation"], {k: round(v, 3) for k, v in row["paired_median_compact_ratios"].items()})
print("Gates, 2550 raw timing rows, value checks and requested-byte accounting verified.")
