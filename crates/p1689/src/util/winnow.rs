use alloc::{borrow::Cow, vec::Vec};

use winnow::{
    ascii::multispace0,
    combinator::{delimited, dispatch, empty, fail, peek, preceded, terminated, trace},
    error::ParserError,
    prelude::*,
    stream::Stream,
    token::{any, literal},
    BStr,
};

use crate::vendor::camino::{Utf8Path, Utf8PathBuf};

#[derive(Debug)]
pub(crate) struct Finders {
    #[cfg(target_feature = "avx2")]
    quotes_or_backslash: memchr::arch::x86_64::avx2::memchr::Two,
    #[cfg(all(not(target_feature = "avx2"), target_feature = "sse2"))]
    quotes_or_backslash: memchr::arch::x86_64::sse2::memchr::Two,
    #[cfg(all(not(target_feature = "avx2"), not(target_feature = "sse2")))]
    quotes_or_backslash: memchr::arch::all::memchr::Two,
}
impl Default for Finders {
    fn default() -> Self {
        #[cfg(target_feature = "avx2")]
        let quotes_or_backslash = memchr::arch::x86_64::avx2::memchr::Two::new(b'"', b'\\').unwrap();
        #[cfg(all(not(target_feature = "avx2"), target_feature = "sse2"))]
        let quotes_or_backslash = memchr::arch::x86_64::sse2::memchr::Two::new(b'"', b'\\').unwrap();
        #[cfg(all(not(target_feature = "avx2"), not(target_feature = "sse2")))]
        let quotes_or_backslash = memchr::arch::all::memchr::Two::new(b'"', b'\\');

        Self { quotes_or_backslash }
    }
}

#[derive(Debug)]
pub struct State {
    finders: Finders,
    utf8_encode_buffer: [u8; 4],
}
impl State {
    fn encode_utf8<'i>(&mut self, dst: &mut Cow<'i, BStr>, src: &'i [u8], esc: char) {
        let cow = self::string::to_mut_with_reserve(dst, src.len() + esc.len_utf8());
        let esc = esc.encode_utf8(self.utf8_encode_buffer.as_mut()).as_bytes();
        cow.extend_from_slice(src);
        cow.extend_from_slice(esc);
        self.utf8_encode_buffer = [0u8; 4];
    }
}
impl Default for State {
    fn default() -> Self {
        let finders = Finders::default();
        let utf8_encode_buffer = <[u8; 4]>::default();
        Self {
            finders,
            utf8_encode_buffer,
        }
    }
}

pub type StateStream<'i> = winnow::Stateful<&'i BStr, State>;

pub mod json {
    use super::*;

    pub fn bool(stream: &mut StateStream) -> PResult<bool> {
        dispatch! { any;
            b't' => b"rue".value(true),
            b'f' => b"alse".value(false),
            _ => fail,
        }
        .parse_next(stream)
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
        let mut field = move |stream: &mut StateStream<'i>| {
            literal(key.as_ref()).parse_next(stream)?;
            multispace0.parse_next(stream)?;
            b':'.parse_next(stream)?;
            multispace0.parse_next(stream)?;
            let val = val.parse_next(stream)?;
            multispace0.parse_next(stream)?;
            let mut dispatch = dispatch! { peek(any);
                b',' => b','.void(),
                b'}' => empty.value(()), // tarpaulin::hint
                _ => fail, // tarpaulin::hint
            };
            dispatch.parse_next(stream)?;
            Ok(val)
        };
        trace("field", move |stream: &mut StateStream<'i>| {
            self::spaces::suffix(field.by_ref()).parse_next(stream)
        })
    }

    pub fn record<'i, E, V, P>(mut val: P) -> impl Parser<StateStream<'i>, V, E>
    where
        E: ParserError<StateStream<'i>>,
        P: Parser<StateStream<'i>, V, E>,
    {
        trace("record", move |stream: &mut StateStream<'i>| {
            let val = delimited(b'{', self::spaces::prefix(val.by_ref()), b'}').parse_next(stream)?;
            Ok(val)
        })
    }

    #[rustfmt::skip]
