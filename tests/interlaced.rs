extern crate oxipng;

use oxipng::internal_tests::*;
use oxipng::{InFile, OutFile};
use std::collections::HashSet;
use std::fs::remove_file;
use std::path::Path;
use std::path::PathBuf;

fn get_opts(input: &Path) -> (OutFile, oxipng::Options) {
    let mut options = oxipng::Options::default();
    options.verbosity = None;
    options.force = true;
    let mut filter = HashSet::new();
    filter.insert(0);
    options.filter = filter;

    (
        OutFile::Path(Some(input.with_extension("out.png").to_owned())),
        options,
    )
}

fn test_it_converts(
    input: &str,
    color_type_in: ColorType,
    bit_depth_in: BitDepth,
    color_type_out: ColorType,
    bit_depth_out: BitDepth,
) {
    let input = PathBuf::from(input);
    let (output, opts) = get_opts(&input);
    let png = PngData::new(&input, opts.fix_errors).unwrap();

    assert_eq!(png.raw.ihdr.color_type, color_type_in);
    assert_eq!(png.raw.ihdr.bit_depth, bit_depth_in);
    assert_eq!(png.raw.ihdr.interlaced, 1);

    match oxipng::optimize(&InFile::Path(input), &output, &opts) {
        Ok(_) => (),
        Err(x) => panic!("{}", x),
    };
    let output = output.path().unwrap();
    assert!(output.exists());

    let png = match PngData::new(output, opts.fix_errors) {
        Ok(x) => x,
        Err(x) => {
            remove_file(output).ok();
            panic!("{}", x)
        }
    };

    assert_eq!(png.raw.ihdr.color_type, color_type_out);
    assert_eq!(png.raw.ihdr.bit_depth, bit_depth_out);

    remove_file(output).ok();
}

#[test]
fn interlaced_rgba_16_should_be_rgba_16() {
    test_it_converts(
        "tests/files/interlaced_rgba_16_should_be_rgba_16.png",
        ColorType::RGBA,
        BitDepth::Sixteen,
        ColorType::RGBA,
        BitDepth::Sixteen,
    );
}

#[test]
fn interlaced_rgba_16_should_be_rgba_8() {
    test_it_converts(
        "tests/files/interlaced_rgba_16_should_be_rgba_8.png",
        ColorType::RGBA,
        BitDepth::Sixteen,
        ColorType::RGBA,
        BitDepth::Eight,
    );
}

#[test]
fn interlaced_rgba_8_should_be_rgba_8() {
    test_it_converts(
        "tests/files/interlaced_rgba_8_should_be_rgba_8.png",
        ColorType::RGBA,
        BitDepth::Eight,
        ColorType::RGBA,
        BitDepth::Eight,
    );
}

#[test]
fn interlaced_rgba_16_should_be_rgb_16() {
    test_it_converts(
        "tests/files/interlaced_rgba_16_should_be_rgb_16.png",
        ColorType::RGBA,
        BitDepth::Sixteen,
        ColorType::RGB,
        BitDepth::Sixteen,
    );
}

#[test]
fn interlaced_rgba_16_should_be_rgb_8() {
    test_it_converts(
        "tests/files/interlaced_rgba_16_should_be_rgb_8.png",
        ColorType::RGBA,
        BitDepth::Sixteen,
        ColorType::RGB,
        BitDepth::Eight,
    );
}

#[test]
fn interlaced_rgba_8_should_be_rgb_8() {
    test_it_converts(
        "tests/files/interlaced_rgba_8_should_be_rgb_8.png",
        ColorType::RGBA,
        BitDepth::Eight,
        ColorType::RGB,
        BitDepth::Eight,
    );
}

#[test]
fn interlaced_rgba_16_should_be_palette_8() {
    test_it_converts(
        "tests/files/interlaced_rgba_16_should_be_palette_8.png",
        ColorType::RGBA,
        BitDepth::Sixteen,
        ColorType::Indexed,
        BitDepth::Eight,
    );
}

#[test]
fn interlaced_rgba_8_should_be_palette_8() {
    test_it_converts(
        "tests/files/interlaced_rgba_8_should_be_palette_8.png",
        ColorType::RGBA,
        BitDepth::Eight,
        ColorType::Indexed,
        BitDepth::Eight,
    );
}

#[test]
fn interlaced_rgba_16_should_be_palette_4() {
    test_it_converts(
        "tests/files/interlaced_rgba_16_should_be_palette_4.png",
        ColorType::RGBA,
        BitDepth::Sixteen,
        ColorType::Indexed,
        BitDepth::Four,
    );
}

#[test]
fn interlaced_rgba_8_should_be_palette_4() {
    test_it_converts(
        "tests/files/interlaced_rgba_8_should_be_palette_4.png",
        ColorType::RGBA,
        BitDepth::Eight,
        ColorType::Indexed,
        BitDepth::Four,
    );
}

