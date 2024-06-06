use alloc::borrow::Cow;

use winnow::{error::ParserError, prelude::*, BStr};

pub(crate) mod json;
pub(crate) mod util;

#[derive(Debug)]
pub(crate) struct Finders {
    #[cfg(target_feature = "avx2")]
    quotes_or_backslash: memchr::arch::x86_64::avx2::memchr::Two,
    #[cfg(all(not(target_feature = "avx2"), target_feature = "sse2"))]
    quotes_or_backslash: memchr::arch::x86_64::sse2::memchr::Two,
    #[cfg(all(not(target_feature = "avx2"), not(target_feature = "sse2")))]
    quotes_or_backslash: memchr::arch::all::memchr::Two,

    #[cfg(all(target_feature = "bmi2", target_feature = "avx2"))]
    curly_close: memchr::arch::x86_64::avx2::memchr::One,
    #[cfg(all(target_feature = "bmi2", not(target_feature = "avx2"), target_feature = "sse2"))]
    curly_close: memchr::arch::x86_64::sse2::memchr::One,
    #[cfg(all(target_feature = "bmi2", not(target_feature = "avx2"), not(target_feature = "sse2")))]
    curly_close: memchr::arch::all::memchr::One,
}
impl Default for Finders {
    fn default() -> Self {
        #[cfg(target_feature = "avx2")]
        let quotes_or_backslash = memchr::arch::x86_64::avx2::memchr::Two::new(b'"', b'\\').unwrap();
        #[cfg(all(not(target_feature = "avx2"), target_feature = "sse2"))]
        let quotes_or_backslash = memchr::arch::x86_64::sse2::memchr::Two::new(b'"', b'\\').unwrap();
        #[cfg(all(not(target_feature = "avx2"), not(target_feature = "sse2")))]
        let quotes_or_backslash = memchr::arch::all::memchr::Two::new(b'"', b'\\');

        #[cfg(all(target_feature = "bmi2", target_feature = "avx2"))]
        let curly_close = memchr::arch::x86_64::avx2::memchr::One::new(b'}').unwrap();
        #[cfg(all(target_feature = "bmi2", not(target_feature = "avx2"), target_feature = "sse2"))]
        let curly_close = memchr::arch::x86_64::sse2::memchr::One::new(b'}').unwrap();
        #[cfg(all(target_feature = "bmi2", not(target_feature = "avx2"), not(target_feature = "sse2")))]
        let curly_close = memchr::arch::all::memchr::One::new(b'}');

        Self {
            quotes_or_backslash,
            #[cfg(target_feature = "bmi2")]
            curly_close,
        }
    }
}

