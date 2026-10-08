//! E-R0: smallest relation-learning falsification test.
//! Separates class learning, memorized lookup, relational learning and transfer.
//! Writes experiments/results/r0.json. Floats only for reporting.

use bm_bench::baselines::{ClassLearner, LookupLearner};
use bm_relation::*;
use bm_worlds::r0::*;
use std::fmt::Write as _;

struct Learners {
    rel: RelationEngine,
    class: ClassLearner,
    class_lvl: ClassLearner,
    lookup: LookupLearner,
}

impl Learners {
    fn new(seed: u64) -> Self {
        Learners {
            rel: { let mut r = RelationEngine::new(seed); bm_bench::declare(&mut r, bm_worlds::r0::ORDINAL); r },
            class: ClassLearner::new(seed ^ 1, false),
            class_lvl: ClassLearner::new(seed ^ 1, true),
            lookup: LookupLearner::new(seed ^ 2),
        }
    }
    fn train(&mut self, ep: &Episode) {
        self.class.train(ep, TARGET);
        self.class_lvl.train(ep, TARGET);
        self.lookup.train(ep, TARGET);
        self.rel.observe(ep.clone());
    }
}

#[derive(Default, Clone)]
struct Score {
    correct: u32,
    wrong: u32,
    abstain: u32,
}

impl Score {
    fn add(&mut self, pred: Option<i64>, truth: i64) {
        match pred {
            Some(p) if p == truth => self.correct += 1,
            Some(_) => self.wrong += 1,
            None => self.abstain += 1,
        }
    }
    fn n(&self) -> u32 {
        self.correct + self.wrong + self.abstain
    }
    fn acc(&self) -> f64 {
        self.correct as f64 / self.n().max(1) as f64
    }
    fn json(&self) -> String {
        format!("{{\"correct\":{},\"wrong\":{},\"abstain\":{},\"acc\":{:.4}}}", self.correct, self.wrong, self.abstain, self.acc())
    }
}

/// Balanced held-out set: equal number of cases per true outcome.
fn balanced(world: &mut R0World, pool: Pool, per_class: usize, classes: &[i64]) -> Vec<(Ent, Ent, i64)> {
    let mut out = Vec::new();
    for &c in classes {
        let mut got = 0;
        let mut tries = 0;
        while got < per_class && tries < 200_000 {
            tries += 1;
            let (a, b, _) = world.intervene(pool);
            let t = world.truth(&a, &b);
            if t == c {
                out.push((a, b, t));
                got += 1;
            }
        }
    }
    out
}

struct Eval {
    class: Score,
    class_lvl: Score,
    lookup: Score,
    rel: Score,
    oracle: Score,
}

fn evaluate(l: &mut Learners, world: &mut R0World, cases: &[(Ent, Ent, i64)]) -> Eval {
    let mut e = Eval {
        class: Score::default(),
        class_lvl: Score::default(),
        lookup: Score::default(),
        rel: Score::default(),
        oracle: Score::default(),
    };
    for (a, b, t) in cases {
        let ep = world.episode(a, b, *t, Kind::Intervention).without_outcomes();
        e.class.add(l.class.predict(&ep), *t);
        e.class_lvl.add(l.class_lvl.predict(&ep), *t);
        e.lookup.add(l.lookup.predict(&ep), *t);
        e.rel.add(l.rel.predict(&ep, TARGET).value(), *t);
        e.oracle.add(Some(world.truth(a, b)), *t);
    }
    e
}

/// False-licence audit (v2): every licensed law is checked against the oracle on fresh
/// intervention episodes from all pools. Returns (licensed, false, checked applications).
fn audit(rel: &mut RelationEngine, world: &mut R0World, confound: bool) -> (usize, usize, u32) {
    let ctx = world.context;
    let lic = rel.licensed_in(ctx);
    if lic.is_empty() {
        return (0, 0, 0);
    }
    if confound {
        return (lic.len(), lic.len(), 0);
    }
    let mut bad: std::collections::HashSet<usize> = std::collections::HashSet::new();
    let mut checked = 0u32;
    for i in 0..2000 {
        let pool = match i % 3 {
            0 => Pool::Train,
            1 => Pool::Test,
            _ => Pool::Mixed,
        };
        let (a, b, _) = world.intervene(pool);
        let truth = world.truth(&a, &b);
        let ep = world.episode(&a, &b, truth, Kind::Intervention).without_outcomes();
        for (l, v) in rel.explain(&ep, TARGET) {
            checked += 1;
            if v != truth {
                bad.insert(l);
            }
        }
    }
    (lic.len(), bad.len(), checked)
}

