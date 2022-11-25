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
fn filter_0_for_rgba_16() {
    test_it_converts(
        "tests/files/filter_0_for_rgba_16.png",
        RowFilter::None,
        ColorType::RGBA,
        BitDepth::Sixteen,
        ColorType::RGBA,
        BitDepth::Sixteen,
    );
}

#[test]
fn filter_1_for_rgba_16() {
    test_it_converts(
        "tests/files/filter_1_for_rgba_16.png",
        RowFilter::Sub,
        ColorType::RGBA,
        BitDepth::Sixteen,
        ColorType::RGBA,
        BitDepth::Sixteen,
    );
}

#[test]
fn filter_2_for_rgba_16() {
    test_it_converts(
        "tests/files/filter_2_for_rgba_16.png",
        RowFilter::Up,
        ColorType::RGBA,
        BitDepth::Sixteen,
        ColorType::RGBA,
        BitDepth::Sixteen,
    );
}

#[test]
fn filter_3_for_rgba_16() {
    test_it_converts(
        "tests/files/filter_3_for_rgba_16.png",
        RowFilter::Average,
        ColorType::RGBA,
        BitDepth::Sixteen,
        ColorType::RGBA,
        BitDepth::Sixteen,
    );
}

#[test]
fn filter_4_for_rgba_16() {
    test_it_converts(
        "tests/files/filter_4_for_rgba_16.png",
        RowFilter::Paeth,
        ColorType::RGBA,
        BitDepth::Sixteen,
        ColorType::RGBA,
        BitDepth::Sixteen,
    );
}

#[test]
fn filter_5_for_rgba_16() {
    test_it_converts(
        "tests/files/filter_5_for_rgba_16.png",
        RowFilter::MinSum,
        ColorType::RGBA,
        BitDepth::Sixteen,
        ColorType::RGBA,
        BitDepth::Sixteen,
    );
}

#[test]
fn filter_0_for_rgba_8() {
    test_it_converts(
        "tests/files/filter_0_for_rgba_8.png",
        RowFilter::None,
        ColorType::RGBA,
        BitDepth::Eight,
        ColorType::RGBA,
        BitDepth::Eight,
    );
}

#[test]
fn filter_1_for_rgba_8() {
    test_it_converts(
        "tests/files/filter_1_for_rgba_8.png",
        RowFilter::Sub,
        ColorType::RGBA,
        BitDepth::Eight,
        ColorType::RGBA,
        BitDepth::Eight,
    );
}

#[test]
fn filter_2_for_rgba_8() {
    test_it_converts(
        "tests/files/filter_2_for_rgba_8.png",
        RowFilter::Up,
        ColorType::RGBA,
        BitDepth::Eight,
        ColorType::RGBA,
        BitDepth::Eight,
    );
}

#[test]
fn filter_3_for_rgba_8() {
    test_it_converts(
        "tests/files/filter_3_for_rgba_8.png",
        RowFilter::Average,
        ColorType::RGBA,
        BitDepth::Eight,
        ColorType::RGBA,
        BitDepth::Eight,
    );
}

#[test]
fn filter_4_for_rgba_8() {
    test_it_converts(
        "tests/files/filter_4_for_rgba_8.png",
        RowFilter::Paeth,
        ColorType::RGBA,
        BitDepth::Eight,
        ColorType::RGBA,
        BitDepth::Eight,
    );
}

#[test]
fn filter_5_for_rgba_8() {
    test_it_converts(
        "tests/files/filter_5_for_rgba_8.png",
        RowFilter::MinSum,
        ColorType::RGBA,
        BitDepth::Eight,
        ColorType::RGBA,
        BitDepth::Eight,
    );
}

#[test]
fn filter_0_for_rgb_16() {
    test_it_converts(
        "tests/files/filter_0_for_rgb_16.png",
        RowFilter::None,
        ColorType::RGB,
        BitDepth::Sixteen,
        ColorType::RGB,
        BitDepth::Sixteen,
    );
}

