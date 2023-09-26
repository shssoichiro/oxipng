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

#[cfg(not(feature = "parallel"))]
mod rayon;

use clap::{value_parser, Arg, ArgAction, ArgMatches, Command};
use indexmap::IndexSet;
use log::{error, warn, Level, LevelFilter};
use oxipng::Deflaters;
use oxipng::Options;
use oxipng::RowFilter;
use oxipng::StripChunks;
use oxipng::{InFile, OutFile};
use rayon::prelude::*;
use std::ffi::OsString;
use std::fs::DirBuilder;
use std::io::Write;
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
        .arg(
            Arg::new("files")
                .help("File(s) to compress (use \"-\" for stdin)")
                .index(1)
                .num_args(1..)
                .use_value_delimiter(false)
                .required(true)
                .value_parser(value_parser!(PathBuf)),
        )
        .arg(
            Arg::new("optimization")
                .help("Optimization level - Default: 2")
                .short('o')
                .long("opt")
                .value_name("level")
                .value_parser(["0", "1", "2", "3", "4", "5", "6", "max"]),
        )
        .arg(
            Arg::new("backup")
                .help("Back up modified files")
                .short('b')
                .long("backup")
                .hide(true)
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("recursive")
                .help("Recurse into subdirectories and optimize all *.png/*.apng files")
                .short('r')
                .long("recursive")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("output_dir")
                .help("Write output file(s) to <directory>")
                .long("dir")
                .value_name("directory")
                .value_parser(value_parser!(PathBuf))
                .conflicts_with("output_file")
                .conflicts_with("stdout"),
        )
        .arg(
            Arg::new("output_file")
                .help("Write output file to <file>")
                .long("out")
                .value_name("file")
                .value_parser(value_parser!(PathBuf))
                .conflicts_with("output_dir")
                .conflicts_with("stdout"),
        )
        .arg(
            Arg::new("stdout")
                .help("Write output to stdout")
                .long("stdout")
                .action(ArgAction::SetTrue)
                .conflicts_with("output_dir")
                .conflicts_with("output_file"),
        )
        .arg(
            Arg::new("preserve")
                .help("Preserve file attributes if possible")
                .short('p')
                .long("preserve")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("pretend")
                .help("Do not write any files, only calculate compression gains")
                .short('P')
                .long("pretend")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("strip-safe")
                .help("Strip safely-removable metadata objects")
                .short('s')
                .action(ArgAction::SetTrue)
                .conflicts_with("strip"),
        )
        .arg(
            Arg::new("strip")
                .help("Strip metadata objects ['safe', 'all', or comma-separated list]\nCAUTION: stripping 'all' will convert APNGs to standard PNGs")
                .long("strip")
                .value_name("mode")
                .conflicts_with("strip-safe"),
        )
        .arg(
            Arg::new("keep")
                .help("Strip all optional metadata except objects in the comma-separated list")
                .long("keep")
                .value_name("list")
                .conflicts_with("strip")
                .conflicts_with("strip-safe"),
        )
        .arg(
            Arg::new("alpha")
                .help("Perform additional alpha optimizations")
                .short('a')
                .long("alpha")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("interlace")
                .help("PNG interlace type - Default: 0")
                .short('i')
                .long("interlace")
                .value_name("type")
                .value_parser(["0", "1", "keep"]),
        )
        .arg(
            Arg::new("scale16")
                .help("Forcibly reduce 16-bit images to 8-bit")
                .long("scale16")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("verbose")
                .help("Run in verbose mode (use multiple times to increase verbosity)")
                .short('v')
                .long("verbose")
                .action(ArgAction::Count)
                .conflicts_with("quiet"),
        )
        .arg(
            Arg::new("quiet")
                .help("Run in quiet mode")
                .short('q')
                .long("quiet")
                .action(ArgAction::SetTrue)
                .conflicts_with("verbose"),
        )
        .arg(
            Arg::new("filters")
                .help(format!("PNG delta filters (0-{})", RowFilter::LAST))
                .short('f')
                .long("filters")
                .value_parser(|x: &str| {
                    parse_numeric_range_opts(x, 0, RowFilter::LAST)
                        .map_err(|_| "Invalid option for filters")
                }),
        )
        .arg(
            Arg::new("fast")
                .help("Use fast filter evaluation (helpful when you have more filters enabled than CPU cores)")
                .long("fast")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("compression")
                .help("zlib compression level (1-12)")
                .long("zc")
                .value_name("level")
                .value_parser(1..=12)
                .conflicts_with("zopfli"),
        )
        .arg(
            Arg::new("no-bit-reduction")
                .help("No bit depth reduction")
                .long("nb")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("no-color-reduction")
                .help("No color type reduction")
                .long("nc")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("no-palette-reduction")
                .help("No palette reduction")
                .long("np")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("no-grayscale-reduction")
                .help("No grayscale reduction")
                .long("ng")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("no-reductions")
                .help("No reductions or deinterlacing")
                .long("nx")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("no-recoding")
                .help("No recoding of IDAT or other compressed chunks unless necessary")
                .long("nz")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("fix")
                .help("Enable error recovery")
                .long("fix")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("force")
                .help("Write the output even if it is larger than the input")
                .long("force")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("zopfli")
                .help("Use the slow but stronger Zopfli compressor (recommended use is with all filters and `--fast` enabled)")
                .short('Z')
                .long("zopfli")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("timeout")
                .help("Maximum amount of time, in seconds, to spend on optimizations (currently of limited use due to the shift away from zlib)")
                .value_name("secs")
                .long("timeout")
                .value_parser(value_parser!(u64)),
        )
        .arg(
            Arg::new("threads")
                .help("Set number of threads to use - Default: num CPU cores")
                .long("threads")
                .short('t')
                .value_name("num")
                .value_parser(value_parser!(usize)),
        )
        .after_help(
            "Optimization levels:
    -o 0   =>  --zc 5 --fast                (1 trial, determined heuristically)
    -o 1   =>  --zc 10 --fast               (1 trial, determined heuristically)
    -o 2   =>  --zc 11 -f 0,1,6,7 --fast    (1 trial, determined by fast evaluation)
    -o 3   =>  --zc 11 -f 0,7,8,9           (4 trials)
    -o 4   =>  --zc 12 -f 0,7,8,9           (4 trials; same as `-o 3` for zopfli)
    -o 5   =>  --zc 12 -f 0,1,2,5,6,7,8,9   (8 trials)
    -o 6   =>  --zc 12 -f 0-9               (10 trials)
    -o max =>                               (stable alias for the max compression)

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
        .get_matches_from(std::env::args());

    if matches.get_flag("backup") {
        eprintln!("The --backup flag is no longer supported. Please use --out or --dir to preserve your existing files.");
        exit(1)
    }

    let (out_file, out_dir, opts) = match parse_opts_into_struct(&matches) {
        Ok(x) => x,
        Err(x) => {
            error!("{}", x);
            exit(1)
        }
    };

    let files = collect_files(
        #[cfg(windows)]
        matches
            .get_many::<PathBuf>("files")
            .unwrap()
            .cloned()
            .flat_map(apply_glob_pattern)
            .collect(),
        #[cfg(not(windows))]
        matches
            .get_many::<PathBuf>("files")
            .unwrap()
            .cloned()
            .collect(),
        &out_dir,
        &out_file,
        matches.get_flag("recursive"),
        true,
    );

    let success = files.into_par_iter().filter(|(input, output)| {
        match oxipng::optimize(input, output, &opts) {
            // For optimizing single files, this will return the correct exit code always.
            // For recursive optimization, the correct choice is a bit subjective.
            // We're choosing to return a 0 exit code if ANY file in the set
            // runs correctly.
            // The reason for this is that recursion may pick up files that are not
            // PNG files, and return an error for them.
            // We don't really want to return an error code for those files.
            Ok(_) => true,
            Err(e) => {
                error!("{}: {}", input, e);
                false
            }
        }
    });

    if success.count() == 0 {
        exit(1);
    }
}

