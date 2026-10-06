//! R0 worlds: the smallest relation-learning falsification test (EXPERIMENTS.md E-R0).
//!
//! Entities have four channels: colour, weight, shape, id. A pair action returns one outcome.
//! Train and test pools use disjoint colour and weight ranges, so held-out cases are
//! compositionally new.

use bm_relation::{context_of, Entity, Episode, Kind};
use hdc_core::rng::{mix64, Rng};

pub const COLOUR: u16 = 0;
pub const WEIGHT: u16 = 1;
pub const SHAPE: u16 = 2;
pub const ID: u16 = 3;
pub const TARGET: u32 = 0;
pub const ACTION: u16 = 1;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum R0Kind {
    /// outcome = colour(a) == colour(b)
    Equality,
    /// outcome = sign(weight(a) - weight(b))
    Order,
    /// outcome = fixed random bit per ordered pair identity: nothing reusable exists
    NoRelation,
    /// outcome = colour(a) == 3: a property, not a relation
    AbsClass,
    /// passive: hidden cause sets both colour equality and outcome; intervened: outcome random
    Confound,
    /// outcome = colour(a) != colour(b) (used as a changed world for negative-transfer tests)
    Inverted,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Ent {
    pub colour: i64,
    pub weight: i64,
    pub shape: i64,
    pub id: i64,
}

impl Ent {
    pub fn entity(&self) -> Entity {
        Entity::new(&[(COLOUR, self.colour), (WEIGHT, self.weight), (SHAPE, self.shape), (ID, self.id)])
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pool {
    Train,
    Test,
    /// role a from train, role b from test (compositional novelty for property laws)
    Mixed,
}

pub struct R0World {
    pub kind: R0Kind,
    pub train: Vec<Ent>,
    pub test: Vec<Ent>,
    pub context: u64,
    rng: Rng,
    salt: u64,
    t: u64,
}

impl R0World {
    pub fn new(kind: R0Kind, seed: u64, context_label: &str) -> Self {
        let mut rng = Rng::new(seed);
        let mut train = Vec::new();
        let mut test = Vec::new();
        for i in 0..40 {
            train.push(Ent {
                colour: rng.below(6) as i64,
                weight: 1 + rng.below(10) as i64,
                shape: rng.below(5) as i64,
                id: i,
            });
            test.push(Ent {
                colour: 6 + rng.below(4) as i64,
                weight: 11 + rng.below(10) as i64,
                shape: rng.below(5) as i64,
                id: 100 + i,
            });
        }
        // guarantee every train colour appears at least twice so equal pairs exist
        for (i, e) in train.iter_mut().enumerate().take(12) {
            e.colour = (i % 6) as i64;
        }
        R0World { kind, train, test, context: context_of(context_label), rng, salt: seed ^ 0xC0FFEE, t: 0 }
    }

    /// Ground truth (oracle). Never called by a learner.
    pub fn truth(&self, a: &Ent, b: &Ent) -> i64 {
        match self.kind {
            R0Kind::Equality => (a.colour == b.colour) as i64,
            R0Kind::Order => (a.weight - b.weight).signum(),
            R0Kind::NoRelation => (mix64(self.salt, (a.id as u64) * 1_000 + b.id as u64) & 1) as i64,
            R0Kind::AbsClass => (a.colour == 3) as i64,
            R0Kind::Confound => 0, // no stable truth under intervention
            R0Kind::Inverted => (a.colour != b.colour) as i64,
        }
    }

    fn pick(&mut self, pool: Pool) -> (Ent, Ent) {
        let (pa, pb) = match pool {
            Pool::Train => (&self.train, &self.train),
            Pool::Test => (&self.test, &self.test),
            Pool::Mixed => (&self.train, &self.test),
        };
        let i = self.rng.below(pa.len() as u64) as usize;
        let mut j = self.rng.below(pb.len() as u64) as usize;
        if pool != Pool::Mixed {
            while j == i {
                j = self.rng.below(pb.len() as u64) as usize;
            }
        }
        (pa[i], pb[j])
    }

    /// The agent chooses a pair (uniform random policy for R0) and acts.
    pub fn intervene(&mut self, pool: Pool) -> (Ent, Ent, i64) {
        let (a, b) = self.pick(pool);
        let out = match self.kind {
            R0Kind::Confound => self.rng.below(2) as i64,
            _ => self.truth(&a, &b),
        };
        (a, b, out)
    }

    /// The world acts on its own; the agent only watches.
    pub fn passive(&mut self) -> (Ent, Ent, i64) {
        match self.kind {
            R0Kind::Confound => {
                let h = self.rng.below(2) == 1;
                loop {
                    let (a, b) = self.pick(Pool::Train);
                    if (a.colour == b.colour) == h {
                        return (a, b, h as i64);
                    }
                }
            }
            _ => {
                let (a, b) = self.pick(Pool::Train);
                let o = self.truth(&a, &b);
                (a, b, o)
            }
        }
    }

    pub fn episode(&mut self, a: &Ent, b: &Ent, outcome: i64, kind: Kind) -> Episode {
        self.t += 1;
        Episode {
            id: 0,
            t: self.t,
            context: self.context,
            source: 0,
            action: ACTION,
            roles: vec![a.entity(), b.entity()],
            n_args: 0,
            outcomes: vec![(TARGET, outcome)],
            kind,
        }
    }
}
