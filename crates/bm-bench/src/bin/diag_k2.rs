//! v0.3 K2′ diagnosis (PREREG-v0.3 section 2): attribute retained memory by law lifecycle,
//! condition type, channel origin and cardinality, and target outcome cardinality, at checkpoints.
//! Modes: `h5 <seed>` (the Phase H H5 world and agent setup, 20,000 steps) and `os <seed>` (the
//! real-OS hard-link world of C4b, 20,000 probes). Read-only; changes nothing in the learner.

use bm_memory::*;
use bm_relation::RelationEngine;
use bm_worlds::heldout2::{self as h2, Mechanism, ObjWorld};
use bm_worlds::os::{self as osw, FsWorld};

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

fn report(tag: &str, step: u32, a: &Agent) {
    let ws = bm_bench::working_set_mb().unwrap_or(-1.0);
    let peak = bm_bench::peak_working_set_mb().unwrap_or(-1.0);
    let rows = a.rel.k2_breakdown();
    let law_bytes: u64 = rows.iter().filter(|r| r.0.starts_with("status ")).map(|r| r.2).sum();
    println!("== {tag} step {step}: working set {ws:.0} MB (peak {peak:.0} MB); laws {} ({} MB attributed to laws)", a.rel.laws.len(), law_bytes / 1048576);
    for (k, n, b) in rows.iter().filter(|r| r.2 >= 1048576 || r.0.starts_with("status ")).take(40) {
        println!("   {k:<52} {n:>9} laws/items {:>7} MB", b / 1048576);
    }
}

fn h5(seed: u64) {
    let n = 20;
    let mut w = ObjWorld::new(Mechanism::Fit, seed, n, (1, 12), 0, "h5");
    let mut rel = RelationEngine::new(seed ^ 0x48);
    rel.identity_channel = Some(INST_CH);
    bm_bench::declare(&mut rel, h2::ORDINAL);
    let mut a = Agent { g: Grounder::new(seed), rel };
    let mut held: Vec<(usize, usize)> = Vec::new();
    for i in 0..n {
        for j in 0..n {
            if i != j && w.rng().below(5) == 0 {
                held.push((i, j));
            }
        }
    }
    let checkpoints = [1000u32, 5000, 10000, 20000];
    let mut step = 0u32;
    while step < 20000 {
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
        if checkpoints.contains(&step) {
            report(&format!("H5 seed {seed}"), step, &a);
        }
    }
}

fn os(seed: u64) {
    let mut w = FsWorld::new(seed, 12, 7, "k2diag").expect("sandbox");
    let mut rel = RelationEngine::new(seed ^ 0xC0);
    rel.identity_channel = Some(INST_CH);
    bm_bench::declare(&mut rel, osw::ORDINAL);
    let mut a = Agent { g: Grounder::new(seed), rel };
    let mut rng = hdc_core::Rng::new(seed ^ 0x05);
    let n = w.files.len();
    let checkpoints = [1000u32, 5000, 10000, 20000];
    for step in 1..=20000u32 {
        let p = rng.sample_distinct(n, 2);
        let Ok((ev, _)) = w.step(Some((osw::PROBE, vec![p[0], p[1]]))) else { break };
        a.feed(ev);
        if checkpoints.contains(&step) {
            report(&format!("real-OS seed {seed}"), step, &a);
        }
    }
}

fn main() {
    let started = std::time::Instant::now();
    let args: Vec<String> = std::env::args().collect();
    let mode = args.get(1).map(|s| s.as_str()).unwrap_or("h5");
    let seed: u64 = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(201);
    match mode {
        "os" => os(seed),
        _ => h5(seed),
    }
    println!("{}", bm_bench::resources_line(started));
}
