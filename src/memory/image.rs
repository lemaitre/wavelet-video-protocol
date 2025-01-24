use std::{
    alloc::Layout,
    borrow::{Borrow, BorrowMut},
    ops::{Deref, DerefMut},
};

use super::{ptr_mut_offset, ptr_offset, StridedSlice, StridedSliceMut};

#[derive(Copy)]
#[repr(C)]
pub struct ImageView<'a, T> {
    width: usize,
    rows: StridedSlice<'a, T>,
}

impl<'a, T> ImageView<'a, T> {
    pub fn width(&self) -> usize {
        self.width
    }
    pub fn height(&self) -> usize {
        self.rows.len()
    }
    pub fn size(&self) -> usize {
        self.width() * self.height()
    }
    pub fn stride(&self) -> usize {
        self.rows.stride() as usize
    }

    pub unsafe fn unchecked_get(&self, x: usize, y: usize) -> &T {
        unsafe { self.unchecked_row(y).get_unchecked(x) }
    }
    pub fn checked_get(&self, x: usize, y: usize) -> Option<&T> {
        if x < self.width() && y < self.height() {
            Some(unsafe { self.unchecked_get(x, y) })
        } else {
            None
        }
    }
    pub fn get(&self, x: usize, y: usize) -> &T {
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

    pub unsafe fn unchecked_row(&self, y: usize) -> &[T] {
        unsafe {
            std::slice::from_raw_parts(
                ptr_offset(self.rows.as_ptr(), y, self.stride() as isize),
                self.width(),
            )
        }
    }
    pub fn checked_row(&self, y: usize) -> Option<&[T]> {
        if y < self.height() {
            Some(unsafe { self.unchecked_row(y) })
        } else {
            None
        }
    }
    pub fn row(&self, y: usize) -> &[T] {
        if y < self.height() {
            unsafe { self.unchecked_row(y) }
        } else {
            panic!(
                "Trying to access row {y} from an image with height {}",
                self.height()
            )
        }
    }

    pub unsafe fn unchecked_col(&self, x: usize) -> StridedSlice<'_, T> {
        unsafe {
            StridedSlice::from_raw_parts(
                ptr_offset(self.rows.as_ptr(), x, std::mem::size_of::<T>() as isize),
                self.stride() as isize,
                self.height(),
            )
        }
    }
    pub fn checked_col(&self, x: usize) -> Option<StridedSlice<'_, T>> {
        if x < self.height() {
            Some(unsafe { self.unchecked_col(x) })
        } else {
            None
        }
    }
    pub fn col(&self, x: usize) -> StridedSlice<'_, T> {
        if x < self.height() {
            unsafe { self.unchecked_col(x) }
        } else {
            panic!(
                "Trying to access column {x} from an image with width {}",
                self.height()
            )
        }
    }

    pub fn for_each(&self, mut f: impl FnMut(usize, usize, &T)) {
        for_each_ptr(
            self.rows.as_ptr(),
            self.width(),
            self.height(),
            self.stride(),
            move |x, y, ptr| f(x, y, unsafe { ptr.as_ref().unwrap_unchecked() }),
        );
    }

    pub fn view(&self) -> ImageView<'_, T> {
        self.subview(0, 0, self.width(), self.height())
    }
    pub fn into_subview(self, x: usize, y: usize, width: usize, height: usize) -> Self {
        let x = x.min(self.width());
        let y = y.min(self.height());
        let width = width.min(self.width().checked_sub(x).unwrap_or(0));
        let height = height.min(self.height().checked_sub(y).unwrap_or(0));

        unsafe {
            let ptr = self.rows.as_ptr();
            let ptr = ptr_offset(ptr, y, self.stride() as isize);
            let ptr = ptr_offset(ptr, x, std::mem::size_of::<T>() as isize);

            Self {
                width,
                rows: StridedSlice::from_raw_parts(ptr, self.stride() as isize, height),
            }
        }
    }
    pub fn subview(&self, x: usize, y: usize, width: usize, height: usize) -> ImageView<'_, T> {
        self.clone().into_subview(x, y, width, height)
    }
}

impl<'a, T> Clone for ImageView<'a, T> {
    fn clone(&self) -> Self {
        Self {
            width: self.width.clone(),
            rows: self.rows.clone(),
        }
    }
}

impl<'a, T> From<ImageViewMut<'a, T>> for ImageView<'a, T> {
    fn from(value: ImageViewMut<'a, T>) -> Self {
        unsafe { std::mem::transmute(value) }
    }
}

#[repr(C)]
pub struct ImageViewMut<'a, T> {
    width: usize,
    rows: StridedSliceMut<'a, T>,
}

