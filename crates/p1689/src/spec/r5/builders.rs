#![cfg(not(tarpaulin_include))]

use alloc::{borrow::Cow, vec::Vec};

use camino::Utf8Path;

use crate::spec::r5;

pub struct DepFile<'a> {
    version: Option<u32>,
    revision: Option<u32>,
    rules: Option<Vec<r5::DepInfo<'a>>>,
}
#[allow(clippy::derivable_impls)]
impl Default for DepFile<'_> {
    fn default() -> Self {
        Self {
            version: Option::default(),
            revision: Option::default(),
            rules: Option::default(),
        }
    }
}
impl<'a> DepFile<'a> {
    #[must_use]
    pub fn new() -> Self {
        Self {
            version: Option::default(),
            revision: Option::default(),
            rules: Option::default(),
        }
    }

    #[must_use]
    pub fn build(self) -> r5::DepFile<'a> {
        r5::DepFile {
            version: self.version.unwrap_or(1),
            revision: self.revision,
            rules: self.rules.unwrap_or_default(),
        }
    }

    #[must_use]
    pub const fn version(mut self, version: u32) -> Self {
        self.version = Some(version);
        self
    }

    #[must_use]
    pub const fn revision(mut self, revision: u32) -> Self {
        self.revision = Some(revision);
        self
    }

    #[must_use]
    pub fn rules(mut self, rules: Vec<r5::DepInfo<'a>>) -> Self {
        self.rules = Some(rules);
        self
    }
}

pub struct DepInfo<'a> {
    work_directory: Option<Cow<'a, Utf8Path>>,
    primary_output: Option<Cow<'a, Utf8Path>>,
    outputs: Option<Vec<Cow<'a, Utf8Path>>>,
    provides: Option<Vec<r5::ProvidedModuleDesc<'a>>>,
    requires: Option<Vec<r5::RequiredModuleDesc<'a>>>,
}
#[allow(clippy::derivable_impls)]
impl<'a> Default for DepInfo<'a> {
    fn default() -> Self {
        Self {
            work_directory: Option::default(),
            primary_output: Option::default(),
            outputs: Option::default(),
            provides: Option::default(),
            requires: Option::default(),
        }
    }
}
impl<'a> DepInfo<'a> {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn build(self) -> r5::DepInfo<'a> {
        r5::DepInfo {
            work_directory: self.work_directory,
            primary_output: self.primary_output,
            outputs: self.outputs.unwrap_or_default(),
            provides: self.provides.unwrap_or_default(),
            requires: self.requires.unwrap_or_default(),
        }
    }

    #[must_use]
    pub fn work_directory(mut self, work_directory: Cow<'a, Utf8Path>) -> Self {
        self.work_directory = Some(work_directory);
        self
    }

    #[must_use]
    pub fn primary_output(mut self, primary_output: Cow<'a, Utf8Path>) -> Self {
        self.work_directory = Some(primary_output);
        self
    }

    #[must_use]
    pub fn outputs(mut self, outputs: Vec<Cow<'a, Utf8Path>>) -> Self {
        self.outputs = Some(outputs);
        self
    }

    #[must_use]
    pub fn provides(mut self, provides: Vec<r5::ProvidedModuleDesc<'a>>) -> Self {
        self.provides = Some(provides);
        self
    }

    #[must_use]
    pub fn requires(mut self, requires: Vec<r5::RequiredModuleDesc<'a>>) -> Self {
        self.requires = Some(requires);
        self
    }
}

struct ModuleDesc<'a> {
    unique_by: self::UniqueBy<'a>,
    logical_name: Cow<'a, str>,
    compiled_module_path: Option<Cow<'a, Utf8Path>>,
}
#[non_exhaustive]
pub enum UniqueBy<'a> {
    LogicalName { source_path: Option<Cow<'a, Utf8Path>> },
    SourcePath { source_path: Cow<'a, Utf8Path> },
}
impl<'a> ModuleDesc<'a> {
    #[must_use]
    fn build(self) -> r5::ModuleDesc<'a> {
        match self.unique_by {
            self::UniqueBy::LogicalName { source_path } => r5::ModuleDesc::ByLogicalName {
                source_path,
                compiled_module_path: self.compiled_module_path,
                logical_name: self.logical_name,
                #[cfg(feature = "monostate")]
                unique_on_source_path: Some(monostate::MustBe!(false)),
            },
            self::UniqueBy::SourcePath { source_path } => r5::ModuleDesc::BySourcePath {
                source_path,
                compiled_module_path: self.compiled_module_path,
                logical_name: self.logical_name,
                #[cfg(feature = "monostate")]
                unique_on_source_path: monostate::MustBe!(true),
            },
        }
    }
}

pub struct ProvidedModuleDesc<'a> {
    unique_by: self::UniqueBy<'a>,
    logical_name: Cow<'a, str>,
    compiled_module_path: Option<Cow<'a, Utf8Path>>,
    is_interface: Option<bool>,
}
impl<'a> ProvidedModuleDesc<'a> {
    #[must_use]
    pub fn new(unique_by: self::UniqueBy<'a>, logical_name: Cow<'a, str>) -> Self {
        Self {
            unique_by,
            logical_name,
            compiled_module_path: Option::default(),
            is_interface: Option::default(),
        }
    }

    #[must_use]
    pub fn build(self) -> r5::ProvidedModuleDesc<'a> {
        let desc = self::ModuleDesc {
            unique_by: self.unique_by,
            logical_name: self.logical_name,
            compiled_module_path: self.compiled_module_path,
        };
        r5::ProvidedModuleDesc {
            desc: desc.build(),
            is_interface: self.is_interface.unwrap_or(true),
        }
    }

    #[must_use]
    pub fn compiled_module_path(mut self, compiled_module_path: Cow<'a, Utf8Path>) -> Self {
        self.compiled_module_path = Some(compiled_module_path);
        self
    }

    #[must_use]
    #[allow(clippy::wrong_self_convention)]
    pub const fn is_interface(mut self, is_interface: bool) -> Self {
        self.is_interface = Some(is_interface);
        self
    }
}

pub struct RequiredModuleDesc<'a> {
    unique_by: self::UniqueBy<'a>,
    logical_name: Cow<'a, str>,
    compiled_module_path: Option<Cow<'a, Utf8Path>>,
    lookup_method: Option<r5::RequiredModuleDescLookupMethod>,
}
impl<'a> RequiredModuleDesc<'a> {
    #[must_use]
    pub fn new(unique_by: self::UniqueBy<'a>, logical_name: Cow<'a, str>) -> Self {
        Self {
            unique_by,
            logical_name,
            compiled_module_path: Option::default(),
            lookup_method: Option::default(),
        }
    }

    #[must_use]
    pub fn build(self) -> r5::RequiredModuleDesc<'a> {
        let desc = self::ModuleDesc {
            unique_by: self.unique_by,
            logical_name: self.logical_name,
            compiled_module_path: self.compiled_module_path,
        };
        r5::RequiredModuleDesc {
            desc: desc.build(),
            lookup_method: self.lookup_method.unwrap_or_default(),
        }
    }

    #[must_use]
    pub fn compiled_module_path(mut self, compiled_module_path: Cow<'a, Utf8Path>) -> Self {
        self.compiled_module_path = Some(compiled_module_path);
        self
    }

    #[must_use]
    pub const fn lookup_method(mut self, lookup_method: r5::RequiredModuleDescLookupMethod) -> Self {
        self.lookup_method = Some(lookup_method);
        self
    }
}
