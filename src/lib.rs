// Copyright 2018 Koji Higasa.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! A vector which has gap inside the buf implemented with a growable
//! ring buffer. It's written `GapVec<T>` but pronounced 'gap vector'.
//!
//! # Examples
//!
//! You can explicitly create a `GapVec<T>` with `new`:
//!
//! ```
//! let v: GapVec<i32> = Vec::new();
//! ```
//!

#![feature(alloc, raw_vec_internals)]
extern crate alloc;
use alloc::raw_vec::RawVec;
use std;
use std::ops::Range;

/// A vector which has gap inside the buf implemented with a growable
/// ring buffer. It's written `GapVec<T>` but pronounced 'gap vector'.
///
/// # Examples
///
/// You can explicitly create a `GapVec<T>` with `new` :
///
/// ```
/// let v: GapVec<i32> = Vec::new();
/// ```
///

pub struct GapVec<T> {
    buf: RawVec<T>,
    gap: Range<usize>,
}

impl<T> GapVec<T> {
    pub fn new() -> GapVec<T> {
        GapVec {
            buf: RawVec::new(),
            gap: 0..0,
        }
    }
}

