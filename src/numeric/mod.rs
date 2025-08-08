use crate::memory::Image;

pub trait Convert<T> {
    fn convert(&self) -> T;
}

macro_rules! convert_impl {
    ($src:ty => saturate $dst:ty) => {
        impl Convert<$dst> for $src {
            fn convert(&self) -> $dst {
                if *self < <$dst>::MIN as $src {
                    <$dst>::MIN
                } else if *self <= <$dst>::MAX as $src {
                    *self as $dst
                } else {
                    <$dst>::MAX
                }
            }
        }
    };
    ($src:ty => lossless $dst:ty) => {
        impl Convert<$dst> for $src {
            fn convert(&self) -> $dst {
                *self as $dst
            }
        }
    };
    ($src:ty => chained($int:ty) $dst:ty) => {
        impl Convert<$dst> for $src {
            fn convert(&self) -> $dst {
                let int: $int = self.convert();
                int.convert()
            }
        }
    };
}

convert_impl!(u8 => lossless u8);
convert_impl!(u16 => lossless u16);
convert_impl!(u32 => lossless u32);
convert_impl!(u64 => lossless u64);
convert_impl!(i8 => lossless i8);
convert_impl!(i16 => lossless i16);
convert_impl!(i32 => lossless i32);
convert_impl!(i64 => lossless i64);

convert_impl!(i16 => saturate i8);
convert_impl!(i32 => saturate i8);
convert_impl!(i32 => saturate i16);
convert_impl!(i64 => saturate i8);
convert_impl!(i64 => saturate i16);
convert_impl!(i64 => saturate i32);

convert_impl!(u16 => saturate u8);
convert_impl!(u32 => saturate u8);
convert_impl!(u32 => saturate u16);
convert_impl!(u64 => saturate u8);
convert_impl!(u64 => saturate u16);
convert_impl!(u64 => saturate u32);

convert_impl!(i8 => lossless i16);
convert_impl!(i8 => lossless i32);
convert_impl!(i8 => lossless i64);
convert_impl!(i16 => lossless i32);
convert_impl!(i16 => lossless i64);
convert_impl!(i32 => lossless i64);

convert_impl!(u8 => lossless u16);
convert_impl!(u8 => lossless u32);
convert_impl!(u8 => lossless u64);
convert_impl!(u16 => lossless u32);
convert_impl!(u16 => lossless u64);
convert_impl!(u32 => lossless u64);

convert_impl!(u8 => chained(i8) i16);
convert_impl!(u8 => chained(i8) i32);
convert_impl!(u8 => chained(i8) i64);
convert_impl!(u16 => chained(i16) i32);
convert_impl!(u16 => chained(i16) i64);
convert_impl!(u32 => chained(i32) i64);

convert_impl!(i8 => chained(u8) u16);
convert_impl!(i8 => chained(u8) u32);
convert_impl!(i8 => chained(u8) u64);
convert_impl!(i16 => chained(u16) u32);
convert_impl!(i16 => chained(u16) u64);
convert_impl!(i32 => chained(u32) u64);

convert_impl!(i16 => chained(u16) u8);
convert_impl!(i32 => chained(u32) u8);
convert_impl!(i32 => chained(u32) u16);
convert_impl!(i64 => chained(u64) u8);
convert_impl!(i64 => chained(u64) u16);
convert_impl!(i64 => chained(u64) u32);

convert_impl!(u16 => chained(i16) i8);
convert_impl!(u32 => chained(i32) i8);
convert_impl!(u32 => chained(i32) i16);
convert_impl!(u64 => chained(i64) i8);
convert_impl!(u64 => chained(i64) i16);
convert_impl!(u64 => chained(i64) i32);

impl Convert<u8> for i8 {
    fn convert(&self) -> u8 {
        self.wrapping_add(i8::MIN) as u8
    }
}
impl Convert<u16> for i16 {
    fn convert(&self) -> u16 {
        self.wrapping_add(i16::MIN) as u16
    }
}
impl Convert<u32> for i32 {
    fn convert(&self) -> u32 {
        self.wrapping_add(i32::MIN) as u32
    }
}
impl Convert<u64> for i64 {
    fn convert(&self) -> u64 {
        self.wrapping_add(i64::MIN) as u64
    }
}

impl Convert<i8> for u8 {
    fn convert(&self) -> i8 {
        self.wrapping_add_signed(i8::MIN) as i8
    }
}
impl Convert<i16> for u16 {
    fn convert(&self) -> i16 {
        self.wrapping_add_signed(i16::MIN) as i16
    }
}
impl Convert<i32> for u32 {
    fn convert(&self) -> i32 {
        self.wrapping_add_signed(i32::MIN) as i32
    }
}
impl Convert<i64> for u64 {
    fn convert(&self) -> i64 {
        self.wrapping_add_signed(i64::MIN) as i64
    }
}

impl<S, D> Convert<Image<D>> for Image<S>
where
    S: Convert<D>,
{
    fn convert(&self) -> Image<D> {
        let mut image = Image::new_uninit(self.width(), self.height(), self.stride());

        for (row_dst, row_src) in image.rows_mut().zip(self.view().rows()) {
            for (dst, src) in row_dst.iter_mut().zip(row_src) {
                dst.write(src.convert());
            }
        }

        unsafe { image.assume_init() }
    }
}

#[cfg(test)]
mod tests {
    use crate::numeric::Convert;

    #[test]
    fn foo() {
        assert_eq!(Convert::<i8>::convert(&128u8), 0i8);
        assert_eq!(Convert::<i8>::convert(&130i16), 127i8);
    }
}
