"""Build diagnostic copies and semantic checks; never run performance gates."""
import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
sys.dont_write_bytecode = True
import patch_core
import patch_runtime

MODES = ["base", "wide", "wide-flat", "flat"]
PIN = "77029938f999b14dbe5b0755bebdf98e51517ae6"


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main(out):
    study = Path(__file__).resolve().parent
    repo = study.parents[2]
    assert not out.exists() or not any(out.iterdir())
    out.mkdir(parents=True, exist_ok=True)
    source = Path(tempfile.mkdtemp(prefix="rum-work-source-", dir="/var/tmp"))
    target = Path(tempfile.mkdtemp(prefix="rum-work-target-", dir="/var/tmp"))
    cache = Path("/var/tmp/rum-copy-build/base-final")
    preserved = {str(p): sha(p) for p in [cache / "release/rum-cli", Path("/var/tmp/rum-copy-build/matched-candidate/release/rum-cli")]}
    total = sum(p.stat().st_size for p in cache.rglob("*") if p.is_file())
    assert shutil.disk_usage(target).free > total + 1024 ** 3, "Insufficient space for isolated cached target"
    spec = importlib.util.spec_from_file_location("prior_copy_prepare", study.parent / "copy-attribution/prepare.py")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    module.prepare(source, out / "base-overlay.json", "base")
    bounded = ["python3", str(repo / "scripts/run-bounded.py")]
    subprocess.run(bounded + ["--timeout", "300", "--", "cp", "-a", "--reflink=auto", str(cache) + "/.", str(target)], check=True, timeout=315)
    originals = {p.relative_to(source).as_posix(): p.read_bytes() for p in (source / "crates").rglob("*.rs")}
    shutil.copyfile(study / "ledger.rs", source / "crates/rum-core/src/rewrite_work_ledger.rs")
    env = {k: v for k, v in os.environ.items() if not k.startswith("RUM_")}
    env.update(CARGO_TARGET_DIR=str(target), CARGO_BUILD_JOBS="2", CARGO_PROFILE_DEV_DEBUG="0")
    plan = {"status": "RUNNING", "source_root": str(source), "target_root": str(target), "pin": PIN,
            "jobs": 2, "diagnostic_only": True, "formal_performance_gate": "DEFERRED_USER_ACTIVE_DESKTOP",
            "preserved_binaries": [{"path": p, "sha256": digest} for p, digest in preserved.items()], "variants": []}
    saved = target / "saved"
    saved.mkdir()
    changed_names = set()
    for mode in MODES:
        for name in changed_names:
            (source / name).write_bytes(originals[name])
        for patch in ([study.parent / "copy-attribution/candidate.patch"] if "wide" in mode else []) + ([study / "flat-noop.patch"] if "flat" in mode else []):
            subprocess.run(["git", "apply", "--check", str(patch)], cwd=source, check=True, timeout=30)
            subprocess.run(["git", "apply", str(patch)], cwd=source, check=True, timeout=30)
        patch_core.instrument(source)
        patch_runtime.instrument(source)
        helper = source / "crates/rum-runtime/src/runtime/expr_runtime_helpers.rs"
        if "flat" in mode:
            value = helper.read_text()
            before = "    }) {\n        return args;\n    }\n\n    let mut flattened = Vec::new();"
            assert value.count(before) == 1
            helper.write_text(value.replace(before, '''    }) {
        rum_core::rewrite_work_ledger::event("flatten-reuse-slots", ledger_head, args.len());
        rum_core::rewrite_work_ledger::event("flatten-reuse-capacity-slots", ledger_head, args.capacity());
        return args;
    }

    let mut flattened = Vec::new();'''))
        changed_names = {name for name, data in originals.items() if (source / name).read_bytes() != data}
        rust = [str(source / name) for name in sorted(changed_names)] + [str(source / "crates/rum-core/src/rewrite_work_ledger.rs")]
        subprocess.run(["rustup", "run", "1.85.0", "rustfmt", "--edition", "2024", *rust], check=True, timeout=120)
        if mode == "base":
            for path in (source / "crates").rglob("*.rs"):
                os.utime(path, None)
        logs = {"build": ["cargo", "+1.85.0", "build", "--release", "--locked", "--offline", "-p", "rum-cli"],
                "format": ["cargo", "+1.85.0", "fmt", "--check"],
                "workspace": ["cargo", "+1.85.0", "test", "--workspace", "--locked", "--offline", "--", "--test-threads=1"]}
        for label, command in logs.items():
            limit = 1800 if label == "workspace" else 900
            with (out / f"{mode}-{label}.log").open("w") as log:
                subprocess.run(bounded + ["--timeout", str(limit), "--", *command], cwd=source, env=env, stdout=log, stderr=subprocess.STDOUT, check=True, timeout=limit + 15)
        binary = saved / mode
        shutil.copy2(target / "release/rum-cli", binary)
        result = subprocess.run(bounded + ["--timeout", "120", "--", str(binary), "--file", str(study.parent / "copy-attribution/acceptance.rum")], env=env, stdin=subprocess.DEVNULL, capture_output=True, timeout=135)
        (out / f"{mode}-acceptance.stdout").write_bytes(result.stdout)
        (out / f"{mode}-acceptance.stderr").write_bytes(result.stderr)
        assert result.returncode == 0 and b"COPY_ATTRIBUTION_ACCEPTANCE_PASS" in result.stdout and not result.stderr.strip()
        plan["variants"].append({"mode": mode, "binary": str(binary), "binary_sha256": sha(binary),
                                 "patched_files": {name: {"uninstrumented_base_sha256": hashlib.sha256(originals[name]).hexdigest(), "diagnostic_sha256": sha(source / name), "bytes": (source / name).stat().st_size} for name in sorted(changed_names)},
                                 "ledger_module_sha256": sha(source / "crates/rum-core/src/rewrite_work_ledger.rs")})
        (out / "builds.json").write_text(json.dumps(plan, indent=2) + "\n")
        print(mode, "BUILD_AND_SEMANTICS_PASS", flush=True)
    assert all(sha(Path(p)) == digest for p, digest in preserved.items())
    plan.update(status="PASS", prior_binaries_preserved=True, default_checkout_changed=False)
    (out / "builds.json").write_text(json.dumps(plan, indent=2) + "\n")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("evidence", type=Path)
    main(parser.parse_args().evidence.resolve())
