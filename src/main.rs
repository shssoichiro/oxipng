#![warn(trivial_casts, trivial_numeric_casts, unused_import_braces)]
#![deny(missing_debug_implementations, missing_copy_implementations)]
#![warn(clippy::expl_impl_clone_on_copy)]
#![warn(clippy::float_cmp_const)]
#![warn(clippy::linkedlist)]
#![warn(clippy::map_flatten)]
#![warn(clippy::match_same_arms)]
#![warn(clippy::mem_forget)]
#![warn(clippy::mut_mut)]
#![warn(clippy::mutex_integer)]
#![warn(clippy::needless_continue)]
#![warn(clippy::path_buf_push_overwrite)]
#![warn(clippy::range_plus_one)]
#![allow(clippy::cognitive_complexity)]

use clap::{AppSettings, Arg, ArgMatches, Command};
use indexmap::IndexSet;
use log::{error, warn};
use oxipng::AlphaOptim;
use oxipng::Deflaters;
use oxipng::Headers;
use oxipng::Options;
use oxipng::RowFilter;
use oxipng::{InFile, OutFile};
use std::fs::DirBuilder;
#[cfg(feature = "zopfli")]
use std::num::NonZeroU8;
use std::path::PathBuf;
use std::process::exit;
use std::time::Duration;

