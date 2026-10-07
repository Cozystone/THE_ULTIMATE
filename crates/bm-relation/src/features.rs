//! Role-structured feature extraction.
//!
//! Absolute features bind role, channel and value. Relational features are computed by the
//! substrate's own group operations on the two role fillers (D006):
//! * XOR transform `T = val(a) ^ val(b)`: `T == 0` iff the fillers are equal (any value).
//! * permutation offset `k` with `rho^k(val(a)) == val(b)` for the ordinal value code.
//! No relation type is hand-coded per world; every channel and role pair is treated alike.
//! D049: the permutation-offset transforms (order, offset) exist only for channels whose
//! measurement level is ordinal; on nominal channels (names, identities, labels, hashes) only the
//! identity transform (same / different) is defined.

use crate::episode::Episode;
use hdc_core::{find_offset, ordinal, Hv16k, ItemMemory};
use std::collections::{BTreeSet, HashMap};

pub type H = Hv16k;
pub const W: usize = 256;

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum FeatureKind {
    /// role `role` has value `val` on channel `ch`.
    Abs { role: u8, ch: u16, val: i64 },
    /// transform between roles on `ch` is the identity (XOR difference is zero).
    Same { r1: u8, r2: u8, ch: u16 },
    /// transform is not the identity.
    Diff { r1: u8, r2: u8, ch: u16 },
    /// permutation offset from r1's value to r2's value has this sign.
    Order { r1: u8, r2: u8, ch: u16, sign: i8 },
    /// exact permutation offset.
    Delta { r1: u8, r2: u8, ch: u16, k: i64 },
}

impl FeatureKind {
    pub fn is_relational(&self) -> bool {
        !matches!(self, FeatureKind::Abs { .. })
    }

    pub fn describe(&self) -> String {
        match self {
            FeatureKind::Abs { role, ch, val } => format!("r{role}.c{ch}={val}"),
            FeatureKind::Same { r1, r2, ch } => format!("same(r{r1},r{r2};c{ch})"),
            FeatureKind::Diff { r1, r2, ch } => format!("diff(r{r1},r{r2};c{ch})"),
            FeatureKind::Order { r1, r2, ch, sign } => {
                format!("order(r{r1},r{r2};c{ch}){}", if *sign > 0 { "<" } else { ">" })
            }
            FeatureKind::Delta { r1, r2, ch, k } => format!("delta(r{r1},r{r2};c{ch})={k:+}"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Feature {
    pub kind: FeatureKind,
    /// Identity of the feature in the substrate.
    pub hv: H,
    /// For relational features: the transform code between the fillers (zero for identity).
    pub transform: Option<H>,
}

/// Deterministic codebook over the item memory.
pub struct Codebook {
    im: ItemMemory<W>,
    cache: HashMap<String, H>,
    /// Largest permutation offset searched for ordinal relations.
    pub max_offset: i64,
}

impl Codebook {
    pub fn new(seed: u64) -> Self {
        Codebook { im: ItemMemory::new(seed), cache: HashMap::new(), max_offset: 64 }
    }

    pub fn sym(&mut self, name: &str) -> H {
        if let Some(h) = self.cache.get(name) {
            return h.clone();
        }
        let h = self.im.get(name);
        self.cache.insert(name.to_string(), h.clone());
        h
    }

    pub fn role(&mut self, r: u8) -> H {
        self.sym(&format!("role:{r}"))
    }
    pub fn ch(&mut self, c: u16) -> H {
        self.sym(&format!("ch:{c}"))
    }
    pub fn base(&mut self, c: u16) -> H {
        self.sym(&format!("base:{c}"))
    }
    /// Ordinal group code of a channel value.
    pub fn val(&mut self, c: u16, v: i64) -> H {
        ordinal(&self.base(c), v)
    }
    pub fn action(&mut self, a: u16) -> H {
        self.sym(&format!("act:{a}"))
    }
    pub fn target(&mut self, t: u32) -> H {
        self.sym(&format!("target:{t}"))
    }
    pub fn outcome(&mut self, t: u32, v: i64) -> H {
        ordinal(&self.sym(&format!("outbase:{t}")), v)
    }

    pub fn feature_hv(&mut self, k: &FeatureKind) -> H {
        match *k {
            FeatureKind::Abs { role, ch, val } => self.role(role).bind(&self.ch(ch)).bind(&self.val(ch, val)),
            FeatureKind::Same { r1, r2, ch } => self.rel_frame("rel:same", r1, r2, ch),
            FeatureKind::Diff { r1, r2, ch } => self.rel_frame("rel:diff", r1, r2, ch),
            FeatureKind::Order { r1, r2, ch, sign } => {
                self.rel_frame("rel:order", r1, r2, ch).bind(&self.sym(if sign > 0 { "sign:+" } else { "sign:-" }))
            }
            FeatureKind::Delta { r1, r2, ch, k } => {
                self.rel_frame("rel:delta", r1, r2, ch).bind(&self.sym("deltabase").permute(k))
            }
        }
    }

    fn rel_frame(&mut self, rel: &str, r1: u8, r2: u8, ch: u16) -> H {
        self.sym(rel).bind(&self.role(r1)).bind(&self.role(r2).permute(1)).bind(&self.ch(ch))
    }

    /// Every role-structured feature of an episode.
    /// `ordinal`: channels on which order and offset are defined (D049).
    pub fn features(&mut self, ep: &Episode, ordinal: &BTreeSet<u16>) -> Vec<Feature> {
        let mut out = Vec::new();
        for (r, ent) in ep.roles.iter().enumerate() {
            for f in &ent.fillers {
                let kind = FeatureKind::Abs { role: r as u8, ch: f.ch, val: f.val };
                let hv = self.feature_hv(&kind);
                out.push(Feature { kind, hv, transform: None });
            }
        }
        let n_args = if ep.n_args == 0 { ep.roles.len() } else { ep.n_args as usize };
        for r1 in 0..ep.roles.len().min(n_args) {
            for r2 in (r1 + 1)..ep.roles.len() {
                for fa in &ep.roles[r1].fillers {
                    let Some(vb) = ep.roles[r2].get(fa.ch) else { continue };
                    let ch = fa.ch;
                    let (r1u, r2u) = (r1 as u8, r2 as u8);
                    let ha = self.val(ch, fa.val);
                    let hb = self.val(ch, vb);
                    // XOR group: identity test
                    let t = ha.bind(&hb);
                    let kind = if t.is_zero() {
                        FeatureKind::Same { r1: r1u, r2: r2u, ch }
                    } else {
                        FeatureKind::Diff { r1: r1u, r2: r2u, ch }
                    };
                    let hv = self.feature_hv(&kind);
                    out.push(Feature { kind, hv, transform: Some(t.clone()) });
                    // permutation group: offset mapping a's code onto b's code
                    if !t.is_zero() && ordinal.contains(&ch) {
                        if let Some(k) = find_offset(&ha, &hb, self.max_offset, 0) {
                            let sign = if k > 0 { 1 } else { -1 };
                            let ok = FeatureKind::Order { r1: r1u, r2: r2u, ch, sign };
                            let hv = self.feature_hv(&ok);
                            out.push(Feature { kind: ok, hv, transform: Some(ha.permute(k).bind(&ha)) });
                            let dk = FeatureKind::Delta { r1: r1u, r2: r2u, ch, k };
                            let hv = self.feature_hv(&dk);
                            out.push(Feature { kind: dk, hv, transform: Some(t) });
                        }
                    }
                }
            }
        }
        out
    }
}
