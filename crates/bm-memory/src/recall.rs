//! Event recall: restore a stored event from a noisy, partial copy.
//!
//! The event vector binds grounded object identities with channels, values and phase, so a copy
//! whose tokens are partly missing or flipped is first grounded (which completes and corrects
//! property values from concepts) and then cleaned up against stored events. Below the noise
//! floor the memory abstains (D003).

use crate::ground::Grounded;
use hdc_core::*;

type H = Hv32k;

/// Event vectors are 32,768 bits (D029). Minimum gap between best and second-best stored event:
/// the difference of two Hamming distances to independent references has sigma = sqrt(D/2) = 128
/// bits at 32k; 5 sigma = 640.
pub const MARGIN: u32 = 640;

pub struct EventMemory {
    im: ItemMemory<512>,
    mem: CleanupMemory<512>,
    tiebreak: H,
    /// Symbolic token set of every stored grounded event, for verification (D030).
    tokens: std::collections::HashMap<u64, Vec<u64>>,
    /// Grounded slots of every stored event, for slot alignment (D030a).
    slots: std::collections::HashMap<u64, Grounded>,
}

/// Agreement between a raw query slot (tokens with phase) and a stored grounded slot.
fn slot_agreement(q: &[(u8, u16, i64)], s: &crate::ground::SlotGround) -> u32 {
    let mut n = 0;
    for &(ph, c, v) in q {
        if let Some(p) = s.props.iter().find(|x| x.0 == c) {
            n += (p.1 == v) as u32;
            continue;
        }
        let st = if ph == 0 { &s.pre_states } else { &s.post_states };
        if let Some(p) = st.iter().find(|x| x.0 == c) {
            n += (p.1 == v) as u32;
        }
    }
    n
}

/// Best one-to-one alignment of query slots to stored slots (greedy on agreement), plus action
/// and argument agreement. Returns (agreeing tokens, query tokens).
fn aligned_agreement(q: &crate::event::Event, stored: &Grounded) -> (u32, u32) {
    let (a, t, _) = align(q, stored);
    (a, t)
}

fn align(q: &crate::event::Event, stored: &Grounded) -> (u32, u32, Vec<(u16, u16)>) {
    let slots = {
        let mut v = q.pre.slots();
        for s in q.post.slots() {
            if !v.contains(&s) {
                v.push(s);
            }
        }
        v
    };
    let qtok: Vec<(u16, Vec<(u8, u16, i64)>)> = slots
        .iter()
        .map(|&s| {
            let mut t: Vec<(u8, u16, i64)> = q.pre.slot(s).into_iter().map(|(c, v)| (0, c, v)).collect();
            t.extend(q.post.slot(s).into_iter().map(|(c, v)| (1, c, v)));
            (s, t)
        })
        .collect();
    let total: u32 = qtok.iter().map(|x| x.1.len() as u32).sum::<u32>() + 1;
    let mut cells: Vec<(u32, usize, usize)> = Vec::new();
    for (i, (_, t)) in qtok.iter().enumerate() {
        for (j, s) in stored.slots.iter().enumerate() {
            cells.push((slot_agreement(t, s), i, j));
        }
    }
    cells.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)).then(a.2.cmp(&b.2)));
    let (mut used_q, mut used_s) = (vec![false; qtok.len()], vec![false; stored.slots.len()]);
    let mut map: Vec<(u16, u16)> = Vec::new();
    let mut agree = 0;
    for (a, i, j) in cells {
        if used_q[i] || used_s[j] {
            continue;
        }
        used_q[i] = true;
        used_s[j] = true;
        agree += a;
        map.push((qtok[i].0, stored.slots[j].slot));
    }
    if let (Some(qa), Some(sa)) = (&q.act, &stored.act) {
        if qa.id == sa.id {
            agree += 1;
            for (k, qs) in qa.args.iter().enumerate() {
                if let (Some(m), Some(ss)) = (map.iter().find(|x| x.0 == *qs), sa.args.get(k)) {
                    if m.1 == *ss {
                        agree += 1;
                    }
                }
            }
        }
    }
    (agree, total + q.act.as_ref().map(|a| a.args.len() as u32).unwrap_or(0), map)
}

