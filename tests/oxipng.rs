extern crate oxipng;

use oxipng::png;
use std::collections::HashSet;
use std::fs::remove_file;
use std::path::Path;
use std::path::PathBuf;

fn get_opts(input: &Path) -> oxipng::Options {
    let mut filter = HashSet::new();
    filter.insert(0);
    filter.insert(5);
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
        verbosity: Some(0),
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
        strip: false,
    }
}

#[test]
fn reduce_rgba_png() {
    let input = PathBuf::from("tests/files/test_rgba.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    match oxipng::optimize(&input, &opts) {
        Ok(_) => (),
        Err(x) => panic!(x.to_owned())
    };
    assert!(output.exists());

    let png = match png::PngData::new(&output) {
        Ok(x) => {
            remove_file(output).ok();
            x
        },
        Err(x) => {
            remove_file(output).ok();
            panic!(x.to_owned())
        },
    };

    assert!(png.ihdr_data.color_type == png::ColorType::Indexed);
    assert!(png.ihdr_data.bit_depth == png::BitDepth::Eight);
    assert!(png.palette.unwrap().len() == 43 * 3);
}

#[test]
fn reduce_rgb_png() {
    let input = PathBuf::from("tests/files/test_rgb.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    match oxipng::optimize(&input, &opts) {
        Ok(_) => (),
        Err(x) => panic!(x.to_owned())
    };
    assert!(output.exists());

    let png = match png::PngData::new(&output) {
        Ok(x) => {
            remove_file(output).ok();
            x
        },
        Err(x) => {
            remove_file(output).ok();
            panic!(x.to_owned())
        },
    };

    assert!(png.ihdr_data.color_type == png::ColorType::Indexed);
    assert!(png.ihdr_data.bit_depth == png::BitDepth::Eight);
    assert!(png.palette.unwrap().len() == 43 * 3);
}

#[test]
fn reduce_palette_png() {
    let input = PathBuf::from("tests/files/test_palette.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    match oxipng::optimize(&input, &opts) {
        Ok(_) => (),
        Err(x) => panic!(x.to_owned())
    };
    assert!(output.exists());

    let png = match png::PngData::new(&output) {
        Ok(x) => {
            remove_file(output).ok();
            x
        },
        Err(x) => {
            remove_file(output).ok();
            panic!(x.to_owned())
        },
    };

    assert!(png.ihdr_data.color_type == png::ColorType::Indexed);
    assert!(png.ihdr_data.bit_depth == png::BitDepth::Eight);
    assert!(png.palette.unwrap().len() == 43 * 3);
}
