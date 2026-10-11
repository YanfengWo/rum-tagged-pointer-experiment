"""Validate immutable corpus, physical/logical census and fresh graph accounting."""
import hashlib
import json
from pathlib import Path
import re
import sys

study = Path(__file__).resolve().parent
out = Path(sys.argv[1]).resolve()
assert (out / "status.txt").read_text().strip() == "PASS"
checks = {}
for name in ["native.log", "native-all-features.log"]:
    log = (out / name).read_text()
    counts = re.findall(r"test result: ok\. (\d+) passed; 0 failed", log)
    assert sum(map(int, counts)) == 3, name
    checks[name] = 3
for name in ["census.log", "allocations.log"]:
    assert "PASS: 127 snapshots; Compact and Safe roundtrip equality" in (out / name).read_text(), name
checks["roundtrip_equalities_per_command"] = 254
checks["format.log"] = "PASS"
for name in ["build.log", "allocation-build.log", "format.log"]:
    assert not re.search(r"(^|\n)(error:|error\[|warning:)", (out / name).read_text()), name
pin = json.loads((study / "corpus/PIN.json").read_text())
assert {p.relative_to(study / "corpus").as_posix() for p in (study / "corpus").rglob("*")
        if p.is_file() and p.name != "PIN.json"} == {r["path"] for r in pin["files"]}
for item in pin["files"]:
    data = (study / "corpus" / item["path"]).read_bytes()
    assert hashlib.sha256(data).hexdigest() == item["sha256"], item["path"]
for relative, digest in pin["prototype_source_hashes"].items():
    assert hashlib.sha256((study.parent / relative).read_bytes()).hexdigest() == digest, relative
for relative, digest in pin["upstream_key_source_hashes"].items():
    assert hashlib.sha256((study.parents[2] / relative).read_bytes()).hexdigest() == digest, relative
index = json.loads((study / "corpus/index.json").read_text())
census = json.loads((out / "census.json").read_text())
alloc = json.loads((out / "allocations.json").read_text())
assert len(index) == len(census) == len(alloc) == 127
assert [r["entry"] for r in census] == [r["entry"] for r in alloc] == index
fields = ["inline_integer", "other_small_integer", "big_integer", "rational", "string", "symbol", "call"]
expected_results = {
    # Independent size checks follow the saved RHSs, not conversion output.
    "gMicroAssociationPartWidth32000": (64000, 32001, 32001),
    "gMicroAssociationPartWidth4000": (8000, 4001, 4001),
    "gMicroWideAssociation": (3200, 1601, 1601),
    "gMicroReplacePartRules": (1601, 1602, 3201),
    "gMicroSortCallbackLarge": (200, 1, 1),
    "gMicroSortCallbackSmall": (100, 1, 1),
    "gMicroSortDefaultWide": (1600, 1, 1),
    "gMicroWideTraversal": (1600, 1, 1),
    "gMicroSortSymbolWidth16000": (0, 16001, 1),
    "gMicroSortSymbolWidth4000": (0, 4001, 1),
    "list10": (10, 1, 1), "rulesSimple": (3, 7, 4),
}
rows = []
for record, measurement in zip(census, alloc):
    entry, graph = record["entry"], record["graph"]
    physical, logical = graph["physical"], graph["logical"]
    if entry["kind"] == "evaluated-result":
        if entry["label"] in expected_results:
            assert tuple(physical[f] for f in ["inline_integer", "symbol", "call"]) == expected_results[entry["label"]]
            assert physical["string"] == 0
        else:
            assert physical == dict(zip(fields, [0, 0, 0, 0, 1, 0, 0]))
            assert graph["string_length_sum"] == {
                "gMicroStringLiteralHaystack": 316,
                "gMicroStringTrimLiteralList4000": 4000,
                "gMicroStringTrimLiteralList32000": 32000}[entry["label"]]
        assert physical["big_integer"] == physical["other_small_integer"] == physical["rational"] == 0
    assert not graph["logical_counts_saturated"], entry["label"]
    assert sum(physical.values()) == graph["physical_nodes"]
    assert sum(logical.values()) == graph["logical_nodes"]
    assert graph["physical_nodes"] == 1 + graph["unique_heads"] + graph["argument_length_sum"]
    assert graph["argument_capacity_sum"] >= graph["argument_length_sum"]
    assert graph["string_capacity_sum"] >= graph["string_length_sum"]
    assert graph["unique_symbols"] <= physical["symbol"]
    assert graph["unique_string_values"] <= physical["string"]
    assert sum(graph["call_arity_histogram"]) == physical["call"]
    for field in fields:
        assert logical[field] >= physical[field]
    for key in ["head", "argument"]:
        unique = graph["unique_heads" if key == "head" else "unique_argument_lists"]
        assert unique + graph[f"repeated_{key}_handles"] == physical["call"]
    measures = {r["representation"]: r for r in measurement["measurements"]}
    assert set(measures) == {"original-normalized", "safe", "compact"}
    for name, root_size in [("original-normalized", 32), ("safe", 16), ("compact", 8)]:
        m = measures[name]
        assert m["remaining_bytes"] == 0 and m["root_stack_bytes"] == root_size
        assert 0 <= m["retained_requested_bytes"] <= m["peak_requested_bytes"]
    kept = {name: m["retained_requested_bytes"] for name, m in measures.items()}
    # Pinned x86_64 layout: each non-immediate physical node adds a 48-byte
    # Arc<Payload<Body>>. Original Head allocation is 48 bytes; Compact Head
    # is 24 bytes, Safe Head 32. Arc<Vec> and owned atom data cancel in deltas.
    # Both wrappers normalize vector capacity to length.
    slots = graph["argument_length_sum"] + graph["unique_heads"]
    boxed = graph["physical_nodes"] - physical["inline_integer"]
    assert kept["safe"] - kept["compact"] == 8 * slots, entry["label"]
    original = kept["original-normalized"]
    assert kept["compact"] - original == 48 * boxed - 24 * slots, entry["label"]
    assert kept["safe"] - original == 48 * boxed - 16 * slots, entry["label"]
    rows.append({"label": entry["label"], "kind": entry["kind"],
                 "physical_nodes": graph["physical_nodes"], "physical": physical,
                 "depth": graph["max_depth_nodes"], "unique_symbols": graph["unique_symbols"],
                 "unique_string_values": graph["unique_string_values"],
                 "retained_requested_bytes": kept,
                 "compact_over_original_normalized": kept["compact"] / original if original else None,
                 "safe_over_original_normalized": kept["safe"] / original if original else None,
                 "layout_delta_check": {"boxed_nodes": boxed, "boxed_node_requested_bytes": 48 * boxed,
                     "compact_slot_saving_bytes": 24 * slots, "compact_net_heap_change": kept["compact"] - original}})
