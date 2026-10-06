//! Symbol grounding: anonymous slot percepts -> property/state channels -> object concepts.
//!
//! * Channel classes are learned from within-event continuity: a channel that changes between
//!   pre and post of the same slot more often than the sensor-noise floor is a STATE channel,
//!   otherwise a PROPERTY channel. Identity uses property channels only.
//! * Object identity: HDC cleanup proposes candidates; role unbinding of the candidate prototype
//!   verifies channel by channel with integer counts.
//! * Concept birth, merge and split are accepted only when they shorten the description of the
//!   sightings (integer MDL in Q16 bits). One-off noise never becomes a concept.
//!
//! Class/instance concepts are infrastructure (ARCHITECTURE 2.3 outcome 1). They are not counted
//! as progress; they feed role fillers to the relational substrate.

use crate::event::*;
use hdc_core::fixed::log2_q16;
use hdc_core::*;
use std::collections::{BTreeMap, HashMap, HashSet};

type H = Hv16k;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChannelClass {
    Unknown,
    Property,
    State,
}

#[derive(Clone, Debug, Default)]
pub struct ChannelStats {
    pub changed_arg: u32,
    pub total_arg: u32,
    pub changed_other: u32,
    pub total_other: u32,
    pub values: HashSet<i64>,
    /// Marginal value frequencies (pre-scene tokens), for chance-recurrence tests (D028).
    pub freq: BTreeMap<i64, u32>,
    pub n: u32,
}

