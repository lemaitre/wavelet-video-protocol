#![allow(clippy::missing_safety_doc)]

use std::{alloc::Layout, marker::PhantomData, mem::MaybeUninit, ptr::NonNull};

use super::{
    ptr_offset, slice::SlicePtr, ImageColIter, ImageColIterMut, ImageColIterPtr, ImageRowIter,
    ImageRowIterMut, ImageRowIterPtr, StridedSlice, StridedSliceMut, StridedSlicePtr,
};

pub struct ImageViewPtr<T> {
    ptr: NonNull<T>,
    stride: usize,
    width: usize,
    height: usize,
}

impl<T> ImageViewPtr<T> {
    pub fn width(&self) -> usize {
        self.width
    }
    pub fn height(&self) -> usize {
        self.height
    }
    pub fn size(&self) -> usize {
        self.width() * self.height()
    }
    pub fn stride(&self) -> usize {
        self.stride
    }
    pub fn ptr(&self) -> NonNull<T> {
        self.ptr
    }
    pub fn cast<U>(&self) -> ImageViewPtr<U> {
        ImageViewPtr {
            ptr: self.ptr().cast(),
            stride: self.stride(),
            width: self.width(),
            height: self.height(),
        }
    }

    pub unsafe fn unchecked_get(&self, x: usize, y: usize) -> NonNull<T> {
        unsafe { self.unchecked_row(y).unchecked_get(x) }
    }
    pub fn checked_get(&self, x: usize, y: usize) -> Option<NonNull<T>> {
        if x < self.width() && y < self.height() {
            Some(unsafe { self.unchecked_get(x, y) })
        } else {
            None
        }
    }
    pub fn get(&self, x: usize, y: usize) -> NonNull<T> {
        if x < self.width() && y < self.height() {
            unsafe { self.unchecked_get(x, y) }
        } else {
            panic!(
                "Cell ({}, {}) is outside of image of size {} x {}",
                x,
                y,
                self.width(),
                self.height()
            )
        }
    }

    pub unsafe fn unchecked_row(&self, y: usize) -> SlicePtr<T> {
        unsafe {
            SlicePtr::from_raw_parts(
                ptr_offset(self.ptr(), y, self.stride() as isize),
                self.width(),
            )
        }
    }
    pub fn checked_row(&self, y: usize) -> Option<SlicePtr<T>> {
        if y < self.height() {
            Some(unsafe { self.unchecked_row(y) })
        } else {
            None
        }
    }
    pub fn row(&self, y: usize) -> SlicePtr<T> {
        if y < self.height() {
            unsafe { self.unchecked_row(y) }
        } else {
            panic!(
                "Trying to access row {y} from an image with height {}",
                self.height()
            )
        }
    }

    pub unsafe fn unchecked_col(&self, x: usize) -> StridedSlicePtr<T> {
        unsafe {
            StridedSlicePtr::from_raw_parts(
                ptr_offset(self.ptr(), x, std::mem::size_of::<T>() as isize).as_ptr(),
                self.stride() as isize,
                self.height(),
            )
        }
    }
    pub fn checked_col(&self, x: usize) -> Option<StridedSlicePtr<T>> {
        if x < self.height() {
            Some(unsafe { self.unchecked_col(x) })
        } else {
            None
        }
    }
    pub fn col(&self, x: usize) -> StridedSlicePtr<T> {
        if x < self.height() {
            unsafe { self.unchecked_col(x) }
        } else {
            panic!(
                "Trying to access column {x} from an image with width {}",
                self.height()
            )
        }
    }

    pub fn rows(&self) -> ImageRowIterPtr<T> {
        ImageRowIterPtr(*self)
    }
    pub fn cols(&self) -> ImageColIterPtr<T> {
        ImageColIterPtr(*self)
    }

    pub fn subview(&self, x: usize, y: usize, width: usize, height: usize) -> Self {
        let x = x.min(self.width());
        let y = y.min(self.height());
        let width = width.min(self.width().saturating_sub(x));
        let height = height.min(self.height().saturating_sub(y));

        unsafe {
            let ptr = self.ptr();
            let ptr = ptr_offset(ptr, y, self.stride() as isize);
            let ptr = ptr_offset(ptr, x, std::mem::size_of::<T>() as isize);

            Self {
                ptr,
                stride: self.stride(),
                width,
                height,
            }
        }
    }
}

