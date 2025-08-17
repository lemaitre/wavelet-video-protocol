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
const ENCODE: bool = true;

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

    // encode
    if ENCODE {
        for row in output.rows_mut() {
            for cell in row {
                *cell = encode(*cell);
            }
        }
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

    // decode
    if ENCODE {
        for row in reconstructed.rows_mut() {
            for cell in row {
                *cell = decode(*cell);
            }
        }
    }

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

    let variance = s2 / n - s1 * s1 / (n * n);
    let mse = s2 / n;
    let psnr = 10. * (255. * 255. / mse).log10();
    println!(
        "PSNR: {psnr:<6.2} MSE: {mse:<9.3} VARIANCE: {variance:<9.3} STDDEV: {:<9.3}",
        variance.sqrt()
    );

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

#[allow(unused)]
fn scale_down(x: Int, t: Int, n: Int) -> Int {
    if x < -t {
        ((x + t) / n) - t
    } else if x > t {
        ((x - t) / n) + t
    } else {
        x
    }
}

#[allow(unused)]
fn scale_up(x: Int, t: Int, n: Int) -> Int {
    if x < -t {
        ((x + t) * n) - t
    } else if x > t {
        ((x - t) * n) + t
    } else {
        x
    }
}

#[allow(unused)]
fn contract(x: Int, t: Int, n: Int) -> Int {
    if x < -t {
        (x + t) / n
    } else if x > t {
        (x - t) / n
    } else {
        0
    }
}

#[allow(unused)]
fn expand(x: Int, t: Int, n: Int) -> Int {
    if x < 0 {
        (x * n) - t
    } else if x > 0 {
        (x * n) + t
    } else {
        0
    }
}

fn encode(x: Int) -> Int {
    x / 4
    // scale_down(x, 64, 3)

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
fn decode(x: Int) -> Int {
    x * 4
    // scale_up(x, 64, 3)

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
