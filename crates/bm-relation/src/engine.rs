//! RelationEngine: candidate generation, prequential evidence, pre-registered transfer tests,
//! hidden-condition search and the licensing lifecycle.

use crate::episode::{Episode, EpisodeStore, Kind};
use crate::latent::LatentInducer;
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
    /// Latent-cause inducers (Phase C). Their partitions are added to role fillers.
    pub latent: Vec<LatentInducer>,
    /// Channel carrying entity identity; enables automatic latent induction when set.
    pub identity_channel: Option<u16>,
    /// (action, target, context) -> episodes seen; used to trigger latent induction.
    unexplained: HashMap<(u16, u32, u64), u32>,
}

/// What one sleep consolidation did.
#[derive(Clone, Debug, Default)]
pub struct SleepStats {
    pub laws_before: u32,
    pub laws_after: u32,
    pub refined_children: u32,
    pub latent_inductions: u32,
    pub pruned: u32,
    pub redundant: u32,
}

/// Epistemic state of one query/target (see `RelationEngine::uncertainty`).
#[derive(Clone, Debug)]
pub struct Uncertainty {
    pub licensed: bool,
    pub counts: Vec<u64>,
    pub near_licence: u32,
}

/// One edge of the licensed causal model: under `action` and `condition`, `target` takes `value`.
#[derive(Clone, Debug)]
pub struct CausalEdge {
    pub law: usize,
    pub action: u16,
    pub condition: Vec<FeatureKind>,
    pub target: u32,
    pub value: i64,
    pub interventions: u32,
}

/// D020: failures at or below the sensor-noise tolerance may be explained as observation error
/// (deterministic worlds: tolerance 0, so any failure counts).
fn within_noise(fail: u32, total: u32, p: &LicensePolicy) -> bool {
    fail as u64 <= noise_allowance(total, p)
}

/// D023: largest failure count measured sensor noise can explain among `n` observations:
/// mean + 3 standard deviations of a binomial(n, tol), rounded up (0 when tol = 0).
pub fn noise_allowance(n: u32, p: &LicensePolicy) -> u64 {
    if p.noise_tol_num == 0 {
        return 0;
    }
    let num = p.noise_tol_num as u64;
    let den = p.noise_tol_den.max(1) as u64;
    let n = n as u64;
    let mean_x100 = n * num * 100 / den; // mean scaled by 100
    let var = n * num * (den - num.min(den)) / (den * den); // n p (1-p), integer floor
    (mean_x100 + 99) / 100 + 3 * (hdc_core::isqrt(var) + 1)
}

fn key(cond_hv: &H, target: u32) -> u64 {
    key_fp(cond_hv.fingerprint(), target)
}

fn key_fp(fp: u64, target: u32) -> u64 {
    mix64(fp, target as u64 ^ 0x7A11)
}

