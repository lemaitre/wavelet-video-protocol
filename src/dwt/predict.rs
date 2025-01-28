use super::Dwt1;

pub struct Predict<A>(pub A);

impl<A: Dwt1<u8>> Dwt1<u8> for Predict<A> {
    fn dwt1(
        &self,
        mut sig: crate::memory::StridedSliceMut<'_, u8>,
        tmp: crate::memory::StridedSliceMut<'_, u8>,
    ) {
        self.0.dwt1(sig.as_strided_slice_mut(), tmp);

        let (low, high) = sig.split_at(sig.len() / 2);
        for (i, h) in high.enumerate() {
            let prev = low[i.max(1) - 1] as i16;
            let next = low[i.min(low.len() - 2) + 1] as i16;
            *h = h.wrapping_add(((2 + prev - next) / 4) as u8);
        }
    }

    fn idwt1(
        &self,
        mut sig: crate::memory::StridedSliceMut<'_, u8>,
        tmp: crate::memory::StridedSliceMut<'_, u8>,
    ) {
        let (low, high) = sig.split_at(sig.len() / 2);
        for (i, h) in high.enumerate() {
            let prev = low[i.max(1) - 1] as i16;
            let next = low[i.min(low.len() - 2) + 1] as i16;
            *h = h.wrapping_sub(((2 + prev - next) / 4) as u8);
        }

        self.0.idwt1(sig, tmp);
    }
}

impl<A: Dwt1<i16>> Dwt1<i16> for Predict<A> {
    fn dwt1(
        &self,
        mut sig: crate::memory::StridedSliceMut<'_, i16>,
        tmp: crate::memory::StridedSliceMut<'_, i16>,
    ) {
        self.0.dwt1(sig.as_strided_slice_mut(), tmp);

        let (low, high) = sig.split_at(sig.len() / 2);
        for (i, h) in high.enumerate() {
            let prev = low[i.max(1) - 1];
            let next = low[i.min(low.len() - 2) + 1];
            *h += (2 + prev - next) / 4;
        }
    }

    fn idwt1(
        &self,
        mut sig: crate::memory::StridedSliceMut<'_, i16>,
        tmp: crate::memory::StridedSliceMut<'_, i16>,
    ) {
        let (low, high) = sig.split_at(sig.len() / 2);
        for (i, h) in high.enumerate() {
            let prev = low[i.max(1) - 1];
            let next = low[i.min(low.len() - 2) + 1];
            *h -= (2 + prev - next) / 4;
        }

        self.0.idwt1(sig, tmp);
    }
}
