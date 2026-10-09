"""Aggregate the formal I-series per-seed outputs (one process per seed) and evaluate the
pre-registered criteria (PREREG-capability-h4 section 6) over all seeds of each world.
Usage: python experiments/aggregate_iseries.py experiments/results/iseries > SUMMARY.txt"""
import re, sys, os, glob, collections

D = sys.argv[1]
LINE = re.compile(r'^\s+(Agent|Random|Wait)\s+saved (\d+): settled-correct (\d+) \(active (\d+)\), dissolved-correct (\d+) \(active (\d+)\), wrong (\d+) \(while unresolved (\d+)\), unsettled answers (\d+), unresolved (\d+), leakage (\d+); region probes (\d+); chosen diagnostic-new (\d+) redundant (\d+) non-diagnostic (\d+); steps with an informative candidate (\d+) \(diagnostic-new taken (\d+)\), with none (\d+); actions (\d+); ([\d.]+) s')
KEYS = ['saved', 's_ok', 's_act', 'd_ok', 'd_act', 'wrong', 'wrong_unres', 'unsettled', 'unresolved', 'leak', 'region', 'dnew', 'dred', 'nond', 'inf_steps', 'dnew_offered', 'none_steps', 'steps', 'secs']


def pct(a, b):
    return 100.0 * a / b if b else 0.0


for W in ['I1', 'I2', 'I3', 'I4']:
    files = sorted(glob.glob(os.path.join(D, f'{W}_*.txt')))
    if not files:
        continue
    tot = {p: collections.Counter() for p in ['Agent', 'Random', 'Wait']}
    stage1 = []
    peaks = []
    walls = []
    for f in files:
        seed = f.rsplit('_', 1)[1][:-4]
        for line in open(f, encoding='utf-8', errors='replace'):
            m = LINE.match(line)
            if m:
                p = m.group(1)
                for k, v in zip(KEYS, m.groups()[1:]):
                    tot[p][k] += float(v)
            m2 = re.search(r'===== seed \d+: (.*)$', line)
            if m2:
                stage1.append(f'{seed}: {m2.group(1)}')
            m3 = re.search(r'resources: wall ([\d.]+) s, peak working set (\d+) MB', line)
            if m3:
                walls.append(float(m3.group(1)))
                peaks.append(int(m3.group(2)))
    print(f'===== {W}: {len(files)} seeds')
    for s in stage1:
        print('  stage 1 seed', s)
    for p in ['Agent', 'Random', 'Wait']:
        t = tot[p]
        res = t['s_ok'] + (t['d_act'] if p == 'Agent' else t['d_ok'])
        ans = t['s_ok'] + t['d_ok'] + t['wrong']
        print(f"  {p:<6} saved {int(t['saved'])}: resolved correctly {int(res)} ({pct(res, t['saved']):.1f}%) [settled {int(t['s_ok'])}, dissolved {int(t['d_ok'])} (agent-active {int(t['d_act'])})], wrong {int(t['wrong'])} ({pct(t['wrong'], ans):.1f}% of answers; while unresolved {int(t['wrong_unres'])}), unsettled answers {int(t['unsettled'])}, still abstaining {int(t['unresolved'])} ({pct(t['unresolved'], t['saved']):.1f}%), leakage {int(t['leak'])}; region probes {int(t['region'])}; chosen diagnostic-new {int(t['dnew'])} / redundant {int(t['dred'])} / non-diagnostic {int(t['nond'])}; diagnostic-new taken in {pct(t['dnew_offered'], t['inf_steps']):.1f}% of {int(t['inf_steps'])} steps with an informative candidate; no informative candidate in {pct(t['none_steps'], t['steps']):.1f}% of {int(t['steps'])} steps; policy time {t['secs']:.0f} s")
    ag, rd, wt = tot['Agent'], tot['Random'], tot['Wait']
    r_ag = pct(ag['s_ok'] + ag['d_act'], ag['saved'])
    r_rd = pct(rd['s_ok'] + rd['d_ok'], rd['saved'])
    r_wt = pct(wt['s_ok'] + wt['d_ok'], wt['saved'])
    ans = ag['s_ok'] + ag['d_ok'] + ag['wrong']
    g = []
    if W in ('I1', 'I2'):
        g += [(f'resolved correctly {r_ag:.1f}% >= 80%', r_ag >= 80), (f"wrong while unresolved {int(ag['wrong_unres'])} = 0", ag['wrong_unres'] == 0),
              (f"wrong {pct(ag['wrong'], ans):.1f}% of answers <= 2%", pct(ag['wrong'], ans) <= 2), (f'agent {r_ag:.1f}% >= random {r_rd:.1f}% + 30 pp', r_ag >= r_rd + 30),
              (f'WAIT-only {r_wt:.1f}% <= 10%', r_wt <= 10), (f"leakage {int(ag['leak'])} = 0", ag['leak'] == 0)]
        if W == 'I2':
            d = pct(ag['dnew_offered'], ag['inf_steps'])
            g.append((f'diagnostic-new {d:.1f}% of steps with a diagnostic candidate >= 80%', d >= 80))
    elif W == 'I3':
        still = pct(ag['unresolved'], ag['saved'])
        none = pct(ag['none_steps'], ag['steps'])
        g += [(f"wrong {int(ag['wrong'])} = 0", ag['wrong'] == 0), (f'still abstaining {still:.1f}% >= 95%', still >= 95), (f'no informative probe in {none:.1f}% of steps >= 90%', none >= 90)]
    else:
        corr = ag['s_ok'] + ag['d_ok']
        sh = pct(ag['s_ok'], corr)
        g += [(f'resolved correctly {r_ag:.1f}% >= 80%', r_ag >= 80), (f'settlement route {sh:.1f}% of correct >= 70%', sh >= 70),
              (f"settlement audit {int(ag['unsettled'])} = 0", ag['unsettled'] == 0), (f'agent {r_ag:.1f}% >= random {r_rd:.1f}% + 30 pp', r_ag >= r_rd + 30)]
    for s, ok in g:
        print(f"  {'PASS' if ok else 'FAIL'} {s}")
    print(f"  {W} overall: {'PASS' if all(ok for _, ok in g) else 'FAIL'}")
    if peaks:
        print(f'  resources: per-seed process peak {min(peaks)}-{max(peaks)} MB, wall {min(walls):.0f}-{max(walls):.0f} s per seed (total {sum(walls)/3600:.1f} h)')
    print()
