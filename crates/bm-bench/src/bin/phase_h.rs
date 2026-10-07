//! E-H: v0.2 held-out validation on worlds written after the `bitmind-v0.2` freeze
//! (`bm_worlds::heldout2`). The learner crates may not change during Phase H.
//! H1 relational transfer to new objects and never-seen magnitudes; H2 cyclic hidden ranks;
//! H3 identity/label-permutation invariance of H1 and H2; H4 reachable conflict resolution;
//! H5 long horizon under a memory budget. Every answer is classified (PREREG-v0.2 section 5).
//! Floats only for reporting.

use bm_memory::*;
use bm_relation::{Abstain, Answer, FeatureKind, RelationEngine};
use bm_worlds::heldout2::{self as h2, Mechanism, ObjWorld};
use std::collections::{BTreeMap, HashSet};
use std::fmt::Write as _;

struct Out {
    report: String,
    json: Vec<String>,
    all: bool,
}

impl Out {
    fn gate(&mut self, name: &str, pass: bool, detail: String) {
        self.all &= pass;
        let _ = writeln!(self.report, "  {name:<34} {}  {detail}", if pass { "PASS" } else { "FAIL" });
        self.json.push(format!("{{\"gate\":\"{name}\",\"pass\":{pass},\"detail\":{:?}}}", detail));
    }
}

/// Pre-registered outcome classes.
#[derive(Default, Clone, Copy, Debug)]
struct Classes {
    correct: u32,
    wrong: u32,
    safe_abstain: u32,
    insufficient: u32,
    leakage: u32,
}

impl Classes {
    fn n(&self) -> u32 {
        self.correct + self.wrong + self.safe_abstain + self.insufficient + self.leakage
    }
    fn s(&self) -> String {
        format!(
            "correct relational {}, wrong {}, safe abstention {}, insufficient evidence {}, identity leakage {} of {}",
            self.correct,
            self.wrong,
            self.safe_abstain,
            self.insufficient,
            self.leakage,
            self.n()
        )
    }
}

struct Agent {
    g: Grounder,
    rel: RelationEngine,
}

impl Agent {
    fn new(seed: u64) -> Self {
        let mut rel = RelationEngine::new(seed ^ 0x48);
        rel.identity_channel = Some(INST_CH);
        bm_bench::declare(&mut rel, h2::ORDINAL);
        Agent { g: Grounder::new(seed), rel }
    }
    fn feed(&mut self, ev: Event) {
        if let Some(gr) = self.g.observe(ev) {
            if let Some(ep) = to_episode_scene(&gr) {
                self.rel.observe(ep);
            }
        }
    }
    /// Query: the episode and target of "does b's ON change?" for a previewed event.
    fn query(&mut self, ev: &Event, truth: &[(u16, usize)], b: usize) -> Option<(bm_relation::Episode, u32)> {
        let gr = self.g.ground(ev);
        let slot = truth.iter().find(|x| x.1 == b)?.0;
        let r = scene_role_order(&gr)?.iter().position(|&s| s == slot)?;
        let ep = to_episode_scene(&gr)?.without_outcomes();
        Some((ep, target_id(r, h2::ON) + CHANGE))
    }
    /// Classify one answer. `new_objects`: the query involves an object never seen in training.
    fn classify(&mut self, q: &bm_relation::Episode, t: u32, truth: i64, new_objects: bool, c: &mut Classes) -> Option<i64> {
        let ans = self.rel.predict(q, t);
        match &ans {
            Answer::Value { val, laws, .. } => {
                // identity-specific answer: a law naming a mark or a concept identity, used on an
                // object it was never about
                let names_identity = laws.iter().any(|&l| {
                    self.rel.laws[l].condition.iter().any(|f| matches!(f, FeatureKind::Abs { ch, .. } if *ch == h2::MARK || *ch == INST_CH))
                });
                if new_objects && names_identity {
                    c.leakage += 1;
                } else if *val == truth {
                    c.correct += 1;
                } else {
                    c.wrong += 1;
                }
                Some(*val)
            }
            Answer::Abstain(Abstain::Conflict) => {
                c.safe_abstain += 1;
                None
            }
            Answer::Abstain(_) => {
                c.insufficient += 1;
                None
            }
        }
    }
}

