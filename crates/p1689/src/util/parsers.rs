use alloc::{borrow::Cow, string::String};

pub struct ParseStream<'i, E> {
    pub(crate) path: &'i str,
    pub(crate) input: &'i [u8],
    pub(crate) bytes: &'i [u8],
    pub(crate) state: State,
    error: core::marker::PhantomData<E>,
}
impl<'i, E> ParseStream<'i, E> {
    pub fn new(path: &'i str, input: &'i [u8], state: State) -> Self {
        Self {
            path,
            input,
            bytes: input,
            state,
            error: Default::default(),
        }
    }

    #[rustfmt::skip]
    #[inline(always)]
    pub fn next_byte(&mut self) -> Result<u8, Error<'i, E>> { // tarpaulin::hint
        let head = self.peek_byte()?;
        self.bytes = &self.bytes[1 ..];
        Ok(head)
    }

    #[rustfmt::skip]
    #[inline(always)]
    pub fn peek_byte(&mut self) -> Result<u8, Error<'i, E>> { // tarpaulin::hint
        if self.bytes.is_empty() {
            let error = ErrorKind::NextByte;
            return Err(self.error(error));
        }
        // SAFETY: We just checked that the slice was non-empty.
        Ok(*unsafe { self.bytes.get_unchecked(0) })
    }

    #[rustfmt::skip]
    #[inline(always)]
    pub fn match_byte(&mut self, expected: u8) -> Result<(), Error<'i, E>> { // tarpaulin::hint
        if self.next_byte()? != expected {
            let error = ErrorKind::ByteMismatch { expected };
            return Err(self.error(error));
        }
        Ok(())
    }

    #[rustfmt::skip]
    #[inline(always)]
    pub fn next_slice(&mut self, offset: usize) -> Result<&'i [u8], Error<'i, E>> { // tarpaulin::hint
        let (slice, next) = self.bytes.split_at_checked(offset).ok_or_else(|| {
            let error = ErrorKind::NextSlice { offset };
            self.error(error)
        })?;
        self.bytes = next;
        Ok(slice)
    }

    #[rustfmt::skip]
    #[inline(always)]
    pub fn match_slice(&mut self, expected: &'i [u8]) -> Result<(), Error<'i, E>> { // tarpaulin::hint
        if self.next_slice(expected.len())? != expected {
            let error = ErrorKind::SliceMismatch { expected };
            return Err(self.error(error));
        }
        Ok(())
    }

    pub fn error(&self, error: ErrorKind<'i, E>) -> Error<'i, E> {
        let path = self.path;
        let input = self.input;
        let bytes = self.bytes;
        Error {
            path,
            input,
            bytes,
            error,
        }
    }
}

#[cfg(feature = "memchr")]
#[derive(Debug)]
pub(crate) struct Finders {
    #[cfg(target_feature = "avx2")]
    quotes_or_backslash: memchr::arch::x86_64::avx2::memchr::Two,
    #[cfg(all(not(target_feature = "avx2"), target_feature = "sse2"))]
    quotes_or_backslash: memchr::arch::x86_64::sse2::memchr::Two,
    #[cfg(all(not(target_feature = "avx2"), not(target_feature = "sse2")))]
    quotes_or_backslash: memchr::arch::all::memchr::Two,
}
#[cfg(feature = "memchr")]
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

#[derive(Debug, Default)]
pub struct State {
    #[cfg(feature = "memchr")]
    finders: Finders,
    utf8_encode_buffer: [u8; 4],
}
impl State {
    fn encode_utf8<'i>(&mut self, dst: &mut Cow<'i, [u8]>, src: &'i [u8], esc: char) {
        let cow = self::string::to_mut_with_reserve(dst, src.len() + esc.len_utf8());
        let esc = esc.encode_utf8(self.utf8_encode_buffer.as_mut()).as_bytes();
        cow.extend_from_slice(src);
        cow.extend_from_slice(esc);
        self.utf8_encode_buffer = [0u8; 4];
    }
}

pub trait Parser<'i, O, E> {
    fn parse(&mut self, input: &mut ParseStream<'i, E>) -> Result<O, Error<'i, E>>;
}

