#![feature(test)]

extern crate oxipng;
extern crate test;

use oxipng::{internal_tests::*, RowFilter};
use std::path::PathBuf;
use test::Bencher;

#[bench]
fn filters_16_bits_filter_0(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from("tests/files/rgb_16_should_be_rgb_16.png"));
    let png = PngData::new(&input, false).unwrap();

    b.iter(|| {
        png.raw.filter_image(RowFilter::None);
    });
}

#[bench]
fn filters_8_bits_filter_0(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from("tests/files/rgb_8_should_be_rgb_8.png"));
    let png = PngData::new(&input, false).unwrap();

    b.iter(|| {
        png.raw.filter_image(RowFilter::None);
    });
}

#[bench]
fn filters_4_bits_filter_0(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from(
        "tests/files/palette_4_should_be_palette_4.png",
    ));
    let png = PngData::new(&input, false).unwrap();

    b.iter(|| {
        png.raw.filter_image(RowFilter::None);
    });
}

#[bench]
fn filters_2_bits_filter_0(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from(
        "tests/files/palette_2_should_be_palette_2.png",
    ));
    let png = PngData::new(&input, false).unwrap();

    b.iter(|| {
        png.raw.filter_image(RowFilter::None);
    });
}

#[bench]
fn filters_1_bits_filter_0(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from(
        "tests/files/palette_1_should_be_palette_1.png",
    ));
    let png = PngData::new(&input, false).unwrap();

    b.iter(|| {
        png.raw.filter_image(RowFilter::None);
    });
}

#[bench]
fn filters_16_bits_filter_1(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from("tests/files/rgb_16_should_be_rgb_16.png"));
    let png = PngData::new(&input, false).unwrap();

    b.iter(|| {
        png.raw.filter_image(RowFilter::Sub);
    });
}

#[bench]
fn filters_8_bits_filter_1(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from("tests/files/rgb_8_should_be_rgb_8.png"));
    let png = PngData::new(&input, false).unwrap();

    b.iter(|| {
        png.raw.filter_image(RowFilter::Sub);
    });
}

#[bench]
fn filters_4_bits_filter_1(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from(
        "tests/files/palette_4_should_be_palette_4.png",
    ));
    let png = PngData::new(&input, false).unwrap();

    b.iter(|| {
        png.raw.filter_image(RowFilter::Sub);
    });
}

#[bench]
fn filters_2_bits_filter_1(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from(
        "tests/files/palette_2_should_be_palette_2.png",
    ));
    let png = PngData::new(&input, false).unwrap();

    b.iter(|| {
        png.raw.filter_image(RowFilter::Sub);
    });
}

#[bench]
fn filters_1_bits_filter_1(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from(
        "tests/files/palette_1_should_be_palette_1.png",
    ));
    let png = PngData::new(&input, false).unwrap();

    b.iter(|| {
        png.raw.filter_image(RowFilter::Sub);
    });
}

#[bench]
fn filters_16_bits_filter_2(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from("tests/files/rgb_16_should_be_rgb_16.png"));
    let png = PngData::new(&input, false).unwrap();

    b.iter(|| {
        png.raw.filter_image(RowFilter::Up);
    });
}

#[bench]
fn filters_8_bits_filter_2(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from("tests/files/rgb_8_should_be_rgb_8.png"));
    let png = PngData::new(&input, false).unwrap();

    b.iter(|| {
        png.raw.filter_image(RowFilter::Up);
    });
}

#[bench]
fn filters_4_bits_filter_2(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from(
        "tests/files/palette_4_should_be_palette_4.png",
    ));
    let png = PngData::new(&input, false).unwrap();

    b.iter(|| {
        png.raw.filter_image(RowFilter::Up);
    });
}

#[bench]
fn filters_2_bits_filter_2(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from(
        "tests/files/palette_2_should_be_palette_2.png",
    ));
    let png = PngData::new(&input, false).unwrap();

    b.iter(|| {
        png.raw.filter_image(RowFilter::Up);
    });
}

#[bench]
fn filters_1_bits_filter_2(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from(
        "tests/files/palette_1_should_be_palette_1.png",
    ));
    let png = PngData::new(&input, false).unwrap();

    b.iter(|| {
        png.raw.filter_image(RowFilter::Up);
    });
}

