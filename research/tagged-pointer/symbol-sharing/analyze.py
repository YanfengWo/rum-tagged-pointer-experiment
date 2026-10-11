"""Check the sharing policy against frozen corpus and measured allocation deltas."""
import hashlib
import json
from pathlib import Path
import re
import sys
sys.dont_write_bytecode = True
from timings import analyze as timings

study = Path(__file__).resolve().parent
research = study.parent
out = Path(sys.argv[1]).resolve()
assert (out / "status.txt").read_text().strip() == "PASS"
checks = {}
for name in ["native.log", "native-all-features.log", "miri-stacked.log", "miri-tree.log"]:
    log = (out / name).read_text()
    matches = re.findall(r"test result: ok\. (\d+) passed; 0 failed", log)
    assert sum(map(int, matches)) == 8, name
    checks[name] = 8
for name in ["native.log", "native-all-features.log", "timing-build.log", "allocation-build.log", "format.log"]:
    assert not re.search(r"(^|\n)(error:|error\[|warning:)", (out / name).read_text()), name
assert "PASS: 128 snapshots; 512 full equalities and scan checks; COW" in (out / "semantic.log").read_text()
assert "640 fresh graph windows returned to baseline" in (out / "allocations.log").read_text()
pin = json.loads((research / "expression-census/corpus/PIN.json").read_text())
for relative, digest in pin["prototype_source_hashes"].items():
    assert hashlib.sha256((research / relative).read_bytes()).hexdigest() == digest, relative
for relative, digest in pin["upstream_key_source_hashes"].items():
    assert hashlib.sha256((study.parents[2] / relative).read_bytes()).hexdigest() == digest, relative
census_summary = json.loads((research / "evidence/expression-census/summary.json").read_text())
for relative, digest in census_summary["source_hashes"].items():
    assert hashlib.sha256((research / "expression-census" / relative).read_bytes()).hexdigest() == digest, relative
