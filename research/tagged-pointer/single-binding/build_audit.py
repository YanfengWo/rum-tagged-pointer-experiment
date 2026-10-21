"""Read the saved correctness certificate before a future timing analysis."""
import hashlib
import json
from pathlib import Path


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def audit(out):
    study = Path(__file__).resolve().parent
    record = json.loads((out / "correctness.json").read_text(encoding="utf-8"))
    builds = json.loads((out / "builds.json").read_text(encoding="utf-8"))
    assert record["status"] == builds["status"] == "PASS"
    assert record["phase"] == "CORRECTNESS_ONLY"
    assert builds["source_restored"] and builds["source_entries_checked"]
    assert builds["prior_binaries_preserved"] and not builds["default_checkout_changed"]
    assert not builds["instrumentation_present"]
    assert builds["builder_sha256"] == sha(study / "build.py")
    assert builds["build_checks_sha256"] == sha(study / "build_checks.py")
    for name, digest in builds["overlay_inputs"].items():
        assert sha(study / name) == digest
    for path, digest in record["metadata_sha256"].items():
        assert sha(Path(path)) == digest
    for name, digest in record["raw_files_sha256"].items():
        assert sha(out / name) == digest
    variants = {v["mode"]: v for v in builds["variants"]}
    assert set(variants) == {"base", "matcher", "single"}
    assert len({v["binary_sha256"] for v in variants.values()}) == 3
    for mode, item in variants.items():
        assert item["status"] == "PASS" and sha(Path(item["binary"])) == item["binary_sha256"]
        assert item["workspace"] == {"passed": 2262, "ignored": 7, "groups": 29}
        assert record["workspace"][mode] == item["workspace"]
        assert record["binary_sha256"][mode] == item["binary_sha256"]
        assert sha(out / f"{mode}-source-diff.patch") == item["source_diff_sha256"]
    for item in builds["preserved_binaries"]:
        assert sha(Path(item["path"])) == item["sha256"]
    return builds, variants, record["workspace"]
