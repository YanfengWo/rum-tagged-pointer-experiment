"""Small retained-sample parser and descriptive, block-level comparisons."""
import csv
from fractions import Fraction
import math
from statistics import median

MODES = ["base-a", "candidate-a", "base-b", "candidate-b"]


def rows(path):
    with path.open(encoding="utf-8", newline="") as source:
        return list(csv.DictReader(source, delimiter="\t"))


def describe(values):
    assert values and all(math.isfinite(v) and v > 0 for v in values)
    return {"median": median(values), "range": [min(values), max(values)],
            "below_one": sum(v < 1 for v in values), "above_one": sum(v > 1 for v in values),
            "per_block": values}


def retained(path, labels, sentinel):
    text = path.read_text(encoding="utf-8").replace("\\t", "\t")
    assert sentinel in text
    groups = {label: {} for label in labels}
    for line in text.splitlines():
        line = line.strip().strip('"')
        if not line.startswith("ISO_TIME\t"):
            continue
        _, label, rep, seconds = line.split("\t")
        rep = int(rep)
        assert label in groups and rep not in groups[label]
        seconds = float(Fraction(seconds)) if "*^" not in seconds else float(seconds.replace("*^", "e"))
        assert math.isfinite(seconds) and seconds > 0
        groups[label][rep] = seconds
    assert all(set(v) == set(range(1, 12)) for v in groups.values())
    return {label: [groups[label][rep] for rep in range(1, 12)] for label in labels}


def compare(label, values, blocks):
    ratios = {"candidate/base": [], "base-b/base-a": [], "candidate-b/candidate-a": [],
              "candidate-a/base-a": [], "candidate-b/base-b": []}
    for block in range(1, blocks + 1):
        a, b, c, d = [values[block, m, label] for m in MODES]
        ratios["candidate/base"].append(math.sqrt(b * d / (a * c)))
        ratios["base-b/base-a"].append(c / a)
        ratios["candidate-b/candidate-a"].append(d / b)
        ratios["candidate-a/base-a"].append(b / a)
        ratios["candidate-b/base-b"].append(d / c)
    return {"label": label, "ratios": {k: describe(v) for k, v in ratios.items()},
            "seconds_by_block": {mode: [values[b, mode, label] for b in range(1, blocks + 1)] for mode in MODES}}