fn main() {
    let matches = Command::new("oxipng")
        .version(env!("CARGO_PKG_VERSION"))
        .author("Joshua Holmer <jholmer.in@gmail.com>")
        .about("Losslessly improves compression of PNG files")
        .setting(AppSettings::DeriveDisplayOrder)
        .arg(
            Arg::new("files")
                .help("File(s) to compress (use \"-\" for stdin)")
                .index(1)
                .multiple_values(true)
                .use_value_delimiter(false)
                .required(true),
        )
        .arg(
            Arg::new("optimization")
                .help("Optimization level - Default: 2")
                .short('o')
                .long("opt")
                .takes_value(true)
                .value_name("level")
                .possible_value("0")
                .possible_value("1")
                .possible_value("2")
                .possible_value("3")
                .possible_value("4")
                .possible_value("5")
                .possible_value("6")
                .possible_value("max"),
        )
        .arg(
            Arg::new("backup")
                .help("Back up modified files")
                .short('b')
                .long("backup"),
        )
        .arg(
            Arg::new("recursive")
                .help("Recurse into subdirectories")
                .short('r')
                .long("recursive"),
        )
        .arg(
            Arg::new("output_dir")
                .help("Write output file(s) to <directory>")
                .long("dir")
                .takes_value(true)
                .value_name("directory")
                .conflicts_with("output_file")
                .conflicts_with("stdout"),
        )
        .arg(
            Arg::new("output_file")
                .help("Write output file to <file>")
                .long("out")
                .takes_value(true)
                .value_name("file")
                .conflicts_with("output_dir")
                .conflicts_with("stdout"),
        )
        .arg(
            Arg::new("stdout")
                .help("Write output to stdout")
                .long("stdout")
                .conflicts_with("output_dir")
                .conflicts_with("output_file"),
        )
        .arg(
            Arg::new("preserve")
                .help("Preserve file attributes if possible")
                .short('p')
                .long("preserve"),
        )
        .arg(
            Arg::new("check")
                .help("Do not run any optimization passes")
                .short('c')
                .long("check"),
        )
        .arg(
            Arg::new("pretend")
                .help("Do not write any files, only calculate compression gains")
                .short('P')
                .long("pretend"),
        )
        .arg(
            Arg::new("strip-safe")
                .help("Strip safely-removable metadata objects")
                .short('s')
                .conflicts_with("strip"),
        )
        .arg(
            Arg::new("strip")
                .help("Strip metadata objects ['safe', 'all', or comma-separated list]")
                .long("strip")
                .takes_value(true)
                .value_name("mode")
                .conflicts_with("strip-safe"),
        )
        .arg(
            Arg::new("keep")
                .help("Strip all optional metadata except objects in the comma-separated list")
                .long("keep")
                .takes_value(true)
                .value_name("list")
                .conflicts_with("strip")
                .conflicts_with("strip-safe"),
        )
        .arg(
            Arg::new("alpha")
                .help("Perform additional alpha optimizations")
                .short('a')
                .long("alpha"),
        )
        .arg(
            Arg::new("interlace")
                .help("PNG interlace type")
                .short('i')
                .long("interlace")
                .takes_value(true)
                .value_name("0/1")
                .possible_value("0")
                .possible_value("1"),
        )
        .arg(
            Arg::new("verbose")
                .help("Run in verbose mode")
                .short('v')
                .long("verbose")
                .conflicts_with("quiet"),
        )
        .arg(
            Arg::new("quiet")
                .help("Run in quiet mode")
                .short('q')
                .long("quiet")
                .conflicts_with("verbose"),
        )
        .arg(
            Arg::new("filters")
                .help(&*format!(
                    "PNG delta filters (0-{}) - Default: 0,{}",
                    RowFilter::LAST,
                    RowFilter::MinSum as u8
                ))
                .short('f')
                .long("filters")
                .takes_value(true)
                .validator(|x| match parse_numeric_range_opts(x, 0, RowFilter::LAST) {
                    Ok(_) => Ok(()),
                    Err(_) => Err("Invalid option for filters".to_owned()),
                }),
        )
        .arg(
            Arg::new("fast")
                .help("Use fast filter evaluation")
                .long("fast"),
        )
        .arg(
            Arg::new("compression")
                .help("zlib compression level (1-12) - Default: 11")
                .long("zc")
                .takes_value(true)
                .value_name("level")
                .value_parser(1..=12)
                .conflicts_with("zopfli"),
        )
        .arg(
            Arg::new("no-bit-reduction")
                .help("No bit depth reduction")
                .long("nb"),
        )
        .arg(
            Arg::new("no-color-reduction")
                .help("No color type reduction")
                .long("nc"),
        )
        .arg(
            Arg::new("no-palette-reduction")
                .help("No palette reduction")
                .long("np"),
        )
        .arg(
            Arg::new("no-grayscale-reduction")
                .help("No grayscale reduction")
                .long("ng"),
        )
        .arg(Arg::new("no-reductions").help("No reductions").long("nx"))
        .arg(
            Arg::new("no-recoding")
                .help("No IDAT recoding unless necessary")
                .long("nz"),
        )
        .arg(Arg::new("fix").help("Enable error recovery").long("fix"))
        .arg(
            Arg::new("force")
                .help("Write the output even if it is larger than the input")
                .long("force"),
        )
        .arg(
            Arg::new("zopfli")
                .help("Use the slower but better compressing Zopfli algorithm")
                .short('Z')
                .long("zopfli"),
        )
        .arg(
            Arg::new("timeout")
                .help("Maximum amount of time, in seconds, to spend on optimizations")
                .takes_value(true)
                .value_name("secs")
                .long("timeout"),
        )
        .arg(
            Arg::new("threads")
                .help("Set number of threads to use - default 1.5x CPU cores")
                .long("threads")
                .short('t')
                .takes_value(true)
                .value_name("num")
                .validator(|x| match x.parse::<usize>() {
                    Ok(val) => {
                        if val > 0 {
                            Ok(())
                        } else {
                            Err("Thread count must be >= 1".to_owned())
                        }
                    }
                    Err(_) => Err("Thread count must be >= 1".to_owned()),
                }),
        )
        .after_help(
            "Optimization levels:
    -o 0   =>  --zc 6 --nz          (0 or 1 trials)
    -o 1   =>  --zc 10              (1 trial, determined heuristically)
    -o 2   =>  --zc 11 -f 0,5       (2 trials)
    -o 3   =>  --zc 11 -f 0-5       (6 trials)
    -o 4   =>  --zc 12 -f 0-5       (6 trials; same as `-o 3` for zopfli)
    -o 5   =>  --zc 9-12 -f 0-5     (24 trials; same as `-o 3` for zopfli)
    -o 6   =>  --zc 1-12 -f 0-5     (72 trials; same as `-o 3` for zopfli)
    -o max =>                       (stable alias for the max compression)

    Manually specifying a compression option (zc, f, etc.) will override the optimization preset,
    regardless of the order you write the arguments.

PNG delta filters:
    0  =>  None
    1  =>  Sub
    2  =>  Up
    3  =>  Average
    4  =>  Paeth
Heuristic filter selection strategies:
    5  =>  MinSum    Minimum sum of absolute differences
    6  =>  Entropy   Highest Shannon entropy
    7  =>  Bigrams   Lowest count of distinct bigrams
    8  =>  BigEnt    Highest Shannon entropy of bigrams
    9  =>  Brute     Smallest compressed size (slow)",
        )
        .get_matches_from(wild::args());

    let (out_file, out_dir, opts) = match parse_opts_into_struct(&matches) {
        Ok(x) => x,
        Err(x) => {
            error!("{}", x);
            exit(1)
        }
    };

    let files = collect_files(
        matches
            .values_of("files")
            .unwrap()
            .map(PathBuf::from)
            .collect(),
        &out_dir,
        &out_file,
        matches.is_present("recursive"),
        true,
    );

    let mut success = false;
    for (input, output) in files {
        match oxipng::optimize(&input, &output, &opts) {
            // For optimizing single files, this will return the correct exit code always.
            // For recursive optimization, the correct choice is a bit subjective.
            // We're choosing to return a 0 exit code if ANY file in the set
            // runs correctly.
            // The reason for this is that recursion may pick up files that are not
            // PNG files, and return an error for them.
            // We don't really want to return an error code for those files.
            Ok(_) => {
                success = true;
            }
            Err(e) => {
                error!("{}", e);
            }
        };
    }

    if !success {
        exit(1);
    }
}

