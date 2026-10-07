//! E-G: independent replication of the frozen BITMIND v0.1 learner in held-out worlds written after
//! the freeze (`bm_worlds::heldout`). The learner crates may not change during Phase G.
//! G1 real OS: hidden shared directories. G2 real OS (exploratory): hidden read-only attribute.
//! G3 conjunction of two relations on never-seen values. G4 anti-equivalence latent cause.
//! Writes experiments/results/phase_g.json. Floats only for reporting.

use bm_memory::*;
use bm_relation::{Answer, RelationEngine};
use bm_worlds::heldout::{self as hw, DirWorld, LockWorld, MagnetWorld, RoWorld};
use std::collections::HashSet;
use std::fmt::Write as _;

struct Out {
    report: String,
    json: Vec<String>,
    all: bool,
}

impl Out {
    fn gate(&mut self, name: &str, pass: bool, detail: String) {
        self.all &= pass;
        let _ = writeln!(self.report, "  {name:<30} {}  {detail}", if pass { "PASS" } else { "FAIL" });
        self.json.push(format!("{{\"gate\":\"{name}\",\"pass\":{pass},\"detail\":{:?}}}", detail));
    }
    fn info(&mut self, name: &str, detail: String) {
        let _ = writeln!(self.report, "  {name:<30} INFO  {detail}");
        self.json.push(format!("{{\"gate\":\"{name}\",\"pass\":null,\"detail\":{:?}}}", detail));
    }
}

#[derive(Default, Clone, Copy)]
struct Score {
    ok: u32,
    wrong: u32,
    abst: u32,
}

impl Score {
    fn add(&mut self, p: Option<i64>, t: i64) {
        match p {
            Some(v) if v == t => self.ok += 1,
            Some(_) => self.wrong += 1,
            None => self.abst += 1,
        }
    }
    fn n(&self) -> u32 {
        self.ok + self.wrong + self.abst
    }
    fn s(&self) -> String {
        format!("correct {} wrong {} abstain {} of {}", self.ok, self.wrong, self.abst, self.n())
    }
}

struct Agent {
    g: Grounder,
    rel: RelationEngine,
}

impl Agent {
    fn new(seed: u64) -> Self {
        let mut rel = RelationEngine::new(seed ^ 0x6A);
        rel.identity_channel = Some(INST_CH);
        Agent { g: Grounder::new(seed), rel }
    }
    fn feed(&mut self, ev: Event) {
        if let Some(gr) = self.g.observe(ev) {
            if let Some(ep) = to_episode_scene(&gr) {
                self.rel.observe(ep);
            }
        }
    }
    /// Predicted change of channel `ch` on the object `obj` for a live event (not learned from).
    fn predict(&mut self, ev: &Event, truth: &[(u16, usize)], obj: usize, ch: u16) -> (Option<i64>, bool) {
        let gr = self.g.ground(ev);
        let Some(slot) = truth.iter().find(|x| x.1 == obj).map(|x| x.0) else { return (None, false) };
        let Some(r) = scene_role_order(&gr).and_then(|o| o.iter().position(|&s| s == slot)) else { return (None, false) };
        let Some(ep) = to_episode_scene(&gr).map(|e| e.without_outcomes()) else { return (None, false) };
        let t = target_id(r, ch) + CHANGE;
        let ans = self.rel.predict(&ep, t);
        let ce = self.rel.conflict_evidence(&ep, t);
        let unresolved = !ce.is_empty() && ce.iter().any(|c| c.2 < bm_relation::engine::MIN_SHARED_INDEPENDENT);
        (ans.value(), unresolved)
    }
}

fn components(n: usize, pos: &[(usize, usize)]) -> Vec<usize> {
    let mut p: Vec<usize> = (0..n).collect();
    fn find(p: &mut Vec<usize>, x: usize) -> usize {
        if p[x] != x {
            let r = find(p, p[x]);
            p[x] = r;
        }
        p[x]
    }
    for &(a, b) in pos {
        let (ra, rb) = (find(&mut p, a), find(&mut p, b));
        p[ra] = rb;
    }
    (0..n).map(|x| find(&mut p, x)).collect()
}

