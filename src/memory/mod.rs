mod image;
mod image_iter;
mod slice;

pub use image::{Image, ImageView, ImageViewMut, ImageViewPtr};
pub use image_iter::{
    ImageColIter, ImageColIterMut, ImageColIterPtr, ImageRowIter, ImageRowIterMut, ImageRowIterPtr,
};
pub use slice::{StridedSlice, StridedSliceMut, StridedSlicePtr};

/// Adjust the pointer with the given offset
/// Semantically equivalent to ADDR + i * stride
///
/// # SAFETY
///
/// The product i * stride should not overflow.
/// The resulting address should not overflow.
///
/// If the result is outside the current allocation,
/// the resulting pointer would not be dereferenceable, but still valid.
unsafe fn ptr_offset<T>(
    ptr: std::ptr::NonNull<T>,
    i: usize,
    stride: isize,
) -> std::ptr::NonNull<T> {
    ptr.map_addr(|addr| unsafe {
        let addr = addr.get() as isize;
        let offset = stride.unchecked_mul(i as isize);
        let addr = addr.unchecked_add(offset);
        std::num::NonZero::new_unchecked(addr as usize)
    })
}
