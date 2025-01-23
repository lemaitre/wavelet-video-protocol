use super::{StridedSlice, StridedSliceMut};

pub struct ImageViewMut<'a, T> {
    width: usize,
    rows: StridedSliceMut<'a, T>,
}

impl<'a, T> ImageViewMut<'a, T> {
    pub fn width(&self) -> usize {
        self.width
    }
    pub fn height(&self) -> usize {
        self.rows.len()
    }
    pub fn stride(&self) -> usize {
        self.rows.stride() as usize
    }

    pub unsafe fn unchecked_row_mut(&mut self, i: usize) -> &mut [T] {
        unsafe {
            std::slice::from_raw_parts_mut(
                ptr_mut_offset(self.rows.as_ptr_mut(), i, self.stride() as isize),
                self.width(),
            )
        }
    }
    pub fn checked_row_mut(&mut self, i: usize) -> Option<&mut [T]> {
        if i < self.height() {
            Some(unsafe { self.unchecked_row_mut(i) })
        } else {
            None
        }
    }
    pub fn row_mut(&mut self, i: usize) -> &mut [T] {
        if i < self.height() {
            unsafe { self.unchecked_row_mut(i) }
        } else {
            panic!(
                "Trying to access row {i} from an image with height {}",
                self.height()
            )
        }
    }

    pub unsafe fn unchecked_row(&self, i: usize) -> &[T] {
        unsafe {
            std::slice::from_raw_parts(
                ptr_offset(self.rows.as_ptr(), i, self.stride() as isize),
                self.width(),
            )
        }
    }
    pub fn checked_row(&self, i: usize) -> Option<&[T]> {
        if i < self.height() {
            Some(unsafe { self.unchecked_row(i) })
        } else {
            None
        }
    }
    pub fn row(&self, i: usize) -> &[T] {
        if i < self.height() {
            unsafe { self.unchecked_row(i) }
        } else {
            panic!(
                "Trying to access row {i} from an image with height {}",
                self.height()
            )
        }
    }

    pub unsafe fn unchecked_col_mut(&mut self, i: usize) -> StridedSliceMut<'_, T> {
        unsafe {
            StridedSliceMut::from_raw_parts(
                ptr_mut_offset(self.rows.as_ptr_mut(), i, std::mem::size_of::<T>() as isize),
                self.stride() as isize,
                self.height(),
            )
        }
    }
    pub fn checked_col_mut(&mut self, i: usize) -> Option<StridedSliceMut<'_, T>> {
        if i < self.width() {
            Some(unsafe { self.unchecked_col_mut(i) })
        } else {
            None
        }
    }
    pub fn col_mut(&mut self, i: usize) -> StridedSliceMut<'_, T> {
        if i < self.width() {
            unsafe { self.unchecked_col_mut(i) }
        } else {
            panic!(
                "Trying to access column {i} from an image with width {}",
                self.height()
            )
        }
    }

    pub unsafe fn unchecked_col(&self, i: usize) -> StridedSlice<'_, T> {
        unsafe {
            StridedSlice::from_raw_parts(
                ptr_offset(self.rows.as_ptr(), i, std::mem::size_of::<T>() as isize),
                self.stride() as isize,
                self.height(),
            )
        }
    }
    pub fn checked_col(&self, i: usize) -> Option<StridedSlice<'_, T>> {
        if i < self.height() {
            Some(unsafe { self.unchecked_col(i) })
        } else {
            None
        }
    }
    pub fn col(&self, i: usize) -> StridedSlice<'_, T> {
        if i < self.height() {
            unsafe { self.unchecked_col(i) }
        } else {
            panic!(
                "Trying to access column {i} from an image with width {}",
                self.height()
            )
        }
    }
}

unsafe fn ptr_offset<T>(ptr: *const T, i: usize, stride: isize) -> *const T {
    unsafe {
        let offset = stride.unchecked_mul(i as isize);
        ptr.byte_offset(offset)
    }
}
unsafe fn ptr_mut_offset<T>(ptr: *mut T, i: usize, stride: isize) -> *mut T {
    unsafe {
        let offset = stride.unchecked_mul(i as isize);
        ptr.byte_offset(offset)
    }
}
