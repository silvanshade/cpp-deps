#![deny(clippy::all)]
#![deny(clippy::cargo)]
#![deny(clippy::nursery)]
#![deny(clippy::pedantic)]
// clippy::restricted allow overrides
#![allow(clippy::match_ref_pats)]
#![allow(clippy::needless_borrowed_reference)]
#![allow(clippy::redundant_pub_crate)]
// clippy::restricted
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

#[cfg(feature = "deserialize")]
use serde_with::DeserializeAs;
#[cfg(feature = "serialize")]
use serde_with::SerializeAs;
#[cfg(any(feature = "deserialize", feature = "serialize"))]
use ::{alloc::borrow::Cow, camino::Utf8Path};
#[cfg(feature = "deserialize")]
use ::{camino::Utf8PathBuf, serde_with::BorrowCow};

#[cfg(any(feature = "deserialize", feature = "serialize"))]
use crate::spec::r5::DepInfo;

// TODO: adjust skip serializations for default values (including bools, etc)

/// Helper to deserialize [`Cow<T>`] as borrowed.
#[cfg(any(feature = "deserialize", feature = "serialize"))]
#[derive(Default)]
#[non_exhaustive]
pub struct CowUtf8Path;

#[cfg(feature = "deserialize")]
impl<'de> DeserializeAs<'de, Cow<'de, Utf8Path>> for CowUtf8Path {
    fn deserialize_as<D>(deserializer: D) -> Result<Cow<'de, Utf8Path>, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        let cow: Cow<str> = BorrowCow::deserialize_as(deserializer)?;
        let path = match cow {
            Cow::Borrowed(str) => Cow::Borrowed(Utf8Path::new(str)),
            Cow::Owned(string) => Cow::Owned(Utf8PathBuf::from(string)),
        };
        Ok(path)
    }
}
#[cfg(feature = "serialize")]
impl<'a> SerializeAs<Cow<'a, Utf8Path>> for CowUtf8Path {
    fn serialize_as<S>(source: &Cow<'a, Utf8Path>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ::serde::Serializer,
    {
        serializer.serialize_str(source.as_ref().as_ref())
    }
}

/// Helper to deserialize [`DepInfo`] with `primary-output` removed from `outputs`. This provides an additional
/// invariant which in various processing algorithms.
#[cfg(any(feature = "deserialize", feature = "serialize"))]
#[derive(Default)]
#[non_exhaustive]
pub struct DepInfoUniqueOutputs;

#[cfg(feature = "deserialize")]
impl<'de> DeserializeAs<'de, DepInfo<'de>> for DepInfoUniqueOutputs {
    fn deserialize_as<D>(deserializer: D) -> Result<DepInfo<'de>, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        let mut dep_info: DepInfo = ::serde::Deserialize::deserialize(deserializer)?;
        if let Some(ref primary_output) = dep_info.primary_output {
            dep_info.outputs.swap_remove(primary_output);
        }
        Ok(dep_info)
    }
}
#[cfg(feature = "serialize")]
impl<'a> SerializeAs<DepInfo<'a>> for DepInfoUniqueOutputs {
    fn serialize_as<S>(source: &DepInfo<'a>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ::serde::Serializer,
    {
        <DepInfo as ::serde::Serialize>::serialize(source, serializer)
    }
}

#[cfg(test)]
#[cfg(feature = "deserialize")]
mod test {
    use proptest::proptest;

    mod r5 {
        pub use crate::spec::r5::{proptest::strategy, *};
    }

    #[cfg(feature = "deserialize")]
    pub mod deserialize {
        use super::*;

        pub mod dep_file {
            use super::*;

            proptest! {
                #[test]
                fn dep_file(input in r5::strategy::dep_file()) {
                    serde_json::from_str::<r5::DepFile>(&input).unwrap();
                }
            }
        }
    }
}