fn collect_files(
    files: Vec<PathBuf>,
    out_dir: &Option<PathBuf>,
    out_file: &OutFile,
    recursive: bool,
    top_level: bool, //explicitly specify files
) -> Vec<(InFile, OutFile)> {
    let mut in_out_pairs = Vec::new();
    let allow_stdin = top_level && files.len() == 1;
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
                    Err(e) => {
                        warn!("{}: {}", input.display(), e);
                    }
                }
            } else {
                warn!("{} is a directory, skipping", input.display());
            }
            continue;
        };
        let out_file =
            if let (Some(out_dir), &OutFile::Path { preserve_attrs, .. }) = (out_dir, out_file) {
                let path = Some(out_dir.join(input.file_name().unwrap()));
                OutFile::Path {
                    path,
                    preserve_attrs,
                }
            } else {
                (*out_file).clone()
            };
        let in_file = if using_stdin {
            InFile::StdIn
        } else {
            // Skip non png files if not given on top level
            if !top_level && {
                let extension = input.extension().map(|f| f.to_ascii_lowercase());
                extension != Some(OsString::from("png"))
                    && extension != Some(OsString::from("apng"))
            } {
                continue;
            }
            InFile::Path(input)
        };
        in_out_pairs.push((in_file, out_file));
    }
    in_out_pairs
}

#[cfg(windows)]
fn apply_glob_pattern(path: PathBuf) -> Vec<PathBuf> {
    let matches = path
        .to_str()
        .and_then(|pattern| glob::glob(pattern).ok())
        .map(|paths| paths.flatten().collect::<Vec<_>>());

    match matches {
        Some(paths) if !paths.is_empty() => paths,
        _ => vec![path],
    }
}

