// Almost all parsers are inherently fallible so don't require error docs.
#![allow(clippy::missing_errors_doc)]

use winnow::{
    combinator::{dispatch, empty, fail, peek, trace},
    error::ParserError,
    prelude::*,
    token::{any, take},
};

use crate::{
    spec::r5,
    util::winnow::{json, number, spaces, string, StateStream},
};

#[rustfmt::skip]
pub fn dep_file<'i>(stream: &mut StateStream<'i>) -> PResult<r5::DepFile<'i>> {
    let fields = |stream0: &mut StateStream<'i>| {
        let mut rules = None;
        let mut revision = Option::default();
        let mut version = None;
        while b'}' != peek(any).parse_next(stream0)? {
            let next0 = any.parse_next(stream0)?;
            match next0 {
                b'"' => { // tarpaulin::hint
                    let next1 = any.parse_next(stream0)?;
                    match next1 {
                        b'r' => { // tarpaulin::hint
                            let next2 = any.parse_next(stream0)?;
                            match next2 {
                                b'e' => { // tarpaulin::hint
                                    if revision.is_some() {
                                        let message = r#"duplicate "revision" field"#;
                                        return Err(winnow::error::ErrMode::assert(stream0, message));
                                    }
                                    let key = b"vision\"".as_slice();
                                    let val = self::number::dec_uint; // tarpaulin::hint
                                    let val = trace("\"revision\"", self::json::field(key, val)).parse_next(stream0)?;
                                    revision = Some(val);
                                },
                                b'u' => { // tarpaulin::hint
                                    if rules.is_some() {
                                        let message = r#"duplicate "rules" field"#;
                                        return Err(winnow::error::ErrMode::assert(stream0, message));
                                    }
                                    let key = b"les\"".as_slice();
                                    let val = self::json::vec(dep_info);
                                    let val = trace("\"rules\"", self::json::field(key, val)).parse_next(stream0)?;
                                    rules = Some(val);
                                },
                                _ => fail.parse_next(stream0)?,
                            }
                        },
                        b'v' => { // tarpaulin::hint
                            if version.is_some() {
                                let message = r#"duplicate "version" field"#;
                                return Err(winnow::error::ErrMode::assert(stream0, message));
                            }
                            let key = b"ersion\"".as_slice();
                            let val = self::number::dec_uint; // tarpaulin::hint
                            let val = trace("\"version\"", self::json::field(key, val)).parse_next(stream0)?;
                            version = Some(val);
                        },
                        _ => fail.parse_next(stream0)?,
                    }
                },
                _ => fail.parse_next(stream0)?,
            }
        }
        let dep_file = r5::DepFile {
            version: version.ok_or_else(|| {
                let message = r#"missing "version" field"#;
                winnow::error::ErrMode::assert(stream0, message)
            })?,
            revision,
            rules: rules.ok_or_else(|| {
                let message = r#"missing "rules" field"#;
                winnow::error::ErrMode::assert(stream0, message)
            })?,
        };
        Ok(dep_file)
    };
    trace("r5::DepFile", self::spaces::around(self::json::record(fields))).parse_next(stream)
}

#[cfg(test)]
pub(crate) mod dep_file {
    use alloc::vec::Vec;

    use winnow::{ascii::multispace0, combinator::delimited};

    use super::*;

    #[cfg(test)]
    pub fn version(stream: &mut StateStream) -> PResult<u32> {
        let key = b"\"version\"".as_slice();
        let val = self::number::dec_uint;
        super::json::field(key, val).parse_next(stream)
    }

    #[cfg(test)]
    pub fn revision(stream: &mut StateStream) -> PResult<u32> {
        let key = b"\"revision\"".as_slice();
        let val = self::number::dec_uint;
        super::json::field(key, val).parse_next(stream)
    }

    #[cfg(test)]
    pub fn rules<'i>(stream: &mut StateStream<'i>) -> PResult<Vec<r5::DepInfo<'i>>> {
        let key = b"\"rules\"".as_slice();
        let val = |stream0: &mut StateStream<'i>| {
            delimited(b'[', multispace0, b']').parse_next(stream0)?;
            Ok(alloc::vec![])
        };
        super::json::field(key, val).parse_next(stream)
    }
}

