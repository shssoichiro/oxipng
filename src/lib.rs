#![cfg_attr(feature = "dev", feature(plugin))]
#![cfg_attr(feature = "dev", plugin(clippy))]

extern crate bit_vec;
extern crate byteorder;
extern crate crc;
extern crate image;
extern crate itertools;
extern crate miniz_oxide;
extern crate num_cpus;
#[cfg(feature = "parallel")]
extern crate rayon;
extern crate zopfli;
#[cfg(feature = "cfzlib")]
extern crate cloudflare_zlib_sys;

use image::{DynamicImage, GenericImage, ImageFormat, Pixel};
use png::PngData;
#[cfg(feature = "parallel")]
use rayon::prelude::*;
use std::collections::{HashMap, HashSet};
use std::fs::{copy, File};
use std::io::{stdout, BufWriter, Write};
use std::path::{Path, PathBuf};
use atomicmin::AtomicMin;

pub use colors::AlphaOptim;
pub use deflate::Deflaters;
pub use error::PngError;
pub use headers::Headers;

#[doc(hidden)]
pub mod colors;
#[doc(hidden)]
pub mod deflate;
#[doc(hidden)]
pub mod error;
mod filters;
#[doc(hidden)]
pub mod headers;
mod interlace;
#[doc(hidden)]
pub mod png;
mod reduction;
mod atomicmin;

#[derive(Clone, Debug)]
/// Options controlling the output of the `optimize` function
pub struct Options {
    /// Whether the input file should be backed up before writing the output.
    ///
    /// Default: `false`
    pub backup: bool,
    /// Path to write the output file to. If not set, the application will default to
    /// overwriting the input file.
    pub out_file: Option<PathBuf>,
    /// Used only in CLI interface
    #[doc(hidden)]
    pub out_dir: Option<PathBuf>,
    /// Write to stdout instead of a file.
    ///
    /// Default: `false`
    pub stdout: bool,
    /// Attempt to fix errors when decoding the input file rather than returning an `Err`.
    ///
    /// Default: `false`
    pub fix_errors: bool,
    /// Don't actually write any output, just calculate the best results.
    ///
    /// Default: `false`
    pub pretend: bool,
    /// Used only in CLI interface
    #[doc(hidden)]
    pub recursive: bool,
    /// Overwrite existing output files.
    ///
    /// Default: `true`
    pub clobber: bool,
    /// Create new output files if they don't exist.
    ///
    /// Default: `true`
    pub create: bool,
    /// Write to output even if there was no improvement in compression.
    ///
    /// Default: `false`
    pub force: bool,
    /// Ensure the output file has the same permissions as the input file.
    ///
    /// Default: `false`
    pub preserve_attrs: bool,
    /// How verbose the console logging should be (`None` for quiet, `Some(0)` for normal, `Some(1)` for verbose)
    ///
    /// Default: `Some(0)`
    pub verbosity: Option<u8>,
    /// Which filters to try on the file (0-5)
    ///
    /// Default: `0,5`
    pub filter: HashSet<u8>,
    /// Whether to change the interlacing type of the file.
    ///
    /// `None` will not change the current interlacing type.
    ///
    /// `Some(x)` will change the file to interlacing mode `x`.
    ///
    /// Default: `None`
    pub interlace: Option<u8>,
    /// Which zlib compression levels to try on the file (1-9)
    ///
    /// Default: `9`
    pub compression: HashSet<u8>,
    /// Which zlib compression strategies to try on the file (0-3)
    ///
    /// Default: `0-3`
    pub strategies: HashSet<u8>,
    /// Window size to use when compressing the file, as `2^window` bytes.
    ///
    /// Doesn't affect compression but may affect speed and memory usage.
    /// 8-15 are valid values.
    ///
    /// Default: `15`
    pub window: u8,
    /// Alpha filtering strategies to use
    pub alphas: HashSet<colors::AlphaOptim>,
    /// Whether to attempt bit depth reduction
    ///
    /// Default: `true`
    pub bit_depth_reduction: bool,
    /// Whether to attempt color type reduction
    ///
    /// Default: `true`
    pub color_type_reduction: bool,
    /// Whether to attempt palette reduction
    ///
    /// Default: `true`
    pub palette_reduction: bool,
    /// Whether to perform IDAT recoding
    ///
    /// If any type of reduction is performed, IDAT recoding will be performed
    /// regardless of this setting
    ///
    /// Default: `true`
    pub idat_recoding: bool,
    /// Which headers to strip from the PNG file, if any
    ///
    /// Default: `None`
    pub strip: Headers,
    /// Which DEFLATE algorithm to use
    ///
    /// Default: `Zlib`
    pub deflate: Deflaters,
    /// Whether to use heuristics to pick the best filter and compression
    ///
    /// Intended for use with `-o 1` from the CLI interface
    ///
    /// Default: `false`
    pub use_heuristics: bool,
    /// Number of threads to use
    ///
    /// Default: 1.5x CPU cores, rounded down
    pub threads: usize,
}

