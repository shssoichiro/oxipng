extern crate image;
extern crate oxipng;

use image::GenericImage;
use image::Pixel;
use oxipng::png;
use std::collections::HashSet;
use std::fs::remove_file;
use std::path::Path;
use std::path::PathBuf;

fn get_opts(input: &Path) -> oxipng::Options {
    let mut options = oxipng::Options::default();
    options.out_file = input.with_extension("out.png").to_owned();
    options.verbosity = None;
    options.force = true;
    let mut filter = HashSet::new();
    filter.insert(0);
    options.filter = filter;

    options
}

fn test_it_converts(input: &Path,
                    output: &Path,
                    opts: &oxipng::Options,
                    color_type_in: png::ColorType,
                    bit_depth_in: png::BitDepth,
                    color_type_out: png::ColorType,
                    bit_depth_out: png::BitDepth) {
    let png = png::PngData::new(input, opts.fix_errors).unwrap();

    assert_eq!(png.ihdr_data.color_type, color_type_in);
    assert_eq!(png.ihdr_data.bit_depth, bit_depth_in);
    assert_eq!(png.ihdr_data.interlaced, 1);

    match oxipng::optimize(input, opts) {
        Ok(_) => (),
        Err(x) => panic!(x.to_owned()),
    };
    assert!(output.exists());

    let png = match png::PngData::new(output, opts.fix_errors) {
        Ok(x) => x,
        Err(x) => {
            remove_file(output).ok();
            panic!(x.to_owned())
        }
    };

    assert_eq!(png.ihdr_data.color_type, color_type_out);
    assert_eq!(png.ihdr_data.bit_depth, bit_depth_out);

    let old_png = image::open(input).unwrap();
    let new_png = image::open(output).unwrap();

    // Conversion should be lossless
    assert_eq!(old_png.pixels().map(|x| x.2.channels().to_owned()).collect::<Vec<Vec<u8>>>(),
               new_png.pixels().map(|x| x.2.channels().to_owned()).collect::<Vec<Vec<u8>>>());

    remove_file(output).ok();
}


#[test]
fn interlaced_rgba_16_should_be_rgba_16() {
    let input = PathBuf::from("tests/files/interlaced_rgba_16_should_be_rgba_16.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::RGBA,
                     png::BitDepth::Sixteen,
                     png::ColorType::RGBA,
                     png::BitDepth::Sixteen);
}

#[test]
fn interlaced_rgba_16_should_be_rgba_8() {
    let input = PathBuf::from("tests/files/interlaced_rgba_16_should_be_rgba_8.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::RGBA,
                     png::BitDepth::Sixteen,
                     png::ColorType::RGBA,
                     png::BitDepth::Eight);
}

#[test]
fn interlaced_rgba_8_should_be_rgba_8() {
    let input = PathBuf::from("tests/files/interlaced_rgba_8_should_be_rgba_8.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::RGBA,
                     png::BitDepth::Eight,
                     png::ColorType::RGBA,
                     png::BitDepth::Eight);
}

#[test]
fn interlaced_rgba_16_should_be_rgb_16() {
    let input = PathBuf::from("tests/files/interlaced_rgba_16_should_be_rgb_16.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::RGBA,
                     png::BitDepth::Sixteen,
                     png::ColorType::RGB,
                     png::BitDepth::Sixteen);
}

#[test]
fn interlaced_rgba_16_should_be_rgb_8() {
    let input = PathBuf::from("tests/files/interlaced_rgba_16_should_be_rgb_8.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::RGBA,
                     png::BitDepth::Sixteen,
                     png::ColorType::RGB,
                     png::BitDepth::Eight);
}

#[test]
fn interlaced_rgba_8_should_be_rgb_8() {
    let input = PathBuf::from("tests/files/interlaced_rgba_8_should_be_rgb_8.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::RGBA,
                     png::BitDepth::Eight,
                     png::ColorType::RGB,
                     png::BitDepth::Eight);
}

