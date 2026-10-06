//! E-E: Phase E gates (self-model and sleep consolidation).
//! E1 consolidation improves later prediction and transfer (paired against a no-sleep twin),
//! E2 stated uncertainty is calibrated against actual error, E3 the agent chooses verification
//! actions for hypotheses generated in sleep without being told to.
//! Writes experiments/results/phase_e.json. Floats only for reporting.

use bm_agent::*;
use bm_memory::*;
use bm_relation::{Answer, Episode, LicensePolicy, RelationEngine};
use bm_worlds::devices::{self as dv, DeviceWorld, Scenario};
use bm_worlds::ground::{self as gw, GroundWorld, Setup};
use hdc_core::fixed::Q;
use std::fmt::Write as _;

struct Out {
    report: String,
    json: Vec<String>,
    all: bool,
}

impl Out {
    fn gate(&mut self, name: &str, pass: bool, detail: String) {
        self.all &= pass;
        let _ = writeln!(self.report, "  {name:<28} {}  {detail}", if pass { "PASS" } else { "FAIL" });
        self.json.push(format!("{{\"gate\":\"{name}\",\"pass\":{pass},\"detail\":{:?}}}", detail));
    }
}

/// Wake-only learner: counting and licensing online, structure search only in sleep.
fn wake_policy() -> LicensePolicy {
    LicensePolicy { online_refine: false, ..Default::default() }
}

fn scene(g: &mut Grounder, ev: Event, learn: bool) -> Option<(Grounded, Episode)> {
    let gr = if learn { g.observe(ev)? } else { g.ground(&ev) };
    let ep = to_episode_scene(&gr)?;
    Some((gr, ep))
}

fn target_for(gr: &Grounded, truth: &[(u16, usize)], dev: usize) -> Option<u32> {
    let slot = truth.iter().find(|x| x.1 == dev)?.0;
    let r = scene_role_order(gr)?.iter().position(|&s| s == slot)?;
    Some(target_id(r, dv::ON) + CHANGE)
}

// ------------------------------------------------------------ E1 consolidation (door + links)
fn e1_door(seed: u64) -> (u32, u32, u32, u32, String) {
    let mut w = DeviceWorld::new(Scenario::Door, seed, "door-e1");
    let mut g = Grounder::new(seed);
    let mut rel = RelationEngine::with_policy(seed ^ 0xE1, wake_policy());
    w.use_any = true;
    for _ in 0..600 {
        let (act, args) = w.random_action();
        let (ev, _) = w.step(act, args);
        if let Some((_, ep)) = scene(&mut g, ev, true) {
            rel.observe(ep);
        }
    }
    w.power_varies = true;
    for _ in 0..1800 {
        let (act, args) = w.random_action();
        let (ev, _) = w.step(act, args);
        if let Some((_, ep)) = scene(&mut g, ev, true) {
            rel.observe(ep);
        }
    }
    // twin without sleep
    let ctx = w.context;
    let mut twin_rel = clone_engine(&rel);
    let st = rel.sleep(ctx, 200);
    let desc = format!("{st:?}");
    // identical future: score USE predictions on the door before each outcome
    let (mut ok_s, mut ok_t, mut n_s, mut n_t) = (0, 0, 0, 0);
    for _ in 0..1600 {
        let (act, args) = w.random_action();
        let before = w.devs[dv::DOOR].on;
        let (ev, truth) = w.step(act, args.clone());
        let Some((gr, ep)) = scene(&mut g, ev, true) else { continue };
        if act == dv::USE {
            let truth_change = (w.devs[dv::DOOR].on != before) as i64;
            if let Some(t) = target_for(&gr, &truth, dv::DOOR) {
                let q = ep.without_outcomes();
                n_s += 1;
                n_t += 1;
                let ps = rel.predict(&q, t).value();
                let pt = twin_rel.predict(&q, t).value();
                ok_s += (ps == Some(truth_change)) as u32;
                ok_t += (pt == Some(truth_change)) as u32;
                if std::env::var("DIAG_E").is_ok() {
                    eprintln!("DIAG_E key={} power={} truth={truth_change} sleep={ps:?} twin={pt:?}", args[0] == dv::KEY, w.devs[dv::POWER].on);
                    if args[0] == dv::KEY && w.devs[dv::POWER].on == 1 && ps.is_none() {
                        for l in rel.debug_matching(&q, t).iter().filter(|x| x.contains(" & ")).take(12) {
                            eprintln!("DIAG_L {l}");
                        }
                    }
                }
            }
        }
        rel.observe(ep.clone());
        twin_rel.observe(ep);
    }
    (ok_s, n_s, ok_t, n_t, desc)
}

