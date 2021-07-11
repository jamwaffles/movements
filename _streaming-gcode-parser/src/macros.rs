#[macro_export]
#[doc(hidden)]
macro_rules! assert_parse {
    ($parser:expr , $input:expr , ($remaining:expr, $expected:expr)) => {
        let remaining_span = unsafe {
            $crate::Span::new_from_raw_offset($input.len() - $remaining.len(), 1, $remaining, ())
        };

        assert_eq!($parser($input.into()), Ok((remaining_span, $expected)));
    };
}
