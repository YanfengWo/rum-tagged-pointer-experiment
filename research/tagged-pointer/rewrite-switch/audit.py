"""Audit completed switch correctness evidence without running or claiming timings."""
import argparse
import ast
from datetime import datetime, timezone
import json
from pathlib import Path
import sys

sys.dont_write_bytecode = True
STUDY = Path(__file__).resolve().parent
from build_audit import audit as audit_builds, sha
from build_support import observations

MODES = ("base", "matcher", "depth", "combined")
WORKSPACE = {"passed": 2256, "ignored": 7, "groups": 29}
MARKERS = {
    "acceptance": "COPY_ATTRIBUTION_ACCEPTANCE_PASS",
    "resource": "RESOURCE_PROBE_VALUE_ALIAS_PASS",
    "capacity": "CAPACITY_VALUE_PASS",
    "focus": "ISO_FOCUS_PASS",
    "pipeline": "ISO_PIPELINE_PASS",
    "shape": "ISO_SHAPE_PASS",
    "semantic": "ISO_SEMANTICS_PASS",
}


def load(path):
    return json.loads(path.read_text(encoding="utf-8"))


def audit_smoke(out):
    smoke = load(out / "mode-smoke.json")
    assert smoke["status"] == "PASS"
    labels = ("unset", *MODES, "empty", "unknown", "spaced")
    assert [row["label"] for row in smoke["checks"]] == list(labels)
    for row in smoke["checks"]:
        valid = row["label"] in {"unset", *MODES}
        assert row["valid"] is valid
        stdout = (out / f"mode-smoke-{row['label']}.stdout").read_bytes()
        stderr = (out / f"mode-smoke-{row['label']}.stderr").read_bytes()
        if valid:
            assert row["exit_code"] == 0
            assert b"MODE_SMOKE_PASS" in stdout and not stderr.strip()
        else:
            assert row["exit_code"] != 0
            assert b"RUM_REWRITE_SHORTCUTS_MODE" in stderr
            assert b"MODE_SMOKE_PASS" not in stdout
    assert (out / "mode-smoke.rum").read_text(encoding="utf-8") == 'Print["MODE_SMOKE_PASS"];\n'
    return {"status": "PASS", "checks": smoke["checks"],
            "record_sha256": sha(out / "mode-smoke.json"),
            "input_sha256": sha(out / "mode-smoke.rum"),
            "limits": "Eight independent-process startup checks only; no in-process mode mutation, multiple-Session or threaded-construction result is claimed."}