impl<'i, E, O, F> Parser<'i, O, E> for F
where
    F: FnMut(&mut ParseStream<'i, E>) -> Result<O, Error<'i, E>>,
{
    #[rustfmt::skip]
    #[inline(always)]
    fn parse(&mut self, i: &mut ParseStream<'i, E>) -> Result<O, Error<'i, E>> { // tarpaulin::hint
        self(i)
    }
}

#[derive(Clone, Debug)]
pub enum ErrorKind<'i, E> {
    CharFromUnicodeFailed { unicode: u32 },
    DuplicateField { field: &'static str },
    EndOfStringNotFound,
    FailedParsingBool,
    FailedParsingJsonArray,
    FailedParsingJsonObjectProperty,
    FailedParsingJsonStringEscape,
    FailedParsingJsonUnsignedInteger,
    InvalidUnicodeEscapeHex { bytes: [u8; 4] },
    InvalidUnicodeLowerSurrogate { lo: u32 },
    MissingField { field: &'static str },
    MissingUnicodeLowerSurrogate { hi: u32 },
    NextByte,
    ByteMismatch { expected: u8 },
    NextSlice { offset: usize },
    SliceMismatch { expected: &'i [u8] },
    Utf8ValidationFailedPtr { err: core::str::Utf8Error },
    Utf8ValidationFailedVal { err: alloc::string::FromUtf8Error },
    Other { error: E },
}
#[derive(Debug)]
pub struct Error<'i, E> {
    pub path: &'i str,
    pub input: &'i [u8],
    pub bytes: &'i [u8],
    pub error: ErrorKind<'i, E>,
}

impl<'i, E> Error<'i, E> {
    pub fn context(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let input = String::from_utf8_lossy(self.input);
        let bytes = String::from_utf8_lossy(self.bytes);
        let mut row = 0u64;
        let mut col = 0u64;
        let text = &input[.. input.len() - bytes.len()];
        if text.is_empty() {
            row += 1;
            col += 1;
        }
        for line in text.lines() {
            col = u64::try_from(line.len()).unwrap();
            row += 1;
        }
        write!(f, "{}:{}:{}: error: ", self.path, row, col)?;
        Ok(())
    }
}

#[cfg(feature = "std")]
impl<E> std::error::Error for Error<'_, E> where E: core::fmt::Debug + core::fmt::Display {}

impl<E> core::fmt::Display for Error<'_, E>
where
    E: core::fmt::Display,
{
    #[rustfmt::skip]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        self.context(f)?;
        match &self.error {
            ErrorKind::CharFromUnicodeFailed { unicode } => {
                writeln!(f, "Conversion of unicode u32 to char failed: u32 value: {unicode:#06x}")?;
            },
            ErrorKind::DuplicateField { field } => {
                writeln!(f, "Duplicate field: `{field}`")?;
            },
            ErrorKind::EndOfStringNotFound => { // tarpaulin::hint
                writeln!(f, "End of string not found")?;
            },
            ErrorKind::FailedParsingBool => { // tarpaulin::hint
                writeln!(f, "Failed parsing bool")?;
            },
            ErrorKind::FailedParsingJsonArray => { // tarpaulin::hint
                writeln!(f, "Failed parsing JSON array:")?;
                writeln!(f, "expected one of: {{ ',', ']' }}")?;
            },
            ErrorKind::FailedParsingJsonObjectProperty => { // tarpaulin::hint
                writeln!(f, "Failed parsing JSON object property:")?;
                writeln!(f, "expected one of: {{ ',', '}}' }}")?;
            },
            ErrorKind::FailedParsingJsonStringEscape => { // tarpaulin::hint
                writeln!(f, "Failed parsing JSON string escape:")?;
                writeln!(
                    f, // tarpaulin::hint
                    "expected one of: {{ '\"', '\\', '/', 'b', 'f', 'n', 'r', 't', 'u' }}"
                )?;
            },
            ErrorKind::FailedParsingJsonUnsignedInteger => { // tarpaulin::hint
                writeln!(f, "Failed parsing JSON unsigned integer")?;
            },
            ErrorKind::InvalidUnicodeEscapeHex { bytes } => { // tarpaulin::hint
                writeln!(f, "Invalid unicode escape hex: {}", String::from_utf8_lossy(bytes.as_slice()))?;
            },
            ErrorKind::InvalidUnicodeLowerSurrogate { lo } => {
                writeln!(f, "Invalid unicode lower surrogate: {lo:#04x}")?;
            }
            &ErrorKind::MissingField { field } => {
                writeln!(f, "Missing field: `{field}`")?;
            },
            ErrorKind::MissingUnicodeLowerSurrogate { hi } => { // tarpaulin::hint
                writeln!(f, "Missing unicode lower surrogate pair for leading high surrogate: {hi:#04x}")?;
            },
            ErrorKind::NextByte => { // tarpaulin::hint
                writeln!(f, "No remaining bytes")?;
            },
            ErrorKind::ByteMismatch { expected } => {
                writeln!(
                    f, // tarpaulin::hint
                    "Byte mismatch: expected `{}`, actual `{}`",
                    expected.escape_ascii(),
                    self.input[(self.input.len() - self.bytes.len()).saturating_sub(1)].escape_ascii()
                )?;
            },
            ErrorKind::NextSlice { offset } => {
                writeln!(
                    f, // tarpaulin::hint
                    "Remaining bytes less than requested slice length: remaining: {}, requested: {offset}",
                    self.bytes.len()
                )?;
            },
            &ErrorKind::SliceMismatch { expected } => {
                let snd = self.input.len() - self.bytes.len();
                let fst = snd - expected.len();
                writeln!(
                    f, // tarpaulin::hint
                    "Slice mismatch: expected `{}`, actual `{}`",
                    expected.escape_ascii(),
                    self.input[fst .. snd].escape_ascii()
                )?;
            },
            ErrorKind::Utf8ValidationFailedPtr { err } => {
                writeln!(f, "UTF-8 validation failed:")?;
                writeln!(f, "{err}")?;
            },
            ErrorKind::Utf8ValidationFailedVal { err } => {
                writeln!(f, "UTF-8 validation failed:")?;
                writeln!(f, "{err}")?;
            },
            ErrorKind::Other { error } => {
                core::fmt::Display::fmt(error, f)?;
            },
        }
        Ok(())
    }
}

pub mod ascii {
    use super::Error;

    #[rustfmt::skip]
    #[inline(always)]
    // TODO: http://0x80.pl/notesen/2019-01-05-avx512vbmi-remove-spaces.html
    pub fn multispace0<'i, E>(stream: &mut crate::util::parsers::ParseStream<'i, E>) -> Result<&'i [u8], Error<'i, E>> { // tarpaulin::hint
        let mut slice: &[u8] = &[];
        for (index, byte) in stream.bytes.iter().enumerate() {
            if !byte.is_ascii_whitespace() {
                slice = &stream.bytes[.. index];
                stream.bytes = &stream.bytes[index ..];
                break; // tarpaulin::hint
            }
        }
        Ok(slice)
    }
}

pub mod json {
    use alloc::vec::Vec;

    use super::{ascii::multispace0, Error, ErrorKind, ParseStream, Parser};

    #[rustfmt::skip]
    pub fn bool<'i, E>(stream: &mut ParseStream<'i, E>) -> Result<bool, Error<'i, E>> {
        match stream.next_byte()? {
            b't' => { // tarpaulin::hint
                stream.match_slice(b"rue")?;
                Ok(true)
            },
            b'f' => { // tarpaulin::hint
                stream.match_slice(b"alse")?;
                Ok(false)
            },
            _ => return Err(stream.error(ErrorKind::FailedParsingBool)),
        }
    }

    #[rustfmt::skip]
    pub fn field<'i, E, V, P>(key: &'i [u8], mut val: P) -> impl Parser<'i, V, E>
    where
        P: Parser<'i, V, E>,
    {
        let key = Into::<&[u8]>::into(key);
        move |stream: &mut ParseStream<'i, E>| {
            stream.match_slice(key)?;
            multispace0.parse(stream)?;
            stream.match_byte(b':')?;
            multispace0.parse(stream)?;
            let val = val.parse(stream)?;
            multispace0.parse(stream)?;
            match stream.peek_byte()? {
                b',' => { // tarpaulin::hint
                    stream.match_byte(b',')?;
                },
                b'}' => {}, // tarpaulin::hint
                _ => return Err(stream.error(ErrorKind::FailedParsingJsonObjectProperty)),
            }
            multispace0.parse(stream)?;
            Ok(val)
        }
    }

    pub fn record<'i, E, V, P>(mut val: P) -> impl Parser<'i, V, E>
    where
        P: Parser<'i, V, E>,
    {
        move |stream: &mut ParseStream<'i, E>| {
            stream.match_byte(b'{')?;
            multispace0.parse(stream)?;
            let val = val.parse(stream)?;
            stream.match_byte(b'}')?;
            Ok(val)
        }
    }

    #[rustfmt::skip]
    pub fn vec<'i, E, V, P>(mut val: P) -> impl Parser<'i, Vec<V>, E>
    where
        P: Parser<'i, V, E>,
    {
        move |stream: &mut ParseStream<'i, E>| {
            stream.match_byte(b'[')?;
            let mut vec = Vec::default();
            multispace0.parse(stream)?;
            if b']' != stream.peek_byte()? {
                loop {
                    // tarpaulin::hint
                    vec.push(val.parse(stream)?);
                    multispace0.parse(stream)?;
                    match stream.next_byte()? {
                        b',' => { // tarpaulin::hint
                            multispace0.parse(stream)?;
                        },
                        b']' => break, // tarpaulin::hint
                        _ => return Err(stream.error(ErrorKind::FailedParsingJsonArray)),
                    }
                }
            } else {
                stream.match_byte(b']')?;
            }
            Ok(vec)
        }
    }
}

