extern crate oxipng;

use oxipng::colors::{BitDepth, ColorType};
use oxipng::png;
use std::collections::HashSet;
use std::error::Error;
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
fn issue_29() {
    let input = PathBuf::from("tests/files/issue-29.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

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
fn issue_42() {
    let input = PathBuf::from("tests/files/issue_42.png");
    let mut opts = get_opts(&input);
    opts.interlace = Some(1);
    let output = opts.out_file.clone();

    let png = png::PngData::new(&input, opts.fix_errors).unwrap();

    assert_eq!(png.ihdr_data.interlaced, 0);
    assert_eq!(png.ihdr_data.color_type, ColorType::GrayscaleAlpha);
    assert_eq!(png.ihdr_data.bit_depth, BitDepth::Eight);

    match oxipng::optimize(&input, &opts) {
        Ok(_) => (),
        Err(x) => panic!(x.description().to_owned()),
    };
    assert!(output.exists());

    let png = match png::PngData::new(&output, opts.fix_errors) {
        Ok(x) => x,
        Err(x) => {
            remove_file(output).ok();
            panic!(x.description().to_owned())
        }
    };

    assert_eq!(png.ihdr_data.interlaced, 1);
    assert_eq!(png.ihdr_data.color_type, ColorType::GrayscaleAlpha);
    assert_eq!(png.ihdr_data.bit_depth, BitDepth::Eight);

    remove_file(output).ok();
}

#[test]
fn issue_52_01() {
    let input = PathBuf::from("tests/files/issue-52-01.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

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
fn issue_52_02() {
    let input = PathBuf::from("tests/files/issue-52-02.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

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
fn issue_52_03() {
    let input = PathBuf::from("tests/files/issue-52-03.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

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
fn issue_52_04() {
    let input = PathBuf::from("tests/files/issue-52-04.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

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
fn issue_52_05() {
    let input = PathBuf::from("tests/files/issue-52-05.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

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
fn issue_52_06() {
    let input = PathBuf::from("tests/files/issue-52-06.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

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
fn issue_56() {
    let input = PathBuf::from("tests/files/issue-56.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

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
fn issue_58() {
    let input = PathBuf::from("tests/files/issue-58.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

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
fn issue_59() {
    let input = PathBuf::from("tests/files/issue-59.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

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
fn issue_60() {
    let input = PathBuf::from("tests/files/issue-60.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

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
fn issue_80() {
    let input = PathBuf::from("tests/files/issue-80.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

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
fn issue_82() {
    let input = PathBuf::from("tests/files/issue-82.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

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