impl ChannelStats {
    fn pct_arg(&self) -> u32 {
        if self.total_arg == 0 {
            0
        } else {
            self.changed_arg * 100 / self.total_arg
        }
    }
    fn pct_other(&self) -> u32 {
        if self.total_other == 0 {
            0
        } else {
            self.changed_other * 100 / self.total_other
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConceptStatus {
    /// Proto-concept: recurring candidate, not yet worth its description cost.
    Proto,
    Concept,
    Merged(u32),
    Split,
    Decayed,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ConceptEvent {
    Created { t: u64, event: u64 },
    Born { t: u64, sightings: u32, saving_bits: i64 },
    Absorbed { from: u32, t: u64 },
    MergedInto { into: u32, t: u64, saving_bits: i64 },
    SplitInto { children: Vec<u32>, ch: u16, t: u64, saving_bits: i64 },
    SplitFrom { parent: u32, t: u64 },
    Retired { t: u64 },
}

#[derive(Clone, Debug)]
pub struct ObjectConcept {
    pub id: u32,
    pub status: ConceptStatus,
    /// Integer evidence per property channel: value histogram.
    pub hist: BTreeMap<u16, Vec<(i64, u32)>>,
    /// HDC prototype: majority of bound (channel, majority value) pairs.
    pub proto: H,
    pub sightings: u32,
    /// support = verified sightings, refute = channel disagreements seen.
    pub evidence: Evidence,
    /// Provenance: (event id, slot) of sightings (capped).
    pub exemplars: Vec<(u64, u16)>,
    pub first_t: u64,
    pub last_t: u64,
    pub lineage: Vec<ConceptEvent>,
    /// Global percept counter at creation (for chance-recurrence tests).
    pub created_percept: u64,
}

impl ObjectConcept {
    pub fn majority(&self, ch: u16) -> Option<(i64, u32, u32)> {
        let h = self.hist.get(&ch)?;
        let total: u32 = h.iter().map(|x| x.1).sum();
        h.iter().max_by_key(|x| (x.1, -x.0)).map(|x| (x.0, x.1, total))
    }

    pub fn is_active(&self) -> bool {
        matches!(self.status, ConceptStatus::Proto | ConceptStatus::Concept)
    }

    fn add_value(&mut self, ch: u16, v: i64) {
        let h = self.hist.entry(ch).or_default();
        match h.iter_mut().find(|x| x.0 == v) {
            Some(x) => x.1 += 1,
            None => h.push((v, 1)),
        }
    }
}

/// Per-channel observation model measured from data (D036): log2-likelihood (Q16) of a token
/// that matches the true value, that is a specific wrong value, and of an uninformative channel.
#[derive(Clone, Debug, Default)]
pub struct NoiseModel {
    pub ch: BTreeMap<u16, (i64, i64, i64)>,
}

impl NoiseModel {
    /// (log2 P(match), log2 P(specific mismatch), log2 P(uniform)) for a channel.
    pub fn terms(&self, c: u16) -> (i64, i64, i64) {
        self.ch.get(&c).copied().unwrap_or((-hdc_core::fixed::Q / 16, -8 * hdc_core::fixed::Q, -3 * hdc_core::fixed::Q))
    }
}

/// Posterior target for acting on an inferred value (completion): 98% (Q16).
pub const FILL_POSTERIOR_Q16: i64 = 64225;

/// Result of identity verification.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ident {
    Hit((u32, u32, u32)),
    /// Several concepts explain the percept equally well.
    Ambiguous,
    /// Nothing explains it.
    Unknown,
}

#[derive(Clone, Debug)]
pub struct GroundParams {
    /// Events used to learn channel classes before identity is attempted (developmental stage 0).
    pub warmup: usize,
    /// Property-channel disagreements tolerated when verifying identity.
    pub tol_mismatch: u32,
    /// Minimum matching channels for identity.
    pub min_match: u32,
    pub min_sightings: u32,
    pub pending_ttl: u64,
    pub top_k: usize,
    pub maintenance_every: u64,
    /// A merge requires the older concept to have been silent this long.
    pub merge_stale: u64,
    /// D035: born concepts with fewer sightings than this that stay unused this long retire.
    pub retire_young: u32,
    pub retire_after: u64,
}

impl Default for GroundParams {
    fn default() -> Self {
        GroundParams {
            warmup: 100,
            tol_mismatch: 1,
            min_match: 3,
            min_sightings: 3,
            pending_ttl: 400,
            top_k: 12,
            maintenance_every: 100,
            merge_stale: 150,
            retire_young: 20,
            retire_after: 600,
        }
    }
}

/// One slot after grounding.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SlotGround {
    pub slot: u16,
    pub concept: Option<u32>,
    /// Property values: observed if consistent, corrected or completed from the concept.
    pub props: Vec<(u16, i64)>,
    /// State values observed in pre and post.
    pub pre_states: Vec<(u16, i64)>,
    pub post_states: Vec<(u16, i64)>,
    /// Property channels filled in from the concept (pattern completion).
    pub completed: Vec<u16>,
    /// Property channels whose observed value disagreed with the concept and was corrected.
    pub corrected: Vec<u16>,
}

#[derive(Clone, Debug)]
pub struct Grounded {
    pub event: u64,
    pub context: u64,
    pub kind: Kind,
    pub t: u64,
    pub act: Option<Act>,
    pub slots: Vec<SlotGround>,
}

impl Grounded {
    pub fn slot(&self, s: u16) -> Option<&SlotGround> {
        self.slots.iter().find(|g| g.slot == s)
    }
}

#[derive(Clone, Debug, Default)]
pub struct GroundStats {
    pub matched: u64,
    pub unmatched: u64,
    pub born: u32,
    pub decayed: u32,
    pub merges: u32,
    pub splits: u32,
    pub reclassified: u32,
    pub ambiguous: u64,
    pub proto_merges: u32,
    pub retired: u32,
}

pub struct Grounder {
    im: ItemMemory<256>,
    pub chans: BTreeMap<u16, ChannelStats>,
    pub class: BTreeMap<u16, ChannelClass>,
    /// (action, channel) -> (changed on argument slots, total argument observations).
    pub action_effects: BTreeMap<(u16, u16), (u32, u32)>,
    pub concepts: Vec<ObjectConcept>,
    mem: CleanupMemory<256>,
    /// Concept pairs ever assigned in the same scene (evidence of distinct individuals).
    cooccur: HashSet<(u32, u32)>,
    pub params: GroundParams,
    pending_events: Vec<Event>,
    pub store: EventStore,
    pub tick: u64,
    /// Number of slot percepts processed with learning on.
    pub percepts: u64,
    /// Number of pre-scene slots observed (denominator of missing rates).
    pub slot_obs: u64,
    /// Noise model cache, refreshed at every maintenance pass.
    nm_cache: Option<NoiseModel>,
    pub stats: GroundStats,
    tiebreak: H,
}

fn q(n: u64) -> i64 {
    log2_q16(n.max(1))
}

impl Grounder {
    pub fn new(seed: u64) -> Self {
        Self::with_params(seed, GroundParams::default())
    }

    pub fn with_params(seed: u64, params: GroundParams) -> Self {
        let mut im = ItemMemory::new(seed);
        let tiebreak = im.get("__tiebreak");
        Grounder {
            im,
            chans: BTreeMap::new(),
            class: BTreeMap::new(),
            action_effects: BTreeMap::new(),
            concepts: Vec::new(),
            mem: CleanupMemory::with_z(6),
            cooccur: HashSet::new(),
            params,
            pending_events: Vec::new(),
            store: EventStore::new(),
            tick: 0,
            percepts: 0,
            slot_obs: 0,
            nm_cache: None,
            stats: GroundStats::default(),
            tiebreak,
        }
    }

    // ------------------------------------------------------------ codes

    fn ch_hv(&mut self, c: u16) -> H {
        self.im.get(&format!("gch:{c}"))
    }

    fn val_hv(&mut self, c: u16, v: i64) -> H {
        self.im.get(&format!("gv:{c}:{v}"))
    }

    fn percept_hv(&mut self, props: &[(u16, i64)]) -> H {
        let mut b = BitPlaneBundler::<256>::new();
        for &(c, v) in props {
            let x = self.ch_hv(c).bind(&self.val_hv(c, v));
            b.add(&x);
        }
        b.majority(&self.tiebreak)
    }

    fn refresh_proto(&mut self, k: u32) {
        let props: Vec<(u16, i64)> = {
            let c = &self.concepts[k as usize];
            c.hist.keys().filter_map(|&ch| c.majority(ch).map(|m| (ch, m.0))).collect()
        };
        let p = self.percept_hv(&props);
        self.concepts[k as usize].proto = p.clone();
        if self.concepts[k as usize].is_active() && !self.mem.update(k as u64, &p) {
            self.mem.insert(k as u64, &p);
        }
    }

    /// Decode channel `c` of a prototype by unbinding and nearest value code (HDC path).
    pub fn decode(&mut self, proto: &H, c: u16) -> Option<i64> {
        let probe = proto.unbind(&self.ch_hv(c));
        let vals: Vec<i64> = self.chans.get(&c).map(|s| s.values.iter().copied().collect()).unwrap_or_default();
        let floor = noise_floor::<256>(6);
        let mut best: Option<(i64, u32)> = None;
        for v in vals {
            let d = probe.distance(&self.val_hv(c, v));
            if d <= floor && best.map(|b| d < b.1 || (d == b.1 && v < b.0)).unwrap_or(true) {
                best = Some((v, d));
            }
        }
        best.map(|b| b.0)
    }

    // ------------------------------------------------------------ channel classes

    pub fn is_property(&self, c: u16) -> bool {
        self.class.get(&c).copied().unwrap_or(ChannelClass::Unknown) != ChannelClass::State
    }

    fn update_channel_stats(&mut self, e: &Event) {
        let args: HashSet<u16> = e.act.as_ref().map(|a| a.args.iter().copied().collect()).unwrap_or_default();
        for t in e.pre.tokens.iter().chain(e.post.tokens.iter()) {
            self.chans.entry(t.ch).or_default().values.insert(t.val);
        }
        for t in &e.pre.tokens {
            let s = self.chans.entry(t.ch).or_default();
            *s.freq.entry(t.val).or_insert(0) += 1;
            s.n += 1;
        }
        self.slot_obs += e.pre.slots().len() as u64;
        for t in &e.pre.tokens {
            if let Some(v2) = e.post.value(t.slot, t.ch) {
                if let Some(a) = &e.act {
                    if args.contains(&t.slot) {
                        let x = self.action_effects.entry((a.id, t.ch)).or_insert((0, 0));
                        x.1 += 1;
                        if v2 != t.val {
                            x.0 += 1;
                        }
                    }
                }
                let s = self.chans.entry(t.ch).or_default();
                if args.contains(&t.slot) {
                    s.total_arg += 1;
                    if v2 != t.val {
                        s.changed_arg += 1;
                    }
                } else {
                    s.total_other += 1;
                    if v2 != t.val {
                        s.changed_other += 1;
                    }
                }
            }
        }
    }

    /// Noise floor = smallest change rate among well-sampled channels; a channel is STATE if it
    /// changes clearly more often than that (on acted-on or other slots).
    pub fn classify_channels(&mut self) -> bool {
        let floor = self
            .chans
            .values()
            .filter(|s| s.total_other >= 30)
            .map(|s| s.pct_other())
            .min()
            .unwrap_or(0);
        let limit = floor * 2 + 4;
        let mut changed = false;
        let keys: Vec<u16> = self.chans.keys().copied().collect();
        for c in keys {
            let s = &self.chans[&c];
            let cls = if s.total_arg + s.total_other < 10 {
                ChannelClass::Unknown
            } else if s.pct_arg() > limit || s.pct_other() > limit {
                ChannelClass::State
            } else {
                ChannelClass::Property
            };
            if self.class.get(&c) != Some(&cls) {
                if self.class.contains_key(&c) {
                    self.stats.reclassified += 1;
                }
                self.class.insert(c, cls);
                changed = true;
            }
        }
        changed
    }

    // ------------------------------------------------------------ identity

    /// Candidate concepts by cleanup, verified per channel by unbinding.
    /// Returns (concept, matches, mismatches) of the unique best verified candidate.
    pub fn identify(&mut self, props: &[(u16, i64)], exclude: &HashSet<u32>) -> Option<(u32, u32, u32)> {
        match self.identify_ex(props, exclude) {
            Ident::Hit(h) => Some(h),
            _ => None,
        }
    }

    /// Identity with the reason for failure. An ambiguous percept is not novel: it must not create
    /// a new proto-concept (D021).
    pub fn identify_ex(&mut self, props: &[(u16, i64)], exclude: &HashSet<u32>) -> Ident {
        if props.is_empty() || self.mem.is_empty() {
            return Ident::Unknown;
        }
        // D027: verify every active concept. The cleanup memory ranks candidates for scale, but
        // identity needs completeness: a consistent concept left out of a top-k list turns a tie
        // into a wrong identity.
        let qhv = self.percept_hv(props);
        let cands: Vec<(u64, u32)> = self.mem.top_k(&qhv, self.mem.len());
        let mut scored: Vec<(u32, u32, u32)> = Vec::new();
        for (id, _d) in cands {
            if exclude.contains(&(id as u32)) {
                continue;
            }
            let proto = self.concepts[id as usize].proto.clone();
            let (mut m, mut mm) = (0u32, 0u32);
            for &(c, v) in props {
                match self.decode(&proto, c) {
                    Some(x) if x == v => m += 1,
                    Some(_) => mm += 1,
                    None => {}
                }
            }
            scored.push((id as u32, m, mm));
        }
        // D033: one token cannot establish identity (no redundancy to detect a noisy token)
        let need = self.params.min_match.min(props.len() as u32).max(2);
        scored.retain(|s| s.2 <= self.params.tol_mismatch && s.1 >= need);
        scored.sort_by(|a, b| b.1.cmp(&a.1).then(a.2.cmp(&b.2)).then(a.0.cmp(&b.0)));
        match scored.as_slice() {
            [] => Ident::Unknown,
            [only] => Ident::Hit(*only),
            [a, b, ..] => {
                if (a.1, a.2) != (b.1, b.2) {
                    return Ident::Hit(*a);
                }
                // tie on evidence: a born concept beats an unborn proto; between two born
                // concepts only a 3x larger history breaks the tie, otherwise refuse to guess
                let (ca, cb) = (&self.concepts[a.0 as usize], &self.concepts[b.0 as usize]);
                let born = |c: &ObjectConcept| c.status == ConceptStatus::Concept;
                if born(ca) && !born(cb) {
                    Ident::Hit(*a)
                } else if born(cb) && !born(ca) {
                    Ident::Hit(*b)
                } else if !born(ca) && !born(cb) {
                    // D026a: a tie between two tentative protos says the percept cannot tell them
                    // apart, not that they are the same thing (D026 merged different objects into
                    // chimeras). Treat the percept as unexplained: it may seed a cleaner proto, and
                    // the chance test (D028) plus decay remove the losers.
                    Ident::Unknown
                } else {
                    // two born concepts explain it equally: refuse to guess (no frequency prior)
                    Ident::Ambiguous
                }
            }
        }
    }

    // ------------------------------------------------------------ MDL

    fn ch_bits(&self, c: u16) -> i64 {
        q(self.chans.get(&c).map(|s| s.values.len() as u64 + 1).unwrap_or(2))
    }

    /// Description cost of a concept's sightings given the concept (Q16 bits), excluding the
    /// concept's own cost.
    fn coded_bits(&self, k: &ObjectConcept, n_concepts: u64) -> i64 {
        let mut bits = k.sightings as i64 * q(n_concepts + 1);
        for (&c, h) in &k.hist {
            let total: u32 = h.iter().map(|x| x.1).sum();
            let maj = h.iter().map(|x| x.1).max().unwrap_or(0);
            bits += (total - maj) as i64 * self.ch_bits(c);
        }
        bits
    }

    fn raw_bits(&self, k: &ObjectConcept) -> i64 {
        k.hist.iter().map(|(&c, h)| h.iter().map(|x| x.1).sum::<u32>() as i64 * self.ch_bits(c)).sum()
    }

    fn concept_cost(&self, k: &ObjectConcept) -> i64 {
        k.hist.keys().map(|&c| self.ch_bits(c)).sum()
    }

    fn n_concepts(&self) -> u64 {
        self.concepts.iter().filter(|c| c.status == ConceptStatus::Concept).count() as u64
    }

    /// Probability (Q32) that a random percept agrees with the concept's majority on every channel
    /// or on all but one, under the marginal value frequencies (null model of coincidence).
    fn chance_match_q32(&self, c: &ObjectConcept) -> u128 {
        let one: u128 = 1 << 32;
        let mut ps: Vec<u128> = Vec::new();
        for (&ch, _) in &c.hist {
            let Some((v, own, own_total)) = c.majority(ch) else { continue };
            let st = self.chans.get(&ch);
            let (f, n, k) = st.map(|s| (*s.freq.get(&v).unwrap_or(&0) as u128, s.n as u128, s.freq.len() as u128)).unwrap_or((0, 0, 1));
            // D028b: leave-self-out background frequency (the proto's own sightings are not
            // evidence that its pattern is common)
            let f = f.saturating_sub(own as u128);
            let n = n.saturating_sub(own_total as u128);
            ps.push(((f + 1) << 32) / (n + k + 1));
        }
        let all = ps.iter().fold(one, |acc, &p| (acc * p) >> 32);
        // mirror the identity rule: one mismatch is allowed only if the remaining channels still
        // reach the required number of matches
        let need = self.params.min_match.min(ps.len() as u32) as usize;
        if ps.len() < need + 1 || self.params.tol_mismatch == 0 {
            return all;
        }
        let mut one_off: u128 = 0;
        for j in 0..ps.len() {
            let mut prod = one - ps[j];
            for (i, &p) in ps.iter().enumerate() {
                if i != j {
                    prod = (prod * p) >> 32;
                }
            }
            one_off += prod;
        }
        (all + one_off).min(one)
    }

    /// Q32 probability that one sighting of born concept `kc` is perceived in a way that the proto
    /// explains better than `kc` itself: on every channel where their majorities differ the
    /// percept shows the proto's value through noise or is missing, and not all of them missing.
    fn noise_view_q32(&self, proto: &ObjectConcept, kc: &ObjectConcept) -> u128 {
        let one: u128 = 1 << 32;
        let mut all: u128 = one;
        let mut miss_all: u128 = one;
        let mut differs = false;
        for (&ch, _) in &proto.hist {
            let (Some(pv), Some(kv)) = (proto.majority(ch), kc.majority(ch)) else { continue };
            if pv.0 == kv.0 {
                continue;
            }
            differs = true;
            let st = self.chans.get(&ch);
            // per-observation error = half the pre/post change rate on non-argument slots
            let (eps, range, present) = st
                .map(|s| (s.pct_other() as u128, s.values.len().max(2) as u128, s.n as u128))
                .unwrap_or((0, 2, 0));
            let p_show = (eps << 32) / (200 * range);
            let slots = self.slot_obs.max(1) as u128;
            let mu = (slots.saturating_sub(present) << 32) / slots;
            all = (all * (p_show + mu).min(one)) >> 32;
            miss_all = (miss_all * mu) >> 32;
        }
        if !differs {
            return one;
        }
        all.saturating_sub(miss_all)
    }

    /// Expected number of sightings the proto would receive from noisy views of born concepts
    /// since it was created (Q16).
    fn expected_noise_views_q16(&self, c: &ObjectConcept) -> u128 {
        let since = self.percepts.saturating_sub(c.created_percept) as u128;
        let mut total: u128 = 0;
        for kc in &self.concepts {
            if kc.status != ConceptStatus::Concept || kc.id == c.id {
                continue;
            }
            let life = self.percepts.saturating_sub(kc.created_percept).max(1) as u128;
            let in_window = (kc.sightings as u128 * since.min(life)) / life;
            total += (in_window * self.noise_view_q32(c, kc)) >> 16;
        }
        total
    }

    fn try_birth(&mut self, k: u32) {
        let n = self.n_concepts();
        let c = &self.concepts[k as usize];
        if c.status != ConceptStatus::Proto || c.sightings < self.params.min_sightings {
            return;
        }
        // D028: sightings must exceed what coincidence explains: expected chance matches among the
        // percepts seen since creation, plus three standard deviations, plus one
        // D031: recurrence evidence is the number of sightings consistent with the majority
        // pattern (second-smallest per-channel majority count when one mismatch is tolerated),
        // not raw sightings: absorbed or noisy sightings that disagree do not count
        let mut maj_counts: Vec<u32> = c.hist.keys().filter_map(|&ch| c.majority(ch).map(|m| m.1)).collect();
        maj_counts.sort_unstable();
        let support = if maj_counts.len() >= 3 && self.params.tol_mismatch > 0 { maj_counts[1] } else { maj_counts.first().copied().unwrap_or(0) };
        if support < self.params.min_sightings {
            return;
        }
        let since = self.percepts.saturating_sub(c.created_percept) as u128;
        let expected_q16 = ((since * self.chance_match_q32(c)) >> 16) + self.expected_noise_views_q16(c);
        let ceiling_q16 = expected_q16 + 3 * (isqrt((expected_q16 << 16) as u64) as u128) + (1 << 16);
        if ((support as u128) << 16) <= ceiling_q16 {
            return;
        }
        // D024: a recurring pattern must also pay for having been picked out of every proto
        // currently under consideration (chance recurrences among many protos are expected)
        let alive = self.concepts.iter().filter(|c| c.is_active()).count() as u64;
        let selection = log2_q16(alive.max(2));
        let saving = self.raw_bits(c) - self.coded_bits(c, n) - self.concept_cost(c) - selection;
        if saving > 0 {
            let t = self.tick;
            let s = c.sightings;
            let c = &mut self.concepts[k as usize];
            c.status = ConceptStatus::Concept;
            c.lineage.push(ConceptEvent::Born { t, sightings: s, saving_bits: saving >> 16 });
            self.stats.born += 1;
        }
    }

    // ------------------------------------------------------------ event processing

    /// Ingest one event. Returns its grounding (None during warmup).
    pub fn observe(&mut self, e: Event) -> Option<Grounded> {
        self.tick += 1;
        self.update_channel_stats(&e);
        let id = self.store.append(e);
        if (self.store.len()) <= self.params.warmup {
            self.pending_events.push(self.store.get(id).clone());
            if self.store.len() == self.params.warmup {
                self.classify_channels();
                let buf = std::mem::take(&mut self.pending_events);
                for ev in &buf {
                    self.assign(ev, true);
                }
            }
            return None;
        }
        if self.tick % self.params.maintenance_every == 0 {
            self.maintenance();
        }
        let ev = self.store.get(id).clone();
        Some(self.assign(&ev, true))
    }

    /// Ground an event without learning from it (for queries and restoration).
    pub fn ground(&mut self, e: &Event) -> Grounded {
        self.assign(e, false)
    }

    fn props_of(&self, e: &Event, slot: u16) -> Vec<(u16, i64)> {
        let mut props: Vec<(u16, i64)> = e.pre.slot(slot).into_iter().filter(|&(c, _)| self.is_property(c)).collect();
        for (c, v) in e.post.slot(slot) {
            if self.is_property(c) && !props.iter().any(|p| p.0 == c) {
                props.push((c, v));
            }
        }
        props.sort_unstable();
        props
    }

    fn assign(&mut self, e: &Event, learn: bool) -> Grounded {
        let mut slots: Vec<u16> = e.pre.slots();
        for s in e.post.slots() {
            if !slots.contains(&s) {
                slots.push(s);
            }
        }
        // identify every slot, then resolve conflicts: one concept per scene
        let mut proposals: Vec<(u16, Vec<(u16, i64)>, Option<(u32, u32, u32)>)> = Vec::new();
        let mut ambiguous: HashSet<u16> = HashSet::new();
        for &s in &slots {
            let props = self.props_of(e, s);
            let hit = match self.identify_ex(&props, &HashSet::new()) {
                Ident::Hit(h) => Some(h),
                Ident::Ambiguous => {
                    ambiguous.insert(s);
                    None
                }
                Ident::Unknown => None,
            };
            proposals.push((s, props, hit));
        }
        let mut taken: HashSet<u32> = HashSet::new();
        let mut order: Vec<usize> = (0..proposals.len()).collect();
        order.sort_by_key(|&i| proposals[i].2.map(|h| (u32::MAX - h.1, h.2)).unwrap_or((u32::MAX, u32::MAX)));
        let mut result: Vec<Option<u32>> = vec![None; proposals.len()];
        for i in order {
            let (_, ref props, hit) = proposals[i];
            let chosen = match hit {
                Some((k, _, _)) if !taken.contains(&k) => Some(k),
                Some(_) => self.identify(props, &taken).map(|h| h.0),
                None => None,
            };
            if let Some(k) = chosen {
                taken.insert(k);
            }
            result[i] = chosen;
        }
        let mut out = Vec::new();
        for (i, (s, props, _)) in proposals.into_iter().enumerate() {
            let mut k = result[i];
            if learn {
                self.percepts += 1;
                k = match k {
                    Some(k) => {
                        self.reinforce(k, &props, e.id, s);
                        Some(k)
                    }
                    None => {
                        self.stats.unmatched += 1;
                        // D021: ambiguous or too thin percepts are not evidence of a new object
                        if ambiguous.contains(&s) || (props.len() as u32) < 2 {
                            self.stats.ambiguous += 1;
                            None
                        } else {
                            let nk = self.new_proto(&props, e.id, s);
                            taken.insert(nk);
                            Some(nk)
                        }
                    }
                };
            }
            let mut sg = SlotGround {
                slot: s,
                concept: k.filter(|&k| self.concepts[k as usize].status == ConceptStatus::Concept),
                props: Vec::new(),
                pre_states: e.pre.slot(s).into_iter().filter(|&(c, _)| !self.is_property(c)).collect(),
                post_states: e.post.slot(s).into_iter().filter(|&(c, _)| !self.is_property(c)).collect(),
                completed: Vec::new(),
                corrected: Vec::new(),
            };
            // D036: completing a missing value is an action on an inference; only do it when the
            // identity posterior reaches the target accuracy
            let confident = match sg.concept {
                Some(k) => {
                    let missing = self.concepts[k as usize].hist.keys().any(|c| !props.iter().any(|p| p.0 == *c));
                    if missing {
                        let nm = match &self.nm_cache {
                            Some(n) => n.clone(),
                            None => self.noise_model(),
                        };
                        matches!(self.identity_posterior(&props, &nm), Some((b, p)) if b == k && p >= FILL_POSTERIOR_Q16)
                    } else {
                        true
                    }
                }
                None => false,
            };
            match sg.concept {
                Some(k) => {
                    let proto = self.concepts[k as usize].proto.clone();
                    let chans: Vec<u16> = self.concepts[k as usize].hist.keys().copied().collect();
                    for c in chans {
                        let decoded = self.decode(&proto, c);
                        let obs = props.iter().find(|p| p.0 == c).map(|p| p.1);
                        match (obs, decoded) {
                            (Some(o), Some(d)) if o == d => sg.props.push((c, o)),
                            (Some(_), Some(d)) => {
                                sg.props.push((c, d));
                                sg.corrected.push(c);
                            }
                            (None, Some(d)) if confident => {
                                sg.props.push((c, d));
                                sg.completed.push(c);
                            }
                            (None, Some(_)) => {}
                            (Some(o), None) => sg.props.push((c, o)),
                            (None, None) => {}
                        }
                    }
                }
                None => sg.props = props,
            }
            out.push(sg);
        }
        if learn {
            let ids: Vec<u32> = out.iter().filter_map(|g| g.concept).collect();
            for i in 0..ids.len() {
                for j in (i + 1)..ids.len() {
                    let (a, b) = (ids[i].min(ids[j]), ids[i].max(ids[j]));
                    self.cooccur.insert((a, b));
                }
            }
        }
        Grounded { event: e.id, context: e.context, kind: e.kind, t: e.t, act: e.act.clone(), slots: out }
    }

    fn reinforce(&mut self, k: u32, props: &[(u16, i64)], event: u64, slot: u16) {
        self.stats.matched += 1;
        let t = self.tick;
        let mut disagreements = 0u32;
        {
            let c = &mut self.concepts[k as usize];
            for &(ch, v) in props {
                if let Some(m) = c.majority(ch) {
                    if m.0 != v {
                        disagreements += 1;
                    }
                }
                c.add_value(ch, v);
            }
            c.sightings += 1;
            c.evidence.observe(disagreements == 0, t);
            if c.exemplars.len() < 64 {
                c.exemplars.push((event, slot));
            }
            c.last_t = t;
        }
        self.refresh_proto(k);
        self.try_birth(k);
    }

    fn new_proto(&mut self, props: &[(u16, i64)], event: u64, slot: u16) -> u32 {
        let id = self.concepts.len() as u32;
        let t = self.tick;
        let mut c = ObjectConcept {
            id,
            status: ConceptStatus::Proto,
            hist: BTreeMap::new(),
            proto: H::zero(),
            sightings: 1,
            evidence: Evidence::new(),
            exemplars: vec![(event, slot)],
            first_t: t,
            last_t: t,
            lineage: vec![ConceptEvent::Created { t, event }],
            created_percept: self.percepts,
        };
        for &(ch, v) in props {
            c.add_value(ch, v);
        }
        self.concepts.push(c);
        self.refresh_proto(id);
        id
    }

    // ------------------------------------------------------------ maintenance: decay, merge, split

    pub fn maintenance(&mut self) {
        self.nm_cache = Some(self.noise_model());
        if self.classify_channels() {
            self.rebuild_property_view();
        }
        self.decay();
        self.retire_unused();
        self.merge_pass();
        self.split_pass();
    }

    /// D035: a young born concept that has not been used for `retire_after` ticks never amortised
    /// its description cost. It leaves the active set (status Decayed, lineage kept).
    fn retire_unused(&mut self) {
        let now = self.tick;
        for k in 0..self.concepts.len() {
            let c = &self.concepts[k];
            if c.status == ConceptStatus::Concept
                && c.sightings < self.params.retire_young
                && now.saturating_sub(c.last_t) > self.params.retire_after
            {
                let t = now;
                self.concepts[k].status = ConceptStatus::Decayed;
                self.concepts[k].lineage.push(ConceptEvent::Retired { t });
                self.mem.remove(k as u64);
                self.stats.retired += 1;
            }
        }
    }

    /// After channel classes change, drop state channels from concept histograms.
    fn rebuild_property_view(&mut self) {
        let state: Vec<u16> = self.class.iter().filter(|x| *x.1 == ChannelClass::State).map(|x| *x.0).collect();
        for k in 0..self.concepts.len() {
            let mut changed = false;
            for c in &state {
                if self.concepts[k].hist.remove(c).is_some() {
                    changed = true;
                }
            }
            if changed {
                self.refresh_proto(k as u32);
            }
        }
    }

    fn decay(&mut self) {
        let now = self.tick;
        for k in 0..self.concepts.len() {
            let c = &self.concepts[k];
            if c.status == ConceptStatus::Proto && now.saturating_sub(c.last_t) > self.params.pending_ttl {
                self.concepts[k].status = ConceptStatus::Decayed;
                self.mem.remove(k as u64);
                self.stats.decayed += 1;
            }
        }
    }

    /// Merge a silent concept into an active successor that it never co-occurred with, when
    /// one concept describes both histories in fewer bits.
    fn merge_pass(&mut self) {
        let now = self.tick;
        let n = self.n_concepts();
        let ids: Vec<u32> = self.concepts.iter().filter(|c| c.status == ConceptStatus::Concept).map(|c| c.id).collect();
        for &old in &ids {
            let (stale, proto) = {
                let c = &self.concepts[old as usize];
                (now.saturating_sub(c.last_t) >= self.params.merge_stale, c.proto.clone())
            };
            if !stale || self.concepts[old as usize].status != ConceptStatus::Concept {
                continue;
            }
            for (cand, _) in self.mem.top_k(&proto, self.params.top_k) {
                let cand = cand as u32;
                if cand == old || self.concepts[cand as usize].status != ConceptStatus::Concept {
                    continue;
                }
                let key = (old.min(cand), old.max(cand));
                if self.cooccur.contains(&key) || self.concepts[cand as usize].last_t <= self.concepts[old as usize].last_t {
                    continue;
                }
                // channel disagreement between majorities
                let (a, b) = (&self.concepts[old as usize], &self.concepts[cand as usize]);
                let mut diff = 0;
                for (&ch, _) in &a.hist {
                    if let (Some(x), Some(y)) = (a.majority(ch), b.majority(ch)) {
                        if x.0 != y.0 {
                            diff += 1;
                        }
                    }
                }
                if diff > self.params.tol_mismatch {
                    continue;
                }
                let mut merged = b.clone();
                for (&ch, h) in &a.hist {
                    for &(v, cnt) in h {
                        for _ in 0..cnt {
                            merged.add_value(ch, v);
                        }
                    }
                }
                merged.sightings += a.sightings;
                let separate = self.coded_bits(a, n) + self.coded_bits(b, n) + self.concept_cost(a) + self.concept_cost(b);
                let together = self.coded_bits(&merged, n - 1) + self.concept_cost(&merged);
                let saving = separate - together;
                if saving > 0 {
                    let t = now;
                    let old_c = self.concepts[old as usize].clone();
                    {
                        let c = &mut self.concepts[cand as usize];
                        c.hist = merged.hist;
                        c.sightings = merged.sightings;
                        c.exemplars.extend(old_c.exemplars.iter().take(8));
                        c.first_t = c.first_t.min(old_c.first_t);
                        c.lineage.push(ConceptEvent::Absorbed { from: old, t });
                    }
                    let c = &mut self.concepts[old as usize];
                    c.status = ConceptStatus::Merged(cand);
                    c.lineage.push(ConceptEvent::MergedInto { into: cand, t, saving_bits: saving >> 16 });
                    self.mem.remove(old as u64);
                    self.refresh_proto(cand);
                    self.stats.merges += 1;
                    break;
                }
            }
        }
    }

    /// Split a concept whose histogram on some property channel has two strong modes, when two
    /// concepts describe its sightings in fewer bits than one.
    fn split_pass(&mut self) {
        let n = self.n_concepts();
        let ids: Vec<u32> = self.concepts.iter().filter(|c| c.status == ConceptStatus::Concept).map(|c| c.id).collect();
        for k in ids {
            let c = self.concepts[k as usize].clone();
            let mut best: Option<(i64, u16, i64, i64)> = None;
            for (&ch, h) in &c.hist {
                if h.len() < 2 {
                    continue;
                }
                let mut hs = h.clone();
                hs.sort_by_key(|x| std::cmp::Reverse(x.1));
                let total: u32 = hs.iter().map(|x| x.1).sum();
                let (v1, n1) = hs[0];
                let (v2, n2) = hs[1];
                // both modes must be substantial: second mode >= 30% and >= 4 sightings
                if n2 < 4 || n2 * 10 < total * 3 {
                    continue;
                }
                let bits = self.ch_bits(ch);
                // one concept pays a mismatch for every non-majority value; two concepts pay the
                // residual noise on that channel plus one extra concept and an index bit per sighting
                let one = (total - n1) as i64 * bits;
                let residual = (total - n1 - n2) as i64 * bits;
                let two = residual + self.concept_cost(&c) + c.sightings as i64 * q(n + 2) - c.sightings as i64 * q(n + 1);
                let saving = one - two;
                if saving > 0 && best.map(|b| saving > b.0).unwrap_or(true) {
                    best = Some((saving, ch, v1, v2));
                }
            }
            if let Some((saving, ch, v1, v2)) = best {
                let t = self.tick;
                let mut children = Vec::new();
                for v in [v1, v2] {
                    let id = self.concepts.len() as u32;
                    let mut child = c.clone();
                    child.id = id;
                    child.status = ConceptStatus::Concept;
                    child.lineage = vec![ConceptEvent::SplitFrom { parent: k, t }];
                    let share = c.hist[&ch].iter().find(|x| x.0 == v).map(|x| x.1).unwrap_or(1);
                    child.hist.insert(ch, vec![(v, share)]);
                    child.sightings = share;
                    child.exemplars.clear();
                    self.concepts.push(child);
                    self.refresh_proto(id);
                    children.push(id);
                }
                let pc = &mut self.concepts[k as usize];
                pc.status = ConceptStatus::Split;
                pc.lineage.push(ConceptEvent::SplitInto { children, ch, t, saving_bits: saving >> 16 });
                self.mem.remove(k as u64);
                self.stats.splits += 1;
            }
        }
    }

    /// Noise floor of pre/post change rate (percent) over well-sampled channels.
    pub fn noise_floor_pct(&self) -> u32 {
        self.chans.values().filter(|s| s.total_other >= 30).map(|s| s.pct_other()).min().unwrap_or(0)
    }

    /// Action candidates: for each action, the channels it changes on its arguments clearly
    /// more often than sensor noise does.
    pub fn action_effect_map(&self) -> BTreeMap<u16, Vec<u16>> {
        let limit = self.noise_floor_pct() * 2 + 4;
        let mut out: BTreeMap<u16, Vec<u16>> = BTreeMap::new();
        for (&(a, c), &(ch, tot)) in &self.action_effects {
            let e = out.entry(a).or_default();
            if tot >= 10 && ch * 100 / tot > limit {
                e.push(c);
            }
        }
        out
    }

    /// D036: measured per-channel observation model. Per-observation error = half the pre/post
    /// change rate on non-argument slots (two independent observations), floored at 0.5%.
    pub fn noise_model(&self) -> NoiseModel {
        use hdc_core::fixed::log2_ratio_q16;
        let mut nm = NoiseModel::default();
        for (&c, st) in &self.chans {
            let r = st.values.len().max(2) as u64;
            let eps_num = (st.pct_other() as u64).max(1); // percent * 2 -> eps = eps_num / 200
            let den = 200u64;
            let l_match = log2_ratio_q16(den - eps_num.min(den - 1), den);
            let l_mis = log2_ratio_q16(eps_num, den * r);
            let l_unif = log2_ratio_q16(1, r);
            nm.ch.insert(c, (l_match, l_mis, l_unif));
        }
        nm
    }

    /// Posterior (Q16) that the percept comes from its most likely born concept, against every
    /// other born concept and one unknown-source hypothesis (uniform prior, no frequency prior).
    pub fn identity_posterior(&mut self, props: &[(u16, i64)], nm: &NoiseModel) -> Option<(u32, i64)> {
        if props.is_empty() {
            return None;
        }
        let born: Vec<u32> = self.live_concepts();
        let mut ll: Vec<i64> = Vec::with_capacity(born.len() + 1);
        for &k in &born {
            let proto = self.concepts[k as usize].proto.clone();
            let mut l = 0i64;
            for &(c, v) in props {
                let (lm, lx, lu) = nm.terms(c);
                l += match self.decode(&proto, c) {
                    Some(x) if x == v => lm,
                    Some(_) => lx,
                    None => lu,
                };
            }
            ll.push(l);
        }
        // unknown source: every token uniform
        ll.push(props.iter().map(|&(c, _)| nm.terms(c).2).sum());
        let (i, p) = hdc_core::fixed::best_posterior_q16(&ll)?;
        if i >= born.len() {
            return None;
        }
        Some((born[i], p))
    }

    /// Concepts that are currently usable (born, not merged/split/decayed).
    pub fn live_concepts(&self) -> Vec<u32> {
        self.concepts.iter().filter(|c| c.status == ConceptStatus::Concept).map(|c| c.id).collect()
    }

    /// Follow merge links to the surviving concept.
    pub fn resolve(&self, mut k: u32) -> u32 {
        while let ConceptStatus::Merged(into) = self.concepts[k as usize].status {
            k = into;
        }
        k
    }

    pub fn memory_bytes(&self) -> usize {
        self.mem.bytes() + self.concepts.len() * (std::mem::size_of::<ObjectConcept>() + 256) + self.store.bytes()
    }

    pub fn channel_report(&self) -> Vec<(u16, ChannelClass, u32, u32)> {
        self.chans
            .iter()
            .map(|(&c, s)| (c, self.class.get(&c).copied().unwrap_or(ChannelClass::Unknown), s.pct_arg(), s.pct_other()))
            .collect()
    }
}

/// Map from a hidden truth label to concept assignments: purity and completeness in percent.
pub fn cluster_quality(pairs: &[(u32, u32)]) -> (u32, u32) {
    // pairs: (truth, concept)
    let mut by_concept: HashMap<u32, HashMap<u32, u32>> = HashMap::new();
    let mut by_truth: HashMap<u32, HashMap<u32, u32>> = HashMap::new();
    for &(t, c) in pairs {
        *by_concept.entry(c).or_default().entry(t).or_insert(0) += 1;
        *by_truth.entry(t).or_default().entry(c).or_insert(0) += 1;
    }
    let total = pairs.len().max(1) as u32;
    let pure: u32 = by_concept.values().map(|h| *h.values().max().unwrap_or(&0)).sum();
    let comp: u32 = by_truth.values().map(|h| *h.values().max().unwrap_or(&0)).sum();
    (pure * 100 / total, comp * 100 / total)
}
