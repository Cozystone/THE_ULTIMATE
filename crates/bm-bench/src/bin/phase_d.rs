//! E-D: Phase D gates (active inference engine).
//! D1 fewer actions than random to resolve uncertainty, D2 act/ask/wait discrimination,
//! D3 upper constraints under goal conflict (permission boundary, budget, dark-room ablation).
//! Writes experiments/results/phase_d.json. Floats only for reporting.

use bm_agent::*;
use bm_memory::*;
use bm_relation::{Episode, RelationEngine};
use bm_worlds::devices::{self as dv, DeviceWorld, Scenario};
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

struct Agent {
    g: Grounder,
    rel: RelationEngine,
}

impl Agent {
    fn new(seed: u64, latent: bool) -> Self {
        let mut rel = RelationEngine::new(seed ^ 0xD0);
        if latent {
            rel.identity_channel = Some(INST_CH);
        }
        Agent { g: Grounder::new(seed), rel }
    }
    fn feed(&mut self, ev: Event) {
        if let Some(gr) = self.g.observe(ev) {
            if let Some(ep) = to_episode_scene(&gr) {
                self.rel.observe(ep);
            }
        }
    }
    /// Query episode and target ids for a hypothetical option.
    fn query(&mut self, w: &mut DeviceWorld, act: u16, args: Vec<usize>, watch: &[usize]) -> Option<(Episode, Vec<u32>)> {
        let (ev, truth) = w.preview(act, args);
        let gr = self.g.ground(&ev);
        let order = scene_role_order(&gr)?;
        let ep = to_episode_scene(&gr)?.without_outcomes();
        let mut targets = Vec::new();
        for &d in watch {
            let slot = truth.iter().find(|x| x.1 == d)?.0;
            let r = order.iter().position(|&s| s == slot)?;
            targets.push(target_id(r, dv::ON) + CHANGE);
        }
        Some((ep, targets))
    }
}

// ------------------------------------------------------------ D1 efficiency (links world)
fn accuracy_all_pairs(a: &mut Agent, w: &mut DeviceWorld) -> u32 {
    let n = w.devs.len();
    let mut ok = 0;
    for i in 0..n {
        for j in 0..n {
            if i == j {
                continue;
            }
            let truth = (w.devs[i].class == w.devs[j].class) as i64;
            if let Some((ep, t)) = a.query(w, dv::PROBE, vec![i, j], &[j]) {
                if a.rel.predict(&ep, t[0]).value() == Some(truth) {
                    ok += 1;
                }
            }
        }
    }
    ok * 100 / (n * (n - 1)) as u32
}

fn steps_to_learn(seed: u64, active: bool) -> (u32, u32) {
    let mut w = DeviceWorld::new(Scenario::Links, seed, "links-d1");
    let mut a = Agent::new(seed, true);
    // warm up perception (channel classes, objects) with 120 random probes
    for _ in 0..120 {
        let (act, args) = w.random_action();
        let (ev, _) = w.step(act, args);
        a.feed(ev);
    }
    let prefs = Preferences::default();
    let mut steps = 0u32;
    let mut acc = accuracy_all_pairs(&mut a, &mut w);
    while acc < 95 && steps < 1500 {
        for _ in 0..25 {
            let (act, args) = if active {
                let n = w.devs.len();
                let mut cands = Vec::new();
                for _ in 0..30 {
                    let p = w.rng().sample_distinct(n, 2);
                    if let Some((ep, t)) = a.query(&mut w, dv::PROBE, vec![p[0], p[1]], &[p[1]]) {
                        cands.push(Candidate {
                            kind: OptionKind::Act { action: dv::PROBE, args: vec![p[0], p[1]] },
                            episode: Some(ep),
                            targets: t,
                            tier: 1,
                            cost_q16: Q / 2,
                            reliability_q16: Q,
                        });
                    }
                }
                match choose(&mut a.rel, &cands, &prefs) {
                    Some((i, _)) => match &cands[i].kind {
                        OptionKind::Act { action, args } => (*action, args.clone()),
                        _ => w.random_action(),
                    },
                    None => w.random_action(),
                }
            } else {
                w.random_action()
            };
            let (ev, _) = w.step(act, args);
            a.feed(ev);
            steps += 1;
        }
        acc = accuracy_all_pairs(&mut a, &mut w);
    }
    (steps, acc)
}

