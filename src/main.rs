#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]
#![cfg_attr(feature="clippy", warn(enum_glob_use))]
#![cfg_attr(feature="clippy", warn(if_not_else))]
#![cfg_attr(feature="clippy", warn(string_add))]
#![cfg_attr(feature="clippy", warn(string_add_assign))]
#![warn(trivial_casts, trivial_numeric_casts, unused_import_braces)]
#![deny(missing_debug_implementations, missing_copy_implementations)]

extern crate oxipng;
extern crate clap;
extern crate regex;

use clap::{App, Arg, ArgMatches};
use oxipng::deflate::Deflaters;
use oxipng::headers::Headers;
use oxipng::Options;
use regex::Regex;
use std::collections::HashSet;
use std::fs::DirBuilder;
use std::io::{Write, stderr};
use std::path::PathBuf;

fn main() {
    let matches = App::new("oxipng")
        .version(env!("CARGO_PKG_VERSION"))
        .author("Joshua Holmer <jholmer.in@gmail.com>")
        .about("Losslessly improves compression of PNG files")
        .arg(Arg::with_name("files")
            .help("File(s) to compress")
            .index(1)
            .multiple(true)
            .use_delimiter(false)
            .required(true))
        .arg(Arg::with_name("optimization")
            .help("Optimization level - Default: 2")
            .short("o")
            .long("opt")
            .takes_value(true)
            .possible_value("0")
            .possible_value("1")
            .possible_value("2")
            .possible_value("3")
            .possible_value("4")
            .possible_value("5")
            .possible_value("6"))
        .arg(Arg::with_name("backup")
            .help("Back up modified files")
            .short("b")
            .long("backup"))
        .arg(Arg::with_name("force")
            .help("Write output even if larger than the original")
            .short("F")
            .long("force"))
        .arg(Arg::with_name("recursive")
            .help("Recurse into subdirectories")
            .short("r")
            .long("recursive"))
        .arg(Arg::with_name("output_dir")
            .help("Write output file(s) to <directory>")
            .long("dir")
            .takes_value(true)
            .conflicts_with("output_file")
            .conflicts_with("stdout"))
        .arg(Arg::with_name("output_file")
            .help("Write output file to <file>")
            .long("out")
            .takes_value(true)
            .conflicts_with("output_dir")
            .conflicts_with("stdout"))
        .arg(Arg::with_name("stdout")
            .help("Write output to stdout")
            .long("stdout")
            .conflicts_with("output_dir")
            .conflicts_with("output_file"))
        .arg(Arg::with_name("fix")
            .help("Enable error recovery")
            .long("fix"))
        .arg(Arg::with_name("no-clobber")
            .help("Do not overwrite existing files")
            .long("no-clobber"))
        .arg(Arg::with_name("pretend")
            .help("Do not write any files, only calculate compression gains")
            .short("P")
            .long("pretend"))
        .arg(Arg::with_name("preserve")
            .help("Preserve file attributes if possible")
            .short("p")
            .long("preserve"))
        .arg(Arg::with_name("quiet")
            .help("Run in quiet mode")
            .short("q")
            .long("quiet")
            .conflicts_with("verbose"))
        .arg(Arg::with_name("verbose")
            .help("Run in verbose mode")
            .short("v")
            .long("verbose")
            .conflicts_with("quiet"))
        .arg(Arg::with_name("filters")
            .help("PNG delta filters (0-5) - Default: 0,5")
            .short("f")
            .long("filters")
            .takes_value(true)
            .validator(|x| {
                match parse_numeric_range_opts(&x, 0, 5) {
                    Ok(_) => Ok(()),
                    Err(_) => Err("Invalid option for filters".to_owned()),
                }
            }))
        .arg(Arg::with_name("interlace")
            .help("PNG interlace type")
            .short("i")
            .long("interlace")
            .takes_value(true)
            .possible_value("0")
            .possible_value("1"))
        .arg(Arg::with_name("compression")
            .help("zlib compression levels (1-9) - Default: 9")
            .long("zc")
            .takes_value(true)
            .validator(|x| {
                match parse_numeric_range_opts(&x, 1, 9) {
                    Ok(_) => Ok(()),
                    Err(_) => Err("Invalid option for compression".to_owned()),
                }
            }))
        .arg(Arg::with_name("memory")
            .help("zlib memory levels (1-9) - Default: 9")
            .long("zm")
            .takes_value(true)
            .validator(|x| {
                match parse_numeric_range_opts(&x, 1, 9) {
                    Ok(_) => Ok(()),
                    Err(_) => Err("Invalid option for memory".to_owned()),
                }
            }))
        .arg(Arg::with_name("strategies")
            .help("zlib compression strategies (0-3) - Default: 0-3")
            .long("zs")
            .takes_value(true)
            .validator(|x| {
                match parse_numeric_range_opts(&x, 0, 3) {
                    Ok(_) => Ok(()),
                    Err(_) => Err("Invalid option for strategies".to_owned()),
                }
            }))
        .arg(Arg::with_name("window")
            .help("zlib window size - Default: 32k")
            .long("zw")
            .takes_value(true)
            .possible_value("256")
            .possible_value("512")
            .possible_value("1k")
            .possible_value("2k")
            .possible_value("4k")
            .possible_value("8k")
            .possible_value("16k")
            .possible_value("32k"))
        .arg(Arg::with_name("no-bit-reduction")
            .help("No bit depth reduction")
            .long("nb"))
        .arg(Arg::with_name("no-color-reduction")
            .help("No color type reduction")
            .long("nc"))
        .arg(Arg::with_name("no-palette-reduction")
            .help("No palette reduction")
            .long("np"))
        .arg(Arg::with_name("no-reductions")
            .help("No reductions")
            .long("nx"))
        .arg(Arg::with_name("no-recoding")
            .help("No IDAT recoding unless necessary")
            .long("nz"))
        .arg(Arg::with_name("strip")
            .help("Strip metadata objects ['safe', 'all', or comma-separated list]")
            .long("strip")
            .takes_value(true)
            .conflicts_with("strip-safe"))
        .arg(Arg::with_name("strip-safe")
            .help("Strip safely-removable metadata objects")
            .short("s")
            .conflicts_with("strip"))
        .arg(Arg::with_name("zopfli")
            .help("Use the slower but better compressing Zopfli algorithm, overrides zlib-specific options")
            .short("Z")
            .long("zopfli"))
        .arg(Arg::with_name("threads")
            .help("Set number of threads to use - default 1.5x CPU cores")
            .long("threads")
            .short("t")
            .takes_value(true)
            .validator(|x| {
                match x.parse::<usize>() {
                    Ok(val) => {
                        if val > 0 {
                            Ok(())
                        } else {
                            Err("Thread count must be >= 1".to_owned())
                        }
                    }
                    Err(_) => Err("Thread count must be >= 1".to_owned()),
                }
            }))
        .after_help("Optimization levels:
    -o 0		=>	--zc 3 --nz				(0 or 1 trials)
    -o 1		=>	--zc 9					(1 trial, determined heuristically)
    -o 2		=>	--zc 9 --zs 0-3 --f 0,5			(8 trials)
    -o 3		=>	--zc 9 --zm 8-9 --zs 0-3 --f 0,5	(16 trials)
    -o 4		=>	--zc 9 --zm 8-9 --zs 0-3 --f 0-5	(48 trials)
    -o 5		=>	--zc 3-9 --zm 8-9 --zs 0-3 --f 0-5	(192 trials)
    -o 6		=>	--zc 1-9 --zm 7-9 --zs 0-3 --f 0-5	(360 trials)
    -o 6 --zm 1-9	=>	--zc 1-9 --zm 1-9 --zs 0-3 --f 0-5	(1080 trials)

    Exhaustive combinations such as \"-o6 -zm1-9\" are not generally recommended.
    These are very slow and generally provide no compression gain.

    Manually specifying a compression option (zc, zm, etc.) will override the optimization preset,
    regardless of the order you write the arguments.")
        .get_matches();

    let opts = match parse_opts_into_struct(&matches) {
        Ok(x) => x,
        Err(x) => {
            writeln!(&mut stderr(), "{}", x).ok();
            return ();
        }
    };

    handle_optimization(matches.values_of("files")
                            .unwrap()
                            .map(PathBuf::from)
                            .collect(),
                        &opts);
}

fn handle_optimization(inputs: Vec<PathBuf>, opts: &Options) {
    for input in inputs {
        let mut current_opts = opts.clone();
        if input.is_dir() {
            if current_opts.recursive {
                handle_optimization(input.read_dir().unwrap().map(|x| x.unwrap().path()).collect(),
                                    &current_opts)
            } else {
                writeln!(&mut stderr(),
                         "{} is a directory, skipping",
                         input.display())
                    .ok();
            }
            continue;
        }
        if let Some(ref out_dir) = current_opts.out_dir {
            current_opts.out_file = out_dir.join(input.file_name().unwrap());
        } else if current_opts.out_file.components().count() == 0 {
            current_opts.out_file = input.clone();
        }
        match oxipng::optimize(&input, &current_opts) {
            Ok(_) => (),
            Err(x) => {
                writeln!(&mut stderr(), "{}", x).ok();
            }
        };
    }
}

#[cfg_attr(feature="clippy", allow(cyclomatic_complexity))]
fn parse_opts_into_struct(matches: &ArgMatches) -> Result<Options, String> {
    let mut opts = if let Some(x) = matches.value_of("optimization") {
        if let Ok(opt) = x.parse::<u8>() {
            Options::from_preset(opt)
        } else {
            unreachable!()
        }
    } else {
        Options::default()
    };

    if let Some(x) = matches.value_of("interlace") {
        opts.interlace = x.parse::<u8>().ok();
    }

    if let Some(x) = matches.value_of("filters") {
        opts.filter = parse_numeric_range_opts(x, 0, 5).unwrap();
    }

    if let Some(x) = matches.value_of("compression") {
        opts.compression = parse_numeric_range_opts(x, 1, 9).unwrap();
    }

    if let Some(x) = matches.value_of("memory") {
        opts.memory = parse_numeric_range_opts(x, 1, 9).unwrap();
    }

    if let Some(x) = matches.value_of("strategies") {
        opts.strategies = parse_numeric_range_opts(x, 0, 3).unwrap();
    }

    match matches.value_of("window") {
        Some("256") => opts.window = 8,
        Some("512") => opts.window = 9,
        Some("1k") => opts.window = 10,
        Some("2k") => opts.window = 11,
        Some("4k") => opts.window = 12,
        Some("8k") => opts.window = 13,
        Some("16k") => opts.window = 14,
        // 32k is default
        _ => (),
    }

    if let Some(x) = matches.value_of("output_dir") {
        let path = PathBuf::from(x);
        if !path.exists() {
            match DirBuilder::new()
                .recursive(true)
                .create(&path) {
                Ok(_) => (),
                Err(x) => return Err(format!("Could not create output directory {}", x)),
            };
        } else if !path.is_dir() {
            return Err(format!("{} is an existing file (not a directory), cannot create directory",
                               x));
        }
        opts.out_dir = Some(path);
    }

    if let Some(x) = matches.value_of("output_file") {
        opts.out_file = PathBuf::from(x);
    }

    if matches.is_present("stdout") {
        opts.stdout = true;
    }

    if matches.is_present("backup") {
        opts.backup = true;
    }

    if matches.is_present("force") {
        opts.force = true;
    }

    if matches.is_present("recursive") {
        opts.recursive = true;
    }

    if matches.is_present("fix") {
        opts.fix_errors = true;
    }

    if matches.is_present("clobber") {
        opts.clobber = false;
    }

    if matches.is_present("pretend") {
        opts.pretend = true;
    }

    if matches.is_present("preserve") {
        opts.preserve_attrs = true;
    }

    if matches.is_present("quiet") {
        opts.verbosity = None;
    }

    if matches.is_present("verbose") {
        opts.verbosity = Some(1);
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

    if matches.is_present("no-reductions") {
        opts.bit_depth_reduction = false;
        opts.color_type_reduction = false;
        opts.palette_reduction = false;
    }

    if matches.is_present("no-recoding") {
        opts.idat_recoding = false;
    }

    if let Some(hdrs) = matches.value_of("strip") {
        let hdrs = hdrs.split(',').map(|x| x.trim().to_owned()).collect::<Vec<String>>();
        if hdrs.contains(&"safe".to_owned()) || hdrs.contains(&"all".to_owned()) {
            if hdrs.len() > 1 {
                return Err("'safe' or 'all' presets for --strip should be used by themselves"
                    .to_owned());
            }
            if hdrs[0] == "safe" {
                opts.strip = Headers::Safe;
            } else {
                opts.strip = Headers::All;
            }
        } else {
            const FORBIDDEN_CHUNKS: [&'static str; 5] = ["IHDR", "IDAT", "tRNS", "PLTE", "IEND"];
            for i in &hdrs {
                if FORBIDDEN_CHUNKS.contains(&i.as_ref()) {
                    return Err(format!("{} chunk is not allowed to be stripped", i));
                }
            }
            opts.strip = Headers::Some(hdrs);
        }
    }

    if matches.is_present("strip-safe") {
        opts.strip = Headers::Safe;
    }

    if matches.is_present("zopfli") {
        opts.deflate = Deflaters::Zopfli;
    }

    if let Some(x) = matches.value_of("threads") {
        opts.threads = x.parse::<usize>().unwrap();
    }

    Ok(opts)
}

fn parse_numeric_range_opts(input: &str,
                            min_value: u8,
                            max_value: u8)
                            -> Result<HashSet<u8>, String> {
    let one_item = Regex::new(format!(r"^[{}-{}]$", min_value, max_value).as_ref()).unwrap();
    let multiple_items = Regex::new(format!(r"^([{}-{}])(,|-)([{}-{}])$",
                                            min_value,
                                            max_value,
                                            min_value,
                                            max_value)
            .as_ref())
        .unwrap();
    let mut items = HashSet::new();

    if one_item.is_match(input) {
        items.insert(input.parse::<u8>().unwrap());
        return Ok(items);
    }

    if let Some(captures) = multiple_items.captures(input) {
        let first = captures[1].parse::<u8>().unwrap();
        let second = captures[3].parse::<u8>().unwrap();
        if first >= second {
            return Err("Not a valid input".to_owned());
        }

        match &captures[2] {
            "," => {
                items.insert(first);
                items.insert(second);
            }
            "-" => {
                for i in first..second + 1 {
                    items.insert(i);
                }
            }
            _ => unreachable!(),
        };

        return Ok(items);
    }

    Err("Not a valid input".to_owned())
}