#[test]
fn interlaced_rgba_16_should_be_palette_8() {
    let input = PathBuf::from("tests/files/interlaced_rgba_16_should_be_palette_8.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::RGBA,
                     png::BitDepth::Sixteen,
                     png::ColorType::Indexed,
                     png::BitDepth::Eight);
}

#[test]
fn interlaced_rgba_8_should_be_palette_8() {
    let input = PathBuf::from("tests/files/interlaced_rgba_8_should_be_palette_8.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::RGBA,
                     png::BitDepth::Eight,
                     png::ColorType::Indexed,
                     png::BitDepth::Eight);
}

#[test]
fn interlaced_rgba_16_should_be_palette_4() {
    let input = PathBuf::from("tests/files/interlaced_rgba_16_should_be_palette_4.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::RGBA,
                     png::BitDepth::Sixteen,
                     png::ColorType::Indexed,
                     png::BitDepth::Four);
}

#[test]
fn interlaced_rgba_8_should_be_palette_4() {
    let input = PathBuf::from("tests/files/interlaced_rgba_8_should_be_palette_4.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::RGBA,
                     png::BitDepth::Eight,
                     png::ColorType::Indexed,
                     png::BitDepth::Four);
}

#[test]
fn interlaced_rgba_16_should_be_palette_2() {
    let input = PathBuf::from("tests/files/interlaced_rgba_16_should_be_palette_2.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::RGBA,
                     png::BitDepth::Sixteen,
                     png::ColorType::Indexed,
                     png::BitDepth::Two);
}

#[test]
fn interlaced_rgba_8_should_be_palette_2() {
    let input = PathBuf::from("tests/files/interlaced_rgba_8_should_be_palette_2.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::RGBA,
                     png::BitDepth::Eight,
                     png::ColorType::Indexed,
                     png::BitDepth::Two);
}

#[test]
fn interlaced_rgba_16_should_be_palette_1() {
    let input = PathBuf::from("tests/files/interlaced_rgba_16_should_be_palette_1.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::RGBA,
                     png::BitDepth::Sixteen,
                     png::ColorType::Indexed,
                     png::BitDepth::One);
}

#[test]
fn interlaced_rgba_8_should_be_palette_1() {
    let input = PathBuf::from("tests/files/interlaced_rgba_8_should_be_palette_1.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::RGBA,
                     png::BitDepth::Eight,
                     png::ColorType::Indexed,
                     png::BitDepth::One);
}

#[test]
fn interlaced_rgba_16_should_be_grayscale_alpha_16() {
    let input = PathBuf::from("tests/files/interlaced_rgba_16_should_be_grayscale_alpha_16.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::RGBA,
                     png::BitDepth::Sixteen,
                     png::ColorType::GrayscaleAlpha,
                     png::BitDepth::Sixteen);
}

#[test]
fn interlaced_rgba_16_should_be_grayscale_alpha_8() {
    let input = PathBuf::from("tests/files/interlaced_rgba_16_should_be_grayscale_alpha_8.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::RGBA,
                     png::BitDepth::Sixteen,
                     png::ColorType::GrayscaleAlpha,
                     png::BitDepth::Eight);
}

#[test]
fn interlaced_rgba_8_should_be_grayscale_alpha_8() {
    let input = PathBuf::from("tests/files/interlaced_rgba_8_should_be_grayscale_alpha_8.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::RGBA,
                     png::BitDepth::Eight,
                     png::ColorType::GrayscaleAlpha,
                     png::BitDepth::Eight);
}

#[test]
fn interlaced_rgba_16_should_be_grayscale_16() {
    let input = PathBuf::from("tests/files/interlaced_rgba_16_should_be_grayscale_16.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::RGBA,
                     png::BitDepth::Sixteen,
                     png::ColorType::Grayscale,
                     png::BitDepth::Sixteen);
}

