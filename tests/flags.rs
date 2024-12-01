#[cfg(feature = "filetime")]
use std::cell::RefCell;
#[cfg(feature = "zopfli")]
use std::num::NonZeroU8;
#[cfg(feature = "filetime")]
use std::ops::Deref;
use std::{
    fs::remove_file,
    path::{Path, PathBuf},
};

use indexmap::indexset;
use oxipng::{internal_tests::*, *};

const RGB: u8 = 2;
const INDEXED: u8 = 3;

fn get_opts(input: &Path) -> (OutFile, oxipng::Options) {
    let mut options = oxipng::Options {
        force: true,
        fast_evaluation: false,
        ..Default::default()
    };
    let mut filter = IndexSet::new();
    filter.insert(RowFilter::None);
    options.filter = filter;

    (OutFile::from_path(input.with_extension("out.png")), options)
}

/// Add callback to allow checks before the output file is deleted again
#[allow(clippy::too_many_arguments)]
fn test_it_converts_callbacks<CBPRE, CBPOST>(
    input: PathBuf,
    output: &OutFile,
    opts: &oxipng::Options,
    color_type_in: u8,
    bit_depth_in: BitDepth,
    color_type_out: u8,
    bit_depth_out: BitDepth,
    mut callback_pre: CBPRE,
    mut callback_post: CBPOST,
) where
    CBPOST: FnMut(&Path),
    CBPRE: FnMut(&Path),
{
    let png = PngData::new(&input, opts).unwrap();

    assert_eq!(png.raw.ihdr.color_type.png_header_code(), color_type_in);
    assert_eq!(png.raw.ihdr.bit_depth, bit_depth_in);

    callback_pre(&input);

    match oxipng::optimize(&InFile::Path(input), output, opts) {
        Ok(_) => (),
        Err(x) => panic!("{}", x),
    };
    let output = output.path().unwrap();
    assert!(output.exists());

    callback_post(output);

    let png = match PngData::new(output, opts) {
        Ok(x) => x,
        Err(x) => {
            remove_file(output).ok();
            panic!("{}", x)
        }
    };

    assert_eq!(png.raw.ihdr.color_type.png_header_code(), color_type_out);
    assert_eq!(png.raw.ihdr.bit_depth, bit_depth_out);

    remove_file(output).ok();
}

/// Shim for new callback functionality
fn test_it_converts(
    input: PathBuf,
    output: &OutFile,
    opts: &oxipng::Options,
    color_type_in: u8,
    bit_depth_in: BitDepth,
    color_type_out: u8,
    bit_depth_out: BitDepth,
) {
    test_it_converts_callbacks(
        input,
        output,
        opts,
        color_type_in,
        bit_depth_in,
        color_type_out,
        bit_depth_out,
        |_| {},
        |_| {},
    )
}

