"""Keep timing inputs and live output on Linux storage; copy evidence afterward."""
import argparse
from datetime import datetime, timezone
import json
from pathlib import Path
import subprocess
import tempfile
import time


def main(out):
    study = Path(__file__).resolve().parent
    repo = study.parents[2]
    assert json.loads((out / "builds.json").read_text())["status"] == "PASS"
    assert not (out / "native-execution.json").exists()
    native = Path(tempfile.mkdtemp(prefix="rum-shortcuts-evidence-", dir="/var/tmp"))
    bounded = ["python3", str(repo / "scripts/run-bounded.py")]
    record = {"status": "RUNNING", "native_evidence": str(native), "local_evidence": str(out),
              "started_utc": datetime.now(timezone.utc).isoformat(), "idle_seconds_before_timing": 30,
              "method": "Preflight and all timing sources/live output on Linux storage. No evidence export or project build during timing. Copy retained files to the local workspace only after timing ends."}
    subprocess.run(bounded + ["--timeout", "120", "--", "cp", "-a", str(out) + "/.", str(native)], check=True, timeout=135)
    print("Native evidence:", native, flush=True)
    try:
        subprocess.run(bounded + ["--timeout", "300", "--", "python3", str(study / "run.py"), str(native), "--preflight"], check=True, timeout=315)
        print("Untimed preflight PASS; fixed 30-second idle interval", flush=True)
        time.sleep(30)
        subprocess.run(bounded + ["--timeout", "2400", "--", "python3", str(study / "run.py"), str(native)], check=True, timeout=2415)
        record["status"] = "PASS"
    except BaseException as error:
        record.update(status="FAILED", error_type=type(error).__name__, error=str(error))
        raise
    finally:
        record["finished_utc"] = datetime.now(timezone.utc).isoformat()
        (native / "native-execution.json").write_text(json.dumps(record, indent=2) + "\n")
        subprocess.run(bounded + ["--timeout", "120", "--", "cp", "-a", str(native) + "/.", str(out)], check=True, timeout=135)
    print("Native timing complete; retained evidence copied to workspace", flush=True)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("evidence", type=Path)
    main(parser.parse_args().evidence.resolve())
