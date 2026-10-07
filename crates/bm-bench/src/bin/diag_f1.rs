//! v0.2 F1 diagnosis (pre-registered): replay the Phase F one-shot recall test exactly and
//! attribute every miss to one failure class before any mechanism change.
//! Classes:
//! * collision: ground truth cannot separate the cue from another stored event (another event's
//!   true facts overlap the cue at least as much as the true event's);
//! * retrieval: the true event is not among the 32 candidates the HDC memory proposes;
//! * alignment-loss: the true event is a candidate but another candidate scores higher;
//! * alignment-tie: the true event ties with the best other candidate;
//! * decision-rule: the true event scores best but the rule abstains (lead < 2 tokens or less than
//!   half of the query explained);
//! * harness: anything else (e.g. the queried event was never stored).

use bm_memory::*;
use bm_worlds::ground::{GroundWorld, Setup};
use std::collections::BTreeMap;
use std::fmt::Write as _;

fn true_facts(e: &Event, truth: &[(u16, usize)]) -> Vec<u64> {
    use hdc_core::rng::mix64;
    let obj = |slot: u16| truth.iter().find(|x| x.0 == slot).map(|x| x.1 as u64).unwrap_or(u64::MAX);
    let mut v: Vec<u64> = Vec::new();
    for (ph, sc) in [(1u64, &e.pre), (2u64, &e.post)] {
        for t in &sc.tokens {
            v.push(mix64(mix64(obj(t.slot), ph), mix64(t.ch as u64, t.val as u64)));
        }
    }
    if let Some(a) = &e.act {
        v.push(mix64(0xAC7, a.id as u64));
        for (i, s) in a.args.iter().enumerate() {
            v.push(mix64(0xA6 + i as u64, obj(*s)));
        }
    }
    v.sort_unstable();
    v.dedup();
    v
}

fn overlap(a: &[u64], b: &[u64]) -> u32 {
    let (mut i, mut j, mut n) = (0, 0, 0u32);
    while i < a.len() && j < b.len() {
        match a[i].cmp(&b[j]) {
            std::cmp::Ordering::Less => i += 1,
            std::cmp::Ordering::Greater => j += 1,
            std::cmp::Ordering::Equal => {
                n += 1;
                i += 1;
                j += 1;
            }
        }
    }
    n
}

fn corrupt(e: &Event, miss_pct: u64, rng: &mut hdc_core::Rng) -> Event {
    let mut c = e.clone();
    for sc in [&mut c.pre, &mut c.post] {
        sc.tokens.retain(|_| rng.below(100) >= miss_pct);
    }
    c
}

fn main() {
    let started = std::time::Instant::now();
    let seeds: Vec<u64> = std::env::args().nth(1).map(|s| s.split(',').filter_map(|x| x.parse().ok()).collect()).unwrap_or((101..=115).collect());
    let mut report = String::new();
    let mut totals: BTreeMap<String, u32> = BTreeMap::new();
    for &seed in &seeds {
        // identical to phase_f::f1
        let setup = Setup { noise_pct: 5, miss_pct: 10, ..Default::default() };
        let mut w = GroundWorld::new(seed, 4, 4, setup, "f1");
        let mut g = Grounder::new(seed);
        let mut mem = EventMemory::new(seed ^ 0xF1);
        let mut ids = Vec::new();
        let mut truth_of: BTreeMap<u64, Vec<(u16, usize)>> = BTreeMap::new();
        for _ in 0..2000 {
            let (ev, truth) = w.step(None);
            let id = g.store.len() as u64;
            if let Some(gr) = g.observe(ev) {
                mem.store(&gr);
                truth_of.insert(id, truth);
                if id >= 150 {
                    ids.push(id);
                }
            }
        }
        if std::env::var("RECONSOLIDATE").is_ok() {
            // D032 as in Phase B2: re-encode every stored event with the mature grounding
            let regrounded: Vec<Grounded> = truth_of.keys().map(|&id| g.ground(&g.store.get(id).clone())).collect();
            mem.reconsolidate(regrounded.iter());
        }
        let stored_true: Vec<(u64, Vec<u64>)> = truth_of.iter().map(|(&id, tr)| (id, true_facts(g.store.get(id), tr))).collect();
        let nm = g.noise_model();
        let mut rng = hdc_core::Rng::new(seed ^ 0xF11);
        let mut per: BTreeMap<String, u32> = BTreeMap::new();
        for miss in [30u64, 50] {
            for q in 0..200 {
                let id = ids[(q * 9) % ids.len()];
                let orig = g.store.get(id).clone();
                let cue = corrupt(&orig, miss, &mut rng);
                let gq = g.ground(&cue);
                let ans = mem.restore_calibrated(&gq, &cue, &nm).map(|(h, _)| h.id);
                if ans == Some(id) {
                    continue;
                }
                let outcome = if ans.is_some() { "wrong" } else { "abstain" };
                let tr = mem.recall_trace(&gq, &cue, id);
                let Some(truth) = truth_of.get(&id) else {
                    *per.entry(format!("{miss}% {outcome} harness")).or_insert(0) += 1;
                    continue;
                };
                let qf = true_facts(&cue, truth);
                let mine = overlap(&qf, &stored_true.iter().find(|x| x.0 == id).map(|x| x.1.clone()).unwrap_or_default());
                let rival = stored_true.iter().filter(|x| x.0 != id).map(|x| overlap(&qf, &x.1)).max().unwrap_or(0);
                let class = if rival >= mine {
                    "collision"
                } else if tr.true_rank.is_none() {
                    "retrieval"
                } else if tr.best.map(|b| b.0 != id && b.1 > tr.true_score).unwrap_or(false) {
                    "alignment-loss"
                } else if tr.best.map(|b| b.0 != id && b.1 == tr.true_score).unwrap_or(false) || (tr.true_rank.is_some() && tr.true_score == tr.second_score && tr.true_rank != Some(0)) {
                    "alignment-tie"
                } else if tr.true_rank == Some(0) {
                    "decision-rule"
                } else {
                    "harness"
                };
                if std::env::var("DIAG_F1_DETAIL").is_ok() {
                    let _ = writeln!(report, "  seed {seed} {miss}% q{q} event {id}: {outcome} {class}: answered {ans:?}; oracle overlap true {mine} best rival {rival}; trace {tr:?}");
                }
                *per.entry(format!("{miss}% {outcome} {class}")).or_insert(0) += 1;
            }
        }
        let _ = writeln!(report, "seed {seed}: {per:?}");
        for (k, v) in per {
            *totals.entry(k).or_insert(0) += v;
        }
    }
    println!("{report}");
    println!("TOTAL over seeds {seeds:?}: {totals:?}");
    println!("{}", bm_bench::resources_line(started));
}
