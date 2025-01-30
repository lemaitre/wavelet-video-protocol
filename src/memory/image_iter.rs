use std::{iter::FusedIterator, marker::PhantomData};

use super::{
    ptr_offset, slice::SlicePtr, ImageViewPtr, StridedSlice, StridedSliceMut, StridedSlicePtr,
};

pub struct ImageRowIterPtr<T>(pub(super) ImageViewPtr<T>);

impl<T> Iterator for ImageRowIterPtr<T> {
    type Item = SlicePtr<T>;

    fn next(&mut self) -> Option<Self::Item> {
        let width = self.0.width();
        let height = self.0.height();
        if height > 0 {
            let ptr = self.0.ptr();
            self.0 = self.0.subview(0, 1, width, height);
            Some(unsafe { SlicePtr::from_raw_parts(ptr, width) })
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.0.height(), Some(self.0.height()))
    }

    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.0.height()
    }

    fn last(mut self) -> Option<Self::Item>
    where
        Self: Sized,
    {
        self.next_back()
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.0 = self.0.subview(0, n, self.0.width(), self.0.height());
        self.next()
    }
}

impl<T> ExactSizeIterator for ImageRowIterPtr<T> {
    fn len(&self) -> usize {
        self.0.height()
    }
}
impl<T> FusedIterator for ImageRowIterPtr<T> {}
impl<T> DoubleEndedIterator for ImageRowIterPtr<T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let width = self.0.width();
        let height = self.0.height();
        if height > 0 {
            let height = height - 1;
            let ptr = unsafe { ptr_offset(self.0.ptr(), height, self.0.stride() as isize) };
            self.0 = self.0.subview(0, 0, width, height);
            Some(unsafe { SlicePtr::from_raw_parts(ptr, width) })
        } else {
            None
        }
    }

    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        self.0 = self
            .0
            .subview(0, 0, self.0.width(), self.0.height().saturating_sub(n));
        self.next_back()
    }
}

pub struct ImageColIterPtr<T>(pub(super) ImageViewPtr<T>);

impl<T> Iterator for ImageColIterPtr<T> {
    type Item = StridedSlicePtr<T>;

    fn next(&mut self) -> Option<Self::Item> {
        let stride = self.0.stride();
        let width = self.0.width();
        let height = self.0.height();
        if width > 0 {
            let ptr = self.0.ptr();
            self.0 = self.0.subview(1, 0, width, height);
            unsafe {
                Some(StridedSlicePtr::from_raw_parts(
                    ptr.as_ptr(),
                    stride as isize,
                    height,
                ))
            }
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.0.height(), Some(self.0.height()))
    }

    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.0.height()
    }

    fn last(mut self) -> Option<Self::Item>
    where
        Self: Sized,
    {
        self.next_back()
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        let width = self.0.width();
        let height = self.0.height();
        self.0 = self.0.subview(n, 0, width, height);
        self.next()
    }
}

impl<T> ExactSizeIterator for ImageColIterPtr<T> {
    fn len(&self) -> usize {
        self.0.width()
    }
}
impl<T> FusedIterator for ImageColIterPtr<T> {}
impl<T> DoubleEndedIterator for ImageColIterPtr<T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let stride = self.0.stride();
        let width = self.0.width();
        let height = self.0.height();
        if width > 0 {
            let width = width - 1;
            let ptr = unsafe { self.0.ptr().add(width) };
            self.0 = self.0.subview(0, 0, width, height);
            unsafe {
                Some(StridedSlicePtr::from_raw_parts(
                    ptr.as_ptr(),
                    stride as isize,
                    height,
                ))
            }
        } else {
            None
        }
    }
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        self.0 = self
            .0
            .subview(0, 0, self.0.width().saturating_sub(n), self.0.height());
        self.next_back()
    }
}

pub struct ImageRowIter<'a, T>(
    pub(super) ImageRowIterPtr<T>,
    pub(super) PhantomData<&'a [T]>,
);