aggregates = {}
checks.update(allocation_windows_returned_to_baseline=381, physical_layout_delta_equalities=127,
              selected_result_shape_checks=15, pinned_dependency_sources="PASS")
for kind in ["lowered-input", "evaluated-result"]:
    selected = [r for r in rows if r["kind"] == kind]
    aggregates[kind] = {"snapshots": len(selected),
        "physical_counts_sum": {f: sum(r["physical"][f] for r in selected) for f in fields},
        "compact_heap_comparison_counts": {direction: sum(
            compare(r["retained_requested_bytes"]["compact"], r["retained_requested_bytes"]["original-normalized"])
            for r in selected) for direction, compare in [
                ("less", lambda a, b: a < b), ("equal", lambda a, b: a == b), ("more", lambda a, b: a > b)]}}
hashes = {p.name: hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted(out.iterdir())
          if p.is_file() and p.name != "summary.json"}
sources = {p.relative_to(study).as_posix(): hashlib.sha256(p.read_bytes()).hexdigest()
           for p in sorted(study.rglob("*")) if p.is_file() and "target" not in p.parts
           and "__pycache__" not in p.parts and p.name != "README.md"}
summary = {"status": "PASS", "recorded_on": "2026-10-10", "checks": checks,
    "upstream_commit": pin["upstream_commit"], "recursive_prototype_commit": pin["recursive_prototype_commit"],
    "kernel_integration_implemented": False, "allocation_counter": "System requested bytes; retained heap excludes root stack, input graph, pre-existing symbol intern pool and allocator overhead; not RSS",
    "normalization": "Fresh original graph via Safe roundtrip preserves Head/Args alias classes; Vec/String spare capacity discarded. Original bridge peak/call count includes two bridges and is not a fair construction comparator.",
    "sampling": "112 lowered benchmark bodies and 15 selected evaluated setup RHS snapshots at scale 1; no dynamic allocation trace, workload-frequency weighting or evaluator timing",
    "aggregates": aggregates, "rows": rows, "source_hashes": sources, "raw_evidence_hashes": hashes}
(out / "summary.json").write_text(json.dumps(summary, indent=2) + "\n", encoding="utf-8", newline="\n")
print(json.dumps({"status": "PASS", "checks": checks, "aggregates": aggregates}, indent=2))