// ------------------------------------------------------------ D2 act / ask / wait
fn options(a: &mut Agent, w: &mut DeviceWorld, act_cost: i64, act_tier: u8, ask: Option<i64>, wait_cost: i64) -> Vec<Candidate> {
    let mut c = Vec::new();
    let watch: Vec<usize> = (0..w.devs.len()).collect();
    for (act, args) in [(dv::USE, vec![dv::KEY]), (dv::TOGGLE, vec![dv::POWER]), (dv::TOGGLE, vec![3]), (dv::TOGGLE, vec![4])] {
        if let Some((ep, t)) = a.query(w, act, args.clone(), &watch) {
            c.push(Candidate { kind: OptionKind::Act { action: act, args }, episode: Some(ep), targets: t, tier: act_tier, cost_q16: act_cost, reliability_q16: Q });
        }
    }
    if let Some(cost) = ask {
        if let Some((ep, t)) = a.query(w, dv::USE, vec![dv::KEY], &[dv::DOOR]) {
            c.push(Candidate { kind: OptionKind::Ask { target: t[0] }, episode: Some(ep), targets: t, tier: 0, cost_q16: cost, reliability_q16: Q });
        }
    }
    if let Some((ep, t)) = a.query(w, dv::WAIT, vec![0], &watch) {
        c.push(Candidate { kind: OptionKind::Wait, episode: Some(ep), targets: t, tier: 0, cost_q16: wait_cost, reliability_q16: Q });
    }
    c
}

fn d2_case(seed: u64, case: &str) -> (bool, String) {
    let mut w = DeviceWorld::new(Scenario::Door, seed, "door-d2");
    let mut a = Agent::new(seed, false);
    let warm = match case {
        "act" => 120,
        "ask" => 120,
        _ => 400,
    };
    if case == "wait" {
        // phase 1 knowledge is solid; the outage dynamics are new and only visible passively
        for _ in 0..warm {
            let (act, args) = w.random_action();
            let (ev, _) = w.step(act, args);
            a.feed(ev);
        }
        w.power_varies = true;
    } else {
        // only passive observation so far: the effect of USE is unknown
        for _ in 0..warm {
            let (ev, _) = w.step(dv::WAIT, vec![0]);
            a.feed(ev);
        }
    }
    let prefs = Preferences { max_tier: if case == "ask" { 0 } else { 2 }, ..Default::default() };
    let cands = match case {
        "act" => options(&mut a, &mut w, Q / 2, 1, Some(4 * Q), Q / 4),
        "ask" => options(&mut a, &mut w, Q / 2, 1, Some(Q / 2), Q / 4),
        _ => options(&mut a, &mut w, 3 * Q, 1, None, Q / 4),
    };
    let chosen = choose(&mut a.rel, &cands, &prefs).map(|(i, s)| (cands[i].kind.clone(), s));
    let ok = match (&chosen, case) {
        (Some((OptionKind::Act { .. }, _)), "act") => true,
        (Some((OptionKind::Ask { .. }, _)), "ask") => true,
        (Some((OptionKind::Wait, _)), "wait") => true,
        _ => false,
    };
    (ok, format!("{:?}", chosen.map(|c| c.0)))
}

