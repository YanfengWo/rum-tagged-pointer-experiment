"""Pinned-input preservation and semantic checks for the binding experiment."""
import difflib
import json
from pathlib import Path
import re

from build_support import sha, digest, files


def variant_source(source, original, mode, primary, study, out, run):
    patches = [primary / "matcher.patch"] if mode != "base" else []
    if mode == "single":
        patches.append(study / "candidate.patch")
    for patch in patches:
        run(["git", "apply", "--check", str(patch)], 30, cwd=source)
        run(["git", "apply", str(patch)], 30, cwd=source)
    run(["cargo", "+1.85.0", "fmt"], 120, cwd=source)
    assert set(files(source)) == set(original)
    changed = sorted(n for n, d in original.items() if (source / n).read_bytes() != d)
    allowed = {"crates/rum-runtime/src/runtime/" + n for n in (
        "definitions/application/replace_all.rs", "substitution/binding_substitution.rs")}
    assert set(changed) == (set() if mode == "base" else
                           {n for n in allowed if mode == "single" or "replace_all.rs" in n})
    diff = "".join("".join(difflib.unified_diff(
        original[n].decode().splitlines(keepends=True),
        (source / n).read_text().splitlines(keepends=True), fromfile="a/" + n, tofile="b/" + n))
                   for n in changed)
    (out / f"{mode}-source-diff.patch").write_text(diff, encoding="utf-8")
    return {"mode": mode, "status": "RUNNING", "source_diff_sha256": digest(diff.encode()),
            "patched_files": {n: {"uninstrumented_base_sha256": digest(original[n]),
                "source_sha256": sha(source / n), "bytes": (source / n).stat().st_size} for n in changed}}


def preserved_binaries(primary_path, switch_path):
    primary = json.loads(primary_path.read_text(encoding="utf-8"))
    assert primary["status"] == "PASS" and primary["prior_binaries_preserved"]
    result = {item["path"]: item["sha256"] for item in primary["preserved_binaries"]}
    result.update({item["binary"]: item["binary_sha256"] for item in primary["variants"]})
    assert len(result) == 7
    switch_record = {"path": str(switch_path), "present": switch_path.exists()}
    if switch_path.exists():
        switch = json.loads(switch_path.read_text(encoding="utf-8"))
        assert switch["status"] == "PASS" and switch["same_image_switch"]
        assert switch["prior_binaries_preserved"] and not switch["default_checkout_changed"]
        common = switch["common_binary_sha256"]
        result[str(Path(switch["target_root"]) / "release/rum-cli")] = common
        for item in switch["variants"]:
            assert item["status"] == "PASS" and item["binary_sha256"] == common
            result[item["binary"]] = common
        switch_record.update(sha256=sha(switch_path), common_binary_sha256=common)
    assert all(sha(Path(path)) == digest for path, digest in result.items())
    return primary, result, switch_record


def workspace(path, primary, study):
    log = path.read_text(encoding="utf-8")
    groups = re.findall(r"test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;", log)
    result = {"passed": sum(int(r[0]) for r in groups),
              "ignored": sum(int(r[2]) for r in groups), "groups": len(groups)}
    assert result == {"passed": 2262, "ignored": 7, "groups": 29}
    assert all(int(r[1]) == 0 for r in groups)
    for module, source in [(f"rewrite_shortcuts_{name}", primary / f"{name}-tests.rs")
                           for name in ("matcher", "depth")] + [
                               ("single_binding_regression", study / "regression-tests.rs")]:
        for test in re.findall(r"#\[test\]\s*fn (\w+)\(", source.read_text(encoding="utf-8")):
            assert f"{module}::{test} ... ok" in log
    return result
