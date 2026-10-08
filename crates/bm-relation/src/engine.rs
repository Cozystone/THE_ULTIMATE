//! RelationEngine: candidate generation, prequential evidence, pre-registered transfer tests,
//! hidden-condition search and the licensing lifecycle.

use crate::episode::{Episode, EpisodeStore, Kind};
use crate::latent::LatentInducer;
use crate::features::{Codebook, Feature, FeatureKind, H};
use crate::law::*;
use hdc_core::fixed::{log2_q16, ratio_q16};
use hdc_core::rng::mix64;
use std::collections::HashMap;

/// D041a: independent shared situations needed before a conflict between licensed laws is settled.
pub const MIN_SHARED_INDEPENDENT: u32 = 3;

/// D055: bits a deferred hypothesis must carry beyond log2(M) to be materialized.
pub const GATE_MARGIN_BITS: i64 = 4;
/// D055: minimum cases before a deferred value is tested.
pub const GATE_MIN_CASES: u32 = 3;
/// D055: values tracked per feature family and target; further values cannot earn capacity.
pub const VALUES_PER_FAMILY: u32 = 4096;
/// D055: outcomes kept per value table entry; further outcomes are pooled.
pub const OUTCOMES_PER_VALUE: usize = 8;

/// D055: outcome counts of one deferred feature value for one target.
#[derive(Clone, Debug, Default)]
pub struct DeferredValue {
    pub hist: Vec<(i64, u32)>,
    pub pooled: u32,
    /// D055a: sum over its cases of log2 p_baseline(outcome) (Q16, <= 0).
    pub ll_base: i64,
    /// amendment 4: sum over its cases of the prequential log2 p_hypothesis(outcome) (Q16).
    pub ll_model: i64,
    /// amendment 6: the same information restricted to intervention cases.
    pub info_int: i64,
    /// D057: relevant situations of this value's cases (prospective transfer novelty); dropped
    /// (saturated) at the first counterexample, after which the value earns no trial.
    pub rsits: KeySet,
    /// D057: up to 3 distinct relevant bindings of its cases.
    pub binds: Vec<u64>,
    /// D057: pre-registered predictions on unseen relevant situations while deferred.
    pub t_ok: u32,
    pub t_fail: u32,
    /// D057: failed prospective trials (episode, predicted, actual), bounded.
    pub t_failed: Vec<(u64, i64, i64)>,
}

/// D057: failed prospective trials kept per deferred value.
pub const DEFERRED_FAILED_CAP: usize = 16;

/// D055a: how often each input filler (role, channel) equalled a target's outcome (copy model).
#[derive(Clone, Debug, Default)]
pub struct EchoStats {
    pub hits: HashMap<(u8, u16), u32>,
    pub total: u32,
}

impl DeferredValue {
    pub fn total(&self) -> u32 {
        self.hist.iter().map(|x| x.1).sum::<u32>() + self.pooled
    }
}

/// D055: deferred evidence for one (action, target, context): outcome counts per feature value
/// that has not earned a law, the number of values ever tested (M, multiple-comparison count) and
/// per-family value counts (bounded).
#[derive(Clone, Debug, Default)]
pub struct ValueTable {
    pub tested: u64,
    pub entries: HashMap<FeatureKind, DeferredValue>,
    pub family_values: HashMap<u64, u32>,
    pub untracked: u64,
}

/// D055: a feature family = the feature kind with its value removed.
fn family_of(k: &FeatureKind) -> u64 {
    use hdc_core::rng::mix64;
    match *k {
        FeatureKind::Abs { role, ch, .. } => mix64(mix64(1, role as u64), ch as u64),
        FeatureKind::Same { r1, r2, ch } | FeatureKind::Diff { r1, r2, ch } => mix64(mix64(2, (r1 as u64) << 8 | r2 as u64), ch as u64),
        FeatureKind::Order { r1, r2, ch, .. } => mix64(mix64(3, (r1 as u64) << 8 | r2 as u64), ch as u64),
        FeatureKind::Delta { r1, r2, ch, .. } => mix64(mix64(4, (r1 as u64) << 8 | r2 as u64), ch as u64),
    }
}

/// D057: the roles a feature mentions (as `Law::cond_roles` for a one-feature condition).
fn feature_roles(k: &FeatureKind) -> Vec<u8> {
    match *k {
        FeatureKind::Abs { role, .. } => vec![role],
        FeatureKind::Same { r1, r2, .. } | FeatureKind::Diff { r1, r2, .. } | FeatureKind::Order { r1, r2, .. } | FeatureKind::Delta { r1, r2, .. } => vec![r1, r2],
    }
}

