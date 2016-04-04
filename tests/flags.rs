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
    let mut filter = HashSet::new();
    filter.insert(0);
    let mut compression = HashSet::new();
    compression.insert(9);
    let mut memory = HashSet::new();
    memory.insert(9);
    let mut strategies = HashSet::new();
    for i in 0..4 {
        strategies.insert(i);
    }

    oxipng::Options {
        backup: false,
        out_file: input.with_extension("out.png").to_owned(),
        out_dir: None,
        stdout: false,
        pretend: false,
        recursive: false,
        fix_errors: false,
        force: true,
        clobber: true,
        create: true,
        preserve_attrs: false,
        verbosity: None,
        filter: filter,
        interlace: None,
        compression: compression,
        memory: memory,
        strategies: strategies,
        window: 15,
        bit_depth_reduction: true,
        color_type_reduction: true,
        palette_reduction: true,
        idat_recoding: true,
        strip: png::Headers::None,
        use_heuristics: false,
    }
}

fn test_it_converts(input: &Path,
                    output: &Path,
                    opts: &oxipng::Options,
                    color_type_in: png::ColorType,
                    bit_depth_in: png::BitDepth,
                    color_type_out: png::ColorType,
                    bit_depth_out: png::BitDepth) {
    let png = png::PngData::new(input).unwrap();

    assert!(png.ihdr_data.color_type == color_type_in);
    assert!(png.ihdr_data.bit_depth == bit_depth_in);

    match oxipng::optimize(input, opts) {
        Ok(_) => (),
        Err(x) => panic!(x.to_owned()),
    };
    assert!(output.exists());

    let png = match png::PngData::new(output) {
        Ok(x) => x,
        Err(x) => {
            remove_file(output).ok();
            panic!(x.to_owned())
        }
    };

    assert!(png.ihdr_data.color_type == color_type_out);
    assert!(png.ihdr_data.bit_depth == bit_depth_out);

    let old_png = image::open(input).unwrap();
    let new_png = image::open(output).unwrap();

    // Conversion should be lossless
    assert!(old_png.pixels().map(|x| x.2.channels().to_owned()).collect::<Vec<Vec<u8>>>() ==
            new_png.pixels().map(|x| x.2.channels().to_owned()).collect::<Vec<Vec<u8>>>());

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
                     png::ColorType::RGB,
                     png::BitDepth::Eight,
                     png::ColorType::RGB,
                     png::BitDepth::Eight);
}

#[test]
fn strip_headers_list() {
    let input = PathBuf::from("tests/files/strip_headers_list.png");
    let mut opts = get_opts(&input);
    opts.strip = png::Headers::Some(vec!["iCCP".to_owned(), "tEXt".to_owned()]);
    let output = opts.out_file.clone();

    let png = png::PngData::new(&input).unwrap();

    assert!(png.aux_headers.contains_key("tEXt"));
    assert!(png.aux_headers.contains_key("iTXt"));
    assert!(png.aux_headers.contains_key("iCCP"));

    match oxipng::optimize(&input, &opts) {
        Ok(_) => (),
        Err(x) => panic!(x.to_owned()),
    };
    assert!(output.exists());

    let png = match png::PngData::new(&output) {
        Ok(x) => x,
        Err(x) => {
            remove_file(output).ok();
            panic!(x.to_owned())
        }
    };

    assert!(!png.aux_headers.contains_key("tEXt"));
    assert!(png.aux_headers.contains_key("iTXt"));
    assert!(!png.aux_headers.contains_key("iCCP"));

    let old_png = image::open(&input).unwrap();
    let new_png = image::open(&output).unwrap();

    // Conversion should be lossless
    assert!(old_png.pixels().map(|x| x.2.channels().to_owned()).collect::<Vec<Vec<u8>>>() ==
            new_png.pixels().map(|x| x.2.channels().to_owned()).collect::<Vec<Vec<u8>>>());

    remove_file(output).ok();
}

