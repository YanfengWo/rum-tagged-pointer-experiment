"""Audit uninstrumented four-implementation comparisons and every original label."""
import argparse
import ast
from collections import Counter
import hashlib
import json
import math
from pathlib import Path
from statistics import median
import sys
sys.dont_write_bytecode = True
from statistics_helpers import COMPARISONS, IMPLEMENTATIONS, compare, retained, rows
from build_audit import audit as audit_builds
from session_audit import audit as audit_session


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def load(path):
    return json.loads(path.read_text(encoding="utf-8"))


def analyze(out):
    study = Path(__file__).resolve().parent
    for path in study.glob("*.py"):
        ast.parse(path.read_text(encoding="utf-8"), filename=path.name)
        assert path.stat().st_size < 10000, path.name
    builds, variants, checks = audit_builds(out)
    native = load(out / "native-execution.json")
    assert native["status"] == "PASS" and native["idle_seconds_before_timing"] == 30
    assert (out / "preflight-status.txt").read_text().strip() == "PREFLIGHT_PASS"
    assert (out / "timing-status.txt").read_text().strip() == "TIMINGS_PASS"
    plan = load(out / "timing-plan.json")
    assert plan["builds_sha256"] == sha(out / "builds.json")
    assert plan["semantic_source_sha256"] == sha(out / "semantic-corpus.rum")
    preflight = load(out / "preflight-record.json")
    assert preflight["builds_sha256"] == sha(out / "builds.json")
    assert preflight["sources"] == {p.name: sha(p) for p in out.glob("*-corpus.rum")}
    modes, orders, blocks = plan["modes"], plan["orders"], plan["blocks"]
    assert blocks == len(orders) == 8 and len(set(modes)) == len(modes) == 8
    assert set(modes) == {name + "-" + copy for name in IMPLEMENTATIONS for copy in ["a", "b"]}
    assert all(Counter(order[p] for order in orders) == Counter(modes) for p in range(8))
    predecessors = Counter((order[i], order[i + 1]) for order in orders for i in range(7))
    assert len(predecessors) == 56 and set(predecessors.values()) == {1}
    assert plan["scale"] == 5 and plan["reps"] == 9 and plan["retained_reps"] == 11
    pin = load(study.parent / "expression-census/corpus/PIN.json")
    assert plan["suite_script_sha256"] == pin["upstream_key_source_hashes"]["scripts/core-language-microbench.sh"]
    for mode, item in plan["binaries"].items():
        assert sha(Path(item["binary"])) == item["sha256"] == variants[mode.rsplit("-", 1)[0]]["binary_sha256"]
    for kind, meta in plan["sources"].items():
        assert sha(out / (kind + "-corpus.rum")) == meta["sha256"]
        assert sha(out / (kind + "-preflight-corpus.rum")) == meta["preflight_sha256"]
        assert len(meta["labels"]) == len(set(meta["labels"]))
        for name in IMPLEMENTATIONS:
            assert meta["sentinel"] in (out / f"preflight-{kind}-{name}.stdout").read_text()
            assert not (out / f"preflight-{kind}-{name}.stderr").read_text().strip()
    for name in IMPLEMENTATIONS:
        assert "ISO_SEMANTICS_PASS" in (out / f"preflight-semantic-{name}.stdout").read_text()
        assert not (out / f"preflight-semantic-{name}.stderr").read_text().strip()
    progress = rows(out / "timing-progress.tsv")
    assert len(progress) == 128
    suite, corpora = {}, {kind: {} for kind in plan["sources"]}
    raw = {kind: {} for kind in plan["sources"]}
    labels, generated, iterations = None, None, {}
    for block, order in enumerate(orders, 1):
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
            for row in timings:
                value, count, label = float(row["rum_seconds"]), int(row["iterations"]), row["label"]
                assert math.isfinite(value) and value > 0 and count > 0
                assert label not in iterations or iterations[label] == count
                iterations[label] = count
                suite[block, mode, label] = value
            for kind, meta in plan["sources"].items():
                assert not (out / f"{kind}-{block:02d}-{mode}.stderr").read_text().strip()
                samples = retained(out / f"{kind}-{block:02d}-{mode}.stdout", meta["labels"], meta["sentinel"])
                for label, values in samples.items():
                    raw[kind][block, mode, label] = values
                    corpora[kind][block, mode, label] = median(values)
    assert len(suite) == 7168
    assert generated == load(out.parent / "regression-isolation/summary.json")["benchmark_generated_sha256"]
    suite_results = [dict(compare(label, suite, blocks, modes), nominal_iterations=iterations[label]) for label in labels]
    corpus_results = {kind: [dict(compare(label, data, blocks, modes),
        raw_samples_by_block={mode: [raw[kind][b, mode, label] for b in range(1, blocks + 1)] for mode in modes})
        for label in plan["sources"][kind]["labels"]] for kind, data in corpora.items()}
    slower = {n + "/" + d: [r["label"] for r in suite_results if r["ratios"][n + "/" + d]["median"] > 1]
              for n, d in COMPARISONS}
    limits = ["One nominally idle desktop, same WSL host and toolchain; external background work is not controlled completely.",
              "Eight paired process blocks; retained in-process repetitions are not independent experiment blocks.",
              "Every slower original-suite median is retained; A/A ratios are controls and are never subtracted.",
              "Same-binary aliases check timing noise, not independent compilation reproducibility.",
              "Synthetic focus/pipeline programs do not establish a real application distribution or whole-runtime speedup.",
              "Value, alias, continuation, trace and selected resource-limit checks are finite evidence; this is not full contextual equivalence."]
    diagnostic = load(out / "diagnostic/summary.json")
    assert diagnostic["status"] == "PASS" and diagnostic["no_clock_measurements"]
    assert diagnostic["counts"]["completed_ledger_roots"] == 96
    for path, digest in diagnostic["raw_evidence_hashes"].items():
        assert sha(out / "diagnostic" / path) == digest
    for path, digest in diagnostic["source_hashes"].items():
        assert sha(study.parent / path) == digest
    summary = {"status": "PASS", "recorded_on": "2026-10-20", "builds": builds, "workspace": checks,
               "diagnostic_summary_sha256": sha(out / "diagnostic/summary.json"), "session": audit_session(out), "plan": plan, "native_execution": native, "benchmark_generated_sha256": generated, "suite_results": suite_results,
               "corpus_results": corpus_results, "slower_suite_medians": slower,
               "counts": {"full_suite_processes": 64, "suite_rows": 7168,
                          "retained_corpus_processes": 64 * len(corpora),
                          "retained_samples": 64 * 11 * sum(len(meta["labels"]) for meta in plan["sources"].values())},
               "default_integration_selected": False, "limitations": limits,
               "source_hashes": {p.name: sha(p) for p in sorted(study.iterdir()) if p.is_file()},
               "raw_evidence_hashes": {p.relative_to(out).as_posix(): sha(p) for p in sorted(out.rglob("*"))
                                       if p.is_file() and p not in {out / "summary.json", out / "report.md"}}}
    (out / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")
    lines = ["# Safe rewrite shortcut validation", "", "Elapsed ratios below one are faster. Every original label and all A/A controls are retained."]
    for scope, results in [("Original suite", suite_results), *corpus_results.items()]:
        lines += ["", "## " + scope, "", "| Label | Matcher/base | Depth/base | Combined/base | Combined/matcher | Combined/depth | base A/A | matcher A/A | depth A/A | combined A/A |", "| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |"]
        for result in results:
            ratios = result["ratios"]
            keys = [n + "/" + d for n, d in COMPARISONS] + [n + "-b/" + n + "-a" for n in IMPLEMENTATIONS]
            lines.append("| " + result["label"] + " | " + " | ".join(f'{ratios[key]["median"]:.4f}' for key in keys) + " |")
    lines += ["", "All slower original-suite medians (including very small differences):"]
    lines += ["", json.dumps(slower, indent=2), "", *limits]
    (out / "report.md").write_text("\n".join(lines) + "\n")
    print(json.dumps({"status": "PASS", "counts": summary["counts"],
                      "slower_suite_counts": {key: len(value) for key, value in slower.items()}}, indent=2))


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("evidence", type=Path)
    analyze(parser.parse_args().evidence.resolve())
