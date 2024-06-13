use alloc::borrow::Cow;
use std::collections::BTreeSet;

use p1689::r5::Utf8Path;

use crate::{
    testing::{BoxResult, ValidateOrder},
    CppDepsItem,
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

pub fn bar() -> CppDepsItem<Cow<'static, Utf8Path>, Cow<'static, [u8]>> {
    let src_file = None;
    let dep_path = Cow::from(Utf8Path::new("bar.ddi"));
    let dep_text = Cow::from(
        br#"{
        "rules": [
            {
                "primary-output": "bar.o",
                "provides": [
                    {
                        "logical-name": "bar",
                        "is-interface": true
                    }
                ],
                "requires": [
                ]
            }
        ],
        "version": 0,
        "revision": 0
    }"#,
    );
    CppDepsItem::DepText {
        src_file,
        dep_path,
        dep_text,
    }
}

pub fn foo_part1() -> CppDepsItem<Cow<'static, Utf8Path>, Cow<'static, [u8]>> {
    let src_file = None;
    let dep_path = Cow::from(Utf8Path::new("foo/part1.ddi"));
    let dep_text = Cow::from(
        br#"{
        "rules": [
            {
                "primary-output": "foo/part1.o",
                "provides": [
                    {
                        "logical-name": "foo:part1",
                        "is-interface": true
                    }
                ],
                "requires": [
                ]
            }
        ],
        "version": 0,
        "revision": 0
    }"#,
    );
    CppDepsItem::DepText {
        src_file,
        dep_path,
        dep_text,
    }
}

pub fn foo_part2() -> CppDepsItem<Cow<'static, Utf8Path>, Cow<'static, [u8]>> {
    let src_file = None;
    let dep_path = Cow::from(Utf8Path::new("foo/part2.ddi"));
    let dep_text = Cow::from(
        br#"{
        "rules": [
            {
                "primary-output": "foo/part2.o",
                "provides": [
                    {
                        "logical-name": "foo:part2",
                        "is-interface": true
                    }
                ],
                "requires": [
                ]
            }
        ],
        "version": 0,
        "revision": 0
    }"#,
    );
    CppDepsItem::DepText {
        src_file,
        dep_path,
        dep_text,
    }
}

pub fn foo() -> CppDepsItem<Cow<'static, Utf8Path>, Cow<'static, [u8]>> {
    let src_file = None;
    let dep_path = Cow::from(Utf8Path::new("foo.ddi"));
    let dep_text = Cow::from(
        br#"{
        "rules": [
            {
                "primary-output": "foo.o",
                "provides": [
                    {
                        "logical-name": "foo",
                        "is-interface": true
                    }
                ],
                "requires": [
                    {
                        "logical-name": "bar"
                    },
                    {
                        "logical-name": "foo:part2"
                    },
                    {
                        "logical-name": "foo:part1"
                    }
                ]
            }
        ],
        "version": 0,
        "revision": 0
    }"#,
    );
    CppDepsItem::DepText {
        src_file,
        dep_path,
        dep_text,
    }
}

pub fn main() -> CppDepsItem<Cow<'static, Utf8Path>, Cow<'static, [u8]>> {
    let src_file = None;
    let dep_path = Cow::from(Utf8Path::new("main.ddi"));
    let dep_text = Cow::from(
        br#"{
        "rules": [
            {
                "primary-output": "main.o",
                "requires": [
                    {
                        "logical-name": "foo"
                    },
                    {
                        "logical-name": "bar"
                    }
                ]
            }
        ],
        "version": 0,
        "revision": 0
    }"#,
    );
    CppDepsItem::DepText {
        src_file,
        dep_path,
        dep_text,
    }
}

pub fn foo_bar_cycle() -> CppDepsItem<Cow<'static, Utf8Path>, Cow<'static, [u8]>> {
    let src_file = None;
    let dep_path = Cow::from(Utf8Path::new("foo.ddi"));
    let dep_text = Cow::from(
        br#"{
        "rules": [
            {
                "primary-output": "foo.o",
                "provides": [
                    {
                        "logical-name": "foo",
                        "is-interface": true
                    }
                ],
                "requires": [
                    {
                        "logical-name": "bar"
                    }
                ]
            }
        ],
        "version": 0,
        "revision": 0
    }"#,
    );
    CppDepsItem::DepText {
        src_file,
        dep_path,
        dep_text,
    }
}

pub fn bar_foo_cycle() -> CppDepsItem<Cow<'static, Utf8Path>, Cow<'static, [u8]>> {
    let src_file = None;
    let dep_path = Cow::from(Utf8Path::new("bar.ddi"));
    let dep_text = Cow::from(
        br#"{
        "rules": [
            {
                "primary-output": "bar.o",
                "provides": [
                    {
                        "logical-name": "bar",
                        "is-interface": true
                    }
                ],
                "requires": [
                    {
                        "logical-name": "foo"
                    }
                ]
            }
        ],
        "version": 0,
        "revision": 0
    }"#,
    );
    CppDepsItem::DepText {
        src_file,
        dep_path,
        dep_text,
    }
}

pub fn src_proj() -> &'static Utf8Path {
    Utf8Path::new("gnu-make")
}

pub fn items() -> impl DoubleEndedIterator<Item = CppDepsItem<Cow<'static, Utf8Path>, Cow<'static, [u8]>>> {
    [
        crate::testing::corpus::dep_text::bar(),
        crate::testing::corpus::dep_text::foo_part1(),
        crate::testing::corpus::dep_text::foo_part2(),
        crate::testing::corpus::dep_text::foo(),
        crate::testing::corpus::dep_text::main(),
    ]
    .into_iter()
}

pub fn expected_outputs() -> BTreeSet<&'static Utf8Path> {
    BTreeSet::from(["bar.o", "foo/part1.o", "foo/part2.o", "foo.o", "main.o"].map(Utf8Path::new))
}
