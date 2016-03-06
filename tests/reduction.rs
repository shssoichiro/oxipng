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
fn rgba_16_should_be_rgba_16() {
    let input = PathBuf::from("tests/files/rgba_16_should_be_rgba_16.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::RGBA,
                     png::BitDepth::Sixteen,
                     png::ColorType::RGBA,
                     png::BitDepth::Sixteen);
}

#[test]
fn rgba_16_should_be_rgba_8() {
    let input = PathBuf::from("tests/files/rgba_16_should_be_rgba_8.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::RGBA,
                     png::BitDepth::Sixteen,
                     png::ColorType::RGBA,
                     png::BitDepth::Eight);
}

#[test]
fn rgba_8_should_be_rgba_8() {
    let input = PathBuf::from("tests/files/rgba_8_should_be_rgba_8.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::RGBA,
                     png::BitDepth::Eight,
                     png::ColorType::RGBA,
                     png::BitDepth::Eight);
}

#[test]
fn rgba_16_should_be_rgb_16() {
    let input = PathBuf::from("tests/files/rgba_16_should_be_rgb_16.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::RGBA,
                     png::BitDepth::Sixteen,
                     png::ColorType::RGB,
                     png::BitDepth::Sixteen);
}

#[test]
fn rgba_16_should_be_rgb_8() {
    let input = PathBuf::from("tests/files/rgba_16_should_be_rgb_8.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::RGBA,
                     png::BitDepth::Sixteen,
                     png::ColorType::RGB,
                     png::BitDepth::Eight);
}

#[test]
fn rgba_8_should_be_rgb_8() {
    let input = PathBuf::from("tests/files/rgba_8_should_be_rgb_8.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::RGBA,
                     png::BitDepth::Eight,
                     png::ColorType::RGB,
                     png::BitDepth::Eight);
}

#[test]
fn rgba_16_should_be_palette_8() {
    let input = PathBuf::from("tests/files/rgba_16_should_be_palette_8.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::RGBA,
                     png::BitDepth::Sixteen,
                     png::ColorType::Indexed,
                     png::BitDepth::Eight);
}

#[test]
fn rgba_8_should_be_palette_8() {
    let input = PathBuf::from("tests/files/rgba_8_should_be_palette_8.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::RGBA,
                     png::BitDepth::Eight,
                     png::ColorType::Indexed,
                     png::BitDepth::Eight);
}

#[test]
fn rgba_16_should_be_palette_4() {
    let input = PathBuf::from("tests/files/rgba_16_should_be_palette_4.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::RGBA,
                     png::BitDepth::Sixteen,
                     png::ColorType::Indexed,
                     png::BitDepth::Four);
}

#[test]
fn rgba_8_should_be_palette_4() {
    let input = PathBuf::from("tests/files/rgba_8_should_be_palette_4.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::RGBA,
                     png::BitDepth::Eight,
                     png::ColorType::Indexed,
                     png::BitDepth::Four);
}

#[test]
fn rgba_16_should_be_palette_2() {
    let input = PathBuf::from("tests/files/rgba_16_should_be_palette_2.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::RGBA,
                     png::BitDepth::Sixteen,
                     png::ColorType::Indexed,
                     png::BitDepth::Two);
}

#[test]
fn rgba_8_should_be_palette_2() {
    let input = PathBuf::from("tests/files/rgba_8_should_be_palette_2.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::RGBA,
                     png::BitDepth::Eight,
                     png::ColorType::Indexed,
                     png::BitDepth::Two);
}

#[test]
fn rgba_16_should_be_palette_1() {
    let input = PathBuf::from("tests/files/rgba_16_should_be_palette_1.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::RGBA,
                     png::BitDepth::Sixteen,
                     png::ColorType::Indexed,
                     png::BitDepth::One);
}

