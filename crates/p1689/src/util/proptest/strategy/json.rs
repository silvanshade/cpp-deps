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
        let res = format!("{key}{ws0}:{ws1}{v}{ws2}{t}{ws3}");
        res
    })
}
