//! Grounding microworld: anonymous, reshuffled, noisy slot percepts of persistent objects.

use bm_memory::{Act, Event, Kind, Scene, Token};
use bm_relation::context_of;
use hdc_core::rng::Rng;

pub const COLOUR: u16 = 0;
pub const SHAPE: u16 = 1;
pub const SIZE: u16 = 2;
pub const TEXTURE: u16 = 3;
pub const MARK: u16 = 4;
pub const LIT: u16 = 5;
pub const POS: u16 = 6;
pub const N_CH: u16 = 7;
/// D049: ordinal sensor channels (size and position are magnitudes; mark, colour, shape, texture
/// and the lit state are nominal).
pub const ORDINAL: &[u16] = &[SIZE, POS];

pub const IDLE: u16 = 0;
pub const TOGGLE: u16 = 1;
pub const MOVE: u16 = 2;
pub const COMBINE: u16 = 3;

/// Value range per channel (used for noise).
pub fn range(ch: u16) -> i64 {
    match ch {
        COLOUR => 12,
        SHAPE => 6,
        SIZE => 4,
        TEXTURE => 8,
        MARK => 64,
        LIT => 2,
        _ => 10,
    }
}

#[derive(Clone, Debug)]
pub struct GObj {
    pub props: [i64; 5],
    pub lit: i64,
    pub pos: i64,
    pub ty: u32,
}

