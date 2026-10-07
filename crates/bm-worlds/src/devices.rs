//! Device worlds for Phase C causal tests. Every device is an object with property channels
//! KIND, COLOUR, MARK and a state channel ON. A hidden mechanism decides effects.
//!
//! * Confound: switch S, indicator I, lamp L, fan F, bell B. TOGGLE(S) flips S and L; TOGGLE(x)
//!   flips only x for the others. WAIT is passive: a hidden timer sets I and L to the same random
//!   value, so I and L are perfectly correlated under observation, yet I never causes L.
//! * Door: key K, door D (ON = open), power P, fan, bell. USE(K) flips D iff P is on.
//!   Phase 1: P is always on (the condition is invisible as a condition). Phase 2: TOGGLE(P) and
//!   a passive outage make P vary, exposing the hidden condition.
//! * Links: N tokens with KIND/COLOUR/MARK; hidden class. PROBE(a, b) changes b's ON iff
//!   class(a) == class(b) (same mechanism as hard links, in a microworld).

use bm_memory::{Act, Event, Kind, Scene, Token};
use bm_relation::context_of;
use hdc_core::rng::Rng;

pub const KIND: u16 = 0;
pub const COLOUR: u16 = 1;
pub const MARK: u16 = 2;
pub const ON: u16 = 3;
/// D049: no device channel is a magnitude.
pub const ORDINAL: &[u16] = &[];

pub const WAIT: u16 = 0;
pub const TOGGLE: u16 = 1;
pub const USE: u16 = 2;
pub const PROBE: u16 = 3;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Scenario {
    Confound,
    Door,
    Links,
}

#[derive(Clone, Debug)]
pub struct Device {
    pub kind: i64,
    pub colour: i64,
    pub mark: i64,
    pub on: i64,
    /// Hidden class (Links scenario only).
    pub class: i64,
}

#[derive(Clone)]
pub struct DeviceWorld {
    pub scenario: Scenario,
    pub devs: Vec<Device>,
    pub power_varies: bool,
    /// Door variant: USE is applied to any device, so the effect needs the conjunction
    /// "argument is the key AND power is on" (a hidden condition of two features).
    pub use_any: bool,
    pub context: u64,
    pub visible: usize,
    rng: Rng,
    pub t: u64,
}

pub const S: usize = 0;
pub const I: usize = 1;
pub const L: usize = 2;
pub const KEY: usize = 0;
pub const DOOR: usize = 1;
pub const POWER: usize = 2;

impl DeviceWorld {
    pub fn new(scenario: Scenario, seed: u64, label: &str) -> Self {
        let mut rng = Rng::new(seed);
        let mut devs = Vec::new();
        match scenario {
            Scenario::Confound | Scenario::Door => {
                for i in 0..5 {
                    devs.push(Device {
                        kind: 10 * (scenario as i64 + 1) + i,
                        colour: rng.below(6) as i64,
                        mark: 100 + i,
                        on: rng.below(2) as i64,
                        class: 0,
                    });
                }
                if scenario == Scenario::Door {
                    devs[POWER].on = 1;
                }
            }
            Scenario::Links => {
                // 16 tokens in 7 hidden classes (sizes 4,3,3,2,2,1,1)
                let classes = [0, 0, 0, 0, 1, 1, 1, 2, 2, 2, 3, 3, 4, 4, 5, 6];
                for (i, &c) in classes.iter().enumerate() {
                    devs.push(Device { kind: 50, colour: rng.below(4) as i64, mark: 200 + i as i64, on: 0, class: c });
                }
            }
        }
        let visible = if scenario == Scenario::Links { 4 } else { devs.len() };
        DeviceWorld { scenario, devs, power_varies: false, use_any: false, context: context_of(label), visible, rng, t: 0 }
    }

    pub fn rng(&mut self) -> &mut Rng {
        &mut self.rng
    }

