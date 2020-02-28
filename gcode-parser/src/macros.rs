/// Create a located token with a given optional offset and line number
///
/// Calls the unsafe [`new_from_raw_offset()`] method
#[cfg(test)]
#[macro_export]
#[doc(hidden)]
macro_rules! tok {
    ($tok:expr, $start_offs:expr, $end_offs:expr, $start_line:expr, $end_line:expr) => {{
        let start_pos =
            unsafe { $crate::ParseInput::new_from_raw_offset($start_offs, $start_line, "", ()) };
        let end_pos =
            unsafe { $crate::ParseInput::new_from_raw_offset($end_offs, $end_line, "", ()) };

        $crate::token::Token {
            start_pos,
            end_pos,
            token: $tok,
        }
    }};
    ($tok:expr, $start_offs:expr, $end_offs:expr) => {{
        tok!($tok, $start_offs, $end_offs, 1, 1)
    }};
}

/// Remaining parse input
#[cfg(test)]
#[macro_export]
#[doc(hidden)]
macro_rules! rem {
    ($frag:expr, $offs:expr, $line:expr) => {{
        unsafe { $crate::ParseInput::new_from_raw_offset($offs, $line, $frag, ()) }
    }};
    ($frag:expr, $offs:expr) => {{
        rem!($frag, $offs, 1)
    }};
    ($frag:expr) => {{
        rem!($frag, 0, 1)
    }};
}
