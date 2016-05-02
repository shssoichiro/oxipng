#![feature(test)]

extern crate test;
extern crate oxipng;

use oxipng::png;
use test::Bencher;
use std::path::PathBuf;

#[bench]
fn bench_16_bits_filter_0(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from("tests/files/rgb_16_should_be_rgb_16.png"));
    let png = png::PngData::new(&input, false).unwrap();

    b.iter(|| {
        png.filter_image(0);
    });
}

#[bench]
fn bench_8_bits_filter_0(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from("tests/files/rgb_8_should_be_rgb_8.png"));
    let png = png::PngData::new(&input, false).unwrap();

    b.iter(|| {
        png.filter_image(0);
    });
}

#[bench]
fn bench_4_bits_filter_0(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from("tests/files/palette_4_should_be_palette_4.png"));
    let png = png::PngData::new(&input, false).unwrap();

    b.iter(|| {
        png.filter_image(0);
    });
}

#[bench]
fn bench_2_bits_filter_0(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from("tests/files/palette_2_should_be_palette_2.png"));
    let png = png::PngData::new(&input, false).unwrap();

    b.iter(|| {
        png.filter_image(0);
    });
}

#[bench]
fn bench_1_bits_filter_0(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from("tests/files/palette_1_should_be_palette_1.png"));
    let png = png::PngData::new(&input, false).unwrap();

    b.iter(|| {
        png.filter_image(0);
    });
}

#[bench]
fn bench_16_bits_filter_1(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from("tests/files/rgb_16_should_be_rgb_16.png"));
    let png = png::PngData::new(&input, false).unwrap();

    b.iter(|| {
        png.filter_image(1);
    });
}

#[bench]
fn bench_8_bits_filter_1(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from("tests/files/rgb_8_should_be_rgb_8.png"));
    let png = png::PngData::new(&input, false).unwrap();

    b.iter(|| {
        png.filter_image(1);
    });
}

#[bench]
fn bench_4_bits_filter_1(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from("tests/files/palette_4_should_be_palette_4.png"));
    let png = png::PngData::new(&input, false).unwrap();

    b.iter(|| {
        png.filter_image(1);
    });
}

#[bench]
fn bench_2_bits_filter_1(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from("tests/files/palette_2_should_be_palette_2.png"));
    let png = png::PngData::new(&input, false).unwrap();

    b.iter(|| {
        png.filter_image(1);
    });
}

#[bench]
fn bench_1_bits_filter_1(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from("tests/files/palette_1_should_be_palette_1.png"));
    let png = png::PngData::new(&input, false).unwrap();

    b.iter(|| {
        png.filter_image(1);
    });
}

#[bench]
fn bench_16_bits_filter_2(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from("tests/files/rgb_16_should_be_rgb_16.png"));
    let png = png::PngData::new(&input, false).unwrap();

    b.iter(|| {
        png.filter_image(2);
    });
}

#[bench]
fn bench_8_bits_filter_2(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from("tests/files/rgb_8_should_be_rgb_8.png"));
    let png = png::PngData::new(&input, false).unwrap();

    b.iter(|| {
        png.filter_image(2);
    });
}

#[bench]
fn bench_4_bits_filter_2(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from("tests/files/palette_4_should_be_palette_4.png"));
    let png = png::PngData::new(&input, false).unwrap();

    b.iter(|| {
        png.filter_image(2);
    });
}

#[bench]
fn bench_2_bits_filter_2(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from("tests/files/palette_2_should_be_palette_2.png"));
    let png = png::PngData::new(&input, false).unwrap();

    b.iter(|| {
        png.filter_image(2);
    });
}

#[bench]
fn bench_1_bits_filter_2(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from("tests/files/palette_1_should_be_palette_1.png"));
    let png = png::PngData::new(&input, false).unwrap();

    b.iter(|| {
        png.filter_image(2);
    });
}

#[bench]
fn bench_16_bits_filter_3(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from("tests/files/rgb_16_should_be_rgb_16.png"));
    let png = png::PngData::new(&input, false).unwrap();

    b.iter(|| {
        png.filter_image(3);
    });
}

#[bench]
fn bench_8_bits_filter_3(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from("tests/files/rgb_8_should_be_rgb_8.png"));
    let png = png::PngData::new(&input, false).unwrap();

    b.iter(|| {
        png.filter_image(3);
    });
}

#[bench]
fn bench_4_bits_filter_3(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from("tests/files/palette_4_should_be_palette_4.png"));
    let png = png::PngData::new(&input, false).unwrap();

    b.iter(|| {
        png.filter_image(3);
    });
}

#[bench]
fn bench_2_bits_filter_3(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from("tests/files/palette_2_should_be_palette_2.png"));
    let png = png::PngData::new(&input, false).unwrap();

    b.iter(|| {
        png.filter_image(3);
    });
}

#[bench]
fn bench_1_bits_filter_3(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from("tests/files/palette_1_should_be_palette_1.png"));
    let png = png::PngData::new(&input, false).unwrap();

    b.iter(|| {
        png.filter_image(3);
    });
}

#[bench]
fn bench_16_bits_filter_4(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from("tests/files/rgb_16_should_be_rgb_16.png"));
    let png = png::PngData::new(&input, false).unwrap();

    b.iter(|| {
        png.filter_image(4);
    });
}

#[bench]
fn bench_8_bits_filter_4(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from("tests/files/rgb_8_should_be_rgb_8.png"));
    let png = png::PngData::new(&input, false).unwrap();

    b.iter(|| {
        png.filter_image(4);
    });
}

#[bench]
fn bench_4_bits_filter_4(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from("tests/files/palette_4_should_be_palette_4.png"));
    let png = png::PngData::new(&input, false).unwrap();

    b.iter(|| {
        png.filter_image(4);
    });
}

#[bench]
fn bench_2_bits_filter_4(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from("tests/files/palette_2_should_be_palette_2.png"));
    let png = png::PngData::new(&input, false).unwrap();

    b.iter(|| {
        png.filter_image(4);
    });
}

#[bench]
fn bench_1_bits_filter_4(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from("tests/files/palette_1_should_be_palette_1.png"));
    let png = png::PngData::new(&input, false).unwrap();

    b.iter(|| {
        png.filter_image(4);
    });
}

#[bench]
fn bench_16_bits_filter_5(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from("tests/files/rgb_16_should_be_rgb_16.png"));
    let png = png::PngData::new(&input, false).unwrap();

    b.iter(|| {
        png.filter_image(5);
    });
}

#[bench]
fn bench_8_bits_filter_5(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from("tests/files/rgb_8_should_be_rgb_8.png"));
    let png = png::PngData::new(&input, false).unwrap();

    b.iter(|| {
        png.filter_image(5);
    });
}

#[bench]
fn bench_4_bits_filter_5(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from("tests/files/palette_4_should_be_palette_4.png"));
    let png = png::PngData::new(&input, false).unwrap();

    b.iter(|| {
        png.filter_image(5);
    });
}

#[bench]
fn bench_2_bits_filter_5(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from("tests/files/palette_2_should_be_palette_2.png"));
    let png = png::PngData::new(&input, false).unwrap();

    b.iter(|| {
        png.filter_image(5);
    });
}

#[bench]
fn bench_1_bits_filter_5(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from("tests/files/palette_1_should_be_palette_1.png"));
    let png = png::PngData::new(&input, false).unwrap();

    b.iter(|| {
        png.filter_image(5);
    });
}
