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
fn rgba_should_be_rgba() {
    let input = PathBuf::from("tests/files/rgba_should_be_rgba.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    match oxipng::optimize(&input, &opts) {
        Ok(_) => (),
        Err(x) => panic!(x.to_owned())
    };
    assert!(output.exists());

    let png = match png::PngData::new(&output) {
        Ok(x) => x,
        Err(x) => {
            remove_file(output).ok();
            panic!(x.to_owned())
        },
    };

    assert!(png.ihdr_data.color_type == png::ColorType::RGBA);
    assert!(png.ihdr_data.bit_depth == png::BitDepth::Eight);

    let old_png = image::open(&input).unwrap();
    let new_png = image::open(&output).unwrap();

    // Conversion should be lossless
    assert!(old_png.pixels().map(|x| x.2.channels().to_owned()).collect::<Vec<Vec<u8>>>() == new_png.pixels().map(|x| x.2.channels().to_owned()).collect::<Vec<Vec<u8>>>());

    remove_file(output).ok();
}

#[test]
fn rgba_should_be_rgb() {
    let input = PathBuf::from("tests/files/rgba_should_be_rgb.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    match oxipng::optimize(&input, &opts) {
        Ok(_) => (),
        Err(x) => panic!(x.to_owned())
    };
    assert!(output.exists());

    let png = match png::PngData::new(&output) {
        Ok(x) => x,
        Err(x) => {
            remove_file(output).ok();
            panic!(x.to_owned())
        },
    };

    assert!(png.ihdr_data.color_type == png::ColorType::RGB);
    assert!(png.ihdr_data.bit_depth == png::BitDepth::Eight);

    let old_png = image::open(&input).unwrap();
    let new_png = image::open(&output).unwrap();

    // Conversion should be lossless
    assert!(old_png.pixels().map(|x| x.2.channels().to_owned()).collect::<Vec<Vec<u8>>>() == new_png.pixels().map(|x| x.2.channels().to_owned()).collect::<Vec<Vec<u8>>>());

    remove_file(output).ok();
}

#[test]
fn rgba_should_be_palette() {
    let input = PathBuf::from("tests/files/rgba_should_be_palette.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    match oxipng::optimize(&input, &opts) {
        Ok(_) => (),
        Err(x) => panic!(x.to_owned())
    };
    assert!(output.exists());

    let png = match png::PngData::new(&output) {
        Ok(x) => x,
        Err(x) => {
            remove_file(output).ok();
            panic!(x.to_owned())
        },
    };

    assert!(png.ihdr_data.color_type == png::ColorType::Indexed);
    assert!(png.ihdr_data.bit_depth == png::BitDepth::Eight);

    let old_png = image::open(&input).unwrap();
    let new_png = image::open(&output).unwrap();

    // Conversion should be lossless
    assert!(old_png.pixels().map(|x| x.2.channels().to_owned()).collect::<Vec<Vec<u8>>>() == new_png.pixels().map(|x| x.2.channels().to_owned()).collect::<Vec<Vec<u8>>>());

    remove_file(output).ok();
}

#[test]
fn rgba_should_be_grayscale_alpha() {
    let input = PathBuf::from("tests/files/rgba_should_be_grayscale_alpha.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    match oxipng::optimize(&input, &opts) {
        Ok(_) => (),
        Err(x) => panic!(x.to_owned())
    };
    assert!(output.exists());

    let png = match png::PngData::new(&output) {
        Ok(x) => x,
        Err(x) => {
            remove_file(output).ok();
            panic!(x.to_owned())
        },
    };

    assert!(png.ihdr_data.color_type == png::ColorType::GrayscaleAlpha);
    assert!(png.ihdr_data.bit_depth == png::BitDepth::Eight);

    let old_png = image::open(&input).unwrap();
    let new_png = image::open(&output).unwrap();

    // Conversion should be lossless
    assert!(old_png.pixels().map(|x| x.2.channels().to_owned()).collect::<Vec<Vec<u8>>>() == new_png.pixels().map(|x| x.2.channels().to_owned()).collect::<Vec<Vec<u8>>>());

    remove_file(output).ok();
}