impl<'a, T> Iterator for ImageRowIter<'a, T> {
    type Item = &'a [T];

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|ptr| unsafe { ptr.as_ref() })
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
        self.0.last().map(|ptr| unsafe { ptr.as_ref() })
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.0.nth(n).map(|ptr| unsafe { ptr.as_ref() })
    }
}

impl<T> ExactSizeIterator for ImageRowIter<'_, T> {
    fn len(&self) -> usize {
        self.0.len()
    }
}
impl<T> FusedIterator for ImageRowIter<'_, T> {}
impl<T> DoubleEndedIterator for ImageRowIter<'_, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back().map(|ptr| unsafe { ptr.as_ref() })
    }
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        self.0.nth_back(n).map(|ptr| unsafe { ptr.as_ref() })
    }
}

pub struct ImageRowIterMut<'a, T>(
    pub(super) ImageRowIterPtr<T>,
    pub(super) PhantomData<&'a mut [T]>,
);

impl<'a, T> Iterator for ImageRowIterMut<'a, T> {
    type Item = &'a mut [T];

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|mut ptr| unsafe { ptr.as_mut() })
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
        self.0.last().map(|mut ptr| unsafe { ptr.as_mut() })
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.0.nth(n).map(|mut ptr| unsafe { ptr.as_mut() })
    }
}

impl<T> ExactSizeIterator for ImageRowIterMut<'_, T> {
    fn len(&self) -> usize {
        self.0.len()
    }
}
impl<T> FusedIterator for ImageRowIterMut<'_, T> {}
impl<T> DoubleEndedIterator for ImageRowIterMut<'_, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back().map(|mut ptr| unsafe { ptr.as_mut() })
    }
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        self.0.nth_back(n).map(|mut ptr| unsafe { ptr.as_mut() })
    }
}

pub struct ImageColIter<'a, T>(
    pub(super) ImageColIterPtr<T>,
    pub(super) PhantomData<&'a [T]>,
);

impl<'a, T> Iterator for ImageColIter<'a, T> {
    type Item = StridedSlice<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|ptr| unsafe { ptr.as_strided_slice() })
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
        self.0.last().map(|ptr| unsafe { ptr.as_strided_slice() })
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.0.nth(n).map(|ptr| unsafe { ptr.as_strided_slice() })
    }
}

impl<T> ExactSizeIterator for ImageColIter<'_, T> {
    fn len(&self) -> usize {
        self.0.len()
    }
}
impl<T> FusedIterator for ImageColIter<'_, T> {}
impl<T> DoubleEndedIterator for ImageColIter<'_, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0
            .next_back()
            .map(|ptr| unsafe { ptr.as_strided_slice() })
    }
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        self.0
            .nth_back(n)
            .map(|ptr| unsafe { ptr.as_strided_slice() })
    }
}

pub struct ImageColIterMut<'a, T>(
    pub(super) ImageColIterPtr<T>,
    pub(super) PhantomData<&'a mut [T]>,
);

impl<'a, T> Iterator for ImageColIterMut<'a, T> {
    type Item = StridedSliceMut<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0
            .next()
            .map(|ptr| unsafe { ptr.as_strided_slice_mut() })
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
        self.0
            .last()
            .map(|ptr| unsafe { ptr.as_strided_slice_mut() })
    }

    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.0
            .nth(n)
            .map(|ptr| unsafe { ptr.as_strided_slice_mut() })
    }
}

impl<T> ExactSizeIterator for ImageColIterMut<'_, T> {
    fn len(&self) -> usize {
        self.0.len()
    }
}
impl<T> FusedIterator for ImageColIterMut<'_, T> {}
impl<T> DoubleEndedIterator for ImageColIterMut<'_, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0
            .next_back()
            .map(|ptr| unsafe { ptr.as_strided_slice_mut() })
    }
    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        self.0
            .nth_back(n)
            .map(|ptr| unsafe { ptr.as_strided_slice_mut() })
    }
}
