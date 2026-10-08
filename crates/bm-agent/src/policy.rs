//! Active inference policy scoring (Phase D), discrete and integer-only.
//!
//! PolicyScore = Risk + Ambiguity + ComputeCost - InformationGain - PreferenceRecovery
//! (all in Q16 bits). The option with the lowest score is chosen.
//!
//! * Risk: violation of the permission boundary or of a hard preference is infinite (the option is
//!   never chosen, whatever its information gain), plus predicted damage to preferred states.
//! * Ambiguity: outcome entropy that is expected to remain after acting (aleatoric part), taken
//!   from licensed laws when they exist.
//! * InformationGain: expected reduction of predictive entropy of the current best hypothesis plus
//!   the value of testing hypotheses that only lack interventions or transfer trials.
//! * PreferenceRecovery: predicted movement of a preferred target towards its preferred value.
//!
//! Without the epistemic term the agent prefers doing nothing (the "dark room"); the ablation is
//! kept as a test.

use bm_relation::{Episode, RelationEngine};
use hdc_core::fixed::{dirichlet_eig_q16, predictive_entropy_q16, Q};

pub const INF: i64 = i64::MAX / 4;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OptionKind {
    /// Intervene: action with object arguments (world indices).
    Act { action: u16, args: Vec<usize> },
    /// Ask a teacher about a target.
    Ask { target: u32 },
    /// Do nothing and observe passive dynamics.
    Wait,
}

#[derive(Clone, Debug)]
pub struct Candidate {
    pub kind: OptionKind,
    /// The hypothetical query episode for this option (outcomes empty). For Wait it is the
    /// passive "action"; for Ask it is the episode the question is about.
    pub episode: Option<Episode>,
    /// Targets whose outcomes this option would reveal.
    pub targets: Vec<u32>,
    /// Permission tier required by the option (0 = harmless).
    pub tier: u8,
    pub cost_q16: i64,
    /// Fraction (Q16) of reliability: 1.0 for acting/observing, teacher reliability for Ask.
    pub reliability_q16: i64,
}

#[derive(Clone, Debug)]
pub struct Preferences {
    /// Permission boundary: options above this tier have infinite risk.
    pub max_tier: u8,
    /// Remaining compute/energy budget; options costing more than this are infeasible.
    pub budget_q16: i64,
    /// Weight of the epistemic term (Q16). 0 = stability-only ablation ("dark room").
    pub info_weight_q16: i64,
    /// Preferred value of a target (preference recovery).
    pub goal: Option<(u32, i64)>,
    pub goal_weight_q16: i64,
}

