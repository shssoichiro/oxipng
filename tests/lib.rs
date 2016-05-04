extern crate image;
extern crate oxipng;

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
fn optimize() {
    let mut opts: oxipng::Options = Default::default();
    opts.verbosity = Some(1);

    let result = oxipng::optimize(Path::new("tests/files/fully_optimized.png"), &opts);
    assert!(result.is_ok());
}

#[test]
fn optimize_corrupted() {
    let mut opts: oxipng::Options = Default::default();
    opts.verbosity = Some(1);

    let result = oxipng::optimize(Path::new("tests/files/corrupted_header.png"), &opts);
    assert!(result.is_err());
}
