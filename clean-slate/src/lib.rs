#![deny(missing_debug_implementations)]

pub mod block;
pub mod const_generics_spanned;
pub mod const_generics_test;
pub mod program;
pub mod spanned_word;

use nom_locate::LocatedSpan;

pub type Span<'a> = LocatedSpan<&'a str>;