pub mod number {
    use super::*;

    const HEX_LUT: [u32; 0x10000] = {
        #[allow(clippy::identity_op)]
        const fn dec_to_hex(num: u8) -> [u8; 2] {
            let lut = b"0123456789abcdef";
            let mut hex = [0u8; 2];
            hex[0] = lut[(num as usize & 0xf0) >> 4];
            hex[1] = lut[(num as usize & 0x0f) >> 0];
            hex
        }
        let mut lut = [u32::MAX; 0x10000];
        let mut dec = 0u8;
        loop {
            let mut hex = dec_to_hex(dec);
            lut[u16::from_ne_bytes(hex) as usize] = dec as u32;
            hex[0] -= 32;
            lut[u16::from_ne_bytes(hex) as usize] = dec as u32;
            hex[1] -= 32;
            lut[u16::from_ne_bytes(hex) as usize] = dec as u32;
            hex[0] += 32;
            lut[u16::from_ne_bytes(hex) as usize] = dec as u32;
            if dec == u8::MAX {
                break;
            }
            dec += 1;
        }
        lut
    };

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

    pub fn from_radix_16(bytes: [u8; 4]) -> Option<u32> {
        let fst = HEX_LUT[u16::from_ne_bytes([bytes[0], bytes[1]]) as usize];
        let snd = HEX_LUT[u16::from_ne_bytes([bytes[2], bytes[3]]) as usize];
        let val = fst << 8 | snd;
        (val != u32::MAX).then_some(val)
    }

    #[cfg(feature = "parsing")]
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

