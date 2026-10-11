"""Validate complete rotating samples and report within-round paired ratios."""
import csv
import math
from statistics import median

DATASETS = {"gMicroWideTraversal", "gMicroSortSymbolWidth16000", "gMicroAssociationPartWidth32000",
            "gMicroReplacePartRules", "rulesSimple", "synthetic-repeated-symbol-16000"}
CASES = {"safe-plain", "safe-shared", "compact-plain", "compact-shared", "original"}

def analyze(out):
    samples = {}
    for run in [1, 2, 3]:
        records = list(csv.DictReader((out / f"run-{run}.tsv").open(), delimiter="\t"))
        assert len(records) == 840
        assert not (out / f"run-{run}.stderr").read_text().strip()
        seen = set()
        scans = {}
        for record in records:
            dataset, op, case = record["dataset"], record["operation"], record["representation"]
            round_ = int(record["round"])
            assert dataset in DATASETS and case in CASES and op in {"scan", "build_drop", "cow"}
            assert 0 <= round_ <= 9
            assert not (op == "build_drop" and case == "original")
            n = int(record["iterations"])
            assert n == (200 if dataset == "rulesSimple" else 3 if op == "scan" else 2)
            ms, checksum = float(record["ms"]), int(record["checksum"])
            assert math.isfinite(ms) and ms > 0
            if op == "scan":
                key = (dataset, n)
                if key in scans: assert checksum == scans[key]
                scans[key] = checksum
            else: assert checksum == n
            key = (dataset, op, case, round_)
            assert key not in seen
            seen.add(key)
            if round_: samples[(run, *key)] = ms
        expected = {(d, op, c, round_) for d in DATASETS for op in ["scan", "build_drop", "cow"]
                    for c in CASES if not (op == "build_drop" and c == "original") for round_ in range(10)}
        assert seen == expected
    results = []
    for dataset in sorted(DATASETS):
        for op in ["scan", "build_drop", "cow"]:
            cases = CASES - ({"original"} if op == "build_drop" else set())
            values = {c: [samples[(run, dataset, op, c, round_)] for run in [1,2,3] for round_ in range(1,10)] for c in sorted(cases)}
            pairs = [("safe-shared", "safe-plain"), ("compact-shared", "compact-plain"),
                     ("compact-shared", "safe-shared"), ("compact-plain", "safe-plain")]
            if "original" in cases:
                pairs.extend([("compact-shared", "original"), ("safe-shared", "original")])
            ratios = {}
            for numerator, denominator in pairs:
                ratio = [a / b for a,b in zip(values[numerator], values[denominator])]
                ratios[f"{numerator}/{denominator}"] = {"paired_median": median(ratio),
                    "per_run_medians": [median(ratio[i*9:(i+1)*9]) for i in range(3)],
                    "range": [min(ratio),max(ratio)]}
            results.append({"dataset": dataset, "operation": op, "measured_samples_per_case": 27,
                "median_ms": {c: median(v) for c,v in values.items()},
                "sample_ms_range": {c: [min(v),max(v)] for c,v in values.items()}, "ratios": ratios})
    return {"raw_rows": 2520, "measured_rows": len(samples), "results": results}
