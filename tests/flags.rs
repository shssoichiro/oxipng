extern crate oxipng;

use oxipng::colors::{BitDepth, ColorType};
use oxipng::deflate::Deflaters;
use oxipng::headers::Headers;
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

fn test_it_converts(input: &Path,
                    output: &Path,
                    opts: &oxipng::Options,
                    color_type_in: ColorType,
                    bit_depth_in: BitDepth,
                    color_type_out: ColorType,
                    bit_depth_out: BitDepth) {
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
fn verbose_mode() {
    let input = PathBuf::from("tests/files/verbose_mode.png");
    let mut opts = get_opts(&input);
    opts.verbosity = Some(1);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     ColorType::RGB,
                     BitDepth::Eight,
                     ColorType::RGB,
                     BitDepth::Eight);
}

#[test]
fn strip_headers_list() {
    let input = PathBuf::from("tests/files/strip_headers_list.png");
    let mut opts = get_opts(&input);
    opts.strip = Headers::Some(vec!["iCCP".to_owned(), "tEXt".to_owned()]);
    let output = opts.out_file.clone();

    let png = png::PngData::new(&input, opts.fix_errors).unwrap();

    assert!(png.aux_headers.contains_key("tEXt"));
    assert!(png.aux_headers.contains_key("iTXt"));
    assert!(png.aux_headers.contains_key("iCCP"));

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

    assert!(!png.aux_headers.contains_key("tEXt"));
    assert!(png.aux_headers.contains_key("iTXt"));
    assert!(!png.aux_headers.contains_key("iCCP"));

    remove_file(output).ok();
}

#[test]
fn strip_headers_safe() {
    let input = PathBuf::from("tests/files/strip_headers_safe.png");
    let mut opts = get_opts(&input);
    opts.strip = Headers::Safe;
    let output = opts.out_file.clone();

    let png = png::PngData::new(&input, opts.fix_errors).unwrap();

    assert!(png.aux_headers.contains_key("tEXt"));
    assert!(png.aux_headers.contains_key("iTXt"));
    assert!(png.aux_headers.contains_key("iCCP"));

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

    assert!(!png.aux_headers.contains_key("tEXt"));
    assert!(!png.aux_headers.contains_key("iTXt"));
    assert!(png.aux_headers.contains_key("iCCP"));

    remove_file(output).ok();
}

#[test]
fn strip_headers_all() {
    let input = PathBuf::from("tests/files/strip_headers_all.png");
    let mut opts = get_opts(&input);
    opts.strip = Headers::All;
    let output = opts.out_file.clone();

    let png = png::PngData::new(&input, opts.fix_errors).unwrap();

    assert!(png.aux_headers.contains_key("tEXt"));
    assert!(png.aux_headers.contains_key("iTXt"));
    assert!(png.aux_headers.contains_key("iCCP"));

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

    assert!(!png.aux_headers.contains_key("tEXt"));
    assert!(!png.aux_headers.contains_key("iTXt"));
    assert!(!png.aux_headers.contains_key("iCCP"));

    remove_file(output).ok();
}

#[test]
fn strip_headers_none() {
    let input = PathBuf::from("tests/files/strip_headers_none.png");
    let mut opts = get_opts(&input);
    opts.strip = Headers::None;
    let output = opts.out_file.clone();

    let png = png::PngData::new(&input, opts.fix_errors).unwrap();

    assert!(png.aux_headers.contains_key("tEXt"));
    assert!(png.aux_headers.contains_key("iTXt"));
    assert!(png.aux_headers.contains_key("iCCP"));

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

    assert!(png.aux_headers.contains_key("tEXt"));
    assert!(png.aux_headers.contains_key("iTXt"));
    assert!(png.aux_headers.contains_key("iCCP"));

    remove_file(output).ok();
}

#[test]
fn interlacing_0_to_1() {
    let input = PathBuf::from("tests/files/interlacing_0_to_1.png");
    let mut opts = get_opts(&input);
    opts.interlace = Some(1);
    let output = opts.out_file.clone();

    let png = png::PngData::new(&input, opts.fix_errors).unwrap();

    assert_eq!(png.ihdr_data.interlaced, 0);

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

    remove_file(output).ok();
}