impl<'a, T> ImageViewMut<'a, T> {
    pub unsafe fn unchecked_get_mut(&mut self, x: usize, y: usize) -> &mut T {
        unsafe { self.unchecked_row_mut(y).get_unchecked_mut(x) }
    }
    pub fn checked_get_mut(&mut self, x: usize, y: usize) -> Option<&mut T> {
        if x < self.width() && y < self.height() {
            Some(unsafe { self.unchecked_get_mut(x, y) })
        } else {
            None
        }
    }
    pub fn get_mut(&mut self, x: usize, y: usize) -> &mut T {
        if x < self.width() && y < self.height() {
            unsafe { self.unchecked_get_mut(x, y) }
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

    pub unsafe fn unchecked_row_mut(&mut self, y: usize) -> &mut [T] {
        unsafe {
            std::slice::from_raw_parts_mut(
                ptr_mut_offset(self.rows.as_ptr_mut(), y, self.stride() as isize),
                self.width(),
            )
        }
    }
    pub fn checked_row_mut(&mut self, y: usize) -> Option<&mut [T]> {
        if y < self.height() {
            Some(unsafe { self.unchecked_row_mut(y) })
        } else {
            None
        }
    }
    pub fn row_mut(&mut self, y: usize) -> &mut [T] {
        if y < self.height() {
            unsafe { self.unchecked_row_mut(y) }
        } else {
            panic!(
                "Trying to access row {y} from an image with height {}",
                self.height()
            )
        }
    }

    pub unsafe fn unchecked_col_mut(&mut self, x: usize) -> StridedSliceMut<'_, T> {
        unsafe {
            StridedSliceMut::from_raw_parts(
                ptr_mut_offset(self.rows.as_ptr_mut(), x, std::mem::size_of::<T>() as isize),
                self.stride() as isize,
                self.height(),
            )
        }
    }
    pub fn checked_col_mut(&mut self, x: usize) -> Option<StridedSliceMut<'_, T>> {
        if x < self.width() {
            Some(unsafe { self.unchecked_col_mut(x) })
        } else {
            None
        }
    }
    pub fn col_mut(&mut self, x: usize) -> StridedSliceMut<'_, T> {
        if x < self.width() {
            unsafe { self.unchecked_col_mut(x) }
        } else {
            panic!(
                "Trying to access column {x} from an image with width {}",
                self.height()
            )
        }
    }

    pub fn for_each_mut(&mut self, mut f: impl FnMut(usize, usize, &mut T)) {
        for_each_ptr_mut(
            self.rows.as_ptr_mut(),
            self.width(),
            self.height(),
            self.stride(),
            move |x, y, ptr| f(x, y, unsafe { ptr.as_mut().unwrap_unchecked() }),
        );
    }

    pub fn view_mut(&mut self) -> ImageViewMut<'_, T> {
        self.subview_mut(0, 0, self.width(), self.height())
    }
    pub fn subview_mut(
        &mut self,
        x: usize,
        y: usize,
        width: usize,
        height: usize,
    ) -> ImageViewMut<'_, T> {
        unsafe {
            Self {
                width: self.width,
                rows: StridedSliceMut::from_raw_parts(
                    self.rows.as_ptr_mut(),
                    self.stride() as isize,
                    self.height(),
                ),
            }
        }
        .into_subview_mut(x, y, width, height)
    }

    pub fn into_subview_mut(mut self, x: usize, y: usize, width: usize, height: usize) -> Self {
        let x = x.min(self.width());
        let y = y.min(self.height());
        let width = width.min(self.width().checked_sub(x).unwrap_or(0));
        let height = height.min(self.height().checked_sub(y).unwrap_or(0));

        unsafe {
            let ptr = self.rows.as_ptr_mut();
            let ptr = ptr_mut_offset(ptr, y, self.stride() as isize);
            let ptr = ptr_mut_offset(ptr, x, std::mem::size_of::<T>() as isize);

            Self {
                width,
                rows: StridedSliceMut::from_raw_parts(ptr, self.stride() as isize, height),
            }
        }
    }
}

impl<'a, T> Deref for ImageViewMut<'a, T> {
    type Target = ImageView<'a, T>;

    fn deref(&self) -> &Self::Target {
        unsafe { std::mem::transmute(self) }
    }
}
impl<'a, T> DerefMut for ImageViewMut<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { std::mem::transmute(self) }
    }
}
impl<'a, T> AsRef<ImageView<'a, T>> for ImageViewMut<'a, T> {
    fn as_ref(&self) -> &ImageView<'a, T> {
        self.deref()
    }
}
impl<'a, T> AsMut<ImageView<'a, T>> for ImageViewMut<'a, T> {
    fn as_mut(&mut self) -> &mut ImageView<'a, T> {
        self.deref_mut()
    }
}