fn collect_files(
    files: Vec<PathBuf>,
    out_dir: &Option<PathBuf>,
    out_file: &OutFile,
    recursive: bool,
    allow_stdin: bool,
) -> Vec<(InFile, OutFile)> {
    let mut in_out_pairs = Vec::new();
    let allow_stdin = allow_stdin && files.len() == 1;
    for input in files {
        let using_stdin = allow_stdin && input.to_str().map_or(false, |p| p == "-");
        if !using_stdin && input.is_dir() {
            if recursive {
                match input.read_dir() {
                    Ok(dir) => {
                        let files = dir.filter_map(|x| x.ok().map(|x| x.path())).collect();
                        in_out_pairs
                            .extend(collect_files(files, out_dir, out_file, recursive, false));
                    }
                    Err(_) => {
                        return Vec::new();
                    }
                }
            } else {
                warn!("{} is a directory, skipping", input.display());
            }
            continue;
        };
        let out_file = if let Some(ref out_dir) = *out_dir {
            let out_path = Some(out_dir.join(input.file_name().unwrap()));
            OutFile::Path(out_path)
        } else {
            (*out_file).clone()
        };
        let in_file = if using_stdin {
            InFile::StdIn
        } else {
            InFile::Path(input)
        };
        in_out_pairs.push((in_file, out_file));
    }
    in_out_pairs
}

