"""Audit copy attribution and retain all paired counter-off timing results."""
import argparse
import json
from pathlib import Path
import sys
sys.dont_write_bytecode = True
from analysis_diagnostics import audit_diagnostics, audit_sources, read, sha
from analysis_timings import analyze_timings

def write_report(out, summary):
    lines = ["# Scoped copy attribution and safe rewrite experiment", "",
        "Exploratory rum-vs-rum evidence. Ratios below 1 mean less time. No whole-program equal-label average is computed.", "",
        "Scoped hit probes at N=64/256/1024 confirm one shallow N-slot copy removed from Replace's parent traversal; N-1 two-slot normalization copies and one into_owned copy remain. These event counts do not measure speed or total allocation bytes.", "",
        "Timing uses eight paired process blocks. Focus: eleven retained monotonic repetitions per process, reduced to one median before pairing. Original complete suite: generally min9; retained GC keeps the suite's original final-batch convention.", "",
        "| Focus label | Candidate/fresh median [p10,p90] | Faster blocks | Original A/A | Fresh/original pair | Candidate/original pair |",
        "| --- | --- | ---: | ---: | ---: | ---: |"]
    for row in summary["focus_results"]:
        r = row["ratios"]; c = r["candidate/fresh-base"]
        lines.append(f'| {row["label"]} | {c["median"]:.3f} [{c["p10"]:.3f},{c["p90"]:.3f}] | {c["below_one"]}/8 | {r["old-b/old-a"]["median"]:.3f} | {r["fresh-base/old-pair"]["median"]:.3f} | {r["candidate/old-pair"]["median"]:.3f} |')
    lines += ["", "All complete-suite labels, including regressions:", "",
        "| Label | Candidate/fresh | Original A/A | Fresh/original pair | Candidate/original pair |", "| --- | ---: | ---: | ---: | ---: |"]
    for row in summary["suite_results"]:
        lines.append("| " + row["label"] + " | " + " | ".join(f'{row["ratios"][k]["median"]:.3f}' for k in ["candidate/fresh-base", "old-b/old-a", "fresh-base/old-pair", "candidate/old-pair"]) + " |")
    lines += ["", "Limits:", ""] + ["- " + value for value in summary["limits"]]
    (out / "report.md").write_text("\n".join(lines) + "\n", encoding="utf-8")


def main(out):
    study = Path(__file__).resolve().parent
    assert read(out / "checks-status.txt").strip() == "CHECKS_PASS"
    provenance = audit_sources(out, study)
    diagnostics = audit_diagnostics(out)
    plan, generated, suite, focus = analyze_timings(out, provenance, study)
    summary = {"status": "PASS", "recorded_on": plan["recorded_on"], "checks": {
        "complete_suite_processes": 32, "focused_processes": 32, "paired_blocks": 8,
        "suite_labels": 112, "aggregate_timing_rows": 3584, "focus_labels": len(focus),
        "retained_focus_samples": len(focus) * 11 * 32, "position_balance": True,
        "diagnostic_runs": 74, "source_and_test_audit": True},
        "plan": plan, "provenance": provenance, "diagnostics": diagnostics,
        "benchmark_generated_sha256": generated, "suite_results": suite, "focus_results": focus,
        "limits": ["Eight paired process blocks, not 88 independent focused repetitions. p10/p90 describe eight observed ratios; they are not confidence intervals.",
            "Single machine/toolchain and semantic micro workloads; representative teacher applications remain unmeasured.",
            "Position-balanced rotation and reversal do not remove all within-block drift, carryover or binary-specific variance. Original A/A is not a bound on every candidate's noise.",
            "All labels are retained. Exploratory comparisons across many labels do not establish statistical significance or a general no-regression claim.",
            "Matched source-root builds reuse dependency artifacts. A fresh baseline controls rebuilding; unexplained rebuild effects cannot be attributed to a source optimization.",
            "Diagnostics use separate instrumented binaries and fresh-process controls. Shallow slot bytes omit recursive ownership work, allocator overhead and total memory traffic.",
            "Copy traces only cover the selected ownership operations. Remaining two-slot copies arise in normalization; no complete Expr clone census or hardware profile is claimed.",
            "The candidate is isolated and safe Rust; default kernel source bytes are unchanged. Passing semantic tests is not a general semantic-equivalence proof."]}
    if provenance["test_history"]["initial_failure_observed"]:
        summary["limits"].append("Initial candidate workspace failed two Jupyter pump checks; the isolated five-check retry and complete rerun passed. The initial failure's root cause is unestablished and its log is retained.")
    write_report(out, summary)
    summary["source_hashes"] = {p.relative_to(study).as_posix(): sha(p) for p in sorted(study.rglob("*")) if p.is_file() and "__pycache__" not in p.parts and p.name != "README.md"}
    summary["raw_evidence_hashes"] = {p.relative_to(out).as_posix(): sha(p) for p in sorted(out.rglob("*")) if p.is_file() and p != out / "summary.json"}
    (out / "summary.json").write_text(json.dumps(summary, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(summary["checks"], indent=2))


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("evidence", type=Path)
    main(parser.parse_args().evidence.resolve())
