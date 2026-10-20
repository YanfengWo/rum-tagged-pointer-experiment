"""Audit replay provenance and every timed label, then emit a descriptive report."""
import argparse
from collections import Counter
import hashlib
import json
import math
from pathlib import Path
from statistics import median
import sys
sys.dont_write_bytecode = True
from statistics_helpers import MODES, compare, retained, rows
from context_analysis import analyze_context


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def load(path):
    return json.loads(path.read_text(encoding="utf-8"))


def analyze(out):
    study = Path(__file__).resolve().parent
    builds = load(out / "build-controls.json")
    assert builds["status"] == "PASS" and builds["baseline_source_restored"] and builds["prior_binaries_unchanged"]
    for name in ["base_replay_byte_identical", "candidate_replay_byte_identical", "base_matches_previous", "candidate_matches_previous"]:
        assert builds[name], name
    assert [b["mode"] for b in builds["builds"]] == ["base-0", "candidate-0", "base-1", "candidate-1"]
    for item in builds["builds"]:
        assert sha(Path(item["binary"])) == item["sha256"]
        assert item["source_sha256"] == builds["expected_candidate_source_sha256" if "candidate" in item["mode"] else "baseline_source_sha256"]
        assert "COPY_ATTRIBUTION_ACCEPTANCE_PASS" in (out / (item["mode"] + "-acceptance.stdout")).read_text()
        assert not (out / (item["mode"] + "-acceptance.stderr")).read_text().strip()
    assert (out / "preflight-status.txt").read_text().strip() == "PREFLIGHT_PASS"
    assert (out / "timing-status.txt").read_text().strip() == "TIMINGS_PASS"
    plan = load(out / "timing-plan.json")
    assert plan["modes"] == MODES and plan["blocks"] == len(plan["orders"]) == 8
    assert plan["reps"] == 9 and plan["scale"] == 5 and plan["retained_reps"] == 11
    assert all(Counter(o[p] for o in plan["orders"]) == Counter({m: 2 for m in MODES}) for p in range(4))
    predecessor = Counter((o[i], o[i + 1]) for o in plan["orders"] for i in range(3))
    assert len(predecessor) == 12 and set(predecessor.values()) == {2}
    assert plan["binaries"]["base-a"]["sha256"] == plan["binaries"]["base-b"]["sha256"]
    assert plan["binaries"]["candidate-a"]["sha256"] == plan["binaries"]["candidate-b"]["sha256"]
    assert plan["suite_script_sha256"] == load(study.parent / "expression-census/corpus/PIN.json")["upstream_key_source_hashes"]["scripts/core-language-microbench.sh"]
    for kind, meta in plan["sources"].items():
        assert sha(out / (kind + "-corpus.rum")) == meta["sha256"]
        assert len(meta["labels"]) == len(set(meta["labels"])) == 6
        for mode in ["base-a", "candidate-a"]:
            retained(out / f"preflight-{kind}-{mode}.stdout", meta["labels"], meta["sentinel"])
            assert not (out / f"preflight-{kind}-{mode}.stderr").read_text().strip()
    progress = rows(out / "timing-progress.tsv")
    assert len(progress) == 64
    suite, corpus, raw = {}, {k: {} for k in plan["sources"]}, {k: {} for k in plan["sources"]}
    labels, generated, iterations = None, None, {}
    for block, order in enumerate(plan["orders"], 1):
        for position, mode in enumerate(order, 1):
            states = [r for r in progress if int(r["block"]) == block and r["mode"] == mode]
            assert [r["status"] for r in states] == ["started", "passed"]
            assert all(int(r["position"]) == position for r in states)
            folder = out / f"suite-{block:02d}-{mode}"
            status = rows(folder / "runs.tsv")
            assert len(status) == 1 and status[0]["status"] == "ok" and status[0]["exit_code"] == "0"
            assert float(status[0]["elapsed_seconds"]) > 0
            assert not (folder / "raw/rum.stderr").read_text().strip()
            generated = generated or sha(folder / "core-language-microbench.rum")
            assert sha(folder / "core-language-microbench.rum") == generated
            timings = rows(folder / "timings.tsv")
            current = [r["label"] for r in timings]
            labels = labels or current
            assert current == labels and len(set(labels)) == len(labels) == 112
            for r in timings:
                value, count, label = float(r["rum_seconds"]), int(r["iterations"]), r["label"]
                assert math.isfinite(value) and value > 0 and count > 0
                assert label not in iterations or iterations[label] == count
                iterations[label] = count
                suite[block, mode, label] = value
            for kind, meta in plan["sources"].items():
                assert not (out / f"{kind}-{block:02d}-{mode}.stderr").read_text().strip()
                samples = retained(out / f"{kind}-{block:02d}-{mode}.stdout", meta["labels"], meta["sentinel"])
                for label, values in samples.items():
                    raw[kind][block, mode, label] = values
                    corpus[kind][block, mode, label] = median(values)
    assert len(suite) == 3584
    previous = load(out.parent / "copy-attribution/summary.json")
    assert generated == previous["benchmark_generated_sha256"]
    suite_results = [dict(compare(label, suite, 8), nominal_iterations=iterations[label]) for label in labels]
    corpus_results = {kind: [dict(compare(label, values, 8), raw_samples_by_block={m: [raw[kind][b, m, label] for b in range(1, 9)] for m in MODES})
                             for label in plan["sources"][kind]["labels"]] for kind, values in corpus.items()}
    context = analyze_context(out, plan)
    static = load(out / "static-comparison.json")
    assert len(static["comparisons"]) == 21
    assert static["binaries"]["base"]["binary_sha256"] == plan["binaries"]["base-a"]["sha256"]
    assert static["binaries"]["candidate"]["binary_sha256"] == plan["binaries"]["candidate-a"]["sha256"]
    slower = [r for r in suite_results if r["ratios"]["candidate/base"]["median"] > 1]
    summary = {"status": "PASS", "recorded_on": "2026-10-19", "build_replay": builds,
               "plan": plan, "benchmark_generated_sha256": generated,
               "counts": {"full_suite_processes": 32, "suite_rows": 3584, "standalone_processes": 32,
                          "pipeline_processes": 32, "retained_timing_samples": 32 * 12 * 11,
                          "followup_processes": 160, "followup_retained_samples": 1760, "total_retained_samples": 5984,
                          "slower_suite_medians": len(slower)},
               "suite_results": suite_results, **{k + "_results": v for k, v in corpus_results.items()}, "context_followup": context,
               "static_comparison": {"local_unredacted_sha256": sha(out / "static-comparison.json"), "scope": static["scope"], "limitation": static["limitation"], "comparisons": static["comparisons"]},
               "limitations": ["Eight descriptive paired blocks; repetitions within a process are not independent blocks.",
                               "No real application corpus, cross-machine validation, or whole-runtime acceleration claim.",
                               "All slower suite medians are retained. A 5% threshold does not excuse smaller regressions.",
                               "Rebuild byte reproducibility does not identify the cause of any persistent timing difference."],
               "source_hashes": {p.name: sha(p) for p in sorted(study.iterdir()) if p.is_file()},
               "raw_evidence_hashes": {p.relative_to(out).as_posix(): sha(p) for p in sorted(out.rglob("*")) if p.is_file() and p.name not in {"summary.json", "report.md"}}}
    (out / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")
    lines = ["# Rebuild and synthetic pipeline verification", "", "Elapsed candidate/base ratios below one are faster. A/A medians are controls, not corrections.", "", "| Scope | Label | candidate/base | faster blocks | base A/A | candidate A/A |", "| --- | --- | ---: | ---: | ---: | ---: |"]
    def row(r, scope):
        v = r["ratios"]
        return f'| {scope} | {r["label"]} | {v["candidate/base"]["median"]:.4f} | {v["candidate/base"]["below_one"]}/8 | {v["base-b/base-a"]["median"]:.4f} | {v["candidate-b/candidate-a"]["median"]:.4f} |'
    for scope, group in [("original suite", suite_results), ("standalone controls", corpus_results["control"]),
                         ("combined pipelines", corpus_results["pipeline"]), ("fresh case followup", context["results"])]:
        lines += [row(r, scope) for r in group]
    lines += ["", f'{len(slower)} of 112 suite medians are slower; all per-block ratios and raw retained samples are in summary.json.',
              "", "The standalone Part control retains an output-list alias; it is a distinct ownership context. Fresh-case exact-part-400-null returns Null, matching the original suite pattern. Synthetic pipelines construct fresh input inside each timed window and retain explicit input and sorted aliases.",
              "", "The followup was selected after primary results, is descriptive, and does not establish a GC/allocator or code-layout cause. Repetition counts are not independent block counts.",
              "", "No source implementation changed in this stage. The isolated safe candidate remains under review."]
    (out / "report.md").write_text("\n".join(lines) + "\n")
    print(json.dumps({"status": "PASS", "counts": summary["counts"], "key_results": [
        {"label": r["label"], **r["ratios"]["candidate/base"]} for r in suite_results if r["label"] in {"sort-default-wide", "set-part-fill-large", "replace-wide-no-match"}],
        "pipeline": [{"label": r["label"], **r["ratios"]["candidate/base"]} for r in corpus_results["pipeline"]],
        "context": [{"label": r["label"], **r["ratios"]["candidate/base"]} for r in context["results"]]}, indent=2))


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("evidence", type=Path)
    analyze(parser.parse_args().evidence.resolve())
