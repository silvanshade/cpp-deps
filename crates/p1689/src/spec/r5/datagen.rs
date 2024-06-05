pub mod graph;
#[cfg(all(feature = "serde", feature = "serialize"))]
pub mod json;

use alloc::boxed::Box;

pub type BoxError = Box<dyn std::error::Error + Send + Sync + 'static>;
pub type BoxResult<T> = Result<T, BoxError>;
