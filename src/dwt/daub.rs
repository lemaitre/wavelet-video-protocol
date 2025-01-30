use super::Dwt1;

pub struct Daub53;

impl Dwt1<u8> for Daub53 {
    fn dwt1(
        &self,
        mut sig: crate::memory::StridedSliceMut<'_, u8>,
        mut tmp: crate::memory::StridedSliceMut<'_, u8>,
    ) {
        for (&src, dst) in sig.as_strided_slice().zip(tmp.as_strided_slice_mut()) {
            *dst = src;
        }
        let tmp = tmp.into_strided_slice();
        let (src1, src2) = tmp.deinterleave();
        let (dst1, dst2) = sig.split_at(sig.len() / 2);

        let mut h0 = 0i16;
        for (i, (&a, (&b, (l, h)))) in src1.zip(src2.zip(dst1.zip(dst2))).enumerate() {
            let c = tmp.checked_get(2 * i + 2).copied().unwrap_or(a);
            *h = b.wrapping_sub(a.midpoint(c));
            *l = a.wrapping_add_signed(((h0 + *h as i8 as i16) / 4) as i8);
            h0 = *h as i8 as i16;

            *h = h.wrapping_add(128);
        }
    }

    fn idwt1(
        &self,
        mut sig: crate::memory::StridedSliceMut<'_, u8>,
        mut tmp: crate::memory::StridedSliceMut<'_, u8>,
    ) {
        for (&src, dst) in sig.as_strided_slice().zip(tmp.as_strided_slice_mut()) {
            *dst = src;
        }
        let tmp = tmp.into_strided_slice();
        let (src1, src2) = tmp.split_at(tmp.len() / 2);
        let (dst1, dst2) = sig.deinterleave();

        let mut c = None;
        for (i, (&l, (&h, (dst1, dst2)))) in src1.zip(src2.zip(dst1.zip(dst2))).enumerate().rev() {
            let h0 = src2
                .checked_get(i.wrapping_sub(1))
                .copied()
                .map(|h| h.wrapping_add(128))
                .unwrap_or_default();
            let h = h.wrapping_add(128);
            let a = l.wrapping_add_signed(-((h0 as i8 as i16 + h as i8 as i16) / 4) as i8);
            let b = h.wrapping_add(a.midpoint(c.unwrap_or(a)));
            c = Some(a);

            *dst1 = a;
            *dst2 = b;
        }
    }
}

pub struct LossyDaub53;

fn scale_down(x: i16, t: i16, n: i16) -> i16 {
    if x < -t {
        ((x + t) / n) - t
    } else if x > t {
        ((x - t) / n) + t
    } else {
        x
    }
}
fn scale_up(x: i16, t: i16, n: i16) -> i16 {
    if x < -t {
        ((x + t) * n) - t
    } else if x > t {
        ((x - t) * n) + t
    } else {
        x
    }
}

