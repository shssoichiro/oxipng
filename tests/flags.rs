use indexmap::IndexSet;
use oxipng::internal_tests::*;
use oxipng::{InFile, OutFile};
use std::fs::remove_file;
use std::path::Path;
use std::path::PathBuf;

fn get_opts(input: &Path) -> (OutFile, oxipng::Options) {
    let mut options = oxipng::Options::default();
    options.force = true;
    let mut filter = IndexSet::new();
    filter.insert(0);
    options.filter = filter;

    (
        OutFile::Path(Some(input.with_extension("out.png").to_owned())),
        options,
    )
}

fn test_it_converts(
    input: PathBuf,
    output: &OutFile,
    opts: &oxipng::Options,
    color_type_in: ColorType,
    bit_depth_in: BitDepth,
    color_type_out: ColorType,
    bit_depth_out: BitDepth,
) {
    let png = PngData::new(&input, opts.fix_errors).unwrap();

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

    remove_file(output).ok();
}

#[test]
fn verbose_mode() {
    use crossbeam_channel::{unbounded, Sender};
    use log::{set_logger, set_max_level, Level, LevelFilter, Log, Metadata, Record};
    use std::cell::RefCell;

    // Rust runs tests in parallel by default.
    // We want to make sure that we verify only logs from our test.
    //
    // For that, we store an Option in a thread-local variable and
    // initialise it with Some(sender) only on threads spawned within
    // our test.
    thread_local! {
        static VERBOSE_LOGS: RefCell<Option<Sender<String>>> = RefCell::new(None);
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
            ColorType::RGB,
            BitDepth::Eight,
            ColorType::RGB,
            BitDepth::Eight,
        );
    };

    #[cfg(feature = "rayon")]
    rayon::ThreadPoolBuilder::new()
        .start_handler(move |_| thread_init())
        .num_threads(rayon::current_num_threads() + 1)
        .build()
        .unwrap()
        .install(move || rayon::spawn(thread_exec));

    #[cfg(not(feature = "rayon"))]
    std::thread::spawn(move || {
        thread_init();
        thread_exec();
    });

    let mut logs: Vec<_> = receiver.into_iter().collect();
    assert_eq!(logs.len(), 4);
    logs.sort();
    for (i, log) in logs.into_iter().enumerate() {
        let expected_prefix = format!("    zc = 9  zs = {}  f = 0 ", i);
        assert!(
            log.starts_with(&expected_prefix),
            "logs[{}] = {:?} doesn't start with {:?}",
            i,
            log,
            expected_prefix
        );
    }
}

#[test]
fn strip_headers_list() {
    let input = PathBuf::from("tests/files/strip_headers_list.png");
    let (output, mut opts) = get_opts(&input);
    opts.strip = Headers::Strip(vec!["iCCP".to_owned(), "tEXt".to_owned()]);

    let png = PngData::new(&input, opts.fix_errors).unwrap();

    assert!(png.raw.aux_headers.contains_key(b"tEXt"));
    assert!(png.raw.aux_headers.contains_key(b"iTXt"));
    assert!(png.raw.aux_headers.contains_key(b"iCCP"));

    match oxipng::optimize(&InFile::Path(input), &output, &opts) {
        Ok(_) => (),
        Err(x) => panic!("{}", x),
    };
    let output = output.path().unwrap();
    assert!(output.exists());

    let png = match PngData::new(&output, opts.fix_errors) {
        Ok(x) => x,
        Err(x) => {
            remove_file(output).ok();
            panic!("{}", x)
        }
    };

    assert!(!png.raw.aux_headers.contains_key(b"tEXt"));
    assert!(png.raw.aux_headers.contains_key(b"iTXt"));
    assert!(!png.raw.aux_headers.contains_key(b"iCCP"));

    remove_file(output).ok();
}

#[test]
fn strip_headers_safe() {
    let input = PathBuf::from("tests/files/strip_headers_safe.png");
    let (output, mut opts) = get_opts(&input);
    opts.strip = Headers::Safe;

    let png = PngData::new(&input, opts.fix_errors).unwrap();

    assert!(png.raw.aux_headers.contains_key(b"tEXt"));
    assert!(png.raw.aux_headers.contains_key(b"iTXt"));
    assert!(png.raw.aux_headers.contains_key(b"iCCP"));

    match oxipng::optimize(&InFile::Path(input), &output, &opts) {
        Ok(_) => (),
        Err(x) => panic!("{}", x),
    };
    let output = output.path().unwrap();
    assert!(output.exists());

    let png = match PngData::new(&output, opts.fix_errors) {
        Ok(x) => x,
        Err(x) => {
            remove_file(output).ok();
            panic!("{}", x)
        }
    };

    assert!(!png.raw.aux_headers.contains_key(b"tEXt"));
    assert!(!png.raw.aux_headers.contains_key(b"iTXt"));
    assert!(png.raw.aux_headers.contains_key(b"sRGB"));

    remove_file(output).ok();
}

