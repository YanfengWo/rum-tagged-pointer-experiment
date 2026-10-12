"""Validate the exact timing plan and summarize within-round paired ratios."""
import argparse
import csv
import json
import math
from pathlib import Path
from statistics import median

DATASETS = [
    "gMicroWideTraversal", "gMicroSortSymbolWidth16000",
    "gMicroAssociationPartWidth32000", "gMicroReplacePartRules", "rulesSimple",
    "synthetic-repeated-symbol-16000", "synthetic-generated-symbol-16000",
    "synthetic-strings-16000", "synthetic-deep-256", "synthetic-shared-args-200",
    "synthetic-manual-big-small-16000", "synthetic-big-large-16000", "synthetic-rational-16000",
]
CASES = ["original", "previous16", "safe24-borrow", "typed16-move", "typed16-borrow"]
OPERATIONS = ["scan_full", "scan_key", "build_drop", "cow", "clone_drop", "drop"]
HEADER = ["dataset", "round", "operation", "representation", "iterations", "ms", "checksum"]
RUNS = [1, 2, 3]
U64_MAX = (1 << 64) - 1


def require(condition, message):
    if not condition:
        raise ValueError(message)


def dataset_order():
    """main filters snapshots in frozen corpus order, then adds eight controls."""
    corpus = Path(__file__).resolve().parent.parent / "expression-census/corpus/index.json"
    entries = json.loads(corpus.read_text(encoding="utf-8"))
    selected = [e["label"] for e in entries if e["label"] in DATASETS]
    require(len(selected) == 5 and set(selected) == set(DATASETS[:5]),
            "Frozen corpus does not contain the five expected distinct datasets")
    return selected + DATASETS[5:]


def iterations(dataset, operation):
    return 200 if dataset == "rulesSimple" or operation == "clone_drop" else 3 if operation.startswith("scan_") else 2


def plan(run, datasets):
    for dataset in datasets:
        for round_ in range(10):
            for op_index, operation in enumerate(OPERATIONS):
                for offset in range(len(CASES)):
                    rotated = (offset + round_ + op_index + run) % len(CASES)
                    index = len(CASES) - 1 - rotated if run == 2 else rotated
                    yield dataset, round_, operation, CASES[index]


