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
    filter: u8,
    color_type_in: ColorType,
    bit_depth_in: BitDepth,
    color_type_out: ColorType,
    bit_depth_out: BitDepth,
) {
    let input = PathBuf::from(input);

    let (output, mut opts) = get_opts(&input);
    let png = PngData::new(&input, opts.fix_errors).unwrap();
    opts.filter = HashSet::new();
    opts.filter.insert(filter);
    assert_eq!(png.ihdr_data.color_type, color_type_in);
    assert_eq!(png.ihdr_data.bit_depth, bit_depth_in);

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

    assert_eq!(png.ihdr_data.color_type, color_type_out);
    assert_eq!(png.ihdr_data.bit_depth, bit_depth_out);

    remove_file(output).ok();
}

#[test]
fn filter_0_for_rgba_16() {
    test_it_converts(
        "tests/files/filter_0_for_rgba_16.png",
        0,
        ColorType::RGBA,
        BitDepth::Sixteen,
        ColorType::RGBA,
        BitDepth::Sixteen,
    );
}

#[test]
fn filter_1_for_rgba_16() {
    test_it_converts(
        "tests/files/filter_1_for_rgba_16.png",
        1,
        ColorType::RGBA,
        BitDepth::Sixteen,
        ColorType::RGBA,
        BitDepth::Sixteen,
    );
}

#[test]
fn filter_2_for_rgba_16() {
    test_it_converts(
        "tests/files/filter_2_for_rgba_16.png",
        2,
        ColorType::RGBA,
        BitDepth::Sixteen,
        ColorType::RGBA,
        BitDepth::Sixteen,
    );
}

#[test]
fn filter_3_for_rgba_16() {
    test_it_converts(
        "tests/files/filter_3_for_rgba_16.png",
        3,
        ColorType::RGBA,
        BitDepth::Sixteen,
        ColorType::RGBA,
        BitDepth::Sixteen,
    );
}

#[test]
fn filter_4_for_rgba_16() {
    test_it_converts(
        "tests/files/filter_4_for_rgba_16.png",
        4,
        ColorType::RGBA,
        BitDepth::Sixteen,
        ColorType::RGBA,
        BitDepth::Sixteen,
    );
}

#[test]
fn filter_5_for_rgba_16() {
    test_it_converts(
        "tests/files/filter_5_for_rgba_16.png",
        5,
        ColorType::RGBA,
        BitDepth::Sixteen,
        ColorType::RGBA,
        BitDepth::Sixteen,
    );
}

#[test]
fn filter_0_for_rgba_8() {
    test_it_converts(
        "tests/files/filter_0_for_rgba_8.png",
        0,
        ColorType::RGBA,
        BitDepth::Eight,
        ColorType::RGBA,
        BitDepth::Eight,
    );
}

#[test]
fn filter_1_for_rgba_8() {
    test_it_converts(
        "tests/files/filter_1_for_rgba_8.png",
        1,
        ColorType::RGBA,
        BitDepth::Eight,
        ColorType::RGBA,
        BitDepth::Eight,
    );
}

#[test]
fn filter_2_for_rgba_8() {
    test_it_converts(
        "tests/files/filter_2_for_rgba_8.png",
        2,
        ColorType::RGBA,
        BitDepth::Eight,
        ColorType::RGBA,
        BitDepth::Eight,
    );
}

#[test]
fn filter_3_for_rgba_8() {
    test_it_converts(
        "tests/files/filter_3_for_rgba_8.png",
        3,
        ColorType::RGBA,
        BitDepth::Eight,
        ColorType::RGBA,
        BitDepth::Eight,
    );
}

#[test]
fn filter_4_for_rgba_8() {
    test_it_converts(
        "tests/files/filter_4_for_rgba_8.png",
        4,
        ColorType::RGBA,
        BitDepth::Eight,
        ColorType::RGBA,
        BitDepth::Eight,
    );
}

