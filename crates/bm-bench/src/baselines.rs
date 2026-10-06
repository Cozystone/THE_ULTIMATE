//! Non-relational baselines for the four-outcome test (ARCHITECTURE 2.3).
//! * `ClassLearner`  outcome 1: "these are similar examples"
//! * `LookupLearner` outcome 2: "this resembles something already stored"

use bm_relation::Episode;
use hdc_core::*;
use std::collections::HashMap;

type H = Hv16k;

/// Whole-episode hypervector: majority of role-bound entity vectors and the action.
pub struct EpisodeEncoder {
    im: ItemMemory<256>,
    /// Use similarity-preserving level codes for numeric channels (stronger baseline).
    pub level: Option<LevelCodebook<256>>,
}

impl EpisodeEncoder {
    pub fn new(seed: u64, level_codes: bool) -> Self {
        EpisodeEncoder {
            im: ItemMemory::new(seed),
            level: if level_codes { Some(LevelCodebook::linear(41, seed ^ 0x1E7E1)) } else { None },
        }
    }

    fn value(&mut self, ch: u16, v: i64) -> H {
        match &self.level {
            Some(l) if (0..=40).contains(&v) => l.get(v as usize).bind(&self.im.get(&format!("lvlch:{ch}"))),
            _ => self.im.get(&format!("v:{ch}:{v}")),
        }
    }

    pub fn encode(&mut self, ep: &Episode) -> H {
        let mut b = BitPlaneBundler::<256>::new();
        let tb = self.im.get("__tb");
        for (r, ent) in ep.roles.iter().enumerate() {
            let mut eb = BitPlaneBundler::<256>::new();
            for f in &ent.fillers {
                let c = self.im.get(&format!("ch:{}", f.ch));
                eb.add(&c.bind(&self.value(f.ch, f.val)));
            }
            let ent_hv = eb.majority(&tb);
            b.add(&self.im.get(&format!("role:{r}")).bind(&ent_hv));
        }
        b.add(&self.im.get(&format!("act:{}", ep.action)));
        b.majority(&tb)
    }
}

/// Prototype per outcome value; predicts the nearest prototype.
pub struct ClassLearner {
    enc: EpisodeEncoder,
    protos: HashMap<i64, CounterBundler<256>>,
}

impl ClassLearner {
    pub fn new(seed: u64, level_codes: bool) -> Self {
        ClassLearner { enc: EpisodeEncoder::new(seed, level_codes), protos: HashMap::new() }
    }

    pub fn train(&mut self, ep: &Episode, target: u32) {
        let h = self.enc.encode(ep);
        if let Some(v) = ep.outcome(target) {
            self.protos.entry(v).or_default().add(&h);
        }
    }

    pub fn predict(&mut self, ep: &Episode) -> Option<i64> {
        let h = self.enc.encode(ep);
        let mut best: Option<(i64, i64)> = None;
        let mut keys: Vec<&i64> = self.protos.keys().collect();
        keys.sort();
        for k in keys {
            let p = &self.protos[k];
            // normalise by class weight so frequent classes do not win by size
            let s = p.agreement(&h) / p.total_weight().max(1);
            if best.map(|b| s > b.1).unwrap_or(true) {
                best = Some((*k, s));
            }
        }
        best.map(|b| b.0)
    }
}

/// Exact memorised table plus nearest stored episode.
pub struct LookupLearner {
    enc: EpisodeEncoder,
    table: HashMap<u64, i64>,
    mem: CleanupMemory<256>,
    outs: Vec<i64>,
    pub exact_hits: u32,
}

impl LookupLearner {
    pub fn new(seed: u64) -> Self {
        LookupLearner {
            enc: EpisodeEncoder::new(seed, false),
            table: HashMap::new(),
            mem: CleanupMemory::new(),
            outs: Vec::new(),
            exact_hits: 0,
        }
    }

    pub fn train(&mut self, ep: &Episode, target: u32) {
        let Some(v) = ep.outcome(target) else { return };
        self.table.insert(ep.signature(), v);
        let h = self.enc.encode(ep);
        self.mem.insert(self.outs.len() as u64, &h);
        self.outs.push(v);
    }

    pub fn predict(&mut self, ep: &Episode) -> Option<i64> {
        if let Some(&v) = self.table.get(&ep.signature()) {
            self.exact_hits += 1;
            return Some(v);
        }
        let h = self.enc.encode(ep);
        self.mem.nearest_raw(&h).map(|hit| self.outs[hit.id as usize])
    }
}
