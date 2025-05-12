use super::Dwt0;

/// Implementation of the Haar wavelet from the following paper
///
/// > Calderbank, A. Robert, et al.
/// > "Wavelet transforms that map integers to integers."
/// > Applied and computational harmonic analysis 5.3 (1998): 332-369.
pub struct Haar;

impl Dwt0<i8> for Haar {
    fn dwt0(&self, a: i8, b: i8) -> (i8, i8) {
        let h = b.wrapping_sub(a);
        let l = a.wrapping_add(h / 2);

        (l, h)
    }

    fn idwt0(&self, l: i8, h: i8) -> (i8, i8) {
        let a = l.wrapping_sub(h / 2);
        let b = a.wrapping_add(h);

        (a, b)
    }
}

impl Dwt0<i16> for Haar {
    fn dwt0(&self, a: i16, b: i16) -> (i16, i16) {
        let h = b - a;
        let l = a + h / 2;

        (l, h)
    }

    fn idwt0(&self, l: i16, h: i16) -> (i16, i16) {
        let a = l - h / 2;
        let b = a + h;

        (a, b)
    }
}

pub struct LossyHaar;

impl Dwt0<i8> for LossyHaar {
    fn dwt0(&self, a: i8, b: i8) -> (i8, i8) {
        let h = (b as i16) - (a as i16);
        let h = h / 2;
        let h = h.clamp(-128, 127) as i8;
        let l = a.wrapping_add(h);

        (l, h)
    }

    fn idwt0(&self, l: i8, h: i8) -> (i8, i8) {
        let a = l.saturating_sub(h);
        let b = a.saturating_add(h).saturating_add(h);

        (a, b)
    }
}

impl Dwt0<i16> for LossyHaar {
    fn dwt0(&self, a: i16, b: i16) -> (i16, i16) {
        let h = b - a;
        let l = a + h / 2;

        (l, h)
    }

    fn idwt0(&self, l: i16, h: i16) -> (i16, i16) {
        let a = l - h / 2;
        let b = a + h;

        (a, b)
    }
}
