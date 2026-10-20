"""Verify the completed uninstrumented build and workspace records."""
import hashlib
import json
from pathlib import Path
import re
from statistics_helpers import IMPLEMENTATIONS


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def audit(out):
    builds = json.loads((out / "builds.json").read_text(encoding="utf-8"))
    assert builds["status"] == "PASS" and builds["prior_binaries_preserved"]
    assert builds["pin"] == "77029938f999b14dbe5b0755bebdf98e51517ae6"
    assert not builds["default_checkout_changed"] and not builds["instrumentation_present"]
    assert builds["jobs"] == 2 and builds["source_restored"]
    assert builds["builder_sha256"] == sha(Path(__file__).with_name("build.py"))
    variants = {item["mode"]: item for item in builds["variants"]}
    assert set(variants) == set(IMPLEMENTATIONS)
    checks = {}
    for name, item in variants.items():
        assert item["status"] == "PASS"
        assert sha(Path(item["binary"])) == item["binary_sha256"]
        assert sha(out / f"{name}-source-diff.patch") == item["source_diff_sha256"]
        assert not (out / f"{name}-format.log").read_text().strip()
        text = (out / f"{name}-workspace.log").read_text(encoding="utf-8")
        groups = re.findall(r"test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;", text)
        assert groups and all(int(row[1]) == 0 for row in groups)
        result = {"passed": sum(int(row[0]) for row in groups),
                  "ignored": sum(int(row[2]) for row in groups), "groups": len(groups)}
        assert result == {"passed": 2241, "ignored": 7, "groups": 29}, result
        checks[name] = result
        assert "COPY_ATTRIBUTION_ACCEPTANCE_PASS" in (out / f"{name}-acceptance.stdout").read_text()
        assert not (out / f"{name}-acceptance.stderr").read_text().strip()
        for probe, marker, prefix in [("resource", "RESOURCE_PROBE_VALUE_ALIAS_PASS", "RESOURCE_PROBE"),
                                      ("capacity", "CAPACITY_VALUE_PASS", "CAPACITY_OBSERVATION")]:
            meta = item[probe]
            assert sha(Path(meta["source"])) == meta["source_sha256"] and meta["exit_code"] == 0
            text = (out / f"{name}-{probe}.stdout").read_text().replace("\\t", "\t")
            assert marker in text and not (out / f"{name}-{probe}.stderr").read_text().strip()
            observed = {}
            for line in text.splitlines():
                if line.startswith(prefix + "\t"):
                    _, key, value = line.split("\t")
                    assert key not in observed
                    observed[key] = int(value)
            assert observed == meta["values"]
    for item in builds["preserved_binaries"]:
        assert sha(Path(item["path"])) == item["sha256"]
    return builds, variants, checks
