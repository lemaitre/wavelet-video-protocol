use crate::memory::{ImageViewMut, Strided};

pub mod daub;
pub mod haar;
pub mod predict;

pub trait Dwt0<T> {
    fn dwt0(&self, a: T, b: T) -> (T, T);
    fn idwt0(&self, l: T, h: T) -> (T, T);
}
pub trait Dwt1<T> {
    fn dwt1(&self, sig: Strided<&mut T>, tmp: Strided<&mut T>);
    fn dwt1_slice(&self, sig: &mut [T], tmp: &mut [T]) {
        self.dwt1(sig.into(), tmp.into());
    }

    fn idwt1(&self, sig: Strided<&mut T>, tmp: Strided<&mut T>);
    fn idwt1_slice(&self, sig: &mut [T], tmp: &mut [T]) {
        self.idwt1(sig.into(), tmp.into());
    }
}
pub trait Dwt2<T> {
    fn dwt2(&self, img: ImageViewMut<'_, T>, tmp: ImageViewMut<'_, T>);
    fn idwt2(&self, img: ImageViewMut<'_, T>, tmp: ImageViewMut<'_, T>);
}

impl<T: Clone + std::fmt::Debug, A: Dwt0<T>> Dwt1<T> for A {
    fn dwt1(&self, mut sig: Strided<&mut T>, mut tmp: Strided<&mut T>) {
        for (src, dst) in sig.iter().zip(tmp.iter_mut()) {
            *dst = src.clone();
        }
        let [src1, src2] = tmp.into_deinterleave_array();
        let (dst1, dst2) = sig.split_at_mut(sig.len() / 2);

        for (a, (b, (l, h))) in src1
            .into_iter()
            .zip(src2.into_iter().zip(dst1.into_iter().zip(dst2)))
        {
            (*l, *h) = self.dwt0(a.clone(), b.clone());
        }
    }

    fn idwt1(&self, sig: Strided<&mut T>, mut tmp: Strided<&mut T>) {
        for (src, dst) in sig.iter().zip(tmp.iter_mut()) {
            *dst = src.clone();
        }
        let (src1, src2) = tmp.split_at(tmp.len() / 2);
        let [dst1, dst2] = sig.into_deinterleave_array();

        for (l, (h, (a, b))) in src1
            .into_iter()
            .zip(src2.into_iter().zip(dst1.into_iter().zip(dst2)))
        {
            (*a, *b) = self.idwt0(l.clone(), h.clone());
        }
    }
}

impl<T, A: Dwt1<T>> Dwt2<T> for A {
    fn dwt2(&self, mut img: ImageViewMut<'_, T>, mut tmp: ImageViewMut<'_, T>) {
        for (row_img, row_tmp) in img.rows_mut().zip(tmp.rows_mut()) {
            self.dwt1_slice(row_img, row_tmp);
        }

        for (col_img, col_tmp) in img.cols_mut().zip(tmp.cols_mut()) {
            self.dwt1(col_img, col_tmp);
        }
    }

    fn idwt2(&self, mut img: ImageViewMut<'_, T>, mut tmp: ImageViewMut<'_, T>) {
        for (col_img, col_tmp) in img.cols_mut().zip(tmp.cols_mut()) {
            self.idwt1(col_img, col_tmp);
        }

        for (row_img, row_tmp) in img.rows_mut().zip(tmp.rows_mut()) {
            self.idwt1_slice(row_img, row_tmp);
        }
    }
}