#[test]
fn verbose_mode() {
    use std::cell::RefCell;
    #[cfg(not(feature = "parallel"))]
    use std::sync::mpsc::{channel as unbounded, Sender};

    #[cfg(feature = "parallel")]
    use crossbeam_channel::{unbounded, Sender};
    use log::{set_logger, set_max_level, Level, LevelFilter, Log, Metadata, Record};

    // Rust runs tests in parallel by default.
    // We want to make sure that we verify only logs from our test.
    //
    // For that, we store an Option in a thread-local variable and
    // initialise it with Some(sender) only on threads spawned within
    // our test.
    thread_local! {
        static VERBOSE_LOGS: RefCell<Option<Sender<String>>> = const { RefCell::new(None) };
    }

    struct LogTester;

    impl Log for LogTester {
        fn enabled(&self, metadata: &Metadata) -> bool {
            metadata.level() <= Level::Debug
        }

        fn log(&self, record: &Record) {
            if record.level() == Level::Debug {
                VERBOSE_LOGS.with(|logs| {
                    // If current thread has a storage for logs, add our line.
                    // Otherwise our handler is invoked from an unrelated test.
                    if let Some(logs) = logs.borrow().as_ref() {
                        logs.send(record.args().to_string()).unwrap();
                    }
                });
            }
        }

        fn flush(&self) {}
    }

    set_logger(&LogTester).unwrap();
    set_max_level(LevelFilter::Debug);

    let input = PathBuf::from("tests/files/verbose_mode.png");
    let (output, opts) = get_opts(&input);

    let (sender, receiver) = unbounded();

    let thread_init = move || {
        // Initialise logs storage for all threads within our test.
        VERBOSE_LOGS.with(|logs| *logs.borrow_mut() = Some(sender.clone()));
    };
    let thread_exec = move || {
        test_it_converts(
            input,
            &output,
            &opts,
            RGB,
            BitDepth::Eight,
            RGB,
            BitDepth::Eight,
        );
    };

    #[cfg(feature = "parallel")]
    rayon::ThreadPoolBuilder::new()
        .start_handler(move |_| thread_init())
        .num_threads(rayon::current_num_threads() + 1)
        .build()
        .unwrap()
        .install(move || rayon::spawn(thread_exec));

    #[cfg(not(feature = "parallel"))]
    std::thread::spawn(move || {
        thread_init();
        thread_exec();
    });

    let logs: Vec<_> = receiver.into_iter().collect();
    let expected_prefixes = [
        "    500x400 pixels, PNG format",
        "    8-bit RGB, non-interlaced",
        "    IDAT size = 113794 bytes",
        "    File size = 114708 bytes",
        "Trying: 1 filters",
        "Found better combination:",
        "    zc = 11  f = None ",
        "    IDAT size = ",
        "    file size = ",
    ];
    assert_eq!(logs.len(), expected_prefixes.len());
    for (i, log) in logs.into_iter().enumerate() {
        let expected_prefix = expected_prefixes[i];
        assert!(
            log.starts_with(expected_prefix),
            "logs[{}] = {:?} doesn't start with {:?}",
            i,
            log,
            expected_prefix
        );
    }
}

fn count_chunk(png: &PngData, name: &[u8; 4]) -> usize {
    png.aux_chunks
        .iter()
        .filter(|chunk| &chunk.name == name)
        .count()
}

#[test]
fn strip_headers_list() {
    let input = PathBuf::from("tests/files/strip_headers_list.png");
    let (output, mut opts) = get_opts(&input);
    opts.strip = StripChunks::Strip(indexset![*b"iCCP", *b"tEXt"]);

    let png = PngData::new(&input, &Options::default()).unwrap();

    assert_eq!(count_chunk(&png, b"tEXt"), 3);
    assert_eq!(count_chunk(&png, b"iTXt"), 1);
    assert_eq!(count_chunk(&png, b"iCCP"), 1);

    match oxipng::optimize(&InFile::Path(input), &output, &opts) {
        Ok(_) => (),
        Err(x) => panic!("{}", x),
    };
    let output = output.path().unwrap();
    assert!(output.exists());

    let png = match PngData::new(output, &opts) {
        Ok(x) => x,
        Err(x) => {
            remove_file(output).ok();
            panic!("{}", x)
        }
    };

    assert_eq!(count_chunk(&png, b"tEXt"), 0);
    assert_eq!(count_chunk(&png, b"iTXt"), 1);
    assert_eq!(count_chunk(&png, b"iCCP"), 0);

    remove_file(output).ok();
}

#[test]
fn strip_headers_safe() {
    let input = PathBuf::from("tests/files/strip_headers_safe.png");
    let (output, mut opts) = get_opts(&input);
    opts.strip = StripChunks::Safe;

    let png = PngData::new(&input, &Options::default()).unwrap();

    assert_eq!(count_chunk(&png, b"tEXt"), 3);
    assert_eq!(count_chunk(&png, b"iTXt"), 1);
    assert_eq!(count_chunk(&png, b"iCCP"), 1);

    match oxipng::optimize(&InFile::Path(input), &output, &opts) {
        Ok(_) => (),
        Err(x) => panic!("{}", x),
    };
    let output = output.path().unwrap();
    assert!(output.exists());

    let png = match PngData::new(output, &opts) {
        Ok(x) => x,
        Err(x) => {
            remove_file(output).ok();
            panic!("{}", x)
        }
    };

    assert_eq!(count_chunk(&png, b"tEXt"), 0);
    assert_eq!(count_chunk(&png, b"iTXt"), 0);
    assert_eq!(count_chunk(&png, b"sRGB"), 1);

    remove_file(output).ok();
}

