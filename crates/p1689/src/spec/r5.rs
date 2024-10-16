#[cfg(feature = "builders")]
pub mod builders;
#[cfg(feature = "datagen")]
pub mod datagen;
#[cfg(feature = "parsing")]
pub mod parsers;
#[cfg(test)]
pub mod proptest;
#[cfg(feature = "serde")]
pub mod serde;
#[cfg(feature = "yoke")]
pub mod yoke;

use alloc::{borrow::Cow, vec::Vec};
use core::borrow::Borrow;

use crate::vendor::camino::Utf8Path;

#[cfg(all(feature = "serde", feature = "deserialize"))]
mod defaults {
    #[cfg(not(tarpaulin_include))]
    pub const fn bool<const V: bool>() -> bool {
        V
    }
}

#[cfg_attr(
    all(feature = "serde", any(feature = "deserialize", feature = "serialize")),
    cfg_attr(feature = "deserialize", derive(::serde::Deserialize)),
    cfg_attr(feature = "serialize", derive(::serde::Serialize)),
    serde(rename_all = "kebab-case")
)]
#[cfg_attr(feature = "extra_traits", derive(Clone, Eq, Hash, Ord, PartialEq, PartialOrd))]
#[cfg_attr(
    any(test, feature = "debug", feature = "arbitrary", feature = "extra_traits"),
    derive(Debug)
)]
#[non_exhaustive]
pub struct DepFile<'i> {
    pub version: u32,
    #[cfg_attr(
        all(feature = "serde", any(feature = "deserialize", feature = "serialize")),
        serde(default),
        serde(skip_serializing_if = "Option::is_none")
    )]
    pub revision: Option<u32>,
    #[cfg_attr(
        all(feature = "serde", any(feature = "deserialize", feature = "serialize")),
        serde(borrow)
    )]
    pub rules: Vec<DepInfo<'i>>,
}
#[cfg(test)]
impl DepFile<'_> {
    #[must_use]
    pub fn count_copies(&self) -> u64 {
        self.rules.as_slice().iter().map(DepInfo::count_copies).sum()
    }

    pub fn count_escapes_total(&self) -> u64 {
        self.rules.as_slice().iter().map(DepInfo::count_escapes).sum()
    }
}

