//! Language as I/O (ARCHITECTURE section 6). No language model.
//!
//! * Words are encoded from their characters (position-bound trigram profile over bytes), so a
//!   misspelled word stays close to the original.
//! * Grounding: cross-situational learning. Every utterance co-occurs with the grounded features
//!   of the object it is about; integer counts per (word, feature). A word's meaning is licensed
//!   only when it co-occurred with one feature in enough independent scenes and that feature
//!   explains nearly all of the word's uses (same discipline as relation licensing).
//! * Realizer: externalise a grounded object or a law with the licensed words; features without a
//!   licensed word are reported as `ch=value`.

use hdc_core::*;
use std::collections::{BTreeMap, HashMap, HashSet};

type H = Hv16k;

pub fn word_hv(im: &mut ItemMemory<256>, w: &str) -> H {
    let bytes: Vec<H> = w.bytes().map(|b| im.get(&format!("byte:{b}"))).collect();
    let tb = im.get("__tb");
    seq::ngram_profile(&bytes, 3, &tb)
}

#[derive(Clone, Debug, Default)]
pub struct WordStats {
    pub uses: u32,
    /// feature (ch, val) -> co-occurrence count
    pub co: BTreeMap<(u16, i64), u32>,
    /// independent scenes (signatures) per feature
    pub scenes: BTreeMap<(u16, i64), HashSet<u64>>,
}

pub struct Lexicon {
    im: ItemMemory<256>,
    pub words: BTreeMap<String, WordStats>,
    /// word vectors for robust lookup of misspellings
    mem: CleanupMemory<256>,
    ids: Vec<String>,
    pub min_scenes: u32,
}

impl Lexicon {
    pub fn new(seed: u64) -> Self {
        Lexicon { im: ItemMemory::new(seed), words: BTreeMap::new(), mem: CleanupMemory::with_z(5), ids: Vec::new(), min_scenes: 4 }
    }

    /// Hear an utterance about an object with grounded property features.
    pub fn hear(&mut self, utterance: &str, features: &[(u16, i64)], scene: u64) {
        for w in utterance.split_whitespace() {
            let w = w.to_lowercase();
            if !self.words.contains_key(&w) {
                let h = word_hv(&mut self.im, &w);
                self.mem.insert(self.ids.len() as u64, &h);
                self.ids.push(w.clone());
            }
            let st = self.words.entry(w).or_default();
            st.uses += 1;
            for &f in features {
                *st.co.entry(f).or_insert(0) += 1;
                st.scenes.entry(f).or_default().insert(scene);
            }
        }
    }

    /// Licensed meaning of a word: the feature present in >= 95% of its uses, in at least
    /// `min_scenes` independent scenes, and unique (no second feature equally consistent).
    pub fn meaning(&self, w: &str) -> Option<(u16, i64)> {
        let st = self.words.get(&w.to_lowercase())?;
        let mut best: Vec<((u16, i64), u32)> = st
            .co
            .iter()
            .filter(|(f, &c)| c * 100 >= st.uses * 95 && st.scenes.get(f).map(|s| s.len() as u32).unwrap_or(0) >= self.min_scenes)
            .map(|(f, &c)| (*f, c))
            .collect();
        best.sort_by(|a, b| b.1.cmp(&a.1));
        match best.as_slice() {
            [only] => Some(only.0),
            [a, b, ..] if a.1 > b.1 => Some(a.0),
            _ => None,
        }
    }

    /// Resolve a possibly misspelled word to a known word by its character profile.
    pub fn resolve(&mut self, w: &str) -> Option<String> {
        let w = w.to_lowercase();
        if self.words.contains_key(&w) {
            return Some(w);
        }
        let h = word_hv(&mut self.im, &w);
        self.mem.cleanup(&h).map(|hit| self.ids[hit.id as usize].clone())
    }

    /// Inverse lexicon: feature -> licensed word.
    pub fn word_for(&self) -> HashMap<(u16, i64), String> {
        let mut m = HashMap::new();
        for w in self.words.keys() {
            if let Some(f) = self.meaning(w) {
                m.insert(f, w.clone());
            }
        }
        m
    }

    /// Describe an object's features with licensed words (grammar: property words in channel
    /// order, unlicensed features as `ch=value`).
    pub fn describe(&self, features: &[(u16, i64)]) -> String {
        let inv = self.word_for();
        let mut f = features.to_vec();
        f.sort();
        f.iter()
            .map(|x| inv.get(x).cloned().unwrap_or_else(|| format!("c{}={}", x.0, x.1)))
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Understand a phrase: the conjunction of features its licensed words mean.
    pub fn understand(&mut self, phrase: &str) -> Vec<(u16, i64)> {
        let mut v = Vec::new();
        for w in phrase.split_whitespace() {
            if let Some(k) = self.resolve(w) {
                if let Some(f) = self.meaning(&k) {
                    v.push(f);
                }
            }
        }
        v.sort();
        v.dedup();
        v
    }
}
