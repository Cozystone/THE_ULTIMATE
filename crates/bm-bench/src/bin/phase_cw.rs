//! Capability-h4 development harness on the conflict world CW (PREREG-capability-h4 sections 5
//! and 6). Development data only. Stage 0: learning with the conflict region never probed. Stage 1:
//! region queries (never offered as actions) are classified; conflict abstentions are saved.
//! Stage 2: from identical states, each policy acts for `BUDGET` steps on offered candidates; saved
//! queries are re-judged every 10 steps and classified U / D / S / W (section 2).
//!
//! Policies: `agent` (the learner's own `bm_agent::choose`), `reference` (harness smoke check:
//! a candidate on which `explain` values disagree, oracle-free), `random`, `wait`, and `repeat`
//! (ablation: the same region pair probed every step; uses world knowledge, smoke only).
//! With SMOKE set, smoke diagnostics are printed. Floats only for reporting.

use bm_agent::*;
use bm_memory::*;
use bm_relation::{Abstain, Answer, Episode, FeatureKind, RelationEngine};
use bm_worlds::conflict_dev::{self as cw, ConflictWorld};
use hdc_core::fixed::Q;
use std::fmt::Write as _;

/// Smoke finding 2 (cw_smoke/SMOKE_NOTES.md): at 300 actions random sampling reaches three region
/// bindings passively; 60 keeps the world a test of choice (the reference resolves in 3-6 probes).
const BUDGET: u32 = 60;
const STAGE0: u32 = 2500;
const OFFER_PROBES: usize = 6;
const NOISE_PCT: u64 = 0;

#[derive(Clone)]
struct Agent {
    g: Grounder,
    rel: RelationEngine,
    fed: u32,
}

