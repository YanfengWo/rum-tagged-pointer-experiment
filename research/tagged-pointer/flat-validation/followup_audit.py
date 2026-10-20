"""Audit post-primary value, static-code and fresh-session diagnostics."""
from fractions import Fraction
import hashlib
import importlib.util
import json
from pathlib import Path
from statistics_helpers import compare, rows


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def load(path):
    return json.loads(path.read_text(encoding="utf-8"))


def generator(study, name):
    spec = importlib.util.spec_from_file_location(name.replace("-", "_"), study / (name + ".py"))
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def audit(out, builds, primary):
    study = Path(__file__).resolve().parent
    variants = {v["mode"]: v for v in builds["variants"]}
    checks = load(out / "trim-values.json")
    assert checks["status"] == "PASS" and checks["checks"] == 24 and checks["processes"] == 4
    assert checks["timing_operations"] is False and checks["builds_sha256"] == sha(out / "builds.json")
    assert checks["script_sha256"] == sha(study / "check-trim-values.py")
    expected, cases = generator(study, "check-trim-values").source_and_cases()
    assert checks["cases"] == cases and (out / checks["source"]["file"]).read_text() == expected
    assert checks["source"]["sha256"] == sha(out / checks["source"]["file"])
    assert {r["mode"] for r in checks["runs"]} == set(variants)
    for record in checks["runs"]:
        assert record["status"] == "PASS" and record["returncode"] == 0
        assert record["binary_sha256"] == variants[record["mode"]]["binary_sha256"]
        for stream in ["stdout", "stderr"]:
            assert sha(out / record[stream]) == record[stream + "_sha256"]
        text = (out / record["stdout"]).read_text().replace("\\t", "\t")
        assert "TRIM_VALUES_PASS" in text and not (out / record["stderr"]).read_text().strip()
        assert record["labels"] == [c["label"] for c in cases]
    static = load(out / "static-comparison.json")
    assert static["status"] == "PASS" and static["builds_sha256"] == sha(out / "builds.json")
    for name, data in static["binaries"].items():
        assert data["binary_sha256"] == variants[name]["binary_sha256"]
        assert sha(out / f"{name}-static-disassembly.log") == data["disassembly_log_sha256"]
        for function in data["functions"].values():
            code = "\n".join(function["normalized_instructions"]).encode()
            assert hashlib.sha256(code).hexdigest() == function["normalized_sha256"]
    for name, record in static["source"]["files"].items():
        assert sha(Path(builds["source_root"]) / name) == record["sha256"]
    fresh = load(out / "fresh-plan.json")
    assert fresh["status"] == "PASS" and fresh["blocks"] == 8
    assert fresh["primary_plan_sha256"] == sha(out / "timing-plan.json")
    assert fresh["orders"] == primary["orders"] and fresh["binaries"] == primary["binaries"]
    template = out / "suite-01-base-a/core-language-microbench.rum"
    assert fresh["parent_generated_sha256"] == sha(template)
    expected, fragments = generator(study, "run-fresh-trim").source(template.read_text())
    assert (out / "fresh-source.rum").read_text() == expected and fresh["fragment_hashes"] == fragments
    assert fresh["source_sha256"] == sha(out / "fresh-source.rum")
    progress = rows(out / "fresh-progress.tsv")
    assert len(progress) == 128
    values = {}
    for block, order in enumerate(fresh["orders"], 1):
        for position, mode in enumerate(order, 1):
            journal = [r for r in progress if int(r["block"]) == block and r["mode"] == mode]
            assert [r["status"] for r in journal] == ["started", "passed"]
            assert all(int(r["position"]) == position for r in journal)
            prefix = out / f"fresh-{block:02d}-{mode}"
            assert not prefix.with_suffix(".stderr").read_text().strip()
            text = prefix.with_suffix(".stdout").read_text().replace("\\t", "\t")
            assert "FRESH_STRINGTRIM_PASS" in text
            found = []
            for line in text.splitlines():
                line = line.strip().strip('"')
                if not line.startswith("RUM_CORE_MICRO\t"):
                    continue
                _, label, count, seconds = line.split("\t")
                assert label not in found and int(count) == 500
                found.append(label)
                value = float(Fraction(seconds)) if "/" in seconds else float(seconds.replace("*^", "e"))
                assert value > 0
                values[block, mode, label] = value
            assert found == fresh["labels"]
    results = [compare(label, values, 8, primary["modes"]) for label in fresh["labels"]]
    return {"trim_values": {"status": "PASS", "checks": 24, "processes": 4, "timing_operations": False},
            "static_review": {"sha256": sha(out / "static-comparison.json"), "scope": static["scope"],
                              "limitations": static["limitations"],
                              "selected_roots": [{"left": c["left"], "right": c["right"],
                                  "functions": [r for r in c["common"] if r["role"] == "selected-root"]}
                                  for c in static["comparisons"]]},
            "fresh_context": {"plan": fresh, "processes": 64, "original_wrapper_minima": 384,
                              "selected_after_primary": True, "results": results}}
