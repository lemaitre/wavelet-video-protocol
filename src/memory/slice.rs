use std::{iter::FusedIterator, marker::PhantomData, ptr::NonNull};

use super::ptr_offset;

#[repr(C)]
pub struct StridedSlicePtr<T> {
    ptr: NonNull<T>,
    stride: isize,
    len: usize,
}

impl<T> StridedSlicePtr<T> {
    pub unsafe fn from_raw_parts(ptr: *const T, stride: isize, len: usize) -> Self {
        Self {
            ptr: NonNull::new_unchecked(ptr as *mut T),
            stride,
            len,
        }
    }

    pub fn len(&self) -> usize {
        self.len
    }
    pub fn stride(&self) -> isize {
        self.stride
    }
    pub fn as_ptr(&self) -> NonNull<T> {
        self.ptr
    }

    pub unsafe fn as_strided_slice<'a>(&self) -> StridedSlice<'a, T> {
        StridedSlice(*self, PhantomData)
    }
    pub unsafe fn as_strided_slice_mut<'a>(&self) -> StridedSliceMut<'a, T> {
        StridedSliceMut(*self, PhantomData)
    }

    pub unsafe fn unchecked_get(&self, i: usize) -> NonNull<T> {
        unsafe { ptr_offset(self.ptr, i, self.stride) }
    }
    pub fn checked_get(&self, i: usize) -> Option<NonNull<T>> {
        if i < self.len {
            Some(unsafe { self.unchecked_get(i) })
        } else {
            None
        }
    }
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
}

unsafe impl<T: Sync> Send for StridedSlicePtr<T> {}
unsafe impl<T: Sync> Sync for StridedSlicePtr<T> {}

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
            ptr: value.ptr,
            stride: std::mem::size_of::<T>() as isize,
            len: value.len,
        }
    }
}
impl<T> From<NonNull<[T]>> for StridedSlicePtr<T> {
    fn from(value: NonNull<[T]>) -> Self {
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
                self.ptr = ptr.byte_offset(self.stride);
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
        unsafe {
            let steps = n.min(self.len);
            let offset = (steps as isize).unchecked_mul(self.stride);
            self.ptr = self.ptr.byte_offset(offset);
            self.len = self.len.unchecked_sub(steps);

            self.next()
        }
    }
}

impl<T> ExactSizeIterator for StridedSlicePtr<T> {}
impl<T> FusedIterator for StridedSlicePtr<T> {}
impl<T> DoubleEndedIterator for StridedSlicePtr<T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.len > 0 {
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
}

#[repr(C)]
pub struct StridedSlice<'a, T>(StridedSlicePtr<T>, PhantomData<&'a [T]>);

impl<T> Default for StridedSlice<'_, T> {
    fn default() -> Self {
        Self(StridedSlicePtr::default(), PhantomData)
    }
}

impl<T> Clone for StridedSlice<'_, T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<T> Copy for StridedSlice<'_, T> {}

impl<'a, T> StridedSlice<'a, T> {
    pub unsafe fn from_raw_parts(ptr: *const T, stride: isize, len: usize) -> Self {
        Self(
            StridedSlicePtr::from_raw_parts(ptr, stride, len),
            PhantomData,
        )
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
    pub fn stride(&self) -> isize {
        self.0.stride()
    }
    pub fn as_ptr(&self) -> NonNull<T> {
        self.0.as_ptr()
    }

    pub unsafe fn unchecked_get(&self, i: usize) -> &T {
        unsafe { self.0.unchecked_get(i).as_ref() }
    }
    pub fn checked_get(&self, i: usize) -> Option<&T> {
        self.0.checked_get(i).map(|value| unsafe { value.as_ref() })
    }
    pub fn get(&self, i: usize) -> &T {
        unsafe { self.0.get(i).as_ref() }
    }
}

impl<'a, 'b, T> PartialEq<StridedSlice<'a, T>> for StridedSlice<'b, T> {
    fn eq(&self, other: &StridedSlice<'a, T>) -> bool {
        self.0 == other.0
    }
}
impl<T> Eq for StridedSlice<'_, T> {}
impl<'a, 'b, T> PartialEq<StridedSliceMut<'a, T>> for StridedSlice<'b, T> {
    fn eq(&self, other: &StridedSliceMut<'a, T>) -> bool {
        self.0 == other.0
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
        unsafe { Self::from_raw_parts(value.as_ptr().as_ptr(), value.stride(), value.len()) }
    }
}

impl<'a, T> Iterator for StridedSlice<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
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
        self.0.last().map(|item| unsafe { item.as_ref() })
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.0.nth(n).map(|item| unsafe { item.as_ref() })
    }
}

impl<'a, T> ExactSizeIterator for StridedSlice<'a, T> {}
impl<'a, T> FusedIterator for StridedSlice<'a, T> {}
impl<'a, T> DoubleEndedIterator for StridedSlice<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back().map(|item| unsafe { item.as_ref() })
    }
}