#[test]
fn strip_headers_safe() {
    let input = PathBuf::from("tests/files/strip_headers_safe.png");
    let mut opts = get_opts(&input);
    opts.strip = png::Headers::Safe;
    let output = opts.out_file.clone();

    let png = png::PngData::new(&input).unwrap();

    assert!(png.aux_headers.contains_key("tEXt"));
    assert!(png.aux_headers.contains_key("iTXt"));
    assert!(png.aux_headers.contains_key("iCCP"));

    match oxipng::optimize(&input, &opts) {
        Ok(_) => (),
        Err(x) => panic!(x.to_owned()),
    };
    assert!(output.exists());

    let png = match png::PngData::new(&output) {
        Ok(x) => x,
        Err(x) => {
            remove_file(output).ok();
            panic!(x.to_owned())
        }
    };

    assert!(!png.aux_headers.contains_key("tEXt"));
    assert!(!png.aux_headers.contains_key("iTXt"));
    assert!(png.aux_headers.contains_key("iCCP"));

    let old_png = image::open(&input).unwrap();
    let new_png = image::open(&output).unwrap();

    // Conversion should be lossless
    assert!(old_png.pixels().map(|x| x.2.channels().to_owned()).collect::<Vec<Vec<u8>>>() ==
            new_png.pixels().map(|x| x.2.channels().to_owned()).collect::<Vec<Vec<u8>>>());

    remove_file(output).ok();
}

#[test]
fn strip_headers_all() {
    let input = PathBuf::from("tests/files/strip_headers_all.png");
    let mut opts = get_opts(&input);
    opts.strip = png::Headers::All;
    let output = opts.out_file.clone();

    let png = png::PngData::new(&input).unwrap();

    assert!(png.aux_headers.contains_key("tEXt"));
    assert!(png.aux_headers.contains_key("iTXt"));
    assert!(png.aux_headers.contains_key("iCCP"));

    match oxipng::optimize(&input, &opts) {
        Ok(_) => (),
        Err(x) => panic!(x.to_owned()),
    };
    assert!(output.exists());

    let png = match png::PngData::new(&output) {
        Ok(x) => x,
        Err(x) => {
            remove_file(output).ok();
            panic!(x.to_owned())
        }
    };

    assert!(!png.aux_headers.contains_key("tEXt"));
    assert!(!png.aux_headers.contains_key("iTXt"));
    assert!(!png.aux_headers.contains_key("iCCP"));

    let old_png = image::open(&input).unwrap();
    let new_png = image::open(&output).unwrap();

    // Conversion should be lossless
    assert!(old_png.pixels().map(|x| x.2.channels().to_owned()).collect::<Vec<Vec<u8>>>() ==
            new_png.pixels().map(|x| x.2.channels().to_owned()).collect::<Vec<Vec<u8>>>());

    remove_file(output).ok();
}

#[test]
fn strip_headers_none() {
    let input = PathBuf::from("tests/files/strip_headers_none.png");
    let mut opts = get_opts(&input);
    opts.strip = png::Headers::None;
    let output = opts.out_file.clone();

    let png = png::PngData::new(&input).unwrap();

    assert!(png.aux_headers.contains_key("tEXt"));
    assert!(png.aux_headers.contains_key("iTXt"));
    assert!(png.aux_headers.contains_key("iCCP"));

    match oxipng::optimize(&input, &opts) {
        Ok(_) => (),
        Err(x) => panic!(x.to_owned()),
    };
    assert!(output.exists());

    let png = match png::PngData::new(&output) {
        Ok(x) => x,
        Err(x) => {
            remove_file(output).ok();
            panic!(x.to_owned())
        }
    };

    assert!(png.aux_headers.contains_key("tEXt"));
    assert!(png.aux_headers.contains_key("iTXt"));
    assert!(png.aux_headers.contains_key("iCCP"));

    let old_png = image::open(&input).unwrap();
    let new_png = image::open(&output).unwrap();

    // Conversion should be lossless
    assert!(old_png.pixels().map(|x| x.2.channels().to_owned()).collect::<Vec<Vec<u8>>>() ==
            new_png.pixels().map(|x| x.2.channels().to_owned()).collect::<Vec<Vec<u8>>>());

    remove_file(output).ok();
}

#[test]
fn interlacing_0_to_1() {
    let input = PathBuf::from("tests/files/interlacing_0_to_1.png");
    let mut opts = get_opts(&input);
    opts.interlace = Some(1);
    let output = opts.out_file.clone();

    let png = png::PngData::new(&input).unwrap();

    assert!(png.ihdr_data.interlaced == 0);

    match oxipng::optimize(&input, &opts) {
        Ok(_) => (),
        Err(x) => panic!(x.to_owned()),
    };
    assert!(output.exists());

    let png = match png::PngData::new(&output) {
        Ok(x) => x,
        Err(x) => {
            remove_file(output).ok();
            panic!(x.to_owned())
        }
    };

    assert!(png.ihdr_data.interlaced == 1);

    let old_png = image::open(&input).unwrap();
    let new_png = image::open(&output).unwrap();

    // Conversion should be lossless
    assert!(old_png.pixels().map(|x| x.2.channels().to_owned()).collect::<Vec<Vec<u8>>>() ==
            new_png.pixels().map(|x| x.2.channels().to_owned()).collect::<Vec<Vec<u8>>>());

    remove_file(output).ok();
}

