"""Build independent original, matcher, and borrowed-one-binding variants."""
import argparse
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile

sys.dont_write_bytecode = True
STUDY = Path(__file__).resolve().parent
PRIMARY = STUDY.parent / "rewrite-shortcuts"
sys.path.append(str(PRIMARY))
from build_support import observations, sha, digest, files
import build_checks
import workloads

PIN = "77029938f999b14dbe5b0755bebdf98e51517ae6"
IMPLEMENTATIONS = ["base", "matcher", "single"]


def main(out, primary_path, switch_path):
    repo = STUDY.parents[2]
    prior, ledger = STUDY.parent / "copy-attribution", STUDY.parent / "rewrite-work-ledger"
    primary, preserved, switch = build_checks.preserved_binaries(primary_path, switch_path)
    assert primary["pin"] == PIN and primary["toolchain"] == "1.85.0"
    assert not out.exists() or not any(out.iterdir())
    out.mkdir(parents=True, exist_ok=True)
    source = Path(tempfile.mkdtemp(prefix="rum-single-source-", dir="/var/tmp"))
    target = Path(tempfile.mkdtemp(prefix="rum-single-target-", dir="/var/tmp"))
    cache = Path("/var/tmp/rum-copy-build/base-final")
    default_files = {n: digest(d) for n, d in files(repo / "crates").items()}
    bounded = ["python3", str(repo / "scripts/run-bounded.py")]
    env = {k: v for k, v in os.environ.items() if not k.upper().startswith("RUM_")}
    env.update(CARGO_TARGET_DIR=str(target), CARGO_BUILD_JOBS="2", CARGO_PROFILE_DEV_DEBUG="0")
    expected = 2241 + sum((PRIMARY / f"{name}-tests.rs").read_text().count("#[test]")
                          for name in ("matcher", "depth"))
    assert expected + (STUDY / "regression-tests.rs").read_text().count("#[test]") == 2262
    record = {"status": "RUNNING", "pin": PIN, "toolchain": "1.85.0", "jobs": 2,
              "source_root": str(source), "target_root": str(target),
              "instrumentation_present": False, "same_image_switch": False,
              "expected_workspace": {"passed": 2262, "ignored": 7, "groups": 29},
              "default_checkout_changed": False, "source_entries_checked": False,
              "preserved_binaries": [{"path": p, "sha256": h} for p, h in preserved.items()],
              "prior_binaries_preserved": False, "variants": [],
              "primary_builds": str(primary_path), "primary_builds_sha256": sha(primary_path),
              "switch_builds": switch, "builder_sha256": sha(Path(__file__)),
              "build_checks_sha256": sha(STUDY / "build_checks.py"),
              "overlay_inputs": {"candidate.patch": sha(STUDY / "candidate.patch"),
                  "regression-tests.rs": sha(STUDY / "regression-tests.rs"),
                  **{f"../rewrite-shortcuts/{name}": sha(PRIMARY / name) for name in (
                      "matcher.patch", "matcher-tests.rs", "depth-tests.rs")}}}
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
                                          "--file", str(path)], env=env, cwd=source,
                                stdin=subprocess.DEVNULL, capture_output=True, timeout=135)
        (out / f"{mode}-{label}.stdout").write_bytes(result.stdout)
        (out / f"{mode}-{label}.stderr").write_bytes(result.stderr)
        assert result.returncode == 0 and marker.encode() in result.stdout and not result.stderr.strip()
        return {"source": str(path), "source_sha256": sha(path), "exit_code": result.returncode}

    save()
    try:
        total = sum(p.stat().st_size for p in cache.rglob("*") if p.is_file())
        assert shutil.disk_usage(target).free > total + 1024 ** 3, "Insufficient space"
        logged("prepare", ["python3", str(prior / "prepare.py"), "--output", str(source),
                            "--metadata", str(out / "base-overlay.json"), "--mode", "base"], 300)
        assert json.loads((out / "base-overlay.json").read_text())["upstream_commit"] == PIN
        tests = [(f"rewrite_shortcuts_{name}", PRIMARY / f"{name}-tests.rs") for name in ("matcher", "depth")]
        tests.append(("single_binding_regression", STUDY / "regression-tests.rs"))
        for module, path in tests:
            shutil.copyfile(path, source / f"crates/rum-runtime/src/runtime/tests/{module}.rs")
        with (source / "crates/rum-runtime/src/runtime/tests/mod.rs").open("a") as modules:
            modules.write("".join(f"\nmod {module};\n" for module, _ in tests))
        run(["cargo", "+1.85.0", "fmt"], 120, cwd=source)
        original = files(source)
        assert not any(b"rewrite_work_ledger" in d or b"rewrite_shortcuts_switch" in d
                       for n, d in original.items() if n.endswith(".rs"))
        record["common_original_source_hashes"] = {n: digest(d) for n, d in original.items()}
        logged("copy-cache", ["cp", "-a", "--reflink=auto", str(cache) + "/.", str(target)], 300)
        (target / "saved").mkdir()
        (out / "semantic-preflight.rum").write_text(workloads.semantic_source(), encoding="utf-8")
        for mode in IMPLEMENTATIONS:
            assert set(files(source)) == set(original), "Unexpected source entry"
            for name, data in original.items():
                if (source / name).read_bytes() != data:
                    (source / name).write_bytes(data)
            item = build_checks.variant_source(source, original, mode, PRIMARY, STUDY, out, run)
            record["variants"].append(item)
            if mode == "base":
                for path in (source / "crates").rglob("*.rs"):
                    os.utime(path, None)
            logged(mode + "-build", ["cargo", "+1.85.0", "build", "--release", "--locked",
                                     "--offline", "-p", "rum-cli"], 900)
            binary = target / "saved" / mode
            shutil.copy2(target / "release/rum-cli", binary)
            item.update(binary=str(binary), binary_sha256=sha(binary))
            assert b"RUM_REWRITE_WORK_LEDGER" not in binary.read_bytes()
            logged(mode + "-format", ["cargo", "+1.85.0", "fmt", "--check"], 120)
            logged(mode + "-workspace", ["cargo", "+1.85.0", "test", "--workspace", "--locked",
                                         "--offline", "--", "--test-threads=1"], 1800)
            item["workspace"] = build_checks.workspace(out / f"{mode}-workspace.log", PRIMARY, STUDY)
            for label, path, marker in (
                ("acceptance", prior / "acceptance.rum", "COPY_ATTRIBUTION_ACCEPTANCE_PASS"),
                ("resource", ledger / "resource-probes.rum", "RESOURCE_PROBE_VALUE_ALIAS_PASS"),
                ("capacity", ledger / "capacity-observability.rum", "CAPACITY_VALUE_PASS"),
                ("semantic", out / "semantic-preflight.rum", "ISO_SEMANTICS_PASS")):
                item[label] = probe(mode, label, path, marker)
            for label, prefix, count in (("resource", "RESOURCE_PROBE", 7), ("capacity", "CAPACITY_OBSERVATION", 2)):
                item[label]["values"] = observations((out / f"{mode}-{label}.stdout").read_bytes(), prefix, count)
                assert item[label]["values"] == record["variants"][0][label]["values"]
                assert item[label]["values"] == primary["variants"][0][label]["values"]
            item["status"] = "PASS"
            save()
            print(mode, "UNINSTRUMENTED_BINDING_BUILD_AND_SEMANTICS_PASS", flush=True)
        assert len({v["binary_sha256"] for v in record["variants"]}) == 3
        record.update(status="PASS", stage="COMPLETE")
    except BaseException as error:
        record.update(status="FAILED", error_type=type(error).__name__, error=str(error))
        raise
    finally:
        if original is not None:
            record["source_entries_checked"] = set(files(source)) == set(original)
            for name, data in original.items():
                if (source / name).read_bytes() != data:
                    (source / name).write_bytes(data)
            record["source_restored"] = files(source) == original
            record["source_restored_state"] = "common original runtime plus uniform regression tests"
        record["prior_binaries_preserved"] = all(sha(Path(p)) == h for p, h in preserved.items())
        record["default_checkout_changed"] = default_files != {n: digest(d) for n, d in files(repo / "crates").items()}
        valid = (record.get("source_restored", original is None) and record["prior_binaries_preserved"]
                 and not record["default_checkout_changed"])
        if not valid:
            record["status"] = "FAILED"
        save()
        assert valid


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("evidence", type=Path)
    parser.add_argument("--primary-builds", type=Path)
    parser.add_argument("--switch-builds", type=Path)
    args = parser.parse_args()
    out = args.evidence.resolve()
    main(out, args.primary_builds.resolve() if args.primary_builds else out.parent / "rewrite-shortcuts/builds.json",
         args.switch_builds.resolve() if args.switch_builds else out.parent / "rewrite-switch/builds.json")