#[bench]
fn filters_16_bits_filter_3(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from("tests/files/rgb_16_should_be_rgb_16.png"));
    let png = PngData::new(&input, false).unwrap();

    b.iter(|| {
        png.raw.filter_image(RowFilter::Average);
    });
}

#[bench]
fn filters_8_bits_filter_3(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from("tests/files/rgb_8_should_be_rgb_8.png"));
    let png = PngData::new(&input, false).unwrap();

    b.iter(|| {
        png.raw.filter_image(RowFilter::Average);
    });
}

#[bench]
fn filters_4_bits_filter_3(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from(
        "tests/files/palette_4_should_be_palette_4.png",
    ));
    let png = PngData::new(&input, false).unwrap();

    b.iter(|| {
        png.raw.filter_image(RowFilter::Average);
    });
}

#[bench]
fn filters_2_bits_filter_3(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from(
        "tests/files/palette_2_should_be_palette_2.png",
    ));
    let png = PngData::new(&input, false).unwrap();

    b.iter(|| {
        png.raw.filter_image(RowFilter::Average);
    });
}

#[bench]
fn filters_1_bits_filter_3(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from(
        "tests/files/palette_1_should_be_palette_1.png",
    ));
    let png = PngData::new(&input, false).unwrap();

    b.iter(|| {
        png.raw.filter_image(RowFilter::Average);
    });
}

#[bench]
fn filters_16_bits_filter_4(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from("tests/files/rgb_16_should_be_rgb_16.png"));
    let png = PngData::new(&input, false).unwrap();

    b.iter(|| {
        png.raw.filter_image(RowFilter::Paeth);
    });
}

#[bench]
fn filters_8_bits_filter_4(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from("tests/files/rgb_8_should_be_rgb_8.png"));
    let png = PngData::new(&input, false).unwrap();

    b.iter(|| {
        png.raw.filter_image(RowFilter::Paeth);
    });
}

#[bench]
fn filters_4_bits_filter_4(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from(
        "tests/files/palette_4_should_be_palette_4.png",
    ));
    let png = PngData::new(&input, false).unwrap();

    b.iter(|| {
        png.raw.filter_image(RowFilter::Paeth);
    });
}

#[bench]
fn filters_2_bits_filter_4(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from(
        "tests/files/palette_2_should_be_palette_2.png",
    ));
    let png = PngData::new(&input, false).unwrap();

    b.iter(|| {
        png.raw.filter_image(RowFilter::Paeth);
    });
}

#[bench]
fn filters_1_bits_filter_4(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from(
        "tests/files/palette_1_should_be_palette_1.png",
    ));
    let png = PngData::new(&input, false).unwrap();

    b.iter(|| {
        png.raw.filter_image(RowFilter::Paeth);
    });
}

#[bench]
fn filters_16_bits_filter_5(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from("tests/files/rgb_16_should_be_rgb_16.png"));
    let png = PngData::new(&input, false).unwrap();

    b.iter(|| {
        png.raw.filter_image(RowFilter::MinSum);
    });
}

#[bench]
fn filters_8_bits_filter_5(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from("tests/files/rgb_8_should_be_rgb_8.png"));
    let png = PngData::new(&input, false).unwrap();

    b.iter(|| {
        png.raw.filter_image(RowFilter::MinSum);
    });
}

#[bench]
fn filters_4_bits_filter_5(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from(
        "tests/files/palette_4_should_be_palette_4.png",
    ));
    let png = PngData::new(&input, false).unwrap();

    b.iter(|| {
        png.raw.filter_image(RowFilter::MinSum);
    });
}

#[bench]
fn filters_2_bits_filter_5(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from(
        "tests/files/palette_2_should_be_palette_2.png",
    ));
    let png = PngData::new(&input, false).unwrap();

    b.iter(|| {
        png.raw.filter_image(RowFilter::MinSum);
    });
}

#[bench]
fn filters_1_bits_filter_5(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from(
        "tests/files/palette_1_should_be_palette_1.png",
    ));
    let png = PngData::new(&input, false).unwrap();

    b.iter(|| {
        png.raw.filter_image(RowFilter::MinSum);
    });
}
