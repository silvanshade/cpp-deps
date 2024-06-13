#[cfg(any(feature = "deserialize", feature = "serialize"))]
use alloc::borrow::Cow;

#[cfg(any(feature = "deserialize", feature = "serialize"))]
use crate::vendor::camino::Utf8Path;
#[cfg(feature = "deserialize")]
use crate::vendor::camino::Utf8PathBuf;

#[cfg(feature = "deserialize")]
pub mod deserialize {
    use alloc::{borrow::ToOwned, string::String, vec::Vec};

    use serde::Deserialize;

    use super::*;

    struct CowStrVisitor;
    impl<'de> serde::de::Visitor<'de> for CowStrVisitor {
        type Value = Cow<'de, str>;

        #[inline]
        fn expecting(&self, formatter: &mut alloc::fmt::Formatter) -> alloc::fmt::Result {
            formatter.write_str("a UTF-8 path")
        }

        #[inline]
        fn visit_borrowed_str<E>(self, value: &'de str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(Cow::Borrowed(value))
        }

        #[inline]
        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(Cow::Owned(value.to_owned()))
        }

        #[inline]
        fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(Cow::Owned(value))
        }
    }

    struct CowUtf8Path<'a>(Cow<'a, Utf8Path>);
    impl<'de> serde::Deserialize<'de> for CowUtf8Path<'de> {
        #[inline]
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            let cow = match deserializer.deserialize_str(CowStrVisitor)? {
                Cow::Borrowed(ptr) => Cow::Borrowed(ptr.into()),
                Cow::Owned(val) => Cow::Owned(Utf8PathBuf::from(val)),
            };
            Ok(CowUtf8Path(cow))
        }
    }

    struct VecCowUtf8PathVisitor;
    impl<'de> serde::de::Visitor<'de> for VecCowUtf8PathVisitor {
        type Value = Vec<Cow<'de, Utf8Path>>;

        #[inline]
        fn expecting(&self, formatter: &mut alloc::fmt::Formatter) -> alloc::fmt::Result {
            formatter.write_str("a sequence of UTF-8 paths")
        }

        #[inline]
        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: serde::de::SeqAccess<'de>,
        {
            let mut vec = Vec::with_capacity(seq.size_hint().unwrap_or_default());
            while let Some(value) = seq.next_element::<CowUtf8Path>()? {
                vec.push(value.0);
            }
            Ok(vec)
        }
    }

    #[inline]
    pub fn cow_utf8path<'de, D>(deserializer: D) -> Result<Cow<'de, Utf8Path>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let val = match deserializer.deserialize_str(CowStrVisitor)? {
            Cow::Borrowed(ptr) => Cow::Borrowed(ptr.into()),
            Cow::Owned(ref val) => Cow::Owned(Utf8PathBuf::from(val)),
        };
        Ok(val)
    }

    #[inline]
    pub fn logical_name<'de, D>(deserializer: D) -> Result<Cow<'de, str>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(CowStrVisitor)
    }

    #[inline]
    pub fn option_cow_utf8path<'de, D>(deserializer: D) -> Result<Option<Cow<'de, Utf8Path>>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let option = Option::<CowUtf8Path>::deserialize(deserializer)?;
        let option = option.map(|wrapper| wrapper.0);
        Ok(option)
    }

    #[inline]
    pub fn vec_cow_utf8path<'de, D>(deserializer: D) -> Result<Vec<Cow<'de, Utf8Path>>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_seq(VecCowUtf8PathVisitor)
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
