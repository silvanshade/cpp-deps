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
        let cow = self::util::cow_to_mut_with_reserve(dst, src.len() + esc.len_utf8());
        let esc = esc.encode_utf8(self.utf8_encode_buffer.as_mut()).as_bytes();
        cow.extend_from_slice(src);
        cow.extend_from_slice(esc);
        self.utf8_encode_buffer = [0u8; 4];
    }

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
                break; // tarpaulin::hint
            }
        }
        #[cfg(test)] // tarpaulin::hint
        if input[index] != b'}' {
            let message = "failed to locate UCS sequence closing delimiter";
            #[cfg(not(tarpaulin_include))] // NOTE: the `return Err` is always missed by tarpaulin
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
        let input = char.escape_unicode().to_string().replace("}", "#");
        let input = input.strip_prefix("\\u{").unwrap();
        let input = winnow::BStr::new(input);
        let state = crate::r5::parsers::State::default();
        let stream = &mut winnow::Stateful { input, state };
        stream.state.hex_to_u32::<()>(stream).unwrap();
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
    }
}
