"""Check complete fresh-probe construction samples and report paired ratios."""
import csv
import math
from statistics import median

DATASETS = {"gMicroWideTraversal", "gMicroSortSymbolWidth16000", "gMicroAssociationPartWidth32000",
    "gMicroReplacePartRules", "rulesSimple", "synthetic-repeated-symbol-16000",
    "synthetic-late-repeats-16000", "synthetic-early-repeats-16000"}
CASES = [f"{owner}-{policy}" for owner in ["safe", "compact"] for policy in ["plain", "shared", "selective"]]

def analyze(out):
    samples = {}; orders = []
    for run in [1,2,3]:
        records = list(csv.DictReader((out/f"run-{run}.tsv").open(encoding="utf-8"), delimiter="\t"))
        assert len(records)==480 and not (out/f"run-{run}.stderr").read_text(encoding="utf-8").strip()
        seen=set(); order=[]
        for i,r in enumerate(records):
            d,c,round_=r["dataset"],r["representation"],int(r["round"])
            assert d in DATASETS and c in CASES and 0<=round_<10
            assert int(r["iterations"])==(200 if d=="rulesSimple" else 2)
            ms=float(r["ms"]); assert math.isfinite(ms) and ms>0
            if i%60==0: order.append(d)
            assert d==order[-1] and round_==(i%60)//6
            case=(i%6+round_)%6
            assert c==CASES[5-case if run==2 else case]
            key=(d,c,round_); assert key not in seen; seen.add(key)
            if round_: samples[(run,*key)]=ms
        assert seen=={(d,c,r) for d in DATASETS for c in CASES for r in range(10)}
        orders.append(order)
    assert orders[0]==orders[2] and orders[1]==list(reversed(orders[0]))
    rows=[]
    for d in sorted(DATASETS):
        values={c:[samples[(run,d,c,r)] for run in [1,2,3] for r in range(1,10)] for c in CASES}
        pairs=[(f"{owner}-{a}",f"{owner}-{b}") for owner in ["safe","compact"]
            for a,b in [("selective","plain"),("selective","shared"),("shared","plain")]]
        pairs.append(("compact-selective","safe-selective"))
        ratios={}
        for a,b in pairs:
            v=[x/y for x,y in zip(values[a],values[b])]
            ratios[f"{a}/{b}"]={"paired_median":median(v),"per_run_medians":[median(v[i*9:(i+1)*9]) for i in range(3)],"range":[min(v),max(v)]}
        rows.append({"dataset":d,"operation":"build_drop","measured_samples_per_case":27,
            "median_ms":{c:median(v) for c,v in values.items()},
            "sample_ms_range":{c:[min(v),max(v)] for c,v in values.items()},"ratios":ratios})
    return {"raw_rows":1440,"measured_rows":len(samples),"reversed_order_run":2,"results":rows}
