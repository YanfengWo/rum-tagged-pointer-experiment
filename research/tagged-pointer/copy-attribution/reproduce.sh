#!/usr/bin/env bash
# Fresh primary replication; does not require the historical saved binaries.
set -euo pipefail
study="$(cd -- "$(dirname -- "$0")" && pwd)"
repo="$(cd -- "$study/../../.." && pwd)"
[[ "$#" = 2 ]] || { printf 'Usage: bash reproduce.sh NEW_SOURCE_DIRECTORY NEW_EVIDENCE_DIRECTORY\n'; exit 2; }
src="$(python3 -c 'from pathlib import Path;import sys;print(Path(sys.argv[1]).resolve())' "$1")"
out="$(python3 -c 'from pathlib import Path;import sys;print(Path(sys.argv[1]).resolve())' "$2")"
python3 - "$src" "$out" <<'PY'
from pathlib import Path
import sys
a,b=map(Path,sys.argv[1:]);assert a!=b and a not in b.parents and b not in a.parents
for p in [a,b]:assert not p.exists() or (p.is_dir() and not any(p.iterdir()))
b.mkdir(parents=True,exist_ok=True)
PY
bounded() { python3 "$repo/scripts/run-bounded.py" --timeout "$1" -- "${@:2}"; }
bounded 180 python3 "$study/prepare.py" --mode base --output "$src" --metadata "$out/base-overlay.json"
export CARGO_TARGET_DIR="$src/target" CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-4}" CARGO_PROFILE_DEV_DEBUG=0
mkdir -p "$out/binaries"
once=crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/once.rs
cp "$src/$once" "$out/original-once.rs"
trap 'cp "$out/original-once.rs" "$src/$once"' EXIT
cd "$src"
for mode in base candidate; do
    if [[ "$mode" = candidate ]]; then bounded 120 git apply "$study/candidate.patch"; fi
    bounded 900 cargo +1.85.0 build --release --locked --offline -p rum-cli > "$out/$mode-build.log" 2>&1
    cp "$CARGO_TARGET_DIR/release/rum-cli" "$out/binaries/$mode"
    sha256sum "$out/binaries/$mode" > "$out/$mode-binary.sha256"
    bounded 180 cargo +1.85.0 fmt --check > "$out/$mode-format.log" 2>&1
    bounded 1800 cargo +1.85.0 test --workspace --locked --offline -- --test-threads=1 > "$out/$mode-workspace.log" 2>&1
    bounded 120 "$out/binaries/$mode" --file "$study/acceptance.rum" < /dev/null > "$out/$mode-acceptance.stdout" 2> "$out/$mode-acceptance.stderr"
    grep -q COPY_ATTRIBUTION_ACCEPTANCE_PASS "$out/$mode-acceptance.stdout"
done
cp "$out/original-once.rs" "$src/$once"
python3 - "$study" "$out" <<'PY'
import importlib.util
from pathlib import Path
import sys
sys.dont_write_bytecode=True
study,out=map(Path,sys.argv[1:])
s=importlib.util.spec_from_file_location('copy_timing_corpus',study/'run-timings.py');m=importlib.util.module_from_spec(s);s.loader.exec_module(m)
(out/'focus-corpus.rum').write_text(m.corpus(),encoding='utf-8')
PY
cpu="$(python3 -c 'import os;print(min(os.sched_getaffinity(0)))')"
# Six complete permutations balance positions and predecessor pairs for 3 modes.
orders=('base-a candidate base-b' 'candidate base-b base-a' 'base-b base-a candidate' 'base-b candidate base-a' 'base-a base-b candidate' 'candidate base-a base-b')
block=0
for order in "${orders[@]}"; do
    block=$((block+1))
    for mode in $order; do
        binary="$out/binaries/base";if [[ "$mode" = candidate ]];then binary="$out/binaries/candidate";fi
        # A clean RUM environment; script-local profiling-off variables still apply.
        python3 - "$repo" "$src" "$out" "$binary" "$cpu" "$block" "$mode" <<'PY'
import os
from pathlib import Path
import subprocess
import sys
repo,src,out,binary,cpu,block,mode=sys.argv[1:];out=Path(out)
env={k:v for k,v in os.environ.items() if not k.startswith('RUM_')};env['RUM_CORE_MICRO_RETAINED_MODULE_GC_BATCHES']='3'
bounded=['python3',str(Path(repo)/'scripts/run-bounded.py'),'--timeout','900','--','taskset','-c',cpu]
with (out/f'suite-{block}-{mode}.log').open('w') as log:
 subprocess.run(bounded+['bash',str(Path(src)/'scripts/core-language-microbench.sh'),'--rum-cli',binary,'--output-dir',str(out/f'suite-{block}-{mode}'),'--scale','5','--reps','9','--timeout','600'],env=env,stdin=subprocess.DEVNULL,stdout=log,stderr=subprocess.STDOUT,check=True)
r=subprocess.run(bounded+[binary,'--file',str(out/'focus-corpus.rum')],env=env,stdin=subprocess.DEVNULL,capture_output=True,timeout=915)
(out/f'focus-{block}-{mode}.stdout').write_bytes(r.stdout);(out/f'focus-{block}-{mode}.stderr').write_bytes(r.stderr)
assert r.returncode==0 and b'COPY_TIMING_PASS' in r.stdout and not r.stderr.strip()
PY
    done
done
python3 - "$out" <<'PY'
import csv,json,math,re,sys
from fractions import Fraction
from pathlib import Path
from statistics import median
out=Path(sys.argv[1]);results={}
for lane in ['suite','focus']:
 values={}
 for block in range(1,7):
  for mode in ['base-a','candidate','base-b']:
   if lane=='suite':
    with (out/f'suite-{block}-{mode}/timings.tsv').open() as f:rows=list(csv.DictReader(f,delimiter='\t'))
    current={r['label']:float(r['rum_seconds']) for r in rows};assert len(current)==112
   else:
    text=(out/f'focus-{block}-{mode}.stdout').read_text().replace('\\t','\t');samples={}
    for label,rep,seconds in re.findall(r'COPY_TIME\t([^\t\n]+)\t(\d+)\t([^"\s]+)',text):samples.setdefault(label,[]).append(float(Fraction(seconds.replace('*^','e'))))
    assert len(samples)==8 and all(len(v)==11 for v in samples.values());current={k:median(v) for k,v in samples.items()}
   values[block,mode]=current
 results[lane]=[{"label":label,"candidate_over_base_pair":[values[b,'candidate'][label]/math.sqrt(values[b,'base-a'][label]*values[b,'base-b'][label]) for b in range(1,7)]} for label in values[1,'base-a']]
(out/'primary-replication.json').write_text(json.dumps({'status':'PASS','blocks':6,'scope':'fresh primary replication; no historical build comparison or whole-application speedup claim','results':results},indent=2)+'\n')
PY
printf 'PASS: fresh primary replication retained all full-suite rows and focused samples.\n'
