"""Audit saved correctness evidence without running rum or timings."""
import argparse
import hashlib
import json
from pathlib import Path
import re

STUDY = Path(__file__).resolve().parent
PRIMARY = STUDY.parent / "rewrite-shortcuts"
PIN = "77029938f999b14dbe5b0755bebdf98e51517ae6"
COUNTS = {"passed": 2262, "ignored": 7, "groups": 29}
RUNTIME = "crates/rum-runtime/src/runtime/"
SCANNER = RUNTIME + "definitions/application/replace_all.rs"
SUBSTITUTION = RUNTIME + "substitution/binding_substitution.rs"


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def text(path):
    return path.read_text(encoding="utf-8")


def read_json(path):
    return json.loads(text(path))


def hashes(root):
    return {p.relative_to(root).as_posix(): sha(p)
            for p in sorted(root.rglob("*")) if p.is_file()}


def check_inputs(record, root):
    assert sha(root / "build.py") == record["builder_sha256"]
    for name, digest in record["overlay_inputs"].items():
        assert sha(root / name) == digest, name


def test_sources():
    sources = [("rewrite_shortcuts_" + n, PRIMARY / (n + "-tests.rs"))
               for n in ("matcher", "depth")]
    sources.append(("single_binding_regression", STUDY / "regression-tests.rs"))
    result = {}
    for module, path in sources:
        names = re.findall(r"#\[test\]\s*fn (\w+)\(", text(path))
        assert names
        result[module] = names
    assert len(result["single_binding_regression"]) == 6
    assert 2241 + sum(map(len, result.values())) == 2262
    return result


def workspace(path, tests):
    log = text(path)
    rows = re.findall(r"test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;", log)
    result = {"passed": sum(int(r[0]) for r in rows),
              "ignored": sum(int(r[2]) for r in rows), "groups": len(rows)}
    assert result == COUNTS and all(int(r[1]) == 0 for r in rows), path
    assert "test result: FAILED" not in log
    for module, names in tests.items():
        for name in names:
            assert f"{module}::{name} ... ok" in log, name
    return result


def observations(text, prefix, count):
    result = {}
    for line in text.replace("\\t", "\t").splitlines():
        if line.startswith(prefix + "\t"):
            _, key, value = line.split("\t")
            assert key not in result
            result[key] = int(value)
    assert len(result) == count and all(v > 0 for v in result.values())
    return result


def check_diff(path, item, source, original):
    """Reconstruct saved formatted changes in memory."""
    assert sha(path) == item["source_diff_sha256"]
    lines = text(path).splitlines(keepends=True)
    index, changed = 0, set()
    while index < len(lines):
        assert lines[index].startswith("--- a/")
        name = lines[index][6:].rstrip("\n")
        assert lines[index + 1] == "+++ b/" + name + "\n"
        changed.add(name)
        old = text(source / name).splitlines(keepends=True)
        result, cursor = [], 0
        index += 2
        while index < len(lines) and not lines[index].startswith("--- a/"):
            match = re.match(r"@@ -(\d+)(?:,\d+)? \+\d+(?:,\d+)? @@", lines[index])
            assert match, lines[index]
            start = int(match[1]) - 1
            assert start >= cursor
            result.extend(old[cursor:start])
            cursor, index = start, index + 1
            while index < len(lines) and not lines[index].startswith(("@@ ", "--- a/")):
                line = lines[index]
                assert line[:1] in (" ", "-", "+")
                if line[0] != "+":
                    assert old[cursor] == line[1:]
                    cursor += 1
                if line[0] != "-":
                    result.append(line[1:])
                index += 1
        result.extend(old[cursor:])
        data = "".join(result).encode("utf-8")
        saved = item["patched_files"][name]
        assert original[name] == saved["uninstrumented_base_sha256"]
        assert hashlib.sha256(data).hexdigest() == saved["source_sha256"]
        assert len(data) == saved["bytes"]
    assert changed == set(item["patched_files"])


