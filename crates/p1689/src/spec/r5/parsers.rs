// Almost all parsers are inherently fallible so don't require error docs.
#![allow(clippy::missing_errors_doc)]

use crate::{
    spec::r5,
    util::parsers::{ascii::multispace0, json, number, string, Error, Parser},
};

#[cfg_attr(test, derive(Eq, PartialEq))]
#[derive(Clone, Copy, Debug)]
pub enum ErrorKind {
    DepFile,
    DepInfo,
    LookupMethod,
    ProvidedModuleDesc,
    RequiredModuleDesc,
}
impl core::fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match &self {
            ErrorKind::DepFile => {
                writeln!(f, "Failed parsing DepFile fields:")?;
                writeln!(f, "expected one of: {{ \"revision\", \"rules\", \"version\" }}")?;
            },
            ErrorKind::DepInfo => {
                writeln!(f, "Failed parsing DepInfo fields:")?;
                writeln!(
                    f,
                    "expected one of: {{ \"outputs\", \"primary-output\", \"provides\", \"requires\", \"work-directory\" }}"
                )?;
            },
            ErrorKind::LookupMethod => {
                writeln!(f, "Failed parsing `LookupMethod`:")?;
                writeln!(
                    f,
                    "expected one of: {{ \"by-name\", \"include-angle\", \"include-quote\" }}"
                )?;
            },
            ErrorKind::ProvidedModuleDesc => {
                writeln!(f, "Failed parsing object fields:")?;
                writeln!(
                    f,
                    "expected one of: {{ \"compiled-module-path\", \"is-interface\", \"logical-name\", \"source-path\", \"unique-on-source-path\" }}"
                )?;
            },
            ErrorKind::RequiredModuleDesc => {
                writeln!(f, "Failed parsing object fields:")?;
                writeln!(
                    f,
                    "expected one of: {{ \"compiled-module-path\", \"logical-name\", \"lookup-method\", \"source-path\", \"unique-on-source-path\" }}"
                )?;
            },
        }
        Ok(())
    }
}

impl ErrorKind {
    fn error<'i>(self, stream: &mut ParseStream<'i>) -> crate::util::parsers::Error<'i, Self> {
        let error = crate::util::parsers::ErrorKind::Other { error: self };
        stream.error(error)
    }
}

type ParseStream<'i> = crate::util::parsers::ParseStream<'i, ErrorKind>;
type PResult<'i, T> = Result<T, Error<'i, ErrorKind>>;

#[rustfmt::skip]
pub fn dep_file<'i>(stream: &mut ParseStream<'i>) -> PResult<'i, r5::DepFile<'i>> {
    let fields = |stream0: &mut ParseStream<'i>| {
        let mut rules = None;
        let mut revision = Option::default();
        let mut version = None;
        while b'}' != stream0.peek_byte()? {
            let next0 = stream0.next_byte()?;
            match next0 {
                b'"' => { // tarpaulin::hint
                    let next1 = stream0.next_byte()?;
                    match next1 {
                        b'r' => { // tarpaulin::hint
                            let next2 = stream0.next_byte()?;
                            match next2 {
                                b'e' => { // tarpaulin::hint
                                    if revision.is_some() {
                                        let field = "revision";
                                        let error = crate::util::parsers::ErrorKind::DuplicateField { field };
                                        return Err(stream0.error(error));
                                    }
                                    let key = b"vision\"".as_slice();
                                    let val = self::number::dec_uint; // tarpaulin::hint
                                    let val = self::json::field(key, val).parse(stream0)?;
                                    revision = Some(val);
                                },
                                b'u' => { // tarpaulin::hint
                                    if rules.is_some() {
                                        let field = "rules";
                                        let error = crate::util::parsers::ErrorKind::DuplicateField { field };
                                        return Err(stream0.error(error));
                                    }
                                    let key = b"les\"".as_slice();
                                    let val = self::json::vec(dep_info);
                                    let val = self::json::field(key, val).parse(stream0)?;
                                    rules = Some(val);
                                },
                                _ => return Err(ErrorKind::DepFile.error(stream0)),
                            }
                        },
                        b'v' => { // tarpaulin::hint
                            if version.is_some() {
                                let field = "version";
                                let error = crate::util::parsers::ErrorKind::DuplicateField { field };
                                return Err(stream0.error(error));
                            }
                            let key = b"ersion\"".as_slice();
                            let val = self::number::dec_uint; // tarpaulin::hint
                            let val = self::json::field(key, val).parse(stream0)?;
                            version = Some(val);
                        },
                        _ => return Err(ErrorKind::DepFile.error(stream0)),
                    }
                },
                _ => return Err(ErrorKind::DepFile.error(stream0)),
            }
        }
        let dep_file = r5::DepFile {
            version: version.ok_or_else(|| {
                let field = "version";
                let error = crate::util::parsers::ErrorKind::MissingField { field };
                stream0.error(error)
            })?,
            revision,
            rules: rules.ok_or_else(|| {
                let field = "rules";
                let error = crate::util::parsers::ErrorKind::MissingField { field };
                stream0.error(error)
            })?,
        };
        Ok(dep_file)
    };
    multispace0.parse(stream)?;
    let res = self::json::record(fields).parse(stream)?;
    multispace0.parse(stream)?;
    Ok(res)
}

