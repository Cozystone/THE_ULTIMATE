//! observation-evidence held-out N-series (PREREG-observation-evidence section 2). Written after the
//! freeze of `bitmind-oe-v0.1`; none of these worlds existed while the learner was developed.
//!
//! Objects carry nominal FORM, GRADE, BAND, MARK, an ordinal HEIGHT and states OPEN and HUM. All
//! nominal values pass through a per-seed relabelling; MARKs are random without replacement.
//!
//! LINK(a, b) toggles b's OPEN:
//! * rule A: if form(a) = form(b);
//! * rule B: never, if grade(a) = grade(b) (a relational exception over both arguments);
//! * otherwise iff height(a) < height(b).
//! In the conflict region (same form and same grade) the rule that holds is drawn per seed.
//!
//! TUG(a, b) (N4) has LINK's mechanics, but b's OPEN after the action is read through a noisy
//! sensor (25% flipped). HUM(a) randomises a's HUM, a salient nuisance. WAIT changes nothing.
//!
//! Families:
//! * N0: noise-free;
//! * N1: look-alike groups (objects sharing every property but MARK and HEIGHT), 1% per-reading
//!   noise, mark misreads land on a look-alike's mark;
//! * N2: for a fixed third of the objects BAND always reads a fixed wrong value, plus 1% noise;
//! * N3: as N1, but no region pair is ever offered, and HUM is offered often;
//! * N4: noise-free LINK plus noisy-outcome TUG offers.

use bm_memory::{Act, Event, Kind, Scene, Token};
use bm_relation::context_of;
use hdc_core::rng::Rng;
use std::collections::HashMap;

pub const FORM: u16 = 30;
pub const GRADE: u16 = 31;
pub const BAND: u16 = 32;
pub const MARK: u16 = 33;
pub const HEIGHT: u16 = 34;
pub const OPEN: u16 = 35;
pub const HUMS: u16 = 36;
/// D049: HEIGHT is the only magnitude.
pub const ORDINAL: &[u16] = &[HEIGHT];

pub const WAIT: u16 = 0;
pub const LINK: u16 = 31;
pub const TUG: u16 = 32;
pub const HUM: u16 = 33;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Family {
    N0,
    N1,
    N2,
    N3,
    N4,
}

#[derive(Clone, Debug)]
pub struct Obj {
    pub form: i64,
    pub grade: i64,
    pub band: i64,
    pub mark: i64,
    pub height: i64,
    pub open: i64,
    pub hum: i64,
    /// look-alike group (N1, N3); objects of a group share form, grade and band
    pub group: usize,
    /// N2: BAND always reads this value
    pub band_bias: Option<i64>,
}

#[derive(Clone)]
pub struct NWorld {
    pub family: Family,
    pub objs: Vec<Obj>,
    pub context: u64,
    pub region_toggles: bool,
    noise_pct: u64,
    relabel: HashMap<(u16, i64), i64>,
    rng: Rng,
    vis_rng: Rng,
    noise_rng: Rng,
    pub t: u64,
    /// Grading-only record of corrupted readings since the last `take_misreads`.
    misreads: Vec<(usize, u16)>,
}

const FORMS: i64 = 6;
const GRADES: i64 = 8;
const BANDS: i64 = 5;

