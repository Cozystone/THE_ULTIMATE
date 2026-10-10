//! capability-h4 held-out I-series worlds (PREREG-capability-h4 section 6). Written after the
//! freeze of `bitmind-capability-h4-v0.1`; none of them existed while the learner was developed.
//!
//! Objects carry nominal channels (MATERIAL, COATING, HUE, MARK), an ordinal WEIGHT and a state LIT.
//! All nominal values pass through a per-seed random relabelling; MARKs are random without
//! replacement; the object count is random.
//!
//! PRESS(a, b) toggles b's LIT:
//! * rule A: if material(a) = material(b);
//! * rule B: never, if b is coated (COATING = SEALED);
//! * otherwise iff weight(a) > weight(b).
//! The two rules disagree on the conflict region (b sealed, same material); the rule that holds
//! there is drawn per seed. Unlike CW and H4, the exception names the *second* argument.
//!
//! Other actions:
//! * SHAKE(a) randomly sets a's GLOW, a salient nuisance state with unpredictable outcomes (I2);
//! * TAP(a) toggles a's LIT;
//! * WAIT changes nothing.
//!
//! Variants:
//! * I1 is the base world;
//! * I2 adds frequent SHAKE offers;
//! * I3 never offers a region pair;
//! * I4 adds sensor noise on every channel.

use bm_memory::{Act, Event, Kind, Scene, Token};
use bm_relation::context_of;
use hdc_core::rng::Rng;
use std::collections::HashMap;

pub const MATERIAL: u16 = 10;
pub const COATING: u16 = 11;
pub const HUE: u16 = 12;
pub const MARK: u16 = 13;
pub const WEIGHT: u16 = 14;
pub const LIT: u16 = 15;
pub const GLOW: u16 = 16;
/// D049: WEIGHT is the only magnitude.
pub const ORDINAL: &[u16] = &[WEIGHT];

pub const WAIT: u16 = 0;
pub const PRESS: u16 = 21;
pub const SHAKE: u16 = 22;
pub const TAP: u16 = 23;

const SEALED: i64 = 1;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Variant {
    I1,
    I2,
    I3,
    I4,
}

#[derive(Clone, Debug)]
pub struct Obj {
    pub material: i64,
    pub coating: i64,
    pub hue: i64,
    pub mark: i64,
    pub weight: i64,
    pub lit: i64,
    pub glow: i64,
}

#[derive(Clone)]
pub struct IWorld {
    pub variant: Variant,
    pub objs: Vec<Obj>,
    pub context: u64,
    pub region_lights: bool,
    pub noise_pct: u64,
    relabel: HashMap<(u16, i64), i64>,
    rng: Rng,
    vis_rng: Rng,
    noise_rng: Rng,
    pub t: u64,
}

impl IWorld {
    pub fn new(variant: Variant, seed: u64) -> Self {
        let mut rng = Rng::new(seed ^ 0x1_5E21);
        let n = 36 + rng.below(13) as usize;
        let region_lights = rng.below(2) == 1;
        let objs = loop {
            let mut marks: Vec<i64> = Vec::new();
            while marks.len() < n {
                let m = 100_000 + rng.below(900_000) as i64;
                if !marks.contains(&m) {
                    marks.push(m);
                }
            }
            let sealed: Vec<usize> = rng.sample_distinct(n, 3);
            let objs: Vec<Obj> = (0..n)
                .map(|i| Obj {
                    material: rng.below(6) as i64,
                    coating: if sealed.contains(&i) { SEALED } else { 0 },
                    hue: rng.below(7) as i64,
                    mark: marks[i],
                    weight: 1 + rng.below(12) as i64,
                    lit: rng.below(2) as i64,
                    glow: rng.below(2) as i64,
                })
                .collect();
            let region = region_of(&objs).len();
            // at least 8 region pairs (>= 4 queries, >= 4 evidence pairs)
            if region >= 8 {
                break objs;
            }
        };
        let mut relabel = HashMap::new();
        let mut rr = Rng::new(seed ^ 0x2_7AB3);
        for (ch, k) in [(MATERIAL, 6i64), (COATING, 2), (HUE, 7)] {
            let vals: Vec<i64> = (0..k).collect();
            let mut img: Vec<i64> = (0..k).map(|x| 500 + 37 * x + rr.below(30) as i64).collect();
            rr.shuffle(&mut img);
            for (v, i) in vals.into_iter().zip(img) {
                relabel.insert((ch, v), i);
            }
        }
        let noise_pct = if variant == Variant::I4 { 1 } else { 0 };
        IWorld {
            variant,
            objs,
            context: context_of(&format!("iseries-{variant:?}")),
            region_lights,
            noise_pct,
            relabel,
            vis_rng: Rng::new(seed ^ 0x3_51F7),
            noise_rng: Rng::new(seed ^ 0x4_A0A1),
            rng,
            t: 0,
        }
    }

    pub fn region_pairs(&self) -> Vec<(usize, usize)> {
        region_of(&self.objs)
    }

    pub fn in_region(&self, a: usize, b: usize) -> bool {
        self.objs[b].coating == SEALED && self.objs[a].material == self.objs[b].material
    }

