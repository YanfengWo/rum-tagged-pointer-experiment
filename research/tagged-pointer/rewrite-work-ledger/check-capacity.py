"""Observe SameQ expressions constructed differently; no timing comparison."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess


def main(out):
    study = Path(__file__).resolve().parent
    repo = study.parents[2]
    builds = json.loads((out / "builds.json").read_text())
    assert builds["status"] == "PASS"
    assert not (out / "capacity-observations.json").exists()
    source = study / "capacity-observability.rum"
    binaries = {"original": builds["preserved_binaries"][0],
                "flat": next({"path": v["binary"], "sha256": v["binary_sha256"]} for v in builds["variants"] if v["mode"] == "flat")}
    env = {k: v for k, v in os.environ.items() if not k.startswith("RUM_")}
    records = {}
    for name, item in binaries.items():
        assert hashlib.sha256(Path(item["path"]).read_bytes()).hexdigest() == item["sha256"]
        result = subprocess.run(["python3", str(repo / "scripts/run-bounded.py"), "--timeout", "120", "--",
                                 item["path"], "--file", str(source)], env=env, stdin=subprocess.DEVNULL,
                                capture_output=True, timeout=135)
        (out / "raw" / f"capacity-{name}.stdout").write_bytes(result.stdout)
        (out / "raw" / f"capacity-{name}.stderr").write_bytes(result.stderr)
        assert result.returncode == 0 and b"CAPACITY_VALUE_PASS" in result.stdout and not result.stderr.strip()
        values = {}
        for line in result.stdout.decode().replace("\\t", "\t").splitlines():
            if line.startswith("CAPACITY_OBSERVATION\t"):
                _, key, value = line.split("\t")
                values[key] = int(value)
        assert set(values) == {"literal", "applied"}
        records[name] = {"binary_sha256": item["sha256"], "values": values}
    record = {"status": "PASS", "source_sha256": hashlib.sha256(source.read_bytes()).hexdigest(),
              "results": records, "scope": "Both SameQ value checks pass before printing resource observations. No cross-version numerical-equivalence assumption or timing inference."}
    (out / "capacity-observations.json").write_text(json.dumps(record, indent=2) + "\n")
    print(json.dumps(record, indent=2))


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("evidence", type=Path)
    main(parser.parse_args().evidence.resolve())
