//! RelationEngine: candidate generation, prequential evidence, pre-registered transfer tests,
//! hidden-condition search and the licensing lifecycle.

use crate::episode::{Episode, EpisodeStore, Kind};
use crate::features::{Codebook, Feature, FeatureKind, H};
use crate::law::*;
use hdc_core::fixed::{log2_q16, ratio_q16};
use hdc_core::rng::mix64;
use std::collections::HashMap;

/// Why the engine refused to answer. Silence is preferred to confident wrongness.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Abstain {
    /// No law matching this case is licensed anywhere.
    NoLicensedLaw,
    /// A matching law is licensed, but not in this context (D013).
    OutOfScope,
    /// Licensed laws disagree.
    Conflict,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Answer {
    Value { val: i64, laws: Vec<usize>, confidence_q16: i64 },
    Abstain(Abstain),
}

impl Answer {
    pub fn value(&self) -> Option<i64> {
        match self {
            Answer::Value { val, .. } => Some(*val),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct ObserveReport {
    pub preregistered: u32,
    pub transfer_ok: u32,
    pub transfer_fail: u32,
    pub new_laws: u32,
}

pub struct RelationEngine {
    pub cb: Codebook,
    pub policy: LicensePolicy,
    pub laws: Vec<RelationLaw>,
    index: HashMap<u64, usize>,
    pub store: EpisodeStore,
    /// Feature kinds of each stored episode (replay without re-encoding).
    feat_kinds: Vec<Vec<FeatureKind>>,
    alphabet: HashMap<u32, Vec<i64>>,
    per_target: HashMap<u32, u32>,
    /// D018: laws with evidence per (target, context).
    per_target_ctx: HashMap<(u32, u64), u32>,
    pub tick: u64,
}

fn key(cond_hv: &H, target: u32) -> u64 {
    mix64(cond_hv.fingerprint(), target as u64 ^ 0x7A11)
}

impl RelationEngine {
    pub fn new(seed: u64) -> Self {
        Self::with_policy(seed, LicensePolicy::default())
    }

    pub fn with_policy(seed: u64, policy: LicensePolicy) -> Self {
        RelationEngine {
            cb: Codebook::new(seed),
            policy,
            laws: Vec::new(),
            index: HashMap::new(),
            store: EpisodeStore::new(),
            feat_kinds: Vec::new(),
            alphabet: HashMap::new(),
            per_target: HashMap::new(),
            per_target_ctx: HashMap::new(),
            tick: 0,
        }
    }

    fn cond_hv(&mut self, action: u16, feats: &[&H]) -> H {
        let mut h = self.cb.action(action);
        for f in feats {
            h.bind_assign(f);
        }
        h
    }

    fn transform_code(&mut self, cond: &[FeatureKind]) -> H {
        let mut t = H::zero();
        for f in cond {
            match *f {
                FeatureKind::Abs { .. } | FeatureKind::Same { .. } => {}
                FeatureKind::Diff { .. } => t.bind_assign(&self.cb.sym("rel:diff")),
                FeatureKind::Order { sign, .. } => {
                    t.bind_assign(&self.cb.sym(if sign > 0 { "sign:+" } else { "sign:-" }))
                }
                FeatureKind::Delta { k, .. } => t.bind_assign(&self.cb.sym("deltabase").permute(k)),
            }
        }
        t
    }

    fn lookup(&self, cond_hv: &H, target: u32) -> Option<usize> {
        self.index.get(&key(cond_hv, target)).copied()
    }

    fn create(
        &mut self,
        action: u16,
        target: u32,
        mut cond: Vec<FeatureKind>,
        cond_hv: H,
        origin: Origin,
        episode: u64,
    ) -> usize {
        cond.sort();
        let id = self.laws.len();
        let transformation_hv = self.transform_code(&cond);
        self.laws.push(RelationLaw {
            id,
            target,
            action,
            condition: cond,
            transformation_hv,
            predicted_effect_hv: H::zero(),
            condition_hv: cond_hv.clone(),
            ctx: Vec::new(),
            lineage: Lineage { origin, created_t: self.tick, created_episode: episode, children: Vec::new() },
            competing: Vec::new(),
            active_ctx: Vec::new(),
        });
        self.index.insert(key(&cond_hv, target), id);
        *self.per_target.entry(target).or_insert(0) += 1;
        id
    }

    fn get_or_create(&mut self, action: u16, target: u32, feats: &[&Feature], episode: u64) -> (usize, bool) {
        let hvs: Vec<&H> = feats.iter().map(|f| &f.hv).collect();
        let h = self.cond_hv(action, &hvs);
        if let Some(i) = self.lookup(&h, target) {
            return (i, false);
        }
        let cond = feats.iter().map(|f| f.kind.clone()).collect();
        (self.create(action, target, cond, h, Origin::Generated, episode), true)
    }

    fn alphabet_size(&self, target: u32) -> u64 {
        self.alphabet.get(&target).map(|v| v.len() as u64).unwrap_or(0) + 1
    }

    fn loss(&self, law: usize, ctx: u64, actual: i64, alpha: u64) -> i64 {
        match self.laws[law].ctx(ctx) {
            Some(e) => e.loss_q16(actual, alpha),
            None => log2_q16(alpha),
        }
    }

    /// Value a law would predict in `ctx` and whether it is borrowed: its own majority there,
    /// else (if licensed elsewhere) the majority of a licensed context.
    fn prediction(&self, law: usize, ctx: u64) -> Option<(i64, bool)> {
        let l = &self.laws[law];
        if let Some(e) = l.ctx(ctx) {
            if e.total() > 0 && e.counters() == 0 && !e.status.terminal() {
                return e.majority().map(|m| (m.0, false));
            }
        }
        l.ctx
            .iter()
            .find(|e| e.status == Status::Licensed)
            .and_then(|e| e.majority().map(|m| (m.0, true)))
    }

    /// Majority value of the first context where the law is licensed (excluding `ctx`).
    fn licensed_value_elsewhere(&self, law: usize, ctx: u64) -> Option<i64> {
        self.laws[law]
            .ctx
            .iter()
            .find(|e| e.context != ctx && e.status == Status::Licensed)
            .and_then(|e| e.majority().map(|m| m.0))
    }

    fn transfer_eligible(&self, law: usize, ctx: u64, fps: &[u64]) -> bool {
        let l = &self.laws[law];
        if l.is_base() {
            return false;
        }
        let own = match l.ctx(ctx) {
            Some(e) if e.total() > 0 => {
                !e.status.terminal()
                    && e.counters() == 0
                    && e.independent() >= 3
                    && fps.iter().any(|f| !e.fillers_seen.contains(f))
            }
            _ => false,
        };
        // borrowed licence: test it here until this context has its own verdict
        let borrowed = l.ctx.iter().any(|e| e.context != ctx && e.status == Status::Licensed)
            && l.ctx(ctx).map(|e| e.status != Status::Licensed && !e.status.terminal()).unwrap_or(true);
        own || borrowed
    }

    /// Observe one complete episode. Predictions for transfer tests are fixed before the
    /// outcome is read.
    pub fn observe(&mut self, ep: Episode) -> ObserveReport {
        self.tick += 1;
        let mut rep = ObserveReport::default();
        let feats = self.cb.features(&ep);
        let sig = ep.signature();
        let fps = ep.filler_fps();
        let ctx = ep.context;
        let intervention = ep.kind == Kind::Intervention;
        let outcomes = ep.outcomes.clone();
        let action = ep.action;
        let eid = self.store.append(ep);
        self.feat_kinds.push(feats.iter().map(|f| f.kind.clone()).collect());

        for (target, actual) in outcomes {
            let before = self.laws.len();
            let (base, _) = self.get_or_create(action, target, &[], eid);
            let mut singles = Vec::with_capacity(feats.len());
            for f in &feats {
                singles.push(self.get_or_create(action, target, &[f], eid).0);
            }
            let mut pairs: Vec<(usize, usize, usize)> = Vec::new();
            for i in 0..feats.len() {
                for j in (i + 1)..feats.len() {
                    let h = self.cond_hv(action, &[&feats[i].hv, &feats[j].hv]);
                    if let Some(l) = self.lookup(&h, target) {
                        if self.laws[l].active_in(ctx) {
                            pairs.push((l, singles[i], singles[j]));
                        }
                    }
                }
            }
            rep.new_laws += (self.laws.len() - before) as u32;

            // 1. pre-registration (outcome not yet read)
            let mut prereg: Vec<(usize, i64, bool)> = Vec::new();
            for &l in singles.iter().chain(pairs.iter().map(|p| &p.0)) {
                if self.transfer_eligible(l, ctx, &fps) {
                    if let Some((v, borrowed)) = self.prediction(l, ctx) {
                        prereg.push((l, v, borrowed));
                    }
                }
            }
            rep.preregistered += prereg.len() as u32;

            // 2. read the outcome; prequential utility against the best competitor (D012)
            if !self.alphabet.entry(target).or_default().contains(&actual) {
                self.alphabet.get_mut(&target).expect("entry").push(actual);
            }
            let alpha = self.alphabet_size(target);
            let base_loss = self.loss(base, ctx, actual, alpha);
            // D017: credible, strictly more general hypotheses that already cover this case
            let mut cands: Vec<usize> = singles.clone();
            cands.extend(pairs.iter().map(|p| p.0));
            cands.sort_unstable();
            cands.dedup();
            let info: Vec<(usize, i64, u32, bool)> = cands
                .iter()
                .map(|&l| {
                    let (tot, cred) = match self.laws[l].ctx(ctx) {
                        Some(e) => (
                            e.total(),
                            e.counters() == 0 && matches!(e.status, Status::Provisional | Status::Licensed),
                        ),
                        None => (0, false),
                    };
                    (l, self.loss(l, ctx, actual, alpha), tot, cred)
                })
                .collect();
            let general_comp = |l: usize, tot_l: u32| -> i64 {
                let mut best = i64::MAX;
                for &(m, loss_m, tot_m, cred_m) in &info {
                    if m != l && cred_m && (tot_m > tot_l || (tot_m == tot_l && m < l)) {
                        best = best.min(loss_m);
                    }
                }
                best
            };
            let find = |l: usize| info.iter().find(|x| x.0 == l).copied().expect("info");
            let mut deltas: Vec<(usize, i64)> = Vec::new();
            for &s in &singles {
                let (_, ls, ts, _) = find(s);
                let comp = base_loss.min(general_comp(s, ts));
                deltas.push((s, comp - ls));
            }
            for &(p, a, b) in &pairs {
                let (_, lp, tp, _) = find(p);
                let comp = find(a).1.min(find(b).1).min(general_comp(p, tp));
                deltas.push((p, comp - lp));
            }
            deltas.sort_by_key(|d| d.0);
            deltas.dedup_by_key(|d| d.0);
            let t = self.tick;
            for (l, d) in deltas {
                let e = self.laws[l].ctx_mut(ctx, t);
                e.utility_q16 += d;
                if intervention {
                    e.utility_int_q16 += d;
                }
            }

            // 3. resolve pre-registered predictions
            for (l, pred, borrowed) in prereg {
                let e = self.laws[l].ctx_mut(ctx, t);
                if borrowed {
                    e.scope_trials.record(eid, pred, actual);
                } else {
                    e.transfer.record(eid, pred, actual);
                }
                if pred == actual {
                    rep.transfer_ok += 1;
                } else {
                    rep.transfer_fail += 1;
                }
            }

            // 4. update counts
            let mut touched: Vec<usize> = vec![base];
            touched.extend(singles.iter().copied());
            touched.extend(pairs.iter().map(|p| p.0));
            touched.sort_unstable();
            touched.dedup();
            for &l in &touched {
                if self.laws[l].ctx(ctx).map(|e| e.total() == 0).unwrap_or(true) {
                    *self.per_target_ctx.entry((target, ctx)).or_insert(0) += 1;
                }
                self.laws[l].ctx_mut(ctx, t).add(actual, eid, sig, intervention, &fps, t);
            }

            // 5. hidden-condition search on impure singles (D014)
            for &s in &singles {
                let (impure, total, refined_at) = match self.laws[s].ctx(ctx) {
                    Some(e) => (e.counters() > 0, e.total(), e.refined_at),
                    None => (false, 0, 0),
                };
                if impure && total >= self.policy.refine_min_total && total >= refined_at.saturating_mul(2) {
                    self.refine(s, ctx, target);
                    self.laws[s].ctx_mut(ctx, t).refined_at = total;
                }
            }

            // 6. lifecycle
            for &l in &touched {
                if l != base {
                    self.evaluate(l, ctx);
                }
            }
        }
        rep
    }

    /// Hidden-condition search: rank co-present features by entropy reduction among the
    /// parent's episodes, create the best as child candidates initialised by replay.
    fn refine(&mut self, parent: usize, ctx: u64, target: u32) {
        let ids = match self.laws[parent].ctx(ctx) {
            Some(e) => e.all_ids(),
            None => return,
        };
        let pcond = self.laws[parent].condition.clone();
        let action = self.laws[parent].action;
        let mut tally: HashMap<FeatureKind, HashMap<i64, u64>> = HashMap::new();
        let mut parent_hist: HashMap<i64, u64> = HashMap::new();
        for &id in &ids {
            let Some(out) = self.store.get(id).outcome(target) else { continue };
            *parent_hist.entry(out).or_insert(0) += 1;
            for k in &self.feat_kinds[id as usize] {
                if !pcond.contains(k) {
                    *tally.entry(k.clone()).or_default().entry(out).or_insert(0) += 1;
                }
            }
        }
        let ph: Vec<u64> = parent_hist.values().copied().collect();
        let h_parent = hdc_core::fixed::entropy_q16(&ph);
        let mut scored: Vec<(i64, FeatureKind)> = tally
            .into_iter()
            .filter_map(|(k, h)| {
                let counts: Vec<u64> = h.values().copied().collect();
                let n: u64 = counts.iter().sum();
                let hk = hdc_core::fixed::entropy_q16(&counts);
                if n >= 2 && hk < h_parent {
                    Some(((h_parent - hk) * n as i64, k))
                } else {
                    None
                }
            })
            .collect();
        scored.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
        let t = self.tick;
        for (_, g) in scored.into_iter().take(self.policy.refine_top) {
            let active_children = self.laws[parent]
                .lineage
                .children
                .iter()
                .filter(|&&c| self.laws[c].active_ctx.contains(&ctx))
                .count();
            if active_children >= self.policy.max_children {
                break;
            }
            let mut cond = pcond.clone();
            cond.push(g.clone());
            let mut hvs = Vec::new();
            for f in &cond {
                hvs.push(self.cb.feature_hv(f));
            }
            let refs: Vec<&H> = hvs.iter().collect();
            let h = self.cond_hv(action, &refs);
            let child = match self.lookup(&h, target) {
                Some(c) => {
                    // hypothesis exists from another context: activate it here only if this
                    // context's own search selected it, and initialise its evidence by replay
                    if self.laws[c].active_ctx.contains(&ctx) || self.laws[c].ctx(ctx).map(|e| e.total() > 0).unwrap_or(false) {
                        continue;
                    }
                    self.laws[c].active_ctx.push(ctx);
                    c
                }
                None => {
                    let last = *ids.last().unwrap_or(&0);
                    let c = self.create(action, target, cond, h, Origin::Refined { parent }, last);
                    self.laws[c].active_ctx.push(ctx);
                    self.laws[parent].lineage.children.push(c);
                    c
                }
            };
            // replay: counts and independence only; no utility, no transfer credit (D014)
            for &id in &ids {
                if !self.feat_kinds[id as usize].contains(&g) {
                    continue;
                }
                let ep = self.store.get(id);
                let Some(out) = ep.outcome(target) else { continue };
                let (sig, fps, iv) = (ep.signature(), ep.filler_fps(), ep.kind == Kind::Intervention);
                if self.laws[child].ctx(ctx).map(|e| e.total() == 0).unwrap_or(true) {
                    *self.per_target_ctx.entry((target, ctx)).or_insert(0) += 1;
                }
                self.laws[child].ctx_mut(ctx, t).add(out, id, sig, iv, &fps, t);
            }
            self.evaluate(child, ctx);
        }
    }

    /// Recompute the lifecycle status of one law in one context.
    pub fn evaluate(&mut self, l: usize, ctx: u64) {
        let p = self.policy.clone();
        let n_cand = *self.per_target_ctx.get(&(self.laws[l].target, ctx)).unwrap_or(&1) as u64;
        let thresh = log2_q16(n_cand.max(2)) + p.utility_margin_q16;
        let elsewhere_val = self.licensed_value_elsewhere(l, ctx);
        let child_licensed = self.laws[l]
            .lineage
            .children
            .iter()
            .any(|&c| self.laws[c].status_in(ctx) == Status::Licensed);
        let t = self.tick;
        let Some(e) = self.laws[l].ctx(ctx) else { return };
        let old = e.status;
        let new = if old == Status::Split {
            Status::Split
        } else if old == Status::Revoked {
            if child_licensed {
                Status::Split
            } else {
                Status::Revoked
            }
        } else {
            let total = e.total();
            let counters = e.counters();
            let tolerated = counters as u64 * p.noise_tol_den as u64 <= total as u64 * p.noise_tol_num as u64;
            if total == 0 {
                Status::Candidate
            } else if counters > 0 && !tolerated {
                let high = counters as u64 * p.revoke_rate_den as u64 > total as u64 * p.revoke_rate_num as u64;
                if (high && total >= p.min_total_for_revoke) || e.utility_int_q16 < -p.utility_margin_q16 {
                    if child_licensed {
                        Status::Split
                    } else {
                        Status::Revoked
                    }
                } else {
                    Status::Contested
                }
            } else if elsewhere_val.is_some()
                && e.majority().map(|m| m.0) == elsewhere_val
                && e.interventions() >= p.min_scope_support
                && e.scope_trials.fail == 0
            {
                // scope extension (D013): same condition, same effect, verified here
                Status::Licensed
            } else if e.independent() >= p.min_independent
                && e.interventions() >= p.min_interventions
                && e.utility_int_q16 > 0
            {
                // D016: intervention data alone must pay the selection cost
                if e.transfer.ok >= p.min_transfer_ok
                    && e.transfer.fail == 0
                    && e.utility_q16 >= thresh
                    && e.utility_int_q16 >= thresh
                {
                    Status::Licensed
                } else {
                    Status::Provisional
                }
            } else {
                Status::Candidate
            }
        };
        let maj = e.majority().map(|m| m.0);
        if let Some(v) = maj {
            let tgt = self.laws[l].target;
            let eff = self.cb.target(tgt).bind(&self.cb.outcome(tgt, v));
            self.laws[l].predicted_effect_hv = eff;
        }
        self.laws[l].ctx_mut(ctx, t).set_status(new, t);
    }

    fn matching(&mut self, ep: &Episode, target: u32) -> Vec<usize> {
        let feats = self.cb.features(ep);
        let mut v = Vec::new();
        for f in &feats {
            let h = self.cond_hv(ep.action, &[&f.hv]);
            if let Some(l) = self.lookup(&h, target) {
                v.push(l);
            }
        }
        for i in 0..feats.len() {
            for j in (i + 1)..feats.len() {
                let h = self.cond_hv(ep.action, &[&feats[i].hv, &feats[j].hv]);
                if let Some(l) = self.lookup(&h, target) {
                    if self.laws[l].active_in(ep.context) {
                        v.push(l);
                    }
                }
            }
        }
        v
    }

    /// Every law applicable in the query's context that matches it, with its predicted value.
    pub fn explain(&mut self, ep: &Episode, target: u32) -> Vec<(usize, i64)> {
        let ctx = ep.context;
        let m = self.matching(ep, target);
        m.into_iter()
            .filter(|&l| self.laws[l].applicable(ctx))
            .filter_map(|l| self.laws[l].ctx(ctx).and_then(|e| e.majority()).map(|v| (l, v.0)))
            .collect()
    }

    /// Answer a query using only laws licensed in the query's context.
    pub fn predict(&mut self, ep: &Episode, target: u32) -> Answer {
        self.predict_with(ep, target, false)
    }

    /// `ignore_scope = true` is the ablation that reproduces ATANOR's DS1 failure mode.
    pub fn predict_with(&mut self, ep: &Episode, target: u32, ignore_scope: bool) -> Answer {
        let ctx = ep.context;
        let m = self.matching(ep, target);
        let usable: Vec<usize> = m
            .iter()
            .copied()
            .filter(|&l| {
                if ignore_scope {
                    self.laws[l].ctx.iter().any(|e| e.status == Status::Licensed)
                } else {
                    self.laws[l].applicable(ctx)
                }
            })
            .collect();
        if usable.is_empty() {
            let elsewhere = m.iter().any(|&l| self.laws[l].ctx.iter().any(|e| e.status == Status::Licensed));
            return Answer::Abstain(if elsewhere { Abstain::OutOfScope } else { Abstain::NoLicensedLaw });
        }
        let mut val = None;
        let mut conf = i64::MAX;
        for &l in &usable {
            let lic_ctx = if ignore_scope {
                self.laws[l].ctx.iter().find(|e| e.status == Status::Licensed).map(|e| e.context).unwrap_or(ctx)
            } else {
                ctx
            };
            let e = self.laws[l].ctx(lic_ctx).expect("licensed ctx");
            let v = e.majority().map(|m| m.0);
            let c = e.confidence();
            conf = conf.min(ratio_q16(c.support as u64 + 1, c.total() as u64 + 2));
            match (val, v) {
                (None, Some(x)) => val = Some(x),
                (Some(a), Some(b)) if a != b => return Answer::Abstain(Abstain::Conflict),
                _ => {}
            }
        }
        match val {
            Some(v) => Answer::Value { val: v, laws: usable, confidence_q16: conf },
            None => Answer::Abstain(Abstain::NoLicensedLaw),
        }
    }

    /// Competing hypotheses: other non-base laws for the same action/target sharing at least one
    /// episode in `ctx` and predicting a different value there.
    pub fn competitors(&self, l: usize, ctx: u64) -> Vec<usize> {
        let law = &self.laws[l];
        let Some(e) = law.ctx(ctx) else { return vec![] };
        let mine: std::collections::HashSet<u64> = e.all_ids().into_iter().collect();
        let my_val = e.majority().map(|m| m.0);
        let mut out = Vec::new();
        for o in &self.laws {
            if o.id == l || o.is_base() || o.target != law.target || o.action != law.action {
                continue;
            }
            let Some(oe) = o.ctx(ctx) else { continue };
            if oe.status < Status::Contested || oe.status.terminal() {
                continue;
            }
            if oe.majority().map(|m| m.0) != my_val && oe.all_ids().iter().any(|i| mine.contains(i)) {
                out.push(o.id);
            }
        }
        out
    }

    /// Laws licensed in `ctx`.
    pub fn licensed_in(&self, ctx: u64) -> Vec<usize> {
        self.laws.iter().filter(|l| l.applicable(ctx)).map(|l| l.id).collect()
    }

    pub fn candidates_for(&self, target: u32) -> u32 {
        *self.per_target.get(&target).unwrap_or(&0)
    }

    /// One-line evidence summary of a law in a context.
    pub fn summary(&self, l: usize, ctx: u64) -> String {
        let law = &self.laws[l];
        match law.ctx(ctx) {
            None => format!("#{l} {} [no evidence here]", law.describe()),
            Some(e) => format!(
                "#{l} {} => {:?} | {} | n={} counter={} indep={} int={} transfer {}/{} utility {}b (int {}b) origin {:?}",
                law.describe(),
                e.majority().map(|m| m.0),
                e.status.name(),
                e.total(),
                e.counters(),
                e.independent(),
                e.interventions(),
                e.transfer.ok,
                e.transfer.ok + e.transfer.fail,
                e.utility_q16 >> 16,
                e.utility_int_q16 >> 16,
                law.lineage.origin
            ),
        }
    }
}