#[repr(C)]
pub struct Image<T: 'static> {
    width: usize,
    rows: StridedSliceMut<'static, T>,
}

impl<T> Image<T> {
    pub fn with_stride_and_fn(
        width: usize,
        height: usize,
        stride: usize,
        f: impl Fn(usize, usize) -> T,
    ) -> Self {
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
                std::ptr::null_mut()
            } else {
                let layout = Layout::from_size_align_unchecked(size, align);
                let ptr = std::alloc::alloc(layout) as *mut T;

                if ptr.is_null() {
                    std::alloc::handle_alloc_error(layout);
                }

                for_each_ptr_mut(ptr, width, height, stride, move |x, y, ptr| {
                    ptr.write(f(x, y))
                });

                ptr
            };

            Self {
                width,
                rows: StridedSliceMut::from_raw_parts(ptr, stride as isize, height),
            }
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
}

impl<T: 'static> Drop for Image<T> {
    fn drop(&mut self) {
        unsafe {
            for_each_ptr_mut(
                self.rows.as_ptr_mut(),
                self.width(),
                self.height(),
                self.stride(),
                move |_, _, ptr| ptr.drop_in_place(),
            );

            let size = self.stride().unchecked_mul(self.height());
            let align = std::mem::align_of::<T>();
            let layout = Layout::from_size_align_unchecked(size, align);

            if size != 0 {
                std::alloc::dealloc(self.rows.as_ptr_mut() as *mut u8, layout);
            }
        }
    }
}

impl<T> Deref for Image<T> {
    type Target = ImageViewMut<'static, T>;

    fn deref(&self) -> &Self::Target {
        unsafe { std::mem::transmute(self) }
    }
}
impl<T> DerefMut for Image<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { std::mem::transmute(self) }
    }
}
impl<T> AsRef<ImageViewMut<'static, T>> for Image<T> {
    fn as_ref(&self) -> &ImageViewMut<'static, T> {
        self.deref()
    }
}
impl<T> AsMut<ImageViewMut<'static, T>> for Image<T> {
    fn as_mut(&mut self) -> &mut ImageViewMut<'static, T> {
        self.deref_mut()
    }
}
impl<T> AsRef<ImageView<'static, T>> for Image<T> {
    fn as_ref(&self) -> &ImageView<'static, T> {
        self.deref()
    }
}
impl<T> AsMut<ImageView<'static, T>> for Image<T> {
    fn as_mut(&mut self) -> &mut ImageView<'static, T> {
        self.deref_mut()
    }
}

impl<T> Borrow<ImageView<'static, T>> for Image<T> {
    fn borrow(&self) -> &ImageView<'static, T> {
        self.as_ref()
    }
}
impl<T> BorrowMut<ImageView<'static, T>> for Image<T> {
    fn borrow_mut(&mut self) -> &mut ImageView<'static, T> {
        self.as_mut()
    }
}
impl<T> Borrow<ImageViewMut<'static, T>> for Image<T> {
    fn borrow(&self) -> &ImageViewMut<'static, T> {
        self.as_ref()
    }
}
impl<T> BorrowMut<ImageViewMut<'static, T>> for Image<T> {
    fn borrow_mut(&mut self) -> &mut ImageViewMut<'static, T> {
        self.as_mut()
    }
}

fn for_each_ptr<T>(
    ptr: *const T,
    width: usize,
    height: usize,
    stride: usize,
    mut f: impl FnMut(usize, usize, *const T),
) {
    let mut row = ptr;
    for y in 0usize..height {
        let mut cell = row;
        for x in 0usize..width {
            f(x, y, cell);
            cell = unsafe { cell.add(1) };
        }
        row = unsafe { row.byte_add(stride) };
    }
}
fn for_each_ptr_mut<T>(
    ptr: *mut T,
    width: usize,
    height: usize,
    stride: usize,
    mut f: impl FnMut(usize, usize, *mut T),
) {
    let mut row = ptr;
    for y in 0usize..height {
        let mut cell = row;
        for x in 0usize..width {
            f(x, y, cell);
            cell = unsafe { cell.add(1) };
        }
        row = unsafe { row.byte_add(stride) };
    }
}

#[cfg(test)]
mod test {
    use std::usize;

    use super::Image;

    #[test]
    fn clone() {
        let mut image = Image::<u8>::new(2, 2);
        let mut view1 = image.view_mut();

        // FIXME: view1 should not be cloneable into an ImageView as it is a soundness issue
        let view2 = view1.clone();

        assert_eq!(view1.get_mut(0, 0), view2.get(0, 0));
    }

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
