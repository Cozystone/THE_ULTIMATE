//! hdc-core: the BITMIND substrate.
//!
//! Packed binary hypervectors with XOR bind/unbind, cyclic permutation, integer bundling,
//! popcount/Hamming kernels (scalar, POPCNT, AVX2, AVX-512 VPOPCNTDQ, NEON) selected at runtime,
//! deterministic item memory, level and ordinal codes, cleanup memory with a noise floor,
//! integer evidence and fixed-point information measures.
//!
//! Cognitive path rule: no floating point anywhere in this crate.

pub mod bundle;
pub mod cleanup;
pub mod evidence;
pub mod fixed;
pub mod hv;
pub mod item;
pub mod kernel;
pub mod level;
pub mod rng;
pub mod seq;

pub use bundle::{bundle, BitPlaneBundler, CounterBundler};
pub use cleanup::{CleanupMemory, Hit};
pub use evidence::Evidence;
pub use hv::{isqrt, noise_floor, Hv, Hv16k, Hv32k};
pub use item::ItemMemory;
pub use kernel::{active_kernel, KernelKind};
pub use level::{find_offset, ordinal, LevelCodebook};
pub use rng::Rng;
