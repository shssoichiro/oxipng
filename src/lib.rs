#![cfg_attr(feature="dev", feature(plugin))]
#![cfg_attr(feature="dev", plugin(clippy))]

extern crate bit_vec;
extern crate byteorder;
extern crate crc;
extern crate image;
extern crate itertools;
extern crate libc;
extern crate miniz_sys;
extern crate num_cpus;
extern crate rayon;
extern crate zopfli;

use deflate::Deflaters;
use error::PngError;
use image::{GenericImage, Pixel, ImageFormat};
use headers::Headers;
use png::PngData;
use rayon::prelude::*;
use std::collections::{HashMap, HashSet};
use std::fs::{File, copy};
use std::io::{BufWriter, Write, stderr, stdout};
use std::path::{Path, PathBuf};

pub mod colors;
pub mod deflate;
mod error;
mod filters;
pub mod headers;
mod interlace;
pub mod png;
mod reduction;

#[derive(Clone,Debug)]
/// Options controlling the output of the `optimize` function
pub struct Options {
    /// Whether the input file should be backed up before writing the output
    /// Default: `false`
    pub backup: bool,
    /// Path to write the output file to
    pub out_file: PathBuf,
    /// Used only in CLI interface
    pub out_dir: Option<PathBuf>,
    /// Write to stdout instead of a file
    /// Default: `false`
    pub stdout: bool,
    /// Attempt to fix errors when decoding the input file rather than returning an `Err`
    /// Default: `false`
    pub fix_errors: bool,
    /// Don't actually write any output, just calculate the best results
    /// Default: `false`
    pub pretend: bool,
    /// Used only in CLI interface
    pub recursive: bool,
    /// Overwrite existing output files
    /// Default: `true`
    pub clobber: bool,
    /// Create new output files if they don't exist
    /// Default: `true`
    pub create: bool,
    /// Write to output even if there was no improvement in compression
    /// Default: `false`
    pub force: bool,
    /// Ensure the output file has the same permissions as the input file
    /// Default: `false`
    pub preserve_attrs: bool,
    /// How verbose the console logging should be (`None` for quiet, `Some(0)` for normal, `Some(1)` for verbose)
    /// Default: `Some(0)`
    pub verbosity: Option<u8>,
    /// Which filters to try on the file (0-5)
    /// Default: `0,5`
    pub filter: HashSet<u8>,
    /// Whether to change the interlacing type of the file
    /// `None` will not change the current interlacing type
    /// `Some(x)` will change the file to interlacing mode `x`
    /// Default: `None`
    pub interlace: Option<u8>,
    /// Which zlib compression levels to try on the file (1-9)
    /// Default: `9`
    pub compression: HashSet<u8>,
    /// Which zlib memory levels to try on the file (1-9)
    /// Default: `9`
    pub memory: HashSet<u8>,
    /// Which zlib compression strategies to try on the file (0-3)
    /// Default: `0-3`
    pub strategies: HashSet<u8>,
    /// Window size to use when compressing the file, as `2^window` bytes
    /// Doesn't affect compression but may affect speed and memory usage
    /// 8-15 are valid values
    /// Default: `15`
    pub window: u8,
    /// Whether to attempt bit depth reduction
    /// Default: `true`
    pub bit_depth_reduction: bool,
    /// Whether to attempt color type reduction
    /// Default: `true`
    pub color_type_reduction: bool,
    /// Whether to attempt palette reduction
    /// Default: `true`
    pub palette_reduction: bool,
    /// Whether to perform IDAT recoding
    /// If any type of reduction is performed, IDAT recoding will be performed
    /// regardless of this setting
    /// Default: `true`
    pub idat_recoding: bool,
    /// Which headers to strip from the PNG file, if any
    /// Default: `None`
    pub strip: Headers,
    /// Which DEFLATE algorithm to use
    /// Default: `Zlib`
    pub deflate: Deflaters,
    /// Whether to use heuristics to pick the best filter and compression
    /// Intended for use with `-o 1` from the CLI interface
    /// Default: `false`
    pub use_heuristics: bool,
    /// Number of threads to use
    /// Default: 1.5x CPU cores, rounded down
    pub threads: usize,
}

