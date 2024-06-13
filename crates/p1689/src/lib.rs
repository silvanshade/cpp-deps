#![no_std]
#![allow(unexpected_cfgs)]
#![allow(clippy::result_unit_err)]

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
            spec::r5::parsers::dep_file,
            util::parsers::{ParseStream, State},
        };
    }

    pub use crate::vendor::camino::{Utf8Path, Utf8PathBuf};
}
