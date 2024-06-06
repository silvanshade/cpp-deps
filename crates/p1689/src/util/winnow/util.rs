use alloc::{borrow::Cow, vec::Vec};

use camino::{Utf8Path, Utf8PathBuf};
use winnow::{
    ascii::{digit0, multispace0, Uint},
    combinator::{delimited, fail, peek, preceded, terminated, trace},
    error::ParserError,
    prelude::*,
    stream::{AsBStr, Stream},
    token::{any, one_of},
    BStr,
};

use crate::util::winnow::StateStream;

pub mod atoi {

    pub mod u32 {
        #[cfg(feature = "winnow")]
        pub fn ascii_to_hexdigit(character: u8) -> Option<u32> {
            match character {
                b'0' => Some(0),
                b'1' => Some(1),
                b'2' => Some(2),
                b'3' => Some(3),
                b'4' => Some(4),
                b'5' => Some(5),
                b'6' => Some(6),
                b'7' => Some(7),
                b'8' => Some(8),
                b'9' => Some(9),
                b'a' | b'A' => Some(10),
                b'b' | b'B' => Some(11),
                b'c' | b'C' => Some(12),
                b'd' | b'D' => Some(13),
                b'e' | b'E' => Some(14),
                b'f' | b'F' => Some(15),
                _ => None,
            }
        }
    }
}

pub fn cow_to_mut_with_reserve<'i>(cow: &'i mut Cow<BStr>, off: usize) -> &'i mut Vec<u8> {
    match cow {
        Cow::Borrowed(slice) => {
            let mut buf = Vec::with_capacity(slice.len() + off);
            buf.extend_from_slice(slice);
            *cow = Cow::Owned(buf);
        },
        Cow::Owned(ref mut buf) => {
            buf.reserve(off);
        },
    }
    cow.to_mut()
}

fn cow_extend_bytes<'i>(text: &mut Cow<'i, BStr>, data: &'i [u8], char: u8) {
    let lhs = cow_to_mut_with_reserve(text, data.len() + 1);
    lhs.extend_from_slice(data);
    lhs.push(char);
}

fn cow_bstr_to_utf8<'i, E>(input: &StateStream<'i>, cow: Cow<'i, BStr>) -> PResult<Cow<'i, str>, E>
where
    E: ParserError<StateStream<'i>>,
{
    match cow {
        Cow::Borrowed(ptr) => {
            let str = core::str::from_utf8(ptr).map_err(|_| {
                let message = "UTF-8 validation failed";
                winnow::error::ErrMode::assert(input, message)
            })?;
            Ok(Cow::Borrowed(str))
        },
        Cow::Owned(val) => {
            let str = alloc::string::String::from_utf8(val).map_err(|_| {
                let message = "UTF-8 validation failed";
                winnow::error::ErrMode::assert(input, message)
            })?;
            Ok(Cow::Owned(str))
        },
    }
}

pub fn cow_module_str<'i, E>(input: &mut StateStream<'i>) -> PResult<Cow<'i, str>, E>
where
    E: ParserError<StateStream<'i>>,
{
    trace("cow_module_str", |input0: &mut StateStream<'i>| {
        let str = unescaped_string.parse_next(input0)?;
        Ok(str)
    })
    .parse_next(input)
}

pub fn cow_utf8_path<'i, E>(input: &mut StateStream<'i>) -> PResult<Cow<'i, Utf8Path>, E>
where
    E: ParserError<StateStream<'i>>,
{
    trace("cow_utf8_path", |input0: &mut StateStream<'i>| {
        let str = unescaped_string.parse_next(input0)?;
        let path = match str {
            Cow::Borrowed(ptr) => Cow::Borrowed(Utf8Path::new(ptr)),
            Cow::Owned(val) => Cow::Owned(Utf8PathBuf::from(val)),
        };
        Ok(path)
    })
    .parse_next(input)
}

// NOTE: Specialized version of `dec_uint` that does not pessimize the `0` parse.
pub fn dec_uint<'i, Output, Error>(input: &mut StateStream<'i>) -> PResult<Output, Error>
where
    Output: Uint,
    Error: ParserError<StateStream<'i>>,
{
    trace("dec_uint_from_0", move |input0: &mut StateStream<'i>| {
        (one_of('0' ..= '9'), digit0)
            .void()
            .recognize()
            .verify_map(|s: <StateStream<'i> as Stream>::Slice| {
                let s = s.as_bstr();
                // SAFETY: Only 7-bit ASCII characters are parsed
                let s = unsafe { core::str::from_utf8_unchecked(s) };
                Output::try_from_dec_uint(s)
            })
            .parse_next(input0)
    })
    .parse_next(input)
}

