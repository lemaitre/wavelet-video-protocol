#![allow(clippy::missing_safety_doc)]

use std::{
    iter::FusedIterator,
    marker::PhantomData,
    ops::{Deref, DerefMut},
    ptr::NonNull,
};

use super::ptr_offset;

/// Pointer to a slice were elements are not contiguous, but evenly spaced nonetheless.
#[derive(Debug)]
pub struct StridedSlicePtr<T> {
    ptr: NonNull<T>,
    stride: isize,
    len: usize,
}

impl<T> StridedSlicePtr<T> {
    /// Construct a strided slice pointer starting at `ptr`,
    /// with `len` elements spaced by `stride` bytes.
    ///
    /// # SAFETY
    ///
    /// `ptr + i * stride` should be a valid location for an object of type `T` for all `i` in `0..len`.
    pub unsafe fn from_raw_parts(ptr: *const T, stride: isize, len: usize) -> Self {
        Self {
            ptr: NonNull::new_unchecked(ptr as *mut T),
            stride,
            len,
        }
    }

    /// Stride in bytes between objects of the slice.
    pub fn stride(&self) -> isize {
        self.stride
    }
    /// Pointer of the first element of the slice.
    pub fn as_non_null_ptr(&self) -> NonNull<T> {
        self.ptr
    }

    /// Interpret the current strided slice pointer into a strided slice reference
    ///
    /// # SAFETY
    ///
    /// All elements of the slice must properly initialized.
    /// No mutable reference should be active on any of the elements.
    pub unsafe fn as_strided_slice<'a>(&self) -> StridedSlice<'a, T> {
        StridedSlice(*self, PhantomData)
    }
    /// Interpret the current strided slice pointer into a mutable strided slice reference
    ///
    /// # SAFETY
    ///
    /// All elements of the slice must properly initialized.
    /// No reference (mutable or shared) should be active on any of the elements.
    pub unsafe fn as_strided_slice_mut<'a>(&self) -> StridedSliceMut<'a, T> {
        StridedSliceMut(*self, PhantomData)
    }

    /// Get the pointer of the `i`-th element of the slice.
    ///
    /// # SAFETY
    ///
    /// `i < self.len()`
    pub unsafe fn unchecked_get(&self, i: usize) -> NonNull<T> {
        unsafe { ptr_offset(self.ptr, i, self.stride) }
    }

    /// Get the pointer of the `i`-th element of the slice.
    /// Return `None` if `i` is out of bound.
    pub fn checked_get(&self, i: usize) -> Option<NonNull<T>> {
        if i < self.len {
            Some(unsafe { self.unchecked_get(i) })
        } else {
            None
        }
    }
    /// Get the pointer of the `i`-th element of the slice.
    /// Panic if `i` is out of bound.
    pub fn get(&self, i: usize) -> NonNull<T> {
        if i < self.len {
            unsafe { self.unchecked_get(i) }
        } else {
            panic!(
                "Trying to access element #{} from a slice with {} elements",
                i, self.len
            )
        }
    }

    /// Create a subslice of the current slice.
    ///
    /// The resulting subslice is functionally equivalent to to following iterator:
    ///
    /// ```rust
    /// if step > 0 {
    ///     slice.skip(start).step_by(step).take(len)
    /// } else if step < 0 {
    ///     slice.skip(start).rev().step_by(-step).take(len)
    /// } else {
    ///     slice.skip(start).take(1).cycle().take(len)
    /// }
    /// ```
    pub fn subslice(&self, start: usize, len: usize, step: isize) -> Self {
        unsafe {
            let start = start.min(self.len());
            let i = if step >= 0 {
                start
            } else {
                self.len().saturating_sub(1)
            };

            // SAFETY: i is in 0..self.len
            let ptr = ptr_offset(self.as_non_null_ptr(), i, self.stride());

            match std::num::NonZero::try_from(step) {
                Ok(step) => {
                    let mut available_len = self.len().saturating_sub(start);
                    if available_len > 0 {
                        available_len = available_len
                            .unchecked_sub(1)
                            .div_euclid(step.get().unsigned_abs())
                            .unchecked_add(1);
                    }
                    let len = len.min(available_len);
                    let stride = step.get().saturating_mul(self.stride());
                    Self::from_raw_parts(ptr.as_ptr(), stride, len)
                }
                Err(_) => Self::from_raw_parts(ptr.as_ptr(), 0, if start < len { len } else { 0 }),
            }
        }
    }

    pub unsafe fn split_at_unchecked(&self, i: usize) -> (Self, Self) {
        (
            Self::from_raw_parts(self.as_non_null_ptr().as_ptr(), self.stride(), i),
            Self::from_raw_parts(
                self.unchecked_get(i).as_ptr(),
                self.stride(),
                self.len().unchecked_sub(i),
            ),
        )
    }
    pub fn split_at_checked(&self, i: usize) -> Option<(Self, Self)> {
        if i <= self.len() {
            Some(unsafe { self.split_at_unchecked(i) })
        } else {
            None
        }
    }
    pub fn split_at(&self, i: usize) -> (Self, Self) {
        if i <= self.len() {
            unsafe { self.split_at_unchecked(i) }
        } else {
            panic!("Cannot split at {i} a slice with {} elements", self.len())
        }
    }

