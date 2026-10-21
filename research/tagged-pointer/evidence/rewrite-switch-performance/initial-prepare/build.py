"""Build one isolated executable and check four immutable process modes."""
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

PIN = "77029938f999b14dbe5b0755bebdf98e51517ae6"
MODES = ["base", "matcher", "depth", "combined"]


def main(out, primary_builds):
    repo = STUDY.parents[2]
    prior, ledger = STUDY.parent / "copy-attribution", STUDY.parent / "rewrite-work-ledger"
    primary = json.loads(primary_builds.read_text(encoding="utf-8"))
    assert primary["status"] == "PASS" and primary["pin"] == PIN
    assert not out.exists() or not any(out.iterdir())
    out.mkdir(parents=True, exist_ok=True)
    source = Path(tempfile.mkdtemp(prefix="rum-switch-source-", dir="/var/tmp"))
    target = Path(tempfile.mkdtemp(prefix="rum-switch-target-", dir="/var/tmp"))
    cache = Path("<LAB_REWRITE_SWITCH_PERFORMANCE_PATH_0001>")
    preserved = {item["path"]: item["sha256"] for item in primary["preserved_binaries"]}
    preserved.update({item["binary"]: item["binary_sha256"] for item in primary["variants"]})
    assert len(preserved) == 7 and all(sha(Path(p)) == h for p, h in preserved.items())
    default_files = {n: digest(d) for n, d in files(repo / "crates").items()}
    env = {k: v for k, v in os.environ.items() if not k.upper().startswith("RUM_")}
    env.update(CARGO_TARGET_DIR=str(target), CARGO_BUILD_JOBS="2", CARGO_PROFILE_DEV_DEBUG="0")
    bounded = ["python3", str(repo / "scripts/run-bounded.py")]
    test_count = 2241 + sum((PRIMARY / f"{name}-tests.rs").read_text().count("#[test]")
                            for name in ("matcher", "depth"))
    assert test_count == 2256 and "#[test]" not in (STUDY / "switch.patch").read_text()
    expected = {"passed": test_count, "ignored": 7, "groups": 29}
    record = {"status": "RUNNING", "pin": PIN, "toolchain": "1.85.0", "jobs": 2,
              "source_root": str(source), "target_root": str(target),
              "instrumentation_present": False, "same_image_switch": True,
              "default_base_is_untouched_original": False,
              "control_boundary": "All modes pay common cached switch queries; fresh process per mode.",
              "patch_basis": "Independent switch.patch includes matcher and depth logic; no primary patches applied.",
              "expected_workspace": expected, "default_checkout_changed": False,
              "preserved_binaries": [{"path": p, "sha256": h} for p, h in preserved.items()],
              "prior_binaries_preserved": False, "variants": [],
              "primary_builds": str(primary_builds), "primary_builds_sha256": sha(primary_builds),
              "builder_sha256": sha(Path(__file__)), "build_checks_sha256": sha(STUDY / "build_checks.py"),
              "overlay_inputs": {"switch.patch": sha(STUDY / "switch.patch"),
                  **{f"../rewrite-shortcuts/{name}-tests.rs": sha(PRIMARY / f"{name}-tests.rs")
                     for name in ("matcher", "depth")}}}
    common = None

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
        return build_checks.probe(target / "saved" / mode, out, mode, label, path,
                                  marker, bounded, env, source)

    save()
    try:
        total = sum(p.stat().st_size for p in cache.rglob("*") if p.is_file())
        assert shutil.disk_usage(target).free > total + 1024 ** 3, "Insufficient space"
        logged("prepare", ["python3", str(prior / "prepare.py"), "--output", str(source),
                            "--metadata", str(out / "base-overlay.json"), "--mode", "base"], 300)
        assert json.loads((out / "base-overlay.json").read_text())["upstream_commit"] == PIN
        for name in ("matcher", "depth"):
            shutil.copyfile(PRIMARY / f"{name}-tests.rs",
                            source / f"crates/rum-runtime/src/runtime/tests/rewrite_shortcuts_{name}.rs")
        with (source / "crates/rum-runtime/src/runtime/tests/mod.rs").open("a") as modules:
            modules.write("\nmod rewrite_shortcuts_matcher;\nmod rewrite_shortcuts_depth;\n")
        run(["cargo", "+1.85.0", "fmt"], 120, cwd=source)
        original = files(source)
        for name in original:
            assert "rewrite_work_ledger" not in name and not name.endswith("/copy_attribution.rs")
        run(["git", "apply", "--check", str(STUDY / "switch.patch")], 30, cwd=source)
        run(["git", "apply", str(STUDY / "switch.patch")], 30, cwd=source)
        run(["cargo", "+1.85.0", "fmt"], 120, cwd=source)
        common = files(source)
        record.update(build_checks.source_record(original, common, out))
        logged("copy-cache", ["cp", "-a", "--reflink=auto", str(cache) + "/.", str(target)], 300)
        (target / "saved").mkdir()
        for path in (source / "crates").rglob("*.rs"):
            os.utime(path, None)
        env["RUM_REWRITE_SHORTCUTS_MODE"] = "base"
        logged("common-build", ["cargo", "+1.85.0", "build", "--release", "--locked",
                                 "--offline", "-p", "rum-cli"], 900)
        binary_hash = sha(target / "release/rum-cli")
        image = (target / "release/rum-cli").read_bytes()
        assert b"RUM_REWRITE_SHORTCUTS_MODE" in image and b"RUM_REWRITE_WORK_LEDGER" not in image
        record["common_binary_sha256"] = binary_hash
        corpora = build_checks.preflight_sources(out)
        for mode in MODES:
            env["RUM_REWRITE_SHORTCUTS_MODE"] = mode
            binary = target / "saved" / mode
            shutil.copy2(target / "release/rum-cli", binary)
            assert sha(binary) == binary_hash
            item = {"mode": mode, "status": "RUNNING", "binary": str(binary),
                    "binary_sha256": binary_hash, "source_diff_sha256": record["common_source_diff_sha256"],
                    "environment": {"RUM_REWRITE_SHORTCUTS_MODE": mode}, "preflights": {}}
            record["variants"].append(item)
            logged(mode + "-format", ["cargo", "+1.85.0", "fmt", "--check"], 120)
            item["format"] = {"status": "PASS", "log": mode + "-format.log"}
            logged(mode + "-workspace", ["cargo", "+1.85.0", "test", "--workspace", "--locked",
                                         "--offline", "--", "--test-threads=1"], 1800)
            item["workspace"] = build_checks.workspace(out / f"{mode}-workspace.log", PRIMARY, expected)
            for label, path, marker in (
                ("acceptance", prior / "acceptance.rum", "COPY_ATTRIBUTION_ACCEPTANCE_PASS"),
                ("resource", ledger / "resource-probes.rum", "RESOURCE_PROBE_VALUE_ALIAS_PASS"),
                ("capacity", ledger / "capacity-observability.rum", "CAPACITY_VALUE_PASS")):
                item[label] = probe(mode, label, path, marker)
            for label, prefix, count in (("resource", "RESOURCE_PROBE", 7), ("capacity", "CAPACITY_OBSERVATION", 2)):
                item[label]["values"] = observations((out / f"{mode}-{label}.stdout").read_bytes(), prefix, count)
                assert item[label]["values"] == record["variants"][0][label]["values"]
                assert item[label]["values"] == primary["variants"][0][label]["values"]
            for label, (_, _, marker) in corpora.items():
                item["preflights"][label] = probe(mode, label, out / f"{label}-preflight.rum", marker)
            assert files(source) == common
            item["status"] = "PASS"
            save()
            print(mode, "SAME_IMAGE_BUILD_AND_SEMANTICS_PASS", flush=True)
        record.update(status="PASS", stage="COMPLETE")
    except BaseException as error:
        record.update(status="FAILED", error_type=type(error).__name__, error=str(error))
        raise
    finally:
        if common is not None:
            for name, data in common.items():
                if (source / name).read_bytes() != data:
                    (source / name).write_bytes(data)
            record["source_restored"] = files(source) == common
            record["source_restored_state"] = "common-switch-patched; added switch module remains"
        record["prior_binaries_preserved"] = all(sha(Path(p)) == h for p, h in preserved.items())
        record["default_checkout_changed"] = default_files != {n: digest(d) for n, d in files(repo / "crates").items()}
        if not (record.get("source_restored", common is None) and record["prior_binaries_preserved"]
                and not record["default_checkout_changed"]):
            record["status"] = "FAILED"
        save()
        assert record.get("source_restored", common is None)
        assert record["prior_binaries_preserved"] and not record["default_checkout_changed"]


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("evidence", type=Path)
    parser.add_argument("--primary-builds", type=Path)
    args = parser.parse_args()
    out = args.evidence.resolve()
    main(out, args.primary_builds.resolve() if args.primary_builds else out.parent / "rewrite-shortcuts/builds.json")