#[rustfmt::skip]
pub fn dep_info<'i>(stream: &mut StateStream<'i>) -> PResult<r5::DepInfo<'i>> {
    let fields = |stream0: &mut StateStream<'i>| {
        let mut work_directory = Option::default();
        let mut primary_output = Option::default();
        let mut outputs = Option::default();
        let mut provides = Option::default();
        let mut requires = Option::default();
        while b'}' != peek(any).parse_next(stream0)? {
            let next0 = any.parse_next(stream0)?;
            match next0 {
                b'"' => { // tarpaulin::hint
                    let next1 = any.parse_next(stream0)?;
                    match next1 {
                        b'o' => { // tarpaulin::hint
                            if outputs.is_some() {
                                let message = r#"duplicate "outputs" field"#;
                                return Err(winnow::error::ErrMode::assert(stream0, message));
                            }
                            let key = b"utputs\"".as_slice();
                            let val = self::json::vec(self::string::utf8_path);
                            let val = trace("\"outputs\"", self::json::field(key, val)).parse_next(stream0)?;
                            outputs = Some(val);
                        },
                        b'p' => { // tarpaulin::hint
                            let next2 = any.parse_next(stream0)?;
                            match next2 {
                                b'r' => { // tarpaulin::hint
                                    let next3 = any.parse_next(stream0)?;
                                    match next3 {
                                        b'i' => { // tarpaulin::hint
                                            if primary_output.is_some() {
                                                let message = r#"duplicate "primary_output" field"#;
                                                return Err(winnow::error::ErrMode::assert(stream0, message));
                                            }
                                            let key = b"mary-output\"".as_slice();
                                            let val = self::string::utf8_path; // tarpaulin::hint
                                            let val = trace("\"primary-output\"", self::json::field(key, val)).parse_next(stream0)?;
                                            primary_output = Some(val);
                                        },
                                        b'o' => { // tarpaulin::hint
                                            if provides.is_some() {
                                                let message = r#"duplicate "provides" field"#;
                                                return Err(winnow::error::ErrMode::assert(stream0, message));
                                            }
                                            let key = b"vides\"".as_slice();
                                            let val = self::json::vec(provided_module_desc);
                                            let val = trace("\"provides\"", self::json::field(key, val)).parse_next(stream0)?;
                                            provides = Some(val);
                                        },
                                        _ => fail.parse_next(stream0)?,
                                    }
                                },
                                _ => fail.parse_next(stream0)?,
                            }
                        },
                        b'r' => { // tarpaulin::hint
                            if requires.is_some() {
                                let message = r#"duplicate "requires" field"#;
                                return Err(winnow::error::ErrMode::assert(stream0, message));
                            }
                            let key = b"equires\"".as_slice();
                            let val = self::json::vec(required_module_desc);
                            let val = trace("\"requires\"", self::json::field(key, val)).parse_next(stream0)?;
                            requires = Some(val);
                        },
                        b'w' => { // tarpaulin::hint
                            if work_directory.is_some() {
                                let message = r#"duplicate "work_directory" field"#;
                                return Err(winnow::error::ErrMode::assert(stream0, message));
                            }
                            let key = b"ork-directory\"".as_slice();
                            let val = self::string::utf8_path; // tarpaulin::hint
                            let val = trace("\"work-directory\"", self::json::field(key, val)).parse_next(stream0)?;
                            work_directory = Some(val);
                        },
                        _ => fail.parse_next(stream0)?,
                    }
                },
                _ => fail.parse_next(stream0)?,
            }
        }
        let dep_info = r5::DepInfo {
            work_directory,
            primary_output,
            outputs: outputs.unwrap_or_default(),
            provides: provides.unwrap_or_default(),
            requires: requires.unwrap_or_default(),
        };
        Ok(dep_info)
    };
    trace("r5::DepInfo", self::spaces::around(self::json::record(fields))).parse_next(stream)
}
pub mod dep_info {}