impl<T> Copy for ImageViewPtr<T> {}

impl<T> Clone for ImageViewPtr<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Default for ImageViewPtr<T> {
    fn default() -> Self {
        Self {
            ptr: NonNull::dangling(),
            stride: Default::default(),
            width: Default::default(),
            height: Default::default(),
        }
    }
}

pub struct ImageView<'a, T>(ImageViewPtr<T>, PhantomData<&'a [T]>);

impl<'a, T> ImageView<'a, T> {
    pub fn width(&self) -> usize {
        self.0.width()
    }
    pub fn height(&self) -> usize {
        self.0.height()
    }
    pub fn size(&self) -> usize {
        self.0.size()
    }
    pub fn stride(&self) -> usize {
        self.0.stride()
    }
    pub fn ptr(&self) -> NonNull<T> {
        self.0.ptr()
    }

    pub unsafe fn unchecked_get(&self, x: usize, y: usize) -> &T {
        unsafe { self.0.unchecked_get(x, y).as_ref() }
    }
    pub fn checked_get(&self, x: usize, y: usize) -> Option<&T> {
        self.0.checked_get(x, y).map(|ptr| unsafe { ptr.as_ref() })
    }
    pub fn get(&self, x: usize, y: usize) -> &T {
        unsafe { self.0.get(x, y).as_ref() }
    }

    pub unsafe fn unchecked_row(&self, y: usize) -> &[T] {
        unsafe { self.0.unchecked_row(y).as_ref() }
    }
    pub fn checked_row(&self, y: usize) -> Option<&[T]> {
        self.0.checked_row(y).map(|ptr| unsafe { ptr.as_ref() })
    }
    pub fn row(&self, y: usize) -> &[T] {
        unsafe { self.0.row(y).as_ref() }
    }

    pub unsafe fn unchecked_col(&self, x: usize) -> StridedSlice<'_, T> {
        unsafe { self.0.unchecked_col(x).as_strided_slice() }
    }
    pub fn checked_col(&self, x: usize) -> Option<StridedSlice<'_, T>> {
        self.0
            .checked_col(x)
            .map(|ptr| unsafe { ptr.as_strided_slice() })
    }
    pub fn col(&self, x: usize) -> StridedSlice<'_, T> {
        unsafe { self.0.col(x).as_strided_slice() }
    }

    pub fn into_rows(self) -> ImageRowIter<'a, T> {
        ImageRowIter(self.0.rows(), PhantomData)
    }
    pub fn into_cols(self) -> ImageColIter<'a, T> {
        ImageColIter(self.0.cols(), PhantomData)
    }
    pub fn rows(&self) -> ImageRowIter<'_, T> {
        ImageRowIter(self.0.rows(), PhantomData)
    }
    pub fn cols(&self) -> ImageColIter<'_, T> {
        ImageColIter(self.0.cols(), PhantomData)
    }

    pub fn for_each(&self, mut f: impl FnMut(usize, usize, &T)) {
        for (y, row) in self.rows().enumerate() {
            for (x, cell) in row.iter().enumerate() {
                f(x, y, cell)
            }
        }
    }

    pub fn view(&self) -> ImageView<'_, T> {
        ImageView(self.0, PhantomData)
    }
    pub fn into_subview(self, x: usize, y: usize, width: usize, height: usize) -> Self {
        ImageView(self.0.subview(x, y, width, height), PhantomData)
    }
    pub fn subview(&self, x: usize, y: usize, width: usize, height: usize) -> ImageView<'_, T> {
        ImageView(self.0.subview(x, y, width, height), PhantomData)
    }
}

unsafe impl<T: Sync> Send for ImageView<'_, T> {}
unsafe impl<T: Sync> Sync for ImageView<'_, T> {}

impl<T> Default for ImageView<'_, T> {
    fn default() -> Self {
        ImageView(ImageViewPtr::default(), PhantomData)
    }
}
impl<T> Copy for ImageView<'_, T> {}
impl<T> Clone for ImageView<'_, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, T> From<ImageViewMut<'a, T>> for ImageView<'a, T> {
    fn from(value: ImageViewMut<'a, T>) -> Self {
        Self(value.0, PhantomData)
    }
}