    pub fn deinterleave(&self) -> (Self, Self) {
        (
            self.subslice(0, self.len(), 2),
            self.subslice(1, self.len(), 2),
        )
    }
}

impl<T> Default for StridedSlicePtr<T> {
    fn default() -> Self {
        Self {
            ptr: NonNull::dangling(),
            stride: Default::default(),
            len: Default::default(),
        }
    }
}

impl<T> Clone for StridedSlicePtr<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for StridedSlicePtr<T> {}

impl<T> From<StridedSlice<'_, T>> for StridedSlicePtr<T> {
    fn from(value: StridedSlice<'_, T>) -> Self {
        value.0
    }
}
impl<T> From<StridedSliceMut<'_, T>> for StridedSlicePtr<T> {
    fn from(value: StridedSliceMut<'_, T>) -> Self {
        value.0
    }
}
impl<T> From<SlicePtr<T>> for StridedSlicePtr<T> {
    fn from(value: SlicePtr<T>) -> Self {
        Self {
            ptr: value.as_non_null_ptr(),
            stride: std::mem::size_of::<T>() as isize,
            len: value.len(),
        }
    }
}
impl<T> From<NonNull<[T]>> for StridedSlicePtr<T> {
    fn from(value: NonNull<[T]>) -> Self {
        Self::from(SlicePtr::from(value))
    }
}
impl<T> From<&'_ [T]> for StridedSlicePtr<T> {
    fn from(value: &[T]) -> Self {
        Self::from(SlicePtr::from(value))
    }
}
impl<T> From<&'_ mut [T]> for StridedSlicePtr<T> {
    fn from(value: &mut [T]) -> Self {
        Self::from(SlicePtr::from(value))
    }
}

impl<T> PartialEq for StridedSlicePtr<T> {
    fn eq(&self, other: &Self) -> bool {
        self.ptr == other.ptr && self.stride == other.stride && self.len == other.len
    }
}
impl<T> Eq for StridedSlicePtr<T> {}

impl<T> Iterator for StridedSlicePtr<T> {
    type Item = NonNull<T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.len > 0 {
            unsafe {
                let ptr = self.ptr;
                // SAFETY: if this is the last element, the new ptr will be outside of the allocation
                // but will never be dereferenced
                self.ptr = ptr_offset(ptr, 1, self.stride);
                self.len -= 1;
                Some(ptr)
            }
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }

    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.len
    }

    fn last(mut self) -> Option<Self::Item>
    where
        Self: Sized,
    {
        self.next_back()
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        // SAFETY: if `n` is larger than `len`, pointer will be moved outside of the allocation
        // but length would also be set to 0 ensuring the pointer will never be dereferenced.
        unsafe {
            let n = n.min(self.len);
            self.ptr = ptr_offset(self.ptr, n, self.stride);
            self.len = self.len.unchecked_sub(n);

            self.next()
        }
    }
}

impl<T> ExactSizeIterator for StridedSlicePtr<T> {
    fn len(&self) -> usize {
        self.len
    }
}
impl<T> FusedIterator for StridedSlicePtr<T> {}
impl<T> DoubleEndedIterator for StridedSlicePtr<T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.len > 0 {
            // SAFETY: as length is > 0, len - 1 is within bounds
            unsafe {
                let len = self.len.unchecked_sub(1);
                let ptr = ptr_offset(self.ptr, len, self.stride);
                self.len = len;
                Some(ptr)
            }
        } else {
            None
        }
    }
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        self.len = self.len.saturating_sub(n);
        self.next_back()
    }
}

/// Shared reference to a slice were elements are not contiguous, but evenly spaced nonetheless.
#[derive(Debug)]
pub struct StridedSlice<'a, T>(StridedSlicePtr<T>, PhantomData<&'a [T]>);

impl<'a, T> StridedSlice<'a, T> {
    /// Construct a strided slice starting at `ptr`,
    /// with `len` elements spaced by `stride` bytes.
    ///
    /// # SAFETY
    ///
    /// `ptr + i * stride` should be a valid location for an object of type `T` for all `i` in `0..len`.
    /// All elements of the slice must properly initialized.
    /// No mutable reference should be active on any of the elements.
    pub unsafe fn from_raw_parts(ptr: *const T, stride: isize, len: usize) -> Self {
        unsafe { StridedSlicePtr::from_raw_parts(ptr, stride, len).as_strided_slice() }
    }

