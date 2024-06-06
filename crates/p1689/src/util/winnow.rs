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
    pub fn hex_to_u32<'i, E>(&self, input: &StateStream<'i>) -> PResult<(u32, usize), E>
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