pub fn vec<'i, E, V, P>(mut val: P) -> impl Parser<StateStream<'i>, Vec<V>, E>
where
    E: ParserError<StateStream<'i>>,
    P: Parser<StateStream<'i>, V, E>,
{
    trace("set", move |stream: &mut StateStream<'i>| {
        b'['.parse_next(stream)?;
        let mut vec = Vec::default();
        multispace0.parse_next(stream)?;
        if b']' != peek(any).parse_next(stream)? {
            loop { // tarpaulin::hint
                vec.push(val.parse_next(stream)?);
                multispace0.parse_next(stream)?;
                match any.parse_next(stream)? {
                    b',' => multispace0.void().parse_next(stream)?,
                    b']' => break, // tarpaulin::hint
                    _ => fail.parse_next(stream)?,
                }
            }
        } else {
            b']'.parse_next(stream)?;
        }
        Ok(vec)
    })
}
}

pub mod number {
    use super::*;

    pub fn from_radix_10(text: &[u8]) -> (u32, usize) {
        let mut idx = 0;
        let mut num = 0;
        while idx != text.len() {
            if let Some(dig) = self::number::ascii_to_decimal_digit(text[idx]) {
                num *= 10;
                num += dig;
                idx += 1;
            } else {
                break;
            }
        }
        (num, idx)
    }

    pub fn from_radix_16(text: &[u8]) -> (u32, usize) {
        let mut idx = 0;
        let mut num = 0u32;
        while idx != text.len() {
            if let Some(dig) = self::number::ascii_to_hex_digit(text[idx]) {
                num *= 16;
                num += dig;
                idx += 1;
            } else {
                break;
            }
        }
        (num, idx)
    }

    #[cfg(feature = "winnow")]
    pub fn ascii_to_decimal_digit(character: u8) -> Option<u32> {
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
            _ => None,
        }
    }

    #[cfg(feature = "winnow")]
    pub fn ascii_to_hex_digit(character: u8) -> Option<u32> {
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

    // NOTE: Specialized version of `dec_uint` that does not pessimize the `0` parse.
    pub fn dec_uint<'i, Error>(stream: &mut StateStream<'i>) -> PResult<u32, Error>
    where
        Error: ParserError<StateStream<'i>>,
    {
        trace("dec_uint_from_0", |stream0: &mut StateStream<'i>| {
            let (number, needle) = from_radix_10(stream0);
            if needle == 0 {
                let message = "Failed to parse an unsigned integer";
                return Err(winnow::error::ErrMode::assert(stream0, message));
            }
            stream0.next_slice(needle); // tarpaulin::hint
            Ok(number)
        })
        .parse_next(stream) // tarpaulin::hint
    }
}
pub mod string {
    use super::*;