#[test]
fn filter_1_for_rgb_16() {
    test_it_converts(
        "tests/files/filter_1_for_rgb_16.png",
        RowFilter::Sub,
        ColorType::RGB,
        BitDepth::Sixteen,
        ColorType::RGB,
        BitDepth::Sixteen,
    );
}

#[test]
fn filter_2_for_rgb_16() {
    test_it_converts(
        "tests/files/filter_2_for_rgb_16.png",
        RowFilter::Up,
        ColorType::RGB,
        BitDepth::Sixteen,
        ColorType::RGB,
        BitDepth::Sixteen,
    );
}

#[test]
fn filter_3_for_rgb_16() {
    test_it_converts(
        "tests/files/filter_3_for_rgb_16.png",
        RowFilter::Average,
        ColorType::RGB,
        BitDepth::Sixteen,
        ColorType::RGB,
        BitDepth::Sixteen,
    );
}

#[test]
fn filter_4_for_rgb_16() {
    test_it_converts(
        "tests/files/filter_4_for_rgb_16.png",
        RowFilter::Paeth,
        ColorType::RGB,
        BitDepth::Sixteen,
        ColorType::RGB,
        BitDepth::Sixteen,
    );
}

#[test]
fn filter_5_for_rgb_16() {
    test_it_converts(
        "tests/files/filter_5_for_rgb_16.png",
        RowFilter::MinSum,
        ColorType::RGB,
        BitDepth::Sixteen,
        ColorType::RGB,
        BitDepth::Sixteen,
    );
}

#[test]
fn filter_0_for_rgb_8() {
    test_it_converts(
        "tests/files/filter_0_for_rgb_8.png",
        RowFilter::None,
        ColorType::RGB,
        BitDepth::Eight,
        ColorType::RGB,
        BitDepth::Eight,
    );
}

#[test]
fn filter_1_for_rgb_8() {
    test_it_converts(
        "tests/files/filter_1_for_rgb_8.png",
        RowFilter::Sub,
        ColorType::RGB,
        BitDepth::Eight,
        ColorType::RGB,
        BitDepth::Eight,
    );
}

#[test]
fn filter_2_for_rgb_8() {
    test_it_converts(
        "tests/files/filter_2_for_rgb_8.png",
        RowFilter::Up,
        ColorType::RGB,
        BitDepth::Eight,
        ColorType::RGB,
        BitDepth::Eight,
    );
}

#[test]
fn filter_3_for_rgb_8() {
    test_it_converts(
        "tests/files/filter_3_for_rgb_8.png",
        RowFilter::Average,
        ColorType::RGB,
        BitDepth::Eight,
        ColorType::RGB,
        BitDepth::Eight,
    );
}

#[test]
fn filter_4_for_rgb_8() {
    test_it_converts(
        "tests/files/filter_4_for_rgb_8.png",
        RowFilter::Paeth,
        ColorType::RGB,
        BitDepth::Eight,
        ColorType::RGB,
        BitDepth::Eight,
    );
}

#[test]
fn filter_5_for_rgb_8() {
    test_it_converts(
        "tests/files/filter_5_for_rgb_8.png",
        RowFilter::MinSum,
        ColorType::RGB,
        BitDepth::Eight,
        ColorType::RGB,
        BitDepth::Eight,
    );
}

#[test]
fn filter_0_for_grayscale_alpha_16() {
    test_it_converts(
        "tests/files/filter_0_for_grayscale_alpha_16.png",
        RowFilter::None,
        ColorType::GrayscaleAlpha,
        BitDepth::Sixteen,
        ColorType::GrayscaleAlpha,
        BitDepth::Sixteen,
    );
}

#[test]
fn filter_1_for_grayscale_alpha_16() {
    test_it_converts(
        "tests/files/filter_1_for_grayscale_alpha_16.png",
        RowFilter::Sub,
        ColorType::GrayscaleAlpha,
        BitDepth::Sixteen,
        ColorType::GrayscaleAlpha,
        BitDepth::Sixteen,
    );
}

#[test]
fn filter_2_for_grayscale_alpha_16() {
    test_it_converts(
        "tests/files/filter_2_for_grayscale_alpha_16.png",
        RowFilter::Up,
        ColorType::GrayscaleAlpha,
        BitDepth::Sixteen,
        ColorType::GrayscaleAlpha,
        BitDepth::Sixteen,
    );
}

