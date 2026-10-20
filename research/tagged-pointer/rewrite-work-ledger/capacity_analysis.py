"""Audit representation-sensitive resource observations separately from values."""
import hashlib
import json


def audit(out, builds, study):
    record = json.loads((out / "capacity-observations.json").read_text(encoding="utf-8"))
    assert record["status"] == "PASS"
    assert hashlib.sha256((study / "capacity-observability.rum").read_bytes()).hexdigest() == record["source_sha256"]
    expected = {"original": builds["preserved_binaries"][0]["sha256"],
                "flat": next(v["binary_sha256"] for v in builds["variants"] if v["mode"] == "flat")}
    for name, digest in expected.items():
        data = record["results"][name]
        assert data["binary_sha256"] == digest
        assert not (out / "raw" / f"capacity-{name}.stderr").read_text().strip()
        text = (out / "raw" / f"capacity-{name}.stdout").read_text().replace("\\t", "\t")
        assert "CAPACITY_VALUE_PASS" in text
        values = {}
        for line in text.splitlines():
            if line.startswith("CAPACITY_OBSERVATION\t"):
                _, key, value = line.split("\t")
                assert key not in values
                values[key] = int(value)
        assert values == data["values"] and set(values) == {"literal", "applied"}
    original, flat = [record["results"][name]["values"] for name in ["original", "flat"]]
    assert original["literal"] > original["applied"] > 0
    assert flat["literal"] == flat["applied"] == original["applied"]
    return record
