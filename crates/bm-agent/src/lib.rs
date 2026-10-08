//! bm-agent: layers 2-4 of BITMIND.
//!
//! * `policy`: active inference option scoring (risk, ambiguity, compute, information gain,
//!   preference recovery) over act / ask / wait options.
//!
//! Cognitive path rule: no floating point in this crate.

pub mod lang;
pub mod policy;
pub mod selfmodel;

pub use policy::{choose, choose_probe, record_outcome, score, Candidate, OptionKind, Preferences, ProbeRecord, Score, INF};
pub use selfmodel::{realize, Calibration, Conflict, Introspection, Resources, SelfModel};
pub use lang::Lexicon;
