#![allow(clippy::missing_safety_doc)]

use std::{alloc::Layout, mem::MaybeUninit, ptr::NonNull};

use super::{
    slice::SlicePtr,
    strided::{self, StridedState},
    Strided,
};

#[derive(Debug, Default, Clone, Copy, Eq, PartialEq, PartialOrd, Ord)]
pub struct ImageView<'a, T>(Strided<&'a [T]>);

#[derive(Debug, Default, Eq, PartialEq, PartialOrd, Ord)]
pub struct ImageViewMut<'a, T>(Strided<&'a mut [T]>);

#[derive(Debug, Default, Eq, PartialEq, PartialOrd, Ord)]
pub struct Image<T>(Strided<strided::Owned<[T]>>);

impl<'a, T> ImageView<'a, T> {
    pub fn width(&self) -> usize {
        self.0.state().inner
    }
    pub fn height(&self) -> usize {
        self.0.len()
    }
    pub fn size(&self) -> usize {
        self.0.total_size()
    }
    pub fn stride(&self) -> usize {
        self.0.stride() as usize
    }
    pub fn ptr(&self) -> NonNull<T> {
        self.0.as_non_null_ptr()
    }
    pub unsafe fn cast<U>(&self) -> ImageView<'a, U> {
        ImageView(unsafe { self.0.cast_as() })
    }

    pub fn as_matrix(&self) -> Strided<Strided<&'a T>> {
        self.0.into()
    }

    pub unsafe fn unchecked_get(&self, x: usize, y: usize) -> &T {
        unsafe { self.0.unchecked_get(y).get_unchecked(x) }
    }
    pub fn checked_get(&self, x: usize, y: usize) -> Option<&T> {
        self.0.checked_get(y).and_then(|row| row.get(x))
    }
    pub fn get(&self, x: usize, y: usize) -> &T {
        &self.0.get(y)[x]
    }

    pub unsafe fn unchecked_row(&self, y: usize) -> &[T] {
        unsafe { self.0.unchecked_get(y) }
    }
    pub fn checked_row(&self, y: usize) -> Option<&[T]> {
        self.0.checked_get(y)
    }
    pub fn row(&self, y: usize) -> &[T] {
        self.0.get(y)
    }

    pub unsafe fn unchecked_col(&self, x: usize) -> Strided<&T> {
        unsafe { self.as_matrix().into_transpose01().unchecked_into_get(x) }
    }
    pub fn checked_col(&self, x: usize) -> Option<Strided<&T>> {
        self.as_matrix().into_transpose01().checked_into_get(x)
    }
    pub fn col(&self, x: usize) -> Strided<&'_ T> {
        self.as_matrix().into_transpose01().into_get(x)
    }

    pub fn into_rows(self) -> strided::Iter<&'a [T]> {
        self.0.into_iter()
    }
    pub fn into_cols(self) -> strided::Iter<Strided<&'a T>> {
        self.as_matrix().into_transpose01().into_iter()
    }
    pub fn rows(&self) -> strided::Iter<&'_ [T]> {
        self.0.iter()
    }
    pub fn cols(&self) -> strided::Iter<Strided<&'_ T>> {
        self.as_matrix().into_transpose01().into_iter()
    }

    pub fn for_each(&self, mut f: impl FnMut(usize, usize, &T)) {
        for (y, row) in self.rows().enumerate() {
            for (x, cell) in row.iter().enumerate() {
                f(x, y, cell)
            }
        }
    }

    pub fn view(&self) -> ImageView<'_, T> {
        ImageView(self.0)
    }

    pub fn into_subview(self, x: usize, y: usize, width: usize, height: usize) -> Self {
        let mut matrix = self.as_matrix();
        matrix = matrix.into_partial(x, height, strided::STEP_1);
        matrix.transpose01();
        matrix = matrix.into_partial(y, width, strided::STEP_1);
        matrix.transpose01();

        Self(unsafe { matrix.try_into().unwrap_unchecked() })
    }

    pub fn subview(&self, x: usize, y: usize, width: usize, height: usize) -> ImageView<'_, T> {
        self.view().into_subview(x, y, width, height)
    }
}