pub struct ImageViewMut<'a, T>(ImageViewPtr<T>, PhantomData<&'a mut [T]>);

impl<'a, T> ImageViewMut<'a, T> {
    pub fn width(&self) -> usize {
        self.0.width()
    }
    pub fn height(&self) -> usize {
        self.0.height()
    }
    pub fn size(&self) -> usize {
        self.0.size()
    }
    pub fn stride(&self) -> usize {
        self.0.stride()
    }
    pub fn ptr(&self) -> NonNull<T> {
        self.0.ptr()
    }

    pub unsafe fn unchecked_get(&self, x: usize, y: usize) -> &T {
        unsafe { self.0.unchecked_get(x, y).as_ref() }
    }
    pub fn checked_get(&self, x: usize, y: usize) -> Option<&T> {
        self.0.checked_get(x, y).map(|ptr| unsafe { ptr.as_ref() })
    }
    pub fn get(&self, x: usize, y: usize) -> &T {
        unsafe { self.0.get(x, y).as_ref() }
    }

    pub unsafe fn unchecked_get_mut(&mut self, x: usize, y: usize) -> &mut T {
        unsafe { self.0.unchecked_get(x, y).as_mut() }
    }
    pub fn checked_get_mut(&mut self, x: usize, y: usize) -> Option<&mut T> {
        self.0
            .checked_get(x, y)
            .map(|mut ptr| unsafe { ptr.as_mut() })
    }
    pub fn get_mut(&mut self, x: usize, y: usize) -> &mut T {
        unsafe { self.0.get(x, y).as_mut() }
    }

    pub unsafe fn unchecked_row(&self, y: usize) -> &[T] {
        unsafe { self.0.unchecked_row(y).as_ref() }
    }
    pub fn checked_row(&self, y: usize) -> Option<&[T]> {
        self.0.checked_row(y).map(|ptr| unsafe { ptr.as_ref() })
    }
    pub fn row(&self, y: usize) -> &[T] {
        unsafe { self.0.row(y).as_ref() }
    }

    pub unsafe fn unchecked_row_mut(&mut self, y: usize) -> &mut [T] {
        unsafe { self.0.unchecked_row(y).as_mut() }
    }
    pub fn checked_row_mut(&mut self, y: usize) -> Option<&mut [T]> {
        self.0.checked_row(y).map(|mut ptr| unsafe { ptr.as_mut() })
    }
    pub fn row_mut(&mut self, y: usize) -> &mut [T] {
        unsafe { self.0.row(y).as_mut() }
    }

