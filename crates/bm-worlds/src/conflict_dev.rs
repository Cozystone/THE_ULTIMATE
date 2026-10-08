//! Capability-h4 development conflict world CW (PREREG-capability-h4 section 5). Development
//! data only; the held-out I-series is written after the freeze.
//!
//! Objects carry nominal channels (KIND, COLOUR, SHAPE, PATTERN, MARK), one ordinal channel
//! (SIZE) and a state (ON). MARK labels are drawn without replacement from a large range; PATTERN is
//! a nuisance property. Every emitted token is read through sensor noise: with probability
//! `noise_pct`% its value is replaced by a random value of its channel (ON is flipped).
//!
//! PROBE(a, b) flips b's ON:
//! * rule A: if colour(a) = colour(b);
//! * rule B: never, if shape(a) = CROWN;
//! * otherwise iff size(a) > size(b).
//! In the conflict region (a is a crown, b has a's colour) the two rules disagree. Which one holds
//! there is drawn per seed (`region_flips`). TOGGLE(a) flips a; WAIT changes nothing (bystander
//! noise only). A relabelling seed maps every nominal value through a random bijection.

use bm_memory::{Act, Event, Kind, Scene, Token};
use bm_relation::context_of;
use hdc_core::rng::Rng;
use std::collections::HashMap;

pub const KIND: u16 = 0;
pub const COLOUR: u16 = 1;
pub const SHAPE: u16 = 2;
pub const MARK: u16 = 3;
pub const SIZE: u16 = 4;
pub const ON: u16 = 5;
pub const PATTERN: u16 = 6;
/// D049: SIZE is the only magnitude.
pub const ORDINAL: &[u16] = &[SIZE];

pub const WAIT: u16 = 0;
pub const PROBE: u16 = 7;
pub const TOGGLE: u16 = 9;

/// The SHAPE value (before relabelling) of a crown.
pub const CROWN: i64 = 2;

#[derive(Clone, Debug)]
pub struct Obj {
    pub kind: i64,
    pub colour: i64,
    pub shape: i64,
    pub pattern: i64,
    pub mark: i64,
    pub size: i64,
    pub on: i64,
}

#[derive(Clone)]
pub struct ConflictWorld {
    pub objs: Vec<Obj>,
    pub context: u64,
    /// true: rule A holds in the conflict region (b flips); false: rule B holds (no flip)
    pub region_flips: bool,
    pub noise_pct: u64,
    relabel: HashMap<(u16, i64), i64>,
    rng: Rng,
    vis_rng: Rng,
    noise_rng: Rng,
    pub t: u64,
}

impl ConflictWorld {
    /// Objects: a random count in `n_range`; regenerated until the conflict region has at least
    /// `min_region` ordered pairs and at least 3 crowns of distinct colour partners exist.
    pub fn new(seed: u64, n_range: (usize, usize), noise_pct: u64, relabel_seed: u64, label: &str) -> Self {
        let mut rng = Rng::new(seed);
        let n = n_range.0 + rng.below((n_range.1 - n_range.0 + 1) as u64) as usize;
        let region_flips = rng.below(2) == 1;
        let objs = loop {
            let mut marks: Vec<i64> = Vec::new();
            while marks.len() < n {
                let m = 1000 + rng.below(9000) as i64;
                if !marks.contains(&m) {
                    marks.push(m);
                }
            }
            let objs: Vec<Obj> = (0..n)
                .map(|i| Obj {
                    kind: rng.below(3) as i64,
                    colour: rng.below(4) as i64,
                    shape: rng.below(4) as i64,
                    pattern: rng.below(6) as i64,
                    mark: marks[i],
                    size: 1 + rng.below(10) as i64,
                    on: rng.below(2) as i64,
                })
                .collect();
            let region = region_pairs_of(&objs).len();
            let crowns = objs.iter().filter(|o| o.shape == CROWN).count();
            if region >= 12 && crowns >= 3 && crowns * 3 <= n {
                break objs;
            }
        };
        let mut w = ConflictWorld {
            objs,
            context: context_of(label),
            region_flips,
            noise_pct,
            relabel: HashMap::new(),
            vis_rng: Rng::new(seed ^ 0x5157),
            noise_rng: Rng::new(seed ^ 0x401E),
            rng,
            t: 0,
        };
        if relabel_seed != 0 {
            let mut rr = Rng::new(relabel_seed);
            for (ch, k) in [(KIND, 3i64), (COLOUR, 4), (SHAPE, 4), (PATTERN, 6)] {
                let vals: Vec<i64> = (0..k).collect();
                let mut img: Vec<i64> = vals.iter().map(|x| x + 100).collect();
                rr.shuffle(&mut img);
                for (v, i) in vals.into_iter().zip(img) {
                    w.relabel.insert((ch, v), i);
                }
            }
            let marks: Vec<i64> = w.objs.iter().map(|o| o.mark).collect();
            let mut img: Vec<i64> = Vec::new();
            while img.len() < marks.len() {
                let m = 10_000 + rr.below(90_000) as i64;
                if !img.contains(&m) {
                    img.push(m);
                }
            }
            for (v, i) in marks.into_iter().zip(img) {
                w.relabel.insert((MARK, v), i);
            }
        }
        w
    }

    /// Ordered pairs (a, b) of the conflict region.
    pub fn region_pairs(&self) -> Vec<(usize, usize)> {
        region_pairs_of(&self.objs)
    }

    pub fn in_region(&self, a: usize, b: usize) -> bool {
        let (x, y) = (&self.objs[a], &self.objs[b]);
        x.shape == CROWN && x.colour == y.colour
    }

