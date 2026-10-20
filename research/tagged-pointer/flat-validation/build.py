"""Build four uninstrumented variants and check semantics; no timings."""
import argparse
import difflib
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import tempfile

PIN = "77029938f999b14dbe5b0755bebdf98e51517ae6"
MODES = ["base", "wide", "flat", "combined"]


def digest(data):
    return hashlib.sha256(data).hexdigest()


def sha(path):
    return digest(path.read_bytes())


def files(root):
    return {p.relative_to(root).as_posix(): p.read_bytes()
            for p in root.rglob("*") if p.is_file()}


def workspace(path):
    groups = re.findall(r"test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;",
                        path.read_text(encoding="utf-8"))
    result = {"passed": sum(int(r[0]) for r in groups),
              "ignored": sum(int(r[2]) for r in groups), "groups": len(groups)}
    assert groups and all(int(r[1]) == 0 for r in groups), path
    assert result == {"passed": 2241, "ignored": 7, "groups": 29}, result
    return result


def observations(output, prefix, count):
    result = {}
    for line in output.decode("utf-8").replace("\\t", "\t").splitlines():
        if line.startswith(prefix + "\t"):
            _, key, value = line.split("\t")
            assert key not in result
            result[key] = int(value)
    assert len(result) == count and all(value > 0 for value in result.values())
    return result