    pub unsafe fn unchecked_col(&self, x: usize) -> StridedSlice<'_, T> {
        unsafe { self.0.unchecked_col(x).as_strided_slice() }
    }
    pub fn checked_col(&self, x: usize) -> Option<StridedSlice<'_, T>> {
        self.0
            .checked_col(x)
            .map(|ptr| unsafe { ptr.as_strided_slice() })
    }
    pub fn col(&self, x: usize) -> StridedSlice<'_, T> {
        unsafe { self.0.col(x).as_strided_slice() }
    }

    pub unsafe fn unchecked_col_mut(&mut self, x: usize) -> StridedSliceMut<'_, T> {
        unsafe { self.0.unchecked_col(x).as_strided_slice_mut() }
    }
    pub fn checked_col_mut(&mut self, x: usize) -> Option<StridedSliceMut<'_, T>> {
        self.0
            .checked_col(x)
            .map(|ptr| unsafe { ptr.as_strided_slice_mut() })
    }
    pub fn col_mut(&mut self, x: usize) -> StridedSliceMut<'_, T> {
        unsafe { self.0.col(x).as_strided_slice_mut() }
    }

    pub fn into_rows(self) -> ImageRowIter<'a, T> {
        ImageRowIter(self.0.rows(), PhantomData)
    }
    pub fn into_cols(self) -> ImageColIter<'a, T> {
        ImageColIter(self.0.cols(), PhantomData)
    }
    pub fn rows(&self) -> ImageRowIter<'_, T> {
        ImageRowIter(self.0.rows(), PhantomData)
    }
    pub fn cols(&self) -> ImageColIter<'_, T> {
        ImageColIter(self.0.cols(), PhantomData)
    }

    pub fn into_rows_mut(self) -> ImageRowIterMut<'a, T> {
        ImageRowIterMut(self.0.rows(), PhantomData)
    }
    pub fn into_cols_mut(self) -> ImageColIterMut<'a, T> {
        ImageColIterMut(self.0.cols(), PhantomData)
    }
    pub fn rows_mut(&mut self) -> ImageRowIterMut<'_, T> {
        ImageRowIterMut(self.0.rows(), PhantomData)
    }
    pub fn cols_mut(&mut self) -> ImageColIterMut<'_, T> {
        ImageColIterMut(self.0.cols(), PhantomData)
    }

    pub fn for_each(&self, mut f: impl FnMut(usize, usize, &T)) {
        for (y, row) in self.rows().enumerate() {
            for (x, cell) in row.iter().enumerate() {
                f(x, y, cell)
            }
        }
    }
    pub fn for_each_mut(&mut self, mut f: impl FnMut(usize, usize, &mut T)) {
        for (y, row) in self.rows_mut().enumerate() {
            for (x, cell) in row.iter_mut().enumerate() {
                f(x, y, cell)
            }
        }
    }

    pub fn view(&self) -> ImageView<'_, T> {
        ImageView(self.0, PhantomData)
    }
    pub fn into_subview(self, x: usize, y: usize, width: usize, height: usize) -> ImageView<'a, T> {
        ImageView(self.0.subview(x, y, width, height), PhantomData)
    }
    pub fn subview(&self, x: usize, y: usize, width: usize, height: usize) -> ImageView<'_, T> {
        ImageView(self.0.subview(x, y, width, height), PhantomData)
    }

    pub fn view_mut(&mut self) -> ImageViewMut<'_, T> {
        ImageViewMut(self.0, PhantomData)
    }
    pub fn into_subview_mut(
        self,
        x: usize,
        y: usize,
        width: usize,
        height: usize,
    ) -> ImageViewMut<'a, T> {
        ImageViewMut(self.0.subview(x, y, width, height), PhantomData)
    }
    pub fn subview_mut(
        &mut self,
        x: usize,
        y: usize,
        width: usize,
        height: usize,
    ) -> ImageViewMut<'_, T> {
        ImageViewMut(self.0.subview(x, y, width, height), PhantomData)
    }
}

unsafe impl<T: Sync> Send for ImageViewMut<'_, T> {}
unsafe impl<T: Sync> Sync for ImageViewMut<'_, T> {}

impl<T> Default for ImageViewMut<'_, T> {
    fn default() -> Self {
        Self(ImageViewPtr::default(), PhantomData)
    }
}

pub struct Image<T>(ImageViewPtr<T>, PhantomData<Box<[T]>>);

impl<T> Image<T> {
    fn new_uninit_or_zeroed(
        width: usize,
        height: usize,
        stride: usize,
        zeroed: bool,
    ) -> Image<MaybeUninit<T>> {
        let size = stride * height;
        let align = std::mem::align_of::<T>();
        if stride % align != 0 {
            panic!("Stride {stride} is invalid for alignment {align}");
        }
        if stride < width * std::mem::size_of::<T>() {
            panic!(
                "Stride {stride} is less than width {width} * {}",
                std::mem::size_of::<T>()
            );
        }
        unsafe {
            let ptr = if size == 0 {
                NonNull::dangling()
            } else {
                let layout = Layout::from_size_align_unchecked(size, align);
                let ptr = if zeroed {
                    std::alloc::alloc(layout)
                } else {
                    std::alloc::alloc_zeroed(layout)
                } as *mut MaybeUninit<T>;

                if ptr.is_null() {
                    std::alloc::handle_alloc_error(layout);
                }

                NonNull::new_unchecked(ptr)
            };

            Image(
                ImageViewPtr {
                    ptr,
                    stride,
                    width,
                    height,
                },
                PhantomData,
            )
        }
    }