fn encode(x: i16) -> i16 {
    scale_down(x, 64, 3)

    // match x {
    //     ..-256 => -17, // x < -256
    //     ..-192 => -16, // -256 <= x < -192
    //     ..-128 => -15, // -192 <= x < -128
    //     ..-96 => -14,  // -128 <= x < -96
    //     ..-64 => -13,  // -96 <= x < -64
    //     ..-48 => -12,  // -64 <= x < -48
    //     ..-32 => -11,  // -48 <= x < -32
    //     ..-24 => -10,  // -32 <= x < -24
    //     ..-16 => -9,   // -24 <= x < -16
    //     ..-12 => -8,   // -16 <= x < -12
    //     ..-8 => -7,    // -12 <= x < -8
    //     ..-6 => -6,    // -8 <= x < -6
    //     ..-4 => -5,    // -6 <= x < -4
    //     ..-3 => -4,    // x = -4
    //     ..-2 => -3,    // x = -3
    //     ..-1 => -2,    // x = -2
    //     ..0 => -1,     // x = -1
    //     ..1 => 0,      // x = 0
    //     ..2 => 1,      // x = 1
    //     ..3 => 2,      // x = 2
    //     ..4 => 3,      // x = 3
    //     ..6 => 4,      // 4 <= x < 6
    //     ..8 => 5,      // 6 <= x < 8
    //     ..12 => 6,     // 8 <= x < 12
    //     ..16 => 7,     // 12 <= x < 16
    //     ..24 => 8,     // 16 <= x < 24
    //     ..32 => 9,     // 24 <= x < 32
    //     ..48 => 10,    // 32 <= x < 48
    //     ..64 => 11,    // 48 <= x < 64
    //     ..96 => 12,    // 64 <= x < 96
    //     ..128 => 13,   // 96 <= x < 128
    //     ..192 => 14,   // 128 <= x < 192
    //     ..256 => 15,   // 192 <= x < 256
    //     _ => 16,       // 256 <= x
    // }
}
fn decode(x: i16) -> i16 {
    scale_up(x, 64, 3)

    // match x {
    //     ..=-17 => -256, // x < -256
    //     -16 => -224,    // -256 <= x < -192
    //     -15 => -160,    // -192 <= x < -128
    //     -14 => -112,    // -128 <= x < -96
    //     -13 => -80,     // -96 <= x < -64
    //     -12 => -56,     // -64 <= x < -48
    //     -11 => -40,     // -48 <= x < -32
    //     -10 => -28,     // -32 <= x < -24
    //     -9 => -20,      // -24 <= x < -16
    //     -8 => -14,      // -16 <= x < -12
    //     -7 => -10,      // -12 <= x < -8
    //     -6 => -7,       // -8 <= x < -6
    //     -5 => -5,       // -6 <= x < -4
    //     -4 => -4,       // x = -4
    //     -3 => -3,       // x = -3
    //     -2 => -2,       // x = -2
    //     -1 => -1,       // x = -1
    //     0 => 0,         // x = 0
    //     1 => 1,         // x = 1
    //     2 => 2,         // x = 2
    //     3 => 3,         // x = 3
    //     4 => 4,         // 4 <= x < 6
    //     5 => 6,         // 6 <= x < 8
    //     6 => 9,         // 8 <= x < 12
    //     7 => 13,        // 12 <= x < 16
    //     8 => 19,        // 16 <= x < 24
    //     9 => 27,        // 24 <= x < 32
    //     10 => 39,       // 32 <= x < 48
    //     11 => 55,       // 48 <= x < 64
    //     12 => 79,       // 64 <= x < 96
    //     13 => 111,      // 96 <= x < 128
    //     14 => 159,      // 128 <= x < 192
    //     15 => 223,      // 192 <= x < 256
    //     16.. => 256,    // 256 <= x
    // }
}

impl Dwt1<u8> for LossyDaub53 {
    fn dwt1(
        &self,
        mut sig: crate::memory::StridedSliceMut<'_, u8>,
        mut tmp: crate::memory::StridedSliceMut<'_, u8>,
    ) {
        for (&src, dst) in sig.as_strided_slice().zip(tmp.as_strided_slice_mut()) {
            *dst = src;
        }
        let tmp = tmp.into_strided_slice();
        let (src1, src2) = tmp.deinterleave();
        let (dst1, dst2) = sig.split_at(sig.len() / 2);

        let mut h0 = 0i16;
        for (i, (&a, (&b, (dst1, dst2)))) in src1.zip(src2.zip(dst1.zip(dst2))).enumerate() {
            let c = tmp.checked_get(2 * i + 2).copied().unwrap_or(a);
            let a = a as i16;
            let b = b as i16;
            let c = c as i16;

            let h = b - (a + c) / 2;
            let l = a + (h0 + h) / 4;
            h0 = h;

            let h = encode(h);
            let h = h.clamp(-128, 127) as i8 as u8;
            let l = l.clamp(0, 255) as u8;

            *dst1 = l;
            *dst2 = h.wrapping_add(128);
        }
    }

    fn idwt1(
        &self,
        mut sig: crate::memory::StridedSliceMut<'_, u8>,
        mut tmp: crate::memory::StridedSliceMut<'_, u8>,
    ) {
        for (&src, dst) in sig.as_strided_slice().zip(tmp.as_strided_slice_mut()) {
            *dst = src;
        }
        let tmp = tmp.into_strided_slice();
        let (src1, src2) = tmp.split_at(tmp.len() / 2);
        let (dst1, dst2) = sig.deinterleave();

        let mut c = None;
        for (i, (&l, (&h, (dst1, dst2)))) in src1.zip(src2.zip(dst1.zip(dst2))).enumerate().rev() {
            let h0 = src2
                .checked_get(i.wrapping_sub(1))
                .copied()
                .map(|h| h.wrapping_add(128))
                .unwrap_or_default();
            let h = h.wrapping_add(128);
            let l = l as i16;
            let h = h as i8 as i16;
            let h0 = h0 as i8 as i16;

            let h = decode(h);
            let h0 = decode(h0);

            let a = l - (h + h0) / 4;
            let b = h + (a + c.unwrap_or(a)) / 2;
            c = Some(a);

            let a = a.clamp(0, 256) as u8;
            let b = b.clamp(0, 256) as u8;
            *dst1 = a;
            *dst2 = b;
        }
    }
}
