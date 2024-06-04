use alloc::{format, string::String, vec::Vec};

use proptest::prelude::*;

use crate::util::proptest::strategy::{json, util};

#[cfg_attr(test, allow(unused))]
pub fn dep_file() -> impl Strategy<Value = String> {
    let strat = (
        self::util::ws_around(self::dep_file::version("")),
        self::util::ws_around(self::dep_file::revision("", true)),
        self::util::ws_around(self::dep_file::rules("")),
    );
    Strategy::prop_map(strat, <[_; 3]>::from)
        .prop_shuffle()
        .prop_map(|fields| {
            format!(
                "{{{}}}",
                fields
                    .into_iter()
                    .filter(|field| !field.is_empty())
                    .collect::<Vec<String>>()
                    .join(",")
            )
        })
}

pub mod dep_file {
    #[allow(clippy::wildcard_imports)]
    use super::*;

    pub fn revision(
        term: impl Strategy<Value = impl core::fmt::Display> + 'static,
        allow_empty: bool,
    ) -> impl Strategy<Value = String> {
        let field = self::json::field("\"revision\"", any::<u32>(), term);
        let strat = if allow_empty {
            proptest::option::of(field).boxed()
        } else {
            field.prop_map_into::<Option<String>>().boxed()
        };
        strat.prop_map(Option::unwrap_or_default)
    }
    pub fn rules(term: impl Strategy<Value = impl core::fmt::Display> + 'static) -> impl Strategy<Value = String> {
        self::json::field("\"rules\"", "\\[[ \t\n\r]*\\]", term)
    }
    pub fn version(term: impl Strategy<Value = impl core::fmt::Display> + 'static) -> impl Strategy<Value = String> {
        self::json::field("\"version\"", any::<u32>(), term)
    }
}

pub mod required_module_desc {
    #[allow(clippy::wildcard_imports)]
    use super::*;

    #[cfg_attr(test, allow(unused))]
    pub fn lookup_method() -> impl Strategy<Value = String> {
        prop_oneof!["\"by-name\"", "\"include-angle\"", "\"include-quote\""]
    }
}