impl Options {
    pub fn from_preset(level: u8) -> Options {
        let opts = Options::default();
        match level {
            0 => opts.apply_preset_0(),
            1 => opts.apply_preset_1(),
            2 => opts.apply_preset_2(),
            3 => opts.apply_preset_3(),
            4 => opts.apply_preset_4(),
            5 => opts.apply_preset_5(),
            _ => opts.apply_preset_6(),
        }
    }

    // The following methods make assumptions that they are operating
    // on an `Options` struct generated by the `default` method.
    fn apply_preset_0(mut self) -> Self {
        self.idat_recoding = false;
        self.compression.clear();
        self.compression.insert(3);
        self
    }

    fn apply_preset_1(mut self) -> Self {
        self.filter.clear();
        self.strategies.clear();
        self.use_heuristics = true;
        self
    }

    fn apply_preset_2(self) -> Self {
        self
    }

    fn apply_preset_3(mut self) -> Self {
        for i in 1..5 {
            self.filter.insert(i);
        }
        self
    }

    fn apply_preset_4(mut self) -> Self {
        self.alphas.insert(AlphaOptim::White);
        self.alphas.insert(AlphaOptim::Up);
        self.alphas.insert(AlphaOptim::Down);
        self.alphas.insert(AlphaOptim::Left);
        self.alphas.insert(AlphaOptim::Right);
        self.apply_preset_3()
    }

    fn apply_preset_5(mut self) -> Self {
        for i in 3..9 {
            self.compression.insert(i);
        }
        self.apply_preset_4()
    }