/// Dependency information for a compilation rule.
#[cfg_attr(
    all(feature = "serde", any(feature = "deserialize", feature = "serialize")),
    cfg_attr(feature = "deserialize", derive(::serde::Deserialize)),
    cfg_attr(feature = "serialize", derive(::serde::Serialize)),
    serde(rename_all = "kebab-case")
)]
#[cfg_attr(feature = "extra_traits", derive(Clone, Eq, Hash, Ord, PartialEq, PartialOrd))]
#[cfg_attr(
    any(test, feature = "debug", feature = "arbitrary", feature = "extra_traits"),
    derive(Debug)
)]
#[non_exhaustive]
pub struct DepInfo<'i> {
    #[cfg_attr(
        all(feature = "serde", any(feature = "deserialize", feature = "serialize")),
        serde(default, borrow),
        serde(deserialize_with = "self::serde::deserialize::option_cow_utf8path"),
        serde(skip_serializing_if = "Option::is_none")
    )]
    pub work_directory: Option<Cow<'i, Utf8Path>>,
    /// The primary output for the compilation.
    #[cfg_attr(
        all(feature = "serde", any(feature = "deserialize", feature = "serialize")),
        serde(default, borrow),
        serde(deserialize_with = "self::serde::deserialize::option_cow_utf8path"),
        serde(skip_serializing_if = "Option::is_some")
    )]
    pub primary_output: Option<Cow<'i, Utf8Path>>,
    /// Other files output by a compiling this source using the same flags.
    #[cfg_attr(
        all(feature = "serde", any(feature = "deserialize", feature = "serialize")),
        serde(default, borrow),
        serde(deserialize_with = "self::serde::deserialize::vec_cow_utf8path"),
        serde(skip_serializing_if = "Vec::is_empty")
    )]
    pub outputs: Vec<Cow<'i, Utf8Path>>,
    #[cfg_attr(
        all(feature = "serde", any(feature = "deserialize", feature = "serialize")),
        serde(default, borrow),
        serde(skip_serializing_if = "Vec::is_empty")
    )]
    pub provides: Vec<ProvidedModuleDesc<'i>>,
    #[cfg_attr(
        all(feature = "serde", any(feature = "deserialize", feature = "serialize")),
        serde(default, borrow),
        serde(skip_serializing_if = "Vec::is_empty")
    )]
    pub requires: Vec<RequiredModuleDesc<'i>>,
}
#[cfg(test)]
impl DepInfo<'_> {
    #[must_use]
    #[allow(clippy::arithmetic_side_effects)]
    pub fn count_copies(&self) -> u64 {
        u64::from(self.work_directory.as_ref().is_some_and(crate::util::cow_is_owned))
            + u64::from(self.primary_output.as_ref().is_some_and(crate::util::cow_is_owned))
            + self
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

    #[must_use]
    #[allow(clippy::arithmetic_side_effects)]
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
                .map(|output| crate::util::count_escapes(output.as_ref()))
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

#[cfg_attr(
    all(feature = "serde", any(feature = "deserialize", feature = "serialize")),
    cfg_attr(feature = "deserialize", derive(::serde::Deserialize)),
    cfg_attr(feature = "serialize", derive(::serde::Serialize)),
    serde(rename_all = "kebab-case", untagged)
)]
#[cfg_attr(feature = "extra_traits", derive(Clone, Eq, Hash, Ord, PartialEq, PartialOrd))]
#[cfg_attr(
    any(test, feature = "debug", feature = "arbitrary", feature = "extra_traits"),
    derive(Debug)
)]
#[non_exhaustive]
pub enum ModuleDesc<'i> {
    #[cfg_attr(
        all(feature = "serde", any(feature = "deserialize", feature = "serialize")),
        serde(rename_all = "kebab-case")
    )]
    ByLogicalName {
        #[cfg_attr(
            all(feature = "serde", any(feature = "deserialize", feature = "serialize")),
            serde(borrow),
            serde(deserialize_with = "self::serde::deserialize::logical_name")
        )]
        logical_name: Cow<'i, str>,
        #[cfg_attr(
            all(feature = "serde", any(feature = "deserialize", feature = "serialize")),
            serde(default, borrow),
            serde(deserialize_with = "self::serde::deserialize::option_cow_utf8path"),
            serde(skip_serializing_if = "Option::is_none")
        )]
        source_path: Option<Cow<'i, Utf8Path>>,
        #[cfg_attr(
            all(feature = "serde", any(feature = "deserialize", feature = "serialize")),
            serde(default, borrow),
            serde(deserialize_with = "self::serde::deserialize::option_cow_utf8path"),
            serde(skip_serializing_if = "Option::is_none")
        )]
        compiled_module_path: Option<Cow<'i, Utf8Path>>,
        /// Whether the module name is unique on `logical-name` or `source-path`.
        #[cfg(any(test, feature = "monostate"))]
        #[cfg_attr(
            all(feature = "serde", feature = "serialize"),
            serde(skip_serializing_if = "Option::is_none")
        )]
        unique_on_source_path: Option<monostate::MustBeBool<false>>,
    },
    #[cfg_attr(
        all(feature = "serde", any(feature = "deserialize", feature = "serialize")),
        serde(rename_all = "kebab-case")
    )]
    BySourcePath {
        #[cfg_attr(
            all(feature = "serde", any(feature = "deserialize", feature = "serialize")),
            serde(borrow),
            serde(deserialize_with = "self::serde::deserialize::logical_name")
        )]
        logical_name: Cow<'i, str>,
        #[cfg_attr(
            all(feature = "serde", any(feature = "deserialize", feature = "serialize")),
            serde(borrow),
            serde(deserialize_with = "self::serde::deserialize::cow_utf8path")
        )]
        source_path: Cow<'i, Utf8Path>,
        #[cfg_attr(
            all(feature = "serde", any(feature = "deserialize", feature = "serialize")),
            serde(default, borrow),
            serde(deserialize_with = "self::serde::deserialize::option_cow_utf8path"),
            serde(skip_serializing_if = "Option::is_none")
        )]
        compiled_module_path: Option<Cow<'i, Utf8Path>>,
        /// Whether the module name is unique on `logical-name` or `source-path`.
        #[cfg(any(test, feature = "monostate"))]
        unique_on_source_path: monostate::MustBeBool<true>,
    },
}
impl<'i> ModuleDesc<'i> {
    #[cfg(test)]
    #[cfg(not(tarpaulin_include))]
    #[must_use]
    #[allow(clippy::arithmetic_side_effects)]
    pub fn count_copies(&self) -> u64 {
        match *self {
            Self::ByLogicalName {
                ref logical_name,
                ref source_path,
                ref compiled_module_path,
                ..
            } => {
                u64::from(crate::util::cow_is_owned(logical_name))
                    + u64::from(source_path.as_ref().is_some_and(crate::util::cow_is_owned))
                    + u64::from(compiled_module_path.as_ref().is_some_and(crate::util::cow_is_owned))
            },
            Self::BySourcePath {
                ref logical_name,
                ref source_path,
                ref compiled_module_path,
                ..
            } => {
                u64::from(crate::util::cow_is_owned(logical_name))
                    + u64::from(crate::util::cow_is_owned(source_path))
                    + u64::from(compiled_module_path.as_ref().is_some_and(crate::util::cow_is_owned))
            },
        }
    }