    pub(super) fn to_mut_with_reserve<'i>(cow: &'i mut Cow<BStr>, off: usize) -> &'i mut Vec<u8> {
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

    fn extend_bytes<'i>(text: &mut Cow<'i, BStr>, data: &'i [u8], char: u8) {
        let lhs = to_mut_with_reserve(text, data.len() + 1);
        lhs.extend_from_slice(data);
        lhs.push(char);
    }

    pub(crate) fn bstr_to_utf8<'i, E>(stream: &StateStream<'i>, cow: Cow<'i, BStr>) -> PResult<Cow<'i, str>, E>
    where
        E: ParserError<StateStream<'i>>,
    {
        match cow {
            Cow::Borrowed(ptr) => {
                let str = core::str::from_utf8(ptr).map_err(|_| {
                    let message = "UTF-8 validation failed";
                    winnow::error::ErrMode::assert(stream, message)
                })?;
                Ok(Cow::Borrowed(str))
            },
            Cow::Owned(val) => {
                let str = alloc::string::String::from_utf8(val).map_err(|_| {
                    let message = "UTF-8 validation failed";
                    winnow::error::ErrMode::assert(stream, message)
                })?;
                Ok(Cow::Owned(str))
            },
        }
    }

    pub fn module<'i, E>(stream: &mut StateStream<'i>) -> PResult<Cow<'i, str>, E>
    where
        E: ParserError<StateStream<'i>>,
    {
        trace("cow_module_str", |stream0: &mut StateStream<'i>| {
            let str = json_string.parse_next(stream0)?;
            Ok(str)
        })
        .parse_next(stream) // tarpaulin::hint
    }

    #[rustfmt::skip]
    pub(crate) fn json_string<'i, E>(stream: &mut StateStream<'i>) -> PResult<Cow<'i, str>, E>
    where
        E: ParserError<StateStream<'i>>,
    {
        trace("parse_string", |stream0: &mut StateStream<'i>| {
            peek(b'"').parse_next(stream0)?;
            let mut off = 1;
            let mut text = Cow::Borrowed(BStr::new(b"".as_slice()));
            loop {
                let Some(needle) = stream0.state.finders.quotes_or_backslash.find(&stream0[off ..]) else {
                    let message = "Failed to find end of string";
                    return Err(winnow::error::ErrMode::assert(stream0, message));
                };
                let data = stream0.next_slice(needle + off + 1);
                match data[data.len() - 1] {
                    b'"' => { // tarpaulin::hint
                        if text.is_empty() {
                            text = Cow::Borrowed(data.into())
                        } else {
                            text.to_mut().extend_from_slice(data);
                        };
                        break; // tarpaulin::hint
                    },
                    b'\\' => unescape(&mut text, data).parse_next(stream0)?,
                    _ => unreachable!(),
                }
                off = 0;
            }
            let utf8 = bstr_to_utf8(stream0, text)?;
            Ok(utf8)
        })
        .parse_next(stream) // tarpaulin::hint
    }

    pub fn utf8_path<'i, E>(stream: &mut StateStream<'i>) -> PResult<Cow<'i, Utf8Path>, E>
    where
        E: ParserError<StateStream<'i>>,
    {
        trace("cow_utf8_path", |stream0: &mut StateStream<'i>| {
            let str = json_string.parse_next(stream0)?;
            let path = match str {
                #[allow(clippy::useless_conversion)]
                Cow::Borrowed(ptr) => Cow::Borrowed(ptr.into()),
                Cow::Owned(val) => Cow::Owned(Utf8PathBuf::from(val)),
            };
            Ok(path)
        })
        .parse_next(stream) // tarpaulin::hint
    }

    fn unescape<'i, 'r, E>(dst: &'r mut Cow<'i, BStr>, src: &'i [u8]) -> impl Parser<StateStream<'i>, (), E> + 'r
    where
        E: ParserError<StateStream<'i>> + 'r,
    {
        trace("unescape", |stream: &mut StateStream<'i>| {
            let src = &src[.. src.len() - 1];
            match any.parse_next(stream)? {
                b'"' => extend_bytes(dst, src, b'"'),
                b'\\' => extend_bytes(dst, src, b'\\'),
                b'/' => extend_bytes(dst, src, b'/'),
                b'b' => extend_bytes(dst, src, 0x08),
                b'f' => extend_bytes(dst, src, 0x0c),
                b'n' => extend_bytes(dst, src, b'\n'),
                b'r' => extend_bytes(dst, src, b'\r'),
                b't' => extend_bytes(dst, src, b'\t'),
                b'u' => unescape_unicode(dst, src).parse_next(stream)?,
                _ => fail.parse_next(stream)?,
            }
            Ok(())
        })
    }

    pub(crate) fn unescape_unicode<'i, 'r, E>(
        dst: &'r mut Cow<'i, BStr>,
        src: &'i [u8],
    ) -> impl Parser<StateStream<'i>, (), E> + 'r
    where
        E: ParserError<StateStream<'i>> + 'r,
    {
        trace("unescape_unicode", |stream: &mut StateStream<'i>| {
            b'{'.parse_next(stream)?;
            let (number, needle) = self::number::from_radix_16(stream);
            stream.next_slice(needle); // tarpaulin::hint
            b'}'.parse_next(stream)?;
            let escaped = core::char::from_u32(number).ok_or_else(|| {
                let message = "Failed to convert unicode u32 to char";
                winnow::error::ErrMode::assert(stream, message)
            })?;
            stream.state.encode_utf8(dst, src, escaped);
            Ok(())
        })
    }
}

pub mod spaces {
    use super::*;

    pub fn around<'i, O, E, F>(mut inner: F) -> impl Parser<StateStream<'i>, O, E>
    where
        E: ParserError<StateStream<'i>>,
        F: Parser<StateStream<'i>, O, E>,
    {
        trace("ws_around", move |stream: &mut StateStream<'i>| {
            let val = delimited(multispace0, inner.by_ref(), multispace0).parse_next(stream)?;
            Ok(val)
        })
    }

    pub fn prefix<'i, O, E, F>(mut inner: F) -> impl Parser<StateStream<'i>, O, E>
    where
        E: ParserError<StateStream<'i>>,
        F: Parser<StateStream<'i>, O, E>,
    {
        trace("ws_prefix", move |stream: &mut StateStream<'i>| {
            let val = preceded(multispace0, inner.by_ref()).parse_next(stream)?;
            Ok(val)
        })
    }

    pub fn suffix<'i, O, E, F>(mut inner: F) -> impl Parser<StateStream<'i>, O, E>
    where
        E: ParserError<StateStream<'i>>,
        F: Parser<StateStream<'i>, O, E>,
    {
        trace("ws_suffix", move |stream: &mut StateStream<'i>| {
            let val = terminated(inner.by_ref(), multispace0).parse_next(stream)?;
            Ok(val)
        })
    }
}