/// Oracle: is the answer for (i, j) determined by the training evidence? Same class: connected by
/// trained positive pairs. Different class: some trained probe connected the two components.
fn established(n: usize, pos: &[(usize, usize)], neg: &[(usize, usize)], i: usize, j: usize, same: bool) -> bool {
    let c = components(n, pos);
    if same {
        c[i] == c[j]
    } else {
        neg.iter().any(|&(a, b)| (c[a] == c[i] && c[b] == c[j]) || (c[a] == c[j] && c[b] == c[i]))
    }
}

fn held_pairs(rng: &mut hdc_core::Rng, n: usize) -> HashSet<(usize, usize)> {
    let mut h = HashSet::new();
    for i in 0..n {
        for j in 0..n {
            if i != j && rng.below(5) == 0 {
                h.insert((i, j));
            }
        }
    }
    h
}

// --------------------------------------------------------------------- G1 hidden directories
fn g1(seed: u64, out: &mut Out) {
    let mut w = match DirWorld::new(seed, 12, 5, "g1") {
        Ok(w) => w,
        Err(e) => {
            out.gate("G1 hidden directories (OS)", false, format!("sandbox error {e}"));
            return;
        }
    };
    let mut a = Agent::new(seed);
    bm_bench::declare(&mut a.rel, hw::OS_ORDINAL);
    let n = w.files.len();
    let held = held_pairs(w.rng(), n);
    let (mut pos, mut neg) = (Vec::new(), Vec::new());
    let mut fed = 0;
    while fed < 3000 {
        let p = w.rng().sample_distinct(n, 2);
        if held.contains(&(p[0], p[1])) {
            continue;
        }
        if w.same_dir(p[0], p[1]) {
            pos.push((p[0], p[1]));
        } else {
            neg.push((p[0], p[1]));
        }
        let Ok((ev, _)) = w.step(p[0], p[1]) else { break };
        a.feed(ev);
        fed += 1;
    }
    let mut hv: Vec<(usize, usize)> = held.into_iter().collect();
    hv.sort();
    let (mut all, mut feas_clear, mut unres) = (Score::default(), Score::default(), 0u32);
    let (mut n_same, mut n_diff) = (0, 0);
    for (i, j) in hv {
        let same = w.same_dir(i, j);
        if same {
            n_same += 1;
        } else {
            n_diff += 1;
        }
        let Ok((ev, truth)) = w.step(i, j) else { break };
        let sj = truth.iter().find(|x| x.1 == j).map(|x| x.0).unwrap_or(0);
        let t_change = (ev.pre.value(sj, hw::LOC) != ev.post.value(sj, hw::LOC)) as i64;
        let (p, unresolved) = a.predict(&ev, &truth, j, hw::LOC);
        all.add(p, t_change);
        if unresolved {
            unres += 1;
        } else if established(n, &pos, &neg, i, j, same) {
            feas_clear.add(p, t_change);
        }
    }
    out.gate(
        "G1 hidden directories (OS)",
        feas_clear.ok * 100 >= feas_clear.n() * 90 && all.wrong * 100 <= all.n() * 5 && feas_clear.n() > 0,
        format!("seed {seed}: real folders in {}: never-probed pairs ({n_same} same folder, {n_diff} different): all {}; established without unresolved conflict {}; unresolved conflicts {unres}", w.root.display(), all.s(), feas_clear.s()),
    );
}