/// Condition vectors and fingerprints of one episode, shared by all its targets.
struct CondCache {
    base_hv: H,
    base_fp: u64,
    single_hv: Vec<H>,
    single_fp: Vec<u64>,
    /// (i, j, fingerprint of act ^ f_i ^ f_j)
    pair_fp: Vec<(usize, usize, u64)>,
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
            latent: Vec::new(),
            identity_channel: None,
            unexplained: HashMap::new(),
        }
    }

    /// Add latent fillers from every inducer.
    pub fn augment(&self, ep: &mut Episode) {
        for ind in &self.latent {
            ind.augment(ep);
        }
    }

    /// Start latent induction for (action, target) explicitly.
    pub fn enable_latent(&mut self, action: u16, target: u32, id_ch: u16) {
        if self.latent.iter().any(|l| l.action == action && l.target == target) {
            return;
        }
        // versioned channel ranges: inducer k owns 3000 + 1000k .. (block even, link odd)
        let k = self.latent.len() as u16;
        self.latent.push(LatentInducer::new(action, target, id_ch, 3000 + 1000 * k, 3001 + 1000 * k));
    }

    fn latent_bookkeeping(&mut self, ep: &Episode) {
        for ind in self.latent.iter_mut() {
            let before = ind.observations();
            ind.add(ep);
            if self.policy.online_refine && ind.observations() != before && ind.due(ind.observations()) {
                ind.induce();
            }
        }
        // automatic trigger: a pair action whose target has no licensed law after 60 episodes
        let Some(idc) = self.identity_channel else { return };
        if ep.roles.len() < 2 || ep.roles[0].get(idc).is_none() || ep.roles[1].get(idc).is_none() {
            return;
        }
        let n_args = if ep.n_args == 0 { ep.roles.len() } else { ep.n_args as usize };
        for &(t, _) in &ep.outcomes {
            // D038: latent pair causes explain effects on the action's arguments
            if (t / 10_000) as usize >= n_args {
                continue;
            }
            let n = self.unexplained.entry((ep.action, t, ep.context)).or_insert(0);
            *n += 1;
            if *n == 60 {
                let licensed = self.laws.iter().any(|l| l.target == t && l.action == ep.action && l.applicable(ep.context));
                let same_slot = self.latent.iter().any(|l| l.action == ep.action && l.target / 10_000 == t / 10_000 && l.target % 1000 == t % 1000);
                if !licensed && !same_slot {
                    // D038a: among this slot's targets, base the hypothesis on the most structured
                    // one (lowest outcome entropy under the unconditional law)
                    let ctx = ep.context;
                    let mut best = (i64::MAX, t);
                    for &(t2, _) in &ep.outcomes {
                        if t2 / 10_000 != t / 10_000 || t2 % 1000 != t % 1000 {
                            continue;
                        }
                        let h = self
                            .laws
                            .iter()
                            .find(|l| l.is_base() && l.action == ep.action && l.target == t2)
                            .and_then(|l| l.ctx(ctx))
                            .map(|e| hdc_core::fixed::entropy_q16(&e.bins.iter().map(|b| b.count as u64).collect::<Vec<_>>()))
                            .unwrap_or(i64::MAX - 1);
                        if h < best.0 {
                            best = (h, t2);
                        }
                    }
                    self.enable_latent(ep.action, best.1, idc);
                }
            }
        }
    }

    /// Sleep consolidation (Phase E, layer 4) for one context:
    /// 1. hidden-condition search on every impure single-feature law (replay from the store);
    /// 2. latent-cause induction on every inducer (and auto-enable on unexplained pair targets);
    /// 3. compression: hypotheses with at most one observation that are older than `min_age`
    ///    ticks and licensed nowhere leave the hypothesis space (selection cost drops, D018);
    /// 4. licensed laws whose condition strictly contains another licensed law's condition with
    ///    the same prediction are counted as redundant.
    /// Hypotheses created here earn utility and transfer only from later live episodes (D014).
    pub fn sleep(&mut self, ctx: u64, min_age: u64) -> SleepStats {
        let mut st = SleepStats { laws_before: self.active_hypotheses(ctx), ..Default::default() };
        // 1. refinement
        let impure: Vec<(usize, u32)> = self
            .laws
            .iter()
            .filter(|l| !l.pruned && l.condition.len() == 1)
            .filter_map(|l| {
                let e = l.ctx(ctx)?;
                (e.counters() > 0 && e.total() >= self.policy.refine_min_total).then_some((l.id, l.target))
            })
            .collect();
        for (l, t) in impure {
            let before = self.laws.len();
            self.refine(l, ctx, t);
            st.refined_children += (self.laws.len() - before) as u32;
        }
        // 2. latent induction
        if let Some(idc) = self.identity_channel {
            let keys: Vec<(u16, u32)> = self
                .unexplained
                .iter()
                .filter(|(k, &n)| k.2 == ctx && n >= 20)
                .map(|(k, _)| (k.0, k.1))
                .collect();
            for (a, t) in keys {
                let licensed = self.laws.iter().any(|l| l.target == t && l.action == a && l.applicable(ctx));
                if !licensed {
                    self.enable_latent(a, t, idc);
                }
            }
        }
        // feed every stored episode to newly enabled inducers, then induce
        let n_ind = self.latent.len();
        for i in 0..n_ind {
            if self.latent[i].observations() == 0 {
                for id in 0..self.store.len() as u64 {
                    let ep = self.store.get(id).clone();
                    if ep.context == ctx {
                        self.latent[i].add(&ep);
                    }
                }
            }
            self.latent[i].induce();
            st.latent_inductions += 1;
        }
        // 3. compression
        let now = self.tick;
        for l in 0..self.laws.len() {
            let law = &self.laws[l];
            if law.pruned || law.is_base() || law.ctx.iter().any(|e| e.status == Status::Licensed) {
                continue;
            }
            let Some(e) = law.ctx(ctx) else { continue };
            if e.total() <= 1 && now.saturating_sub(law.lineage.created_t) > min_age {
                let key = key(&law.condition_hv, law.target);
                self.index.remove(&key);
                if let Some(c) = self.per_target_ctx.get_mut(&(law.target, ctx)) {
                    *c = c.saturating_sub(1);
                }
                self.laws[l].pruned = true;
                st.pruned += 1;
            }
        }
        // 4. redundancy among licensed laws
        let lic = self.licensed_in(ctx);
        for &a in &lic {
            let la = &self.laws[a];
            let va = la.ctx(ctx).and_then(|e| e.majority()).map(|m| m.0);
            if lic.iter().any(|&b| {
                let lb = &self.laws[b];
                b != a
                    && lb.target == la.target
                    && lb.action == la.action
                    && lb.condition.len() < la.condition.len()
                    && lb.condition.iter().all(|f| la.condition.contains(f))
                    && lb.ctx(ctx).and_then(|e| e.majority()).map(|m| m.0) == va
            }) {
                st.redundant += 1;
            }
        }
        st.laws_after = self.active_hypotheses(ctx);
        st
    }

    /// Hypotheses currently in the space of a context (not pruned, with evidence there).
    pub fn active_hypotheses(&self, ctx: u64) -> u32 {
        self.laws.iter().filter(|l| !l.pruned && l.ctx(ctx).map(|e| e.total() > 0).unwrap_or(false)).count() as u32
    }

    /// The licensed causal model of a context as edges (Phase C).
    pub fn causal_graph(&self, ctx: u64) -> Vec<CausalEdge> {
        self.laws
            .iter()
            .filter(|l| l.applicable(ctx))
            .filter_map(|l| {
                let e = l.ctx(ctx)?;
                Some(CausalEdge {
                    law: l.id,
                    action: l.action,
                    condition: l.condition.clone(),
                    target: l.target,
                    value: e.majority()?.0,
                    interventions: e.interventions(),
                })
            })
            .collect()
    }

    /// Counterfactual query (Phase C): what would `target` have been in the actual episode had
    /// the agent taken `alt_action` and/or had some role fillers been different? Everything not
    /// edited is held at its actual value (abduction of the unedited state).
    pub fn counterfactual(&mut self, actual: &Episode, alt_action: Option<u16>, edits: &[(usize, u16, i64)], target: u32) -> Answer {
        let mut q = actual.without_outcomes();
        if let Some(a) = alt_action {
            q.action = a;
        }
        for &(r, ch, v) in edits {
            if let Some(role) = q.roles.get_mut(r) {
                match role.fillers.iter_mut().find(|f| f.ch == ch) {
                    Some(f) => f.val = v,
                    None => role.fillers.push(crate::episode::Filler { ch, val: v }),
                }
            }
        }
        self.predict(&q, target)
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
            pruned: false,
        });
        self.index.insert(key(&cond_hv, target), id);
        *self.per_target.entry(target).or_insert(0) += 1;
        id
    }

    fn cond_cache(&mut self, action: u16, feats: &[Feature]) -> CondCache {
        let base_hv = self.cb.action(action);
        let base_fp = base_hv.fingerprint();
        let single_hv: Vec<H> = feats.iter().map(|f| base_hv.bind(&f.hv)).collect();
        let single_fp: Vec<u64> = single_hv.iter().map(|h| h.fingerprint()).collect();
        let mut pair_fp = Vec::with_capacity(feats.len() * feats.len() / 2);
        for i in 0..feats.len() {
            for j in (i + 1)..feats.len() {
                pair_fp.push((i, j, single_hv[i].fingerprint_xor(&feats[j].hv)));
            }
        }
        CondCache { base_hv, base_fp, single_hv, single_fp, pair_fp }
    }

    fn get_or_create_cached(&mut self, action: u16, target: u32, fp: u64, hv: &H, kinds: Vec<FeatureKind>, episode: u64) -> usize {
        if let Some(&i) = self.index.get(&key_fp(fp, target)) {
            return i;
        }
        self.create(action, target, kinds, hv.clone(), Origin::Generated, episode)
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
            if e.total() > 0 && within_noise(e.counters(), e.total(), &self.policy) && !e.status.terminal() {
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

    fn transfer_eligible(&self, law: usize, ctx: u64, fps: &[u64], sig: u64) -> bool {
        let l = &self.laws[law];
        let own = match l.ctx(ctx) {
            Some(e) if e.total() > 0 => {
                // D015a: a held-out compositional case is a new filler OR a combination of
                // known fillers this law has never been supported by
                !e.status.terminal()
                    && within_noise(e.counters(), e.total(), &self.policy)
                    && e.independent() >= 3
                    && (fps.iter().any(|f| !e.fillers_seen.contains(f)) || !e.bins.iter().any(|b| b.signatures.contains(&sig)))
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
        self.latent_bookkeeping(&ep);
        let mut ep = ep;
        self.augment(&mut ep);
        let feats = self.cb.features(&ep);
        let sig = ep.signature();
        let fps = ep.filler_fps();
        let ctx = ep.context;
        let intervention = ep.kind == Kind::Intervention;
        let outcomes = ep.outcomes.clone();
        let action = ep.action;
        let eid = self.store.append(ep);
        self.feat_kinds.push(feats.iter().map(|f| f.kind.clone()).collect());

        let cc = self.cond_cache(action, &feats);
        for (target, actual) in outcomes {
            let before = self.laws.len();
            let base = self.get_or_create_cached(action, target, cc.base_fp, &cc.base_hv, Vec::new(), eid);
            let mut singles = Vec::with_capacity(feats.len());
            for (i, f) in feats.iter().enumerate() {
                singles.push(self.get_or_create_cached(action, target, cc.single_fp[i], &cc.single_hv[i], vec![f.kind.clone()], eid));
            }
            let mut pairs: Vec<(usize, usize, usize)> = Vec::new();
            for &(i, j, fp) in &cc.pair_fp {
                if let Some(&l) = self.index.get(&key_fp(fp, target)) {
                    if self.laws[l].active_in(ctx) && !self.laws[l].pruned {
                        pairs.push((l, singles[i], singles[j]));
                    }
                }
            }
            rep.new_laws += (self.laws.len() - before) as u32;

            // 1. pre-registration (outcome not yet read)
            let mut prereg: Vec<(usize, i64, bool)> = Vec::new();
            for &l in std::iter::once(&base).chain(singles.iter()).chain(pairs.iter().map(|p| &p.0)) {
                if self.transfer_eligible(l, ctx, &fps, sig) {
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
            // D037: the unconditional law competes against the ignorant uniform model
            let uniform = log2_q16(alpha);
            // D017: credible, strictly more general hypotheses that already cover this case
            let mut cands: Vec<usize> = singles.clone();
            cands.push(base);
            cands.extend(pairs.iter().map(|p| p.0));
            cands.sort_unstable();
            cands.dedup();
            let maj_of = |l: usize| self.laws[l].ctx(ctx).and_then(|e| e.majority()).map(|m| m.0);
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
            let majs: Vec<(usize, Option<i64>)> = cands.iter().map(|&l| (l, maj_of(l))).collect();
            let maj = |l: usize| majs.iter().find(|x| x.0 == l).and_then(|x| x.1);
            // D042: (loss, competitor id) of the best credible strictly-more-general competitor
            let general_comp = |l: usize, tot_l: u32| -> (i64, Option<usize>) {
                let mut best = (i64::MAX, None);
                for &(m, loss_m, tot_m, cred_m) in &info {
                    if m != l && cred_m && (tot_m > tot_l || (tot_m == tot_l && m < l)) && loss_m < best.0 {
                        best = (loss_m, Some(m));
                    }
                }
                best
            };
            // incremental utility: where a credible general competitor already predicted the
            // actual value as its majority and so did this law, nothing was gained or lost
            let incremental = |l: usize, own: i64, comp: (i64, Option<usize>), fallback: i64| -> i64 {
                match comp.1 {
                    Some(m) if maj(m) == Some(actual) && maj(l) == Some(actual) => 0,
                    _ => fallback.min(comp.0) - own,
                }
            };
            let find = |l: usize| info.iter().find(|x| x.0 == l).copied().expect("info");
            let mut deltas: Vec<(usize, i64)> = vec![(base, uniform - base_loss)];
            for &s in &singles {
                let (_, ls, ts, _) = find(s);
                let gc = general_comp(s, ts);
                deltas.push((s, incremental(s, ls, gc, base_loss)));
            }
            for &(p, a, b) in &pairs {
                let (_, lp, tp, _) = find(p);
                let gc = general_comp(p, tp);
                let parents = find(a).1.min(find(b).1);
                deltas.push((p, incremental(p, lp, gc, parents)));
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
                if self.policy.online_refine && impure && total >= self.policy.refine_min_total && total >= refined_at.saturating_mul(2) {
                    self.refine(s, ctx, target);
                    self.laws[s].ctx_mut(ctx, t).refined_at = total;
                }
            }

            // 6. lifecycle
            for &l in &touched {
                self.evaluate(l, ctx);
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

    /// Licensing policy for a target: the global policy, with the noise tolerance raised to the
    /// measured repeatability noise of that target when a latent inducer observes it (D040).
    pub fn policy_for(&self, target: u32) -> LicensePolicy {
        let mut p = self.policy.clone();
        for ind in &self.latent {
            if ind.target != target {
                continue;
            }
            let (m, t) = ind.repeat_noise();
            if t >= 30 && m > 0 {
                // compare m/t with num/den without floats; express as percent, rounded up
                let pct = ((m * 100 + t - 1) / t) as u32;
                if pct * p.noise_tol_den > p.noise_tol_num * 100 {
                    p.noise_tol_num = pct;
                    p.noise_tol_den = 100;
                }
            }
        }
        p
    }

    /// Recompute the lifecycle status of one law in one context.
    pub fn evaluate(&mut self, l: usize, ctx: u64) {
        let p = self.policy_for(self.laws[l].target);
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
            let allow = noise_allowance(total, &p);
            let tolerated = counters as u64 <= allow;
            if total == 0 {
                Status::Candidate
            } else if counters > 0 && !tolerated {
                // revoke only on counterevidence beyond both the noise allowance and the revoke rate
                let excess = counters as u64 - allow;
                let high = excess * p.revoke_rate_den as u64 > total as u64 * p.revoke_rate_num as u64;
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
                && within_noise(e.scope_trials.fail, e.scope_trials.ok + e.scope_trials.fail, &p)
            {
                // scope extension (D013): same condition, same effect, verified here
                Status::Licensed
            } else if e.independent() >= p.min_independent
                && e.interventions() >= p.min_interventions
                && e.utility_int_q16 > 0
            {
                // D016: intervention data alone must pay the selection cost
                if e.transfer.ok >= p.min_transfer_ok
                    && within_noise(e.transfer.fail, e.transfer.ok + e.transfer.fail, &p)
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
        let cc = self.cond_cache(ep.action, &feats);
        let mut v = Vec::new();
        if let Some(&l) = self.index.get(&key_fp(cc.base_fp, target)) {
            v.push(l);
        }
        for &fp in &cc.single_fp {
            if let Some(&l) = self.index.get(&key_fp(fp, target)) {
                v.push(l);
            }
        }
        for &(_, _, fp) in &cc.pair_fp {
            if let Some(&l) = self.index.get(&key_fp(fp, target)) {
                if self.laws[l].active_in(ep.context) {
                    v.push(l);
                }
            }
        }
        v
    }

    /// Epistemic state of a query for one target (Phase D belief state):
    /// * `licensed`: a licensed law answers it;
    /// * `counts`: outcome histogram (over the target's alphabet) of the best current hypothesis
    ///   matching the query (lowest predictive entropy among non-terminal matching laws);
    /// * `near_licence`: matching hypotheses that are consistent so far and lack only more
    ///   interventions or transfer trials (testing them is informative).
    pub fn uncertainty(&mut self, ep: &Episode, target: u32) -> Uncertainty {
        let ctx = ep.context;
        let mut aug = ep.clone();
        self.augment(&mut aug);
        let m = self.matching(&aug, target);
        let alpha: Vec<i64> = self.alphabet.get(&target).cloned().unwrap_or_default();
        let licensed = m.iter().any(|&l| self.laws[l].applicable(ctx));
        let mut best: Option<(i64, Vec<u64>)> = None;
        let mut near = 0u32;
        for &l in &m {
            let Some(e) = self.laws[l].ctx(ctx) else { continue };
            if e.status.terminal() || e.total() == 0 {
                continue;
            }
            // open alphabet: one extra bucket for a value never seen yet
            let mut counts: Vec<u64> = alpha.iter().map(|&v| e.count_of(v) as u64).collect();
            counts.push(0);
            let h = hdc_core::fixed::predictive_entropy_q16(&counts);
            if best.as_ref().map(|b| h < b.0).unwrap_or(true) {
                best = Some((h, counts));
            }
            if e.status != Status::Licensed
                && within_noise(e.counters(), e.total(), &self.policy)
                && e.independent() >= 3
            {
                near += 1;
            }
        }
        let counts = best.map(|b| b.1).unwrap_or_else(|| vec![0; alpha.len() + 1]);
        Uncertainty { licensed, counts, near_licence: near }
    }

    /// Ids of every active (non-pruned) hypothesis matching a query for a target.
    pub fn matching_ids(&mut self, ep: &Episode, target: u32) -> Vec<usize> {
        let mut aug = ep.clone();
        self.augment(&mut aug);
        self.matching(&aug, target).into_iter().filter(|&l| !self.laws[l].pruned).collect()
    }

    /// Every law applicable in the query's context that matches it, with its predicted value.
    pub fn explain(&mut self, ep: &Episode, target: u32) -> Vec<(usize, i64)> {
        let ctx = ep.context;
        let mut aug = ep.clone();
        self.augment(&mut aug);
        let ep = &aug;
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
        let mut aug = ep.clone();
        self.augment(&mut aug);
        let ep = &aug;
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
        let mut by_val: Vec<(i64, Vec<usize>, i64)> = Vec::new();
        for &l in &usable {
            let lic_ctx = if ignore_scope {
                self.laws[l].ctx.iter().find(|e| e.status == Status::Licensed).map(|e| e.context).unwrap_or(ctx)
            } else {
                ctx
            };
            let e = self.laws[l].ctx(lic_ctx).expect("licensed ctx");
            let Some(v) = e.majority().map(|m| m.0) else { continue };
            let c = e.confidence();
            let cq = ratio_q16(c.support as u64 + 1, c.total() as u64 + 2);
            match by_val.iter_mut().find(|x| x.0 == v) {
                Some(x) => {
                    x.1.push(l);
                    x.2 = x.2.min(cq);
                }
                None => by_val.push((v, vec![l], cq)),
            }
        }
        match by_val.len() {
            0 => Answer::Abstain(Abstain::NoLicensedLaw),
            1 => {
                let (v, laws, c) = by_val.pop().expect("one");
                Answer::Value { val: v, laws, confidence_q16: c }
            }
            _ => match self.resolve_conflict(&by_val, ctx, target) {
                Some(i) => {
                    let (v, laws, c) = by_val.swap_remove(i);
                    Answer::Value { val: v, laws, confidence_q16: c }
                }
                None => Answer::Abstain(Abstain::Conflict),
            },
        }
    }

    /// D041: settle a conflict between licensed laws by their record on the cases where both
    /// applied. For every pair of value groups, the overlap of their laws' episodes is scored; the
    /// group whose laws were right strictly more often (with >= 3 shared cases) wins every pairwise
    /// comparison, otherwise the conflict stands and the engine abstains.
    fn resolve_conflict(&self, groups: &[(i64, Vec<usize>, i64)], ctx: u64, target: u32) -> Option<usize> {
        let ids = |l: usize| -> std::collections::HashSet<u64> {
            self.laws[l].ctx(ctx).map(|e| e.all_ids().into_iter().collect()).unwrap_or_default()
        };
        let mut wins = vec![0usize; groups.len()];
        for a in 0..groups.len() {
            for b in (a + 1)..groups.len() {
                let ia: std::collections::HashSet<u64> = groups[a].1.iter().flat_map(|&l| ids(l)).collect();
                let ib: std::collections::HashSet<u64> = groups[b].1.iter().flat_map(|&l| ids(l)).collect();
                let shared: Vec<u64> = ia.intersection(&ib).copied().collect();
                if shared.len() < 3 {
                    continue;
                }
                let (mut ra, mut rb) = (0, 0);
                for e in shared {
                    match self.store.get(e).outcome(target) {
                        Some(o) if o == groups[a].0 => ra += 1,
                        Some(o) if o == groups[b].0 => rb += 1,
                        _ => {}
                    }
                }
                if ra > rb {
                    wins[a] += 1;
                } else if rb > ra {
                    wins[b] += 1;
                }
            }
        }
        let need = groups.len() - 1;
        wins.iter().position(|&w| w == need)
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
