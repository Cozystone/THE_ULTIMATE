//! Deterministic seeded item memory: symbol -> atomic hypervector (D005).

use crate::hv::Hv;
use crate::rng::Rng;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct ItemMemory<const W: usize> {
    seed: u64,
    index: HashMap<String, u32>,
    names: Vec<String>,
    vecs: Vec<Hv<W>>,
}

impl<const W: usize> ItemMemory<W> {
    pub fn new(seed: u64) -> Self {
        ItemMemory { seed, index: HashMap::new(), names: Vec::new(), vecs: Vec::new() }
    }

    pub fn seed(&self) -> u64 {
        self.seed
    }

    /// Atomic vector for a symbol, independent of insertion order.
    pub fn atom(seed: u64, symbol: &str) -> Hv<W> {
        Hv::random(&mut Rng::for_symbol(seed, symbol))
    }

    /// Get (creating if needed) the id of a symbol.
    pub fn id(&mut self, symbol: &str) -> u32 {
        if let Some(&i) = self.index.get(symbol) {
            return i;
        }
        let i = self.vecs.len() as u32;
        self.vecs.push(Self::atom(self.seed, symbol));
        self.names.push(symbol.to_string());
        self.index.insert(symbol.to_string(), i);
        i
    }

    /// Get (creating if needed) the vector of a symbol.
    pub fn get(&mut self, symbol: &str) -> Hv<W> {
        let i = self.id(symbol);
        self.vecs[i as usize].clone()
    }

    pub fn lookup(&self, symbol: &str) -> Option<&Hv<W>> {
        self.index.get(symbol).map(|&i| &self.vecs[i as usize])
    }

    pub fn by_id(&self, id: u32) -> &Hv<W> {
        &self.vecs[id as usize]
    }

    pub fn name(&self, id: u32) -> &str {
        &self.names[id as usize]
    }

    pub fn len(&self) -> usize {
        self.vecs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.vecs.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = (u32, &str, &Hv<W>)> {
        self.vecs.iter().enumerate().map(move |(i, v)| (i as u32, self.names[i].as_str(), v))
    }
}
