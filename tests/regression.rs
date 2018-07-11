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
    input: &str,
    custom: Option<(OutFile, oxipng::Options)>,
    color_type_in: ColorType,
    bit_depth_in: BitDepth,
    color_type_out: ColorType,
    bit_depth_out: BitDepth,
) {
    let input = PathBuf::from(input);
    let (output, opts) = custom.unwrap_or_else(|| get_opts(&input));
    let png = png::PngData::new(&input, opts.fix_errors).unwrap();

    assert_eq!(png.ihdr_data.color_type, color_type_in);
    assert_eq!(png.ihdr_data.bit_depth, bit_depth_in);

    match oxipng::optimize(&input, &output, &opts) {
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
fn issue_29() {
    test_it_converts(
        "tests/files/issue-29.png",
        None,
        ColorType::RGB,
        BitDepth::Eight,
        ColorType::RGB,
        BitDepth::Eight,
    );
}

#[test]
fn issue_42() {
    let input = PathBuf::from("tests/files/issue_42.png");
    let (output, mut opts) = get_opts(&input);
    opts.interlace = Some(1);

    let png = png::PngData::new(&input, opts.fix_errors).unwrap();

    assert_eq!(png.ihdr_data.interlaced, 0);
    assert_eq!(png.ihdr_data.color_type, ColorType::GrayscaleAlpha);
    assert_eq!(png.ihdr_data.bit_depth, BitDepth::Eight);

    match oxipng::optimize(&input, &output, &opts) {
        Ok(_) => (),
        Err(x) => panic!("{}", x),
    };
    let output = output.path().unwrap();
    assert!(output.exists());

    let png = match png::PngData::new(&output, opts.fix_errors) {
        Ok(x) => x,
        Err(x) => {
            remove_file(output).ok();
            panic!("{}", x)
        }
    };

    assert_eq!(png.ihdr_data.interlaced, 1);
    assert_eq!(png.ihdr_data.color_type, ColorType::GrayscaleAlpha);
    assert_eq!(png.ihdr_data.bit_depth, BitDepth::Eight);

    remove_file(output).ok();
}

#[test]
fn issue_52_01() {
    test_it_converts(
        "tests/files/issue-52-01.png",
        None,
        ColorType::RGBA,
        BitDepth::Eight,
        ColorType::Indexed,
        BitDepth::Eight,
    );
}

#[test]
fn issue_52_02() {
    test_it_converts(
        "tests/files/issue-52-02.png",
        None,
        ColorType::RGBA,
        BitDepth::Eight,
        ColorType::Indexed,
        BitDepth::Eight,
    );
}

#[test]
fn issue_52_03() {
    test_it_converts(
        "tests/files/issue-52-03.png",
        None,
        ColorType::RGBA,
        BitDepth::Eight,
        ColorType::Indexed,
        BitDepth::Eight,
    );
}

#[test]
fn issue_52_04() {
    test_it_converts(
        "tests/files/issue-52-04.png",
        None,
        ColorType::RGBA,
        BitDepth::Eight,
        ColorType::Indexed,
        BitDepth::One,
    );
}

#[test]
fn issue_52_05() {
    test_it_converts(
        "tests/files/issue-52-05.png",
        None,
        ColorType::RGBA,
        BitDepth::Eight,
        ColorType::Indexed,
        BitDepth::One,
    );
}

#[test]
fn issue_52_06() {
    test_it_converts(
        "tests/files/issue-52-06.png",
        None,
        ColorType::RGBA,
        BitDepth::Eight,
        ColorType::Indexed,
        BitDepth::Two,
    );
}

#[test]
fn issue_56() {
    test_it_converts(
        "tests/files/issue-56.png",
        None,
        ColorType::Indexed,
        BitDepth::Four,
        ColorType::Indexed,
        BitDepth::Four,
    );
}

#[test]
fn issue_58() {
    test_it_converts(
        "tests/files/issue-58.png",
        None,
        ColorType::Indexed,
        BitDepth::Four,
        ColorType::Indexed,
        BitDepth::Four,
    );
}

#[test]
fn issue_59() {
    test_it_converts(
        "tests/files/issue-59.png",
        None,
        ColorType::RGBA,
        BitDepth::Eight,
        ColorType::RGBA,
        BitDepth::Eight,
    );
}

#[test]
fn issue_60() {
    test_it_converts(
        "tests/files/issue-60.png",
        None,
        ColorType::RGBA,
        BitDepth::Eight,
        ColorType::GrayscaleAlpha,
        BitDepth::Eight,
    );
}

#[test]
fn issue_80() {
    test_it_converts(
        "tests/files/issue-80.png",
        None,
        ColorType::Indexed,
        BitDepth::Two,
        ColorType::Indexed,
        BitDepth::One,
    );
}

#[test]
fn issue_82() {
    test_it_converts(
        "tests/files/issue-82.png",
        None,
        ColorType::Indexed,
        BitDepth::Four,
        ColorType::Indexed,
        BitDepth::Four,
    );
}

#[test]
fn issue_89() {
    test_it_converts(
        "tests/files/issue-89.png",
        None,
        ColorType::RGBA,
        BitDepth::Eight,
        ColorType::Grayscale,
        BitDepth::Eight,
    );
}

#[test]
fn issue_92_filter_0() {
    test_it_converts(
        "tests/files/issue-92.png",
        None,
        ColorType::Grayscale,
        BitDepth::Eight,
        ColorType::Grayscale,
        BitDepth::Eight,
    );
}

#[test]
fn issue_92_filter_5() {
    let input = "tests/files/issue-92.png";
    let (_, mut opts) = get_opts(Path::new(input));
    opts.filter = [5].iter().cloned().collect();
    let output = OutFile::Path(Some(Path::new(input).with_extension("-f5-out.png").to_owned()));

    test_it_converts(
        &input,
        Some((output, opts)),
        ColorType::Grayscale,
        BitDepth::Eight,
        ColorType::Grayscale,
        BitDepth::Eight,
    );
}