fn clone_engine(r: &RelationEngine) -> RelationEngine {
    // engines are rebuilt by replaying the store (a twin with identical evidence)
    let mut t = RelationEngine::with_policy(0, r.policy.clone());
    t.identity_channel = r.identity_channel;
    for id in 0..r.store.len() as u64 {
        t.observe(r.store.get(id).clone());
    }
    t
}

fn e1_links(seed: u64) -> (u32, u32, u32, u32) {
    let mut w = DeviceWorld::new(Scenario::Links, seed, "links-e1");
    let mut g = Grounder::new(seed);
    let mut rel = RelationEngine::with_policy(seed ^ 0xE2, wake_policy());
    rel.identity_channel = Some(INST_CH);
    let n = w.devs.len();
    let mut held = std::collections::HashSet::new();
    for i in 0..n {
        for j in 0..n {
            if i != j && w.rng().below(5) == 0 {
                held.insert((i, j));
            }
        }
    }
    let mut fed = 0;
    while fed < 900 {
        let (act, args) = w.random_action();
        if held.contains(&(args[0], args[1])) {
            continue;
        }
        let (ev, _) = w.step(act, args);
        if let Some((_, ep)) = scene(&mut g, ev, true) {
            rel.observe(ep);
        }
        fed += 1;
    }
    let ctx = w.context;
    let mut twin = clone_engine(&rel);
    rel.sleep(ctx, 200);
    // wake again on non-held pairs so latent laws earn live evidence, twin sees the same
    let mut fed = 0;
    while fed < 600 {
        let (act, args) = w.random_action();
        if held.contains(&(args[0], args[1])) {
            continue;
        }
        let (ev, _) = w.step(act, args);
        if let Some((_, ep)) = scene(&mut g, ev, true) {
            rel.observe(ep.clone());
            twin.observe(ep);
        }
        fed += 1;
    }
    // transfer: never-probed pairs
    let (mut ok_s, mut ok_t, mut n) = (0, 0, 0);
    let mut hv: Vec<_> = held.into_iter().collect();
    hv.sort();
    for (i, j) in hv {
        let before = w.devs[j].on;
        let (ev, truth) = w.step(dv::PROBE, vec![i, j]);
        let t_change = (w.devs[j].on != before) as i64;
        let Some((gr, ep)) = scene(&mut g, ev, false) else { continue };
        let Some(t) = target_for(&gr, &truth, j) else { continue };
        let q = ep.without_outcomes();
        n += 1;
        ok_s += (rel.predict(&q, t).value() == Some(t_change)) as u32;
        ok_t += (twin.predict(&q, t).value() == Some(t_change)) as u32;
    }
    (ok_s, n, ok_t, n)
}