impl Options {
    pub fn from_preset(level: u8) -> Options {
        let mut opts = Options::default();
        match level {
            0 => {
                opts.idat_recoding = false;
                let mut compression = HashSet::new();
                compression.insert(3);
                opts.compression = compression;
            }
            1 => {
                let filter = HashSet::new();
                opts.filter = filter;
                let strategies = HashSet::new();
                opts.strategies = strategies;
                opts.use_heuristics = true;
            }
            // 2 is the default
            3 => {
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
            4 => {
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
            5 => {
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
            // Level 6
            // If higher than 6, assume 6
            _ => {
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
        }
        opts
    }
}

impl Default for Options {
    fn default() -> Options {
        // Default settings based on -o 2 from the CLI interface
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

        // Default to 1 thread on single-core, otherwise use threads = 1.5x CPU cores
        let num_cpus = num_cpus::get();
        let thread_count = num_cpus + (num_cpus >> 1);

        Options {
            backup: false,
            out_file: PathBuf::new(),
            out_dir: None,
            stdout: false,
            pretend: false,
            recursive: false,
            fix_errors: false,
            clobber: true,
            create: true,
            force: false,
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
            strip: Headers::None,
            deflate: Deflaters::Zlib,
            use_heuristics: false,
            threads: thread_count,
        }
    }
}

/// Perform optimization on the input file using the options provided
pub fn optimize(filepath: &Path, opts: &Options) -> Result<(), PngError> {
    // Initialize the thread pool with correct number of threads
    let thread_count = opts.threads;
    rayon::initialize(rayon::Configuration::new().set_num_threads(thread_count)).ok();

    // Read in the file and try to decode as PNG.
    if opts.verbosity.is_some() {
        writeln!(&mut stderr(), "Processing: {}", filepath.to_str().unwrap()).ok();
    }

    let in_file = Path::new(filepath);
    let in_data = PngData::read_file(in_file)?;
    let mut png = PngData::from_slice(&in_data, opts.fix_errors)?;

    // Run the optimizer on the decoded PNG.
    let optimized_output = optimize_png(&mut png, &in_data, opts)?;

    if is_fully_optimized(in_data.len(), optimized_output.len(), opts) {
        writeln!(&mut stderr(), "File already optimized").ok();
        return Ok(());
    }

    if opts.pretend {
        if opts.verbosity.is_some() {
            writeln!(&mut stderr(), "Running in pretend mode, no output").ok();
        }
    } else {
        if opts.backup {
            match copy(in_file,
                       in_file.with_extension(format!("bak.{}",
                                                      in_file.extension()
                                                          .unwrap()
                                                          .to_str()
                                                          .unwrap()))) {
                Ok(x) => x,
                Err(_) => {
                    return Err(PngError::new(&format!("Unable to write to backup file at {}",
                                                      opts.out_file.display())))
                }
            };
        }

        if opts.stdout {
            let mut buffer = BufWriter::new(stdout());
            match buffer.write_all(&optimized_output) {
                Ok(_) => (),
                Err(_) => return Err(PngError::new("Unable to write to stdout")),
            }
        } else {
            let out_file = match File::create(&opts.out_file) {
                Ok(x) => x,
                Err(_) => {
                    return Err(PngError::new(&format!("Unable to write to file {}",
                                                      opts.out_file.display())))
                }
            };

            if opts.preserve_attrs {
                match File::open(filepath) {
                    Ok(f) => {
                        match f.metadata() {
                            Ok(metadata) => {
                                // TODO: Implement full permission changing on Unix
                                // Not available in stable, requires block cfg statements
                                // See https://github.com/rust-lang/rust/issues/15701
                                {
                                    match out_file.metadata() {
                                        Ok(out_meta) => {
                                            let readonly = metadata.permissions()
                                                .readonly();
                                            out_meta.permissions()
                                                .set_readonly(readonly);
                                        }
                                        Err(_) => {
                                            if opts.verbosity.is_some() {
                                                writeln!(&mut stderr(),
                                                         "Failed to set permissions on output file")
                                                    .ok();
                                            }
                                        }
                                    }
                                }
                            }
                            Err(_) => {
                                if opts.verbosity.is_some() {
                                    writeln!(&mut stderr(),
                                             "Failed to read permissions on input file")
                                        .ok();
                                }
                            }
                        }
                    }
                    Err(_) => {
                        if opts.verbosity.is_some() {
                            writeln!(&mut stderr(), "Failed to read permissions on input file")
                                .ok();
                        }
                    }
                };
            }

            let mut buffer = BufWriter::new(out_file);
            match buffer.write_all(&optimized_output) {
                Ok(_) => {
                    if opts.verbosity.is_some() {
                        writeln!(&mut stderr(), "Output: {}", opts.out_file.display()).ok();
                    }
                }
                Err(_) => {
                    return Err(PngError::new(&format!("Unable to write to file {}",
                                                      opts.out_file.display())))
                }
            }
        }
    }
    Ok(())
}

/// Perform optimization on the input file using the options provided, where the file is already
/// loaded in-memory
pub fn optimize_from_memory(data: &[u8], opts: &Options) -> Result<Vec<u8>, PngError> {
    // Initialize the thread pool with correct number of threads
    let thread_count = opts.threads;
    rayon::initialize(rayon::Configuration::new().set_num_threads(thread_count)).ok();

    // Read in the file and try to decode as PNG.
    if opts.verbosity.is_some() {
        writeln!(&mut stderr(), "Processing from memory").ok();
    }
    let original_size = data.len() as usize;
    let mut png = PngData::from_slice(data, opts.fix_errors)?;

    // Run the optimizer on the decoded PNG.
    let optimized_output = optimize_png(&mut png, data, opts)?;

    if is_fully_optimized(original_size, optimized_output.len(), opts) {
        writeln!(&mut stderr(), "Image already optimized").ok();
        Ok(data.to_vec())
    } else {
        Ok(optimized_output)
    }
}

/// Perform optimization on the input PNG object using the options provided
fn optimize_png(png: &mut PngData,
                original_data: &[u8],
                opts: &Options)
                -> Result<Vec<u8>, PngError> {
    type TrialWithData = (u8, u8, u8, u8, Vec<u8>);

    let original_png = png.clone();

    // Print png info
    let file_original_size = original_data.len();
    let idat_original_size = png.idat_data.len();
    if opts.verbosity.is_some() {
        writeln!(&mut stderr(),
                 "    {}x{} pixels, PNG format",
                 png.ihdr_data.width,
                 png.ihdr_data.height)
            .ok();
        if let Some(ref palette) = png.palette {
            writeln!(&mut stderr(),
                     "    {} bits/pixel, {} colors in palette",
                     png.ihdr_data.bit_depth,
                     palette.len() / 3)
                .ok();
        } else {
            writeln!(&mut stderr(),
                     "    {}x{} bits/pixel, {:?}",
                     png.channels_per_pixel(),
                     png.ihdr_data.bit_depth,
                     png.ihdr_data.color_type)
                .ok();
        }
        writeln!(&mut stderr(),
                 "    IDAT size = {} bytes",
                 idat_original_size)
            .ok();
        writeln!(&mut stderr(),
                 "    File size = {} bytes",
                 file_original_size)
            .ok();
    }

    let mut filter = opts.filter.clone();
    let compression = &opts.compression;
    let memory = &opts.memory;
    let mut strategies = opts.strategies.clone();

    if opts.use_heuristics {
        // Heuristically determine which set of options to use
        if png.ihdr_data.bit_depth.as_u8() >= 8 &&
           png.ihdr_data.color_type != colors::ColorType::Indexed {
            if filter.is_empty() {
                filter.insert(5);
            }
            if strategies.is_empty() {
                strategies.insert(1);
            }
        } else {
            if filter.is_empty() {
                filter.insert(0);
            }
            if strategies.is_empty() {
                strategies.insert(0);
            }
        }
    }

    let reduction_occurred = perform_reductions(png, opts);

    if opts.idat_recoding || reduction_occurred {
        // Go through selected permutations and determine the best
        let combinations = if opts.deflate == Deflaters::Zlib {
            filter.len() * compression.len() * memory.len() * strategies.len()
        } else {
            filter.len()
        };
        let mut results: Vec<(u8, u8, u8, u8)> = Vec::with_capacity(combinations);
        if opts.verbosity.is_some() {
            writeln!(&mut stderr(), "Trying: {} combinations", combinations).ok();
        }

        for f in &filter {
            if opts.deflate == Deflaters::Zlib {
                for zc in compression {
                    for zm in memory {
                        for zs in &strategies {
                            results.push((*f, *zc, *zm, *zs));
                        }
                    }
                }
            } else {
                // Zopfli compression has no additional options
                results.push((*f, 0, 0, 0));
            }
        }

        let mut filters_tmp: Vec<(u8, Vec<u8>)> = Vec::with_capacity(filter.len());
        filter.par_iter()
            .weight_max()
            .map(|f| (*f, png.filter_image(*f)))
            .collect_into(&mut filters_tmp);

        let filters: HashMap<u8, Vec<u8>> = filters_tmp.into_iter().collect();

        let original_len = original_png.idat_data.len();
        let added_interlacing = opts.interlace == Some(1) && original_png.ihdr_data.interlaced == 0;

        let best: Option<TrialWithData> = results.into_par_iter()
            .weight_max()
            .filter_map(|trial| {
                let filtered = &filters[&trial.0];
                let new_idat = if opts.deflate == Deflaters::Zlib {
                        deflate::deflate(filtered, trial.1, trial.2, trial.3, opts.window)
                    } else {
                        deflate::zopfli_deflate(filtered)
                    }
                    .unwrap();

                if opts.verbosity == Some(1) {
                    writeln!(&mut stderr(),
                             "    zc = {}  zm = {}  zs = {}  f = {}        {} bytes",
                             trial.1,
                             trial.2,
                             trial.3,
                             trial.0,
                             new_idat.len())
                        .ok();
                }

                if new_idat.len() < original_len || added_interlacing || opts.force {
                    Some((trial.0, trial.1, trial.2, trial.3, new_idat))
                } else {
                    None
                }
            })
            .reduce_with(|i, j| if i.4.len() <= j.4.len() { i } else { j });

        if let Some(better) = best {
            png.idat_data = better.4;
            if opts.verbosity.is_some() {
                writeln!(&mut stderr(), "Found better combination:").ok();
                writeln!(&mut stderr(),
                         "    zc = {}  zm = {}  zs = {}  f = {}        {} bytes",
                         better.1,
                         better.2,
                         better.3,
                         better.0,
                         png.idat_data.len())
                    .ok();
            }
        } else if reduction_occurred {
            png.reset_from_original(original_png);
        }
    }

    perform_strip(png, opts);

    let output = png.output();

    if opts.verbosity.is_some() {
        if idat_original_size >= png.idat_data.len() {
            writeln!(&mut stderr(),
                     "    IDAT size = {} bytes ({} bytes decrease)",
                     png.idat_data.len(),
                     idat_original_size - png.idat_data.len())
                .ok();
        } else {
            writeln!(&mut stderr(),
                     "    IDAT size = {} bytes ({} bytes increase)",
                     png.idat_data.len(),
                     png.idat_data.len() - idat_original_size)
                .ok();
        }
        if file_original_size >= output.len() {
            writeln!(&mut stderr(),
                     "    file size = {} bytes ({} bytes = {:.2}% decrease)",
                     output.len(),
                     file_original_size - output.len(),
                     (file_original_size - output.len()) as f64 / file_original_size as f64 *
                     100f64)
                .ok();
        } else {
            writeln!(&mut stderr(),
                     "    file size = {} bytes ({} bytes = {:.2}% increase)",
                     output.len(),
                     output.len() - file_original_size,
                     (output.len() - file_original_size) as f64 / file_original_size as f64 *
                     100f64)
                .ok();
        }
    }

    let old_png = image::load_from_memory_with_format(original_data, ImageFormat::PNG);
    let new_png = image::load_from_memory_with_format(&output, ImageFormat::PNG);

    if let Ok(new_png) = new_png {
        if let Ok(old_png) = old_png {
            if old_png.pixels().map(|x| x.2.channels().to_owned()).collect::<Vec<Vec<u8>>>() ==
               new_png.pixels().map(|x| x.2.channels().to_owned()).collect::<Vec<Vec<u8>>>() {
                return Ok(output);
            }
        } else {
            // The original image might be invalid if, for example, there is a CRC error,
            // and we set fix_errors to true. In that case, all we can do is check that the
            // new image is decodable.
            return Ok(output);
        }
    }

    writeln!(&mut stderr(), "The resulting image is corrupted and will not be outputted.\nThis is a bug! Please report it at https://github.com/shssoichiro/oxipng/issues").ok();
    Err(PngError::new("The resulting image is corrupted"))
}

/// Attempt all reduction operations requested by the given `Options` struct
/// and apply them directly to the `PngData` passed in
fn perform_reductions(png: &mut png::PngData, opts: &Options) -> bool {
    let mut reduction_occurred = false;

    if opts.palette_reduction && png.reduce_palette() {
        reduction_occurred = true;
        if opts.verbosity == Some(1) {
            report_reduction(png);
        }
    }

    if opts.bit_depth_reduction && png.reduce_bit_depth() {
        reduction_occurred = true;
        if opts.verbosity == Some(1) {
            report_reduction(png);
        }
    }

    if opts.color_type_reduction && png.reduce_color_type() {
        reduction_occurred = true;
        if opts.verbosity == Some(1) {
            report_reduction(png);
        }
    }

    if reduction_occurred && opts.verbosity.is_some() {
        report_reduction(png);
    }

    if let Some(interlacing) = opts.interlace {
        if png.change_interlacing(interlacing) {
            png.ihdr_data.interlaced = interlacing;
            reduction_occurred = true;
        }
    }

    reduction_occurred
}

/// Display the status of the image data after a reduction has taken place
#[inline]
fn report_reduction(png: &png::PngData) {
    if let Some(ref palette) = png.palette {
        writeln!(&mut stderr(),
                 "Reducing image to {} bits/pixel, {} colors in palette",
                 png.ihdr_data.bit_depth,
                 palette.len() / 3)
            .ok();
    } else {
        writeln!(&mut stderr(),
                 "Reducing image to {}x{} bits/pixel, {}",
                 png.channels_per_pixel(),
                 png.ihdr_data.bit_depth,
                 png.ihdr_data.color_type)
            .ok();
    }
}

/// Strip headers from the `PngData` object, as requested by the passed `Options`
fn perform_strip(png: &mut png::PngData, opts: &Options) {
    match opts.strip {
        // Strip headers
        Headers::None => (),
        Headers::Some(ref hdrs) => {
            for hdr in hdrs {
                png.aux_headers.remove(hdr);
            }
        }
        Headers::Safe => {
            const PRESERVED_HEADERS: [&'static str; 9] = ["cHRM", "gAMA", "iCCP", "sBIT", "sRGB",
                                                          "bKGD", "hIST", "pHYs", "sPLT"];
            let hdrs = png.aux_headers.keys().cloned().collect::<Vec<String>>();
            for hdr in hdrs {
                if !PRESERVED_HEADERS.contains(&hdr.as_ref()) {
                    png.aux_headers.remove(&hdr);
                }
            }
        }
        Headers::All => {
            png.aux_headers = HashMap::new();
        }
    }
}

/// Check if an image was already optimized prior to oxipng's operations
#[inline]
fn is_fully_optimized(original_size: usize, optimized_size: usize, opts: &Options) -> bool {
    original_size <= optimized_size && !opts.force && opts.interlace.is_none()
}