#[test]
fn filter_3_for_grayscale_alpha_16() {
    test_it_converts(
        "tests/files/filter_3_for_grayscale_alpha_16.png",
        RowFilter::Average,
        ColorType::GrayscaleAlpha,
        BitDepth::Sixteen,
        ColorType::GrayscaleAlpha,
        BitDepth::Sixteen,
    );
}

#[test]
fn filter_4_for_grayscale_alpha_16() {
    test_it_converts(
        "tests/files/filter_4_for_grayscale_alpha_16.png",
        RowFilter::Paeth,
        ColorType::GrayscaleAlpha,
        BitDepth::Sixteen,
        ColorType::GrayscaleAlpha,
        BitDepth::Sixteen,
    );
}

#[test]
fn filter_5_for_grayscale_alpha_16() {
    test_it_converts(
        "tests/files/filter_5_for_grayscale_alpha_16.png",
        RowFilter::MinSum,
        ColorType::GrayscaleAlpha,
        BitDepth::Sixteen,
        ColorType::GrayscaleAlpha,
        BitDepth::Sixteen,
    );
}

#[test]
fn filter_0_for_grayscale_alpha_8() {
    test_it_converts(
        "tests/files/filter_0_for_grayscale_alpha_8.png",
        RowFilter::None,
        ColorType::GrayscaleAlpha,
        BitDepth::Eight,
        ColorType::GrayscaleAlpha,
        BitDepth::Eight,
    );
}

#[test]
fn filter_1_for_grayscale_alpha_8() {
    test_it_converts(
        "tests/files/filter_1_for_grayscale_alpha_8.png",
        RowFilter::Sub,
        ColorType::GrayscaleAlpha,
        BitDepth::Eight,
        ColorType::GrayscaleAlpha,
        BitDepth::Eight,
    );
}

#[test]
fn filter_2_for_grayscale_alpha_8() {
    test_it_converts(
        "tests/files/filter_2_for_grayscale_alpha_8.png",
        RowFilter::Up,
        ColorType::GrayscaleAlpha,
        BitDepth::Eight,
        ColorType::GrayscaleAlpha,
        BitDepth::Eight,
    );
}

#[test]
fn filter_3_for_grayscale_alpha_8() {
    test_it_converts(
        "tests/files/filter_3_for_grayscale_alpha_8.png",
        RowFilter::Average,
        ColorType::GrayscaleAlpha,
        BitDepth::Eight,
        ColorType::GrayscaleAlpha,
        BitDepth::Eight,
    );
}

#[test]
fn filter_4_for_grayscale_alpha_8() {
    test_it_converts(
        "tests/files/filter_4_for_grayscale_alpha_8.png",
        RowFilter::Paeth,
        ColorType::GrayscaleAlpha,
        BitDepth::Eight,
        ColorType::GrayscaleAlpha,
        BitDepth::Eight,
    );
}

#[test]
fn filter_5_for_grayscale_alpha_8() {
    test_it_converts(
        "tests/files/filter_5_for_grayscale_alpha_8.png",
        RowFilter::MinSum,
        ColorType::GrayscaleAlpha,
        BitDepth::Eight,
        ColorType::GrayscaleAlpha,
        BitDepth::Eight,
    );
}

#[test]
fn filter_0_for_grayscale_16() {
    test_it_converts(
        "tests/files/filter_0_for_grayscale_16.png",
        RowFilter::None,
        ColorType::Grayscale,
        BitDepth::Sixteen,
        ColorType::Grayscale,
        BitDepth::Sixteen,
    );
}

#[test]
fn filter_1_for_grayscale_16() {
    test_it_converts(
        "tests/files/filter_1_for_grayscale_16.png",
        RowFilter::Sub,
        ColorType::Grayscale,
        BitDepth::Sixteen,
        ColorType::Grayscale,
        BitDepth::Sixteen,
    );
}