#[rustfmt::skip]
pub fn dep_info<'i>(stream: &mut ParseStream<'i>) -> PResult<'i, r5::DepInfo<'i>> {
    let fields = |stream0: &mut ParseStream<'i>| {
        let mut work_directory = Option::default();
        let mut primary_output = Option::default();
        let mut outputs = Option::default();
        let mut provides = Option::default();
        let mut requires = Option::default();
        while b'}' != stream0.peek_byte()? {
            let next0 = stream0.next_byte()?;
            match next0 {
                b'"' => { // tarpaulin::hint
                    let next1 = stream0.next_byte()?;
                    match next1 {
                        b'o' => { // tarpaulin::hint
                            if outputs.is_some() {
                                let field = "outputs";
                                let error = crate::util::parsers::ErrorKind::DuplicateField { field };
                                return Err(stream0.error(error));
                            }
                            let key = b"utputs\"".as_slice();
                            let val = self::json::vec(self::string::utf8_path);
                            let val = self::json::field(key, val).parse(stream0)?;
                            outputs = Some(val);
                        },
                        b'p' => { // tarpaulin::hint
                            let next2 = stream0.next_byte()?;
                            match next2 {
                                b'r' => { // tarpaulin::hint
                                    let next3 = stream0.next_byte()?;
                                    match next3 {
                                        b'i' => { // tarpaulin::hint
                                            if primary_output.is_some() {
                                                let field = "primary_output";
                                                let error = crate::util::parsers::ErrorKind::DuplicateField { field };
                                                return Err(stream0.error(error));
                                            }
                                            let key = b"mary-output\"".as_slice();
                                            let val = self::string::utf8_path; // tarpaulin::hint
                                            let val = self::json::field(key, val).parse(stream0)?;
                                            primary_output = Some(val);
                                        },
                                        b'o' => { // tarpaulin::hint
                                            if provides.is_some() {
                                                let field = "provides";
                                                let error = crate::util::parsers::ErrorKind::DuplicateField { field };
                                                return Err(stream0.error(error));
                                            }
                                            let key = b"vides\"".as_slice();
                                            let val = self::json::vec(provided_module_desc);
                                            let val = self::json::field(key, val).parse(stream0)?;
                                            provides = Some(val);
                                        },
                                        _ => return Err(ErrorKind::DepInfo.error(stream0)),
                                    }
                                },
                                _ => return Err(ErrorKind::DepInfo.error(stream0)),
                            }
                        },
                        b'r' => { // tarpaulin::hint
                            if requires.is_some() {
                                let field = "requires";
                                let error = crate::util::parsers::ErrorKind::DuplicateField { field };
                                return Err(stream0.error(error));
                            }
                            let key = b"equires\"".as_slice();
                            let val = self::json::vec(required_module_desc);
                            let val = self::json::field(key, val).parse(stream0)?;
                            requires = Some(val);
                        },
                        b'w' => { // tarpaulin::hint
                            if work_directory.is_some() {
                                let field = "work-directory";
                                let error = crate::util::parsers::ErrorKind::DuplicateField { field };
                                return Err(stream0.error(error));
                            }
                            let key = b"ork-directory\"".as_slice();
                            let val = self::string::utf8_path; // tarpaulin::hint
                            let val = self::json::field(key, val).parse(stream0)?;
                            work_directory = Some(val);
                        },
                        _ => return Err(ErrorKind::DepInfo.error(stream0)),
                    }
                },
                _ => return Err(ErrorKind::DepInfo.error(stream0)),
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
    multispace0.parse(stream)?;
    let res = self::json::record(fields).parse(stream)?;
    multispace0.parse(stream)?;
    Ok(res)
}
pub mod dep_info {}

#[allow(clippy::too_many_lines)]
#[rustfmt::skip]
pub fn provided_module_desc<'i>(stream: &mut ParseStream<'i>) -> PResult<'i, r5::ProvidedModuleDesc<'i>> {
    let fields = |stream0: &mut ParseStream<'i>| {
        let mut source_path = None;
        let mut compiled_module_path = None;
        let mut logical_name = None;
        let mut unique_on_source_path = None;
        let mut is_interface = None;
        while b'}' != stream0.peek_byte()? {
            let next0 = stream0.next_byte()?;
            match next0 {
                b'"' => { // tarpaulin::hint
                    let next1 = stream0.next_byte()?;
                    match next1 {
                        b's' => { // tarpaulin::hint
                            if source_path.is_some() {
                                let field = "source-path";
                                let error = crate::util::parsers::ErrorKind::DuplicateField { field };
                                return Err(stream0.error(error));
                            }
                            let key = b"ource-path\"".as_slice();
                            let val = self::string::utf8_path; // tarpaulin::hint
                            let val = self::json::field(key, val).parse(stream0)?;
                            source_path = Some(val);
                        },
                        b'c' => { // tarpaulin::hint
                            if compiled_module_path.is_some() {
                                let field = "compiled-module-path";
                                let error = crate::util::parsers::ErrorKind::DuplicateField { field };
                                return Err(stream0.error(error));
                            }
                            let key = b"ompiled-module-path\"".as_slice();
                            let val = self::string::utf8_path; // tarpaulin::hint
                            let val = self::json::field(key, val).parse(stream0)?;
                            compiled_module_path = Some(val);
                        },
                        b'l' => { // tarpaulin::hint
                            if logical_name.is_some() {
                                let field = "logical-name";
                                let error = crate::util::parsers::ErrorKind::DuplicateField { field };
                                return Err(stream0.error(error));
                            }
                            let key = b"ogical-name\"".as_slice();
                            let val = self::string::json_string; // tarpaulin::hint
                            let val = self::json::field(key, val).parse(stream0)?;
                            logical_name = Some(val);
                        },
                        b'u' => { // tarpaulin::hint
                            if unique_on_source_path.is_some() {
                                let field = "unique-on-source-path";
                                let error = crate::util::parsers::ErrorKind::DuplicateField { field };
                                return Err(stream0.error(error));
                            }
                            let key = b"nique-on-source-path\"".as_slice();
                            let val = self::json::bool; // tarpaulin::hint
                            let val = self::json::field(key, val).parse(stream0)?;
                            unique_on_source_path = Some(val);
                        },
                        b'i' => { // tarpaulin::hint
                            if is_interface.is_some() {
                                let field = "is-interface";
                                let error = crate::util::parsers::ErrorKind::DuplicateField { field };
                                return Err(stream0.error(error));
                            }
                            let key = b"s-interface\"".as_slice();
                            let val = self::json::bool; // tarpaulin::hint
                            let val = self::json::field(key, val).parse(stream0)?;
                            is_interface = Some(val);
                        },
                        _ => return Err(ErrorKind::ProvidedModuleDesc.error(stream0)),
                    }
                },
                _ => return Err(ErrorKind::ProvidedModuleDesc.error(stream0)),
            }
        }
        let desc = r5::ProvidedModuleDesc {
            desc: if unique_on_source_path.unwrap_or(false) {
                r5::ModuleDesc::BySourcePath {
                    source_path: source_path.ok_or_else(|| {
                        let field = "source-path";
                        let error = crate::util::parsers::ErrorKind::MissingField { field };
                        stream0.error(error)
                    })?,
                    compiled_module_path,
                    logical_name: logical_name.ok_or_else(|| {
                        let field = "logical_name";
                        let error = crate::util::parsers::ErrorKind::MissingField { field };
                        stream0.error(error)
                    })?,
                    #[cfg(any(test, feature = "monostate"))]
                    unique_on_source_path: monostate::MustBe!(true),
                }
            } else {
                r5::ModuleDesc::ByLogicalName {
                    source_path,
                    compiled_module_path,
                    logical_name: logical_name.ok_or_else(|| {
                        let field = "logical-name";
                        let error = crate::util::parsers::ErrorKind::MissingField { field };
                        stream0.error(error)
                    })?,
                    #[cfg(any(test, feature = "monostate"))]
                    unique_on_source_path: unique_on_source_path.and(Some(monostate::MustBe!(false))),
                }
            },
            is_interface: is_interface.ok_or_else(|| {
                let field = "is-interface";
                let error = crate::util::parsers::ErrorKind::MissingField { field };
                stream0.error(error)
            })?,
        };
        Ok(desc)
    };
    multispace0.parse(stream)?;
    let res = self::json::record(fields).parse(stream)?;
    multispace0.parse(stream)?;
    Ok(res)
}

