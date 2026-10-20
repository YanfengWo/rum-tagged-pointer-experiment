"""Audit descriptive fresh-process followup; no causal allocator/GC claims."""
import hashlib
import json
from statistics import median
from statistics_helpers import MODES, compare, retained, rows


def analyze_context(out, primary):
    plan = json.loads((out / "context-plan.json").read_text(encoding="utf-8"))
    assert (out / "context-status.txt").read_text().strip() == "CONTEXT_PASS"
    assert plan["orders"] == primary["orders"] and plan["binaries"] == primary["binaries"]
    assert plan["blocks"] == 8 and plan["repetitions"] == 11 and len(plan["cases"]) == 5
    journal = rows(out / "context-progress.tsv")
    assert len(journal) == 320
    values, raw = {}, {}
    for label, meta in plan["cases"].items():
        assert hashlib.sha256((out / "context-sources" / (label + ".rum")).read_bytes()).hexdigest() == meta["sha256"]
    for block, order in enumerate(plan["orders"], 1):
        for mode in order:
            for label, meta in plan["cases"].items():
                states = [r["status"] for r in journal if r["block"] == str(block) and r["mode"] == mode and r["label"] == label]
                assert states == ["started", "passed"]
                prefix = out / f"context-{block:02d}-{mode}-{label}"
                assert not prefix.with_suffix(".stderr").read_text().strip()
                samples = retained(prefix.with_suffix(".stdout"), [label], meta["sentinel"])[label]
                raw[block, mode, label] = samples
                values[block, mode, label] = median(samples)
    results = [dict(compare(label, values, 8), raw_samples_by_block={m: [raw[b, m, label] for b in range(1, 9)] for m in MODES})
               for label in plan["cases"]]
    return {"plan": plan, "processes": 160, "retained_samples": 1760, "results": results}
