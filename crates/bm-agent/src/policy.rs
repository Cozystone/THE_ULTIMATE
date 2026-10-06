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
use hdc_core::fixed::{expected_info_gain_q16, predictive_entropy_q16, Q};

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
    pub risk: i64,
    pub ambiguity: i64,
    pub compute: i64,
    pub info_gain: i64,
    pub pref_recovery: i64,
    pub total: i64,
}

/// Epistemic value of observing `target` for the query `ep`.
fn target_value(rel: &mut RelationEngine, ep: &Episode, target: u32) -> (i64, i64) {
    let u = rel.uncertainty(ep, target);
    if u.licensed {
        // known: nothing to learn, residual ambiguity from the licensed law's own spread
        return (0, predictive_entropy_q16(&u.counts).min(Q / 8));
    }
    let ig = expected_info_gain_q16(&u.counts) + (u.near_licence.min(4) as i64) * (Q / 4);
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