#[test]
fn strip_headers_all() {
    let input = PathBuf::from("tests/files/strip_headers_all.png");
    let (output, mut opts) = get_opts(&input);
    opts.strip = StripChunks::All;

    let png = PngData::new(&input, &Options::default()).unwrap();

    assert_eq!(count_chunk(&png, b"tEXt"), 3);
    assert_eq!(count_chunk(&png, b"iTXt"), 1);
    assert_eq!(count_chunk(&png, b"iCCP"), 1);

    match oxipng::optimize(&InFile::Path(input), &output, &opts) {
        Ok(_) => (),
        Err(x) => panic!("{}", x),
    };
    let output = output.path().unwrap();
    assert!(output.exists());

    let png = match PngData::new(output, &opts) {
        Ok(x) => x,
        Err(x) => {
            remove_file(output).ok();
            panic!("{}", x)
        }
    };

    assert_eq!(count_chunk(&png, b"tEXt"), 0);
    assert_eq!(count_chunk(&png, b"iTXt"), 0);
    assert_eq!(count_chunk(&png, b"iCCP"), 0);

    remove_file(output).ok();
}

#[test]
fn strip_headers_none() {
    let input = PathBuf::from("tests/files/strip_headers_none.png");
    let (output, mut opts) = get_opts(&input);
    opts.strip = StripChunks::None;

    let png = PngData::new(&input, &Options::default()).unwrap();

    assert_eq!(count_chunk(&png, b"tEXt"), 3);
    assert_eq!(count_chunk(&png, b"iTXt"), 1);
    assert_eq!(count_chunk(&png, b"iCCP"), 1);

    match oxipng::optimize(&InFile::Path(input), &output, &opts) {
        Ok(_) => (),
        Err(x) => panic!("{}", x),
    };
    let output = output.path().unwrap();
    assert!(output.exists());

    let png = match PngData::new(output, &opts) {
        Ok(x) => x,
        Err(x) => {
            remove_file(output).ok();
            panic!("{}", x)
        }
    };

    assert_eq!(count_chunk(&png, b"tEXt"), 3);
    assert_eq!(count_chunk(&png, b"iTXt"), 1);
    assert_eq!(count_chunk(&png, b"iCCP"), 1);

    remove_file(output).ok();
}

#[test]
fn interlacing_0_to_1() {
    let input = PathBuf::from("tests/files/interlacing_0_to_1.png");
    let (output, mut opts) = get_opts(&input);
    opts.interlace = Some(Interlacing::Adam7);

    let png = PngData::new(&input, &opts).unwrap();

    assert_eq!(png.raw.ihdr.interlaced, Interlacing::None);

    match oxipng::optimize(&InFile::Path(input), &output, &opts) {
        Ok(_) => (),
        Err(x) => panic!("{}", x),
    };
    let output = output.path().unwrap();
    assert!(output.exists());

    let png = match PngData::new(output, &opts) {
        Ok(x) => x,
        Err(x) => {
            remove_file(output).ok();
            panic!("{}", x)
        }
    };

    assert_eq!(png.raw.ihdr.interlaced, Interlacing::Adam7);

    remove_file(output).ok();
}

#[test]
fn interlacing_1_to_0() {
    let input = PathBuf::from("tests/files/interlacing_1_to_0.png");
    let (output, mut opts) = get_opts(&input);
    opts.interlace = Some(Interlacing::None);

    let png = PngData::new(&input, &opts).unwrap();

    assert_eq!(png.raw.ihdr.interlaced, Interlacing::Adam7);

    match oxipng::optimize(&InFile::Path(input), &output, &opts) {
        Ok(_) => (),
        Err(x) => panic!("{}", x),
    };
    let output = output.path().unwrap();
    assert!(output.exists());

    let png = match PngData::new(output, &opts) {
        Ok(x) => x,
        Err(x) => {
            remove_file(output).ok();
            panic!("{}", x)
        }
    };

    assert_eq!(png.raw.ihdr.interlaced, Interlacing::None);

    remove_file(output).ok();
}