#[repr(C)]
pub struct StridedSliceMut<'a, T>(StridedSlicePtr<T>, PhantomData<&'a mut [T]>);

impl<'a, T> StridedSliceMut<'a, T> {
    pub unsafe fn from_raw_parts(ptr: *mut T, stride: isize, len: usize) -> Self {
        Self(
            StridedSlicePtr::from_raw_parts(ptr, stride, len),
            PhantomData,
        )
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
    pub fn stride(&self) -> isize {
        self.0.stride()
    }
    pub fn as_ptr(&self) -> NonNull<T> {
        self.0.as_ptr()
    }
    pub fn as_strided_slice(&mut self) -> StridedSlice<'_, T> {
        unsafe { StridedSlice::from_raw_parts(self.as_ptr().as_ptr(), self.stride(), self.len()) }
    }

    pub unsafe fn unchecked_get(&self, i: usize) -> &T {
        unsafe { self.0.unchecked_get(i).as_ref() }
    }
    pub fn checked_get(&self, i: usize) -> Option<&T> {
        self.0.checked_get(i).map(|value| unsafe { value.as_ref() })
    }
    pub fn get(&self, i: usize) -> &T {
        unsafe { self.0.get(i).as_ref() }
    }

    pub unsafe fn unchecked_get_mut(&mut self, i: usize) -> &mut T {
        unsafe { self.0.unchecked_get(i).as_mut() }
    }
    pub fn checked_get_mut(&mut self, i: usize) -> Option<&mut T> {
        self.0
            .checked_get(i)
            .map(|mut value| unsafe { value.as_mut() })
    }
    pub fn get_mut(&self, i: usize) -> &mut T {
        unsafe { self.0.get(i).as_mut() }
    }
}

impl<'a, 'b, T> PartialEq<StridedSliceMut<'a, T>> for StridedSliceMut<'b, T> {
    fn eq(&self, other: &StridedSliceMut<'a, T>) -> bool {
        self.0 == other.0
    }
}
impl<T> Eq for StridedSliceMut<'_, T> {}
impl<'a, 'b, T> PartialEq<StridedSlice<'a, T>> for StridedSliceMut<'b, T> {
    fn eq(&self, other: &StridedSlice<'a, T>) -> bool {
        self.0 == other.0
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
        self.0.last().map(|mut item| unsafe { item.as_mut() })
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.0.nth(n).map(|mut item| unsafe { item.as_mut() })
    }
}

impl<'a, T> ExactSizeIterator for StridedSliceMut<'a, T> {}
impl<'a, T> FusedIterator for StridedSliceMut<'a, T> {}
impl<'a, T> DoubleEndedIterator for StridedSliceMut<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back().map(|mut item| unsafe { item.as_mut() })
    }
}

pub struct SlicePtr<T> {
    ptr: NonNull<T>,
    len: usize,
}

unsafe impl<T: Sync> Send for SlicePtr<T> {}
unsafe impl<T: Sync> Sync for SlicePtr<T> {}
impl<T> Copy for SlicePtr<T> {}

impl<T> Clone for SlicePtr<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Default for SlicePtr<T> {
    fn default() -> Self {
        Self {
            ptr: NonNull::dangling(),
            len: Default::default(),
        }
    }
}

impl<T> From<NonNull<[T]>> for SlicePtr<T> {
    fn from(value: NonNull<[T]>) -> Self {
        Self {
            ptr: unsafe { NonNull::new_unchecked(value.as_ptr() as *mut T) },
            len: value.len(),
        }
    }
}
impl<T> From<SlicePtr<T>> for NonNull<[T]> {
    fn from(value: SlicePtr<T>) -> Self {
        NonNull::slice_from_raw_parts(value.ptr, value.len)
    }
}

impl<T> PartialEq for SlicePtr<T> {
    fn eq(&self, other: &Self) -> bool {
        self.ptr == other.ptr && self.len == other.len
    }
}
impl<T> Eq for SlicePtr<T> {}

impl<T> Iterator for SlicePtr<T> {
    type Item = NonNull<T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.len > 0 {
            unsafe {
                let ptr = self.ptr;
                self.ptr = self.ptr.add(1);
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
        unsafe {
            let n = n.min(self.len);
            self.ptr = self.ptr.add(n);
            self.len = self.len.unchecked_sub(n);

            self.next()
        }
    }
}

impl<T> ExactSizeIterator for SlicePtr<T> {}
impl<T> FusedIterator for SlicePtr<T> {}
impl<T> DoubleEndedIterator for SlicePtr<T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.len > 0 {
            unsafe {
                let len = self.len.unchecked_sub(1);
                let ptr = self.ptr.add(len);
                self.len = len;
                Some(ptr)
            }
        } else {
            None
        }
    }
}
