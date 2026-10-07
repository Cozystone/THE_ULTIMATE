//! E-F: Phase F generalization battery. Every number is measured here, nothing is copied from
//! earlier phases. Floats only for reporting.
//! F1 one-shot episodic recall, F2 compositional held-out generalization, F3 causal intervention
//! accuracy (vs an association baseline), F4 counterfactual accuracy, F5 calibration,
//! F6 compression ratio of consequences, F7 transfer speed to a new context, F8 resources
//! (latency, RAM, energy estimate).

use bm_agent::Calibration;
use bm_memory::*;
use bm_relation::{Answer, Kind as EpKind, RelationEngine};
use bm_worlds::devices::{self as dv, DeviceWorld, Scenario};
use bm_worlds::ground::{GroundWorld, Setup};
use bm_worlds::r0::{Pool, R0Kind, R0World, TARGET};
use hdc_core::fixed::Q;
use std::fmt::Write as _;
use std::time::Instant;

struct Out {
    report: String,
    json: Vec<String>,
    all: bool,
}

impl Out {
    fn gate(&mut self, name: &str, pass: bool, detail: String) {
        self.all &= pass;
        let _ = writeln!(self.report, "  {name:<26} {}  {detail}", if pass { "PASS" } else { "FAIL" });
        self.json.push(format!("{{\"gate\":\"{name}\",\"pass\":{pass},\"detail\":{:?}}}", detail));
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
    fn pct(&self) -> f64 {
        self.ok as f64 * 100.0 / self.n().max(1) as f64
    }
    fn s(&self) -> String {
        format!("correct {} wrong {} abstain {} of {} ({:.1}%)", self.ok, self.wrong, self.abst, self.n(), self.pct())
    }
}

// ------------------------------------------------------------ F1 one-shot episodic recall
fn corrupt(e: &Event, miss_pct: u64, rng: &mut hdc_core::Rng) -> Event {
    let mut c = e.clone();
    for sc in [&mut c.pre, &mut c.post] {
        sc.tokens.retain(|_| rng.below(100) >= miss_pct);
    }
    c
}

fn f1(seed: u64) -> (Score, Score) {
    let setup = Setup { noise_pct: 5, miss_pct: 10, ..Default::default() };
    let mut w = GroundWorld::new(seed, 4, 4, setup, "f1");
    let mut g = Grounder::new(seed);
    let mut mem = EventMemory::new(seed ^ 0xF1);
    let mut ids = Vec::new();
    for _ in 0..2000 {
        let (ev, _) = w.step(None);
        let id = g.store.len() as u64;
        if let Some(gr) = g.observe(ev) {
            mem.store(&gr);
            if id >= 150 {
                ids.push(id);
            }
        }
    }
    let nm = g.noise_model();
    let mut rng = hdc_core::Rng::new(seed ^ 0xF11);
    let mut res = [Score::default(), Score::default()];
    for (k, miss) in [30u64, 50].into_iter().enumerate() {
        for q in 0..200 {
            let id = ids[(q * 9) % ids.len()];
            let orig = g.store.get(id).clone();
            let cue = corrupt(&orig, miss, &mut rng);
            let gq = g.ground(&cue);
            let got = mem.restore_calibrated(&gq, &cue, &nm).map(|(h, _)| (h.id == id) as i64);
            res[k].add(got, 1);
        }
    }
    (res[0], res[1])
}

// ------------------------------------------------------------ F2 compositional held-out
fn f2(seed: u64) -> (Score, Score) {
    let mut out = Vec::new();
    for kind in [R0Kind::Equality, R0Kind::Order] {
        let mut w = R0World::new(kind, seed, "f2");
        let mut rel = RelationEngine::new(seed ^ 0xF2);
        bm_bench::declare(&mut rel, bm_worlds::r0::ORDINAL);
        for _ in 0..600 {
            let (a, b, o) = w.intervene(Pool::Train);
            let ep = w.episode(&a, &b, o, EpKind::Intervention);
            rel.observe(ep);
        }
        let mut s = Score::default();
        for _ in 0..300 {
            let (a, b, o) = w.intervene(Pool::Test);
            let q = w.episode(&a, &b, o, EpKind::Intervention).without_outcomes();
            s.add(rel.predict(&q, TARGET).value(), o);
        }
        out.push(s);
    }
    (out[0], out[1])
}

// ------------------------------------------------------------ F3 + F4 causal (confound world)
struct Agent {
    g: Grounder,
    rel: RelationEngine,
}

impl Agent {
    fn feed(&mut self, ev: Event) {
        if let Some(gr) = self.g.observe(ev) {
            if let Some(ep) = to_episode_scene(&gr) {
                self.rel.observe(ep);
            }
        }
    }
}

fn predict_change(a: &mut Agent, ev: &Event, truth: &[(u16, usize)], dev: usize) -> Option<i64> {
    let gr = a.g.ground(ev);
    let slot = truth.iter().find(|x| x.1 == dev)?.0;
    let r = scene_role_order(&gr)?.iter().position(|&s| s == slot)?;
    let ep = to_episode_scene(&gr)?.without_outcomes();
    a.rel.predict(&ep, target_id(r, dv::ON) + CHANGE).value()
}

fn f3_f4(seed: u64) -> (Score, Score, Score) {
    let mut w = DeviceWorld::new(Scenario::Confound, seed, "f3");
    let mut a = Agent { g: Grounder::new(seed), rel: RelationEngine::new(seed ^ 0xF3) };
    // association baseline, learned from passive data only: "x influences y" iff y changed in
    // at least half of the passive steps in which x changed
    let nd = w.devs.len();
    let mut co = vec![vec![0u32; nd]; nd];
    let mut ch = vec![0u32; nd];
    for _ in 0..1500 {
        let (act, args) = w.random_action();
        let before: Vec<i64> = w.devs.iter().map(|d| d.on).collect();
        let (ev, _) = w.step(act, args);
        if act == dv::WAIT {
            let moved: Vec<bool> = (0..nd).map(|i| w.devs[i].on != before[i]).collect();
            for x in 0..nd {
                if moved[x] {
                    ch[x] += 1;
                    for y in 0..nd {
                        co[x][y] += moved[y] as u32;
                    }
                }
            }
        }
        a.feed(ev);
    }
    let assoc = |x: usize, y: usize| ch[x] > 0 && co[x][y] * 2 >= ch[x];
    let (mut causal, mut base, mut cf) = (Score::default(), Score::default(), Score::default());
    for _ in 0..100 {
        for d in 0..w.devs.len() {
            w.devs[d].on = w.rng().below(2) as i64;
        }
        for d in 0..w.devs.len() {
            let mut wq = w.clone();
            let (ev, truth) = wq.step(dv::TOGGLE, vec![d]);
            for tgt in [dv::I, dv::L] {
                let t_change = (wq.devs[tgt].on != w.devs[tgt].on) as i64;
                causal.add(predict_change(&mut a, &ev, &truth, tgt), t_change);
                let b = d == tgt || assoc(d, tgt);
                base.add(Some(b as i64), t_change);
            }
        }
        // F4: something else was toggled; had the switch been toggled instead, would the lamp
        // have changed? (same pre-state, alternative intervention)
        let actual_dev = 1 + w.rng().below(4) as usize;
        let alt = w.simulate(dv::TOGGLE, &[dv::S]);
        let truth_lamp = (alt[dv::L].on != w.devs[dv::L].on) as i64;
        let mut wq = w.clone();
        let (alt_ev, alt_truth) = wq.step(dv::TOGGLE, vec![dv::S]);
        cf.add(predict_change(&mut a, &alt_ev, &alt_truth, dv::L), truth_lamp);
        let (ev, _) = w.step(dv::TOGGLE, vec![actual_dev]);
        a.feed(ev);
    }
    (causal, base, cf)
}

// ------------------------------------------------------------ F5 calibration (noisy world)
fn f5(seed: u64) -> (i64, i64, u32) {
    let setup = Setup { noise_pct: 5, miss_pct: 10, ..Default::default() };
    let mut w = GroundWorld::new(seed, 4, 4, setup, "f5");
    let mut g = Grounder::new(seed);
    let mut rel = RelationEngine::new(seed ^ 0xF5);
    bm_bench::declare(&mut rel, bm_worlds::ground::ORDINAL);
    let mut cal = Calibration::default();
    let (mut raw, mut fixed) = (Vec::new(), Vec::new());
    let n = 4000;
    for i in 0..n {
        let (ev, truth) = w.step(None);
        let Some(gr) = g.observe(ev) else { continue };
        if rel.policy.noise_tol_num == 0 {
            rel.policy.noise_tol_num = g.noise_floor_pct().max(1);
            rel.policy.noise_tol_den = 100;
        }
        let Some(ep) = to_episode(&gr) else { continue };
        let q = ep.without_outcomes();
        for &(t, _) in &ep.outcomes {
            let (r, ch) = decode_target(t);
            let Some(act) = &gr.act else { continue };
            let Some(&slot) = act.args.get(r) else { continue };
            let Some(&(_, o)) = truth.iter().find(|x| x.0 == slot) else { continue };
            let actual = w.objs[o].get(ch);
            if let Answer::Value { val, confidence_q16, .. } = rel.predict(&q, t) {
                let ok = val == actual;
                if i < n / 2 {
                    cal.record(confidence_q16, ok);
                } else {
                    raw.push((confidence_q16, ok));
                    fixed.push((cal.calibrated(confidence_q16), ok));
                }
            }
        }
        rel.observe(ep);
    }
    (Calibration::ece_q16(&raw), Calibration::ece_q16(&fixed), fixed.len() as u32)
}

// ------------------------------------------------------------ F6 compression of consequences
fn f6(seed: u64) -> (f64, f64, f64, usize) {
    let mut w = R0World::new(R0Kind::Equality, seed, "f6");
    let mut rel = RelationEngine::new(seed ^ 0xF6);
    bm_bench::declare(&mut rel, bm_worlds::r0::ORDINAL);
    for _ in 0..1000 {
        let (a, b, o) = w.intervene(Pool::Train);
        let ep = w.episode(&a, &b, o, EpKind::Intervention);
        rel.observe(ep);
    }
    // two-part code on 1000 fresh episodes (outcome alphabet of 2):
    // model = for every licensed law, log2(#hypotheses) bits to name it + 16 bits for its value;
    // data = -log2 P(actual) with P = stated confidence if the prediction is right,
    // 1 - confidence if wrong, 1/2 when abstaining. Raw = 1 bit per outcome.
    let ctx = w.context;
    let n_hyp = rel.active_hypotheses(ctx).max(2) as f64;
    let licensed = rel.laws.iter().filter(|l| l.applicable(ctx)).count();
    let model = licensed as f64 * (n_hyp.log2() + 16.0);
    let mut data = 0.0;
    let n = 1000;
    for _ in 0..n {
        let (a, b, o) = w.intervene(Pool::Train);
        let ep = w.episode(&a, &b, o, EpKind::Intervention);
        let p = match rel.predict(&ep.without_outcomes(), TARGET) {
            Answer::Value { val, confidence_q16, .. } => {
                let c = (confidence_q16 as f64 / Q as f64).clamp(0.5, 0.9999);
                if val == o {
                    c
                } else {
                    1.0 - c
                }
            }
            _ => 0.5,
        };
        data += -p.log2();
        rel.observe(ep);
    }
    let raw = n as f64;
    (raw / (model + data), model, data, licensed)
}

// ------------------------------------------------------------ F7 transfer speed
fn steps_to_competence(rel: &mut RelationEngine, seed: u64, label: &str) -> u32 {
    let mut w = R0World::new(R0Kind::Equality, seed, label);
    let mut probe = R0World::new(R0Kind::Equality, seed, label);
    let probes: Vec<_> = (0..60).map(|_| probe.intervene(Pool::Test)).collect();
    for step in 1..=2000u32 {
        let (a, b, o) = w.intervene(Pool::Train);
        let ep = w.episode(&a, &b, o, EpKind::Intervention);
        rel.observe(ep);
        if step % 5 == 0 {
            let mut ok = 0;
            for (a, b, o) in &probes {
                let q = probe.episode(a, b, *o, EpKind::Intervention).without_outcomes();
                ok += (rel.predict(&q, TARGET).value() == Some(*o)) as u32;
            }
            if ok * 100 >= 60 * 95 {
                return step;
            }
        }
    }
    2001
}

fn f7(seed: u64) -> (u32, u32) {
    let mut experienced = RelationEngine::new(seed ^ 0xF7);
    bm_bench::declare(&mut experienced, bm_worlds::r0::ORDINAL);
    let mut wa = R0World::new(R0Kind::Equality, seed ^ 0xA, "f7-room-A");
    for _ in 0..600 {
        let (a, b, o) = wa.intervene(Pool::Train);
        let ep = wa.episode(&a, &b, o, EpKind::Intervention);
        experienced.observe(ep);
    }
    let t = steps_to_competence(&mut experienced, seed ^ 0xB, "f7-room-B");
    let mut fresh = RelationEngine::new(seed ^ 0xF7);
    bm_bench::declare(&mut fresh, bm_worlds::r0::ORDINAL);
    let s = steps_to_competence(&mut fresh, seed ^ 0xB, "f7-room-B");
    (t, s)
}

// ------------------------------------------------------------ F8 resources
fn f8(seed: u64) -> (f64, f64, f64, f64) {
    let mut w = R0World::new(R0Kind::Equality, seed, "f8");
    let mut rel = RelationEngine::new(seed ^ 0xF8);
    bm_bench::declare(&mut rel, bm_worlds::r0::ORDINAL);
    let mut obs = Vec::new();
    let mut pred = Vec::new();
    for _ in 0..2000 {
        let (a, b, o) = w.intervene(Pool::Train);
        let ep = w.episode(&a, &b, o, EpKind::Intervention);
        let q = ep.without_outcomes();
        let t0 = Instant::now();
        let _ = rel.predict(&q, TARGET);
        pred.push(t0.elapsed().as_secs_f64());
        let t1 = Instant::now();
        rel.observe(ep);
        obs.push(t1.elapsed().as_secs_f64());
    }
    obs.sort_by(|a, b| a.partial_cmp(b).unwrap());
    pred.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let med = |v: &Vec<f64>| v[v.len() / 2] * 1e6;
    let p99 = |v: &Vec<f64>| v[v.len() * 99 / 100] * 1e6;
    (med(&obs), p99(&obs), med(&pred), p99(&pred))
}

fn peak_working_set_mb() -> Option<f64> {
    let pid = std::process::id();
    let out = std::process::Command::new("powershell")
        .args(["-NoProfile", "-Command", &format!("(Get-Process -Id {pid}).PeakWorkingSet64")])
        .output()
        .ok()?;
    String::from_utf8_lossy(&out.stdout).trim().parse::<f64>().ok().map(|b| b / 1048576.0)
}

fn main() {
    let started = std::time::Instant::now();
    let seeds: Vec<u64> = std::env::args().nth(1).map(|s| s.split(',').filter_map(|x| x.parse().ok()).collect()).unwrap_or(vec![1, 2, 3]);
    let mut out = Out { report: String::new(), json: Vec::new(), all: true };
    let t_all = Instant::now();
    let mut lat = Vec::new();
    for &seed in &seeds {
        let _ = writeln!(out.report, "\n===== seed {seed}");
        let (r30, r50) = f1(seed);
        out.gate(
            "F1 one-shot recall",
            r30.ok * 100 >= r30.n() * 95 && r30.wrong * 100 <= r30.n() * 2 && r50.wrong * 100 <= r50.n() * 2,
            format!("seed {seed}: events seen once, cue with 30% tokens missing: {}; 50% missing: {}", r30.s(), r50.s()),
        );
        let (eq, ord) = f2(seed);
        out.gate(
            "F2 compositional held-out",
            eq.ok * 100 >= eq.n() * 95 && ord.ok * 100 >= ord.n() * 95 && eq.wrong * 100 <= eq.n() && ord.wrong * 100 <= ord.n(),
            format!("seed {seed}: never-seen colour values (equality) {}; never-seen weights (order) {}", eq.s(), ord.s()),
        );
        let (causal, base, cf) = f3_f4(seed);
        out.gate(
            "F3 intervention accuracy",
            causal.ok * 100 >= causal.n() * 95 && causal.wrong * 100 <= causal.n() * 2,
            format!("seed {seed}: do(toggle x) -> does I / L change: {}; association baseline {}", causal.s(), base.s()),
        );
        out.gate(
            "F4 counterfactual",
            cf.ok * 100 >= cf.n() * 90 && cf.wrong * 100 <= cf.n() * 5,
            format!("seed {seed}: 'had I toggled the switch instead': {}", cf.s()),
        );
        let (raw, cal, n5) = f5(seed);
        out.gate(
            "F5 calibration",
            cal * 100 <= 5 * Q && n5 >= 500,
            format!("seed {seed}: ECE raw {:.2}% calibrated {:.2}% over {n5} answers", raw as f64 * 100.0 / Q as f64, cal as f64 * 100.0 / Q as f64),
        );
        let (ratio, model, data, lic) = f6(seed);
        out.gate(
            "F6 compression",
            ratio >= 5.0,
            format!("seed {seed}: 1000 outcomes, raw 1000 bits vs model {model:.0} bits ({lic} licensed laws) + data {data:.1} bits: ratio {ratio:.1}x"),
        );
        let (t, s) = f7(seed);
        out.gate(
            "F7 transfer speed",
            t <= 2000 && t * 3 <= s,
            format!("seed {seed}: episodes to 95% on unseen values in a new room: experienced {t}, from scratch {s}"),
        );
        lat.push(f8(seed));
    }
    let n = lat.len() as f64;
    let mean = |f: fn(&(f64, f64, f64, f64)) -> f64| lat.iter().map(f).sum::<f64>() / n;
    let (om, o99, pm, p99) = (mean(|x| x.0), mean(|x| x.1), mean(|x| x.2), mean(|x| x.3));
    let ws = peak_working_set_mb().unwrap_or(-1.0);
    // energy: no power meter attached. Upper-bound estimate: one fully loaded core of a 170 W,
    // 16-core package (10.6 W per core) for the measured time.
    let joule_per_obs = 170.0 / 16.0 * om * 1e-6;
    let _ = writeln!(out.report);
    out.gate(
        "F8 resources",
        om <= 1000.0 && pm <= 1000.0 && ws > 0.0 && ws <= 2048.0,
        format!(
            "observe median {om:.0} us (p99 {o99:.0} us), predict median {pm:.0} us (p99 {p99:.0} us), peak working set of the whole battery {ws:.0} MB, energy estimate <= {:.2} mJ per observation (CPU only, no GPU, no network)",
            joule_per_obs * 1e3
        ),
    );
    println!("{}", out.report);
    println!("E-F OVERALL: {}  (wall {:.0} s)", if out.all { "PASS" } else { "FAIL" }, t_all.elapsed().as_secs_f64());
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../experiments/results/phase_f.json");
    std::fs::write(&path, format!("{{\"overall_pass\":{},\"gates\":[\n{}\n]}}", out.all, out.json.join(",\n"))).expect("write");
    println!("{}", bm_bench::resources_line(started));
}
