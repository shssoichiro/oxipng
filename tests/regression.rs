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
    custom: Option<(OutFile, oxipng::Options)>,
    color_type_in: ColorType,
    bit_depth_in: BitDepth,
    color_type_out: ColorType,
    bit_depth_out: BitDepth,
) {
    let input = PathBuf::from(input);
    let (output, opts) = custom.unwrap_or_else(|| get_opts(&input));
    let png = PngData::new(&input, opts.fix_errors).unwrap();

    assert_eq!(
        png.raw.ihdr.color_type, color_type_in,
        "test file is broken"
    );
    assert_eq!(png.raw.ihdr.bit_depth, bit_depth_in, "test file is broken");

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

    assert_eq!(
        png.raw.ihdr.color_type, color_type_out,
        "optimized to wrong color type"
    );
    assert_eq!(
        png.raw.ihdr.bit_depth, bit_depth_out,
        "optimized to wrong bit depth"
    );
    if let Some(palette) = png.raw.palette.as_ref() {
        let mut max_palette_size = 1 << (png.raw.ihdr.bit_depth.as_u8() as usize);
        // Ensure bKGD color is valid
        if let Some(&idx) = png.raw.aux_headers.get(b"bKGD").and_then(|b| b.first()) {
            assert!(palette.len() > idx as usize);
            max_palette_size = max_palette_size.max(idx as usize + 1);
        }
        assert!(palette.len() <= max_palette_size);
    } else {
        assert_ne!(png.raw.ihdr.color_type, ColorType::Indexed);
    }

    remove_file(output).ok();
}

#[test]
fn issue_29() {
    test_it_converts(
        "tests/files/issue-29.png",
        None,
        ColorType::RGB,
        BitDepth::Eight,
        ColorType::RGB,
        BitDepth::Eight,
    );
}

#[test]
fn issue_42() {
    let input = PathBuf::from("tests/files/issue_42.png");
    let (output, mut opts) = get_opts(&input);
    opts.interlace = Some(1);

    let png = PngData::new(&input, opts.fix_errors).unwrap();

    assert_eq!(png.raw.ihdr.interlaced, 0);
    assert_eq!(png.raw.ihdr.color_type, ColorType::GrayscaleAlpha);
    assert_eq!(png.raw.ihdr.bit_depth, BitDepth::Eight);

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

    assert_eq!(png.raw.ihdr.interlaced, 1);
    assert_eq!(png.raw.ihdr.color_type, ColorType::GrayscaleAlpha);
    assert_eq!(png.raw.ihdr.bit_depth, BitDepth::Eight);

    remove_file(output).ok();
}

#[test]
fn issue_52_01() {
    test_it_converts(
        "tests/files/issue-52-01.png",
        None,
        ColorType::RGBA,
        BitDepth::Eight,
        ColorType::Indexed,
        BitDepth::Eight,
    );
}

#[test]
fn issue_52_02() {
    test_it_converts(
        "tests/files/issue-52-02.png",
        None,
        ColorType::RGBA,
        BitDepth::Eight,
        ColorType::Indexed,
        BitDepth::Eight,
    );
}

#[test]
fn issue_52_03() {
    test_it_converts(
        "tests/files/issue-52-03.png",
        None,
        ColorType::RGBA,
        BitDepth::Eight,
        ColorType::Indexed,
        BitDepth::Eight,
    );
}

#[test]
fn issue_52_04() {
    test_it_converts(
        "tests/files/issue-52-04.png",
        None,
        ColorType::RGBA,
        BitDepth::Eight,
        ColorType::Indexed,
        BitDepth::One,
    );
}

#[test]
fn issue_52_05() {
    test_it_converts(
        "tests/files/issue-52-05.png",
        None,
        ColorType::RGBA,
        BitDepth::Eight,
        ColorType::Indexed,
        BitDepth::One,
    );
}

#[test]
fn issue_52_06() {
    test_it_converts(
        "tests/files/issue-52-06.png",
        None,
        ColorType::RGBA,
        BitDepth::Eight,
        ColorType::Indexed,
        BitDepth::Two,
    );
}

#[test]
fn issue_56() {
    test_it_converts(
        "tests/files/issue-56.png",
        None,
        ColorType::Indexed,
        BitDepth::Four,
        ColorType::Indexed,
        BitDepth::Four,
    );
}

#[test]
fn issue_58() {
    test_it_converts(
        "tests/files/issue-58.png",
        None,
        ColorType::Indexed,
        BitDepth::Four,
        ColorType::Indexed,
        BitDepth::Four,
    );
}

#[test]
fn issue_59() {
    test_it_converts(
        "tests/files/issue-59.png",
        None,
        ColorType::RGBA,
        BitDepth::Eight,
        ColorType::RGBA,
        BitDepth::Eight,
    );
}

#[test]
fn issue_60() {
    test_it_converts(
        "tests/files/issue-60.png",
        None,
        ColorType::RGBA,
        BitDepth::Eight,
        ColorType::GrayscaleAlpha,
        BitDepth::Eight,
    );
}

#[test]
fn issue_80() {
    test_it_converts(
        "tests/files/issue-80.png",
        None,
        ColorType::Indexed,
        BitDepth::Two,
        ColorType::Indexed,
        BitDepth::One,
    );
}