#[test]
fn rgba_8_should_be_palette_1() {
    let input = PathBuf::from("tests/files/rgba_8_should_be_palette_1.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::RGBA,
                     png::BitDepth::Eight,
                     png::ColorType::Indexed,
                     png::BitDepth::One);
}

#[test]
fn rgba_16_should_be_grayscale_alpha_16() {
    let input = PathBuf::from("tests/files/rgba_16_should_be_grayscale_alpha_16.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::RGBA,
                     png::BitDepth::Sixteen,
                     png::ColorType::GrayscaleAlpha,
                     png::BitDepth::Sixteen);
}

#[test]
fn rgba_16_should_be_grayscale_alpha_8() {
    let input = PathBuf::from("tests/files/rgba_16_should_be_grayscale_alpha_8.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::RGBA,
                     png::BitDepth::Sixteen,
                     png::ColorType::GrayscaleAlpha,
                     png::BitDepth::Eight);
}

#[test]
fn rgba_8_should_be_grayscale_alpha_8() {
    let input = PathBuf::from("tests/files/rgba_8_should_be_grayscale_alpha_8.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::RGBA,
                     png::BitDepth::Eight,
                     png::ColorType::GrayscaleAlpha,
                     png::BitDepth::Eight);
}

#[test]
fn rgba_16_should_be_grayscale_16() {
    let input = PathBuf::from("tests/files/rgba_16_should_be_grayscale_16.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::RGBA,
                     png::BitDepth::Sixteen,
                     png::ColorType::Grayscale,
                     png::BitDepth::Sixteen);
}

#[test]
fn rgba_16_should_be_grayscale_8() {
    let input = PathBuf::from("tests/files/rgba_16_should_be_grayscale_8.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::RGBA,
                     png::BitDepth::Sixteen,
                     png::ColorType::Grayscale,
                     png::BitDepth::Eight);
}

#[test]
fn rgba_8_should_be_grayscale_8() {
    let input = PathBuf::from("tests/files/rgba_8_should_be_grayscale_8.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::RGBA,
                     png::BitDepth::Eight,
                     png::ColorType::Grayscale,
                     png::BitDepth::Eight);
}

#[test]
fn rgba_16_should_be_palette_4_grayscale() {
    let input = PathBuf::from("tests/files/rgba_16_should_be_palette_4_grayscale.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::RGBA,
                     png::BitDepth::Sixteen,
                     png::ColorType::Indexed,
                     png::BitDepth::Four);
}

#[test]
fn rgba_8_should_be_palette_4_grayscale() {
    let input = PathBuf::from("tests/files/rgba_8_should_be_palette_4_grayscale.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::RGBA,
                     png::BitDepth::Eight,
                     png::ColorType::Indexed,
                     png::BitDepth::Four);
}

#[test]
fn rgba_16_should_be_palette_2_grayscale() {
    let input = PathBuf::from("tests/files/rgba_16_should_be_palette_2_grayscale.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::RGBA,
                     png::BitDepth::Sixteen,
                     png::ColorType::Indexed,
                     png::BitDepth::Two);
}

#[test]
fn rgba_8_should_be_palette_2_grayscale() {
    let input = PathBuf::from("tests/files/rgba_8_should_be_palette_2_grayscale.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::RGBA,
                     png::BitDepth::Eight,
                     png::ColorType::Indexed,
                     png::BitDepth::Two);
}

#[test]
fn rgba_16_should_be_palette_1_grayscale() {
    let input = PathBuf::from("tests/files/rgba_16_should_be_palette_1_grayscale.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::RGBA,
                     png::BitDepth::Sixteen,
                     png::ColorType::Indexed,
                     png::BitDepth::One);
}

#[test]
fn rgba_8_should_be_palette_1_grayscale() {
    let input = PathBuf::from("tests/files/rgba_8_should_be_palette_1_grayscale.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::RGBA,
                     png::BitDepth::Eight,
                     png::ColorType::Indexed,
                     png::BitDepth::One);
}

