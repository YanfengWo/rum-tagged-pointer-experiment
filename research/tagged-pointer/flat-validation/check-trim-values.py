"""Untimed full-value/alias checks supplement the unchanged StringTrim suite."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
sys.dont_write_bytecode = True

MODES = ("base", "wide", "flat", "combined")
FAMILIES = ("literal-list", "single-literal", "whitespace")


def digest(data):
    return hashlib.sha256(data).hexdigest()


def sha(path):
    return digest(path.read_bytes())


def source_and_cases():
    source = "(* Untimed; every expected input and result below is a complete string literal. *)\n"
    cases = []
    for family in FAMILIES:
        for size in (4000, 32000):
            expected = "x" * (size - 2)
            left, right, trim = {
                "literal-list": ("1", "2", ', {"1","2"}'),
                "single-literal": ("1", "1", ', "1"'),
                "whitespace": (" ", " ", ""),
            }[family]
            original = left + expected + right
            label = f"stringtrim-{family}-{size}"
            source += (
                f'trimInput = StringJoin["{left}",StringRepeat["x",{size - 2}],"{right}"];\n'
                'trimAlias = trimInput;\n'
                f'trimExpectedInput = "{original}";\n'
                f'trimExpectedResult = "{expected}";\n'
                f'trimResult = StringTrim[trimInput{trim}];\n'
                'If[SameQ[trimResult,trimExpectedResult],Null,Exit[2]];\n'
                'If[SameQ[trimInput,trimExpectedInput],Null,Exit[2]];\n'
                'If[SameQ[trimAlias,trimExpectedInput],Null,Exit[2]];\n'
                f'Print["TRIM_VALUE_PASS\\t{label}"];\n'
            )
            cases.append({"label": label, "family": family, "size": size,
                          "input_sha256": digest(original.encode()),
                          "expected_sha256": digest(expected.encode()),
                          "expected_length": size - 2})
    return source + 'Print["TRIM_VALUES_PASS"];\n', cases


def main(out):
    study = Path(__file__).resolve().parent
    repo = study.parents[2]
    builds_file = out / "builds.json"
    builds = json.loads(builds_file.read_text(encoding="utf-8"))
    assert builds["status"] == "PASS" and builds["instrumentation_present"] is False
    variants = {item["mode"]: item for item in builds["variants"]}
    assert set(variants) == set(MODES)
    record_file = out / "trim-values.json"
    assert not record_file.exists(), "Preserve the previous value-check record rather than overwriting it"
    content, cases = source_and_cases()
    source = out / "trim-values.rum"
    if source.exists():
        assert source.read_text(encoding="utf-8") == content
    else:
        source.write_text(content, encoding="utf-8")
    assert "AbsoluteTiming[" not in content and "Timing[" not in content
    record = {"status": "IN_PROGRESS", "recorded_on": "2026-10-19",
              "builds_sha256": sha(builds_file), "script_sha256": sha(Path(__file__)),
              "source": {"file": source.name, "sha256": sha(source)},
              "cases": cases, "runs": [], "processes": 0, "checks": 0,
              "timing_operations": False,
              "method": "Six same-family StringTrim inputs per uninstrumented implementation; three complete-literal SameQ assertions check result, input and original string alias. No timing and no original benchmark changes.",
              "limits": "These checks cover the six exact ASCII inputs, not all StringTrim behavior; they do not establish the cause of measured regressions."}

    def save():
        record_file.write_text(json.dumps(record, indent=2) + "\n", encoding="utf-8")

    env = {key: value for key, value in os.environ.items() if not key.startswith("RUM_")}
    bounded = ["python3", str(repo / "scripts/run-bounded.py"), "--timeout", "120", "--"]
    expected_labels = [case["label"] for case in cases]
    save()
    try:
        for mode in MODES:
            item = variants[mode]
            binary = Path(item["binary"])
            assert os.access(binary, os.X_OK) and sha(binary) == item["binary_sha256"]
            assert b"RUM_REWRITE_WORK_LEDGER" not in binary.read_bytes()
            result = subprocess.run(bounded + [str(binary), "--file", str(source)], cwd=repo, env=env,
                                    stdin=subprocess.DEVNULL, capture_output=True, timeout=135)
            stdout = out / f"trim-values-{mode}.stdout"
            stderr = out / f"trim-values-{mode}.stderr"
            stdout.write_bytes(result.stdout)
            stderr.write_bytes(result.stderr)
            lines = [line.strip().strip('"').replace("\\t", "\t")
                     for line in result.stdout.decode("utf-8").splitlines()]
            labels = [line.split("\t", 1)[1] for line in lines if line.startswith("TRIM_VALUE_PASS\t")]
            passed = (result.returncode == 0 and not result.stderr.strip()
                      and labels == expected_labels and lines.count("TRIM_VALUES_PASS") == 1
                      and all(line in ("", "Null", "TRIM_VALUES_PASS")
                              or line.startswith("TRIM_VALUE_PASS\t") for line in lines))
            record["runs"].append({"mode": mode, "binary": str(binary),
                                   "binary_sha256": sha(binary), "returncode": result.returncode,
                                   "stdout": stdout.name, "stderr": stderr.name,
                                   "stdout_sha256": sha(stdout), "stderr_sha256": sha(stderr),
                                   "labels": labels, "status": "PASS" if passed else "FAIL"})
            record["processes"] += 1
            record["checks"] += len(labels) if passed else 0
            save()
            assert passed and sha(binary) == item["binary_sha256"], mode
        assert record["checks"] == 24 and record["processes"] == 4
        assert sha(source) == record["source"]["sha256"] and sha(builds_file) == record["builds_sha256"]
        record["status"] = "PASS"
        save()
    except Exception as error:
        record["status"] = "FAIL"
        record["error"] = repr(error)
        save()
        raise
    print("PASS: 24 complete StringTrim result/input/alias checks across four uninstrumented binaries; no timings")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("evidence", type=Path)
    args = parser.parse_args()
    main(args.evidence.resolve())
