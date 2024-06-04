#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

mod spec;
mod util;

pub mod r5 {
    #[cfg(feature = "builders")]
    pub use crate::spec::r5::builders;
    #[cfg(all(test, feature = "arbitrary", feature = "extra_traits"))]
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

    #[cfg(feature = "winnow")]
    pub mod parsers {
        pub use crate::spec::r5::winnow::dep_file;
    }
}