#[test]
fn interlaced_rgba_16_should_be_palette_2() {
    test_it_converts(
        "tests/files/interlaced_rgba_16_should_be_palette_2.png",
        ColorType::RGBA,
        BitDepth::Sixteen,
        ColorType::Indexed,
        BitDepth::Two,
    );
}

#[test]
fn interlaced_rgba_8_should_be_palette_2() {
    test_it_converts(
        "tests/files/interlaced_rgba_8_should_be_palette_2.png",
        ColorType::RGBA,
        BitDepth::Eight,
        ColorType::Indexed,
        BitDepth::Two,
    );
}

#[test]
fn interlaced_rgba_16_should_be_palette_1() {
    test_it_converts(
        "tests/files/interlaced_rgba_16_should_be_palette_1.png",
        ColorType::RGBA,
        BitDepth::Sixteen,
        ColorType::Indexed,
        BitDepth::One,
    );
}

#[test]
fn interlaced_rgba_8_should_be_palette_1() {
    test_it_converts(
        "tests/files/interlaced_rgba_8_should_be_palette_1.png",
        ColorType::RGBA,
        BitDepth::Eight,
        ColorType::Indexed,
        BitDepth::One,
    );
}

#[test]
fn interlaced_rgba_16_should_be_grayscale_alpha_16() {
    test_it_converts(
        "tests/files/interlaced_rgba_16_should_be_grayscale_alpha_16.png",
        ColorType::RGBA,
        BitDepth::Sixteen,
        ColorType::GrayscaleAlpha,
        BitDepth::Sixteen,
    );
}

#[test]
fn interlaced_rgba_16_should_be_grayscale_alpha_8() {
    test_it_converts(
        "tests/files/interlaced_rgba_16_should_be_grayscale_alpha_8.png",
        ColorType::RGBA,
        BitDepth::Sixteen,
        ColorType::GrayscaleAlpha,
        BitDepth::Eight,
    );
}

#[test]
fn interlaced_rgba_8_should_be_grayscale_alpha_8() {
    test_it_converts(
        "tests/files/interlaced_rgba_8_should_be_grayscale_alpha_8.png",
        ColorType::RGBA,
        BitDepth::Eight,
        ColorType::GrayscaleAlpha,
        BitDepth::Eight,
    );
}

#[test]
fn interlaced_rgba_16_should_be_grayscale_16() {
    test_it_converts(
        "tests/files/interlaced_rgba_16_should_be_grayscale_16.png",
        ColorType::RGBA,
        BitDepth::Sixteen,
        ColorType::Grayscale,
        BitDepth::Sixteen,
    );
}

#[test]
fn interlaced_rgba_16_should_be_grayscale_8() {
    test_it_converts(
        "tests/files/interlaced_rgba_16_should_be_grayscale_8.png",
        ColorType::RGBA,
        BitDepth::Sixteen,
        ColorType::Grayscale,
        BitDepth::Eight,
    );
}

#[test]
fn interlaced_rgba_8_should_be_grayscale_8() {
    test_it_converts(
        "tests/files/interlaced_rgba_8_should_be_grayscale_8.png",
        ColorType::RGBA,
        BitDepth::Eight,
        ColorType::Grayscale,
        BitDepth::Eight,
    );
}

#[test]
fn interlaced_rgb_16_should_be_rgb_16() {
    test_it_converts(
        "tests/files/interlaced_rgb_16_should_be_rgb_16.png",
        ColorType::RGB,
        BitDepth::Sixteen,
        ColorType::RGB,
        BitDepth::Sixteen,
    );
}

#[test]
fn interlaced_rgb_16_should_be_rgb_8() {
    test_it_converts(
        "tests/files/interlaced_rgb_16_should_be_rgb_8.png",
        ColorType::RGB,
        BitDepth::Sixteen,
        ColorType::RGB,
        BitDepth::Eight,
    );
}

#[test]
fn interlaced_rgb_8_should_be_rgb_8() {
    test_it_converts(
        "tests/files/interlaced_rgb_8_should_be_rgb_8.png",
        ColorType::RGB,
        BitDepth::Eight,
        ColorType::RGB,
        BitDepth::Eight,
    );
}

#[test]
fn interlaced_rgb_16_should_be_palette_8() {
    test_it_converts(
        "tests/files/interlaced_rgb_16_should_be_palette_8.png",
        ColorType::RGB,
        BitDepth::Sixteen,
        ColorType::Indexed,
        BitDepth::Eight,
    );
}

#[test]
fn interlaced_rgb_8_should_be_palette_8() {
    test_it_converts(
        "tests/files/interlaced_rgb_8_should_be_palette_8.png",
        ColorType::RGB,
        BitDepth::Eight,
        ColorType::Indexed,
        BitDepth::Eight,
    );
}

