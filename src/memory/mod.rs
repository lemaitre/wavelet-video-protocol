mod image;
mod image_iter;
mod slice;

pub use image::{Image, ImageView, ImageViewMut, ImageViewPtr};
pub use image_iter::{
    ImageColIter, ImageColIterMut, ImageColIterPtr, ImageRowIter, ImageRowIterMut, ImageRowIterPtr,
};
pub use slice::{StridedSlice, StridedSliceMut, StridedSlicePtr};

unsafe fn ptr_offset<T>(
    ptr: std::ptr::NonNull<T>,
    i: usize,
    stride: isize,
) -> std::ptr::NonNull<T> {
    unsafe {
        let offset = stride.unchecked_mul(i as isize);
        ptr.byte_offset(offset)
    }
}
