//! RelationLaw and its licensing lifecycle (ARCHITECTURE 2.4).

use crate::features::FeatureKind;
use hdc_core::fixed::{log2_q16, Q};
use hdc_core::Evidence;

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
    /// D044: the evidence must bound the exception rate: (excess counterexamples + 3) / independent
    /// supports <= max_exception_num / max_exception_den (rule of three, ~95% upper bound).
    pub max_exception_num: u32,
    pub max_exception_den: u32,
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
            max_exception_num: 1,
            max_exception_den: 10,
            noise_tol_num: 0,
            noise_tol_den: 1,
        }
    }
}

const EP_CAP: usize = 4096;
/// D054 (K2 stage 2): key sets per bin stop growing here (v0.3 amendment 6: 1,024, so that a law
/// materialized by replay keeps its replayed situations decidable).
pub const SET_CAP: usize = 1024;
const TRIAL_CAP: usize = 256;

/// D054c: a bounded set of 64-bit keys as a sorted vector with exact capacity (binary search).
/// Same semantics as the bounded hash set it replaces, at a fraction of the fixed overhead.
#[derive(Clone, Debug, Default)]
pub struct KeySet {
    v: Vec<u64>,
    /// The set reached SET_CAP (or was compacted): membership of unseen keys is unknown.
    pub sat: bool,
}

impl KeySet {
    pub fn len(&self) -> usize {
        self.v.len()
    }
    pub fn is_empty(&self) -> bool {
        self.v.is_empty()
    }
    pub fn contains(&self, k: &u64) -> bool {
        self.v.binary_search(k).is_ok()
    }
    pub fn iter(&self) -> impl Iterator<Item = &u64> {
        self.v.iter()
    }
    /// Insert unless full; a full set stops growing and marks itself saturated.
    pub fn put(&mut self, k: u64) {
        if let Err(i) = self.v.binary_search(&k) {
            if self.v.len() < SET_CAP {
                self.v.reserve_exact(1);
                self.v.insert(i, k);
            } else {
                self.sat = true;
            }
        }
    }
    /// Drop the keys, keep the knowledge that membership is no longer decidable.
    pub fn compact(&mut self) {
        self.v = Vec::new();
        self.sat = true;
    }
    pub fn bytes(&self) -> u64 {
        (self.v.capacity() * 8) as u64
    }
}

#[derive(Clone, Debug)]
pub struct OutcomeBin {
    pub val: i64,
    pub count: u32,
    pub interventions: u32,
    /// Situation keys (episode signatures, bystanders included): robustness of a particular law
    /// across background conditions.
    pub signatures: KeySet,
    /// D050: relevant-binding keys (the objects the law connects): independence of a general law.
    pub bindings: KeySet,
    /// D050a: relevant-situation keys (fillers of the relevant roles, bystanders excluded):
    /// transfer novelty of a general law.
    pub rsits: KeySet,
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
    /// D054: the record was compacted (terminal law); key sets are gone, counts remain.
    pub compacted: bool,
}

impl CtxEvidence {
    pub fn new(context: u64, t: u64) -> Self {
        CtxEvidence {
            context,
            bins: Vec::new(),
            utility_q16: 0,
            utility_int_q16: 0,
            transfer: TransferRecord::default(),
            scope_trials: TransferRecord::default(),
            status: Status::Candidate,
            history: vec![(t, Status::Candidate)],
            first_t: t,
            last_t: t,
            refined_at: 0,
            compacted: false,
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

    /// D050: gate-1 independence. A general law (support spans >= 2 relevant bindings) counts
    /// distinct bindings; a particular law (one binding) counts distinct situations, and is scoped
    /// to its binding (`particular`).
    pub fn independent(&self) -> u32 {
        self.majority_bin()
            .map(|b| if b.bindings.len() >= 2 || b.bindings.sat { b.bindings.len() as u32 } else { b.signatures.len() as u32 })
            .unwrap_or(0)
    }

    /// D050: the single relevant binding of a particular law.
    pub fn particular(&self) -> Option<u64> {
        self.majority_bin().and_then(|b| if b.bindings.len() == 1 && !b.bindings.sat { b.bindings.iter().next().copied() } else { None })
    }

    /// D050/D050a: a held-out case for this law. General law: a situation of its relevant
    /// objects (their fillers, bystanders excluded) it has never been supported by. Particular law:
    /// a new situation of its own binding, background included.
    pub fn new_case(&self, rsit: u64, sig: u64) -> bool {
        // D054: a case is new only if provably unseen; a saturated set proves nothing
        match self.particular() {
            Some(_) => !self.bins.iter().any(|b| b.signatures.sat || b.signatures.contains(&sig)),
            None => !self.bins.iter().any(|b| b.rsits.sat || b.rsits.contains(&rsit)),
        }
    }

    /// D054/D054e: compaction of a terminal (revoked / split) record. Counts, status history,
    /// transfer tallies and the episode lists are kept: hidden-condition search replays exactly
    /// these episodes (a revoked law is what refinement splits), and its counterexamples are
    /// evidence. Only the key sets are freed; membership then counts as unknown (conservative).
    /// The law keeps its index entry, so a recurring hypothesis is revived with its record.
    pub fn compact(&mut self) {
        for b in self.bins.iter_mut() {
            b.signatures.compact();
            b.bindings.compact();
            b.rsits.compact();
        }
        self.transfer.trials = Vec::new();
        self.scope_trials.trials = Vec::new();
        self.compacted = true;
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

    pub fn add(&mut self, val: i64, episode: u64, binding: u64, rsit: u64, sig: u64, intervention: bool, t: u64) {
        let pos = match self.bins.iter().position(|b| b.val == val) {
            Some(p) => p,
            None => {
                self.bins.reserve_exact(1);
                self.bins.push(OutcomeBin {
                    val,
                    count: 0,
                    interventions: 0,
                    signatures: KeySet::default(),
                    bindings: KeySet::default(),
                    rsits: KeySet::default(),
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
        // D054/D054c: bounded key sets; a full set stops growing and marks itself saturated
        b.signatures.put(sig);
        b.bindings.put(binding);
        b.rsits.put(rsit);
        if b.episodes.len() < EP_CAP {
            b.episodes.push(episode);
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
    /// D053: fingerprint of the condition hypervector (the index key). The vector itself, the
    /// relational transform code and the predicted-effect vector are recomputed on demand from
    /// the symbolic condition (`RelationEngine::condition_hv`, `predicted_effect_hv`).
    pub condition_fp: u64,
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

    /// Roles mentioned by the condition (D050).
    pub fn cond_roles(&self) -> Vec<u8> {
        let mut v: Vec<u8> = Vec::new();
        for f in &self.condition {
            match *f {
                FeatureKind::Abs { role, .. } => v.push(role),
                FeatureKind::Same { r1, r2, .. } | FeatureKind::Diff { r1, r2, .. } | FeatureKind::Order { r1, r2, .. } | FeatureKind::Delta { r1, r2, .. } => {
                    v.push(r1);
                    v.push(r2);
                }
            }
        }
        v.sort_unstable();
        v.dedup();
        v
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