#[test]
fn filter_5_for_rgba_8() {
    test_it_converts(
        "tests/files/filter_5_for_rgba_8.png",
        5,
        ColorType::RGBA,
        BitDepth::Eight,
        ColorType::RGBA,
        BitDepth::Eight,
    );
}

#[test]
fn filter_0_for_rgb_16() {
    test_it_converts(
        "tests/files/filter_0_for_rgb_16.png",
        0,
        ColorType::RGB,
        BitDepth::Sixteen,
        ColorType::RGB,
        BitDepth::Sixteen,
    );
}

#[test]
fn filter_1_for_rgb_16() {
    test_it_converts(
        "tests/files/filter_1_for_rgb_16.png",
        1,
        ColorType::RGB,
        BitDepth::Sixteen,
        ColorType::RGB,
        BitDepth::Sixteen,
    );
}

#[test]
fn filter_2_for_rgb_16() {
    test_it_converts(
        "tests/files/filter_2_for_rgb_16.png",
        2,
        ColorType::RGB,
        BitDepth::Sixteen,
        ColorType::RGB,
        BitDepth::Sixteen,
    );
}

#[test]
fn filter_3_for_rgb_16() {
    test_it_converts(
        "tests/files/filter_3_for_rgb_16.png",
        3,
        ColorType::RGB,
        BitDepth::Sixteen,
        ColorType::RGB,
        BitDepth::Sixteen,
    );
}

#[test]
fn filter_4_for_rgb_16() {
    test_it_converts(
        "tests/files/filter_4_for_rgb_16.png",
        4,
        ColorType::RGB,
        BitDepth::Sixteen,
        ColorType::RGB,
        BitDepth::Sixteen,
    );
}

#[test]
fn filter_5_for_rgb_16() {
    test_it_converts(
        "tests/files/filter_5_for_rgb_16.png",
        5,
        ColorType::RGB,
        BitDepth::Sixteen,
        ColorType::RGB,
        BitDepth::Sixteen,
    );
}

#[test]
fn filter_0_for_rgb_8() {
    test_it_converts(
        "tests/files/filter_0_for_rgb_8.png",
        0,
        ColorType::RGB,
        BitDepth::Eight,
        ColorType::RGB,
        BitDepth::Eight,
    );
}

#[test]
fn filter_1_for_rgb_8() {
    test_it_converts(
        "tests/files/filter_1_for_rgb_8.png",
        1,
        ColorType::RGB,
        BitDepth::Eight,
        ColorType::RGB,
        BitDepth::Eight,
    );
}

#[test]
fn filter_2_for_rgb_8() {
    test_it_converts(
        "tests/files/filter_2_for_rgb_8.png",
        2,
        ColorType::RGB,
        BitDepth::Eight,
        ColorType::RGB,
        BitDepth::Eight,
    );
}

#[test]
fn filter_3_for_rgb_8() {
    test_it_converts(
        "tests/files/filter_3_for_rgb_8.png",
        3,
        ColorType::RGB,
        BitDepth::Eight,
        ColorType::RGB,
        BitDepth::Eight,
    );
}

#[test]
fn filter_4_for_rgb_8() {
    test_it_converts(
        "tests/files/filter_4_for_rgb_8.png",
        4,
        ColorType::RGB,
        BitDepth::Eight,
        ColorType::RGB,
        BitDepth::Eight,
    );
}

#[test]
fn filter_5_for_rgb_8() {
    test_it_converts(
        "tests/files/filter_5_for_rgb_8.png",
        5,
        ColorType::RGB,
        BitDepth::Eight,
        ColorType::RGB,
        BitDepth::Eight,
    );
}

#[test]
fn filter_0_for_grayscale_alpha_16() {
    test_it_converts(
        "tests/files/filter_0_for_grayscale_alpha_16.png",
        0,
        ColorType::GrayscaleAlpha,
        BitDepth::Sixteen,
        ColorType::GrayscaleAlpha,
        BitDepth::Sixteen,
    );
}

