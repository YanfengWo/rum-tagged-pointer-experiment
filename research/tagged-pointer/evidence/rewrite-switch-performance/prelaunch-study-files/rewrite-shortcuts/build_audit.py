"""Recheck saved build outputs and regression execution without rebuilding."""
import hashlib
import json
from pathlib import Path
from build_support import workspace, observations


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def audit(out):
    study = Path(__file__).resolve().parent
    builds = json.loads((out / "builds.json").read_text())
    assert builds["status"] == "PASS" and builds["source_restored"]
    assert builds["prior_binaries_preserved"] and not builds["default_checkout_changed"]
    assert not builds["instrumentation_present"]
    assert builds["pin"] == "77029938f999b14dbe5b0755bebdf98e51517ae6"
    assert builds["toolchain"] == "1.85.0" and builds["builder_sha256"] == sha(study / "build.py")
    for path, digest in builds["overlay_inputs"].items():
        assert sha(study / path) == digest
    for item in builds["preserved_binaries"]:
        assert sha(Path(item["path"])) == item["sha256"]
    variants = {item["mode"]: item for item in builds["variants"]}
    expected = {
        "base": set(),
        "matcher": {"crates/rum-runtime/src/runtime/definitions/application/replace_all.rs"},
        "depth": {"crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/once.rs"},
    }
    expected["combined"] = expected["matcher"] | expected["depth"]
    assert set(variants) == set(expected)
    assert len({v["binary_sha256"] for v in variants.values()}) == 4
    checks = {}
    for mode, item in variants.items():
        assert item["status"] == "PASS"
        assert sha(Path(item["binary"])) == item["binary_sha256"]
        assert sha(out / f"{mode}-source-diff.patch") == item["source_diff_sha256"]
        assert set(item["patched_files"]) == expected[mode]
        checks[mode] = workspace(out / f"{mode}-workspace.log")
        assert checks[mode] == item["workspace"]
        log = (out / f"{mode}-workspace.log").read_text()
        for module in ("matcher", "depth"):
            test_names = __import__("re").findall(r"#\[test\]\s*fn (\w+)\(", (study / f"{module}-tests.rs").read_text())
            for name in test_names:
                assert f"rewrite_shortcuts_{module}::{name} ... ok" in log, name
        for label, marker in (("acceptance", "COPY_ATTRIBUTION_ACCEPTANCE_PASS"),
                              ("resource", "RESOURCE_PROBE_VALUE_ALIAS_PASS"),
                              ("capacity", "CAPACITY_VALUE_PASS")):
            assert item[label]["exit_code"] == 0
            assert marker in (out / f"{mode}-{label}.stdout").read_text()
            assert not (out / f"{mode}-{label}.stderr").read_text().strip()
        for label, prefix, count in (("resource", "RESOURCE_PROBE", 7),
                                     ("capacity", "CAPACITY_OBSERVATION", 2)):
            assert item[label]["values"] == observations(
                (out / f"{mode}-{label}.stdout").read_bytes(), prefix, count)
    for item in variants.values():
        for label in ("resource", "capacity"):
            assert item[label]["values"] == variants["base"][label]["values"], (item["mode"], label)
    return builds, variants, checks