fn parse_opts_into_struct(
    matches: &ArgMatches,
) -> Result<(OutFile, Option<PathBuf>, Options), String> {
    stderrlog::new()
        .module(module_path!())
        .quiet(matches.is_present("quiet"))
        .verbosity(if matches.is_present("verbose") { 3 } else { 2 })
        .show_level(false)
        .init()
        .unwrap();

    let (explicit_level, mut opts) = match matches.value_of("optimization") {
        None => (None, Options::default()),
        Some("max") => (None, Options::max_compression()),
        Some(level) => {
            let level = level.parse::<u8>().unwrap();
            (Some(level), Options::from_preset(level))
        }
    };

    if let Some(x) = matches.value_of("interlace") {
        opts.interlace = x.parse::<u8>().ok();
    }

    if let Some(x) = matches.value_of("filters") {
        opts.filter.clear();
        for f in parse_numeric_range_opts(x, 0, RowFilter::LAST).unwrap() {
            opts.filter.insert(RowFilter::try_from(f).unwrap());
        }
    }

    if let Some(x) = matches.value_of("timeout") {
        let num = x
            .parse()
            .map_err(|_| "Timeout must be a number".to_owned())?;
        opts.timeout = Some(Duration::from_secs(num));
    }

    let out_dir = if let Some(x) = matches.value_of("output_dir") {
        let path = PathBuf::from(x);
        if !path.exists() {
            match DirBuilder::new().recursive(true).create(&path) {
                Ok(_) => (),
                Err(x) => return Err(format!("Could not create output directory {}", x)),
            };
        } else if !path.is_dir() {
            return Err(format!(
                "{} is an existing file (not a directory), cannot create directory",
                x
            ));
        }
        Some(path)
    } else {
        None
    };

    let out_file = if matches.is_present("stdout") {
        OutFile::StdOut
    } else if let Some(x) = matches.value_of("output_file") {
        OutFile::Path(Some(PathBuf::from(x)))
    } else {
        OutFile::Path(None)
    };

    if matches.is_present("alpha") {
        opts.alphas.insert(AlphaOptim::Black);
        opts.alphas.insert(AlphaOptim::White);
        opts.alphas.insert(AlphaOptim::Up);
        opts.alphas.insert(AlphaOptim::Left);
    }

    if matches.is_present("fast") {
        opts.fast_evaluation = true;
    }

    if matches.is_present("backup") {
        opts.backup = true;
    }

    if matches.is_present("force") {
        opts.force = true;
    }

    if matches.is_present("fix") {
        opts.fix_errors = true;
    }

    if matches.is_present("check") {
        opts.check = true;
    }

    if matches.is_present("pretend") {
        opts.pretend = true;
    }

    if matches.is_present("preserve") {
        opts.preserve_attrs = true;
    }

    if matches.is_present("no-bit-reduction") {
        opts.bit_depth_reduction = false;
    }

    if matches.is_present("no-color-reduction") {
        opts.color_type_reduction = false;
    }

    if matches.is_present("no-palette-reduction") {
        opts.palette_reduction = false;
    }

    if matches.is_present("no-grayscale-reduction") {
        opts.grayscale_reduction = false;
    }

    if matches.is_present("no-reductions") {
        opts.bit_depth_reduction = false;
        opts.color_type_reduction = false;
        opts.palette_reduction = false;
        opts.grayscale_reduction = false;
    }

    if matches.is_present("no-recoding") {
        opts.idat_recoding = false;
    }

    if let Some(hdrs) = matches.value_of("keep") {
        opts.strip = Headers::Keep(hdrs.split(',').map(|x| x.trim().to_owned()).collect())
    }

    if let Some(hdrs) = matches.value_of("strip") {
        let hdrs = hdrs
            .split(',')
            .map(|x| x.trim().to_owned())
            .collect::<Vec<String>>();
        if hdrs.contains(&"safe".to_owned()) || hdrs.contains(&"all".to_owned()) {
            if hdrs.len() > 1 {
                return Err(
                    "'safe' or 'all' presets for --strip should be used by themselves".to_owned(),
                );
            }
            if hdrs[0] == "safe" {
                opts.strip = Headers::Safe;
            } else {
                opts.strip = Headers::All;
            }
        } else {
            const FORBIDDEN_CHUNKS: [[u8; 4]; 5] =
                [*b"IHDR", *b"IDAT", *b"tRNS", *b"PLTE", *b"IEND"];
            for i in &hdrs {
                if FORBIDDEN_CHUNKS.iter().any(|chunk| chunk == i.as_bytes()) {
                    return Err(format!("{} chunk is not allowed to be stripped", i));
                }
            }
            opts.strip = Headers::Strip(hdrs);
        }
    }

    if matches.is_present("strip-safe") {
        opts.strip = Headers::Safe;
    }

    if matches.is_present("zopfli") {
        if explicit_level > Some(3) {
            warn!("Level 4 and above are equivalent to level 3 for zopfli");
        }
        #[cfg(feature = "zopfli")]
        if let Some(iterations) = NonZeroU8::new(15) {
            opts.deflate = Deflaters::Zopfli { iterations };
        }
    } else if let Deflaters::Libdeflater { compression } = &mut opts.deflate {
        if let Some(x) = matches.get_one::<i64>("compression") {
            *compression = *x as u8;
        }
    }

    #[cfg(feature = "parallel")]
    if let Some(x) = matches.value_of("threads") {
        let threads = x.parse::<usize>().unwrap();

        rayon::ThreadPoolBuilder::new()
            .num_threads(threads)
            .build_global()
            .map_err(|err| err.to_string())?;
    }

    Ok((out_file, out_dir, opts))
}

fn parse_numeric_range_opts(
    input: &str,
    min_value: u8,
    max_value: u8,
) -> Result<IndexSet<u8>, String> {
    const ERROR_MESSAGE: &str = "Not a valid input";
    let mut items = IndexSet::new();

    // one value
    if let Ok(one_value) = input.parse::<u8>() {
        if (min_value <= one_value) && (one_value <= max_value) {
            items.insert(one_value);
            return Ok(items);
        }
    }

    // a range ("A-B")
    let range_values = input.split('-').collect::<Vec<&str>>();
    if range_values.len() == 2 {
        let first_opt = range_values[0].parse::<u8>();
        let second_opt = range_values[1].parse::<u8>();
        if let (Ok(first), Ok(second)) = (first_opt, second_opt) {
            if min_value <= first && first < second && second <= max_value {
                for i in first..=second {
                    items.insert(i);
                }
                return Ok(items);
            }
        }
        return Err(ERROR_MESSAGE.to_owned());
    }

    // a list ("A,B[,…]")
    let list_items = input.split(',').collect::<Vec<&str>>();
    if list_items.len() > 1 {
        for value in list_items {
            if let Ok(value_int) = value.parse::<u8>() {
                if (min_value <= value_int)
                    && (value_int <= max_value)
                    && !items.contains(&value_int)
                {
                    items.insert(value_int);
                    continue;
                }
            }
            return Err(ERROR_MESSAGE.to_owned());
        }
        return Ok(items);
    }

    Err(ERROR_MESSAGE.to_owned())
}
