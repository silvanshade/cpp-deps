use alloc::borrow::Cow;
use std::collections::BTreeSet;

use p1689::r5::{Utf8Path, Utf8PathBuf};

use crate::{
    testing::{BoxResult, ValidateOrder},
    CppDepsItem,
    CppDepsSrc,
};

pub fn validate_order<P, B, Is>(items: Is) -> BoxResult<ValidateOrder<'static, P, B>>
where
    P: AsRef<Utf8Path> + Send + Sync + 'static,
    B: AsRef<[u8]> + Send + Sync + 'static,
    Is: IntoIterator<Item = CppDepsItem<P, B>>,
{
    let src_proj = self::src_proj();
    let expected_outputs = self::expected_outputs();
    ValidateOrder::new(src_proj, items, expected_outputs)
}

pub fn bar(workspace_root: &Utf8PathBuf) -> CppDepsItem<Cow<'static, Utf8Path>, Cow<'static, [u8]>> {
    let src_base = workspace_root.join("examples");
    let src_proj = self::src_proj();
    let src_path = src_base.join(src_proj).join("bar.cppm");
    CppDepsItem::SrcFile {
        src_file: CppDepsSrc {
            src_base: src_base.into(),
            src_path: src_path.into(),
        },
    }
}

pub fn foo_part1(workspace_root: &Utf8PathBuf) -> CppDepsItem<Cow<'static, Utf8Path>, Cow<'static, [u8]>> {
    let src_base = workspace_root.join("examples");
    let src_proj = self::src_proj();
    let src_path = src_base.join(src_proj).join("foo").join("part1.cppm");
    CppDepsItem::SrcFile {
        src_file: CppDepsSrc {
            src_base: src_base.into(),
            src_path: src_path.into(),
        },
    }
}

pub fn foo_part2(workspace_root: &Utf8PathBuf) -> CppDepsItem<Cow<'static, Utf8Path>, Cow<'static, [u8]>> {
    let src_base = workspace_root.join("examples");
    let src_proj = self::src_proj();
    let src_path = src_base.join(src_proj).join("foo").join("part2.cppm");
    CppDepsItem::SrcFile {
        src_file: CppDepsSrc {
            src_base: src_base.into(),
            src_path: src_path.into(),
        },
    }
}

pub fn foo(workspace_root: &Utf8PathBuf) -> CppDepsItem<Cow<'static, Utf8Path>, Cow<'static, [u8]>> {
    let src_base = workspace_root.join("examples");
    let src_proj = self::src_proj();
    let src_path = src_base.join(src_proj).join("foo.cppm");
    CppDepsItem::SrcFile {
        src_file: CppDepsSrc {
            src_base: src_base.into(),
            src_path: src_path.into(),
        },
    }
}

pub fn main(workspace_root: &Utf8PathBuf) -> CppDepsItem<Cow<'static, Utf8Path>, Cow<'static, [u8]>> {
    let src_base = workspace_root.join("examples");
    let src_proj = self::src_proj();
    let src_path = src_base.join(src_proj).join("main.cpp");
    CppDepsItem::SrcFile {
        src_file: CppDepsSrc {
            src_base: src_base.into(),
            src_path: src_path.into(),
        },
    }
}

pub fn src_proj() -> &'static Utf8Path {
    Utf8Path::new("gnu-make")
}

pub fn items() -> Result<
    impl DoubleEndedIterator<Item = CppDepsItem<Cow<'static, Utf8Path>, Cow<'static, [u8]>>>,
    Box<dyn std::error::Error + Send + Sync + 'static>,
> {
    let metadata = cargo_metadata::MetadataCommand::new().exec()?;
    let workspace_root = metadata.workspace_root.clone();
    let iter = [
        crate::testing::corpus::src_file::bar(&workspace_root),
        crate::testing::corpus::src_file::foo_part1(&workspace_root),
        crate::testing::corpus::src_file::foo_part2(&workspace_root),
        crate::testing::corpus::src_file::foo(&workspace_root),
        crate::testing::corpus::src_file::main(&workspace_root),
    ]
    .into_iter();
    Ok(iter)
}

pub fn expected_outputs() -> BTreeSet<&'static Utf8Path> {
    BTreeSet::from(["bar.o", "foo/part1.o", "foo/part2.o", "foo.o", "main.o"].map(Utf8Path::new))
}