fn pass_rate(c: &Classes) -> bool {
    c.n() > 0 && c.correct * 100 >= c.n() * 90 && c.wrong * 100 <= c.n() * 2 && c.leakage == 0
}

// ------------------------------------------------------------------------ H1 relational transfer
/// Returns the classes and the answers in query order (for H3).
fn h1(seed: u64, relabel: u64) -> (Classes, Vec<Option<i64>>) {
    let mut w = ObjWorld::new(Mechanism::Fit, seed, 12, (1, 10), relabel, "h1");
    let mut a = Agent::new(seed);
    for _ in 0..1500 {
        let p = w.rng().sample_distinct(12, 2);
        let (ev, _) = w.step(h2::PLACE, vec![p[0], p[1]]);
        a.feed(ev);
    }
    // test: six never-seen objects with never-seen sizes 11..16
    let new = w.add_objects(6, (11, 16));
    let mut c = Classes::default();
    let mut answers = Vec::new();
    for k in 0..200 {
        let x = new[k % new.len()];
        let y = if k % 2 == 0 { new[(k / 2 + 1) % new.len()] } else { (k * 7) % 12 };
        let (pa, pb) = if (k / 3) % 2 == 0 { (x, y) } else { (y, x) };
        if pa == pb {
            answers.push(None);
            continue;
        }
        let (ev, truth) = w.preview(h2::PLACE, vec![pa, pb]);
        let Some((q, t)) = a.query(&ev, &truth, pb) else {
            c.insufficient += 1;
            answers.push(None);
            continue;
        };
        answers.push(a.classify(&q, t, w.effect(pa, pb) as i64, true, &mut c));
    }
    (c, answers)
}

// ------------------------------------------------------------------------ H2 cyclic hidden ranks
fn h2_rank(seed: u64, relabel: u64) -> (Classes, Classes, Vec<Option<i64>>, u32) {
    let n = 12;
    let mut w = ObjWorld::new(Mechanism::Rank, seed, n, (1, 10), relabel, "h2");
    let mut a = Agent::new(seed);
    let mut held: Vec<(usize, usize)> = Vec::new();
    for i in 0..n {
        for j in 0..n {
            if i != j && w.rng().below(5) == 0 {
                held.push((i, j));
            }
        }
    }
    let mut seen = vec![0u32; n];
    let mut fed = 0;
    while fed < 2000 {
        let p = w.rng().sample_distinct(n, 2);
        if held.contains(&(p[0], p[1])) {
            continue;
        }
        seen[p[0]] += 1;
        seen[p[1]] += 1;
        let (ev, _) = w.step(h2::PROBE, vec![p[0], p[1]]);
        a.feed(ev);
        fed += 1;
    }
    let lat = a.rel.licensed_in(w.context).into_iter().filter(|&l| a.rel.laws[l].condition.iter().any(|f| match f {
        FeatureKind::Abs { ch, .. } | FeatureKind::Same { ch, .. } | FeatureKind::Diff { ch, .. } => *ch >= 3000,
        _ => false,
    })).count() as u32;
    let (mut all, mut est) = (Classes::default(), Classes::default());
    let mut answers = Vec::new();
    for &(i, j) in &held {
        let (ev, truth) = w.preview(h2::PROBE, vec![i, j]);
        let Some((q, t)) = a.query(&ev, &truth, j) else {
            all.insufficient += 1;
            answers.push(None);
            continue;
        };
        let tv = w.effect(i, j) as i64;
        let ans = a.classify(&q, t, tv, false, &mut all);
        // pre-registered oracle: established if both objects took part in >= 5 training probes
        if seen[i] >= 5 && seen[j] >= 5 {
            let mut tmp = Classes::default();
            a.classify(&q, t, tv, false, &mut tmp);
            est.correct += tmp.correct;
            est.wrong += tmp.wrong;
            est.safe_abstain += tmp.safe_abstain;
            est.insufficient += tmp.insufficient;
            est.leakage += tmp.leakage;
        }
        answers.push(ans);
    }
    (all, est, answers, lat)
}

// ------------------------------------------------------------------------ H4 reachable conflict
struct H4 {
    stage1: Classes,
    unsettled_answers: u32,
    saved: u32,
    settled_ok: u32,
    settled_wrong: u32,
    other_answers: u32,
    sought: u32,
    probes: u32,
    /// distinct region pairs probed when each "other path" answer arrived (reporting only)
    other_evidence: Vec<usize>,
}

