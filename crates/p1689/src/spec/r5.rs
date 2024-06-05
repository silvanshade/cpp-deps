#[cfg(feature = "arbitrary")]
pub mod arbitrary;
#[cfg(feature = "builders")]
pub mod builders;
#[cfg(feature = "datagen")]
pub mod datagen;
#[cfg(test)]
pub mod proptest;
#[cfg(feature = "serde")]
pub mod serde;
#[cfg(feature = "winnow")]
pub mod winnow;

use alloc::borrow::Cow;
use core::borrow::Borrow;

use camino::Utf8Path;
use indexmap::IndexSet;
#[cfg(all(feature = "serde", any(feature = "deserialize", feature = "serialize")))]
use serde_with::serde_as;
#[cfg(all(feature = "serde", feature = "serialize"))]
use serde_with::skip_serializing_none;

#[cfg(all(feature = "serde", feature = "deserialize"))]
mod defaults {
    pub const fn bool<const V: bool>() -> bool {
        V
    }
}

#[cfg_attr(feature = "serde", cfg_eval::cfg_eval)]
#[cfg_attr(all(feature = "serde", feature = "serialize"), skip_serializing_none)]
#[cfg_attr(
    all(feature = "serde", any(feature = "deserialize", feature = "serialize")),
    serde_as,
    cfg_attr(feature = "deserialize", derive(::serde::Deserialize)),
    cfg_attr(feature = "serialize", derive(::serde::Serialize)),
    serde(rename_all = "kebab-case")
)]
#[cfg_attr(feature = "extra_traits", derive(Clone, Eq, PartialEq))]
#[cfg_attr(
    any(test, feature = "debug", feature = "arbitrary", feature = "extra_traits"),
    derive(Debug)
)]
#[non_exhaustive]
pub struct DepFile<'a> {
    pub version: u32,
    #[cfg_attr(
        all(feature = "serde", any(feature = "deserialize", feature = "serialize")),
        serde(default)
    )]
    pub revision: Option<u32>,
    #[cfg_attr(
        all(feature = "serde", any(feature = "deserialize", feature = "serialize")),
        serde(borrow),
        serde(skip_serializing_if = "IndexSet::is_empty"),
        serde_as(as = "IndexSet<self::serde::DepInfoUniqueOutputs>")
    )]
    pub rules: IndexSet<DepInfo<'a>>,
}
#[cfg(feature = "extra_traits")]
impl core::hash::Hash for DepFile<'_> {
    fn hash<H>(&self, state: &mut H)
    where
        H: core::hash::Hasher,
    {
        self.version.hash(state);
        self.revision.hash(state);
        self.rules.as_slice().hash(state);
    }
}
#[cfg(feature = "extra_traits")]
impl PartialOrd for DepFile<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
#[cfg(feature = "extra_traits")]
impl Ord for DepFile<'_> {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.version
            .cmp(&other.version)
            .then(self.revision.cmp(&other.revision))
            .then(self.rules.as_slice().cmp(other.rules.as_slice()))
    }
}

/// Dependency information for a compilation rule.
#[cfg_attr(feature = "serde", cfg_eval::cfg_eval)]
#[cfg_attr(all(feature = "serde", feature = "serialize"), skip_serializing_none)]
#[cfg_attr(
    all(feature = "serde", any(feature = "deserialize", feature = "serialize")),
    serde_as,
    cfg_attr(feature = "deserialize", derive(::serde::Deserialize)),
    cfg_attr(feature = "serialize", derive(::serde::Serialize)),
    serde(rename_all = "kebab-case")
)]
#[cfg_attr(feature = "extra_traits", derive(Clone))]
#[cfg_attr(
    any(test, feature = "debug", feature = "arbitrary", feature = "extra_traits"),
    derive(Debug)
)]
#[derive(Eq, PartialEq)]
#[non_exhaustive]
pub struct DepInfo<'a> {
    #[cfg_attr(
        all(feature = "serde", any(feature = "deserialize", feature = "serialize")),
        serde(default, borrow),
        serde_as(as = "Option<self::serde::CowUtf8Path>")
    )]
    pub work_directory: Option<Cow<'a, Utf8Path>>,
    /// The primary output for the compilation.
    #[cfg_attr(
        all(feature = "serde", any(feature = "deserialize", feature = "serialize")),
        serde(default, borrow),
        serde_as(as = "Option<self::serde::CowUtf8Path>")
    )]
    pub primary_output: Option<Cow<'a, Utf8Path>>,
    /// Other files output by a compiling this source using the same flags.
    #[cfg_attr(
        all(feature = "serde", any(feature = "deserialize", feature = "serialize")),
        serde(default, borrow),
        serde(skip_serializing_if = "IndexSet::is_empty"),
        serde_as(as = "IndexSet<self::serde::CowUtf8Path>")
    )]
    pub outputs: IndexSet<Cow<'a, Utf8Path>>,
    #[cfg_attr(
        all(feature = "serde", any(feature = "deserialize", feature = "serialize")),
        serde(default, borrow),
        serde(skip_serializing_if = "IndexSet::is_empty")
    )]
    pub provides: IndexSet<ProvidedModuleDesc<'a>>,
    #[cfg_attr(
        all(feature = "serde", any(feature = "deserialize", feature = "serialize")),
        serde(default, borrow),
        serde(skip_serializing_if = "IndexSet::is_empty")
    )]
    pub requires: IndexSet<RequiredModuleDesc<'a>>,
}
impl core::hash::Hash for DepInfo<'_> {
    fn hash<H>(&self, state: &mut H)
    where
        H: core::hash::Hasher,
    {
        self.work_directory.hash(state);
        self.primary_output.hash(state);
        self.outputs.as_slice().hash(state);
        self.provides.as_slice().hash(state);
        self.requires.as_slice().hash(state);
    }
}
#[cfg(feature = "extra_traits")]
impl PartialOrd for DepInfo<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
#[cfg(feature = "extra_traits")]
impl Ord for DepInfo<'_> {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.work_directory
            .cmp(&other.work_directory)
            .then(self.primary_output.cmp(&other.primary_output))
            .then(self.outputs.as_slice().cmp(other.outputs.as_slice()))
            .then(self.provides.as_slice().cmp(other.provides.as_slice()))
            .then(self.requires.as_slice().cmp(other.requires.as_slice()))
    }
}

