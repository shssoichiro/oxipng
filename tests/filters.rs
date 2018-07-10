extern crate oxipng;

use oxipng::colors::{BitDepth, ColorType};
use oxipng::png;
use oxipng::OutFile;
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

    (OutFile::Path(Some(input.with_extension("out.png").to_owned())), options)
}

fn test_it_converts(
    input: &Path,
    output: &OutFile,
    opts: &oxipng::Options,
    color_type_in: ColorType,
    bit_depth_in: BitDepth,
    color_type_out: ColorType,
    bit_depth_out: BitDepth,
) {
    let png = png::PngData::new(input, opts.fix_errors).unwrap();

    assert_eq!(png.ihdr_data.color_type, color_type_in);
    assert_eq!(png.ihdr_data.bit_depth, bit_depth_in);

    match oxipng::optimize(input, output, opts) {
        Ok(_) => (),
        Err(x) => panic!("{}", x),
    };
    let output = output.path().unwrap();
    assert!(output.exists());

    let png = match png::PngData::new(output, opts.fix_errors) {
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
    let input = PathBuf::from("tests/files/filter_0_for_rgba_16.png");
    let (output, mut opts) = get_opts(&input);
    opts.filter = HashSet::new();
    opts.filter.insert(0);

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
fn filter_1_for_rgba_16() {
    let input = PathBuf::from("tests/files/filter_1_for_rgba_16.png");
    let (output, mut opts) = get_opts(&input);
    opts.filter = HashSet::new();
    opts.filter.insert(1);

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
fn filter_2_for_rgba_16() {
    let input = PathBuf::from("tests/files/filter_2_for_rgba_16.png");
    let (output, mut opts) = get_opts(&input);
    opts.filter = HashSet::new();
    opts.filter.insert(2);

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
fn filter_3_for_rgba_16() {
    let input = PathBuf::from("tests/files/filter_3_for_rgba_16.png");
    let (output, mut opts) = get_opts(&input);
    opts.filter = HashSet::new();
    opts.filter.insert(3);

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
fn filter_4_for_rgba_16() {
    let input = PathBuf::from("tests/files/filter_4_for_rgba_16.png");
    let (output, mut opts) = get_opts(&input);
    opts.filter = HashSet::new();
    opts.filter.insert(4);

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
fn filter_5_for_rgba_16() {
    let input = PathBuf::from("tests/files/filter_5_for_rgba_16.png");
    let (output, mut opts) = get_opts(&input);
    opts.filter = HashSet::new();
    opts.filter.insert(5);

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
fn filter_0_for_rgba_8() {
    let input = PathBuf::from("tests/files/filter_0_for_rgba_8.png");
    let (output, mut opts) = get_opts(&input);
    opts.filter = HashSet::new();
    opts.filter.insert(0);

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
fn filter_1_for_rgba_8() {
    let input = PathBuf::from("tests/files/filter_1_for_rgba_8.png");
    let (output, mut opts) = get_opts(&input);
    opts.filter = HashSet::new();
    opts.filter.insert(1);

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
fn filter_2_for_rgba_8() {
    let input = PathBuf::from("tests/files/filter_2_for_rgba_8.png");
    let (output, mut opts) = get_opts(&input);
    opts.filter = HashSet::new();
    opts.filter.insert(2);

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
fn filter_3_for_rgba_8() {
    let input = PathBuf::from("tests/files/filter_3_for_rgba_8.png");
    let (output, mut opts) = get_opts(&input);
    opts.filter = HashSet::new();
    opts.filter.insert(3);

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
fn filter_4_for_rgba_8() {
    let input = PathBuf::from("tests/files/filter_4_for_rgba_8.png");
    let (output, mut opts) = get_opts(&input);
    opts.filter = HashSet::new();
    opts.filter.insert(4);

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
fn filter_5_for_rgba_8() {
    let input = PathBuf::from("tests/files/filter_5_for_rgba_8.png");
    let (output, mut opts) = get_opts(&input);
    opts.filter = HashSet::new();
    opts.filter.insert(5);

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
fn filter_0_for_rgb_16() {
    let input = PathBuf::from("tests/files/filter_0_for_rgb_16.png");
    let (output, mut opts) = get_opts(&input);
    opts.filter = HashSet::new();
    opts.filter.insert(0);

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
fn filter_1_for_rgb_16() {
    let input = PathBuf::from("tests/files/filter_1_for_rgb_16.png");
    let (output, mut opts) = get_opts(&input);
    opts.filter = HashSet::new();
    opts.filter.insert(1);

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
fn filter_2_for_rgb_16() {
    let input = PathBuf::from("tests/files/filter_2_for_rgb_16.png");
    let (output, mut opts) = get_opts(&input);
    opts.filter = HashSet::new();
    opts.filter.insert(2);

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
fn filter_3_for_rgb_16() {
    let input = PathBuf::from("tests/files/filter_3_for_rgb_16.png");
    let (output, mut opts) = get_opts(&input);
    opts.filter = HashSet::new();
    opts.filter.insert(3);

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
fn filter_4_for_rgb_16() {
    let input = PathBuf::from("tests/files/filter_4_for_rgb_16.png");
    let (output, mut opts) = get_opts(&input);
    opts.filter = HashSet::new();
    opts.filter.insert(4);

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
fn filter_5_for_rgb_16() {
    let input = PathBuf::from("tests/files/filter_5_for_rgb_16.png");
    let (output, mut opts) = get_opts(&input);
    opts.filter = HashSet::new();
    opts.filter.insert(5);

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
fn filter_0_for_rgb_8() {
    let input = PathBuf::from("tests/files/filter_0_for_rgb_8.png");
    let (output, mut opts) = get_opts(&input);
    opts.filter = HashSet::new();
    opts.filter.insert(0);

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
fn filter_1_for_rgb_8() {
    let input = PathBuf::from("tests/files/filter_1_for_rgb_8.png");
    let (output, mut opts) = get_opts(&input);
    opts.filter = HashSet::new();
    opts.filter.insert(1);

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
fn filter_2_for_rgb_8() {
    let input = PathBuf::from("tests/files/filter_2_for_rgb_8.png");
    let (output, mut opts) = get_opts(&input);
    opts.filter = HashSet::new();
    opts.filter.insert(2);

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
fn filter_3_for_rgb_8() {
    let input = PathBuf::from("tests/files/filter_3_for_rgb_8.png");
    let (output, mut opts) = get_opts(&input);
    opts.filter = HashSet::new();
    opts.filter.insert(3);

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
fn filter_4_for_rgb_8() {
    let input = PathBuf::from("tests/files/filter_4_for_rgb_8.png");
    let (output, mut opts) = get_opts(&input);
    opts.filter = HashSet::new();
    opts.filter.insert(4);

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
fn filter_5_for_rgb_8() {
    let input = PathBuf::from("tests/files/filter_5_for_rgb_8.png");
    let (output, mut opts) = get_opts(&input);
    opts.filter = HashSet::new();
    opts.filter.insert(5);

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
fn filter_0_for_grayscale_alpha_16() {
    let input = PathBuf::from("tests/files/filter_0_for_grayscale_alpha_16.png");
    let (output, mut opts) = get_opts(&input);
    opts.filter = HashSet::new();
    opts.filter.insert(0);

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
fn filter_1_for_grayscale_alpha_16() {
    let input = PathBuf::from("tests/files/filter_1_for_grayscale_alpha_16.png");
    let (output, mut opts) = get_opts(&input);
    opts.filter = HashSet::new();
    opts.filter.insert(1);

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
fn filter_2_for_grayscale_alpha_16() {
    let input = PathBuf::from("tests/files/filter_2_for_grayscale_alpha_16.png");
    let (output, mut opts) = get_opts(&input);
    opts.filter = HashSet::new();
    opts.filter.insert(2);

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
fn filter_3_for_grayscale_alpha_16() {
    let input = PathBuf::from("tests/files/filter_3_for_grayscale_alpha_16.png");
    let (output, mut opts) = get_opts(&input);
    opts.filter = HashSet::new();
    opts.filter.insert(3);

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
fn filter_4_for_grayscale_alpha_16() {
    let input = PathBuf::from("tests/files/filter_4_for_grayscale_alpha_16.png");
    let (output, mut opts) = get_opts(&input);
    opts.filter = HashSet::new();
    opts.filter.insert(4);

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
fn filter_5_for_grayscale_alpha_16() {
    let input = PathBuf::from("tests/files/filter_5_for_grayscale_alpha_16.png");
    let (output, mut opts) = get_opts(&input);
    opts.filter = HashSet::new();
    opts.filter.insert(5);

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
fn filter_0_for_grayscale_alpha_8() {
    let input = PathBuf::from("tests/files/filter_0_for_grayscale_alpha_8.png");
    let (output, mut opts) = get_opts(&input);
    opts.filter = HashSet::new();
    opts.filter.insert(0);

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
fn filter_1_for_grayscale_alpha_8() {
    let input = PathBuf::from("tests/files/filter_1_for_grayscale_alpha_8.png");
    let (output, mut opts) = get_opts(&input);
    opts.filter = HashSet::new();
    opts.filter.insert(1);

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
fn filter_2_for_grayscale_alpha_8() {
    let input = PathBuf::from("tests/files/filter_2_for_grayscale_alpha_8.png");
    let (output, mut opts) = get_opts(&input);
    opts.filter = HashSet::new();
    opts.filter.insert(2);

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
fn filter_3_for_grayscale_alpha_8() {
    let input = PathBuf::from("tests/files/filter_3_for_grayscale_alpha_8.png");
    let (output, mut opts) = get_opts(&input);
    opts.filter = HashSet::new();
    opts.filter.insert(3);

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
fn filter_4_for_grayscale_alpha_8() {
    let input = PathBuf::from("tests/files/filter_4_for_grayscale_alpha_8.png");
    let (output, mut opts) = get_opts(&input);
    opts.filter = HashSet::new();
    opts.filter.insert(4);

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
fn filter_5_for_grayscale_alpha_8() {
    let input = PathBuf::from("tests/files/filter_5_for_grayscale_alpha_8.png");
    let (output, mut opts) = get_opts(&input);
    opts.filter = HashSet::new();
    opts.filter.insert(5);

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
fn filter_0_for_grayscale_16() {
    let input = PathBuf::from("tests/files/filter_0_for_grayscale_16.png");
    let (output, mut opts) = get_opts(&input);
    opts.filter = HashSet::new();
    opts.filter.insert(0);

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
fn filter_1_for_grayscale_16() {
    let input = PathBuf::from("tests/files/filter_1_for_grayscale_16.png");
    let (output, mut opts) = get_opts(&input);
    opts.filter = HashSet::new();
    opts.filter.insert(1);

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
fn filter_2_for_grayscale_16() {
    let input = PathBuf::from("tests/files/filter_2_for_grayscale_16.png");
    let (output, mut opts) = get_opts(&input);
    opts.filter = HashSet::new();
    opts.filter.insert(2);

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
fn filter_3_for_grayscale_16() {
    let input = PathBuf::from("tests/files/filter_3_for_grayscale_16.png");
    let (output, mut opts) = get_opts(&input);
    opts.filter = HashSet::new();
    opts.filter.insert(3);

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
fn filter_4_for_grayscale_16() {
    let input = PathBuf::from("tests/files/filter_4_for_grayscale_16.png");
    let (output, mut opts) = get_opts(&input);
    opts.filter = HashSet::new();
    opts.filter.insert(4);

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
fn filter_5_for_grayscale_16() {
    let input = PathBuf::from("tests/files/filter_5_for_grayscale_16.png");
    let (output, mut opts) = get_opts(&input);
    opts.filter = HashSet::new();
    opts.filter.insert(5);

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
fn filter_0_for_grayscale_8() {
    let input = PathBuf::from("tests/files/filter_0_for_grayscale_8.png");
    let (output, mut opts) = get_opts(&input);
    opts.filter = HashSet::new();
    opts.filter.insert(0);

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
fn filter_1_for_grayscale_8() {
    let input = PathBuf::from("tests/files/filter_1_for_grayscale_8.png");
    let (output, mut opts) = get_opts(&input);
    opts.filter = HashSet::new();
    opts.filter.insert(1);

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
fn filter_2_for_grayscale_8() {
    let input = PathBuf::from("tests/files/filter_2_for_grayscale_8.png");
    let (output, mut opts) = get_opts(&input);
    opts.filter = HashSet::new();
    opts.filter.insert(2);

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
fn filter_3_for_grayscale_8() {
    let input = PathBuf::from("tests/files/filter_3_for_grayscale_8.png");
    let (output, mut opts) = get_opts(&input);
    opts.filter = HashSet::new();
    opts.filter.insert(3);

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
fn filter_4_for_grayscale_8() {
    let input = PathBuf::from("tests/files/filter_4_for_grayscale_8.png");
    let (output, mut opts) = get_opts(&input);
    opts.filter = HashSet::new();
    opts.filter.insert(4);

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
fn filter_5_for_grayscale_8() {
    let input = PathBuf::from("tests/files/filter_5_for_grayscale_8.png");
    let (output, mut opts) = get_opts(&input);
    opts.filter = HashSet::new();
    opts.filter.insert(5);

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
fn filter_0_for_palette_4() {
    let input = PathBuf::from("tests/files/filter_0_for_palette_4.png");
    let (output, mut opts) = get_opts(&input);
    opts.filter = HashSet::new();
    opts.filter.insert(0);

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
fn filter_1_for_palette_4() {
    let input = PathBuf::from("tests/files/filter_1_for_palette_4.png");
    let (output, mut opts) = get_opts(&input);
    opts.filter = HashSet::new();
    opts.filter.insert(1);

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
fn filter_2_for_palette_4() {
    let input = PathBuf::from("tests/files/filter_2_for_palette_4.png");
    let (output, mut opts) = get_opts(&input);
    opts.filter = HashSet::new();
    opts.filter.insert(2);

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
fn filter_3_for_palette_4() {
    let input = PathBuf::from("tests/files/filter_3_for_palette_4.png");
    let (output, mut opts) = get_opts(&input);
    opts.filter = HashSet::new();
    opts.filter.insert(3);

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
fn filter_4_for_palette_4() {
    let input = PathBuf::from("tests/files/filter_4_for_palette_4.png");
    let (output, mut opts) = get_opts(&input);
    opts.filter = HashSet::new();
    opts.filter.insert(4);

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
fn filter_5_for_palette_4() {
    let input = PathBuf::from("tests/files/filter_5_for_palette_4.png");
    let (output, mut opts) = get_opts(&input);
    opts.filter = HashSet::new();
    opts.filter.insert(5);

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
fn filter_0_for_palette_2() {
    let input = PathBuf::from("tests/files/filter_0_for_palette_2.png");
    let (output, mut opts) = get_opts(&input);
    opts.filter = HashSet::new();
    opts.filter.insert(0);

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
fn filter_1_for_palette_2() {
    let input = PathBuf::from("tests/files/filter_1_for_palette_2.png");
    let (output, mut opts) = get_opts(&input);
    opts.filter = HashSet::new();
    opts.filter.insert(1);

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
fn filter_2_for_palette_2() {
    let input = PathBuf::from("tests/files/filter_2_for_palette_2.png");
    let (output, mut opts) = get_opts(&input);
    opts.filter = HashSet::new();
    opts.filter.insert(2);

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
fn filter_3_for_palette_2() {
    let input = PathBuf::from("tests/files/filter_3_for_palette_2.png");
    let (output, mut opts) = get_opts(&input);
    opts.filter = HashSet::new();
    opts.filter.insert(3);

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
fn filter_4_for_palette_2() {
    let input = PathBuf::from("tests/files/filter_4_for_palette_2.png");
    let (output, mut opts) = get_opts(&input);
    opts.filter = HashSet::new();
    opts.filter.insert(4);

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
fn filter_5_for_palette_2() {
    let input = PathBuf::from("tests/files/filter_5_for_palette_2.png");
    let (output, mut opts) = get_opts(&input);
    opts.filter = HashSet::new();
    opts.filter.insert(5);

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
fn filter_0_for_palette_1() {
    let input = PathBuf::from("tests/files/filter_0_for_palette_1.png");
    let (output, mut opts) = get_opts(&input);
    opts.filter = HashSet::new();
    opts.filter.insert(0);

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
fn filter_1_for_palette_1() {
    let input = PathBuf::from("tests/files/filter_1_for_palette_1.png");
    let (output, mut opts) = get_opts(&input);
    opts.filter = HashSet::new();
    opts.filter.insert(1);

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
fn filter_2_for_palette_1() {
    let input = PathBuf::from("tests/files/filter_2_for_palette_1.png");
    let (output, mut opts) = get_opts(&input);
    opts.filter = HashSet::new();
    opts.filter.insert(2);

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
fn filter_3_for_palette_1() {
    let input = PathBuf::from("tests/files/filter_3_for_palette_1.png");
    let (output, mut opts) = get_opts(&input);
    opts.filter = HashSet::new();
    opts.filter.insert(3);

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
fn filter_4_for_palette_1() {
    let input = PathBuf::from("tests/files/filter_4_for_palette_1.png");
    let (output, mut opts) = get_opts(&input);
    opts.filter = HashSet::new();
    opts.filter.insert(4);

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
fn filter_5_for_palette_1() {
    let input = PathBuf::from("tests/files/filter_5_for_palette_1.png");
    let (output, mut opts) = get_opts(&input);
    opts.filter = HashSet::new();
    opts.filter.insert(5);

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
