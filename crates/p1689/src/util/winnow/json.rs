use indexmap::IndexSet;
use winnow::{
    ascii::multispace0,
    combinator::{delimited, dispatch, empty, fail, peek, trace},
    error::ParserError,
    prelude::*,
    token::{any, literal},
    BStr,
};

use crate::util::winnow::{
    util::{ws_prefix, ws_suffix},
    StateStream,
};

pub fn bool(input: &mut StateStream) -> PResult<bool> {
    dispatch! { any;
        b't' => b"rue".value(true),
        b'f' => b"alse".value(false),
        _ => fail,
    }
    .parse_next(input)
}

pub fn field<'i, 'k, E, K, V, P>(key: &'k K, mut val: P) -> impl Parser<StateStream<'i>, V, E> + 'k
where
    'i: 'k,
    E: ParserError<StateStream<'i>> + 'k,
    K: ?Sized,
    &'k BStr: From<&'k K>,
    V: 'k,
    P: Parser<StateStream<'i>, V, E> + 'k,
{
    let key = Into::<&BStr>::into(key);
    let mut field = move |input: &mut StateStream<'i>| {
        literal(key.as_ref()).parse_next(input)?;
        multispace0.parse_next(input)?;
        b':'.parse_next(input)?;
        multispace0.parse_next(input)?;
        let val = val.parse_next(input)?;
        multispace0.parse_next(input)?;
        dispatch! { peek(any);
            b',' => b','.void(),
            b'}' => empty.value(()),
            _ => fail,
        }
        .parse_next(input)?;
        Ok(val)
    };
    trace("field", move |input: &mut StateStream<'i>| {
        ws_suffix(field.by_ref()).parse_next(input)
    })
}

pub fn record<'i, E, V, P>(mut val: P) -> impl Parser<StateStream<'i>, V, E>
where
    E: ParserError<StateStream<'i>>,
    P: Parser<StateStream<'i>, V, E>,
{
    trace("record", move |input: &mut StateStream<'i>| {
        let val = delimited(b'{', ws_prefix(val.by_ref()), b'}').parse_next(input)?;
        Ok(val)
    })
}

pub fn set<'i, E, V, P>(mut val: P) -> impl Parser<StateStream<'i>, IndexSet<V>, E>
where
    E: ParserError<StateStream<'i>>,
    P: Parser<StateStream<'i>, V, E>,
    V: Eq + core::hash::Hash,
{
    trace("set", move |input: &mut StateStream<'i>| {
        b'['.parse_next(input)?;
        let mut set = IndexSet::default();
        multispace0.parse_next(input)?;
        if b']' != peek(any).parse_next(input)? {
            loop {
                set.insert(val.parse_next(input)?);
                multispace0.parse_next(input)?;
                match any.parse_next(input)? {
                    b',' => multispace0.void().parse_next(input)?,
                    b']' => break,
                    _ => fail.parse_next(input)?,
                }
            }
        } else {
            b']'.parse_next(input)?;
        }
        Ok(set)
    })
}
