extern crate alloc;

pub type BoxError<'i> = Box<dyn std::error::Error + Send + Sync + 'i>;
pub type BoxResult<'i, T> = Result<T, BoxError<'i>>;

pub use alloc::borrow::Cow;
#[cfg(feature = "std")]
pub use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};

pub use regex::Regex;