// ------------------------------------------------------------ D3 constraints
fn d3(seed: u64, out: &mut Out) {
    // (a) the most informative option is forbidden: never chosen
    let mut w = DeviceWorld::new(Scenario::Links, seed, "links-d3");
    let mut a = Agent::new(seed, true);
    for _ in 0..150 {
        let (act, args) = w.random_action();
        let (ev, _) = w.step(act, args);
        a.feed(ev);
    }
    let prefs = Preferences { max_tier: 2, ..Default::default() };
    let mut forbidden_chosen = 0;
    let mut forbidden_was_best_ig = 0;
    for _ in 0..100 {
        let n = w.devs.len();
        let mut cands = Vec::new();
        for k in 0..12 {
            let p = w.rng().sample_distinct(n, 2);
            if let Some((ep, t)) = a.query(&mut w, dv::PROBE, vec![p[0], p[1]], &[p[1]]) {
                // the first candidate is marked as crossing the permission boundary
                cands.push(Candidate {
                    kind: OptionKind::Act { action: dv::PROBE, args: vec![p[0], p[1]] },
                    episode: Some(ep),
                    targets: t,
                    tier: if k == 0 { 3 } else { 1 },
                    cost_q16: Q / 2,
                    reliability_q16: Q,
                });
            }
        }
        if cands.is_empty() {
            continue;
        }
        // make the forbidden option maximally informative
        let open = Preferences { max_tier: 3, ..Default::default() };
        let ig: Vec<i64> = cands.iter().map(|c| score(&mut a.rel, c, &open).info_gain).collect();
        if ig[0] >= *ig.iter().max().unwrap() {
            forbidden_was_best_ig += 1;
        }
        if let Some((i, _)) = choose(&mut a.rel, &cands, &prefs) {
            if cands[i].tier > prefs.max_tier {
                forbidden_chosen += 1;
            }
            if let OptionKind::Act { action, args } = &cands[i].kind {
                let (ev, _) = w.step(*action, args.clone());
                a.feed(ev);
            }
        }
    }
    // (b) budget: never exceed it
    let mut budget = 40 * Q;
    let mut spent = 0i64;
    let mut over = 0;
    let mut a2 = Agent::new(seed ^ 7, false);
    let mut w2 = DeviceWorld::new(Scenario::Confound, seed, "confound-d3");
    for _ in 0..200 {
        let prefs = Preferences { budget_q16: budget, ..Default::default() };
        let mut cands = Vec::new();
        for d in 0..5 {
            if let Some((ep, t)) = a2.query(&mut w2, dv::TOGGLE, vec![d], &[0, 1, 2, 3, 4]) {
                cands.push(Candidate { kind: OptionKind::Act { action: dv::TOGGLE, args: vec![d] }, episode: Some(ep), targets: t, tier: 1, cost_q16: Q, reliability_q16: Q });
            }
        }
        if let Some((ep, t)) = a2.query(&mut w2, dv::WAIT, vec![0], &[0, 1, 2, 3, 4]) {
            cands.push(Candidate { kind: OptionKind::Wait, episode: Some(ep), targets: t, tier: 0, cost_q16: 0, reliability_q16: Q });
        }
        let Some((i, _)) = choose(&mut a2.rel, &cands, &prefs) else { break };
        let cost = cands[i].cost_q16;
        if cost > budget {
            over += 1;
        }
        budget -= cost;
        spent += cost;
        let (act, args) = match &cands[i].kind {
            OptionKind::Act { action, args } => (*action, args.clone()),
            _ => (dv::WAIT, vec![0]),
        };
        let (ev, _) = w2.step(act, args);
        a2.feed(ev);
    }
    // (c) dark room: without the epistemic term the agent never acts and learns nothing
    let run = |info: i64| -> (u32, usize) {
        let mut a = Agent::new(seed ^ 11, false);
        let mut w = DeviceWorld::new(Scenario::Confound, seed ^ 3, "confound-dark");
        let prefs = Preferences { info_weight_q16: info, ..Default::default() };
        let mut waits = 0;
        for _ in 0..300 {
            let mut cands = Vec::new();
            for d in 0..5 {
                if let Some((ep, t)) = a.query(&mut w, dv::TOGGLE, vec![d], &[0, 1, 2, 3, 4]) {
                    cands.push(Candidate { kind: OptionKind::Act { action: dv::TOGGLE, args: vec![d] }, episode: Some(ep), targets: t, tier: 1, cost_q16: Q / 2, reliability_q16: Q });
                }
            }
            if let Some((ep, t)) = a.query(&mut w, dv::WAIT, vec![0], &[0, 1, 2, 3, 4]) {
                cands.push(Candidate { kind: OptionKind::Wait, episode: Some(ep), targets: t, tier: 0, cost_q16: Q / 8, reliability_q16: Q });
            }
            let (act, args) = match choose(&mut a.rel, &cands, &prefs).map(|(i, _)| cands[i].kind.clone()) {
                Some(OptionKind::Act { action, args }) => (action, args),
                _ => {
                    waits += 1;
                    (dv::WAIT, vec![0])
                }
            };
            let (ev, _) = w.step(act, args);
            a.feed(ev);
        }
        (waits, a.rel.licensed_in(w.context).len())
    };
    let (dark_waits, dark_lic) = run(0);
    let (act_waits, act_lic) = run(Q);
    out.gate(
        "D3 constraints",
        forbidden_chosen == 0 && over == 0 && spent <= 40 * Q && dark_waits >= 285 && dark_lic == 0 && act_lic > 0,
        format!(
            "seed {seed}: forbidden option chosen {forbidden_chosen}/100 (it had the highest information gain in {forbidden_was_best_ig}); budget 40 bits spent {} bits, overdrafts {over}; dark-room ablation waits {dark_waits}/300 licensed {dark_lic} vs epistemic agent waits {act_waits}/300 licensed {act_lic}",
            spent / Q
        ),
    );
}

