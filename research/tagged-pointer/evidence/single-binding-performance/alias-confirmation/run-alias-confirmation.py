"""Separate focused check with a/b labels sharing each actual binary file."""
import argparse
from collections import Counter
from datetime import datetime, timezone
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import shutil
from statistics import median
import subprocess
import sys
import tempfile
import time
sys.dont_write_bytecode = True

IMPL = ['base', 'matcher', 'single']
MODES = [m + '-' + a for a in ['a', 'b'] for m in IMPL]
LABELS = ['focus-increment-16000', 'focus-miss-16000',
          'synthetic-rewrite-heavy-hit-16000', 'synthetic-mixed-hit-16000']
LIMIT = ('Separate focused confirmation; never pooled with the original blocks. '
         'Cannot replace the 112-label gate or establish layout, alias-path or lock-state causality. '
         'A/A ratios are descriptive controls and are never subtracted. '
         'Eleven repetitions are not eleven independent process blocks.')


def sha(p):
    return hashlib.sha256(Path(p).read_bytes()).hexdigest()


def load(p):
    return json.loads(Path(p).read_text(encoding='utf-8'))


def save(p, v):
    Path(p).write_text(json.dumps(v, indent=2) + '\n', encoding='utf-8')


def utc():
    return datetime.now(timezone.utc).isoformat()


def module(name, p):
    spec = importlib.util.spec_from_file_location(name, p)
    obj = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(obj)
    return obj


def balance(orders):
    assert len(orders) == 12
    assert all(Counter(o[p] for o in orders) == Counter({m: 2 for m in MODES}) for p in range(6))
    pairs = Counter((o[i], o[i + 1]) for o in orders for i in range(5))
    assert len(pairs) == 30 and set(pairs.values()) == {2}


def binary_checks(plan):
    for m in IMPL:
        a, b = [plan['binaries'][m + '-' + x] for x in ['a', 'b']]
        assert a == b
        p = Path(a['binary'])
        st = p.stat()
        assert os.access(p, os.X_OK) and sha(p) == a['sha256']
        assert [st.st_dev, st.st_ino] == a['file_identity']


def execute(out, binary, prefix, timed, plan):
    env = {k: v for k, v in os.environ.items() if not k.startswith('RUM_')}
    env['RUM_CORE_MICRO_RETAINED_MODULE_GC_BATCHES'] = '3'
    corpus = out / ('timed-corpus.rum' if timed else 'preflight-corpus.rum')
    cmd = ['python3', plan['bounded'], '--timeout', '90', '--', 'taskset', '-c',
           str(plan['cpu']), binary, '--file', str(corpus)]
    r = subprocess.run(cmd, stdin=subprocess.DEVNULL, capture_output=True, env=env, timeout=100)
    (out / (prefix + '.stdout')).write_bytes(r.stdout)
    (out / (prefix + '.stderr')).write_bytes(r.stderr)
    save(out / (prefix + '.status.json'), {'exit_code': r.returncode, 'command': cmd})
    assert r.returncode == 0 and not r.stderr.strip() and b'ALIAS_CONFIRM_PASS' in r.stdout
    assert b'rum-work\t' not in r.stdout + r.stderr


def worker(out):
    plan = load(out / 'timing-plan.json')
    binary_checks(plan)
    with (out / 'timing-progress.tsv').open('w', encoding='utf-8', buffering=1) as f:
        f.write('block\tposition\tmode\tstatus\tutc\n')
        for block, order in enumerate(plan['orders'], 1):
            for pos, mode in enumerate(order, 1):
                f.write(f'{block}\t{pos}\t{mode}\tstarted\t{utc()}\n')
                execute(out, plan['binaries'][mode]['binary'], f'run-{block:02d}-{mode}', True, plan)
                f.write(f'{block}\t{pos}\t{mode}\tpassed\t{utc()}\n')
            print('alias confirmation block', block, 'PASS', flush=True)
    binary_checks(plan)
    (out / 'timing-status.txt').write_text('TIMINGS_PASS\n', encoding='utf-8')