fn parse_opts_into_struct(
    matches: &ArgMatches,
) -> Result<(OutFile, Option<PathBuf>, Options), String> {
    let log_level = match matches.get_count("verbose") {
        _ if matches.get_flag("quiet") => LevelFilter::Off,
        0 => LevelFilter::Info,
        1 => LevelFilter::Debug,
        _ => LevelFilter::Trace,
    };
    env_logger::builder()
        .filter_module(module_path!(), log_level)
        .format(|buf, record| {
            let style = match record.level() {
                Level::Error | Level::Warn => buf.default_level_style(record.level()),
                _ => buf.style(), // Leave info, debug and trace unstyled
            };
            writeln!(buf, "{}", style.value(record.args()))
        })
        .init();

    let mut opts = match matches.get_one::<String>("optimization") {
        None => Options::default(),
        Some(x) if x == "max" => Options::max_compression(),
        Some(level) => Options::from_preset(level.parse::<u8>().unwrap()),
    };

    if let Some(x) = matches.get_one::<IndexSet<u8>>("filters") {
        opts.filter.clear();
        for &f in x {
            opts.filter.insert(f.try_into().unwrap());
        }
    }

    if let Some(&num) = matches.get_one::<u64>("timeout") {
        opts.timeout = Some(Duration::from_secs(num));
    }

    let out_dir = if let Some(path) = matches.get_one::<PathBuf>("output_dir") {
        if !path.exists() {
            match DirBuilder::new().recursive(true).create(path) {
                Ok(_) => (),
                Err(x) => return Err(format!("Could not create output directory {}", x)),
            };
        } else if !path.is_dir() {
            return Err(format!(
                "{} is an existing file (not a directory), cannot create directory",
                path.display()
            ));
        }
        Some(path.to_owned())
    } else {
        None
    };

    let out_file = if matches.get_flag("pretend") {
        OutFile::None
    } else if matches.get_flag("stdout") {
        OutFile::StdOut
    } else {
        OutFile::Path {
            path: matches.get_one::<PathBuf>("output_file").cloned(),
            preserve_attrs: matches.get_flag("preserve"),
        }
    };

    opts.optimize_alpha = matches.get_flag("alpha");

    opts.scale_16 = matches.get_flag("scale16");

    // The default value for fast depends on the preset - make sure we don't change when not provided
    if matches.get_flag("fast") {
        opts.fast_evaluation = matches.get_flag("fast");
    }

    opts.force = matches.get_flag("force");

    opts.fix_errors = matches.get_flag("fix");

    opts.bit_depth_reduction = !matches.get_flag("no-bit-reduction");

    opts.color_type_reduction = !matches.get_flag("no-color-reduction");

    opts.palette_reduction = !matches.get_flag("no-palette-reduction");

    opts.grayscale_reduction = !matches.get_flag("no-grayscale-reduction");

    if matches.get_flag("no-reductions") {
        opts.bit_depth_reduction = false;
        opts.color_type_reduction = false;
        opts.palette_reduction = false;
        opts.grayscale_reduction = false;
        opts.interlace = None;
    }

    opts.idat_recoding = !matches.get_flag("no-recoding");

    if let Some(x) = matches.get_one::<String>("interlace") {
        opts.interlace = if x == "keep" {
            None
        } else {
            x.parse::<u8>().unwrap().try_into().ok()
        };
    }

    if let Some(keep) = matches.get_one::<String>("keep") {
        let names = keep
            .split(',')
            .map(parse_chunk_name)
            .collect::<Result<_, _>>()?;
        opts.strip = StripChunks::Keep(names)
    }

    if let Some(strip) = matches.get_one::<String>("strip") {
        if strip == "safe" {
            opts.strip = StripChunks::Safe;
        } else if strip == "all" {
            opts.strip = StripChunks::All;
        } else {
            const FORBIDDEN_CHUNKS: [[u8; 4]; 5] =
                [*b"IHDR", *b"IDAT", *b"tRNS", *b"PLTE", *b"IEND"];
            let names = strip
                .split(',')
                .map(|x| {
                    if x == "safe" || x == "all" {
                        return Err(
                            "'safe' or 'all' presets for --strip should be used by themselves"
                                .to_owned(),
                        );
                    }
                    let name = parse_chunk_name(x)?;
                    if FORBIDDEN_CHUNKS.contains(&name) {
                        return Err(format!("{} chunk is not allowed to be stripped", x));
                    }
                    Ok(name)
                })
                .collect::<Result<_, _>>()?;
            opts.strip = StripChunks::Strip(names);
        }
    }

    if matches.get_flag("strip-safe") {
        opts.strip = StripChunks::Safe;
    }

    if matches.get_flag("zopfli") {
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
    if let Some(&threads) = matches.get_one::<usize>("threads") {
        rayon::ThreadPoolBuilder::new()
            .num_threads(threads)
            .build_global()
            .map_err(|err| err.to_string())?;
    }

    Ok((out_file, out_dir, opts))
}

fn parse_chunk_name(name: &str) -> Result<[u8; 4], String> {
    name.trim()
        .as_bytes()
        .try_into()
        .map_err(|_| format!("Invalid chunk name {}", name))
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