#[cfg_attr(feature = "serde", cfg_eval::cfg_eval)]
#[cfg_attr(all(feature = "serde", feature = "serialize"), skip_serializing_none)]
#[cfg_attr(
    all(feature = "serde", any(feature = "deserialize", feature = "serialize")),
    serde_as,
    cfg_attr(feature = "deserialize", derive(::serde::Deserialize)),
    cfg_attr(feature = "serialize", derive(::serde::Serialize)),
    serde(rename_all = "kebab-case", untagged)
)]
#[cfg_attr(feature = "extra_traits", derive(Clone, Ord, PartialOrd))]
#[cfg_attr(
    any(test, feature = "debug", feature = "arbitrary", feature = "extra_traits"),
    derive(Debug)
)]
#[derive(Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum ModuleDesc<'a> {
    ByLogicalName {
        #[cfg_attr(
            all(feature = "serde", any(feature = "deserialize", feature = "serialize")),
            serde_as(as = "serde_with::BorrowCow")
        )]
        logical_name: Cow<'a, str>,
        #[cfg_attr(
            all(feature = "serde", any(feature = "deserialize", feature = "serialize")),
            serde(default),
            serde_as(as = "Option<self::serde::CowUtf8Path>")
        )]
        source_path: Option<Cow<'a, Utf8Path>>,
        #[cfg_attr(
            all(feature = "serde", any(feature = "deserialize", feature = "serialize")),
            serde(default),
            serde_as(as = "Option<self::serde::CowUtf8Path>")
        )]
        compiled_module_path: Option<Cow<'a, Utf8Path>>,
        /// Whether the module name is unique on `logical-name` or `source-path`.
        #[cfg(any(test, feature = "monostate"))]
        unique_on_source_path: Option<monostate::MustBeBool<false>>,
    },
    BySourcePath {
        #[cfg_attr(
            all(feature = "serde", any(feature = "deserialize", feature = "serialize")),
            serde_as(as = "serde_with::BorrowCow")
        )]
        logical_name: Cow<'a, str>,
        #[cfg_attr(
            all(feature = "serde", any(feature = "deserialize", feature = "serialize")),
            serde_as(as = "self::serde::CowUtf8Path")
        )]
        source_path: Cow<'a, Utf8Path>,
        #[cfg_attr(
            all(feature = "serde", any(feature = "deserialize", feature = "serialize")),
            serde(default),
            serde_as(as = "Option<self::serde::CowUtf8Path>")
        )]
        compiled_module_path: Option<Cow<'a, Utf8Path>>,
        /// Whether the module name is unique on `logical-name` or `source-path`.
        #[cfg(any(test, feature = "monostate"))]
        unique_on_source_path: monostate::MustBeBool<true>,
    },
}

impl<'a> ModuleDesc<'a> {
    #[must_use]
    pub fn view(&self) -> ModuleDescView {
        match *self {
            ModuleDesc::ByLogicalName {
                ref logical_name,
                ref source_path,
                ref compiled_module_path,
                ..
            } => ModuleDescView {
                key: logical_name.borrow(),
                unique_by: UniqueBy::LogicalName,
                source_path: source_path.as_deref(),
                compiled_module_path: compiled_module_path.as_deref(),
                logical_name: logical_name.borrow(),
            },
            ModuleDesc::BySourcePath {
                ref logical_name,
                ref source_path,
                ref compiled_module_path,
                ..
            } => ModuleDescView {
                key: source_path.as_str(),
                unique_by: UniqueBy::SourcePath,
                source_path: Some(source_path.borrow()),
                compiled_module_path: compiled_module_path.as_deref(),
                logical_name: logical_name.borrow(),
            },
        }
    }
}