    /// Stride in bytes between objects of the slice.
    pub fn stride(&self) -> isize {
        self.0.stride()
    }
    /// Pointer of the first element of the slice.
    pub fn as_non_null_ptr(&self) -> NonNull<T> {
        self.0.as_non_null_ptr()
    }
    /// Convert the current mutable slice into a shared slice.
    pub fn into_strided_slice(self) -> StridedSlice<'a, T> {
        // SAFETY: Consume the mutable slice, and keep its lifetime
        unsafe { self.0.as_strided_slice() }
    }
    /// Create a borrowed shared slice from the current slice.
    pub fn as_strided_slice(&mut self) -> StridedSlice<'_, T> {
        // SAFETY: output borrows the input value
        unsafe { self.0.as_strided_slice() }
    }

    /// Get a shared reference to the `i`-th element of the slice.
    ///
    /// # SAFETY
    ///
    /// `i < self.len()`
    pub unsafe fn unchecked_get(&self, i: usize) -> &T {
        // SAFETY: [`Self::according to from_raw_parts`],
        // there could only be StridedSlice that reference the elements
        unsafe { self.0.unchecked_get(i).as_ref() }
    }
    /// Get a shared reference to the `i`-th element of the slice.
    /// Return `None` if `i` is out of bound.
    pub fn checked_get(&self, i: usize) -> Option<&T> {
        // SAFETY: [`Self::according to from_raw_parts`],
        // there could only be StridedSlice that reference the elements
        self.0.checked_get(i).map(|value| unsafe { value.as_ref() })
    }
    /// Get a shared reference to the `i`-th element of the slice.
    /// Panic if `i` is out of bound.
    pub fn get(&self, i: usize) -> &T {
        // SAFETY: [`Self::according to from_raw_parts`],
        // there could only be StridedSlice that reference the elements
        unsafe { self.0.get(i).as_ref() }
    }

    /// Transform the current slice into a shared subslice.
    ///
    /// The resulting subslice is functionally equivalent to to following iterator:
    ///
    /// ```rust
    /// if step > 0 {
    ///     slice.skip(start).step_by(step).take(len)
    /// } else if step < 0 {
    ///     slice.skip(start).rev().step_by(-step).take(len)
    /// } else {
    ///     slice.skip(start).take(1).cycle().take(len)
    /// }
    /// ```
    pub fn into_subslice(self, start: usize, len: usize, step: isize) -> StridedSlice<'a, T> {
        unsafe { self.0.subslice(start, len, step).as_strided_slice() }
    }
    /// Create a shared subsliced borrowed from the current slice.
    ///
    /// The resulting subslice is functionally equivalent to to following iterator:
    ///
    /// ```rust
    /// if step > 0 {
    ///     slice.skip(start).step_by(step).take(len)
    /// } else if step < 0 {
    ///     slice.skip(start).rev().step_by(-step).take(len)
    /// } else {
    ///     slice.skip(start).take(1).cycle().take(len)
    /// }
    /// ```
    pub fn subslice(&self, start: usize, len: usize, step: isize) -> StridedSlice<'_, T> {
        unsafe { self.0.subslice(start, len, step).as_strided_slice() }
    }

    pub unsafe fn split_at_unchecked(
        &self,
        i: usize,
    ) -> (StridedSlice<'_, T>, StridedSlice<'_, T>) {
        unsafe {
            let (a, b) = self.0.split_at_unchecked(i);
            (a.as_strided_slice(), b.as_strided_slice())
        }
    }
    pub fn split_at_checked(&self, i: usize) -> Option<(StridedSlice<'_, T>, StridedSlice<'_, T>)> {
        self.0
            .split_at_checked(i)
            .map(|(a, b)| unsafe { (a.as_strided_slice(), b.as_strided_slice()) })
    }
    pub fn split_at(&self, i: usize) -> (StridedSlice<'_, T>, StridedSlice<'_, T>) {
        unsafe {
            let (a, b) = self.0.split_at_unchecked(i);
            (a.as_strided_slice(), b.as_strided_slice())
        }
    }
    pub fn deinterleave(&self) -> (StridedSlice<'_, T>, StridedSlice<'_, T>) {
        unsafe {
            let (a, b) = self.0.deinterleave();
            (a.as_strided_slice(), b.as_strided_slice())
        }
    }
}

// SAFETY: StridedSlice has a semantic of reference
unsafe impl<T: Sync> Send for StridedSlice<'_, T> {}
unsafe impl<T: Sync> Sync for StridedSlice<'_, T> {}

impl<T> Default for StridedSlice<'_, T> {
    fn default() -> Self {
        unsafe { StridedSlicePtr::default().as_strided_slice() }
    }
}

impl<T> Clone for StridedSlice<'_, T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T> Copy for StridedSlice<'_, T> {}