#[allow(clippy::too_many_lines)]
#[rustfmt::skip]
pub fn required_module_desc<'i>(stream: &mut ParseStream<'i>) -> PResult<'i, r5::RequiredModuleDesc<'i>> {
    let fields = |stream0: &mut ParseStream<'i>| {
        let mut source_path = None;
        let mut compiled_module_path = None;
        let mut lookup_method = None;
        let mut unique_on_source_path = None;
        let mut logical_name = None;
        while b'}' != stream0.peek_byte()? {
            let next0 = stream0.next_byte()?;
            match next0 {
                b'"' => { // tarpaulin::hint
                    let next1 = stream0.next_byte()?;
                    match next1 {
                        b's' => { // tarpaulin::hint
                            if source_path.is_some() {
                                let field = "source-path";
                                let error = crate::util::parsers::ErrorKind::DuplicateField { field };
                                return Err(stream0.error(error));
                            }
                            let key = b"ource-path\"".as_slice();
                            let val = self::string::utf8_path; // tarpaulin::hint
                            let val = self::json::field(key, val).parse(stream0)?;
                            source_path = Some(val);
                        },
                        b'c' => { // tarpaulin::hint
                            if compiled_module_path.is_some() {
                                let field = "compiled-module-path";
                                let error = crate::util::parsers::ErrorKind::DuplicateField { field };
                                return Err(stream0.error(error));
                            }
                            let key = b"ompiled-module-path\"".as_slice();
                            let val = self::string::utf8_path; // tarpaulin::hint
                            let val = self::json::field(key, val).parse(stream0)?;
                            compiled_module_path = Some(val);
                        },
                        b'l' => { // tarpaulin::hint
                            match stream0.next_slice(2usize)? {
                                b"og" => {
                                    if logical_name.is_some() {
                                        let field = "logical-name";
                                        let error = crate::util::parsers::ErrorKind::DuplicateField { field };
                                        return Err(stream0.error(error));
                                    }
                                    let key = b"ical-name\"".as_slice();
                                    let val = self::string::json_string; // tarpaulin::hint
                                    let val = self::json::field(key, val).parse(stream0)?;
                                    logical_name = Some(val);
                                },
                                b"oo" => { // tarpaulin::hint
                                    if lookup_method.is_some() {
                                        let field = "lookup-method";
                                        let error = crate::util::parsers::ErrorKind::DuplicateField { field };
                                        return Err(stream0.error(error));
                                    }
                                    let key = b"kup-method\"".as_slice();
                                    let val = self::required_module_desc::lookup_method; // tarpaulin::hint
                                    let val = self::json::field(key, val).parse(stream0)?;
                                    lookup_method = Some(val);
                                },
                                _ => return Err(ErrorKind::RequiredModuleDesc.error(stream0)),
                            }
                        },
                        b'u' => { // tarpaulin::hint
                            if unique_on_source_path.is_some() {
                                let field = "unique-on-source-path";
                                let error = crate::util::parsers::ErrorKind::DuplicateField { field };
                                return Err(stream0.error(error));
                            }
                            let key = b"nique-on-source-path\"".as_slice();
                            let val = self::json::bool; // tarpaulin::hint
                            let val = self::json::field(key, val).parse(stream0)?;
                            unique_on_source_path = Some(val);
                        },
                        _ => return Err(ErrorKind::RequiredModuleDesc.error(stream0)),
                    }
                },
                _ => return Err(ErrorKind::RequiredModuleDesc.error(stream0)),
            }
        }
        let desc = r5::RequiredModuleDesc {
            desc: if unique_on_source_path.unwrap_or(false) {
                r5::ModuleDesc::BySourcePath {
                    source_path: source_path.ok_or_else(|| {
                        let field = "source-path";
                        let error = crate::util::parsers::ErrorKind::MissingField { field };
                        stream0.error(error)
                    })?,
                    compiled_module_path,
                    logical_name: logical_name.ok_or_else(|| {
                        let field = "logical-name";
                        let error = crate::util::parsers::ErrorKind::MissingField { field };
                        stream0.error(error)
                    })?,
                    #[cfg(any(test, feature = "monostate"))]
                    unique_on_source_path: monostate::MustBe!(true),
                }
            } else {
                r5::ModuleDesc::ByLogicalName {
                    source_path,
                    compiled_module_path,
                    logical_name: logical_name.ok_or_else(|| {
                        let field = "logical-name";
                        let error = crate::util::parsers::ErrorKind::MissingField { field };
                        stream0.error(error)
                    })?,
                    #[cfg(any(test, feature = "monostate"))]
                    unique_on_source_path: unique_on_source_path.and(Some(monostate::MustBe!(false))),
                }
            },
            lookup_method: lookup_method.unwrap_or_default(),
        };
        Ok(desc)
    };
    multispace0.parse(stream)?;
    let res = self::json::record(fields).parse(stream)?;
    multispace0.parse(stream)?;
    Ok(res)
}