pub fn ws_around<'i, O, E, F>(mut inner: F) -> impl Parser<StateStream<'i>, O, E>
where
    E: ParserError<StateStream<'i>>,
    F: Parser<StateStream<'i>, O, E>,
{
    trace("ws_around", move |input: &mut StateStream<'i>| {
        let val = delimited(multispace0, inner.by_ref(), multispace0).parse_next(input)?;
        Ok(val)
    })
}

pub fn ws_prefix<'i, O, E, F>(mut inner: F) -> impl Parser<StateStream<'i>, O, E>
where
    E: ParserError<StateStream<'i>>,
    F: Parser<StateStream<'i>, O, E>,
{
    trace("ws_prefix", move |input: &mut StateStream<'i>| {
        let val = preceded(multispace0, inner.by_ref()).parse_next(input)?;
        Ok(val)
    })
}

pub fn ws_suffix<'i, O, E, F>(mut inner: F) -> impl Parser<StateStream<'i>, O, E>
where
    E: ParserError<StateStream<'i>>,
    F: Parser<StateStream<'i>, O, E>,
{
    trace("ws_suffix", move |input: &mut StateStream<'i>| {
        let val = terminated(inner.by_ref(), multispace0).parse_next(input)?;
        Ok(val)
    })
}

pub fn unescaped_string<'i, E>(input: &mut StateStream<'i>) -> PResult<Cow<'i, str>, E>
where
    E: ParserError<StateStream<'i>>,
{
    trace("parse_string", move |input: &mut StateStream<'i>| {
        peek(b'"').parse_next(input)?;
        let mut off = 1;
        let mut text = Cow::Borrowed(BStr::new(b"".as_slice()));
        loop {
            let Some(needle) = input.state.finders.quotes_or_backslash.find(&input[off ..]) else {
                let message = "failed to find end of string";
                return Err(winnow::error::ErrMode::assert(input, message));
            };
            let data = input.next_slice(needle + off + 1);
            match data[data.len() - 1] {
                b'"' => {
                    if text.is_empty() {
                        text = Cow::Borrowed(data.into())
                    } else {
                        text.to_mut().extend_from_slice(data);
                    };
                    break;
                },
                b'\\' => self::string::unescape(&mut text, data).parse_next(input)?,
                _ => unreachable!(),
            }
            off = 0;
        }
        let utf8 = cow_bstr_to_utf8(input, text)?;
        Ok(utf8)
    })
    .parse_next(input)
}

mod string {

    use super::*;

    pub fn unescape_unicode<'i, 'r, E>(
        dst: &'r mut Cow<'i, BStr>,
        src: &'i [u8],
    ) -> impl Parser<StateStream<'i>, (), E> + 'r
    where
        E: ParserError<StateStream<'i>> + 'r,
    {
        trace("unescape_unicode", move |input: &mut StateStream<'i>| {
            b'{'.parse_next(input)?;
            let (number, needle) = input.state.hex_to_u32(input)?;
            input.next_slice(needle + 1);
            let escaped = core::char::from_u32(number).ok_or_else(|| {
                let message = "failed to convert unicode u32 to char";
                winnow::error::ErrMode::assert(input, message)
            })?;
            input.state.encode_utf8(dst, src, escaped);
            Ok(())
        })
    }

    // #[cold]
    pub fn unescape<'i, 'r, E>(dst: &'r mut Cow<'i, BStr>, src: &'i [u8]) -> impl Parser<StateStream<'i>, (), E> + 'r
    where
        E: ParserError<StateStream<'i>> + 'r,
    {
        trace("unescape", move |input: &mut StateStream<'i>| {
            let src = &src[.. src.len() - 1];
            match any.parse_next(input)? {
                b'"' => cow_extend_bytes(dst, src, b'"'),
                b'\\' => cow_extend_bytes(dst, src, b'\\'),
                b'/' => cow_extend_bytes(dst, src, b'/'),
                b'b' => cow_extend_bytes(dst, src, b'\\'),
                b'f' => cow_extend_bytes(dst, src, 0x0c),
                b'n' => cow_extend_bytes(dst, src, b'\n'),
                b'r' => cow_extend_bytes(dst, src, b'\r'),
                b't' => cow_extend_bytes(dst, src, b'\t'),
                b'u' => unescape_unicode(dst, src).parse_next(input)?,
                _ => fail.parse_next(input)?,
            }
            Ok(())
        })
    }
}