impl<'a, T> PartialEq<StridedSlice<'a, T>> for StridedSlice<'_, T> {
    fn eq(&self, other: &StridedSlice<'a, T>) -> bool {
        self.0 == other.0
    }
}
impl<T> Eq for StridedSlice<'_, T> {}
impl<'a, T> PartialEq<StridedSliceMut<'a, T>> for StridedSlice<'_, T> {
    fn eq(&self, other: &StridedSliceMut<'a, T>) -> bool {
        self.0 == other.0
    }
}

impl<T> std::ops::Index<usize> for StridedSlice<'_, T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        self.get(index)
    }
}

impl<'a, T> From<&'a mut [T]> for StridedSlice<'a, T> {
    fn from(value: &'a mut [T]) -> Self {
        // SAFETY: output borrows the input value
        unsafe { StridedSlicePtr::from(value).as_strided_slice() }
    }
}
impl<'a, T> From<&'a [T]> for StridedSlice<'a, T> {
    fn from(value: &'a [T]) -> Self {
        // SAFETY: output borrows the input value
        unsafe { StridedSlicePtr::from(value).as_strided_slice() }
    }
}
impl<'a, T> From<StridedSliceMut<'a, T>> for StridedSlice<'a, T> {
    fn from(value: StridedSliceMut<'a, T>) -> Self {
        // SAFETY: Consume the mutable slice, and keep its lifetime
        unsafe { value.0.as_strided_slice() }
    }
}

impl<'a, T> Iterator for StridedSlice<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        // SAFETY: [`Self::according to from_raw_parts`],
        // there could only be StridedSlice that reference the elements
        self.0.next().map(|item| unsafe { item.as_ref() })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }

    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.0.count()
    }

    fn last(self) -> Option<Self::Item>
    where
        Self: Sized,
    {
        // SAFETY: [`Self::according to from_raw_parts`],
        // there could only be StridedSlice that reference the elements
        self.0.last().map(|item| unsafe { item.as_ref() })
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        // SAFETY: [`Self::according to from_raw_parts`],
        // there could only be StridedSlice that reference the elements
        self.0.nth(n).map(|item| unsafe { item.as_ref() })
    }
}

impl<T> ExactSizeIterator for StridedSlice<'_, T> {
    fn len(&self) -> usize {
        self.0.len()
    }
}
impl<T> FusedIterator for StridedSlice<'_, T> {}
impl<T> DoubleEndedIterator for StridedSlice<'_, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        // SAFETY: [`Self::according to from_raw_parts`],
        // there could only be StridedSlice that reference the elements
        self.0.next_back().map(|item| unsafe { item.as_ref() })
    }
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        // SAFETY: [`Self::according to from_raw_parts`],
        // there could only be StridedSlice that reference the elements
        self.0.nth_back(n).map(|item| unsafe { item.as_ref() })
    }
}

/// Mutable reference to a slice were elements are not contiguous, but evenly spaced nonetheless.
#[derive(Debug)]
pub struct StridedSliceMut<'a, T>(StridedSlicePtr<T>, PhantomData<&'a mut [T]>);

impl<'a, T> StridedSliceMut<'a, T> {
    /// Construct a mutable strided slice starting at `ptr`,
    /// with `len` elements spaced by `stride` bytes
    ///
    /// # SAFETY
    ///
    /// `ptr + i * stride` should be a valid location for an object of type `T` for all `i` in `0..len`.
    /// All elements of the slice must properly initialized.
    /// No reference (mutable or shared) should be active on any of the elements.
    pub unsafe fn from_raw_parts(ptr: *mut T, stride: isize, len: usize) -> Self {
        unsafe { StridedSlicePtr::from_raw_parts(ptr, stride, len).as_strided_slice_mut() }
    }