    // NOTE: Specialized version of `dec_uint` that does not pessimize the `0` parse.
    pub fn dec_uint<'i, E>(stream: &mut ParseStream<'i, E>) -> Result<u32, Error<'i, E>> {
        let (number, needle) = from_radix_10(stream.bytes);
        if needle == 0 {
            let error = ErrorKind::FailedParsingJsonUnsignedInteger;
            return Err(stream.error(error));
        }
        stream.next_slice(needle)?; // tarpaulin::hint
        Ok(number)
    }
}
pub mod string {

    use alloc::{borrow::Cow, vec::Vec};

    use super::{number, Error, ErrorKind, ParseStream, Parser};
    use crate::vendor::camino::{Utf8Path, Utf8PathBuf};

    pub(super) fn to_mut_with_reserve<'i>(cow: &'i mut Cow<[u8]>, off: usize) -> &'i mut Vec<u8> {
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

    fn extend_bytes<'i>(text: &mut Cow<'i, [u8]>, data: &'i [u8], char: u8) {
        let lhs = to_mut_with_reserve(text, data.len() + 1);
        lhs.extend_from_slice(data);
        lhs.push(char);
    }

    pub(crate) fn bstr_to_utf8<'i, E>(
        stream: &ParseStream<'i, E>,
        cow: Cow<'i, [u8]>,
    ) -> Result<Cow<'i, str>, Error<'i, E>> {
        match cow {
            Cow::Borrowed(ptr) => {
                let str = core::str::from_utf8(ptr).map_err(|err| {
                    let error = ErrorKind::Utf8ValidationFailedPtr { err };
                    stream.error(error)
                })?;
                Ok(Cow::Borrowed(str))
            },
            Cow::Owned(val) => {
                let str = alloc::string::String::from_utf8(val).map_err(|err| {
                    let error = ErrorKind::Utf8ValidationFailedVal { err };
                    stream.error(error)
                })?;
                Ok(Cow::Owned(str))
            },
        }
    }

    // NOTE: Using `memchr` here is only marginally faster in the benchmarks, likely due to the fact
    // that we are parsing few strings overall and they are relatively short.
    //
    // Most of the gains are probably due to the more efficient API, were we are able to re-use the
    // finders once initialized.
    //
    // But unlike many of the other dependencies used previously, `memchr` has almost no impact on
    // build time, so might as well include it.
    //
    // And for some pathological cases (very large dependency file with very long paths), it could
    // still make a significant difference.
    #[cfg(feature = "memchr")]
    #[rustfmt::skip]
    pub(crate) fn json_string<'i, E>(stream: &mut ParseStream<'i, E>) -> Result<Cow<'i, str>, Error<'i, E>> {
        stream.match_byte(b'"')?;
        let mut text = Cow::Borrowed(b"".as_slice());
        loop {
            let Some(needle) = stream.state.finders.quotes_or_backslash.find(stream.bytes) else {
                let error = ErrorKind::EndOfStringNotFound;
                return Err(stream.error(error));
            };
            let data = stream.next_slice(needle + 1)?;
            match data[data.len() - 1] {
                b'"' => { // tarpaulin::hint
                    let trim = &data[.. data.len() - 1];
                    #[allow(clippy::useless_conversion)] // tarpaulin::hint
                    if text.is_empty() {
                        text = Cow::Borrowed(trim.into()) // tarpaulin::hint
                    } else {
                        text.to_mut().extend_from_slice(trim);
                    };
                    break; // tarpaulin::hint
                },
                b'\\' => unescape(&mut text, data).parse(stream)?,
                _ => unreachable!(),
            }
        }
        let utf8 = bstr_to_utf8(stream, text)?;
        Ok(utf8)
    }

    #[cfg(not(tarpaulin_include))]
    #[cfg(not(feature = "memchr"))]
    #[inline(always)]
    pub(crate) fn json_string<'i, E>(stream: &mut ParseStream<'i, E>) -> Result<Cow<'i, str>, Error<'i, E>> {
        json_string_sans_memchr(stream)
    }

    #[cfg(any(test, not(feature = "memchr")))]
    #[rustfmt::skip]
    #[inline(always)]
    pub(crate) fn json_string_sans_memchr<'i, E>(stream: &mut ParseStream<'i, E>) -> Result<Cow<'i, str>, Error<'i, E>> { let _ = ();
        stream.match_byte(b'"')?;
        let mut text = Cow::Borrowed(b"".as_slice());
        let mut off = 0;
        loop {
            let Some(byte) = stream.bytes.get(off) else {
                let error = ErrorKind::EndOfStringNotFound;
                return Err(stream.error(error));
            };
            match byte {
                b'"' => { // tarpaulin::hint
                    let data = stream.next_slice(off + 1)?;
                    let trim = &data[.. data.len() - 1];
                    #[allow(clippy::useless_conversion)] // tarpaulin::hint
                    if text.is_empty() {
                        #[cfg(not(tarpaulin_include))]
                        {
                            text = Cow::Borrowed(trim.into())
                        }
                    } else {
                        text.to_mut().extend_from_slice(trim);
                    };
                    break; // tarpaulin::hint
                },
                b'\\' => { // tarpaulin::hint
                    let data = stream.next_slice(off + 1)?;
                    unescape(&mut text, data).parse(stream)?;
                    off = 0;
                },
                _ => {}, // tarpaulin::hint
            }
            off += 1;
        }
        let utf8 = bstr_to_utf8(stream, text)?;
        Ok(utf8)
    }

    pub fn utf8_path<'i, E>(stream: &mut ParseStream<'i, E>) -> Result<Cow<'i, Utf8Path>, Error<'i, E>> {
        let str = json_string.parse(stream)?;
        let path = match str {
            #[allow(clippy::useless_conversion)]
            Cow::Borrowed(ptr) => Cow::Borrowed(ptr.into()),
            Cow::Owned(val) => Cow::Owned(Utf8PathBuf::from(val)),
        };
        Ok(path)
    }

    pub(crate) fn unescape<'i, 'r, E>(dst: &'r mut Cow<'i, [u8]>, src: &'i [u8]) -> impl Parser<'i, (), E> + 'r {
        |stream: &mut ParseStream<'i, E>| {
            let src = &src[.. src.len() - 1];
            match stream.next_byte()? {
                b'"' => extend_bytes(dst, src, b'"'),
                b'\\' => extend_bytes(dst, src, b'\\'),
                b'/' => extend_bytes(dst, src, b'/'),
                b'b' => extend_bytes(dst, src, 0x08),
                b'f' => extend_bytes(dst, src, 0x0c),
                b'n' => extend_bytes(dst, src, b'\n'),
                b'r' => extend_bytes(dst, src, b'\r'),
                b't' => extend_bytes(dst, src, b'\t'),
                b'u' => unescape_unicode(dst, src).parse(stream)?,
                _ => return Err(stream.error(ErrorKind::FailedParsingJsonStringEscape)),
            }
            Ok(())
        }
    }

    #[cfg(test)]
    pub(crate) fn u32_to_utf16(char: char) -> alloc::string::String {
        let mut dst = [0u16; 2];
        char.encode_utf16(&mut dst);
        alloc::format!("{:#06x}{:#06x}", dst[0], dst[1]).replace("0x", "\\u")
    }

    #[cfg(not(tarpaulin_include))]
    #[inline(always)]
    fn is_high_surrogate(code: u32) -> bool {
        (0xd800 ..= 0xdbff).contains(&code)
    }

    #[cfg(not(tarpaulin_include))]
    #[inline(always)]
    fn is_low_surrogate(code: u32) -> bool {
        (0xdc00 ..= 0xdfff).contains(&code)
    }

    #[rustfmt::skip]
    #[inline(always)]
    fn code_to_char<'i, E>(unicode: u32, stream: &ParseStream<'i, E>) -> Result<char, Error<'i, E>> { // tarpaulin::hint
        core::char::from_u32(unicode).ok_or_else(|| {
            let error = ErrorKind::CharFromUnicodeFailed { unicode };
            stream.error(error)
        })
    }

    #[rustfmt::skip]
    pub(crate) fn unescape_unicode<'i, 'r, E>(
        dst: &'r mut Cow<'i, [u8]>,
        src: &'i [u8],
    ) -> impl Parser<'i, (), E> + 'r {
        |stream: &mut ParseStream<'i, E>| {
            let hi = { // tarpaulin::hint
                let code = ucs_hex_code.parse(stream)?;
                if !is_high_surrogate(code) {
                    stream.state.encode_utf8(dst, src, code_to_char(code, stream)?);
                    return Ok(());
                }
                code // tarpaulin::hint
            };
            let lo = { // tarpaulin::hint
                if stream.next_slice(2)? != b"\\u" {
                    let error = ErrorKind::MissingUnicodeLowerSurrogate { hi };
                    return Err(stream.error(error));
                };
                let code = ucs_hex_code(stream)?;
                if !is_low_surrogate(code) {
                    let error = ErrorKind::InvalidUnicodeLowerSurrogate { lo: code };
                    return Err(stream.error(error));
                }
                code // tarpaulin::hint
            };
            let code = ((hi - 0xd800) << 10) + (lo - 0xdc00) + 0x10000;
            stream.state.encode_utf8(dst, src, code_to_char(code, stream)?);
            Ok(())
        }
    }

    #[rustfmt::skip]
    #[inline(always)]
    pub(crate) fn ucs_hex_code<'i, E>(stream: &mut ParseStream<'i, E>) -> Result<u32, Error<'i, E>> { // tarpaulin::hint
        let offset = 4;
        let Ok(bytes) = stream.next_slice(offset)?.try_into() else {
            // SAFETY: `.next_slice` already verified we have a slice of length 4
            unsafe { core::hint::unreachable_unchecked() }
        };
        let unicode = self::number::from_radix_16(bytes).ok_or_else(|| {
            let error = ErrorKind::InvalidUnicodeEscapeHex { bytes };
            stream.error(error)
        })?;
        Ok(unicode)
    }
}

