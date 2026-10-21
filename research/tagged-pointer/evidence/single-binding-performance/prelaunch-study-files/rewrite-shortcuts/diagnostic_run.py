"""Check 96 complete logical ledgers and values; no timing or allocator claims."""
import argparse
from collections import Counter
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
sys.dont_write_bytecode = True

MODES = ("base", "matcher", "depth", "combined")
FACTORS = {"increment": 5, "increment-reversed": 5, "identity": 3,
           "miss": 2, "opaque-body": 4, "flat-nested": 6}


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main(out):
    study = Path(__file__).resolve().parent
    repo, legacy = study.parents[2], study.parent / "rewrite-work-ledger"
    sys.path.insert(0, str(legacy))
    import workloads
    from parse_ledger import describe, parse, total
    builds = json.loads((out / "builds.json").read_text(encoding="utf-8"))
    assert builds["status"] == "PASS" and builds["diagnostic_only"]
    assert builds["no_clock_measurements"] and builds["source_restored_to_instrumented_base"]
    assert builds["prior_binaries_preserved"] and not builds["default_checkout_changed"]
    assert [v["mode"] for v in builds["variants"]] == list(MODES)
    for name, expected in builds["source_hashes"].items():
        assert sha(study.parent / name) == expected, name
    assert not (out / "runs.json").exists() and not (out / "summary.json").exists()
    (out / "inputs").mkdir(exist_ok=False)
    (out / "raw").mkdir(exist_ok=False)
    sources = {}
    for label in workloads.CASES:
        for size in workloads.SIZES:
            path = out / "inputs" / f"{label}-{size}.rum"
            path.write_text(workloads.program(label, size), encoding="utf-8")
            sources[label, size] = path
    assert len(sources) == 24
    env = {k: v for k, v in os.environ.items() if not k.startswith("RUM_")}
    env["RUM_REWRITE_WORK_LEDGER"] = "1"
    bounded = ["python3", str(repo / "scripts/run-bounded.py"), "--timeout", "120", "--"]
    records, results, baseline, outputs = [], [], {}, {}
    runs = {"status": "RUNNING", "builds_sha256": sha(out / "builds.json"), "runs": records,
            "diagnostic_only": True, "no_clock_measurements": True}

    def save():
        (out / "runs.json").write_text(json.dumps(runs, indent=2) + "\n", encoding="utf-8")

    def invariant_rows(rows):
        # The only new work is the explicit leaf-result event. General matcher
        # scopes/snapshots and input-depth visits are the intended removals.
        excluded = {"matcher", "binding-snapshot", "replace-depth-node", "rewrite-leaf-match"}
        result = Counter()
        for row in rows:
            if row["kind"] in excluded or (row["kind"] == "scope" and row["detail"] == "matcher"):
                continue
            result[row["kind"], row["detail"], row["amount"]] += row["calls"]
        return result

    save()
    try:
        for item in builds["variants"]:
            mode, binary = item["mode"], Path(item["binary"])
            assert sha(binary) == item["binary_sha256"]
            for (label, size), source in sources.items():
                name = f"{mode}-{label}-{size}"
                result = subprocess.run(bounded + [str(binary), "--file", str(source)],
                                        cwd=repo, env=env, stdin=subprocess.DEVNULL,
                                        capture_output=True, timeout=135)
                stdout, stderr = out / "raw" / (name + ".stdout"), out / "raw" / (name + ".stderr")
                stdout.write_bytes(result.stdout)
                stderr.write_bytes(result.stderr)
                assert result.returncode == 0 and b"WORK_LEDGER_PASS" in result.stdout, name
                rows = parse(stderr)
                stats = describe(rows, size)
                hit, plus = label != "miss", label in {"increment", "increment-reversed"}
                fast, depth = mode in {"matcher", "combined"}, mode in {"depth", "combined"}
                stats["fast_leaf_hits"] = total(rows, "rewrite-leaf-match", "hit")
                assert total(rows, "rewrite-leaf-match", "miss") == 0
                assert stats["traversal_entries"] == size + 1
                expected_depth = (FACTORS[label] - 2) * size if depth else FACTORS[label] * size + 1
                assert stats["depth_helper_entries"] == expected_depth, (name, stats)
                general = 0 if fast or not hit else 2 * size
                assert stats["matcher_entries"] == stats["binding_snapshot_entries"] == general
                assert stats["snapshot_named_items"] == (size if hit and not fast else 0)
                assert stats["fast_leaf_hits"] == (size if hit and fast else 0)
                assert stats["rhs_instantiations"] == (size if hit else 0)
                assert stats["plus_builtin_calls"] == stats["plus_integer_success"] == (size if plus else 0)
                assert stats["wide_copy_events"] == 1
                count = size - 1 if label == "increment" else 0
                assert stats["plus_normalization_rebuilds"] == stats["plus_sort_comparisons"] == count
                assert stats["plus_fresh_flatten_outputs"] == count
                assert stats["plus_reused_flatten_outputs"] == 0
                assert stats["plus_fresh_flatten_capacity_slots"] == 4 * count
                if plus:
                    assert stats["two_slot_copy_events"] == count
                    assert stats["into_vec_copy_slots"] == size + 2 * count
                if label == "flat-nested":
                    assert total(rows, "flatten-nested-calls", "phaseF") == size
                    assert stats["all_fresh_flatten_outputs"] == size
                assert total(rows, "rhs-substitution", "request", phase_prefix="replace/traversal/") == (size if hit else 0)
                assert total(rows, "plus-builtin", "calls", phase_prefix="replace/result-evaluation") == (size if plus else 0)
                key = label, size
                if mode == "base":
                    baseline[key] = (stats, invariant_rows(rows))
                    outputs[key] = result.stdout
                else:
                    assert result.stdout == outputs[key]
                    changed = {"depth_helper_entries", "matcher_entries", "binding_snapshot_entries",
                               "snapshot_named_items", "fast_leaf_hits"}
                    assert {k: v for k, v in stats.items() if k not in changed} == {
                        k: v for k, v in baseline[key][0].items() if k not in changed}, name
                    assert invariant_rows(rows) == baseline[key][1], name
                    assert baseline[key][0]["depth_helper_entries"] - stats["depth_helper_entries"] == (2 * size + 1 if depth else 0)
                    assert baseline[key][0]["matcher_entries"] - stats["matcher_entries"] == (2 * size if fast and hit else 0)
                records.append({"name": name, "mode": mode, "case": label, "size": size,
                                "binary_sha256": item["binary_sha256"], "source_sha256": sha(source),
                                "stdout_sha256": sha(stdout), "stderr_sha256": sha(stderr), "exit_code": 0})
                results.append({"mode": mode, "case": label, "size": size, "stats": stats, "rows": rows})
                save()
            assert sha(binary) == item["binary_sha256"]
            print(mode, "24_COMPLETE_VALUE_ALIAS_AND_LEDGER_CHECKS_PASS", flush=True)
        assert len(records) == len(results) == 96
        assert sha(out / "builds.json") == runs["builds_sha256"]
        for name, expected in builds["source_hashes"].items():
            assert sha(study.parent / name) == expected, name
        for item in builds["preserved_binaries"]:
            assert sha(Path(item["path"])) == item["sha256"]
        runs["status"] = "PASS"
    except BaseException as error:
        runs.update(status="FAILED", error_type=type(error).__name__, error=str(error))
        raise
    finally:
        save()
    summary = {"status": "PASS", "diagnostic_only": True, "no_clock_measurements": True,
               "builds_sha256": sha(out / "builds.json"), "runs_sha256": sha(out / "runs.json"),
               "counts": {"variants": 4, "families": 6, "sizes": 4, "completed_ledger_roots": 96},
               "results": results,
               "source_hashes": builds["source_hashes"],
               "raw_evidence_hashes": {p.relative_to(out).as_posix(): sha(p) for p in sorted(out.rglob("*"))
                                       if p.is_file() and p.name != "summary.json"},
               "limitations": ["Logical events and Vec capacity slots are not allocation counts or elapsed times.",
                               "Zero general matcher entries means the general matcher was bypassed, not that no leaf matching occurred; explicit fast_leaf_hits record the retained matches.",
                               "One synchronous Replace per fresh process; no attribution across later resumption.",
                               "Finite generated full-value and alias checks; separate uninstrumented workspace tests and performance gates remain authoritative."]}
    (out / "summary.json").write_text(json.dumps(summary, indent=2) + "\n", encoding="utf-8")
    print(json.dumps({"status": "PASS", "counts": summary["counts"], "no_clock_measurements": True}))


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("evidence", type=Path)
    main(parser.parse_args().evidence.resolve())
