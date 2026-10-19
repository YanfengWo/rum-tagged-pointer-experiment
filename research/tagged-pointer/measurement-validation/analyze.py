"""Audit balanced native remeasurement; retain every label and paired process ratio."""
import argparse
import csv
import hashlib
import json
import math
from pathlib import Path
from statistics import median

def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

def rows(path):
    with path.open(encoding="utf-8", newline="") as source:
        return list(csv.DictReader(source, delimiter="\t"))

def quantile(values, fraction):
    values = sorted(values)
    position = (len(values) - 1) * fraction
    lower = math.floor(position)
    upper = math.ceil(position)
    return values[lower] + (values[upper] - values[lower]) * (position - lower)

def describe(values):
    return {"median": median(values), "p10": quantile(values, .1),
            "p90": quantile(values, .9), "range": [min(values), max(values)],
            "below_one": sum(v < 1 for v in values), "above_one": sum(v > 1 for v in values),
            "per_block": values}

def analyze(out, previous):
    study = Path(__file__).resolve().parent
    plan = json.loads((out / "plan.json").read_text(encoding="utf-8"))
    assert (out / "status.txt").read_text(encoding="utf-8").strip() == "TIMINGS_PASS"
    assert plan["blocks"] == len(plan["orders"]) == 10
    modes = plan["modes"]
    assert len(modes) == len(set(modes)) == 5
    assert set(modes) == {"base-a", "base-b", "api", "sort-api", "sort-24"}
    assert plan["binaries"]["base-a"] == plan["binaries"]["base-b"]
    assert plan["scale"] == 5 and plan["reps"] == 9
    assert plan["focus_labels"] == ["sort-symbol-default-width4000", "sort-symbol-default-width16000", "association-part-key-read-width4000", "association-part-key-read-width32000"]
    pin = json.loads((study.parent / "expression-census/corpus/PIN.json").read_text(encoding="utf-8"))
    assert plan["script_sha256"] == pin["upstream_key_source_hashes"]["scripts/core-language-microbench.sh"]
    for position in range(5):
        assert all(sum(order[position] == mode for order in plan["orders"]) == 2 for mode in modes)
    progress = rows(out / "progress.tsv")
    assert len(progress) == 100
    values = {}; labels = None; iterations = {}; generated = None
    for block, order in enumerate(plan["orders"], 1):
        assert sorted(order) == sorted(modes)
        for position, mode in enumerate(order, 1):
            states = [r for r in progress if r["block"] == str(block) and r["mode"] == mode]
            assert [r["status"] for r in states] == ["started", "passed"]
            assert all(r["position"] == str(position) for r in states)
            folder = out / f"block-{block:02d}-{mode}"
            state = rows(folder / "runs.tsv")
            assert len(state) == 1 and state[0]["status"] == "ok" and state[0]["exit_code"] == "0"
            elapsed = float(state[0]["elapsed_seconds"])
            assert math.isfinite(elapsed) and elapsed > 0
            assert not (folder / "raw/rum.stderr").read_text(encoding="utf-8").strip()
            source = folder / "core-language-microbench.rum"
            if generated is None:
                generated = sha(source)
            assert sha(source) == generated
            text = source.read_text(encoding="utf-8")
            assert "rumMicroScale = 5;" in text and "rumMicroReps = 9;" in text
            current = rows(folder / "timings.tsv")
            names = [r["label"] for r in current]
            assert len(names) == len(set(names)) == 112
            if labels is None:
                labels = names
            assert names == labels
            for row in current:
                label = row["label"]
                count = int(row["iterations"])
                value = float(row["rum_seconds"])
                assert count > 0 and math.isfinite(value) and value > 0
                if label in iterations:
                    assert iterations[label] == count
                iterations[label] = count
                values[block, mode, label] = value
    old = json.loads((previous / "summary.json").read_text(encoding="utf-8"))
    assert generated == old["benchmark_generated_sha256"]
    for mode in modes:
        original = "base" if mode.startswith("base-") else mode
        assert plan["binaries"][mode]["sha256"] == (previous / f"{original}-binary.sha256").read_text(encoding="utf-8").split()[0]
    results = []
    def stratum(ratios, predicate):
        indices = [b for b, order in enumerate(plan["orders"]) if predicate(order)]
        result = describe([ratios[b] for b in indices])
        result["block_ids"] = [b + 1 for b in indices]
        return result
    for label in labels:
        aa = [values[b, "base-b", label] / values[b, "base-a", label] for b in range(1, 11)]
        noise = quantile([abs(math.log(v)) for v in aa], .9)
        pairs = {}; screens = {}; strata = {}
        for mode in ["api", "sort-api", "sort-24"]:
            ratios = [values[b, mode, label] / math.sqrt(values[b, "base-a", label] * values[b, "base-b", label]) for b in range(1, 11)]
            stats = describe(ratios)
            pairs[mode + "/base-pair"] = stats
            for control in ["api", "sort-api"]:
                pairs[mode + "/" + control] = describe([values[b, mode, label] / values[b, control, label] for b in range(1, 11)])
            m = stats["median"]
            direction = "faster" if m < 1 else "slower"
            same = stats["below_one"] if m < 1 else stats["above_one"]
            screens[mode] = {"direction": direction, "repeatable_direction_screen": same >= 8 and abs(m - 1) >= .05 and abs(math.log(m)) > noise}
            strata[mode] = {"positions": {str(p + 1): stratum(ratios, lambda order: order.index(mode) == p) for p in range(5)},
                            "base_b_after_a": stratum(ratios, lambda order: order.index("base-b") > order.index("base-a")),
                            "base_b_before_a": stratum(ratios, lambda order: order.index("base-b") < order.index("base-a"))}
        results.append({"label": label, "focus": label in plan["focus_labels"], "nominal_iterations": iterations[label],
                        "base_b_over_a": describe(aa), "aa_absolute_log_p90": noise,
                        "seconds_by_block": {mode: [values[b, mode, label] for b in range(1, 11)] for mode in modes},
                        "ratios": pairs, "strata": strata, "screens": screens})
    summary = {"status": "PASS", "recorded_on": "2026-10-19", "checks": {"process_runs": 50, "aa_pairs": 10, "labels": 112, "aggregate_timing_rows": len(values), "identical_prior_binaries": True, "identical_generated_source": True, "position_balance": True},
               "plan": plan, "benchmark_generated_sha256": generated,
               "limits": ["Ten paired blocks, not ninety independent repetitions; most script measurements are min9, retained GC uses the original final-batch convention.",
                          "Position-balanced rotation and reversal do not balance all treatment carryover pairs or remove arbitrary within-block drift.",
                          "Each position stratum contains only two blocks; filtered per_block values are mapped by explicit block_ids and do not support strong position-specific conclusions.",
                          "All modes inherit one common parent environment. Non-profile RUM variables were not individually recorded; do not infer equality with the earlier session.",
                          "Base A/A measures base variability; it does not bound all candidate-specific variance or establish a code-generation cause.",
                          "Screen is exploratory and does not perform multiplicity correction across 112 labels; no statistical significance or general no-regression claim.",
                          "Wall-clock engine elapsed is used for successful-run diagnostics only. Decisions use the original monotonic in-language timing rows.",
                          "Same machine and old binaries; independent builds, hardware and representative user applications are not covered."],
               "results": results, "source_hashes": {p.name: sha(p) for p in sorted(study.glob("*.py"))}}
    lines = ["# Balanced A/A and native Sort replication", "", "Time ratios below 1 are faster. Each result has ten paired process blocks. The original suite generally takes min9; no whole-application average is computed.", "",
             "| Label | A/A median [range] | api/base | sort-api/base | sort24/base [p10,p90] | sort24/sort-api | Sort wins |", "| --- | --- | ---: | ---: | --- | ---: | ---: |"]
    for row in results:
        if not row["focus"]:
            continue
        aa = row["base_b_over_a"]; ratios = row["ratios"]; sort = ratios["sort-24/base-pair"]
        lines.append(f'| {row["label"]} | {aa["median"]:.3f} [{aa["range"][0]:.3f},{aa["range"][1]:.3f}] | {ratios["api/base-pair"]["median"]:.3f} | {ratios["sort-api/base-pair"]["median"]:.3f} | {sort["median"]:.3f} [{sort["p10"]:.3f},{sort["p90"]:.3f}] | {ratios["sort-24/sort-api"]["median"]:.3f} | {sort["below_one"]}/10 |')
    lines += ["", "All labels (exploratory outside predeclared focus):", "", "| Label | A/A | api/base | sort-api/base | sort24/base |", "| --- | ---: | ---: | ---: | ---: |"]
    for row in results:
        lines.append("| " + row["label"] + " | " + f'{row["base_b_over_a"]["median"]:.3f}' + " | " + " | ".join(f'{row["ratios"][m+"/base-pair"]["median"]:.3f}' for m in ["api", "sort-api", "sort-24"]) + " |")
    lines += ["", "Limits:", ""] + ["- " + value for value in summary["limits"]]
    (out / "report.md").write_text("\n".join(lines) + "\n", encoding="utf-8")
    summary["raw_evidence_hashes"] = {p.relative_to(out).as_posix(): sha(p) for p in sorted(out.rglob("*")) if p.is_file() and p.name != "summary.json"}
    (out / "summary.json").write_text(json.dumps(summary, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(summary["checks"], indent=2))

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("evidence", type=Path)
    parser.add_argument("previous", type=Path)
    args = parser.parse_args()
    analyze(args.evidence.resolve(), args.previous.resolve())
