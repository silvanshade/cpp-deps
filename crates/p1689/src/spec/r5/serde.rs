#[cfg(any(feature = "deserialize", feature = "serialize"))]
use alloc::borrow::Cow;

#[cfg(feature = "deserialize")]
use serde_with::BorrowCow;
#[cfg(feature = "deserialize")]
use serde_with::DeserializeAs;
#[cfg(feature = "serialize")]
use serde_with::SerializeAs;

#[cfg(any(feature = "deserialize", feature = "serialize"))]
use crate::vendor::camino::Utf8Path;
#[cfg(feature = "deserialize")]
use crate::vendor::camino::Utf8PathBuf;

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
            #[allow(clippy::useless_conversion)]
            Cow::Borrowed(str) => Cow::Borrowed(str.into()),
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

#[cfg(test)]
#[cfg(feature = "deserialize")]
mod test {
    use proptest::proptest;
    #[cfg(feature = "datagen")]
    use rand::prelude::*;

    mod r5 {
        pub use crate::spec::r5::{proptest::strategy, *};
    }

    #[cfg(feature = "deserialize")]
    pub mod deserialize {
        use super::*;

        pub mod dep_file {
            use super::*;

            proptest! {
                #[cfg_attr(miri, ignore)]
                #[test]
                fn dep_file(text in r5::strategy::dep_file()) {
                    serde_json::from_str::<r5::DepFile>(&text).unwrap();
                }
            }

            #[cfg_attr(miri, ignore)] // NOTE: too expensive for `miri`
            #[cfg(feature = "datagen")]
            #[test]
            fn only_escaped_strings_are_copied() {
                let rng = &mut rand_chacha::ChaCha8Rng::seed_from_u64(crate::r5::datagen::CHACHA8RNG_SEED);
                let config =
                    r5::datagen::graph::GraphGeneratorConfig::default().node_count(rng.gen_range(0u8 ..= 16u8));
                let mut dep_file_texts = r5::datagen::graph::GraphGenerator::gen_dep_files(rng, config)
                    .flat_map(|result| result.and_then(r5::datagen::json::pretty_print_unindented));
                let mut num_files_with_escaped_strings = 0;
                // NOTE: Keep iterating until at least 16 files with escapes have been checked
                while num_files_with_escaped_strings < 16 {
                    if let Some(text) = dep_file_texts.next() {
                        let num_escaped_strings_within_file = crate::util::count_escaped_strings(&text).1;
                        let dep_file = serde_json::from_str::<r5::DepFile>(&text).unwrap();
                        assert_eq!(num_escaped_strings_within_file, dep_file.count_copies());
                        num_files_with_escaped_strings += u64::from(0 < num_escaped_strings_within_file);
                    }
                }
            }
        }
    }
}