def audit(out):
    builds = read_json(out / "builds.json")
    assert builds["status"] == "PASS" and builds["stage"] == "COMPLETE"
    assert builds["pin"] == PIN and builds["toolchain"] == "1.85.0"
    assert builds["expected_workspace"] == COUNTS
    assert builds["source_restored"] and builds["source_entries_checked"]
    assert builds["prior_binaries_preserved"] and not builds["default_checkout_changed"]
    assert not builds["instrumentation_present"] and not builds["same_image_switch"]
    check_inputs(builds, STUDY)
    assert sha(STUDY / "build_checks.py") == builds["build_checks_sha256"]
    primary_path = Path(builds["primary_builds"])
    assert sha(primary_path) == builds["primary_builds_sha256"]
    primary = read_json(primary_path)
    assert primary["status"] == "PASS" and primary["pin"] == PIN
    check_inputs(primary, PRIMARY)
    switch_info = builds["switch_builds"]
    switch_path = Path(switch_info["path"])
    assert switch_info["present"] and sha(switch_path) == switch_info["sha256"]
    switch = read_json(switch_path)
    assert switch["status"] == "PASS" and switch["same_image_switch"]
    assert switch["common_binary_sha256"] == switch_info["common_binary_sha256"]
    check_inputs(switch, STUDY.parent / "rewrite-switch")
    preserved = {v["path"]: v["sha256"] for v in primary["preserved_binaries"]}
    preserved.update({v["binary"]: v["binary_sha256"] for v in primary["variants"]})
    assert len(preserved) == 7
    preserved[str(Path(switch["target_root"]) / "release/rum-cli")] = switch["common_binary_sha256"]
    for item in switch["variants"]:
        assert item["status"] == "PASS" and item["binary_sha256"] == switch["common_binary_sha256"]
        preserved[item["binary"]] = item["binary_sha256"]
    assert len(preserved) == 12 and len(builds["preserved_binaries"]) == 12
    assert preserved == {v["path"]: v["sha256"] for v in builds["preserved_binaries"]}
    assert all(sha(Path(p)) == digest for p, digest in preserved.items())
    source = Path(builds["source_root"])
    original = builds["common_original_source_hashes"]
    assert hashes(source) == original
    overlay = read_json(out / "base-overlay.json")
    assert overlay["upstream_commit"] == PIN and overlay["mode"] == "base"
    assert not overlay["default_source_changed"]
    tests = test_sources()
    for module in tests:
        assert RUNTIME + "tests/" + module + ".rs" in original
    expected = {"base": set(), "matcher": {SCANNER}, "single": {SCANNER, SUBSTITUTION}}
    variants = {v["mode"]: v for v in builds["variants"]}
    assert len(builds["variants"]) == 3 and set(variants) == set(expected)
    assert len({v["binary_sha256"] for v in variants.values()}) == 3
    checks, output = {}, {}
    for mode, item in variants.items():
        assert item["status"] == "PASS" and set(item["patched_files"]) == expected[mode]
        assert sha(Path(item["binary"])) == item["binary_sha256"]
        check_diff(out / f"{mode}-source-diff.patch", item, source, original)
        assert not text(out / f"{mode}-format.log").strip()
        checks[mode] = workspace(out / f"{mode}-workspace.log", tests)
        assert checks[mode] == item["workspace"]
        for label, marker in (("acceptance", "COPY_ATTRIBUTION_ACCEPTANCE_PASS"),
                              ("resource", "RESOURCE_PROBE_VALUE_ALIAS_PASS"),
                              ("capacity", "CAPACITY_VALUE_PASS"), ("semantic", "ISO_SEMANTICS_PASS")):
            probe = item[label]
            assert probe["exit_code"] == 0 and sha(Path(probe["source"])) == probe["source_sha256"]
            stdout = text(out / f"{mode}-{label}.stdout")
            assert marker in stdout and not text(out / f"{mode}-{label}.stderr").strip()
            if mode == "base":
                output[label] = stdout
            else:
                assert stdout == output[label], (mode, label)
            if label in ("resource", "capacity"):
                prefix, count = ("RESOURCE_PROBE", 7) if label == "resource" else ("CAPACITY_OBSERVATION", 2)
                assert observations(stdout, prefix, count) == probe["values"]
                assert probe["values"] == primary["variants"][0][label]["values"]
    research = {p.relative_to(STUDY.parent).as_posix(): sha(p)
                for folder in (STUDY, PRIMARY, STUDY.parent / "rewrite-switch")
                for p in folder.rglob("*") if p.is_file() and "__pycache__" not in p.parts}
    metadata = (out / "builds.json", out / "base-overlay.json", primary_path, switch_path)
    raw = hashes(out)
    raw.pop("correctness.json", None)
    report = {"status": "PASS", "phase": "CORRECTNESS_ONLY",
              "formal_performance_gate": "DEFERRED_USER_ACTIVE_DESKTOP",
              "workspace": checks, "test_functions": tests,
              "single_binding_coverage": {"functions": 6, "new": 5, "duplicate_resume": 1},
              "source_entries_checked": len(original), "original_snapshot_restored": True,
              "preserved_binaries_checked": len(preserved),
              "binary_sha256": {m: v["binary_sha256"] for m, v in variants.items()},
              "metadata_sha256": {str(p): sha(p) for p in metadata},
              "research_files_sha256": research, "raw_files_sha256": raw,
              "limits": ["No performance or full contextual proof; allocations, reference counts and resource thresholds can differ."]}
    (out / "correctness.json").write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    print("SINGLE_BINDING_CORRECTNESS_ONLY_PASS")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("evidence", type=Path)
    audit(parser.parse_args().evidence.resolve())
