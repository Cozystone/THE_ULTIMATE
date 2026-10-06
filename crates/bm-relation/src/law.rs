//! RelationLaw and its licensing lifecycle (ARCHITECTURE 2.4).

use crate::features::{FeatureKind, H};
use hdc_core::fixed::{log2_q16, Q};
use hdc_core::Evidence;
use std::collections::HashSet;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Status {
    Candidate,
    Contested,
    Provisional,
    Licensed,
    Restricted,
    Revoked,
    Split,
}

impl Status {
    pub fn name(self) -> &'static str {
        match self {
            Status::Candidate => "CANDIDATE",
            Status::Contested => "CONTESTED",
            Status::Provisional => "PROVISIONAL",
            Status::Licensed => "LICENSED",
            Status::Restricted => "RESTRICTED",
            Status::Revoked => "REVOKED",
            Status::Split => "SPLIT",
        }
    }
    pub fn terminal(self) -> bool {
        matches!(self, Status::Revoked | Status::Split)
    }
}

/// All licensing thresholds in one place (D007). Changes are logged in DECISIONS.md.
#[derive(Clone, Debug)]
pub struct LicensePolicy {
    /// Gate 1: distinct role-filler combinations supporting the majority outcome.
    pub min_independent: u32,
    /// Gate 2: supporting episodes that were the agent's own interventions.
    pub min_interventions: u32,
    /// Gate 3: pre-registered successes on held-out compositional cases.
    pub min_transfer_ok: u32,
    /// Gate 4: counterexample rate above num/den (with enough data) revokes or splits.
    pub revoke_rate_num: u32,
    pub revoke_rate_den: u32,
    pub min_total_for_revoke: u32,
    /// Gate 5: bits a law must save beyond log2(#candidates) (D012).
    pub utility_margin_q16: i64,
    /// D013: consistent interventions needed to extend a licence to a new context.
    pub min_scope_support: u32,
    /// D014: hidden-condition search.
    pub refine_min_total: u32,
    pub refine_top: usize,
    pub max_children: usize,
    /// Hidden-condition search during wake (false = only during sleep consolidation).
    pub online_refine: bool,
    /// Sensor-error tolerance: counterexample rate at or below num/den may be attributed to
    /// observation error instead of falsifying the law (0 = deterministic worlds).
    pub noise_tol_num: u32,
    pub noise_tol_den: u32,
}

impl Default for LicensePolicy {
    fn default() -> Self {
        LicensePolicy {
            min_independent: 5,
            min_interventions: 3,
            min_transfer_ok: 3,
            revoke_rate_num: 1,
            revoke_rate_den: 10,
            min_total_for_revoke: 6,
            utility_margin_q16: 4 * Q,
            min_scope_support: 3,
            refine_min_total: 4,
            refine_top: 8,
            max_children: 24,
            online_refine: true,
            noise_tol_num: 0,
            noise_tol_den: 1,
        }
    }
}

const EP_CAP: usize = 4096;
const TRIAL_CAP: usize = 256;

#[derive(Clone, Debug)]
pub struct OutcomeBin {
    pub val: i64,
    pub count: u32,
    pub interventions: u32,
    /// Independence keys (episode signatures).
    pub signatures: HashSet<u64>,
    /// Episode ids (lineage). Capped; `count` stays exact.
    pub episodes: Vec<u64>,
}

#[derive(Clone, Debug)]
pub struct TransferTrial {
    pub episode: u64,
    pub predicted: i64,
    pub actual: i64,
}

#[derive(Clone, Debug, Default)]
pub struct TransferRecord {
    pub ok: u32,
    pub fail: u32,
    pub trials: Vec<TransferTrial>,
}

impl TransferRecord {
    pub fn record(&mut self, episode: u64, predicted: i64, actual: i64) {
        if predicted == actual {
            self.ok += 1;
        } else {
            self.fail += 1;
        }
        if self.trials.len() < TRIAL_CAP {
            self.trials.push(TransferTrial { episode, predicted, actual });
        }
    }
}