    #[cfg(test)]
    #[cfg(not(tarpaulin_include))]
    #[must_use]
    #[allow(clippy::arithmetic_side_effects)]
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
                    + crate::util::count_escapes(source_path.as_ref())
                    + compiled_module_path
                        .as_deref()
                        .map(crate::util::count_escapes)
                        .unwrap_or_default()
            },
        }
    }

    #[must_use]
    pub fn logical_name(&self) -> Cow<'i, str> {
        match *self {
            ModuleDesc::BySourcePath { ref logical_name, .. } | ModuleDesc::ByLogicalName { ref logical_name, .. } => {
                logical_name.clone()
            },
        }
    }

    #[must_use]
    pub fn view(&self) -> ModuleDescView {
        match *self {
            #[rustfmt::skip]
            ModuleDesc::ByLogicalName { ref logical_name, ref source_path, ref compiled_module_path, .. } => ModuleDescView {
                key: logical_name.borrow(),
                unique_by: UniqueBy::LogicalName,
                source_path: source_path.as_deref(),
                compiled_module_path: compiled_module_path.as_deref(),
                logical_name: logical_name.borrow(),
            },
            #[rustfmt::skip]
            ModuleDesc::BySourcePath { ref logical_name, ref source_path, ref compiled_module_path, .. } => ModuleDescView {
                #[allow(clippy::useless_asref)]
                key: source_path.as_ref().as_ref(),
                unique_by: UniqueBy::SourcePath,
                source_path: Some(source_path.borrow()),
                compiled_module_path: compiled_module_path.as_deref(),
                logical_name: logical_name.borrow(),
            },
        }
    }
}

/// Borrowed view of the common fields between the unique-by `ModuleDesc` variants.
#[cfg_attr(feature = "extra_traits", derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd))]
#[cfg_attr(
    any(feature = "debug", feature = "arbitrary", feature = "extra_traits"),
    derive(Debug)
)]
#[non_exhaustive]
pub struct ModuleDescView<'i> {
    pub key: &'i str,
    pub unique_by: UniqueBy,
    pub logical_name: &'i str,
    pub source_path: Option<&'i Utf8Path>,
    pub compiled_module_path: Option<&'i Utf8Path>,
}

#[cfg_attr(feature = "extra_traits", derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd))]
#[cfg_attr(
    any(test, feature = "debug", feature = "arbitrary", feature = "extra_traits"),
    derive(Debug)
)]
#[allow(clippy::exhaustive_enums)]
pub enum UniqueBy {
    LogicalName,
    SourcePath,
}

#[cfg_attr(
    all(feature = "serde", any(feature = "deserialize", feature = "serialize")),
    cfg_attr(feature = "deserialize", derive(::serde::Deserialize)),
    cfg_attr(feature = "serialize", derive(::serde::Serialize)),
    serde(rename_all = "kebab-case")
)]
#[cfg_attr(feature = "extra_traits", derive(Clone, Eq, Hash, Ord, PartialEq, PartialOrd))]
#[cfg_attr(
    any(test, feature = "debug", feature = "arbitrary", feature = "extra_traits"),
    derive(Debug)
)]
#[non_exhaustive]
pub struct ProvidedModuleDesc<'i> {
    #[cfg_attr(
        all(feature = "serde", any(feature = "deserialize", feature = "serialize")),
        serde(borrow, flatten)
    )]
    pub desc: ModuleDesc<'i>,
    #[cfg_attr(
        all(feature = "serde", any(feature = "deserialize", feature = "serialize")),
        serde(default = "defaults::bool::<true>")
    )]
    pub is_interface: bool,
}

#[cfg_attr(
    all(feature = "serde", any(feature = "deserialize", feature = "serialize")),
    cfg_attr(feature = "deserialize", derive(::serde::Deserialize)),
    cfg_attr(feature = "serialize", derive(::serde::Serialize)),
    serde(rename_all = "kebab-case")
)]
#[cfg_attr(feature = "extra_traits", derive(Clone, Eq, Hash, Ord, PartialEq, PartialOrd))]
#[cfg_attr(
    any(test, feature = "debug", feature = "arbitrary", feature = "extra_traits"),
    derive(Debug)
)]
#[non_exhaustive]
pub struct RequiredModuleDesc<'i> {
    #[cfg_attr(
        all(feature = "serde", any(feature = "deserialize", feature = "serialize")),
        serde(borrow, flatten)
    )]
    pub desc: ModuleDesc<'i>,
    #[cfg_attr(
        all(feature = "serde", any(feature = "deserialize", feature = "serialize")),
        serde(default)
    )]
    pub lookup_method: RequiredModuleDescLookupMethod,
}

