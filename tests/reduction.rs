extern crate oxipng;

use oxipng::colors::{AlphaOptim, BitDepth, ColorType};
use oxipng::png;
use std::collections::HashSet;
use std::error::Error;
use std::fs::remove_file;
use std::path::Path;
use std::path::PathBuf;

fn get_opts(input: &Path) -> oxipng::Options {
    let mut options = oxipng::Options::default();
    options.out_file = Some(input.with_extension("out.png").to_owned());
    options.verbosity = None;
    options.force = true;
    let mut filter = HashSet::new();
    filter.insert(0);
    options.filter = filter;

    options
}

fn test_it_converts(
    input: &Path,
    output: &Path,
    opts: &oxipng::Options,
    color_type_in: ColorType,
    bit_depth_in: BitDepth,
    color_type_out: ColorType,
    bit_depth_out: BitDepth,
) {
    let png = png::PngData::new(input, opts.fix_errors).unwrap();

    assert_eq!(png.ihdr_data.color_type, color_type_in);
    assert_eq!(png.ihdr_data.bit_depth, bit_depth_in);
    assert_eq!(png.ihdr_data.interlaced, 0);

    match oxipng::optimize(input, opts) {
        Ok(_) => (),
        Err(x) => panic!(x.description().to_owned()),
    };
    assert!(output.exists());

    let png = match png::PngData::new(output, opts.fix_errors) {
        Ok(x) => x,
        Err(x) => {
            remove_file(output).ok();
            panic!(x.description().to_owned())
        }
    };

    assert_eq!(png.ihdr_data.color_type, color_type_out);
    assert_eq!(png.ihdr_data.bit_depth, bit_depth_out);

    remove_file(output).ok();
}