/// Evidence of one law inside one context.
#[derive(Clone, Debug)]
pub struct CtxEvidence {
    pub context: u64,
    pub bins: Vec<OutcomeBin>,
    pub fillers_seen: HashSet<u64>,
    /// Prequential bits saved versus the best competitor, all live episodes.
    pub utility_q16: i64,
    /// Same, interventions only (gate 2).
    pub utility_int_q16: i64,
    pub transfer: TransferRecord,
    /// Predictions made here by borrowing a licence from another context (scope-extension
    /// trials, D013). Kept apart from `transfer` so a borrowed failure never blocks this
    /// context's own licensing.
    pub scope_trials: TransferRecord,
    pub status: Status,
    pub history: Vec<(u64, Status)>,
    pub first_t: u64,
    pub last_t: u64,
    pub refined_at: u32,
}

impl CtxEvidence {
    pub fn new(context: u64, t: u64) -> Self {
        CtxEvidence {
            context,
            bins: Vec::new(),
            fillers_seen: HashSet::new(),
            utility_q16: 0,
            utility_int_q16: 0,
            transfer: TransferRecord::default(),
            scope_trials: TransferRecord::default(),
            status: Status::Candidate,
            history: vec![(t, Status::Candidate)],
            first_t: t,
            last_t: t,
            refined_at: 0,
        }
    }

    pub fn total(&self) -> u32 {
        self.bins.iter().map(|b| b.count).sum()
    }

    /// Majority outcome and its count (ties: first bin wins).
    pub fn majority(&self) -> Option<(i64, u32)> {
        let mut best: Option<(i64, u32)> = None;
        for b in &self.bins {
            if best.map(|x| b.count > x.1).unwrap_or(true) {
                best = Some((b.val, b.count));
            }
        }
        best
    }

    fn majority_bin(&self) -> Option<&OutcomeBin> {
        let (v, _) = self.majority()?;
        self.bins.iter().find(|b| b.val == v)
    }

    pub fn counters(&self) -> u32 {
        self.total() - self.majority().map(|m| m.1).unwrap_or(0)
    }

    pub fn independent(&self) -> u32 {
        self.majority_bin().map(|b| b.signatures.len() as u32).unwrap_or(0)
    }

    pub fn interventions(&self) -> u32 {
        self.majority_bin().map(|b| b.interventions).unwrap_or(0)
    }

    pub fn count_of(&self, v: i64) -> u32 {
        self.bins.iter().find(|b| b.val == v).map(|b| b.count).unwrap_or(0)
    }

    /// Prequential code length (Q16 bits) of `actual` under Laplace smoothing over `alphabet`.
    pub fn loss_q16(&self, actual: i64, alphabet: u64) -> i64 {
        log2_q16(self.total() as u64 + alphabet) - log2_q16(self.count_of(actual) as u64 + 1)
    }

    pub fn add(&mut self, val: i64, episode: u64, sig: u64, intervention: bool, fps: &[u64], t: u64) {
        let pos = match self.bins.iter().position(|b| b.val == val) {
            Some(p) => p,
            None => {
                self.bins.push(OutcomeBin {
                    val,
                    count: 0,
                    interventions: 0,
                    signatures: HashSet::new(),
                    episodes: Vec::new(),
                });
                self.bins.len() - 1
            }
        };
        let b = &mut self.bins[pos];
        b.count += 1;
        if intervention {
            b.interventions += 1;
        }
        b.signatures.insert(sig);
        if b.episodes.len() < EP_CAP {
            b.episodes.push(episode);
        }
        for &f in fps {
            self.fillers_seen.insert(f);
        }
        self.last_t = t;
    }

    pub fn supporting_ids(&self) -> Vec<u64> {
        self.majority_bin().map(|b| b.episodes.clone()).unwrap_or_default()
    }

    /// Counterexamples are never discarded (gate 4).
    pub fn counterexample_ids(&self) -> Vec<u64> {
        let m = self.majority().map(|m| m.0);
        self.bins.iter().filter(|b| Some(b.val) != m).flat_map(|b| b.episodes.iter().copied()).collect()
    }

    pub fn all_ids(&self) -> Vec<u64> {
        self.bins.iter().flat_map(|b| b.episodes.iter().copied()).collect()
    }

    pub fn confidence(&self) -> Evidence {
        let maj = self.majority().map(|m| m.1).unwrap_or(0);
        Evidence { support: maj, refute: self.total() - maj, last: self.last_t }
    }