/// Borrowed view of the common fields between the unique-by `ModuleDesc` variants.
#[cfg_attr(feature = "serde", cfg_eval::cfg_eval)]
#[cfg_attr(feature = "extra_traits", derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd))]
#[cfg_attr(
    any(feature = "debug", feature = "arbitrary", feature = "extra_traits"),
    derive(Debug)
)]
#[non_exhaustive]
pub struct ModuleDescView<'a> {
    pub key: &'a str,
    pub unique_by: UniqueBy,
    pub logical_name: &'a str,
    pub source_path: Option<&'a Utf8Path>,
    pub compiled_module_path: Option<&'a Utf8Path>,
}

#[cfg_attr(feature = "extra_traits", derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd))]
#[cfg_attr(
    any(feature = "debug", feature = "arbitrary", feature = "extra_traits"),
    derive(Debug)
)]
#[allow(clippy::exhaustive_enums)]
pub enum UniqueBy {
    LogicalName,
    SourcePath,
}

#[cfg_attr(feature = "serde", cfg_eval::cfg_eval)]
#[cfg_attr(all(feature = "serde", feature = "serialize"), skip_serializing_none)]
#[cfg_attr(
    all(feature = "serde", any(feature = "deserialize", feature = "serialize")),
    serde_as,
    cfg_attr(feature = "deserialize", derive(::serde::Deserialize)),
    cfg_attr(feature = "serialize", derive(::serde::Serialize)),
    serde(rename_all = "kebab-case")
)]
#[cfg_attr(feature = "extra_traits", derive(Clone, Ord, PartialOrd))]
#[cfg_attr(
    any(test, feature = "debug", feature = "arbitrary", feature = "extra_traits"),
    derive(Debug)
)]
#[derive(Eq, Hash, PartialEq)]
#[non_exhaustive]
pub struct ProvidedModuleDesc<'a> {
    #[cfg_attr(
        all(feature = "serde", any(feature = "deserialize", feature = "serialize")),
        serde(borrow, flatten)
    )]
    pub desc: ModuleDesc<'a>,
    #[cfg_attr(
        all(feature = "serde", any(feature = "deserialize", feature = "serialize")),
        serde(default = "defaults::bool::<true>")
    )]
    pub is_interface: bool,
}

#[cfg_attr(feature = "serde", cfg_eval::cfg_eval)]
#[cfg_attr(all(feature = "serde", feature = "serialize"), skip_serializing_none)]
#[cfg_attr(
    all(feature = "serde", any(feature = "deserialize", feature = "serialize")),
    serde_as,
    cfg_attr(feature = "deserialize", derive(::serde::Deserialize)),
    cfg_attr(feature = "serialize", derive(::serde::Serialize)),
    serde(rename_all = "kebab-case")
)]
#[cfg_attr(feature = "extra_traits", derive(Clone, Ord, PartialOrd))]
#[cfg_attr(
    any(test, feature = "debug", feature = "arbitrary", feature = "extra_traits"),
    derive(Debug)
)]
#[derive(Eq, Hash, PartialEq)]
#[non_exhaustive]
pub struct RequiredModuleDesc<'a> {
    #[cfg_attr(
        all(feature = "serde", any(feature = "deserialize", feature = "serialize")),
        serde(borrow, flatten)
    )]
    pub desc: ModuleDesc<'a>,
    #[cfg_attr(
        all(feature = "serde", any(feature = "deserialize", feature = "serialize")),
        serde(default)
    )]
    pub lookup_method: RequiredModuleDescLookupMethod,
}

#[cfg_attr(feature = "serde", cfg_eval::cfg_eval)]
#[cfg_attr(
    all(feature = "serde", any(feature = "deserialize", feature = "serialize")),
    serde_as,
    cfg_attr(feature = "deserialize", derive(::serde::Deserialize)),
    cfg_attr(feature = "serialize", derive(::serde::Serialize)),
    serde(rename_all = "kebab-case")
)]
#[cfg_attr(feature = "extra_traits", derive(Ord, PartialOrd))]
#[cfg_attr(
    any(test, feature = "debug", feature = "arbitrary", feature = "extra_traits"),
    derive(Debug)
)]
#[derive(Clone, Copy, Default, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum RequiredModuleDescLookupMethod {
    #[default]
    ByName,
    IncludeAngle,
    IncludeQuote,
}
#[cfg(feature = "extra_traits")]
impl core::fmt::Display for RequiredModuleDescLookupMethod {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let repr = match *self {
            Self::ByName => "\"by-name\"",
            Self::IncludeAngle => "\"include-angle\"",
            Self::IncludeQuote => "\"include-quote\"",
        };
        f.write_str(repr)
    }
}
