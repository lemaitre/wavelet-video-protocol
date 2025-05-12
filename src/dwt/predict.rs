use super::Dwt1;

pub struct Predict<A>(pub A);

impl<A: Dwt1<i8>> Dwt1<i8> for Predict<A> {
    fn dwt1(&self, mut sig: crate::memory::Strided<&mut i8>, tmp: crate::memory::Strided<&mut i8>) {
        self.0.dwt1(sig.as_strided_mut(), tmp);

        let (low, high) = sig.split_at_mut(sig.len() / 2);
        for (i, h) in high.into_iter().enumerate() {
            let prev = low[i.max(1) - 1] as i16;
            let next = low[i.min(low.len() - 2) + 1] as i16;
            *h = h.wrapping_add(((2 + prev - next) / 4) as i8);
        }
    }

    fn idwt1(
        &self,
        mut sig: crate::memory::Strided<&mut i8>,
        tmp: crate::memory::Strided<&mut i8>,
    ) {
        let (low, high) = sig.split_at_mut(sig.len() / 2);
        for (i, h) in high.into_iter().enumerate() {
            let prev = low[i.max(1) - 1] as i16;
            let next = low[i.min(low.len() - 2) + 1] as i16;
            *h = h.wrapping_sub(((2 + prev - next) / 4) as i8);
        }

        self.0.idwt1(sig, tmp);
    }
}

impl<A: Dwt1<i16>> Dwt1<i16> for Predict<A> {
    fn dwt1(
        &self,
        mut sig: crate::memory::Strided<&mut i16>,
        tmp: crate::memory::Strided<&mut i16>,
    ) {
        self.0.dwt1(sig.as_strided_mut(), tmp);

        let (low, high) = sig.split_at_mut(sig.len() / 2);
        for (i, h) in high.into_iter().enumerate() {
            let prev = low[i.max(1) - 1];
            let next = low[i.min(low.len() - 2) + 1];
            *h += (2 + prev - next) / 4;
        }
    }

    fn idwt1(
        &self,
        mut sig: crate::memory::Strided<&mut i16>,
        tmp: crate::memory::Strided<&mut i16>,
    ) {
        let (low, high) = sig.split_at_mut(sig.len() / 2);
        for (i, h) in high.into_iter().enumerate() {
            let prev = low[i.max(1) - 1];
            let next = low[i.min(low.len() - 2) + 1];
            *h -= (2 + prev - next) / 4;
        }

        self.0.idwt1(sig, tmp);
    }
}