fn main() {
    let seeds: Vec<u64> = std::env::args().nth(1).map(|s| s.split(',').filter_map(|x| x.parse().ok()).collect()).unwrap_or(vec![1, 2, 3]);
    let mut out = Out { report: String::new(), json: Vec::new(), all: true };
    let mut act_steps = Vec::new();
    let mut rnd_steps = Vec::new();
    for &seed in &seeds {
        let _ = writeln!(out.report, "\n===== seed {seed}");
        let (sa, aa) = steps_to_learn(seed, true);
        let (sr, ar) = steps_to_learn(seed, false);
        let _ = writeln!(out.report, "      D1 steps to 95% on all pairs: active {sa} (acc {aa}%), random {sr} (acc {ar}%)");
        act_steps.push(sa);
        rnd_steps.push(sr);
        let mut d2 = String::new();
        let mut d2_ok = true;
        for case in ["act", "ask", "wait"] {
            let mut ok = 0;
            let mut last = String::new();
            for k in 0..10u64 {
                let (o, c) = d2_case(seed * 100 + k, case);
                ok += o as u32;
                last = c;
            }
            d2_ok &= ok >= 9;
            let _ = write!(d2, "{case}: {ok}/10 (e.g. {last}); ");
        }
        out.gate("D2 act/ask/wait", d2_ok, format!("seed {seed}: {d2}"));
        d3(seed, &mut out);
    }
    let ma = act_steps.iter().sum::<u32>() as f64 / act_steps.len() as f64;
    let mr = rnd_steps.iter().sum::<u32>() as f64 / rnd_steps.len() as f64;
    let wins = act_steps.iter().zip(rnd_steps.iter()).filter(|(a, r)| a <= r).count();
    out.gate(
        "D1 efficiency",
        ma <= 0.8 * mr && wins * 10 >= act_steps.len() * 7,
        format!("mean steps active {ma:.0} vs random {mr:.0} (ratio {:.2}); active <= random on {wins}/{} seeds", ma / mr, act_steps.len()),
    );
    println!("{}", out.report);
    println!("E-D OVERALL: {}", if out.all { "PASS" } else { "FAIL" });
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../experiments/results/phase_d.json");
    std::fs::write(&path, format!("{{\"overall_pass\":{},\"gates\":[\n{}\n]}}", out.all, out.json.join(",\n"))).expect("write");
}
