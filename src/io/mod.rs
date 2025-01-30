use std::io::{BufRead, Read};

use crate::memory::{Image, ImageView};

pub fn load_pgm(path: impl AsRef<std::path::Path>) -> Result<Image<u8>, std::io::Error> {
    let mut file = std::io::BufReader::new(std::fs::OpenOptions::new().read(true).open(path)?);

    let mut line = String::new();
    file.read_line(&mut line)?;
    if line != "P5\n" {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Wrong PGM file header: {line:?}"),
        ));
    }

    let mut nums = Vec::<usize>::new();
    loop {
        line.clear();
        file.read_line(&mut line)?;

        match line.chars().next() {
            None => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    String::from("Wrong PGM file: Unexpected end of file"),
                ))
            }
            Some('#') => (),
            _ => {
                for num in line.split_ascii_whitespace() {
                    nums.push(num.parse().map_err(|err| {
                        std::io::Error::new(std::io::ErrorKind::InvalidData, err)
                    })?);
                }
                match nums.len() {
                    ..3 => (),
                    3 => break,
                    4.. => {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            String::from("Wrong PGM file format"),
                        ));
                    }
                }
            }
        }
    }

    let width = nums[0];
    let height = nums[1];
    let max_value = nums[2];

    if max_value >= 256 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            String::from("Wrong PGM file: max value too large"),
        ));
    }

    let mut image = Image::with_stride(width, height, width.next_multiple_of(64));
    for row in image.rows_mut() {
        file.read_exact(row)?;
    }

    Ok(image)
}

pub fn save_pgm(
    image: ImageView<u8>,
    path: impl AsRef<std::path::Path>,
) -> Result<(), std::io::Error> {
    use std::io::Write;
    let mut file = std::io::BufWriter::new(
        std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(path)?,
    );

    write!(file, "P5\n{} {}\n255\n", image.width(), image.height())?;

    for row in image.rows() {
        file.write_all(row)?;
    }

    Ok(())
}