// ------------------------------------------------------------- G2 hidden read-only (explore)
fn g2(seed: u64, out: &mut Out) {
    let mut w = match RoWorld::new(seed, 8, "g2") {
        Ok(w) => w,
        Err(e) => {
            out.gate("G2 hidden read-only (OS)", false, format!("sandbox error {e}"));
            return;
        }
    };
    let mut a = Agent::new(seed);
    bm_bench::declare(&mut a.rel, hw::OS_ORDINAL);
    let n = w.names.len();
    let act = |w: &mut RoWorld| -> (u16, usize) {
        let f = w.rng().below(n as u64) as usize;
        match w.rng().below(4) {
            0 => (hw::LOCK, f),
            1 => (hw::UNLOCK, f),
            _ => (hw::WRITE, f),
        }
    };
    for _ in 0..3000 {
        let (k, f) = act(&mut w);
        let Ok((ev, _)) = w.step(k, f) else { break };
        a.feed(ev);
    }
    let (mut s, mut s_locked, mut s_free) = (Score::default(), Score::default(), Score::default());
    let mut done = 0;
    while done < 300 {
        let (k, f) = act(&mut w);
        if k != hw::WRITE {
            if let Ok((ev, _)) = w.step(k, f) {
                a.feed(ev);
            }
            continue;
        }
        let locked = w.locked(f);
        let Ok((ev, truth)) = w.step(hw::WRITE, f) else { break };
        let sf = truth.iter().find(|x| x.1 == f).map(|x| x.0).unwrap_or(0);
        let t_change = (ev.pre.value(sf, hw::CONTENT) != ev.post.value(sf, hw::CONTENT)) as i64;
        let (p, _) = a.predict(&ev, &truth, f, hw::CONTENT);
        s.add(p, t_change);
        if locked {
            s_locked.add(p, t_change);
        } else {
            s_free.add(p, t_change);
        }
        done += 1;
    }
    out.gate(
        "G2 hidden read-only (OS, explore)",
        s.wrong * 100 <= s.n() * 5,
        format!("seed {seed}: WRITE outcome with an unsensed, history-dependent attribute: all {}; on read-only files {}; on writable files {} (criterion: never confidently wrong; accuracy reported only)", s.s(), s_locked.s(), s_free.s()),
    );
}

// --------------------------------------------------- G3 conjunction of relations (held-out values)
fn g3(seed: u64, out: &mut Out) {
    let mut w = LockWorld::new(seed, "g3");
    let mut rel = RelationEngine::new(seed ^ 0x63);
    bm_bench::declare(&mut rel, hw::LOCK_ORDINAL);
    for _ in 0..1500 {
        let (k, l) = w.pair(false);
        let ep = w.episode(&k, &l);
        rel.observe(ep);
    }
    let mut s = Score::default();
    let mut s_pos = Score::default();
    for _ in 0..300 {
        let (k, l) = w.pair(true);
        let truth = LockWorld::opens(&k, &l) as i64;
        let q = w.episode(&k, &l).without_outcomes();
        let p = rel.predict(&q, hw::OPEN).value();
        s.add(p, truth);
        if truth == 1 {
            s_pos.add(p, truth);
        }
    }
    let lic: Vec<String> = rel.licensed_in(w.context).into_iter().take(4).map(|l| rel.laws[l].describe()).collect();
    out.gate(
        "G3 two-relation conjunction",
        s.ok * 100 >= s.n() * 95 && s.wrong * 100 <= s.n(),
        format!("seed {seed}: never-seen colours and sizes: {}; of which opening cases {}; licensed e.g. {:?}", s.s(), s_pos.s(), lic),
    );
}

