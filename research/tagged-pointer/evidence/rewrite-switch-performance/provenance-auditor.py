"""Check historical correctness inventories against exact prelaunch source copies."""
import argparse
import hashlib
import json
from pathlib import Path


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def load(path):
    return json.loads(path.read_text(encoding="utf-8"))


def audit(out, research, stage):
    capsule = load(out / "provenance-capsule.json")
    prelaunch = load(out / "prelaunch-inputs.json")
    correctness = load(out / "correctness.json")
    assert capsule["status"] == correctness["status"] == "PASS"
    assert capsule["correctness_baseline_commit"] == "9f6bf08b8378cb0cd9cd0ab26eb5148b82091608"
    assert sha(out / "prelaunch-inputs.json") == capsule["prelaunch_inputs_sha256"]
    actual = {p.relative_to(out).as_posix() for p in (out / "prelaunch-study-files").rglob("*") if p.is_file()}
    actual.add("monitor-executed.py")
    assert actual == set(capsule["files_sha256"])
    assert all(sha(out / name) == digest for name, digest in capsule["files_sha256"].items())
    assert sha(out / "monitor-executed.py") == prelaunch["monitor_sha256"]
    baseline = out / "prelaunch-study-files"
    for name, digest in prelaunch["study_files_sha256"].items():
        assert sha(baseline / stage / name) == digest
    inventory = correctness.get("research_files_sha256")
    if inventory is None:
        inventory = {stage + "/" + name: digest
                     for name, digest in correctness["study_file_hashes"].items()}
    for name, digest in inventory.items():
        assert sha(baseline / name) == digest, name
    allowed = {"single-binding/README.md", "rewrite-switch/README.md"}
    changed = []
    for name in sorted(actual - {"monitor-executed.py"}):
        relative = Path(name).relative_to("prelaunch-study-files").as_posix()
        current = research / relative
        if sha(current) != capsule["files_sha256"][name]:
            assert relative in allowed, relative
            changed.append(relative)
    native = load(out / "native-execution.json")
    session = load(out / "lock-state.json")
    assert native["status"] == "PASS" and session["exit_code"] == 0
    assert (out / "timing-status.txt").read_text().strip() == "TIMINGS_PASS"
    console = (out / "monitor-runner.stdout").read_bytes()
    nul_bytes = console.count(b"\x00")
    if stage == "single-binding":
        limitation = load(out / "monitor-log-limitation.json")
        assert nul_bytes == limitation["stdout_nul_bytes"] == 424
    else:
        assert nul_bytes == 0
        assert "PASS: 64 complete suites" in console.decode("utf-8")
    result = {
        "status": "PASS", "stage": stage,
        "capsule_files_checked": len(actual), "historical_inventory_files_checked": len(inventory),
        "changed_since_prelaunch": changed,
        "change_scope": "Only the two post-run results READMEs may differ; all measured study code and patches match prelaunch copies.",
        "capsule_sha256": sha(out / "provenance-capsule.json"),
        "correctness_sha256": sha(out / "correctness.json"),
        "prelaunch_inputs_sha256": sha(out / "prelaunch-inputs.json"),
        "executed_monitor_sha256": sha(out / "monitor-executed.py"),
        "outer_console_nul_bytes": nul_bytes,
        "auditor_sha256": sha(Path(__file__).resolve()),
        "limits": "Integrity and finite experiment audit, not a full semantic proof or a timing-noise correction.",
    }
    (out / "provenance-audit.json").write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
    print(stage, "PERFORMANCE_PROVENANCE_PASS", len(actual), "source copies")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("evidence", type=Path)
    parser.add_argument("--research", required=True, type=Path)
    parser.add_argument("--stage", required=True, choices=["single-binding", "rewrite-switch"])
    args = parser.parse_args()
    audit(args.evidence.resolve(), args.research.resolve(), args.stage)