fn h4(seed: u64) -> H4 {
    let n = 20;
    let mut w = ObjWorld::new(Mechanism::Spike, seed, n, (1, 10), 0, "h4");
    let mut a = Agent::new(seed);
    let spike = |w: &ObjWorld, i: usize| w.objs[i].shape == h2::SPIKE;
    let same = |w: &ObjWorld, i: usize, j: usize| w.objs[i].colour == w.objs[j].colour;
    // the conflict region: a spike acting on a same-colour partner. Half of its pairs are the
    // saved queries (never probed); the other half can supply evidence later.
    let mut region: Vec<(usize, usize)> = Vec::new();
    for i in 0..n {
        for j in 0..n {
            if i != j && spike(&w, i) && same(&w, i, j) {
                region.push((i, j));
            }
        }
    }
    let queries: Vec<(usize, usize)> = region.iter().copied().enumerate().filter(|(k, _)| k % 2 == 0).map(|x| x.1).collect();
    let evidence: HashSet<(usize, usize)> = region.iter().copied().enumerate().filter(|(k, _)| k % 2 == 1).map(|x| x.1).collect();
    // phase 1: the region is never probed
    let mut fed = 0;
    while fed < 2500 {
        let p = w.rng().sample_distinct(n, 2);
        if spike(&w, p[0]) && same(&w, p[0], p[1]) {
            continue;
        }
        let (ev, _) = w.step(h2::TOUCH, vec![p[0], p[1]]);
        a.feed(ev);
        fed += 1;
    }
    // stage 1: the agent's licensed laws conflict on the region; it must not answer unsettled
    let mut st = H4 { stage1: Classes::default(), unsettled_answers: 0, saved: 0, settled_ok: 0, settled_wrong: 0, other_answers: 0, sought: 0, probes: 0, other_evidence: Vec::new() };
    let mut probed_region: HashSet<(usize, usize)> = HashSet::new();
    let mut saved: Vec<(bm_relation::Episode, u32, i64)> = Vec::new();
    for &(i, j) in &queries {
        let (ev, truth) = w.preview(h2::TOUCH, vec![i, j]);
        let Some((q, t)) = a.query(&ev, &truth, j) else { continue };
        let ce = a.rel.conflict_evidence(&q, t);
        let unresolved = !ce.is_empty() && ce.iter().any(|c| c.2 < bm_relation::engine::MIN_SHARED_INDEPENDENT);
        let tv = w.effect(i, j) as i64;
        let ans = a.classify(&q, t, tv, false, &mut st.stage1);
        if ans.is_some() && unresolved {
            st.unsettled_answers += 1;
        }
        if ans.is_none() {
            saved.push((q, t, tv));
        }
    }
    st.saved = saved.len() as u32;
    // stage 2: life goes on; the agent previews candidates and probes one on which its licensed
    // laws disagree (never a saved query pair). Saved queries are re-judged every 50 probes.
    let mut done = vec![false; saved.len()];
    while st.probes < 4000 && done.iter().any(|d| !d) {
        let mut choice = None;
        for _ in 0..20 {
            let p = w.rng().sample_distinct(n, 2);
            if queries.contains(&(p[0], p[1])) {
                continue;
            }
            let (pv, tr) = w.preview(h2::TOUCH, vec![p[0], p[1]]);
            if let Some((q, t)) = a.query(&pv, &tr, p[1]) {
                let vals: std::collections::BTreeSet<i64> = a.rel.explain(&q, t).into_iter().map(|x| x.1).collect();
                if vals.len() > 1 {
                    choice = Some((p[0], p[1]));
                    break;
                }
            }
            if choice.is_none() && evidence.contains(&(p[0], p[1])) {
                choice = Some((p[0], p[1]));
            }
        }
        let (i, j) = match choice {
            Some(c) => {
                st.sought += 1;
                c
            }
            None => {
                let p = w.rng().sample_distinct(n, 2);
                if queries.contains(&(p[0], p[1])) {
                    continue;
                }
                (p[0], p[1])
            }
        };
        if evidence.contains(&(i, j)) {
            probed_region.insert((i, j));
        }
        let (ev, _) = w.step(h2::TOUCH, vec![i, j]);
        a.feed(ev);
        st.probes += 1;
        if st.probes % 50 == 0 {
            for (k, (q, t, tv)) in saved.iter().enumerate() {
                if done[k] {
                    continue;
                }
                let ce = a.rel.conflict_evidence(q, *t);
                let settled = ce.is_empty() || ce.iter().all(|c| c.2 >= bm_relation::engine::MIN_SHARED_INDEPENDENT);
                if let Some(v) = a.rel.predict(q, *t).value() {
                    done[k] = true;
                    if !settled {
                        st.unsettled_answers += 1;
                    } else if ce.is_empty() {
                        st.other_answers += 1;
                        st.other_evidence.push(probed_region.len());
                        if v != *tv {
                            st.settled_wrong += 1;
                        }
                    } else if v == *tv {
                        st.settled_ok += 1;
                    } else {
                        st.settled_wrong += 1;
                    }
                }
            }
        }
    }
    st
}

