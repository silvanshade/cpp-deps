use alloc::{borrow::Cow, format, string::String};

use camino::{Utf8Path, Utf8PathBuf};
use proptest::prelude::*;

pub struct CowUtf8PathHelper<'a> {
    pub inner: Cow<'a, Utf8Path>,
}
impl core::fmt::Debug for CowUtf8PathHelper<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Debug::fmt(&self.inner, f)
    }
}

impl<'a> proptest::arbitrary::Arbitrary for CowUtf8PathHelper<'a> {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        (
            alloc::vec::IntoIter::<String>::arbitrary(),
            String::arbitrary(),
            Option::<String>::arbitrary(),
        )
            .prop_map(|(dirs, filename, ext)| {
                let ext = ext.map(|ext| format!(".{ext}")).unwrap_or_default();
                let inner = dirs.into_iter().chain([format!("{filename}{ext}")]).collect::<String>();
                let inner = Utf8PathBuf::from(inner);
                let inner = Cow::Owned(inner);
                CowUtf8PathHelper { inner }
            })
            .boxed()
    }
}
