"""Replay base/candidate/base/candidate in one source and target directory."""
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
    previous = out.parent / "copy-attribution"
    baseline = Path(json.loads((previous / "base-final-overlay.json").read_text())["overlay_path"])
    change = "crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/once.rs"
    source = baseline / change
    original = source.read_bytes()
    prior = json.loads((previous / "matched-build.json").read_text())
    assert sha(source) == prior["baseline_source_sha256"]
    target = Path("/var/tmp/rum-regression-build/replay")
    saved = target.parent / "saved"
    assert not target.exists() and not saved.exists()
    out.mkdir(parents=True, exist_ok=True)
    assert not (out / "build-controls.json").exists()
    target.parent.mkdir(parents=True, exist_ok=True)
    subprocess.run(["cp", "-a", "--reflink=auto", "/var/tmp/rum-copy-build/base-final", str(target)], check=True)
    saved.mkdir()
    preserved = {str(path): sha(path) for path in [
        Path("/var/tmp/rum-copy-build/base-final/release/rum-cli"),
        Path("/var/tmp/rum-copy-build/matched-candidate/release/rum-cli"),
        Path("/var/tmp/rum-copy-build/candidate/release/rum-cli")]}
    env = os.environ.copy()
    env.update(CARGO_TARGET_DIR=str(target), CARGO_BUILD_JOBS="4", CARGO_PROFILE_DEV_DEBUG="0")
    record = {"status": "RUNNING", "source_root": str(baseline), "target": str(target),
              "baseline_source_sha256": sha(source), "expected_candidate_source_sha256": prior["candidate_expected_source_sha256"],
              "preserved_binaries": preserved, "builds": [], "method":
              "One source root and one target directory; forced first local-crate codegen then base/candidate/base/candidate rebuilds. Dependencies reused; all binaries saved before timing. Source restored in finally."}
    try:
        for path in (baseline / "crates").rglob("*.rs"):
            os.utime(path, None)
        for name in ["base-0", "candidate-0", "base-1", "candidate-1"]:
            source.write_bytes(original)
            if name.startswith("candidate"):
                subprocess.run(["git", "apply", "-"], input=(study.parent / "copy-attribution/candidate.patch").read_text(), text=True, cwd=baseline, check=True)
                assert sha(source) == record["expected_candidate_source_sha256"]
            else:
                assert sha(source) == record["baseline_source_sha256"]
            with (out / (name + "-build.log")).open("w") as log:
                subprocess.run(["python3", str(repo / "scripts/run-bounded.py"), "--timeout", "900", "--",
                                "cargo", "+1.85.0", "build", "--release", "--locked", "--offline", "-p", "rum-cli"],
                               cwd=baseline, env=env, stdout=log, stderr=subprocess.STDOUT, check=True, timeout=915)
            binary = saved / name
            shutil.copy2(target / "release/rum-cli", binary)
            text_file = saved / (name + ".text")
            subprocess.run(["objcopy", "--dump-section", ".text=" + str(text_file), str(binary)], check=True)
            item = {"mode": name, "binary": str(binary), "sha256": sha(binary),
                    "text_sha256": sha(text_file), "text_bytes": text_file.stat().st_size,
                    "source_sha256": sha(source)}
            record["builds"].append(item)
            (out / "build-controls.json").write_text(json.dumps(record, indent=2) + "\n")
            print(name, item["sha256"], flush=True)
        for item in record["builds"]:
            result = subprocess.run(["python3", str(repo / "scripts/run-bounded.py"), "--timeout", "120", "--",
                                     item["binary"], "--file", str(study.parent / "copy-attribution/acceptance.rum")],
                                    stdin=subprocess.DEVNULL, capture_output=True, timeout=135)
            (out / (item["mode"] + "-acceptance.stdout")).write_bytes(result.stdout)
            (out / (item["mode"] + "-acceptance.stderr")).write_bytes(result.stderr)
            assert result.returncode == 0 and b"COPY_ATTRIBUTION_ACCEPTANCE_PASS" in result.stdout and not result.stderr.strip()
        by_name = {b["mode"]: b for b in record["builds"]}
        record["base_replay_byte_identical"] = by_name["base-0"]["sha256"] == by_name["base-1"]["sha256"]
        record["candidate_replay_byte_identical"] = by_name["candidate-0"]["sha256"] == by_name["candidate-1"]["sha256"]
        record["base_matches_previous"] = by_name["base-0"]["sha256"] == prior["baseline_binary_sha256"]
        record["candidate_matches_previous"] = by_name["candidate-0"]["sha256"] == prior["candidate_binary_sha256"]
        record["status"] = "PASS"
    finally:
        source.write_bytes(original)
        assert sha(source) == record["baseline_source_sha256"]
        assert all(sha(Path(p)) == digest for p, digest in preserved.items())
        record["baseline_source_restored"] = True
        record["prior_binaries_unchanged"] = True
        (out / "build-controls.json").write_text(json.dumps(record, indent=2) + "\n")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("evidence", type=Path)
    main(parser.parse_args().evidence.resolve())