#[test]
fn strip_headers_all() {
    let input = PathBuf::from("tests/files/strip_headers_all.png");
    let (output, mut opts) = get_opts(&input);
    opts.strip = Headers::All;

    let png = PngData::new(&input, opts.fix_errors).unwrap();

    assert!(png.raw.aux_headers.contains_key(b"tEXt"));
    assert!(png.raw.aux_headers.contains_key(b"iTXt"));
    assert!(png.raw.aux_headers.contains_key(b"iCCP"));

    match oxipng::optimize(&InFile::Path(input), &output, &opts) {
        Ok(_) => (),
        Err(x) => panic!("{}", x),
    };
    let output = output.path().unwrap();
    assert!(output.exists());

    let png = match PngData::new(&output, opts.fix_errors) {
        Ok(x) => x,
        Err(x) => {
            remove_file(output).ok();
            panic!("{}", x)
        }
    };

    assert!(!png.raw.aux_headers.contains_key(b"tEXt"));
    assert!(!png.raw.aux_headers.contains_key(b"iTXt"));
    assert!(!png.raw.aux_headers.contains_key(b"iCCP"));

    remove_file(output).ok();
}

#[test]
fn strip_headers_none() {
    let input = PathBuf::from("tests/files/strip_headers_none.png");
    let (output, mut opts) = get_opts(&input);
    opts.strip = Headers::None;

    let png = PngData::new(&input, opts.fix_errors).unwrap();

    assert!(png.raw.aux_headers.contains_key(b"tEXt"));
    assert!(png.raw.aux_headers.contains_key(b"iTXt"));
    assert!(png.raw.aux_headers.contains_key(b"iCCP"));

    match oxipng::optimize(&InFile::Path(input), &output, &opts) {
        Ok(_) => (),
        Err(x) => panic!("{}", x),
    };
    let output = output.path().unwrap();
    assert!(output.exists());

    let png = match PngData::new(&output, opts.fix_errors) {
        Ok(x) => x,
        Err(x) => {
            remove_file(output).ok();
            panic!("{}", x)
        }
    };

    assert!(png.raw.aux_headers.contains_key(b"tEXt"));
    assert!(png.raw.aux_headers.contains_key(b"iTXt"));
    assert!(png.raw.aux_headers.contains_key(b"iCCP"));

    remove_file(output).ok();
}

#[test]
fn interlacing_0_to_1() {
    let input = PathBuf::from("tests/files/interlacing_0_to_1.png");
    let (output, mut opts) = get_opts(&input);
    opts.interlace = Some(1);

    let png = PngData::new(&input, opts.fix_errors).unwrap();

    assert_eq!(png.raw.ihdr.interlaced, 0);

    match oxipng::optimize(&InFile::Path(input), &output, &opts) {
        Ok(_) => (),
        Err(x) => panic!("{}", x),
    };
    let output = output.path().unwrap();
    assert!(output.exists());

    let png = match PngData::new(&output, opts.fix_errors) {
        Ok(x) => x,
        Err(x) => {
            remove_file(output).ok();
            panic!("{}", x)
        }
    };

    assert_eq!(png.raw.ihdr.interlaced, 1);

    remove_file(output).ok();
}

#[test]
fn interlacing_1_to_0() {
    let input = PathBuf::from("tests/files/interlacing_1_to_0.png");
    let (output, mut opts) = get_opts(&input);
    opts.interlace = Some(0);

    let png = PngData::new(&input, opts.fix_errors).unwrap();

    assert_eq!(png.raw.ihdr.interlaced, 1);

    match oxipng::optimize(&InFile::Path(input), &output, &opts) {
        Ok(_) => (),
        Err(x) => panic!("{}", x),
    };
    let output = output.path().unwrap();
    assert!(output.exists());

    let png = match PngData::new(&output, opts.fix_errors) {
        Ok(x) => x,
        Err(x) => {
            remove_file(output).ok();
            panic!("{}", x)
        }
    };

    assert_eq!(png.raw.ihdr.interlaced, 0);

    remove_file(output).ok();
}

#[test]
fn interlacing_0_to_1_small_files() {
    let input = PathBuf::from("tests/files/interlacing_0_to_1_small_files.png");
    let (output, mut opts) = get_opts(&input);
    opts.interlace = Some(1);

    let png = PngData::new(&input, opts.fix_errors).unwrap();

    assert_eq!(png.raw.ihdr.interlaced, 0);
    assert_eq!(png.raw.ihdr.color_type, ColorType::Indexed);
    assert_eq!(png.raw.ihdr.bit_depth, BitDepth::Eight);

    match oxipng::optimize(&InFile::Path(input), &output, &opts) {
        Ok(_) => (),
        Err(x) => panic!("{}", x),
    };
    let output = output.path().unwrap();
    assert!(output.exists());

    let png = match PngData::new(&output, opts.fix_errors) {
        Ok(x) => x,
        Err(x) => {
            remove_file(output).ok();
            panic!("{}", x)
        }
    };

    assert_eq!(png.raw.ihdr.interlaced, 1);
    assert_eq!(png.raw.ihdr.color_type, ColorType::Indexed);
    assert_eq!(png.raw.ihdr.bit_depth, BitDepth::One);

    remove_file(output).ok();
}

