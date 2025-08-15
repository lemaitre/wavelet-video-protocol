#[allow(unused)]
use dwt::{
    daub::{Daub53, LossyDaub53},
    haar::{Haar, LossyHaar},
    predict::Predict,
    Dwt2,
};
use memory::{Image, ImageView};
use numeric::Convert;

pub mod dwt;
pub mod io;
pub mod memory;
pub mod numeric;

type Int = i16;
const N: usize = 6;
const LOW_BAND_ONLY: bool = true;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dwt = Daub53;

    let input8 = io::load_pgm("input.pgm")?;
    let input = Convert::<Image<Int>>::convert(&input8);
    let mut tmp = input.clone();
    let mut output = input.clone();

    for i in 0..N {
        let i = if LOW_BAND_ONLY { i } else { 0 };
        let w = output.width() >> i;
        let h = output.height() >> i;

        dwt.dwt2(output.subview_mut(0, 0, w, h), tmp.subview_mut(0, 0, w, h));
    }

    print_minmax(
        output.subview(0, 0, output.width() / 2, output.height() / 2),
        "LL",
    );
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

    let output8 = output.convert();
    io::save_pgm(output8.view(), "output.pgm")?;
    let mut reconstructed = output8.convert();

    println!("Written");

    for i in (0..N).rev() {
        let i = if LOW_BAND_ONLY { i } else { 0 };
        let w = output.width() >> i;
        let h = output.height() >> i;
        dwt.idwt2(
            reconstructed.subview_mut(0, 0, w, h),
            tmp.subview_mut(0, 0, w, h),
        );
    }
    let reconstructed8 = reconstructed.convert();

    println!("Reconstructed");

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

fn print_minmax(image: ImageView<'_, Int>, name: &str) {
    let mut min = Int::MAX;
    let mut max = Int::MIN;
    for row in image.into_rows() {
        for &cell in row {
            min = min.min(cell);
            max = max.max(cell);
        }
    }

    println!("{name}: [{min}; {max}]");
}
