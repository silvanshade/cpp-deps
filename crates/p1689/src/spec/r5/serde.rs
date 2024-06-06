#[cfg(feature = "deserialize")]
use serde_with::DeserializeAs;
#[cfg(feature = "serialize")]
use serde_with::SerializeAs;
#[cfg(any(feature = "deserialize", feature = "serialize"))]
use ::{alloc::borrow::Cow, camino::Utf8Path};
#[cfg(feature = "deserialize")]
use ::{camino::Utf8PathBuf, serde_with::BorrowCow};

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

#[cfg(test)]
#[cfg(feature = "deserialize")]
mod test {
    use proptest::proptest;
    use rand::prelude::*;

    mod r5 {
        pub use crate::{
            r5::parsers,
            spec::r5::{proptest::strategy, *},
        };
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

            #[test]
            fn only_escaped_strings_are_copied() {
                let rng = &mut rand::thread_rng();
                let mut bytes = alloc::vec![0u8; 8192];
                rng.fill_bytes(&mut bytes);
                let mut u = arbitrary::Unstructured::new(&bytes);
                let node_count = u.int_in_range(0u8 ..= 16u8).unwrap();
                let mut dep_file_texts = r5::datagen::graph::GraphGenerator::gen_dep_files(rng, node_count)
                    .flat_map(|result| result.and_then(r5::datagen::json::pretty_print_unindented));
                let mut num_files_with_escaped_strings = 0;
                // NOTE: Keep iterating until at least 16 files with escapes have been checked
                while num_files_with_escaped_strings < 16 {
                    if let Some(dep_file_text) = dep_file_texts.next() {
                        let num_escaped_strings_within_file = crate::util::count_escaped_strings(&dep_file_text).1;
                        let input = winnow::BStr::new(dep_file_text.as_bytes());
                        let state = r5::parsers::State::default();
                        let mut state_stream = winnow::Stateful { input, state };
                        let dep_file = r5::parsers::dep_file(&mut state_stream).unwrap();
                        assert_eq!(num_escaped_strings_within_file, dep_file.count_copies());
                        num_files_with_escaped_strings += u64::from(0 < num_escaped_strings_within_file);
                    }
                }
            }
        }
    }
}
