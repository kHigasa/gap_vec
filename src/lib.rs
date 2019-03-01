// Copyright 2018 Koji Higasa.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! A contiguous growable array type with heap-allocated contens and gap. 
//! It's written `GapVec<T>` but pronounced 'gap vector'.
//!
//! # Examples
//!
//! You can explicitly create a `GapVec<T>` with `new`:
//!
//! ```
//! use gap_vec::GapVec;
//!
//! let mut gap_vec: GapVec<i32> = GapVec::new();
//! ```
//!
//! You can `insert` values (which will grow the gap vector as needed):
//!
//! ```
//! use gap_vec::GapVec;
//!
//! let mut gap_vec = GapVec::new();
//!
//! gap_vec.insert("onion".to_string());
//! ```
//!
//! You can `remove` values in much the same way:
//!
//! ```
//! use gap_vec::GapVec;
//!
//! let mut gap_vec = GapVec::new();
//!
//! gap_vec.insert("foo".to_string());
//! gap_vec.set_position(0);
//! assert_eq!(gap_vec.remove().unwrap(), "foo".to_string());
//! ```

#![feature(core_intrinsics, alloc, raw_vec_internals)]
extern crate alloc;

use core::intrinsics::assume;
use core::ops::{Deref, DerefMut};
use core::slice;

use alloc::raw_vec::RawVec;
use std::fmt;
use std::fmt::{Debug, Formatter};
use std::ops::Range;
use std::ptr;

/// A contiguous growable array type with heap-allocated contens and gap. 
/// It's written `GapVec<T>` but pronounced 'gap vector'.
///
/// # Examples
///
/// You can explicitly create a `GapVec<T>` with `new` :
///
/// ```
/// use gap_vec::GapVec;
///
/// let v: GapVec<i32> = GapVec::new();
/// ```
///

pub struct GapVec<T> {
    buf: RawVec<T>,
    gap: Range<usize>,
}

////////////////////////////////////////////////////////////////////////////////
// Inherent methods
////////////////////////////////////////////////////////////////////////////////

