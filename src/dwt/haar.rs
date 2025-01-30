use super::Dwt0;

/// Implementation of the Haar wavelet from the following paper
///
/// > Calderbank, A. Robert, et al.
/// > "Wavelet transforms that map integers to integers."
/// > Applied and computational harmonic analysis 5.3 (1998): 332-369.
pub struct Haar;

impl Dwt0<u8> for Haar {
    fn dwt0(&self, a: u8, b: u8) -> (u8, u8) {
        let h = b.wrapping_sub(a);
        let l = a.wrapping_add_signed((h as i8) / 2);

        // Offset value for nicer display
        let h = h.wrapping_add(128);

        (l, h)
    }

    fn idwt0(&self, l: u8, h: u8) -> (u8, u8) {
        // Offset value back
        let h = h.wrapping_sub(128);

        let a = l.wrapping_add_signed(-((h as i8) / 2));
        let b = a.wrapping_add_signed(h as i8);

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

impl Dwt0<u8> for LossyHaar {
    fn dwt0(&self, a: u8, b: u8) -> (u8, u8) {
        let h = (b as i16) - (a as i16);
        let h = h / 2;
        let h = h.clamp(-128, 127) as u8;
        let l = a.wrapping_add(h);

        // Offset value for nicer display
        let h = h.wrapping_add(128);

        (l, h)
    }

    fn idwt0(&self, l: u8, h: u8) -> (u8, u8) {
        // Offset value back
        let h = h.wrapping_sub(128) as i8;

        let a = l.saturating_add_signed(h.saturating_neg());
        let b = a.saturating_add_signed(h).saturating_add_signed(h);

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
