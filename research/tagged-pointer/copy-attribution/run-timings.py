"""Counter-off full-suite gate and retained focused samples, sequential only."""
import argparse
from datetime import datetime, timezone
import hashlib
import json
import os
from pathlib import Path
import platform
import subprocess

MODES = ["old-a", "fresh-base", "candidate", "old-b"]
FOCUS = [f"rewrite-{kind}-{size}" for kind in ["hit", "identity", "miss", "compound-miss"] for size in [4000, 16000]]
def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

def corpus():
    value = '''(* Focused monotonic timings; individual repetitions are retained. *)
SetAttributes[copyMeasure, HoldAll];
copyMeasure[label_String, expression_, expected_] := Module[{i, result, seconds},
  result = expression;
  If[SameQ[result, expected], Null, Exit[2]];
  Do[
    seconds = First[AbsoluteTiming[result = expression]];
    Print[StringJoin["COPY_TIME\\t", label, "\\t", ToString[i], "\\t", ToString[seconds]]],
    {i, 1, 11}
  ];
  If[SameQ[result, expected], Null, Exit[2]];
];
'''
    for size in [4000, 16000]:
        value += f"copyInput = Range[{size}]; copyAlias = copyInput;\n"
        operations = [
            ("hit", "Replace[copyInput, copyX_Integer :> copyX + 1, {1}]", f"Range[2, {size + 1}]"),
            ("identity", "Replace[copyInput, copyX_Integer :> copyX, {1}]", f"Range[{size}]"),
            ("miss", "Replace[copyInput, copyMissing -> 0, {1}]", f"Range[{size}]")]
        for kind, operation, expected in operations:
            value += f'copyMeasure["rewrite-{kind}-{size}", {operation}, {expected}];\n'
            value += f"If[SameQ[copyInput, Range[{size}]] && SameQ[copyAlias, copyInput], Null, Exit[2]];\n"
        value += f"copyCompound = Table[copyF[i, i + 1], {{i, 1, {size}}}]; copyCompoundAlias = copyCompound;\n"
        value += f'copyMeasure["rewrite-compound-miss-{size}", Replace[copyCompound, copyMissing -> 0, {{1}}], copyCompound];\n'
        value += "If[SameQ[copyCompoundAlias, copyCompound], Null, Exit[2]];\n"
    return value + 'Print["COPY_TIMING_PASS"];\n'

