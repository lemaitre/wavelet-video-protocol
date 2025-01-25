use std::{iter::FusedIterator, marker::PhantomData, ptr::NonNull};

use super::{ptr_offset, ImageView, StridedSlice, StridedSliceMut, StridedSlicePtr};

pub struct ImageRowIterPtr<'a, T>(pub(super) ImageView<'a, T>);

impl<'a, T> Iterator for ImageRowIterPtr<'a, T> {
    type Item = NonNull<[T]>;

    fn next(&mut self) -> Option<Self::Item> {
        let width = self.0.width();
        let height = self.0.height();
        if height > 0 {
            let ptr = self.0.ptr();
            self.0 = std::mem::take(&mut self.0).into_subview(0, 1, width, height);
            Some(NonNull::slice_from_raw_parts(ptr, width))
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
        self.0 = std::mem::take(&mut self.0).into_subview(0, n, width, height);
        self.next()
    }
}

impl<'a, T> ExactSizeIterator for ImageRowIterPtr<'a, T> {}
impl<'a, T> FusedIterator for ImageRowIterPtr<'a, T> {}
impl<'a, T> DoubleEndedIterator for ImageRowIterPtr<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let width = self.0.width();
        let height = self.0.height();
        if height > 0 {
            let height = height - 1;
            let ptr = unsafe { ptr_offset(self.0.ptr(), height, self.0.stride() as isize) };
            self.0 = std::mem::take(&mut self.0).into_subview(0, 0, width, height);
            Some(NonNull::slice_from_raw_parts(ptr, width))
        } else {
            None
        }
    }
}

pub struct ImageColIterPtr<'a, T>(pub(super) ImageView<'a, T>);

impl<'a, T> Iterator for ImageColIterPtr<'a, T> {
    type Item = StridedSlicePtr<T>;

    fn next(&mut self) -> Option<Self::Item> {
        let stride = self.0.stride();
        let width = self.0.width();
        let height = self.0.height();
        if width > 0 {
            let ptr = self.0.ptr();
            self.0 = std::mem::take(&mut self.0).into_subview(1, 0, width, height);
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
        self.0 = std::mem::take(&mut self.0).into_subview(n, 0, width, height);
        self.next()
    }
}

impl<'a, T> ExactSizeIterator for ImageColIterPtr<'a, T> {}
impl<'a, T> FusedIterator for ImageColIterPtr<'a, T> {}
impl<'a, T> DoubleEndedIterator for ImageColIterPtr<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let stride = self.0.stride();
        let width = self.0.width();
        let height = self.0.height();
        if width > 0 {
            let width = width - 1;
            let ptr = unsafe { self.0.ptr().add(width) };
            self.0 = std::mem::take(&mut self.0).into_subview(0, 0, width, height);
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
}

pub struct ImageRowIter<'a, T>(
    pub(super) ImageRowIterPtr<'a, T>,
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

impl<'a, T> ExactSizeIterator for ImageRowIter<'a, T> {}
impl<'a, T> FusedIterator for ImageRowIter<'a, T> {}
impl<'a, T> DoubleEndedIterator for ImageRowIter<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back().map(|ptr| unsafe { ptr.as_ref() })
    }
}

pub struct ImageRowIterMut<'a, T>(
    pub(super) ImageRowIterPtr<'a, T>,
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

impl<'a, T> ExactSizeIterator for ImageRowIterMut<'a, T> {}
impl<'a, T> FusedIterator for ImageRowIterMut<'a, T> {}
impl<'a, T> DoubleEndedIterator for ImageRowIterMut<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0.next_back().map(|mut ptr| unsafe { ptr.as_mut() })
    }
}

pub struct ImageColIter<'a, T>(
    pub(super) ImageColIterPtr<'a, T>,
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

impl<'a, T> ExactSizeIterator for ImageColIter<'a, T> {}
impl<'a, T> FusedIterator for ImageColIter<'a, T> {}
impl<'a, T> DoubleEndedIterator for ImageColIter<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0
            .next_back()
            .map(|ptr| unsafe { ptr.as_strided_slice() })
    }
}

pub struct ImageColIterMut<'a, T>(
    pub(super) ImageColIterPtr<'a, T>,
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

impl<'a, T> ExactSizeIterator for ImageColIterMut<'a, T> {}
impl<'a, T> FusedIterator for ImageColIterMut<'a, T> {}
impl<'a, T> DoubleEndedIterator for ImageColIterMut<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.0
            .next_back()
            .map(|ptr| unsafe { ptr.as_strided_slice_mut() })
    }
}