#[test]
fn rgba_should_be_grayscale() {
    let input = PathBuf::from("tests/files/rgba_should_be_grayscale.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    match oxipng::optimize(&input, &opts) {
        Ok(_) => (),
        Err(x) => panic!(x.to_owned())
    };
    assert!(output.exists());

    let png = match png::PngData::new(&output) {
        Ok(x) => x,
        Err(x) => {
            remove_file(output).ok();
            panic!(x.to_owned())
        },
    };

    assert!(png.ihdr_data.color_type == png::ColorType::Grayscale);
    assert!(png.ihdr_data.bit_depth == png::BitDepth::Eight);

    let old_png = image::open(&input).unwrap();
    let new_png = image::open(&output).unwrap();

    // Conversion should be lossless
    assert!(old_png.pixels().map(|x| x.2.channels().to_owned()).collect::<Vec<Vec<u8>>>() == new_png.pixels().map(|x| x.2.channels().to_owned()).collect::<Vec<Vec<u8>>>());

    remove_file(output).ok();
}

#[test]
fn rgb_should_be_rgb() {
    let input = PathBuf::from("tests/files/rgb_should_be_rgb.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    match oxipng::optimize(&input, &opts) {
        Ok(_) => (),
        Err(x) => panic!(x.to_owned())
    };
    assert!(output.exists());

    let png = match png::PngData::new(&output) {
        Ok(x) => x,
        Err(x) => {
            remove_file(output).ok();
            panic!(x.to_owned())
        },
    };

    assert!(png.ihdr_data.color_type == png::ColorType::RGB);
    assert!(png.ihdr_data.bit_depth == png::BitDepth::Eight);

    let old_png = image::open(&input).unwrap();
    let new_png = image::open(&output).unwrap();

    // Conversion should be lossless
    assert!(old_png.pixels().map(|x| x.2.channels().to_owned()).collect::<Vec<Vec<u8>>>() == new_png.pixels().map(|x| x.2.channels().to_owned()).collect::<Vec<Vec<u8>>>());

    remove_file(output).ok();
}

#[test]
fn rgb_should_be_palette() {
    let input = PathBuf::from("tests/files/rgb_should_be_palette.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    match oxipng::optimize(&input, &opts) {
        Ok(_) => (),
        Err(x) => panic!(x.to_owned())
    };
    assert!(output.exists());

    let png = match png::PngData::new(&output) {
        Ok(x) => x,
        Err(x) => {
            remove_file(output).ok();
            panic!(x.to_owned())
        },
    };

    assert!(png.ihdr_data.color_type == png::ColorType::Indexed);
    assert!(png.ihdr_data.bit_depth == png::BitDepth::Eight);

    let old_png = image::open(&input).unwrap();
    let new_png = image::open(&output).unwrap();

    // Conversion should be lossless
    assert!(old_png.pixels().map(|x| x.2.channels().to_owned()).collect::<Vec<Vec<u8>>>() == new_png.pixels().map(|x| x.2.channels().to_owned()).collect::<Vec<Vec<u8>>>());

    remove_file(output).ok();
}

#[test]
fn rgb_should_be_grayscale() {
    let input = PathBuf::from("tests/files/rgb_should_be_grayscale.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    match oxipng::optimize(&input, &opts) {
        Ok(_) => (),
        Err(x) => panic!(x.to_owned())
    };
    assert!(output.exists());

    let png = match png::PngData::new(&output) {
        Ok(x) => x,
        Err(x) => {
            remove_file(output).ok();
            panic!(x.to_owned())
        },
    };

    assert!(png.ihdr_data.color_type == png::ColorType::Grayscale);
    assert!(png.ihdr_data.bit_depth == png::BitDepth::Eight);

    let old_png = image::open(&input).unwrap();
    let new_png = image::open(&output).unwrap();

    // Conversion should be lossless
    assert!(old_png.pixels().map(|x| x.2.channels().to_owned()).collect::<Vec<Vec<u8>>>() == new_png.pixels().map(|x| x.2.channels().to_owned()).collect::<Vec<Vec<u8>>>());

    remove_file(output).ok();
}

