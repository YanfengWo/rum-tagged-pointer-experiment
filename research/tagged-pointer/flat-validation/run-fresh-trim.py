"""Post-primary diagnostic: unchanged StringTrim windows in fresh small sessions."""
import argparse
from datetime import datetime, timezone
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import time
sys.dont_write_bytecode = True


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def source(full):
    start = full.index("rumMicroScale = 5;")
    header = full[start:full.index("SetAttributes[rumMicroOnce", start)]
    start = full.index("gMicroStringTrimLiteralList4000 = StringJoin[")
    setup = full[start:full.index("fExact[1] = 2;", start)]
    start = full.index('rumMicroBench[\n  "stringtrim-literal-list-4000"')
    stop = full.index('rumMicroBench[\n  "stringreplace-single-literal-unguarded"', start)
    body = full[start:stop]
    assert body.count("rumMicroBench[") == 6 and body.count("StringLength[rumMicroSink]") == 6
    fragments = {"header": header, "setup": setup, "benchmarks": body}
    text = header + setup + body + 'Print["FRESH_STRINGTRIM_PASS"];\n'
    return text, {name: hashlib.sha256(value.encode()).hexdigest() for name, value in fragments.items()}


def main(out):
    study = Path(__file__).resolve().parent
    repo = study.parents[2]
    primary = json.loads((out / "timing-plan.json").read_text())
    assert (out / "timing-status.txt").read_text().strip() == "TIMINGS_PASS"
    assert not (out / "fresh-plan.json").exists()
    native = Path(tempfile.mkdtemp(prefix="rum-flat-context-", dir="/var/tmp"))
    template = out / "suite-01-base-a/core-language-microbench.rum"
    text, fragments = source(template.read_text())
    (native / "fresh-source.rum").write_text(text)
    labels = [f"stringtrim-{family}-{size}" for family in ["literal-list", "single-literal", "whitespace"] for size in [4000, 32000]]
    record = {"status": "RUNNING", "recorded_on": "2026-10-20", "orders": primary["orders"],
              "binaries": primary["binaries"], "cpu": primary["cpu"], "blocks": 8, "labels": labels,
              "native_evidence": str(native), "source_sha256": sha(native / "fresh-source.rum"),
              "parent_generated_sha256": sha(template), "fragment_hashes": fragments,
              "primary_plan_sha256": sha(out / "timing-plan.json"),
              "started_utc": datetime.now(timezone.utc).isoformat(),
              "method": "Post-primary diagnostic, selected after StringTrim regressions. Each mode/block starts a fresh process containing only the original six StringTrim setups and unchanged benchmark wrapper/windows: scale5, reps9, 500 inner evaluations, minimum reported. Original length checks retained; complete literal values independently checked in four untimed processes. Eight balanced mode orders and all A/A controls retained. No project builds or exports during timing.",
              "limits": "Changes session context; does not establish a unique allocator/cache/code-layout cause or supersede the complete-suite gate. Nine raw repetitions are not emitted by the unchanged wrapper; only its six minima per process are retained."}
    plan = native / "fresh-plan.json"
    plan.write_text(json.dumps(record, indent=2) + "\n")
    env = {k: v for k, v in os.environ.items() if not k.startswith("RUM_")}
    env["RUM_CORE_MICRO_RETAINED_MODULE_GC_BATCHES"] = "3"
    bounded = ["python3", str(repo / "scripts/run-bounded.py")]
    for item in record["binaries"].values():
        assert sha(Path(item["binary"])) == item["sha256"]
    print("Fresh-context native evidence:", native, flush=True)
    time.sleep(30)
    try:
        with (native / "fresh-progress.tsv").open("w", buffering=1) as journal:
            journal.write("block\tposition\tmode\tstatus\n")
            for block, order in enumerate(record["orders"], 1):
                for position, mode in enumerate(order, 1):
                    journal.write(f"{block}\t{position}\t{mode}\tstarted\n")
                    item = record["binaries"][mode]
                    result = subprocess.run(bounded + ["--timeout", "180", "--", "taskset", "-c", str(record["cpu"]),
                                                       item["binary"], "--file", str(native / "fresh-source.rum")],
                                            cwd=repo, env=env, stdin=subprocess.DEVNULL, capture_output=True, timeout=195)
                    (native / f"fresh-{block:02d}-{mode}.stdout").write_bytes(result.stdout)
                    (native / f"fresh-{block:02d}-{mode}.stderr").write_bytes(result.stderr)
                    assert result.returncode == 0 and b"FRESH_STRINGTRIM_PASS" in result.stdout and not result.stderr.strip()
                    journal.write(f"{block}\t{position}\t{mode}\tpassed\n")
                print("fresh block", block, "PASS", flush=True)
        for item in record["binaries"].values():
            assert sha(Path(item["binary"])) == item["sha256"]
        assert sha(native / "fresh-source.rum") == record["source_sha256"]
        record["status"] = "PASS"
    except BaseException as error:
        record.update(status="FAILED", error_type=type(error).__name__, error=str(error))
        raise
    finally:
        record["finished_utc"] = datetime.now(timezone.utc).isoformat()
        plan.write_text(json.dumps(record, indent=2) + "\n")
        subprocess.run(bounded + ["--timeout", "120", "--", "cp", "-a", str(native) + "/.", str(out)], check=True, timeout=135)
    print("PASS: 64 fresh sessions; 384 original-wrapper minima", flush=True)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("evidence", type=Path)
    main(parser.parse_args().evidence.resolve())
