"""Validate frozen controls, complete checks, physical costs and paired timings."""
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
    for name, wanted in [("native.log", 20), ("native-all-features.log", 20),
                         ("core-api.log", 11), ("miri-stacked.log", 14),
                         ("miri-tree.log", 14)]:
        log = read(out / name)
        count = sum(map(int, re.findall(r"test result: ok\. (\d+) passed; 0 failed", log)))
        assert count == wanted, (name, count)
        assert not re.search(r"test result: FAILED|(^|\n)error", log)
        checks[name] = count
    results = re.findall(r"test result: ok\. (\d+) passed; 0 failed; (\d+) ignored", read(out / "workspace.log"))
    assert sum(int(n) for n, _ in results) == 2242, results
    assert sum(int(n) for _, n in results) == 7 and len(results) == 29, results
    assert not re.search(r"test result: FAILED|(^|\n)error", read(out / "workspace.log"))
    checks.update(isolated_workspace_passed=2242, isolated_workspace_ignored=7, workspace_result_groups=29)
    for name in ["native.log", "native-all-features.log", "core-api.log",
                 "timing-build.log", "allocation-build.log", "format.log"]:
        assert not re.search(r"(^|\n)(error:|error\[|warning:)", read(out / name)), name
    assert "135 snapshots; 675 full equalities; 1350 full/key scans" in read(out / "semantic.log")
    cow = re.search(r"PASS: (\d+) full COW/source equalities", read(out / "semantic.log"))
    assert cow and int(cow[1]) == 635
    assert "PASS: RUM_PROFILE_REPR unset" in read(out / "semantic.log")
    assert "135 snapshots; 675 fresh graph windows returned to baseline" in read(out / "allocations.log")
    pin = json.loads(read(research / "expression-census/corpus/PIN.json"))
    for row in pin["files"]:
        p = research / "expression-census/corpus" / row["path"]
        assert sha(p) == row["sha256"] and p.stat().st_size == row["bytes"], p
    for rel, digest in pin["prototype_source_hashes"].items():
        assert sha(research / rel) == digest, rel
    for rel, digest in pin["upstream_key_source_hashes"].items():
        assert sha(repo / rel) == digest, rel
    for stage in ["expression-census", "symbol-sharing", "layout-design"]:
        old = json.loads(read(research / f"evidence/{stage}/summary.json"))
        for rel, digest in old["source_hashes"].items():
            assert sha(research / stage / rel) == digest, rel
    overlay = json.loads(read(out / "overlay.json"))
    base = (repo / "crates/rum-core/src/expr.rs").read_bytes()
    extension = (study / "symbol-access.rs").read_bytes()
    assert b"unsafe" not in extension and overlay["upstream_commit"] == pin["upstream_commit"]
    assert overlay["original_expr_sha256"] == hashlib.sha256(base).hexdigest()
    assert overlay["extension_sha256"] == hashlib.sha256(extension).hexdigest()
    assert overlay["overlay_expr_sha256"] == hashlib.sha256(base + b"\n" + extension).hexdigest()
    assert not re.search(r"\bunsafe\s+(impl|fn|\{)", read(study / "src/safe24.rs"))

    prior_rows = json.loads(read(research / "evidence/layout-design/allocations.json"))
    prior = {(r["entry"]["label"], r["entry"]["kind"]): r for r in prior_rows}
    raw = json.loads(read(out / "allocations.json"))
    assert len(raw) == 135
    cases = {"original": 32, "previous16": 16, "safe24-borrow": 24,
             "typed16-move": 16, "typed16-borrow": 16}
    seen, rows, verified_prior = set(), [], 0
    for row in raw:
        key = (row["entry"]["label"], row["entry"]["kind"])
        assert key not in seen
        seen.add(key)
        m = {r["representation"]: r for r in row["measurements"]}
        assert len(row["measurements"]) == 5 and set(m) == set(cases)
        p = row["physical"]
        assert p["nodes"] == p["slots"] + 1
        assert all(isinstance(v, int) and v >= 0 for v in p.values())
        assert p["owned_atoms"] == p["strings"] + p["bigs"] + p["rationals"]
        retained = {c: r["retained_requested_bytes"] for c, r in m.items()}
        for c, r in m.items():
            assert r["root_stack_bytes"] == cases[c]
            assert 0 <= r["retained_requested_bytes"] <= r["peak_requested_bytes"]
            assert r["remaining_bytes"] == 0 and r["allocation_calls"] >= 0
        original = retained["original"]
        payload = p["string_arc_bytes"] - p["string_bytes"] + 16*(p["bigs"]+p["rationals"])
        assert retained["previous16"] - original == 48*p["owned_atoms"] - 16*p["slots"], key
        assert retained["typed16-borrow"] - original == payload - 16*p["slots"], key
        assert retained["safe24-borrow"] - original == payload - 8*p["slots"], key
        assert retained["typed16-move"] == retained["typed16-borrow"], key
        assert retained["safe24-borrow"] - retained["typed16-borrow"] == 8*p["slots"], key
        if key in prior:
            old = {r["representation"]: r for r in prior[key]["measurements"]}
            assert row["entry"] == prior[key]["entry"], key
            assert original == old["original"]["retained_requested_bytes"], key
            assert retained["previous16"] == old["tagged16"]["retained_requested_bytes"], key
            verified_prior += 1
        rows.append({"entry": row["entry"], "physical": p, "retained_requested_bytes": retained,
                     "peak_requested_bytes": {c: r["peak_requested_bytes"] for c, r in m.items()},
                     "allocation_calls": {c: r["allocation_calls"] for c, r in m.items()}})
    assert set(prior).issubset(seen) and verified_prior == 132
    checks.update(roundtrip_equalities=675, full_and_key_scan_equalities=1350,
                  full_cow_and_source_equalities=635, allocation_windows=675,
                  physical_cost_equations=405, same_policy_width_equations=135,
                  move_borrow_retained_equalities=135, previous_retained_exact_matches=132,
                  frozen_sources_and_corpus="PASS", default_core_unchanged="PASS")
    timing = analyze_timings(out)
    checks.update(timing_raw_rows=timing["raw_rows"], timing_measured_rows=timing["measured_rows"])
    source_hashes = {p.relative_to(study).as_posix(): sha(p) for p in sorted(study.rglob("*"))
        if p.is_file() and not {"target", "__pycache__"}.intersection(p.parts)
        and p.name not in {"README.md", "DESIGN.md", "SEMANTICS.md"}}
    raw_hashes = {p.name: sha(p) for p in sorted(out.iterdir())
                  if p.is_file() and p.name != "summary.json"}
    summary = {"status": "PASS", "recorded_on": "2026-10-11", "checks": checks,
        "previous_commit": "5d0bc48aa7d19c5f097493e101df7c88f0fb6262",
        "upstream_commit": pin["upstream_commit"], "kernel_integration_implemented": False,
        "safe_control_scope": "Safe24 safe enum storage; common borrowed views contain private unsafe Arc projections.",
        "additive_api_scope": "Two safe Symbol accessors and SymbolText::interned_id exist only in an isolated pinned copy.",
        "allocation_scope": "Fresh graph System requested heap bytes, excluding input/root stack/permanent interner/allocator overhead. Arc<str> padded request equation is pinned-target evidence, not a portable layout contract. No aligned-string fallback observed in these windows.",
        "timing_scope": "Fixed snapshot operations, no evaluator A/B. Actual unchanged stage22 implementation is previous16. Borrow builds typed payloads directly from source; move means clone an Atom then consume it, not move the entire source graph. All use the same iterative alias-preserving bridge algorithm. Original wrappers retain disabled profiling branches. Owned-payload COW differs from original deep clones. Full view reconstructs Symbol; key view observes equality/hash identity only. Big/Rational checksums format decimal text and allocate in all cases. Counters absent; three rotating affinity runs with reversed run2, descriptive paired ratios.",
        "layout_bytes": cases, "allocation_rows": rows, "timings": timing,
        "source_hashes": source_hashes, "raw_evidence_hashes": raw_hashes}
    (out / "summary.json").write_text(json.dumps(summary, indent=2) + "\n", encoding="utf-8", newline="\n")
    return summary


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("evidence", type=Path)
    args = parser.parse_args()
    summary = analyze(args.evidence.resolve())
    print(json.dumps({"status": "PASS", "checks": summary["checks"]}, indent=2))