    pub fn new_uninit(width: usize, height: usize, stride: usize) -> Image<MaybeUninit<T>> {
        Self::new_uninit_or_zeroed(width, height, stride, false)
    }
    pub fn new_zeroed(width: usize, height: usize, stride: usize) -> Image<MaybeUninit<T>> {
        Self::new_uninit_or_zeroed(width, height, stride, true)
    }
    pub fn with_stride_and_fn(
        width: usize,
        height: usize,
        stride: usize,
        mut f: impl FnMut(usize, usize) -> T,
    ) -> Self {
        unsafe {
            let mut image = Self::new_uninit(width, height, stride);

            for (y, row) in image.rows_mut().enumerate() {
                for (x, cell) in row.iter_mut().enumerate() {
                    cell.write(f(x, y));
                }
            }

            image.assume_init()
        }
    }

    pub fn with_stride_and_value(width: usize, height: usize, stride: usize, value: &T) -> Self
    where
        T: Clone,
    {
        Self::with_stride_and_fn(width, height, stride, |_, _| value.clone())
    }
    pub fn with_stride(width: usize, height: usize, stride: usize) -> Self
    where
        T: Default,
    {
        Self::with_stride_and_fn(width, height, stride, |_, _| Default::default())
    }

    pub fn with_fn(width: usize, height: usize, f: impl Fn(usize, usize) -> T) -> Self {
        Self::with_stride_and_fn(width, height, width * std::mem::size_of::<T>(), f)
    }
    pub fn with_value(width: usize, height: usize, value: &T) -> Self
    where
        T: Clone,
    {
        Self::with_stride_and_value(width, height, width * std::mem::size_of::<T>(), value)
    }
    pub fn new(width: usize, height: usize) -> Self
    where
        T: Default,
    {
        Self::with_stride(width, height, width * std::mem::size_of::<T>())
    }

    pub fn width(&self) -> usize {
        self.0.width()
    }
    pub fn height(&self) -> usize {
        self.0.height()
    }
    pub fn size(&self) -> usize {
        self.0.size()
    }
    pub fn stride(&self) -> usize {
        self.0.stride()
    }
    pub fn ptr(&self) -> NonNull<T> {
        self.0.ptr()
    }

    pub unsafe fn unchecked_get(&self, x: usize, y: usize) -> &T {
        unsafe { self.0.unchecked_get(x, y).as_ref() }
    }
    pub fn checked_get(&self, x: usize, y: usize) -> Option<&T> {
        self.0.checked_get(x, y).map(|ptr| unsafe { ptr.as_ref() })
    }
    pub fn get(&self, x: usize, y: usize) -> &T {
        unsafe { self.0.get(x, y).as_ref() }
    }

    pub unsafe fn unchecked_get_mut(&mut self, x: usize, y: usize) -> &mut T {
        unsafe { self.0.unchecked_get(x, y).as_mut() }
    }
    pub fn checked_get_mut(&mut self, x: usize, y: usize) -> Option<&mut T> {
        self.0
            .checked_get(x, y)
            .map(|mut ptr| unsafe { ptr.as_mut() })
    }
    pub fn get_mut(&mut self, x: usize, y: usize) -> &mut T {
        unsafe { self.0.get(x, y).as_mut() }
    }

    pub unsafe fn unchecked_row(&self, y: usize) -> &[T] {
        unsafe { self.0.unchecked_row(y).as_ref() }
    }
    pub fn checked_row(&self, y: usize) -> Option<&[T]> {
        self.0.checked_row(y).map(|ptr| unsafe { ptr.as_ref() })
    }
    pub fn row(&self, y: usize) -> &[T] {
        unsafe { self.0.row(y).as_ref() }
    }

    pub unsafe fn unchecked_row_mut(&mut self, y: usize) -> &mut [T] {
        unsafe { self.0.unchecked_row(y).as_mut() }
    }
    pub fn checked_row_mut(&mut self, y: usize) -> Option<&mut [T]> {
        self.0.checked_row(y).map(|mut ptr| unsafe { ptr.as_mut() })
    }
    pub fn row_mut(&mut self, y: usize) -> &mut [T] {
        unsafe { self.0.row(y).as_mut() }
    }