# Apart from the atom callback and imports/signature, this is the pinned bridge.
original_bridge = (research / "recursive-expr/src/conversion.rs").read_text().split("enum Back")[0]
expected_bridge = original_bridge.replace("use crate::{Args, AtomRef, Head, IntRef, Tree, View};", "use rum_recursive_expr::{Args, Head, Tree};")
expected_bridge = expected_bridge.replace("use rum_core::{Atom, Expr, IntValue, SharedArgs, SharedExpr};", "use rum_core::{Atom, Expr, SharedArgs, SharedExpr};")
expected_bridge = expected_bridge.replace("pub(crate) fn from_expr<N: Tree>(expr: &Expr) -> N {", "pub(crate) fn convert<N: Tree>(expr: &Expr, atom_node: &mut impl FnMut(&Atom) -> N) -> N {")
expected_bridge = expected_bridge.replace("N::atom(atom.clone())", "atom_node(atom)")
expected_bridge = expected_bridge.replace("use rum_recursive_expr::{Args, Head, Tree};\nuse rum_core::{Atom, Expr, SharedArgs, SharedExpr};", "use rum_core::{Atom, Expr, SharedArgs, SharedExpr};\nuse rum_recursive_expr::{Args, Head, Tree};")
assert re.sub(r"\s+", "", expected_bridge) == re.sub(r"\s+", "", (study / "src/bridge.rs").read_text())
assert not any(re.search(r"\bunsafe\s+(impl|fn|\{)", p.read_text()) for p in (study / "src").rglob("*.rs"))
prior = {r["entry"]["label"] + "/" + r["entry"]["kind"]: r for r in json.loads((research / "evidence/expression-census/allocations.json").read_text())}
physical = {r["entry"]["label"] + "/" + r["entry"]["kind"]: r["graph"] for r in json.loads((research / "evidence/expression-census/census.json").read_text())}
alloc = json.loads((out / "allocations.json").read_text())
assert len(alloc) == 128
seen = set(); rows = []
for record in alloc:
    key = record["entry"]["label"] + "/" + record["entry"]["kind"]
    assert key not in seen; seen.add(key)
    m = {r["representation"]:r for r in record["measurements"]}
    assert set(m) == {"original-normalized", "safe-plain", "safe-shared", "compact-plain", "compact-shared"}
    kept = {c:r["retained_requested_bytes"] for c,r in m.items()}
    for c,r in m.items():
        assert 0 <= r["retained_requested_bytes"] <= r["peak_requested_bytes"] and r["remaining_bytes"] == 0
        assert r["root_stack_bytes"] == (32 if c=="original-normalized" else 16 if c.startswith("safe") else 8)
    assert m["safe-shared"]["stats"] == m["compact-shared"]["stats"]
    s = m["compact-shared"]["stats"]
    assert 0 <= s["entries"] <= 128
    assert s["symbol_visits"] == s["entries"] + s["hits"] + s["uncached"]
    for rep in ["safe", "compact"]:
        assert kept[f"{rep}-plain"]-kept[f"{rep}-shared"] == 48*s["hits"], key
    if key in prior:
        assert record["entry"] == prior[key]["entry"]
        old = {r["representation"]:r["retained_requested_bytes"] for r in prior[key]["measurements"]}
        assert old == {"original-normalized":kept["original-normalized"], "safe":kept["safe-plain"], "compact":kept["compact-plain"]}
        assert s["symbol_visits"] == physical[key]["physical"]["symbol"]
    else:
        assert key == "synthetic-repeated-symbol-16000/synthetic-control"
        assert s == {"symbol_visits":16001,"entries":2,"hits":15999,"uncached":0}
    rows.append({"entry":record["entry"], "retained_requested_bytes":kept,"stats":s,
        "payload_saving_bytes":48*s["hits"], "compact_shared_over_original_normalized":kept["compact-shared"]/kept["original-normalized"] if kept["original-normalized"] else None,
        "peak_requested_bytes":{c:r["peak_requested_bytes"] for c,r in m.items()},
        "allocation_calls":{c:r["allocation_calls"] for c,r in m.items()}})
assert set(prior).issubset(seen)
checks.update(roundtrip_equalities=512, allocation_windows=640, symbol_payload_cost_equalities=128,
    same_policy_stats="PASS", prior_plain_layout_bytes_match=127, frozen_bridge_and_dependencies="PASS", format="PASS")
source_hashes = {p.relative_to(study).as_posix():hashlib.sha256(p.read_bytes()).hexdigest()
    for p in sorted(study.rglob("*")) if p.is_file() and "target" not in p.parts and "__pycache__" not in p.parts and p.name not in {"README.md", "SEMANTICS.md"}}
raw_hashes = {p.name:hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted(out.iterdir()) if p.is_file() and p.name != "summary.json"}
summary = {"status":"PASS", "recorded_on":"2026-10-10", "cache_entry_limit":128,"checks":checks,
    "upstream_commit":pin["upstream_commit"], "recursive_owner_commit":pin["recursive_prototype_commit"],
    "corpus_commit":"1f3c21bee022a24aea6cce4f653a9a381d9eada8", "kernel_integration_implemented":False,
    "allocation_scope":"Fresh normalized retained System requested heap bytes; root/input/static intern pool/allocator overhead excluded. Original peak/calls include two bridges and are not comparable construction costs.",
    "timing_scope":"Expression scan, build-and-drop and append COW; counters absent, per-build cache empty. Snapshot controls, not evaluator A/B; host interference not excluded and no statistical significance claimed.",
    "timings":timings(out), "allocation_rows":rows,"source_hashes":source_hashes,"raw_evidence_hashes":raw_hashes}
(out / "summary.json").write_text(json.dumps(summary,indent=2)+"\n",encoding="utf-8",newline="\n")
print(json.dumps({"status":"PASS","checks":checks,"raw_rows":summary["timings"]["raw_rows"],"measured_rows":summary["timings"]["measured_rows"]},indent=2))