impl<'a, T> ImageViewMut<'a, T> {
    pub fn width(&self) -> usize {
        self.0.state().inner
    }
    pub fn height(&self) -> usize {
        self.0.len()
    }
    pub fn size(&self) -> usize {
        self.0.total_size()
    }
    pub fn stride(&self) -> usize {
        self.0.stride() as usize
    }
    pub fn ptr(&self) -> NonNull<T> {
        self.0.as_non_null_ptr()
    }
    pub unsafe fn cast<U>(&self) -> ImageViewMut<'a, U> {
        ImageViewMut(unsafe { self.0.cast_as() })
    }

    pub fn as_matrix(&self) -> Strided<Strided<&'_ T>> {
        self.0.borrow().into()
    }
    pub fn as_matrix_mut(&mut self) -> Strided<Strided<&'_ mut T>> {
        self.0.borrow_mut().into()
    }
    pub fn into_matrix_mut(self) -> Strided<Strided<&'a mut T>> {
        self.0.into()
    }

    pub unsafe fn unchecked_get(&self, x: usize, y: usize) -> &T {
        unsafe { self.0.unchecked_get(y).get_unchecked(x) }
    }
    pub fn checked_get(&self, x: usize, y: usize) -> Option<&T> {
        self.0.checked_get(y).and_then(|row| row.get(x))
    }
    pub fn get(&self, x: usize, y: usize) -> &T {
        &self.0.get(y)[x]
    }

    pub unsafe fn unchecked_get_mut(&mut self, x: usize, y: usize) -> &mut T {
        unsafe { self.0.unchecked_get_mut(y).get_unchecked_mut(x) }
    }
    pub fn checked_get_mut(&mut self, x: usize, y: usize) -> Option<&mut T> {
        self.0.checked_get_mut(y).and_then(|row| row.get_mut(x))
    }
    pub fn get_mut(&mut self, x: usize, y: usize) -> &mut T {
        &mut self.0.get_mut(y)[x]
    }

    pub unsafe fn unchecked_row(&self, y: usize) -> &[T] {
        unsafe { self.0.unchecked_get(y) }
    }
    pub fn checked_row(&self, y: usize) -> Option<&[T]> {
        self.0.checked_get(y)
    }
    pub fn row(&self, y: usize) -> &[T] {
        self.0.get(y)
    }

    pub unsafe fn unchecked_row_mut(&mut self, y: usize) -> &mut [T] {
        unsafe { self.0.unchecked_get_mut(y) }
    }
    pub fn checked_row_mut(&mut self, y: usize) -> Option<&mut [T]> {
        self.0.checked_get_mut(y)
    }
    pub fn row_mut(&mut self, y: usize) -> &mut [T] {
        self.0.get_mut(y)
    }

    pub unsafe fn unchecked_col(&self, x: usize) -> Strided<&T> {
        unsafe { self.as_matrix().into_transpose01().unchecked_into_get(x) }
    }
    pub fn checked_col(&self, x: usize) -> Option<Strided<&T>> {
        self.as_matrix().into_transpose01().checked_into_get(x)
    }
    pub fn col(&self, x: usize) -> Strided<&'_ T> {
        self.as_matrix().into_transpose01().into_get(x)
    }

    pub unsafe fn unchecked_col_mut(&mut self, x: usize) -> Strided<&mut T> {
        unsafe {
            self.as_matrix_mut()
                .into_transpose01()
                .unchecked_into_get(x)
        }
    }
    pub fn checked_col_mut(&mut self, x: usize) -> Option<Strided<&mut T>> {
        self.as_matrix_mut().into_transpose01().checked_into_get(x)
    }
    pub fn col_mut(&mut self, x: usize) -> Strided<&'_ mut T> {
        self.as_matrix_mut().into_transpose01().into_get(x)
    }

    pub fn into_rows(self) -> strided::Iter<&'a [T]> {
        self.0.into_borrow().into_iter()
    }
    pub fn into_cols(self) -> strided::Iter<Strided<&'a T>> {
        self.into_matrix_mut()
            .into_borrow()
            .into_transpose01()
            .into_iter()
    }
    pub fn rows(&self) -> strided::Iter<&'_ [T]> {
        self.0.iter()
    }
    pub fn cols(&self) -> strided::Iter<Strided<&'_ T>> {
        self.as_matrix().into_transpose01().into_iter()
    }

    pub fn into_rows_mut(self) -> strided::Iter<&'a mut [T]> {
        self.0.into_iter()
    }
    pub fn into_cols_mut(self) -> strided::Iter<Strided<&'a mut T>> {
        self.into_matrix_mut().into_transpose01().into_iter()
    }
    pub fn rows_mut(&mut self) -> strided::Iter<&'_ mut [T]> {
        self.0.iter_mut()
    }
    pub fn cols_mut(&mut self) -> strided::Iter<Strided<&'_ mut T>> {
        self.as_matrix_mut().into_transpose01().into_iter()
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
        ImageView(self.0.borrow())
    }
    pub fn view_mut(&mut self) -> ImageViewMut<'_, T> {
        ImageViewMut(self.0.borrow_mut())
    }

    pub fn into_subview_mut(self, x: usize, y: usize, width: usize, height: usize) -> Self {
        let mut matrix = self.into_matrix_mut();
        matrix = matrix.into_partial(x, height, strided::STEP_1);
        matrix.transpose01();
        matrix = matrix.into_partial(y, width, strided::STEP_1);
        matrix.transpose01();

        Self(unsafe { matrix.try_into().unwrap_unchecked() })
    }
    pub fn subview_mut(
        &mut self,
        x: usize,
        y: usize,
        width: usize,
        height: usize,
    ) -> ImageViewMut<'_, T> {
        self.view_mut().into_subview_mut(x, y, width, height)
    }

    pub fn into_subview(self, x: usize, y: usize, width: usize, height: usize) -> ImageView<'a, T> {
        ImageView(self.0.into_borrow()).into_subview(x, y, width, height)
    }

    pub fn subview(&self, x: usize, y: usize, width: usize, height: usize) -> ImageView<'_, T> {
        self.view().into_subview(x, y, width, height)
    }
}

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

            Image(Strided::from_raw_parts(
                ptr.cast(),
                StridedState {
                    len: height,
                    stride: stride as isize,
                    inner: width,
                },
            ))
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
        self.0.state().inner
    }
    pub fn height(&self) -> usize {
        self.0.len()
    }
    pub fn size(&self) -> usize {
        self.0.total_size()
    }
    pub fn stride(&self) -> usize {
        self.0.stride() as usize
    }
    pub fn ptr(&self) -> NonNull<T> {
        self.0.as_non_null_ptr()
    }
    pub unsafe fn cast<U>(self) -> Image<U> {
        unsafe {
            let casted = self.0.cast_as();
            std::mem::forget(self);
            Image(casted)
        }
    }

    pub fn as_matrix(&self) -> Strided<Strided<&'_ T>> {
        self.0.borrow().into()
    }
    pub fn as_matrix_mut(&mut self) -> Strided<Strided<&'_ mut T>> {
        self.0.borrow_mut().into()
    }

    pub unsafe fn unchecked_get(&self, x: usize, y: usize) -> &T {
        unsafe { self.0.unchecked_get(y).get_unchecked(x) }
    }
    pub fn checked_get(&self, x: usize, y: usize) -> Option<&T> {
        self.0.checked_get(y).and_then(|row| row.get(x))
    }
    pub fn get(&self, x: usize, y: usize) -> &T {
        &self.0.get(y)[x]
    }

    pub unsafe fn unchecked_get_mut(&mut self, x: usize, y: usize) -> &mut T {
        unsafe { self.0.unchecked_get_mut(y).get_unchecked_mut(x) }
    }
    pub fn checked_get_mut(&mut self, x: usize, y: usize) -> Option<&mut T> {
        self.0.checked_get_mut(y).and_then(|row| row.get_mut(x))
    }
    pub fn get_mut(&mut self, x: usize, y: usize) -> &mut T {
        &mut self.0.get_mut(y)[x]
    }

    pub unsafe fn unchecked_row(&self, y: usize) -> &[T] {
        unsafe { self.0.unchecked_get(y) }
    }
    pub fn checked_row(&self, y: usize) -> Option<&[T]> {
        self.0.checked_get(y)
    }
    pub fn row(&self, y: usize) -> &[T] {
        self.0.get(y)
    }

    pub unsafe fn unchecked_row_mut(&mut self, y: usize) -> &mut [T] {
        unsafe { self.0.unchecked_get_mut(y) }
    }
    pub fn checked_row_mut(&mut self, y: usize) -> Option<&mut [T]> {
        self.0.checked_get_mut(y)
    }
    pub fn row_mut(&mut self, y: usize) -> &mut [T] {
        self.0.get_mut(y)
    }

    pub unsafe fn unchecked_col(&self, x: usize) -> Strided<&T> {
        unsafe { self.as_matrix().into_transpose01().unchecked_into_get(x) }
    }
    pub fn checked_col(&self, x: usize) -> Option<Strided<&T>> {
        self.as_matrix().into_transpose01().checked_into_get(x)
    }
    pub fn col(&self, x: usize) -> Strided<&'_ T> {
        self.as_matrix().into_transpose01().into_get(x)
    }

    pub unsafe fn unchecked_col_mut(&mut self, x: usize) -> Strided<&mut T> {
        unsafe {
            self.as_matrix_mut()
                .into_transpose01()
                .unchecked_into_get(x)
        }
    }
    pub fn checked_col_mut(&mut self, x: usize) -> Option<Strided<&mut T>> {
        self.as_matrix_mut().into_transpose01().checked_into_get(x)
    }
    pub fn col_mut(&mut self, x: usize) -> Strided<&'_ mut T> {
        self.as_matrix_mut().into_transpose01().into_get(x)
    }

    pub fn rows(&self) -> strided::Iter<&'_ [T]> {
        self.0.iter()
    }
    pub fn cols(&self) -> strided::Iter<Strided<&'_ T>> {
        self.as_matrix().into_transpose01().into_iter()
    }

    pub fn rows_mut(&mut self) -> strided::Iter<&'_ mut [T]> {
        self.0.iter_mut()
    }
    pub fn cols_mut(&mut self) -> strided::Iter<Strided<&'_ mut T>> {
        self.as_matrix_mut().into_transpose01().into_iter()
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
        ImageView(self.0.borrow())
    }
    pub fn view_mut(&mut self) -> ImageViewMut<'_, T> {
        ImageViewMut(self.0.borrow_mut())
    }

    pub fn subview_mut(
        &mut self,
        x: usize,
        y: usize,
        width: usize,
        height: usize,
    ) -> ImageViewMut<'_, T> {
        self.view_mut().into_subview_mut(x, y, width, height)
    }

    pub fn subview(&self, x: usize, y: usize, width: usize, height: usize) -> ImageView<'_, T> {
        self.view().into_subview(x, y, width, height)
    }
}

impl<T> Image<MaybeUninit<T>> {
    pub unsafe fn assume_init(self) -> Image<T> {
        unsafe { self.cast() }
    }
}

impl<T> Drop for Image<T> {
    fn drop(&mut self) {
        unsafe {
            for row in self.0.cast_as::<NonNull<[T]>>() {
                for cell in SlicePtr::<T>::from(row) {
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