impl GObj {
    pub fn get(&self, ch: u16) -> i64 {
        match ch {
            LIT => self.lit,
            POS => self.pos,
            c => self.props[c as usize],
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Setup {
    pub noise_pct: u64,
    pub miss_pct: u64,
    /// Percent of scenes that contain one junk slot of random tokens.
    pub junk_pct: u64,
    /// Object `obj` shows only `channels` (plus states) until tick `until`.
    pub partial: Option<(usize, Vec<u16>, u64)>,
    /// Channel hidden for every object until tick `until`.
    pub hidden_until: Option<(u16, u64)>,
    /// Objects that are never visible together.
    pub exclusive: Option<(usize, usize)>,
}

pub struct GroundWorld {
    pub objs: Vec<GObj>,
    pub active: Vec<usize>,
    pub setup: Setup,
    pub visible: usize,
    pub context: u64,
    rng: Rng,
    pub t: u64,
}

/// Truth for evaluation: slot -> object index (usize::MAX for junk).
pub type SlotTruth = Vec<(u16, usize)>;

pub const JUNK: usize = usize::MAX;

impl GroundWorld {
    /// `n_types` x `per_type` objects. Type fixes shape and texture; colour, size and mark vary.
    pub fn new(seed: u64, n_types: usize, per_type: usize, setup: Setup, label: &str) -> Self {
        let mut rng = Rng::new(seed);
        let mut objs = Vec::new();
        let marks = rng.sample_distinct(64, n_types * per_type);
        for ty in 0..n_types {
            for i in 0..per_type {
                objs.push(GObj {
                    props: [
                        rng.below(8) as i64,
                        ty as i64,
                        rng.below(4) as i64,
                        (ty * 2 + 1) as i64 % 8,
                        marks[ty * per_type + i] as i64,
                    ],
                    lit: rng.below(2) as i64,
                    pos: rng.below(10) as i64,
                    ty: ty as u32,
                });
            }
        }
        let active = (0..objs.len()).collect();
        GroundWorld { objs, active, setup, visible: 4, context: context_of(label), rng, t: 0 }
    }

    /// Make objects `a` and `b` identical on channels 0..4 and ensure no other object of the same
    /// type shares their (colour, size), so the pair is the only look-alike group.
    pub fn make_lookalike_pair(&mut self, a: usize, b: usize) {
        let p = self.objs[a].props;
        self.objs[b].props[..4].copy_from_slice(&p[..4]);
        for i in 0..self.objs.len() {
            if i == a || i == b {
                continue;
            }
            while self.objs[i].props[1] == p[1] && self.objs[i].props[0] == p[0] && self.objs[i].props[2] == p[2] {
                self.objs[i].props[0] = (self.objs[i].props[0] + 1) % 8;
            }
        }
    }

    /// True if no two objects share every property channel except `MARK`.
    pub fn lookalike_groups(&self) -> usize {
        let mut n = 0;
        for i in 0..self.objs.len() {
            for j in (i + 1)..self.objs.len() {
                if self.objs[i].props[..4] == self.objs[j].props[..4] {
                    n += 1;
                }
            }
        }
        n
    }

    /// Add an object (e.g. a novel object or a held-out colour) and make it visible.
    pub fn add(&mut self, o: GObj) -> usize {
        self.objs.push(o);
        let i = self.objs.len() - 1;
        self.active.push(i);
        i
    }

    pub fn rng(&mut self) -> &mut Rng {
        &mut self.rng
    }

    fn visible_set(&mut self) -> Vec<usize> {
        loop {
            let pick = self.rng.sample_distinct(self.active.len(), self.visible.min(self.active.len()));
            let v: Vec<usize> = pick.into_iter().map(|i| self.active[i]).collect();
            if let Some((a, b)) = self.setup.exclusive {
                if v.contains(&a) && v.contains(&b) {
                    continue;
                }
            }
            return v;
        }
    }

    fn emit(&mut self, obj: usize, slot: u16, out: &mut Vec<Token>) {
        for ch in 0..N_CH {
            if let Some((h, until)) = self.setup.hidden_until {
                if h == ch && self.t < until {
                    continue;
                }
            }
            if let Some((o, ref chans, until)) = self.setup.partial {
                if o == obj && self.t < until && ch < LIT && !chans.contains(&ch) {
                    continue;
                }
            }
            if self.rng.below(100) < self.setup.miss_pct {
                continue;
            }
            let mut v = self.objs[obj].get(ch);
            if self.rng.below(100) < self.setup.noise_pct {
                v = self.rng.below(range(ch) as u64) as i64;
            }
            out.push(Token { slot, ch, val: v });
        }
    }

    fn junk(&mut self, slot: u16, out: &mut Vec<Token>) {
        for ch in 0..N_CH {
            if self.rng.below(100) < 70 {
                out.push(Token { slot, ch, val: self.rng.below(range(ch) as u64) as i64 });
            }
        }
    }

    /// Apply an action to the true state.
    pub fn apply(&mut self, act: u16, args: &[usize]) {
        match act {
            TOGGLE => self.objs[args[0]].lit ^= 1,
            MOVE => self.objs[args[0]].pos = (self.objs[args[0]].pos + 1) % 10,
            COMBINE => {
                let same = (self.objs[args[0]].props[0] == self.objs[args[1]].props[0]) as i64;
                self.objs[args[0]].lit = same;
                self.objs[args[1]].lit = same;
            }
            _ => {}
        }
    }

    /// One event with a random action. `force` picks the action and its object arguments.
    pub fn step(&mut self, force: Option<(u16, Vec<usize>)>) -> (Event, SlotTruth) {
        self.t += 1;
        let mut vis = self.visible_set();
        if let Some((_, ref args)) = force {
            // forced arguments first, then the other visible objects, same scene size
            let n = vis.len().max(args.len());
            let mut v: Vec<usize> = args.clone();
            for o in vis {
                if v.len() < n && !v.contains(&o) {
                    v.push(o);
                }
            }
            vis = v;
        }
        let mut slots: Vec<u16> = (0..(vis.len() as u16 + 1)).collect();
        self.rng.shuffle(&mut slots);
        let mut truth: SlotTruth = vis.iter().enumerate().map(|(i, &o)| (slots[i], o)).collect();
        let junk_slot = if self.rng.below(100) < self.setup.junk_pct { Some(slots[vis.len()]) } else { None };
        let (act, arg_objs) = match force {
            Some(f) => f,
            None => {
                let a = self.rng.below(4) as u16;
                match a {
                    IDLE => (IDLE, vec![]),
                    COMBINE => {
                        let p = self.rng.sample_distinct(vis.len(), 2);
                        (COMBINE, vec![vis[p[0]], vis[p[1]]])
                    }
                    _ => (a, vec![vis[self.rng.below(vis.len() as u64) as usize]]),
                }
            }
        };
        let slot_of = |o: usize, truth: &SlotTruth| truth.iter().find(|x| x.1 == o).map(|x| x.0).unwrap_or_else(|| panic!("arg {o} not visible in {truth:?}"));
        let arg_slots: Vec<u16> = arg_objs.iter().map(|&o| slot_of(o, &truth)).collect();
        let mut pre = Vec::new();
        for &(s, o) in &truth {
            self.emit(o, s, &mut pre);
        }
        if let Some(js) = junk_slot {
            self.junk(js, &mut pre);
        }
        self.apply(act, &arg_objs);
        let mut post = Vec::new();
        for &(s, o) in &truth {
            self.emit(o, s, &mut post);
        }
        if let Some(js) = junk_slot {
            self.junk(js, &mut post);
            truth.push((js, JUNK));
        }
        let ev = Event {
            id: 0,
            t: self.t,
            source: 10,
            context: self.context,
            pre: Scene { tokens: pre },
            act: Some(Act { id: act, args: arg_slots }),
            post: Scene { tokens: post },
            kind: Kind::Intervention,
        };
        (ev, truth)
    }
}