#[cfg(test)]
mod test {
    use alloc::{string::ToString, vec};

    use proptest::prelude::*;

    use super::*;
    use crate::r5::parsers::State;

    #[test]
    fn string_json_string_correctly_unescapes_while_parsing() {
        for (esc, raw) in [
            ('"', b'"'),
            ('\\', b'\\'),
            ('/', b'/'),
            ('b', 0x08),
            ('f', 0x0c),
            ('n', b'\n'),
            ('r', b'\r'),
            ('t', b'\t'),
        ] {
            let text = std::format!("\"foo\\{esc}bar\"");
            let input = winnow::BStr::new(&text);
            let state = State::default();
            let mut stream = winnow::Stateful { input, state };
            let unescaped = self::string::json_string::<()>.parse_next(&mut stream).unwrap();
            let lhs = unescaped;
            let rhs = std::format!("\"foo{}bar\"", char::from(raw));
            assert_eq!(lhs, rhs);
        }
    }

    #[test]
    #[should_panic(expected = "Failed to find end of string")]
    fn string_json_string_expectedly_fails_invalid_utf8() {
        let text = "\"foo";
        let input = BStr::new(text);
        let state = State::default();
        let mut stream = winnow::Stateful { input, state };
        self::string::json_string::<()>.parse_next(&mut stream).unwrap();
    }

    #[test]
    #[should_panic(expected = "UTF-8 validation failed")]
    fn string_bstr_to_utf8_expectedly_fails_invalid_utf8_borrowed() {
        let text = "";
        let input = BStr::new(text);
        let state = State::default();
        let stream = winnow::Stateful { input, state };
        // NOTE: Here we use \u{D800} (leading surrogate) which is invalid standalone
        let bor = BStr::new(&[0xed, 0xa0, 0x80]);
        let cow = Cow::Borrowed(bor);
        self::string::bstr_to_utf8::<()>(&stream, cow).unwrap();
    }

    #[test]
    #[should_panic(expected = "UTF-8 validation failed")]
    fn string_bstr_to_utf8_expectedly_fails_invalid_utf8_owned() {
        let text = "";
        let input = BStr::new(text);
        let state = State::default();
        let stream = winnow::Stateful { input, state };
        // NOTE: Here we use \u{D800} (leading surrogate) which is invalid standalone
        let own = vec![0xed, 0xa0, 0x80];
        let cow = Cow::Owned(own);
        self::string::bstr_to_utf8::<()>(&stream, cow).unwrap();
    }

    #[test]
    #[should_panic(expected = "Failed to convert unicode u32 to char")]
    fn string_unescape_unicode_expectedly_fails_invalid_utf8() {
        let mut dst = Cow::Owned(vec![]);
        let src = &[];
        let text = "{D800}";
        let input = BStr::new(text);
        let state = State::default();
        let mut stream = winnow::Stateful { input, state };
        self::string::unescape_unicode::<()>(&mut dst, src)
            .parse_next(&mut stream)
            .unwrap();
    }

    #[test]
    fn number_from_radix_16_correct_static() {
        let char = '💯';
        let text = char.escape_unicode().to_string();
        let text = text.strip_prefix("\\u{").unwrap();
        let input = winnow::BStr::new(text);
        let state = crate::r5::parsers::State::default();
        let stream = &mut winnow::Stateful { input, state };
        let (number, index) = self::number::from_radix_16(stream);
        assert_eq!(number, u32::from(char));
        assert_eq!(index, text.strip_suffix("}").unwrap().len());
    }

    proptest! {
        #[cfg_attr(miri, ignore)]
        #[test]
        fn number_from_radix_16_correct(char in proptest::prelude::any::<char>()) {
            let text = char.escape_unicode().to_string();
            let text = text.strip_prefix("\\u{").unwrap();
            let state = crate::r5::parsers::State::default();
            let input = winnow::BStr::new(&text);
            let stream = &mut winnow::Stateful { input, state };
            let (number, index) = self::number::from_radix_16(stream);
            assert_eq!(number, u32::from(char));
            assert_eq!(index, text.strip_suffix("}").unwrap().len());
        }
    }
}