def main(out):
    study = Path(__file__).resolve().parent
    repo = study.parents[2]
    assert (out / "checks-status.txt").read_text().strip() == "CHECKS_PASS"
    assert json.loads((out / "matched-build.json").read_text())["status"] == "PASS"
    assert not (out / "timing-plan.json").exists()
    binaries = {"old-a": Path("/var/tmp/rum-native-dispatch-build/base/release/rum-cli"),
                "old-b": Path("/var/tmp/rum-native-dispatch-build/base/release/rum-cli"),
                "fresh-base": Path("/var/tmp/rum-copy-build/base-final/release/rum-cli"),
                "candidate": Path("/var/tmp/rum-copy-build/matched-candidate/release/rum-cli")}
    previous = out.parent / "native-dispatch"
    expected = {"old-a": (previous / "base-binary.sha256").read_text().split()[0],
                "old-b": (previous / "base-binary.sha256").read_text().split()[0],
                "fresh-base": (out / "base-final-binary.sha256").read_text().split()[0],
                "candidate": (out / "matched-candidate-binary.sha256").read_text().split()[0]}
    for mode, path in binaries.items():
        assert sha(path) == expected[mode]
    source = out / "focus-corpus.rum"
    source.write_text(corpus(), encoding="utf-8")
    script = repo / "scripts/core-language-microbench.sh"
    pin = json.loads((study.parent / "expression-census/corpus/PIN.json").read_text())
    assert sha(script) == pin["upstream_key_source_hashes"]["scripts/core-language-microbench.sh"]
    orders = [MODES[n:] + MODES[:n] for n in range(4)]
    orders += [list(reversed(order)) for order in orders.copy()]
    env = {k: v for k, v in os.environ.items() if not k.startswith("RUM_")}
    env["RUM_CORE_MICRO_RETAINED_MODULE_GC_BATCHES"] = "3"
    cpu = min(os.sched_getaffinity(0))
    plan = {"recorded_on": "2026-10-19", "modes": MODES, "blocks": 8, "orders": orders,
            "binaries": {mode: {"binary": str(path), "sha256": expected[mode]} for mode, path in binaries.items()},
            "cpu": cpu, "kernel": platform.release(), "machine": platform.machine(),
            "cpu_model": next((line.split(":", 1)[1].strip() for line in Path("/proc/cpuinfo").read_text().splitlines() if line.startswith("model name")), "unavailable"),
            "aslr": Path("/proc/sys/kernel/randomize_va_space").read_text().strip(),
            "script_sha256": sha(script), "scale": 5, "reps": 9, "focus_reps": 11,
            "focus_labels": FOCUS, "focus_source_sha256": sha(source),
            "suite_focus_labels": ["replace-wide-no-match", "replaceall-simple", "replacepart-wide-rule-list", "deletecases-wide-no-match", "cases-wide-no-match", "count-wide-no-match", "position-wide-no-match"],
            "started_utc": datetime.now(timezone.utc).isoformat(),
            "inherited_RUM_environment": "All RUM_* variables removed, then retained GC batches fixed at 3; original suite sets its own profiling-off variables.",
            "method": "Eight position-balanced paired blocks, no concurrent project build/diagnostic. Complete unchanged min9 suite plus focused corpus with one warm evaluation and eleven retained monotonic samples per label. Focus primary: within-process median then candidate/fresh-base per block. Original A/A controls and fresh-base/original-pair recorded. No equal-label whole-program average.",
            "matched_source_root": json.loads((out / "matched-build.json").read_text())["source_root"],
            "limits": "Same machine/toolchain. Baseline local crates recompiled with unchanged bytes via updated mtimes. Production candidate built at that exact source root with only the candidate patch; baseline bytes restored afterward. Third-party dependency artifacts reused. Position balance does not fully balance carryover. Eight blocks, not 88 independent focused repetitions. No general significance or no-regression theorem."}
    (out / "timing-plan.json").write_text(json.dumps(plan, indent=2) + "\n")
    bounded = ["python3", str(repo / "scripts/run-bounded.py"), "--timeout", "900", "--", "taskset", "-c", str(cpu)]
    with (out / "timing-progress.tsv").open("w", buffering=1) as journal:
        journal.write("block\tposition\tmode\tstatus\tutc\n")
        for block, order in enumerate(orders, 1):
            for position, mode in enumerate(order, 1):
                journal.write(f'{block}\t{position}\t{mode}\tstarted\t{datetime.now(timezone.utc).isoformat()}\n')
                target = out / f"timing-{block:02d}-{mode}"
                with (out / f"timing-{block:02d}-{mode}.log").open("w") as log:
                    subprocess.run(bounded + ["bash", str(script), "--rum-cli", str(binaries[mode]), "--output-dir", str(target), "--reps", "9", "--scale", "5", "--timeout", "600"], cwd=repo, env=env, stdin=subprocess.DEVNULL, stdout=log, stderr=subprocess.STDOUT, check=True)
                result = subprocess.run(bounded + [str(binaries[mode]), "--file", str(source)], cwd=repo, env=env, stdin=subprocess.DEVNULL, capture_output=True, timeout=915)
                (out / f"focus-{block:02d}-{mode}.stdout").write_bytes(result.stdout)
                (out / f"focus-{block:02d}-{mode}.stderr").write_bytes(result.stderr)
                assert result.returncode == 0 and b"COPY_TIMING_PASS" in result.stdout and not result.stderr.strip(), mode
                journal.write(f'{block}\t{position}\t{mode}\tpassed\t{datetime.now(timezone.utc).isoformat()}\n')
    for mode, path in binaries.items():
        assert sha(path) == expected[mode]
    (out / "timing-status.txt").write_text("TIMINGS_PASS\n")
    print("PASS: 32 full-suite processes and 32 focused processes; retained 2816 focused samples")

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("evidence", type=Path)
    parser.add_argument("--preflight", action="store_true")
    args = parser.parse_args()
    out = args.evidence.resolve()
    if args.preflight:
        source = out / "focus-preflight.rum"
        source.write_text(corpus(), encoding="utf-8")
        repo = Path(__file__).resolve().parents[3]
        result = subprocess.run(["python3", str(repo / "scripts/run-bounded.py"), "--timeout", "180", "--", "/var/tmp/rum-native-dispatch-build/base/release/rum-cli", "--file", str(source)], stdin=subprocess.DEVNULL, capture_output=True, timeout=195)
        (out / "focus-preflight.stdout").write_bytes(result.stdout)
        (out / "focus-preflight.stderr").write_bytes(result.stderr)
        assert result.returncode == 0 and b"COPY_TIMING_PASS" in result.stdout and not result.stderr.strip()
        print("PASS: focused corpus accepted on the original baseline; preflight values are not A/B data")
    else:
        main(out)