#[test]
fn interlaced_rgba_16_should_be_grayscale_8() {
    let input = PathBuf::from("tests/files/interlaced_rgba_16_should_be_grayscale_8.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::RGBA,
                     png::BitDepth::Sixteen,
                     png::ColorType::Grayscale,
                     png::BitDepth::Eight);
}

#[test]
fn interlaced_rgba_8_should_be_grayscale_8() {
    let input = PathBuf::from("tests/files/interlaced_rgba_8_should_be_grayscale_8.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::RGBA,
                     png::BitDepth::Eight,
                     png::ColorType::Grayscale,
                     png::BitDepth::Eight);
}

#[test]
fn interlaced_rgba_16_should_be_palette_4_grayscale() {
    let input = PathBuf::from("tests/files/interlaced_rgba_16_should_be_palette_4_grayscale.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::RGBA,
                     png::BitDepth::Sixteen,
                     png::ColorType::Indexed,
                     png::BitDepth::Four);
}

#[test]
fn interlaced_rgba_8_should_be_palette_4_grayscale() {
    let input = PathBuf::from("tests/files/interlaced_rgba_8_should_be_palette_4_grayscale.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::RGBA,
                     png::BitDepth::Eight,
                     png::ColorType::Indexed,
                     png::BitDepth::Four);
}

#[test]
fn interlaced_rgba_16_should_be_palette_2_grayscale() {
    let input = PathBuf::from("tests/files/interlaced_rgba_16_should_be_palette_2_grayscale.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::RGBA,
                     png::BitDepth::Sixteen,
                     png::ColorType::Indexed,
                     png::BitDepth::Two);
}

#[test]
fn interlaced_rgba_8_should_be_palette_2_grayscale() {
    let input = PathBuf::from("tests/files/interlaced_rgba_8_should_be_palette_2_grayscale.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::RGBA,
                     png::BitDepth::Eight,
                     png::ColorType::Indexed,
                     png::BitDepth::Two);
}

#[test]
fn interlaced_rgba_16_should_be_palette_1_grayscale() {
    let input = PathBuf::from("tests/files/interlaced_rgba_16_should_be_palette_1_grayscale.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::RGBA,
                     png::BitDepth::Sixteen,
                     png::ColorType::Indexed,
                     png::BitDepth::One);
}

#[test]
fn interlaced_rgba_8_should_be_palette_1_grayscale() {
    let input = PathBuf::from("tests/files/interlaced_rgba_8_should_be_palette_1_grayscale.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::RGBA,
                     png::BitDepth::Eight,
                     png::ColorType::Indexed,
                     png::BitDepth::One);
}

#[test]
fn interlaced_rgb_16_should_be_rgb_16() {
    let input = PathBuf::from("tests/files/interlaced_rgb_16_should_be_rgb_16.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::RGB,
                     png::BitDepth::Sixteen,
                     png::ColorType::RGB,
                     png::BitDepth::Sixteen);
}

#[test]
fn interlaced_rgb_16_should_be_rgb_8() {
    let input = PathBuf::from("tests/files/interlaced_rgb_16_should_be_rgb_8.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::RGB,
                     png::BitDepth::Sixteen,
                     png::ColorType::RGB,
                     png::BitDepth::Eight);
}

#[test]
fn interlaced_rgb_8_should_be_rgb_8() {
    let input = PathBuf::from("tests/files/interlaced_rgb_8_should_be_rgb_8.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::RGB,
                     png::BitDepth::Eight,
                     png::ColorType::RGB,
                     png::BitDepth::Eight);
}

#[test]
fn interlaced_rgb_16_should_be_palette_8() {
    let input = PathBuf::from("tests/files/interlaced_rgb_16_should_be_palette_8.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::RGB,
                     png::BitDepth::Sixteen,
                     png::ColorType::Indexed,
                     png::BitDepth::Eight);
}

#[test]
fn interlaced_rgb_8_should_be_palette_8() {
    let input = PathBuf::from("tests/files/interlaced_rgb_8_should_be_palette_8.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::RGB,
                     png::BitDepth::Eight,
                     png::ColorType::Indexed,
                     png::BitDepth::Eight);
}

