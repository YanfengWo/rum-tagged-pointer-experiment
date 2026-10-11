"""Validate frozen sources, checks, physical cost equations and study timings."""
import argparse
import hashlib
import json
from pathlib import Path
import re
import sys
sys.dont_write_bytecode = True
from timings import analyze as analyze_timings

def read(p):
    return p.read_text(encoding="utf-8")

def sha(p):
    return hashlib.sha256(p.read_bytes()).hexdigest()

def analyze(out):
    study = Path(__file__).resolve().parent
    research = study.parent
    repo = study.parents[2]
    assert read(out / "status.txt").strip() == "PASS"
    checks = {}
    for name, wanted in [("native.log", 14), ("native-all-features.log", 14),
                         ("core-api.log", 10), ("miri-stacked.log", 10),
                         ("miri-tree.log", 10)]:
        log = read(out / name)
        count = sum(map(int, re.findall(r"test result: ok\. (\d+) passed; 0 failed", log)))
        assert count == wanted, (name, count)
        assert not re.search(r"test result: FAILED|(^|\n)error", log)
        checks[name] = count
    log = read(out / "workspace.log")
    results = re.findall(r"test result: ok\. (\d+) passed; 0 failed; (\d+) ignored", log)
    assert sum(int(n) for n, _ in results) == 2241, results
    assert sum(int(n) for _, n in results) == 7, results
    assert not re.search(r"test result: FAILED|(^|\n)error", log)
    checks["isolated_workspace_passed"] = 2241
    checks["isolated_workspace_ignored"] = 7
    checks["workspace_result_groups"] = len(results)
    for name in ["native.log", "native-all-features.log", "core-api.log",
                 "timing-build.log", "allocation-build.log", "format.log"]:
        assert not re.search(r"(^|\n)(error:|error\[|warning:)", read(out / name)), name
    assert "132 snapshots; 660 full equalities and scans" in read(out / "semantic.log")
    cow = re.search(r"PASS: (\d+) full COW/source equalities", read(out / "semantic.log"))
    assert cow and int(cow[1]) == 620
    checks["full_cow_and_source_equalities"] = int(cow[1])
    assert "PASS: RUM_PROFILE_REPR unset" in read(out / "semantic.log")
    checks["original_profiling_disabled"] = "PASS"
    assert "132 snapshots; 660 fresh graph windows returned to baseline" in read(out / "allocations.log")
    pin = json.loads(read(research / "expression-census/corpus/PIN.json"))
    for row in pin["files"]:
        p = research / "expression-census/corpus" / row["path"]
        assert sha(p) == row["sha256"] and p.stat().st_size == row["bytes"], p
    for rel, digest in pin["prototype_source_hashes"].items():
        assert sha(research / rel) == digest, rel
    for rel, digest in pin["upstream_key_source_hashes"].items():
        assert sha(repo / rel) == digest, rel
    for stage in ["expression-census", "symbol-sharing"]:
        previous = json.loads(read(research / f"evidence/{stage}/summary.json"))
        for rel, digest in previous["source_hashes"].items():
            assert sha(research / stage / rel) == digest, rel
    overlay = json.loads(read(out / "overlay.json"))
    assert overlay["upstream_commit"] == pin["upstream_commit"]
    base = (repo / "crates/rum-core/src/expr.rs").read_bytes()
    extension = (study / "symbol-access.rs").read_bytes()
    assert b"unsafe" not in extension
    assert overlay["original_expr_sha256"] == hashlib.sha256(base).hexdigest()
    assert overlay["extension_sha256"] == hashlib.sha256(extension).hexdigest()
    assert overlay["overlay_expr_sha256"] == hashlib.sha256(base + b"\n" + extension).hexdigest()
    assert not re.search(r"\bunsafe\s+(impl|fn|\{)", read(study / "src/safe24.rs"))

    prior_rows = json.loads(read(research / "evidence/symbol-sharing/allocations.json"))
    prior = {(r["entry"]["label"], r["entry"]["kind"]): r for r in prior_rows}
    raw = json.loads(read(out / "allocations.json"))
    assert len(raw) == 132
    cases = {"original": 32, "safe24": 24, "tagged16": 16, "tagged8": 8, "previous-shared8": 8}
    seen = set()
    verified_prior = 0
    rows = []
    for row in raw:
        key = (row["entry"]["label"], row["entry"]["kind"])
        assert key not in seen
        seen.add(key)
        measurements = {r["representation"]: r for r in row["measurements"]}
        assert len(row["measurements"]) == 5 and set(measurements) == set(cases)
        p = row["physical"]
        assert p["nodes"] == p["slots"] + 1
        assert all(isinstance(v, int) and v >= 0 for v in p.values())
        retained = {c: r["retained_requested_bytes"] for c, r in measurements.items()}
        for c, r in measurements.items():
            assert r["root_stack_bytes"] == cases[c]
            assert 0 <= r["retained_requested_bytes"] <= r["peak_requested_bytes"]
            assert r["remaining_bytes"] == 0 and r["allocation_calls"] >= 0
        original = retained["original"]
        assert retained["safe24"] - original == 48*p["owned_atoms"] - 8*p["slots"], key
        assert retained["tagged16"] - original == 48*p["owned_atoms"] - 16*p["slots"], key
        assert retained["tagged8"] - original == 48*p["tagged8_heap_nodes"] - 24*p["slots"], key
        assert retained["safe24"] - retained["tagged16"] == 8*p["slots"], key
        if key in prior:
            old = {r["representation"]: r for r in prior[key]["measurements"]}
            assert row["entry"] == prior[key]["entry"], key
            assert original == old["original-normalized"]["retained_requested_bytes"], key
            assert retained["previous-shared8"] == old["compact-shared"]["retained_requested_bytes"], key
            verified_prior += 1
        rows.append({"entry": row["entry"], "physical": p,
                     "retained_requested_bytes": retained,
                     "peak_requested_bytes": {c: r["peak_requested_bytes"] for c, r in measurements.items()},
                     "allocation_calls": {c: r["allocation_calls"] for c, r in measurements.items()}})
    assert set(prior).issubset(seen) and verified_prior == 128
    checks.update(roundtrip_equalities=660, scan_equalities=660, allocation_windows=660,
                  physical_cost_equations=396, same_policy_width_equations=132,
                  prior_retained_bytes_match=128, frozen_dependencies="PASS", format="PASS",
                  default_core_source_unchanged="PASS", isolated_safe_extension_hashes="PASS")
    timing = analyze_timings(out)
    initial = None
    if (out / "initial-codegen-source.json").exists():
        record = json.loads(read(out / "initial-codegen-source.json"))
        def strip_annotations(value):
            return re.sub(r'\s+|#\[inline(?:\(always\))?\]|#\[cfg\(feature="allocation-count"\)\]', "", re.sub(r'\s+', "", value))
        for rel, value in record["source_text"].items():
            assert hashlib.sha256(value.encode("utf-8")).hexdigest() == record["source_sha256"][rel], rel
            assert strip_annotations(value) == strip_annotations(read(study / rel)), rel
        initial = analyze_timings(out, prefix="initial-")
        assert initial["scan_checksums"] == timing["scan_checksums"]
        checks["initial_and_final_codegen_same_algorithm"] = "PASS"
    hint = None
    if (out / "hint-codegen-source.json").exists():
        record = json.loads(read(out / "hint-codegen-source.json"))
        assert hashlib.sha256(record["scan_source_text"].encode("utf-8")).hexdigest() == record["scan_sha256"]
        assert strip_annotations(record["scan_source_text"]) == strip_annotations(read(study / "src/bin/scan.rs"))
        hint = analyze_timings(out, prefix="hint-")
        assert hint["scan_checksums"] == timing["scan_checksums"]
    source_hashes = {p.relative_to(study).as_posix(): sha(p) for p in sorted(study.rglob("*"))
        if p.is_file() and not {"target", "__pycache__"}.intersection(p.parts)
        and p.name not in {"README.md", "DESIGN.md", "SEMANTICS.md"}}
    raw_hashes = {p.name: sha(p) for p in sorted(out.iterdir())
        if p.is_file() and p.name not in {"summary.json", "timing-summary.json"}}
    summary = {"status": "PASS", "recorded_on": "2026-10-11", "checks": checks,
        "previous_commit": "3c101a4d590abc4a8ed9fdefea2d0369d60d638f",
        "upstream_commit": pin["upstream_commit"], "kernel_integration_implemented": False,
        "safe_control_scope": "Safe24 has safe enum storage. Common borrowed views use private raw Arc projections; the package is not zero-unsafe.",
        "additive_api_scope": "Two safe Symbol accessors exist only in the isolated source copy. The default rum core is unchanged.",
        "allocation_scope": "Fresh alias-preserving one-pass Original/new graphs, System requested retained heap bytes. Excludes input, root stack, permanent symbol pool and allocator overhead. Previous-shared8 keeps its previous bridge/cache implementation; its scratch is included in peak/calls.",
        "timing_scope": "Snapshot operations, not evaluator A/B. Same one-pass bridge for Original and new layouts; old shared8 keeps previous construction. Default release profile, allocation counters absent. RUM_PROFILE_REPR unset: Original wrappers retain disabled profiling branches while new wrappers omit them. Original/new timing differences include this API/codegen difference, not solely layout. Three rotating runs, one warmup plus nine measured rounds each, run2 reversed. Paired medians/ranges are descriptive, not significance tests.",
        "layout_bytes": cases, "allocation_rows": rows, "timings": timing,
        "initial_codegen_timings": initial,
        "hint_codegen_timings": hint,
        "source_hashes": source_hashes, "raw_evidence_hashes": raw_hashes}
    (out / "summary.json").write_text(json.dumps(summary, indent=2) + "\n", encoding="utf-8", newline="\n")
    return summary

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("evidence", type=Path)
    args = parser.parse_args()
    summary = analyze(args.evidence.resolve())
    print(json.dumps({"status": "PASS", "checks": summary["checks"]}, indent=2))
