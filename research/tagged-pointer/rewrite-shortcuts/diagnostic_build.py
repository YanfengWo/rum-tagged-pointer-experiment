"""Diagnostic builds; no timing."""
import argparse
import difflib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
sys.dont_write_bytecode = True
from build_support import digest, files, sha

MODES = ("base", "matcher", "depth", "combined")
PIN = "77029938f999b14dbe5b0755bebdf98e51517ae6"
ONCE = "crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/once.rs"
APPLICATION = "crates/rum-runtime/src/runtime/definitions/application/replace_all.rs"
LEDGER = "rum_core::rewrite_work_ledger"

def replace_one(path, old, new):
    text = path.read_text(encoding="utf-8")
    assert text.count(old) == 1, (path, old)
    path.write_text(text.replace(old, new, 1), encoding="utf-8", newline="\n")


def difference(before, after):
    return "".join("".join(difflib.unified_diff(
        before.get(n, b"").decode().splitlines(keepends=True),
        after[n].decode().splitlines(keepends=True), fromfile="a/" + n, tofile="b/" + n))
        for n in sorted(after) if before.get(n) != after[n])

def main(out):
    study = Path(__file__).resolve().parent
    repo, legacy = study.parents[2], study.parent / "rewrite-work-ledger"
    sys.path.insert(0, str(legacy))
    import patch_core
    import patch_runtime
    assert not out.exists() or not any(out.iterdir())
    out.mkdir(parents=True, exist_ok=True)
    source = Path(tempfile.mkdtemp(prefix="rum-shortcut-ledger-source-", dir="/var/tmp"))
    target = Path(tempfile.mkdtemp(prefix="rum-shortcut-ledger-target-", dir="/var/tmp"))
    cache = Path("/var/tmp/rum-copy-build/base-final")
    preserved = {str(p): sha(p) for p in (
        cache / "release/rum-cli",
        Path("/var/tmp/rum-copy-build/matched-candidate/release/rum-cli"),
        Path("/var/tmp/rum-copy-build/candidate/release/rum-cli"))}
    default = {name: digest(data) for name, data in files(repo / "crates").items()}
    env = {k: v for k, v in os.environ.items() if not k.startswith("RUM_")}
    env.update(CARGO_TARGET_DIR=str(target), CARGO_BUILD_JOBS="2", CARGO_PROFILE_DEV_DEBUG="0")
    bounded = ["python3", str(repo / "scripts/run-bounded.py")]
    inputs = [Path(__file__).resolve(), study / "diagnostic_run.py", study / "build_support.py",
              study / "matcher.patch", study / "depth.patch",
              study.parent / "copy-attribution/prepare.py",
              study.parent / "copy-attribution/regression-tests.rs"]
    inputs += [legacy / name for name in ("patch_core.py", "patch_runtime.py", "ledger.rs",
                                         "workloads.py", "parse_ledger.py")]
    record = {"status": "RUNNING", "pin": PIN, "source_root": str(source),
              "target_root": str(target), "toolchain": "1.85.0", "jobs": 2,
              "diagnostic_only": True, "no_clock_measurements": True,
              "source_hashes": {str(p.relative_to(study.parent)): sha(p) for p in inputs},
              "preserved_binaries": [{"path": p, "sha256": h} for p, h in preserved.items()],
              "variants": [], "default_checkout_changed": False}
    diagnostic_base = None

    def save():
        (out / "builds.json").write_text(json.dumps(record, indent=2) + "\n", encoding="utf-8")

    def run(command, seconds, name):
        record["stage"] = name
        save()
        with (out / (name + ".log")).open("w", encoding="utf-8") as log:
            subprocess.run(bounded + ["--timeout", str(seconds), "--", *command],
                           cwd=source, env=env, stdin=subprocess.DEVNULL,
                           stdout=log, stderr=subprocess.STDOUT, check=True, timeout=seconds + 15)

    def restore(snapshot):
        assert set(files(source)) == set(snapshot), "Source entries"
        for name, data in snapshot.items():
            if (source / name).read_bytes() != data:
                (source / name).write_bytes(data)

    save()
    try:
        required = sum(p.stat().st_size for p in cache.rglob("*") if p.is_file())
        assert shutil.disk_usage(target).free > required + 1024 ** 3
        run(["python3", str(study.parent / "copy-attribution/prepare.py"),
             "--output", str(source), "--metadata", str(out / "base-overlay.json"),
             "--mode", "base"], 300, "prepare")
        assert json.loads((out / "base-overlay.json").read_text())["upstream_commit"] == PIN
        original = files(source)
        assert not any(b"rewrite_work_ledger" in d for n, d in original.items() if n.endswith(".rs"))
        functional_bases = {}
        for mode in MODES:
            restore(original)
            for kind in ("matcher", "depth"):
                if mode in {kind, "combined"}:
                    run(["git", "apply", "--check", str(study / (kind + ".patch"))], 30, mode + "-raw-check-" + kind)
                    run(["git", "apply", str(study / (kind + ".patch"))], 30, mode + "-raw-" + kind)
            functional_bases[mode] = files(source)
            (out / (mode + "-functional-source-diff.patch")).write_text(
                difference(original, functional_bases[mode]), encoding="utf-8")
        restore(original)
        shutil.copyfile(legacy / "ledger.rs", source / "crates/rum-core/src/rewrite_work_ledger.rs")
        patch_core.instrument(source)
        patch_runtime.instrument(source)
        run(["cargo", "+1.85.0", "fmt"], 120, "instrumentation-format")
        diagnostic_base = files(source)
        run(["cp", "-a", "--reflink=auto", str(cache) + "/.", str(target)], 300, "copy-cache")
        (target / "saved").mkdir()
        for mode in MODES:
            restore(diagnostic_base)
            adaptations = []
            if mode in {"matcher", "combined"}:
                patch = study / "matcher.patch"
                run(["git", "apply", "--check", str(patch)], 30, mode + "-patch-check")
                run(["git", "apply", str(patch)], 30, mode + "-patch")
                old = "                Some(outcome) => Ok(outcome),\n                None => self.match_compiled_owned("
                new = f'''                Some(outcome) => {{
                    {LEDGER}::event("rewrite-leaf-match", if outcome.matched {{ "hit" }} else {{ "miss" }}, 1);
                    Ok(outcome)
                }}
                None => self.match_compiled_owned('''
                replace_one(source / APPLICATION, old, new)
                adaptations.append("unique-leaf-event")
            if mode in {"depth", "combined"}:
                assert "+        let depth = if levels.min >= 0 {" in (study / "depth.patch").read_text()
                old = f'''        {LEDGER}::event("replace-node", "entry", 1);
        let depth = if include_heads {{'''
                new = f'''        {LEDGER}::event("replace-node", "entry", 1);
        let depth = if levels.min >= 0 {{
            1
        }} else if include_heads {{'''
                replace_one(source / ONCE, old, new)
                adaptations.append("unique-input-depth")
            run(["cargo", "+1.85.0", "fmt"], 120, mode + "-format-write")
            actual = files(source)
            functional = {n for n, d in diagnostic_base.items() if actual[n] != d}
            expected = ({APPLICATION} if mode in {"matcher", "combined"} else set())
            expected |= {ONCE} if mode in {"depth", "combined"} else set()
            assert functional == expected, (mode, functional)
            changed = {n for n, d in actual.items() if original.get(n) != d}
            diff = difference(original, actual)
            instrumentation = difference(functional_bases[mode], actual)
            instrumentation_path = out / (mode + "-instrumentation-diff.patch")
            instrumentation_path.write_text(instrumentation, encoding="utf-8")
            (out / (mode + "-all-source-diff.patch")).write_text(diff, encoding="utf-8")
            item = {"mode": mode, "status": "RUNNING", "adaptations": adaptations,
                    "functional_files": sorted(functional), "source_diff_sha256": digest(diff.encode()),
                    "functional_diff_sha256": sha(out / (mode + "-functional-source-diff.patch")),
                    "instrumentation_diff_sha256": sha(instrumentation_path),
                    "source_files": {n: {"original_sha256": digest(original[n]) if n in original else None,
                                         "diagnostic_sha256": digest(actual[n])} for n in sorted(changed)}}
            record["variants"].append(item)
            if mode == "base":
                for path in (source / "crates").rglob("*.rs"):
                    os.utime(path, None)
            run(["cargo", "+1.85.0", "build", "--release", "--locked", "--offline", "-p", "rum-cli"], 900, mode + "-build")
            binary = target / "saved" / mode
            shutil.copy2(target / "release/rum-cli", binary)
            assert b"RUM_REWRITE_WORK_LEDGER" in binary.read_bytes()
            run(["cargo", "+1.85.0", "fmt", "--check"], 120, mode + "-format-check")
            item.update(status="PASS", binary=str(binary), binary_sha256=sha(binary))
            save()
            print(mode, "PASS", flush=True)
        record.update(status="PASS", stage="COMPLETE")
    except BaseException as error:
        record.update(status="FAILED", error_type=type(error).__name__, error=str(error))
        raise
    finally:
        if diagnostic_base is not None:
            restore(diagnostic_base)
            record["source_restored_to_instrumented_base"] = files(source) == diagnostic_base
        record["prior_binaries_preserved"] = all(sha(Path(p)) == h for p, h in preserved.items())
        record["default_checkout_changed"] = default != {n: digest(d) for n, d in files(repo / "crates").items()}
        save()
        assert record["prior_binaries_preserved"] and not record["default_checkout_changed"]


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("evidence", type=Path)
    main(parser.parse_args().evidence.resolve())
