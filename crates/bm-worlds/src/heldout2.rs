//! v0.2 held-out worlds H1-H5. Written after the `bitmind-v0.2` freeze; none of them existed while
//! the v0.2 learner was developed, and the learner may not change to pass them.
//!
//! One object world with interchangeable mechanisms. Objects carry nominal channels (KIND, COLOUR,
//! SHAPE, MARK), one ordinal channel (SIZE) and a state (ON). MARK labels are drawn at random
//! without replacement from a large range (no order or parity structure). A relabelling seed maps
//! every nominal value through a random bijection while leaving magnitudes, hidden attributes and
//! the event sequence unchanged (H3). Every scene shows the two arguments plus two bystanders in
//! shuffled slots.
//!
//! Mechanisms (`b` is the second argument; only its ON state can change):
//! * `Fit` (H1, H5): PLACE(a, b) flips b iff size(a) < size(b). An ordinal relation.
//! * `Rank` (H2): PROBE(a, b) flips b iff rank(a) = rank(b) + 1 (mod 3). Hidden ranks, cyclic,
//!   asymmetric: not an equivalence and not an anti-equivalence.
//! * `Spike` (H4): TOUCH(a, b) never flips b when shape(a) is SPIKE; otherwise it flips b iff
//!   colour(a) = colour(b) or size(a) > size(b). (Amendment 6: the size clause makes "a spike
//!   never flips" informative outside the conflict region, so the conflict can actually form.)
//!
//! Anti-leakage (asserted at construction): no non-unique visible channel determines a hidden
//! attribute.

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
/// D049: SIZE is the only magnitude.
pub const ORDINAL: &[u16] = &[SIZE];

pub const WAIT: u16 = 0;
pub const PLACE: u16 = 6;
pub const PROBE: u16 = 7;
pub const TOUCH: u16 = 8;
pub const TOGGLE: u16 = 9;

/// The SHAPE value (before relabelling) that marks a spike in the Spike mechanism.
pub const SPIKE: i64 = 3;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mechanism {
    Fit,
    Rank,
    Spike,
}

#[derive(Clone, Debug)]
pub struct Obj {
    pub kind: i64,
    pub colour: i64,
    pub shape: i64,
    pub mark: i64,
    pub size: i64,
    pub on: i64,
    /// hidden attribute (rank for `Rank`, unused otherwise)
    pub hidden: i64,
}

pub struct ObjWorld {
    pub mech: Mechanism,
    pub objs: Vec<Obj>,
    pub context: u64,
    /// nominal relabelling: (channel, true value) -> shown value
    relabel: HashMap<(u16, i64), i64>,
    rng: Rng,
    vis_rng: Rng,
    pub t: u64,
}

impl ObjWorld {
    /// `n` objects; sizes drawn from `size_range`. `relabel_seed` = 0 shows true labels.
    pub fn new(mech: Mechanism, seed: u64, n: usize, size_range: (i64, i64), relabel_seed: u64, label: &str) -> Self {
        let mut rng = Rng::new(seed);
        // MARK: random labels without replacement from 1000..9999
        let mut marks: Vec<i64> = Vec::new();
        while marks.len() < n {
            let m = 1000 + rng.below(9000) as i64;
            if !marks.contains(&m) {
                marks.push(m);
            }
        }
        let mut objs: Vec<Obj> = (0..n)
            .map(|i| Obj {
                kind: rng.below(3) as i64,
                colour: rng.below(5) as i64,
                shape: rng.below(4) as i64,
                mark: marks[i],
                size: size_range.0 + rng.below((size_range.1 - size_range.0 + 1) as u64) as i64,
                on: rng.below(2) as i64,
                hidden: 0,
            })
            .collect();
        if mech == Mechanism::Rank {
            // balanced random ranks, re-drawn until no non-unique visible channel determines them
            loop {
                let mut r: Vec<i64> = (0..n).map(|i| (i % 3) as i64).collect();
                rng.shuffle(&mut r);
                for (o, h) in objs.iter_mut().zip(r) {
                    o.hidden = h;
                }
                if !leaks(&objs) {
                    break;
                }
            }
        }
        assert!(!leaks(&objs), "anti-leakage check failed");
        let mut w = ObjWorld { mech, objs, context: context_of(label), relabel: HashMap::new(), rng, vis_rng: Rng::new(seed ^ 0x5157), t: 0 };
        if relabel_seed != 0 {
            let mut rr = Rng::new(relabel_seed);
            for ch in [KIND, COLOUR, SHAPE, MARK] {
                let mut vals: Vec<i64> = w.objs.iter().map(|o| w.true_value(o, ch)).collect();
                if ch == COLOUR {
                    vals = (0..5).collect();
                }
                if ch == SHAPE {
                    vals = (0..4).collect();
                }
                if ch == KIND {
                    vals = (0..3).collect();
                }
                vals.sort();
                vals.dedup();
                let mut img = if ch == MARK {
                    let mut v = Vec::new();
                    while v.len() < vals.len() {
                        let m = 10_000 + rr.below(90_000) as i64;
                        if !v.contains(&m) {
                            v.push(m);
                        }
                    }
                    v
                } else {
                    let mut v: Vec<i64> = vals.iter().map(|x| x + 100).collect();
                    rr.shuffle(&mut v);
                    v
                };
                if ch == MARK {
                    rr.shuffle(&mut img);
                }
                for (v, i) in vals.into_iter().zip(img) {
                    w.relabel.insert((ch, v), i);
                }
            }
        }
        w
    }

