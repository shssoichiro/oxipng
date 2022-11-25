use indexmap::IndexSet;
use oxipng::{internal_tests::*, RowFilter};
use oxipng::{InFile, OutFile};
use std::fs::remove_file;
use std::path::Path;
use std::path::PathBuf;

fn get_opts(input: &Path) -> (OutFile, oxipng::Options) {
    let mut options = oxipng::Options {
        force: true,
        ..Default::default()
    };
    let mut filter = IndexSet::new();
    filter.insert(RowFilter::None);
    options.filter = filter;

    (
        OutFile::Path(Some(input.with_extension("out.png"))),
        options,
    )
}

fn test_it_converts(
    input: &str,
    filter: RowFilter,
    color_type_in: ColorType,
    bit_depth_in: BitDepth,
    color_type_out: ColorType,
    bit_depth_out: BitDepth,
) {
    let input = PathBuf::from(input);

    let (output, mut opts) = get_opts(&input);
    let png = PngData::new(&input, opts.fix_errors).unwrap();
    opts.filter = IndexSet::new();
    opts.filter.insert(filter);
    assert_eq!(png.raw.ihdr.color_type, color_type_in);
    assert_eq!(png.raw.ihdr.bit_depth, bit_depth_in);

    match oxipng::optimize(&InFile::Path(input), &output, &opts) {
        Ok(_) => (),
        Err(x) => panic!("{}", x),
    };
    let output = output.path().unwrap();
    assert!(output.exists());

    let png = match PngData::new(output, opts.fix_errors) {
        Ok(x) => x,
        Err(x) => {
            remove_file(output).ok();
            panic!("{}", x)
        }
    };

    assert_eq!(png.raw.ihdr.color_type, color_type_out);
    assert_eq!(png.raw.ihdr.bit_depth, bit_depth_out);
    if let Some(palette) = png.raw.palette.as_ref() {
        assert!(palette.len() <= 1 << (png.raw.ihdr.bit_depth.as_u8() as usize));
    } else {
        assert_ne!(png.raw.ihdr.color_type, ColorType::Indexed);
    }

    remove_file(output).ok();
}

#[test]
fn filter_minsum() {
    test_it_converts(
        "tests/files/rgb_16_should_be_rgb_16.png",
        RowFilter::MinSum,
        ColorType::RGB,
        BitDepth::Sixteen,
        ColorType::RGB,
        BitDepth::Sixteen,
    );
}

#[test]
fn filter_entropy() {
    test_it_converts(
        "tests/files/rgb_8_should_be_rgb_8.png",
        RowFilter::Entropy,
        ColorType::RGB,
        BitDepth::Eight,
        ColorType::RGB,
        BitDepth::Eight,
    );
}

#[test]
fn filter_bigrams() {
    test_it_converts(
        "tests/files/rgba_8_should_be_rgba_8.png",
        RowFilter::Bigrams,
        ColorType::RGBA,
        BitDepth::Eight,
        ColorType::RGBA,
        BitDepth::Eight,
    );
}

#[test]
fn filter_bigent() {
    test_it_converts(
        "tests/files/grayscale_8_should_be_grayscale_8.png",
        RowFilter::BigEnt,
        ColorType::Grayscale,
        BitDepth::Eight,
        ColorType::Grayscale,
        BitDepth::Eight,
    );
}

#[test]
fn filter_brute() {
    test_it_converts(
        "tests/files/palette_8_should_be_palette_8.png",
        RowFilter::Brute,
        ColorType::Indexed,
        BitDepth::Eight,
        ColorType::Indexed,
        BitDepth::Eight,
    );
}
