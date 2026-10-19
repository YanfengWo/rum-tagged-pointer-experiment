#!/usr/bin/env python3
"""Validate semantic probes and connect scoped REPR counts to diagnostic paths."""
import hashlib
import json
from pathlib import Path
import re
import sys

FIELDS = ("into_vec_total", "into_vec_clone", "into_owned_total", "into_owned_clone",
          "shared_args_alloc", "shared_expr_alloc")


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def fields(line):
    return {k: int(v) if v.isdigit() else v for k, v in re.findall(r"(\w+)=([^ ]+)", line)}


def main():
    out = Path(sys.argv[1]).resolve()
    meta = json.loads((out / "run-metadata.json").read_text(encoding="utf-8"))
    assert meta["status"] == "PASS"
    parsed = {}
    for run in meta["runs"]:
        assert run["exit_code"] == 0
        assert sha(out / run["input"]) == run["input_sha256"]
        stdout = (out / "raw" / (run["stem"] + ".stdout")).read_text(encoding="utf-8")
        expected = f"RUM_HOTSPOT_PASS\t{run['label']}\t{run['size']}\t{run['mode']}"
        assert expected in stdout.replace("\\t", "\t"), run["stem"]
        lines = (out / "raw" / (run["stem"] + ".stderr")).read_text(encoding="utf-8").splitlines()
        repr_rows = [fields(line) for line in lines if line.startswith("rum-profile repr ")]
        assert len(repr_rows) == 2, (run["stem"], len(repr_rows))
        delta = {key: repr_rows[1][key] - repr_rows[0][key] for key in FIELDS}
        assert all(value >= 0 for value in delta.values())
        rows = [fields(line) for line in lines if line.startswith("rum-profile builtin-timing ")]
        runtime = [fields(line) for line in lines if line.startswith("rum-profile runtime-path-timing ")]
        assert rows and runtime, run["stem"]
        parsed[run["stem"]] = {"repr_before": repr_rows[0], "repr_after": repr_rows[1],
                               "repr_delta": delta, "builtin_timing": rows,
                               "runtime_path_timing": runtime}
    comparisons = []
    for run in meta["runs"]:
        if run["mode"] != "active":
            continue
        active = parsed[run["stem"]]
        control = parsed[run["stem"].removesuffix("active") + "control"]
        comparisons.append({"label": run["label"], "size": run["size"],
                            "active": active, "control": control,
                            "repr_delta_minus_control": {
                                key: active["repr_delta"][key] - control["repr_delta"][key]
                                for key in FIELDS}})
    root = Path(__file__).resolve().parent
    source_hashes = {p.relative_to(root).as_posix(): sha(p) for p in sorted(root.rglob("*"))
                     if p.is_file() and "__pycache__" not in p.parts}
    raw_hashes = {p.relative_to(out).as_posix(): sha(p) for p in sorted(out.rglob("*"))
                  if p.is_file() and p.name != "summary.json"}
    summary = {"status": "PASS", "scope": "original rum diagnostics; no optimization or speedup claim",
               "binary_sha256": meta["binary_sha256"], "baseline_commit": meta["baseline_commit"],
               "baseline_source_hashes": meta["baseline_source_hashes"],
               "source_hashes": source_hashes, "raw_evidence_hashes": raw_hashes,
               "comparisons": comparisons,
               "limitations": ["One diagnostic observation per case, not a performance sample.",
                   "REPR counters cover selected ownership/COW operations, not every Expr or Arc clone.",
                   "Same-process REPR snapshots exclude setup/parse and post-snapshot validation, but include wrapper work.",
                   "Subtracting fresh-process empty controls is approximate; operation-induced cache/state changes remain.",
                   "Builtin self time subtracts nested builtin scopes, not all evaluator work.",
                   "Builtin and runtime-path profiles overlap and cannot be added into a total.",
                   "No whole-process alloc-count, RSS, sampled stacks, or hardware cache counters collected.",
                   "These semantic micro workloads do not establish representative application behavior."]}
    (out / "summary.json").write_text(json.dumps(summary, indent=2) + "\n", encoding="utf-8")
    print(f"ANALYSIS_PASS {len(comparisons)} active/control pairs")


if __name__ == "__main__":
    main()
