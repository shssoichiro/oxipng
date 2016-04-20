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
    compression.insert(9);
    let mut memory = HashSet::new();
    memory.insert(9);
    let strategies = HashSet::new();

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
fn bench_deflate_16_bits_strategy_0(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from("tests/files/rgb_16_should_be_rgb_16.png"));
    let mut opts = get_opts(&input);
    opts.strategies.insert(0);

    b.iter(|| {
        oxipng::optimize(&input, &opts).ok();
    });
}

#[bench]
fn bench_deflate_8_bits_strategy_0(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from("tests/files/rgb_8_should_be_rgb_8.png"));
    let mut opts = get_opts(&input);
    opts.strategies.insert(0);

    b.iter(|| {
        oxipng::optimize(&input, &opts).ok();
    });
}

#[bench]
fn bench_deflate_4_bits_strategy_0(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from("tests/files/palette_4_should_be_palette_4.png"));
    let mut opts = get_opts(&input);
    opts.strategies.insert(0);

    b.iter(|| {
        oxipng::optimize(&input, &opts).ok();
    });
}

#[bench]
fn bench_deflate_2_bits_strategy_0(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from("tests/files/palette_2_should_be_palette_2.png"));
    let mut opts = get_opts(&input);
    opts.strategies.insert(0);

    b.iter(|| {
        oxipng::optimize(&input, &opts).ok();
    });
}

#[bench]
fn bench_deflate_1_bits_strategy_0(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from("tests/files/palette_1_should_be_palette_1.png"));
    let mut opts = get_opts(&input);
    opts.strategies.insert(0);

    b.iter(|| {
        oxipng::optimize(&input, &opts).ok();
    });
}

#[bench]
fn bench_deflate_16_bits_strategy_1(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from("tests/files/rgb_16_should_be_rgb_16.png"));
    let mut opts = get_opts(&input);
    opts.strategies.insert(1);

    b.iter(|| {
        oxipng::optimize(&input, &opts).ok();
    });
}

#[bench]
fn bench_deflate_8_bits_strategy_1(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from("tests/files/rgb_8_should_be_rgb_8.png"));
    let mut opts = get_opts(&input);
    opts.strategies.insert(1);

    b.iter(|| {
        oxipng::optimize(&input, &opts).ok();
    });
}

#[bench]
fn bench_deflate_4_bits_strategy_1(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from("tests/files/palette_4_should_be_palette_4.png"));
    let mut opts = get_opts(&input);
    opts.strategies.insert(1);

    b.iter(|| {
        oxipng::optimize(&input, &opts).ok();
    });
}

#[bench]
fn bench_deflate_2_bits_strategy_1(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from("tests/files/palette_2_should_be_palette_2.png"));
    let mut opts = get_opts(&input);
    opts.strategies.insert(1);

    b.iter(|| {
        oxipng::optimize(&input, &opts).ok();
    });
}

#[bench]
fn bench_deflate_1_bits_strategy_1(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from("tests/files/palette_1_should_be_palette_1.png"));
    let mut opts = get_opts(&input);
    opts.strategies.insert(1);

    b.iter(|| {
        oxipng::optimize(&input, &opts).ok();
    });
}

#[bench]
fn bench_deflate_16_bits_strategy_2(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from("tests/files/rgb_16_should_be_rgb_16.png"));
    let mut opts = get_opts(&input);
    opts.strategies.insert(2);

    b.iter(|| {
        oxipng::optimize(&input, &opts).ok();
    });
}

#[bench]
fn bench_deflate_8_bits_strategy_2(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from("tests/files/rgb_8_should_be_rgb_8.png"));
    let mut opts = get_opts(&input);
    opts.strategies.insert(2);

    b.iter(|| {
        oxipng::optimize(&input, &opts).ok();
    });
}

#[bench]
fn bench_deflate_4_bits_strategy_2(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from("tests/files/palette_4_should_be_palette_4.png"));
    let mut opts = get_opts(&input);
    opts.strategies.insert(2);

    b.iter(|| {
        oxipng::optimize(&input, &opts).ok();
    });
}

#[bench]
fn bench_deflate_2_bits_strategy_2(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from("tests/files/palette_2_should_be_palette_2.png"));
    let mut opts = get_opts(&input);
    opts.strategies.insert(2);

    b.iter(|| {
        oxipng::optimize(&input, &opts).ok();
    });
}

#[bench]
fn bench_deflate_1_bits_strategy_2(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from("tests/files/palette_1_should_be_palette_1.png"));
    let mut opts = get_opts(&input);
    opts.strategies.insert(2);

    b.iter(|| {
        oxipng::optimize(&input, &opts).ok();
    });
}

#[bench]
fn bench_deflate_16_bits_strategy_3(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from("tests/files/rgb_16_should_be_rgb_16.png"));
    let mut opts = get_opts(&input);
    opts.strategies.insert(3);

    b.iter(|| {
        oxipng::optimize(&input, &opts).ok();
    });
}

#[bench]
fn bench_deflate_8_bits_strategy_3(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from("tests/files/rgb_8_should_be_rgb_8.png"));
    let mut opts = get_opts(&input);
    opts.strategies.insert(3);

    b.iter(|| {
        oxipng::optimize(&input, &opts).ok();
    });
}

#[bench]
fn bench_deflate_4_bits_strategy_3(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from("tests/files/palette_4_should_be_palette_4.png"));
    let mut opts = get_opts(&input);
    opts.strategies.insert(3);

    b.iter(|| {
        oxipng::optimize(&input, &opts).ok();
    });
}

#[bench]
fn bench_deflate_2_bits_strategy_3(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from("tests/files/palette_2_should_be_palette_2.png"));
    let mut opts = get_opts(&input);
    opts.strategies.insert(3);

    b.iter(|| {
        oxipng::optimize(&input, &opts).ok();
    });
}

#[bench]
fn bench_deflate_1_bits_strategy_3(b: &mut Bencher) {
    let input = test::black_box(PathBuf::from("tests/files/palette_1_should_be_palette_1.png"));
    let mut opts = get_opts(&input);
    opts.strategies.insert(3);

    b.iter(|| {
        oxipng::optimize(&input, &opts).ok();
    });
}
