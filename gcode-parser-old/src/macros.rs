/// Create a located token with a given optional offset and line number
///
/// Calls the unsafe [`new_from_raw_offset()`] method
#[cfg(test)]
#[macro_export]
#[doc(hidden)]
macro_rules! tok {
    ($tok:expr, offs = ($start_offs:expr, $end_offs:expr), line = ($start_line:expr, $end_line:expr), col = ($start_col:expr, $end_col:expr)) => {{
        $crate::token::Token {
            start_pos: $crate::Location::new($start_offs, $start_line, $start_col),
            end_pos: $crate::Location::new($end_offs, $end_line, $end_col),
            token: $tok,
        }
    }};
    ($tok:expr, offs = ($start_offs:expr, $end_offs:expr), line = ($start_line:expr, $end_line:expr)) => {{
        let start =
            unsafe { $crate::ParseInput::new_from_raw_offset($start_offs, $start_line, "", ()) };

        let end = unsafe { $crate::ParseInput::new_from_raw_offset($end_offs, $end_line, "", ()) };

        tok!(
            $tok,
            offs = ($start_offs, $end_offs),
            line = ($start_line, $end_line),
            col = (start.get_utf8_column(), end.get_utf8_column())
        )
    }};
    ($tok:expr, offs = ($start_offs:expr, $end_offs:expr)) => {{
        tok!($tok, offs = ($start_offs, $end_offs), line = (1, 1))
    }};
}

/// Remaining parse input
#[cfg(test)]
#[macro_export]
#[doc(hidden)]
macro_rules! rem {
    ($frag:expr, $offs:expr, $line:expr) => {{
        unsafe { $crate::ParseInput::new_from_raw_offset($offs, $line, $frag, ()) }.into()
    }};
    ($frag:expr, $offs:expr) => {{
        rem!($frag, $offs, 1)
    }};
    ($frag:expr) => {{
        rem!($frag, 0, 1)
    }};
}
