use alloc::{format, string::String};

use proptest::strategy::Strategy;

use crate::util::proptest::strategy::util;

pub fn field<'k>(
    key: &'k str,
    val: impl Strategy<Value = impl core::fmt::Display> + 'k,
    term: impl Strategy<Value = impl core::fmt::Display> + 'k,
) -> impl Strategy<Value = String> + 'k {
    let strat = (
        self::util::ws(),
        self::util::ws(),
        val,
        self::util::ws(),
        term,
        self::util::ws(),
    );
    Strategy::prop_map(strat, move |(ws0, ws1, v, ws2, t, ws3)| {
        format!("{key}{ws0}:{ws1}{v}{ws2}{t}{ws3}")
    })
}

#[cfg(test)]
mod test {
    use alloc::vec::Vec;

    use proptest::prelude::*;

    use super::*;

    proptest! {
        #[cfg_attr(miri, ignore)]
        #[test]
        fn field_works(text in field("key", Just("val"), Just("term"))) {
            let res = text.split_whitespace().collect::<Vec<&str>>();
            assert_eq!(res, ["key", ":", "val", "term"])
        }
    }
}