#[derive(Debug)]
pub struct State {
    finders: Finders,
    utf8_encode_buffer: [u8; 4],
}
impl State {
    fn encode_utf8<'i>(&mut self, dst: &mut Cow<'i, BStr>, src: &'i [u8], esc: char) {
        let cow = self::util::cow_to_mut_with_reserve(dst, src.len() + esc.len_utf8());
        let esc = esc.encode_utf8(self.utf8_encode_buffer.as_mut()).as_bytes();
        cow.extend_from_slice(src);
        cow.extend_from_slice(esc);
        self.utf8_encode_buffer = [0u8; 4];
    }

    #[cfg(target_feature = "bmi2")]
    pub fn hex_to_u32<'i, E>(&self, input: &StateStream<'i>) -> PResult<(u32, usize), E>
    where
        E: ParserError<StateStream<'i>>,
    {
        let needle = self.finders.curly_close.find(input).ok_or_else(|| {
            let message = "failed to locate UCS sequence closing delimiter";
            winnow::error::ErrMode::assert(input, message)
        })?;
        debug_assert!(needle < 9);
        let input = &input[0 .. needle];
        let (hi_bytes, lo_bytes, lsl) = self::util::atoi::u32::split_into_words(input);
        let lo = self::util::atoi::u32::u32_from_u8x4(lo_bytes);
        let hi = self::util::atoi::u32::u32_from_u8x4(hi_bytes);
        let word = (hi << (lsl * 4)) + lo;
        Ok((word, needle))
    }

    #[cfg(not(target_feature = "bmi2"))]
    #[no_coverage]
    pub fn hex_to_u32<'i, E>(&self, input: &StateStream<'i>) -> PResult<(u32, usize), E>
    where
        E: ParserError<StateStream<'i>>,
    {
        hex_to_u32_sans_bmi2(input)
    }

    #[inline(always)]
    pub fn hex_to_u32_sans_bmi2<'i, E>(&self, input: &StateStream<'i>) -> PResult<(u32, usize), E>
    where
        E: ParserError<StateStream<'i>>,
    {
        let mut index = 0;
        let mut number = 0u32;
        while index != input.len() {
            if let Some(digit) = crate::util::winnow::util::atoi::u32::ascii_to_hexdigit(input[index]) {
                number *= 16;
                number += digit;
                index += 1;
            } else {
                break;
            }
        }
        #[cfg(test)]
        if input[index] != b'}' {
            let message = "failed to locate UCS sequence closing delimiter";
            return Err(winnow::error::ErrMode::assert(input, message));
        };
        Ok((number, index))
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

#[cfg(test)]
mod test {
    use alloc::string::ToString;

    use proptest::prelude::*;

    #[test]
    fn hex_to_u32_works_static() {
        let char = '💯';
        let text = char.escape_unicode().to_string();
        let text = text.strip_prefix("\\u{").unwrap();
        let input = winnow::BStr::new(text);
        let state = crate::r5::parsers::State::default();
        let stream = &mut winnow::Stateful { input, state };
        let (number, index) = stream.state.hex_to_u32::<()>(stream).unwrap();
        assert_eq!(number, u32::from(char));
        assert_eq!(index, text.strip_suffix("}").unwrap().len());
    }

    #[test]
    #[should_panic(expected = "failed to locate UCS sequence closing delimiter")]
    fn hex_to_u32_fails_expectedly() {
        let char = '💯';
        let text = char.escape_unicode().to_string().replace("}", "#");
        let text = text.strip_prefix("\\u{").unwrap();
        let input = winnow::BStr::new(text);
        let state = crate::r5::parsers::State::default();
        let stream = &mut winnow::Stateful { input, state };
        stream.state.hex_to_u32::<()>(stream).unwrap();
    }

    #[test]
    fn hex_to_u32_sans_bmi2_works_static() {
        let char = '💯';
        let text = char.escape_unicode().to_string();
        let text = text.strip_prefix("\\u{").unwrap();
        let input = winnow::BStr::new(text);
        let state = crate::r5::parsers::State::default();
        let stream = &mut winnow::Stateful { input, state };
        let (number, index) = stream.state.hex_to_u32_sans_bmi2::<()>(stream).unwrap();
        assert_eq!(number, u32::from(char));
        assert_eq!(index, text.strip_suffix("}").unwrap().len());
    }

    #[test]
    #[should_panic(expected = "failed to locate UCS sequence closing delimiter")]
    fn hex_to_u32_sans_bmi2_fails_expectedly() {
        let char = '💯';
        let input = char.escape_unicode().to_string().replace("}", "#");
        let input = input.strip_prefix("\\u{").unwrap();
        let input = winnow::BStr::new(input);
        let state = crate::r5::parsers::State::default();
        let stream = &mut winnow::Stateful { input, state };
        stream.state.hex_to_u32_sans_bmi2::<()>(stream).unwrap();
    }

    proptest! {
        #[test]
        fn hex_to_u32_works(char in any::<char>()) {
            let text = char.escape_unicode().to_string();
            let text = text.strip_prefix("\\u{").unwrap();
            let state = crate::r5::parsers::State::default();
            let input = winnow::BStr::new(&text);
            let stream = &mut winnow::Stateful { input, state };
            let (number, index) = stream.state.hex_to_u32::<()>(stream).unwrap();
            assert_eq!(number, u32::from(char));
            assert_eq!(index, text.strip_suffix("}").unwrap().len());
        }

        #[test]
        fn hex_to_u32_sans_bmi2_works(char in any::<char>()) {
            let text = char.escape_unicode().to_string();
            let text = text.strip_prefix("\\u{").unwrap();
            let state = crate::r5::parsers::State::default();
            let input = winnow::BStr::new(&text);
            let stream = &mut winnow::Stateful { input, state };
            let (number, index) = stream.state.hex_to_u32_sans_bmi2::<()>(stream).unwrap();
            assert_eq!(number, u32::from(char));
            assert_eq!(index, text.strip_suffix("}").unwrap().len());
        }
    }
}
