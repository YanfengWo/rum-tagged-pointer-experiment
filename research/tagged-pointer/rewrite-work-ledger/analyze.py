"""Audit exact counts and semantics; deliberately no performance conclusion."""
import argparse
import ast
import hashlib
import json
from pathlib import Path
import re
import sys
sys.dont_write_bytecode = True
from parse_ledger import describe, parse, total
from capacity_analysis import audit as audit_capacity
import workloads

MODES = ["base", "wide", "wide-flat", "flat"]


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def load(path):
    return json.loads(path.read_text(encoding="utf-8"))


def workspace(path):
    groups = re.findall(r"test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;", path.read_text(encoding="utf-8"))
    assert groups and all(int(r[1]) == 0 for r in groups)
    result = {"passed": sum(int(r[0]) for r in groups), "ignored": sum(int(r[2]) for r in groups), "groups": len(groups)}
    assert result == {"passed": 2241, "ignored": 7, "groups": 29}, result
    return result


def resources(path):
    result = {}
    for line in path.read_text(encoding="utf-8").replace("\\t", "\t").splitlines():
        if line.startswith("RESOURCE_PROBE\t"):
            _, key, value = line.split("\t")
            assert key not in result
            result[key] = int(value)
    assert len(result) == 7 and all(v > 0 for v in result.values())
    return result


def expectations(mode, label, size, stats, rows):
    wide = "wide" in mode
    flat = "flat" in mode
    hit = label != "miss"
    plus = label in {"increment", "increment-reversed"}
    depth_factor = {"increment": 5, "increment-reversed": 5, "identity": 3,
                    "miss": 2, "opaque-body": 4, "flat-nested": 6}[label]
    assert stats["traversal_entries"] == size + 1
    assert stats["depth_helper_entries"] == depth_factor * size + 1
    assert stats["matcher_entries"] == stats["binding_snapshot_entries"] == (2 * size if hit else 0)
    assert stats["snapshot_named_items"] == stats["rhs_instantiations"] == (size if hit else 0)
    assert stats["wide_copy_events"] == (0 if wide else 1)
    assert stats["plus_builtin_calls"] == stats["plus_integer_success"] == (size if plus else 0)
    count = size - 1 if label == "increment" else 0
    assert stats["plus_normalization_rebuilds"] == stats["plus_sort_comparisons"] == count
    assert stats["plus_flatten_calls"] == count and stats["plus_flatten_nested"] == 0
    assert stats["plus_fresh_flatten_outputs"] == (0 if flat else count)
    assert stats["plus_reused_flatten_outputs"] == (count if flat else 0)
    assert stats["plus_fresh_flatten_capacity_slots"] == (0 if flat else 4 * count)
    assert stats["plus_reused_flatten_capacity_slots"] == (2 * count if flat else 0)
    if plus:
        assert stats["into_vec_copy_slots"] == (0 if wide else size) + 2 * count
        assert stats["two_slot_copy_events"] == count
    if label == "flat-nested":
        assert stats["all_fresh_flatten_outputs"] == size
        assert stats["all_reused_flatten_outputs"] == 0
        assert total(rows, "flatten-nested-calls", "phaseF") == size
    assert total(rows, "rhs-substitution", "request", phase_prefix="replace/traversal/") == (size if hit else 0)
    assert total(rows, "plus-builtin", "calls", phase_prefix="replace/result-evaluation") == (size if plus else 0)


