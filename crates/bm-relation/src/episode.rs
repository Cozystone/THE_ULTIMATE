//! Immutable episodes with lineage, and an append-only store.

use hdc_core::rng::{fnv1a64, mix64};

/// How the episode came about. Only interventions can license a causal relation (gate 2).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Kind {
    /// The agent chose the action (efference copy available).
    Intervention,
    /// The agent only watched.
    Observation,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Filler {
    pub ch: u16,
    pub val: i64,
}

/// One role filler: an entity as a set of channel values.
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct Entity {
    pub fillers: Vec<Filler>,
    /// D050: identity of the bound object, supplied by the producer (a grounded concept, a world
    /// object). Used only to count evidence per relevant binding and to scope particular laws; it
    /// never becomes a feature. `None` falls back to a hash of the fillers.
    pub binding: Option<i64>,
}

impl Entity {
    pub fn new(pairs: &[(u16, i64)]) -> Self {
        Entity { fillers: pairs.iter().map(|&(ch, val)| Filler { ch, val }).collect(), binding: None }
    }

    /// An entity whose object identity is known to the producer.
    pub fn bound(id: i64, pairs: &[(u16, i64)]) -> Self {
        Entity { fillers: pairs.iter().map(|&(ch, val)| Filler { ch, val }).collect(), binding: Some(id) }
    }

    pub fn get(&self, ch: u16) -> Option<i64> {
        self.fillers.iter().find(|f| f.ch == ch).map(|f| f.val)
    }
}

#[derive(Clone, Debug)]
pub struct Episode {
    /// Assigned by the store.
    pub id: u64,
    pub t: u64,
    /// Observable context signature (where this happened).
    pub context: u64,
    /// Which adapter/world produced it (provenance).
    pub source: u32,
    pub action: u16,
    pub roles: Vec<Entity>,
    /// Number of leading roles that are action arguments; the rest are scene context
    /// (0 = every role is an argument). Relational features are computed only for role pairs that
    /// include an argument.
    pub n_args: u8,
    /// (target id, observed value).
    pub outcomes: Vec<(u32, i64)>,
    pub kind: Kind,
}

impl Episode {
    /// Fingerprint of action + every role filler. Two episodes with the same signature are
    /// copies of one pattern and count once for independence (gate 1).
    pub fn signature(&self) -> u64 {
        let mut h = mix64(self.action as u64, 0x5157);
        for (r, e) in self.roles.iter().enumerate() {
            let mut fs: Vec<&Filler> = e.fillers.iter().collect();
            fs.sort_by_key(|f| f.ch);
            for f in fs {
                h = mix64(h, filler_fp(r as u8, f.ch, f.val));
            }
        }
        h
    }

    /// D050: the action's argument roles.
    pub fn arg_roles(&self) -> Vec<u8> {
        let n = if self.n_args == 0 { self.roles.len() } else { (self.n_args as usize).min(self.roles.len()) };
        (0..n as u8).collect()
    }

    /// D050: identity of the object bound to role `r` (producer binding, else a filler hash).
    pub fn role_binding(&self, r: usize) -> u64 {
        match self.roles.get(r) {
            Some(e) => match e.binding {
                Some(b) => mix64(b as u64, 0xB1D0),
                None => {
                    let mut fs: Vec<&Filler> = e.fillers.iter().collect();
                    fs.sort_by_key(|f| (f.ch, f.val));
                    fs.iter().fold(0xB1D2u64, |h, f| mix64(h, filler_fp(0, f.ch, f.val)))
                }
            },
            None => 0,
        }
    }

    /// D050: key of the objects bound to `roles` (sorted, deduplicated by the caller).
    pub fn binding_key(&self, roles: &[u8]) -> u64 {
        let mut h = mix64(self.action as u64, 0xB1D1);
        for &r in roles {
            h = mix64(h, mix64(r as u64 + 1, self.role_binding(r as usize)));
        }
        h
    }

    /// Fingerprints of every (role, channel, value).
    pub fn filler_fps(&self) -> Vec<u64> {
        let mut v = Vec::new();
        for (r, e) in self.roles.iter().enumerate() {
            for f in &e.fillers {
                v.push(filler_fp(r as u8, f.ch, f.val));
            }
        }
        v
    }

    pub fn outcome(&self, target: u32) -> Option<i64> {
        self.outcomes.iter().find(|o| o.0 == target).map(|o| o.1)
    }

    /// Copy with outcomes removed (what a learner may see before predicting).
    pub fn without_outcomes(&self) -> Episode {
        let mut e = self.clone();
        e.outcomes.clear();
        e
    }
}

pub fn filler_fp(role: u8, ch: u16, val: i64) -> u64 {
    mix64(mix64(role as u64 + 1, ch as u64 + 0x100), val as u64 ^ 0xF111)
}

/// Context signature from a world-provided label (observable context token).
pub fn context_of(label: &str) -> u64 {
    fnv1a64(label.as_bytes())
}

/// Append-only episode store. Episodes are never modified after insertion.
#[derive(Clone, Debug, Default)]
pub struct EpisodeStore {
    eps: Vec<Episode>,
}

impl EpisodeStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn append(&mut self, mut ep: Episode) -> u64 {
        let id = self.eps.len() as u64;
        ep.id = id;
        self.eps.push(ep);
        id
    }

    pub fn get(&self, id: u64) -> &Episode {
        &self.eps[id as usize]
    }

    pub fn len(&self) -> usize {
        self.eps.len()
    }

    pub fn is_empty(&self) -> bool {
        self.eps.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Episode> {
        self.eps.iter()
    }
}
