//! Layer-0 input as immutable events: `pre state + action -> post state + time + provenance`.
//!
//! Scenes are anonymous: slots are perceptual positions, not identities, and slot numbering is
//! free to change between events. Within one event, slot `s` in `pre` and `post` refers to the
//! same thing (short-term tracking).

pub use bm_relation::Kind;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Token {
    pub slot: u16,
    pub ch: u16,
    pub val: i64,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Scene {
    pub tokens: Vec<Token>,
}

impl Scene {
    pub fn slots(&self) -> Vec<u16> {
        let mut s: Vec<u16> = self.tokens.iter().map(|t| t.slot).collect();
        s.sort_unstable();
        s.dedup();
        s
    }

    /// (channel, value) pairs seen in a slot.
    pub fn slot(&self, slot: u16) -> Vec<(u16, i64)> {
        let mut v: Vec<(u16, i64)> = self.tokens.iter().filter(|t| t.slot == slot).map(|t| (t.ch, t.val)).collect();
        v.sort_unstable();
        v
    }

    pub fn value(&self, slot: u16, ch: u16) -> Option<i64> {
        self.tokens.iter().find(|t| t.slot == slot && t.ch == ch).map(|t| t.val)
    }
}

/// Efference copy of the agent's own action (or an observed actor's action).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Act {
    pub id: u16,
    /// Argument slots in the pre scene.
    pub args: Vec<u16>,
}

#[derive(Clone, Debug)]
pub struct Event {
    pub id: u64,
    pub t: u64,
    /// Provenance: which adapter produced this event.
    pub source: u32,
    pub context: u64,
    pub pre: Scene,
    pub act: Option<Act>,
    pub post: Scene,
    pub kind: Kind,
}

/// Append-only event store. Events are never edited; every derived structure points back here.
#[derive(Clone, Debug, Default)]
pub struct EventStore {
    events: Vec<Event>,
}

impl EventStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn append(&mut self, mut e: Event) -> u64 {
        let id = self.events.len() as u64;
        e.id = id;
        self.events.push(e);
        id
    }

    pub fn get(&self, id: u64) -> &Event {
        &self.events[id as usize]
    }

    pub fn len(&self) -> usize {
        self.events.len()
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Event> {
        self.events.iter()
    }

    /// Approximate resident bytes (for resource accounting).
    pub fn bytes(&self) -> usize {
        self.events
            .iter()
            .map(|e| 96 + (e.pre.tokens.len() + e.post.tokens.len()) * std::mem::size_of::<Token>())
            .sum()
    }
}
