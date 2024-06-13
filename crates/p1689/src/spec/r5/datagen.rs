#![cfg(not(tarpaulin_include))]

pub mod graph;
#[cfg(all(feature = "serde", feature = "serialize"))]
pub mod json;

use alloc::boxed::Box;

pub type BoxError = Box<dyn std::error::Error + Send + Sync + 'static>;
pub type BoxResult<T> = Result<T, BoxError>;

pub const CHACHA8RNG_SEED: u64 = 0xb6ab_77b5_bb6a_d6ab;
