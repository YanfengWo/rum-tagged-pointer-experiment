"""Validate exact paired plan, complete results and unchanged kernel controls."""
import argparse
import csv
import hashlib
import json
import math
from pathlib import Path
import re
from statistics import median

DATASETS=["gMicroWideTraversal","gMicroSortSymbolWidth16000","gMicroAssociationPartWidth32000","gMicroReplacePartRules","rulesSimple","repeat-16000","generated-16000","strings-16000","contexts-16000","shared-200","unique-16","unique-17"]
CASES=["original","previous16-full","safe24-full","typed16-full","safe24-key","typed16-derived","typed16-key"]
OPS=["dedup-1","dedup-8","convert-dedup-1","convert-dedup-8"]
HEADER=["dataset","round","operation","representation","iterations","ms","checksum"]
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def read(p):return p.read_text(encoding="utf-8")
def normalized(s):return re.sub(r"\s+","",s)
def algorithm_check(repo,study):
    old=read(repo/"crates/rum-runtime/src/runtime/definitions/unique_symbols.rs")
    old=old.split("pub(super) fn push(&mut self, symbol: Symbol) {",1)[1].split("pub(super) fn into_symbols",1)[0]
    old=old.rsplit("}",1)[0]
    old=old.replace("self.","").replace("capacity_hint","hint").replace("UNIQUE_SYMBOL_INDEX_THRESHOLD","16").replace("return;","continue;")
    old=old.replace("let mut index = std::collections::HashSet::with_capacity(capacity);","let mut set = HashSet::with_capacity_and_hasher(capacity, seed.clone());")
    old=old.replace("index.extend(symbols.iter().copied());","set.extend(symbols.iter().copied());")
    old=old.replace("let inserted = index.insert(symbol);","let inserted = set.insert(symbol);")
    old=old.replace('debug_assert!(inserted, "linear duplicate check rejected this symbol");','debug_assert!(inserted);')
    old=old.replace("index = Some(index);","index = Some(set);")
    new=read(study/"src/collector.rs").split("for symbol in input {",1)[1].split("symbols.into_iter()",1)[0].rsplit("}",1)[0]
    assert normalized(old)==normalized(new),"generic push decisions differ from pinned collector"

