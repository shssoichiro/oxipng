#![feature(test)]

extern crate oxipng;
extern crate test;

use oxipng::internal_tests::*;
use std::path::PathBuf;
use test::Bencher;

#[bench]
fn reductions_16_to_8_bits(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from("tests/files/rgb_16_should_be_rgb_8.png"));
    let png = PngData::new(&input, false).unwrap();

    b.iter(|| {
        let mut safe_png = png.clone();
        safe_png.reduce_bit_depth();
    });
}

#[bench]
fn reductions_8_to_4_bits(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from(
        "tests/files/palette_8_should_be_palette_4.png",
    ));
    let png = PngData::new(&input, false).unwrap();

    b.iter(|| {
        let mut safe_png = png.clone();
        safe_png.reduce_bit_depth();
    });
}

#[bench]
fn reductions_8_to_2_bits(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from(
        "tests/files/palette_8_should_be_palette_2.png",
    ));
    let png = PngData::new(&input, false).unwrap();

    b.iter(|| {
        let mut safe_png = png.clone();
        safe_png.reduce_bit_depth();
    });
}

#[bench]
fn reductions_8_to_1_bits(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from(
        "tests/files/palette_8_should_be_palette_1.png",
    ));
    let png = PngData::new(&input, false).unwrap();

    b.iter(|| {
        let mut safe_png = png.clone();
        safe_png.reduce_bit_depth();
    });
}

#[bench]
fn reductions_4_to_2_bits(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from(
        "tests/files/palette_4_should_be_palette_2.png",
    ));
    let png = PngData::new(&input, false).unwrap();

    b.iter(|| {
        let mut safe_png = png.clone();
        safe_png.reduce_bit_depth();
    });
}

#[bench]
fn reductions_4_to_1_bits(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from(
        "tests/files/palette_4_should_be_palette_1.png",
    ));
    let png = PngData::new(&input, false).unwrap();

    b.iter(|| {
        let mut safe_png = png.clone();
        safe_png.reduce_bit_depth();
    });
}

#[bench]
fn reductions_2_to_1_bits(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from(
        "tests/files/palette_2_should_be_palette_1.png",
    ));
    let png = PngData::new(&input, false).unwrap();

    b.iter(|| {
        let mut safe_png = png.clone();
        safe_png.reduce_bit_depth();
    });
}

#[bench]
fn reductions_rgba_to_rgb_16(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from("tests/files/rgba_16_should_be_rgb_16.png"));
    let png = PngData::new(&input, false).unwrap();

    b.iter(|| {
        let mut safe_png = png.clone();
        safe_png.reduce_color_type();
    });
}

#[bench]
fn reductions_rgba_to_rgb_8(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from("tests/files/rgba_8_should_be_rgb_8.png"));
    let png = PngData::new(&input, false).unwrap();

    b.iter(|| {
        let mut safe_png = png.clone();
        safe_png.reduce_color_type();
    });
}

#[bench]
fn reductions_rgba_to_grayscale_alpha_16(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from(
        "tests/files/rgba_16_should_be_grayscale_alpha_16.png",
    ));
    let png = PngData::new(&input, false).unwrap();

    b.iter(|| {
        let mut safe_png = png.clone();
        safe_png.reduce_color_type();
    });
}

#[bench]
fn reductions_rgba_to_grayscale_alpha_8(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from(
        "tests/files/rgba_8_should_be_grayscale_alpha_8.png",
    ));
    let png = PngData::new(&input, false).unwrap();

    b.iter(|| {
        let mut safe_png = png.clone();
        safe_png.reduce_color_type();
    });
}

#[bench]
fn reductions_rgba_to_grayscale_16(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from(
        "tests/files/rgba_16_should_be_grayscale_16.png",
    ));
    let png = PngData::new(&input, false).unwrap();

    b.iter(|| {
        let mut safe_png = png.clone();
        safe_png.reduce_color_type();
    });
}

#[bench]
fn reductions_rgba_to_grayscale_8(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from(
        "tests/files/rgba_8_should_be_grayscale_8.png",
    ));
    let png = PngData::new(&input, false).unwrap();

    b.iter(|| {
        let mut safe_png = png.clone();
        safe_png.reduce_color_type();
    });
}