impl Agent {
    fn new(seed: u64) -> Self {
        let mut rel = RelationEngine::new(seed ^ 0xC3);
        rel.identity_channel = Some(INST_CH);
        bm_bench::declare(&mut rel, cw::ORDINAL);
        Agent { g: Grounder::new(seed), rel, fed: 0 }
    }
    /// D020 (agent assembly, as Phase B): the licensing noise tolerance is the grounder's own
    /// measured sensor noise floor, refreshed every 100 grounded episodes.
    fn feed(&mut self, ev: Event) {
        if let Some(gr) = self.g.observe(ev) {
            self.fed += 1;
            if self.fed % 100 == 0 {
                self.rel.policy.noise_tol_num = self.g.noise_floor_pct().max(1);
                self.rel.policy.noise_tol_den = 100;
            }
            if let Some(ep) = to_episode_scene(&gr) {
                self.rel.observe(ep);
            }
        }
    }
    /// "Does object `obj`'s ON change?" for a previewed event: (episode without outcomes, target).
    fn query(&mut self, ev: &Event, truth: &[(u16, usize)], obj: usize) -> Option<(Episode, u32)> {
        let gr = self.g.ground(ev);
        let slot = truth.iter().find(|x| x.1 == obj)?.0;
        let r = scene_role_order(&gr)?.iter().position(|&s| s == slot)?;
        let ep = to_episode_scene(&gr)?.without_outcomes();
        Some((ep, target_id(r, cw::ON) + CHANGE))
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum Policy {
    Agent,
    Reference,
    Random,
    Wait,
    Repeat,
}

#[derive(Default, Debug, Clone)]
struct Outcome {
    saved: u32,
    s_ok: u32,
    d_ok: u32,
    d_active: u32,
    s_active: u32,
    wrong: u32,
    unsettled_answers: u32,
    unresolved: u32,
    probes_region: u32,
    distinct_region: u32,
    diagnostic_choices: u32,
    max_units: u32,
    steps_no_informative: u32,
    /// D060 classes of the chosen actions, by the agent's own laws at decision time.
    diag_new: u32,
    diag_redundant: u32,
    non_diag: u32,
    /// Actions taken when the first / last saved query was answered (0 = never).
    first_answer_step: u32,
    last_answer_step: u32,
}

#[derive(Clone)]
struct Setup {
    w: ConflictWorld,
    a: Agent,
    queries: Vec<(usize, usize)>,
    saved: Vec<(Episode, u32, i64)>,
    stage1_conflict: u32,
    stage1_other: Vec<String>,
}

/// Stages 0 and 1 (deterministic per seed and relabel seed).
fn setup(seed: u64, relabel: u64) -> Setup {
    let mut w = ConflictWorld::new(seed, (22, 30), NOISE_PCT, relabel, "cw");
    let mut a = Agent::new(seed);
    let region = w.region_pairs();
    let queries: Vec<(usize, usize)> = region.iter().copied().enumerate().filter(|(k, _)| k % 2 == 0).map(|x| x.1).collect();
    let n = w.objs.len();
    let mut fed = 0;
    while fed < STAGE0 {
        let r = w.rng().below(10);
        let (act, args) = if r < 8 {
            let p = w.rng().sample_distinct(n, 2);
            if w.in_region(p[0], p[1]) {
                continue;
            }
            (cw::PROBE, p)
        } else if r < 9 {
            (cw::TOGGLE, vec![w.rng().below(n as u64) as usize])
        } else {
            (cw::WAIT, vec![])
        };
        let (ev, _) = w.step(act, args);
        a.feed(ev);
        fed += 1;
    }
    let mut saved = Vec::new();
    let mut stage1_conflict = 0;
    let mut stage1_other = Vec::new();
    for &(i, j) in &queries {
        let (ev, truth) = w.preview(cw::PROBE, vec![i, j]);
        let Some((q, t)) = a.query(&ev, &truth, j) else { continue };
        let tv = w.effect(i, j) as i64;
        match a.rel.predict(&q, t) {
            Answer::Abstain(Abstain::Conflict) => {
                stage1_conflict += 1;
                saved.push((q, t, tv));
            }
            Answer::Value { val, .. } => stage1_other.push(format!("answered {} (truth {tv})", val)),
            Answer::Abstain(x) => stage1_other.push(format!("abstain {x:?}")),
        }
    }
    Setup { w, a, queries, saved, stage1_conflict, stage1_other }
}

fn candidate(a: &mut Agent, w: &mut ConflictWorld, act: u16, args: &[usize]) -> Option<Candidate> {
    if act == cw::WAIT {
        let (ev, _) = w.preview(cw::WAIT, vec![]);
        let gr = a.g.ground(&ev);
        let ep = to_episode_scene(&gr).map(|e| e.without_outcomes());
        return Some(Candidate { kind: OptionKind::Wait, episode: ep, targets: vec![], tier: 0, cost_q16: Q / 4, reliability_q16: Q });
    }
    let obj = if act == cw::PROBE { args[1] } else { args[0] };
    let (ev, truth) = w.preview(act, args.to_vec());
    let (ep, t) = a.query(&ev, &truth, obj)?;
    Some(Candidate { kind: OptionKind::Act { action: act, args: args.to_vec() }, episode: Some(ep), targets: vec![t], tier: 1, cost_q16: Q / 4, reliability_q16: Q })
}

/// Do the agent's licensed laws disagree on this candidate (harness view via `explain`)?
fn disagrees(a: &mut Agent, c: &Candidate) -> bool {
    let (Some(ep), Some(&t)) = (&c.episode, c.targets.first()) else { return false };
    let vals: std::collections::BTreeSet<i64> = a.rel.explain(ep, t).into_iter().map(|x| x.1).collect();
    vals.len() > 1
}

fn run_policy(seed: u64, base: &Setup, pol: Policy, smoke: bool) -> (Outcome, Setup) {
    let mut s = base.clone();
    let mut o = Outcome { saved: s.saved.len() as u32, ..Default::default() };
    let mut done = vec![false; s.saved.len()];
    let mut region_seen: std::collections::HashSet<(usize, usize)> = std::collections::HashSet::new();
    let evidence: Vec<(usize, usize)> = s.w.region_pairs().into_iter().filter(|p| !s.queries.contains(p)).collect();
    let mut prng = hdc_core::Rng::new(seed ^ 0xBA5E);
    let prefs = Preferences::default();
    let mut diag_so_far = false;
    for step in 0..BUDGET {
        if done.iter().all(|d| *d) {
            break;
        }
        let offers = s.w.offers(OFFER_PROBES, &s.queries);
        let (act, args) = match pol {
            Policy::Wait => (cw::WAIT, vec![]),
            Policy::Random => offers[prng.below(offers.len() as u64) as usize].clone(),
            Policy::Repeat => (cw::PROBE, vec![evidence[0].0, evidence[0].1]),
            Policy::Agent | Policy::Reference => {
                let mut cands = Vec::new();
                let mut idx = Vec::new();
                for (k, (act, args)) in offers.iter().enumerate() {
                    if let Some(c) = candidate(&mut s.a, &mut s.w, *act, args) {
                        cands.push(c);
                        idx.push(k);
                    }
                }
                let dis: Vec<bool> = cands.iter().map(|c| disagrees(&mut s.a, c)).collect();
                // D060 view: informative pairs and disagreement per candidate (agent's own laws)
                let cps: Vec<(usize, bool)> = cands
                    .iter()
                    .map(|c| match (&c.episode, c.targets.first()) {
                        (Some(ep), Some(&t)) if !matches!(c.kind, OptionKind::Wait) => {
                            let cp = s.a.rel.conflict_probe(ep, t);
                            (cp.informative_pairs(), cp.disagrees())
                        }
                        _ => (0, false),
                    })
                    .collect();
                let informative = cps.iter().any(|x| x.0 > 0);
                if (pol == Policy::Agent && !informative) || (pol == Policy::Reference && !dis.iter().any(|d| *d)) {
                    o.steps_no_informative += 1;
                }
                let pick = if pol == Policy::Reference {
                    dis.iter().position(|d| *d).or_else(|| cands.iter().position(|c| matches!(c.kind, OptionKind::Wait)))
                } else {
                    choose(&mut s.a.rel, &cands, &prefs).map(|x| x.0)
                };
                match pick {
                    Some(p) => {
                        if dis[p] {
                            o.diagnostic_choices += 1;
                            diag_so_far = true;
                        }
                        match cps[p] {
                            (k, _) if k > 0 => o.diag_new += 1,
                            (_, true) => o.diag_redundant += 1,
                            _ => o.non_diag += 1,
                        }
                        if std::env::var("PROBELOG").is_ok() && cps[p].1 {
                            let (Some(ep), Some(&t)) = (&cands[p].episode, cands[p].targets.first()) else { unreachable!() };
                            let cp = s.a.rel.conflict_probe(ep, t);
                            let (pa, pb) = (offers[idx[p]].1.first().copied(), offers[idx[p]].1.get(1).copied());
                            let region = matches!((pa, pb), (Some(x), Some(y)) if s.w.in_region(x, y));
                            let truth = match (pa, pb) { (Some(x), Some(y)) => Some(s.w.effect(x, y) as i64), _ => None };
                            eprintln!("PROBE seed {seed} {pol:?} step {step} action {:?} region {region} predicted {:?} observed(world) {truth:?}", offers[idx[p]], cp.pairs.iter().map(|x| (x.val_a, x.val_b, x.units_before, x.new_unit, x.unit)).collect::<Vec<_>>());
                        }
                        offers[idx[p]].clone()
                    }
                    None => (cw::WAIT, vec![]),
                }
            }
        };
        if act == cw::PROBE && s.w.in_region(args[0], args[1]) {
            o.probes_region += 1;
            region_seen.insert((args[0], args[1]));
        }
        if smoke && step < 3 && pol == Policy::Agent {
            eprintln!("CWSTEP seed {seed} step {step} chose {act} {args:?} region {}", act == cw::PROBE && s.w.in_region(args[0], args[1]));
        }
        let (ev, _) = s.w.step(act, args);
        s.a.feed(ev);
        if smoke && pol == Policy::Repeat && (step + 1) % 5 == 0 && step < 30 {
            for (k, (q, t, _)) in s.saved.clone().into_iter().enumerate() {
                let ce = s.a.rel.conflict_evidence(&q, t);
                if k > 0 {
                    eprintln!("REPEAT seed {seed} step {step} query {k} ce {ce:?}");
                    continue;
                }
                let mut roles: Vec<String> = Vec::new();
                for (l, v) in s.a.rel.explain(&q, t) {
                    roles.push(format!("{v}:{:?}:{}", s.a.rel.laws[l].cond_roles(), s.a.rel.laws[l].describe()));
                }
                eprintln!("REPEAT seed {seed} step {step} ce {ce:?} groups {roles:?}");
            }
        }
        if (step + 1) % 10 == 0 {
            for (k, (q, t, tv)) in s.saved.iter().enumerate() {
                if done[k] {
                    continue;
                }
                let ce = s.a.rel.conflict_evidence(q, *t);
                o.max_units = o.max_units.max(ce.iter().map(|c| c.2).min().unwrap_or(0));
                let settled = ce.iter().all(|c| c.2 >= bm_relation::engine::MIN_SHARED_INDEPENDENT);
                let Some(v) = s.a.rel.predict(q, *t).value() else { continue };
                done[k] = true;
                if o.first_answer_step == 0 {
                    o.first_answer_step = step + 1;
                }
                o.last_answer_step = step + 1;
                let active = diag_so_far && matches!(pol, Policy::Agent | Policy::Reference);
                if v != *tv {
                    o.wrong += 1;
                } else if ce.is_empty() {
                    o.d_ok += 1;
                    o.d_active += active as u32;
                } else if settled {
                    o.s_ok += 1;
                    o.s_active += active as u32;
                }
                if !ce.is_empty() && !settled {
                    o.unsettled_answers += 1;
                }
            }
        }
    }
    o.unresolved = done.iter().filter(|d| !**d).count() as u32;
    o.distinct_region = region_seen.len() as u32;
    (o, s)
}

/// Writes through to stdout immediately (long runs keep partial output) and keeps a copy.
struct Flush<'a>(&'a mut String);

impl std::fmt::Write for Flush<'_> {
    fn write_str(&mut self, x: &str) -> std::fmt::Result {
        use std::io::Write as _;
        print!("{x}");
        let _ = std::io::stdout().flush();
        self.0.push_str(x);
        Ok(())
    }
}

