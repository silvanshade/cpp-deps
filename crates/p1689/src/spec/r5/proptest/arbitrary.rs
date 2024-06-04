use alloc::string::String;

use proptest::prelude::*;

use crate::{spec::r5, util::proptest::arbitrary::CowUtf8PathHelper};

impl<'a> proptest::arbitrary::Arbitrary for r5::DepFile<'a> {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        (
            u32::arbitrary(),
            Option::<u32>::arbitrary(),
            alloc::vec::IntoIter::<r5::DepInfo>::arbitrary(),
        )
            .prop_map(move |(version, revision, rules)| Self {
                version,
                revision,
                rules: rules.collect(),
            })
            .boxed()
    }
}

impl<'a> proptest::arbitrary::Arbitrary for r5::DepInfo<'a> {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with((): Self::Parameters) -> Self::Strategy {
        (
            Option::<CowUtf8PathHelper>::arbitrary(),
            Option::<CowUtf8PathHelper>::arbitrary(),
            alloc::vec::IntoIter::<CowUtf8PathHelper>::arbitrary(),
            alloc::vec::IntoIter::<r5::ProvidedModuleDesc>::arbitrary(),
            alloc::vec::IntoIter::<r5::RequiredModuleDesc>::arbitrary(),
        )
            .prop_map(
                move |(work_directory, primary_output, outputs, provides, requires)| Self {
                    work_directory: work_directory.map(|helper| helper.inner),
                    primary_output: primary_output.map(|helper| helper.inner),
                    outputs: outputs.map(|helper| helper.inner).collect(),
                    provides: provides.collect(),
                    requires: requires.collect(),
                },
            )
            .boxed()
    }
}

impl<'a> proptest::arbitrary::Arbitrary for r5::ModuleDesc<'a> {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with((): Self::Parameters) -> Self::Strategy {
        (
            Option::<CowUtf8PathHelper>::arbitrary(),
            Option::<CowUtf8PathHelper>::arbitrary(),
            String::arbitrary(),
            bool::arbitrary(),
            bool::arbitrary(),
        )
            .prop_map(
                move |(
                    source_path,
                    compiled_module_path,
                    logical_name,
                    unique_on_source_path,
                    _unique_on_source_path_some_false,
                )| match source_path {
                    Some(source_path) if unique_on_source_path => Self::BySourcePath {
                        source_path: source_path.inner,
                        compiled_module_path: compiled_module_path.map(|helper| helper.inner),
                        logical_name: logical_name.into(),
                        #[cfg(any(test, feature = "monostate"))]
                        unique_on_source_path: monostate::MustBe!(true),
                    },
                    _ => Self::ByLogicalName {
                        source_path: source_path.map(|helper| helper.inner),
                        compiled_module_path: compiled_module_path.map(|helper| helper.inner),
                        logical_name: logical_name.into(),
                        #[cfg(any(test, feature = "monostate"))]
                        #[allow(clippy::used_underscore_binding)]
                        unique_on_source_path: _unique_on_source_path_some_false.then_some(monostate::MustBe!(false)),
                    },
                },
            )
            .boxed()
    }
}

impl<'a> proptest::arbitrary::Arbitrary for r5::ProvidedModuleDesc<'a> {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with((): Self::Parameters) -> Self::Strategy {
        (r5::ModuleDesc::arbitrary(), bool::arbitrary())
            .prop_map(move |(desc, is_interface)| Self { desc, is_interface })
            .boxed()
    }
}

impl<'a> proptest::arbitrary::Arbitrary for r5::RequiredModuleDesc<'a> {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with((): Self::Parameters) -> Self::Strategy {
        (
            r5::ModuleDesc::arbitrary(),
            r5::RequiredModuleDescLookupMethod::arbitrary(),
        )
            .prop_map(move |(desc, lookup_method)| Self { desc, lookup_method })
            .boxed()
    }
}

impl proptest::arbitrary::Arbitrary for r5::RequiredModuleDescLookupMethod {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with((): Self::Parameters) -> Self::Strategy {
        prop_oneof![Just(Self::ByName), Just(Self::IncludeAngle), Just(Self::IncludeQuote)].boxed()
    }
}