// ------------------------------------------------------------------------ H5 long horizon
struct H5 {
    mid: Classes,
    end: Classes,
    peak_mb: f64,
    lost_counterexamples: u32,
    trace: Vec<(u32, u64, usize)>,
}

fn h5(seed: u64, steps: u32) -> H5 {
    let n = 20;
    let mut w = ObjWorld::new(Mechanism::Fit, seed, n, (1, 12), 0, "h5");
    let mut a = Agent::new(seed);
    let mut held: Vec<(usize, usize)> = Vec::new();
    for i in 0..n {
        for j in 0..n {
            if i != j && w.rng().below(5) == 0 {
                held.push((i, j));
            }
        }
    }
    let probe_set: Vec<(usize, usize)> = held.iter().copied().take(100).collect();
    let evaluate = |a: &mut Agent, w: &mut ObjWorld| -> Classes {
        let mut c = Classes::default();
        for &(i, j) in &probe_set {
            let (ev, truth) = w.preview(h2::PLACE, vec![i, j]);
            if let Some((q, t)) = a.query(&ev, &truth, j) {
                a.classify(&q, t, w.effect(i, j) as i64, false, &mut c);
            } else {
                c.insufficient += 1;
            }
        }
        c
    };
    let mut res = H5 { mid: Classes::default(), end: Classes::default(), peak_mb: 0.0, lost_counterexamples: 0, trace: Vec::new() };
    let mut audit: BTreeMap<(usize, u64), u32> = BTreeMap::new();
    let mut step = 0u32;
    while step < steps {
        let r = w.rng().below(10);
        let (act, args) = if r < 6 {
            let p = w.rng().sample_distinct(n, 2);
            if held.contains(&(p[0], p[1])) {
                continue;
            }
            (h2::PLACE, vec![p[0], p[1]])
        } else if r < 8 {
            (h2::TOGGLE, vec![w.rng().below(n as u64) as usize])
        } else {
            (h2::WAIT, vec![w.rng().below(n as u64) as usize])
        };
        let (ev, _) = w.step(act, args);
        a.feed(ev);
        step += 1;
        if step % 2000 == 0 {
            let ws = bm_bench::working_set_mb().unwrap_or(-1.0);
            res.peak_mb = res.peak_mb.max(bm_bench::peak_working_set_mb().unwrap_or(-1.0));
            res.trace.push((step, ws as u64, a.rel.laws.len()));
        }
        if step == steps / 2 {
            res.mid = evaluate(&mut a, &mut w);
            for l in &a.rel.laws {
                for e in &l.ctx {
                    if e.counters() > 0 {
                        audit.insert((l.id, e.context), e.counters());
                    }
                }
            }
        }
    }
    res.end = evaluate(&mut a, &mut w);
    for ((l, c), before) in &audit {
        let now = a.rel.laws[*l].ctx(*c).map(|e| e.counters()).unwrap_or(0);
        if now < *before {
            res.lost_counterexamples += 1;
        }
    }
    res.peak_mb = res.peak_mb.max(bm_bench::peak_working_set_mb().unwrap_or(-1.0));
    res
}

