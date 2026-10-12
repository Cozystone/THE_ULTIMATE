"""Aggregate the formal N-series (one process per family and seed) and evaluate the criteria of
PREREG-observation-evidence section 5 exactly as written.
Usage: python experiments/aggregate_nseries.py experiments/results/nseries > SUMMARY.txt"""
import re, sys, os, glob, collections

D = sys.argv[1]
LINE = re.compile(r'^\s+(Agent|Random|Wait|Oracle)\s+saved (\d+): settled-correct (\d+) \(active (\d+)\), dissolved-correct (\d+) \(active (\d+)\), wrong (\d+) \(while unresolved (\d+)\), unsettled answers (\d+), unresolved (\d+), leakage (\d+); region probes (\d+); chosen diagnostic-new (\d+) redundant (\d+) non-diagnostic (\d+); steps with an informative candidate (\d+) \(diagnostic-new taken (\d+)\), with none (\d+); actions (\d+); evidence items (\d+) \(verified (\d+), truly in region (\d+)\); unresolved needing re-observation (\d+); reliable-vs-cheap steps (\d+) \(chose LINK (\d+)\); ([\d.]+) s')
KEYS = ['saved', 's_ok', 's_act', 'd_ok', 'd_act', 'wrong', 'wrong_unres', 'unsettled', 'unresolved', 'leak', 'region', 'dnew', 'dred', 'nond', 'inf_steps', 'dnew_offered', 'none_steps', 'steps', 'ev_items', 'ev_ver', 'ev_reg', 'end_unver', 'n4_both', 'n4_link', 'secs']


def pct(a, b):
    return 100.0 * a / b if b else 0.0


def load(pattern):
    tot = collections.defaultdict(collections.Counter)
    stage = []
    peaks, walls, files = [], [], sorted(glob.glob(pattern))
    for f in files:
        seed = re.search(r'_(\d+)\.txt$', f).group(1)
        for line in open(f, encoding='utf-8', errors='replace'):
            m = LINE.match(line)
            if m:
                for k, v in zip(KEYS, m.groups()[1:]):
                    tot[m.group(1)][k] += float(v)
            m2 = re.search(r'===== seed \d+: (.*)$', line)
            if m2:
                stage.append((seed, m2.group(1)))
            m3 = re.search(r'resources: wall ([\d.]+) s, peak working set (\d+) MB', line)
            if m3:
                walls.append(float(m3.group(1)))
                peaks.append(int(m3.group(2)))
    return tot, stage, peaks, walls, files


