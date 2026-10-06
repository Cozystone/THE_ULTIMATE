//! Layer 2 self-model (Phase E): what the agent knows, what it does not, why it believes it,
//! how reliable its predictions are, and what its resources are. Integer-only.
//!
//! Calibration: every prediction the agent commits to is recorded with its stated confidence;
//! the empirical accuracy per confidence decile is the self-model's estimate of its own
//! reliability. `calibrated()` maps a raw confidence to that empirical accuracy (monotone).

use bm_relation::{RelationEngine, Status};
use hdc_core::fixed::Q;
use std::collections::BTreeMap;

pub const BINS: usize = 10;

#[derive(Clone, Debug, Default)]
pub struct Calibration {
    /// (correct, total) per stated-confidence decile.
    pub bins: [(u32, u32); BINS],
}

impl Calibration {
    fn bin(conf_q16: i64) -> usize {
        ((conf_q16.clamp(0, Q - 1) * BINS as i64) / Q) as usize
    }

    pub fn record(&mut self, conf_q16: i64, correct: bool) {
        let b = Self::bin(conf_q16);
        self.bins[b].1 += 1;
        if correct {
            self.bins[b].0 += 1;
        }
    }

    /// Empirical accuracy for a stated confidence: isotonic (pool-adjacent-violators) regression
    /// over bins that have data, Laplace-smoothed; bins without data use the nearest fitted bin
    /// below (or the raw confidence if none).
    pub fn calibrated(&self, conf_q16: i64) -> i64 {
        // blocks of (correct, total, first bin, last bin)
        let mut blocks: Vec<(i64, i64, usize, usize)> = Vec::new();
        for (i, &(c, t)) in self.bins.iter().enumerate() {
            if t == 0 {
                continue;
            }
            blocks.push((c as i64, t as i64, i, i));
            while blocks.len() >= 2 {
                let n = blocks.len();
                let (c1, t1, _, _) = blocks[n - 2];
                let (c2, t2, _, _) = blocks[n - 1];
                // violation: earlier block more accurate than later block
                if c1 * t2 > c2 * t1 {
                    let b2 = blocks.pop().expect("block");
                    let b1 = blocks.last_mut().expect("block");
                    b1.0 += b2.0;
                    b1.1 += b2.1;
                    b1.3 = b2.3;
                } else {
                    break;
                }
            }
        }
        let b = Self::bin(conf_q16);
        let mut val = None;
        for &(c, t, lo, _hi) in &blocks {
            if lo <= b {
                val = Some(((c + 1) * Q) / (t + 2));
            }
        }
        val.unwrap_or(conf_q16)
    }

    /// Expected calibration error (Q16) of a set of (stated confidence, correct) pairs under a
    /// confidence mapping.
    pub fn ece_q16(pairs: &[(i64, bool)]) -> i64 {
        let mut bins = [(0i64, 0i64, 0i64); BINS]; // (sum conf, correct, n)
        for &(c, ok) in pairs {
            let b = Self::bin(c);
            bins[b].0 += c;
            bins[b].1 += ok as i64;
            bins[b].2 += 1;
        }
        let n: i64 = bins.iter().map(|b| b.2).sum::<i64>().max(1);
        let mut e = 0i64;
        for (sc, ok, k) in bins {
            if k == 0 {
                continue;
            }
            let mean_conf = sc / k;
            let acc = ok * Q / k;
            e += (mean_conf - acc).abs() * k;
        }
        e / n
    }
}

#[derive(Clone, Debug, Default)]
pub struct TargetRecord {
    pub correct: u32,
    pub wrong: u32,
    pub abstain: u32,
}

/// A detected memory conflict: two applicable laws predict different values for the same case.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Conflict {
    pub target: u32,
    pub laws: (usize, usize),
    pub episode: u64,
}

#[derive(Clone, Debug, Default)]
pub struct Resources {
    pub events: u64,
    pub laws: usize,
    pub bytes: usize,
    pub cycle_ns: u64,
    pub budget_q16: i64,
}

#[derive(Clone, Debug, Default)]
pub struct SelfModel {
    pub calib: Calibration,
    pub targets: BTreeMap<u32, TargetRecord>,
    pub conflicts: Vec<Conflict>,
    pub resources: Resources,
}