#[rustfmt::skip]
#[allow(clippy::too_many_lines)]
pub fn provided_module_desc<'i>(stream: &mut StateStream<'i>) -> PResult<r5::ProvidedModuleDesc<'i>> {
    let fields = |stream0: &mut StateStream<'i>| {
        let mut source_path = None;
        let mut compiled_module_path = None;
        let mut logical_name = None;
        let mut unique_on_source_path = None;
        let mut is_interface = None;
        while b'}' != peek(any).parse_next(stream0)? {
            let next0 = any.parse_next(stream0)?;
            match next0 {
                b'"' => { // tarpaulin::hint
                    let next1 = any.parse_next(stream0)?;
                    match next1 {
                        b's' => { // tarpaulin::hint
                            if source_path.is_some() {
                                let message = r#"duplicate "source-path" field"#;
                                return Err(winnow::error::ErrMode::assert(stream0, message));
                            }
                            let key = b"ource-path\"".as_slice();
                            let val = self::string::utf8_path; // tarpaulin::hint
                            let val = self::json::field(key, val).parse_next(stream0)?;
                            source_path = Some(val);
                        },
                        b'c' => { // tarpaulin::hint
                            if compiled_module_path.is_some() {
                                let message = r#"duplicate "compiled-module-path" field"#;
                                return Err(winnow::error::ErrMode::assert(stream0, message));
                            }
                            let key = b"ompiled-module-path\"".as_slice();
                            let val = self::string::utf8_path; // tarpaulin::hint
                            let val = self::json::field(key, val).parse_next(stream0)?;
                            compiled_module_path = Some(val);
                        },
                        b'l' => { // tarpaulin::hint
                            if logical_name.is_some() {
                                let message = r#"duplicate "logical-name" field"#;
                                return Err(winnow::error::ErrMode::assert(stream0, message));
                            }
                            let key = b"ogical-name\"".as_slice();
                            let val = self::string::module; // tarpaulin::hint
                            let val = self::json::field(key, val).parse_next(stream0)?;
                            logical_name = Some(val);
                        },
                        b'u' => { // tarpaulin::hint
                            if unique_on_source_path.is_some() {
                                let message = r#"duplicate "unique-on-source-path" field"#;
                                return Err(winnow::error::ErrMode::assert(stream0, message));
                            }
                            let key = b"nique-on-source-path\"".as_slice();
                            let val = self::json::bool; // tarpaulin::hint
                            let val = self::json::field(key, val).parse_next(stream0)?;
                            unique_on_source_path = Some(val);
                        },
                        b'i' => { // tarpaulin::hint
                            if is_interface.is_some() {
                                let message = r#"duplicate "is-interface" field"#;
                                return Err(winnow::error::ErrMode::assert(stream0, message));
                            }
                            let key = b"s-interface\"".as_slice();
                            let val = self::json::bool; // tarpaulin::hint
                            let val = self::json::field(key, val).parse_next(stream0)?;
                            is_interface = Some(val);
                        },
                        _ => fail.parse_next(stream0)?,
                    }
                },
                _ => fail.parse_next(stream0)?,
            }
        }
        let desc = r5::ProvidedModuleDesc {
            desc: if unique_on_source_path.unwrap_or(false) {
                r5::ModuleDesc::BySourcePath {
                    source_path: source_path.ok_or_else(|| {
                        let message = r#"missing "source-path" field (which should exist since "unique-on-source-path" is `true`)"#;
                        winnow::error::ErrMode::assert(stream0, message)
                    })?,
                    compiled_module_path,
                    logical_name: logical_name.ok_or_else(|| {
                        let message = r#"missing "logical-name" field"#;
                        winnow::error::ErrMode::assert(stream0, message)
                    })?,
                    #[cfg(feature = "monostate")]
                    unique_on_source_path: monostate::MustBe!(true),
                }
            } else {
                r5::ModuleDesc::ByLogicalName {
                    source_path,
                    compiled_module_path,
                    logical_name: logical_name.ok_or_else(|| {
                        let message = r#"missing "logical-name" field"#;
                        winnow::error::ErrMode::assert(stream0, message)
                    })?,
                    #[cfg(feature = "monostate")]
                    unique_on_source_path: unique_on_source_path.and(Some(monostate::MustBe!(false))),
                }
            },
            is_interface: is_interface.ok_or_else(|| {
                let message = r#"missing "lookup-method" field"#;
                winnow::error::ErrMode::assert(stream0, message)
            })?,
        };
        Ok(desc)
    };
    trace(
        "r5::ProvidedModuleDesc",
        self::spaces::around(self::json::record(fields)),
    )
    .parse_next(stream) // tarpaulin::hint
}