#[test]
fn interlaced_rgb_16_should_be_palette_4() {
    test_it_converts(
        "tests/files/interlaced_rgb_16_should_be_palette_4.png",
        ColorType::RGB,
        BitDepth::Sixteen,
        ColorType::Indexed,
        BitDepth::Four,
    );
}

#[test]
fn interlaced_rgb_8_should_be_palette_4() {
    test_it_converts(
        "tests/files/interlaced_rgb_8_should_be_palette_4.png",
        ColorType::RGB,
        BitDepth::Eight,
        ColorType::Indexed,
        BitDepth::Four,
    );
}

#[test]
fn interlaced_rgb_16_should_be_palette_2() {
    test_it_converts(
        "tests/files/interlaced_rgb_16_should_be_palette_2.png",
        ColorType::RGB,
        BitDepth::Sixteen,
        ColorType::Indexed,
        BitDepth::Two,
    );
}

#[test]
fn interlaced_rgb_8_should_be_palette_2() {
    test_it_converts(
        "tests/files/interlaced_rgb_8_should_be_palette_2.png",
        ColorType::RGB,
        BitDepth::Eight,
        ColorType::Indexed,
        BitDepth::Two,
    );
}

#[test]
fn interlaced_rgb_16_should_be_palette_1() {
    test_it_converts(
        "tests/files/interlaced_rgb_16_should_be_palette_1.png",
        ColorType::RGB,
        BitDepth::Sixteen,
        ColorType::Indexed,
        BitDepth::One,
    );
}

#[test]
fn interlaced_rgb_8_should_be_palette_1() {
    test_it_converts(
        "tests/files/interlaced_rgb_8_should_be_palette_1.png",
        ColorType::RGB,
        BitDepth::Eight,
        ColorType::Indexed,
        BitDepth::One,
    );
}

#[test]
fn interlaced_rgb_16_should_be_grayscale_16() {
    test_it_converts(
        "tests/files/interlaced_rgb_16_should_be_grayscale_16.png",
        ColorType::RGB,
        BitDepth::Sixteen,
        ColorType::Grayscale,
        BitDepth::Sixteen,
    );
}

#[test]
fn interlaced_rgb_16_should_be_grayscale_8() {
    test_it_converts(
        "tests/files/interlaced_rgb_16_should_be_grayscale_8.png",
        ColorType::RGB,
        BitDepth::Sixteen,
        ColorType::Grayscale,
        BitDepth::Eight,
    );
}

#[test]
fn interlaced_rgb_8_should_be_grayscale_8() {
    test_it_converts(
        "tests/files/interlaced_rgb_8_should_be_grayscale_8.png",
        ColorType::RGB,
        BitDepth::Eight,
        ColorType::Grayscale,
        BitDepth::Eight,
    );
}

#[test]
fn interlaced_palette_8_should_be_palette_8() {
    test_it_converts(
        "tests/files/interlaced_palette_8_should_be_palette_8.png",
        ColorType::Indexed,
        BitDepth::Eight,
        ColorType::Indexed,
        BitDepth::Eight,
    );
}

#[test]
fn interlaced_palette_8_should_be_palette_4() {
    test_it_converts(
        "tests/files/interlaced_palette_8_should_be_palette_4.png",
        ColorType::Indexed,
        BitDepth::Eight,
        ColorType::Indexed,
        BitDepth::Four,
    );
}

#[test]
fn interlaced_palette_4_should_be_palette_4() {
    test_it_converts(
        "tests/files/interlaced_palette_4_should_be_palette_4.png",
        ColorType::Indexed,
        BitDepth::Four,
        ColorType::Indexed,
        BitDepth::Four,
    );
}

#[test]
fn interlaced_palette_8_should_be_palette_2() {
    test_it_converts(
        "tests/files/interlaced_palette_8_should_be_palette_2.png",
        ColorType::Indexed,
        BitDepth::Eight,
        ColorType::Indexed,
        BitDepth::Two,
    );
}

#[test]
fn interlaced_palette_4_should_be_palette_2() {
    test_it_converts(
        "tests/files/interlaced_palette_4_should_be_palette_2.png",
        ColorType::Indexed,
        BitDepth::Four,
        ColorType::Indexed,
        BitDepth::Two,
    );
}

#[test]
fn interlaced_palette_2_should_be_palette_2() {
    test_it_converts(
        "tests/files/interlaced_palette_2_should_be_palette_2.png",
        ColorType::Indexed,
        BitDepth::Two,
        ColorType::Indexed,
        BitDepth::Two,
    );
}