fn licensed_report(rel: &RelationEngine, ctx: u64) -> (Vec<String>, usize, usize) {
    let ids = rel.licensed_in(ctx);
    let relational = ids.iter().filter(|&&i| rel.laws[i].is_relational()).count();
    let lines = ids.iter().take(12).map(|&i| rel.summary(i, ctx)).collect();
    (lines, ids.len(), relational)
}

fn find_law(rel: &RelationEngine, f: impl Fn(&RelationLaw) -> bool) -> Option<usize> {
    rel.laws.iter().find(|l| f(l)).map(|l| l.id)
}

fn table(name: &str, seen: &Eval, held: &Eval) -> String {
    let mut s = String::new();
    let _ = writeln!(s, "  {name}");
    let _ = writeln!(s, "    learner              seen-acc  held-out acc  held-out wrong  held-out abstain");
    for (n, a, b) in [
        ("class (random codes)", &seen.class, &held.class),
        ("class (level codes)", &seen.class_lvl, &held.class_lvl),
        ("memorized lookup", &seen.lookup, &held.lookup),
        ("RELATIONAL", &seen.rel, &held.rel),
        ("oracle", &seen.oracle, &held.oracle),
    ] {
        let _ = writeln!(s, "    {n:<20} {:>7.3}   {:>10.3}   {:>12}   {:>14}", a.acc(), b.acc(), b.wrong, b.abstain);
    }
    s
}

fn json_eval(e: &Eval) -> String {
    format!(
        "{{\"class\":{},\"class_level\":{},\"lookup\":{},\"relational\":{},\"oracle\":{}}}",
        e.class.json(),
        e.class_lvl.json(),
        e.lookup.json(),
        e.rel.json(),
        e.oracle.json()
    )
}

const N_TRAIN: usize = 800;