    /// Stride in bytes between objects of the slice.
    pub fn stride(&self) -> isize {
        self.0.stride()
    }
    /// Pointer of the first element of the slice.
    pub fn as_non_null_ptr(&self) -> NonNull<T> {
        self.0.as_non_null_ptr()
    }
    /// Convert the current mutable slice into a shared slice.
    pub fn into_strided_slice(self) -> StridedSlice<'a, T> {
        // SAFETY: Consume the mutable slice, and keep its lifetime
        unsafe { self.0.as_strided_slice() }
    }
    /// Create a borrowed shared slice from the current slice.
    pub fn as_strided_slice(&mut self) -> StridedSlice<'_, T> {
        // SAFETY: output borrows the input value
        unsafe { self.0.as_strided_slice() }
    }
    /// Convert the current mutable slice into a shared slice.
    pub fn into_strided_slice_mut(self) -> StridedSliceMut<'a, T> {
        // SAFETY: Consume the mutable slice, and keep its lifetime
        unsafe { self.0.as_strided_slice_mut() }
    }
    /// Create a borrowed shared slice from the current slice.
    pub fn as_strided_slice_mut(&mut self) -> StridedSliceMut<'_, T> {
        // SAFETY: output borrows the input value
        unsafe { self.0.as_strided_slice_mut() }
    }

    /// Get a shared reference to the `i`-th element of the slice.
    ///
    /// # SAFETY
    ///
    /// `i < self.len()`
    pub unsafe fn unchecked_get(&self, i: usize) -> &T {
        // SAFETY: [`Self::according to from_raw_parts`],
        // self is the only active slice onto the elements
        unsafe { self.0.unchecked_get(i).as_ref() }
    }
    /// Get a shared reference to the `i`-th element of the slice.
    /// Return `None` if `i` is out of bound.
    pub fn checked_get(&self, i: usize) -> Option<&T> {
        // SAFETY: [`Self::according to from_raw_parts`],
        // self is the only active slice onto the elements
        self.0.checked_get(i).map(|value| unsafe { value.as_ref() })
    }
    /// Get a shared reference to the `i`-th element of the slice.
    /// Panic if `i` is out of bound.
    pub fn get(&self, i: usize) -> &T {
        // SAFETY: [`Self::according to from_raw_parts`],
        // self is the only active slice onto the elements
        unsafe { self.0.get(i).as_ref() }
    }

    /// Get a mutable reference to the `i`-th element of the slice.
    ///
    /// # SAFETY
    ///
    /// `i < self.len()`
    pub unsafe fn unchecked_get_mut(&mut self, i: usize) -> &mut T {
        // SAFETY: [`Self::according to from_raw_parts`],
        // self is the only active slice onto the elements.
        // Moreover, as the output reference borrows from self,
        // No other references could be created as long as the borrow is active.
        unsafe { self.0.unchecked_get(i).as_mut() }
    }
    /// Get a mutable reference to the `i`-th element of the slice.
    /// Return `None` if `i` is out of bound.
    pub fn checked_get_mut(&mut self, i: usize) -> Option<&mut T> {
        // SAFETY: [`Self::according to from_raw_parts`],
        // self is the only active slice onto the elements.
        // Moreover, as the output reference borrows from self,
        // No other references could be created as long as the borrow is active.
        self.0
            .checked_get(i)
            .map(|mut value| unsafe { value.as_mut() })
    }
    /// Get a mutable reference to the `i`-th element of the slice.
    /// Panic if `i` is out of bound.
    pub fn get_mut(&mut self, i: usize) -> &mut T {
        // SAFETY: [`Self::according to from_raw_parts`],
        // self is the only active slice onto the elements.
        // Moreover, as the output reference borrows from self,
        // No other references could be created as long as the borrow is active.
        unsafe { self.0.get(i).as_mut() }
    }

    /// Transform the current slice into a shared subslice.
    ///
    /// The resulting subslice is functionally equivalent to to following iterator:
    ///
    /// ```rust
    /// if step > 0 {
    ///     slice.skip(start).step_by(step).take(len)
    /// } else if step < 0 {
    ///     slice.skip(start).rev().step_by(-step).take(len)
    /// } else {
    ///     slice.skip(start).take(1).cycle().take(len)
    /// }
    /// ```
    pub fn into_subslice(self, start: usize, len: usize, step: isize) -> StridedSlice<'a, T> {
        // SAFETY: Consume the mutable slice, and keep its lifetime
        unsafe { self.0.subslice(start, len, step).as_strided_slice() }
    }
    /// Create a shared subsliced borrowed from the current slice.
    ///
    /// The resulting subslice is functionally equivalent to to following iterator:
    ///
    /// ```rust
    /// if step > 0 {
    ///     slice.skip(start).step_by(step).take(len)
    /// } else if step < 0 {
    ///     slice.skip(start).rev().step_by(-step).take(len)
    /// } else {
    ///     slice.skip(start).take(1).cycle().take(len)
    /// }
    /// ```
    pub fn subslice(&self, start: usize, len: usize, step: isize) -> StridedSlice<'_, T> {
        // SAFETY: [`Self::according to from_raw_parts`],
        // self is the only active slice onto the elements
        unsafe { self.0.subslice(start, len, step).as_strided_slice() }
    }
    /// Transform the current slice into a mutable subslice.
    ///
    /// The resulting subslice is functionally equivalent to to following iterator:
    ///
    /// ```rust
    /// if step > 0 {
    ///     slice.skip(start).step_by(step).take(len)
    /// } else if step < 0 {
    ///     slice.skip(start).rev().step_by(-step).take(len)
    /// } else {
    ///     slice.skip(start).take(1).cycle().take(len)
    /// }
    /// ```
    pub fn into_subslice_mut(
        self,
        start: usize,
        len: usize,
        step: isize,
    ) -> StridedSliceMut<'a, T> {
        // SAFETY: Consume the mutable slice, and keep its lifetime
        unsafe { self.0.subslice(start, len, step).as_strided_slice_mut() }
    }
    /// Create a mutable subsliced borrowed from the current slice.
    ///
    /// The resulting subslice is functionally equivalent to to following iterator:
    ///
    /// ```rust
    /// if step > 0 {
    ///     slice.skip(start).step_by(step).take(len)
    /// } else if step < 0 {
    ///     slice.skip(start).rev().step_by(-step).take(len)
    /// } else {
    ///     slice.skip(start).take(1).cycle().take(len)
    /// }
    /// ```
    pub fn subslice_mut(&self, start: usize, len: usize, step: isize) -> StridedSliceMut<'_, T> {
        // SAFETY: [`Self::according to from_raw_parts`],
        // self is the only active slice onto the elements.
        // Moreover, as the output reference borrows from self,
        // No other references could be created as long as the borrow is active.
        unsafe { self.0.subslice(start, len, step).as_strided_slice_mut() }
    }

    pub unsafe fn split_at_unchecked(
        &mut self,
        i: usize,
    ) -> (StridedSliceMut<'_, T>, StridedSliceMut<'_, T>) {
        unsafe {
            let (a, b) = self.0.split_at_unchecked(i);
            (a.as_strided_slice_mut(), b.as_strided_slice_mut())
        }
    }
    pub fn split_at_checked(
        &mut self,
        i: usize,
    ) -> Option<(StridedSliceMut<'_, T>, StridedSliceMut<'_, T>)> {
        self.0
            .split_at_checked(i)
            .map(|(a, b)| unsafe { (a.as_strided_slice_mut(), b.as_strided_slice_mut()) })
    }
    pub fn split_at(&mut self, i: usize) -> (StridedSliceMut<'_, T>, StridedSliceMut<'_, T>) {
        unsafe {
            let (a, b) = self.0.split_at_unchecked(i);
            (a.as_strided_slice_mut(), b.as_strided_slice_mut())
        }
    }
    pub fn deinterleave(&mut self) -> (StridedSliceMut<'_, T>, StridedSliceMut<'_, T>) {
        unsafe {
            let (a, b) = self.0.deinterleave();
            (a.as_strided_slice_mut(), b.as_strided_slice_mut())
        }
    }
}

