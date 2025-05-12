#![allow(clippy::missing_safety_doc)]

use std::{
    iter::FusedIterator,
    ops::{Deref, DerefMut},
    ptr::NonNull,
};

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

    use super::SlicePtr;

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
}