#[test]
fn filter_1_for_grayscale_alpha_16() {
    test_it_converts(
        "tests/files/filter_1_for_grayscale_alpha_16.png",
        1,
        ColorType::GrayscaleAlpha,
        BitDepth::Sixteen,
        ColorType::GrayscaleAlpha,
        BitDepth::Sixteen,
    );
}

#[test]
fn filter_2_for_grayscale_alpha_16() {
    test_it_converts(
        "tests/files/filter_2_for_grayscale_alpha_16.png",
        2,
        ColorType::GrayscaleAlpha,
        BitDepth::Sixteen,
        ColorType::GrayscaleAlpha,
        BitDepth::Sixteen,
    );
}

#[test]
fn filter_3_for_grayscale_alpha_16() {
    test_it_converts(
        "tests/files/filter_3_for_grayscale_alpha_16.png",
        3,
        ColorType::GrayscaleAlpha,
        BitDepth::Sixteen,
        ColorType::GrayscaleAlpha,
        BitDepth::Sixteen,
    );
}

#[test]
fn filter_4_for_grayscale_alpha_16() {
    test_it_converts(
        "tests/files/filter_4_for_grayscale_alpha_16.png",
        4,
        ColorType::GrayscaleAlpha,
        BitDepth::Sixteen,
        ColorType::GrayscaleAlpha,
        BitDepth::Sixteen,
    );
}

#[test]
fn filter_5_for_grayscale_alpha_16() {
    test_it_converts(
        "tests/files/filter_5_for_grayscale_alpha_16.png",
        5,
        ColorType::GrayscaleAlpha,
        BitDepth::Sixteen,
        ColorType::GrayscaleAlpha,
        BitDepth::Sixteen,
    );
}

#[test]
fn filter_0_for_grayscale_alpha_8() {
    test_it_converts(
        "tests/files/filter_0_for_grayscale_alpha_8.png",
        0,
        ColorType::GrayscaleAlpha,
        BitDepth::Eight,
        ColorType::GrayscaleAlpha,
        BitDepth::Eight,
    );
}

#[test]
fn filter_1_for_grayscale_alpha_8() {
    test_it_converts(
        "tests/files/filter_1_for_grayscale_alpha_8.png",
        1,
        ColorType::GrayscaleAlpha,
        BitDepth::Eight,
        ColorType::GrayscaleAlpha,
        BitDepth::Eight,
    );
}

#[test]
fn filter_2_for_grayscale_alpha_8() {
    test_it_converts(
        "tests/files/filter_2_for_grayscale_alpha_8.png",
        2,
        ColorType::GrayscaleAlpha,
        BitDepth::Eight,
        ColorType::GrayscaleAlpha,
        BitDepth::Eight,
    );
}

#[test]
fn filter_3_for_grayscale_alpha_8() {
    test_it_converts(
        "tests/files/filter_3_for_grayscale_alpha_8.png",
        3,
        ColorType::GrayscaleAlpha,
        BitDepth::Eight,
        ColorType::GrayscaleAlpha,
        BitDepth::Eight,
    );
}

#[test]
fn filter_4_for_grayscale_alpha_8() {
    test_it_converts(
        "tests/files/filter_4_for_grayscale_alpha_8.png",
        4,
        ColorType::GrayscaleAlpha,
        BitDepth::Eight,
        ColorType::GrayscaleAlpha,
        BitDepth::Eight,
    );
}

#[test]
fn filter_5_for_grayscale_alpha_8() {
    test_it_converts(
        "tests/files/filter_5_for_grayscale_alpha_8.png",
        5,
        ColorType::GrayscaleAlpha,
        BitDepth::Eight,
        ColorType::GrayscaleAlpha,
        BitDepth::Eight,
    );
}

#[test]
fn filter_0_for_grayscale_16() {
    test_it_converts(
        "tests/files/filter_0_for_grayscale_16.png",
        0,
        ColorType::Grayscale,
        BitDepth::Sixteen,
        ColorType::Grayscale,
        BitDepth::Sixteen,
    );
}

