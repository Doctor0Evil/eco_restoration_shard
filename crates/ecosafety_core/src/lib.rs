//! ecosafety_core - Shared types and invariants for eco_restoration_shard
//! 
//! This crate defines the shared types and invariants; everything else depends on it.
//! It is non-actuating: it computes metrics and decisions only, leaving actuation to higher layers.

pub mod corridor;
pub mod residual;
pub mod ker;
pub mod invariants;

pub use corridor::CorridorBands;
pub use residual::Residual;
pub use ker::KerTriad;
pub use invariants::{corridor_present, safestep, ker_deployable};