fn main() {
    let started = std::time::Instant::now();
    let seeds: Vec<u64> = std::env::args().nth(1).map(|s| s.split(',').filter_map(|x| x.parse().ok()).collect()).unwrap_or(vec![501]);
    let smoke = std::env::var("SMOKE").is_ok();
    let mut report = String::new();
    let mut tot: Vec<(Policy, Outcome)> = Vec::new();
    for &seed in &seeds {
        let _guard = bm_bench::SeedGuard::new(seed);
        let _ = writeln!(Flush(&mut report), "\n===== seed {seed}");
        // smoke: world facts, stage-1 conflict, both laws' support, relabel invariance
        let base = setup(seed, 0);
        if std::env::var("GROUNDCHK").is_ok() {
            // smoke 5: does one physical pair keep one relevant binding across repeated previews?
            let mut s = base.clone();
            let (a0, b0) = s.w.region_pairs()[1];
            let mut keys: std::collections::BTreeMap<(Option<i64>, Option<i64>), u32> = std::collections::BTreeMap::new();
            for _ in 0..60 {
                let (ev, truth) = s.w.preview(cw::PROBE, vec![a0, b0]);
                let gr = s.a.g.ground(&ev);
                let order = scene_role_order(&gr).unwrap_or_default();
                let Some(ep) = to_episode_scene(&gr) else { continue };
                let role = |o: usize| truth.iter().find(|x| x.1 == o).and_then(|x| order.iter().position(|&sl| sl == x.0));
                let ka = role(a0).and_then(|r| ep.roles.get(r)).and_then(|e| e.binding);
                let kb = role(b0).and_then(|r| ep.roles.get(r)).and_then(|e| e.binding);
                *keys.entry((ka, kb)).or_insert(0) += 1;
            }
            let _ = writeln!(Flush(&mut report), "  grounding check: pair ({a0},{b0}) previewed 60 times -> {} distinct (binding a, binding b): {:?}", keys.len(), keys);
            continue;
        }
        {
            let mut s = base.clone();
            let region = s.w.region_pairs();
            let _ = writeln!(
                Flush(&mut report),
                "  world: {} objects, region pairs {} (queries {}, evidence {}), region truth flips {}, measured noise tolerance {}/{}; stage 1: conflict abstentions {} of {} queries; other {:?}",
                s.w.objs.len(),
                region.len(),
                s.queries.len(),
                region.len() - s.queries.len(),
                s.w.region_flips,
                s.a.rel.policy.noise_tol_num,
                s.a.rel.policy.noise_tol_den,
                s.stage1_conflict,
                s.queries.len(),
                s.stage1_other
            );
            if let Some((q, t, _)) = s.saved.first().cloned() {
                for (l, v) in s.a.rel.explain(&q, t) {
                    let _ = writeln!(Flush(&mut report), "    conflict law (predicts {v}): {}", s.a.rel.summary(l, s.w.context));
                }
                let _ = writeln!(Flush(&mut report), "    conflict evidence at stage 1: {:?}", s.a.rel.conflict_evidence(&q, t));
            }
            if smoke {
                // relabel invariance of stage-1 classes
                let r = setup(seed, seed ^ 0x3E1A);
                let _ = writeln!(Flush(&mut report), "    relabelled stage 1: conflict abstentions {} (original {}), other {:?}", r.stage1_conflict, s.stage1_conflict, r.stage1_other);
                let mut sl: Vec<String> = Vec::new();
                for l in s.a.rel.laws.iter().filter(|l| l.ctx.iter().any(|c| c.status == bm_relation::Status::Licensed)) {
                    if l.condition.iter().any(|f| matches!(f, FeatureKind::Abs { ch, .. } if *ch == cw::MARK || *ch == INST_CH)) {
                        sl.push(s.a.rel.summary(l.id, s.w.context));
                    }
                }
                let _ = writeln!(Flush(&mut report), "    licensed laws naming a mark or identity: {}", sl.len());
            }
        }
        let pols: &[Policy] = if smoke { &[Policy::Agent, Policy::Reference, Policy::Random, Policy::Wait, Policy::Repeat] } else { &[Policy::Agent, Policy::Random, Policy::Wait] };
        for &pol in pols {
            let t0 = std::time::Instant::now();
            let (o, s) = run_policy(seed, &base, pol, smoke);
            let _ = writeln!(
                Flush(&mut report),
                "  {:<9} saved {}: settled-correct {} (active {}), dissolved-correct {} (active {}), wrong {}, unsettled answers {}, unresolved {}; region probes {} ({} distinct pairs), diagnostic choices {}, steps with no informative candidate {}, min units reached {}, answers at steps {}-{}; chosen actions diagnostic-new {} diagnostic-redundant {} non-diagnostic {}; {:.1} s",
                format!("{pol:?}"),
                o.saved,
                o.s_ok,
                o.s_active,
                o.d_ok,
                o.d_active,
                o.wrong,
                o.unsettled_answers,
                o.unresolved,
                o.probes_region,
                o.distinct_region,
                o.diagnostic_choices,
                o.steps_no_informative,
                o.max_units,
                o.first_answer_step,
                o.last_answer_step,
                o.diag_new,
                o.diag_redundant,
                o.non_diag,
                t0.elapsed().as_secs_f64()
            );
            if smoke && pol == Policy::Repeat {
                if let Some((q, t, _)) = s.saved.first() {
                    let mut rel = s.a.rel;
                    let _ = writeln!(Flush(&mut report), "    repeat ablation, conflict evidence after: {:?}", rel.conflict_evidence(q, *t));
                }
            }
            match tot.iter_mut().find(|x| x.0 == pol) {
                Some(x) => {
                    let y = &mut x.1;
                    y.saved += o.saved;
                    y.s_ok += o.s_ok;
                    y.d_ok += o.d_ok;
                    y.s_active += o.s_active;
                    y.d_active += o.d_active;
                    y.wrong += o.wrong;
                    y.unsettled_answers += o.unsettled_answers;
                    y.unresolved += o.unresolved;
                    y.probes_region += o.probes_region;
                    y.diagnostic_choices += o.diagnostic_choices;
                }
                None => tot.push((pol, o)),
            }
        }
    }
    let _ = writeln!(Flush(&mut report), "\n===== totals");
    for (p, o) in &tot {
        let _ = writeln!(Flush(&mut report), "  {:<9} saved {} settled-correct {} dissolved-correct {} wrong {} unsettled answers {} unresolved {} region probes {} diagnostic choices {}", format!("{p:?}"), o.saved, o.s_ok, o.d_ok, o.wrong, o.unsettled_answers, o.unresolved, o.probes_region, o.diagnostic_choices);
    }
    let _ = report;
    println!("{}", bm_bench::resources_line(started));
}