impl NWorld {
    pub fn new(family: Family, seed: u64) -> Self {
        let mut rng = Rng::new(seed ^ 0x9_0B5E);
        let n = 34 + rng.below(15) as usize;
        let region_toggles = rng.below(2) == 1;
        let lookalike = matches!(family, Family::N1 | Family::N3);
        let objs = loop {
            let mut marks: Vec<i64> = Vec::new();
            while marks.len() < n {
                let m = 20_000 + rng.below(980_000) as i64;
                if !marks.contains(&m) {
                    marks.push(m);
                }
            }
            let mut objs: Vec<Obj> = Vec::with_capacity(n);
            let mut i = 0;
            let mut group = 0;
            while i < n {
                // look-alike groups of 2-3 share form, grade and band
                let size = if lookalike { 2 + rng.below(2) as usize } else { 1 };
                let (f, g, b) = (rng.below(FORMS as u64) as i64, rng.below(GRADES as u64) as i64, rng.below(BANDS as u64) as i64);
                for _ in 0..size {
                    if i >= n {
                        break;
                    }
                    objs.push(Obj { form: f, grade: g, band: b, mark: marks[i], height: 1 + rng.below(12) as i64, open: rng.below(2) as i64, hum: rng.below(2) as i64, group, band_bias: None });
                    i += 1;
                }
                group += 1;
            }
            if family == Family::N2 {
                for (k, o) in objs.iter_mut().enumerate() {
                    if k % 3 == 0 {
                        o.band_bias = Some((o.band + 1 + (k as i64 % (BANDS - 1))) % BANDS);
                    }
                }
            }
            let region = region_of(&objs).len();
            let range = if lookalike { 16..=140 } else { 12..=60 };
            if range.contains(&region) {
                break objs;
            }
        };
        let mut relabel = HashMap::new();
        let mut rr = Rng::new(seed ^ 0xA_77C1);
        for (ch, k) in [(FORM, FORMS), (GRADE, GRADES), (BAND, BANDS)] {
            let vals: Vec<i64> = (0..k).collect();
            let mut img: Vec<i64> = (0..k).map(|x| 3_000 + 53 * x + rr.below(40) as i64).collect();
            rr.shuffle(&mut img);
            for (v, i) in vals.into_iter().zip(img) {
                relabel.insert((ch, v), i);
            }
        }
        let noise_pct = match family {
            Family::N1 | Family::N2 | Family::N3 => 1,
            _ => 0,
        };
        NWorld {
            family,
            objs,
            context: context_of(&format!("nseries-{family:?}")),
            region_toggles,
            noise_pct,
            relabel,
            vis_rng: Rng::new(seed ^ 0xB_51F7),
            noise_rng: Rng::new(seed ^ 0xC_A0A1),
            rng,
            t: 0,
            misreads: Vec::new(),
        }
    }

    pub fn region_pairs(&self) -> Vec<(usize, usize)> {
        region_of(&self.objs)
    }

    pub fn in_region(&self, a: usize, b: usize) -> bool {
        let (x, y) = (&self.objs[a], &self.objs[b]);
        x.form == y.form && x.grade == y.grade
    }

    /// Ground truth of LINK/TUG(a, b).
    pub fn effect(&self, a: usize, b: usize) -> bool {
        if self.in_region(a, b) {
            return self.region_toggles;
        }
        let (x, y) = (&self.objs[a], &self.objs[b]);
        if x.form == y.form {
            return true;
        }
        if x.grade == y.grade {
            return false;
        }
        x.height < y.height
    }

    fn read(&mut self, i: usize, ch: u16) -> i64 {
        let o = &self.objs[i];
        let mut v = match ch {
            FORM => o.form,
            GRADE => o.grade,
            BAND => o.band_bias.unwrap_or(o.band),
            MARK => o.mark,
            HEIGHT => o.height,
            OPEN => o.open,
            _ => o.hum,
        };
        if self.noise_pct > 0 && self.noise_rng.below(100) < self.noise_pct {
            self.misreads.push((i, ch));
            v = match ch {
                OPEN | HUMS => 1 - v,
                FORM => (v + 1 + self.noise_rng.below(FORMS as u64 - 1) as i64) % FORMS,
                GRADE => (v + 1 + self.noise_rng.below(GRADES as u64 - 1) as i64) % GRADES,
                BAND => (v + 1 + self.noise_rng.below(BANDS as u64 - 1) as i64) % BANDS,
                HEIGHT => 1 + self.noise_rng.below(12) as i64,
                _ => {
                    // a mark misread lands on a look-alike's mark when one exists
                    let g = self.objs[i].group;
                    let twins: Vec<usize> = (0..self.objs.len()).filter(|&k| k != i && self.objs[k].group == g).collect();
                    if twins.is_empty() {
                        self.objs[self.noise_rng.below(self.objs.len() as u64) as usize].mark
                    } else {
                        self.objs[twins[self.noise_rng.below(twins.len() as u64) as usize]].mark
                    }
                }
            };
        }
        *self.relabel.get(&(ch, v)).unwrap_or(&v)
    }