overall = []
grand = collections.Counter()
for F in ['N0', 'N1', 'N2', 'N3', 'N4']:
    tot, stage, peaks, walls, files = load(os.path.join(D, f'{F}_*.txt'))
    if not files:
        continue
    otot, _, opeaks, _, ofiles = load(os.path.join(D, f'oracle_{F}_*.txt'))
    print(f'===== {F}: {len(files)} seeds' + (f'; region oracle {len(ofiles)} seeds' if ofiles else ''))
    pre_wrong = 0
    for seed, s in stage:
        m = re.search(r'answered wrong (\d+)', s)
        pre_wrong += int(m.group(1)) if m else 0
        print(f'  stage 1 seed {seed}: {s}')
    for p in ['Agent', 'Random', 'Wait']:
        t = tot[p]
        res = t['s_ok'] + (t['d_act'] if p == 'Agent' else t['d_ok'])
        print(f"  {p:<6} saved {int(t['saved'])}: resolved correctly {int(res)} ({pct(res, t['saved']):.1f}%) [settled {int(t['s_ok'])}, dissolved {int(t['d_ok'])} (agent-active {int(t['d_act'])})], wrong {int(t['wrong'])}, audit violations {int(t['unsettled'])}, still abstaining {int(t['unresolved'])} ({pct(t['unresolved'], t['saved']):.1f}%; needing re-observation {int(t['end_unver'])}), leakage {int(t['leak'])}; region probes {int(t['region'])}; diagnostic-new {int(t['dnew'])} / redundant {int(t['dred'])} / non-diagnostic {int(t['nond'])}; no informative candidate in {pct(t['none_steps'], t['steps']):.1f}% of {int(t['steps'])} steps; evidence items {int(t['ev_items'])} (verified {int(t['ev_ver'])}, truly in region {int(t['ev_reg'])}); reliable-vs-cheap {int(t['n4_both'])} (LINK {int(t['n4_link'])})")
    if ofiles:
        o = otot['Oracle']
        print(f"  Oracle saved {int(o['saved'])}: resolved correctly {int(o['s_ok'] + o['d_ok'])} ({pct(o['s_ok'] + o['d_ok'], o['saved']):.1f}%), wrong {int(o['wrong'])}  [harness-only reachability bound]")
    ag, rd, wt = tot['Agent'], tot['Random'], tot['Wait']
    r_ag = pct(ag['s_ok'] + ag['d_act'], ag['saved'])
    r_rd = pct(rd['s_ok'] + rd['d_ok'], rd['saved'])
    r_wt = pct(wt['s_ok'] + wt['d_ok'], wt['saved'])
    g = []
    if F == 'N0':
        g += [(f'resolved correctly {r_ag:.1f}% >= 80%', r_ag >= 80), (f'agent {r_ag:.1f}% >= random {r_rd:.1f}% + 30 pp', r_ag >= r_rd + 30), (f'WAIT {r_wt:.1f}% <= 10%', r_wt <= 10)]
    elif F in ('N1', 'N2', 'N4'):
        o = otot['Oracle']
        reach = o['s_ok'] + o['d_ok']
        agres = ag['s_ok'] + ag['d_act']
        ratio = pct(agres, reach)
        g += [(f'agent resolves {int(agres)} >= 50% of the oracle-reachable {int(reach)} ({ratio:.1f}%)', reach > 0 and agres * 2 >= reach),
              (f'agent {r_ag:.1f}% >= random {r_rd:.1f}% + 20 pp', r_ag >= r_rd + 20), (f'WAIT {r_wt:.1f}% <= 10%', r_wt <= 10)]
        if F == 'N4':
            sh = pct(ag['n4_link'], ag['n4_both'])
            g.append((f"reliable action chosen in {sh:.1f}% of {int(ag['n4_both'])} steps with both offered >= 90%", ag['n4_both'] > 0 and sh >= 90))
    elif F == 'N3':
        still = pct(ag['unresolved'], ag['saved'])
        none = pct(ag['none_steps'], ag['steps'])
        g += [(f'still abstaining {still:.1f}% >= 95%', still >= 95), (f'no informative candidate in {none:.1f}% of steps >= 90%', none >= 90)]
    for s, ok in g:
        print(f"  {'PASS' if ok else 'FAIL'} {s}")
    fam_ok = all(ok for _, ok in g)
    print(f"  {F} family criteria: {'PASS' if fam_ok else 'FAIL'}")
    print(f'  pre-action wrong answers on region queries (reported, not gated): {pre_wrong}')
    if peaks:
        print(f'  resources: per-seed process peak {min(peaks)}-{max(peaks)} MB, wall {min(walls):.0f}-{max(walls):.0f} s per seed (total {sum(walls)/3600:.1f} h)' + (f'; oracle peaks up to {max(opeaks)} MB' if opeaks else ''))
    print()
    overall.append((F, fam_ok))
    for k in ('wrong', 'leak', 'unsettled'):
        grand[k] += ag[k] + rd[k] + wt[k] if k != 'unsettled' else ag[k]
    grand['agent_wrong'] += ag['wrong']

fb = open(os.path.join(D, 'freeze_before.txt'), encoding='utf-8', errors='replace').read() if os.path.exists(os.path.join(D, 'freeze_before.txt')) else ''
fa = open(os.path.join(D, 'freeze_after.txt'), encoding='utf-8', errors='replace').read() if os.path.exists(os.path.join(D, 'freeze_after.txt')) else ''
print('===== all families (PREREG section 5)')
for s, ok in [(f"agent wrong answers on saved conflicts (incl. wrong settlements) {int(grand['agent_wrong'])} = 0", grand['agent_wrong'] == 0),
              (f"identity leakage (all policies) {int(grand['leak'])} = 0", grand['leak'] == 0),
              (f"settlement audit violations (agent) {int(grand['unsettled'])} = 0", grand['unsettled'] == 0),
              ('freeze guard before and after', 'OK' in fb and 'OK' in fa)]:
    print(f"  {'PASS' if ok else 'FAIL'} {s}")
print('  families: ' + ', '.join(f"{f} {'PASS' if ok else 'FAIL'}" for f, ok in overall))