pub mod required_module_desc {
    #[allow(clippy::wildcard_imports)]
    use super::*;

    #[rustfmt::skip]
    pub fn lookup_method<'i>(stream: &mut ParseStream<'i>) -> PResult<'i, r5::RequiredModuleDescLookupMethod> {
        match stream.next_byte()? {
            b'"' => match stream.next_byte()? {
                b'b' => { // tarpaulin::hint
                    stream.match_slice(b"y-name\"")?;
                    Ok(r5::RequiredModuleDescLookupMethod::ByName)
                },
                b'i' => match stream.next_slice(13)? {
                    b"nclude-angle\"" => Ok(r5::RequiredModuleDescLookupMethod::IncludeAngle),
                    b"nclude-quote\"" => Ok(r5::RequiredModuleDescLookupMethod::IncludeQuote),
                    _ => return Err(ErrorKind::LookupMethod.error(stream)),
                },
                _ => return Err(ErrorKind::LookupMethod.error(stream)),
            },
            _ => return Err(ErrorKind::LookupMethod.error(stream)),
        }
    }
}

#[cfg(test)]
mod test {
    use proptest::proptest;
    #[cfg(feature = "datagen")]
    use rand::prelude::*;

    use super::*;
    #[cfg(feature = "datagen")]
    use crate::util::parsers::ParseStream;
    use crate::util::parsers::State;

    mod r5 {
        pub use crate::spec::r5::{proptest::strategy, *};
    }

    mod error_kind {
        use super::*;

        #[test]
        fn error() {
            let path = "test.ddi";
            let input = b"";
            let state = State::default();
            let stream = &mut ParseStream::new(path, input, state);
            for e in &[
                ErrorKind::DepFile,
                ErrorKind::DepInfo,
                ErrorKind::LookupMethod,
                ErrorKind::ProvidedModuleDesc,
                ErrorKind::RequiredModuleDesc,
            ] {
                match e.error(stream).error {
                    crate::util::parsers::ErrorKind::Other { error } => assert_eq!(e, &error),
                    _ => panic!(),
                }
            }
        }
    }

    mod parse {

        use super::*;

        pub mod dep_file {
            use super::*;

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
                    if let Some(dep_file_text) = dep_file_texts.next() {
                        let num_escaped_strings_within_file = crate::util::count_escaped_strings(&dep_file_text).1;
                        let path = "test.ddi";
                        let input = dep_file_text.as_bytes();
                        let state = crate::util::parsers::State::default();
                        let mut stream = ParseStream::new(path, input, state);
                        let dep_file = r5::parsers::dep_file(&mut stream).unwrap();
                        assert_eq!(num_escaped_strings_within_file, dep_file.count_copies());
                        num_files_with_escaped_strings += u64::from(0 < num_escaped_strings_within_file);
                    }
                }
            }

            pub mod errors {
                use super::*;

                #[test]
                #[should_panic(expected = "test.ddi:6:28: error: Duplicate field: `revision`\n")]
                fn duplicate_field_revision() {
                    let text = r#"{
                        "version": 1,
                        "revision": 0,
                        "rules": [
                        ],
                        "revision": 0,
                    }"#;
                    let path = "test.ddi";
                    let input = text.as_bytes();
                    let state = State::default();
                    let mut stream = ParseStream::new(path, input, state);
                    if let Err(err) = r5::parsers::dep_file.parse(&mut stream) {
                        panic!("{err}");
                    }
                }

