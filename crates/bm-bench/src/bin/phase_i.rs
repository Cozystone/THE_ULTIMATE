//! capability-h4 formal held-out I-series (PREREG-capability-h4 section 6), run on the frozen
//! learner `bitmind-capability-h4-v0.1`.
//!
//! Protocol per world and seed:
//! * stage 0: 3,000 actions, the conflict region never touched;
//! * stage 1: region queries (never offered as actions) classified; conflict abstentions saved;
//! * stage 2: from identical copies of the stage-1 state, `agent` (the learner's own action
//!   score), `random` (uniform over the offered candidates) and `wait` (WAIT-only) each act for
//!   300 actions on about 12 offered candidates per step; saved queries are re-judged every 10
//!   actions.
//!
//! Usage: `phase_i <I1|I2|I3|I4> <seeds>`. Floats only for reporting.

use bm_agent::*;
use bm_memory::*;
use bm_relation::{Abstain, Answer, Episode, FeatureKind, RelationEngine};
use bm_worlds::heldout_i::{self as hi, IWorld, Variant};
use hdc_core::fixed::Q;
use std::fmt::Write as _;

const BUDGET: u32 = 300;
const STAGE0: u32 = 3000;

#[derive(Clone)]
struct Agent {
    g: Grounder,
    rel: RelationEngine,
    fed: u32,
}

