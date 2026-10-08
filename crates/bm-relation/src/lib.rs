//! bm-relation: the relational substrate of BITMIND.
//!
//! Relations are group transforms between role fillers (D006). Every relation is a
//! `RelationLaw` that moves through CANDIDATE -> CONTESTED -> PROVISIONAL -> LICENSED ->
//! RESTRICTED | REVOKED | SPLIT, and may answer only when licensed in the query's context.
//!
//! Cognitive path rule: no floating point in this crate.

pub mod engine;
pub mod episode;
pub mod features;
pub mod latent;
pub mod law;

pub use engine::{noise_allowance, Abstain, Answer, CausalEdge, ConflictProbe, ObserveReport, ProbePair, RelationEngine, SleepStats, Uncertainty};
pub use latent::LatentInducer;
pub use episode::{context_of, Entity, Episode, EpisodeStore, Filler, Kind};
pub use features::{Codebook, Feature, FeatureKind, H};
pub use law::{LicensePolicy, Origin, RelationLaw, Status};