/// The answers to the self-model's six questions (ARCHITECTURE layer 2).
#[derive(Clone, Debug)]
pub struct Introspection {
    /// What do I know? Licensed laws in this context (id, description).
    pub known: Vec<(usize, String)>,
    /// What don't I know? Targets with predictions mostly abstained or wrong.
    pub unknown: Vec<u32>,
    /// Why do I believe it? For each known law: (supporting episodes, interventions, transfer ok).
    pub basis: Vec<(usize, u32, u32, u32)>,
    /// How reliable am I? Per target accuracy when I answered (Q16) and overall calibrated ECE.
    pub reliability: Vec<(u32, i64)>,
    /// What would reduce uncertainty most? (filled by the policy layer)
    pub best_probe: Option<String>,
    /// Resources.
    pub resources: Resources,
}

impl SelfModel {
    /// Record a committed prediction (or abstention) and its outcome.
    pub fn record(&mut self, target: u32, predicted: Option<(i64, i64)>, actual: i64) {
        let r = self.targets.entry(target).or_default();
        match predicted {
            Some((v, conf)) => {
                let ok = v == actual;
                if ok {
                    r.correct += 1;
                } else {
                    r.wrong += 1;
                }
                self.calib.record(conf, ok);
            }
            None => r.abstain += 1,
        }
    }

    pub fn detect_conflicts(&mut self, rel: &mut RelationEngine, ep: &bm_relation::Episode, target: u32) {
        let ex = rel.explain(ep, target);
        for i in 0..ex.len() {
            for j in (i + 1)..ex.len() {
                if ex[i].1 != ex[j].1 {
                    self.conflicts.push(Conflict { target, laws: (ex[i].0, ex[j].0), episode: ep.id });
                }
            }
        }
    }

    pub fn introspect(&self, rel: &RelationEngine, ctx: u64) -> Introspection {
        let known: Vec<(usize, String)> = rel.licensed_in(ctx).into_iter().map(|l| (l, rel.laws[l].describe())).collect();
        let basis = known
            .iter()
            .filter_map(|&(l, _)| {
                let e = rel.laws[l].ctx(ctx)?;
                Some((l, e.independent(), e.interventions(), e.transfer.ok))
            })
            .collect();
        let mut unknown = Vec::new();
        let mut reliability = Vec::new();
        for (&t, r) in &self.targets {
            let answered = r.correct + r.wrong;
            let total = answered + r.abstain;
            if total > 0 && (answered * 2 < total || r.wrong * 10 > answered) {
                unknown.push(t);
            }
            if answered > 0 {
                reliability.push((t, (r.correct as i64 * Q) / answered as i64));
            }
        }
        let _ = Status::Licensed;
        Introspection { known, unknown, basis, reliability, best_probe: None, resources: self.resources.clone() }
    }
}

/// Minimal template realizer: externalise the introspection as plain sentences (language is
/// I/O, not the mind; no fluency is attempted).
pub fn realize(i: &Introspection, names: &dyn Fn(u32) -> String) -> Vec<String> {
    let mut out = Vec::new();
    out.push(format!("I hold {} licensed laws here.", i.known.len()));
    for ((l, d), (_, ind, int, tr)) in i.known.iter().zip(i.basis.iter()).take(5) {
        out.push(format!("law {l}: {d} because of {ind} independent cases, {int} of my own interventions, {tr} successful predictions on new combinations."));
    }
    if i.unknown.is_empty() {
        out.push("I have no target that I mostly fail to predict.".to_string());
    } else {
        let u: Vec<String> = i.unknown.iter().map(|&t| names(t)).collect();
        out.push(format!("I do not know: {}.", u.join(", ")));
    }
    for (t, acc) in i.reliability.iter().take(5) {
        out.push(format!("when I answer about {}, I am right {}% of the time.", names(*t), acc * 100 / Q));
    }
    if let Some(p) = &i.best_probe {
        out.push(format!("the observation that would reduce my uncertainty most: {p}."));
    }
    out.push(format!(
        "resources: {} events, {} hypotheses, {} KB, {} us per cycle.",
        i.resources.events,
        i.resources.laws,
        i.resources.bytes / 1024,
        i.resources.cycle_ns / 1000
    ));
    out
}