#[test]
fn interlaced_rgb_16_should_be_palette_4() {
    let input = PathBuf::from("tests/files/interlaced_rgb_16_should_be_palette_4.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::RGB,
                     png::BitDepth::Sixteen,
                     png::ColorType::Indexed,
                     png::BitDepth::Four);
}

#[test]
fn interlaced_rgb_8_should_be_palette_4() {
    let input = PathBuf::from("tests/files/interlaced_rgb_8_should_be_palette_4.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::RGB,
                     png::BitDepth::Eight,
                     png::ColorType::Indexed,
                     png::BitDepth::Four);
}

#[test]
fn interlaced_rgb_16_should_be_palette_2() {
    let input = PathBuf::from("tests/files/interlaced_rgb_16_should_be_palette_2.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::RGB,
                     png::BitDepth::Sixteen,
                     png::ColorType::Indexed,
                     png::BitDepth::Two);
}

#[test]
fn interlaced_rgb_8_should_be_palette_2() {
    let input = PathBuf::from("tests/files/interlaced_rgb_8_should_be_palette_2.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::RGB,
                     png::BitDepth::Eight,
                     png::ColorType::Indexed,
                     png::BitDepth::Two);
}

#[test]
fn interlaced_rgb_16_should_be_palette_1() {
    let input = PathBuf::from("tests/files/interlaced_rgb_16_should_be_palette_1.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::RGB,
                     png::BitDepth::Sixteen,
                     png::ColorType::Indexed,
                     png::BitDepth::One);
}

#[test]
fn interlaced_rgb_8_should_be_palette_1() {
    let input = PathBuf::from("tests/files/interlaced_rgb_8_should_be_palette_1.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::RGB,
                     png::BitDepth::Eight,
                     png::ColorType::Indexed,
                     png::BitDepth::One);
}

#[test]
fn interlaced_rgb_16_should_be_grayscale_16() {
    let input = PathBuf::from("tests/files/interlaced_rgb_16_should_be_grayscale_16.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::RGB,
                     png::BitDepth::Sixteen,
                     png::ColorType::Grayscale,
                     png::BitDepth::Sixteen);
}

#[test]
fn interlaced_rgb_16_should_be_grayscale_8() {
    let input = PathBuf::from("tests/files/interlaced_rgb_16_should_be_grayscale_8.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::RGB,
                     png::BitDepth::Sixteen,
                     png::ColorType::Grayscale,
                     png::BitDepth::Eight);
}

#[test]
fn interlaced_rgb_8_should_be_grayscale_8() {
    let input = PathBuf::from("tests/files/interlaced_rgb_8_should_be_grayscale_8.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::RGB,
                     png::BitDepth::Eight,
                     png::ColorType::Grayscale,
                     png::BitDepth::Eight);
}

#[test]
fn interlaced_rgb_16_should_be_palette_4_grayscale() {
    let input = PathBuf::from("tests/files/interlaced_rgb_16_should_be_palette_4_grayscale.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::RGB,
                     png::BitDepth::Sixteen,
                     png::ColorType::Indexed,
                     png::BitDepth::Four);
}

#[test]
fn interlaced_rgb_8_should_be_palette_4_grayscale() {
    let input = PathBuf::from("tests/files/interlaced_rgb_8_should_be_palette_4_grayscale.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::RGB,
                     png::BitDepth::Eight,
                     png::ColorType::Indexed,
                     png::BitDepth::Four);
}

#[test]
fn interlaced_rgb_16_should_be_palette_2_grayscale() {
    let input = PathBuf::from("tests/files/interlaced_rgb_16_should_be_palette_2_grayscale.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::RGB,
                     png::BitDepth::Sixteen,
                     png::ColorType::Indexed,
                     png::BitDepth::Two);
}

