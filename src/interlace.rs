use headers::IhdrData;
use bit_vec::BitVec;
use png::PngImage;

#[must_use]
pub fn interlace_image(png: &PngImage) -> PngImage {
    let mut passes: Vec<BitVec> = vec![BitVec::new(); 7];
    let bits_per_pixel = png.ihdr.bit_depth.as_u8() * png.channels_per_pixel();
    for (index, line) in png.scan_lines().enumerate() {
        match index % 8 {
            // Add filter bytes to passes that will be in the output image
            0 => {
                passes[0].extend(BitVec::from_elem(8, false));
                if png.ihdr.width >= 5 {
                    passes[1].extend(BitVec::from_elem(8, false));
                }
                if png.ihdr.width >= 3 {
                    passes[3].extend(BitVec::from_elem(8, false));
                }
                if png.ihdr.width >= 2 {
                    passes[5].extend(BitVec::from_elem(8, false));
                }
            }
            4 => {
                passes[2].extend(BitVec::from_elem(8, false));
                if png.ihdr.width >= 3 {
                    passes[3].extend(BitVec::from_elem(8, false));
                }
                if png.ihdr.width >= 2 {
                    passes[5].extend(BitVec::from_elem(8, false));
                }
            }
            2 | 6 => {
                passes[4].extend(BitVec::from_elem(8, false));
                if png.ihdr.width >= 2 {
                    passes[5].extend(BitVec::from_elem(8, false));
                }
            }
            _ => {
                passes[6].extend(BitVec::from_elem(8, false));
            }
        }
        let bit_vec = BitVec::from_bytes(&line.data);
        for (i, bit) in bit_vec.iter().enumerate() {
            // Avoid moving padded 0's into new image
            if i >= (png.ihdr.width * u32::from(bits_per_pixel)) as usize {
                break;
            }
            // Copy pixels into interlaced passes
            let pix_modulo = (i / bits_per_pixel as usize) % 8;
            match index % 8 {
                0 => match pix_modulo {
                    0 => passes[0].push(bit),
                    4 => passes[1].push(bit),
                    2 | 6 => passes[3].push(bit),
                    _ => passes[5].push(bit),
                },
                4 => match pix_modulo {
                    0 | 4 => passes[2].push(bit),
                    2 | 6 => passes[3].push(bit),
                    _ => passes[5].push(bit),
                },
                2 | 6 => match pix_modulo % 2 {
                    0 => passes[4].push(bit),
                    _ => passes[5].push(bit),
                },
                _ => {
                    passes[6].push(bit);
                }
            }
        }
        // Pad end of line on each pass to get 8 bits per byte
        for pass in &mut passes {
            while pass.len() % 8 != 0 {
                pass.push(false);
            }
        }
    }

    let mut output = Vec::with_capacity(png.data.len());
    for pass in &passes {
        output.extend(pass.to_bytes());
    }

    PngImage {
        data: output,
        ihdr: IhdrData {
            interlaced: 1,
            ..png.ihdr
        },
        aux_headers: png.aux_headers.clone(),
        palette: png.palette.clone(),
        transparency_pixel: png.transparency_pixel.clone(),
    }
}

pub fn deinterlace_image(png: &PngImage) -> PngImage {
    let bits_per_pixel = png.ihdr.bit_depth.as_u8() * png.channels_per_pixel();
    let bits_per_line = 8 + bits_per_pixel as usize * png.ihdr.width as usize;
    // Initialize each output line with a starting filter byte of 0
    // as well as some blank data
    let mut lines: Vec<BitVec> =
        vec![BitVec::from_elem(bits_per_line, false); png.ihdr.height as usize];
    let mut current_pass = 1;
    let mut pass_constants = interlaced_constants(current_pass);
    let mut current_y: usize = pass_constants.y_shift as usize;
    for line in png.scan_lines() {
        let bit_vec = BitVec::from_bytes(&line.data);
        let bits_in_line = ((png.ihdr.width - u32::from(pass_constants.x_shift)
            + u32::from(pass_constants.x_step)
            - 1)
            / u32::from(pass_constants.x_step)) as usize
            * bits_per_pixel as usize;
        for (i, bit) in bit_vec.iter().enumerate() {
            // Avoid moving padded 0's into new image
            if i >= bits_in_line {
                break;
            }
            let current_x: usize = pass_constants.x_shift as usize
                + (i / bits_per_pixel as usize) * pass_constants.x_step as usize;
            // Copy this bit into the output line, offset by 8 because of filter byte
            let index = 8 + (i % bits_per_pixel as usize) + current_x * bits_per_pixel as usize;
            lines[current_y].set(index, bit);
        }
        // Calculate the next line and move to next pass if necessary
        current_y += pass_constants.y_step as usize;
        if current_y >= png.ihdr.height as usize {
            if current_pass == 7 {
                break;
            }
            current_pass += 1;
            if current_pass == 2 && png.ihdr.width <= 4 {
                current_pass += 1;
            }
            if current_pass == 3 && png.ihdr.height <= 4 {
                current_pass += 1;
            }
            pass_constants = interlaced_constants(current_pass);
            current_y = pass_constants.y_shift as usize;
        }
    }
    let mut output = Vec::with_capacity(png.data.len());
    for line in &mut lines {
        while line.len() % 8 != 0 {
            line.push(false);
        }
        output.extend(line.to_bytes());
    }
    PngImage {
        data: output,
        ihdr: IhdrData {
            interlaced: 0,
            ..png.ihdr
        },
        aux_headers: png.aux_headers.clone(),
        palette: png.palette.clone(),
        transparency_pixel: png.transparency_pixel.clone(),
    }
}

#[derive(Clone, Copy)]
struct InterlacedConstants {
    x_shift: u8,
    y_shift: u8,
    x_step: u8,
    y_step: u8,
}

fn interlaced_constants(pass: u8) -> InterlacedConstants {
    match pass {
        1 => InterlacedConstants {
            x_shift: 0,
            y_shift: 0,
            x_step: 8,
            y_step: 8,
        },
        2 => InterlacedConstants {
            x_shift: 4,
            y_shift: 0,
            x_step: 8,
            y_step: 8,
        },
        3 => InterlacedConstants {
            x_shift: 0,
            y_shift: 4,
            x_step: 4,
            y_step: 8,
        },
        4 => InterlacedConstants {
            x_shift: 2,
            y_shift: 0,
            x_step: 4,
            y_step: 4,
        },
        5 => InterlacedConstants {
            x_shift: 0,
            y_shift: 2,
            x_step: 2,
            y_step: 4,
        },
        6 => InterlacedConstants {
            x_shift: 1,
            y_shift: 0,
            x_step: 2,
            y_step: 2,
        },
        7 => InterlacedConstants {
            x_shift: 0,
            y_shift: 1,
            x_step: 1,
            y_step: 2,
        },
        _ => unreachable!(),
    }
}