impl Agent {
    fn new(seed: u64) -> Self {
        let mut rel = RelationEngine::new(seed ^ 0x1C4);
        rel.identity_channel = Some(INST_CH);
        bm_bench::declare(&mut rel, hi::ORDINAL);
        Agent { g: Grounder::new(seed), rel, fed: 0 }
    }
    /// Agent assembly (as CW and Phase B, D020): licensing noise tolerance = the grounder's own
    /// measured noise floor (minimum 1%), refreshed every 100 grounded episodes.
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
    fn query(&mut self, ev: &Event, truth: &[(u16, usize)], obj: usize, ch: u16) -> Option<(Episode, u32)> {
        let gr = self.g.ground(ev);
        let slot = truth.iter().find(|x| x.1 == obj)?.0;
        let r = scene_role_order(&gr)?.iter().position(|&s| s == slot)?;
        let ep = to_episode_scene(&gr)?.without_outcomes();
        Some((ep, target_id(r, ch) + CHANGE))
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum Policy {
    Agent,
    Random,
    Wait,
}

#[derive(Default, Debug, Clone)]
struct Outcome {
    saved: u32,
    s_ok: u32,
    s_active: u32,
    d_ok: u32,
    d_active: u32,
    wrong: u32,
    wrong_unresolved: u32,
    unsettled_answers: u32,
    unresolved: u32,
    leakage: u32,
    region_probes: u32,
    diag_new: u32,
    diag_redundant: u32,
    non_diag: u32,
    steps_informative_offered: u32,
    diag_new_when_offered: u32,
    steps_no_informative: u32,
    steps: u32,
}

impl Outcome {
    fn add(&mut self, o: &Outcome) {
        self.saved += o.saved;
        self.s_ok += o.s_ok;
        self.s_active += o.s_active;
        self.d_ok += o.d_ok;
        self.d_active += o.d_active;
        self.wrong += o.wrong;
        self.wrong_unresolved += o.wrong_unresolved;
        self.unsettled_answers += o.unsettled_answers;
        self.unresolved += o.unresolved;
        self.leakage += o.leakage;
        self.region_probes += o.region_probes;
        self.diag_new += o.diag_new;
        self.diag_redundant += o.diag_redundant;
        self.non_diag += o.non_diag;
        self.steps_informative_offered += o.steps_informative_offered;
        self.diag_new_when_offered += o.diag_new_when_offered;
        self.steps_no_informative += o.steps_no_informative;
        self.steps += o.steps;
    }
    /// PREREG: correctly resolved = S + D-active for the agent; for baselines every correct
    /// resolution (S + D) is counted, which is conservative for the agent-vs-baseline gap.
    fn resolved_ok(&self, pol: Policy) -> u32 {
        if pol == Policy::Agent {
            self.s_ok + self.d_active
        } else {
            self.s_ok + self.d_ok
        }
    }
}

#[derive(Clone)]
struct Setup {
    w: IWorld,
    a: Agent,
    queries: Vec<(usize, usize)>,
    saved: Vec<(Episode, u32, i64)>,
    stage1: String,
}

fn setup(variant: Variant, seed: u64) -> Setup {
    let mut w = IWorld::new(variant, seed);
    let mut a = Agent::new(seed);
    let region = w.region_pairs();
    let queries: Vec<(usize, usize)> = region.iter().copied().enumerate().filter(|(k, _)| k % 2 == 0).map(|x| x.1).collect();
    let n = w.objs.len();
    let mut fed = 0;
    while fed < STAGE0 {
        let r = w.rng().below(100);
        let (act, args) = if r < 80 {
            let p = w.rng().sample_distinct(n, 2);
            if w.in_region(p[0], p[1]) {
                continue;
            }
            (hi::PRESS, p)
        } else if r < 87 {
            (hi::SHAKE, vec![w.rng().below(n as u64) as usize])
        } else if r < 94 {
            (hi::TAP, vec![w.rng().below(n as u64) as usize])
        } else {
            (hi::WAIT, vec![])
        };
        let (ev, _) = w.step(act, args);
        a.feed(ev);
        fed += 1;
    }
    let mut saved = Vec::new();
    let (mut conflict, mut answered_ok, mut answered_wrong, mut other) = (0, 0, 0, 0);
    for &(i, j) in &queries {
        let (ev, truth) = w.preview(hi::PRESS, vec![i, j]);
        let Some((q, t)) = a.query(&ev, &truth, j, hi::LIT) else { other += 1; continue };
        let tv = w.effect(i, j) as i64;
        match a.rel.predict(&q, t) {
            Answer::Abstain(Abstain::Conflict) => {
                conflict += 1;
                saved.push((q, t, tv));
            }
            Answer::Value { val, .. } if val == tv => answered_ok += 1,
            Answer::Value { .. } => answered_wrong += 1,
            Answer::Abstain(_) => other += 1,
        }
    }
    let stage1 = format!(
        "{} objects, region pairs {} (queries {}), region truth lights {}, tolerance {}/{}; stage 1: conflict abstentions {conflict}, answered correct {answered_ok}, answered wrong {answered_wrong}, other abstentions {other}",
        w.objs.len(),
        region.len(),
        queries.len(),
        w.region_lights,
        a.rel.policy.noise_tol_num,
        a.rel.policy.noise_tol_den
    );
    Setup { w, a, queries, saved, stage1 }
}

fn candidate(a: &mut Agent, w: &mut IWorld, act: u16, args: &[usize]) -> Option<Candidate> {
    if act == hi::WAIT {
        let (ev, _) = w.preview(hi::WAIT, vec![]);
        let gr = a.g.ground(&ev);
        let ep = to_episode_scene(&gr).map(|e| e.without_outcomes());
        return Some(Candidate { kind: OptionKind::Wait, episode: ep, targets: vec![], tier: 0, cost_q16: Q / 4, reliability_q16: Q });
    }
    let (obj, ch) = match act {
        hi::PRESS => (args[1], hi::LIT),
        hi::SHAKE => (args[0], hi::GLOW),
        _ => (args[0], hi::LIT),
    };
    let (ev, truth) = w.preview(act, args.to_vec());
    let (ep, t) = a.query(&ev, &truth, obj, ch)?;
    Some(Candidate { kind: OptionKind::Act { action: act, args: args.to_vec() }, episode: Some(ep), targets: vec![t], tier: 1, cost_q16: Q / 4, reliability_q16: Q })
}

fn names_identity(rel: &RelationEngine, laws: &[usize]) -> bool {
    // identity-specific answer: every answering law names a mark or a grounded identity
    !laws.is_empty()
        && laws.iter().all(|&l| rel.laws[l].condition.iter().any(|f| matches!(f, FeatureKind::Abs { ch, .. } if *ch == hi::MARK || *ch == INST_CH)))
}

fn run_policy(seed: u64, base: &Setup, pol: Policy, log: bool) -> Outcome {
    let mut s = base.clone();
    let mut o = Outcome { saved: s.saved.len() as u32, ..Default::default() };
    let mut done = vec![false; s.saved.len()];
    let mut prng = hdc_core::Rng::new(seed ^ 0xBA5E_1);
    let prefs = Preferences::default();
    let mut diag_so_far = false;
    for step in 0..BUDGET {
        if done.iter().all(|d| *d) {
            break;
        }
        o.steps += 1;
        let offers = s.w.offers(&s.queries);
        let (act, args) = match pol {
            Policy::Wait => (hi::WAIT, vec![]),
            Policy::Random => offers[prng.below(offers.len() as u64) as usize].clone(),
            Policy::Agent => {
                let mut cands = Vec::new();
                let mut idx = Vec::new();
                for (k, (act, args)) in offers.iter().enumerate() {
                    if let Some(c) = candidate(&mut s.a, &mut s.w, *act, args) {
                        cands.push(c);
                        idx.push(k);
                    }
                }
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
                if informative {
                    o.steps_informative_offered += 1;
                } else {
                    o.steps_no_informative += 1;
                }
                match choose(&mut s.a.rel, &cands, &prefs).map(|x| x.0) {
                    Some(p) => {
                        match cps[p] {
                            (k, _) if k > 0 => {
                                o.diag_new += 1;
                                diag_so_far = true;
                                if informative {
                                    o.diag_new_when_offered += 1;
                                }
                            }
                            (_, true) => o.diag_redundant += 1,
                            _ => o.non_diag += 1,
                        }
                        if log && cps[p].1 {
                            let (Some(ep), Some(&t)) = (&cands[p].episode, cands[p].targets.first()) else { unreachable!() };
                            let cp = s.a.rel.conflict_probe(ep, t);
                            let (x, y) = (offers[idx[p]].1.first().copied(), offers[idx[p]].1.get(1).copied());
                            let region = matches!((x, y), (Some(x), Some(y)) if offers[idx[p]].0 == hi::PRESS && s.w.in_region(x, y));
                            eprintln!("PROBE seed {seed} step {step} action {:?} region {region} predicted {:?}", offers[idx[p]], cp.pairs.iter().map(|q| (q.val_a, q.val_b, q.units_before, q.new_unit)).collect::<Vec<_>>());
                        }
                        offers[idx[p]].clone()
                    }
                    None => (hi::WAIT, vec![]),
                }
            }
        };
        if act == hi::PRESS && s.w.in_region(args[0], args[1]) {
            o.region_probes += 1;
        }
        let (ev, _) = s.w.step(act, args);
        s.a.feed(ev);
        if (step + 1) % 10 == 0 || step + 1 == BUDGET {
            for (k, (q, t, tv)) in s.saved.iter().enumerate() {
                if done[k] {
                    continue;
                }
                let ce = s.a.rel.conflict_evidence(q, *t);
                let settled = ce.iter().all(|c| c.2 >= bm_relation::engine::MIN_SHARED_INDEPENDENT);
                let ans = s.a.rel.predict(q, *t);
                let Answer::Value { val, laws, .. } = ans else { continue };
                done[k] = true;
                if names_identity(&s.a.rel, &laws) {
                    o.leakage += 1;
                }
                let active = diag_so_far && pol == Policy::Agent;
                if !ce.is_empty() && !settled {
                    o.unsettled_answers += 1;
                    if val != *tv {
                        o.wrong_unresolved += 1;
                    }
                }
                if val != *tv {
                    o.wrong += 1;
                } else if ce.is_empty() {
                    o.d_ok += 1;
                    o.d_active += active as u32;
                } else if settled {
                    o.s_ok += 1;
                    o.s_active += active as u32;
                }
            }
        }
    }
    o.unresolved = done.iter().filter(|d| !**d).count() as u32;
    o
}

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

fn pct(a: u32, b: u32) -> f64 {
    if b == 0 {
        0.0
    } else {
        100.0 * a as f64 / b as f64
    }
}

fn main() {
    let started = std::time::Instant::now();
    let args: Vec<String> = std::env::args().collect();
    let variant = match args.get(1).map(|s| s.as_str()) {
        Some("I2") => Variant::I2,
        Some("I3") => Variant::I3,
        Some("I4") => Variant::I4,
        _ => Variant::I1,
    };
    let seeds: Vec<u64> = args.get(2).map(|s| s.split(',').filter_map(|x| x.parse().ok()).collect()).unwrap_or(vec![601]);
    let log = std::env::var("PROBELOG").is_ok();
    let mut report = String::new();
    let mut tot: Vec<(Policy, Outcome)> = vec![(Policy::Agent, Outcome::default()), (Policy::Random, Outcome::default()), (Policy::Wait, Outcome::default())];
    let _ = writeln!(Flush(&mut report), "I-series {variant:?}, frozen learner bitmind-capability-h4-v0.1, budget {BUDGET}");
    for &seed in &seeds {
        let _guard = bm_bench::SeedGuard::new(seed);
        let base = setup(variant, seed);
        let _ = writeln!(Flush(&mut report), "\n===== seed {seed}: {}", base.stage1);
        for (pol, t) in tot.iter_mut() {
            let t0 = std::time::Instant::now();
            let o = run_policy(seed, &base, *pol, log);
            let _ = writeln!(
                Flush(&mut report),
                "  {:<6} saved {}: settled-correct {} (active {}), dissolved-correct {} (active {}), wrong {} (while unresolved {}), unsettled answers {}, unresolved {}, leakage {}; region probes {}; chosen diagnostic-new {} redundant {} non-diagnostic {}; steps with an informative candidate {} (diagnostic-new taken {}), with none {}; actions {}; {:.1} s",
                format!("{pol:?}"),
                o.saved,
                o.s_ok,
                o.s_active,
                o.d_ok,
                o.d_active,
                o.wrong,
                o.wrong_unresolved,
                o.unsettled_answers,
                o.unresolved,
                o.leakage,
                o.region_probes,
                o.diag_new,
                o.diag_redundant,
                o.non_diag,
                o.steps_informative_offered,
                o.diag_new_when_offered,
                o.steps_no_informative,
                o.steps,
                t0.elapsed().as_secs_f64()
            );
            t.add(&o);
        }
    }
    let _ = writeln!(Flush(&mut report), "\n===== totals {variant:?} (seeds {})", seeds.len());
    for (p, o) in &tot {
        let answers = o.s_ok + o.d_ok + o.wrong;
        let _ = writeln!(
            Flush(&mut report),
            "  {:<6} saved {} | resolved correctly {} ({:.1}%; settled {}, dissolved {}; agent-active {}) | wrong {} ({:.1}% of answers; while unresolved {}) | unsettled answers {} | still abstaining {} ({:.1}%) | leakage {} | diagnostic-new {:.1}% of steps with an informative candidate | no informative candidate in {:.1}% of steps",
            format!("{p:?}"),
            o.saved,
            o.resolved_ok(*p),
            pct(o.resolved_ok(*p), o.saved),
            o.s_ok,
            o.d_ok,
            o.s_active + o.d_active,
            o.wrong,
            pct(o.wrong, answers),
            o.wrong_unresolved,
            o.unsettled_answers,
            o.unresolved,
            pct(o.unresolved, o.saved),
            o.leakage,
            pct(o.diag_new_when_offered, o.steps_informative_offered),
            pct(o.steps_no_informative, o.steps)
        );
    }
    // pre-registered criteria (PREREG-capability-h4 section 6)
    let ag = &tot[0].1;
    let rd = &tot[1].1;
    let wt = &tot[2].1;
    let r_ag = pct(ag.resolved_ok(Policy::Agent), ag.saved);
    let r_rd = pct(rd.resolved_ok(Policy::Random), rd.saved);
    let r_wt = pct(wt.resolved_ok(Policy::Wait), wt.saved);
    let answers = ag.s_ok + ag.d_ok + ag.wrong;
    let mut gates: Vec<(String, bool)> = Vec::new();
    match variant {
        Variant::I1 | Variant::I2 => {
            gates.push((format!("resolved correctly {r_ag:.1}% >= 80%"), r_ag >= 80.0));
            gates.push((format!("wrong while unresolved {} = 0", ag.wrong_unresolved), ag.wrong_unresolved == 0));
            gates.push((format!("wrong {:.1}% of answers <= 2%", pct(ag.wrong, answers)), pct(ag.wrong, answers) <= 2.0));
            gates.push((format!("agent {r_ag:.1}% >= random {r_rd:.1}% + 30 pp"), r_ag >= r_rd + 30.0));
            gates.push((format!("WAIT-only resolves {r_wt:.1}% <= 10%"), r_wt <= 10.0));
            gates.push((format!("leakage {} = 0", ag.leakage), ag.leakage == 0));
            if variant == Variant::I2 {
                let d = pct(ag.diag_new_when_offered, ag.steps_informative_offered);
                gates.push((format!("diagnostic-new {d:.1}% of steps with a diagnostic candidate >= 80%"), d >= 80.0));
            }
        }
        Variant::I3 => {
            gates.push((format!("wrong {} = 0", ag.wrong), ag.wrong == 0));
            let still = pct(ag.unresolved, ag.saved);
            gates.push((format!("still abstaining {still:.1}% >= 95%"), still >= 95.0));
            let none = pct(ag.steps_no_informative, ag.steps);
            gates.push((format!("no informative probe declared in {none:.1}% of steps >= 90%"), none >= 90.0));
        }
        Variant::I4 => {
            gates.push((format!("resolved correctly {r_ag:.1}% >= 80%"), r_ag >= 80.0));
            let correct = ag.s_ok + ag.d_ok;
            let s_share = pct(ag.s_ok, correct);
            gates.push((format!("settlement route {s_share:.1}% of correct resolutions >= 70%"), s_share >= 70.0));
            gates.push((format!("settlement audit (answers under < 3 units) {} = 0", ag.unsettled_answers), ag.unsettled_answers == 0));
            gates.push((format!("agent {r_ag:.1}% >= random {r_rd:.1}% + 30 pp"), r_ag >= r_rd + 30.0));
        }
    }
    let _ = writeln!(Flush(&mut report), "\n===== pre-registered criteria {variant:?}");
    let mut all = true;
    for (g, ok) in &gates {
        all &= *ok;
        let _ = writeln!(Flush(&mut report), "  {} {g}", if *ok { "PASS" } else { "FAIL" });
    }
    let _ = writeln!(Flush(&mut report), "  {variant:?} overall: {}", if all { "PASS" } else { "FAIL" });
    println!("{}", bm_bench::resources_line(started));
}
