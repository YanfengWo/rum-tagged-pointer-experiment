#!/usr/bin/env python3
"""Prepare retained inputs, optionally execute serial baseline diagnostics."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys

sys.dont_write_bytecode = True
from workloads import LABELS, SIZES, program


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output-dir", type=Path, required=True)
    parser.add_argument("--rum-cli", type=Path)
    parser.add_argument("--base-source", type=Path)
    parser.add_argument("--prepare-only", action="store_true")
    parser.add_argument("--case-timeout", type=int, default=60)
    args = parser.parse_args()
    if not args.prepare_only and (not args.rum_cli or not args.base_source):
        parser.error("execution requires --rum-cli and --base-source")
    out = args.output_dir.resolve()
    out.mkdir(parents=True, exist_ok=True)
    runs = []
    for label in LABELS:
        for size in SIZES:
            for mode in ("control", "active"):
                stem = f"{label}-{size}-{mode}"
                source = out / "inputs" / (stem + ".rum")
                source.parent.mkdir(exist_ok=True)
                source.write_text(program(label, size, mode == "control"), encoding="utf-8")
                runs.append({"label": label, "size": size, "mode": mode,
                             "stem": stem, "input": source.relative_to(out).as_posix(),
                             "input_sha256": sha(source)})
    if args.prepare_only:
        print(f"PREPARED {len(runs)} inputs")
        return
    binary = args.rum_cli.resolve()
    source = args.base_source.resolve()
    source_paths = ("crates/rum-core/src/expr.rs", "crates/rum-core/src/profiling.rs",
                    "crates/rum-runtime/src/runtime/profiling/timing.rs",
                    "crates/rum-runtime/src/runtime/profiling/summary.rs",
                    "crates/rum-runtime/src/runtime/profiling/reset.rs",
                    "crates/rum-cli/src/main.rs", "scripts/core-language-microbench.sh")
    metadata = {
        "status": "RUNNING", "baseline_commit": "77029938f999b14dbe5b0755bebdf98e51517ae6",
        "binary_path": str(binary), "binary_sha256": sha(binary),
        "baseline_source_hashes": {p: sha(source / p) for p in source_paths},
        "environment": {"RUM_PROFILE_REPR": "1", "RUM_PROFILE_LEXICAL_SUBSTITUTION": "1",
                        "RUM_PROFILE_LEXICAL_DEFER": "1", "RUM_PROFILE_BUILTIN_TIMING": "1"},
        "sizes": list(SIZES), "case_timeout_seconds": args.case_timeout,
        "timing_is_diagnostic_only": True, "alloc_count_collected": False, "runs": runs,
    }
    meta = out / "run-metadata.json"
    env = {k: v for k, v in os.environ.items() if not k.startswith("RUM_PROFILE_")}
    env.update(metadata["environment"])
    env["PYTHONDONTWRITEBYTECODE"] = "1"
    raw = out / "raw"
    raw.mkdir(exist_ok=True)
    for run in runs:
        meta.write_text(json.dumps(metadata, indent=2) + "\n", encoding="utf-8")
        command = [str(binary), "--file", str(out / run["input"])]
        try:
            result = subprocess.run(command, env=env, stdin=subprocess.DEVNULL,
                                    capture_output=True, timeout=args.case_timeout)
            code, stdout, stderr = result.returncode, result.stdout, result.stderr
        except subprocess.TimeoutExpired as exc:
            code, stdout, stderr = 124, exc.stdout or b"", exc.stderr or b""
            stderr += b"\nHOTSPOT_CASE_TIMEOUT\n"
        for suffix, data in (("stdout", stdout), ("stderr", stderr)):
            (raw / (run["stem"] + "." + suffix)).write_bytes(data)
        run["exit_code"] = code
        if code:
            metadata["status"] = "FAIL"
            meta.write_text(json.dumps(metadata, indent=2) + "\n", encoding="utf-8")
            raise RuntimeError(f"{run['stem']} exited {code}")
        print("PASS", run["stem"], flush=True)
    metadata["status"] = "PASS"
    meta.write_text(json.dumps(metadata, indent=2) + "\n", encoding="utf-8")
    subprocess.run([sys.executable, str(Path(__file__).with_name("analyze.py")), str(out)], check=True)


if __name__ == "__main__":
    main()
