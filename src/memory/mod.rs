mod image;
mod slice;

pub use image::{Image, ImageView, ImageViewMut};
pub use slice::{StridedSlice, StridedSliceMut};

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