def audit(out):
    destination = out / "correctness.json"
    for path in STUDY.glob("*.py"):
        ast.parse(path.read_text(encoding="utf-8"), filename=path.name)
        assert path.stat().st_size < 10000, path.name
    builds, variants, workspaces = audit_builds(out)
    assert builds["pin"] == "77029938f999b14dbe5b0755bebdf98e51517ae6"
    assert builds["expected_workspace"] == WORKSPACE
    assert len(builds["variants"]) == len(variants) == 4
    assert all(value == WORKSPACE for value in workspaces.values())
    preserved = builds["preserved_binaries"]
    assert len(preserved) == len({item["path"] for item in preserved}) == 7
    source = Path(builds["source_root"])
    assert {p.relative_to(source).as_posix() for p in source.rglob("*") if p.is_file()} == set(builds["common_source_hashes"])
    primary = load(Path(builds["primary_builds"]))
    probes = {}
    for mode in MODES:
        item = variants[mode]
        assert item["format"]["status"] == "PASS"
        assert not (out / item["format"]["log"]).read_text(encoding="utf-8").strip()
        assert set(item["preflights"]) == {"focus", "pipeline", "shape", "semantic"}
        assert item["source_diff_sha256"] == builds["common_source_diff_sha256"]
        probes[mode] = {}
        for label, marker in MARKERS.items():
            metadata = item[label] if label in {"acceptance", "resource", "capacity"} else item["preflights"][label]
            assert metadata["exit_code"] == 0
            assert sha(Path(metadata["source"])) == metadata["source_sha256"]
            if label in item["preflights"]:
                assert sha(out / f"{label}-preflight.rum") == metadata["source_sha256"]
                assert "AbsoluteTiming" not in (out / f"{label}-preflight.rum").read_text(encoding="utf-8")
            stdout = out / f"{mode}-{label}.stdout"
            stderr = out / f"{mode}-{label}.stderr"
            assert marker.encode() in stdout.read_bytes() and not stderr.read_bytes().strip()
            assert b"rum-work\t" not in stdout.read_bytes() + stderr.read_bytes()
            probes[mode][label] = {"status": "PASS", "source_sha256": metadata["source_sha256"],
                                   "stdout_sha256": sha(stdout), "stderr_sha256": sha(stderr)}
        for label, prefix, count in (("resource", "RESOURCE_PROBE", 7), ("capacity", "CAPACITY_OBSERVATION", 2)):
            actual = observations((out / f"{mode}-{label}.stdout").read_bytes(), prefix, count)
            assert actual == item[label]["values"] == primary["variants"][0][label]["values"]
    smoke = audit_smoke(out)
    current = {p.name: sha(p) for p in sorted(STUDY.iterdir()) if p.is_file()}
    raw = {p.relative_to(out).as_posix(): sha(p) for p in sorted(out.rglob("*"))
           if p.is_file() and p != destination}
    timing_names = ("summary.json", "timing-plan.json", "timing-status.txt", "native-execution.json")
    result = {
        "status": "PASS", "phase": "CORRECTNESS_ONLY",
        "formal_performance_gate": "DEFERRED_USER_ACTIVE_DESKTOP",
        "audited_utc": datetime.now(timezone.utc).isoformat(),
        "pin": builds["pin"], "workspace_by_mode": workspaces,
        "same_image_switch": True, "common_binary_sha256": builds["common_binary_sha256"],
        "mode_environment_records": {mode: variants[mode]["environment"] for mode in MODES},
        "binary_hashes": {mode: sha(Path(variants[mode]["binary"])) for mode in MODES},
        "build_record_sha256": sha(out / "builds.json"),
        "build_provenance": {key: builds[key] for key in (
            "builder_sha256", "build_checks_sha256", "overlay_inputs", "primary_builds_sha256",
            "common_source_diff_sha256", "common_patched_files", "common_source_hashes")},
        "preserved_binary_count": len(preserved), "preserved_binaries": preserved,
        "source_snapshot_matches": True, "default_checkout_changed": False,
        "probes": probes, "startup_smoke": smoke,
        "study_file_hashes": current,
        "study_file_hash_scope": "Current research files at audit time, not a claim that every file was executed. Build execution provenance is checked separately against recorded builder, checks and overlay hashes.",
        "raw_evidence_hashes": raw,
        "raw_evidence_hash_scope": "All current files recursively relative to this evidence folder, excluding correctness.json itself; inclusion in this map does not assert a timing result passed.",
        "timing_artifacts_present_but_not_audited": [name for name in timing_names if (out / name).exists()],
        "limitations": [
            "No benchmark or build is launched by this audit; no performance conclusion is made.",
            "Mode environments are verified build-driver records, not independent observations inside each historical child process.",
            "Finite value, alias, continuation and resource checks do not prove full contextual equivalence.",
            "Same executable bytes fix relative code layout; cached-switch overhead and ASLR remain experimental boundaries.",
        ],
    }
    destination.write_text(json.dumps(result, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    print(json.dumps({"status": "PASS", "phase": result["phase"],
                      "formal_performance_gate": result["formal_performance_gate"],
                      "workspace_by_mode": workspaces, "preserved_binary_count": len(preserved),
                      "raw_files": len(raw)}, indent=2))


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("evidence", type=Path)
    audit(parser.parse_args().evidence.resolve())