fn main() {
    let started = std::time::Instant::now();
    let seeds: Vec<u64> = std::env::args().nth(1).map(|s| s.split(',').filter_map(|x| x.parse().ok()).collect()).unwrap_or(vec![999]);
    let only = std::env::var("ONLY_H").ok();
    let run = |k: &str| only.as_deref().map(|o| o.split(',').any(|x| x == k)).unwrap_or(true);
    let h5_steps: u32 = std::env::var("H5_STEPS").ok().and_then(|x| x.parse().ok()).unwrap_or(20_000);
    let mut out = Out { report: String::new(), json: Vec::new(), all: true };
    for &seed in &seeds {
        let _ = writeln!(out.report, "\n===== seed {seed}");
        let relabel = seed ^ 0x3E1A;
        if run("1") || run("3") {
            let (c1, ans1) = h1(seed, 0);
            if run("1") {
                out.gate("H1 relational transfer", pass_rate(&c1), format!("seed {seed}: new objects, never-seen sizes: {}", c1.s()));
            }
            if run("3") {
                let (_, ans1r) = h1(seed, relabel);
                let same = ans1.iter().zip(&ans1r).filter(|(x, y)| x == y).count();
                out.gate("H3a permutation invariance (H1)", same * 100 >= ans1.len() * 99, format!("seed {seed}: identical answers under relabelled identities and labels {same}/{}", ans1.len()));
            }
        }
        if run("2") || run("3") {
            let (all, est, ans2, lat) = h2_rank(seed, 0);
            if run("2") {
                out.gate("H2 cyclic hidden ranks", pass_rate(&est) && est.n() > 0, format!("seed {seed}: never-probed pairs, established {}; all {}; licensed latent laws {lat}", est.s(), all.s()));
            }
            if run("3") {
                let (_, _, ans2r, _) = h2_rank(seed, relabel);
                let same = ans2.iter().zip(&ans2r).filter(|(x, y)| x == y).count();
                out.gate("H3b permutation invariance (H2)", same * 100 >= ans2.len() * 99, format!("seed {seed}: identical answers under relabelled identities and labels {same}/{}", ans2.len()));
            }
        }
        if run("4") {
            let r = h4(seed);
            let answered = r.settled_ok + r.settled_wrong + r.other_answers;
            out.gate(
                "H4 reachable conflict",
                r.unsettled_answers == 0 && r.settled_wrong == 0 && r.stage1.wrong == 0 && r.saved > 0 && r.settled_ok * 100 >= r.saved * 90,
                format!(
                    "seed {seed}: stage 1 {}; answers while unsettled {}; saved {}; after {} probes ({} chosen to test the conflict): settled by >= 3 independent bindings and correct {}, wrong {}, answered after the conflict dissolved (a competing licence lost to counterevidence) {} with distinct region pairs probed by then {:?} (of {answered} answered; only settled answers count for the gate)",
                    r.stage1.s(), r.unsettled_answers, r.saved, r.probes, r.sought, r.settled_ok, r.settled_wrong, r.other_answers, r.other_evidence
                ),
            );
        }
        if run("5") {
            let r = h5(seed, h5_steps);
            let acc = |c: &Classes| c.correct * 100 / c.n().max(1);
            out.gate(
                "H5 long horizon, memory budget",
                r.peak_mb > 0.0 && r.peak_mb <= 2048.0 && acc(&r.end) + 2 >= acc(&r.mid) && r.lost_counterexamples == 0,
                format!(
                    "seed {seed}: {} steps; peak working set {:.0} MB (budget 2048); mid {}; end {}; counterexample counts decreased {}; trace (step, MB, laws) {:?}",
                    h5_steps, r.peak_mb, r.mid.s(), r.end.s(), r.lost_counterexamples, r.trace
                ),
            );
        }
    }
    let _ = writeln!(out.report, "\n  learner: frozen at tag bitmind-v0.2 (crates hdc-core, bm-relation, bm-memory, bm-agent unchanged)");
    println!("{}", out.report);
    println!("E-H OVERALL: {}", if out.all { "PASS" } else { "FAIL" });
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../experiments/results/phase_h.json");
    std::fs::write(&path, format!("{{\"overall_pass\":{},\"gates\":[\n{}\n]}}", out.all, out.json.join(",\n"))).expect("write");
    println!("{}", bm_bench::resources_line(started));
}