def main(out):
    study = Path(__file__).resolve().parent
    repo = study.parents[2]
    prior, ledger = study.parent / "copy-attribution", study.parent / "rewrite-work-ledger"
    assert not out.exists() or not any(out.iterdir())
    out.mkdir(parents=True, exist_ok=True)
    source = Path(tempfile.mkdtemp(prefix="rum-flat-source-", dir="/var/tmp"))
    target = Path(tempfile.mkdtemp(prefix="rum-flat-target-", dir="/var/tmp"))
    cache = Path("/var/tmp/rum-copy-build/base-final")
    prior_paths = [cache / "release/rum-cli",
                   Path("/var/tmp/rum-copy-build/matched-candidate/release/rum-cli"),
                   Path("/var/tmp/rum-copy-build/candidate/release/rum-cli")]
    preserved = {str(p): sha(p) for p in prior_paths}
    default_files = {name: digest(data) for name, data in files(repo / "crates").items()}
    bounded = ["python3", str(repo / "scripts/run-bounded.py")]
    env = {key: value for key, value in os.environ.items() if not key.startswith("RUM_")}
    env.update(CARGO_TARGET_DIR=str(target), CARGO_BUILD_JOBS="2", CARGO_PROFILE_DEV_DEBUG="0")
    record = {"status": "RUNNING", "pin": PIN, "toolchain": "1.85.0",
              "source_root": str(source), "target_root": str(target), "jobs": 2,
              "instrumentation_present": False, "default_checkout_changed": False,
              "preserved_binaries": [{"path": p, "sha256": h} for p, h in preserved.items()],
              "prior_binaries_preserved": False, "variants": [],
              "builder_sha256": sha(Path(__file__))}
    original = None

    def save():
        (out / "builds.json").write_text(json.dumps(record, indent=2) + "\n", encoding="utf-8")

    def run(command, seconds, **kwargs):
        return subprocess.run(bounded + ["--timeout", str(seconds), "--", *command],
                              env=env, timeout=seconds + 15, check=True, **kwargs)

    def logged(name, command, seconds):
        record["stage"] = name
        save()
        with (out / (name + ".log")).open("w", encoding="utf-8") as log:
            run(command, seconds, cwd=source, stdout=log, stderr=subprocess.STDOUT)

    def probe(mode, label, path, marker):
        record["stage"] = mode + "-" + label
        save()
        result = subprocess.run(bounded + ["--timeout", "120", "--", str(target / "saved" / mode),
                                          "--file", str(path)],
                                env=env, stdin=subprocess.DEVNULL, capture_output=True, timeout=135)
        (out / f"{mode}-{label}.stdout").write_bytes(result.stdout)
        (out / f"{mode}-{label}.stderr").write_bytes(result.stderr)
        assert result.returncode == 0 and marker in result.stdout and not result.stderr.strip()
        return {"source": str(path), "source_sha256": sha(path), "exit_code": result.returncode}

    save()
    try:
        total = sum(p.stat().st_size for p in cache.rglob("*") if p.is_file())
        assert shutil.disk_usage(target).free > total + 1024 ** 3, "Insufficient isolated target space"
        logged("prepare", ["python3", str(prior / "prepare.py"),
                            "--output", str(source), "--metadata", str(out / "base-overlay.json"),
                            "--mode", "base"], 300)
        assert json.loads((out / "base-overlay.json").read_text())["upstream_commit"] == PIN
        original = files(source)
        assert not any("rewrite_work_ledger" in name or name.endswith("/copy_attribution.rs") for name in original)
        assert not any(b"rewrite_work_ledger" in data for name, data in original.items() if name.endswith(".rs"))
        logged("copy-cache", ["cp", "-a", "--reflink=auto", str(cache) + "/.", str(target)], 300)
        (target / "saved").mkdir()
        for mode in MODES:
            assert set(files(source)) == set(original), "Unexpected source tree entry"
            for name, data in original.items():
                if (source / name).read_bytes() != data:
                    (source / name).write_bytes(data)
            patches = []
            if mode in {"wide", "combined"}:
                patches.append(prior / "candidate.patch")
            if mode in {"flat", "combined"}:
                patches.append(ledger / "flat-noop.patch")
            for patch in patches:
                run(["git", "apply", "--check", str(patch)], 30, cwd=source)
                run(["git", "apply", str(patch)], 30, cwd=source)
            changed = [name for name, data in original.items() if (source / name).read_bytes() != data]
            if changed:
                run(["rustup", "run", "1.85.0", "rustfmt", "--edition", "2024",
                     *[str(source / name) for name in sorted(changed)]], 120)
            changed = [name for name, data in original.items() if (source / name).read_bytes() != data]
            allowed = {"crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/once.rs",
                       "crates/rum-runtime/src/runtime/expr_runtime_helpers.rs"}
            assert set(changed) <= allowed
            diff = "".join("".join(difflib.unified_diff(
                original[name].decode("utf-8").splitlines(keepends=True),
                (source / name).read_text(encoding="utf-8").splitlines(keepends=True),
                fromfile="a/" + name, tofile="b/" + name)) for name in sorted(changed))
            (out / f"{mode}-source-diff.patch").write_text(diff, encoding="utf-8")
            item = {"mode": mode, "status": "RUNNING", "source_diff_sha256": digest(diff.encode()),
                    "patched_files": {name: {"uninstrumented_base_sha256": digest(original[name]),
                                              "source_sha256": sha(source / name),
                                              "bytes": (source / name).stat().st_size} for name in sorted(changed)}}
            record["variants"].append(item)
            if mode == "base":
                for path in (source / "crates").rglob("*.rs"):
                    os.utime(path, None)
            logged(mode + "-build", ["cargo", "+1.85.0", "build", "--release", "--locked",
                                     "--offline", "-p", "rum-cli"], 900)
            binary = target / "saved" / mode
            shutil.copy2(target / "release/rum-cli", binary)
            item.update(binary=str(binary), binary_sha256=sha(binary))
            logged(mode + "-format", ["cargo", "+1.85.0", "fmt", "--check"], 120)
            logged(mode + "-workspace", ["cargo", "+1.85.0", "test", "--workspace", "--locked",
                                         "--offline", "--", "--test-threads=1"], 1800)
            item["workspace"] = workspace(out / f"{mode}-workspace.log")
            item["acceptance"] = probe(mode, "acceptance", prior / "acceptance.rum",
                                       b"COPY_ATTRIBUTION_ACCEPTANCE_PASS")
            item["resource"] = probe(mode, "resource", ledger / "resource-probes.rum",
                                     b"RESOURCE_PROBE_VALUE_ALIAS_PASS")
            item["resource"]["values"] = observations((out / f"{mode}-resource.stdout").read_bytes(), "RESOURCE_PROBE", 7)
            item["capacity"] = probe(mode, "capacity", ledger / "capacity-observability.rum",
                                     b"CAPACITY_VALUE_PASS")
            item["capacity"]["values"] = observations((out / f"{mode}-capacity.stdout").read_bytes(), "CAPACITY_OBSERVATION", 2)
            item["status"] = "PASS"
            save()
            print(mode, "UNINSTRUMENTED_BUILD_AND_SEMANTICS_PASS", flush=True)
        record.update(status="PASS", stage="COMPLETE")
    except BaseException as error:
        record.update(status="FAILED", error_type=type(error).__name__, error=str(error))
        raise
    finally:
        if original is not None:
            for name, data in original.items():
                if (source / name).read_bytes() != data:
                    (source / name).write_bytes(data)
            record["source_restored"] = files(source) == original
        record["prior_binaries_preserved"] = all(sha(Path(p)) == h for p, h in preserved.items())
        record["default_checkout_changed"] = default_files != {n: digest(d) for n, d in files(repo / "crates").items()}
        save()
        assert record.get("source_restored", original is None)
        assert record["prior_binaries_preserved"] and not record["default_checkout_changed"]


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("evidence", type=Path)
    main(parser.parse_args().evidence.resolve())
