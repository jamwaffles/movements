//! A quick test for `alloc` crate usage.
//!
//! Looks like `extern crate alloc` is required. If `#![no_std]` is left in, this code will fail to
//! compile because no allocator could be found.

#![no_std]

extern crate alloc;

use alloc::vec::Vec;

fn main() {
    let v: Vec<i32> = Vec::new();
}