#[test]
fn interlaced_rgb_8_should_be_palette_2_grayscale() {
    let input = PathBuf::from("tests/files/interlaced_rgb_8_should_be_palette_2_grayscale.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::RGB,
                     png::BitDepth::Eight,
                     png::ColorType::Indexed,
                     png::BitDepth::Two);
}

#[test]
fn interlaced_rgb_16_should_be_palette_1_grayscale() {
    let input = PathBuf::from("tests/files/interlaced_rgb_16_should_be_palette_1_grayscale.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::RGB,
                     png::BitDepth::Sixteen,
                     png::ColorType::Indexed,
                     png::BitDepth::One);
}

#[test]
fn interlaced_rgb_8_should_be_palette_1_grayscale() {
    let input = PathBuf::from("tests/files/interlaced_rgb_8_should_be_palette_1_grayscale.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::RGB,
                     png::BitDepth::Eight,
                     png::ColorType::Indexed,
                     png::BitDepth::One);
}

#[test]
fn interlaced_palette_8_should_be_grayscale_8() {
    let input = PathBuf::from("tests/files/interlaced_palette_8_should_be_grayscale_8.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::Indexed,
                     png::BitDepth::Eight,
                     png::ColorType::Grayscale,
                     png::BitDepth::Eight);
}

#[test]
fn interlaced_palette_8_should_be_palette_8() {
    let input = PathBuf::from("tests/files/interlaced_palette_8_should_be_palette_8.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::Indexed,
                     png::BitDepth::Eight,
                     png::ColorType::Indexed,
                     png::BitDepth::Eight);
}

#[test]
fn interlaced_palette_8_should_be_palette_4() {
    let input = PathBuf::from("tests/files/interlaced_palette_8_should_be_palette_4.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::Indexed,
                     png::BitDepth::Eight,
                     png::ColorType::Indexed,
                     png::BitDepth::Four);
}

#[test]
fn interlaced_palette_4_should_be_palette_4() {
    let input = PathBuf::from("tests/files/interlaced_palette_4_should_be_palette_4.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::Indexed,
                     png::BitDepth::Four,
                     png::ColorType::Indexed,
                     png::BitDepth::Four);
}

#[test]
fn interlaced_palette_8_should_be_palette_2() {
    let input = PathBuf::from("tests/files/interlaced_palette_8_should_be_palette_2.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::Indexed,
                     png::BitDepth::Eight,
                     png::ColorType::Indexed,
                     png::BitDepth::Two);
}

#[test]
fn interlaced_palette_4_should_be_palette_2() {
    let input = PathBuf::from("tests/files/interlaced_palette_4_should_be_palette_2.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::Indexed,
                     png::BitDepth::Four,
                     png::ColorType::Indexed,
                     png::BitDepth::Two);
}

#[test]
fn interlaced_palette_2_should_be_palette_2() {
    let input = PathBuf::from("tests/files/interlaced_palette_2_should_be_palette_2.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::Indexed,
                     png::BitDepth::Two,
                     png::ColorType::Indexed,
                     png::BitDepth::Two);
}

#[test]
fn interlaced_palette_8_should_be_palette_1() {
    let input = PathBuf::from("tests/files/interlaced_palette_8_should_be_palette_1.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::Indexed,
                     png::BitDepth::Eight,
                     png::ColorType::Indexed,
                     png::BitDepth::One);
}

#[test]
fn interlaced_palette_4_should_be_palette_1() {
    let input = PathBuf::from("tests/files/interlaced_palette_4_should_be_palette_1.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::Indexed,
                     png::BitDepth::Four,
                     png::ColorType::Indexed,
                     png::BitDepth::One);
}

#[test]
fn interlaced_palette_2_should_be_palette_1() {
    let input = PathBuf::from("tests/files/interlaced_palette_2_should_be_palette_1.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::Indexed,
                     png::BitDepth::Two,
                     png::ColorType::Indexed,
                     png::BitDepth::One);
}