#[test]
fn interlacing_1_to_0() {
    let input = PathBuf::from("tests/files/interlacing_1_to_0.png");
    let mut opts = get_opts(&input);
    opts.interlace = Some(0);
    let output = opts.out_file.clone();

    let png = png::PngData::new(&input, opts.fix_errors).unwrap();

    assert_eq!(png.ihdr_data.interlaced, 1);

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

    assert_eq!(png.ihdr_data.interlaced, 0);

    remove_file(output).ok();
}

#[test]
fn interlacing_0_to_1_small_files() {
    let input = PathBuf::from("tests/files/interlacing_0_to_1_small_files.png");
    let mut opts = get_opts(&input);
    opts.interlace = Some(1);
    let output = opts.out_file.clone();

    let png = png::PngData::new(&input, opts.fix_errors).unwrap();

    assert_eq!(png.ihdr_data.interlaced, 0);
    assert_eq!(png.ihdr_data.color_type, ColorType::Indexed);
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
    assert_eq!(png.ihdr_data.color_type, ColorType::Indexed);
    assert_eq!(png.ihdr_data.bit_depth, BitDepth::One);

    remove_file(output).ok();
}

#[test]
fn interlacing_1_to_0_small_files() {
    let input = PathBuf::from("tests/files/interlacing_1_to_0_small_files.png");
    let mut opts = get_opts(&input);
    opts.interlace = Some(0);
    let output = opts.out_file.clone();

    let png = png::PngData::new(&input, opts.fix_errors).unwrap();

    assert_eq!(png.ihdr_data.interlaced, 1);
    assert_eq!(png.ihdr_data.color_type, ColorType::Indexed);
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

    assert_eq!(png.ihdr_data.interlaced, 0);
    assert_eq!(png.ihdr_data.color_type, ColorType::Indexed);
    assert_eq!(png.ihdr_data.bit_depth, BitDepth::One);

    remove_file(output).ok();
}

#[test]
fn interlaced_0_to_1_other_filter_mode() {
    let input = PathBuf::from("tests/files/interlaced_0_to_1_other_filter_mode.png");
    let mut opts = get_opts(&input);
    opts.interlace = Some(1);
    let mut filter = HashSet::new();
    filter.insert(4);
    opts.filter = filter;
    let output = opts.out_file.clone();

    let png = png::PngData::new(&input, opts.fix_errors).unwrap();

    assert_eq!(png.ihdr_data.interlaced, 0);

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

    remove_file(output).ok();
}

#[test]
fn preserve_attrs() {
    let input = PathBuf::from("tests/files/preserve_attrs.png");
    let mut opts = get_opts(&input);
    opts.preserve_attrs = true;
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     ColorType::RGB,
                     BitDepth::Eight,
                     ColorType::RGB,
                     BitDepth::Eight);

    // TODO: Actually check permissions
}

#[test]
fn fix_errors() {
    let input = PathBuf::from("tests/files/fix_errors.png");
    let mut opts = get_opts(&input);
    opts.fix_errors = true;
    let output = opts.out_file.clone();

    let png = png::PngData::new(&input, opts.fix_errors).unwrap();

    assert_eq!(png.ihdr_data.color_type, ColorType::RGBA);
    assert_eq!(png.ihdr_data.bit_depth, BitDepth::Eight);

    match oxipng::optimize(&input, &opts) {
        Ok(_) => (),
        Err(x) => panic!(x.description().to_owned()),
    };
    assert!(output.exists());

    let png = match png::PngData::new(&output, false) {
        Ok(x) => x,
        Err(x) => {
            remove_file(output).ok();
            panic!(x.description().to_owned())
        }
    };

    assert_eq!(png.ihdr_data.color_type, ColorType::Grayscale);
    assert_eq!(png.ihdr_data.bit_depth, BitDepth::Eight);

    // Cannot check if pixels are equal because image crate cannot read corrupt (input) PNGs
    remove_file(output).ok();
}

#[test]
fn zopfli_mode() {
    let input = PathBuf::from("tests/files/zopfli_mode.png");
    let mut opts = get_opts(&input);
    opts.deflate = Deflaters::Zopfli;
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     ColorType::RGB,
                     BitDepth::Eight,
                     ColorType::RGB,
                     BitDepth::Eight);
}