#[test]
fn interlacing_0_to_1_small_files() {
    let input = PathBuf::from("tests/files/interlacing_0_to_1_small_files.png");
    let (output, mut opts) = get_opts(&input);
    opts.interlace = Some(Interlacing::Adam7);

    let png = PngData::new(&input, &opts).unwrap();

    assert_eq!(png.raw.ihdr.interlaced, Interlacing::None);
    assert_eq!(png.raw.ihdr.color_type.png_header_code(), INDEXED);
    assert_eq!(png.raw.ihdr.bit_depth, BitDepth::Eight);

    match oxipng::optimize(&InFile::Path(input), &output, &opts) {
        Ok(_) => (),
        Err(x) => panic!("{}", x),
    };
    let output = output.path().unwrap();
    assert!(output.exists());

    let png = match PngData::new(output, &opts) {
        Ok(x) => x,
        Err(x) => {
            remove_file(output).ok();
            panic!("{}", x)
        }
    };

    assert_eq!(png.raw.ihdr.interlaced, Interlacing::Adam7);
    assert_eq!(png.raw.ihdr.color_type.png_header_code(), RGB);
    assert_eq!(png.raw.ihdr.bit_depth, BitDepth::Eight);

    remove_file(output).ok();
}

#[test]
fn interlacing_1_to_0_small_files() {
    let input = PathBuf::from("tests/files/interlacing_1_to_0_small_files.png");
    let (output, mut opts) = get_opts(&input);
    opts.interlace = Some(Interlacing::None);

    let png = PngData::new(&input, &opts).unwrap();

    assert_eq!(png.raw.ihdr.interlaced, Interlacing::Adam7);
    assert_eq!(png.raw.ihdr.color_type.png_header_code(), INDEXED);
    assert_eq!(png.raw.ihdr.bit_depth, BitDepth::Eight);

    match oxipng::optimize(&InFile::Path(input), &output, &opts) {
        Ok(_) => (),
        Err(x) => panic!("{}", x),
    };
    let output = output.path().unwrap();
    assert!(output.exists());

    let png = match PngData::new(output, &opts) {
        Ok(x) => x,
        Err(x) => {
            remove_file(output).ok();
            panic!("{}", x)
        }
    };

    assert_eq!(png.raw.ihdr.interlaced, Interlacing::None);
    assert_eq!(png.raw.ihdr.color_type.png_header_code(), RGB);
    assert_eq!(png.raw.ihdr.bit_depth, BitDepth::Eight);

    remove_file(output).ok();
}

#[test]
fn interlaced_0_to_1_other_filter_mode() {
    let input = PathBuf::from("tests/files/interlaced_0_to_1_other_filter_mode.png");
    let (output, mut opts) = get_opts(&input);
    opts.interlace = Some(Interlacing::Adam7);
    let mut filter = IndexSet::new();
    filter.insert(RowFilter::Paeth);
    opts.filter = filter;

    let png = PngData::new(&input, &opts).unwrap();

    assert_eq!(png.raw.ihdr.interlaced, Interlacing::None);

    match oxipng::optimize(&InFile::Path(input), &output, &opts) {
        Ok(_) => (),
        Err(x) => panic!("{}", x),
    };
    let output = output.path().unwrap();
    assert!(output.exists());

    let png = match PngData::new(output, &opts) {
        Ok(x) => x,
        Err(x) => {
            remove_file(output).ok();
            panic!("{}", x)
        }
    };

    assert_eq!(png.raw.ihdr.interlaced, Interlacing::Adam7);

    remove_file(output).ok();
}