def analyze(evidence, prefix=""):
    evidence = Path(evidence)
    order = dataset_order()
    samples, scans, orders = {}, {}, {}
    for run in RUNS:
        path = evidence / f"{prefix}run-{run}.tsv"
        with path.open(encoding="utf-8", newline="") as stream:
            reader = csv.DictReader(stream, delimiter="\t")
            require(reader.fieldnames == HEADER, f"{path.name}: incorrect header")
            records = list(reader)
        require(len(records) == 3900, f"{path.name}: expected 3900 rows, got {len(records)}")
        stderr = evidence / f"{prefix}run-{run}.stderr"
        if stderr.exists():
            require(not stderr.read_text(encoding="utf-8").strip(), f"{stderr.name}: unexpected stderr")
        datasets = list(reversed(order)) if run == 2 else order.copy()
        orders[str(run)] = datasets
        seen = set()
        for line, (record, wanted_order) in enumerate(zip(records, plan(run, datasets)), start=2):
            context = f"{path.name}:{line}"
            require(set(record) == set(HEADER) and all(v is not None for v in record.values()),
                    f"{context}: missing or extra fields")
            dataset = record["dataset"]
            operation = record["operation"]
            case = record["representation"]
            try:
                round_ = int(record["round"])
                n = int(record["iterations"])
                ms = float(record["ms"])
                checksum = int(record["checksum"])
            except ValueError as error:
                raise ValueError(f"{context}: invalid numeric field") from error
            key = (dataset, round_, operation, case)
            require(key == wanted_order, f"{context}: incorrect dataset/round/operation/case order")
            require(key not in seen, f"{context}: duplicate sample")
            seen.add(key)
            require(n == iterations(dataset, operation), f"{context}: incorrect iterations")
            require(math.isfinite(ms) and ms > 0, f"{context}: elapsed ms must be finite and positive")
            require(0 <= checksum <= U64_MAX, f"{context}: checksum outside u64 range")
            if operation.startswith("scan_"):
                if dataset in scans:
                    require(scans[dataset] == checksum, f"{context}: inconsistent scan checksum")
                scans[dataset] = checksum
            else:
                require(checksum == n, f"{context}: completed-operation count differs")
            if round_ != 0:
                per_iteration = ms / n
                require(math.isfinite(per_iteration) and per_iteration > 0,
                        f"{context}: per-iteration duration is not representable")
                samples[(run, dataset, operation, case, round_)] = per_iteration
        expected = {(d, r, op, case) for d in DATASETS for r in range(10)
                    for op in OPERATIONS for case in CASES}
        require(seen == expected, f"{path.name}: incomplete sample set")
    results = []
    for dataset in DATASETS:
        for operation in OPERATIONS:
            values = {case: [samples[(run, dataset, operation, case, round_)]
                             for run in RUNS for round_ in range(1, 10)] for case in CASES}
            ratios = {}
            for case in CASES:
                for baseline in ["original", "previous16", "safe24-borrow"]:
                    paired = [a / b for a, b in zip(values[case], values[baseline])]
                    require(all(math.isfinite(x) and x > 0 for x in paired),
                            f"{dataset}/{operation}/{case}/{baseline}: unrepresentable ratio")
                    ratios[f"{case}/{baseline}"] = {
                        "paired_median": median(paired),
                        "paired_min": min(paired), "paired_max": max(paired),
                        "per_run_medians": [median(paired[i * 9:(i + 1) * 9]) for i in range(3)],
                    }
            results.append({
                "dataset": dataset, "operation": operation,
                "iterations": iterations(dataset, operation), "measured_samples_per_case": 27,
                "median_ms_per_iteration": {c: median(v) for c, v in values.items()},
                "sample_ms_per_iteration_range": {c: [min(v), max(v)] for c, v in values.items()},
                "per_run_median_ms_per_iteration": {
                    c: [median(v[i * 9:(i + 1) * 9]) for i in range(3)] for c, v in values.items()},
                "ratios": ratios,
            })
    require(len(samples) == 10530, "Unexpected number of measured samples")
    return {
        "validation": "PASS", "runs": RUNS, "datasets": DATASETS, "operations": OPERATIONS,
        "representations": CASES, "dataset_order_by_run": orders,
        "raw_rows": 11700, "measured_rows": len(samples), "excluded_warmup_round": 0,
        "scan_checksums": scans,
        "ratio_direction": "Greater than one means the numerator takes longer",
        "scope": "Expression operations on fixed snapshots; construction is conversion, not evaluation",
        "method": "27 within-round paired ratios per comparison; descriptive ranges are not confidence intervals",
        "results": results,
    }


def report(summary):
    lines = [
        "# Typed payload operation timing report", "",
        "Validated 11,700 rows; round 0 excluded, leaving 27 samples per representation and operation.",
        "Ratios above 1 mean slower. All ratios are medians of within-run, within-round pairs.",
        "Construction measures fresh alias-preserving conversion and normal release, not an evaluator workload.",
        "Drop uses a separately constructed fresh graph outside each timer; it includes per-sample timer overhead.",
        "Ranges and per-run medians in the JSON are descriptive; these runs do not establish statistical significance.", "",
        "| Dataset | Operation | Original ms/iteration | Safe24 / original | Typed16 move / original | Typed16 borrow / original | Typed16 borrow / previous | Typed16 borrow / Safe24 |",
        "| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |",
    ]
    for result in summary["results"]:
        ratio = result["ratios"]
        pairs = ["safe24-borrow/original", "typed16-move/original", "typed16-borrow/original",
                 "typed16-borrow/previous16", "typed16-borrow/safe24-borrow"]
        fields = [result["dataset"], result["operation"],
                  f'{result["median_ms_per_iteration"]["original"]:.9f}']
        fields.extend(f'{ratio[p]["paired_median"]:.3f}' for p in pairs)
        lines.append("| " + " | ".join(fields) + " |")
    return "\n".join(lines) + "\n"


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--evidence", type=Path, required=True, help="Directory containing run-1/2/3.tsv")
    parser.add_argument("--output", type=Path, help="JSON path; defaults to EVIDENCE/timing-summary.json")
    args = parser.parse_args()
    summary = analyze(args.evidence)
    output = args.output or args.evidence / "timing-summary.json"
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(json.dumps(summary, indent=2, allow_nan=False) + "\n", encoding="utf-8")
    text_path = output.with_name("timing-report.md")
    text_path.write_text(report(summary), encoding="utf-8")
    print("PASS: exact dataset/case order, all 11700 samples, iteration counts and checksums")
    print(f"Excluded warmup; 10530 measured rows summarized in {output.name} and {text_path.name}")


if __name__ == "__main__":
    main()