#[test]
fn interlaced_palette_1_should_be_palette_1() {
    let input = PathBuf::from("tests/files/interlaced_palette_1_should_be_palette_1.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::Indexed,
                     png::BitDepth::One,
                     png::ColorType::Indexed,
                     png::BitDepth::One);
}

#[test]
fn interlaced_grayscale_alpha_16_should_be_grayscale_alpha_16() {
    let input = PathBuf::from("tests/files/interlaced_grayscale_alpha_16_should_be_grayscale_alpha_16.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::GrayscaleAlpha,
                     png::BitDepth::Sixteen,
                     png::ColorType::GrayscaleAlpha,
                     png::BitDepth::Sixteen);
}

#[test]
fn interlaced_grayscale_alpha_16_should_be_grayscale_alpha_8() {
    let input = PathBuf::from("tests/files/interlaced_grayscale_alpha_16_should_be_grayscale_alpha_8.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::GrayscaleAlpha,
                     png::BitDepth::Sixteen,
                     png::ColorType::GrayscaleAlpha,
                     png::BitDepth::Eight);
}

#[test]
fn interlaced_grayscale_alpha_8_should_be_grayscale_alpha_8() {
    let input = PathBuf::from("tests/files/interlaced_grayscale_alpha_8_should_be_grayscale_alpha_8.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::GrayscaleAlpha,
                     png::BitDepth::Eight,
                     png::ColorType::GrayscaleAlpha,
                     png::BitDepth::Eight);
}

#[test]
fn interlaced_grayscale_alpha_16_should_be_grayscale_16() {
    let input = PathBuf::from("tests/files/interlaced_grayscale_alpha_16_should_be_grayscale_16.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::GrayscaleAlpha,
                     png::BitDepth::Sixteen,
                     png::ColorType::Grayscale,
                     png::BitDepth::Sixteen);
}

#[test]
fn interlaced_grayscale_alpha_16_should_be_grayscale_8() {
    let input = PathBuf::from("tests/files/interlaced_grayscale_alpha_16_should_be_grayscale_8.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::GrayscaleAlpha,
                     png::BitDepth::Sixteen,
                     png::ColorType::Grayscale,
                     png::BitDepth::Eight);
}

#[test]
fn interlaced_grayscale_alpha_8_should_be_grayscale_8() {
    let input = PathBuf::from("tests/files/interlaced_grayscale_alpha_8_should_be_grayscale_8.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::GrayscaleAlpha,
                     png::BitDepth::Eight,
                     png::ColorType::Grayscale,
                     png::BitDepth::Eight);
}

#[test]
fn interlaced_grayscale_alpha_16_should_be_palette_4() {
    let input = PathBuf::from("tests/files/interlaced_grayscale_alpha_16_should_be_palette_4.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::GrayscaleAlpha,
                     png::BitDepth::Sixteen,
                     png::ColorType::Indexed,
                     png::BitDepth::Four);
}

#[test]
fn interlaced_grayscale_alpha_8_should_be_palette_4() {
    let input = PathBuf::from("tests/files/interlaced_grayscale_alpha_8_should_be_palette_4.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::GrayscaleAlpha,
                     png::BitDepth::Eight,
                     png::ColorType::Indexed,
                     png::BitDepth::Four);
}

#[test]
fn interlaced_grayscale_alpha_16_should_be_palette_2() {
    let input = PathBuf::from("tests/files/interlaced_grayscale_alpha_16_should_be_palette_2.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::GrayscaleAlpha,
                     png::BitDepth::Sixteen,
                     png::ColorType::Indexed,
                     png::BitDepth::Two);
}

#[test]
fn interlaced_grayscale_alpha_8_should_be_palette_2() {
    let input = PathBuf::from("tests/files/interlaced_grayscale_alpha_8_should_be_palette_2.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::GrayscaleAlpha,
                     png::BitDepth::Eight,
                     png::ColorType::Indexed,
                     png::BitDepth::Two);
}