/// Fingerprints of the grounded event's facts (object signature x channel x value x phase,
/// action and argument signatures).
fn facts(g: &Grounded) -> Vec<u64> {
    use hdc_core::rng::mix64;
    let mut v = Vec::new();
    let mut sigs = Vec::new();
    for s in &g.slots {
        let mut sig = 0x51u64;
        for &(c, val) in &s.props {
            sig = mix64(sig, mix64(c as u64, val as u64));
        }
        sigs.push((s.slot, sig));
        for &(c, val) in &s.props {
            v.push(mix64(sig, mix64(1 + c as u64, val as u64)));
        }
        for &(c, val) in &s.pre_states {
            v.push(mix64(sig ^ 0xA, mix64(1 + c as u64, val as u64)));
        }
        for &(c, val) in &s.post_states {
            v.push(mix64(sig ^ 0xB, mix64(1 + c as u64, val as u64)));
        }
    }
    if let Some(a) = &g.act {
        v.push(mix64(0xAC7, a.id as u64));
        for (i, s) in a.args.iter().enumerate() {
            if let Some((_, sig)) = sigs.iter().find(|x| x.0 == *s) {
                v.push(mix64(0xA6 + i as u64, *sig));
            }
        }
    }
    v.sort_unstable();
    v
}

fn overlap(a: &[u64], b: &[u64]) -> u32 {
    let (mut i, mut j, mut n) = (0, 0, 0u32);
    while i < a.len() && j < b.len() {
        match a[i].cmp(&b[j]) {
            std::cmp::Ordering::Less => i += 1,
            std::cmp::Ordering::Greater => j += 1,
            std::cmp::Ordering::Equal => {
                n += 1;
                i += 1;
                j += 1;
            }
        }
    }
    n
}

/// v0.2 F1 diagnostic view of one recall decision.
#[derive(Clone, Debug)]
pub struct RecallTrace {
    pub candidates: usize,
    pub true_rank: Option<usize>,
    pub true_score: u32,
    pub best: Option<(u64, u32)>,
    pub second_score: u32,
    pub query_tokens: u32,
}

impl EventMemory {
    pub fn new(seed: u64) -> Self {
        let mut im = ItemMemory::new(seed);
        let tiebreak = im.get("__tb");
        EventMemory {
            im,
            mem: CleanupMemory::with_z(6),
            tiebreak,
            tokens: std::collections::HashMap::new(),
            slots: std::collections::HashMap::new(),
        }
    }

    /// Object key from its (denoised, completed) property values. Independent of concept ids, so
    /// merges and splits after storage do not invalidate stored events.
    fn obj_key(&mut self, g: &crate::ground::SlotGround) -> H {
        let mut b = BitPlaneBundler::<512>::new();
        for &(c, v) in &g.props {
            b.add(&self.im.get(&format!("k:{c}")).bind(&self.im.get(&format!("u:{c}:{v}"))));
        }
        if b.is_empty() {
            return self.im.get("obj:empty");
        }
        b.majority(&self.tiebreak).permute(1)
    }

    pub fn encode(&mut self, g: &Grounded) -> H {
        let mut b = BitPlaneBundler::<512>::new();
        let pre = self.im.get("phase:pre");
        let post = self.im.get("phase:post");
        let mut keys = Vec::new();
        for s in &g.slots {
            let key = self.obj_key(s);
            keys.push((s.slot, key.clone()));
            for &(c, v) in &s.props {
                b.add(&key.bind(&self.im.get(&format!("c:{c}"))).bind(&self.im.get(&format!("v:{c}:{v}"))));
            }
            for &(c, v) in &s.pre_states {
                b.add(&key.bind(&pre).bind(&self.im.get(&format!("c:{c}"))).bind(&self.im.get(&format!("v:{c}:{v}"))));
            }
            for &(c, v) in &s.post_states {
                b.add(&key.bind(&post).bind(&self.im.get(&format!("c:{c}"))).bind(&self.im.get(&format!("v:{c}:{v}"))));
            }
        }
        if let Some(a) = &g.act {
            b.add(&self.im.get("act").bind(&self.im.get(&format!("a:{}", a.id))));
            for (i, s) in a.args.iter().enumerate() {
                if let Some((_, k)) = keys.iter().find(|x| x.0 == *s) {
                    b.add(&self.im.get(&format!("arg:{i}")).bind(k));
                }
            }
        }
        b.majority(&self.tiebreak)
    }

