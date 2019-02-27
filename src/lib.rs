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
//! let v: GapVec<i32> = GapVec::new();
//! ```
//!

#![feature(alloc, raw_vec_internals)]
extern crate alloc;

use alloc::raw_vec::RawVec;
use std::ops::Range;

/// A vector which has gap inside the buf implemented with a growable
/// ring buffer. It's written `GapVec<T>` but pronounced 'gap vector'.
///
/// # Examples
///
/// You can explicitly create a `GapVec<T>` with `new` :
///
/// ```
/// let v: GapVec<i32> = GapVec::new();
/// ```
///

pub struct GapVec<T> {
    buf: RawVec<T>,
    gap: Range<usize>,
}

impl<T> GapVec<T> {
    /// Constructs a new, empty `GapVec<T>`.
    ///
    /// The gap vector will not allocate until elements are pushed onto it.
    ///
    /// # Examples
    ///
    /// ```
    /// # #![allow(unused_mut)]
    /// let mut gap_vec: GapVec<i32> = GapVec::new();
    /// ```
    #[inline]
    pub fn new() -> GapVec<T> {
        GapVec {
            buf: RawVec::new(),
            gap: 0..0,
        }
    }

    /// Constructs a new, empty `GapVec<T>` with the specified capacity.
    ///
    /// The gap vector will be able to hold exactly `capacity` elements without
    /// reallocating. If `capacity` is 0, the gap vector will not allocate.
    ///
    /// It is important to note that although the returned gap vector has the
    /// *capacity* specified, the vector will have a zero *length*.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut gap_vec = GapVec::with_capacity(10);
    ///
    /// // The gap vector contains no items, even though it has capacity for more.
    /// assert_eq!(gap_vec.len(), 0);
    ///
    /// // These are all done without reallocating
    /// for i in 0..10 {
    ///     gap_vec.push(i);
    /// }
    ///
    /// // ,but this may make the gap vector reallocate.
    /// gap_vec.push(11);
    /// ```
    #[inline]
    pub fn with_capacity(capacity: usize) -> GapVec<T> {
        GapVec {
            buf: RawVec::with_capacity(capacity),
            gap: 0..0,
        }
    }

    /// Returns the number of elements the gap vector can hold without
    /// reallocating.
    ///
    /// # Examples
    ///
    /// ```
    /// let gap_vec: GapVec<i32> = GapVec::with_capacity(10);
    /// assert_eq!(gap_vec.capacity(), 10);
    /// ```
    #[inline]
    pub fn capacity(&self) -> usize {
        self.buf.cap()
    }

    /// Returns the number of elements the gap vector currently holds.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut gap_vec: GapVec<i32> = GapVec::with_capacity(10);
    /// for i in 0..10 {
    ///     gap_vec.push(i);
    /// }
    /// gap_vec.gap = 1..3;
    /// assert_eq!(gap_vec.len(), 8);
    /// ```
    pub fn len(&self) -> usize {
        self.capacity() - self.gap.len()
    }

    /// Returns the current the gap insertion position.
    ///
    /// # Examples
    ///
    /// ```
    /// let gap_vec: GapVec<i32> = GapVec::new();
    /// assert_eq!(gap_vec.position(), 0);
    /// ```
    pub fn position(&self) -> usize {
        self.gap.start
    }
}