#[test]
fn interlaced_grayscale_alpha_16_should_be_palette_1() {
    let input = PathBuf::from("tests/files/interlaced_grayscale_alpha_16_should_be_palette_1.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::GrayscaleAlpha,
                     png::BitDepth::Sixteen,
                     png::ColorType::Indexed,
                     png::BitDepth::One);
}

#[test]
fn interlaced_grayscale_alpha_8_should_be_palette_1() {
    let input = PathBuf::from("tests/files/interlaced_grayscale_alpha_8_should_be_palette_1.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::GrayscaleAlpha,
                     png::BitDepth::Eight,
                     png::ColorType::Indexed,
                     png::BitDepth::One);
}

#[test]
fn interlaced_grayscale_16_should_be_grayscale_16() {
    let input = PathBuf::from("tests/files/interlaced_grayscale_16_should_be_grayscale_16.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::Grayscale,
                     png::BitDepth::Sixteen,
                     png::ColorType::Grayscale,
                     png::BitDepth::Sixteen);
}

#[test]
fn interlaced_grayscale_16_should_be_grayscale_8() {
    let input = PathBuf::from("tests/files/interlaced_grayscale_16_should_be_grayscale_8.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::Grayscale,
                     png::BitDepth::Sixteen,
                     png::ColorType::Grayscale,
                     png::BitDepth::Eight);
}

#[test]
fn interlaced_grayscale_16_should_be_palette_4() {
    let input = PathBuf::from("tests/files/interlaced_grayscale_16_should_be_palette_4.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::Grayscale,
                     png::BitDepth::Sixteen,
                     png::ColorType::Indexed,
                     png::BitDepth::Four);
}

#[test]
fn interlaced_grayscale_16_should_be_palette_2() {
    let input = PathBuf::from("tests/files/interlaced_grayscale_16_should_be_palette_2.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::Grayscale,
                     png::BitDepth::Sixteen,
                     png::ColorType::Indexed,
                     png::BitDepth::Two);
}

#[test]
fn interlaced_grayscale_16_should_be_palette_1() {
    let input = PathBuf::from("tests/files/interlaced_grayscale_16_should_be_palette_1.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::Grayscale,
                     png::BitDepth::Sixteen,
                     png::ColorType::Indexed,
                     png::BitDepth::One);
}

#[test]
fn interlaced_grayscale_8_should_be_grayscale_8() {
    let input = PathBuf::from("tests/files/interlaced_grayscale_8_should_be_grayscale_8.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::Grayscale,
                     png::BitDepth::Eight,
                     png::ColorType::Grayscale,
                     png::BitDepth::Eight);
}

#[test]
fn interlaced_grayscale_8_should_be_palette_4() {
    let input = PathBuf::from("tests/files/interlaced_grayscale_8_should_be_palette_4.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::Grayscale,
                     png::BitDepth::Eight,
                     png::ColorType::Indexed,
                     png::BitDepth::Four);
}

#[test]
fn interlaced_grayscale_8_should_be_palette_2() {
    let input = PathBuf::from("tests/files/interlaced_grayscale_8_should_be_palette_2.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::Grayscale,
                     png::BitDepth::Eight,
                     png::ColorType::Indexed,
                     png::BitDepth::Two);
}

#[test]
fn interlaced_grayscale_8_should_be_palette_1() {
    let input = PathBuf::from("tests/files/interlaced_grayscale_8_should_be_palette_1.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::Grayscale,
                     png::BitDepth::Eight,
                     png::ColorType::Indexed,
                     png::BitDepth::One);
}

#[test]
fn interlaced_small_files() {
    let input = PathBuf::from("tests/files/interlaced_small_files.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::Indexed,
                     png::BitDepth::Eight,
                     png::ColorType::Indexed,
                     png::BitDepth::One);
}

#[test]
fn interlaced_odd_width() {
    let input = PathBuf::from("tests/files/interlaced_odd_width.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::RGB,
                     png::BitDepth::Eight,
                     png::ColorType::RGB,
                     png::BitDepth::Eight);
}