#[test]
fn rgb_16_should_be_rgb_16() {
    let input = PathBuf::from("tests/files/rgb_16_should_be_rgb_16.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::RGB,
                     png::BitDepth::Sixteen,
                     png::ColorType::RGB,
                     png::BitDepth::Sixteen);
}

#[test]
fn rgb_16_should_be_rgb_8() {
    let input = PathBuf::from("tests/files/rgb_16_should_be_rgb_8.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::RGB,
                     png::BitDepth::Sixteen,
                     png::ColorType::RGB,
                     png::BitDepth::Eight);
}

#[test]
fn rgb_8_should_be_rgb_8() {
    let input = PathBuf::from("tests/files/rgb_8_should_be_rgb_8.png");
    let opts = get_opts(&input);
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
fn rgb_16_should_be_palette_8() {
    let input = PathBuf::from("tests/files/rgb_16_should_be_palette_8.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::RGB,
                     png::BitDepth::Sixteen,
                     png::ColorType::Indexed,
                     png::BitDepth::Eight);
}

#[test]
fn rgb_8_should_be_palette_8() {
    let input = PathBuf::from("tests/files/rgb_8_should_be_palette_8.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::RGB,
                     png::BitDepth::Eight,
                     png::ColorType::Indexed,
                     png::BitDepth::Eight);
}

#[test]
fn rgb_16_should_be_palette_4() {
    let input = PathBuf::from("tests/files/rgb_16_should_be_palette_4.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::RGB,
                     png::BitDepth::Sixteen,
                     png::ColorType::Indexed,
                     png::BitDepth::Four);
}

#[test]
fn rgb_8_should_be_palette_4() {
    let input = PathBuf::from("tests/files/rgb_8_should_be_palette_4.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::RGB,
                     png::BitDepth::Eight,
                     png::ColorType::Indexed,
                     png::BitDepth::Four);
}

#[test]
fn rgb_16_should_be_palette_2() {
    let input = PathBuf::from("tests/files/rgb_16_should_be_palette_2.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::RGB,
                     png::BitDepth::Sixteen,
                     png::ColorType::Indexed,
                     png::BitDepth::Two);
}

#[test]
fn rgb_8_should_be_palette_2() {
    let input = PathBuf::from("tests/files/rgb_8_should_be_palette_2.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::RGB,
                     png::BitDepth::Eight,
                     png::ColorType::Indexed,
                     png::BitDepth::Two);
}

#[test]
fn rgb_16_should_be_palette_1() {
    let input = PathBuf::from("tests/files/rgb_16_should_be_palette_1.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::RGB,
                     png::BitDepth::Sixteen,
                     png::ColorType::Indexed,
                     png::BitDepth::One);
}

#[test]
fn rgb_8_should_be_palette_1() {
    let input = PathBuf::from("tests/files/rgb_8_should_be_palette_1.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::RGB,
                     png::BitDepth::Eight,
                     png::ColorType::Indexed,
                     png::BitDepth::One);
}

#[test]
fn rgb_16_should_be_grayscale_16() {
    let input = PathBuf::from("tests/files/rgb_16_should_be_grayscale_16.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::RGB,
                     png::BitDepth::Sixteen,
                     png::ColorType::Grayscale,
                     png::BitDepth::Sixteen);
}

#[test]
fn rgb_16_should_be_grayscale_8() {
    let input = PathBuf::from("tests/files/rgb_16_should_be_grayscale_8.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::RGB,
                     png::BitDepth::Sixteen,
                     png::ColorType::Grayscale,
                     png::BitDepth::Eight);
}

#[test]
fn rgb_8_should_be_grayscale_8() {
    let input = PathBuf::from("tests/files/rgb_8_should_be_grayscale_8.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::RGB,
                     png::BitDepth::Eight,
                     png::ColorType::Grayscale,
                     png::BitDepth::Eight);
}

#[test]
fn rgb_16_should_be_palette_4_grayscale() {
    let input = PathBuf::from("tests/files/rgb_16_should_be_palette_4_grayscale.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::RGB,
                     png::BitDepth::Sixteen,
                     png::ColorType::Indexed,
                     png::BitDepth::Four);
}

#[test]
fn rgb_8_should_be_palette_4_grayscale() {
    let input = PathBuf::from("tests/files/rgb_8_should_be_palette_4_grayscale.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::RGB,
                     png::BitDepth::Eight,
                     png::ColorType::Indexed,
                     png::BitDepth::Four);
}

#[test]
fn rgb_16_should_be_palette_2_grayscale() {
    let input = PathBuf::from("tests/files/rgb_16_should_be_palette_2_grayscale.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::RGB,
                     png::BitDepth::Sixteen,
                     png::ColorType::Indexed,
                     png::BitDepth::Two);
}

#[test]
fn rgb_8_should_be_palette_2_grayscale() {
    let input = PathBuf::from("tests/files/rgb_8_should_be_palette_2_grayscale.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::RGB,
                     png::BitDepth::Eight,
                     png::ColorType::Indexed,
                     png::BitDepth::Two);
}

#[test]
fn rgb_16_should_be_palette_1_grayscale() {
    let input = PathBuf::from("tests/files/rgb_16_should_be_palette_1_grayscale.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::RGB,
                     png::BitDepth::Sixteen,
                     png::ColorType::Indexed,
                     png::BitDepth::One);
}

#[test]
fn rgb_8_should_be_palette_1_grayscale() {
    let input = PathBuf::from("tests/files/rgb_8_should_be_palette_1_grayscale.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::RGB,
                     png::BitDepth::Eight,
                     png::ColorType::Indexed,
                     png::BitDepth::One);
}

#[test]
fn palette_8_should_be_grayscale_8() {
    let input = PathBuf::from("tests/files/palette_8_should_be_grayscale_8.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::Indexed,
                     png::BitDepth::Eight,
                     png::ColorType::Grayscale,
                     png::BitDepth::Eight);
}

#[test]
fn palette_8_should_be_palette_8() {
    let input = PathBuf::from("tests/files/palette_8_should_be_palette_8.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::Indexed,
                     png::BitDepth::Eight,
                     png::ColorType::Indexed,
                     png::BitDepth::Eight);
}

#[test]
fn palette_8_should_be_palette_4() {
    let input = PathBuf::from("tests/files/palette_8_should_be_palette_4.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::Indexed,
                     png::BitDepth::Eight,
                     png::ColorType::Indexed,
                     png::BitDepth::Four);
}

#[test]
fn palette_4_should_be_palette_4() {
    let input = PathBuf::from("tests/files/palette_4_should_be_palette_4.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::Indexed,
                     png::BitDepth::Four,
                     png::ColorType::Indexed,
                     png::BitDepth::Four);
}

#[test]
fn palette_8_should_be_palette_2() {
    let input = PathBuf::from("tests/files/palette_8_should_be_palette_2.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::Indexed,
                     png::BitDepth::Eight,
                     png::ColorType::Indexed,
                     png::BitDepth::Two);
}

#[test]
fn palette_4_should_be_palette_2() {
    let input = PathBuf::from("tests/files/palette_4_should_be_palette_2.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::Indexed,
                     png::BitDepth::Four,
                     png::ColorType::Indexed,
                     png::BitDepth::Two);
}

#[test]
fn palette_2_should_be_palette_2() {
    let input = PathBuf::from("tests/files/palette_2_should_be_palette_2.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::Indexed,
                     png::BitDepth::Two,
                     png::ColorType::Indexed,
                     png::BitDepth::Two);
}

#[test]
fn palette_8_should_be_palette_1() {
    let input = PathBuf::from("tests/files/palette_8_should_be_palette_1.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::Indexed,
                     png::BitDepth::Eight,
                     png::ColorType::Indexed,
                     png::BitDepth::One);
}

#[test]
fn palette_4_should_be_palette_1() {
    let input = PathBuf::from("tests/files/palette_4_should_be_palette_1.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::Indexed,
                     png::BitDepth::Four,
                     png::ColorType::Indexed,
                     png::BitDepth::One);
}

#[test]
fn palette_2_should_be_palette_1() {
    let input = PathBuf::from("tests/files/palette_2_should_be_palette_1.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::Indexed,
                     png::BitDepth::Two,
                     png::ColorType::Indexed,
                     png::BitDepth::One);
}

#[test]
fn palette_1_should_be_palette_1() {
    let input = PathBuf::from("tests/files/palette_1_should_be_palette_1.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::Indexed,
                     png::BitDepth::One,
                     png::ColorType::Indexed,
                     png::BitDepth::One);
}

#[test]
fn grayscale_alpha_16_should_be_grayscale_alpha_16() {
    let input = PathBuf::from("tests/files/grayscale_alpha_16_should_be_grayscale_alpha_16.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::GrayscaleAlpha,
                     png::BitDepth::Sixteen,
                     png::ColorType::GrayscaleAlpha,
                     png::BitDepth::Sixteen);
}

#[test]
fn grayscale_alpha_16_should_be_grayscale_alpha_8() {
    let input = PathBuf::from("tests/files/grayscale_alpha_16_should_be_grayscale_alpha_8.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::GrayscaleAlpha,
                     png::BitDepth::Sixteen,
                     png::ColorType::GrayscaleAlpha,
                     png::BitDepth::Eight);
}

#[test]
fn grayscale_alpha_8_should_be_grayscale_alpha_8() {
    let input = PathBuf::from("tests/files/grayscale_alpha_8_should_be_grayscale_alpha_8.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::GrayscaleAlpha,
                     png::BitDepth::Eight,
                     png::ColorType::GrayscaleAlpha,
                     png::BitDepth::Eight);
}

#[test]
fn grayscale_alpha_16_should_be_grayscale_16() {
    let input = PathBuf::from("tests/files/grayscale_alpha_16_should_be_grayscale_16.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::GrayscaleAlpha,
                     png::BitDepth::Sixteen,
                     png::ColorType::Grayscale,
                     png::BitDepth::Sixteen);
}

#[test]
fn grayscale_alpha_16_should_be_grayscale_8() {
    let input = PathBuf::from("tests/files/grayscale_alpha_16_should_be_grayscale_8.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::GrayscaleAlpha,
                     png::BitDepth::Sixteen,
                     png::ColorType::Grayscale,
                     png::BitDepth::Eight);
}

#[test]
fn grayscale_alpha_8_should_be_grayscale_8() {
    let input = PathBuf::from("tests/files/grayscale_alpha_8_should_be_grayscale_8.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::GrayscaleAlpha,
                     png::BitDepth::Eight,
                     png::ColorType::Grayscale,
                     png::BitDepth::Eight);
}

#[test]
fn grayscale_alpha_16_should_be_palette_4() {
    let input = PathBuf::from("tests/files/grayscale_alpha_16_should_be_palette_4.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::GrayscaleAlpha,
                     png::BitDepth::Sixteen,
                     png::ColorType::Indexed,
                     png::BitDepth::Four);
}

#[test]
fn grayscale_alpha_8_should_be_palette_4() {
    let input = PathBuf::from("tests/files/grayscale_alpha_8_should_be_palette_4.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::GrayscaleAlpha,
                     png::BitDepth::Eight,
                     png::ColorType::Indexed,
                     png::BitDepth::Four);
}

#[test]
fn grayscale_alpha_16_should_be_palette_2() {
    let input = PathBuf::from("tests/files/grayscale_alpha_16_should_be_palette_2.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::GrayscaleAlpha,
                     png::BitDepth::Sixteen,
                     png::ColorType::Indexed,
                     png::BitDepth::Two);
}

#[test]
fn grayscale_alpha_8_should_be_palette_2() {
    let input = PathBuf::from("tests/files/grayscale_alpha_8_should_be_palette_2.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::GrayscaleAlpha,
                     png::BitDepth::Eight,
                     png::ColorType::Indexed,
                     png::BitDepth::Two);
}

#[test]
fn grayscale_alpha_16_should_be_palette_1() {
    let input = PathBuf::from("tests/files/grayscale_alpha_16_should_be_palette_1.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::GrayscaleAlpha,
                     png::BitDepth::Sixteen,
                     png::ColorType::Indexed,
                     png::BitDepth::One);
}

#[test]
fn grayscale_alpha_8_should_be_palette_1() {
    let input = PathBuf::from("tests/files/grayscale_alpha_8_should_be_palette_1.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::GrayscaleAlpha,
                     png::BitDepth::Eight,
                     png::ColorType::Indexed,
                     png::BitDepth::One);
}

#[test]
fn grayscale_16_should_be_grayscale_16() {
    let input = PathBuf::from("tests/files/grayscale_16_should_be_grayscale_16.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::Grayscale,
                     png::BitDepth::Sixteen,
                     png::ColorType::Grayscale,
                     png::BitDepth::Sixteen);
}

#[test]
fn grayscale_16_should_be_grayscale_8() {
    let input = PathBuf::from("tests/files/grayscale_16_should_be_grayscale_8.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::Grayscale,
                     png::BitDepth::Sixteen,
                     png::ColorType::Grayscale,
                     png::BitDepth::Eight);
}

#[test]
fn grayscale_16_should_be_palette_4() {
    let input = PathBuf::from("tests/files/grayscale_16_should_be_palette_4.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::Grayscale,
                     png::BitDepth::Sixteen,
                     png::ColorType::Indexed,
                     png::BitDepth::Four);
}

#[test]
fn grayscale_16_should_be_palette_2() {
    let input = PathBuf::from("tests/files/grayscale_16_should_be_palette_2.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::Grayscale,
                     png::BitDepth::Sixteen,
                     png::ColorType::Indexed,
                     png::BitDepth::Two);
}

#[test]
fn grayscale_16_should_be_palette_1() {
    let input = PathBuf::from("tests/files/grayscale_16_should_be_palette_1.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::Grayscale,
                     png::BitDepth::Sixteen,
                     png::ColorType::Indexed,
                     png::BitDepth::One);
}

#[test]
fn grayscale_8_should_be_grayscale_8() {
    let input = PathBuf::from("tests/files/grayscale_8_should_be_grayscale_8.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::Grayscale,
                     png::BitDepth::Eight,
                     png::ColorType::Grayscale,
                     png::BitDepth::Eight);
}

#[test]
fn grayscale_8_should_be_palette_4() {
    let input = PathBuf::from("tests/files/grayscale_8_should_be_palette_4.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::Grayscale,
                     png::BitDepth::Eight,
                     png::ColorType::Indexed,
                     png::BitDepth::Four);
}

#[test]
fn grayscale_8_should_be_palette_2() {
    let input = PathBuf::from("tests/files/grayscale_8_should_be_palette_2.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::Grayscale,
                     png::BitDepth::Eight,
                     png::ColorType::Indexed,
                     png::BitDepth::Two);
}

#[test]
fn grayscale_8_should_be_palette_1() {
    let input = PathBuf::from("tests/files/grayscale_8_should_be_palette_1.png");
    let opts = get_opts(&input);
    let output = opts.out_file.clone();

    test_it_converts(&input,
                     &output,
                     &opts,
                     png::ColorType::Grayscale,
                     png::BitDepth::Eight,
                     png::ColorType::Indexed,
                     png::BitDepth::One);
}
