extern crate oxipng;

use oxipng::OutFile;
use oxipng::Headers;
use std::default::Default;
use std::fs;
use std::fs::File;
use std::io::prelude::*;

#[test]
fn optimize_from_memory() {
    let mut in_file = File::open("tests/files/fully_optimized.png").unwrap();
    let mut in_file_buf: Vec<u8> = Vec::new();
    in_file.read_to_end(&mut in_file_buf).unwrap();

    let mut opts: oxipng::Options = Default::default();
    opts.verbosity = Some(1);

    let result = oxipng::optimize_from_memory(&in_file_buf, &opts);
    assert!(result.is_ok());
}

#[test]
fn optimize_from_memory_corrupted() {
    let mut in_file = File::open("tests/files/corrupted_header.png").unwrap();
    let mut in_file_buf: Vec<u8> = Vec::new();
    in_file.read_to_end(&mut in_file_buf).unwrap();

    let mut opts: oxipng::Options = Default::default();
    opts.verbosity = Some(1);

    let result = oxipng::optimize_from_memory(&in_file_buf, &opts);
    assert!(result.is_err());
}

#[test]
fn optimize_from_memory_apng() {
    let mut in_file = File::open("tests/files/apng_file.png").unwrap();
    let mut in_file_buf: Vec<u8> = Vec::new();
    in_file.read_to_end(&mut in_file_buf).unwrap();

    let mut opts: oxipng::Options = Default::default();
    opts.verbosity = Some(1);

    let result = oxipng::optimize_from_memory(&in_file_buf, &opts);
    assert!(result.is_err());
}

#[test]
fn optimize() {
    let mut opts: oxipng::Options = Default::default();
    opts.verbosity = Some(1);

    let result = oxipng::optimize(
        &"tests/files/fully_optimized.png".into(),
        &OutFile::Path(None),
        &opts,
    );
    assert!(result.is_ok());
}

#[test]
fn optimize_corrupted() {
    let mut opts: oxipng::Options = Default::default();
    opts.verbosity = Some(1);

    let result = oxipng::optimize(
        &"tests/files/corrupted_header.png".into(),
        &OutFile::Path(None),
        &opts,
    );
    assert!(result.is_err());
}

#[test]
fn optimize_apng() {
    let mut opts: oxipng::Options = Default::default();
    opts.verbosity = Some(1);

    let result = oxipng::optimize(
        &"tests/files/apng_file.png".into(),
        &OutFile::Path(None),
        &opts,
    );
    assert!(result.is_err());
}

#[test]
fn optimize_srgb_icc() {
    let file = fs::read("tests/files/badsrgb.png").unwrap();
    let mut opts: oxipng::Options = Default::default();

    let result = oxipng::optimize_from_memory(&file, &opts);
    assert!(result.unwrap().len() > 1000);

    opts.strip = Headers::Safe;
    let result = oxipng::optimize_from_memory(&file, &opts);
    assert!(result.unwrap().len() < 1000);
}