// ------------------------------------------------------------ E2 calibration (noisy ground world)
fn e2(seed: u64) -> (i64, i64, u32, u32, Vec<String>) {
    let setup = Setup { noise_pct: 5, miss_pct: 10, ..Default::default() };
    let mut w = GroundWorld::new(seed, 4, 4, setup, "calib");
    let mut g = Grounder::new(seed);
    let mut rel = RelationEngine::new(seed ^ 0xE3);
    let mut sm = SelfModel::default();
    let mut test: Vec<(i64, bool)> = Vec::new();
    let mut test_cal: Vec<(i64, bool)> = Vec::new();
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
        for &(t, observed) in &ep.outcomes {
            // ground truth of the post state (not the noisy observation)
            let (r, ch) = decode_target(t);
            let Some(act) = &gr.act else { continue };
            let Some(&slot) = act.args.get(r) else { continue };
            let Some(&(_, o)) = truth.iter().find(|x| x.0 == slot) else { continue };
            let actual = w.objs[o].get(ch);
            let _ = observed;
            let pred = match rel.predict(&q, t) {
                Answer::Value { val, confidence_q16, .. } => Some((val, confidence_q16)),
                _ => None,
            };
            if i < n / 2 {
                sm.record(t, pred, actual);
            } else if let Some((v, c)) = pred {
                test.push((c, v == actual));
                test_cal.push((sm.calib.calibrated(c), v == actual));
                sm.record(t, pred, actual);
            } else {
                sm.record(t, None, actual);
            }
        }
        rel.observe(ep);
    }
    let raw = Calibration::ece_q16(&test);
    let cal = Calibration::ece_q16(&test_cal);
    let intro = sm.introspect(&rel, w.context);
    // known targets: answered accuracy
    let (mut k_ok, mut k_n) = (0, 0);
    for (&t, r) in &sm.targets {
        if !intro.unknown.contains(&t) {
            k_ok += r.correct;
            k_n += r.correct + r.wrong;
        }
    }
    let names = |t: u32| {
        let (r, ch) = decode_target(t);
        format!("role{r}.{}", match ch { gw::LIT => "lit", gw::POS => "pos", _ => "?" })
    };
    let mut intro2 = intro.clone();
    intro2.resources.events = g.store.len() as u64;
    intro2.resources.laws = rel.active_hypotheses(w.context) as usize;
    intro2.resources.bytes = g.memory_bytes();
    let text = realize(&intro2, &names);
    (raw, cal, k_ok, k_n, text)
}

// ------------------------------------------------------------ E3 verification actions
fn e3(seed: u64) -> (u32, u32, u32) {
    let mut w = DeviceWorld::new(Scenario::Door, seed, "door-e3");
    let mut g = Grounder::new(seed);
    let mut rel = RelationEngine::with_policy(seed ^ 0xE4, wake_policy());
    for _ in 0..500 {
        let (act, args) = w.random_action();
        let (ev, _) = w.step(act, args);
        if let Some((_, ep)) = scene(&mut g, ev, true) {
            rel.observe(ep);
        }
    }
    w.power_varies = true;
    for _ in 0..400 {
        let (act, args) = w.random_action();
        let (ev, _) = w.step(act, args);
        if let Some((_, ep)) = scene(&mut g, ev, true) {
            rel.observe(ep);
        }
    }
    let ctx = w.context;
    let first_new = rel.laws.len();
    rel.sleep(ctx, 200);
    let sleep_laws: std::collections::HashSet<usize> = (first_new..rel.laws.len()).collect();
    let prefs = Preferences::default();
    // compare: how often does the chosen action test a sleep-generated, unlicensed hypothesis?
    let options: Vec<(u16, Vec<usize>)> = vec![(dv::USE, vec![dv::KEY]), (dv::TOGGLE, vec![dv::POWER]), (dv::TOGGLE, vec![3]), (dv::TOGGLE, vec![4]), (dv::WAIT, vec![0])];
    let tests_hyp = |rel: &mut RelationEngine, ep: &Episode, targets: &[u32]| -> bool {
        targets.iter().any(|&t| rel.matching_ids(ep, t).iter().any(|l| sleep_laws.contains(l) && !rel.laws[*l].applicable(ctx)))
    };
    let (mut active_hits, mut random_hits) = (0, 0);
    let k = 30;
    let mut wr = w.clone();
    let mut rel_r = clone_engine(&rel);
    let mut g_r = Grounder::new(seed);
    // the random twin needs the same perception history
    for id in 0..g.store.len() as u64 {
        g_r.observe(g.store.get(id).clone());
    }
    for step in 0..k {
        // active
        let mut cands = Vec::new();
        for (act, args) in &options {
            let (ev, truth) = w.preview(*act, args.clone());
            let gr = g.ground(&ev);
            let Some(ep) = to_episode_scene(&gr).map(|e| e.without_outcomes()) else { continue };
            let targets: Vec<u32> = (0..5).filter_map(|d| target_for(&gr, &truth, d)).collect();
            cands.push(Candidate {
                kind: if *act == dv::WAIT { OptionKind::Wait } else { OptionKind::Act { action: *act, args: args.clone() } },
                episode: Some(ep),
                targets,
                tier: 1,
                cost_q16: Q / 4,
                reliability_q16: Q,
            });
        }
        let pick = choose(&mut rel, &cands, &prefs).map(|x| x.0).unwrap_or(0);
        if let (Some(ep), t) = (&cands[pick].episode, cands[pick].targets.clone()) {
            if tests_hyp(&mut rel, ep, &t) {
                active_hits += 1;
            }
        }
        let (act, args) = options[pick].clone();
        let (ev, _) = w.step(act, args);
        if let Some((_, ep)) = scene(&mut g, ev, true) {
            rel.observe(ep);
        }
        // random twin
        let (act, args) = options[(step * 7 + seed as usize) % options.len()].clone();
        let (pev, ptruth) = wr.preview(act, args.clone());
        let pgr = g_r.ground(&pev);
        if let Some(ep) = to_episode_scene(&pgr).map(|e| e.without_outcomes()) {
            let targets: Vec<u32> = (0..5).filter_map(|d| target_for(&pgr, &ptruth, d)).collect();
            if tests_hyp(&mut rel_r, &ep, &targets) {
                random_hits += 1;
            }
        }
        let (ev, _) = wr.step(act, args);
        if let Some((_, ep)) = scene(&mut g_r, ev, true) {
            rel_r.observe(ep);
        }
    }
    (active_hits, random_hits, k as u32)
}

