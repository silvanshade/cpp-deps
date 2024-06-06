#[cfg(test)]
pub(crate) mod proptest;
#[cfg(feature = "winnow")]
pub(crate) mod winnow;

#[cfg(test)]
use alloc::borrow::{Cow, ToOwned};

#[cfg(test)]
#[allow(clippy::ptr_arg)]
pub fn cow_is_owned<B>(cow: &Cow<B>) -> bool
where
    B: ToOwned + ?Sized,
{
    matches!(cow, Cow::Owned(_))
}

#[cfg(test)]
pub fn count_escapes(str: impl AsRef<str>) -> u64 {
    let str = str.as_ref();
    let bytes = str.as_bytes();
    let mut off = 0;
    let mut count = 0;
    if bytes.len() > 1 {
        while off < bytes.len() - 1 {
            match bytes[off] {
                b'\\' if bytes[off + 1] == b'\\' => off += 1,
                b'\\' => count += 1,
                #[cfg(not(tarpaulin_include))]
                _ => {},
            }
            off += 1;
        }
    }
    count
}

#[cfg(test)]
pub fn count_escaped_strings(str: impl AsRef<str>) -> (u64, u64) {
    let str = str.as_ref();
    let bytes = str.as_bytes();
    let mut off = 0;
    let mut num_strings = 0;
    let mut num_escaped = 0;
    if bytes.len() > 1 {
        while off < bytes.len() - 1 {
            if bytes[off] == b'"' {
                off += 1;
                let mut escaped = false;
                'inner: loop {
                    match bytes[off] {
                        #[rustfmt::skip]
                        b'\\' => { // tarpaulin::hint
                            off += 1;
                            if bytes[off] != b'\\' {
                                escaped = true;
                            }
                        },
                        #[rustfmt::skip]
                        b'"' => { // tarpaulin::hint
                            num_strings += 1;
                            num_escaped += u64::from(escaped);
                            off += 1;
                            break 'inner; // tarpaulin::hint
                        },
                        #[cfg(not(tarpaulin_include))]
                        _ => {},
                    }
                    off += 1;
                }
            }
            off += 1;
        }
    }
    (num_strings, num_escaped)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn count_escapes_empty() {
        let str = "";
        assert_eq!(0, count_escapes(str));
        assert_eq!((0, 0), count_escaped_strings(str));
    }

    #[test]
    fn count_escapes_single_backslash() {
        let str = "\\";
        assert_eq!(0, count_escapes(str));
        assert_eq!((0, 0), count_escaped_strings(str));
    }

    #[test]
    fn count_escapes_sans_escapes_short() {
        let str = "\n";
        assert_eq!(0, count_escapes(str));
        assert_eq!((0, 0), count_escaped_strings(str));
    }

    #[test]
    fn count_escapes_sans_escapes() {
        let str = "foo\nbar";
        assert_eq!(0, count_escapes(str));
        assert_eq!((0, 0), count_escaped_strings(str));
    }

    #[test]
    fn count_escapes_with_one_escapes_short() {
        let str = "\\n";
        assert_eq!(1, count_escapes(str));
        assert_eq!((0, 0), count_escaped_strings(str));
    }

    #[test]
    fn count_escapes_with_one_escapes() {
        let str = "foo\\nbar";
        assert_eq!(1, count_escapes(str));
        assert_eq!((0, 0), count_escaped_strings(str));
    }

    #[test]
    fn count_escapes_with_two_escapes() {
        let str = "foo\\nbar\\u{1f4af}baz";
        assert_eq!(2, count_escapes(str));
        assert_eq!((0, 0), count_escaped_strings(str));
    }

    #[test]
    fn count_escaped_strings_zero_quoted() {
        let str = r#"
            foo
        "#;
        assert_eq!(0, count_escapes(str));
        assert_eq!((0, 0), count_escaped_strings(str));
    }

    #[test]
    fn count_escaped_strings_zero_quoted_one_escape() {
        let str = r#"
            foo\nbar
        "#;
        assert_eq!(1, count_escapes(str));
        assert_eq!((0, 0), count_escaped_strings(str));
    }

    #[test]
    fn count_escaped_strings_one_quoted_zero_escapes() {
        let str = r#"
            "foo"
        "#;
        assert_eq!(0, count_escapes(str));
        assert_eq!((1, 0), count_escaped_strings(str));
    }

    #[test]
    fn count_escaped_strings_one_quoted_one_escapes() {
        let str = r#"
            "foo\nbar"
        "#;
        assert_eq!(1, count_escapes(str));
        assert_eq!((1, 1), count_escaped_strings(str));
    }

    #[test]
    fn count_escaped_strings_two_quoted_three_escapes() {
        let str = r#"
            "foo\nbar"%^&*
            "foo\nbar\u{1f4af}baz"
        "#;
        assert_eq!(3, count_escapes(str));
        assert_eq!((2, 2), count_escaped_strings(str));
    }

    #[test]
    fn count_escaped_strings_three_quoted_three_escapes() {
        let str = r#"
            "foo\nbar"%^&*
            "foo\nbar\u{1f4af}baz"
            "qux"
        "#;
        assert_eq!(3, count_escapes(str));
        assert_eq!((3, 2), count_escaped_strings(str));
    }
}
