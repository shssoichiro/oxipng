extern crate oxipng;

use oxipng::OutFile;
use std::default::Default;
use std::fs::File;
use std::path::Path;
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

    let result = oxipng::optimize(Path::new("tests/files/fully_optimized.png"), &OutFile::Path(None), &opts);
    assert!(result.is_ok());
}

#[test]
fn optimize_corrupted() {
    let mut opts: oxipng::Options = Default::default();
    opts.verbosity = Some(1);

    let result = oxipng::optimize(Path::new("tests/files/corrupted_header.png"), &OutFile::Path(None), &opts);
    assert!(result.is_err());
}

#[test]
fn optimize_apng() {
    let mut opts: oxipng::Options = Default::default();
    opts.verbosity = Some(1);

    let result = oxipng::optimize(Path::new("tests/files/apng_file.png"), &OutFile::Path(None), &opts);
    assert!(result.is_err());
}
