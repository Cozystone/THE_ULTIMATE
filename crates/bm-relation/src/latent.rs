//! Latent-cause induction (Phase C).
//!
//! When an action's outcome on a pair of entities is not explained by any observable feature,
//! hypothesise a hidden entity attribute. Two general generators propose partitions of the
//! entities; each becomes a latent channel on the role fillers, and the ordinary licensing gates
//! decide whether any relation over it is real:
//!
//! * BLOCK: merge entities that are interchangeable (same outcome with every common partner, in
//!   both directions), keeping the partition block-consistent (all observed outcomes between two
//!   blocks agree). The outcome then becomes a function of (block_a, block_b).
//! * LINK: transitive closure of the pairs that produced the rarer outcome value (grouping by
//!   co-membership). Useful when the outcome is an equivalence over a hidden class.
//!
//! No relation type is added: the substrate's identity transform (`same`, `diff`) applies to latent
//! channels exactly as to observed ones. Hypotheses that do not earn licences are inert.

use crate::episode::{Episode, Filler};
use std::collections::{BTreeMap, BTreeSet};

/// D039e: earlier partition versions kept per generator.
const PREV_KEEP: usize = 2;

#[derive(Clone, Debug)]
pub struct LatentInducer {
    pub action: u16,
    pub target: u32,
    /// Channel that carries entity identity in role fillers.
    pub id_ch: u16,
    /// Channels written by this inducer: `block_ch`, `link_ch`.
    pub block_ch: u16,
    pub link_ch: u16,
    /// (a, b) -> histogram of outcome values.
    obs: BTreeMap<(i64, i64), BTreeMap<i64, u32>>,
    pub block: BTreeMap<i64, i64>,
    pub link: BTreeMap<i64, i64>,
    pub inductions: u32,
    pub min_common: u32,
    /// D039: partition versions. A changed partition is a different hypothesis, so it is written
    /// to a new channel id and laws about the old version stop matching.
    pub block_version: u16,
    pub link_version: u16,
    /// Consecutive inductions without any change (induction slows down once stable).
    pub stable_runs: u32,
    total: usize,
    /// D039e: the last earlier versions of each partition, (version, partition).
    prev_block: Vec<(u16, BTreeMap<i64, i64>)>,
    prev_link: Vec<(u16, BTreeMap<i64, i64>)>,
}

impl LatentInducer {
    pub fn new(action: u16, target: u32, id_ch: u16, block_ch: u16, link_ch: u16) -> Self {
        LatentInducer {
            action,
            target,
            id_ch,
            block_ch,
            link_ch,
            obs: BTreeMap::new(),
            block: BTreeMap::new(),
            link: BTreeMap::new(),
            inductions: 0,
            min_common: 2,
            block_version: 0,
            link_version: 0,
            stable_runs: 0,
            total: 0,
            prev_block: Vec::new(),
            prev_link: Vec::new(),
        }
    }

    fn ids(&self, ep: &Episode) -> Option<(i64, i64)> {
        if ep.roles.len() < 2 {
            return None;
        }
        Some((ep.roles[0].get(self.id_ch)?, ep.roles[1].get(self.id_ch)?))
    }

    pub fn add(&mut self, ep: &Episode) {
        if ep.action != self.action {
            return;
        }
        let (Some((a, b)), Some(o)) = (self.ids(ep), ep.outcome(self.target)) else { return };
        *self.obs.entry((a, b)).or_default().entry(o).or_insert(0) += 1;
        self.total += 1;
    }

    /// D040: measurement noise from repeatability. In a deterministic world the same intervention
    /// on the same pair must give the same outcome; minority outcomes over repeated pairs estimate
    /// the observation error rate. Returns (minority, total) over pairs seen at least twice.
    pub fn repeat_noise(&self) -> (u64, u64) {
        let (mut minority, mut total) = (0u64, 0u64);
        for h in self.obs.values() {
            let n: u32 = h.values().sum();
            if n < 2 {
                continue;
            }
            let maj = h.values().max().copied().unwrap_or(0);
            minority += (n - maj) as u64;
            total += n as u64;
        }
        (minority, total)
    }

    /// Total observations fed (drives the induction schedule).
    pub fn observations(&self) -> usize {
        self.total
    }

    fn outcome(&self, a: i64, b: i64) -> Option<i64> {
        let h = self.obs.get(&(a, b))?;
        h.iter().max_by_key(|x| (*x.1, -*x.0)).map(|x| *x.0)
    }

    fn entities(&self) -> BTreeSet<i64> {
        let mut s = BTreeSet::new();
        for &(a, b) in self.obs.keys() {
            s.insert(a);
            s.insert(b);
        }
        s
    }

