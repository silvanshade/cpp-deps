#![no_std]
#![allow(unexpected_cfgs)]
#![deny(clippy::all)]
#![deny(clippy::cargo)]
#![deny(clippy::nursery)]
#![deny(clippy::pedantic)]
#![allow(clippy::inline_always)]
#![allow(clippy::match_ref_pats)]
#![allow(clippy::needless_borrowed_reference)]
#![allow(clippy::redundant_pub_crate)]
#![deny(clippy::alloc_instead_of_core)]
#![deny(clippy::allow_attributes_without_reason)]
#![deny(clippy::arithmetic_side_effects)]
#![deny(clippy::as_conversions)]
#![deny(clippy::clone_on_ref_ptr)]
#![deny(clippy::create_dir)]
#![deny(clippy::decimal_literal_representation)]
#![deny(clippy::default_numeric_fallback)]
#![deny(clippy::default_union_representation)]
#![deny(clippy::error_impl_error)]
#![deny(clippy::exhaustive_enums)]
#![deny(clippy::exhaustive_structs)]
#![deny(clippy::filetype_is_file)]
#![deny(clippy::if_then_some_else_none)]
#![deny(clippy::infinite_loop)]
#![deny(clippy::iter_over_hash_type)]
#![deny(clippy::mod_module_files)]
#![deny(clippy::mutex_atomic)]
#![deny(clippy::pattern_type_mismatch)]
#![deny(clippy::shadow_unrelated)]
#![deny(clippy::std_instead_of_alloc)]
#![deny(clippy::std_instead_of_core)]
#![deny(clippy::wildcard_enum_match_arm)]

extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

mod spec;
mod util;
pub mod vendor;

pub mod r5 {
    #[cfg(feature = "builders")]
    pub use crate::spec::r5::builders;
    #[cfg(feature = "datagen")]
    pub use crate::spec::r5::datagen;
    #[cfg(test)]
    pub use crate::spec::r5::proptest;
    pub use crate::spec::r5::{
        DepFile,
        DepInfo,
        ModuleDesc,
        ModuleDescView,
        ProvidedModuleDesc,
        RequiredModuleDesc,
        RequiredModuleDescLookupMethod,
        UniqueBy,
    };

    #[cfg(feature = "parsing")]
    pub mod parsers {
        pub use crate::{
            spec::r5::parsers::{dep_file, ErrorKind},
            util::parsers::{Error, ParseStream, State},
        };
    }

    pub use crate::vendor::camino::{Utf8Path, Utf8PathBuf};

    #[cfg(feature = "yoke")]
    pub mod yoke {
        pub use crate::spec::r5::yoke::{DepFileCart, DepFileYokeExt, DepInfoYokeExt};
        #[allow(clippy::module_name_repetitions)]
        pub use crate::spec::r5::yoke::{DepFileYoke, DepInfoNameYoke, DepInfoYoke};
    }
}