impl<T> GapVec<T> {
    /// Constructs a new, empty `GapVec<T>`.
    ///
    /// The gap vector will not allocate until elements are pushed onto it.
    ///
    /// # Examples
    ///
    /// ```
    /// # #![allow(unused_mut)]
    /// use gap_vec::GapVec;
    ///
    /// let mut gap_vec: GapVec<i32> = GapVec::new();
    /// ```
    #[inline]
    pub const fn new() -> GapVec<T> {
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
    /// use gap_vec::GapVec;
    ///
    /// let mut gap_vec = GapVec::with_capacity(10);
    ///
    /// // These are all done without reallocating
    /// for i in 0..10 {
    ///     gap_vec.insert(i);
    /// }
    ///
    /// // ,but this may make the gap vector reallocate.
    /// gap_vec.insert(10);
    ///
    /// assert!(gap_vec.capacity() >= 11);
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
    /// use gap_vec::GapVec;
    ///
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
    /// use gap_vec::GapVec;
    ///
    /// let mut gap_vec: GapVec<i32> = GapVec::with_capacity(10);
    /// assert_eq!(gap_vec.len(), 10);
    /// ```
    pub fn len(&self) -> usize {
        self.capacity() - self.gap.len()
    }

    /// Returns the current the gap insertion position.
    ///
    /// # Examples
    ///
    /// ```
    /// use gap_vec::GapVec;
    ///
    /// let gap_vec: GapVec<i32> = GapVec::new();
    /// assert_eq!(gap_vec.position(), 0);
    /// ```
    #[inline]
    pub fn position(&self) -> usize {
        self.gap.start
    }

    /// Returns a reference to the `index`'th element,
    /// or `None` if `index` is out of bounds.
    pub fn get(&self, index: usize) -> Option<&T> {
        let raw = self.index_to_raw(index);
        if raw < self.capacity() {
            unsafe {
                Some(&*self.space(raw))
            }
        } else {
            None
        }
    }

    /// Sets the current insertion position to `pos`.
    ///
    /// # Panics
    ///
    /// Panics if `pos > len`.
    pub fn set_position(&mut self, pos: usize) {
        if pos > self.len() {
            panic!("index {} out of range for GapVec buffer", pos);
        }

        unsafe {
            let gap = self.gap.clone();
            if pos > gap.start {
                let distance = pos - gap.start;
                ptr::copy(self.space(pos), self.space_mut(gap.start), distance);
            } else if pos < gap.start {
                let distance = gap.start - pos;
                ptr::copy(self.space(pos), self.space_mut(gap.end - distance), distance);
            }

            self.gap = pos .. pos + gap.len();
        }
    }

    /// Inserts an element at gap start within the gap vector.
    ///
    /// # Examples
    ///
    /// ```
    /// use gap_vec::GapVec;
    ///
    /// let mut gap_vec = GapVec::new();
    /// gap_vec.insert("foo".to_string());
    /// gap_vec.set_position(0);
    /// assert_eq!(gap_vec.remove().unwrap(), "foo".to_string());
    /// ```
    pub fn insert(&mut self, element: T) {
        if self.gap.len() == 0 {
            self.enlarge_gap();
        }

        unsafe {
            let index = self.gap.start;
            ptr::write(self.space_mut(index), element);
        }

        self.gap.start += 1;
    }

    /// Inserts the elements produced by `iter` at the current insertion
    /// position, and leave the insertion position after them.
    ///
    /// # Examples
    ///
    /// ```
    /// use gap_vec::GapVec;
    ///
    /// let mut gap_vec: GapVec<char> = GapVec::new();
    /// gap_vec.insert_iter("Foo bar baz qux quux.".chars());
    /// assert_eq!(gap_vec.get_string(), "Foo bar baz qux quux.");
    /// ```
    pub fn insert_iter<I>(&mut self, iterable: I)
        where I: IntoIterator<Item=T>
    {
        for item in iterable {
            self.insert(item);
        }
    }

    /// Removes and returns the element at gap end within the gap vector,
    ///
    /// # Examples
    ///
    /// ```
    /// use gap_vec::GapVec;
    ///
    /// let mut gap_vec: GapVec<i32> = GapVec::new();
    /// gap_vec.insert(3);
    /// gap_vec.set_position(0);
    /// assert_eq!(gap_vec.remove().unwrap(), 3);
    /// ```
    pub fn remove(&mut self) -> Option<T> {
        if self.gap.end == self.capacity() {
            return None;
        }

        let element = unsafe {
            ptr::read(self.space(self.gap.end))
        };
        self.gap.end += 1;
        Some(element)
    }

    // Returns the offset in the buffer of the `index`'th element, taking
    // the gap into account. This does not check whether index is in range,
    // but it never returns the index of space in the gap.
    fn index_to_raw(&self, index: usize) -> usize {
        if index < self.gap.start {
            index
        } else {
            index + self.gap.len()
        }
    }

    // Doubles the capacity of `self.buf`.
    fn enlarge_gap(&mut self) {
        let mut new_capacity = self.capacity() * 2;

        if new_capacity == 0 {
            new_capacity = 4;
        }

        let new_buf = RawVec::with_capacity(new_capacity);
        let after_gap = self.capacity() - self.gap.end;
        let new_gap = self.gap.start .. new_buf.cap() - after_gap;

        unsafe {
            // Copy buf before gap from self to new.
            ptr::copy_nonoverlapping(self.space(0),
                                     new_buf.ptr(),
                                     self.gap.start);
            // Copy buf after gap from self to new.
            ptr::copy_nonoverlapping(self.space(self.gap.end),
                                     new_buf.ptr().offset(new_gap.end as isize),
                                     after_gap);
        }

        self.buf = new_buf;
        self.gap = new_gap;
    }

    // Returns a pointer to the `index`'th element of the underlying buf,
    // as if the gap were not there.
    //
    // Safety: `index` must be less than self.capacity().
    unsafe fn space(&self, index: usize) -> *const T {
        self.as_ptr().offset(index as isize)
    }

    // Returns a mutable pointer to the `index`'th element of the underlying buf,
    // as if the gap were not there.
    //
    // Safety: `index` must be less than self.capacity().
    unsafe fn space_mut(&mut self, index: usize) -> *mut T {
        self.as_mut_ptr().offset(index as isize)
    }
}

impl GapVec<char> {
    pub fn get_string(&self) -> String {
        let mut text = String::new();
        text.extend(self);
        text
    }
}

////////////////////////////////////////////////////////////////////////////////
// Common trait implementations for Vec
////////////////////////////////////////////////////////////////////////////////
impl<T: fmt::Debug> Debug for GapVec<T> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let indeces = (0..self.gap.start).chain(self.gap.end..self.capacity());
        let elements = indeces.map(|i| unsafe { &*self.space(i) });
        f.debug_list().entries(elements).finish()
    }
}

impl<T> Deref for GapVec<T> {
    type Target = [T];

    fn deref(&self) -> &[T] {
        unsafe {
            let ptr = self.buf.ptr();
            assume(!ptr.is_null());
            slice::from_raw_parts(ptr, self.len())
        }
    }
}

impl<T> DerefMut for GapVec<T> {
    fn deref_mut(&mut self) -> &mut [T] {
        unsafe {
            let ptr = self.buf.ptr();
            assume(!ptr.is_null());
            slice::from_raw_parts_mut(ptr, self.len())
        }
    }
}

impl<T> Drop for GapVec<T> {
    fn drop(&mut self) {
        unsafe {
            for i in 0 .. self.gap.start {
                ptr::drop_in_place(self.space_mut(i));
            }
            for i in self.gap.end .. self.capacity() {
                ptr::drop_in_place(self.space_mut(i));
            }
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
// Iterator
////////////////////////////////////////////////////////////////////////////////

/// An iterator for `GapVec<T>`.

pub struct Iter<'a, T: 'a> {
    buf: &'a GapVec<T>,
    pos: usize
}

impl<'a, T: 'a> Iterator for Iter<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<&'a T> {
        if self.pos >= self.buf.len() {
            None
        } else {
            self.pos += 1;
            self.buf.get(self.pos - 1)
        }
    }
}

impl<'a, T: 'a> IntoIterator for &'a GapVec<T> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;
    fn into_iter(self) -> Iter<'a, T> {
        Iter { buf: self, pos: 0 }
    }
}