def run(out, research):
    parent = out.parent
    builds = load(parent / 'builds.json')
    oldplan = load(parent / 'timing-plan.json')
    protocol = load(out / 'session-protocol.json')
    assert builds['status'] == 'PASS' and protocol['recorded_before_timing']
    assert not (out / 'native-execution.json').exists()
    assert oldplan['cpu'] in os.sched_getaffinity(0)
    native = Path(tempfile.mkdtemp(prefix='rum-alias-confirm-evidence-', dir='/var/tmp'))
    inputs = native / 'study-inputs'
    inputs.mkdir()
    driver = Path(__file__).resolve()
    shutil.copy2(driver, native / 'run-alias-confirmation.py')
    shutil.copy2(driver.with_name('alias_confirmation_analysis.py'), native / 'alias_confirmation_analysis.py')
    for name in ['workloads.py', 'statistics_helpers.py', 'session_audit.py']:
        shutil.copy2(research / name, inputs / name)
    for name in ['builds.json', 'timing-plan.json']:
        shutil.copy2(parent / name, native / ('original-' + name))
    shutil.copy2(out / 'session-protocol.json', native / 'session-protocol.json')
    w = module('alias_workloads', inputs / 'workloads.py')
    w.FAMILIES, w.FOCUS_SIZES = ('increment', 'miss'), (16000,)
    w.PIPELINE_DEFINITIONS = [x for x in w.PIPELINE_DEFINITIONS if x[1] == 16000 and 'mixed-control' not in x[0]]
    for timed in [False, True]:
        source = w.focus_source(timed) + w.pipeline_source(timed) + 'Print["ALIAS_CONFIRM_PASS"];\n'
        (native / ('timed-corpus.rum' if timed else 'preflight-corpus.rum')).write_text(source, encoding='utf-8')
    bounded = research.parents[2] / 'scripts/run-bounded.py'
    binaries = {}
    for v in builds['variants']:
        p = Path(v['binary']).resolve()
        st = p.stat()
        item = {'binary': str(p), 'sha256': v['binary_sha256'], 'file_identity': [st.st_dev, st.st_ino]}
        for a in ['a', 'b']:
            binaries[v['mode'] + '-' + a] = item
    orders = [[MODES[(i + row) % 6] for i in [0, 1, 5, 2, 4, 3]] for row in range(12)]
    balance(orders)
    files = ['run-alias-confirmation.py', 'alias_confirmation_analysis.py', 'study-inputs/workloads.py', 'study-inputs/statistics_helpers.py',
             'study-inputs/session_audit.py', 'original-builds.json', 'original-timing-plan.json',
             'session-protocol.json', 'timed-corpus.rum', 'preflight-corpus.rum']
    plan = {'recorded_on': protocol['recorded_on'], 'blocks': 12, 'modes': MODES, 'orders': orders,
            'binaries': binaries, 'cpu': oldplan['cpu'], 'retained_reps': 11, 'labels': LABELS,
            'bounded': str(bounded), 'bounded_sha256': sha(bounded),
            'original_builds_sha256': sha(parent / 'builds.json'),
            'source_sha256': {n: sha(native / n) for n in files}, 'started_utc': utc(),
            'environment': {'remove_prefix': 'RUM_', 'RUM_CORE_MICRO_RETAINED_MODULE_GC_BATCHES': '3'},
            'method': 'Twelve balanced Williams blocks; a/b share one actual binary file per implementation; fixed original CPU.',
            'limitations': LIMIT}
    save(native / 'timing-plan.json', plan)
    record = {'status': 'RUNNING', 'native_evidence': str(native), 'started_utc': utc(),
              'idle_seconds_before_timing': 30, 'formal_timeout_seconds': 900}
    print('Native alias confirmation evidence:', native, flush=True)
    try:
        binary_checks(plan)
        for mode in IMPL:
            execute(native, binaries[mode + '-a']['binary'], 'preflight-' + mode, False, plan)
        (native / 'preflight-status.txt').write_text('PREFLIGHT_PASS\n', encoding='utf-8')
        print('Untimed alias preflight PASS; 30-second idle interval', flush=True)
        time.sleep(30)
        subprocess.run(['python3', str(bounded), '--timeout', '900', '--', 'python3', '-B',
                        str(native / 'run-alias-confirmation.py'), str(native), '--worker'],
                       check=True, timeout=915)
        record['status'] = 'PASS'
    except BaseException as e:
        record.update(status='FAILED', error_type=type(e).__name__, error=str(e))
        raise
    finally:
        record['finished_utc'] = utc()
        save(native / 'native-execution.json', record)
        subprocess.run(['python3', str(bounded), '--timeout', '120', '--', 'cp', '-a',
                        str(native) + '/.', str(out)], check=True, timeout=135)


def analyze(out):
    helper = Path(__file__).with_name('alias_confirmation_analysis.py')
    module('alias_analysis', helper).analyze(out, globals())


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('evidence', type=Path)
    parser.add_argument('--analyze', action='store_true')
    parser.add_argument('--worker', action='store_true', help=argparse.SUPPRESS)
    parser.add_argument('--research', type=Path, default=Path(__file__).resolve().parents[1] / 'candidate/research/tagged-pointer/single-binding')
    args = parser.parse_args()
    out = args.evidence.resolve()
    if args.analyze:
        analyze(out)
    elif args.worker:
        worker(out)
    else:
        run(out, args.research.resolve())
