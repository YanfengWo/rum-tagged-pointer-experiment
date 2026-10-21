"""Audit sampled Windows session flags, without aligning Windows/WSL clocks."""
from collections import Counter
import json


def audit(out):
    data = json.loads((out / "lock-state.json").read_text())
    protocol = json.loads((out / "session-protocol.json").read_text())
    assert data["exit_code"] == 0 and protocol["recorded_before_timing"]
    samples = data["samples"]
    assert samples and all(row["state"] in {"locked", "unlocked"} for row in samples)
    changes = [row for i, row in enumerate(samples) if i and row["state"] != samples[i - 1]["state"]]
    return {"samples": len(samples), "counts": dict(Counter(row["state"] for row in samples)),
            "transitions": changes, "interval_seconds": data["interval_seconds"],
            "limits": "Discrete outer-process samples, not continuous proof. No cross-clock timestamp alignment. Comparisons are conditional on the observed session state, not a lock/unlock causal comparison."}
