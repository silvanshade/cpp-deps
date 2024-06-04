use alloc::{
    format,
    string::{String, ToString},
};

use proptest::strategy::Strategy;

pub fn ws() -> impl Strategy<Value = String> {
    "[ \n\t\r]"
}

pub fn ws_around(strat: impl Strategy<Value = impl core::fmt::Display>) -> impl Strategy<Value = String> {
    let strat = (ws(), strat, ws());
    Strategy::prop_map(strat, move |(ws0, s, ws1)| {
        let sd = s.to_string();
        // NOTE: If `strat` produces an empty string, don't add any padding.
        if sd.is_empty() {
            String::new()
        } else {
            let res = format!("{ws0}{sd}{ws1}");
            res
        }
    })
}

// pub fn terminated(
//     strat: impl Strategy<Value = impl core::fmt::Display>,
//     term: impl Strategy<Value = impl core::fmt::Display>,
// ) -> impl Strategy<Value = String> {
//     let strat = (strat, term);
//     Strategy::prop_map(strat, move |(s, t)| {
//         let res = format!("{s}{t}");
//         res
//     })
// }