def main(out):
    study = Path(__file__).resolve().parent
    for path in study.glob("*.py"):
        ast.parse(path.read_text(encoding="utf-8"), filename=path.name)
        assert path.stat().st_size < 10000, path.name
    builds, runs, preflight = load(out / "builds.json"), load(out / "runs.json"), load(out / "preflight.json")
    assert builds["status"] == runs["status"] == preflight["status"] == "PASS"
    assert builds["prior_binaries_preserved"] and not builds["default_checkout_changed"]
    assert builds["formal_performance_gate"] == runs["performance_gate"] == "DEFERRED_USER_ACTIVE_DESKTOP"
    assert [v["mode"] for v in builds["variants"]] == MODES
    assert len(runs["runs"]) == 292 and len(preflight["runs"]) == 25
    for record in runs["runs"] + preflight["runs"]:
        assert sha(Path(record["source"])) == record["source_sha256"] and record["exit_code"] == 0
    by_name = {r["name"]: r for r in runs["runs"]}
    checks, results = {}, []
    for item in builds["variants"]:
        mode = item["mode"]
        assert item["ledger_module_sha256"] == sha(study / "ledger.rs")
        assert sha(Path(item["binary"])) == item["binary_sha256"]
        assert not (out / f"{mode}-format.log").read_text().strip()
        checks[mode] = workspace(out / f"{mode}-workspace.log")
        for label in workloads.CASES:
            for size in workloads.SIZES:
                source = out / "inputs" / f"{label}-{size}.rum"
                assert source.read_text() == workloads.program(label, size)
                records = []
                for rep in [1, 2]:
                    name = f"{mode}-{label}-{size}-{rep}"
                    assert by_name[name]["ledger_enabled"] and by_name[name]["binary_sha256"] == item["binary_sha256"]
                    rows = parse(out / "raw" / (name + ".stderr"))
                    stats = describe(rows, size)
                    expectations(mode, label, size, stats, rows)
                    assert "WORK_LEDGER_PASS" in (out / "raw" / (name + ".stdout")).read_text()
                    records.append(rows)
                assert records[0] == records[1], (mode, label, size)
                off = out / "raw" / f"{mode}-{label}-{size}-off"
                assert not off.with_suffix(".stderr").read_text().strip()
                assert off.with_suffix(".stdout").read_bytes() == (out / "raw" / f"{mode}-{label}-{size}-1.stdout").read_bytes()
                assert off.with_suffix(".stdout").read_bytes() == (out / "raw" / f"preflight-{label}-{size}.stdout").read_bytes()
                results.append({"mode": mode, "case": label, "size": size, "stats": stats, "rows": records[0]})
    baseline_resource = resources(out / "raw/preflight-resource.stdout")
    observed = {mode: resources(out / "raw" / f"{mode}-resource.stdout") for mode in MODES}
    assert observed["base"] == observed["wide"] == baseline_resource
    assert observed["flat"] == observed["wide-flat"]
    capacity = audit_capacity(out, builds, study)
    summary = {"status": "PASS", "recorded_on": "2026-10-19", "diagnostic_only": True,
               "performance_gate": "DEFERRED_USER_ACTIVE_DESKTOP", "builds": builds,
               "counts": {"cases": 24, "variants": 4, "ledger_roots": 192, "counter_off_controls": 96,
                          "resource_observation_processes": 4, "main_processes": 292, "preflight_processes": 25,
                          "construction_history_processes": 2},
               "workspace": checks, "results": results, "resource_observations": {"baseline": baseline_resource, **observed},
               "capacity_observations": capacity,
               "limits": ["Logical counts and capacity slots are not elapsed-time or total allocator-byte measurements.",
                          "Generated synchronous Replace workloads; lexical scopes do not cover later suspension resumption.",
                          "Flat early return preserves expression elements/order/identities but resource observations and tight modeled-memory thresholds may differ.",
                          "Candidate full microbenchmark gate deferred while the user uses the desktop; no acceleration or no-regression claim.",
                          "Prior timing stages recorded A/A controls but no external-application load trace; those controls do not exclude every external source of variation."],
               "source_hashes": {p.name: sha(p) for p in sorted(study.iterdir()) if p.is_file()},
               "raw_evidence_hashes": {p.relative_to(out).as_posix(): sha(p) for p in sorted(out.rglob("*")) if p.is_file() and p.name not in {"summary.json", "report.md"}}}
    (out / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")
    lines = ["# Exact synchronous rewrite work counts", "", "No performance timing comparison was run. Capacity is in Expr slots, not allocator bytes.", "", "| Mode | Case | N | depth helper entries | matcher entries | RHS instantiations | Plus calls | wide copies | two-slot copies | fresh Flat Vec outputs | reused Flat Vec outputs |", "| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |"]
    for result in results:
        s = result["stats"]
        lines.append(f'| {result["mode"]} | {result["case"]} | {result["size"]} | {s["depth_helper_entries"]} | {s["matcher_entries"]} | {s["rhs_instantiations"]} | {s["plus_builtin_calls"]} | {s["wide_copy_events"]} | {s["two_slot_copy_events"]} | {s["all_fresh_flatten_outputs"]} | {s["all_reused_flatten_outputs"]} |')
    lines += ["", "Resource observations (not required to be numerically equal):", "", "| Measurement | Base | Flat candidate |", "| --- | ---: | ---: |"]
    for key in baseline_resource:
        lines.append(f'| {key} | {baseline_resource[key]} | {observed["flat"][key]} |')
    lines += ["", "Construction-history ByteCount observations (SameQ passes):", ""]
    for name, data in capacity["results"].items():
        lines.append(f'{name}: literal={data["values"]["literal"]}, Apply={data["values"]["applied"]}')
    lines += ["", *summary["limits"]]
    (out / "report.md").write_text("\n".join(lines) + "\n")
    print(json.dumps({"status": "PASS", "counts": summary["counts"], "workspace": checks,
                      "resources": observed, "formal_performance_gate": summary["performance_gate"]}, indent=2))


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("evidence", type=Path)
    main(parser.parse_args().evidence.resolve())