    pub fn store(&mut self, g: &Grounded) {
        let h = self.encode(g);
        self.mem.insert(g.event, &h);
        self.tokens.insert(g.event, facts(g));
        self.slots.insert(g.event, g.clone());
    }

    /// D030a: event-level inference. The HDC memory proposes 32 candidates from the grounded
    /// query; each candidate is verified by aligning the RAW query slots to the candidate's stored
    /// objects and counting agreeing tokens (no dependence on per-object identity of a sparse
    /// percept). Accept the unique best if it leads by >= 2 tokens and explains >= half of the
    /// query tokens; otherwise abstain.
    pub fn recall_event(&mut self, gq: &Grounded, raw: &crate::event::Event) -> Option<Hit> {
        let h = self.encode(gq);
        let mut scored: Vec<(u32, u32, u64, u32)> = Vec::new();
        for (id, d) in self.mem.top_k(&h, 32) {
            if let Some(st) = self.slots.get(&id) {
                let (a, tot) = aligned_agreement(raw, st);
                scored.push((a, tot, id, d));
            }
        }
        scored.sort_by(|a, b| b.0.cmp(&a.0).then(a.3.cmp(&b.3)));
        let best = *scored.first()?;
        let second = scored.get(1).map(|x| x.0).unwrap_or(0);
        if best.0 >= second + 2 && best.0 * 2 >= best.1 {
            Some(Hit { id: best.2, dist: best.3, margin: best.0 - second })
        } else {
            None
        }
    }

    /// Diagnostic only (v0.2, F1): the `recall_event` decision for a query whose true event is
    /// known: rank of the true event among the proposed candidates, its alignment score, the best
    /// candidate, the runner-up score and the query size. Changes nothing.
    pub fn recall_trace(&mut self, gq: &Grounded, raw: &crate::event::Event, true_id: u64) -> RecallTrace {
        let h = self.encode(gq);
        let mut scored: Vec<(u32, u32, u64, u32)> = Vec::new();
        for (id, d) in self.mem.top_k(&h, 32) {
            if let Some(st) = self.slots.get(&id) {
                let (a, tot) = aligned_agreement(raw, st);
                scored.push((a, tot, id, d));
            }
        }
        scored.sort_by(|a, b| b.0.cmp(&a.0).then(a.3.cmp(&b.3)));
        let true_rank = scored.iter().position(|x| x.2 == true_id);
        RecallTrace {
            candidates: scored.len(),
            true_rank,
            true_score: true_rank.map(|r| scored[r].0).unwrap_or(0),
            best: scored.first().map(|x| (x.2, x.0)),
            second_score: scored.get(1).map(|x| x.0).unwrap_or(0),
            query_tokens: scored.first().map(|x| x.1).unwrap_or(0),
        }
    }

    /// D030: the HDC memory proposes the nearest stored events; integer fact agreement verifies.
    /// Accept the unique best candidate if it leads the runner-up by at least two facts and
    /// explains at least half of the query's facts. Otherwise abstain.
    pub fn recall(&mut self, g: &Grounded) -> Option<Hit> {
        let h = self.encode(g);
        let floor = noise_floor::<512>(6);
        let q = facts(g);
        if q.is_empty() {
            return None;
        }
        let mut scored: Vec<(u32, u64, u32)> = self
            .mem
            .top_k(&h, 8)
            .into_iter()
            .filter(|&(_, d)| d <= floor)
            .map(|(id, d)| (self.tokens.get(&id).map(|t| overlap(&q, t)).unwrap_or(0), id, d))
            .collect();
        scored.sort_by(|a, b| b.0.cmp(&a.0).then(a.2.cmp(&b.2)));
        let best = *scored.first()?;
        let second = scored.get(1).map(|x| x.0).unwrap_or(0);
        if best.0 >= second + 2 && best.0 * 2 >= q.len() as u32 {
            Some(Hit { id: best.1, dist: best.2, margin: (best.0 - second) as u32 })
        } else {
            None
        }
    }