/// C1-C4: train on interventions, test on seen and held-out cases.
fn standard(kind: R0Kind, seed: u64, held_pool: Pool, classes: &[i64], report: &mut String, json: &mut Vec<String>) -> bool {
    let mut world = R0World::new(kind, seed, "room-A");
    let mut l = Learners::new(seed.wrapping_mul(31) ^ 0xB17);
    let mut train_cases = Vec::new();
    for _ in 0..N_TRAIN {
        let (a, b, o) = world.intervene(Pool::Train);
        let ep = world.episode(&a, &b, o, Kind::Intervention);
        l.train(&ep);
        train_cases.push((a, b, o));
    }
    let seen: Vec<(Ent, Ent, i64)> = train_cases.iter().step_by(4).take(200).copied().collect();
    let held = balanced(&mut world, held_pool, 150, classes);
    let es = evaluate(&mut l, &mut world, &seen);
    let eh = evaluate(&mut l, &mut world, &held);
    let ctx = world.context;
    let (lines, n_lic, n_rel) = licensed_report(&l.rel, ctx);
    let chance = 1.0 / classes.len() as f64;
    let rel_transfer = l
        .rel
        .licensed_in(ctx)
        .into_iter()
        .filter(|&i| l.rel.laws[i].is_relational())
        .map(|i| l.rel.laws[i].ctx(ctx).map(|e| e.transfer.ok).unwrap_or(0))
        .max()
        .unwrap_or(0);
    let (_, n_false, n_checked) = audit(&mut l.rel, &mut world, false);
    let (pass, crit) = match kind {
        R0Kind::Equality | R0Kind::Order => {
            let n = eh.rel.n() as f64;
            let base_wrong = |sc: &Score| sc.wrong as f64 / n >= 0.20;
            let v2_bound = eh.class.acc() <= chance + 0.15 && eh.class_lvl.acc() <= chance + 0.15 && eh.lookup.acc() <= chance + 0.15;
            let p = n_false == 0
                && eh.rel.acc() >= 0.95
                && eh.rel.wrong == 0
                && base_wrong(&eh.class)
                && base_wrong(&eh.class_lvl)
                && base_wrong(&eh.lookup)
                && n_rel >= 1
                && rel_transfer >= 3;
            (p, format!("v3: rel held-out >= 0.95 with 0 wrong, every baseline >= 20% wrong, licensed relational law with >= 3 pre-registered transfer successes (max {rel_transfer}); v2 bound (baselines <= {:.2}) would give {}", chance + 0.15, if v2_bound { "PASS" } else { "FAIL" }))
        }
        R0Kind::NoRelation => {
            let abst = eh.rel.abstain as f64 / eh.rel.n() as f64;
            (n_lic == 0 && n_false == 0 && abst >= 0.90, format!("zero licensed laws (got {n_lic}), relational abstains >= 90% (got {:.3})", abst))
        }
        R0Kind::AbsClass => {
            let abs_ok = l.rel.licensed_in(ctx).iter().any(|&i| {
                l.rel.laws[i].condition.iter().any(|f| matches!(f, FeatureKind::Abs { role: 0, ch: COLOUR, val: 3 }))
                    && !l.rel.laws[i].is_relational()
            });
            let same_lic = l.rel.licensed_in(ctx).iter().any(|&i| {
                l.rel.laws[i].condition.len() == 1
                    && matches!(l.rel.laws[i].condition[0], FeatureKind::Same { ch: COLOUR, .. })
            });
            let v1 = if n_rel == 0 { "PASS" } else { "FAIL" };
            (
                abs_ok && !same_lic && n_false == 0,
                format!("v2: r0.colour=3 licensed ({abs_ok}); unconditional same(colour) not licensed ({}); false licences 0 (got {n_false}); v1 criterion no-relational-licence would give {v1} ({n_rel} relational)", !same_lic),
            )
        }
        _ => (false, String::new()),
    };
    let _ = writeln!(report, "\n[{kind:?}] seed {seed}  laws generated {}  licensed {n_lic} (relational {n_rel})", l.rel.laws.len());
    report.push_str(&table(&format!("{kind:?}"), &es, &eh));
    for line in &lines {
        let _ = writeln!(report, "    {line}");
    }
    // show the fate of the most informative relational candidate even if not licensed
    if let Some(id) = find_law(&l.rel, |law| {
        law.condition.len() == 1 && matches!(law.condition[0], FeatureKind::Same { ch: COLOUR, .. })
    }) {
        let _ = writeln!(report, "    same(colour) law: {}", l.rel.summary(id, ctx));
        let comp = l.rel.competitors(id, ctx);
        if !comp.is_empty() {
            let _ = writeln!(report, "      competing hypotheses: {:?}", comp.iter().take(5).collect::<Vec<_>>());
        }
        for &c in l.rel.laws[id].lineage.children.iter().take(3) {
            let _ = writeln!(report, "      child: {}", l.rel.summary(c, ctx));
        }
    }
    let _ = writeln!(report, "    false-licence audit: {n_false} false of {n_lic} licensed ({n_checked} law applications checked)");
    let _ = writeln!(report, "    criterion: {crit}  => {}", if pass { "PASS" } else { "FAIL" });
    json.push(format!(
        "{{\"condition\":\"{kind:?}\",\"seed\":{seed},\"seen\":{},\"heldout\":{},\"licensed\":{n_lic},\"licensed_relational\":{n_rel},\"false_licences\":{n_false},\"pass\":{pass}}}",
        json_eval(&es),
        json_eval(&eh)
    ));
    pass
}