#[test]
fn issue_82() {
    test_it_converts(
        "tests/files/issue-82.png",
        None,
        ColorType::Indexed,
        BitDepth::Four,
        ColorType::Indexed,
        BitDepth::Four,
    );
}

#[test]
fn issue_89() {
    test_it_converts(
        "tests/files/issue-89.png",
        None,
        ColorType::RGBA,
        BitDepth::Eight,
        ColorType::Grayscale,
        BitDepth::Eight,
    );
}

#[test]
fn issue_92_filter_0() {
    test_it_converts(
        "tests/files/issue-92.png",
        None,
        ColorType::Grayscale,
        BitDepth::Eight,
        ColorType::Grayscale,
        BitDepth::Eight,
    );
}

#[test]
fn issue_92_filter_5() {
    let input = "tests/files/issue-92.png";
    let (_, mut opts) = get_opts(Path::new(input));
    opts.filter = [RowFilter::MinSum].iter().cloned().collect();
    let output = OutFile::Path(Some(Path::new(input).with_extension("-f5-out.png")));

    test_it_converts(
        input,
        Some((output, opts)),
        ColorType::Grayscale,
        BitDepth::Eight,
        ColorType::Grayscale,
        BitDepth::Eight,
    );
}

#[test]
fn issue_113() {
    let input = "tests/files/issue-113.png";
    let (output, mut opts) = get_opts(Path::new(input));
    opts.interlace = Some(1);
    opts.optimize_alpha = true;
    test_it_converts(
        input,
        Some((output, opts)),
        ColorType::RGBA,
        BitDepth::Eight,
        ColorType::GrayscaleAlpha,
        BitDepth::Eight,
    );
}

#[test]
fn issue_129() {
    let input = "tests/files/issue-129.png";
    test_it_converts(
        input,
        None,
        ColorType::RGB,
        BitDepth::Eight,
        ColorType::Indexed,
        BitDepth::Eight,
    );
}

#[test]
fn issue_133() {
    let input = "tests/files/issue-133.png";
    let (output, mut opts) = get_opts(Path::new(input));
    opts.optimize_alpha = true;
    test_it_converts(
        input,
        Some((output, opts)),
        ColorType::RGBA,
        BitDepth::Eight,
        ColorType::RGBA,
        BitDepth::Eight,
    );
}

#[test]
fn issue_140() {
    test_it_converts(
        "tests/files/issue-140.png",
        None,
        ColorType::Grayscale,
        BitDepth::Two,
        ColorType::Grayscale,
        BitDepth::Two,
    );
}

#[test]
fn issue_141() {
    test_it_converts(
        "tests/files/issue-141.png",
        None,
        ColorType::RGBA,
        BitDepth::Eight,
        ColorType::RGB,
        BitDepth::Eight,
    );
}

#[test]
fn issue_153() {
    test_it_converts(
        "tests/files/issue-153.png",
        None,
        ColorType::RGBA,
        BitDepth::Eight,
        ColorType::Indexed,
        BitDepth::Eight,
    );
}

#[test]
fn issue_159() {
    test_it_converts(
        "tests/files/issue-159.png",
        None,
        ColorType::Indexed,
        BitDepth::One,
        ColorType::Indexed,
        BitDepth::One,
    );
}

#[test]
#[cfg(target_pointer_width = "64")]
fn issue_167() {
    test_it_converts(
        "tests/files/issue-167.png",
        None,
        ColorType::Grayscale,
        BitDepth::Eight,
        ColorType::Grayscale,
        BitDepth::Eight,
    );
}

#[test]
fn issue_171() {
    test_it_converts(
        "tests/files/issue-171.png",
        None,
        ColorType::Grayscale,
        BitDepth::Eight,
        ColorType::Grayscale,
        BitDepth::Eight,
    );
}

#[test]
fn issue_175() {
    test_it_converts(
        "tests/files/issue-175.png",
        None,
        ColorType::Grayscale,
        BitDepth::One,
        ColorType::Grayscale,
        BitDepth::One,
    );
}

#[test]
fn issue_182() {
    let input = "tests/files/issue-182.png";
    let (output, mut opts) = get_opts(Path::new(input));
    opts.interlace = Some(0);

    test_it_converts(
        input,
        Some((output, opts)),
        ColorType::Grayscale,
        BitDepth::One,
        ColorType::Grayscale,
        BitDepth::One,
    );
}

#[test]
fn issue_195() {
    test_it_converts(
        "tests/files/issue-195.png",
        None,
        ColorType::RGBA,
        BitDepth::Eight,
        ColorType::Indexed,
        BitDepth::Eight,
    );
}

#[test]
fn issue_426_01() {
    test_it_converts(
        "tests/files/issue-426-01.png",
        None,
        ColorType::Grayscale,
        BitDepth::Eight,
        ColorType::Grayscale,
        BitDepth::One,
    );
}

#[test]
fn issue_426_02() {
    test_it_converts(
        "tests/files/issue-426-02.png",
        None,
        ColorType::Grayscale,
        BitDepth::Eight,
        ColorType::Grayscale,
        BitDepth::One,
    );
}