    /// Reconsolidation (D032): re-encode stored events with the current grounding. Event vectors
    /// written while concepts were immature are replaced; the raw events are never changed.
    /// D052: sleep-time consolidation of episodic memory. Every stored event is grounded again
    /// from its immutable raw record with the current concepts; if any grounding changed (an object
    /// that was unknown at storage time now has a concept, or a concept was merged, split or
    /// corrected), the event vectors are rebuilt (D032). Raw events are never modified. Returns the
    /// number of events whose grounding changed.
    pub fn consolidate(&mut self, g: &mut crate::ground::Grounder) -> u32 {
        let mut ids: Vec<u64> = self.slots.keys().copied().collect();
        ids.sort_unstable();
        let mut changed = 0u32;
        let mut regrounded: Vec<Grounded> = Vec::with_capacity(ids.len());
        for id in ids {
            let raw = g.store.get(id).clone();
            let now = g.ground(&raw);
            let before = &self.slots[&id];
            let key = |x: &Grounded| -> Vec<(u16, Option<u32>, Vec<(u16, i64)>)> { x.slots.iter().map(|s| (s.slot, s.concept, s.props.clone())).collect() };
            if key(&now) != key(before) {
                changed += 1;
            }
            regrounded.push(now);
        }
        if changed > 0 {
            self.reconsolidate(regrounded.iter());
        }
        changed
    }

    pub fn reconsolidate<'a, I: IntoIterator<Item = &'a Grounded>>(&mut self, groundings: I) {
        self.mem = CleanupMemory::with_z(6);
        self.tokens.clear();
        self.slots.clear();
        for g in groundings {
            self.store(g);
        }
    }

    /// Event-level pattern completion: recall the event, then return, for every query slot, the
    /// aligned stored slot (whose denoised properties restore missing or corrupted tokens).
    /// Log2-likelihood (Q16) of a raw query slot's tokens if it is the stored object `s`.
    fn slot_loglik(q: &[(u8, u16, i64)], s: &crate::ground::SlotGround, nm: &crate::ground::NoiseModel) -> i64 {
        let mut l = 0;
        for &(ph, c, v) in q {
            let (lm, lx, lu) = nm.terms(c);
            let stored = s.props.iter().find(|x| x.0 == c).map(|x| x.1).or_else(|| {
                let st = if ph == 0 { &s.pre_states } else { &s.post_states };
                st.iter().find(|x| x.0 == c).map(|x| x.1)
            });
            l += match stored {
                Some(x) if x == v => lm,
                Some(_) => lx,
                None => lu,
            };
        }
        l
    }

