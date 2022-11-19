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
    interlace: u8,
    color_type_in: ColorType,
    bit_depth_in: BitDepth,
    color_type_out: ColorType,
    bit_depth_out: BitDepth,
) {
    let input = PathBuf::from(input);
    let (output, mut opts) = get_opts(&input);
    let png = PngData::new(&input, opts.fix_errors).unwrap();
    opts.interlace = Some(interlace);
    assert_eq!(png.raw.ihdr.color_type, color_type_in);
    assert_eq!(png.raw.ihdr.bit_depth, bit_depth_in);
    assert_eq!(png.raw.ihdr.interlaced, if interlace == 1 { 0 } else { 1 });

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

    remove_file(output).ok();
}

#[test]
fn deinterlace_rgb_16() {
    test_it_converts(
        "tests/files/interlaced_rgb_16_should_be_rgb_16.png",
        0,
        ColorType::RGB,
        BitDepth::Sixteen,
        ColorType::RGB,
        BitDepth::Sixteen,
    );
}

#[test]
fn deinterlace_rgb_8() {
    test_it_converts(
        "tests/files/interlaced_rgb_8_should_be_rgb_8.png",
        0,
        ColorType::RGB,
        BitDepth::Eight,
        ColorType::RGB,
        BitDepth::Eight,
    );
}

#[test]
fn deinterlace_palette_8() {
    test_it_converts(
        "tests/files/interlaced_palette_8_should_be_palette_8.png",
        0,
        ColorType::Indexed,
        BitDepth::Eight,
        ColorType::Indexed,
        BitDepth::Eight,
    );
}

#[test]
fn deinterlace_palette_4() {
    test_it_converts(
        "tests/files/interlaced_palette_4_should_be_palette_4.png",
        0,
        ColorType::Indexed,
        BitDepth::Four,
        ColorType::Indexed,
        BitDepth::Four,
    );
}

#[test]
fn deinterlace_palette_2() {
    test_it_converts(
        "tests/files/interlaced_palette_2_should_be_palette_2.png",
        0,
        ColorType::Indexed,
        BitDepth::Two,
        ColorType::Indexed,
        BitDepth::Two,
    );
}

#[test]
fn deinterlace_palette_1() {
    test_it_converts(
        "tests/files/interlaced_palette_1_should_be_palette_1.png",
        0,
        ColorType::Indexed,
        BitDepth::One,
        ColorType::Indexed,
        BitDepth::One,
    );
}

#[test]
fn interlace_rgb_16() {
    test_it_converts(
        "tests/files/rgb_16_should_be_rgb_16.png",
        1,
        ColorType::RGB,
        BitDepth::Sixteen,
        ColorType::RGB,
        BitDepth::Sixteen,
    );
}

#[test]
fn interlace_rgb_8() {
    test_it_converts(
        "tests/files/rgb_8_should_be_rgb_8.png",
        1,
        ColorType::RGB,
        BitDepth::Eight,
        ColorType::RGB,
        BitDepth::Eight,
    );
}

#[test]
fn interlace_palette_8() {
    test_it_converts(
        "tests/files/palette_8_should_be_palette_8.png",
        1,
        ColorType::Indexed,
        BitDepth::Eight,
        ColorType::Indexed,
        BitDepth::Eight,
    );
}

#[test]
fn interlace_palette_4() {
    test_it_converts(
        "tests/files/palette_4_should_be_palette_4.png",
        1,
        ColorType::Indexed,
        BitDepth::Four,
        ColorType::Indexed,
        BitDepth::Four,
    );
}

#[test]
fn interlace_palette_2() {
    test_it_converts(
        "tests/files/palette_2_should_be_palette_2.png",
        1,
        ColorType::Indexed,
        BitDepth::Two,
        ColorType::Indexed,
        BitDepth::Two,
    );
}

#[test]
fn interlace_palette_1() {
    test_it_converts(
        "tests/files/palette_1_should_be_palette_1.png",
        1,
        ColorType::Indexed,
        BitDepth::One,
        ColorType::Indexed,
        BitDepth::One,
    );
}