impl Default for Preferences {
    fn default() -> Self {
        Preferences { max_tier: 2, budget_q16: INF, info_weight_q16: Q, goal: None, goal_weight_q16: 4 * Q }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Score {
    /// D060: bits of conflict-resolving evidence (one per disagreeing licensed pair the option
    /// would give a new independent shared unit).
    pub conflict_evidence: i64,
    pub risk: i64,
    pub ambiguity: i64,
    pub compute: i64,
    pub info_gain: i64,
    pub pref_recovery: i64,
    pub total: i64,
}

/// P(exception rate <= 10%) for a hypothesis confirmed k times without exception, Q16:
/// 1 - 0.9^(k+1) (Beta posterior with a uniform prior on the exception rate).
fn p_licensable_q16(k: u64) -> i64 {
    let mut pow: i64 = Q * 9 / 10;
    for _ in 0..k.min(400) {
        pow = pow * 9 / 10;
    }
    Q - pow
}

/// D043c: expected information gain about the agent's own licensing question ("is the exception
/// rate below 10%?") from one more intervention on a hypothesis confirmed k times so far.
fn licence_eig_q16(k: u64) -> i64 {
    let h = |p: i64| hdc_core::fixed::entropy_q16(&[p.max(0) as u64, (Q - p).max(0) as u64]);
    let p0 = p_licensable_q16(k);
    let p1 = p_licensable_q16(k + 1);
    // with predictive probability (k+1)/(k+2) the hypothesis is confirmed again; otherwise it
    // gains a counterexample and the question is settled (entropy ~0)
    let s_num = (k + 1) as i64;
    let s_den = (k + 2) as i64;
    (h(p0) - h(p1) * s_num / s_den).max(0)
}

/// Epistemic value of observing `target` for the query `ep`.
fn target_value(rel: &mut RelationEngine, ep: &Episode, target: u32) -> (i64, i64) {
    let u = rel.uncertainty(ep, target);
    if u.licensed {
        // known: nothing to learn, residual ambiguity from the licensed law's own spread
        return (0, predictive_entropy_q16(&u.counts).min(Q / 8));
    }
    // D043/D043b: mutual information with the unknown outcome distribution
    let mut ig = dirichlet_eig_q16(&u.counts);
    // D043c: and with the licensing question, for interventions on a still-consistent hypothesis
    // D043d: counted in independent supports; only a new combination advances a hypothesis
    // the outcome distribution and the licensing question are different unknowns: their
    // information adds (the best licence question this case can advance)
    if ep.kind == bm_relation::Kind::Intervention {
        let best = u.licence_k.iter().map(|&k| licence_eig_q16(k as u64)).max().unwrap_or(0);
        ig += best;
    }
    let amb = 0; // unknown outcomes are epistemic, not aleatoric: counted as information gain
    (ig, amb)
}

/// Score one candidate option.
pub fn score(rel: &mut RelationEngine, c: &Candidate, prefs: &Preferences) -> Score {
    let mut s = Score { compute: c.cost_q16, ..Default::default() };
    if c.tier > prefs.max_tier || c.cost_q16 > prefs.budget_q16 {
        s.risk = INF;
        s.total = INF;
        return s;
    }
    if let Some(ep) = &c.episode {
        let mut ig = 0i64;
        let mut amb = 0i64;
        for &t in &c.targets {
            let (g, a) = target_value(rel, ep, t);
            ig += g;
            amb += a;
        }
        if let OptionKind::Ask { .. } = c.kind {
            // a reliable answer resolves the whole predictive uncertainty of the asked target
            ig = 0;
            for &t in &c.targets {
                let u = rel.uncertainty(ep, t);
                if !u.licensed {
                    ig += predictive_entropy_q16(&u.counts);
                }
            }
        }
        // D060: a live conflict between licensed laws is an open question even though each side
        // is "known"; an intervention on which they disagree, with a relevant binding the
        // conflict has not seen, supplies one independent shared unit (D041a)
        if let (OptionKind::Act { .. }, bm_relation::Kind::Intervention) = (&c.kind, ep.kind) {
            let mut bits = 0i64;
            for &t in &c.targets {
                bits += rel.conflict_probe(ep, t).informative_pairs() as i64 * Q;
            }
            s.conflict_evidence = bits;
            ig += bits;
        }
        s.info_gain = (ig * c.reliability_q16 / Q) * prefs.info_weight_q16 / Q;
        s.ambiguity = amb;
        if let (Some((gt, gv)), OptionKind::Act { .. }) = (prefs.goal, &c.kind) {
            if c.targets.contains(&gt) {
                if let Some(v) = rel.predict(ep, gt).value() {
                    s.pref_recovery = if v == gv { prefs.goal_weight_q16 } else { -prefs.goal_weight_q16 };
                }
            }
        }
    }
    s.total = s.risk + s.ambiguity + s.compute - s.info_gain - s.pref_recovery;
    s
}

/// D060: one evidence-seeking decision and its consequence (provenance record).
#[derive(Clone, Debug)]
pub struct ProbeRecord {
    pub option: usize,
    pub target: u32,
    /// Disagreeing pairs at decision time: (value a, value b, units before, new unit).
    pub predicted: Vec<(i64, i64, u32, bool)>,
    pub unit: u64,
    /// Filled after the outcome: the observed value and the pairs' units after.
    pub observed: Option<i64>,
    pub units_after: Vec<u32>,
}

/// D060: choose the candidate that gives the most live licensed-law conflicts a new independent
/// shared unit (ties: lower cost, then the smaller relevant-binding key, never the list position).
/// None when no candidate is informative: the agent then has no evidence-seeking action and keeps
/// abstaining on the conflicted queries.
pub fn choose_probe(rel: &mut RelationEngine, cands: &[Candidate]) -> Option<ProbeRecord> {
    let mut best: Option<(usize, i64, ProbeRecord)> = None;
    for (i, c) in cands.iter().enumerate() {
        let (OptionKind::Act { .. }, Some(ep)) = (&c.kind, &c.episode) else { continue };
        if ep.kind != bm_relation::Kind::Intervention {
            continue;
        }
        for &t in &c.targets {
            let cp = rel.conflict_probe(ep, t);
            let k = cp.informative_pairs();
            if k == 0 {
                continue;
            }
            let unit = cp.pairs.iter().find(|p| p.new_unit).map(|p| p.unit).unwrap_or(0);
            let rec = ProbeRecord {
                option: i,
                target: t,
                predicted: cp.pairs.iter().map(|p| (p.val_a, p.val_b, p.units_before, p.new_unit)).collect(),
                unit,
                observed: None,
                units_after: Vec::new(),
            };
            let better = match &best {
                None => true,
                Some((bi, bk, br)) => {
                    (k as i64) > *bk || ((k as i64) == *bk && (c.cost_q16 < cands[*bi].cost_q16 || (c.cost_q16 == cands[*bi].cost_q16 && unit < br.unit)))
                }
            };
            if better {
                best = Some((i, k as i64, rec));
            }
        }
    }
    best.map(|b| b.2)
}

/// D060: complete a probe record after the outcome was observed (the episode as stored).
pub fn record_outcome(rel: &mut RelationEngine, rec: &mut ProbeRecord, probe: &bm_relation::Episode, observed: Option<i64>) {
    rec.observed = observed;
    rec.units_after = rel.conflict_probe(probe, rec.target).pairs.iter().map(|p| p.units_before).collect();
}

/// Choose the option with the lowest score (ties: lower cost, then earlier candidate).
pub fn choose(rel: &mut RelationEngine, cands: &[Candidate], prefs: &Preferences) -> Option<(usize, Score)> {
    let mut best: Option<(usize, Score)> = None;
    for (i, c) in cands.iter().enumerate() {
        let sc = score(rel, c, prefs);
        if sc.total >= INF {
            continue;
        }
        let better = match &best {
            None => true,
            Some((bi, b)) => sc.total < b.total || (sc.total == b.total && c.cost_q16 < cands[*bi].cost_q16),
        };
        if better {
            best = Some((i, sc));
        }
    }
    best
}