#[bench]
fn reductions_rgb_to_grayscale_16(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from(
        "tests/files/rgb_16_should_be_grayscale_16.png",
    ));
    let png = PngData::new(&input, false).unwrap();

    b.iter(|| {
        let mut safe_png = png.clone();
        safe_png.reduce_color_type();
    });
}

#[bench]
fn reductions_rgb_to_grayscale_8(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from("tests/files/rgb_8_should_be_grayscale_8.png"));
    let png = PngData::new(&input, false).unwrap();

    b.iter(|| {
        let mut safe_png = png.clone();
        safe_png.reduce_color_type();
    });
}

#[bench]
fn reductions_rgba_to_palette_8(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from("tests/files/rgba_8_should_be_palette_8.png"));
    let png = PngData::new(&input, false).unwrap();

    b.iter(|| {
        let mut safe_png = png.clone();
        safe_png.reduce_color_type();
    });
}

#[bench]
fn reductions_rgb_to_palette_8(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from("tests/files/rgb_8_should_be_palette_8.png"));
    let png = PngData::new(&input, false).unwrap();

    b.iter(|| {
        let mut safe_png = png.clone();
        safe_png.reduce_color_type();
    });
}

#[bench]
fn reductions_palette_duplicate_reduction(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from(
        "tests/files/palette_should_be_reduced_with_dupes.png",
    ));
    let png = PngData::new(&input, false).unwrap();

    b.iter(|| {
        let mut safe_png = png.clone();
        safe_png.reduce_palette();
    });
}

#[bench]
fn reductions_palette_unused_reduction(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from(
        "tests/files/palette_should_be_reduced_with_unused.png",
    ));
    let png = PngData::new(&input, false).unwrap();

    b.iter(|| {
        let mut safe_png = png.clone();
        safe_png.reduce_palette();
    });
}

#[bench]
fn reductions_palette_full_reduction(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from(
        "tests/files/palette_should_be_reduced_with_both.png",
    ));
    let png = PngData::new(&input, false).unwrap();

    b.iter(|| {
        let mut safe_png = png.clone();
        safe_png.reduce_palette();
    });
}

#[bench]
fn reductions_alpha_black(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from("tests/files/rgba_8_reduce_alpha_black.png"));
    let png = PngData::new(&input, false).unwrap();

    b.iter(|| {
        let mut safe_png = png.clone();
        safe_png.reduced_alpha_channel(AlphaOptim::Black);
    });
}

#[bench]
fn reductions_alpha_white(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from("tests/files/rgba_8_reduce_alpha_white.png"));
    let png = PngData::new(&input, false).unwrap();

    b.iter(|| {
        let mut safe_png = png.clone();
        safe_png.reduced_alpha_channel(AlphaOptim::White);
    });
}

#[bench]
fn reductions_alpha_left(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from("tests/files/rgba_8_reduce_alpha_left.png"));
    let png = PngData::new(&input, false).unwrap();

    b.iter(|| {
        let mut safe_png = png.clone();
        safe_png.reduced_alpha_channel(AlphaOptim::Left);
    });
}

#[bench]
fn reductions_alpha_right(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from("tests/files/rgba_8_reduce_alpha_right.png"));
    let png = PngData::new(&input, false).unwrap();

    b.iter(|| {
        let mut safe_png = png.clone();
        safe_png.reduced_alpha_channel(AlphaOptim::Right);
    });
}

#[bench]
fn reductions_alpha_up(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from("tests/files/rgba_8_reduce_alpha_up.png"));
    let png = PngData::new(&input, false).unwrap();

    b.iter(|| {
        let mut safe_png = png.clone();
        safe_png.reduced_alpha_channel(AlphaOptim::Up);
    });
}

#[bench]
fn reductions_alpha_down(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from("tests/files/rgba_8_reduce_alpha_down.png"));
    let png = PngData::new(&input, false).unwrap();

    b.iter(|| {
        let mut safe_png = png.clone();
        safe_png.reduced_alpha_channel(AlphaOptim::Down);
    });
}