#[test]
fn filter_1_for_grayscale_16() {
    test_it_converts(
        "tests/files/filter_1_for_grayscale_16.png",
        1,
        ColorType::Grayscale,
        BitDepth::Sixteen,
        ColorType::Grayscale,
        BitDepth::Sixteen,
    );
}

#[test]
fn filter_2_for_grayscale_16() {
    test_it_converts(
        "tests/files/filter_2_for_grayscale_16.png",
        2,
        ColorType::Grayscale,
        BitDepth::Sixteen,
        ColorType::Grayscale,
        BitDepth::Sixteen,
    );
}

#[test]
fn filter_3_for_grayscale_16() {
    test_it_converts(
        "tests/files/filter_3_for_grayscale_16.png",
        3,
        ColorType::Grayscale,
        BitDepth::Sixteen,
        ColorType::Grayscale,
        BitDepth::Sixteen,
    );
}

#[test]
fn filter_4_for_grayscale_16() {
    test_it_converts(
        "tests/files/filter_4_for_grayscale_16.png",
        4,
        ColorType::Grayscale,
        BitDepth::Sixteen,
        ColorType::Grayscale,
        BitDepth::Sixteen,
    );
}

#[test]
fn filter_5_for_grayscale_16() {
    test_it_converts(
        "tests/files/filter_5_for_grayscale_16.png",
        5,
        ColorType::Grayscale,
        BitDepth::Sixteen,
        ColorType::Grayscale,
        BitDepth::Sixteen,
    );
}

#[test]
fn filter_0_for_grayscale_8() {
    test_it_converts(
        "tests/files/filter_0_for_grayscale_8.png",
        0,
        ColorType::Grayscale,
        BitDepth::Eight,
        ColorType::Grayscale,
        BitDepth::Eight,
    );
}

#[test]
fn filter_1_for_grayscale_8() {
    test_it_converts(
        "tests/files/filter_1_for_grayscale_8.png",
        1,
        ColorType::Grayscale,
        BitDepth::Eight,
        ColorType::Grayscale,
        BitDepth::Eight,
    );
}

#[test]
fn filter_2_for_grayscale_8() {
    test_it_converts(
        "tests/files/filter_2_for_grayscale_8.png",
        2,
        ColorType::Grayscale,
        BitDepth::Eight,
        ColorType::Grayscale,
        BitDepth::Eight,
    );
}

#[test]
fn filter_3_for_grayscale_8() {
    test_it_converts(
        "tests/files/filter_3_for_grayscale_8.png",
        3,
        ColorType::Grayscale,
        BitDepth::Eight,
        ColorType::Grayscale,
        BitDepth::Eight,
    );
}

#[test]
fn filter_4_for_grayscale_8() {
    test_it_converts(
        "tests/files/filter_4_for_grayscale_8.png",
        4,
        ColorType::Grayscale,
        BitDepth::Eight,
        ColorType::Grayscale,
        BitDepth::Eight,
    );
}

#[test]
fn filter_5_for_grayscale_8() {
    test_it_converts(
        "tests/files/filter_5_for_grayscale_8.png",
        5,
        ColorType::Grayscale,
        BitDepth::Eight,
        ColorType::Grayscale,
        BitDepth::Eight,
    );
}

#[test]
fn filter_0_for_palette_4() {
    test_it_converts(
        "tests/files/filter_0_for_palette_4.png",
        0,
        ColorType::Indexed,
        BitDepth::Four,
        ColorType::Indexed,
        BitDepth::Four,
    );
}

#[test]
fn filter_1_for_palette_4() {
    test_it_converts(
        "tests/files/filter_1_for_palette_4.png",
        1,
        ColorType::Indexed,
        BitDepth::Four,
        ColorType::Indexed,
        BitDepth::Four,
    );
}

#[test]
fn filter_2_for_palette_4() {
    test_it_converts(
        "tests/files/filter_2_for_palette_4.png",
        2,
        ColorType::Indexed,
        BitDepth::Four,
        ColorType::Indexed,
        BitDepth::Four,
    );
}

