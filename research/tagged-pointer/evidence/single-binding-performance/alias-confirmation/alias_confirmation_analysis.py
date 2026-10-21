"""Independent retained-output audit for the separate alias confirmation."""
from pathlib import Path
from statistics import median


def analyze(out, api):
    load, sha, module = [api[n] for n in ['load', 'sha', 'module']]
    balance, binary_checks, save = [api[n] for n in ['balance', 'binary_checks', 'save']]
    IMPL, MODES, LABELS, LIMIT = [api[n] for n in ['IMPL', 'MODES', 'LABELS', 'LIMIT']]
    plan = load(out / 'timing-plan.json')
    assert load(out / 'native-execution.json')['status'] == 'PASS'
    assert (out / 'timing-status.txt').read_text().strip() == 'TIMINGS_PASS'
    assert (out / 'preflight-status.txt').read_text().strip() == 'PREFLIGHT_PASS'
    assert plan['labels'] == LABELS and plan['modes'] == MODES and plan['blocks'] == 12
    assert sha(api['__file__']) == plan['source_sha256']['run-alias-confirmation.py']
    assert sha(Path(api['__file__']).with_name('alias_confirmation_analysis.py')) == plan['source_sha256']['alias_confirmation_analysis.py']
    balance(plan['orders'])
    binary_checks(plan)
    assert sha(out.parent / 'builds.json') == plan['original_builds_sha256']
    assert all(sha(out / n) == h for n, h in plan['source_sha256'].items())
    assert sha(plan['bounded']) == plan['bounded_sha256']
    assert plan['cpu'] == load(out / 'original-timing-plan.json')['cpu']
    original = {v['mode']: v for v in load(out / 'original-builds.json')['variants']}
    for mode in IMPL:
        item = plan['binaries'][mode + '-a']
        assert item['sha256'] == original[mode]['binary_sha256']
        assert Path(item['binary']).resolve() == Path(original[mode]['binary']).resolve()
    assert plan['environment'] == {'remove_prefix': 'RUM_', 'RUM_CORE_MICRO_RETAINED_MODULE_GC_BATCHES': '3'}
    st = module('alias_stats', out / 'study-inputs/statistics_helpers.py')
    session = module('alias_session', out / 'study-inputs/session_audit.py').audit(out)
    journal = st.rows(out / 'timing-progress.tsv')
    assert len(journal) == 144
    values, samples = {}, {}
    for mode in IMPL:
        prefix = 'preflight-' + mode
        assert load(out / (prefix + '.status.json'))['exit_code'] == 0
        assert not (out / (prefix + '.stderr')).read_bytes().strip()
        assert b'ALIAS_CONFIRM_PASS' in (out / (prefix + '.stdout')).read_bytes()
        assert b'ISO_TIME' not in (out / (prefix + '.stdout')).read_bytes()
    for block, order in enumerate(plan['orders'], 1):
        for pos, mode in enumerate(order, 1):
            rows = [r for r in journal if int(r['block']) == block and r['mode'] == mode]
            assert [r['status'] for r in rows] == ['started', 'passed']
            assert all(int(r['position']) == pos for r in rows)
            prefix = f'run-{block:02d}-{mode}'
            status = load(out / (prefix + '.status.json'))
            assert status['exit_code'] == 0
            assert status['command'][:9] == ['python3', plan['bounded'], '--timeout', '90', '--',
                                            'taskset', '-c', str(plan['cpu']), plan['binaries'][mode]['binary']]
            assert not (out / (prefix + '.stderr')).read_bytes().strip()
            raw = st.retained(out / (prefix + '.stdout'), LABELS, 'ALIAS_CONFIRM_PASS')
            for label, data in raw.items():
                values[block, mode, label] = median(data)
                samples[block, mode, label] = data
    assert len(values) == 288 and sum(len(v) for v in samples.values()) == 3168
    results = [dict(st.compare(label, values, 12, MODES), raw_samples_by_block={
        m: [samples[b, m, label] for b in range(1, 13)] for m in MODES}) for label in LABELS]
    summary = {'status': 'PASS', 'recorded_on': plan['recorded_on'], 'plan': plan, 'session': session,
               'counts': {'processes': 72, 'retained_samples': 3168}, 'results': results, 'limitations': LIMIT,
               'raw_files_sha256': {p.relative_to(out).as_posix(): sha(p) for p in sorted(out.rglob('*'))
                 if p.is_file() and p.name not in {'summary.json', 'report.md'}}}
    save(out / 'summary.json', summary)
    lines = ['# Separate shared-file alias confirmation', '', LIMIT, '', '| Case | matcher/base | single/base | single/matcher | base A/A | matcher A/A | single A/A |',
             '|---|---:|---:|---:|---:|---:|---:|']
    keys = ['matcher/base', 'single/base', 'single/matcher', 'base-b/base-a', 'matcher-b/matcher-a', 'single-b/single-a']
    for r in results:
        lines.append('| ' + r['label'] + ' | ' + ' | '.join(f"{r['ratios'][k]['median']:.6f} ({r['ratios'][k]['below_one']}/12 <1)" for k in keys) + ' |')
    (out / 'report.md').write_text('\n'.join(lines) + '\n', encoding='utf-8')
    print('ALIAS_CONFIRM_ANALYSIS_PASS', flush=True)