#[rustfmt::skip]
#[allow(clippy::too_many_lines)]
pub fn required_module_desc<'i>(stream: &mut StateStream<'i>) -> PResult<r5::RequiredModuleDesc<'i>> {
    let fields = |stream0: &mut StateStream<'i>| {
        let mut source_path = None;
        let mut compiled_module_path = None;
        let mut lookup_method = None;
        let mut unique_on_source_path = None;
        let mut logical_name = None;
        while b'}' != peek(any).parse_next(stream0)? {
            let next0 = any.parse_next(stream0)?;
            match next0 {
                b'"' => { // tarpaulin::hint
                    let next1 = any.parse_next(stream0)?;
                    match next1 {
                        b's' => { // tarpaulin::hint
                            if source_path.is_some() {
                                let message = r#"duplicate "source-path" field"#;
                                return Err(winnow::error::ErrMode::assert(stream0, message));
                            }
                            let key = b"ource-path\"".as_slice();
                            let val = self::string::utf8_path; // tarpaulin::hint
                            let val = self::json::field(key, val).parse_next(stream0)?;
                            source_path = Some(val);
                        },
                        b'c' => { // tarpaulin::hint
                            if compiled_module_path.is_some() {
                                let message = r#"duplicate "compiled-module-path" field"#;
                                return Err(winnow::error::ErrMode::assert(stream0, message));
                            }
                            let key = b"ompiled-module-path\"".as_slice();
                            let val = self::string::utf8_path;
                            let val = self::json::field(key, val).parse_next(stream0)?;
                            compiled_module_path = Some(val);
                        },
                        b'l' => { // tarpaulin::hint
                            let next2 = take(2usize).parse_next(stream0)?;
                            match next2 {
                                b"og" => {
                                    if logical_name.is_some() {
                                        let message = r#"duplicate "logical-name" field"#;
                                        return Err(winnow::error::ErrMode::assert(stream0, message));
                                    }
                                    let key = b"ical-name\"".as_slice();
                                    let val = self::string::module; // tarpaulin::hint
                                    let val = self::json::field(key, val).parse_next(stream0)?;
                                    logical_name = Some(val);
                                },
                                b"oo" => { // tarpaulin::hint
                                    if lookup_method.is_some() {
                                        let message = r#"duplicate "lookup-method" field"#;
                                        return Err(winnow::error::ErrMode::assert(stream0, message));
                                    }
                                    let key = b"kup-method\"".as_slice();
                                    let val = self::required_module_desc::lookup_method; // tarpaulin::hint
                                    let val = self::json::field(key, val).parse_next(stream0)?;
                                    lookup_method = Some(val);
                                },
                                _ => fail.parse_next(stream0)?,
                            }
                        },
                        b'u' => { // tarpaulin::hint
                            if unique_on_source_path.is_some() {
                                let message = r#"duplicate "unique-on-source-path" field"#;
                                return Err(winnow::error::ErrMode::assert(stream0, message));
                            }
                            let key = b"nique-on-source-path\"".as_slice();
                            let val = self::json::bool; // tarpaulin::hint
                            let val = self::json::field(key, val).parse_next(stream0)?;
                            unique_on_source_path = Some(val);
                        },
                        _ => fail.parse_next(stream0)?,
                    }
                },
                _ => fail.parse_next(stream0)?,
            }
        }
        let desc = r5::RequiredModuleDesc {
            desc: if unique_on_source_path.unwrap_or(false) {
                r5::ModuleDesc::BySourcePath {
                    source_path: source_path.ok_or_else(|| {
                        let message = r#"missing "source-path" field (which should exist since "unique-on-source-path" is `true`)"#;
                        winnow::error::ErrMode::assert(stream0, message)
                    })?,
                    compiled_module_path,
                    logical_name: logical_name.ok_or_else(|| {
                        let message = r#"missing "logical-name" field"#;
                        winnow::error::ErrMode::assert(stream0, message)
                    })?,
                    #[cfg(feature = "monostate")]
                    unique_on_source_path: monostate::MustBe!(true),
                }
            } else {
                r5::ModuleDesc::ByLogicalName {
                    source_path,
                    compiled_module_path,
                    logical_name: logical_name.ok_or_else(|| {
                        let message = r#"missing "logical-name" field"#;
                        winnow::error::ErrMode::assert(stream0, message)
                    })?,
                    #[cfg(feature = "monostate")]
                    unique_on_source_path: unique_on_source_path.and(Some(monostate::MustBe!(false))),
                }
            },
            lookup_method: lookup_method.unwrap_or_default(),
        };
        Ok(desc)
    };
    trace(
        "r5::RequiredModuleDesc",
        self::spaces::around(self::json::record(fields)),
    )
    .parse_next(stream) // tarpaulin::hint
}

pub mod required_module_desc {
    #[allow(clippy::wildcard_imports)]
    use super::*;