fn main() {
    let seeds: Vec<u64> = std::env::args().nth(1).map(|s| s.split(',').filter_map(|x| x.parse().ok()).collect()).unwrap_or(vec![1, 2, 3]);
    let mut out = Out { report: String::new(), json: Vec::new(), all: true };
    for &seed in &seeds {
        let _ = writeln!(out.report, "\n===== seed {seed}");
        let (s_ok, s_n, t_ok, t_n, desc) = e1_door(seed);
        let (ls_ok, ls_n, lt_ok, lt_n) = e1_links(seed);
        let _ = writeln!(out.report, "      sleep: {desc}");
        out.gate(
            "E1 consolidation",
            s_ok * 100 >= s_n * 90 && s_ok >= t_ok + s_n / 10 && ls_ok * 100 >= ls_n * 85 && ls_ok >= lt_ok + ls_n / 10,
            format!(
                "seed {seed}: door USE after sleep {s_ok}/{s_n} vs no-sleep twin {t_ok}/{t_n}; never-probed link pairs after sleep {ls_ok}/{ls_n} vs twin {lt_ok}/{lt_n}"
            ),
        );
        let (raw, cal, k_ok, k_n, text) = e2(seed);
        for line in text.iter().take(9) {
            let _ = writeln!(out.report, "      self-report: {line}");
        }
        out.gate(
            "E2 calibration",
            cal * 100 <= 5 * Q && k_n > 0 && k_ok * 100 >= k_n * 95,
            format!(
                "seed {seed}: ECE raw {:.2}% calibrated {:.2}%; accuracy on targets the self-model calls known {k_ok}/{k_n}",
                raw as f64 * 100.0 / Q as f64,
                cal as f64 * 100.0 / Q as f64
            ),
        );
        let (a_hits, r_hits, k) = e3(seed);
        out.gate(
            "E3 verification actions",
            a_hits * 100 >= k * 60 && a_hits >= 2 * r_hits.max(1),
            format!("seed {seed}: first {k} wake actions testing sleep-generated hypotheses: active {a_hits}, fixed-rotation policy {r_hits}"),
        );
    }
    println!("{}", out.report);
    println!("E-E OVERALL: {}", if out.all { "PASS" } else { "FAIL" });
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../experiments/results/phase_e.json");
    std::fs::write(&path, format!("{{\"overall_pass\":{},\"gates\":[\n{}\n]}}", out.all, out.json.join(",\n"))).expect("write");
}
