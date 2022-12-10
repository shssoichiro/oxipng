#![feature(test)]

extern crate oxipng;
extern crate test;

use oxipng::{internal_tests::*, RowFilter};
use std::path::PathBuf;
use test::Bencher;

#[bench]
fn filters_minsum(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from("tests/files/rgb_8_should_be_rgb_8.png"));
    let png = PngData::new(&input, false).unwrap();

    b.iter(|| {
        png.raw.filter_image(RowFilter::MinSum, false);
    });
}

#[bench]
fn filters_entropy(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from("tests/files/rgb_8_should_be_rgb_8.png"));
    let png = PngData::new(&input, false).unwrap();

    b.iter(|| {
        png.raw.filter_image(RowFilter::Entropy, false);
    });
}

#[bench]
fn filters_bigrams(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from("tests/files/rgb_8_should_be_rgb_8.png"));
    let png = PngData::new(&input, false).unwrap();

    b.iter(|| {
        png.raw.filter_image(RowFilter::Bigrams, false);
    });
}

#[bench]
fn filters_bigent(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from("tests/files/rgb_8_should_be_rgb_8.png"));
    let png = PngData::new(&input, false).unwrap();

    b.iter(|| {
        png.raw.filter_image(RowFilter::BigEnt, false);
    });
}

#[bench]
fn filters_brute(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from("tests/files/rgb_8_should_be_rgb_8.png"));
    let png = PngData::new(&input, false).unwrap();

    b.iter(|| {
        png.raw.filter_image(RowFilter::Brute, false);
    });
}