    pub fn lookup_method(stream: &mut StateStream) -> PResult<r5::RequiredModuleDescLookupMethod> {
        trace("r5::RequiredModuleDescLookupMethod",
            dispatch! { any;
                b'"' => dispatch! { any;
                    b'b' => trace("\"by-name\"", b"y-name\"").value(r5::RequiredModuleDescLookupMethod::ByName),
                    b'i' => dispatch!{ take::<usize, StateStream, _>(13usize);
                        b"nclude-angle\"" => trace("\"include-angle\"", empty).value(r5::RequiredModuleDescLookupMethod::IncludeAngle),
                        b"nclude-quote\"" => trace("\"include-quote\"", empty).value(r5::RequiredModuleDescLookupMethod::IncludeQuote),
                        _ => fail,
                    },
                    _ => fail,
                },
                _ => fail,
            }
        ).parse_next(stream)
    }
}

#[cfg(test)]
mod test {
    use proptest::proptest;
    use rand::prelude::*;
    use winnow::BStr;

    use super::*;
    use crate::util::winnow::State;

    mod r5 {
        pub use crate::{
            r5::parsers,
            spec::r5::{proptest::strategy, *},
        };
    }

    mod parse {

        use super::*;

        pub mod dep_file {
            use super::*;

            proptest! {
                #[cfg_attr(miri, ignore)]
                #[test]
                fn revision(text in r5::strategy::dep_file::revision("[ \t\n\r]*[,}}]", false)) {
                    let input = BStr::new(&text);
                    let state = State::default();
                    let mut stream = StateStream { input, state };
                    r5::winnow::dep_file::revision.parse_next(&mut stream).unwrap();
                }

                #[cfg_attr(miri, ignore)]
                #[test]
                fn rules(text in r5::strategy::dep_file::rules("[ \t\n\r]*[,}}]")) {
                    let input = BStr::new(&text);
                    let state = State::default();
                    let mut stream = StateStream { input, state };
                    r5::winnow::dep_file::rules.parse_next(&mut stream).unwrap();
                }

                #[cfg_attr(miri, ignore)]
                #[test]
                fn version(text in r5::strategy::dep_file::version("[ \t\n\r]*[,}}]")) {
                    let input = BStr::new(&text);
                    let state = State::default();
                    let mut stream = StateStream { input, state };
                    r5::winnow::dep_file::version.parse_next(&mut stream).unwrap();
                }
            }

            #[cfg_attr(miri, ignore)] // NOTE: too expensive for `miri`
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

        #[test]
        fn static_dep_file() {
            let text = r#"
            {
                "version": 1,
                "revision": 0,
                "rules": [
                    {
                        "work-directory": "build",
                        "primary-output": "fo\u{2764}o.o",
                        "outputs": [
                            "foo.d"
                        ],
                        "provides": [],
                        "requires": []
                    }
                ]
            }
            "#;
            let input = BStr::new(&text);
            let state = State::default();
            let mut stream = StateStream { input, state };
            r5::winnow::dep_file.parse_next(&mut stream).unwrap();
        }

        #[test]
        fn check_has_escapes() {
            let text = r#"
            {
                "version": 1,
                "revision": 0,
                "rules": [
                    {
                        "work-directory": "build",
                        "primary-output": "fo\u{2764}o.\u{1f4af}o",
                        "outputs": [
                            "foo.d"
                        ],
                        "provides": [],
                        "requires": []
                    }
                ]
            }
            "#;
            let input = BStr::new(&text);
            let state = State::default();
            let mut stream = StateStream { input, state };
            let dep_file = r5::winnow::dep_file.parse_next(&mut stream).unwrap();
            assert_eq!(2, crate::util::count_escapes(text));
            assert_eq!(1, crate::util::count_escaped_strings(text).1);
            assert_eq!(0, dep_file.count_escapes_total());
            assert_eq!(1, dep_file.count_copies());
        }

        proptest! {
            #[cfg_attr(miri, ignore)]
            #[test]
            fn dep_file(text in r5::strategy::dep_file()) {
                let input = BStr::new(&text);
                let state = State::default();
                let mut stream = StateStream { input, state };
                r5::winnow::dep_file.parse_next(&mut stream).unwrap();
            }
        }

        pub mod required_module_desc {
            use super::*;

            proptest! {
                #[cfg_attr(miri, ignore)]
                #[test]
                fn lookup_method(text in r5::strategy::required_module_desc::lookup_method()) {
                    let input = BStr::new(&text);
                    let state = State::default();
                    let mut stream = StateStream { input, state };
                    r5::winnow::required_module_desc::lookup_method(&mut stream).unwrap();
                }
            }
        }
    }
}