#[cfg_attr(
    all(feature = "serde", any(feature = "deserialize", feature = "serialize")),
    cfg_attr(feature = "deserialize", derive(::serde::Deserialize)),
    cfg_attr(feature = "serialize", derive(::serde::Serialize)),
    serde(rename_all = "kebab-case")
)]
#[cfg_attr(feature = "extra_traits", derive(Eq, Hash, Ord, PartialEq, PartialOrd))]
#[cfg_attr(
    any(test, feature = "debug", feature = "arbitrary", feature = "extra_traits"),
    derive(Debug)
)]
#[derive(Clone, Copy)]
#[non_exhaustive]
pub enum RequiredModuleDescLookupMethod {
    ByName,
    IncludeAngle,
    IncludeQuote,
}
#[cfg(not(tarpaulin_include))]
impl Default for RequiredModuleDescLookupMethod {
    fn default() -> Self {
        Self::ByName
    }
}

#[cfg(feature = "extra_traits")]
#[cfg(not(tarpaulin_include))]
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
impl<'i> ::arbitrary::Arbitrary<'i> for RequiredModuleDescLookupMethod {
    fn arbitrary(u: &mut arbitrary::Unstructured<'i>) -> arbitrary::Result<Self> {
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

#[cfg(test)]
mod test {
    use alloc::string::String;

    use ::proptest::prelude::*;

    use super::*;
    use crate::vendor::camino::Utf8PathBuf;

    proptest! {
        #[cfg_attr(miri, ignore)]
        #[test]
        fn module_desc_view_by_logical_name_is_faithful(
            logical_name in any::<String>(),
            source_path in any::<Option<String>>(),
            compiled_module_path in any::<Option<String>>(),
            unique_on_source_path in any::<bool>(),
        ) {
            let logical_name = Cow::Owned::<str>(logical_name);
            let source_path = source_path.map(|s| Cow::Owned::<Utf8Path>(Utf8PathBuf::from(s)));
            let compiled_module_path = compiled_module_path.map(|s| Cow::Owned(Utf8PathBuf::from(s)));
            let desc = match source_path.clone() {
                Some(source_path) if unique_on_source_path => ModuleDesc::BySourcePath {
                    logical_name: logical_name.clone(),
                    source_path: source_path.clone(),
                    compiled_module_path: compiled_module_path.clone(),
                    unique_on_source_path: monostate::MustBeBool::<true>,
                },
                _ => ModuleDesc::ByLogicalName {
                    logical_name: logical_name.clone(),
                    source_path: source_path.clone(),
                    compiled_module_path: compiled_module_path.clone(),
                    unique_on_source_path: Some(monostate::MustBeBool::<false>),
                }
            };
            let view = desc.view();
            assert_eq!(view.logical_name, logical_name);
            assert_eq!(view.source_path, source_path.as_deref());
            assert_eq!(view.compiled_module_path, compiled_module_path.as_deref());
            match desc {
                ModuleDesc::ByLogicalName { .. } => {
                    assert!(matches!(view.unique_by, UniqueBy::LogicalName));
                    assert_eq!(view.key, logical_name);
                }
                #[allow(clippy::useless_asref)]
                ModuleDesc::BySourcePath { .. } => {
                    assert!(matches!(view.unique_by, UniqueBy::SourcePath));
                    assert_eq!(Some(view.key), source_path.as_deref().map(AsRef::as_ref));
                }
            }
        }
    }

    #[cfg_attr(miri, ignore)]
    #[cfg(all(
        feature = "serde",
        feature = "deserialize",
        feature = "serialize",
        feature = "parsing"
    ))]
    #[test]
    fn parser_and_serde_agree() {
        use rand::prelude::*;

        use crate::r5::parsers::{ParseStream, State};

        let rng = &mut rand_chacha::ChaCha8Rng::seed_from_u64(crate::r5::datagen::CHACHA8RNG_SEED);
        let config = crate::r5::datagen::graph::GraphGeneratorConfig::default().node_count(rng.gen_range(0u8 ..= 16u8));
        let dep_files = crate::r5::datagen::graph::GraphGenerator::gen_dep_files(rng, config).flat_map(|result| {
            result.and_then(|dep_file| crate::r5::datagen::json::pretty_print_unindented(&dep_file))
        });
        for text in dep_files.take(128) {
            let from_serde = serde_json::from_str::<crate::r5::DepFile>(&text).unwrap();
            let from_parsers = {
                let path = "test.ddi";
                let input = text.as_bytes();
                let state = State::default();
                let mut stream = ParseStream::new(path, input, state);
                crate::r5::parsers::dep_file(&mut stream)
            }
            .unwrap();
            assert_eq!(from_serde, from_parsers);
        }
    }
}