// SAFETY: StridedSlice has a semantic of reference
unsafe impl<T: Sync> Send for StridedSliceMut<'_, T> {}
unsafe impl<T: Sync> Sync for StridedSliceMut<'_, T> {}

impl<'a, T> PartialEq<StridedSliceMut<'a, T>> for StridedSliceMut<'_, T> {
    fn eq(&self, other: &StridedSliceMut<'a, T>) -> bool {
        self.0 == other.0
    }
}
impl<T> Eq for StridedSliceMut<'_, T> {}
impl<'a, T> PartialEq<StridedSlice<'a, T>> for StridedSliceMut<'_, T> {
    fn eq(&self, other: &StridedSlice<'a, T>) -> bool {
        self.0 == other.0
    }
}

impl<T> std::ops::Index<usize> for StridedSliceMut<'_, T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        self.get(index)
    }
}
impl<T> std::ops::IndexMut<usize> for StridedSliceMut<'_, T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.get_mut(index)
    }
}

impl<'a, T> From<&'a mut [T]> for StridedSliceMut<'a, T> {
    fn from(value: &'a mut [T]) -> Self {
        // SAFETY: output borrows the input value
        unsafe { StridedSlicePtr::from(value).as_strided_slice_mut() }
    }
}

impl<'a, T> Iterator for StridedSliceMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        // SAFETY: [`Self::according to from_raw_parts`],
        // self is the only active slice onto the elements.
        // Moreover, as the output reference borrows from self,
        // No other references could be created as long as the borrow is active.
        self.0.next().map(|mut item| unsafe { item.as_mut() })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }

    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.0.count()
    }

    fn last(self) -> Option<Self::Item>
    where
        Self: Sized,
    {
        // SAFETY: [`Self::according to from_raw_parts`],
        // self is the only active slice onto the elements.
        // Moreover, as the output reference borrows from self,
        // No other references could be created as long as the borrow is active.
        self.0.last().map(|mut item| unsafe { item.as_mut() })
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        // SAFETY: [`Self::according to from_raw_parts`],
        // self is the only active slice onto the elements.
        // Moreover, as the output reference borrows from self,
        // No other references could be created as long as the borrow is active.
        self.0.nth(n).map(|mut item| unsafe { item.as_mut() })
    }
}

impl<T> ExactSizeIterator for StridedSliceMut<'_, T> {
    fn len(&self) -> usize {
        self.0.len()
    }
}
impl<T> FusedIterator for StridedSliceMut<'_, T> {}
impl<T> DoubleEndedIterator for StridedSliceMut<'_, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        // SAFETY: [`Self::according to from_raw_parts`],
        // self is the only active slice onto the elements.
        // Moreover, as the output reference borrows from self,
        // No other references could be created as long as the borrow is active.
        self.0.next_back().map(|mut item| unsafe { item.as_mut() })
    }
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        // SAFETY: [`Self::according to from_raw_parts`],
        // self is the only active slice onto the elements.
        // Moreover, as the output reference borrows from self,
        // No other references could be created as long as the borrow is active.
        self.0.nth_back(n).map(|mut item| unsafe { item.as_mut() })
    }
}