#[test]
fn filter_2_for_grayscale_16() {
    test_it_converts(
        "tests/files/filter_2_for_grayscale_16.png",
        RowFilter::Up,
        ColorType::Grayscale,
        BitDepth::Sixteen,
        ColorType::Grayscale,
        BitDepth::Sixteen,
    );
}

#[test]
fn filter_3_for_grayscale_16() {
    test_it_converts(
        "tests/files/filter_3_for_grayscale_16.png",
        RowFilter::Average,
        ColorType::Grayscale,
        BitDepth::Sixteen,
        ColorType::Grayscale,
        BitDepth::Sixteen,
    );
}

#[test]
fn filter_4_for_grayscale_16() {
    test_it_converts(
        "tests/files/filter_4_for_grayscale_16.png",
        RowFilter::Paeth,
        ColorType::Grayscale,
        BitDepth::Sixteen,
        ColorType::Grayscale,
        BitDepth::Sixteen,
    );
}

#[test]
fn filter_5_for_grayscale_16() {
    test_it_converts(
        "tests/files/filter_5_for_grayscale_16.png",
        RowFilter::MinSum,
        ColorType::Grayscale,
        BitDepth::Sixteen,
        ColorType::Grayscale,
        BitDepth::Sixteen,
    );
}

#[test]
fn filter_0_for_grayscale_8() {
    test_it_converts(
        "tests/files/filter_0_for_grayscale_8.png",
        RowFilter::None,
        ColorType::Grayscale,
        BitDepth::Eight,
        ColorType::Grayscale,
        BitDepth::Eight,
    );
}

#[test]
fn filter_1_for_grayscale_8() {
    test_it_converts(
        "tests/files/filter_1_for_grayscale_8.png",
        RowFilter::Sub,
        ColorType::Grayscale,
        BitDepth::Eight,
        ColorType::Grayscale,
        BitDepth::Eight,
    );
}

#[test]
fn filter_2_for_grayscale_8() {
    test_it_converts(
        "tests/files/filter_2_for_grayscale_8.png",
        RowFilter::Up,
        ColorType::Grayscale,
        BitDepth::Eight,
        ColorType::Grayscale,
        BitDepth::Eight,
    );
}

#[test]
fn filter_3_for_grayscale_8() {
    test_it_converts(
        "tests/files/filter_3_for_grayscale_8.png",
        RowFilter::Average,
        ColorType::Grayscale,
        BitDepth::Eight,
        ColorType::Grayscale,
        BitDepth::Eight,
    );
}

#[test]
fn filter_4_for_grayscale_8() {
    test_it_converts(
        "tests/files/filter_4_for_grayscale_8.png",
        RowFilter::Paeth,
        ColorType::Grayscale,
        BitDepth::Eight,
        ColorType::Grayscale,
        BitDepth::Eight,
    );
}

#[test]
fn filter_5_for_grayscale_8() {
    test_it_converts(
        "tests/files/filter_5_for_grayscale_8.png",
        RowFilter::MinSum,
        ColorType::Grayscale,
        BitDepth::Eight,
        ColorType::Grayscale,
        BitDepth::Eight,
    );
}

#[test]
fn filter_0_for_palette_4() {
    test_it_converts(
        "tests/files/filter_0_for_palette_4.png",
        RowFilter::None,
        ColorType::Indexed,
        BitDepth::Four,
        ColorType::Indexed,
        BitDepth::Four,
    );
}

#[test]
fn filter_1_for_palette_4() {
    test_it_converts(
        "tests/files/filter_1_for_palette_4.png",
        RowFilter::Sub,
        ColorType::Indexed,
        BitDepth::Four,
        ColorType::Indexed,
        BitDepth::Four,
    );
}

#[test]
fn filter_2_for_palette_4() {
    test_it_converts(
        "tests/files/filter_2_for_palette_4.png",
        RowFilter::Up,
        ColorType::Indexed,
        BitDepth::Four,
        ColorType::Indexed,
        BitDepth::Four,
    );
}

#[test]
fn filter_3_for_palette_4() {
    test_it_converts(
        "tests/files/filter_3_for_palette_4.png",
        RowFilter::Average,
        ColorType::Indexed,
        BitDepth::Four,
        ColorType::Indexed,
        BitDepth::Four,
    );
}