    fn apply_preset_6(mut self) -> Self {
        for i in 1..3 {
            self.compression.insert(i);
        }
        self.apply_preset_5()
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
        let mut strategies = HashSet::new();
        for i in 0..4 {
            strategies.insert(i);
        }
        let mut alphas = HashSet::new();
        alphas.insert(colors::AlphaOptim::NoOp);
        alphas.insert(colors::AlphaOptim::Black);

        // Default to 1 thread on single-core, otherwise use threads = 1.5x CPU cores
        let num_cpus = num_cpus::get();
        let thread_count = num_cpus + (num_cpus >> 1);

        Options {
            backup: false,
            out_file: None,
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
            filter,
            interlace: None,
            compression,
            strategies,
            window: 15,
            alphas,
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
pub fn optimize(input_path: &Path, opts: &Options) -> Result<(), PngError> {
    // Initialize the thread pool with correct number of threads
    #[cfg(feature = "parallel")]
    let thread_count = opts.threads;
    #[cfg(feature = "parallel")]
    let _ = rayon::ThreadPoolBuilder::new()
        .num_threads(thread_count)
        .build_global();

    // Read in the file and try to decode as PNG.
    if opts.verbosity.is_some() {
        eprintln!("Processing: {}", input_path.to_str().unwrap());
    }

    let in_data = PngData::read_file(input_path)?;
    let mut png = PngData::from_slice(&in_data, opts.fix_errors)?;
    let output_path = opts.out_file
        .clone()
        .unwrap_or_else(|| input_path.to_path_buf());

    // Run the optimizer on the decoded PNG.
    let mut optimized_output = optimize_png(&mut png, &in_data, opts)?;

    if is_fully_optimized(in_data.len(), optimized_output.len(), opts) {
        eprintln!("File already optimized");
        if input_path == output_path {
            return Ok(());
        } else {
            optimized_output = in_data;
        }
    }

    if opts.pretend {
        if opts.verbosity.is_some() {
            eprintln!("Running in pretend mode, no output");
        }
    } else {
        if opts.backup {
            perform_backup(input_path)?;
        }

        if opts.stdout {
            let mut buffer = BufWriter::new(stdout());
            match buffer.write_all(&optimized_output) {
                Ok(_) => (),
                Err(_) => return Err(PngError::new("Unable to write to stdout")),
            }
        } else {
            let out_file = match File::create(&output_path) {
                Ok(x) => x,
                Err(_) => {
                    return Err(PngError::new(&format!(
                        "Unable to write to file {}",
                        output_path.display()
                    )))
                }
            };

            if opts.preserve_attrs {
                copy_permissions(input_path, &out_file, opts.verbosity);
            }

            let mut buffer = BufWriter::new(out_file);
            match buffer.write_all(&optimized_output) {
                Ok(_) => if opts.verbosity.is_some() {
                    eprintln!("Output: {}", output_path.display());
                },
                Err(_) => {
                    return Err(PngError::new(&format!(
                        "Unable to write to file {}",
                        output_path.display()
                    )))
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
    #[cfg(feature = "parallel")]
    let thread_count = opts.threads;
    #[cfg(feature = "parallel")]
    let _ = rayon::ThreadPoolBuilder::new()
        .num_threads(thread_count)
        .build_global();

    // Read in the file and try to decode as PNG.
    if opts.verbosity.is_some() {
        eprintln!("Processing from memory");
    }
    let original_size = data.len() as usize;
    let mut png = PngData::from_slice(data, opts.fix_errors)?;

    // Run the optimizer on the decoded PNG.
    let optimized_output = optimize_png(&mut png, data, opts)?;

    if is_fully_optimized(original_size, optimized_output.len(), opts) {
        eprintln!("Image already optimized");
        Ok(data.to_vec())
    } else {
        Ok(optimized_output)
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
/// Defines options to be used for a single compression trial
struct TrialOptions {
    pub filter: u8,
    pub compression: u8,
    pub strategy: u8,
}

/// Perform optimization on the input PNG object using the options provided
fn optimize_png(
    png: &mut PngData,
    original_data: &[u8],
    opts: &Options,
) -> Result<Vec<u8>, PngError> {
    type TrialWithData = (TrialOptions, Vec<u8>);

    let original_png = png.clone();

    // Print png info
    let file_original_size = original_data.len();
    let idat_original_size = png.idat_data.len();
    if opts.verbosity.is_some() {
        eprintln!(
            "    {}x{} pixels, PNG format",
            png.ihdr_data.width, png.ihdr_data.height
        );
        if let Some(ref palette) = png.palette {
            eprintln!(
                "    {} bits/pixel, {} colors in palette",
                png.ihdr_data.bit_depth,
                palette.len() / 3
            );
        } else {
            eprintln!(
                "    {}x{} bits/pixel, {:?}",
                png.channels_per_pixel(),
                png.ihdr_data.bit_depth,
                png.ihdr_data.color_type
            );
        }
        eprintln!("    IDAT size = {} bytes", idat_original_size);
        eprintln!("    File size = {} bytes", file_original_size);
    }

    let mut filter = opts.filter.iter().cloned().collect::<Vec<u8>>();
    let compression = &opts.compression;
    let mut strategies = opts.strategies.clone();

    if opts.use_heuristics {
        // Heuristically determine which set of options to use
        if png.ihdr_data.bit_depth.as_u8() >= 8
            && png.ihdr_data.color_type != colors::ColorType::Indexed
        {
            if filter.is_empty() {
                filter.push(5);
            }
            if strategies.is_empty() {
                strategies.insert(1);
            }
        } else {
            if filter.is_empty() {
                filter.push(0);
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
            filter.len() * compression.len() * strategies.len()
        } else {
            filter.len()
        };
        let mut results: Vec<TrialOptions> = Vec::with_capacity(combinations);
        if opts.verbosity.is_some() {
            eprintln!("Trying: {} combinations", combinations);
        }

        for f in &filter {
            if opts.deflate == Deflaters::Zlib {
                for zc in compression {
                    for zs in &strategies {
                        results.push(TrialOptions {
                            filter: *f,
                            compression: *zc,
                            strategy: *zs,
                        });
                    }
                }
            } else {
                // Zopfli compression has no additional options
                results.push(TrialOptions {
                    filter: *f,
                    compression: 0,
                    strategy: 0,
                });
            }
        }

        #[cfg(feature = "parallel")]
        let filter_iter = filter.par_iter().with_max_len(1);
        #[cfg(not(feature = "parallel"))]
        let filter_iter = filter.iter();
        let filters: HashMap<u8, Vec<u8>> = filter_iter
            .map(|f| {
                let png = png.clone();
                (*f, png.filter_image(*f))
            })
            .collect();

        let original_len = original_png.idat_data.len();
        let added_interlacing = opts.interlace == Some(1) && original_png.ihdr_data.interlaced == 0;

        let best_size = AtomicMin::new(if opts.force {None} else {Some(original_len)});
        #[cfg(feature = "parallel")]
        let results_iter = results.into_par_iter().with_max_len(1);
        #[cfg(not(feature = "parallel"))]
        let results_iter = results.into_iter();
        let best = results_iter
            .filter_map(|trial| {
                let filtered = &filters[&trial.filter];
                let new_idat = if opts.deflate == Deflaters::Zlib {
                    deflate::deflate(filtered, trial.compression, trial.strategy, opts.window, best_size.get())
                } else {
                    deflate::zopfli_deflate(filtered)
                };
                let new_idat = match new_idat {
                    Ok(n) => n,
                    Err(PngError::DeflatedDataTooLong(max)) if opts.verbosity == Some(1) => {
                        eprintln!(
                            "    zc = {}  zs = {}  f = {}       >{} bytes",
                            trial.compression,
                            trial.strategy,
                            trial.filter,
                            max,
                        );
                        return None;
                    },
                    _ => return None,
                };

                // update best size across all threads
                let new_size = new_idat.len();
                best_size.set_min(new_size);

                if opts.verbosity == Some(1) {
                    eprintln!(
                        "    zc = {}  zs = {}  f = {}        {} bytes",
                        trial.compression,
                        trial.strategy,
                        trial.filter,
                        new_idat.len()
                    );
                }

                if new_size < original_len || added_interlacing || opts.force {
                    Some((trial, new_idat))
                } else {
                    None
                }
            });
        #[cfg(feature = "parallel")]
        let best: Option<TrialWithData> = best
            .reduce_with(|i, j| if i.1.len() <= j.1.len() { i } else { j });
        #[cfg(not(feature = "parallel"))]
        let best: Option<TrialWithData> = best.fold(None, |i, j| {
                if let Some(i) = i {
                    if i.1.len() <= j.1.len() { Some(i) } else { Some(j) }
                } else {
                    Some(j)
                }
            });

        if let Some(better) = best {
            png.idat_data = better.1;
            if opts.verbosity.is_some() {
                let opts = better.0;
                eprintln!("Found better combination:");
                eprintln!(
                    "    zc = {}  zs = {}  f = {}        {} bytes",
                    opts.compression,
                    opts.strategy,
                    opts.filter,
                    png.idat_data.len()
                );
            }
        } else if reduction_occurred {
            png.reset_from_original(&original_png);
        }
    }

    perform_strip(png, opts);

    let output = png.output();

    if opts.verbosity.is_some() {
        if idat_original_size >= png.idat_data.len() {
            eprintln!(
                "    IDAT size = {} bytes ({} bytes decrease)",
                png.idat_data.len(),
                idat_original_size - png.idat_data.len()
            );
        } else {
            eprintln!(
                "    IDAT size = {} bytes ({} bytes increase)",
                png.idat_data.len(),
                png.idat_data.len() - idat_original_size
            );
        }
        if file_original_size >= output.len() {
            eprintln!(
                "    file size = {} bytes ({} bytes = {:.2}% decrease)",
                output.len(),
                file_original_size - output.len(),
                (file_original_size - output.len()) as f64 / file_original_size as f64 * 100f64
            );
        } else {
            eprintln!(
                "    file size = {} bytes ({} bytes = {:.2}% increase)",
                output.len(),
                output.len() - file_original_size,
                (output.len() - file_original_size) as f64 / file_original_size as f64 * 100f64
            );
        }
    }

    let old_png = image::load_from_memory_with_format(original_data, ImageFormat::PNG);
    let new_png = image::load_from_memory_with_format(&output, ImageFormat::PNG);

    if let Ok(new_png) = new_png {
        if let Ok(old_png) = old_png {
            if images_equal(&old_png, &new_png) {
                return Ok(output);
            }
        } else {
            // The original image might be invalid if, for example, there is a CRC error,
            // and we set fix_errors to true. In that case, all we can do is check that the
            // new image is decodable.
            return Ok(output);
        }
    }

    eprintln!(
        "The resulting image is corrupted and will not be outputted.\nThis is a bug! Please report it at https://github.com/shssoichiro/oxipng/issues"
    );
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

    png.try_alpha_reduction(&opts.alphas);

    reduction_occurred
}

/// Display the status of the image data after a reduction has taken place
fn report_reduction(png: &png::PngData) {
    if let Some(ref palette) = png.palette {
        eprintln!(
            "Reducing image to {} bits/pixel, {} colors in palette",
            png.ihdr_data.bit_depth,
            palette.len() / 3
        );
    } else {
        eprintln!(
            "Reducing image to {}x{} bits/pixel, {}",
            png.channels_per_pixel(),
            png.ihdr_data.bit_depth,
            png.ihdr_data.color_type
        );
    }
}

/// Strip headers from the `PngData` object, as requested by the passed `Options`
fn perform_strip(png: &mut png::PngData, opts: &Options) {
    match opts.strip {
        // Strip headers
        Headers::None => (),
        Headers::Some(ref hdrs) => for hdr in hdrs {
            png.aux_headers.remove(hdr);
        },
        Headers::Safe => {
            const PRESERVED_HEADERS: [&str; 9] = [
                "cHRM", "gAMA", "iCCP", "sBIT", "sRGB", "bKGD", "hIST", "pHYs", "sPLT"
            ];
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
fn is_fully_optimized(original_size: usize, optimized_size: usize, opts: &Options) -> bool {
    original_size <= optimized_size && !opts.force && opts.interlace.is_none()
}

fn perform_backup(input_path: &Path) -> Result<(), PngError> {
    let backup_file = input_path.with_extension(format!(
        "bak.{}",
        input_path.extension().unwrap().to_str().unwrap()
    ));
    copy(input_path, &backup_file).map(|_| ()).map_err(|_| {
        PngError::new(&format!(
            "Unable to write to backup file at {}",
            backup_file.display()
        ))
    })
}

#[cfg(not(unix))]
fn copy_permissions(input_path: &Path, out_file: &File, verbosity: Option<u8>) {
    if let Ok(f) = File::open(input_path) {
        if let Ok(metadata) = f.metadata() {
            if let Ok(out_meta) = out_file.metadata() {
                let readonly = metadata.permissions().readonly();
                out_meta.permissions().set_readonly(readonly);
                return;
            }
        }
    };
    if verbosity.is_some() {
        eprintln!("Failed to set permissions on output file");
    }
}

#[cfg(unix)]
fn copy_permissions(input_path: &Path, out_file: &File, verbosity: Option<u8>) {
    use std::os::unix::fs::PermissionsExt;

    if let Ok(f) = File::open(input_path) {
        if let Ok(metadata) = f.metadata() {
            if let Ok(out_meta) = out_file.metadata() {
                let permissions = metadata.permissions().mode();
                out_meta.permissions().set_mode(permissions);
                return;
            }
        }
    };
    if verbosity.is_some() {
        eprintln!("Failed to set permissions on output file");
    }
}

/// Compares images pixel by pixel for equivalent content
fn images_equal(old_png: &DynamicImage, new_png: &DynamicImage) -> bool {
    let a = old_png.pixels()
        .map(|x| x.2.channels().to_owned())
        .filter(|p| !(p.len() == 4 && p[3] == 0));
    let b = new_png.pixels()
        .map(|x| x.2.channels().to_owned())
        .filter(|p| !(p.len() == 4 && p[3] == 0));
    a.eq(b)
}
