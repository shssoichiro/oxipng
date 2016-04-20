#![feature(test)]

extern crate test;
extern crate oxipng;

use oxipng::png;
use test::Bencher;
use std::collections::HashSet;
use std::path::Path;
use std::path::PathBuf;

fn get_opts(input: &Path) -> oxipng::Options {
    let mut filter = HashSet::new();
    filter.insert(0);
    let mut compression = HashSet::new();
    compression.insert(0);
    let mut memory = HashSet::new();
    memory.insert(9);
    let mut strategies = HashSet::new();
    strategies.insert(0);

    oxipng::Options {
        backup: false,
        out_file: input.with_extension("out.png").to_owned(),
        out_dir: None,
        stdout: false,
        pretend: true,
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
        bit_depth_reduction: false,
        color_type_reduction: false,
        palette_reduction: false,
        idat_recoding: true,
        strip: png::Headers::None,
        use_heuristics: false,
    }
}

#[bench]
fn bench_16_to_8_bits(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from("tests/files/rgb_16_should_be_rgb_8.png"));
    let mut opts = get_opts(&input);
    opts.bit_depth_reduction = true;

    b.iter(|| {
        oxipng::optimize(&input, &opts).ok();
    });
}

#[bench]
fn bench_8_to_4_bits(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from("tests/files/palette_8_should_be_palette_4.png"));
    let mut opts = get_opts(&input);
    opts.bit_depth_reduction = true;

    b.iter(|| {
        oxipng::optimize(&input, &opts).ok();
    });
}

#[bench]
fn bench_8_to_2_bits(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from("tests/files/palette_8_should_be_palette_2.png"));
    let mut opts = get_opts(&input);
    opts.bit_depth_reduction = true;

    b.iter(|| {
        oxipng::optimize(&input, &opts).ok();
    });
}

#[bench]
fn bench_8_to_1_bits(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from("tests/files/palette_8_should_be_palette_1.png"));
    let mut opts = get_opts(&input);
    opts.bit_depth_reduction = true;

    b.iter(|| {
        oxipng::optimize(&input, &opts).ok();
    });
}

#[bench]
fn bench_4_to_2_bits(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from("tests/files/palette_4_should_be_palette_2.png"));
    let mut opts = get_opts(&input);
    opts.bit_depth_reduction = true;

    b.iter(|| {
        oxipng::optimize(&input, &opts).ok();
    });
}

#[bench]
fn bench_4_to_1_bits(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from("tests/files/palette_4_should_be_palette_1.png"));
    let mut opts = get_opts(&input);
    opts.bit_depth_reduction = true;

    b.iter(|| {
        oxipng::optimize(&input, &opts).ok();
    });
}

#[bench]
fn bench_2_to_1_bits(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from("tests/files/palette_2_should_be_palette_1.png"));
    let mut opts = get_opts(&input);
    opts.bit_depth_reduction = true;

    b.iter(|| {
        oxipng::optimize(&input, &opts).ok();
    });
}

#[bench]
fn bench_rgba_to_rgb_16(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from("tests/files/rgba_16_should_be_rgb_16.png"));
    let mut opts = get_opts(&input);
    opts.color_type_reduction = true;

    b.iter(|| {
        oxipng::optimize(&input, &opts).ok();
    });
}

#[bench]
fn bench_rgba_to_rgb_8(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from("tests/files/rgba_8_should_be_rgb_8.png"));
    let mut opts = get_opts(&input);
    opts.color_type_reduction = true;

    b.iter(|| {
        oxipng::optimize(&input, &opts).ok();
    });
}

#[bench]
fn bench_rgba_to_grayscale_alpha_16(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from("tests/files/rgba_16_should_be_grayscale_alpha_16.png"));
    let mut opts = get_opts(&input);
    opts.color_type_reduction = true;

    b.iter(|| {
        oxipng::optimize(&input, &opts).ok();
    });
}

#[bench]
fn bench_rgba_to_grayscale_alpha_8(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from("tests/files/rgba_8_should_be_grayscale_alpha_8.png"));
    let mut opts = get_opts(&input);
    opts.color_type_reduction = true;

    b.iter(|| {
        oxipng::optimize(&input, &opts).ok();
    });
}

#[bench]
fn bench_rgba_to_grayscale_16(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from("tests/files/rgba_16_should_be_grayscale_16.png"));
    let mut opts = get_opts(&input);
    opts.color_type_reduction = true;

    b.iter(|| {
        oxipng::optimize(&input, &opts).ok();
    });
}

#[bench]
fn bench_rgba_to_grayscale_8(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from("tests/files/rgba_8_should_be_grayscale_8.png"));
    let mut opts = get_opts(&input);
    opts.color_type_reduction = true;

    b.iter(|| {
        oxipng::optimize(&input, &opts).ok();
    });
}

#[bench]
fn bench_rgb_to_grayscale_16(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from("tests/files/rgb_16_should_be_grayscale_16.png"));
    let mut opts = get_opts(&input);
    opts.color_type_reduction = true;

    b.iter(|| {
        oxipng::optimize(&input, &opts).ok();
    });
}

#[bench]
fn bench_rgb_to_grayscale_8(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from("tests/files/rgb_8_should_be_grayscale_8.png"));
    let mut opts = get_opts(&input);
    opts.color_type_reduction = true;

    b.iter(|| {
        oxipng::optimize(&input, &opts).ok();
    });
}

#[bench]
fn bench_rgba_to_palette_8(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from("tests/files/rgba_8_should_be_palette_8.png"));
    let mut opts = get_opts(&input);
    opts.color_type_reduction = true;

    b.iter(|| {
        oxipng::optimize(&input, &opts).ok();
    });
}

#[bench]
fn bench_rgb_to_palette_8(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from("tests/files/rgb_8_should_be_palette_8.png"));
    let mut opts = get_opts(&input);
    opts.color_type_reduction = true;

    b.iter(|| {
        oxipng::optimize(&input, &opts).ok();
    });
}

#[bench]
fn bench_palette_duplicate_reduction(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from("tests/files/palette_should_be_reduced_with_dupes.png"));
    let mut opts = get_opts(&input);
    opts.palette_reduction = true;

    b.iter(|| {
        oxipng::optimize(&input, &opts).ok();
    });
}

#[bench]
fn bench_palette_unused_reduction(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from("tests/files/palette_should_be_reduced_with_unused.png"));
    let mut opts = get_opts(&input);
    opts.palette_reduction = true;

    b.iter(|| {
        oxipng::optimize(&input, &opts).ok();
    });
}

#[bench]
fn bench_palette_full_reduction(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from("tests/files/palette_should_be_reduced_with_both.png"));
    let mut opts = get_opts(&input);
    opts.palette_reduction = true;

    b.iter(|| {
        oxipng::optimize(&input, &opts).ok();
    });
}
