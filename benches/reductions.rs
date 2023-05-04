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

    b.iter(|| bit_depth::reduced_bit_depth_16_to_8(&png.raw));
}

#[bench]
fn reductions_8_to_4_bits(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from(
        "tests/files/palette_8_should_be_palette_4.png",
    ));
    let png = PngData::new(&input, false).unwrap();

    b.iter(|| bit_depth::reduced_bit_depth_8_or_less(&png.raw, 1));
}

#[bench]
fn reductions_8_to_2_bits(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from(
        "tests/files/palette_8_should_be_palette_2.png",
    ));
    let png = PngData::new(&input, false).unwrap();

    b.iter(|| bit_depth::reduced_bit_depth_8_or_less(&png.raw, 1));
}

#[bench]
fn reductions_8_to_1_bits(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from(
        "tests/files/palette_8_should_be_palette_1.png",
    ));
    let png = PngData::new(&input, false).unwrap();

    b.iter(|| bit_depth::reduced_bit_depth_8_or_less(&png.raw, 1));
}

#[bench]
fn reductions_4_to_2_bits(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from(
        "tests/files/palette_4_should_be_palette_2.png",
    ));
    let png = PngData::new(&input, false).unwrap();

    b.iter(|| bit_depth::reduced_bit_depth_8_or_less(&png.raw, 1));
}

#[bench]
fn reductions_4_to_1_bits(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from(
        "tests/files/palette_4_should_be_palette_1.png",
    ));
    let png = PngData::new(&input, false).unwrap();

    b.iter(|| bit_depth::reduced_bit_depth_8_or_less(&png.raw, 1));
}

#[bench]
fn reductions_2_to_1_bits(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from(
        "tests/files/palette_2_should_be_palette_1.png",
    ));
    let png = PngData::new(&input, false).unwrap();

    b.iter(|| bit_depth::reduced_bit_depth_8_or_less(&png.raw, 1));
}

#[bench]
fn reductions_grayscale_8_to_4_bits(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from(
        "tests/files/grayscale_8_should_be_grayscale_4.png",
    ));
    let png = PngData::new(&input, false).unwrap();

    b.iter(|| bit_depth::reduced_bit_depth_8_or_less(&png.raw, 1));
}

#[bench]
fn reductions_grayscale_8_to_2_bits(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from(
        "tests/files/grayscale_8_should_be_grayscale_2.png",
    ));
    let png = PngData::new(&input, false).unwrap();

    b.iter(|| bit_depth::reduced_bit_depth_8_or_less(&png.raw, 1));
}

#[bench]
fn reductions_grayscale_8_to_1_bits(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from(
        "tests/files/grayscale_8_should_be_grayscale_1.png",
    ));
    let png = PngData::new(&input, false).unwrap();

    b.iter(|| bit_depth::reduced_bit_depth_8_or_less(&png.raw, 1));
}

#[bench]
fn reductions_grayscale_4_to_2_bits(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from(
        "tests/files/grayscale_4_should_be_grayscale_2.png",
    ));
    let png = PngData::new(&input, false).unwrap();

    b.iter(|| bit_depth::reduced_bit_depth_8_or_less(&png.raw, 1));
}

#[bench]
fn reductions_grayscale_4_to_1_bits(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from(
        "tests/files/grayscale_4_should_be_grayscale_1.png",
    ));
    let png = PngData::new(&input, false).unwrap();

    b.iter(|| bit_depth::reduced_bit_depth_8_or_less(&png.raw, 1));
}

#[bench]
fn reductions_grayscale_2_to_1_bits(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from(
        "tests/files/grayscale_2_should_be_grayscale_1.png",
    ));
    let png = PngData::new(&input, false).unwrap();

    b.iter(|| bit_depth::reduced_bit_depth_8_or_less(&png.raw, 1));
}

#[bench]
fn reductions_rgba_to_rgb_16(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from("tests/files/rgba_16_should_be_rgb_16.png"));
    let png = PngData::new(&input, false).unwrap();

    b.iter(|| reduce_color_type(&png.raw, true, false));
}

#[bench]
fn reductions_rgba_to_rgb_8(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from("tests/files/rgba_8_should_be_rgb_8.png"));
    let png = PngData::new(&input, false).unwrap();

    b.iter(|| reduce_color_type(&png.raw, true, false));
}

#[bench]
fn reductions_rgba_to_grayscale_alpha_16(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from(
        "tests/files/rgba_16_should_be_grayscale_alpha_16.png",
    ));
    let png = PngData::new(&input, false).unwrap();

    b.iter(|| reduce_color_type(&png.raw, true, false));
}

#[bench]
fn reductions_rgba_to_grayscale_alpha_8(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from(
        "tests/files/rgba_8_should_be_grayscale_alpha_8.png",
    ));
    let png = PngData::new(&input, false).unwrap();

    b.iter(|| reduce_color_type(&png.raw, true, false));
}

#[bench]
fn reductions_rgba_to_grayscale_16(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from(
        "tests/files/rgba_16_should_be_grayscale_16.png",
    ));
    let png = PngData::new(&input, false).unwrap();

    b.iter(|| reduce_color_type(&png.raw, true, false));
}

#[bench]
fn reductions_rgba_to_grayscale_8(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from(
        "tests/files/rgba_8_should_be_grayscale_8.png",
    ));
    let png = PngData::new(&input, false).unwrap();

    b.iter(|| reduce_color_type(&png.raw, true, false));
}

#[bench]
fn reductions_rgb_to_grayscale_16(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from(
        "tests/files/rgb_16_should_be_grayscale_16.png",
    ));
    let png = PngData::new(&input, false).unwrap();

    b.iter(|| reduce_color_type(&png.raw, true, false));
}

#[bench]
fn reductions_rgb_to_grayscale_8(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from("tests/files/rgb_8_should_be_grayscale_8.png"));
    let png = PngData::new(&input, false).unwrap();

    b.iter(|| reduce_color_type(&png.raw, true, false));
}

#[bench]
fn reductions_rgba_to_palette_8(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from("tests/files/rgba_8_should_be_palette_8.png"));
    let png = PngData::new(&input, false).unwrap();

    b.iter(|| reduce_color_type(&png.raw, true, false));
}

#[bench]
fn reductions_rgb_to_palette_8(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from("tests/files/rgb_8_should_be_palette_8.png"));
    let png = PngData::new(&input, false).unwrap();

    b.iter(|| reduce_color_type(&png.raw, true, false));
}

#[bench]
fn reductions_palette_duplicate_reduction(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from(
        "tests/files/palette_should_be_reduced_with_dupes.png",
    ));
    let png = PngData::new(&input, false).unwrap();

    b.iter(|| reduced_palette(&png.raw, false));
}

#[bench]
fn reductions_palette_unused_reduction(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from(
        "tests/files/palette_should_be_reduced_with_unused.png",
    ));
    let png = PngData::new(&input, false).unwrap();

    b.iter(|| reduced_palette(&png.raw, false));
}

#[bench]
fn reductions_palette_full_reduction(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from(
        "tests/files/palette_should_be_reduced_with_both.png",
    ));
    let png = PngData::new(&input, false).unwrap();

    b.iter(|| reduced_palette(&png.raw, false));
}

#[bench]
fn reductions_alpha(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from("tests/files/rgba_8_reduce_alpha.png"));
    let png = PngData::new(&input, false).unwrap();

    b.iter(|| alpha::cleaned_alpha_channel(&png.raw));
}
