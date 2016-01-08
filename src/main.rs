extern crate optipng;
extern crate clap;
extern crate regex;

use clap::{App, Arg, ArgMatches};
use regex::Regex;
use std::collections::HashSet;
use std::path::Path;
use std::path::PathBuf;

fn main() {
    let mut filter = HashSet::new();
    filter.insert(0);
    filter.insert(5);
    let mut compression = HashSet::new();
    compression.insert(9);
    let mut memory = HashSet::new();
    memory.insert(9);
    let mut strategies = HashSet::new();
    for i in 0..4 {
        strategies.insert(i);
    }

    let default_opts = optipng::Options {
        backup: false,
        out_file: PathBuf::new(),
        out_dir: None,
        stdout: false,
        pretend: false,
        fix_errors: false,
        clobber: true,
        create: true,
        preserve_attrs: false,
        verbosity: Some(0),
        filter: filter,
        interlace: None,
        compression: compression,
        memory: memory,
        strategies: strategies,
        window: 15,
        bit_depth_reduction: true,
        color_type_reduction: true,
        palette_reduction: true,
        idat_recoding: true,
        strip: false,
    };

    let matches = App::new("optipng")
                      .version("2.0.0-alpha.1")
                      .author("Joshua Holmer <jholmer.in@gmail.com>")
                      .about("Losslessly improves compression of PNG files")
                      .arg(Arg::with_name("files")
                               .help("File(s) to compress")
                               .index(1)
                               .multiple(true)
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
                               .long("backup"))
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
                               .help("Strip all metadata objects")
                               .long("strip"))
                      .after_help("Optimization levels:
    -o0		=>	-zc3 -nz			(0 or 1 trials)
    -o1		=>	-zc9				(1 trial)
    -o2		=>	-zc9 -zs0-3 -f0,5		(8 trials)
    -o3		=>	-zc9 -zm8-9 -zs0-3 -f0,5	(16 trials)
    -o4		=>	-zc9 -zm8-9 -zs0-3 -f0-5	(48 trials)
    -o5		=>	-zc3-9 -zm8-9 -zs0-3 -f0-5	(192 trials)
    -o6		=>	-zc1-9 -zm7-9 -zs0-3 -f0-5	(360 trials)
    -o6 -zm1-9	=>	-zc1-9 -zm1-9 -zs0-3 -f0-5	(1080 trials)

    Exhaustive combinations such as \"-o6 -zm1-9\" are not generally recommended.
    These are very slow and generally provide no compression gain.

    Manually specifying a compression option (zc, zm, etc.) will override the optimization preset,
    regardless of the order you write the arguments.")
                      .get_matches();

    let mut opts = default_opts;

    parse_opts_into_struct(&matches, &mut opts);

    for input in matches.values_of("files").unwrap() {
        opts.out_file = PathBuf::from(input);
        // TODO: Handle wildcards
        match optipng::optimize(Path::new(input), &opts) {
            Ok(_) => (),
            Err(x) => println!("{}", x),
        };
    }
}

fn parse_opts_into_struct(matches: &ArgMatches, opts: &mut optipng::Options) {
    match matches.value_of("optimization") {
        Some("0") => {
            opts.idat_recoding = false;
            let mut compression = HashSet::new();
            compression.insert(3);
            opts.compression = compression;
        }
        Some("1") => {
            let mut filter = HashSet::new();
            filter.insert(0);
            opts.filter = filter;
            let mut strategies = HashSet::new();
            strategies.insert(0);
            opts.strategies = strategies;
        }
        // 2 is the default
        Some("3") => {
            let mut filter = HashSet::new();
            filter.insert(0);
            filter.insert(5);
            opts.filter = filter;
            let mut compression = HashSet::new();
            compression.insert(9);
            opts.compression = compression;
            let mut memory = HashSet::new();
            for i in 8..10 {
                memory.insert(i);
            }
            opts.memory = memory;
            let mut strategies = HashSet::new();
            for i in 0..4 {
                strategies.insert(i);
            }
            opts.strategies = strategies;
        }
        Some("4") => {
            let mut filter = HashSet::new();
            for i in 0..6 {
                filter.insert(i);
            }
            opts.filter = filter;
            let mut compression = HashSet::new();
            compression.insert(9);
            opts.compression = compression;
            let mut memory = HashSet::new();
            for i in 8..10 {
                memory.insert(i);
            }
            opts.memory = memory;
            let mut strategies = HashSet::new();
            for i in 0..4 {
                strategies.insert(i);
            }
            opts.strategies = strategies;
        }
        Some("5") => {
            let mut filter = HashSet::new();
            for i in 0..6 {
                filter.insert(i);
            }
            opts.filter = filter;
            let mut compression = HashSet::new();
            for i in 3..10 {
                compression.insert(i);
            }
            opts.compression = compression;
            let mut memory = HashSet::new();
            for i in 8..10 {
                memory.insert(i);
            }
            opts.memory = memory;
            let mut strategies = HashSet::new();
            for i in 0..4 {
                strategies.insert(i);
            }
            opts.strategies = strategies;
        }
        Some("6") => {
            let mut filter = HashSet::new();
            for i in 0..6 {
                filter.insert(i);
            }
            opts.filter = filter;
            let mut compression = HashSet::new();
            for i in 1..10 {
                compression.insert(i);
            }
            opts.compression = compression;
            let mut memory = HashSet::new();
            for i in 7..10 {
                memory.insert(i);
            }
            opts.memory = memory;
            let mut strategies = HashSet::new();
            for i in 0..4 {
                strategies.insert(i);
            }
            opts.strategies = strategies;
        }
        _ => (), // Use default
    }

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
        opts.out_dir = Some(PathBuf::from(x));
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

    if matches.is_present("strip") {
        opts.strip = true;
    }
}

fn parse_numeric_range_opts(input: &str,
                            min_value: u8,
                            max_value: u8)
                            -> Result<HashSet<u8>, String> {
    let one_item = Regex::new(format!("^[{}-{}]$", min_value, max_value).as_ref()).unwrap();
    let multiple_items = Regex::new(format!("^([{}-{}])(,|-)([{}-{}])$",
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

        return Ok(match &captures[2] {
            "," => {
                items.insert(first);
                items.insert(second);
                return Ok(items);
            }
            "-" => {
                for i in first..second + 1 {
                    items.insert(i);
                }
                return Ok(items);
            }
            _ => panic!("Unreachable"),
        });
    }

    Err("Not a valid input".to_owned())
}