/// C5: passive-only confound, then interventions.
fn confound(seed: u64, report: &mut String, json: &mut Vec<String>) -> bool {
    let mut world = R0World::new(R0Kind::Confound, seed, "room-A");
    let mut rel = RelationEngine::new(seed ^ 0xC5);
    bm_bench::declare(&mut rel, bm_worlds::r0::ORDINAL);
    for _ in 0..N_TRAIN {
        let (a, b, o) = world.passive();
        let ep = world.episode(&a, &b, o, Kind::Observation);
        rel.observe(ep);
    }
    let ctx = world.context;
    let lic_passive = rel.licensed_in(ctx).len();
    let same = find_law(&rel, |law| {
        law.condition.len() == 1 && matches!(law.condition[0], FeatureKind::Same { ch: COLOUR, .. })
    });
    let s1 = same.map(|i| rel.summary(i, ctx)).unwrap_or_default();
    let st1 = same.map(|i| rel.laws[i].status_in(ctx));
    for _ in 0..N_TRAIN {
        let (a, b, o) = world.intervene(Pool::Train);
        let ep = world.episode(&a, &b, o, Kind::Intervention);
        rel.observe(ep);
    }
    let lic_after = rel.licensed_in(ctx).len();
    for &i in rel.licensed_in(ctx).iter().take(5) {
        let _ = writeln!(report, "    [licensed after interventions] {}", rel.summary(i, ctx));
        if let Some(e) = rel.laws[i].ctx(ctx) {
            let ids = e.supporting_ids();
            let obs = ids.iter().filter(|&&x| rel.store.get(x).kind == Kind::Observation).count();
            let _ = writeln!(report, "      supporting episodes: {} ({} passive observations, {} interventions); created at tick {}", ids.len(), obs, ids.len() - obs, rel.laws[i].lineage.created_t);
        }
    }
    let s2 = same.map(|i| rel.summary(i, ctx)).unwrap_or_default();
    let st2 = same.map(|i| rel.laws[i].status_in(ctx));
    let exposed = matches!(st2, Some(Status::Revoked) | Some(Status::Contested) | Some(Status::Split));
    let pass = lic_passive == 0 && lic_after == 0 && exposed;
    let _ = writeln!(report, "\n[Confound] seed {seed}");
    let _ = writeln!(report, "    after {N_TRAIN} passive observations: licensed {lic_passive}; same(colour) status {:?}", st1.map(|s| s.name()));
    let _ = writeln!(report, "      {s1}");
    let _ = writeln!(report, "    after {N_TRAIN} interventions:      licensed {lic_after}; same(colour) status {:?}", st2.map(|s| s.name()));
    let _ = writeln!(report, "      {s2}");
    let _ = writeln!(report, "    criterion: nothing licensed from observation; confound exposed by intervention => {}", if pass { "PASS" } else { "FAIL" });
    json.push(format!(
        "{{\"condition\":\"Confound\",\"seed\":{seed},\"licensed_passive\":{lic_passive},\"licensed_after_intervention\":{lic_after},\"same_status_passive\":\"{}\",\"same_status_after\":\"{}\",\"pass\":{pass}}}",
        st1.map(|s| s.name()).unwrap_or("none"),
        st2.map(|s| s.name()).unwrap_or("none")
    ));
    pass
}

/// C6: negative-transfer guard (ATANOR DS1 in miniature).
fn negative_transfer(seed: u64, report: &mut String, json: &mut Vec<String>) -> bool {
    let mut wa = R0World::new(R0Kind::Equality, seed, "room-A");
    let mut lived = RelationEngine::new(seed ^ 0xC6);
    bm_bench::declare(&mut lived, bm_worlds::r0::ORDINAL);
    for _ in 0..N_TRAIN {
        let (a, b, o) = wa.intervene(Pool::Train);
        let ep = wa.episode(&a, &b, o, Kind::Intervention);
        lived.observe(ep);
    }
    let lic_a = lived.licensed_in(wa.context).len();
    // room-B: same entities and channels, inverted law, different observable context
    let mut wb = R0World::new(R0Kind::Inverted, seed, "room-B");
    let mut naive = RelationEngine::new(seed ^ 0xC6);
    bm_bench::declare(&mut naive, bm_worlds::r0::ORDINAL);
    let (mut lived_s, mut ablate_s, mut naive_s) = (Score::default(), Score::default(), Score::default());
    let (mut lived_first, mut naive_first) = (None, None);
    let n_b = 400;
    for i in 0..n_b {
        let (a, b, o) = wb.intervene(Pool::Train);
        let ep = wb.episode(&a, &b, o, Kind::Intervention);
        let q = ep.without_outcomes();
        let pl_ans = lived.predict(&q, TARGET);
        let pl = pl_ans.value();
        if pl.is_some() && pl != Some(o) {
            if let Answer::Value { laws, .. } = &pl_ans {
                eprintln!("WRONG seed {seed} step {i}: predicted {:?} actual {o} a={:?} b={:?}", pl, a, b);
                for &l in laws {
                    eprintln!("    {}", lived.summary(l, wb.context));
                }
            }
        }
        let pa = lived.predict_with(&q, TARGET, true).value();
        let pn = naive.predict(&q, TARGET).value();
        if pl == Some(o) && lived_first.is_none() {
            lived_first = Some(i);
        }
        if pn == Some(o) && naive_first.is_none() {
            naive_first = Some(i);
        }
        lived_s.add(pl, o);
        ablate_s.add(pa, o);
        naive_s.add(pn, o);
        lived.observe(ep.clone());
        naive.observe(ep);
    }
    let lf = lived_first.unwrap_or(n_b) as f64;
    let nf = naive_first.unwrap_or(n_b) as f64;
    let cost_ratio = lf / nf.max(1.0);
    let pass = lic_a > 0 && lived_s.wrong == 0 && ablate_s.wrong > 0 && cost_ratio <= 1.2;
    let _ = writeln!(report, "\n[NegativeTransfer] seed {seed}  (licensed in room-A: {lic_a})");
    let _ = writeln!(report, "    room-B stream of {n_b} episodes, answers given before each outcome:");
    let _ = writeln!(report, "    learner                         correct  wrong  abstain  first-correct-at");
    let _ = writeln!(report, "    lived, scoped licences (BITMIND)  {:>6}  {:>5}  {:>7}  {:>6}", lived_s.correct, lived_s.wrong, lived_s.abstain, lf);
    let _ = writeln!(report, "    lived, scope ignored (ATANOR-like){:>6}  {:>5}  {:>7}", ablate_s.correct, ablate_s.wrong, ablate_s.abstain);
    let _ = writeln!(report, "    naive (no prior experience)       {:>6}  {:>5}  {:>7}  {:>6}", naive_s.correct, naive_s.wrong, naive_s.abstain, nf);
    let _ = writeln!(report, "    learning-cost ratio lived/naive = {cost_ratio:.2}");
    let _ = writeln!(report, "    criterion: scoped lived agent never confidently wrong, ablation is, cost ratio <= 1.2 => {}", if pass { "PASS" } else { "FAIL" });
    json.push(format!(
        "{{\"condition\":\"NegativeTransfer\",\"seed\":{seed},\"lived\":{},\"ablation\":{},\"naive\":{},\"cost_ratio\":{cost_ratio:.3},\"pass\":{pass}}}",
        lived_s.json(),
        ablate_s.json(),
        naive_s.json()
    ));
    pass
}