#[test]
fn interlacing_1_to_0() {
    let input = PathBuf::from("tests/files/interlacing_1_to_0.png");
    let mut opts = get_opts(&input);
    opts.interlace = Some(0);
    let output = opts.out_file.clone();

    let png = png::PngData::new(&input).unwrap();

    assert!(png.ihdr_data.interlaced == 1);

    match oxipng::optimize(&input, &opts) {
        Ok(_) => (),
        Err(x) => panic!(x.to_owned()),
    };
    assert!(output.exists());

    let png = match png::PngData::new(&output) {
        Ok(x) => x,
        Err(x) => {
            remove_file(output).ok();
            panic!(x.to_owned())
        }
    };

    assert!(png.ihdr_data.interlaced == 0);

    let old_png = image::open(&input).unwrap();
    let new_png = image::open(&output).unwrap();

    // Conversion should be lossless
    assert!(old_png.pixels().map(|x| x.2.channels().to_owned()).collect::<Vec<Vec<u8>>>() ==
            new_png.pixels().map(|x| x.2.channels().to_owned()).collect::<Vec<Vec<u8>>>());

    remove_file(output).ok();
}

#[test]
fn interlacing_0_to_1_small_files() {
    let input = PathBuf::from("tests/files/interlacing_0_to_1_small_files.png");
    let mut opts = get_opts(&input);
    opts.interlace = Some(1);
    let output = opts.out_file.clone();

    let png = png::PngData::new(&input).unwrap();

    assert!(png.ihdr_data.interlaced == 0);
    assert!(png.ihdr_data.color_type == png::ColorType::Indexed);
    assert!(png.ihdr_data.bit_depth == png::BitDepth::Eight);

    match oxipng::optimize(&input, &opts) {
        Ok(_) => (),
        Err(x) => panic!(x.to_owned()),
    };
    assert!(output.exists());

    let png = match png::PngData::new(&output) {
        Ok(x) => x,
        Err(x) => {
            remove_file(output).ok();
            panic!(x.to_owned())
        }
    };

    assert!(png.ihdr_data.interlaced == 1);
    assert!(png.ihdr_data.color_type == png::ColorType::Indexed);
    assert!(png.ihdr_data.bit_depth == png::BitDepth::One);

    let old_png = image::open(&input).unwrap();
    let new_png = image::open(&output).unwrap();

    // Conversion should be lossless
    assert!(old_png.pixels().map(|x| x.2.channels().to_owned()).collect::<Vec<Vec<u8>>>() ==
            new_png.pixels().map(|x| x.2.channels().to_owned()).collect::<Vec<Vec<u8>>>());

    remove_file(output).ok();
}

#[test]
fn interlacing_1_to_0_small_files() {
    let input = PathBuf::from("tests/files/interlacing_1_to_0_small_files.png");
    let mut opts = get_opts(&input);
    opts.interlace = Some(0);
    let output = opts.out_file.clone();

    let png = png::PngData::new(&input).unwrap();

    assert!(png.ihdr_data.interlaced == 1);
    assert!(png.ihdr_data.color_type == png::ColorType::Indexed);
    assert!(png.ihdr_data.bit_depth == png::BitDepth::Eight);

    match oxipng::optimize(&input, &opts) {
        Ok(_) => (),
        Err(x) => panic!(x.to_owned()),
    };
    assert!(output.exists());

    let png = match png::PngData::new(&output) {
        Ok(x) => x,
        Err(x) => {
            remove_file(output).ok();
            panic!(x.to_owned())
        }
    };

    assert!(png.ihdr_data.interlaced == 0);
    assert!(png.ihdr_data.color_type == png::ColorType::Indexed);
    assert!(png.ihdr_data.bit_depth == png::BitDepth::One);

    let old_png = image::open(&input).unwrap();
    let new_png = image::open(&output).unwrap();

    // Conversion should be lossless
    assert!(old_png.pixels().map(|x| x.2.channels().to_owned()).collect::<Vec<Vec<u8>>>() ==
            new_png.pixels().map(|x| x.2.channels().to_owned()).collect::<Vec<Vec<u8>>>());

    remove_file(output).ok();
}