#[test]
fn filter_4_for_palette_4() {
    test_it_converts(
        "tests/files/filter_4_for_palette_4.png",
        RowFilter::Paeth,
        ColorType::Indexed,
        BitDepth::Four,
        ColorType::Indexed,
        BitDepth::Four,
    );
}

#[test]
fn filter_5_for_palette_4() {
    test_it_converts(
        "tests/files/filter_5_for_palette_4.png",
        RowFilter::MinSum,
        ColorType::Indexed,
        BitDepth::Four,
        ColorType::Indexed,
        BitDepth::Four,
    );
}

#[test]
fn filter_0_for_palette_2() {
    test_it_converts(
        "tests/files/filter_0_for_palette_2.png",
        RowFilter::None,
        ColorType::Indexed,
        BitDepth::Two,
        ColorType::Indexed,
        BitDepth::Two,
    );
}

#[test]
fn filter_1_for_palette_2() {
    test_it_converts(
        "tests/files/filter_1_for_palette_2.png",
        RowFilter::Sub,
        ColorType::Indexed,
        BitDepth::Two,
        ColorType::Indexed,
        BitDepth::Two,
    );
}

#[test]
fn filter_2_for_palette_2() {
    test_it_converts(
        "tests/files/filter_2_for_palette_2.png",
        RowFilter::Up,
        ColorType::Indexed,
        BitDepth::Two,
        ColorType::Indexed,
        BitDepth::Two,
    );
}

#[test]
fn filter_3_for_palette_2() {
    test_it_converts(
        "tests/files/filter_3_for_palette_2.png",
        RowFilter::Average,
        ColorType::Indexed,
        BitDepth::Two,
        ColorType::Indexed,
        BitDepth::Two,
    );
}

#[test]
fn filter_4_for_palette_2() {
    test_it_converts(
        "tests/files/filter_4_for_palette_2.png",
        RowFilter::Paeth,
        ColorType::Indexed,
        BitDepth::Two,
        ColorType::Indexed,
        BitDepth::Two,
    );
}

#[test]
fn filter_5_for_palette_2() {
    test_it_converts(
        "tests/files/filter_5_for_palette_2.png",
        RowFilter::MinSum,
        ColorType::Indexed,
        BitDepth::Two,
        ColorType::Indexed,
        BitDepth::Two,
    );
}

#[test]
fn filter_0_for_palette_1() {
    test_it_converts(
        "tests/files/filter_0_for_palette_1.png",
        RowFilter::None,
        ColorType::Indexed,
        BitDepth::One,
        ColorType::Indexed,
        BitDepth::One,
    );
}

#[test]
fn filter_1_for_palette_1() {
    test_it_converts(
        "tests/files/filter_1_for_palette_1.png",
        RowFilter::Sub,
        ColorType::Indexed,
        BitDepth::One,
        ColorType::Indexed,
        BitDepth::One,
    );
}

#[test]
fn filter_2_for_palette_1() {
    test_it_converts(
        "tests/files/filter_2_for_palette_1.png",
        RowFilter::Up,
        ColorType::Indexed,
        BitDepth::One,
        ColorType::Indexed,
        BitDepth::One,
    );
}

#[test]
fn filter_3_for_palette_1() {
    test_it_converts(
        "tests/files/filter_3_for_palette_1.png",
        RowFilter::Average,
        ColorType::Indexed,
        BitDepth::One,
        ColorType::Indexed,
        BitDepth::One,
    );
}

#[test]
fn filter_4_for_palette_1() {
    test_it_converts(
        "tests/files/filter_4_for_palette_1.png",
        RowFilter::Paeth,
        ColorType::Indexed,
        BitDepth::One,
        ColorType::Indexed,
        BitDepth::One,
    );
}

#[test]
fn filter_5_for_palette_1() {
    test_it_converts(
        "tests/files/filter_5_for_palette_1.png",
        RowFilter::MinSum,
        ColorType::Indexed,
        BitDepth::One,
        ColorType::Indexed,
        BitDepth::One,
    );
}