    /// Re-induce both partitions from all observations so far.
    pub fn induce(&mut self) {
        let (old_block, old_link) = (self.block.clone(), self.link.clone());
        self.induce_inner();
        let changed_b = self.block != old_block;
        let changed_l = self.link != old_link;
        if std::env::var("DIAG_LAT").is_ok() && (changed_b || changed_l) {
            eprintln!("LAT obs {} block changed {changed_b} link changed {changed_l} link classes {} -> {}", self.total, distinct(&old_link), distinct(&self.link));
            if changed_l {
                eprintln!("LAT   old {:?}
LAT   new {:?}", old_link, self.link);
            }
        }
        if changed_b && !old_block.is_empty() {
            self.prev_block.push((self.block_version, old_block));
            if self.prev_block.len() > PREV_KEEP {
                self.prev_block.remove(0);
            }
            self.block_version += 1;
        }
        if changed_l && !old_link.is_empty() {
            self.prev_link.push((self.link_version, old_link));
            if self.prev_link.len() > PREV_KEEP {
                self.prev_link.remove(0);
            }
            self.link_version += 1;
        }
        if changed_b || changed_l {
            self.stable_runs = 0;
        } else {
            self.stable_runs += 1;
        }
    }

    /// Whether to re-induce after `n` observations: every 10 until stable 3 times, then every 50.
    pub fn due(&self, n: usize) -> bool {
        self.inductions == 0 || (if self.stable_runs >= 3 { n % 50 == 0 } else { n % 10 == 0 })
    }

    /// Current channel ids (versioned).
    pub fn channels(&self) -> (u16, u16) {
        (self.block_ch.wrapping_add(2 * self.block_version), self.link_ch.wrapping_add(2 * self.link_version))
    }

    fn induce_inner(&mut self) {
        self.inductions += 1;
        let ents: Vec<i64> = self.entities().into_iter().collect();
        // ---- BLOCK: greedy interchangeability merges with block consistency
        let mut class: BTreeMap<i64, i64> = ents.iter().map(|&e| (e, e)).collect();
        let mut cands: Vec<(u32, i64, i64)> = Vec::new();
        for i in 0..ents.len() {
            for j in (i + 1)..ents.len() {
                let (e1, e2) = (ents[i], ents[j]);
                let (mut agree, mut disagree) = (0u32, 0u32);
                for &p in &ents {
                    if p == e1 || p == e2 {
                        continue;
                    }
                    for (x, y) in [(self.outcome(e1, p), self.outcome(e2, p)), (self.outcome(p, e1), self.outcome(p, e2))] {
                        if let (Some(x), Some(y)) = (x, y) {
                            if x == y {
                                agree += 1;
                            } else {
                                disagree += 1;
                            }
                        }
                    }
                }
                if disagree == 0 && agree >= self.min_common {
                    cands.push((agree, e1, e2));
                }
            }
        }
        cands.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)).then(a.2.cmp(&b.2)));
        for (_, e1, e2) in cands {
            let (c1, c2) = (class[&e1], class[&e2]);
            if c1 == c2 {
                continue;
            }
            let mut trial = class.clone();
            for v in trial.values_mut() {
                if *v == c2 {
                    *v = c1;
                }
            }
            if self.block_consistent(&trial) {
                class = trial;
            }
        }
        self.block = class;
        // ---- LINK: closure of the rarer outcome value
        let mut counts: BTreeMap<i64, u32> = BTreeMap::new();
        for &(a, b) in self.obs.keys() {
            if let Some(o) = self.outcome(a, b) {
                *counts.entry(o).or_insert(0) += 1;
            }
        }
        let rare = counts.iter().min_by_key(|x| (*x.1, *x.0)).map(|x| *x.0);
        let mut link: BTreeMap<i64, i64> = ents.iter().map(|&e| (e, e)).collect();
        if counts.len() >= 2 {
            if let Some(rv) = rare {
                // D039a: closure is undirected, so both directions of a pair are pooled
                let sym = |a: i64, b: i64| -> bool {
                    let mut h: BTreeMap<i64, u32> = BTreeMap::new();
                    for k in [(a, b), (b, a)] {
                        if let Some(x) = self.obs.get(&k) {
                            for (&v, &c) in x {
                                *h.entry(v).or_insert(0) += c;
                            }
                        }
                    }
                    h.iter().max_by_key(|x| (*x.1, *x.0 == rv)).map(|x| *x.0) == Some(rv)
                };
                let pairs: Vec<(i64, i64)> = self.obs.keys().copied().filter(|&(a, b)| sym(a, b)).collect();
                for (a, b) in pairs {
                    let (ca, cb) = (link[&a], link[&b]);
                    if ca != cb {
                        let (keep, drop) = (ca.min(cb), ca.max(cb));
                        for v in link.values_mut() {
                            if *v == drop {
                                *v = keep;
                            }
                        }
                    }
                }
            }
        }
        self.link = link;
    }

    /// All observed outcomes between any two blocks agree.
    fn block_consistent(&self, class: &BTreeMap<i64, i64>) -> bool {
        let mut seen: BTreeMap<(i64, i64), i64> = BTreeMap::new();
        for (&(a, b), _) in &self.obs {
            let Some(o) = self.outcome(a, b) else { continue };
            let key = (class[&a], class[&b]);
            match seen.get(&key) {
                Some(&x) if x != o => return false,
                None => {
                    seen.insert(key, o);
                }
                _ => {}
            }
        }
        true
    }

    /// Whether any pair between the members of two classes of `part` was ever observed.
    fn observed_between(&self, part: &BTreeMap<i64, i64>, c1: i64, c2: i64) -> bool {
        self.obs.keys().any(|&(a, b)| {
            let (x, y) = (part.get(&a), part.get(&b));
            (x == Some(&c1) && y == Some(&c2)) || (x == Some(&c2) && y == Some(&c1))
        })
    }

    /// Add latent fillers to every role whose identity is known to the current partitions.
    /// D039b: for the action's argument pair, a difference between two classes is only exported
    /// if some pair between those classes was actually observed; an unestablished difference
    /// (absence of evidence) yields no latent filler for the pair, so no relation can apply.
    pub fn augment(&self, ep: &mut Episode) {
        if self.block.is_empty() {
            return;
        }
        let (bch, lch) = self.channels();
        let ids: Vec<Option<i64>> = ep.roles.iter().map(|r| r.get(self.id_ch)).collect();
        let pair_ok = |part: &BTreeMap<i64, i64>| -> bool {
            match (ids.first().copied().flatten(), ids.get(1).copied().flatten()) {
                (Some(a), Some(b)) => match (part.get(&a), part.get(&b)) {
                    // D039b/D039f: same or different, the relation between the two classes must
                    // have been observed (a block of interchangeable entities says nothing about
                    // the outcome between its own members until such a pair was seen)
                    (Some(&ca), Some(&cb)) => self.observed_between(part, ca, cb),
                    _ => true,
                },
                _ => true,
            }
        };
        let block_ok = pair_ok(&self.block);
        let link_ok = pair_ok(&self.link);
        for (i, r) in ep.roles.iter_mut().enumerate() {
            let Some(id) = ids[i] else { continue };
            let arg = i < 2;
            if let Some(&c) = self.block.get(&id) {
                if !arg || block_ok {
                    r.fillers.push(Filler { ch: bch, val: c });
                }
            }
            if let Some(&c) = self.link.get(&id) {
                if !arg || link_ok {
                    r.fillers.push(Filler { ch: lch, val: c });
                }
            }
        }
        // D039e: an earlier partition version still speaks for an argument pair whose relation
        // (same / different class) it shares with the current version; laws licensed on it keep
        // applying there, and only pairs whose relation changed must wait for new licences
        if let (Some(Some(a)), Some(Some(b))) = (ids.first().copied(), ids.get(1).copied()) {
            for (prev, cur, base) in [(&self.prev_block, &self.block, self.block_ch), (&self.prev_link, &self.link, self.link_ch)] {
                let (Some(ca), Some(cb)) = (cur.get(&a), cur.get(&b)) else { continue };
                for (v, m) in prev.iter() {
                    let (Some(&oa), Some(&ob)) = (m.get(&a), m.get(&b)) else { continue };
                    if (oa == ob) != (ca == cb) || !self.observed_between(m, oa, ob) {
                        continue;
                    }
                    let ch = base.wrapping_add(2 * v);
                    ep.roles[0].fillers.push(Filler { ch, val: oa });
                    ep.roles[1].fillers.push(Filler { ch, val: ob });
                }
            }
        }
    }

    pub fn classes(&self) -> (usize, usize) {
        let b: BTreeSet<i64> = self.block.values().copied().collect();
        let l: BTreeSet<i64> = self.link.values().copied().collect();
        (b.len(), l.len())
    }
}

fn distinct(m: &BTreeMap<i64, i64>) -> usize {
    m.values().collect::<std::collections::BTreeSet<_>>().len()
}