/// Pointer to a slice. Behave like a [`NonNull<[T]>`], but is also an iterator.
pub struct SlicePtr<T>(NonNull<[T]>);

impl<T> SlicePtr<T> {
    pub unsafe fn from_raw_parts(ptr: NonNull<T>, len: usize) -> Self {
        Self(NonNull::slice_from_raw_parts(ptr, len))
    }

    pub fn as_non_null_ptr(&self) -> NonNull<T> {
        unsafe { NonNull::new_unchecked(self.0.as_ptr() as *mut T) }
    }

    pub fn as_mut_ptr(&self) -> *mut T {
        self.as_non_null_ptr().as_ptr()
    }

    pub unsafe fn unchecked_get(&self, i: usize) -> NonNull<T> {
        unsafe { self.as_non_null_ptr().add(i) }
    }
    pub fn checked_get(&self, i: usize) -> Option<NonNull<T>> {
        if i < self.len() {
            Some(unsafe { self.unchecked_get(i) })
        } else {
            None
        }
    }
    pub fn get(&self, i: usize) -> NonNull<T> {
        if i < self.len() {
            unsafe { self.unchecked_get(i) }
        } else {
            panic!(
                "Trying to access element #{} from a slice with {} elements",
                i,
                self.len()
            )
        }
    }
}

impl<T> Copy for SlicePtr<T> {}

impl<T> Clone for SlicePtr<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Default for SlicePtr<T> {
    fn default() -> Self {
        unsafe { Self::from_raw_parts(NonNull::dangling(), 0) }
    }
}

impl<T> Deref for SlicePtr<T> {
    type Target = NonNull<[T]>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<T> DerefMut for SlicePtr<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> From<NonNull<[T]>> for SlicePtr<T> {
    fn from(value: NonNull<[T]>) -> Self {
        Self(value)
    }
}
impl<T> From<&'_ [T]> for SlicePtr<T> {
    fn from(value: &[T]) -> Self {
        Self(value.into())
    }
}
impl<T> From<&'_ mut [T]> for SlicePtr<T> {
    fn from(value: &mut [T]) -> Self {
        Self(value.into())
    }
}
impl<T> From<SlicePtr<T>> for NonNull<[T]> {
    fn from(value: SlicePtr<T>) -> Self {
        value.0
    }
}

impl<T> PartialEq for SlicePtr<T> {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.0.as_ptr(), other.0.as_ptr())
    }
}
impl<T> Eq for SlicePtr<T> {}

impl<T> Iterator for SlicePtr<T> {
    type Item = NonNull<T>;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.0.is_empty() {
            unsafe {
                let ptr = self.as_non_null_ptr();
                *self = Self::from_raw_parts(ptr.add(1), self.len() - 1);
                Some(ptr)
            }
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len(), Some(self.len()))
    }

    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.len()
    }

    fn last(mut self) -> Option<Self::Item>
    where
        Self: Sized,
    {
        self.next_back()
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        unsafe {
            let n = n.min(self.len());
            *self =
                Self::from_raw_parts(self.as_non_null_ptr().add(n), self.len().unchecked_sub(n));

            self.next()
        }
    }
}

impl<T> ExactSizeIterator for SlicePtr<T> {
    fn len(&self) -> usize {
        self.0.len()
    }
}
impl<T> FusedIterator for SlicePtr<T> {}
impl<T> DoubleEndedIterator for SlicePtr<T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if !self.0.is_empty() {
            unsafe {
                let len = self.len().unchecked_sub(1);
                let ptr = self.as_non_null_ptr().add(len);

                *self = Self::from_raw_parts(self.as_non_null_ptr(), len);
                Some(ptr)
            }
        } else {
            None
        }
    }
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        unsafe {
            *self = Self::from_raw_parts(self.as_non_null_ptr(), self.len().saturating_sub(n));
        }
        self.next_back()
    }
}

#[cfg(test)]
mod test {

    use super::{SlicePtr, StridedSlice};

