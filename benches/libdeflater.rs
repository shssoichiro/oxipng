#![feature(test)]

extern crate oxipng;
extern crate test;

use oxipng::internal_tests::*;
use std::path::PathBuf;
use test::Bencher;

#[bench]
fn libdeflater_16_bits_strategy_0(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from("tests/files/rgb_16_should_be_rgb_16.png"));
    let png = PngData::new(&input, false).unwrap();

    b.iter(|| {
        libdeflater_deflate(png.raw.data.as_ref()).ok();
    });
}

#[bench]
fn libdeflater_8_bits_strategy_0(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from("tests/files/rgb_8_should_be_rgb_8.png"));
    let png = PngData::new(&input, false).unwrap();

    b.iter(|| {
        libdeflater_deflate(png.raw.data.as_ref()).ok();
    });
}

#[bench]
fn libdeflater_4_bits_strategy_0(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from(
        "tests/files/palette_4_should_be_palette_4.png",
    ));
    let png = PngData::new(&input, false).unwrap();

    b.iter(|| {
        libdeflater_deflate(png.raw.data.as_ref()).ok();
    });
}

#[bench]
fn libdeflater_2_bits_strategy_0(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from(
        "tests/files/palette_2_should_be_palette_2.png",
    ));
    let png = PngData::new(&input, false).unwrap();

    b.iter(|| {
        libdeflater_deflate(png.raw.data.as_ref()).ok();
    });
}

#[bench]
fn libdeflater_1_bits_strategy_0(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from(
        "tests/files/palette_1_should_be_palette_1.png",
    ));
    let png = PngData::new(&input, false).unwrap();

    b.iter(|| {
        libdeflater_deflate(png.raw.data.as_ref()).ok();
    });
}