    pub fn set_status(&mut self, s: Status, t: u64) {
        if s != self.status {
            self.status = s;
            self.history.push((t, s));
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Origin {
    /// Generated directly from an experienced feature.
    Generated,
    /// Created by hidden-condition search on an impure parent (D014).
    Refined { parent: usize },
}

#[derive(Clone, Debug)]
pub struct Lineage {
    pub origin: Origin,
    pub created_t: u64,
    pub created_episode: u64,
    pub children: Vec<usize>,
}

#[derive(Clone, Debug)]
pub struct RelationLaw {
    pub id: usize,
    pub target: u32,
    pub action: u16,
    /// Symbolic mirror of the condition, for lineage and the realizer.
    pub condition: Vec<FeatureKind>,
    pub condition_hv: H,
    /// Canonical transform code of the relational part (zero = identity / absolute law).
    pub transformation_hv: H,
    /// target bound with the majority outcome code (refreshed on evaluation).
    pub predicted_effect_hv: H,
    pub ctx: Vec<CtxEvidence>,
    pub lineage: Lineage,
    pub competing: Vec<usize>,
    /// D019: contexts where a refined (conjunction) hypothesis was selected by that context's
    /// own hidden-condition search. Empty for directly generated laws (active everywhere).
    pub active_ctx: Vec<u64>,
    /// Removed from the hypothesis space by sleep compression (kept for lineage).
    pub pruned: bool,
}

impl RelationLaw {
    /// Whether this hypothesis takes part in context `c` (D019).
    pub fn active_in(&self, c: u64) -> bool {
        self.lineage.origin == Origin::Generated
            || self.active_ctx.contains(&c)
            || self.ctx.iter().any(|e| e.status == Status::Licensed)
    }

    pub fn is_base(&self) -> bool {
        self.condition.is_empty()
    }

    pub fn is_relational(&self) -> bool {
        self.condition.iter().any(|f| f.is_relational())
    }

    pub fn ctx(&self, c: u64) -> Option<&CtxEvidence> {
        self.ctx.iter().find(|e| e.context == c)
    }

    pub fn ctx_mut(&mut self, c: u64, t: u64) -> &mut CtxEvidence {
        if let Some(p) = self.ctx.iter().position(|e| e.context == c) {
            return &mut self.ctx[p];
        }
        self.ctx.push(CtxEvidence::new(c, t));
        self.ctx.last_mut().expect("just pushed")
    }

    pub fn status_in(&self, c: u64) -> Status {
        self.ctx(c).map(|e| e.status).unwrap_or(Status::Candidate)
    }

    /// Applicability licence for a new case in context `c`.
    pub fn applicable(&self, c: u64) -> bool {
        self.status_in(c) == Status::Licensed
    }

    pub fn licensed_contexts(&self) -> Vec<u64> {
        self.ctx.iter().filter(|e| e.status == Status::Licensed).map(|e| e.context).collect()
    }

    pub fn excluded_contexts(&self) -> Vec<u64> {
        self.ctx.iter().filter(|e| e.status.terminal()).map(|e| e.context).collect()
    }

    /// Aggregate over contexts. Licensed somewhere and revoked/split elsewhere = RESTRICTED.
    pub fn aggregate_status(&self) -> Status {
        let lic = self.ctx.iter().any(|e| e.status == Status::Licensed);
        let bad = self.ctx.iter().any(|e| e.status.terminal() || e.status == Status::Contested);
        if lic && bad {
            return Status::Restricted;
        }
        if lic {
            return Status::Licensed;
        }
        let order = [
            Status::Provisional,
            Status::Contested,
            Status::Candidate,
            Status::Split,
            Status::Revoked,
        ];
        for s in order {
            if self.ctx.iter().any(|e| e.status == s) {
                return s;
            }
        }
        Status::Candidate
    }

    pub fn describe(&self) -> String {
        let cond = if self.condition.is_empty() {
            "true".to_string()
        } else {
            self.condition.iter().map(|f| f.describe()).collect::<Vec<_>>().join(" & ")
        };
        format!("act{} : {} => t{}", self.action, cond, self.target)
    }
}