    /// Ground truth of PRESS(a, b): does b's LIT toggle?
    pub fn effect(&self, a: usize, b: usize) -> bool {
        if self.in_region(a, b) {
            return self.region_lights;
        }
        let (x, y) = (&self.objs[a], &self.objs[b]);
        if x.material == y.material {
            return true;
        }
        if y.coating == SEALED {
            return false;
        }
        x.weight > y.weight
    }

    fn true_value(o: &Obj, ch: u16) -> i64 {
        match ch {
            MATERIAL => o.material,
            COATING => o.coating,
            HUE => o.hue,
            MARK => o.mark,
            WEIGHT => o.weight,
            LIT => o.lit,
            _ => o.glow,
        }
    }

    fn read(&mut self, i: usize, ch: u16) -> i64 {
        let mut v = Self::true_value(&self.objs[i], ch);
        if self.noise_pct > 0 && self.noise_rng.below(100) < self.noise_pct {
            v = match ch {
                LIT | GLOW => 1 - v,
                COATING => 1 - v,
                MATERIAL => (v + 1 + self.noise_rng.below(5) as i64) % 6,
                HUE => (v + 1 + self.noise_rng.below(6) as i64) % 7,
                WEIGHT => 1 + self.noise_rng.below(12) as i64,
                _ => self.objs[self.noise_rng.below(self.objs.len() as u64) as usize].mark,
            };
        }
        *self.relabel.get(&(ch, v)).unwrap_or(&v)
    }

    fn emit(&mut self, i: usize, slot: u16, out: &mut Vec<Token>) {
        for ch in [MATERIAL, COATING, HUE, MARK, WEIGHT, LIT, GLOW] {
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

    pub fn step(&mut self, act: u16, args: Vec<usize>) -> (Event, Vec<(u16, usize)>) {
        self.t += 1;
        let truth = self.scene_slots(&args);
        let mut pre = Vec::new();
        for &(s, i) in &truth {
            self.emit(i, s, &mut pre);
        }
        match act {
            PRESS => {
                if self.effect(args[0], args[1]) {
                    self.objs[args[1]].lit ^= 1;
                }
            }
            SHAKE => self.objs[args[0]].glow = self.rng.below(2) as i64,
            TAP => self.objs[args[0]].lit ^= 1,
            _ => {}
        }
        let mut post = Vec::new();
        for &(s, i) in &truth {
            self.emit(i, s, &mut post);
        }
        let arg_slots = args.iter().map(|a| truth.iter().find(|x| x.1 == *a).expect("arg visible").0).collect();
        let kind = if act == WAIT { Kind::Observation } else { Kind::Intervention };
        let ev = Event { id: 0, t: self.t, source: 42, context: self.context, pre: Scene { tokens: pre }, act: Some(Act { id: act, args: arg_slots }), post: Scene { tokens: post }, kind };
        (ev, truth)
    }

    pub fn preview(&mut self, act: u16, args: Vec<usize>) -> (Event, Vec<(u16, usize)>) {
        let truth = self.scene_slots(&args);
        let mut pre = Vec::new();
        for &(s, i) in &truth {
            self.emit(i, s, &mut pre);
        }
        let arg_slots = args.iter().map(|a| truth.iter().find(|x| x.1 == *a).expect("arg visible").0).collect();
        let ev = Event { id: 0, t: self.t, source: 42, context: self.context, pre: Scene { tokens: pre.clone() }, act: Some(Act { id: act, args: arg_slots }), post: Scene { tokens: pre }, kind: Kind::Intervention };
        (ev, truth)
    }

    /// About 12 candidate actions for this step:
    /// * I1, I4: 9 PRESS pairs, 1 SHAKE, 1 TAP, WAIT;
    /// * I2: 6 PRESS pairs, 4 SHAKE, 1 TAP, WAIT;
    /// * I3: as I1, but no region pair is ever offered.
    ///
    /// Pairs in `exclude` (the saved queries) are never offered.
    pub fn offers(&mut self, exclude: &[(usize, usize)]) -> Vec<(u16, Vec<usize>)> {
        let n = self.objs.len();
        let (k_press, k_shake) = if self.variant == Variant::I2 { (6, 4) } else { (9, 1) };
        let mut v: Vec<(u16, Vec<usize>)> = Vec::new();
        while v.len() < k_press {
            let p = self.rng.sample_distinct(n, 2);
            if exclude.contains(&(p[0], p[1])) || v.iter().any(|x| x.1 == p) {
                continue;
            }
            if self.variant == Variant::I3 && self.in_region(p[0], p[1]) {
                continue;
            }
            v.push((PRESS, p));
        }
        for _ in 0..k_shake {
            v.push((SHAKE, vec![self.rng.below(n as u64) as usize]));
        }
        v.push((TAP, vec![self.rng.below(n as u64) as usize]));
        v.push((WAIT, vec![]));
        v
    }

    pub fn rng(&mut self) -> &mut Rng {
        &mut self.rng
    }
}

fn region_of(objs: &[Obj]) -> Vec<(usize, usize)> {
    let mut v = Vec::new();
    for i in 0..objs.len() {
        for j in 0..objs.len() {
            if i != j && objs[j].coating == SEALED && objs[i].material == objs[j].material {
                v.push((i, j));
            }
        }
    }
    v
}