fn main() {
    let started = std::time::Instant::now();
    let seeds: Vec<u64> = std::env::args().nth(1).map(|s| s.split(',').filter_map(|x| x.parse().ok()).collect()).unwrap_or(vec![1, 2, 3]);
    let mut report = String::new();
    let mut json = Vec::new();
    let mut all = true;
    let mut tally: Vec<(String, u32, u32)> = Vec::new();
    let mut note = |name: &str, p: bool, all: &mut bool| {
        *all &= p;
        match tally.iter_mut().find(|t| t.0 == name) {
            Some(t) => {
                t.1 += p as u32;
                t.2 += 1;
            }
            None => tally.push((name.to_string(), p as u32, 1)),
        }
    };
    for &seed in &seeds {
        let _seed_guard = bm_bench::SeedGuard::new(seed);
        let p = standard(R0Kind::Equality, seed, Pool::Test, &[0, 1], &mut report, &mut json);
        note("C1 equality, held-out values", p, &mut all);
        let p = standard(R0Kind::Order, seed, Pool::Test, &[-1, 0, 1], &mut report, &mut json);
        note("C2 order, held-out values", p, &mut all);
        let p = standard(R0Kind::NoRelation, seed, Pool::Test, &[0, 1], &mut report, &mut json);
        note("C3 no-relation control", p, &mut all);
        let p = standard(R0Kind::AbsClass, seed, Pool::Mixed, &[0, 1], &mut report, &mut json);
        note("C4 absolute-class control", p, &mut all);
        let p = confound(seed, &mut report, &mut json);
        note("C5 observation-only confound", p, &mut all);
        let p = negative_transfer(seed, &mut report, &mut json);
        note("C6 negative-transfer guard", p, &mut all);
    }
    println!("{report}");
    println!("==== E-R0 summary over seeds {seeds:?}");
    for (n, p, t) in &tally {
        println!("  {n:<34} {p}/{t} pass");
    }
    println!("  OVERALL: {}", if all { "PASS" } else { "FAIL" });
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../experiments/results/r0.json");
    std::fs::write(&path, format!("{{\"overall_pass\":{all},\"runs\":[\n{}\n]}}", json.join(",\n"))).expect("write");
    println!("saved {}", path.display());
    println!("{}", bm_bench::resources_line(started));
}