    pub unsafe fn unchecked_col(&self, x: usize) -> StridedSlice<'_, T> {
        unsafe { self.0.unchecked_col(x).as_strided_slice() }
    }
    pub fn checked_col(&self, x: usize) -> Option<StridedSlice<'_, T>> {
        self.0
            .checked_col(x)
            .map(|ptr| unsafe { ptr.as_strided_slice() })
    }
    pub fn col(&self, x: usize) -> StridedSlice<'_, T> {
        unsafe { self.0.col(x).as_strided_slice() }
    }

    pub unsafe fn unchecked_col_mut(&mut self, x: usize) -> StridedSliceMut<'_, T> {
        unsafe { self.0.unchecked_col(x).as_strided_slice_mut() }
    }
    pub fn checked_col_mut(&mut self, x: usize) -> Option<StridedSliceMut<'_, T>> {
        self.0
            .checked_col(x)
            .map(|ptr| unsafe { ptr.as_strided_slice_mut() })
    }
    pub fn col_mut(&mut self, x: usize) -> StridedSliceMut<'_, T> {
        unsafe { self.0.col(x).as_strided_slice_mut() }
    }

    pub fn rows(&self) -> ImageRowIter<'_, T> {
        ImageRowIter(self.0.rows(), PhantomData)
    }
    pub fn cols(&self) -> ImageColIter<'_, T> {
        ImageColIter(self.0.cols(), PhantomData)
    }

    pub fn rows_mut(&mut self) -> ImageRowIterMut<'_, T> {
        ImageRowIterMut(self.0.rows(), PhantomData)
    }
    pub fn cols_mut(&mut self) -> ImageColIterMut<'_, T> {
        ImageColIterMut(self.0.cols(), PhantomData)
    }

    pub fn for_each(&self, mut f: impl FnMut(usize, usize, &T)) {
        for (y, row) in self.rows().enumerate() {
            for (x, cell) in row.iter().enumerate() {
                f(x, y, cell)
            }
        }
    }
    pub fn for_each_mut(&mut self, mut f: impl FnMut(usize, usize, &mut T)) {
        for (y, row) in self.rows_mut().enumerate() {
            for (x, cell) in row.iter_mut().enumerate() {
                f(x, y, cell)
            }
        }
    }

    pub fn view(&self) -> ImageView<'_, T> {
        ImageView(self.0, PhantomData)
    }
    pub fn subview(&self, x: usize, y: usize, width: usize, height: usize) -> ImageView<'_, T> {
        ImageView(self.0.subview(x, y, width, height), PhantomData)
    }

    pub fn view_mut(&mut self) -> ImageViewMut<'_, T> {
        ImageViewMut(self.0, PhantomData)
    }
    pub fn subview_mut(
        &mut self,
        x: usize,
        y: usize,
        width: usize,
        height: usize,
    ) -> ImageViewMut<'_, T> {
        ImageViewMut(self.0.subview(x, y, width, height), PhantomData)
    }
}

impl<T> Image<MaybeUninit<T>> {
    pub unsafe fn assume_init(self) -> Image<T> {
        let image = Image(self.0.cast(), PhantomData);
        std::mem::forget(self);
        image
    }
}

impl<T> Drop for Image<T> {
    fn drop(&mut self) {
        unsafe {
            for row in self.0.rows() {
                for cell in row {
                    cell.drop_in_place();
                }
            }

            let size = self.stride().unchecked_mul(self.height());
            let align = std::mem::align_of::<T>();
            let layout = Layout::from_size_align_unchecked(size, align);

            if size != 0 {
                std::alloc::dealloc(self.ptr().as_ptr() as *mut u8, layout);
            }
        }
    }
}

unsafe impl<T: Sync> Send for Image<T> {}
unsafe impl<T: Sync> Sync for Image<T> {}

impl<T> Default for Image<T> {
    fn default() -> Self {
        Self(ImageViewPtr::default(), PhantomData)
    }
}

impl<T: Clone> Clone for Image<T> {
    fn clone(&self) -> Self {
        let mut image = Self::new_uninit(self.width(), self.height(), self.stride());

        for (row_dst, row_src) in image.rows_mut().zip(self.view().rows()) {
            for (dst, src) in row_dst.iter_mut().zip(row_src) {
                dst.write(src.clone());
            }
        }

        unsafe { image.assume_init() }
    }
}

#[cfg(test)]
mod test {
    use super::Image;