#[test]
fn palette_should_be_palette() {
    let input = PathBuf::from("tests/files/palette_should_be_palette.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    match oxipng::optimize(&input, &opts) {
        Ok(_) => (),
        Err(x) => panic!(x.to_owned())
    };
    assert!(output.exists());

    let png = match png::PngData::new(&output) {
        Ok(x) => x,
        Err(x) => {
            remove_file(output).ok();
            panic!(x.to_owned())
        },
    };

    assert!(png.ihdr_data.color_type == png::ColorType::Indexed);
    assert!(png.ihdr_data.bit_depth == png::BitDepth::Eight);

    let old_png = image::open(&input).unwrap();
    let new_png = image::open(&output).unwrap();

    // Conversion should be lossless
    assert!(old_png.pixels().map(|x| x.2.channels().to_owned()).collect::<Vec<Vec<u8>>>() == new_png.pixels().map(|x| x.2.channels().to_owned()).collect::<Vec<Vec<u8>>>());

    remove_file(output).ok();
}

#[test]
fn palette_should_be_grayscale() {
    let input = PathBuf::from("tests/files/palette_should_be_grayscale.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    match oxipng::optimize(&input, &opts) {
        Ok(_) => (),
        Err(x) => panic!(x.to_owned())
    };
    assert!(output.exists());

    let png = match png::PngData::new(&output) {
        Ok(x) => x,
        Err(x) => {
            remove_file(output).ok();
            panic!(x.to_owned())
        },
    };

    assert!(png.ihdr_data.color_type == png::ColorType::Grayscale);
    assert!(png.ihdr_data.bit_depth == png::BitDepth::Eight);

    let old_png = image::open(&input).unwrap();
    let new_png = image::open(&output).unwrap();

    // Conversion should be lossless
    assert!(old_png.pixels().map(|x| x.2.channels().to_owned()).collect::<Vec<Vec<u8>>>() == new_png.pixels().map(|x| x.2.channels().to_owned()).collect::<Vec<Vec<u8>>>());

    remove_file(output).ok();
}

#[test]
fn strip_headers() {
    let input = PathBuf::from("tests/files/rgb_should_be_rgb.png");
    let mut opts = get_opts(&input);
    opts.strip = true;
    let output = opts.out_file.clone();

    match oxipng::optimize(&input, &opts) {
        Ok(_) => (),
        Err(x) => panic!(x.to_owned())
    };
    assert!(output.exists());

    let png = match png::PngData::new(&output) {
        Ok(x) => x,
        Err(x) => {
            remove_file(output).ok();
            panic!(x.to_owned())
        },
    };

    assert!(!png.aux_headers.contains_key("tEXt"));

    let old_png = image::open(&input).unwrap();
    let new_png = image::open(&output).unwrap();

    // Conversion should be lossless
    assert!(old_png.pixels().map(|x| x.2.channels().to_owned()).collect::<Vec<Vec<u8>>>() == new_png.pixels().map(|x| x.2.channels().to_owned()).collect::<Vec<Vec<u8>>>());

    remove_file(output).ok();
}

#[test]
#[ignore]
fn downgrade_16_to_8() {
    unimplemented!();
}

#[test]
#[ignore]
fn downgrade_8_to_4() {
    unimplemented!();
}

#[test]
#[ignore]
fn downgrade_8_to_2() {
    unimplemented!();
}

#[test]
#[ignore]
fn downgrade_8_to_1() {
    unimplemented!();
}

#[test]
#[ignore]
fn downgrade_4_to_2() {
    unimplemented!();
}

#[test]
#[ignore]
fn downgrade_4_to_1() {
    unimplemented!();
}

#[test]
#[ignore]
fn downgrade_2_to_1() {
    unimplemented!();
}