#[test]
fn interlaced_palette_8_should_be_palette_1() {
    test_it_converts(
        "tests/files/interlaced_palette_8_should_be_palette_1.png",
        ColorType::Indexed,
        BitDepth::Eight,
        ColorType::Indexed,
        BitDepth::One,
    );
}

#[test]
fn interlaced_palette_4_should_be_palette_1() {
    test_it_converts(
        "tests/files/interlaced_palette_4_should_be_palette_1.png",
        ColorType::Indexed,
        BitDepth::Four,
        ColorType::Indexed,
        BitDepth::One,
    );
}

#[test]
fn interlaced_palette_2_should_be_palette_1() {
    test_it_converts(
        "tests/files/interlaced_palette_2_should_be_palette_1.png",
        ColorType::Indexed,
        BitDepth::Two,
        ColorType::Indexed,
        BitDepth::One,
    );
}

#[test]
fn interlaced_palette_1_should_be_palette_1() {
    test_it_converts(
        "tests/files/interlaced_palette_1_should_be_palette_1.png",
        ColorType::Indexed,
        BitDepth::One,
        ColorType::Indexed,
        BitDepth::One,
    );
}

#[test]
fn interlaced_grayscale_alpha_16_should_be_grayscale_alpha_16() {
    test_it_converts(
        "tests/files/interlaced_grayscale_alpha_16_should_be_grayscale_alpha_16.png",
        ColorType::GrayscaleAlpha,
        BitDepth::Sixteen,
        ColorType::GrayscaleAlpha,
        BitDepth::Sixteen,
    );
}

#[test]
fn interlaced_grayscale_alpha_16_should_be_grayscale_alpha_8() {
    test_it_converts(
        "tests/files/interlaced_grayscale_alpha_16_should_be_grayscale_alpha_8.png",
        ColorType::GrayscaleAlpha,
        BitDepth::Sixteen,
        ColorType::GrayscaleAlpha,
        BitDepth::Eight,
    );
}

#[test]
fn interlaced_grayscale_alpha_8_should_be_grayscale_alpha_8() {
    test_it_converts(
        "tests/files/interlaced_grayscale_alpha_8_should_be_grayscale_alpha_8.png",
        ColorType::GrayscaleAlpha,
        BitDepth::Eight,
        ColorType::GrayscaleAlpha,
        BitDepth::Eight,
    );
}

#[test]
fn interlaced_grayscale_alpha_16_should_be_grayscale_16() {
    test_it_converts(
        "tests/files/interlaced_grayscale_alpha_16_should_be_grayscale_16.png",
        ColorType::GrayscaleAlpha,
        BitDepth::Sixteen,
        ColorType::Grayscale,
        BitDepth::Sixteen,
    );
}

#[test]
fn interlaced_grayscale_alpha_16_should_be_grayscale_8() {
    test_it_converts(
        "tests/files/interlaced_grayscale_alpha_16_should_be_grayscale_8.png",
        ColorType::GrayscaleAlpha,
        BitDepth::Sixteen,
        ColorType::Grayscale,
        BitDepth::Eight,
    );
}

#[test]
fn interlaced_grayscale_alpha_8_should_be_grayscale_8() {
    test_it_converts(
        "tests/files/interlaced_grayscale_alpha_8_should_be_grayscale_8.png",
        ColorType::GrayscaleAlpha,
        BitDepth::Eight,
        ColorType::Grayscale,
        BitDepth::Eight,
    );
}

#[test]
fn interlaced_grayscale_16_should_be_grayscale_16() {
    test_it_converts(
        "tests/files/interlaced_grayscale_16_should_be_grayscale_16.png",
        ColorType::Grayscale,
        BitDepth::Sixteen,
        ColorType::Grayscale,
        BitDepth::Sixteen,
    );
}

#[test]
fn interlaced_grayscale_16_should_be_grayscale_8() {
    test_it_converts(
        "tests/files/interlaced_grayscale_16_should_be_grayscale_8.png",
        ColorType::Grayscale,
        BitDepth::Sixteen,
        ColorType::Grayscale,
        BitDepth::Eight,
    );
}

#[test]
fn interlaced_grayscale_8_should_be_grayscale_8() {
    test_it_converts(
        "tests/files/interlaced_grayscale_8_should_be_grayscale_8.png",
        ColorType::Grayscale,
        BitDepth::Eight,
        ColorType::Grayscale,
        BitDepth::Eight,
    );
}

#[test]
fn interlaced_small_files() {
    test_it_converts(
        "tests/files/interlaced_small_files.png",
        ColorType::Indexed,
        BitDepth::Eight,
        ColorType::Indexed,
        BitDepth::One,
    );
}

#[test]
fn interlaced_odd_width() {
    test_it_converts(
        "tests/files/interlaced_odd_width.png",
        ColorType::RGB,
        BitDepth::Eight,
        ColorType::RGB,
        BitDepth::Eight,
    );
}
