#![cfg(not(tarpaulin_include))]

use alloc::{string::String, vec::Vec};

use serde::ser::Serialize;

use super::BoxResult;
use crate::spec::r5;

pub fn pretty_print_unindented(dep_file: r5::DepFile<'_>) -> BoxResult<String> {
    let value = &dep_file;
    let mut bytes = Vec::with_capacity(1024);
    let formatter = serde_json::ser::PrettyFormatter::with_indent(b"");
    let mut serializer = serde_json::ser::Serializer::with_formatter(&mut bytes, formatter);
    value.serialize(&mut serializer)?;
    let string = unsafe { String::from_utf8_unchecked(bytes) };
    Ok(string)
}