    const SLICES: &[&[i32]] = &[
        &[],
        &[0],
        &[0, 1],
        &[0, 1, 2],
        &[0, 1, 2, 3],
        &[0, 1, 2, 3, 4],
        &[0, 1, 2, 3, 4, 5],
        &[0, 1, 2, 3, 4, 5, 6],
        &[0, 1, 2, 3, 4, 5, 6, 7],
        &[0, 1, 2, 3, 4, 5, 6, 7, 8],
        &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9],
        &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10],
        &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11],
        &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12],
        &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13],
        &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14],
        &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15],
        &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
        &[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17],
    ];

    #[test]
    fn slice_ptr_access() {
        let slice = SlicePtr::from([5, 8, 13].as_slice());

        assert_eq!(unsafe { slice.unchecked_get(0).read() }, 5);
        assert_eq!(unsafe { slice.unchecked_get(1).read() }, 8);
        assert_eq!(unsafe { slice.unchecked_get(2).read() }, 13);

        assert_eq!(
            slice.checked_get(0).map(|ptr| unsafe { ptr.read() }),
            Some(5)
        );
        assert_eq!(
            slice.checked_get(1).map(|ptr| unsafe { ptr.read() }),
            Some(8)
        );
        assert_eq!(
            slice.checked_get(2).map(|ptr| unsafe { ptr.read() }),
            Some(13)
        );
        assert_eq!(slice.checked_get(3).map(|ptr| unsafe { ptr.read() }), None);

        assert_eq!(unsafe { slice.get(0).read() }, 5);
        assert_eq!(unsafe { slice.get(1).read() }, 8);
        assert_eq!(unsafe { slice.get(2).read() }, 13);
    }

    #[test]
    #[should_panic]
    fn slice_ptr_oob() {
        let slice = SlicePtr::from([5, 8, 13].as_slice());

        slice.get(3);
    }

    #[test]
    fn slice_ptr_iter() {
        for &slice in SLICES {
            let slice_ptr = SlicePtr::from(slice);

            assert!(slice_ptr
                .map(|ptr| unsafe { ptr.as_ref() })
                .eq(slice.iter()));
            assert!(slice_ptr
                .rev()
                .map(|ptr| unsafe { ptr.as_ref() })
                .eq(slice.iter().rev()));

            for step in [1, 2, 3] {
                assert!(slice_ptr
                    .step_by(step)
                    .map(|ptr| unsafe { ptr.as_ref() })
                    .eq(slice.iter().step_by(step)));
                assert!(slice_ptr
                    .rev()
                    .step_by(step)
                    .map(|ptr| unsafe { ptr.as_ref() })
                    .eq(slice.iter().rev().step_by(step)));
            }
        }
    }

    #[test]
    fn strided_slice_access() {
        let slice = StridedSlice::from([5, 8, 13].as_slice());

        assert_eq!(*unsafe { slice.unchecked_get(0) }, 5);
        assert_eq!(*unsafe { slice.unchecked_get(1) }, 8);
        assert_eq!(*unsafe { slice.unchecked_get(2) }, 13);

        assert_eq!(slice.checked_get(0).copied(), Some(5));
        assert_eq!(slice.checked_get(1).copied(), Some(8));
        assert_eq!(slice.checked_get(2).copied(), Some(13));
        assert_eq!(slice.checked_get(3).copied(), None);

        assert_eq!(*slice.get(0), 5);
        assert_eq!(*slice.get(1), 8);
        assert_eq!(*slice.get(2), 13);
    }

    #[test]
    #[should_panic]
    fn strided_slice_oob() {
        let slice = StridedSlice::from([5, 8, 13].as_slice());

        slice.get(3);
    }

    #[test]
    fn strided_slice_iter() {
        for &slice in SLICES {
            let slice_ptr = StridedSlice::from(slice);

            assert!(slice_ptr.eq(slice.iter()));
            assert!(slice_ptr.rev().eq(slice.iter().rev()));

            for step in [1, 2, 3] {
                assert!(slice_ptr.step_by(step).eq(slice.iter().step_by(step)));
                assert!(slice_ptr
                    .rev()
                    .step_by(step)
                    .eq(slice.iter().rev().step_by(step)));
            }
        }
    }

    #[test]
    fn strided_slice_subslice() {
        for step in [2, 3] {
            for &slice in SLICES {
                let iterator = slice.iter().skip(1).step_by(step);
                let slice_ptr =
                    StridedSlice::from(slice).into_subslice(1, slice.len(), step as isize);

                assert!(slice_ptr.eq(iterator.clone()));
                assert!(slice_ptr.rev().eq(iterator.clone().rev()));

                for step in [1, 2, 3] {
                    assert!(slice_ptr.step_by(step).eq(iterator.clone().step_by(step)));
                    assert!(slice_ptr
                        .rev()
                        .step_by(step)
                        .eq(iterator.clone().rev().step_by(step)));
                }
            }
        }
    }

    #[test]
    fn strided_slice_subslice_neg() {
        for step in [1, 2, 3] {
            for &slice in SLICES {
                let iterator = slice.iter().skip(1).rev().step_by(step);
                let slice_ptr =
                    StridedSlice::from(slice).into_subslice(1, slice.len(), -(step as isize));

                assert!(slice_ptr.eq(iterator.clone()));
                assert!(slice_ptr.rev().eq(iterator.clone().rev()));

                for step in [1, 2, 3] {
                    assert!(slice_ptr.step_by(step).eq(iterator.clone().step_by(step)));
                    assert!(slice_ptr
                        .rev()
                        .step_by(step)
                        .eq(iterator.clone().rev().step_by(step)));
                }
            }
        }
    }
}
