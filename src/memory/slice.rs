use std::{iter::FusedIterator, marker::PhantomData};

use super::{ptr_mut_offset, ptr_offset};

#[derive(Copy)]
#[repr(C)]
pub struct StridedSlice<'a, T> {
    ptr: *const T,
    stride: isize,
    len: usize,
    _marker: PhantomData<&'a [T]>,
}

impl<'a, T> Clone for StridedSlice<'a, T> {
    fn clone(&self) -> Self {
        Self {
            ptr: self.ptr.clone(),
            stride: self.stride.clone(),
            len: self.len.clone(),
            _marker: self._marker.clone(),
        }
    }
}

impl<'a, T> StridedSlice<'a, T> {
    pub unsafe fn from_raw_parts(ptr: *const T, stride: isize, len: usize) -> Self {
        Self {
            ptr,
            stride,
            len,
            _marker: PhantomData,
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }
    pub fn stride(&self) -> isize {
        self.stride
    }
    pub fn as_ptr(&self) -> *const T {
        self.ptr
    }

    pub unsafe fn unchecked_get(&self, i: usize) -> &T {
        unsafe {
            ptr_offset(self.ptr, i, self.stride)
                .as_ref()
                .unwrap_unchecked()
        }
    }
    pub fn checked_get(&self, i: usize) -> Option<&T> {
        if i < self.len {
            Some(unsafe { self.unchecked_get(i) })
        } else {
            None
        }
    }
    pub fn get(&self, i: usize) -> &T {
        if i < self.len {
            unsafe { self.unchecked_get(i) }
        } else {
            panic!(
                "Trying to access element #{} from a slice with {} elements",
                i, self.len
            )
        }
    }
}

unsafe impl<'a, T: Sync> Send for StridedSlice<'a, T> {}
unsafe impl<'a, T: Sync> Sync for StridedSlice<'a, T> {}

impl<'a, 'b, T> PartialEq<StridedSlice<'a, T>> for StridedSlice<'b, T> {
    fn eq(&self, other: &StridedSlice<'a, T>) -> bool {
        self.ptr == other.ptr && self.stride == other.stride && self.len == other.len
    }
}
impl<T> Eq for StridedSlice<'_, T> {}
impl<'a, 'b, T> PartialEq<StridedSliceMut<'a, T>> for StridedSlice<'b, T> {
    fn eq(&self, other: &StridedSliceMut<'a, T>) -> bool {
        self.ptr == other.ptr && self.stride == other.stride && self.len == other.len
    }
}

impl<'a, T> std::ops::Index<usize> for StridedSlice<'a, T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        self.get(index)
    }
}

impl<'a, T> From<&'a mut [T]> for StridedSlice<'a, T> {
    fn from(value: &'a mut [T]) -> Self {
        unsafe {
            Self::from_raw_parts(
                value.as_ptr(),
                std::mem::size_of::<T>() as isize,
                value.len(),
            )
        }
    }
}
impl<'a, T> From<&'a [T]> for StridedSlice<'a, T> {
    fn from(value: &'a [T]) -> Self {
        unsafe {
            Self::from_raw_parts(
                value.as_ptr(),
                std::mem::size_of::<T>() as isize,
                value.len(),
            )
        }
    }
}
impl<'a, T> From<StridedSliceMut<'a, T>> for StridedSlice<'a, T> {
    fn from(value: StridedSliceMut<'a, T>) -> Self {
        Self {
            ptr: value.ptr,
            stride: value.stride,
            len: value.len,
            _marker: PhantomData,
        }
    }
}

impl<'a, T> Iterator for StridedSlice<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.len > 0 {
            unsafe {
                let ptr = self.ptr;
                self.ptr = ptr.byte_offset(self.stride);
                self.len -= 1;
                Some(ptr.as_ref().unwrap_unchecked())
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
        unsafe {
            let steps = n.min(self.len);
            let offset = (steps as isize).unchecked_mul(self.stride);
            self.ptr = self.ptr.byte_offset(offset);
            self.len = self.len.unchecked_sub(steps);

            self.next()
        }
    }
}

impl<'a, T> ExactSizeIterator for StridedSlice<'a, T> {}
impl<'a, T> FusedIterator for StridedSlice<'a, T> {}
impl<'a, T> DoubleEndedIterator for StridedSlice<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.len > 0 {
            unsafe {
                let len = self.len.unchecked_sub(1);
                let ptr = ptr_offset(self.ptr, len, self.stride);
                self.len = len;
                Some(ptr.as_ref().unwrap_unchecked())
            }
        } else {
            None
        }
    }
}

