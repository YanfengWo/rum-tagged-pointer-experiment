"""Verify bounded decisions, branch-equivalent storage and frozen dependencies."""
import hashlib
import json
from pathlib import Path
import re
import sys
sys.dont_write_bytecode=True
from timings import analyze as timings

study=Path(__file__).resolve().parent; research=study.parent; out=Path(sys.argv[1]).resolve()
def read(p): return p.read_text(encoding="utf-8")
def sha(p): return hashlib.sha256(p.read_bytes()).hexdigest()
assert read(out/"status.txt").strip()=="PASS"
checks={}
for name in ["native.log","native-all-features.log","miri-stacked.log","miri-tree.log"]:
    assert sum(map(int,re.findall(r"test result: ok\. (\d+) passed; 0 failed",read(out/name))))==9,name
    checks[name]=9
for name in ["native.log","native-all-features.log","timing-build.log","allocation-build.log","format.log"]:
    assert not re.search(r"(^|\n)(error:|error\[|warning:)",read(out/name)),name
assert "130 snapshots; 780 full equalities" in read(out/"semantic.log")
assert "910 fresh graph windows returned to baseline" in read(out/"allocations.log")
pin=json.loads(read(research/"expression-census/corpus/PIN.json"))
for rel,digest in pin["prototype_source_hashes"].items(): assert sha(research/rel)==digest,rel
for rel,digest in pin["upstream_key_source_hashes"].items(): assert sha(study.parents[2]/rel)==digest,rel
for stage in ["expression-census","symbol-sharing"]:
    summary=json.loads(read(research/f"evidence/{stage}/summary.json"))
    for rel,digest in summary["source_hashes"].items(): assert sha(research/stage/rel)==digest,rel
assert not any(re.search(r"\bunsafe\s+(impl|fn|\{)",read(p)) for p in (study/"src").rglob("*.rs"))
prior={r["entry"]["label"]+"/"+r["entry"]["kind"]:r for r in json.loads(read(research/"evidence/symbol-sharing/allocations.json"))}
alloc=json.loads(read(out/"allocations.json")); assert len(alloc)==130
seen=set(); rows=[]; decisions={"SmallRoot":0,"Plain":0,"Shared":0}
for row in alloc:
    key=row["entry"]["label"]+"/"+row["entry"]["kind"]; assert key not in seen; seen.add(key)
    m={r["representation"]:r for r in row["measurements"]}
    assert set(m)=={"original-normalized",*[f"{o}-{p}" for o in ["safe","compact"] for p in ["plain","shared","selective"]]}
    kept={c:r["retained_requested_bytes"] for c,r in m.items()}
    for c,r in m.items():
        assert 0<=r["retained_requested_bytes"]<=r["peak_requested_bytes"] and r["remaining_bytes"]==0
        assert r["root_stack_bytes"]==(32 if c=="original-normalized" else 16 if c.startswith("safe") else 8)
    for policy in ["plain","shared","selective"]:
        assert m[f"safe-{policy}"]["record"]==m[f"compact-{policy}"]["record"]
    record=m["compact-selective"]["record"]; p=record["probe"]; decision=p["decision"]
    assert decision in decisions; decisions[decision]+=1
    assert 0<=p["repeats"]<=8 and p["repeats"]<=p["symbols"]<=p["nodes"]<=256
    assert (record["sharing"] is not None)==(decision=="Shared")
    if decision=="SmallRoot": assert p["nodes"]==p["symbols"]==p["repeats"]==0
    if decision=="Shared": assert p["repeats"]==8 and record["sharing"]==m["compact-shared"]["record"]["sharing"]
    else: assert p["repeats"]<8
    s=m["compact-shared"]["record"]["sharing"]
    assert s["symbol_visits"]==s["entries"]+s["hits"]+s["uncached"] and s["entries"]<=128
    for owner in ["safe","compact"]:
        assert kept[f"{owner}-plain"]-kept[f"{owner}-shared"]==48*s["hits"]
        chosen="shared" if decision=="Shared" else "plain"
        assert kept[f"{owner}-selective"]==kept[f"{owner}-{chosen}"]
    if key in prior:
        assert row["entry"]==prior[key]["entry"]
        for old in prior[key]["measurements"]:
            assert kept[old["representation"]]==old["retained_requested_bytes"],key
    rows.append({"entry":row["entry"],"decision":p,"sharing_stats":s,"retained_requested_bytes":kept,
        "peak_requested_bytes":{c:r["peak_requested_bytes"] for c,r in m.items()},
        "allocation_calls":{c:r["allocation_calls"] for c,r in m.items()}})
assert set(prior).issubset(seen)
by_label={r["entry"]["label"]:r for r in rows if r["entry"]["kind"]=="synthetic-control"}
late=by_label["synthetic-late-repeats-16000"]; early=by_label["synthetic-early-repeats-16000"]
assert late["decision"]["decision"]=="Plain" and late["sharing_stats"]["hits"]==15700
assert early["decision"]["decision"]=="Shared" and early["sharing_stats"]["hits"]==8
checks.update(roundtrip_equalities=780,allocation_windows=910,branch_storage_equalities=260,
    shared_payload_cost_equalities=260,prior_layout_bytes_match=128,
    same_owner_decisions="PASS",frozen_dependencies="PASS",format="PASS",late_and_early_counterexamples="PASS")
source_hashes={p.relative_to(study).as_posix():sha(p) for p in sorted(study.rglob("*"))
    if p.is_file() and "target" not in p.parts and "__pycache__" not in p.parts and p.name not in {"README.md","SEMANTICS.md"}}
raw_hashes={p.name:sha(p) for p in sorted(out.iterdir()) if p.is_file() and p.name!="summary.json"}
summary={"status":"PASS","recorded_on":"2026-10-11","checks":checks,"decisions":decisions,
    "policy":{"minimum_root_args":64,"node_budget":256,"minimum_repeats":8,"cache_limit":128},
    "previous_commit":"c62eb6ba1f384bacb8c91e91013e6d6247e17ad3","upstream_commit":pin["upstream_commit"],
    "kernel_integration_implemented":False,
    "allocation_scope":"Normalized retained System requested heap bytes; input/root/static intern pool/allocator overhead excluded. Probe scratch drops before construction. Original peaks/calls include two bridges.",
    "timing_scope":"Fresh probe, conversion and ordinary Drop included; counters absent; rotating cases, run 2 reversed; snapshot operations, not evaluator A/B; no statistical significance claimed.",
    "allocation_rows":rows,"timings":timings(out),"source_hashes":source_hashes,"raw_evidence_hashes":raw_hashes}
(out/"summary.json").write_text(json.dumps(summary,indent=2)+"\n",encoding="utf-8",newline="\n")
print(json.dumps({"status":"PASS","checks":checks,"decisions":decisions,"timing_rows":summary["timings"]["raw_rows"]},indent=2))