    /// Apply an action (or passive dynamics for WAIT) to the true state.
    pub fn apply(&mut self, act: u16, args: &[usize]) {
        match (self.scenario, act) {
            (_, WAIT) => match self.scenario {
                Scenario::Confound => {
                    if self.rng.below(2) == 1 {
                        let v = self.rng.below(2) as i64;
                        self.devs[I].on = v;
                        self.devs[L].on = v;
                    }
                }
                Scenario::Door => {
                    if self.power_varies && self.rng.below(4) == 0 {
                        self.devs[POWER].on ^= 1;
                    }
                }
                Scenario::Links => {}
            },
            (Scenario::Confound, TOGGLE) => {
                let a = args[0];
                self.devs[a].on ^= 1;
                if a == S {
                    self.devs[L].on ^= 1;
                }
            }
            (Scenario::Door, USE) => {
                if args[0] == KEY && self.devs[POWER].on == 1 {
                    self.devs[DOOR].on ^= 1;
                }
            }
            (Scenario::Door, TOGGLE) => {
                let a = args[0];
                if a != POWER || self.power_varies {
                    self.devs[a].on ^= 1;
                }
            }
            (Scenario::Links, PROBE) => {
                let (a, b) = (args[0], args[1]);
                if self.devs[a].class == self.devs[b].class {
                    self.devs[b].on ^= 1;
                }
            }
            _ => {}
        }
    }

    fn emit(&self, idx: usize, slot: u16, out: &mut Vec<Token>) {
        let d = &self.devs[idx];
        for (ch, v) in [(KIND, d.kind), (COLOUR, d.colour), (MARK, d.mark), (ON, d.on)] {
            out.push(Token { slot, ch, val: v });
        }
    }

    /// One event. WAIT events are passive observations; others are interventions.
    pub fn step(&mut self, act: u16, args: Vec<usize>) -> (Event, Vec<(u16, usize)>) {
        self.t += 1;
        let mut vis: Vec<usize> = args.clone();
        let n = self.visible.min(self.devs.len());
        while vis.len() < n {
            let c = self.rng.below(self.devs.len() as u64) as usize;
            if !vis.contains(&c) {
                vis.push(c);
            }
        }
        let mut slots: Vec<u16> = (0..vis.len() as u16).collect();
        self.rng.shuffle(&mut slots);
        let truth: Vec<(u16, usize)> = vis.iter().enumerate().map(|(i, &d)| (slots[i], d)).collect();
        let mut pre = Vec::new();
        for &(s, d) in &truth {
            self.emit(d, s, &mut pre);
        }
        self.apply(act, &args);
        let mut post = Vec::new();
        for &(s, d) in &truth {
            self.emit(d, s, &mut post);
        }
        let arg_slots: Vec<u16> = args.iter().map(|a| truth.iter().find(|x| x.1 == *a).expect("arg visible").0).collect();
        let kind = if act == WAIT { Kind::Observation } else { Kind::Intervention };
        let ev = Event {
            id: 0,
            t: self.t,
            source: 30,
            context: self.context,
            pre: Scene { tokens: pre },
            act: Some(Act { id: act, args: arg_slots }),
            post: Scene { tokens: post },
            kind,
        };
        (ev, truth)
    }

    /// Look without acting: an event whose pre and post scenes are the current state, carrying
    /// the hypothetical action (used to build queries for option scoring).
    pub fn preview(&mut self, act: u16, args: Vec<usize>) -> (Event, Vec<(u16, usize)>) {
        let saved = (self.devs.clone(), self.t);
        let mut probe = self.clone();
        let (mut ev, truth) = probe.step(act, args);
        ev.post = ev.pre.clone();
        let _ = saved;
        // keep the world's rng advancing so previews do not replay the same scenes
        let _ = self.rng.next_u64();
        (ev, truth)
    }

    /// Ground truth of an intervention without changing the world (for do-queries and
    /// counterfactuals): the state after applying `act` to a copy.
    pub fn simulate(&self, act: u16, args: &[usize]) -> Vec<Device> {
        let mut c = self.clone();
        c.apply(act, args);
        c.devs
    }

    /// A random action for the scenario (agent's exploratory policy for Phase C).
    pub fn random_action(&mut self) -> (u16, Vec<usize>) {
        match self.scenario {
            Scenario::Confound => {
                if self.rng.below(3) == 0 {
                    (WAIT, vec![0])
                } else {
                    (TOGGLE, vec![self.rng.below(5) as usize])
                }
            }
            Scenario::Door => match self.rng.below(4) {
                0 => (WAIT, vec![0]),
                1 if self.use_any => (USE, vec![self.rng.below(5) as usize]),
                1 => (USE, vec![KEY]),
                _ => (TOGGLE, vec![self.rng.below(5) as usize]),
            },
            Scenario::Links => {
                let p = self.rng.sample_distinct(self.devs.len(), 2);
                (PROBE, vec![p[0], p[1]])
            }
        }
    }
}
