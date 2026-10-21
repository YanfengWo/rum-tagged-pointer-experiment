"""Audit one executable, four immutable startup modes and shared semantics."""
import hashlib
import json
from pathlib import Path
import sys
sys.dont_write_bytecode = True
STUDY = Path(__file__).resolve().parent
PRIMARY = STUDY.parent / "rewrite-shortcuts"
sys.path.append(str(PRIMARY))
import build_checks


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def audit(out):
    data = json.loads((out / "builds.json").read_text())
    assert data["status"] == "PASS" and data["same_image_switch"]
    assert data["source_restored"] and data["prior_binaries_preserved"]
    assert not data["default_checkout_changed"] and not data["instrumentation_present"]
    assert not data["default_base_is_untouched_original"]
    assert data["builder_sha256"] == sha(STUDY / "build.py")
    assert data["build_checks_sha256"] == sha(STUDY / "build_checks.py")
    assert data["common_source_diff_sha256"] == sha(out / "common-source-diff.patch")
    assert data["primary_builds_sha256"] == sha(Path(data["primary_builds"]))
    for path, digest in data["overlay_inputs"].items():
        assert sha(STUDY / path) == digest
    for item in data["preserved_binaries"]:
        assert sha(Path(item["path"])) == item["sha256"]
    variants = {v["mode"]: v for v in data["variants"]}
    assert set(variants) == {"base", "matcher", "depth", "combined"}
    assert {v["binary_sha256"] for v in variants.values()} == {data["common_binary_sha256"]}
    checks = {}
    for mode, item in variants.items():
        assert item["status"] == "PASS" and sha(Path(item["binary"])) == data["common_binary_sha256"]
        assert item["environment"] == {"RUM_REWRITE_SHORTCUTS_MODE": mode}
        checks[mode] = build_checks.workspace(out / f"{mode}-workspace.log", PRIMARY,
                                              {"passed": 2256, "ignored": 7, "groups": 29})
        assert checks[mode] == item["workspace"]
        for label in ("resource", "capacity"):
            assert item[label]["values"] == variants["base"][label]["values"]
        for label in ("acceptance", "resource", "capacity", "focus", "pipeline", "shape", "semantic"):
            assert not (out / f"{mode}-{label}.stderr").read_text().strip()
            assert "PASS" in (out / f"{mode}-{label}.stdout").read_text()
    for name, digest in data["common_source_hashes"].items():
        assert sha(Path(data["source_root"]) / name) == digest
    return data, variants, checks
