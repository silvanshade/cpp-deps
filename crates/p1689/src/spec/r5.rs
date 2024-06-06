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

use alloc::{borrow::Cow, vec::Vec};
use core::borrow::Borrow;

use camino::Utf8Path;
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
        serde(borrow)
    )]
    pub rules: Vec<DepInfo<'a>>,
}
#[cfg(test)]
impl DepFile<'_> {
    pub fn count_copies(&self) -> u64 {
        self.rules
            .as_slice()
            .iter()
            .map(|dep_info| dep_info.count_copies())
            .sum()
    }

    pub fn count_escapes_total(&self) -> u64 {
        self.rules
            .as_slice()
            .iter()
            .map(|dep_info| dep_info.count_escapes())
            .sum()
    }
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
        serde(skip_serializing_if = "Vec::is_empty"),
        serde_as(as = "Vec<self::serde::CowUtf8Path>")
    )]
    pub outputs: Vec<Cow<'a, Utf8Path>>,
    #[cfg_attr(
        all(feature = "serde", any(feature = "deserialize", feature = "serialize")),
        serde(default, borrow),
        serde(skip_serializing_if = "Vec::is_empty")
    )]
    pub provides: Vec<ProvidedModuleDesc<'a>>,
    #[cfg_attr(
        all(feature = "serde", any(feature = "deserialize", feature = "serialize")),
        serde(default, borrow),
        serde(skip_serializing_if = "Vec::is_empty")
    )]
    pub requires: Vec<RequiredModuleDesc<'a>>,
}
#[cfg(test)]
impl DepInfo<'_> {
    pub fn count_copies(&self) -> u64 {
        u64::from(
            self.work_directory
                .as_ref()
                .map(crate::util::cow_is_owned)
                .unwrap_or_default(),
        ) + u64::from(
            self.primary_output
                .as_ref()
                .map(crate::util::cow_is_owned)
                .unwrap_or_default(),
        ) + self
            .outputs
            .iter()
            .map(|output| u64::from(crate::util::cow_is_owned(output)))
            .sum::<u64>()
            + self
                .provides
                .iter()
                .map(|unique| unique.desc.count_copies())
                .sum::<u64>()
            + self
                .requires
                .iter()
                .map(|unique| unique.desc.count_copies())
                .sum::<u64>()
    }

    pub fn count_escapes(&self) -> u64 {
        self.work_directory
            .as_deref()
            .map(crate::util::count_escapes)
            .unwrap_or_default()
            + self
                .primary_output
                .as_deref()
                .map(crate::util::count_escapes)
                .unwrap_or_default()
            + self
                .outputs
                .iter()
                .map(|output| crate::util::count_escapes(output.as_str()))
                .sum::<u64>()
            + self
                .provides
                .iter()
                .map(|unique| unique.desc.count_escapes())
                .sum::<u64>()
            + self
                .requires
                .iter()
                .map(|unique| unique.desc.count_escapes())
                .sum::<u64>()
    }
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
    #[cfg_attr(
        all(feature = "serde", any(feature = "deserialize", feature = "serialize")),
        serde(rename_all = "kebab-case")
    )]
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
    #[cfg_attr(
        all(feature = "serde", any(feature = "deserialize", feature = "serialize")),
        serde(rename_all = "kebab-case")
    )]
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
    #[cfg(test)]
    pub fn count_copies(&self) -> u64 {
        match *self {
            Self::ByLogicalName {
                ref logical_name,
                ref source_path,
                ref compiled_module_path,
                ..
            } => {
                u64::from(crate::util::cow_is_owned(logical_name))
                    + u64::from(source_path.as_ref().map(crate::util::cow_is_owned).unwrap_or_default())
                    + u64::from(
                        compiled_module_path
                            .as_ref()
                            .map(crate::util::cow_is_owned)
                            .unwrap_or_default(),
                    )
            },
            Self::BySourcePath {
                ref logical_name,
                ref source_path,
                ref compiled_module_path,
                ..
            } => {
                u64::from(crate::util::cow_is_owned(logical_name))
                    + u64::from(crate::util::cow_is_owned(source_path))
                    + u64::from(
                        compiled_module_path
                            .as_ref()
                            .map(crate::util::cow_is_owned)
                            .unwrap_or_default(),
                    )
            },
        }
    }

    #[cfg(test)]
    pub fn count_escapes(&self) -> u64 {
        match *self {
            Self::ByLogicalName {
                ref logical_name,
                ref source_path,
                ref compiled_module_path,
                ..
            } => {
                crate::util::count_escapes(logical_name)
                    + source_path
                        .as_deref()
                        .map(crate::util::count_escapes)
                        .unwrap_or_default()
                    + compiled_module_path
                        .as_deref()
                        .map(crate::util::count_escapes)
                        .unwrap_or_default()
            },
            Self::BySourcePath {
                ref logical_name,
                ref source_path,
                ref compiled_module_path,
                ..
            } => {
                crate::util::count_escapes(logical_name)
                    + crate::util::count_escapes(source_path.as_str())
                    + compiled_module_path
                        .as_deref()
                        .map(crate::util::count_escapes)
                        .unwrap_or_default()
            },
        }
    }

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
#[cfg(feature = "arbitrary")]
impl<'a> ::arbitrary::Arbitrary<'a> for RequiredModuleDescLookupMethod {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        #[allow(clippy::same_functions_in_if_condition)]
        Ok(if u.arbitrary::<bool>()? {
            Self::ByName
        } else if u.arbitrary::<bool>()? {
            Self::IncludeAngle
        } else {
            Self::IncludeQuote
        })
    }
}
