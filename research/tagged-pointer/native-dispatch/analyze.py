"""Audit isolated native integration and full per-label language A/B evidence."""
import argparse
import csv
import hashlib
import json
import math
from pathlib import Path
import re
from statistics import median
MODES=["base","api","raw24","safe24","packed16","tagged16","sort-api","sort-24","sort-safe16","sort-tagged16"]
CACHE=["raw24","safe24","packed16","tagged16"]
SORT=["sort-24","sort-safe16","sort-tagged16"]
TARGETS=["sort-symbol-default-width4000","sort-symbol-default-width16000"]
def read(p):return p.read_text(encoding="utf-8")
def sha(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def tests(p,wanted,ignored=0):
    s=read(p);rows=re.findall(r"test result: ok\. (\d+) passed; 0 failed; (\d+) ignored",s)
    assert sum(int(n) for n,_ in rows)==wanted and sum(int(i) for _,i in rows)==ignored,p
    assert not re.search(r"^error|test result: FAILED",s,re.M),p
    return len(rows)
def analyze(out):
    study=Path(__file__).resolve().parent;research=study.parent;repo=study.parents[2]
    assert read(out/"status.txt").strip()=="CHECKS_PASS"
    assert read(out/"sort-status.txt").strip()=="SORT_CHECKS_PASS"
    assert read(out/"timing-status.txt").strip()=="TIMINGS_PASS"
    checks={}
    for mode in MODES[1:]:
        assert tests(out/f"{mode}-workspace.log",2242,7)==29
        checks[mode]={"workspace_passed":2242,"ignored":7}
    for mode in CACHE+SORT:
        n=3 if mode in CACHE else 2
        for suffix in ["native","miri-stacked","miri-tree"]:tests(out/f"{mode}-{suffix}.log",n)
        checks[mode].update(native=n,miri_stacked=n,miri_tree=n)
    patches={}
    for mode in MODES:
        record=json.loads(read(out/f"{mode}-overlay.json"));assert record["mode"]==mode
        assert record["upstream_commit"]=="77029938f999b14dbe5b0755bebdf98e51517ae6"
        for rel,v in record["patched_files"].items():assert sha(repo/rel)==v["base_sha256"],rel
        patches[mode]=record["patched_files"]
        assert "NATIVE_SORT_ACCEPTANCE_PASS" in read(out/f"{mode}-acceptance.stdout")
        assert not read(out/f"{mode}-acceptance.stderr").strip()
    pin=json.loads(read(research/"expression-census/corpus/PIN.json"))
    assert sha(repo/"scripts/core-language-microbench.sh")==pin["upstream_key_source_hashes"]["scripts/core-language-microbench.sh"]
    previous=json.loads(read(research/"evidence/typed-layout/summary.json"))
    for rel,digest in previous["source_hashes"].items():assert sha(research/"typed-layout"/rel)==digest
    values={};counts={};labels=None;source_hash=None;run_states={}
    for run in range(1,4):
        for mode in MODES:
            folder=out/f"timing-{run}-{mode}"
            with (folder/"runs.tsv").open(encoding="utf-8",newline="") as f:state=list(csv.DictReader(f,delimiter="\t"))
            assert len(state)==1 and state[0]["status"]=="ok" and state[0]["exit_code"]=="0"
            assert math.isfinite(float(state[0]["elapsed_seconds"])) and float(state[0]["elapsed_seconds"])>0
            assert not read(folder/"raw/rum.stderr").strip()
            run_states[f"{run}-{mode}"]={k:v for k,v in state[0].items() if k not in {"stdout","stderr"}}
            digest=sha(folder/"core-language-microbench.rum")
            if source_hash is None:source_hash=digest
            assert source_hash==digest
            assert "rumMicroScale = 5;" in read(folder/"core-language-microbench.rum")
            assert "rumMicroReps = 9;" in read(folder/"core-language-microbench.rum")
            with (folder/"timings.tsv").open(encoding="utf-8",newline="") as f:
                reader=csv.DictReader(f,delimiter="\t");assert reader.fieldnames==["label","iterations","rum_seconds"];rows=list(reader)
            current=[r["label"] for r in rows];assert len(current)==len(set(current))
            if labels is None:labels=current
            assert labels==current
            for row in rows:
                label=row["label"];n=int(row["iterations"]);v=float(row["rum_seconds"])
                assert n>0 and math.isfinite(v) and v>0
                if label in counts:assert counts[label]==n
                counts[label]=n;values[run,mode,label]=v
    def ratios(numerator,baseline,label):
        r=[values[run,numerator,label]/values[run,baseline,label] for run in range(1,4)]
        return {"median":median(r),"per_run":r,"range":[min(r),max(r)]}
    comparisons=[]
    for label in labels:
        pairs={}
        for mode in MODES[1:]:
            pairs[mode+"/base"]=ratios(mode,"base",label)
            control="sort-api" if mode.startswith("sort-") else "api"
            pairs[mode+"/"+control]=ratios(mode,control,label)
        for mode in CACHE:pairs[mode+"/raw24"]=ratios(mode,"raw24",label)
        for mode in SORT:pairs[mode+"/sort-24"]=ratios(mode,"sort-24",label)
        pairs["tagged16/packed16"]=ratios("tagged16","packed16",label)
        pairs["sort-tagged16/sort-safe16"]=ratios("sort-tagged16","sort-safe16",label)
        comparisons.append({"label":label,"nominal_iterations":counts[label],
            "seconds_by_run":{mode:[values[r,mode,label] for r in range(1,4)] for mode in MODES},"ratios":pairs})
    slower={};attention={}
    for mode in MODES[1:]:
        slower[mode]=[r["label"] for r in comparisons if r["ratios"][mode+"/base"]["median"]>1]
        attention[mode]=[{"label":r["label"],"ratio":r["ratios"][mode+"/base"]} for r in comparisons
                         if r["ratios"][mode+"/base"]["median"]>=1.05]
    profile=[]
    for line in read(out/"profile-enabled.stderr").splitlines():
        if line.startswith("rum-profile builtin-self-timing rank=") or line.startswith("rum-profile runtime-path-self-timing rank="):
            fields=dict(re.findall(r"(\w+)=([^ ]+)",line));profile.append(fields)
    assert profile
    checks.update(full_language_modes=10,full_result_acceptance_probes=10,
                  language_label_count=len(labels),aggregate_timing_rows=len(values),prior_sources_unchanged="PASS")
    summary={"status":"PASS","recorded_on":"2026-10-13","checks":checks,"modes":MODES,"targets":TARGETS,
        "isolated_kernel_integration_implemented":True,"default_kernel_changed":False,
        "scope":"Native builtin dispatch cache and native wide all-symbol Sort temporary vectors in separately patched source copies. Main Expr remains original; no recursive graph conversion. All comparator, materialization and full language evaluation costs are included.",
        "method":"Complete unchanged core-language-microbench.sh, scale5/reps9, one warm evaluation. Most labels report MINIMUM of nine monotonic-clock repetitions; special GC lane keeps original convention. Three sequential CPU-affinity process runs per mode; modes rotate and run2 reverses. Report three paired process values, not 27 raw repetitions. No significance or whole-program speedup from equal label weighting.",
        "profiles_scope":"Existing instrumented builtin/runtime self-time diagnostics on full source; separate from counter-off A/B. Inclusive categories overlap and profiling itself changes costs. They select candidate paths, not hardware cache miss measurements or exact unprofiled bottleneck fractions.",
        "regression_scope":"Every slower median is retained. >=5% attention list is a screening convention, not a claim that smaller regressions are acceptable or statistically proven. Repository gate requires fixing or reporting regressions; no default kernel change is shipped.",
        "hash_scope":"Pinned hashbrown0.16.1 safe Equivalent lookup; same exact Symbol hash stream, full identities, and collision tests. Raw24 controls container change; api controls additive accessors and inlining; packed16 is safe integer identity control.",
        "sort_scope":"Qualified-name byte order, not Symbol Ord or dense-id order. Identities ignored in comparator but fully retained in stable output. Safe16 and tagged16 both retain text plus full identity; only temporary all-symbol operand vectors >=64 admitted. Unsupported/small/custom-comparator paths preserve original sorting.",
        "previous_commit":"7ab0ff7605d55e07927fd6ab872079db643bb437","patched_source_hashes":patches,
        "benchmark_generated_sha256":source_hash,"run_states":run_states,"profile_self_rows":profile,
        "comparisons":comparisons,"all_observed_slower_labels":slower,"attention_regressions":attention,
        "source_hashes":{stage:{p.relative_to(research/stage).as_posix():sha(p) for p in sorted((research/stage).rglob('*'))
            if p.is_file() and not {'target','__pycache__'}.intersection(p.parts) and p.name!='README.md'} for stage in ["native-dispatch","native-sort"]},
        "raw_evidence_hashes":{p.relative_to(out).as_posix():sha(p) for p in sorted(out.rglob('*')) if p.is_file() and p.name!='summary.json'}}
    (out/"summary.json").write_text(json.dumps(summary,indent=2)+"\n",encoding="utf-8",newline="\n")
    return summary
if __name__=="__main__":
    p=argparse.ArgumentParser(description=__doc__);p.add_argument("evidence",type=Path)
    s=analyze(p.parse_args().evidence.resolve());print(json.dumps(s["checks"],indent=2))