#[test]
fn filter_3_for_palette_4() {
    test_it_converts(
        "tests/files/filter_3_for_palette_4.png",
        3,
        ColorType::Indexed,
        BitDepth::Four,
        ColorType::Indexed,
        BitDepth::Four,
    );
}

#[test]
fn filter_4_for_palette_4() {
    test_it_converts(
        "tests/files/filter_4_for_palette_4.png",
        4,
        ColorType::Indexed,
        BitDepth::Four,
        ColorType::Indexed,
        BitDepth::Four,
    );
}

#[test]
fn filter_5_for_palette_4() {
    test_it_converts(
        "tests/files/filter_5_for_palette_4.png",
        5,
        ColorType::Indexed,
        BitDepth::Four,
        ColorType::Indexed,
        BitDepth::Four,
    );
}

#[test]
fn filter_0_for_palette_2() {
    test_it_converts(
        "tests/files/filter_0_for_palette_2.png",
        0,
        ColorType::Indexed,
        BitDepth::Two,
        ColorType::Indexed,
        BitDepth::Two,
    );
}

#[test]
fn filter_1_for_palette_2() {
    test_it_converts(
        "tests/files/filter_1_for_palette_2.png",
        1,
        ColorType::Indexed,
        BitDepth::Two,
        ColorType::Indexed,
        BitDepth::Two,
    );
}

#[test]
fn filter_2_for_palette_2() {
    test_it_converts(
        "tests/files/filter_2_for_palette_2.png",
        2,
        ColorType::Indexed,
        BitDepth::Two,
        ColorType::Indexed,
        BitDepth::Two,
    );
}

#[test]
fn filter_3_for_palette_2() {
    test_it_converts(
        "tests/files/filter_3_for_palette_2.png",
        3,
        ColorType::Indexed,
        BitDepth::Two,
        ColorType::Indexed,
        BitDepth::Two,
    );
}

#[test]
fn filter_4_for_palette_2() {
    test_it_converts(
        "tests/files/filter_4_for_palette_2.png",
        4,
        ColorType::Indexed,
        BitDepth::Two,
        ColorType::Indexed,
        BitDepth::Two,
    );
}

#[test]
fn filter_5_for_palette_2() {
    test_it_converts(
        "tests/files/filter_5_for_palette_2.png",
        5,
        ColorType::Indexed,
        BitDepth::Two,
        ColorType::Indexed,
        BitDepth::Two,
    );
}

#[test]
fn filter_0_for_palette_1() {
    test_it_converts(
        "tests/files/filter_0_for_palette_1.png",
        0,
        ColorType::Indexed,
        BitDepth::One,
        ColorType::Indexed,
        BitDepth::One,
    );
}

#[test]
fn filter_1_for_palette_1() {
    test_it_converts(
        "tests/files/filter_1_for_palette_1.png",
        1,
        ColorType::Indexed,
        BitDepth::One,
        ColorType::Indexed,
        BitDepth::One,
    );
}

#[test]
fn filter_2_for_palette_1() {
    test_it_converts(
        "tests/files/filter_2_for_palette_1.png",
        2,
        ColorType::Indexed,
        BitDepth::One,
        ColorType::Indexed,
        BitDepth::One,
    );
}

#[test]
fn filter_3_for_palette_1() {
    test_it_converts(
        "tests/files/filter_3_for_palette_1.png",
        3,
        ColorType::Indexed,
        BitDepth::One,
        ColorType::Indexed,
        BitDepth::One,
    );
}

#[test]
fn filter_4_for_palette_1() {
    test_it_converts(
        "tests/files/filter_4_for_palette_1.png",
        4,
        ColorType::Indexed,
        BitDepth::One,
        ColorType::Indexed,
        BitDepth::One,
    );
}

#[test]
fn filter_5_for_palette_1() {
    test_it_converts(
        "tests/files/filter_5_for_palette_1.png",
        5,
        ColorType::Indexed,
        BitDepth::One,
        ColorType::Indexed,
        BitDepth::One,
    );
}