    fn true_value(&self, o: &Obj, ch: u16) -> i64 {
        match ch {
            KIND => o.kind,
            COLOUR => o.colour,
            SHAPE => o.shape,
            MARK => o.mark,
            SIZE => o.size,
            _ => o.on,
        }
    }

    fn shown(&self, o: &Obj, ch: u16) -> i64 {
        let v = self.true_value(o, ch);
        *self.relabel.get(&(ch, v)).unwrap_or(&v)
    }

    /// Ground truth of the mechanism for (a, b).
    pub fn effect(&self, a: usize, b: usize) -> bool {
        let (x, y) = (&self.objs[a], &self.objs[b]);
        match self.mech {
            Mechanism::Fit => x.size < y.size,
            Mechanism::Rank => x.hidden == (y.hidden + 1) % 3,
            Mechanism::Spike => x.shape != SPIKE && (x.colour == y.colour || x.size > y.size),
        }
    }

    pub fn action(&self) -> u16 {
        match self.mech {
            Mechanism::Fit => PLACE,
            Mechanism::Rank => PROBE,
            Mechanism::Spike => TOUCH,
        }
    }

    fn emit(&self, i: usize, slot: u16, out: &mut Vec<Token>) {
        let o = &self.objs[i];
        for ch in [KIND, COLOUR, SHAPE, MARK, SIZE, ON] {
            out.push(Token { slot, ch, val: self.shown(o, ch) });
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

    /// Apply `act` to `args` (WAIT: nuisance, a random bystander's state flips; TOGGLE(a): a flips).
    pub fn step(&mut self, act: u16, args: Vec<usize>) -> (Event, Vec<(u16, usize)>) {
        self.t += 1;
        let truth = self.scene_slots(&args);
        let mut pre = Vec::new();
        for &(s, i) in &truth {
            self.emit(i, s, &mut pre);
        }
        match act {
            WAIT => {
                let i = self.rng.below(self.objs.len() as u64) as usize;
                self.objs[i].on ^= (self.rng.below(4) == 0) as i64;
            }
            TOGGLE => {
                self.objs[args[0]].on ^= 1;
            }
            _ => {
                if self.effect(args[0], args[1]) {
                    self.objs[args[1]].on ^= 1;
                }
            }
        }
        let mut post = Vec::new();
        for &(s, i) in &truth {
            self.emit(i, s, &mut post);
        }
        let arg_slots = args.iter().map(|a| truth.iter().find(|x| x.1 == *a).expect("arg visible").0).collect();
        let kind = if act == WAIT { Kind::Observation } else { Kind::Intervention };
        let ev = Event { id: 0, t: self.t, source: 40, context: self.context, pre: Scene { tokens: pre }, act: Some(Act { id: act, args: arg_slots }), post: Scene { tokens: post }, kind };
        (ev, truth)
    }

    /// A scene for a query without acting (post = pre).
    pub fn preview(&mut self, act: u16, args: Vec<usize>) -> (Event, Vec<(u16, usize)>) {
        let truth = self.scene_slots(&args);
        let mut pre = Vec::new();
        for &(s, i) in &truth {
            self.emit(i, s, &mut pre);
        }
        let arg_slots = args.iter().map(|a| truth.iter().find(|x| x.1 == *a).expect("arg visible").0).collect();
        let ev = Event { id: 0, t: self.t, source: 40, context: self.context, pre: Scene { tokens: pre.clone() }, act: Some(Act { id: act, args: arg_slots }), post: Scene { tokens: pre }, kind: Kind::Intervention };
        (ev, truth)
    }

    pub fn rng(&mut self) -> &mut Rng {
        &mut self.rng
    }

    /// Add `k` new objects (never seen before) with sizes from `size_range` (H1 test objects).
    pub fn add_objects(&mut self, k: usize, size_range: (i64, i64)) -> Vec<usize> {
        let mut ids = Vec::new();
        for _ in 0..k {
            let mut m = 1000 + self.rng.below(9000) as i64;
            while self.objs.iter().any(|o| o.mark == m) {
                m = 1000 + self.rng.below(9000) as i64;
            }
            let o = Obj {
                kind: self.rng.below(3) as i64,
                colour: self.rng.below(5) as i64,
                shape: self.rng.below(4) as i64,
                mark: m,
                size: size_range.0 + self.rng.below((size_range.1 - size_range.0 + 1) as u64) as i64,
                on: self.rng.below(2) as i64,
                hidden: 0,
            };
            if !self.relabel.is_empty() {
                // a relabelled world shows new marks through a fresh injective image
                let img = 200_000 + m;
                self.relabel.insert((MARK, m), img);
            }
            self.objs.push(o);
            ids.push(self.objs.len() - 1);
        }
        ids
    }
}

/// True if some non-unique visible nominal channel (KIND, COLOUR, SHAPE) determines `hidden`.
fn leaks(objs: &[Obj]) -> bool {
    let distinct_hidden: std::collections::BTreeSet<i64> = objs.iter().map(|o| o.hidden).collect();
    if distinct_hidden.len() < 2 {
        return false;
    }
    for get in [|o: &Obj| o.kind, |o: &Obj| o.colour, |o: &Obj| o.shape] {
        let mut m: HashMap<i64, i64> = HashMap::new();
        let mut functional = true;
        for o in objs {
            if let Some(&h) = m.get(&get(o)) {
                if h != o.hidden {
                    functional = false;
                    break;
                }
            } else {
                m.insert(get(o), o.hidden);
            }
        }
        if functional {
            return true;
        }
    }
    false
}