    #[test]
    fn set() {
        let mut image = Image::with_stride(2, 2, 12);
        *image.get_mut(0, 0) = 1;
        *image.get_mut(0, 1) = 2;
        *image.get_mut(1, 0) = 3;
        *image.get_mut(1, 1) = 4;

        image.for_each(|x, y, value| match (x, y) {
            (0, 0) => assert_eq!(*value, 1),
            (0, 1) => assert_eq!(*value, 2),
            (1, 0) => assert_eq!(*value, 3),
            (1, 1) => assert_eq!(*value, 4),
            _ => (),
        });
    }

    #[test]
    fn init() {
        let image = Image::with_stride_and_fn(2, 2, 32, |x, y| y * 10 + x);

        image.for_each(|x, y, value| {
            assert_eq!(*value % 10, x);
            assert_eq!(*value / 10, y);
        });
    }

    #[test]
    fn empty() {
        let images = [
            Image::<u8>::new(0, 0),
            Image::<u8>::new(1, 0),
            Image::<u8>::new(0, 1),
            Image::<u8>::with_stride(0, 0, 1),
            Image::<u8>::with_stride(1, 0, 1),
            Image::<u8>::with_stride(0, 1, 1),
        ];
        for image in images {
            assert_eq!(image.size(), 0);
        }
    }

    #[test]
    fn oob() {
        let mut image = Image::<u8>::new(10, 10);
        assert!(image.checked_get(10, 0).is_none());
        assert!(image.checked_get(0, 10).is_none());
        assert!(image.checked_get(10, 10).is_none());
        assert!(image.checked_get_mut(10, 0).is_none());
        assert!(image.checked_get_mut(0, 10).is_none());
        assert!(image.checked_get_mut(10, 10).is_none());
        assert!(image.checked_row(10).is_none());
        assert!(image.checked_row_mut(10).is_none());
        assert!(image.checked_col(10).is_none());
        assert!(image.checked_col_mut(10).is_none());
    }

    #[test]
    #[should_panic]
    fn oob_x() {
        let image = Image::<u8>::new(10, 10);
        image.get(10, 0);
    }
    #[test]
    #[should_panic]
    fn oob_y() {
        let image = Image::<u8>::new(10, 10);
        image.get(0, 10);
    }
    #[test]
    #[should_panic]
    fn oob_xy() {
        let image = Image::<u8>::new(10, 10);
        image.get(10, 10);
    }

    #[test]
    #[should_panic]
    fn oob_mut_x() {
        let mut image = Image::<u8>::new(10, 10);
        image.get_mut(10, 0);
    }
    #[test]
    #[should_panic]
    fn oob_mut_y() {
        let mut image = Image::<u8>::new(10, 10);
        image.get_mut(0, 10);
    }
    #[test]
    #[should_panic]
    fn oob_mut_xy() {
        let mut image = Image::<u8>::new(10, 10);
        image.get_mut(10, 10);
    }

    #[test]
    #[should_panic]
    fn oob_row() {
        let image = Image::<u8>::new(10, 10);
        image.row(10);
    }
    #[test]
    #[should_panic]
    fn oob_col() {
        let image = Image::<u8>::new(10, 10);
        image.col(10);
    }
    #[test]
    #[should_panic]
    fn oob_mut_row() {
        let mut image = Image::<u8>::new(10, 10);
        image.row_mut(10);
    }
    #[test]
    #[should_panic]
    fn oob_mut_col() {
        let mut image = Image::<u8>::new(10, 10);
        image.col_mut(10);
    }

    #[test]
    #[should_panic]
    fn unaligned_stride() {
        Image::<u32>::with_stride(1, 1, 6);
    }

    #[test]
    #[should_panic]
    fn small_stride() {
        Image::<u32>::with_stride(2, 1, 4);
    }

    #[test]
    #[should_panic]
    fn very_large() {
        Image::<u8>::new(usize::MAX / 8, usize::MAX / 8);
    }
    #[test]
    #[should_panic]
    fn very_large_row() {
        Image::<[u8; 16]>::new(usize::MAX / 8, 1);
    }
    #[test]
    #[should_panic]
    fn very_large_stride() {
        Image::<u8>::with_stride(1, usize::MAX / 8, usize::MAX / 8);
    }
}
