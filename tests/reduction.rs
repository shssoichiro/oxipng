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
    alpha: Option<AlphaOptim>,
    color_type_in: ColorType,
    bit_depth_in: BitDepth,
    color_type_out: ColorType,
    bit_depth_out: BitDepth,
) {
    let input = PathBuf::from(input);
    let (output, mut opts) = get_opts(&input);
    if let Some(alpha) = alpha {
        opts.alphas = [alpha].iter().cloned().collect();
    }
    let png = PngData::new(&input, opts.fix_errors).unwrap();

    assert_eq!(png.raw.ihdr.color_type, color_type_in);
    assert_eq!(png.raw.ihdr.bit_depth, bit_depth_in);
    assert_eq!(png.raw.ihdr.interlaced, 0);

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
fn rgba_16_should_be_rgba_16() {
    test_it_converts(
        "tests/files/rgba_16_should_be_rgba_16.png",
        None,
        ColorType::RGBA,
        BitDepth::Sixteen,
        ColorType::RGBA,
        BitDepth::Sixteen,
    );
}

#[test]
fn rgba_16_should_be_rgba_8() {
    test_it_converts(
        "tests/files/rgba_16_should_be_rgba_8.png",
        None,
        ColorType::RGBA,
        BitDepth::Sixteen,
        ColorType::RGBA,
        BitDepth::Eight,
    );
}

#[test]
fn rgba_8_should_be_rgba_8() {
    test_it_converts(
        "tests/files/rgba_8_should_be_rgba_8.png",
        None,
        ColorType::RGBA,
        BitDepth::Eight,
        ColorType::RGBA,
        BitDepth::Eight,
    );
}

#[test]
fn rgba_16_should_be_rgb_16() {
    test_it_converts(
        "tests/files/rgba_16_should_be_rgb_16.png",
        None,
        ColorType::RGBA,
        BitDepth::Sixteen,
        ColorType::RGB,
        BitDepth::Sixteen,
    );
}

#[test]
fn rgba_16_should_be_rgb_8() {
    test_it_converts(
        "tests/files/rgba_16_should_be_rgb_8.png",
        None,
        ColorType::RGBA,
        BitDepth::Sixteen,
        ColorType::RGB,
        BitDepth::Eight,
    );
}

#[test]
fn rgba_8_should_be_rgb_8() {
    test_it_converts(
        "tests/files/rgba_8_should_be_rgb_8.png",
        None,
        ColorType::RGBA,
        BitDepth::Eight,
        ColorType::RGB,
        BitDepth::Eight,
    );
}

#[test]
fn rgba_16_should_be_palette_8() {
    test_it_converts(
        "tests/files/rgba_16_should_be_palette_8.png",
        None,
        ColorType::RGBA,
        BitDepth::Sixteen,
        ColorType::Indexed,
        BitDepth::Eight,
    );
}

#[test]
fn rgba_8_should_be_palette_8() {
    test_it_converts(
        "tests/files/rgba_8_should_be_palette_8.png",
        None,
        ColorType::RGBA,
        BitDepth::Eight,
        ColorType::Indexed,
        BitDepth::Eight,
    );
}

#[test]
fn rgba_16_should_be_palette_4() {
    test_it_converts(
        "tests/files/rgba_16_should_be_palette_4.png",
        None,
        ColorType::RGBA,
        BitDepth::Sixteen,
        ColorType::Indexed,
        BitDepth::Four,
    );
}

#[test]
fn rgba_8_should_be_palette_4() {
    test_it_converts(
        "tests/files/rgba_8_should_be_palette_4.png",
        None,
        ColorType::RGBA,
        BitDepth::Eight,
        ColorType::Indexed,
        BitDepth::Four,
    );
}

#[test]
fn rgba_16_should_be_palette_2() {
    test_it_converts(
        "tests/files/rgba_16_should_be_palette_2.png",
        None,
        ColorType::RGBA,
        BitDepth::Sixteen,
        ColorType::Indexed,
        BitDepth::Two,
    );
}

#[test]
fn rgba_8_should_be_palette_2() {
    test_it_converts(
        "tests/files/rgba_8_should_be_palette_2.png",
        None,
        ColorType::RGBA,
        BitDepth::Eight,
        ColorType::Indexed,
        BitDepth::Two,
    );
}

#[test]
fn rgba_16_should_be_palette_1() {
    test_it_converts(
        "tests/files/rgba_16_should_be_palette_1.png",
        None,
        ColorType::RGBA,
        BitDepth::Sixteen,
        ColorType::Indexed,
        BitDepth::One,
    );
}

#[test]
fn rgba_8_should_be_palette_1() {
    test_it_converts(
        "tests/files/rgba_8_should_be_palette_1.png",
        None,
        ColorType::RGBA,
        BitDepth::Eight,
        ColorType::Indexed,
        BitDepth::One,
    );
}

#[test]
fn rgba_16_should_be_grayscale_alpha_16() {
    test_it_converts(
        "tests/files/rgba_16_should_be_grayscale_alpha_16.png",
        None,
        ColorType::RGBA,
        BitDepth::Sixteen,
        ColorType::GrayscaleAlpha,
        BitDepth::Sixteen,
    );
}

#[test]
fn rgba_16_should_be_grayscale_alpha_8() {
    test_it_converts(
        "tests/files/rgba_16_should_be_grayscale_alpha_8.png",
        None,
        ColorType::RGBA,
        BitDepth::Sixteen,
        ColorType::GrayscaleAlpha,
        BitDepth::Eight,
    );
}

#[test]
fn rgba_8_should_be_grayscale_alpha_8() {
    test_it_converts(
        "tests/files/rgba_8_should_be_grayscale_alpha_8.png",
        None,
        ColorType::RGBA,
        BitDepth::Eight,
        ColorType::GrayscaleAlpha,
        BitDepth::Eight,
    );
}

#[test]
fn rgba_16_should_be_grayscale_16() {
    test_it_converts(
        "tests/files/rgba_16_should_be_grayscale_16.png",
        None,
        ColorType::RGBA,
        BitDepth::Sixteen,
        ColorType::Grayscale,
        BitDepth::Sixteen,
    );
}

#[test]
fn rgba_16_should_be_grayscale_8() {
    test_it_converts(
        "tests/files/rgba_16_should_be_grayscale_8.png",
        None,
        ColorType::RGBA,
        BitDepth::Sixteen,
        ColorType::Grayscale,
        BitDepth::Eight,
    );
}

#[test]
fn rgba_8_should_be_grayscale_8() {
    test_it_converts(
        "tests/files/rgba_8_should_be_grayscale_8.png",
        None,
        ColorType::RGBA,
        BitDepth::Eight,
        ColorType::Grayscale,
        BitDepth::Eight,
    );
}

#[test]
fn rgb_16_should_be_rgb_16() {
    test_it_converts(
        "tests/files/rgb_16_should_be_rgb_16.png",
        None,
        ColorType::RGB,
        BitDepth::Sixteen,
        ColorType::RGB,
        BitDepth::Sixteen,
    );
}

#[test]
fn rgb_16_should_be_rgb_8() {
    test_it_converts(
        "tests/files/rgb_16_should_be_rgb_8.png",
        None,
        ColorType::RGB,
        BitDepth::Sixteen,
        ColorType::RGB,
        BitDepth::Eight,
    );
}

#[test]
fn rgb_8_should_be_rgb_8() {
    test_it_converts(
        "tests/files/rgb_8_should_be_rgb_8.png",
        None,
        ColorType::RGB,
        BitDepth::Eight,
        ColorType::RGB,
        BitDepth::Eight,
    );
}

#[test]
fn rgb_16_should_be_palette_8() {
    test_it_converts(
        "tests/files/rgb_16_should_be_palette_8.png",
        None,
        ColorType::RGB,
        BitDepth::Sixteen,
        ColorType::Indexed,
        BitDepth::Eight,
    );
}

#[test]
fn rgb_8_should_be_palette_8() {
    test_it_converts(
        "tests/files/rgb_8_should_be_palette_8.png",
        None,
        ColorType::RGB,
        BitDepth::Eight,
        ColorType::Indexed,
        BitDepth::Eight,
    );
}

#[test]
fn rgb_16_should_be_palette_4() {
    test_it_converts(
        "tests/files/rgb_16_should_be_palette_4.png",
        None,
        ColorType::RGB,
        BitDepth::Sixteen,
        ColorType::Indexed,
        BitDepth::Four,
    );
}

#[test]
fn rgb_8_should_be_palette_4() {
    test_it_converts(
        "tests/files/rgb_8_should_be_palette_4.png",
        None,
        ColorType::RGB,
        BitDepth::Eight,
        ColorType::Indexed,
        BitDepth::Four,
    );
}

#[test]
fn rgb_16_should_be_palette_2() {
    test_it_converts(
        "tests/files/rgb_16_should_be_palette_2.png",
        None,
        ColorType::RGB,
        BitDepth::Sixteen,
        ColorType::Indexed,
        BitDepth::Two,
    );
}

#[test]
fn rgb_8_should_be_palette_2() {
    test_it_converts(
        "tests/files/rgb_8_should_be_palette_2.png",
        None,
        ColorType::RGB,
        BitDepth::Eight,
        ColorType::Indexed,
        BitDepth::Two,
    );
}

#[test]
fn rgb_16_should_be_palette_1() {
    test_it_converts(
        "tests/files/rgb_16_should_be_palette_1.png",
        None,
        ColorType::RGB,
        BitDepth::Sixteen,
        ColorType::Indexed,
        BitDepth::One,
    );
}

#[test]
fn rgb_8_should_be_palette_1() {
    test_it_converts(
        "tests/files/rgb_8_should_be_palette_1.png",
        None,
        ColorType::RGB,
        BitDepth::Eight,
        ColorType::Indexed,
        BitDepth::One,
    );
}

#[test]
fn rgb_16_should_be_grayscale_16() {
    test_it_converts(
        "tests/files/rgb_16_should_be_grayscale_16.png",
        None,
        ColorType::RGB,
        BitDepth::Sixteen,
        ColorType::Grayscale,
        BitDepth::Sixteen,
    );
}

#[test]
fn rgb_16_should_be_grayscale_8() {
    test_it_converts(
        "tests/files/rgb_16_should_be_grayscale_8.png",
        None,
        ColorType::RGB,
        BitDepth::Sixteen,
        ColorType::Grayscale,
        BitDepth::Eight,
    );
}

#[test]
fn rgb_8_should_be_grayscale_8() {
    test_it_converts(
        "tests/files/rgb_8_should_be_grayscale_8.png",
        None,
        ColorType::RGB,
        BitDepth::Eight,
        ColorType::Grayscale,
        BitDepth::Eight,
    );
}

#[test]
fn palette_8_should_be_palette_8() {
    test_it_converts(
        "tests/files/palette_8_should_be_palette_8.png",
        None,
        ColorType::Indexed,
        BitDepth::Eight,
        ColorType::Indexed,
        BitDepth::Eight,
    );
}

#[test]
fn palette_8_should_be_palette_4() {
    test_it_converts(
        "tests/files/palette_8_should_be_palette_4.png",
        None,
        ColorType::Indexed,
        BitDepth::Eight,
        ColorType::Indexed,
        BitDepth::Four,
    );
}

#[test]
fn palette_4_should_be_palette_4() {
    test_it_converts(
        "tests/files/palette_4_should_be_palette_4.png",
        None,
        ColorType::Indexed,
        BitDepth::Four,
        ColorType::Indexed,
        BitDepth::Four,
    );
}

#[test]
fn palette_8_should_be_palette_2() {
    test_it_converts(
        "tests/files/palette_8_should_be_palette_2.png",
        None,
        ColorType::Indexed,
        BitDepth::Eight,
        ColorType::Indexed,
        BitDepth::Two,
    );
}

#[test]
fn palette_4_should_be_palette_2() {
    test_it_converts(
        "tests/files/palette_4_should_be_palette_2.png",
        None,
        ColorType::Indexed,
        BitDepth::Four,
        ColorType::Indexed,
        BitDepth::Two,
    );
}

#[test]
fn palette_2_should_be_palette_2() {
    test_it_converts(
        "tests/files/palette_2_should_be_palette_2.png",
        None,
        ColorType::Indexed,
        BitDepth::Two,
        ColorType::Indexed,
        BitDepth::Two,
    );
}

#[test]
fn palette_8_should_be_palette_1() {
    test_it_converts(
        "tests/files/palette_8_should_be_palette_1.png",
        None,
        ColorType::Indexed,
        BitDepth::Eight,
        ColorType::Indexed,
        BitDepth::One,
    );
}

#[test]
fn palette_4_should_be_palette_1() {
    test_it_converts(
        "tests/files/palette_4_should_be_palette_1.png",
        None,
        ColorType::Indexed,
        BitDepth::Four,
        ColorType::Indexed,
        BitDepth::One,
    );
}

#[test]
fn palette_2_should_be_palette_1() {
    test_it_converts(
        "tests/files/palette_2_should_be_palette_1.png",
        None,
        ColorType::Indexed,
        BitDepth::Two,
        ColorType::Indexed,
        BitDepth::One,
    );
}

#[test]
fn palette_1_should_be_palette_1() {
    test_it_converts(
        "tests/files/palette_1_should_be_palette_1.png",
        None,
        ColorType::Indexed,
        BitDepth::One,
        ColorType::Indexed,
        BitDepth::One,
    );
}

#[test]
fn grayscale_alpha_16_should_be_grayscale_alpha_16() {
    test_it_converts(
        "tests/files/grayscale_alpha_16_should_be_grayscale_alpha_16.png",
        None,
        ColorType::GrayscaleAlpha,
        BitDepth::Sixteen,
        ColorType::GrayscaleAlpha,
        BitDepth::Sixteen,
    );
}

#[test]
fn grayscale_alpha_16_should_be_grayscale_alpha_8() {
    test_it_converts(
        "tests/files/grayscale_alpha_16_should_be_grayscale_alpha_8.png",
        None,
        ColorType::GrayscaleAlpha,
        BitDepth::Sixteen,
        ColorType::GrayscaleAlpha,
        BitDepth::Eight,
    );
}

#[test]
fn grayscale_alpha_8_should_be_grayscale_alpha_8() {
    test_it_converts(
        "tests/files/grayscale_alpha_8_should_be_grayscale_alpha_8.png",
        None,
        ColorType::GrayscaleAlpha,
        BitDepth::Eight,
        ColorType::GrayscaleAlpha,
        BitDepth::Eight,
    );
}

#[test]
fn grayscale_alpha_16_should_be_grayscale_16() {
    test_it_converts(
        "tests/files/grayscale_alpha_16_should_be_grayscale_16.png",
        None,
        ColorType::GrayscaleAlpha,
        BitDepth::Sixteen,
        ColorType::Grayscale,
        BitDepth::Sixteen,
    );
}

#[test]
fn grayscale_alpha_16_should_be_grayscale_8() {
    test_it_converts(
        "tests/files/grayscale_alpha_16_should_be_grayscale_8.png",
        None,
        ColorType::GrayscaleAlpha,
        BitDepth::Sixteen,
        ColorType::Grayscale,
        BitDepth::Eight,
    );
}

#[test]
fn grayscale_alpha_8_should_be_grayscale_8() {
    test_it_converts(
        "tests/files/grayscale_alpha_8_should_be_grayscale_8.png",
        None,
        ColorType::GrayscaleAlpha,
        BitDepth::Eight,
        ColorType::Grayscale,
        BitDepth::Eight,
    );
}

#[test]
fn grayscale_16_should_be_grayscale_16() {
    test_it_converts(
        "tests/files/grayscale_16_should_be_grayscale_16.png",
        None,
        ColorType::Grayscale,
        BitDepth::Sixteen,
        ColorType::Grayscale,
        BitDepth::Sixteen,
    );
}

#[test]
fn grayscale_16_should_be_grayscale_8() {
    test_it_converts(
        "tests/files/grayscale_16_should_be_grayscale_8.png",
        None,
        ColorType::Grayscale,
        BitDepth::Sixteen,
        ColorType::Grayscale,
        BitDepth::Eight,
    );
}

#[test]
fn grayscale_8_should_be_grayscale_8() {
    test_it_converts(
        "tests/files/grayscale_8_should_be_grayscale_8.png",
        None,
        ColorType::Grayscale,
        BitDepth::Eight,
        ColorType::Grayscale,
        BitDepth::Eight,
    );
}

#[test]
fn small_files() {
    test_it_converts(
        "tests/files/small_files.png",
        None,
        ColorType::Indexed,
        BitDepth::Eight,
        ColorType::Indexed,
        BitDepth::One,
    );
}

#[test]
fn palette_should_be_reduced_with_dupes() {
    let input = PathBuf::from("tests/files/palette_should_be_reduced_with_dupes.png");
    let (output, opts) = get_opts(&input);

    let png = PngData::new(&input, opts.fix_errors).unwrap();

    assert_eq!(png.raw.ihdr.color_type, ColorType::Indexed);
    assert_eq!(png.raw.ihdr.bit_depth, BitDepth::Eight);
    assert_eq!(png.raw.palette.as_ref().unwrap().len(), 43);

    match oxipng::optimize(&InFile::Path(input), &output, &opts) {
        Ok(_) => (),
        Err(x) => panic!("{}", x),
    };
    let output = output.path().unwrap();
    assert!(output.exists());

    let png = match PngData::new(&output, opts.fix_errors) {
        Ok(x) => x,
        Err(x) => {
            remove_file(&output).ok();
            panic!("{}", x)
        }
    };

    assert_eq!(png.raw.ihdr.color_type, ColorType::Indexed);
    assert_eq!(png.raw.ihdr.bit_depth, BitDepth::Eight);
    assert_eq!(png.raw.palette.as_ref().unwrap().len(), 35);

    remove_file(output).ok();
}

#[test]
fn palette_should_be_reduced_with_unused() {
    let input = PathBuf::from("tests/files/palette_should_be_reduced_with_unused.png");
    let (output, opts) = get_opts(&input);

    let png = PngData::new(&input, opts.fix_errors).unwrap();

    assert_eq!(png.raw.ihdr.color_type, ColorType::Indexed);
    assert_eq!(png.raw.ihdr.bit_depth, BitDepth::Eight);
    assert_eq!(png.raw.palette.as_ref().unwrap().len(), 35);

    match oxipng::optimize(&InFile::Path(input), &output, &opts) {
        Ok(_) => (),
        Err(x) => panic!("{}", x),
    };
    let output = output.path().unwrap();
    assert!(output.exists());

    let png = match PngData::new(&output, opts.fix_errors) {
        Ok(x) => x,
        Err(x) => {
            remove_file(&output).ok();
            panic!("{}", x)
        }
    };

    assert_eq!(png.raw.ihdr.color_type, ColorType::Indexed);
    assert_eq!(png.raw.ihdr.bit_depth, BitDepth::Eight);
    assert_eq!(png.raw.palette.as_ref().unwrap().len(), 33);

    remove_file(output).ok();
}

#[test]
fn palette_should_be_reduced_with_both() {
    let input = PathBuf::from("tests/files/palette_should_be_reduced_with_both.png");
    let (output, opts) = get_opts(&input);

    let png = PngData::new(&input, opts.fix_errors).unwrap();

    assert_eq!(png.raw.ihdr.color_type, ColorType::Indexed);
    assert_eq!(png.raw.ihdr.bit_depth, BitDepth::Eight);
    assert_eq!(png.raw.palette.as_ref().unwrap().len(), 43);

    match oxipng::optimize(&InFile::Path(input), &output, &opts) {
        Ok(_) => (),
        Err(x) => panic!("{}", x),
    };
    let output = output.path().unwrap();
    assert!(output.exists());

    let png = match PngData::new(&output, opts.fix_errors) {
        Ok(x) => x,
        Err(x) => {
            remove_file(&output).ok();
            panic!("{}", x)
        }
    };

    assert_eq!(png.raw.ihdr.color_type, ColorType::Indexed);
    assert_eq!(png.raw.ihdr.bit_depth, BitDepth::Eight);
    assert_eq!(png.raw.palette.as_ref().unwrap().len(), 33);

    remove_file(output).ok();
}

#[test]
fn rgba_16_reduce_alpha_black() {
    test_it_converts(
        "tests/files/rgba_16_reduce_alpha_black.png",
        None,
        ColorType::RGBA,
        BitDepth::Sixteen,
        ColorType::RGBA,
        BitDepth::Eight,
    );
}

#[test]
fn rgba_8_reduce_alpha_black() {
    test_it_converts(
        "tests/files/rgba_8_reduce_alpha_black.png",
        None,
        ColorType::RGBA,
        BitDepth::Eight,
        ColorType::RGBA,
        BitDepth::Eight,
    );
}

#[test]
fn grayscale_alpha_16_reduce_alpha_black() {
    test_it_converts(
        "tests/files/grayscale_alpha_16_reduce_alpha_black.png",
        None,
        ColorType::GrayscaleAlpha,
        BitDepth::Sixteen,
        ColorType::GrayscaleAlpha,
        BitDepth::Eight,
    );
}

#[test]
fn grayscale_alpha_8_reduce_alpha_black() {
    test_it_converts(
        "tests/files/grayscale_alpha_8_reduce_alpha_black.png",
        None,
        ColorType::GrayscaleAlpha,
        BitDepth::Eight,
        ColorType::GrayscaleAlpha,
        BitDepth::Eight,
    );
}

#[test]
fn rgba_16_reduce_alpha_white() {
    test_it_converts(
        "tests/files/rgba_16_reduce_alpha_white.png",
        Some(AlphaOptim::White),
        ColorType::RGBA,
        BitDepth::Sixteen,
        ColorType::RGBA,
        BitDepth::Eight,
    );
}

#[test]
fn rgba_8_reduce_alpha_white() {
    test_it_converts(
        "tests/files/rgba_8_reduce_alpha_white.png",
        Some(AlphaOptim::White),
        ColorType::RGBA,
        BitDepth::Eight,
        ColorType::RGBA,
        BitDepth::Eight,
    );
}

#[test]
fn grayscale_alpha_16_reduce_alpha_white() {
    test_it_converts(
        "tests/files/grayscale_alpha_16_reduce_alpha_white.png",
        Some(AlphaOptim::White),
        ColorType::GrayscaleAlpha,
        BitDepth::Sixteen,
        ColorType::GrayscaleAlpha,
        BitDepth::Eight,
    );
}

#[test]
fn grayscale_alpha_8_reduce_alpha_white() {
    test_it_converts(
        "tests/files/grayscale_alpha_8_reduce_alpha_white.png",
        Some(AlphaOptim::White),
        ColorType::GrayscaleAlpha,
        BitDepth::Eight,
        ColorType::GrayscaleAlpha,
        BitDepth::Eight,
    );
}

#[test]
fn rgba_16_reduce_alpha_down() {
    test_it_converts(
        "tests/files/rgba_16_reduce_alpha_down.png",
        Some(AlphaOptim::Down),
        ColorType::RGBA,
        BitDepth::Sixteen,
        ColorType::RGBA,
        BitDepth::Eight,
    );
}

#[test]
fn rgba_8_reduce_alpha_down() {
    test_it_converts(
        "tests/files/rgba_8_reduce_alpha_down.png",
        Some(AlphaOptim::Down),
        ColorType::RGBA,
        BitDepth::Eight,
        ColorType::RGBA,
        BitDepth::Eight,
    );
}

#[test]
fn grayscale_alpha_16_reduce_alpha_down() {
    test_it_converts(
        "tests/files/grayscale_alpha_16_reduce_alpha_down.png",
        Some(AlphaOptim::Down),
        ColorType::GrayscaleAlpha,
        BitDepth::Sixteen,
        ColorType::GrayscaleAlpha,
        BitDepth::Eight,
    );
}

#[test]
fn grayscale_alpha_8_reduce_alpha_down() {
    test_it_converts(
        "tests/files/grayscale_alpha_8_reduce_alpha_down.png",
        Some(AlphaOptim::Down),
        ColorType::GrayscaleAlpha,
        BitDepth::Eight,
        ColorType::GrayscaleAlpha,
        BitDepth::Eight,
    );
}

#[test]
fn rgba_16_reduce_alpha_up() {
    test_it_converts(
        "tests/files/rgba_16_reduce_alpha_up.png",
        Some(AlphaOptim::Up),
        ColorType::RGBA,
        BitDepth::Sixteen,
        ColorType::RGBA,
        BitDepth::Eight,
    );
}

#[test]
fn rgba_8_reduce_alpha_up() {
    test_it_converts(
        "tests/files/rgba_8_reduce_alpha_up.png",
        Some(AlphaOptim::Up),
        ColorType::RGBA,
        BitDepth::Eight,
        ColorType::RGBA,
        BitDepth::Eight,
    );
}

#[test]
fn grayscale_alpha_16_reduce_alpha_up() {
    test_it_converts(
        "tests/files/grayscale_alpha_16_reduce_alpha_up.png",
        Some(AlphaOptim::Up),
        ColorType::GrayscaleAlpha,
        BitDepth::Sixteen,
        ColorType::GrayscaleAlpha,
        BitDepth::Eight,
    );
}

#[test]
fn grayscale_alpha_8_reduce_alpha_up() {
    test_it_converts(
        "tests/files/grayscale_alpha_8_reduce_alpha_up.png",
        Some(AlphaOptim::Up),
        ColorType::GrayscaleAlpha,
        BitDepth::Eight,
        ColorType::GrayscaleAlpha,
        BitDepth::Eight,
    );
}

#[test]
fn rgba_16_reduce_alpha_left() {
    test_it_converts(
        "tests/files/rgba_16_reduce_alpha_left.png",
        Some(AlphaOptim::Left),
        ColorType::RGBA,
        BitDepth::Sixteen,
        ColorType::RGBA,
        BitDepth::Eight,
    );
}

#[test]
fn rgba_8_reduce_alpha_left() {
    test_it_converts(
        "tests/files/rgba_8_reduce_alpha_left.png",
        Some(AlphaOptim::Left),
        ColorType::RGBA,
        BitDepth::Eight,
        ColorType::RGBA,
        BitDepth::Eight,
    );
}

#[test]
fn grayscale_alpha_16_reduce_alpha_left() {
    test_it_converts(
        "tests/files/grayscale_alpha_16_reduce_alpha_left.png",
        Some(AlphaOptim::Left),
        ColorType::GrayscaleAlpha,
        BitDepth::Sixteen,
        ColorType::GrayscaleAlpha,
        BitDepth::Eight,
    );
}

#[test]
fn grayscale_alpha_8_reduce_alpha_left() {
    test_it_converts(
        "tests/files/grayscale_alpha_8_reduce_alpha_left.png",
        Some(AlphaOptim::Left),
        ColorType::GrayscaleAlpha,
        BitDepth::Eight,
        ColorType::GrayscaleAlpha,
        BitDepth::Eight,
    );
}

#[test]
fn rgba_16_reduce_alpha_right() {
    test_it_converts(
        "tests/files/rgba_16_reduce_alpha_right.png",
        Some(AlphaOptim::Right),
        ColorType::RGBA,
        BitDepth::Sixteen,
        ColorType::RGBA,
        BitDepth::Eight,
    );
}

#[test]
fn rgba_8_reduce_alpha_right() {
    test_it_converts(
        "tests/files/rgba_8_reduce_alpha_right.png",
        Some(AlphaOptim::Right),
        ColorType::RGBA,
        BitDepth::Eight,
        ColorType::RGBA,
        BitDepth::Eight,
    );
}

#[test]
fn grayscale_alpha_16_reduce_alpha_right() {
    test_it_converts(
        "tests/files/grayscale_alpha_16_reduce_alpha_right.png",
        Some(AlphaOptim::Right),
        ColorType::GrayscaleAlpha,
        BitDepth::Sixteen,
        ColorType::GrayscaleAlpha,
        BitDepth::Eight,
    );
}

#[test]
fn grayscale_alpha_8_reduce_alpha_right() {
    test_it_converts(
        "tests/files/grayscale_alpha_8_reduce_alpha_right.png",
        Some(AlphaOptim::Right),
        ColorType::GrayscaleAlpha,
        BitDepth::Eight,
        ColorType::GrayscaleAlpha,
        BitDepth::Eight,
    );
}