#[test]
fn interlacing_1_to_0_small_files() {
    let input = PathBuf::from("tests/files/interlacing_1_to_0_small_files.png");
    let (output, mut opts) = get_opts(&input);
    opts.interlace = Some(0);

    let png = PngData::new(&input, opts.fix_errors).unwrap();

    assert_eq!(png.raw.ihdr.interlaced, 1);
    assert_eq!(png.raw.ihdr.color_type, ColorType::Indexed);
    assert_eq!(png.raw.ihdr.bit_depth, BitDepth::Eight);

    match oxipng::optimize(&InFile::Path(input), &output, &opts) {
        Ok(_) => (),
        Err(x) => panic!("{}", x),
    };
    let output = output.path().unwrap();
    assert!(output.exists());

    let png = match PngData::new(&output, opts.fix_errors) {
        Ok(x) => x,
        Err(x) => {
            remove_file(output).ok();
            panic!("{}", x)
        }
    };

    assert_eq!(png.raw.ihdr.interlaced, 0);
    assert_eq!(png.raw.ihdr.color_type, ColorType::Indexed);
    // the depth can't be asserted reliably, because on such small file different zlib implementations pick different depth as the best

    remove_file(output).ok();
}

#[test]
fn interlaced_0_to_1_other_filter_mode() {
    let input = PathBuf::from("tests/files/interlaced_0_to_1_other_filter_mode.png");
    let (output, mut opts) = get_opts(&input);
    opts.interlace = Some(1);
    let mut filter = IndexSet::new();
    filter.insert(4);
    opts.filter = filter;

    let png = PngData::new(&input, opts.fix_errors).unwrap();

    assert_eq!(png.raw.ihdr.interlaced, 0);

    match oxipng::optimize(&InFile::Path(input), &output, &opts) {
        Ok(_) => (),
        Err(x) => panic!("{}", x),
    };
    let output = output.path().unwrap();
    assert!(output.exists());

    let png = match PngData::new(&output, opts.fix_errors) {
        Ok(x) => x,
        Err(x) => {
            remove_file(output).ok();
            panic!("{}", x)
        }
    };

    assert_eq!(png.raw.ihdr.interlaced, 1);

    remove_file(output).ok();
}

#[test]
fn preserve_attrs() {
    let input = PathBuf::from("tests/files/preserve_attrs.png");
    let (output, mut opts) = get_opts(&input);
    opts.preserve_attrs = true;

    test_it_converts(
        input,
        &output,
        &opts,
        ColorType::RGB,
        BitDepth::Eight,
        ColorType::RGB,
        BitDepth::Eight,
    );

    // TODO: Actually check permissions
}

#[test]
fn fix_errors() {
    let input = PathBuf::from("tests/files/fix_errors.png");
    let (output, mut opts) = get_opts(&input);
    opts.fix_errors = true;

    let png = PngData::new(&input, opts.fix_errors).unwrap();

    assert_eq!(png.raw.ihdr.color_type, ColorType::RGBA);
    assert_eq!(png.raw.ihdr.bit_depth, BitDepth::Eight);

    match oxipng::optimize(&InFile::Path(input), &output, &opts) {
        Ok(_) => (),
        Err(x) => panic!("{}", x),
    };
    let output = output.path().unwrap();
    assert!(output.exists());

    let png = match PngData::new(&output, false) {
        Ok(x) => x,
        Err(x) => {
            remove_file(output).ok();
            panic!("{}", x)
        }
    };

    assert_eq!(png.raw.ihdr.color_type, ColorType::Grayscale);
    assert_eq!(png.raw.ihdr.bit_depth, BitDepth::Eight);

    // Cannot check if pixels are equal because image crate cannot read corrupt (input) PNGs
    remove_file(output).ok();
}

#[test]
fn zopfli_mode() {
    let input = PathBuf::from("tests/files/zopfli_mode.png");
    let (output, mut opts) = get_opts(&input);
    opts.deflate = Deflaters::Zopfli;

    test_it_converts(
        input,
        &output,
        &opts,
        ColorType::RGB,
        BitDepth::Eight,
        ColorType::RGB,
        BitDepth::Eight,
    );
}

#[test]
fn libdeflater_mode() {
    let input = PathBuf::from("tests/files/zopfli_mode.png");
    let (output, mut opts) = get_opts(&input);
    opts.deflate = Deflaters::Libdeflater;

    test_it_converts(
        input,
        &output,
        &opts,
        ColorType::RGB,
        BitDepth::Eight,
        ColorType::RGB,
        BitDepth::Eight,
    );
}
