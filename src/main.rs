#[allow(unused)]
use dwt::{
    daub::{Daub53, LossyDaub53},
    haar::{Haar, LossyHaar},
    predict::Predict,
    Dwt2,
};
use memory::{Image, ImageView, ImageViewMut};

pub mod dwt;
pub mod io;
pub mod memory;

fn print_minmax(image: ImageView<'_, i8>, name: &str) {
    let mut min = i8::MAX;
    let mut max = i8::MIN;
    for row in image.into_rows() {
        for &cell in row {
            min = min.min(cell);
            max = max.max(cell);
        }
    }

    println!("{name}: [{min}; {max}]");
}

fn image_upcast(src: ImageView<'_, u8>, mut dst: ImageViewMut<'_, i8>) {
    for (src, dst) in src.rows().zip(dst.rows_mut()) {
        for (&src, dst) in src.iter().zip(dst) {
            *dst = src.wrapping_sub(128) as i8;
            // *dst = src;
        }
    }
}

fn image_downcast(src: ImageView<'_, i8>, mut dst: ImageViewMut<'_, u8>) {
    for (src, dst) in src.rows().zip(dst.rows_mut()) {
        for (&src, dst) in src.iter().zip(dst) {
            *dst = (src as u8).wrapping_add(128);
            // *dst = (src.clamp(0, 255) as i8 as u8).wrapping_add(128);
            // *dst = src;
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    const N: usize = 4;
    const LOW_BAND_ONLY: bool = false;

    let input8 = io::load_pgm("input.pgm")?;
    let mut output8 = input8.clone();
    let mut reconstructed8 = input8.clone();
    let mut input = Image::with_stride(input8.width(), input8.height(), input8.stride());
    image_upcast(input8.view(), input.view_mut());
    let input = input;
    let mut tmp = input.clone();
    let mut output = input.clone();
    let mut reconstructed = input.clone();

    let dwt = Daub53;

    for i in 0..N {
        let i = if LOW_BAND_ONLY { i } else { 0 };
        let w = output.width() >> i;
        let h = output.height() >> i;

        dwt.dwt2(output.subview_mut(0, 0, w, h), tmp.subview_mut(0, 0, w, h));
    }

    print_minmax(
        output.subview(
            output.width() / 2,
            0,
            output.width() / 2,
            output.height() / 2,
        ),
        "LH",
    );
    print_minmax(
        output.subview(
            0,
            output.height() / 2,
            output.width() / 2,
            output.height() / 2,
        ),
        "HL",
    );
    print_minmax(
        output.subview(
            output.width() / 2,
            output.height() / 2,
            output.width() / 2,
            output.height() / 2,
        ),
        "HH",
    );

    image_downcast(output.view(), output8.view_mut());
    io::save_pgm(output8.view(), "output.pgm")?;
    image_upcast(output8.view(), reconstructed.view_mut());

    for i in (0..N).rev() {
        let i = if LOW_BAND_ONLY { i } else { 0 };
        let w = output.width() >> i;
        let h = output.height() >> i;
        dwt.idwt2(
            reconstructed.subview_mut(0, 0, w, h),
            tmp.subview_mut(0, 0, w, h),
        );
    }
    image_downcast(reconstructed.view(), reconstructed8.view_mut());

    println!("Written");

    let mut s1 = 0i64;
    let mut s2 = 0i64;

    for (src, dst) in input8.rows().zip(reconstructed8.rows()) {
        for (&src, &dst) in src.iter().zip(dst) {
            let d = src.abs_diff(dst) as i64;
            s1 += d;
            s2 += d * d;
        }
    }

    let s1 = s1 as f64;
    let s2 = s2 as f64;
    let n = input.size() as f64;

    let mse = s2 / n - s1 * s1 / (n * n);
    println!("MSE: {mse} ({})", mse.sqrt());

    io::save_pgm(reconstructed8.view(), "reconstructed.pgm")?;

    Ok(())
}