    fn emit(&mut self, i: usize, slot: u16, out: &mut Vec<Token>) {
        for ch in [FORM, GRADE, BAND, MARK, HEIGHT, OPEN, HUMS] {
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
            LINK | TUG => {
                if self.effect(args[0], args[1]) {
                    self.objs[args[1]].open ^= 1;
                }
            }
            HUM => self.objs[args[0]].hum = self.rng.below(2) as i64,
            _ => {}
        }
        let mut post = Vec::new();
        for &(s, i) in &truth {
            self.emit(i, s, &mut post);
        }
        if act == TUG && self.noise_rng.below(100) < 25 {
            // TUG's outcome sensor: b's OPEN after the action is misread 25% of the time
            let bslot = truth.iter().find(|x| x.1 == args[1]).expect("b visible").0;
            for tok in post.iter_mut().filter(|t| t.slot == bslot && t.ch == OPEN) {
                tok.val = if tok.val == self.shown(OPEN, 0) { self.shown(OPEN, 1) } else { self.shown(OPEN, 0) };
            }
            self.misreads.push((args[1], OPEN));
        }
        let arg_slots = args.iter().map(|a| truth.iter().find(|x| x.1 == *a).expect("arg visible").0).collect();
        let kind = if act == WAIT { Kind::Observation } else { Kind::Intervention };
        let ev = Event { id: 0, t: self.t, source: 43, context: self.context, pre: Scene { tokens: pre }, act: Some(Act { id: act, args: arg_slots }), post: Scene { tokens: post }, kind };
        (ev, truth)
    }

    fn shown(&self, ch: u16, v: i64) -> i64 {
        *self.relabel.get(&(ch, v)).unwrap_or(&v)
    }

    pub fn preview(&mut self, act: u16, args: Vec<usize>) -> (Event, Vec<(u16, usize)>) {
        let truth = self.scene_slots(&args);
        let mut pre = Vec::new();
        for &(s, i) in &truth {
            self.emit(i, s, &mut pre);
        }
        let arg_slots = args.iter().map(|a| truth.iter().find(|x| x.1 == *a).expect("arg visible").0).collect();
        let ev = Event { id: 0, t: self.t, source: 43, context: self.context, pre: Scene { tokens: pre.clone() }, act: Some(Act { id: act, args: arg_slots }), post: Scene { tokens: pre }, kind: Kind::Intervention };
        (ev, truth)
    }

    /// Candidate actions for this step:
    /// * N0-N2: 9 LINK pairs, 1 HUM, WAIT;
    /// * N3: 6 LINK pairs (never in the region), 4 HUM, WAIT;
    /// * N4: 5 LINK pairs and 4 TUG pairs (the TUG pairs reuse some LINK pairs), 1 HUM, WAIT.
    ///
    /// Pairs in `exclude` are never offered.
    pub fn offers(&mut self, exclude: &[(usize, usize)]) -> Vec<(u16, Vec<usize>)> {
        let n = self.objs.len();
        let (k_link, k_hum) = match self.family {
            Family::N3 => (6, 4),
            Family::N4 => (5, 1),
            _ => (9, 1),
        };
        let mut v: Vec<(u16, Vec<usize>)> = Vec::new();
        while v.len() < k_link {
            let p = self.rng.sample_distinct(n, 2);
            if exclude.contains(&(p[0], p[1])) || v.iter().any(|x| x.1 == p) {
                continue;
            }
            if self.family == Family::N3 && self.in_region(p[0], p[1]) {
                continue;
            }
            v.push((LINK, p));
        }
        if self.family == Family::N4 {
            for k in 0..4 {
                let p = if k < 2 {
                    v[k].1.clone()
                } else {
                    loop {
                        let p = self.rng.sample_distinct(n, 2);
                        if !exclude.contains(&(p[0], p[1])) {
                            break p;
                        }
                    }
                };
                v.push((TUG, p));
            }
        }
        for _ in 0..k_hum {
            v.push((HUM, vec![self.rng.below(n as u64) as usize]));
        }
        v.push((WAIT, vec![]));
        v
    }

    pub fn take_misreads(&mut self) -> Vec<(usize, u16)> {
        std::mem::take(&mut self.misreads)
    }

    pub fn rng(&mut self) -> &mut Rng {
        &mut self.rng
    }
}

fn region_of(objs: &[Obj]) -> Vec<(usize, usize)> {
    let mut v = Vec::new();
    for i in 0..objs.len() {
        for j in 0..objs.len() {
            if i != j && objs[i].form == objs[j].form && objs[i].grade == objs[j].grade {
                v.push((i, j));
            }
        }
    }
    v
}
