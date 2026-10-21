"""Descriptive paired-block statistics; repetitions are not extra blocks."""
import csv
from fractions import Fraction
import math
from statistics import median

IMPLEMENTATIONS = ["base", "matcher", "single"]
COMPARISONS = [("matcher", "base"), ("single", "base"), ("single", "matcher")]


def rows(path):
    with path.open(encoding="utf-8", newline="") as source:
        return list(csv.DictReader(source, delimiter="\t"))


def describe(values):
    assert values and all(math.isfinite(v) and v > 0 for v in values)
    return {"median": median(values), "range": [min(values), max(values)],
            "below_one": sum(v < 1 for v in values),
            "above_one": sum(v > 1 for v in values), "per_block": values}


def retained(path, labels, sentinel, count=11):
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
        value = float(Fraction(seconds)) if "*^" not in seconds else float(seconds.replace("*^", "e"))
        assert math.isfinite(value) and value > 0
        groups[label][rep] = value
    assert all(set(v) == set(range(1, count + 1)) for v in groups.values())
    return {label: [groups[label][rep] for rep in range(1, count + 1)] for label in labels}


def compare(label, values, blocks, modes):
    implementation_seconds = {
        name: [math.sqrt(values[b, name + "-a", label] * values[b, name + "-b", label])
               for b in range(1, blocks + 1)] for name in IMPLEMENTATIONS}
    ratios = {}
    for numerator, denominator in COMPARISONS:
        ratios[numerator + "/" + denominator] = describe([
            n / d for n, d in zip(implementation_seconds[numerator], implementation_seconds[denominator])])
    for name in IMPLEMENTATIONS:
        ratios[name + "-b/" + name + "-a"] = describe([
            values[b, name + "-b", label] / values[b, name + "-a", label]
            for b in range(1, blocks + 1)])
    return {"label": label, "ratios": ratios,
            "seconds_by_block": {mode: [values[b, mode, label] for b in range(1, blocks + 1)] for mode in modes}}