#[cfg(test)]
mod test {
    use proptest::prelude::*;

    use super::*;
    use crate::r5::parsers::State;

    mod r5 {
        pub use crate::spec::r5::parsers::ErrorKind;
    }

    mod parse_stream {
        use super::*;

        mod errors {
            use super::*;

            #[test]
            #[should_panic(expected = "test.ddi:1:1: error: No remaining bytes\n")]
            fn peek_byte() {
                let path = "test.ddi";
                let input = b"".as_slice();
                let state = State::default();
                let mut stream = ParseStream::<r5::ErrorKind>::new(path, input, state);
                if let Err(err) = stream.next_byte() {
                    panic!("{err}");
                }
            }

            #[test]
            #[should_panic(expected = "test.ddi:1:1: error: Byte mismatch: expected `1`, actual `0`\n")]
            fn match_byte() {
                let path = "test.ddi";
                let input = b"0".as_slice();
                let state = State::default();
                let mut stream = ParseStream::<r5::ErrorKind>::new(path, input, state);
                if let Err(err) = stream.match_byte(b'1') {
                    panic!("{err}");
                }
            }

            #[test]
            #[should_panic(
                expected = "test.ddi:1:1: error: Remaining bytes less than requested slice length: remaining: 3, requested: 6\n"
            )]
            fn next_slice() {
                let path = "test.ddi";
                let input = b"foo".as_slice();
                let state = State::default();
                let mut stream = ParseStream::<r5::ErrorKind>::new(path, input, state);
                if let Err(err) = stream.next_slice(b"barbaz".len()) {
                    panic!("{err}");
                }
            }

