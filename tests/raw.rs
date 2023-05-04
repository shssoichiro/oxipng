use oxipng::internal_tests::*;
use oxipng::*;
use std::path::PathBuf;
use std::sync::Arc;

fn get_opts() -> Options {
    Options {
        force: true,
        filter: indexset! { RowFilter::None },
        ..Default::default()
    }
}

fn test_it_converts(input: &str) {
    let input = PathBuf::from(input);
    let opts = get_opts();

    let original_data = PngData::read_file(&PathBuf::from(input)).unwrap();
    let png = PngData::from_slice(&original_data, opts.fix_errors).unwrap();
    let png = Arc::try_unwrap(png.raw).unwrap();

    let num_headers = png.aux_headers.len();
    assert!(num_headers > 0);

    let mut raw = RawImage::new(
        png.ihdr.width,
        png.ihdr.height,
        png.ihdr.color_type,
        png.ihdr.bit_depth,
        png.data,
    )
    .unwrap();

    for (chunk_type, data) in png.aux_headers {
        raw.add_png_header(chunk_type, data);
    }

    let output = raw.create_optimized_png(&opts).unwrap();

    let new = PngData::from_slice(&output, opts.fix_errors).unwrap();
    assert!(new.raw.aux_headers.len() == num_headers);

    #[cfg(feature = "sanity-checks")]
    assert!(validate_output(&output, &original_data));
}

#[test]
fn from_file() {
    test_it_converts("tests/files/raw_api.png");
}

#[test]
fn custom_indexed() {
    let opts = get_opts();

    let raw = RawImage::new(
        4,
        4,
        ColorType::Indexed {
            palette: vec![
                RGBA8::new(255, 255, 255, 255),
                RGBA8::new(255, 0, 0, 255),
                RGBA8::new(0, 255, 0, 255),
                RGBA8::new(0, 0, 255, 255),
            ],
        },
        BitDepth::Eight,
        vec![0, 0, 1, 1, 0, 0, 1, 1, 2, 2, 3, 3, 2, 2, 3, 3],
    )
    .unwrap();

    raw.create_optimized_png(&opts).unwrap();
}

#[test]
fn invalid_depth() {
    RawImage::new(
        2,
        2,
        ColorType::RGBA,
        BitDepth::Four,
        vec![0, 0, 1, 1, 0, 0, 1, 1, 2, 2, 3, 3, 2, 2, 3, 3],
    )
    .expect_err("Expected invalid depth for color type");
}

#[test]
fn incorrect_length() {
    RawImage::new(
        2,
        2,
        ColorType::RGBA,
        BitDepth::Eight,
        vec![0, 0, 1, 1, 0, 0, 1, 1],
    )
    .expect_err("Expected incorrect data length");
}
