//! bm-memory: immutable event memory, symbol grounding and the bridge to relational learning.
//!
//! Cognitive path rule: no floating point in this crate.

pub mod bridge;
pub mod event;
pub mod ground;
pub mod recall;

pub use bridge::{decode_target, target_id, to_episode, to_episode_scene, scene_role_order, CHANGE, INST_CH};
pub use event::{Act, Event, EventStore, Kind, Scene, Token};
pub use ground::{cluster_quality, ChannelClass, ConceptEvent, ConceptStatus, GroundParams, Grounded, Grounder, Ident, NoiseModel, FILL_POSTERIOR_Q16, ObjectConcept, SlotGround};
pub use recall::{EventMemory, RecallTrace};
