"""Shared build-log checks for the isolated rewrite experiment."""
from pathlib import Path
import re
import hashlib


def expected_tests():
    study = Path(__file__).resolve().parent
    return 2241 + sum((study / f"{name}-tests.rs").read_text().count("#[test]")
                      for name in ("matcher", "depth"))


def workspace(path):
    groups = re.findall(r"test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;",
                        path.read_text(encoding="utf-8"))
    result = {"passed": sum(int(r[0]) for r in groups),
              "ignored": sum(int(r[2]) for r in groups), "groups": len(groups)}
    assert groups and all(int(r[1]) == 0 for r in groups), path
    assert result == {"passed": expected_tests(), "ignored": 7, "groups": 29}, result
    return result


def observations(output, prefix, count):
    result = {}
    for line in output.decode("utf-8").replace("\\t", "\t").splitlines():
        if line.startswith(prefix + "\t"):
            _, key, value = line.split("\t")
            assert key not in result
            result[key] = int(value)
    assert len(result) == count and all(value > 0 for value in result.values())
    return result

def digest(data):
    return hashlib.sha256(data).hexdigest()


def sha(path):
    return digest(path.read_bytes())


def files(root):
    return {p.relative_to(root).as_posix(): p.read_bytes()
            for p in root.rglob("*") if p.is_file()}


