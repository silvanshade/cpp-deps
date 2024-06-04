use alloc::{borrow::Cow, format, string::String, vec::Vec};

use camino::{Utf8Path, Utf8PathBuf};

use crate::spec::r5;

struct CowUtf8PathHelper<'a> {
    inner: Cow<'a, Utf8Path>,
}
impl core::fmt::Debug for CowUtf8PathHelper<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Debug::fmt(&self.inner, f)
    }
}

impl<'a> ::arbitrary::Arbitrary<'a> for CowUtf8PathHelper<'a> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        let dirs = u.arbitrary::<Vec<String>>()?;
        let filename = u.arbitrary::<String>()?;
        let ext = u.arbitrary::<Option<String>>()?;
        let ext = ext.map(|ext0| format!(".{ext0}")).unwrap_or_default();
        let inner = dirs.into_iter().chain([format!("{filename}{ext}")]).collect::<String>();
        let inner = Utf8PathBuf::from(inner);
        let inner = Cow::Owned(inner);
        Ok(Self { inner })
    }
}

impl<'a> ::arbitrary::Arbitrary<'a> for r5::DepFile<'a> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        let version = u.arbitrary()?;
        let revision = u.arbitrary()?;
        let rules = u.arbitrary()?;
        Ok(Self {
            version,
            revision,
            rules,
        })
    }
}

impl<'a> ::arbitrary::Arbitrary<'a> for r5::DepInfo<'a> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        let work_directory = u.arbitrary::<Option<CowUtf8PathHelper>>()?.map(|helper| helper.inner);
        let primary_output = u.arbitrary::<Option<CowUtf8PathHelper>>()?.map(|helper| helper.inner);
        let outputs = u
            .arbitrary::<Vec<CowUtf8PathHelper>>()?
            .into_iter()
            .map(|helper| helper.inner)
            .collect();
        let provides = u.arbitrary()?;
        let requires = u.arbitrary()?;
        Ok(Self {
            work_directory,
            primary_output,
            outputs,
            provides,
            requires,
        })
    }
}

impl<'a> ::arbitrary::Arbitrary<'a> for r5::ModuleDesc<'a> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        let source_path = u.arbitrary::<Option<CowUtf8PathHelper>>()?.map(|helper| helper.inner);
        let compiled_module_path = u.arbitrary::<Option<CowUtf8PathHelper>>()?.map(|helper| helper.inner);
        let logical_name = u.arbitrary()?;
        let unique_on_source_path = u.arbitrary()?;
        let _unique_on_source_path_some_false = u.arbitrary::<bool>()?;
        match source_path {
            Some(source_path) if unique_on_source_path => Ok(Self::BySourcePath {
                source_path,
                compiled_module_path,
                logical_name,
                #[cfg(any(test, feature = "monostate"))]
                unique_on_source_path: monostate::MustBe!(true),
            }),
            _ => Ok(Self::ByLogicalName {
                source_path,
                compiled_module_path,
                logical_name,
                #[cfg(any(test, feature = "monostate"))]
                #[allow(clippy::used_underscore_binding)]
                unique_on_source_path: _unique_on_source_path_some_false.then_some(monostate::MustBe!(false)),
            }),
        }
    }
}

impl<'a> ::arbitrary::Arbitrary<'a> for r5::ProvidedModuleDesc<'a> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        let desc = u.arbitrary()?;
        let is_interface = u.arbitrary()?;
        Ok(Self { desc, is_interface })
    }
}

impl<'a> ::arbitrary::Arbitrary<'a> for r5::RequiredModuleDesc<'a> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        let desc = u.arbitrary()?;
        let lookup_method = u.arbitrary()?;
        Ok(Self { desc, lookup_method })
    }
}

impl<'a> ::arbitrary::Arbitrary<'a> for r5::RequiredModuleDescLookupMethod {
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