            #[test]
            #[should_panic(expected = "test.ddi:1:6: error: Slice mismatch: expected `barbaz`, actual `foobar`\n")]
            fn match_slice() {
                let path = "test.ddi";
                let input = b"foobar".as_slice();
                let state = State::default();
                let mut stream = ParseStream::<r5::ErrorKind>::new(path, input, state);
                if let Err(err) = stream.match_slice(b"barbaz") {
                    panic!("{err}");
                }
            }
        }
    }

    mod json {
        use super::*;
        use crate::util::parsers::json::*;

        mod errors {
            use super::*;

            #[test]
            #[should_panic(expected = "test.ddi:1:1: error: Failed parsing bool\n")]
            fn bool() {
                let text = "#rue";
                let path = "test.ddi";
                let input = text.as_bytes();
                let state = State::default();
                let mut stream = ParseStream::<r5::ErrorKind>::new(path, input, state);
                if let Err(err) = super::bool.parse(&mut stream) {
                    panic!("{err}");
                }
            }

            #[test]
            #[should_panic(
                expected = "test.ddi:1:9: error: Failed parsing JSON object property:\nexpected one of: { ',', '}' }\n"
            )]
            fn field() {
                let text = "\"key\": 0 #";
                let path = "test.ddi";
                let input = text.as_bytes();
                let state = State::default();
                let mut stream = ParseStream::<r5::ErrorKind>::new(path, input, state);
                let key = b"\"key\"".as_slice();
                let val = crate::util::parsers::number::dec_uint;
                let mut p = self::json::field(key, val);
                if let Err(err) = p.parse(&mut stream) {
                    panic!("{err}");
                }
            }

            #[test]
            #[should_panic(
                expected = "test.ddi:1:5: error: Failed parsing JSON array:\nexpected one of: { ',', ']' }\n"
            )]
            fn vec() {
                let text = "[ 0 #";
                let path = "test.ddi";
                let input = text.as_bytes();
                let state = State::default();
                let mut stream = ParseStream::<r5::ErrorKind>::new(path, input, state);
                let val = crate::util::parsers::number::dec_uint;
                let mut p = self::json::vec(val);
                if let Err(err) = p.parse(&mut stream) {
                    panic!("{err}");
                }
            }
        }
    }

    mod number {
        use super::*;

        mod errors {
            use super::*;

            #[test]
            #[should_panic(expected = "test.ddi:1:1: error: Failed parsing JSON unsigned integer\n")]
            fn dec_uint() {
                let text = "true";
                let path = "test.ddi";
                let input = text.as_bytes();
                let state = State::default();
                let mut stream = ParseStream::<r5::ErrorKind>::new(path, input, state);
                let mut p = crate::util::parsers::number::dec_uint;
                if let Err(err) = p.parse(&mut stream) {
                    panic!("{err}");
                }
            }
        }
    }

    mod string {
        use super::*;
        use crate::util::parsers::string::*;

        #[test]
        fn json_string_borrows_when_no_escapes_present() {
            let text = "\"foobar\"";
            let path = "test.ddi";
            let input = text.as_bytes();
            let state = State::default();
            let mut stream = ParseStream::<r5::ErrorKind>::new(path, input, state);
            let mut p = crate::util::parsers::string::json_string;
            let val = p.parse(&mut stream).unwrap();
            assert!(matches!(val, Cow::Borrowed(_)));
        }

        #[test]
        fn json_string_borrows_when_escapes_present() {
            let text = "\"foo\\nbar\"";
            let path = "test.ddi";
            let input = text.as_bytes();
            let state = State::default();
            let mut stream = ParseStream::<r5::ErrorKind>::new(path, input, state);
            let mut p = crate::util::parsers::string::json_string;
            let val = p.parse(&mut stream).unwrap();
            assert!(matches!(val, Cow::Owned(_)));
        }

        #[cfg(feature = "std")]
        #[test]
        fn json_string_correctly_unescapes_while_parsing() {
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
                let path = "test.ddi";
                let input = text.as_bytes();
                let state = State::default();
                let mut stream = ParseStream::<()>::new(path, input, state);
                let unescaped = self::json_string.parse(&mut stream).unwrap();
                let lhs = unescaped;
                let rhs = std::format!("foo{}bar", char::from(raw));
                assert_eq!(lhs, rhs);
            }
        }

        #[cfg(feature = "std")]
        #[test]
        fn json_string_sans_memchr_correctly_unescapes_while_parsing() {
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
                let path = "test.ddi";
                let input = text.as_bytes();
                let state = State::default();
                let mut stream = ParseStream::<()>::new(path, input, state);
                let unescaped = self::json_string_sans_memchr.parse(&mut stream).unwrap();
                let lhs = unescaped;
                let rhs = std::format!("foo{}bar", char::from(raw));
                assert_eq!(lhs, rhs);
            }
        }

        #[test]
        fn unescape_utf16_static() {
            let char = '💯';
            let text = self::string::u32_to_utf16(char);
            let path = "test.ddi";
            let input = text.as_bytes().strip_prefix(b"\\").unwrap();
            let state = State::default();
            let mut stream = ParseStream::<r5::ErrorKind>::new(path, input, state);
            let mut dst = Cow::Owned(alloc::vec![]);
            let src = b"\\";
            if let Err(err) = self::string::unescape(&mut dst, src).parse(&mut stream) {
                panic!("{err}");
            };
            assert_eq!(dst, String::from(char).as_bytes());
        }

        proptest! {
            #[cfg_attr(miri, ignore)]
            #[test]
            fn unescape_utf8(
                char in proptest::char::range(
                    unsafe { char::from_u32_unchecked(0x0000u32) },
                    unsafe { char::from_u32_unchecked(0xFFFFu32) }
                )
            ) {
                let text = alloc::format!("{:#06x}", u32::from(char)).replace("0x", "\\u");
                let path = "test.ddi";
                let input = text.as_bytes().strip_prefix(b"\\").unwrap();
                let state = State::default();
                let mut stream = ParseStream::<r5::ErrorKind>::new(path, input, state);
                let mut dst = Cow::Owned(alloc::vec![]);
                let src = b"\\";
                if let Err(err) = self::string::unescape(&mut dst, src).parse(&mut stream) {
                    panic!("{err}");
                };
                assert_eq!(dst, String::from(char).as_bytes());
            }

            #[cfg_attr(miri, ignore)]
            #[test]
            fn unescape_utf16(
                char in proptest::char::range(
                    unsafe { char::from_u32_unchecked(0x010000u32) },
                    unsafe { char::from_u32_unchecked(0x10FFFFu32) }
                )
            ) {
                let text = self::string::u32_to_utf16(char);
                let path = "test.ddi";
                let input = text.as_bytes().strip_prefix(b"\\").unwrap();
                let state = State::default();
                let mut stream = ParseStream::<r5::ErrorKind>::new(path, input, state);
                let mut dst = Cow::Owned(alloc::vec![]);
                let src = b"\\";
                if let Err(err) = self::string::unescape(&mut dst, src).parse(&mut stream) {
                    panic!("{err}");
                };
                assert_eq!(dst, String::from(char).as_bytes());
            }
        }

        mod errors {
            use super::*;

            #[test]
            #[should_panic(expected = "test.ddi:1:1: error: End of string not found\n")]
            fn json_string_partial_string() {
                let text = "\"foo";
                let path = "test.ddi";
                let input = text.as_bytes();
                let state = State::default();
                let mut stream = ParseStream::<r5::ErrorKind>::new(path, input, state);
                if let Err(err) = self::json_string.parse(&mut stream) {
                    panic!("{err}");
                };
            }

            #[test]
            #[should_panic(expected = "test.ddi:1:1: error: End of string not found\n")]
            fn json_string_sans_memchr_partial_string() {
                let text = "\"foob4";
                let path = "test.ddi";
                let input = text.as_bytes();
                let state = State::default();
                let mut stream = ParseStream::<r5::ErrorKind>::new(path, input, state);
                if let Err(err) = self::json_string_sans_memchr.parse(&mut stream) {
                    panic!("{err}");
                }
            }

            #[test]
            #[should_panic(expected = "test.ddi:1:1: error: Byte mismatch: expected `\\\"`, actual `t`\n")]
            fn json_string_non_string() {
                let text = "true";
                let path = "test.ddi";
                let input = text.as_bytes();
                let state = State::default();
                let mut stream = ParseStream::<r5::ErrorKind>::new(path, input, state);
                let mut p = crate::util::parsers::string::json_string;
                if let Err(err) = p.parse(&mut stream) {
                    panic!("{err}");
                }
            }

            #[test]
            #[should_panic(expected = "test.ddi:1:1: error: Byte mismatch: expected `\\\"`, actual `t`\n")]
            fn json_string_sans_memchr_non_string() {
                let text = "true";
                let path = "test.ddi";
                let input = text.as_bytes();
                let state = State::default();
                let mut stream = ParseStream::<r5::ErrorKind>::new(path, input, state);
                let mut p = crate::util::parsers::string::json_string_sans_memchr;
                if let Err(err) = p.parse(&mut stream) {
                    panic!("{err}");
                }
            }

            #[test]
            #[should_panic(
                expected = "test.ddi:1:3: error: Failed parsing JSON string escape:\nexpected one of: { '\"', '\\', '/', 'b', 'f', 'n', 'r', 't', 'u' }\n"
            )]
            fn unescape() {
                let text = "\"\\#\"";
                let path = "test.ddi";
                let input = text.as_bytes();
                let state = State::default();
                let mut stream = ParseStream::<r5::ErrorKind>::new(path, input, state);
                let mut p = crate::util::parsers::string::json_string;
                if let Err(err) = p.parse(&mut stream) {
                    panic!("{err}");
                }
            }

            #[test]
            #[should_panic(expected = "test.ddi:1:1: error: UTF-8 validation failed:")]
            fn bstr_to_utf8_expectedly_fails_invalid_utf8_borrowed() {
                let text = "";
                let path = "test.ddi";
                let input = text.as_bytes();
                let state = State::default();
                let stream = ParseStream::<r5::ErrorKind>::new(path, input, state);
                // NOTE: Here we use \u{D800} (leading surrogate) which is invalid standalone
                let bor = [0xedu8, 0xa0u8, 0x80u8].as_slice();
                let cow = Cow::Borrowed(bor);
                if let Err(err) = self::bstr_to_utf8(&stream, cow) {
                    panic!("{err}");
                };
            }

            #[test]
            #[should_panic(expected = "test.ddi:1:1: error: UTF-8 validation failed:")]
            fn bstr_to_utf8_expectedly_fails_invalid_utf8_owned() {
                let text = "";
                let path = "test.ddi";
                let input = text.as_bytes();
                let state = State::default();
                let stream = ParseStream::<r5::ErrorKind>::new(path, input, state);
                // NOTE: Here we use \u{D800} (leading surrogate) which is invalid standalone
                let own = alloc::vec![0xed, 0xa0, 0x80];
                let cow = Cow::Owned(own);
                if let Err(err) = self::bstr_to_utf8(&stream, cow) {
                    panic!("{err}");
                };
            }

            #[test]
            #[should_panic(expected = "test.ddi:1:11: error: Invalid unicode lower surrogate: 0x2764\n")]
            fn unescape_unicode_expectedly_fails_invalid_lower_surrogate() {
                let text = "uD800\\u2764";
                let path = "test.ddi";
                let input = text.as_bytes();
                let state = State::default();
                let mut stream = ParseStream::<r5::ErrorKind>::new(path, input, state);
                let mut dst = Cow::Owned(alloc::vec![]);
                let src = b"\\";
                if let Err(err) = super::unescape(&mut dst, src).parse(&mut stream) {
                    panic!("{err}");
                };
            }

            #[test]
            #[should_panic(
                expected = "test.ddi:1:7: error: Missing unicode lower surrogate pair for leading high surrogate: 0xd834\n"
            )]
            fn unescape_unicode_missing_lower_surrogate() {
                let text = "uD834missing";
                let path = "test.ddi";
                let input = text.as_bytes();
                let state = State::default();
                let mut stream = ParseStream::<r5::ErrorKind>::new(path, input, state);
                let mut dst = Cow::Owned(alloc::vec![]);
                let src = b"\\";
                if let Err(err) = super::unescape(&mut dst, src).parse(&mut stream) {
                    panic!("{err}");
                };
            }

            #[test]
            #[should_panic(
                expected = "test.ddi:1:5: error: Conversion of unicode u32 to char failed: u32 value: 0xdc00\n"
            )]
            fn unescape_unicode_expectedly_fails_invalid_utf8() {
                let text = "uDC00";
                let path = "test.ddi";
                let input = text.as_bytes();
                let state = State::default();
                let mut stream = ParseStream::<r5::ErrorKind>::new(path, input, state);
                let mut dst = Cow::Owned(alloc::vec![]);
                let src = b"\\";
                if let Err(err) = super::unescape(&mut dst, src).parse(&mut stream) {
                    panic!("{err}");
                };
            }

            #[test]
            #[should_panic(
                expected = "test.ddi:1:1: error: Remaining bytes less than requested slice length: remaining: 2, requested: 4\n"
            )]
            fn unescape_unicode_truncated() {
                let text = "uDC";
                let path = "test.ddi";
                let input = text.as_bytes();
                let state = State::default();
                let mut stream = ParseStream::<r5::ErrorKind>::new(path, input, state);
                let mut dst = Cow::Owned(alloc::vec![]);
                let src = b"\\";
                if let Err(err) = super::unescape(&mut dst, src).parse(&mut stream) {
                    panic!("{err}");
                };
            }

            #[test]
            #[should_panic(expected = "test.ddi:1:5: error: Invalid unicode escape hex: DCXX\n")]
            fn unescape_unicode_invalid_hex() {
                let text = "uDCXX";
                let path = "test.ddi";
                let input = text.as_bytes();
                let state = State::default();
                let mut stream = ParseStream::<r5::ErrorKind>::new(path, input, state);
                let mut dst = Cow::Owned(alloc::vec![]);
                let src = b"\\";
                if let Err(err) = super::unescape(&mut dst, src).parse(&mut stream) {
                    panic!("{err}");
                };
            }
        }
    }
}