// ------------------------------------------------------------- G4 anti-equivalence latent cause
fn g4(seed: u64, out: &mut Out) {
    let mut w = MagnetWorld::new(seed, 12, "g4");
    let mut a = Agent::new(seed);
    let n = w.toks.len();
    let held = held_pairs(w.rng(), n);
    let (mut pos, mut neg) = (Vec::new(), Vec::new());
    let mut fed = 0;
    while fed < 1500 {
        let p = w.rng().sample_distinct(n, 2);
        if held.contains(&(p[0], p[1])) {
            continue;
        }
        // "same class" here = same polarity (the outcome is the anti-equivalence)
        if !w.differ(p[0], p[1]) {
            pos.push((p[0], p[1]));
        } else {
            neg.push((p[0], p[1]));
        }
        let (ev, _) = w.step(p[0], p[1]);
        a.feed(ev);
        fed += 1;
    }
    if std::env::var("DIAG_G4L").is_ok() {
        for ind in &a.rel.latent {
            eprintln!("G4IND target {} channels {:?} classes {:?} obs {} versions ({}, {})", ind.target, ind.channels(), ind.classes(), ind.observations(), ind.block_version, ind.link_version);
        }
        let mut cands: Vec<(u32, String)> = a
            .rel
            .laws
            .iter()
            .filter(|l| l.condition.len() == 1 && l.condition.iter().any(|f| matches!(f, bm_relation::FeatureKind::Same { ch, .. } | bm_relation::FeatureKind::Diff { ch, .. } if *ch >= 3000)))
            .filter_map(|l| l.ctx(w.context).map(|e| (e.total(), a.rel.summary(l.id, w.context))))
            .collect();
        cands.sort_by(|x, y| y.0.cmp(&x.0));
        for (_, c) in cands.iter().take(8) {
            eprintln!("G4LAW {c}");
        }
    }
    let mut hv: Vec<(usize, usize)> = held.into_iter().collect();
    hv.sort();
    let (mut all, mut feas_clear) = (Score::default(), Score::default());
    for (i, j) in hv {
        let before = w.toks[j].2;
        let same = !w.differ(i, j);
        let (ev, truth) = w.step(i, j);
        let t_change = (w.toks[j].2 != before) as i64;
        let (p, unresolved) = a.predict(&ev, &truth, j, hw::ON);
        if std::env::var("DIAG_G4").is_ok() && p.is_some() && p != Some(t_change) {
            let gr = a.g.ground(&ev);
            let slot = truth.iter().find(|x| x.1 == j).map(|x| x.0).unwrap_or(0);
            if let (Some(r), Some(ep)) = (scene_role_order(&gr).and_then(|o| o.iter().position(|&s| s == slot)), to_episode_scene(&gr)) {
                let q = ep.without_outcomes();
                eprintln!("G4WRONG ({i},{j}) same-polarity {same} predicted {p:?} truth {t_change}");
                for (l, v) in a.rel.explain(&q, target_id(r, hw::ON) + CHANGE) {
                    eprintln!("    by {} => {v}", a.rel.summary(l, w.context));
                }
            }
        }
        all.add(p, t_change);
        if !unresolved && established(n, &pos, &neg, i, j, same) {
            feas_clear.add(p, t_change);
        }
    }
    let lat = a
        .rel
        .licensed_in(w.context)
        .into_iter()
        .filter(|&l| a.rel.laws[l].condition.iter().any(|f| matches!(f, bm_relation::FeatureKind::Same { ch, .. } | bm_relation::FeatureKind::Diff { ch, .. } if *ch >= 3000)))
        .count();
    out.gate(
        "G4 anti-equivalence latent",
        lat > 0 && feas_clear.ok * 100 >= feas_clear.n() * 90 && all.wrong * 100 <= all.n() * 5 && feas_clear.n() > 0,
        format!("seed {seed}: never-probed pairs: all {}; established without unresolved conflict {}; licensed latent laws {lat}", all.s(), feas_clear.s()),
    );
}

fn main() {
    let started = std::time::Instant::now();
    let seeds: Vec<u64> = std::env::args().nth(1).map(|s| s.split(',').filter_map(|x| x.parse().ok()).collect()).unwrap_or(vec![101, 102]);
    let only = std::env::var("ONLY_G").ok();
    let mut out = Out { report: String::new(), json: Vec::new(), all: true };
    let run = |k: &str| only.as_deref().map(|o| o.split(',').any(|x| x == k)).unwrap_or(true);
    for &seed in &seeds {
        let _ = writeln!(out.report, "\n===== seed {seed}");
        if run("1") {
            g1(seed, &mut out);
        }
        if run("2") {
            g2(seed, &mut out);
        }
        if run("3") {
            g3(seed, &mut out);
        }
        if run("4") {
            g4(seed, &mut out);
        }
    }
    let _ = &Answer::Abstain(bm_relation::Abstain::NoLicensedLaw);
    let _ = writeln!(out.report);
    out.info("learner", "frozen at tag bitmind-v0.1 (crates hdc-core, bm-relation, bm-memory, bm-agent unchanged)".to_string());
    println!("{}", out.report);
    println!("E-G OVERALL: {}", if out.all { "PASS" } else { "FAIL" });
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../experiments/results/phase_g.json");
    std::fs::write(&path, format!("{{\"overall_pass\":{},\"gates\":[\n{}\n]}}", out.all, out.json.join(",\n"))).expect("write");
    println!("{}", bm_bench::resources_line(started));
}