#[repr(C)]
pub struct StridedSliceMut<'a, T> {
    ptr: *mut T,
    stride: isize,
    len: usize,
    _marker: PhantomData<&'a mut [T]>,
}

impl<'a, T> StridedSliceMut<'a, T> {
    pub unsafe fn from_raw_parts(ptr: *mut T, stride: isize, len: usize) -> Self {
        Self {
            ptr,
            stride,
            len,
            _marker: PhantomData,
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }
    pub fn stride(&self) -> isize {
        self.stride
    }
    pub fn as_ptr(&self) -> *const T {
        self.ptr
    }
    pub fn as_ptr_mut(&mut self) -> *mut T {
        self.ptr
    }
    pub fn as_ref(&mut self) -> StridedSlice<'_, T> {
        StridedSlice {
            ptr: self.ptr,
            stride: self.stride,
            len: self.len,
            _marker: PhantomData,
        }
    }

    pub unsafe fn unchecked_get(&self, i: usize) -> &T {
        unsafe {
            ptr_offset(self.ptr, i, self.stride)
                .as_ref()
                .unwrap_unchecked()
        }
    }
    pub fn checked_get(&self, i: usize) -> Option<&T> {
        if i < self.len {
            Some(unsafe { self.unchecked_get(i) })
        } else {
            None
        }
    }
    pub fn get(&self, i: usize) -> &T {
        if i < self.len {
            unsafe { self.unchecked_get(i) }
        } else {
            panic!(
                "Trying to access element #{} from a slice with {} elements",
                i, self.len
            )
        }
    }

    pub unsafe fn unchecked_mut(&mut self, i: usize) -> &mut T {
        unsafe {
            ptr_mut_offset(self.ptr, i, self.stride)
                .as_mut()
                .unwrap_unchecked()
        }
    }
    pub fn checked_mut(&mut self, i: usize) -> Option<&mut T> {
        if i < self.len {
            Some(unsafe { self.unchecked_mut(i) })
        } else {
            None
        }
    }
    pub fn get_mut(&mut self, i: usize) -> &mut T {
        if i < self.len {
            unsafe { self.unchecked_mut(i) }
        } else {
            panic!(
                "Trying to access element #{} from a slice with {} elements",
                i, self.len
            )
        }
    }
}

unsafe impl<'a, T: Sync> Send for StridedSliceMut<'a, T> {}
unsafe impl<'a, T: Sync> Sync for StridedSliceMut<'a, T> {}

impl<'a, 'b, T> PartialEq<StridedSliceMut<'a, T>> for StridedSliceMut<'b, T> {
    fn eq(&self, other: &StridedSliceMut<'a, T>) -> bool {
        self.ptr == other.ptr && self.stride == other.stride && self.len == other.len
    }
}
impl<T> Eq for StridedSliceMut<'_, T> {}
impl<'a, 'b, T> PartialEq<StridedSlice<'a, T>> for StridedSliceMut<'b, T> {
    fn eq(&self, other: &StridedSlice<'a, T>) -> bool {
        other.ptr == self.ptr && self.stride == other.stride && self.len == other.len
    }
}

impl<'a, T> std::ops::Index<usize> for StridedSliceMut<'a, T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        self.get(index)
    }
}
impl<'a, T> std::ops::IndexMut<usize> for StridedSliceMut<'a, T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.get_mut(index)
    }
}

impl<'a, T> From<&'a mut [T]> for StridedSliceMut<'a, T> {
    fn from(value: &'a mut [T]) -> Self {
        unsafe {
            Self::from_raw_parts(
                value.as_mut_ptr(),
                std::mem::size_of::<T>() as isize,
                value.len(),
            )
        }
    }
}

impl<'a, T> Iterator for StridedSliceMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.len > 0 {
            unsafe {
                let ptr = self.ptr;
                self.ptr = ptr.byte_offset(self.stride);
                self.len -= 1;
                Some(ptr.as_mut().unwrap_unchecked())
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
        unsafe {
            let steps = n.min(self.len);
            let offset = (steps as isize).unchecked_mul(self.stride);
            self.ptr = self.ptr.byte_offset(offset);
            self.len = self.len.unchecked_sub(steps);

            self.next()
        }
    }
}

impl<'a, T> ExactSizeIterator for StridedSliceMut<'a, T> {}
impl<'a, T> FusedIterator for StridedSliceMut<'a, T> {}
impl<'a, T> DoubleEndedIterator for StridedSliceMut<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.len > 0 {
            unsafe {
                let len = self.len.unchecked_sub(1);
                let ptr = ptr_mut_offset(self.ptr, len, self.stride);
                self.len = len;
                Some(ptr.as_mut().unwrap_unchecked())
            }
        } else {
            None
        }
    }
}