def analyze(out):
    study=Path(__file__).resolve().parent;research=study.parent;repo=study.parents[2]
    assert read(out/"status.txt").strip()=="PASS"
    checks={}
    for name in ["native.log","miri-stacked.log","miri-tree.log"]:
        log=read(out/name);n=sum(map(int,re.findall(r"test result: ok\. (\d+) passed; 0 failed",log)))
        assert n==3 and "test result: FAILED" not in log and not re.search(r"^error",log,re.M),(name,n)
        checks[name]=n
    results=re.findall(r"test result: ok\. (\d+) passed; 0 failed; (\d+) ignored",read(out/"workspace.log"))
    assert sum(int(n) for n,_ in results)==2242 and sum(int(n) for _,n in results)==7 and len(results)==29
    checks.update(workspace_passed=2242,workspace_ignored=7,workspace_result_groups=29)
    assert "134 snapshots; 938 complete ordered Symbol results" in read(out/"semantic.log")
    checks["complete_ordered_results"]=938
    algorithm_check(repo,study);checks["generic_push_matches_kernel_decisions"]="PASS"
    metadata=json.loads(read(out/"overlay.json"))
    assert metadata["upstream_commit"]=="77029938f999b14dbe5b0755bebdf98e51517ae6"
    actual="crates/rum-runtime/src/runtime/definitions/unique_symbols.rs"
    assert sha(repo/actual)==metadata["runtime_collector_unchanged_sha256"]
    for rel,r in metadata["runtime_exports"].items():
        extension=(study/r["extension_file"]).read_bytes();base=(repo/rel).read_bytes()
        assert b"unsafe" not in extension
        assert sha(repo/rel)==r["original_sha256"] and sha(study/r["extension_file"])==r["extension_sha256"]
        assert hashlib.sha256(base+b"\n"+extension).hexdigest()==r["patched_sha256"]
    pin=json.loads(read(research/"expression-census/corpus/PIN.json"))
    for row in pin["files"]:
        p=research/"expression-census/corpus"/row["path"];assert sha(p)==row["sha256"] and p.stat().st_size==row["bytes"]
    for rel,digest in pin["upstream_key_source_hashes"].items():assert sha(repo/rel)==digest
    prior=json.loads(read(research/"evidence/typed-layout/summary.json"))
    for rel,digest in prior["source_hashes"].items():assert sha(research/"typed-layout"/rel)==digest
    checks["prior_representation_and_default_kernel_unchanged"]="PASS"
    order=[e["label"] for e in json.loads(read(research/"expression-census/corpus/index.json")) if e["label"] in DATASETS]
    assert len(order)==5 and set(order)==set(DATASETS[:5]);order+=DATASETS[5:]
    samples={};checksums={}
    for run in range(1,4):
        assert not read(out/f"run-{run}.stderr").strip()
        with (out/f"run-{run}.tsv").open(encoding="utf-8",newline="") as f:
            reader=csv.DictReader(f,delimiter="\t");assert reader.fieldnames==HEADER;records=list(reader)
        assert len(records)==3360
        plan=[]
        for d in (list(reversed(order)) if run==2 else order):
            for round_ in range(10):
                for oi,op in enumerate(OPS):
                    for offset in range(7):
                        rotated=(offset+round_+oi+run)%7;case=CASES[6-rotated if run==2 else rotated]
                        plan.append((d,round_,op,case))
        for row,wanted in zip(records,plan):
            key=(row["dataset"],int(row["round"]),row["operation"],row["representation"])
            assert key==wanted and set(row)==set(HEADER) and None not in row.values()
            d,round_,op,case=key;n=int(row["iterations"]);value=float(row["ms"]);checksum=int(row["checksum"])
            assert n==(100 if d.startswith("unique-") or d=="rulesSimple" else 3)
            assert math.isfinite(value) and value>0 and 0<=checksum<(1<<64)
            ck=(d,op)
            if ck in checksums:assert checksums[ck]==checksum
            checksums[ck]=checksum
            if round_!=0:samples[(run,d,op,case,round_)]=value/n
    for d in DATASETS:
        assert checksums[d,"dedup-1"]==checksums[d,"convert-dedup-1"]
        assert checksums[d,"dedup-8"]==checksums[d,"convert-dedup-8"]==(checksums[d,"dedup-1"]*8)% (1<<64)
    assert len(samples)==9072
    def stats(pairs):return {"paired_median":median(pairs),"paired_range":[min(pairs),max(pairs)],"per_run_medians":[median(pairs[i*9:(i+1)*9]) for i in range(3)]}
    rows=[]
    for d in DATASETS:
        for op in OPS:
            values={c:[samples[run,d,op,c,round_] for run in range(1,4) for round_ in range(1,10)] for c in CASES}
            ratios={c+"/"+b:stats([a/z for a,z in zip(values[c],values[b])]) for c in CASES for b in ["original","typed16-full","safe24-key","typed16-derived"]}
            rows.append({"dataset":d,"operation":op,"median_ms_per_batch":{c:median(v) for c,v in values.items()},"ratios":ratios})
    adoption=[]
    for d in DATASETS:
        for repeats in [1,8]:
            original=[samples[run,d,f"dedup-{repeats}","original",r] for run in range(1,4) for r in range(1,10)]
            ratios={c:stats([samples[run,d,f"convert-dedup-{repeats}",c,r]/b for (run,r),b in zip([(run,r) for run in range(1,4) for r in range(1,10)],original)]) for c in CASES if c!="original"}
            adoption.append({"dataset":d,"operations_per_conversion":repeats,"ratios_vs_existing_original":ratios})
    checks.update(timing_raw_rows=10080,timing_measured_rows=9072,frozen_corpus="PASS")
    summary={"status":"PASS","recorded_on":"2026-10-13","checks":checks,"datasets":DATASETS,"cases":CASES,"operations":OPS,"results":rows,"adoption":adoption,
        "scope":"Actual kernel adaptive symbol-uniqueness algorithm ported to generic observations; original kernel collector is an unchanged result oracle. Full preorder includes heads. Not the complete UpValue planner or an integrated evaluator. All modes return full ordered Vec<Symbol>, including materialization and release.",
        "method":"3 affinity runs, 10 rounds each, round0 excluded; 27 within-run/round ratios. Cases/setup rotate; run2 reverses datasets/cases. Operation order remains fixed. RandomState has fresh seeds per process, common clones across representations within that process. Ratios descriptive, no statistical significance claim.",
        "adoption_scope":"Cross-operation pairs compare candidate conversion + N dedups + release against N dedups on an existing original graph; original conversion excluded. Warm prepared roots coexist in all lanes. This is a bounded snapshot pipeline, not runtime replacement.",
        "hash_scope":"Current SymbolKey derived Hash is preserved in typed16-derived. Local borrowed-entry compatible adapter matches exact pinned Symbol hash input; no original public key API modified. Outputs resolve to complete authentic Symbols only for unique observations.",
        "previous_commit":"d9a9d4763401599451cae95b3167fdf594ce6edd","source_hashes":{p.relative_to(study).as_posix():sha(p) for p in sorted(study.rglob('*')) if p.is_file() and not {'target','__pycache__'}.intersection(p.parts) and p.name!='README.md'},
        "raw_evidence_hashes":{p.name:sha(p) for p in sorted(out.iterdir()) if p.is_file() and p.name!='summary.json'}}
    (out/"summary.json").write_text(json.dumps(summary,indent=2)+"\n",encoding="utf-8",newline="\n")
    return summary
if __name__=="__main__":
    p=argparse.ArgumentParser(description=__doc__);p.add_argument("evidence",type=Path)
    s=analyze(p.parse_args().evidence.resolve());print(json.dumps(s["checks"],indent=2))