                #[test]
                #[should_panic(expected = "test.ddi:6:28: error: Duplicate field: `rules`\n")]
                fn duplicate_field_rules() {
                    let text = r#"{
                        "version": 1,
                        "revision": 0,
                        "rules": [
                        ],
                        "rules": [
                        ],
                    }"#;
                    let path = "test.ddi";
                    let input = text.as_bytes();
                    let state = State::default();
                    let mut stream = ParseStream::new(path, input, state);
                    if let Err(err) = r5::parsers::dep_file.parse(&mut stream) {
                        panic!("{err}");
                    }
                }

                #[test]
                #[should_panic(expected = "test.ddi:6:27: error: Duplicate field: `version`\n")]
                fn duplicate_field_version() {
                    let text = r#"{
                        "version": 1,
                        "revision": 0,
                        "rules": [
                        ],
                        "version": 1,
                    }"#;
                    let path = "test.ddi";
                    let input = text.as_bytes();
                    let state = State::default();
                    let mut stream = ParseStream::new(path, input, state);
                    if let Err(err) = r5::parsers::dep_file.parse(&mut stream) {
                        panic!("{err}");
                    }
                }

                #[test]
                #[should_panic(expected = "test.ddi:3:21: error: Missing field: `rules`\n")]
                fn missing_field_rules() {
                    let text = r#"{
                        "version": 1
                    }"#;
                    let path = "test.ddi";
                    let input = text.as_bytes();
                    let state = State::default();
                    let mut stream = ParseStream::new(path, input, state);
                    if let Err(err) = r5::parsers::dep_file.parse(&mut stream) {
                        panic!("{err}");
                    }
                }

                #[test]
                #[should_panic(expected = "test.ddi:3:21: error: Missing field: `version`\n")]
                fn missing_field_version() {
                    let text = r#"{
                        "rules": []
                    }"#;
                    let path = "test.ddi";
                    let input = text.as_bytes();
                    let state = State::default();
                    let mut stream = ParseStream::new(path, input, state);
                    if let Err(err) = r5::parsers::dep_file.parse(&mut stream) {
                        panic!("{err}");
                    }
                }

                #[test]
                #[should_panic(
                    expected = "test.ddi:3:28: error: Failed parsing DepFile fields:\nexpected one of: { \"revision\", \"rules\", \"version\" }\n"
                )]
                fn mismatch_field_revision() {
                    let text = r#"{
                        "version": 1,
                        "r#vision": 0,
                        "rules": [
                        ],
                    }"#;
                    let path = "test.ddi";
                    let input = text.as_bytes();
                    let state = State::default();
                    let mut stream = ParseStream::new(path, input, state);
                    if let Err(err) = r5::parsers::dep_file.parse(&mut stream) {
                        panic!("{err}");
                    }
                }

                #[test]
                #[should_panic(
                    expected = "test.ddi:4:28: error: Failed parsing DepFile fields:\nexpected one of: { \"revision\", \"rules\", \"version\" }\n"
                )]
                fn mismatch_field_rules() {
                    let text = r#"{
                        "version": 1,
                        "revision": 0,
                        "r#les": [
                        ],
                    }"#;
                    let path = "test.ddi";
                    let input = text.as_bytes();
                    let state = State::default();
                    let mut stream = ParseStream::new(path, input, state);
                    if let Err(err) = r5::parsers::dep_file.parse(&mut stream) {
                        panic!("{err}");
                    }
                }

                #[test]
                #[should_panic(
                    expected = "test.ddi:2:27: error: Failed parsing DepFile fields:\nexpected one of: { \"revision\", \"rules\", \"version\" }\n"
                )]
                fn mismatch_field() {
                    let text = r#"{
                        "bad": [],
                        "version": 1,
                        "revision": 0,
                        "rules": [
                        ],
                    }"#;
                    let path = "test.ddi";
                    let input = text.as_bytes();
                    let state = State::default();
                    let mut stream = ParseStream::new(path, input, state);
                    if let Err(err) = r5::parsers::dep_file.parse(&mut stream) {
                        panic!("{err}");
                    }
                }

                #[test]
                #[should_panic(
                    expected = "test.ddi:4:26: error: Failed parsing DepFile fields:\nexpected one of: { \"revision\", \"rules\", \"version\" }\n"
                )]
                fn mismatch_field_unquoted() {
                    let text = r#"{
                        "version": 1,
                        "revision": 0,
                        bad: [],
                        "rules": [
                        ],
                    }"#;
                    let path = "test.ddi";
                    let input = text.as_bytes();
                    let state = State::default();
                    let mut stream = ParseStream::new(path, input, state);
                    if let Err(err) = r5::parsers::dep_file.parse(&mut stream) {
                        panic!("{err}");
                    }
                }
            }
        }

        mod dep_info {
            use super::*;

            mod errors {
                use super::*;

                #[test]
                #[should_panic(expected = "test.ddi:7:35: error: Duplicate field: `outputs`\n")]
                fn duplicate_field_outputs() {
                    let text = r#"{
                        "version": 1,
                        "revision": 0,
                        "rules": [
                            {
                                "outputs": [],
                                "outputs": [],
                            }
                        ],
                    }"#;
                    let path = "test.ddi";
                    let input = text.as_bytes();
                    let state = State::default();
                    let mut stream = ParseStream::new(path, input, state);
                    if let Err(err) = r5::parsers::dep_file.parse(&mut stream) {
                        panic!("{err}");
                    }
                }

                #[test]
                #[should_panic(expected = "test.ddi:7:37: error: Duplicate field: `primary_output`\n")]
                fn duplicate_field_primary_output() {
                    let text = r#"{
                        "version": 1,
                        "revision": 0,
                        "rules": [
                            {
                                "primary-output": "foo.cpp",
                                "primary-output": "foo.cpp"
                            }
                        ],
                    }"#;
                    let path = "test.ddi";
                    let input = text.as_bytes();
                    let state = State::default();
                    let mut stream = ParseStream::new(path, input, state);
                    if let Err(err) = r5::parsers::dep_file.parse(&mut stream) {
                        panic!("{err}");
                    }
                }

                #[test]
                #[should_panic(expected = "test.ddi:7:37: error: Duplicate field: `provides`\n")]
                fn duplicate_field_provides() {
                    let text = r#"{
                        "version": 1,
                        "revision": 0,
                        "rules": [
                            {
                                "provides": [],
                                "provides": []
                            }
                        ],
                    }"#;
                    let path = "test.ddi";
                    let input = text.as_bytes();
                    let state = State::default();
                    let mut stream = ParseStream::new(path, input, state);
                    if let Err(err) = r5::parsers::dep_file.parse(&mut stream) {
                        panic!("{err}");
                    }
                }

                #[test]
                #[should_panic(expected = "test.ddi:7:35: error: Duplicate field: `requires`\n")]
                fn duplicate_field_requires() {
                    let text = r#"{
                        "version": 1,
                        "revision": 0,
                        "rules": [
                            {
                                "requires": [],
                                "requires": []
                            }
                        ],
                    }"#;
                    let path = "test.ddi";
                    let input = text.as_bytes();
                    let state = State::default();
                    let mut stream = ParseStream::new(path, input, state);
                    if let Err(err) = r5::parsers::dep_file.parse(&mut stream) {
                        panic!("{err}");
                    }
                }

                #[test]
                #[should_panic(expected = "test.ddi:7:35: error: Duplicate field: `work-directory`\n")]
                fn duplicate_field_work_directory() {
                    let text = r#"{
                        "version": 1,
                        "revision": 0,
                        "rules": [
                            {
                                "work-directory": "build",
                                "work-directory": "build"
                            }
                        ],
                    }"#;
                    let path = "test.ddi";
                    let input = text.as_bytes();
                    let state = State::default();
                    let mut stream = ParseStream::new(path, input, state);
                    if let Err(err) = r5::parsers::dep_file.parse(&mut stream) {
                        panic!("{err}");
                    }
                }

                #[test]
                #[should_panic(
                    expected = "test.ddi:6:37: error: Failed parsing DepInfo fields:\nexpected one of: { \"outputs\", \"primary-output\", \"provides\", \"requires\", \"work-directory\" }\n"
                )]
                fn mismatch_field_primary_output() {
                    let text = r#"{
                        "version": 1,
                        "revision": 0,
                        "rules": [
                            {
                                "prxmary-output": "foo.cpp",
                                "provides": []
                            }
                        ],
                    }"#;
                    let path = "test.ddi";
                    let input = text.as_bytes();
                    let state = State::default();
                    let mut stream = ParseStream::new(path, input, state);
                    if let Err(err) = r5::parsers::dep_file.parse(&mut stream) {
                        panic!("{err}");
                    }
                }

                #[test]
                #[should_panic(
                    expected = "test.ddi:7:33: error: Failed parsing DepInfo fields:\nexpected one of: { \"outputs\", \"primary-output\", \"provides\", \"requires\", \"work-directory\" }\n"
                )]
                fn mismatch_field_provides() {
                    let text = r#"{
                    "version": 1,
                    "revision": 0,
                    "rules": [
                        {
                            "primary-output": "foo.cpp",
                            "prxvides": []
                        }
                    ],
                }"#;
                    let path = "test.ddi";
                    let input = text.as_bytes();
                    let state = State::default();
                    let mut stream = ParseStream::new(path, input, state);
                    if let Err(err) = r5::parsers::dep_file.parse(&mut stream) {
                        panic!("{err}");
                    }
                }

                #[test]
                #[should_panic(
                    expected = "test.ddi:6:32: error: Failed parsing DepInfo fields:\nexpected one of: { \"outputs\", \"primary-output\", \"provides\", \"requires\", \"work-directory\" }\n"
                )]
                fn mismatch_field_pr() {
                    let text = r#"{
                    "version": 1,
                    "revision": 0,
                    "rules": [
                        {
                            "px": []
                        }
                    ],
                }"#;
                    let path = "test.ddi";
                    let input = text.as_bytes();
                    let state = State::default();
                    let mut stream = ParseStream::new(path, input, state);
                    if let Err(err) = r5::parsers::dep_file.parse(&mut stream) {
                        panic!("{err}");
                    }
                }

                #[test]
                #[should_panic(
                    expected = "test.ddi:6:31: error: Failed parsing DepInfo fields:\nexpected one of: { \"outputs\", \"primary-output\", \"provides\", \"requires\", \"work-directory\" }\n"
                )]
                fn mismatch_field() {
                    let text = r#"{
                    "version": 1,
                    "revision": 0,
                    "rules": [
                        {
                            "x": []
                        }
                    ],
                }"#;
                    let path = "test.ddi";
                    let input = text.as_bytes();
                    let state = State::default();
                    let mut stream = ParseStream::new(path, input, state);
                    if let Err(err) = r5::parsers::dep_file.parse(&mut stream) {
                        panic!("{err}");
                    }
                }

                #[test]
                #[should_panic(
                    expected = "test.ddi:6:30: error: Failed parsing DepInfo fields:\nexpected one of: { \"outputs\", \"primary-output\", \"provides\", \"requires\", \"work-directory\" }\n"
                )]
                fn mismatch_field_unquoted() {
                    let text = r#"{
                    "version": 1,
                    "revision": 0,
                    "rules": [
                        {
                            primary-output: "foo.cpp"
                        }
                    ],
                }"#;
                    let path = "test.ddi";
                    let input = text.as_bytes();
                    let state = State::default();
                    let mut stream = ParseStream::new(path, input, state);
                    if let Err(err) = r5::parsers::dep_file.parse(&mut stream) {
                        panic!("{err}");
                    }
                }
            }
        }

        mod provided_module_desc {
            use super::*;

            mod errors {
                use super::*;

                #[test]
                #[should_panic(expected = "test.ddi:9:43: error: Duplicate field: `source-path`\n")]
                fn duplicate_field_source_path() {
                    let text = r#"{
                        "version": 1,
                        "revision": 0,
                        "rules": [
                            {
                                "provides": [
                                    {
                                        "source-path": "foo.cpp",
                                        "source-path": "foo.cpp"
                                    }
                                ]
                            }
                        ],
                    }"#;
                    let path = "test.ddi";
                    let input = text.as_bytes();
                    let state = State::default();
                    let mut stream = ParseStream::new(path, input, state);
                    if let Err(err) = r5::parsers::dep_file.parse(&mut stream) {
                        panic!("{err}");
                    }
                }

                #[test]
                #[should_panic(expected = "test.ddi:9:43: error: Duplicate field: `compiled-module-path`\n")]
                fn duplicate_field_compiled_module_path() {
                    let text = r#"{
                        "version": 1,
                        "revision": 0,
                        "rules": [
                            {
                                "provides": [
                                    {
                                        "compiled-module-path": "foo.o",
                                        "compiled-module-path": "foo.o"
                                    }
                                ]
                            }
                        ],
                    }"#;
                    let path = "test.ddi";
                    let input = text.as_bytes();
                    let state = State::default();
                    let mut stream = ParseStream::new(path, input, state);
                    if let Err(err) = r5::parsers::dep_file.parse(&mut stream) {
                        panic!("{err}");
                    }
                }

                #[test]
                #[should_panic(expected = "test.ddi:9:43: error: Duplicate field: `logical-name`\n")]
                fn duplicate_field_logical_name() {
                    let text = r#"{
                        "version": 1,
                        "revision": 0,
                        "rules": [
                            {
                                "provides": [
                                    {
                                        "logical-name": "foo",
                                        "logical-name": "foo"
                                    }
                                ]
                            }
                        ],
                    }"#;
                    let path = "test.ddi";
                    let input = text.as_bytes();
                    let state = State::default();
                    let mut stream = ParseStream::new(path, input, state);
                    if let Err(err) = r5::parsers::dep_file.parse(&mut stream) {
                        panic!("{err}");
                    }
                }

                #[test]
                #[should_panic(expected = "test.ddi:9:43: error: Duplicate field: `unique-on-source-path`\n")]
                fn duplicate_field_unique_on_source_path() {
                    let text = r#"{
                        "version": 1,
                        "revision": 0,
                        "rules": [
                            {
                                "provides": [
                                    {
                                        "unique-on-source-path": false,
                                        "unique-on-source-path": false
                                    }
                                ]
                            }
                        ],
                    }"#;
                    let path = "test.ddi";
                    let input = text.as_bytes();
                    let state = State::default();
                    let mut stream = ParseStream::new(path, input, state);
                    if let Err(err) = r5::parsers::dep_file.parse(&mut stream) {
                        panic!("{err}");
                    }
                }

                #[test]
                #[should_panic(expected = "test.ddi:9:43: error: Duplicate field: `is-interface`\n")]
                fn duplicate_field_is_interface() {
                    let text = r#"{
                        "version": 1,
                        "revision": 0,
                        "rules": [
                            {
                                "provides": [
                                    {
                                        "is-interface": false,
                                        "is-interface": false
                                    }
                                ]
                            }
                        ],
                    }"#;
                    let path = "test.ddi";
                    let input = text.as_bytes();
                    let state = State::default();
                    let mut stream = ParseStream::new(path, input, state);
                    if let Err(err) = r5::parsers::dep_file.parse(&mut stream) {
                        panic!("{err}");
                    }
                }

                #[test]
                #[should_panic(
                    expected = "test.ddi:8:43: error: Failed parsing object fields:\nexpected one of: { \"compiled-module-path\", \"is-interface\", \"logical-name\", \"source-path\", \"unique-on-source-path\" }\n"
                )]
                fn mismatch_field() {
                    let text = r#"{
                        "version": 1,
                        "revision": 0,
                        "rules": [
                            {
                                "provides": [
                                    {
                                        "x": []
                                    }
                                ]
                            }
                        ],
                    }"#;
                    let path = "test.ddi";
                    let input = text.as_bytes();
                    let state = State::default();
                    let mut stream = ParseStream::new(path, input, state);
                    if let Err(err) = r5::parsers::dep_file.parse(&mut stream) {
                        panic!("{err}");
                    }
                }

                #[test]
                #[should_panic(
                    expected = "test.ddi:8:42: error: Failed parsing object fields:\nexpected one of: { \"compiled-module-path\", \"is-interface\", \"logical-name\", \"source-path\", \"unique-on-source-path\" }\n"
                )]
                fn mismatch_field_unquoted() {
                    let text = r#"{
                        "version": 1,
                        "revision": 0,
                        "rules": [
                            {
                                "provides": [
                                    {
                                        x: []
                                    }
                                ]
                            }
                        ],
                    }"#;
                    let path = "test.ddi";
                    let input = text.as_bytes();
                    let state = State::default();
                    let mut stream = ParseStream::new(path, input, state);
                    if let Err(err) = r5::parsers::dep_file.parse(&mut stream) {
                        panic!("{err}");
                    }
                }
            }
        }

        mod required_module_desc {
            use super::*;

            proptest! {
                #[cfg_attr(miri, ignore)]
                #[test]
                fn lookup_method(text in r5::strategy::required_module_desc::lookup_method()) {
                    let path = "test.ddi";
                    let input = text.as_bytes();
                    let state = State::default();
                    let mut stream = ParseStream::new(path, input, state);
                    r5::parsers::required_module_desc::lookup_method(&mut stream).unwrap();
                }
            }

            mod errors {
                use super::*;

                #[test]
                #[should_panic(expected = "test.ddi:9:43: error: Duplicate field: `source-path`\n")]
                fn duplicate_field_source_path() {
                    let text = r#"{
                        "version": 1,
                        "revision": 0,
                        "rules": [
                            {
                                "requires": [
                                    {
                                        "source-path": "foo.cpp",
                                        "source-path": "foo.cpp"
                                    }
                                ]
                            }
                        ],
                    }"#;
                    let path = "test.ddi";
                    let input = text.as_bytes();
                    let state = State::default();
                    let mut stream = ParseStream::new(path, input, state);
                    if let Err(err) = r5::parsers::dep_file.parse(&mut stream) {
                        panic!("{err}");
                    }
                }

                #[test]
                #[should_panic(expected = "test.ddi:9:43: error: Duplicate field: `compiled-module-path`\n")]
                fn duplicate_field_compiled_module_path() {
                    let text = r#"{
                        "version": 1,
                        "revision": 0,
                        "rules": [
                            {
                                "requires": [
                                    {
                                        "compiled-module-path": "foo.o",
                                        "compiled-module-path": "foo.o"
                                    }
                                ]
                            }
                        ],
                    }"#;
                    let path = "test.ddi";
                    let input = text.as_bytes();
                    let state = State::default();
                    let mut stream = ParseStream::new(path, input, state);
                    if let Err(err) = r5::parsers::dep_file.parse(&mut stream) {
                        panic!("{err}");
                    }
                }

                #[test]
                #[should_panic(expected = "test.ddi:9:45: error: Duplicate field: `logical-name`\n")]
                fn duplicate_field_logical_name() {
                    let text = r#"{
                        "version": 1,
                        "revision": 0,
                        "rules": [
                            {
                                "requires": [
                                    {
                                        "logical-name": "foo",
                                        "logical-name": "foo"
                                    }
                                ]
                            }
                        ],
                    }"#;
                    let path = "test.ddi";
                    let input = text.as_bytes();
                    let state = State::default();
                    let mut stream = ParseStream::new(path, input, state);
                    if let Err(err) = r5::parsers::dep_file.parse(&mut stream) {
                        panic!("{err}");
                    }
                }

                #[test]
                #[should_panic(expected = "test.ddi:9:43: error: Duplicate field: `unique-on-source-path`\n")]
                fn duplicate_field_unique_on_source_path() {
                    let text = r#"{
                        "version": 1,
                        "revision": 0,
                        "rules": [
                            {
                                "requires": [
                                    {
                                        "unique-on-source-path": false,
                                        "unique-on-source-path": false
                                    }
                                ]
                            }
                        ],
                    }"#;
                    let path = "test.ddi";
                    let input = text.as_bytes();
                    let state = State::default();
                    let mut stream = ParseStream::new(path, input, state);
                    if let Err(err) = r5::parsers::dep_file.parse(&mut stream) {
                        panic!("{err}");
                    }
                }

                #[test]
                #[should_panic(expected = "test.ddi:9:45: error: Duplicate field: `lookup-method`\n")]
                fn duplicate_field_lookup_method() {
                    let text = r#"{
                        "version": 1,
                        "revision": 0,
                        "rules": [
                            {
                                "requires": [
                                    {
                                        "lookup-method": "by-name",
                                        "lookup-method": "by-name"
                                    }
                                ]
                            }
                        ],
                    }"#;
                    let path = "test.ddi";
                    let input = text.as_bytes();
                    let state = State::default();
                    let mut stream = ParseStream::new(path, input, state);
                    if let Err(err) = r5::parsers::dep_file.parse(&mut stream) {
                        panic!("{err}");
                    }
                }

                #[test]
                #[should_panic(
                    expected = "test.ddi:8:45: error: Failed parsing object fields:\nexpected one of: { \"compiled-module-path\", \"logical-name\", \"lookup-method\", \"source-path\", \"unique-on-source-path\" }\n"
                )]
                fn mismatch_field_l() {
                    let text = r#"{
                        "version": 1,
                        "revision": 0,
                        "rules": [
                            {
                                "requires": [
                                    {
                                        "lx": []
                                    }
                                ]
                            }
                        ],
                    }"#;
                    let path = "test.ddi";
                    let input = text.as_bytes();
                    let state = State::default();
                    let mut stream = ParseStream::new(path, input, state);
                    if let Err(err) = r5::parsers::dep_file.parse(&mut stream) {
                        panic!("{err}");
                    }
                }

                #[test]
                #[should_panic(
                    expected = "test.ddi:8:43: error: Failed parsing object fields:\nexpected one of: { \"compiled-module-path\", \"logical-name\", \"lookup-method\", \"source-path\", \"unique-on-source-path\" }\n"
                )]
                fn mismatch_field() {
                    let text = r#"{
                        "version": 1,
                        "revision": 0,
                        "rules": [
                            {
                                "requires": [
                                    {
                                        "x": []
                                    }
                                ]
                            }
                        ],
                    }"#;
                    let path = "test.ddi";
                    let input = text.as_bytes();
                    let state = State::default();
                    let mut stream = ParseStream::new(path, input, state);
                    if let Err(err) = r5::parsers::dep_file.parse(&mut stream) {
                        panic!("{err}");
                    }
                }

                #[test]
                #[should_panic(
                    expected = "test.ddi:8:42: error: Failed parsing object fields:\nexpected one of: { \"compiled-module-path\", \"logical-name\", \"lookup-method\", \"source-path\", \"unique-on-source-path\" }\n"
                )]
                fn mismatch_field_unquoted() {
                    let text = r#"{
                        "version": 1,
                        "revision": 0,
                        "rules": [
                            {
                                "requires": [
                                    {
                                        x: []
                                    }
                                ]
                            }
                        ],
                    }"#;
                    let path = "test.ddi";
                    let input = text.as_bytes();
                    let state = State::default();
                    let mut stream = ParseStream::new(path, input, state);
                    if let Err(err) = r5::parsers::dep_file.parse(&mut stream) {
                        panic!("{err}");
                    }
                }
            }

            mod lookup_method {
                use super::*;

                mod errors {
                    use super::*;

                    #[test]
                    #[should_panic(
                        expected = "test.ddi:10:6: error: Failed parsing `LookupMethod`:\nexpected one of: { \"by-name\", \"include-angle\", \"include-quote\" }\n"
                    )]
                    fn mismatch_field_include() {
                        let text = r#"{
                            "version": 1,
                            "revision": 0,
                            "rules": [
                                {
                                    "requires": [
                                        {
                                            "logical-name": "foo",
                                            "lookup-method": "invalid"
                                        }
                                    ]
                                }
                            ],
                        }"#;
                        let path = "test.ddi";
                        let input = text.as_bytes();
                        let state = State::default();
                        let mut stream = ParseStream::new(path, input, state);
                        if let Err(err) = r5::parsers::dep_file.parse(&mut stream) {
                            panic!("{err}");
                        }
                    }

                    #[test]
                    #[should_panic(
                        expected = "test.ddi:9:64: error: Failed parsing `LookupMethod`:\nexpected one of: { \"by-name\", \"include-angle\", \"include-quote\" }\n"
                    )]
                    fn mismatch_field_by_name_include() {
                        let text = r#"{
                            "version": 1,
                            "revision": 0,
                            "rules": [
                                {
                                    "requires": [
                                        {
                                            "logical-name": "foo",
                                            "lookup-method": "wrong"
                                        }
                                    ]
                                }
                            ],
                        }"#;
                        let path = "test.ddi";
                        let input = text.as_bytes();
                        let state = State::default();
                        let mut stream = ParseStream::new(path, input, state);
                        if let Err(err) = r5::parsers::dep_file.parse(&mut stream) {
                            panic!("{err}");
                        }
                    }

                    #[test]
                    #[should_panic(
                        expected = "test.ddi:9:63: error: Failed parsing `LookupMethod`:\nexpected one of: { \"by-name\", \"include-angle\", \"include-quote\" }\n"
                    )]
                    fn mismatch_field_unquoted() {
                        let text = r#"{
                            "version": 1,
                            "revision": 0,
                            "rules": [
                                {
                                    "requires": [
                                        {
                                            "logical-name": "foo",
                                            "lookup-method": bad
                                        }
                                    ]
                                }
                            ],
                        }"#;
                        let path = "test.ddi";
                        let input = text.as_bytes();
                        let state = State::default();
                        let mut stream = ParseStream::new(path, input, state);
                        if let Err(err) = r5::parsers::dep_file.parse(&mut stream) {
                            panic!("{err}");
                        }
                    }
                }
            }
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
            let path = "test.ddi";
            let input = text.as_bytes();
            let state = State::default();
            let mut stream = ParseStream::new(path, input, state);
            let dep_file = r5::parsers::dep_file.parse(&mut stream).unwrap();
            assert_eq!(2, crate::util::count_escapes(text));
            assert_eq!(1, crate::util::count_escaped_strings(text).1);
            assert_eq!(0, dep_file.count_escapes_total());
            assert_eq!(1, dep_file.count_copies());
        }

        proptest! {
            #[cfg_attr(miri, ignore)]
            #[test]
            fn dep_file(text in r5::strategy::dep_file()) {
                let path = "test.ddi";
                let input = text.as_bytes();
                let state = State::default();
                let mut stream = ParseStream::new(path, input, state);
                r5::parsers::dep_file.parse(&mut stream).unwrap();
            }
        }
    }
}
