"""Follow up primary anomalies with each case in its own fresh process."""
import argparse
from datetime import datetime, timezone
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
sys.dont_write_bytecode = True
import pipeline_cases
from statistics_helpers import MODES, retained


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def standalone(label, setup, expression, expected, check):
    return f'''{setup}
ctxResult = {expression};
If[SameQ[ctxResult, {expected}] && ({check}), Null, Exit[2]];
Do[
  ctxSeconds = First[AbsoluteTiming[ctxResult = {expression}]];
  Print[StringJoin["ISO_TIME\\t", "{label}", "\\t", ToString[ctxI], "\\t", ToString[ctxSeconds]]];
  If[SameQ[ctxResult, {expected}] && ({check}), Null, Exit[2]],
  {{ctxI, 1, 11}}
];
Print["ISO_CONTEXT_PASS"];
'''


def cases():
    result = {}
    label = "exact-part-400-null"
    result[label] = (standalone(label, "ctxPart = Table[0, {400}];", "Do[ctxPart[[ctxJ]] = ctxJ, {ctxJ, 1, 400}]",
                                "Null", "SameQ[ctxPart, Range[400]]"), "ISO_CONTEXT_PASS")
    label = "exact-sort-8000"
    result[label] = (standalone(label, "ctxInput = Reverse[Range[8000]]; ctxAlias = ctxInput;", "Sort[ctxInput]",
                                "Range[8000]", "SameQ[ctxInput, Reverse[Range[8000]]] && SameQ[ctxInput, ctxAlias]"), "ISO_CONTEXT_PASS")
    label = "fresh-rewrite-hit-16000"
    result[label] = (standalone(label, "ctxInput = Range[16000]; ctxAlias = ctxInput;", "Replace[ctxInput, ctxX_Integer :> ctxX + 1, {1}]",
                                "Range[2, 16001]", "SameQ[ctxInput, Range[16000]] && SameQ[ctxInput, ctxAlias]"), "ISO_CONTEXT_PASS")
    for case in pipeline_cases.CASE_DEFINITIONS:
        if case["size"] == 16000 and case["family"] in {"rewrite-heavy-hit", "mixed-control"}:
            result[case["label"]] = (pipeline_cases.make_source([case]), pipeline_cases.PASS_MARKER)
    assert len(result) == 5
    return result


def main(out):
    study = Path(__file__).resolve().parent
    repo = study.parents[2]
    primary = json.loads((out / "timing-plan.json").read_text())
    assert (out / "timing-status.txt").read_text().strip() == "TIMINGS_PASS"
    assert not (out / "context-plan.json").exists()
    env = {k: v for k, v in os.environ.items() if not k.startswith("RUM_")}
    env["RUM_CORE_MICRO_RETAINED_MODULE_GC_BATCHES"] = "3"
    generated = cases()
    source_dir = out / "context-sources"
    source_dir.mkdir()
    for label, (source, sentinel) in generated.items():
        (source_dir / (label + ".rum")).write_text(source)
    plan = {"followup_reason": "Primary original-suite Part/Sort regressions persist; standalone Part retained a result alias unlike the suite, and synthetic mixed-control16k showed large same-binary A/A variation. Isolate session context rather than change implementation.",
            "orders": primary["orders"], "binaries": primary["binaries"], "cpu": primary["cpu"],
            "repetitions": 11, "blocks": 8,
            "cases": {label: {"sha256": sha(source_dir / (label + ".rum")), "sentinel": sentinel} for label, (_, sentinel) in generated.items()},
            "started_utc": datetime.now(timezone.utc).isoformat(),
            "method": "Each mode/case is a fresh process, warm once then11 retained timings. Full expected results and aliases checked outside each window. Part returns Null and validates contents separately, matching the suite ownership pattern. Fresh pipelines use exactly the existing per-case generator. Eight balanced mode blocks with both A/A controls; descriptive followup selected after seeing primary outcomes, not a confirmatory independent trial."}
    (out / "context-plan.json").write_text(json.dumps(plan, indent=2) + "\n")
    bounded = ["python3", str(repo / "scripts/run-bounded.py"), "--timeout", "180", "--", "taskset", "-c", str(primary["cpu"])]
    with (out / "context-progress.tsv").open("w", buffering=1) as journal:
        journal.write("block\tmode\tlabel\tstatus\n")
        for block, order in enumerate(primary["orders"], 1):
            for mode in order:
                item = primary["binaries"][mode]
                assert sha(Path(item["binary"])) == item["sha256"]
                for label, (_, sentinel) in generated.items():
                    journal.write(f"{block}\t{mode}\t{label}\tstarted\n")
                    result = subprocess.run(bounded + [item["binary"], "--file", str(source_dir / (label + ".rum"))],
                                            cwd=repo, env=env, stdin=subprocess.DEVNULL, capture_output=True, timeout=195)
                    prefix = out / f"context-{block:02d}-{mode}-{label}"
                    prefix.with_suffix(".stdout").write_bytes(result.stdout)
                    prefix.with_suffix(".stderr").write_bytes(result.stderr)
                    assert result.returncode == 0 and sentinel.encode() in result.stdout and not result.stderr.strip()
                    retained(prefix.with_suffix(".stdout"), [label], sentinel)
                    journal.write(f"{block}\t{mode}\t{label}\tpassed\n")
            print("context block", block, "PASS", flush=True)
    (out / "context-status.txt").write_text("CONTEXT_PASS\n")
    print("PASS:160 fresh case processes,1760 retained timing samples")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("evidence", type=Path)
    main(parser.parse_args().evidence.resolve())
