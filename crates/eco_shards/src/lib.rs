//! eco_shards - Typed Rust views over CSV shard schemas
//! 
//! This crate provides the main link between CSV shards and the core math,
//! making CSV the canonical "language" but keeping all math centralized.

pub mod response_turn;
pub mod eco_core_parameters;
pub mod biodegradable_substrate;

pub use response_turn::ResponseShardEcoTurn;
pub use eco_core_parameters::EcoCoreParameters;
pub use biodegradable_substrate::{SubstrateKineticsShard, SubstrateEcosafetyShard};