    /// D036: event-level completion gated by posteriors. The event posterior is computed over the
    /// proposed candidates, the slot posterior over the recalled event's objects, both from the
    /// measured noise model. A slot is completed only if P(event) * P(slot) >= 98%.
    pub fn restore_calibrated(
        &mut self,
        gq: &Grounded,
        raw: &crate::event::Event,
        nm: &crate::ground::NoiseModel,
    ) -> Option<(Hit, Vec<(u16, crate::ground::SlotGround)>)> {
        use crate::ground::FILL_POSTERIOR_Q16;
        use hdc_core::fixed::{best_posterior_q16, Q};
        let hit = self.recall_event(gq, raw)?;
        let qslots: Vec<(u16, Vec<(u8, u16, i64)>)> = {
            let mut v = raw.pre.slots();
            for s in raw.post.slots() {
                if !v.contains(&s) {
                    v.push(s);
                }
            }
            v.into_iter()
                .map(|s| {
                    let mut t: Vec<(u8, u16, i64)> = raw.pre.slot(s).into_iter().map(|(c, x)| (0, c, x)).collect();
                    t.extend(raw.post.slot(s).into_iter().map(|(c, x)| (1, c, x)));
                    (s, t)
                })
                .collect()
        };
        // event posterior over candidates: each candidate scored by its best alignment likelihood
        let h = self.encode(gq);
        let mut ids = Vec::new();
        let mut ll = Vec::new();
        for (id, _) in self.mem.top_k(&h, 32) {
            let Some(st) = self.slots.get(&id) else { continue };
            let (_, _, map) = align(raw, st);
            let mut l = 0;
            for (qs, toks) in &qslots {
                l += match map.iter().find(|m| m.0 == *qs).and_then(|m| st.slot(m.1)) {
                    Some(sg) => Self::slot_loglik(toks, sg, nm),
                    None => toks.iter().map(|t| nm.terms(t.1).2).sum(),
                };
            }
            ids.push(id);
            ll.push(l);
        }
        let (bi, p_event) = best_posterior_q16(&ll)?;
        if ids[bi] != hit.id {
            return Some((hit, Vec::new()));
        }
        let stored = self.slots.get(&hit.id)?.clone();
        let (_, _, map) = align(raw, &stored);
        let mut pairs = Vec::new();
        for (qs, ss) in map {
            let Some((_, toks)) = qslots.iter().find(|x| x.0 == qs) else { continue };
            let lls: Vec<i64> = stored.slots.iter().map(|g| Self::slot_loglik(toks, g, nm)).collect();
            let Some((si, p_slot)) = best_posterior_q16(&lls) else { continue };
            if stored.slots[si].slot != ss {
                continue;
            }
            let p_id = p_event * p_slot / Q;
            if p_id >= FILL_POSTERIOR_Q16 {
                let mut sg = stored.slots[si].clone();
                if sg.concept.is_none() {
                    // D036a: without a concept the stored value is one noisy observation; its own
                    // reliability (measured match probability of the channel) enters the posterior
                    sg.props.retain(|&(c, _)| p_id * hdc_core::fixed::exp2_neg_q16(-nm.terms(c).0) / Q >= FILL_POSTERIOR_Q16);
                }
                pairs.push((qs, sg));
            }
        }
        Some((hit, pairs))
    }

    pub fn restore(&mut self, gq: &Grounded, raw: &crate::event::Event) -> Option<(Hit, Vec<(u16, crate::ground::SlotGround)>)> {
        let hit = self.recall_event(gq, raw)?;
        let stored = self.slots.get(&hit.id)?.clone();
        let (_, _, map) = align(raw, &stored);
        // D030b: a query slot is completed from its aligned stored object only if that alignment
        // is unambiguous: it agrees on >= 1 token and strictly more than with any other object
        let mut pairs = Vec::new();
        for (qs, ss) in map {
            let mut t: Vec<(u8, u16, i64)> = raw.pre.slot(qs).into_iter().map(|(c, v)| (0, c, v)).collect();
            t.extend(raw.post.slot(qs).into_iter().map(|(c, v)| (1, c, v)));
            let scores: Vec<(u32, u16)> = stored.slots.iter().map(|g| (slot_agreement(&t, g), g.slot)).collect();
            let mine = scores.iter().find(|x| x.1 == ss).map(|x| x.0).unwrap_or(0);
            let others = scores.iter().filter(|x| x.1 != ss).map(|x| x.0).max().unwrap_or(0);
            if mine >= 1 && mine > others {
                if let Some(g) = stored.slot(ss) {
                    pairs.push((qs, g.clone()));
                }
            }
        }
        Some((hit, pairs))
    }

    pub fn len(&self) -> usize {
        self.mem.len()
    }

    pub fn is_empty(&self) -> bool {
        self.mem.is_empty()
    }

    pub fn bytes(&self) -> usize {
        self.mem.bytes()
    }
}