    /// Ground truth of PROBE(a, b).
    pub fn effect(&self, a: usize, b: usize) -> bool {
        let (x, y) = (&self.objs[a], &self.objs[b]);
        if self.in_region(a, b) {
            return self.region_flips;
        }
        if x.colour == y.colour {
            return true;
        }
        if x.shape == CROWN {
            return false;
        }
        x.size > y.size
    }

    fn true_value(o: &Obj, ch: u16) -> i64 {
        match ch {
            KIND => o.kind,
            COLOUR => o.colour,
            SHAPE => o.shape,
            PATTERN => o.pattern,
            MARK => o.mark,
            SIZE => o.size,
            _ => o.on,
        }
    }

    fn shown(&self, ch: u16, v: i64) -> i64 {
        *self.relabel.get(&(ch, v)).unwrap_or(&v)
    }

    /// One sensor reading of channel `ch` of object `i` (noise applied before relabelling).
    fn read(&mut self, i: usize, ch: u16) -> i64 {
        let mut v = Self::true_value(&self.objs[i], ch);
        if self.noise_pct > 0 && self.noise_rng.below(100) < self.noise_pct {
            v = match ch {
                ON => 1 - v,
                KIND => (v + 1 + self.noise_rng.below(2) as i64) % 3,
                COLOUR => (v + 1 + self.noise_rng.below(3) as i64) % 4,
                SHAPE => (v + 1 + self.noise_rng.below(3) as i64) % 4,
                PATTERN => (v + 1 + self.noise_rng.below(5) as i64) % 6,
                SIZE => 1 + self.noise_rng.below(10) as i64,
                _ => self.objs[self.noise_rng.below(self.objs.len() as u64) as usize].mark,
            };
        }
        self.shown(ch, v)
    }

    fn emit(&mut self, i: usize, slot: u16, out: &mut Vec<Token>) {
        for ch in [KIND, COLOUR, SHAPE, PATTERN, MARK, SIZE, ON] {
            let val = self.read(i, ch);
            out.push(Token { slot, ch, val });
        }
    }

    fn scene_slots(&mut self, args: &[usize]) -> Vec<(u16, usize)> {
        let n = self.objs.len();
        let mut vis: Vec<usize> = args.to_vec();
        while vis.len() < 4.min(n) {
            let c = self.vis_rng.below(n as u64) as usize;
            if !vis.contains(&c) {
                vis.push(c);
            }
        }
        let mut slots: Vec<u16> = (0..vis.len() as u16).collect();
        self.vis_rng.shuffle(&mut slots);
        vis.iter().enumerate().map(|(i, &o)| (slots[i], o)).collect()
    }

    /// Apply `act` to `args`. WAIT: nothing changes. TOGGLE(a): a flips. PROBE(a, b): see module doc.
    pub fn step(&mut self, act: u16, args: Vec<usize>) -> (Event, Vec<(u16, usize)>) {
        self.t += 1;
        let truth = self.scene_slots(&args);
        let mut pre = Vec::new();
        for &(s, i) in &truth {
            self.emit(i, s, &mut pre);
        }
        match act {
            TOGGLE => self.objs[args[0]].on ^= 1,
            PROBE => {
                if self.effect(args[0], args[1]) {
                    self.objs[args[1]].on ^= 1;
                }
            }
            _ => {}
        }
        let mut post = Vec::new();
        for &(s, i) in &truth {
            self.emit(i, s, &mut post);
        }
        let arg_slots = args.iter().map(|a| truth.iter().find(|x| x.1 == *a).expect("arg visible").0).collect();
        let kind = if act == WAIT { Kind::Observation } else { Kind::Intervention };
        let ev = Event { id: 0, t: self.t, source: 41, context: self.context, pre: Scene { tokens: pre }, act: Some(Act { id: act, args: arg_slots }), post: Scene { tokens: post }, kind };
        (ev, truth)
    }

    /// A scene for a query or a candidate action without acting (post = pre; noisy readings).
    pub fn preview(&mut self, act: u16, args: Vec<usize>) -> (Event, Vec<(u16, usize)>) {
        let truth = self.scene_slots(&args);
        let mut pre = Vec::new();
        for &(s, i) in &truth {
            self.emit(i, s, &mut pre);
        }
        let arg_slots = args.iter().map(|a| truth.iter().find(|x| x.1 == *a).expect("arg visible").0).collect();
        let ev = Event { id: 0, t: self.t, source: 41, context: self.context, pre: Scene { tokens: pre.clone() }, act: Some(Act { id: act, args: arg_slots }), post: Scene { tokens: pre }, kind: Kind::Intervention };
        (ev, truth)
    }

    /// Candidate actions offered this step: `k_probe` PROBE pairs drawn at random (never a pair in
    /// `exclude`), one TOGGLE of a random object and WAIT.
    pub fn offers(&mut self, k_probe: usize, exclude: &[(usize, usize)]) -> Vec<(u16, Vec<usize>)> {
        let n = self.objs.len();
        let mut v: Vec<(u16, Vec<usize>)> = Vec::new();
        while v.len() < k_probe {
            let p = self.rng.sample_distinct(n, 2);
            if exclude.contains(&(p[0], p[1])) || v.iter().any(|x| x.1 == p) {
                continue;
            }
            v.push((PROBE, p));
        }
        v.push((TOGGLE, vec![self.rng.below(n as u64) as usize]));
        v.push((WAIT, vec![]));
        v
    }

    pub fn rng(&mut self) -> &mut Rng {
        &mut self.rng
    }
}

fn region_pairs_of(objs: &[Obj]) -> Vec<(usize, usize)> {
    let mut v = Vec::new();
    for i in 0..objs.len() {
        for j in 0..objs.len() {
            if i != j && objs[i].shape == CROWN && objs[i].colour == objs[j].colour {
                v.push((i, j));
            }
        }
    }
    v
}
