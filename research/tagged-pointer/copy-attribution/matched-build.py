"""Compile the candidate at the exact baseline source root, then restore bytes."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess

def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

def main(out):
    study = Path(__file__).resolve().parent
    repo = study.parents[2]
    assert (out / "checks-status.txt").read_text().strip() == "CHECKS_PASS"
    base_meta = json.loads((out / "base-final-overlay.json").read_text())
    candidate_meta = json.loads((out / "candidate-overlay.json").read_text())
    base = Path(base_meta["overlay_path"])
    name = "crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/once.rs"
    path = base / name
    original = path.read_bytes()
    assert sha(path) == candidate_meta["patched_files"][name]["base_sha256"]
    base_target = Path("/var/tmp/rum-copy-build/base-final")
    target = Path("/var/tmp/rum-copy-build/matched-candidate")
    assert not target.exists()
    subprocess.run(["cp", "-a", "--reflink=auto", str(base_target), str(target)], check=True)
    before = sha(base_target / "release/rum-cli")
    env = os.environ.copy()
    env.update(CARGO_TARGET_DIR=str(target), CARGO_BUILD_JOBS="4", CARGO_PROFILE_DEV_DEBUG="0")
    record = {"status": "RUNNING", "source_root": str(base), "baseline_binary_sha256": before,
              "baseline_source_sha256": sha(path), "changed_file": name,
              "candidate_expected_source_sha256": candidate_meta["patched_files"][name]["patched_sha256"],
              "same_local_source_root": True, "dependency_artifacts_reused": True}
    try:
        subprocess.run(["git", "apply", "--check", "-"], input=(study / "candidate.patch").read_text(), text=True, cwd=base, check=True)
        subprocess.run(["git", "apply", "-"], input=(study / "candidate.patch").read_text(), text=True, cwd=base, check=True)
        assert sha(path) == record["candidate_expected_source_sha256"]
        with (out / "matched-candidate-build.log").open("w") as log:
            subprocess.run(["python3", str(repo / "scripts/run-bounded.py"), "--timeout", "900", "--", "cargo", "+1.85.0", "build", "--release", "--locked", "--offline", "-p", "rum-cli"], cwd=base, env=env, stdout=log, stderr=subprocess.STDOUT, check=True, timeout=915)
        record["candidate_binary_sha256"] = sha(target / "release/rum-cli")
        with (out / "matched-candidate-acceptance.stdout").open("wb") as stdout, (out / "matched-candidate-acceptance.stderr").open("wb") as stderr:
            subprocess.run(["python3", str(repo / "scripts/run-bounded.py"), "--timeout", "120", "--", str(target / "release/rum-cli"), "--file", str(study / "acceptance.rum")], stdin=subprocess.DEVNULL, stdout=stdout, stderr=stderr, check=True, timeout=135)
        assert "COPY_ATTRIBUTION_ACCEPTANCE_PASS" in (out / "matched-candidate-acceptance.stdout").read_text()
        assert not (out / "matched-candidate-acceptance.stderr").read_text().strip()
        record["status"] = "PASS"
    finally:
        path.write_bytes(original)
        assert sha(path) == record["baseline_source_sha256"]
        assert sha(base_target / "release/rum-cli") == before
        record["baseline_source_restored"] = True
        record["baseline_binary_unchanged"] = True
        (out / "matched-build.json").write_text(json.dumps(record, indent=2) + "\n")
    (out / "matched-candidate-binary.sha256").write_text(record["candidate_binary_sha256"] + "  " + str(target / "release/rum-cli") + "\n")
    print("PASS: matched source-root candidate built; baseline source and binary preserved")

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("evidence", type=Path)
    args = parser.parse_args()
    main(args.evidence.resolve())