/// D055: information in bits (Q16) that `hist` carries against a baseline with counts `base`
/// (Laplace-smoothed over an alphabet of `alpha` outcomes): sum h_o * log2((h_o / n) / p_o).
/// Pooled outcomes are ignored (an underestimate, i.e. conservative).
pub fn llr_q16(hist: &[(i64, u32)], base: &[(i64, u64)], alpha: u64) -> i64 {
    let n: u64 = hist.iter().map(|x| x.1 as u64).sum();
    if n == 0 {
        return 0;
    }
    let big_n: u64 = base.iter().map(|x| x.1).sum();
    let mut l = 0i64;
    for &(o, h) in hist {
        if h == 0 {
            continue;
        }
        let c = base.iter().find(|x| x.0 == o).map(|x| x.1).unwrap_or(0);
        l += h as i64 * (log2_q16(h as u64) - log2_q16(n) - log2_q16(c + 1) + log2_q16(big_n + alpha.max(1)));
    }
    l
}

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
    /// D045: how often each action was taken in each context (evidence even when nothing changed).
    trials: HashMap<(u16, u64), u32>,
    /// D049: channels a sensor declared ordinal (order / offset allowed unless vetoed).
    pub ordinal: std::collections::BTreeSet<u16>,
    /// D049: declared-ordinal channels found to behave as identifiers (injective over >= 8 bound
    /// objects); treated as nominal.
    pub vetoed: std::collections::BTreeSet<u16>,
    /// value -> the single binding it was seen with (None once seen with two bindings).
    ord_seen: HashMap<u16, HashMap<i64, Option<u64>>>,
    /// D055: deferred evidence per (action, target, context).
    pub deferred: HashMap<(u16, u32, u64), ValueTable>,
    /// D055: feature -> episodes in which it occurred (replay source when a hypothesis is
    /// materialized), capped per feature.
    inv: HashMap<FeatureKind, Vec<u64>>,
    /// D055: refinement candidates examined per (target, context) (multiple-comparison count).
    refine_tested: HashMap<(u32, u64), u64>,
    /// D055: hypotheses materialized from deferred evidence (for reporting).
    pub materialized: u64,
    /// D055a: copy-model statistics per (action, target, context).
    pub echo: HashMap<(u16, u32, u64), EchoStats>,
    /// amendment 6: the gate's prequential information handed to the next materialized law.
    pending_utility: Option<(i64, i64)>,
    /// D057: the prospective transfer record handed to the next materialized law.
    pending_transfer: Option<(u32, u32, Vec<(u64, i64, i64)>)>,
    /// D056 (K5): effective hypothesis family size per (action, target, context): every
    /// candidate ever examined there (deferred, materialized, untracked, refinement candidate,
    /// or law given evidence). Monotone; never decremented by pruning, retirement or sleep.
    pub family: HashMap<(u16, u32, u64), u64>,
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
    /// Independent supports of consistent unlicensed hypotheses for which this query would be a
    /// new combination (D043d).
    pub licence_k: Vec<u32>,
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
            pending_utility: None,
            pending_transfer: None,
            family: HashMap::new(),
            echo: HashMap::new(),
            deferred: HashMap::new(),
            inv: HashMap::new(),
            refine_tested: HashMap::new(),
            materialized: 0,
            ordinal: std::collections::BTreeSet::new(),
            vetoed: std::collections::BTreeSet::new(),
            ord_seen: HashMap::new(),
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
            trials: HashMap::new(),
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

    /// D051: how much better than chance a repeated (a, b) pair repeats its outcome on `target`:
    /// (observed repeat rate - sum of squared marginal frequencies) in Q16, and the number of
    /// repeats. Uses the stored episodes of this action and context.
    pub fn pair_repeatability(&self, action: u16, ctx: u64, target: u32, idc: u16) -> Option<(i64, u32)> {
        use hdc_core::fixed::Q;
        let mut last: HashMap<(i64, i64), i64> = HashMap::new();
        let mut marg: HashMap<i64, u64> = HashMap::new();
        let (mut hits, mut repeats, mut total) = (0u64, 0u64, 0u64);
        for e in self.store.iter() {
            if e.action != action || e.context != ctx || e.roles.len() < 2 {
                continue;
            }
            let (Some(a), Some(b), Some(o)) = (e.roles[0].get(idc), e.roles[1].get(idc), e.outcome(target)) else { continue };
            *marg.entry(o).or_insert(0) += 1;
            total += 1;
            if let Some(prev) = last.insert((a, b), o) {
                repeats += 1;
                hits += (prev == o) as u64;
            }
        }
        if repeats == 0 || total == 0 {
            return None;
        }
        let rate = hits as i64 * Q / repeats as i64;
        let chance: i64 = marg.values().map(|&c| (c * c) as i64 * Q / (total * total) as i64).sum();
        // significance: excess repeats over chance must exceed 3 standard deviations of
        // Binomial(repeats, chance) (z > 3, as in D046); otherwise no evidence of pair dependence
        let d = hits as i128 * Q as i128 - repeats as i128 * chance as i128;
        let var = repeats as i128 * chance as i128 * (Q - chance) as i128;
        if d <= 0 || d * d <= 9 * var {
            return Some((0, repeats as u32));
        }
        Some((rate - chance, repeats as u32))
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
                    // D051a (replaces D038a and D051's selection): every target of the slot gets
                    // its own latent hypothesis; the licensing gates decide which partition, if
                    // any, explains anything. Selecting one target needs repeated pairs that an
                    // epistemic agent avoids, and a wrong choice silences the slot (K3).
                    let slot_targets: Vec<u32> = ep.outcomes.iter().map(|o| o.0).filter(|&t2| t2 / 10_000 == t / 10_000 && t2 % 1000 == t % 1000).collect();
                    for t2 in slot_targets {
                        self.enable_latent(ep.action, t2, idc);
                    }
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
            self.refine_ex(l, ctx, t, true);
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
                let key = key_fp(law.condition_fp, law.target);
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

    /// D047: condition vector from feature kinds. Single features bind directly; in a conjunction
    /// the features are put in canonical (sorted) order and the k-th is permuted by k, so factors
    /// shared by two features (e.g. the `same` tag) cannot cancel under XOR.
    fn cond_hv_kinds(&mut self, action: u16, kinds: &[FeatureKind]) -> H {
        let mut ks: Vec<FeatureKind> = kinds.to_vec();
        ks.sort();
        let mut h = self.cb.action(action);
        if ks.len() == 1 {
            h.bind_assign(&self.cb.feature_hv(&ks[0]));
        } else {
            for (pos, k) in ks.iter().enumerate() {
                h.bind_assign(&self.cb.feature_hv(k).permute(pos as i64 + 1));
            }
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
        self.laws.push(RelationLaw {
            id,
            target,
            action,
            condition: cond,
            condition_fp: cond_hv.fingerprint(),
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
        // D047: positional permutation in canonical order for conjunctions
        let first: Vec<H> = feats.iter().map(|f| base_hv.bind(&f.hv.permute(1))).collect();
        let second: Vec<H> = feats.iter().map(|f| f.hv.permute(2)).collect();
        let mut pair_fp = Vec::with_capacity(feats.len() * feats.len() / 2);
        for i in 0..feats.len() {
            for j in (i + 1)..feats.len() {
                let (lo, hi) = if feats[i].kind <= feats[j].kind { (i, j) } else { (j, i) };
                pair_fp.push((i, j, first[lo].fingerprint_xor(&second[hi])));
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
            if e.total() > 0 && within_noise(e.counters(), e.total(), &self.policy_for(l.target)) && !e.status.terminal() {
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

    /// D053: the condition hypervector of law `l`, recomputed from its symbolic condition.
    pub fn condition_hv(&mut self, l: usize) -> H {
        let (a, cond) = (self.laws[l].action, self.laws[l].condition.clone());
        self.cond_hv_kinds(a, &cond)
    }

    /// D053: the relational transform code of law `l` (zero for absolute laws), on demand.
    pub fn transformation_hv(&mut self, l: usize) -> H {
        let cond = self.laws[l].condition.clone();
        self.transform_code(&cond)
    }

    /// D053: target bound with the majority outcome code of law `l` in context `c`, on demand.
    pub fn predicted_effect_hv(&mut self, l: usize, c: u64) -> Option<H> {
        let tgt = self.laws[l].target;
        let v = self.laws[l].ctx(c)?.majority()?.0;
        Some(self.cb.target(tgt).bind(&self.cb.outcome(tgt, v)))
    }

    /// K2 measurement (read-only): estimated retained bytes per structure, largest first.
    /// D056 (K5): effective hypothesis family size for (action, target, context): the
    /// multiple-comparison count every licence about that target in that context is charged.
    pub fn family_size(&self, action: u16, target: u32, ctx: u64) -> u64 {
        self.family.get(&(action, target, ctx)).copied().unwrap_or(0)
    }

    /// Diagnostic (read-only, v0.4 C3/E3 provenance):
    /// laws as in v0.4-dev, plus the deferred value entries on channel `ch` and the counters M.
    pub fn diag_target(&self, action: u16, target: u32, ctx: u64, ch: Option<u16>) -> Vec<String> {
        let ch_of = |f: &FeatureKind| match *f {
            FeatureKind::Abs { ch, .. } | FeatureKind::Same { ch, .. } | FeatureKind::Diff { ch, .. } | FeatureKind::Order { ch, .. } | FeatureKind::Delta { ch, .. } => ch,
        };
        let mut v: Vec<String> = Vec::new();
        for l in self.laws.iter().filter(|l| l.action == action && l.target == target) {
            if let Some(c) = ch {
                if !l.condition.iter().any(|f| ch_of(f) == c) && !l.condition.is_empty() {
                    continue;
                }
            }
            v.push(format!("LAW {} | pruned {} | children {}", self.summary(l.id, ctx), l.pruned, l.lineage.children.len()));
        }
        let rt = self.refine_tested.get(&(target, ctx)).copied().unwrap_or(0);
        if let Some(t) = self.deferred.get(&(action, target, ctx)) {
            v.push(format!("DEFERRED table: tested M={} entries {} untracked {} | refine_tested {}", t.tested, t.entries.len(), t.untracked, rt));
            let gate = log2_q16(t.tested.max(2)) + GATE_MARGIN_BITS * hdc_core::fixed::Q;
            let mut es: Vec<(&FeatureKind, &DeferredValue)> = t.entries.iter().filter(|(k, _)| ch.map(|c| ch_of(k) == c).unwrap_or(true)).collect();
            es.sort_by(|a, b| a.0.cmp(b.0));
            for (k, d) in es {
                v.push(format!("DEFERRED {:?} n {} hist {:?} pooled {} info {:.2} bits (gate {:.2})", k, d.total(), d.hist, d.pooled, (d.ll_model - d.ll_base) as f64 / 65536.0, gate as f64 / 65536.0));
            }
        } else {
            v.push(format!("DEFERRED table: none | refine_tested {}", rt));
        }
        v
    }

    /// v0.3 K2′ diagnosis (read-only): laws and their allocated bytes by lifecycle state,
    /// condition type, channel origin, channel cardinality class and target outcome cardinality.
    /// Returns (category, laws, bytes), largest bytes first.
    pub fn k2_breakdown(&self) -> Vec<(String, u64, u64)> {
        // distinct values per channel, from the stored episodes
        let mut card: HashMap<u16, std::collections::HashSet<i64>> = HashMap::new();
        for e in self.store.iter() {
            for r in &e.roles {
                for f in &r.fillers {
                    let s = card.entry(f.ch).or_default();
                    if s.len() < 1000 {
                        s.insert(f.val);
                    }
                }
            }
        }
        let class = |n: usize| if n <= 8 { "<=8" } else if n <= 64 { "9-64" } else { ">64" };
        let latent_hi = 3000 + 1000 * self.latent.len() as u16;
        let origin = |ch: u16| {
            if Some(ch) == self.identity_channel {
                "identity"
            } else if ch >= 3000 && ch < latent_hi {
                "latent"
            } else {
                "sensed"
            }
        };
        let ch_of = |f: &FeatureKind| match *f {
            FeatureKind::Abs { ch, .. } | FeatureKind::Same { ch, .. } | FeatureKind::Diff { ch, .. } | FeatureKind::Order { ch, .. } | FeatureKind::Delta { ch, .. } => ch,
        };
        let mut acc: std::collections::BTreeMap<String, (u64, u64)> = std::collections::BTreeMap::new();
        let mut add = |k: String, b: u64| {
            let e = acc.entry(k).or_insert((0, 0));
            e.0 += 1;
            e.1 += b;
        };
        for l in &self.laws {
            let mut bytes = (std::mem::size_of::<RelationLaw>() + l.condition.capacity() * std::mem::size_of::<FeatureKind>() + l.ctx.capacity() * std::mem::size_of::<CtxEvidence>()) as u64;
            for e in &l.ctx {
                bytes += (e.bins.capacity() * std::mem::size_of::<OutcomeBin>() + e.history.capacity() * 16 + (e.transfer.trials.capacity() + e.scope_trials.trials.capacity()) * std::mem::size_of::<TransferTrial>()) as u64;
                for b in &e.bins {
                    bytes += b.signatures.bytes() + b.bindings.bytes() + b.rsits.bytes() + (b.episodes.capacity() * 8) as u64;
                }
            }
            let status = if l.pruned {
                "pruned".to_string()
            } else {
                let best = l.ctx.iter().map(|e| e.status).max().unwrap_or(Status::Candidate);
                format!("{best:?}")
            };
            add(format!("status {status}"), bytes);
            let ctype = match l.condition.len() {
                0 => "base".to_string(),
                1 => {
                    let f = &l.condition[0];
                    let ch = ch_of(f);
                    let kind = if f.is_relational() { "relational" } else { "absolute" };
                    format!("{kind} {} card {}", origin(ch), class(card.get(&ch).map(|s| s.len()).unwrap_or(0)))
                }
                _ => {
                    let lat = l.condition.iter().any(|f| origin(ch_of(f)) == "latent");
                    let hi = l.condition.iter().any(|f| !f.is_relational() && card.get(&ch_of(f)).map(|s| s.len() > 64).unwrap_or(false));
                    format!("conjunction{}{}", if lat { " with latent" } else { "" }, if hi { " with >64-card absolute" } else { "" })
                }
            };
            add(format!("cond {ctype}"), bytes);
            let oc = self.alphabet_size(l.target).saturating_sub(1) as usize;
            add(format!("target outcomes {}", class(oc)), bytes);
            add(format!("target id {}", l.target), bytes);
        }
        let mut v: Vec<(String, u64, u64)> = acc.into_iter().map(|(k, (n, b))| (k, n, b)).collect();
        let feat_entries: u64 = self.feat_kinds.iter().map(|v| (v.capacity() * std::mem::size_of::<FeatureKind>()) as u64).sum();
        v.push(("store: per-episode feature lists".into(), self.feat_kinds.len() as u64, feat_entries));
        let store_bytes: u64 = self.store.iter().map(|e| (std::mem::size_of::<Episode>() + e.roles.iter().map(|r| r.fillers.capacity() * 16 + 48).sum::<usize>() + e.outcomes.capacity() * 16) as u64).sum();
        v.push(("store: episodes".into(), self.store.len() as u64, store_bytes));
        v.push(("index entries".into(), self.index.len() as u64, self.index.len() as u64 * 32));
        v.sort_by(|a, b| b.2.cmp(&a.2));
        v
    }

    pub fn memory_report(&self) -> Vec<(&'static str, u64, u64)> {
        let laws = self.laws.len() as u64;
        let (mut ctxs, mut bin_eps, mut sigs, mut binds, mut rsits, mut trials) = (0u64, 0u64, 0u64, 0u64, 0u64, 0u64);
        for l in &self.laws {
            ctxs += l.ctx.len() as u64;
            for e in &l.ctx {
                trials += (e.transfer.trials.len() + e.scope_trials.trials.len()) as u64;
                for b in &e.bins {
                    bin_eps += b.episodes.len() as u64;
                    sigs += b.signatures.len() as u64;
                    binds += b.bindings.len() as u64;
                    rsits += b.rsits.len() as u64;
                }
            }
        }
        // capacity-based accounting (allocated, not just used)
        let (mut cap_sets, mut cap_eps, mut cap_hist, mut cap_trials, mut cap_bins, mut cap_cond, mut cap_ctx) = (0u64, 0u64, 0u64, 0u64, 0u64, 0u64, 0u64);
        for l in &self.laws {
            cap_cond += (l.condition.capacity() * std::mem::size_of::<FeatureKind>() + (l.lineage.children.capacity() + l.competing.capacity()) * 8 + l.active_ctx.capacity() * 8) as u64;
            cap_ctx += (l.ctx.capacity() * std::mem::size_of::<CtxEvidence>()) as u64;
            for e in &l.ctx {
                cap_hist += (e.history.capacity() * 16) as u64;
                cap_trials += ((e.transfer.trials.capacity() + e.scope_trials.trials.capacity()) * std::mem::size_of::<TransferTrial>()) as u64;
                cap_bins += (e.bins.capacity() * std::mem::size_of::<OutcomeBin>()) as u64;
                for b in &e.bins {
                    cap_sets += b.signatures.bytes() + b.bindings.bytes() + b.rsits.bytes();
                    cap_eps += (b.episodes.capacity() * 8) as u64;
                }
            }
        }
        let feat_entries: u64 = self.feat_kinds.iter().map(|v| v.len() as u64).sum();
        let store_fillers: u64 = self.store.iter().map(|e| e.roles.iter().map(|r| r.fillers.len() as u64).sum::<u64>()).sum();
        let mut v = vec![
            ("laws (headers incl. inline fields, condition lists)", laws, laws * (std::mem::size_of::<RelationLaw>() as u64 + 48)),
            ("per-context evidence records", ctxs, ctxs * std::mem::size_of::<CtxEvidence>() as u64),
            ("bin episode lists", bin_eps, bin_eps * 8),
            ("situation-key sets", sigs, sigs * 16),
            ("binding-key sets", binds, binds * 16),
            ("relevant-situation sets", rsits, rsits * 16),
            ("transfer trial records", trials, trials * 24),
            ("per-episode feature lists", feat_entries, feat_entries * std::mem::size_of::<FeatureKind>() as u64),
            ("stored episodes (fillers)", self.store.len() as u64, store_fillers * 16 + self.store.len() as u64 * 128),
            ("law index", self.index.len() as u64, self.index.len() as u64 * 24),
        ];
        v.push(("[alloc] key sets (capacity)", 0, cap_sets));
        v.push(("[alloc] bin episode lists (capacity)", 0, cap_eps));
        v.push(("[alloc] status histories", 0, cap_hist));
        v.push(("[alloc] transfer trial lists", 0, cap_trials));
        v.push(("[alloc] outcome bins", 0, cap_bins));
        v.push(("[alloc] per-context records", 0, cap_ctx));
        v.push(("[alloc] conditions, lineage, competitors", 0, cap_cond));
        v.sort_by(|a, b| b.2.cmp(&a.2));
        v
    }

    /// D049: declare that a sensor reports `ch` on an ordinal scale.
    pub fn declare_ordinal(&mut self, ch: u16) {
        self.ordinal.insert(ch);
    }

    /// D049: channels on which order / offset transforms are defined now: declared ordinal, not
    /// vetoed as identifiers, and never a channel the learner labels itself (identity, latent).
    pub fn allowed_ordinal(&self) -> std::collections::BTreeSet<u16> {
        let latent_hi = 3000 + 1000 * self.latent.len() as u16;
        self.ordinal
            .iter()
            .copied()
            .filter(|c| !self.vetoed.contains(c) && Some(*c) != self.identity_channel && !(*c >= 3000 && *c < latent_hi.max(3000)))
            .collect()
    }

    /// D049 identifier veto: track which bound objects carry each value of a declared-ordinal
    /// channel; a channel whose values are injective over >= 8 objects names objects.
    fn track_ordinal(&mut self, ep: &Episode) {
        if self.ordinal.is_empty() {
            return;
        }
        for (r, e) in ep.roles.iter().enumerate() {
            let b = ep.role_binding(r);
            for f in &e.fillers {
                if !self.ordinal.contains(&f.ch) || self.vetoed.contains(&f.ch) {
                    continue;
                }
                let m = self.ord_seen.entry(f.ch).or_default();
                if m.len() >= 4096 && !m.contains_key(&f.val) {
                    continue;
                }
                let slot = m.entry(f.val).or_insert(Some(b));
                if *slot != Some(b) {
                    *slot = None;
                }
            }
        }
        let chans: Vec<u16> = self.ord_seen.keys().copied().collect();
        for c in chans {
            let m = &self.ord_seen[&c];
            if m.len() >= 8 && m.values().all(|v| v.is_some()) {
                let objs: std::collections::HashSet<u64> = m.values().flatten().copied().collect();
                if objs.len() >= 8 {
                    self.vetoed.insert(c);
                }
            }
        }
    }

    /// D050: the roles whose bound objects a law connects in an episode: the action's arguments
    /// and every role its condition mentions.
    pub fn relevant_roles(&self, l: usize, ep: &Episode) -> Vec<u8> {
        let mut r = ep.arg_roles();
        r.extend(self.laws[l].cond_roles());
        r.sort_unstable();
        r.dedup();
        r
    }

    /// D050: relevant-binding key of law `l` in an episode.
    pub fn bkey(&self, l: usize, ep: &Episode) -> u64 {
        ep.binding_key(&self.relevant_roles(l, ep))
    }

    /// D050a: relevant-situation key of law `l` in an episode.
    pub fn rkey(&self, l: usize, ep: &Episode) -> u64 {
        ep.situation_key(&self.relevant_roles(l, ep))
    }

    /// D050: a particular law (one relevant binding) speaks only about its own objects.
    pub fn binds(&self, l: usize, c: u64, ep: &Episode) -> bool {
        match self.laws[l].ctx(c).and_then(|e| e.particular()) {
            Some(b) => self.bkey(l, ep) == b,
            None => true,
        }
    }

    /// D055a: log2 (Q16) of the baseline probability of outcome `o` in episode `ep` for `target`:
    /// a mixture of the best copy source (an input filler that usually equals the outcome, >= 20
    /// cases, at its measured rate q) and the Laplace-smoothed distribution `dist`.
    fn baseline_log2(&self, action: u16, target: u32, ctx: u64, ep: &Episode, o: i64, dist: &[(i64, u64)], alpha: u64) -> i64 {
        use hdc_core::fixed::Q;
        let big_n: u64 = dist.iter().map(|x| x.1).sum::<u64>() + alpha.max(1);
        let c = dist.iter().find(|x| x.0 == o).map(|x| x.1).unwrap_or(0) + 1;
        // amendment 5: copying an input is a chance model only for high-cardinality outcomes
        let high_card = alpha > 65;
        let (q, echo_val) = match self.echo.get(&(action, target, ctx)) {
            Some(st) if st.total >= 20 && high_card => {
                let best = st.hits.iter().max_by(|a, b| a.1.cmp(b.1).then(b.0.cmp(a.0)));
                match best {
                    Some((&(r, ch), &h)) => (h as u64 * Q as u64 / st.total as u64, ep.roles.get(r as usize).and_then(|e| e.get(ch))),
                    None => (0, None),
                }
            }
            _ => (0, None),
        };
        let qq = q as u128;
        let num = qq * big_n as u128 * (echo_val == Some(o)) as u128 + (Q as u128 - qq) * c as u128;
        let den = Q as u128 * big_n as u128;
        let mix = log2_q16(num.max(1) as u64) - log2_q16(den as u64);
        let plain = log2_q16(c) - log2_q16(big_n);
        // amendment 3: the better chance model per case (the gate can only become stricter)
        mix.max(plain)
    }

    /// D055a: update the copy-model statistics with one case (after it was scored).
    fn echo_update(&mut self, action: u16, target: u32, ctx: u64, ep: &Episode, actual: i64) {
        let st = self.echo.entry((action, target, ctx)).or_default();
        st.total += 1;
        for (r, e) in ep.roles.iter().enumerate() {
            for f in &e.fillers {
                if f.val == actual {
                    *st.hits.entry((r as u8, f.ch)).or_insert(0) += 1;
                }
            }
        }
    }

    /// D055/D055a: record one case of a feature value without a law; true if the value has now
    /// earned a hypothesis: LLR = sum h log2(h/n) - sum_cases log2 p_baseline >= log2(M) + margin,
    /// with >= GATE_MIN_CASES cases.
    #[allow(clippy::too_many_arguments)]
    fn defer_and_test(&mut self, action: u16, target: u32, ctx: u64, base: usize, kind: &FeatureKind, actual: i64, ep: &Episode, eid: u64) -> bool {
        let base_counts: Vec<(i64, u64)> = self.laws[base].ctx(ctx).map(|e| e.bins.iter().map(|b| (b.val, b.count as u64)).collect()).unwrap_or_default();
        let alpha = self.alphabet_size(target);
        let lb = self.baseline_log2(action, target, ctx, ep, actual, &base_counts, alpha);
        let table = self.deferred.entry((action, target, ctx)).or_default();
        if !table.entries.contains_key(kind) {
            let fam = family_of(kind);
            let fv = table.family_values.entry(fam).or_insert(0);
            if *fv >= VALUES_PER_FAMILY {
                table.untracked += 1;
                // D056: an untracked case is counted as an examined value (an upper bound)
                *self.family.entry((action, target, ctx)).or_insert(0) += 1;
                return false;
            }
            *fv += 1;
            table.tested += 1;
            *self.family.entry((action, target, ctx)).or_insert(0) += 1;
            table.entries.insert(kind.clone(), DeferredValue::default());
        }
        let m = table.tested;
        let e = table.entries.get_mut(kind).expect("entry");
        // D057: a prospective transfer trial (prediction fixed from the previous cases only)
        let mut roles = ep.arg_roles();
        roles.extend(feature_roles(kind));
        roles.sort_unstable();
        roles.dedup();
        let (rk, bk) = (ep.situation_key(&roles), ep.binding_key(&roles));
        let deterministic = e.pooled == 0 && e.hist.len() == 1;
        if deterministic && e.total() >= GATE_MIN_CASES && e.binds.len() >= 3 && !e.rsits.sat && !e.rsits.contains(&rk) {
            let pred = e.hist[0].0;
            if pred == actual {
                e.t_ok += 1;
            } else {
                e.t_fail += 1;
                if e.t_failed.len() < DEFERRED_FAILED_CAP {
                    e.t_failed.push((eid, pred, actual));
                }
            }
        }
        // amendment 4: predict this case from the entry's previous cases (Laplace), then count it
        let prev = e.hist.iter().find(|x| x.0 == actual).map(|x| x.1).unwrap_or(0) as u64;
        let lm = log2_q16(prev + 1) - log2_q16(e.total() as u64 + alpha.max(1));
        e.ll_model += lm;
        if ep.kind == Kind::Intervention {
            e.info_int += lm - lb;
        }
        if let Some(x) = e.hist.iter_mut().find(|x| x.0 == actual) {
            x.1 += 1;
        } else if e.hist.len() < OUTCOMES_PER_VALUE {
            e.hist.push((actual, 1));
        } else {
            e.pooled += 1;
        }
        e.ll_base += lb;
        // D057: novelty bookkeeping only while the value is deterministic
        if e.pooled == 0 && e.hist.len() == 1 {
            e.rsits.put(rk);
            if e.binds.len() < 3 && !e.binds.contains(&bk) {
                e.binds.push(bk);
            }
        } else if !e.rsits.sat {
            e.rsits.compact();
            e.binds = Vec::new();
        }
        if e.total() < GATE_MIN_CASES {
            return false;
        }
        if e.ll_model - e.ll_base >= log2_q16(m.max(2)) + GATE_MARGIN_BITS * hdc_core::fixed::Q {
            let info = (e.ll_model - e.ll_base, e.info_int);
            let tr = (e.t_ok, e.t_fail, std::mem::take(&mut e.t_failed));
            table.entries.remove(kind);
            self.pending_utility = Some(info);
            self.pending_transfer = Some(tr);
            return true;
        }
        false
    }

    /// D055: create the law for `kind` and initialize it from the stored episodes in which the
    /// feature occurred (excluding `exclude`, which is counted by the caller). As in D014, replay
    /// gives counts, keys and counterexamples, never transfer credit or utility.
    #[allow(clippy::too_many_arguments)]
    fn materialize(&mut self, action: u16, target: u32, ctx: u64, fp: u64, hv: &H, kind: FeatureKind, exclude: u64) -> usize {
        let l = self.get_or_create_cached(action, target, fp, hv, vec![kind.clone()], exclude);
        self.materialized += 1;
        let ids = self.inv.get(&kind).cloned().unwrap_or_default();
        let t = self.tick;
        for id in ids {
            if id == exclude {
                continue;
            }
            let ep = self.store.get(id);
            if ep.context != ctx || ep.action != action {
                continue;
            }
            let Some(out) = ep.outcome(target) else { continue };
            let (sig, iv) = (ep.signature(), ep.kind == Kind::Intervention);
            let (bk, rk) = (self.bkey(l, ep), self.rkey(l, ep));
            if self.laws[l].ctx(ctx).map(|e| e.total() == 0).unwrap_or(true) {
                *self.per_target_ctx.entry((target, ctx)).or_insert(0) += 1;
            }
            self.laws[l].ctx_mut(ctx, t).add(out, id, bk, rk, sig, iv, t);
        }
        // amendment 6: the gate's prequential information is this law's utility so far
        if let Some((u, ui)) = self.pending_utility.take() {
            let e = self.laws[l].ctx_mut(ctx, t);
            e.utility_q16 += u;
            e.utility_int_q16 += ui;
        }
        // D057: prospective trials made while deferred (pre-registered then, not credited now)
        if let Some((ok, fail, failed)) = self.pending_transfer.take() {
            let e = self.laws[l].ctx_mut(ctx, t);
            e.transfer.ok += ok;
            let kept = failed.len() as u32;
            for (episode, predicted, actual) in failed {
                e.transfer.record(episode, predicted, actual);
            }
            e.transfer.fail += fail - kept;
        }
        l
    }

    /// D055: how many of a query's features are held only as deferred evidence for `target`
    /// (an abstention on them is "deferred: insufficient evidence").
    pub fn deferred_features(&mut self, ep: &Episode, target: u32) -> u32 {
        let mut aug = ep.clone();
        self.augment(&mut aug);
        let ord = self.allowed_ordinal();
        let feats = self.cb.features(&aug, &ord);
        let Some(table) = self.deferred.get(&(aug.action, target, aug.context)) else { return 0 };
        feats.iter().filter(|f| table.entries.contains_key(&f.kind)).count() as u32
    }

    fn transfer_eligible(&self, law: usize, ctx: u64, ep: &Episode, sig: u64) -> bool {
        let l = &self.laws[law];
        let own = match l.ctx(ctx) {
            Some(e) if e.total() > 0 => {
                // D015a / D050: a held-out case is a relevant binding this law has never been
                // supported by (a new situation of its own binding for a particular law);
                // bystander novelty does not count
                !e.status.terminal()
                    && within_noise(e.counters(), e.total(), &self.policy_for(l.target))
                    && e.independent() >= 3
                    && self.binds(law, ctx, ep)
                    && e.new_case(self.rkey(law, ep), sig)
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
        *self.trials.entry((ep.action, ep.context)).or_insert(0) += 1;
        let mut ep = ep;
        self.augment(&mut ep);
        self.track_ordinal(&ep);
        let ord = self.allowed_ordinal();
        let feats = self.cb.features(&ep, &ord);
        let sig = ep.signature();
        let ctx = ep.context;
        let intervention = ep.kind == Kind::Intervention;
        let outcomes = ep.outcomes.clone();
        let action = ep.action;
        let eid = self.store.append(ep);
        let epc = self.store.get(eid).clone();
        self.feat_kinds.push(feats.iter().map(|f| f.kind.clone()).collect());
        for f in &feats {
            let v = self.inv.entry(f.kind.clone()).or_default();
            if v.len() < 4096 {
                v.push(eid);
            }
        }

        let cc = self.cond_cache(action, &feats);
        for (target, actual) in outcomes {
            let before = self.laws.len();
            let base = self.get_or_create_cached(action, target, cc.base_fp, &cc.base_hv, Vec::new(), eid);
            // D055: a single-feature law exists only if its value has earned it; otherwise the
            // case goes to the deferred value table, and the law is materialized (by replay) when
            // the value's outcome counts pass the gate
            let mut single_of: Vec<Option<usize>> = Vec::with_capacity(feats.len());
            let mut fresh: Vec<usize> = Vec::new();
            for (i, f) in feats.iter().enumerate() {
                if let Some(&l) = self.index.get(&key_fp(cc.single_fp[i], target)) {
                    single_of.push(Some(l));
                    continue;
                }
                if self.defer_and_test(action, target, ctx, base, &f.kind, actual, &epc, eid) {
                    let l = self.materialize(action, target, ctx, cc.single_fp[i], &cc.single_hv[i], f.kind.clone(), eid);
                    fresh.push(l);
                    single_of.push(Some(l));
                } else {
                    single_of.push(None);
                }
            }
            let singles: Vec<usize> = single_of.iter().flatten().copied().collect();
            self.echo_update(action, target, ctx, &epc, actual);
            let mut pairs: Vec<(usize, usize, usize)> = Vec::new();
            for &(i, j, fp) in &cc.pair_fp {
                if let Some(&l) = self.index.get(&key_fp(fp, target)) {
                    if self.laws[l].active_in(ctx) && !self.laws[l].pruned {
                        pairs.push((l, single_of[i].unwrap_or(base), single_of[j].unwrap_or(base)));
                    }
                }
            }
            rep.new_laws += (self.laws.len() - before) as u32;

            // 1. pre-registration (outcome not yet read)
            let mut prereg: Vec<(usize, i64, bool)> = Vec::new();
            for &l in std::iter::once(&base).chain(singles.iter()).chain(pairs.iter().map(|p| &p.0)) {
                // D055: a law born from this very outcome makes no prediction on it
                if fresh.contains(&l) {
                    continue;
                }
                if self.transfer_eligible(l, ctx, &epc, sig) {
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
                        // D017a: only licensed knowledge subsumes (a provisional hypothesis that
                        // never earns its licence must not block others)
                        Some(e) => (e.total(), e.counters() == 0 && e.status == Status::Licensed),
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
            // D042b withdrawn: base rates and parents are statistical knowledge even when not
            // licensed as deterministic laws; a law must beat them (D012)
            let base_known = base_loss;
            let dbg: Option<usize> = std::env::var("BM_DEBUG_LAW").ok().and_then(|x| x.parse().ok());
            for &s in &singles {
                let (_, ls, ts, _) = find(s);
                let gc = general_comp(s, ts);
                let d = incremental(s, ls, gc, base_known);
                if dbg == Some(s) && self.tick % 40 == 0 {
                    eprintln!("DBGLAW tick {} own {} base {} comp {:?} delta {} comp_desc {:?}", self.tick, ls, base_loss, gc, d, gc.1.map(|m| self.laws[m].describe()));
                }
                deltas.push((s, d));
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
                    // D056: a law examined here for the first time
                    *self.family.entry((action, target, ctx)).or_insert(0) += 1;
                }
                let (bk, rk) = (self.bkey(l, &epc), self.rkey(l, &epc));
                self.laws[l].ctx_mut(ctx, t).add(actual, eid, bk, rk, sig, intervention, t);
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
        // D054 (K2 stage 2, item 1): wake-time retirement of stale single-case candidates
        if self.tick % 1000 == 0 {
            self.retire_stale(2000);
        }
        rep
    }

    /// D054 (item 1b): a hypothesis conditioned on a latent channel version its inducer no longer
    /// emits can never match again (versions only grow). Its record is compacted (counts, status
    /// history and counterexamples kept) and it leaves the index and the selection count.
    pub fn retire_dead_latent(&mut self) -> u32 {
        if self.latent.is_empty() {
            return 0;
        }
        let hi = 3000 + 1000 * self.latent.len() as u16;
        let live: std::collections::HashSet<u16> = self.latent.iter().flat_map(|i| i.live_channels()).collect();
        let dead = |ch: u16| ch >= 3000 && ch < hi && !live.contains(&ch);
        // D055: deferred evidence and replay index of features on retired latent channels can
        // never be used again (those features never recur)
        for table in self.deferred.values_mut() {
            table.entries.retain(|k, _| {
                let ch = match *k {
                    FeatureKind::Abs { ch, .. } | FeatureKind::Same { ch, .. } | FeatureKind::Diff { ch, .. } | FeatureKind::Order { ch, .. } | FeatureKind::Delta { ch, .. } => ch,
                };
                !dead(ch)
            });
        }
        self.inv.retain(|k, _| {
            let ch = match *k {
                FeatureKind::Abs { ch, .. } | FeatureKind::Same { ch, .. } | FeatureKind::Diff { ch, .. } | FeatureKind::Order { ch, .. } | FeatureKind::Delta { ch, .. } => ch,
            };
            !dead(ch)
        });
        let mut n = 0;
        for l in 0..self.laws.len() {
            if self.laws[l].pruned {
                continue;
            }
            let uses_dead = self.laws[l].condition.iter().any(|f| match *f {
                FeatureKind::Abs { ch, .. } | FeatureKind::Same { ch, .. } | FeatureKind::Diff { ch, .. } | FeatureKind::Order { ch, .. } | FeatureKind::Delta { ch, .. } => dead(ch),
            });
            if !uses_dead {
                continue;
            }
            let key = key_fp(self.laws[l].condition_fp, self.laws[l].target);
            if self.index.get(&key) == Some(&l) {
                self.index.remove(&key);
            }
            let target = self.laws[l].target;
            let ctxs: Vec<u64> = self.laws[l].ctx.iter().filter(|e| e.total() > 0).map(|e| e.context).collect();
            for c in ctxs {
                if let Some(x) = self.per_target_ctx.get_mut(&(target, c)) {
                    *x = x.saturating_sub(1);
                }
            }
            for e in self.laws[l].ctx.iter_mut() {
                if !e.compacted {
                    e.compact();
                }
            }
            self.laws[l].pruned = true;
            n += 1;
        }
        n
    }

    /// D054: a never-licensed, non-base hypothesis that has seen at most one case in every
    /// context and is older than `min_age` ticks leaves the hypothesis space (D018 at wake time).
    /// Its one case stays in the episode store; a single-case record holds no counterexample.
    /// If the hypothesis is generated again it starts afresh.
    pub fn retire_stale(&mut self, min_age: u64) -> u32 {
        let now = self.tick;
        let mut n = self.retire_dead_latent();
        for l in 0..self.laws.len() {
            let law = &self.laws[l];
            if law.pruned || law.is_base() || now.saturating_sub(law.lineage.created_t) <= min_age {
                continue;
            }
            if law.ctx.iter().any(|e| e.status == Status::Licensed || e.total() > 1) {
                continue;
            }
            let key = key_fp(law.condition_fp, law.target);
            let ctxs: Vec<u64> = law.ctx.iter().filter(|e| e.total() > 0).map(|e| e.context).collect();
            let target = law.target;
            if self.index.get(&key) == Some(&l) {
                self.index.remove(&key);
            }
            for c in ctxs {
                if let Some(x) = self.per_target_ctx.get_mut(&(target, c)) {
                    *x = x.saturating_sub(1);
                }
            }
            self.laws[l].pruned = true;
            self.laws[l].ctx = Vec::new();
            self.laws[l].competing = Vec::new();
            n += 1;
        }
        n
    }

    /// Hidden-condition search: rank co-present features by entropy reduction among the
    /// parent's episodes, create the best as child candidates initialised by replay.
    fn refine(&mut self, parent: usize, ctx: u64, target: u32) {
        self.refine_ex(parent, ctx, target, false)
    }

    /// D048: with `split`, the parent's episodes are divided by situation signature into a
    /// selection half and a held-out half. Candidate conditions are ranked and the child is
    /// initialised on the selection half only; on the held-out half the child pre-registers its
    /// prediction before each outcome is counted, and situations it has never counted are
    /// held-out transfer trials (a lookup table fitted on the selection half could not answer
    /// them). Used in sleep, where the world may offer no new situations any more.
    fn refine_ex(&mut self, parent: usize, ctx: u64, target: u32, split: bool) {
        let all = match self.laws[parent].ctx(ctx) {
            Some(e) => e.all_ids(),
            None => return,
        };
        let held_of = |s: &Self, id: u64| -> bool { split && mix64(s.store.get(id).signature(), 0xD048) & 1 == 1 };
        let ids: Vec<u64> = all.iter().copied().filter(|&id| !held_of(self, id)).collect();
        let held: Vec<u64> = all.iter().copied().filter(|&id| held_of(self, id)).collect();
        let pcond = self.laws[parent].condition.clone();
        let action = self.laws[parent].action;
        let mut tally: HashMap<FeatureKind, HashMap<i64, u64>> = HashMap::new();
        let mut parent_hist: HashMap<i64, u64> = HashMap::new();
        for &id in &ids {
            let Some(out) = self.store.get(id).outcome(target) else { continue };
            *parent_hist.entry(out).or_insert(0) += 1;
        }
        // D055a: per-case baseline (copy model + parent distribution) accumulated per candidate
        let parent_dist: Vec<(i64, u64)> = parent_hist.iter().map(|(&o, &c)| (o, c)).collect();
        let alpha_r = self.alphabet_size(target);
        let mut ll_base: HashMap<FeatureKind, i64> = HashMap::new();
        let mut ll_model: HashMap<FeatureKind, i64> = HashMap::new();
        let mut sorted_ids = ids.clone();
        sorted_ids.sort_unstable();
        for &id in &sorted_ids {
            let ep = self.store.get(id);
            let Some(out) = ep.outcome(target) else { continue };
            let lb = self.baseline_log2(action, target, ctx, ep, out, &parent_dist, alpha_r);
            for k in &self.feat_kinds[id as usize] {
                if !pcond.contains(k) {
                    let h = tally.entry(k.clone()).or_default();
                    // amendment 4: prequential score of the candidate before counting this case
                    let n: u64 = h.values().sum();
                    let prev = h.get(&out).copied().unwrap_or(0);
                    *ll_model.entry(k.clone()).or_insert(0) += log2_q16(prev + 1) - log2_q16(n + alpha_r.max(1));
                    *h.entry(out).or_insert(0) += 1;
                    *ll_base.entry(k.clone()).or_insert(0) += lb;
                }
            }
        }
        let ph: Vec<u64> = parent_hist.values().copied().collect();
        let h_parent = hdc_core::fixed::entropy_q16(&ph);
        // D055: every candidate feature examined counts toward the multiple-comparison budget of
        // this target; a child must carry LLR >= log2(M_r) + margin bits against its parent
        let examined = tally.len() as u64;
        // D056: every refinement candidate examined joins the target's family
        *self.family.entry((action, target, ctx)).or_insert(0) += examined;
        let m_r = {
            let c = self.refine_tested.entry((target, ctx)).or_insert(0);
            *c += examined;
            *c
        };
        let gate = log2_q16(m_r.max(2)) + GATE_MARGIN_BITS * hdc_core::fixed::Q;
        let mut scored: Vec<(i64, FeatureKind)> = tally
            .into_iter()
            .filter_map(|(k, h)| {
                let counts: Vec<u64> = h.values().copied().collect();
                let n: u64 = counts.iter().sum();
                let hk = hdc_core::fixed::entropy_q16(&counts);
                let llr = ll_model.get(&k).copied().unwrap_or(0) - ll_base.get(&k).copied().unwrap_or(0);
                if n >= GATE_MIN_CASES as u64 && hk < h_parent && llr >= gate {
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
            let h = self.cond_hv_kinds(action, &cond);
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
            // D054d: novelty during a replay is decided against exact, temporary key sets of the
            // situations replayed so far (the child's bounded sets may saturate during replay)
            let mut seen_r: std::collections::HashSet<u64> = std::collections::HashSet::new();
            let mut seen_s: std::collections::HashSet<u64> = std::collections::HashSet::new();
            // replay: counts and independence only; no utility, no transfer credit (D014)
            for &id in &ids {
                if !self.feat_kinds[id as usize].contains(&g) {
                    continue;
                }
                let ep = self.store.get(id);
                let Some(out) = ep.outcome(target) else { continue };
                let (sig, iv) = (ep.signature(), ep.kind == Kind::Intervention);
                let (bk, rk) = (self.bkey(child, ep), self.rkey(child, ep));
                if self.laws[child].ctx(ctx).map(|e| e.total() == 0).unwrap_or(true) {
                    *self.per_target_ctx.entry((target, ctx)).or_insert(0) += 1;
                }
                self.laws[child].ctx_mut(ctx, t).add(out, id, bk, rk, sig, iv, t);
                seen_r.insert(rk);
                seen_s.insert(sig);
            }
            // D048: held-out half, pre-registered predictions on never-counted situations
            for &id in &held {
                if !self.feat_kinds[id as usize].contains(&g) {
                    continue;
                }
                let ep = self.store.get(id);
                let Some(out) = ep.outcome(target) else { continue };
                let (sig, iv) = (ep.signature(), ep.kind == Kind::Intervention);
                let (bk, rk) = (self.bkey(child, ep), self.rkey(child, ep));
                let pol = self.policy_for(target);
                // prequential utility against the parent's final counts (which include this very
                // episode, so the comparison is biased towards the parent: conservative)
                let alpha = self.alphabet_size(target);
                let p_loss = self.laws[parent].ctx(ctx).map(|e| e.loss_q16(out, alpha));
                let c_loss = self.laws[child].ctx(ctx).filter(|e| e.total() > 0).map(|e| e.loss_q16(out, alpha));
                if let (Some(pl), Some(cl)) = (p_loss, c_loss) {
                    let e = self.laws[child].ctx_mut(ctx, t);
                    e.utility_q16 += pl - cl;
                    if iv {
                        e.utility_int_q16 += pl - cl;
                    }
                }
                if let Some(e) = self.laws[child].ctx(ctx) {
                    let new_sit = if e.particular().is_some() { !seen_s.contains(&sig) } else { !seen_r.contains(&rk) };
                    if new_sit && !e.status.terminal() && within_noise(e.counters(), e.total(), &pol) && e.independent() >= 3 {
                        if let Some((pred, _)) = e.majority() {
                            self.laws[child].ctx_mut(ctx, t).transfer.record(id, pred, out);
                        }
                    }
                }
                if self.laws[child].ctx(ctx).map(|e| e.total() == 0).unwrap_or(true) {
                    *self.per_target_ctx.entry((target, ctx)).or_insert(0) += 1;
                }
                self.laws[child].ctx_mut(ctx, t).add(out, id, bk, rk, sig, iv, t);
                seen_r.insert(rk);
                seen_s.insert(sig);
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
        // D056 (K5): the licence is a selection from every hypothesis examined for this target
        let n_cand = self.family.get(&(self.laws[l].action, self.laws[l].target, ctx)).copied().unwrap_or(1);
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
                // D044: rule of three on the exception rate
                let excess = (counters as u64).saturating_sub(allow);
                // D044a: reliability is measured on intervention trials (repeats of a situation are
                // valid samples of the exception rate); breadth stays with min_independent
                let bounded = (excess + 3) * p.max_exception_den as u64 <= e.interventions() as u64 * p.max_exception_num as u64;
                if bounded
                    && e.transfer.ok >= p.min_transfer_ok
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
        self.laws[l].ctx_mut(ctx, t).set_status(new, t);
        // D054: a terminal record is compacted (counts and counterexamples kept)
        if new.terminal() {
            let e = self.laws[l].ctx_mut(ctx, t);
            if !e.compacted {
                e.compact();
            }
        }
    }

    fn matching(&mut self, ep: &Episode, target: u32) -> Vec<usize> {
        let ord = self.allowed_ordinal();
        let feats = self.cb.features(ep, &ord);
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
        let licensed = m.iter().any(|&l| self.laws[l].applicable(ctx) && self.binds(l, ctx, &aug));
        let mut best: Option<(i64, Vec<u64>)> = None;
        let mut near = 0u32;
        let sig = aug.signature();
        let mut licence_k: Vec<u32> = Vec::new();
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
            // D043d: independent supports of each consistent, unlicensed hypothesis this query
            // would extend with a new combination
            if ep.kind == Kind::Intervention
                && e.status != Status::Licensed
                && e.counters() == 0
                && !e.bins.iter().any(|b| b.signatures.contains(&sig))
            {
                licence_k.push(e.independent());
            }
            // D043a: only an intervention can supply what an unlicensed hypothesis lacks
            if ep.kind == Kind::Intervention
                && e.status != Status::Licensed
                && within_noise(e.counters(), e.total(), &self.policy)
                && e.independent() >= 3
            {
                near += 1;
            }
        }
        // D045: no hypothesis has data for this target: the action's trials in this context are
        // evidence that nothing observable varied (status quo), untried actions stay uncertain
        let counts = best.map(|b| b.1).unwrap_or_else(|| {
            let mut c = vec![0u64; alpha.len().max(1) + 1];
            c[0] = *self.trials.get(&(ep.action, ctx)).unwrap_or(&0) as u64;
            c
        });
        Uncertainty { licensed, counts, near_licence: near, licence_k }
    }

    /// Ids of every active (non-pruned) hypothesis matching a query for a target.
    /// Diagnostic: matching hypotheses with their evidence in the query's context.
    pub fn debug_matching(&mut self, ep: &Episode, target: u32) -> Vec<String> {
        let ctx = ep.context;
        let m = self.matching_ids(ep, target);
        m.into_iter()
            .filter_map(|l| {
                let law = &self.laws[l];
                let e = law.ctx(ctx)?;
                Some(format!(
                    "{} [{:?}] tot {} ctr {} indep {} int {} xfer {}/{} util {} active {}",
                    law.describe(),
                    e.status,
                    e.total(),
                    e.counters(),
                    e.independent(),
                    e.interventions(),
                    e.transfer.ok,
                    e.transfer.fail,
                    e.utility_q16 / 65536,
                    law.active_ctx.contains(&ctx)
                ))
            })
            .collect()
    }

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
            .filter(|&l| self.laws[l].applicable(ctx) && self.binds(l, ctx, ep))
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
                    self.laws[l].ctx.iter().any(|e| e.status == Status::Licensed && e.particular().map(|b| b == self.bkey(l, ep)).unwrap_or(true))
                } else {
                    self.laws[l].applicable(ctx) && self.binds(l, ctx, ep)
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
            _ => match self.resolve_conflict(&by_val, ctx, target, ep) {
                Some(i) => {
                    let (v, laws, c) = by_val.swap_remove(i);
                    Answer::Value { val: v, laws, confidence_q16: c }
                }
                None => Answer::Abstain(Abstain::Conflict),
            },
        }
    }

    /// D041/D041a: settle a conflict between licensed laws by their record on the cases where both
    /// applied. For every pair of value groups, the overlap of their laws' episodes is scored; the
    /// group whose laws were right strictly more often (with >= 3 shared cases) wins every pairwise
    /// comparison, otherwise the conflict stands and the engine abstains.
    fn resolve_conflict(&self, groups: &[(i64, Vec<usize>, i64)], ctx: u64, target: u32, query: &Episode) -> Option<usize> {
        let mut wins = vec![0usize; groups.len()];
        for a in 0..groups.len() {
            for b in (a + 1)..groups.len() {
                let (indep, ra, rb) = self.shared_record(groups, a, b, ctx, target, query);
                // D041a: the shared cases must be independent (distinct situations); repeated
                // probes of one situation are one case
                if indep < MIN_SHARED_INDEPENDENT {
                    continue;
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

    /// Record of two value groups on the cases where both applied: (independent shared
    /// situations, situations where group a's value happened, situations where group b's did).
    fn shared_record(&self, groups: &[(i64, Vec<usize>, i64)], a: usize, b: usize, ctx: u64, target: u32, query: &Episode) -> (u32, u32, u32) {
        // D041b: per-law episode lists are capped (EP_CAP); when a list is truncated, shared cases
        // are found by testing each law's condition against the stored features of every episode
        // in the context, so late evidence is never invisible
        let truncated = |l: usize| self.laws[l].ctx(ctx).map(|e| (e.all_ids().len() as u32) < e.total()).unwrap_or(false);
        let applies = |l: usize, id: u64| -> bool {
            let law = &self.laws[l];
            let ep = self.store.get(id);
            ep.action == law.action && law.condition.iter().all(|k| self.feat_kinds[id as usize].contains(k))
        };
        let ids = |l: usize| -> std::collections::HashSet<u64> {
            self.laws[l].ctx(ctx).map(|e| e.all_ids().into_iter().collect()).unwrap_or_default()
        };
        let any_trunc = groups[a].1.iter().chain(groups[b].1.iter()).any(|&l| truncated(l));
        let shared: Vec<u64> = if any_trunc {
            (0..self.store.len() as u64)
                .filter(|&id| self.store.get(id).context == ctx && self.store.get(id).outcome(target).is_some())
                .filter(|&id| groups[a].1.iter().any(|&l| applies(l, id)) && groups[b].1.iter().any(|&l| applies(l, id)))
                .collect()
        } else {
            let ia: std::collections::HashSet<u64> = groups[a].1.iter().flat_map(|&l| ids(l)).collect();
            let ib: std::collections::HashSet<u64> = groups[b].1.iter().flat_map(|&l| ids(l)).collect();
            ia.intersection(&ib).copied().collect()
        };
        // D050: independence of shared cases is counted over the objects the two groups' laws
        // connect (union of their relevant roles). Repeats of one binding count once; only for the
        // query's own binding are distinct situations direct, separate evidence.
        let mut roles: Vec<u8> = groups[a].1.iter().chain(groups[b].1.iter()).flat_map(|&l| self.laws[l].cond_roles()).collect();
        roles.extend(query.arg_roles());
        roles.sort_unstable();
        roles.dedup();
        let qk = query.binding_key(&roles);
        let mut units: std::collections::HashSet<u64> = std::collections::HashSet::new();
        let mut sa: std::collections::HashSet<u64> = std::collections::HashSet::new();
        let mut sb: std::collections::HashSet<u64> = std::collections::HashSet::new();
        for e in shared {
            let ep = self.store.get(e);
            let bk = ep.binding_key(&roles);
            let unit = if bk == qk { hdc_core::rng::mix64(ep.signature(), 0xD050) } else { bk };
            units.insert(unit);
            match ep.outcome(target) {
                Some(o) if o == groups[a].0 => {
                    sa.insert(unit);
                }
                Some(o) if o == groups[b].0 => {
                    sb.insert(unit);
                }
                _ => {}
            }
        }
        (units.len() as u32, sa.len() as u32, sb.len() as u32)
    }

    /// Diagnostic and evaluation view of a conflict for a query: for every pair of disagreeing
    /// licensed value groups, (value a, value b, independent shared situations, right a, right b).
    /// Empty when the licensed laws agree or nothing is licensed.
    pub fn conflict_evidence(&mut self, ep: &Episode, target: u32) -> Vec<(i64, i64, u32, u32, u32)> {
        let ctx = ep.context;
        let mut aug = ep.clone();
        self.augment(&mut aug);
        let m = self.matching(&aug, target);
        let mut by_val: Vec<(i64, Vec<usize>, i64)> = Vec::new();
        for l in m.into_iter().filter(|&l| self.laws[l].applicable(ctx) && self.binds(l, ctx, &aug)) {
            let Some(v) = self.laws[l].ctx(ctx).and_then(|e| e.majority()).map(|m| m.0) else { continue };
            match by_val.iter_mut().find(|x| x.0 == v) {
                Some(x) => x.1.push(l),
                None => by_val.push((v, vec![l], 0)),
            }
        }
        let mut out = Vec::new();
        for a in 0..by_val.len() {
            for b in (a + 1)..by_val.len() {
                let (i, ra, rb) = self.shared_record(&by_val, a, b, ctx, target, &aug);
                out.push((by_val[a].0, by_val[b].0, i, ra, rb));
            }
        }
        out
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
