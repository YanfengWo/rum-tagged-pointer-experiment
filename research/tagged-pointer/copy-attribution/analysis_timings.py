"""Validate balanced suite/focus timings and preserve all paired samples."""
import csv
from fractions import Fraction
import math
from statistics import median
from analysis_diagnostics import load, read, sha

MODES = ["old-a", "fresh-base", "candidate", "old-b"]


def rows(path):
    with path.open(encoding="utf-8", newline="") as source:
        return list(csv.DictReader(source, delimiter="\t"))


def quantile(values, fraction):
    values = sorted(values)
    position = (len(values) - 1) * fraction
    lower, upper = math.floor(position), math.ceil(position)
    return values[lower] + (values[upper] - values[lower]) * (position - lower)


def describe(values):
    assert values and all(math.isfinite(v) and v > 0 for v in values)
    return {"median": median(values), "p10": quantile(values, .1), "p90": quantile(values, .9),
            "range": [min(values), max(values)], "below_one": sum(v < 1 for v in values),
            "above_one": sum(v > 1 for v in values), "per_block": values}


def compare(label, values, extra):
    ratios = {"candidate/fresh-base": [], "old-b/old-a": [],
              "fresh-base/old-pair": [], "candidate/old-pair": []}
    for block in range(1, 9):
        old_pair = math.sqrt(values[block, "old-a", label] * values[block, "old-b", label])
        ratios["candidate/fresh-base"].append(values[block, "candidate", label] / values[block, "fresh-base", label])
        ratios["old-b/old-a"].append(values[block, "old-b", label] / values[block, "old-a", label])
        for mode in ["fresh-base", "candidate"]:
            ratios[mode + "/old-pair"].append(values[block, mode, label] / old_pair)
    return {"label": label, "ratios": {k: describe(v) for k, v in ratios.items()},
            "seconds_by_block": {mode: [values[b, mode, label] for b in range(1, 9)] for mode in MODES}, **extra}


def focus_samples(path, labels):
    samples = {label: {} for label in labels}
    text = read(path).replace("\\t", "\t")
    assert "COPY_TIMING_PASS" in text
    for line in text.splitlines():
        line = line.strip().strip('"')
        if not line.startswith("COPY_TIME\t"):
            continue
        _, label, rep, value = line.split("\t")
        rep, value = int(rep), float(Fraction(value))
        assert label in samples and rep not in samples[label]
        assert math.isfinite(value) and value > 0
        samples[label][rep] = value
    assert all(set(group) == set(range(1, 12)) for group in samples.values())
    return {label: [group[n] for n in range(1, 12)] for label, group in samples.items()}


def analyze_timings(out, provenance, study):
    plan = load(out / "timing-plan.json")
    assert read(out / "timing-status.txt").strip() == "TIMINGS_PASS"
    assert plan["modes"] == MODES and plan["blocks"] == len(plan["orders"]) == 8
    assert plan["scale"] == 5 and plan["reps"] == 9 and plan["focus_reps"] == 11
    assert plan["binaries"]["old-a"] == plan["binaries"]["old-b"]
    expected = [MODES[n:] + MODES[:n] for n in range(4)]
    expected += [list(reversed(order)) for order in expected.copy()]
    assert plan["orders"] == expected
    for position in range(4):
        assert all(sum(order[position] == mode for order in plan["orders"]) == 2 for mode in MODES)
    pin = load(study.parent / "expression-census/corpus/PIN.json")
    assert plan["script_sha256"] == pin["upstream_key_source_hashes"]["scripts/core-language-microbench.sh"]
    assert plan["focus_source_sha256"] == sha(out / "focus-corpus.rum")
    assert plan["matched_source_root"] == provenance["matched_build"]["source_root"]
    previous = out.parent / "native-dispatch"
    for mode, name, folder in [("old-a", "base", previous), ("old-b", "base", previous),
                               ("fresh-base", "base-final", out), ("candidate", "matched-candidate", out)]:
        assert plan["binaries"][mode]["sha256"] == read(folder / (name + "-binary.sha256")).split()[0]
    trace_hashes = {read(out / (name + "-binary.sha256")).split()[0] for name in ["trace-final", "candidate-trace"]}
    assert trace_hashes.isdisjoint(b["sha256"] for b in plan["binaries"].values())
    progress = rows(out / "timing-progress.tsv")
    assert len(progress) == 64
    suite, focus, raw_focus, iterations = {}, {}, {}, {}
    labels = generated = None
    focus_labels = plan["focus_labels"]
    assert len(focus_labels) == len(set(focus_labels)) == 8
    assert set(focus_labels) == {f"rewrite-{kind}-{size}" for kind in ["hit", "identity", "miss", "compound-miss"] for size in [4000, 16000]}
    for block, order in enumerate(plan["orders"], 1):
        for position, mode in enumerate(order, 1):
            states = [r for r in progress if r["block"] == str(block) and r["mode"] == mode]
            assert [r["status"] for r in states] == ["started", "passed"]
            assert all(r["position"] == str(position) for r in states)
            folder = out / f"timing-{block:02d}-{mode}"
            state = rows(folder / "runs.tsv")
            assert len(state) == 1 and state[0]["status"] == "ok" and state[0]["exit_code"] == "0"
            elapsed = float(state[0]["elapsed_seconds"])
            assert math.isfinite(elapsed) and elapsed > 0
            assert not read(folder / "raw/rum.stderr").strip()
            source = folder / "core-language-microbench.rum"
            generated = generated or sha(source)
            assert sha(source) == generated
            assert "rumMicroScale = 5;" in read(source) and "rumMicroReps = 9;" in read(source)
            current = rows(folder / "timings.tsv")
            names = [r["label"] for r in current]
            assert len(names) == len(set(names)) == 112
            labels = labels or names
            assert names == labels
            for row in current:
                label, count, value = row["label"], int(row["iterations"]), float(row["rum_seconds"])
                assert count > 0 and math.isfinite(value) and value > 0
                assert label not in iterations or iterations[label] == count
                iterations[label] = count
                suite[block, mode, label] = value
            assert not read(out / f"focus-{block:02d}-{mode}.stderr").strip()
            samples = focus_samples(out / f"focus-{block:02d}-{mode}.stdout", focus_labels)
            for label, values in samples.items():
                raw_focus[block, mode, label] = values
                focus[block, mode, label] = median(values)
    assert generated == load(previous / "summary.json")["benchmark_generated_sha256"]
    assert len(suite) == 3584
    assert set(plan["suite_focus_labels"]).issubset(labels)
    suite_results = [compare(label, suite, {"nominal_iterations": iterations[label], "focus": label in plan["suite_focus_labels"]}) for label in labels]
    focus_results = [compare(label, focus, {"raw_samples_by_block": {mode: [raw_focus[b, mode, label] for b in range(1, 9)] for mode in MODES}}) for label in focus_labels]
    return plan, generated, suite_results, focus_results
