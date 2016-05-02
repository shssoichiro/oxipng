#![feature(test)]

extern crate test;
extern crate oxipng;

use oxipng::png;
use test::Bencher;
use std::path::PathBuf;

#[bench]
fn bench_interlace_16_bits(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from("tests/files/rgb_16_should_be_rgb_16.png"));
    let png = png::PngData::new(&input, false).unwrap();

    b.iter(|| {
        let mut safe_png = png.clone();
        safe_png.change_interlacing(1);
    });
}

#[bench]
fn bench_interlace_8_bits(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from("tests/files/rgb_8_should_be_rgb_8.png"));
    let png = png::PngData::new(&input, false).unwrap();

    b.iter(|| {
        let mut safe_png = png.clone();
        safe_png.change_interlacing(1);
    });
}

#[bench]
fn bench_interlace_4_bits(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from("tests/files/palette_4_should_be_palette_4.png"));
    let png = png::PngData::new(&input, false).unwrap();

    b.iter(|| {
        let mut safe_png = png.clone();
        safe_png.change_interlacing(1);
    });
}

#[bench]
fn bench_interlace_2_bits(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from("tests/files/palette_2_should_be_palette_2.png"));
    let png = png::PngData::new(&input, false).unwrap();

    b.iter(|| {
        let mut safe_png = png.clone();
        safe_png.change_interlacing(1);
    });
}

#[bench]
fn bench_interlace_1_bits(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from("tests/files/palette_1_should_be_palette_1.png"));
    let png = png::PngData::new(&input, false).unwrap();

    b.iter(|| {
        let mut safe_png = png.clone();
        safe_png.change_interlacing(1);
    });
}

#[bench]
fn bench_deinterlace_16_bits(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from("tests/files/interlaced_rgb_16_should_be_rgb_16.png"));
    let png = png::PngData::new(&input, false).unwrap();

    b.iter(|| {
        let mut safe_png = png.clone();
        safe_png.change_interlacing(0);
    });
}

#[bench]
fn bench_deinterlace_8_bits(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from("tests/files/interlaced_rgb_8_should_be_rgb_8.png"));
    let png = png::PngData::new(&input, false).unwrap();

    b.iter(|| {
        let mut safe_png = png.clone();
        safe_png.change_interlacing(0);
    });
}

#[bench]
fn bench_deinterlace_4_bits(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from("tests/files/interlaced_palette_4_should_be_palette_4.png"));
    let png = png::PngData::new(&input, false).unwrap();

    b.iter(|| {
        let mut safe_png = png.clone();
        safe_png.change_interlacing(0);
    });
}

#[bench]
fn bench_deinterlace_2_bits(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from("tests/files/interlaced_palette_2_should_be_palette_2.png"));
    let png = png::PngData::new(&input, false).unwrap();

    b.iter(|| {
        let mut safe_png = png.clone();
        safe_png.change_interlacing(0);
    });
}

#[bench]
fn bench_deinterlace_1_bits(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from("tests/files/interlaced_palette_1_should_be_palette_1.png"));
    let png = png::PngData::new(&input, false).unwrap();

    b.iter(|| {
        let mut safe_png = png.clone();
        safe_png.change_interlacing(0);
    });
}