#[test]
fn rgba_16_should_be_rgba_16() {
    let input = PathBuf::from("tests/files/rgba_16_should_be_rgba_16.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone().unwrap();

    test_it_converts(
        &input,
        &output,
        &opts,
        ColorType::RGBA,
        BitDepth::Sixteen,
        ColorType::RGBA,
        BitDepth::Sixteen,
    );
}

#[test]
fn rgba_16_should_be_rgba_8() {
    let input = PathBuf::from("tests/files/rgba_16_should_be_rgba_8.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone().unwrap();

    test_it_converts(
        &input,
        &output,
        &opts,
        ColorType::RGBA,
        BitDepth::Sixteen,
        ColorType::RGBA,
        BitDepth::Eight,
    );
}

#[test]
fn rgba_8_should_be_rgba_8() {
    let input = PathBuf::from("tests/files/rgba_8_should_be_rgba_8.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone().unwrap();

    test_it_converts(
        &input,
        &output,
        &opts,
        ColorType::RGBA,
        BitDepth::Eight,
        ColorType::RGBA,
        BitDepth::Eight,
    );
}

#[test]
fn rgba_16_should_be_rgb_16() {
    let input = PathBuf::from("tests/files/rgba_16_should_be_rgb_16.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone().unwrap();

    test_it_converts(
        &input,
        &output,
        &opts,
        ColorType::RGBA,
        BitDepth::Sixteen,
        ColorType::RGB,
        BitDepth::Sixteen,
    );
}

#[test]
fn rgba_16_should_be_rgb_8() {
    let input = PathBuf::from("tests/files/rgba_16_should_be_rgb_8.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone().unwrap();

    test_it_converts(
        &input,
        &output,
        &opts,
        ColorType::RGBA,
        BitDepth::Sixteen,
        ColorType::RGB,
        BitDepth::Eight,
    );
}

#[test]
fn rgba_8_should_be_rgb_8() {
    let input = PathBuf::from("tests/files/rgba_8_should_be_rgb_8.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone().unwrap();

    test_it_converts(
        &input,
        &output,
        &opts,
        ColorType::RGBA,
        BitDepth::Eight,
        ColorType::RGB,
        BitDepth::Eight,
    );
}

#[test]
fn rgba_16_should_be_palette_8() {
    let input = PathBuf::from("tests/files/rgba_16_should_be_palette_8.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone().unwrap();

    test_it_converts(
        &input,
        &output,
        &opts,
        ColorType::RGBA,
        BitDepth::Sixteen,
        ColorType::Indexed,
        BitDepth::Eight,
    );
}

#[test]
fn rgba_8_should_be_palette_8() {
    let input = PathBuf::from("tests/files/rgba_8_should_be_palette_8.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone().unwrap();

    test_it_converts(
        &input,
        &output,
        &opts,
        ColorType::RGBA,
        BitDepth::Eight,
        ColorType::Indexed,
        BitDepth::Eight,
    );
}

#[test]
fn rgba_16_should_be_palette_4() {
    let input = PathBuf::from("tests/files/rgba_16_should_be_palette_4.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone().unwrap();

    test_it_converts(
        &input,
        &output,
        &opts,
        ColorType::RGBA,
        BitDepth::Sixteen,
        ColorType::Indexed,
        BitDepth::Four,
    );
}

#[test]
fn rgba_8_should_be_palette_4() {
    let input = PathBuf::from("tests/files/rgba_8_should_be_palette_4.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone().unwrap();

    test_it_converts(
        &input,
        &output,
        &opts,
        ColorType::RGBA,
        BitDepth::Eight,
        ColorType::Indexed,
        BitDepth::Four,
    );
}

#[test]
fn rgba_16_should_be_palette_2() {
    let input = PathBuf::from("tests/files/rgba_16_should_be_palette_2.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone().unwrap();

    test_it_converts(
        &input,
        &output,
        &opts,
        ColorType::RGBA,
        BitDepth::Sixteen,
        ColorType::Indexed,
        BitDepth::Two,
    );
}

#[test]
fn rgba_8_should_be_palette_2() {
    let input = PathBuf::from("tests/files/rgba_8_should_be_palette_2.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone().unwrap();

    test_it_converts(
        &input,
        &output,
        &opts,
        ColorType::RGBA,
        BitDepth::Eight,
        ColorType::Indexed,
        BitDepth::Two,
    );
}

#[test]
fn rgba_16_should_be_palette_1() {
    let input = PathBuf::from("tests/files/rgba_16_should_be_palette_1.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone().unwrap();

    test_it_converts(
        &input,
        &output,
        &opts,
        ColorType::RGBA,
        BitDepth::Sixteen,
        ColorType::Indexed,
        BitDepth::One,
    );
}

#[test]
fn rgba_8_should_be_palette_1() {
    let input = PathBuf::from("tests/files/rgba_8_should_be_palette_1.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone().unwrap();

    test_it_converts(
        &input,
        &output,
        &opts,
        ColorType::RGBA,
        BitDepth::Eight,
        ColorType::Indexed,
        BitDepth::One,
    );
}

#[test]
fn rgba_16_should_be_grayscale_alpha_16() {
    let input = PathBuf::from("tests/files/rgba_16_should_be_grayscale_alpha_16.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone().unwrap();

    test_it_converts(
        &input,
        &output,
        &opts,
        ColorType::RGBA,
        BitDepth::Sixteen,
        ColorType::GrayscaleAlpha,
        BitDepth::Sixteen,
    );
}

#[test]
fn rgba_16_should_be_grayscale_alpha_8() {
    let input = PathBuf::from("tests/files/rgba_16_should_be_grayscale_alpha_8.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone().unwrap();

    test_it_converts(
        &input,
        &output,
        &opts,
        ColorType::RGBA,
        BitDepth::Sixteen,
        ColorType::GrayscaleAlpha,
        BitDepth::Eight,
    );
}

#[test]
fn rgba_8_should_be_grayscale_alpha_8() {
    let input = PathBuf::from("tests/files/rgba_8_should_be_grayscale_alpha_8.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone().unwrap();

    test_it_converts(
        &input,
        &output,
        &opts,
        ColorType::RGBA,
        BitDepth::Eight,
        ColorType::GrayscaleAlpha,
        BitDepth::Eight,
    );
}

#[test]
fn rgba_16_should_be_grayscale_16() {
    let input = PathBuf::from("tests/files/rgba_16_should_be_grayscale_16.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone().unwrap();

    test_it_converts(
        &input,
        &output,
        &opts,
        ColorType::RGBA,
        BitDepth::Sixteen,
        ColorType::Grayscale,
        BitDepth::Sixteen,
    );
}

#[test]
fn rgba_16_should_be_grayscale_8() {
    let input = PathBuf::from("tests/files/rgba_16_should_be_grayscale_8.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone().unwrap();

    test_it_converts(
        &input,
        &output,
        &opts,
        ColorType::RGBA,
        BitDepth::Sixteen,
        ColorType::Grayscale,
        BitDepth::Eight,
    );
}

#[test]
fn rgba_8_should_be_grayscale_8() {
    let input = PathBuf::from("tests/files/rgba_8_should_be_grayscale_8.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone().unwrap();

    test_it_converts(
        &input,
        &output,
        &opts,
        ColorType::RGBA,
        BitDepth::Eight,
        ColorType::Grayscale,
        BitDepth::Eight,
    );
}

#[test]
fn rgb_16_should_be_rgb_16() {
    let input = PathBuf::from("tests/files/rgb_16_should_be_rgb_16.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone().unwrap();

    test_it_converts(
        &input,
        &output,
        &opts,
        ColorType::RGB,
        BitDepth::Sixteen,
        ColorType::RGB,
        BitDepth::Sixteen,
    );
}

#[test]
fn rgb_16_should_be_rgb_8() {
    let input = PathBuf::from("tests/files/rgb_16_should_be_rgb_8.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone().unwrap();

    test_it_converts(
        &input,
        &output,
        &opts,
        ColorType::RGB,
        BitDepth::Sixteen,
        ColorType::RGB,
        BitDepth::Eight,
    );
}

#[test]
fn rgb_8_should_be_rgb_8() {
    let input = PathBuf::from("tests/files/rgb_8_should_be_rgb_8.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone().unwrap();

    test_it_converts(
        &input,
        &output,
        &opts,
        ColorType::RGB,
        BitDepth::Eight,
        ColorType::RGB,
        BitDepth::Eight,
    );
}

#[test]
fn rgb_16_should_be_palette_8() {
    let input = PathBuf::from("tests/files/rgb_16_should_be_palette_8.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone().unwrap();

    test_it_converts(
        &input,
        &output,
        &opts,
        ColorType::RGB,
        BitDepth::Sixteen,
        ColorType::Indexed,
        BitDepth::Eight,
    );
}

#[test]
fn rgb_8_should_be_palette_8() {
    let input = PathBuf::from("tests/files/rgb_8_should_be_palette_8.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone().unwrap();

    test_it_converts(
        &input,
        &output,
        &opts,
        ColorType::RGB,
        BitDepth::Eight,
        ColorType::Indexed,
        BitDepth::Eight,
    );
}

#[test]
fn rgb_16_should_be_palette_4() {
    let input = PathBuf::from("tests/files/rgb_16_should_be_palette_4.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone().unwrap();

    test_it_converts(
        &input,
        &output,
        &opts,
        ColorType::RGB,
        BitDepth::Sixteen,
        ColorType::Indexed,
        BitDepth::Four,
    );
}

#[test]
fn rgb_8_should_be_palette_4() {
    let input = PathBuf::from("tests/files/rgb_8_should_be_palette_4.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone().unwrap();

    test_it_converts(
        &input,
        &output,
        &opts,
        ColorType::RGB,
        BitDepth::Eight,
        ColorType::Indexed,
        BitDepth::Four,
    );
}

#[test]
fn rgb_16_should_be_palette_2() {
    let input = PathBuf::from("tests/files/rgb_16_should_be_palette_2.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone().unwrap();

    test_it_converts(
        &input,
        &output,
        &opts,
        ColorType::RGB,
        BitDepth::Sixteen,
        ColorType::Indexed,
        BitDepth::Two,
    );
}

#[test]
fn rgb_8_should_be_palette_2() {
    let input = PathBuf::from("tests/files/rgb_8_should_be_palette_2.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone().unwrap();

    test_it_converts(
        &input,
        &output,
        &opts,
        ColorType::RGB,
        BitDepth::Eight,
        ColorType::Indexed,
        BitDepth::Two,
    );
}

#[test]
fn rgb_16_should_be_palette_1() {
    let input = PathBuf::from("tests/files/rgb_16_should_be_palette_1.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone().unwrap();

    test_it_converts(
        &input,
        &output,
        &opts,
        ColorType::RGB,
        BitDepth::Sixteen,
        ColorType::Indexed,
        BitDepth::One,
    );
}

#[test]
fn rgb_8_should_be_palette_1() {
    let input = PathBuf::from("tests/files/rgb_8_should_be_palette_1.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone().unwrap();

    test_it_converts(
        &input,
        &output,
        &opts,
        ColorType::RGB,
        BitDepth::Eight,
        ColorType::Indexed,
        BitDepth::One,
    );
}

#[test]
fn rgb_16_should_be_grayscale_16() {
    let input = PathBuf::from("tests/files/rgb_16_should_be_grayscale_16.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone().unwrap();

    test_it_converts(
        &input,
        &output,
        &opts,
        ColorType::RGB,
        BitDepth::Sixteen,
        ColorType::Grayscale,
        BitDepth::Sixteen,
    );
}

#[test]
fn rgb_16_should_be_grayscale_8() {
    let input = PathBuf::from("tests/files/rgb_16_should_be_grayscale_8.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone().unwrap();

    test_it_converts(
        &input,
        &output,
        &opts,
        ColorType::RGB,
        BitDepth::Sixteen,
        ColorType::Grayscale,
        BitDepth::Eight,
    );
}

#[test]
fn rgb_8_should_be_grayscale_8() {
    let input = PathBuf::from("tests/files/rgb_8_should_be_grayscale_8.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone().unwrap();

    test_it_converts(
        &input,
        &output,
        &opts,
        ColorType::RGB,
        BitDepth::Eight,
        ColorType::Grayscale,
        BitDepth::Eight,
    );
}

#[test]
fn palette_8_should_be_palette_8() {
    let input = PathBuf::from("tests/files/palette_8_should_be_palette_8.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone().unwrap();

    test_it_converts(
        &input,
        &output,
        &opts,
        ColorType::Indexed,
        BitDepth::Eight,
        ColorType::Indexed,
        BitDepth::Eight,
    );
}

#[test]
fn palette_8_should_be_palette_4() {
    let input = PathBuf::from("tests/files/palette_8_should_be_palette_4.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone().unwrap();

    test_it_converts(
        &input,
        &output,
        &opts,
        ColorType::Indexed,
        BitDepth::Eight,
        ColorType::Indexed,
        BitDepth::Four,
    );
}

#[test]
fn palette_4_should_be_palette_4() {
    let input = PathBuf::from("tests/files/palette_4_should_be_palette_4.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone().unwrap();

    test_it_converts(
        &input,
        &output,
        &opts,
        ColorType::Indexed,
        BitDepth::Four,
        ColorType::Indexed,
        BitDepth::Four,
    );
}

#[test]
fn palette_8_should_be_palette_2() {
    let input = PathBuf::from("tests/files/palette_8_should_be_palette_2.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone().unwrap();

    test_it_converts(
        &input,
        &output,
        &opts,
        ColorType::Indexed,
        BitDepth::Eight,
        ColorType::Indexed,
        BitDepth::Two,
    );
}

#[test]
fn palette_4_should_be_palette_2() {
    let input = PathBuf::from("tests/files/palette_4_should_be_palette_2.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone().unwrap();

    test_it_converts(
        &input,
        &output,
        &opts,
        ColorType::Indexed,
        BitDepth::Four,
        ColorType::Indexed,
        BitDepth::Two,
    );
}

#[test]
fn palette_2_should_be_palette_2() {
    let input = PathBuf::from("tests/files/palette_2_should_be_palette_2.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone().unwrap();

    test_it_converts(
        &input,
        &output,
        &opts,
        ColorType::Indexed,
        BitDepth::Two,
        ColorType::Indexed,
        BitDepth::Two,
    );
}

#[test]
fn palette_8_should_be_palette_1() {
    let input = PathBuf::from("tests/files/palette_8_should_be_palette_1.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone().unwrap();

    test_it_converts(
        &input,
        &output,
        &opts,
        ColorType::Indexed,
        BitDepth::Eight,
        ColorType::Indexed,
        BitDepth::One,
    );
}

#[test]
fn palette_4_should_be_palette_1() {
    let input = PathBuf::from("tests/files/palette_4_should_be_palette_1.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone().unwrap();

    test_it_converts(
        &input,
        &output,
        &opts,
        ColorType::Indexed,
        BitDepth::Four,
        ColorType::Indexed,
        BitDepth::One,
    );
}

#[test]
fn palette_2_should_be_palette_1() {
    let input = PathBuf::from("tests/files/palette_2_should_be_palette_1.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone().unwrap();

    test_it_converts(
        &input,
        &output,
        &opts,
        ColorType::Indexed,
        BitDepth::Two,
        ColorType::Indexed,
        BitDepth::One,
    );
}

#[test]
fn palette_1_should_be_palette_1() {
    let input = PathBuf::from("tests/files/palette_1_should_be_palette_1.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone().unwrap();

    test_it_converts(
        &input,
        &output,
        &opts,
        ColorType::Indexed,
        BitDepth::One,
        ColorType::Indexed,
        BitDepth::One,
    );
}

#[test]
fn grayscale_alpha_16_should_be_grayscale_alpha_16() {
    let input = PathBuf::from("tests/files/grayscale_alpha_16_should_be_grayscale_alpha_16.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone().unwrap();

    test_it_converts(
        &input,
        &output,
        &opts,
        ColorType::GrayscaleAlpha,
        BitDepth::Sixteen,
        ColorType::GrayscaleAlpha,
        BitDepth::Sixteen,
    );
}

#[test]
fn grayscale_alpha_16_should_be_grayscale_alpha_8() {
    let input = PathBuf::from("tests/files/grayscale_alpha_16_should_be_grayscale_alpha_8.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone().unwrap();

    test_it_converts(
        &input,
        &output,
        &opts,
        ColorType::GrayscaleAlpha,
        BitDepth::Sixteen,
        ColorType::GrayscaleAlpha,
        BitDepth::Eight,
    );
}

#[test]
fn grayscale_alpha_8_should_be_grayscale_alpha_8() {
    let input = PathBuf::from("tests/files/grayscale_alpha_8_should_be_grayscale_alpha_8.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone().unwrap();

    test_it_converts(
        &input,
        &output,
        &opts,
        ColorType::GrayscaleAlpha,
        BitDepth::Eight,
        ColorType::GrayscaleAlpha,
        BitDepth::Eight,
    );
}

#[test]
fn grayscale_alpha_16_should_be_grayscale_16() {
    let input = PathBuf::from("tests/files/grayscale_alpha_16_should_be_grayscale_16.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone().unwrap();

    test_it_converts(
        &input,
        &output,
        &opts,
        ColorType::GrayscaleAlpha,
        BitDepth::Sixteen,
        ColorType::Grayscale,
        BitDepth::Sixteen,
    );
}

#[test]
fn grayscale_alpha_16_should_be_grayscale_8() {
    let input = PathBuf::from("tests/files/grayscale_alpha_16_should_be_grayscale_8.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone().unwrap();

    test_it_converts(
        &input,
        &output,
        &opts,
        ColorType::GrayscaleAlpha,
        BitDepth::Sixteen,
        ColorType::Grayscale,
        BitDepth::Eight,
    );
}

#[test]
fn grayscale_alpha_8_should_be_grayscale_8() {
    let input = PathBuf::from("tests/files/grayscale_alpha_8_should_be_grayscale_8.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone().unwrap();

    test_it_converts(
        &input,
        &output,
        &opts,
        ColorType::GrayscaleAlpha,
        BitDepth::Eight,
        ColorType::Grayscale,
        BitDepth::Eight,
    );
}

#[test]
fn grayscale_16_should_be_grayscale_16() {
    let input = PathBuf::from("tests/files/grayscale_16_should_be_grayscale_16.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone().unwrap();

    test_it_converts(
        &input,
        &output,
        &opts,
        ColorType::Grayscale,
        BitDepth::Sixteen,
        ColorType::Grayscale,
        BitDepth::Sixteen,
    );
}

#[test]
fn grayscale_16_should_be_grayscale_8() {
    let input = PathBuf::from("tests/files/grayscale_16_should_be_grayscale_8.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone().unwrap();

    test_it_converts(
        &input,
        &output,
        &opts,
        ColorType::Grayscale,
        BitDepth::Sixteen,
        ColorType::Grayscale,
        BitDepth::Eight,
    );
}

#[test]
fn grayscale_8_should_be_grayscale_8() {
    let input = PathBuf::from("tests/files/grayscale_8_should_be_grayscale_8.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone().unwrap();

    test_it_converts(
        &input,
        &output,
        &opts,
        ColorType::Grayscale,
        BitDepth::Eight,
        ColorType::Grayscale,
        BitDepth::Eight,
    );
}

#[test]
fn small_files() {
    let input = PathBuf::from("tests/files/small_files.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone().unwrap();

    test_it_converts(
        &input,
        &output,
        &opts,
        ColorType::Indexed,
        BitDepth::Eight,
        ColorType::Indexed,
        BitDepth::One,
    );
}

#[test]
fn palette_should_be_reduced_with_dupes() {
    let input = PathBuf::from("tests/files/palette_should_be_reduced_with_dupes.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone().unwrap();

    let png = png::PngData::new(&input, opts.fix_errors).unwrap();

    assert_eq!(png.ihdr_data.color_type, ColorType::Indexed);
    assert_eq!(png.ihdr_data.bit_depth, BitDepth::Eight);
    assert_eq!(png.palette.unwrap().len(), 43 * 3);

    match oxipng::optimize(&input, &opts) {
        Ok(_) => (),
        Err(x) => panic!(x.description().to_owned()),
    };
    assert!(output.exists());

    let png = match png::PngData::new(&output, opts.fix_errors) {
        Ok(x) => x,
        Err(x) => {
            remove_file(&output).ok();
            panic!(x.description().to_owned())
        }
    };

    assert_eq!(png.ihdr_data.color_type, ColorType::Indexed);
    assert_eq!(png.ihdr_data.bit_depth, BitDepth::Eight);
    assert_eq!(png.palette.unwrap().len(), 35 * 3);

    remove_file(output).ok();
}

#[test]
fn palette_should_be_reduced_with_unused() {
    let input = PathBuf::from("tests/files/palette_should_be_reduced_with_unused.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone().unwrap();

    let png = png::PngData::new(&input, opts.fix_errors).unwrap();

    assert_eq!(png.ihdr_data.color_type, ColorType::Indexed);
    assert_eq!(png.ihdr_data.bit_depth, BitDepth::Eight);
    assert_eq!(png.palette.unwrap().len(), 35 * 3);

    match oxipng::optimize(&input, &opts) {
        Ok(_) => (),
        Err(x) => panic!(x.description().to_owned()),
    };
    assert!(output.exists());

    let png = match png::PngData::new(&output, opts.fix_errors) {
        Ok(x) => x,
        Err(x) => {
            remove_file(&output).ok();
            panic!(x.description().to_owned())
        }
    };

    assert_eq!(png.ihdr_data.color_type, ColorType::Indexed);
    assert_eq!(png.ihdr_data.bit_depth, BitDepth::Eight);
    assert_eq!(png.palette.unwrap().len(), 33 * 3);

    remove_file(output).ok();
}

#[test]
fn palette_should_be_reduced_with_both() {
    let input = PathBuf::from("tests/files/palette_should_be_reduced_with_both.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone().unwrap();

    let png = png::PngData::new(&input, opts.fix_errors).unwrap();

    assert_eq!(png.ihdr_data.color_type, ColorType::Indexed);
    assert_eq!(png.ihdr_data.bit_depth, BitDepth::Eight);
    assert_eq!(png.palette.unwrap().len(), 43 * 3);

    match oxipng::optimize(&input, &opts) {
        Ok(_) => (),
        Err(x) => panic!(x.description().to_owned()),
    };
    assert!(output.exists());

    let png = match png::PngData::new(&output, opts.fix_errors) {
        Ok(x) => x,
        Err(x) => {
            remove_file(&output).ok();
            panic!(x.description().to_owned())
        }
    };

    assert_eq!(png.ihdr_data.color_type, ColorType::Indexed);
    assert_eq!(png.ihdr_data.bit_depth, BitDepth::Eight);
    assert_eq!(png.palette.unwrap().len(), 33 * 3);

    remove_file(output).ok();
}

#[test]
fn rgba_16_reduce_alpha_black() {
    let input = PathBuf::from("tests/files/rgba_16_reduce_alpha_black.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone().unwrap();

    test_it_converts(
        &input,
        &output,
        &opts,
        ColorType::RGBA,
        BitDepth::Sixteen,
        ColorType::RGBA,
        BitDepth::Eight,
    );
}

#[test]
fn rgba_8_reduce_alpha_black() {
    let input = PathBuf::from("tests/files/rgba_8_reduce_alpha_black.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone().unwrap();

    test_it_converts(
        &input,
        &output,
        &opts,
        ColorType::RGBA,
        BitDepth::Eight,
        ColorType::RGBA,
        BitDepth::Eight,
    );
}

#[test]
fn grayscale_alpha_16_reduce_alpha_black() {
    let input = PathBuf::from("tests/files/grayscale_alpha_16_reduce_alpha_black.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone().unwrap();

    test_it_converts(
        &input,
        &output,
        &opts,
        ColorType::GrayscaleAlpha,
        BitDepth::Sixteen,
        ColorType::GrayscaleAlpha,
        BitDepth::Eight,
    );
}

#[test]
fn grayscale_alpha_8_reduce_alpha_black() {
    let input = PathBuf::from("tests/files/grayscale_alpha_8_reduce_alpha_black.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone().unwrap();

    test_it_converts(
        &input,
        &output,
        &opts,
        ColorType::GrayscaleAlpha,
        BitDepth::Eight,
        ColorType::GrayscaleAlpha,
        BitDepth::Eight,
    );
}

#[test]
fn rgba_16_reduce_alpha_white() {
    let input = PathBuf::from("tests/files/rgba_16_reduce_alpha_white.png");
    let mut opts = get_opts(&input);
    opts.alphas = HashSet::with_capacity(1);
    opts.alphas.insert(AlphaOptim::White);
    let output = opts.out_file.clone().unwrap();

    test_it_converts(
        &input,
        &output,
        &opts,
        ColorType::RGBA,
        BitDepth::Sixteen,
        ColorType::RGBA,
        BitDepth::Eight,
    );
}

#[test]
fn rgba_8_reduce_alpha_white() {
    let input = PathBuf::from("tests/files/rgba_8_reduce_alpha_white.png");
    let mut opts = get_opts(&input);
    opts.alphas = HashSet::with_capacity(1);
    opts.alphas.insert(AlphaOptim::White);
    let output = opts.out_file.clone().unwrap();

    test_it_converts(
        &input,
        &output,
        &opts,
        ColorType::RGBA,
        BitDepth::Eight,
        ColorType::RGBA,
        BitDepth::Eight,
    );
}

#[test]
fn grayscale_alpha_16_reduce_alpha_white() {
    let input = PathBuf::from("tests/files/grayscale_alpha_16_reduce_alpha_white.png");
    let mut opts = get_opts(&input);
    opts.alphas = HashSet::with_capacity(1);
    opts.alphas.insert(AlphaOptim::White);
    let output = opts.out_file.clone().unwrap();

    test_it_converts(
        &input,
        &output,
        &opts,
        ColorType::GrayscaleAlpha,
        BitDepth::Sixteen,
        ColorType::GrayscaleAlpha,
        BitDepth::Eight,
    );
}

#[test]
fn grayscale_alpha_8_reduce_alpha_white() {
    let input = PathBuf::from("tests/files/grayscale_alpha_8_reduce_alpha_white.png");
    let mut opts = get_opts(&input);
    opts.alphas = HashSet::with_capacity(1);
    opts.alphas.insert(AlphaOptim::White);
    let output = opts.out_file.clone().unwrap();

    test_it_converts(
        &input,
        &output,
        &opts,
        ColorType::GrayscaleAlpha,
        BitDepth::Eight,
        ColorType::GrayscaleAlpha,
        BitDepth::Eight,
    );
}

#[test]
fn rgba_16_reduce_alpha_down() {
    let input = PathBuf::from("tests/files/rgba_16_reduce_alpha_down.png");
    let mut opts = get_opts(&input);
    opts.alphas = HashSet::with_capacity(1);
    opts.alphas.insert(AlphaOptim::Down);
    let output = opts.out_file.clone().unwrap();

    test_it_converts(
        &input,
        &output,
        &opts,
        ColorType::RGBA,
        BitDepth::Sixteen,
        ColorType::RGBA,
        BitDepth::Eight,
    );
}

#[test]
fn rgba_8_reduce_alpha_down() {
    let input = PathBuf::from("tests/files/rgba_8_reduce_alpha_down.png");
    let mut opts = get_opts(&input);
    opts.alphas = HashSet::with_capacity(1);
    opts.alphas.insert(AlphaOptim::Down);
    let output = opts.out_file.clone().unwrap();

    test_it_converts(
        &input,
        &output,
        &opts,
        ColorType::RGBA,
        BitDepth::Eight,
        ColorType::RGBA,
        BitDepth::Eight,
    );
}

#[test]
fn grayscale_alpha_16_reduce_alpha_down() {
    let input = PathBuf::from("tests/files/grayscale_alpha_16_reduce_alpha_down.png");
    let mut opts = get_opts(&input);
    opts.alphas = HashSet::with_capacity(1);
    opts.alphas.insert(AlphaOptim::Down);
    let output = opts.out_file.clone().unwrap();

    test_it_converts(
        &input,
        &output,
        &opts,
        ColorType::GrayscaleAlpha,
        BitDepth::Sixteen,
        ColorType::GrayscaleAlpha,
        BitDepth::Eight,
    );
}

#[test]
fn grayscale_alpha_8_reduce_alpha_down() {
    let input = PathBuf::from("tests/files/grayscale_alpha_8_reduce_alpha_down.png");
    let mut opts = get_opts(&input);
    opts.alphas = HashSet::with_capacity(1);
    opts.alphas.insert(AlphaOptim::Down);
    let output = opts.out_file.clone().unwrap();

    test_it_converts(
        &input,
        &output,
        &opts,
        ColorType::GrayscaleAlpha,
        BitDepth::Eight,
        ColorType::GrayscaleAlpha,
        BitDepth::Eight,
    );
}

#[test]
fn rgba_16_reduce_alpha_up() {
    let input = PathBuf::from("tests/files/rgba_16_reduce_alpha_up.png");
    let mut opts = get_opts(&input);
    opts.alphas = HashSet::with_capacity(1);
    opts.alphas.insert(AlphaOptim::Up);
    let output = opts.out_file.clone().unwrap();

    test_it_converts(
        &input,
        &output,
        &opts,
        ColorType::RGBA,
        BitDepth::Sixteen,
        ColorType::RGBA,
        BitDepth::Eight,
    );
}

#[test]
fn rgba_8_reduce_alpha_up() {
    let input = PathBuf::from("tests/files/rgba_8_reduce_alpha_up.png");
    let mut opts = get_opts(&input);
    opts.alphas = HashSet::with_capacity(1);
    opts.alphas.insert(AlphaOptim::Up);
    let output = opts.out_file.clone().unwrap();

    test_it_converts(
        &input,
        &output,
        &opts,
        ColorType::RGBA,
        BitDepth::Eight,
        ColorType::RGBA,
        BitDepth::Eight,
    );
}

#[test]
fn grayscale_alpha_16_reduce_alpha_up() {
    let input = PathBuf::from("tests/files/grayscale_alpha_16_reduce_alpha_up.png");
    let mut opts = get_opts(&input);
    opts.alphas = HashSet::with_capacity(1);
    opts.alphas.insert(AlphaOptim::Up);
    let output = opts.out_file.clone().unwrap();

    test_it_converts(
        &input,
        &output,
        &opts,
        ColorType::GrayscaleAlpha,
        BitDepth::Sixteen,
        ColorType::GrayscaleAlpha,
        BitDepth::Eight,
    );
}

#[test]
fn grayscale_alpha_8_reduce_alpha_up() {
    let input = PathBuf::from("tests/files/grayscale_alpha_8_reduce_alpha_up.png");
    let mut opts = get_opts(&input);
    opts.alphas = HashSet::with_capacity(1);
    opts.alphas.insert(AlphaOptim::Up);
    let output = opts.out_file.clone().unwrap();

    test_it_converts(
        &input,
        &output,
        &opts,
        ColorType::GrayscaleAlpha,
        BitDepth::Eight,
        ColorType::GrayscaleAlpha,
        BitDepth::Eight,
    );
}

#[test]
fn rgba_16_reduce_alpha_left() {
    let input = PathBuf::from("tests/files/rgba_16_reduce_alpha_left.png");
    let mut opts = get_opts(&input);
    opts.alphas = HashSet::with_capacity(1);
    opts.alphas.insert(AlphaOptim::Left);
    let output = opts.out_file.clone().unwrap();

    test_it_converts(
        &input,
        &output,
        &opts,
        ColorType::RGBA,
        BitDepth::Sixteen,
        ColorType::RGBA,
        BitDepth::Eight,
    );
}

#[test]
fn rgba_8_reduce_alpha_left() {
    let input = PathBuf::from("tests/files/rgba_8_reduce_alpha_left.png");
    let mut opts = get_opts(&input);
    opts.alphas = HashSet::with_capacity(1);
    opts.alphas.insert(AlphaOptim::Left);
    let output = opts.out_file.clone().unwrap();

    test_it_converts(
        &input,
        &output,
        &opts,
        ColorType::RGBA,
        BitDepth::Eight,
        ColorType::RGBA,
        BitDepth::Eight,
    );
}

#[test]
fn grayscale_alpha_16_reduce_alpha_left() {
    let input = PathBuf::from("tests/files/grayscale_alpha_16_reduce_alpha_left.png");
    let mut opts = get_opts(&input);
    opts.alphas = HashSet::with_capacity(1);
    opts.alphas.insert(AlphaOptim::Left);
    let output = opts.out_file.clone().unwrap();

    test_it_converts(
        &input,
        &output,
        &opts,
        ColorType::GrayscaleAlpha,
        BitDepth::Sixteen,
        ColorType::GrayscaleAlpha,
        BitDepth::Eight,
    );
}

#[test]
fn grayscale_alpha_8_reduce_alpha_left() {
    let input = PathBuf::from("tests/files/grayscale_alpha_8_reduce_alpha_left.png");
    let mut opts = get_opts(&input);
    opts.alphas = HashSet::with_capacity(1);
    opts.alphas.insert(AlphaOptim::Left);
    let output = opts.out_file.clone().unwrap();

    test_it_converts(
        &input,
        &output,
        &opts,
        ColorType::GrayscaleAlpha,
        BitDepth::Eight,
        ColorType::GrayscaleAlpha,
        BitDepth::Eight,
    );
}

#[test]
fn rgba_16_reduce_alpha_right() {
    let input = PathBuf::from("tests/files/rgba_16_reduce_alpha_right.png");
    let mut opts = get_opts(&input);
    opts.alphas = HashSet::with_capacity(1);
    opts.alphas.insert(AlphaOptim::Right);
    let output = opts.out_file.clone().unwrap();

    test_it_converts(
        &input,
        &output,
        &opts,
        ColorType::RGBA,
        BitDepth::Sixteen,
        ColorType::RGBA,
        BitDepth::Eight,
    );
}

#[test]
fn rgba_8_reduce_alpha_right() {
    let input = PathBuf::from("tests/files/rgba_8_reduce_alpha_right.png");
    let mut opts = get_opts(&input);
    opts.alphas = HashSet::with_capacity(1);
    opts.alphas.insert(AlphaOptim::Right);
    let output = opts.out_file.clone().unwrap();

    test_it_converts(
        &input,
        &output,
        &opts,
        ColorType::RGBA,
        BitDepth::Eight,
        ColorType::RGBA,
        BitDepth::Eight,
    );
}

#[test]
fn grayscale_alpha_16_reduce_alpha_right() {
    let input = PathBuf::from("tests/files/grayscale_alpha_16_reduce_alpha_right.png");
    let mut opts = get_opts(&input);
    opts.alphas = HashSet::with_capacity(1);
    opts.alphas.insert(AlphaOptim::Right);
    let output = opts.out_file.clone().unwrap();

    test_it_converts(
        &input,
        &output,
        &opts,
        ColorType::GrayscaleAlpha,
        BitDepth::Sixteen,
        ColorType::GrayscaleAlpha,
        BitDepth::Eight,
    );
}

#[test]
fn grayscale_alpha_8_reduce_alpha_right() {
    let input = PathBuf::from("tests/files/grayscale_alpha_8_reduce_alpha_right.png");
    let mut opts = get_opts(&input);
    opts.alphas = HashSet::with_capacity(1);
    opts.alphas.insert(AlphaOptim::Right);
    let output = opts.out_file.clone().unwrap();

    test_it_converts(
        &input,
        &output,
        &opts,
        ColorType::GrayscaleAlpha,
        BitDepth::Eight,
        ColorType::GrayscaleAlpha,
        BitDepth::Eight,
    );
}