#[test]
fn preserve_attrs() {
    let input = PathBuf::from("tests/files/preserve_attrs.png");

    #[cfg(feature = "filetime")]
    let atime_canon = RefCell::new(filetime::FileTime::from_unix_time(0, 0));
    #[cfg(feature = "filetime")]
    let mtime_canon = RefCell::new(filetime::FileTime::from_unix_time(0, 0));

    let (mut output, opts) = get_opts(&input);
    if let OutFile::Path { preserve_attrs, .. } = &mut output {
        *preserve_attrs = true;
    }

    #[cfg(feature = "filetime")]
    let callback_pre = |path_in: &Path| {
        let meta_input = path_in
            .metadata()
            .expect("unable to get file metadata for output file");

        atime_canon.replace(filetime::FileTime::from_last_access_time(&meta_input));
        mtime_canon.replace(filetime::FileTime::from_last_modification_time(&meta_input));
    };
    #[cfg(not(feature = "filetime"))]
    let callback_pre = |_: &Path| {};

    #[cfg(feature = "filetime")]
    let callback_post = |path_out: &Path| {
        let meta_output = path_out
            .metadata()
            .expect("unable to get file metadata for output file");

        let cellref_atime_canon = atime_canon.borrow();
        let cellref_mtime_canon = mtime_canon.borrow();
        let ref_atime_canon: &filetime::FileTime = cellref_atime_canon.deref();
        let ref_mtime_canon: &filetime::FileTime = cellref_mtime_canon.deref();

        assert_eq!(
            ref_atime_canon,
            &filetime::FileTime::from_last_access_time(&meta_output),
            "expected access time to be identical to that of input",
        );
        assert_eq!(
            ref_mtime_canon,
            &filetime::FileTime::from_last_modification_time(&meta_output),
            "expected modification time to be identical to that of input",
        );
    };
    #[cfg(not(feature = "filetime"))]
    let callback_post = |_: &Path| {};

    test_it_converts_callbacks(
        input,
        &output,
        &opts,
        RGB,
        BitDepth::Eight,
        RGB,
        BitDepth::Eight,
        callback_pre,
        callback_post,
    );

    // TODO: Actually check permissions
}

#[test]
fn fix_errors() {
    let input = PathBuf::from("tests/files/fix_errors.png");
    let (output, mut opts) = get_opts(&input);
    opts.fix_errors = true;

    let png = PngData::new(&input, &opts).unwrap();

    assert_eq!(png.raw.ihdr.color_type.png_header_code(), INDEXED);
    assert_eq!(png.raw.ihdr.bit_depth, BitDepth::Eight);

    match oxipng::optimize(&InFile::Path(input), &output, &opts) {
        Ok(_) => (),
        Err(x) => panic!("{}", x),
    };
    let output = output.path().unwrap();
    assert!(output.exists());

    let png = match PngData::new(output, &Options::default()) {
        Ok(x) => x,
        Err(x) => {
            remove_file(output).ok();
            panic!("{}", x)
        }
    };

    assert_eq!(png.raw.ihdr.color_type.png_header_code(), INDEXED);
    assert_eq!(png.raw.ihdr.bit_depth, BitDepth::Eight);

    // Cannot check if pixels are equal because image crate cannot read corrupt (input) PNGs
    remove_file(output).ok();
}

#[test]
fn no_color_type_change() {
    let input = PathBuf::from("tests/files/palette_8_should_be_rgb.png");
    let (output, mut opts) = get_opts(&input);
    opts.color_type_reduction = false;

    test_it_converts(
        input,
        &output,
        &opts,
        INDEXED,
        BitDepth::Eight,
        INDEXED,
        BitDepth::One,
    );
}

#[test]
fn no_grayscale_change() {
    let input = PathBuf::from("tests/files/rgb_8_should_be_grayscale_8.png");
    let (output, mut opts) = get_opts(&input);
    opts.grayscale_reduction = false;

    test_it_converts(
        input,
        &output,
        &opts,
        RGB,
        BitDepth::Eight,
        INDEXED,
        BitDepth::Eight,
    );
}

#[test]
fn no_bit_depth_change() {
    let input = PathBuf::from("tests/files/palette_4_should_be_palette_2.png");
    let (output, mut opts) = get_opts(&input);
    opts.bit_depth_reduction = false;

    test_it_converts(
        input,
        &output,
        &opts,
        INDEXED,
        BitDepth::Four,
        INDEXED,
        BitDepth::Four,
    );
}

#[test]
fn scale_16() {
    let input = PathBuf::from("tests/files/rgb_16_should_be_rgb_16.png");
    let (output, mut opts) = get_opts(&input);
    opts.scale_16 = true;

    test_it_converts(
        input,
        &output,
        &opts,
        RGB,
        BitDepth::Sixteen,
        RGB,
        BitDepth::Eight,
    );
}

#[test]
#[cfg(feature = "zopfli")]
fn zopfli_mode() {
    let input = PathBuf::from("tests/files/zopfli_mode.png");
    let (output, mut opts) = get_opts(&input);
    opts.deflate = Deflaters::Zopfli {
        iterations: NonZeroU8::new(15).unwrap(),
    };

    test_it_converts(
        input,
        &output,
        &opts,
        INDEXED,
        BitDepth::Eight,
        INDEXED,
        BitDepth::Eight,
    );
}
