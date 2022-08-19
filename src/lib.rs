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
#![allow(clippy::upper_case_acronyms)]
#![cfg_attr(
    not(any(feature = "libdeflater", feature = "zopfli")),
    allow(irrefutable_let_patterns),
    allow(unreachable_patterns)
)]

#[cfg(feature = "parallel")]
extern crate rayon;
#[cfg(not(feature = "parallel"))]
mod rayon;

use crate::atomicmin::AtomicMin;
use crate::colors::BitDepth;
use crate::deflate::inflate;
use crate::evaluate::Evaluator;
use crate::png::PngData;
use crate::png::PngImage;
use crate::reduction::*;
use crc::{Crc, CRC_32_ISO_HDLC};
use image::{DynamicImage, GenericImageView, ImageFormat, Pixel};
use log::{debug, error, info, warn};
use rayon::prelude::*;
use std::fmt;
use std::fs::{copy, File, Metadata};
use std::io::{stdin, stdout, BufWriter, Read, Write, Cursor};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

pub use crate::colors::AlphaOptim;
pub use crate::deflate::Deflaters;
pub use crate::error::PngError;
pub use crate::headers::Headers;
pub use indexmap::{IndexMap, IndexSet};

mod atomicmin;
mod colors;
mod deflate;
mod error;
mod evaluate;
mod filters;
mod headers;
mod interlace;
mod png;
mod reduction;

/// Private to oxipng; don't use outside tests and benches
#[doc(hidden)]
pub mod internal_tests {
    pub use crate::atomicmin::*;
    pub use crate::colors::*;
    pub use crate::deflate::*;
    pub use crate::headers::*;
    pub use crate::png::*;
    pub use crate::reduction::*;
}

#[derive(Clone, Debug)]
pub enum OutFile {
    /// Path(None) means same as input
    Path(Option<PathBuf>),
    StdOut,
}

impl OutFile {
    pub fn path(&self) -> Option<&Path> {
        match *self {
            OutFile::Path(Some(ref p)) => Some(p.as_path()),
            _ => None,
        }
    }
}

/// Where to read images from
#[derive(Clone, Debug)]
pub enum InFile {
    Path(PathBuf),
    StdIn,
}

impl InFile {
    pub fn path(&self) -> Option<&Path> {
        match *self {
            InFile::Path(ref p) => Some(p.as_path()),
            InFile::StdIn => None,
        }
    }
}

impl fmt::Display for InFile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            InFile::Path(ref p) => write!(f, "{}", p.display()),
            InFile::StdIn => f.write_str("stdin"),
        }
    }
}

impl<T: Into<PathBuf>> From<T> for InFile {
    fn from(s: T) -> Self {
        InFile::Path(s.into())
    }
}

pub type PngResult<T> = Result<T, PngError>;

#[derive(Clone, Debug)]
/// Options controlling the output of the `optimize` function
pub struct Options {
    /// Whether the input file should be backed up before writing the output.
    ///
    /// Default: `false`
    pub backup: bool,
    /// Attempt to fix errors when decoding the input file rather than returning an `Err`.
    ///
    /// Default: `false`
    pub fix_errors: bool,
    /// Don't actually run any optimizations, just parse the PNG file.
    ///
    /// Default: `false`
    pub check: bool,
    /// Don't actually write any output, just calculate the best results.
    ///
    /// Default: `false`
    pub pretend: bool,
    /// Write to output even if there was no improvement in compression.
    ///
    /// Default: `false`
    pub force: bool,
    /// Ensure the output file has the same permissions as the input file.
    ///
    /// Default: `false`
    pub preserve_attrs: bool,
    /// Which filters to try on the file (0-5)
    ///
    /// Default: `0,5`
    pub filter: IndexSet<u8>,
    /// Whether to change the interlacing type of the file.
    ///
    /// `None` will not change the current interlacing type.
    ///
    /// `Some(x)` will change the file to interlacing mode `x`.
    ///
    /// Default: `None`
    pub interlace: Option<u8>,
    /// Alpha filtering strategies to use
    pub alphas: IndexSet<colors::AlphaOptim>,
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
    /// Whether to attempt grayscale reduction
    ///
    /// Default: `true`
    pub grayscale_reduction: bool,
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

    /// Maximum amount of time to spend on optimizations.
    /// Further potential optimizations are skipped if the timeout is exceeded.
    pub timeout: Option<Duration>,
}

impl Options {
    pub fn from_preset(level: u8) -> Options {
        let opts = Options::default();
        match level {
            0 => opts.apply_preset_0(),
            1 => opts.apply_preset_1(),
            2 => opts.apply_preset_2(),
            3 => opts.apply_preset_3(),
            4 => {
                warn!("Level 4 is deprecated and is identical to level 3");
                opts.apply_preset_4()
            }
            5 => opts.apply_preset_5(),
            6 => opts.apply_preset_6(),
            _ => {
                warn!("Level 7 and above don't exist yet and are identical to level 6");
                opts.apply_preset_6()
            }
        }
    }

    pub fn max_compression() -> Options {
        Options::from_preset(6)
    }

    // The following methods make assumptions that they are operating
    // on an `Options` struct generated by the `default` method.
    fn apply_preset_0(mut self) -> Self {
        self.idat_recoding = false;
        if let Deflaters::Zlib { compression, .. } = &mut self.deflate {
            compression.clear();
            compression.insert(3);
        }
        self
    }

    fn apply_preset_1(mut self) -> Self {
        self.filter.clear();
        if let Deflaters::Zlib { strategies, .. } = &mut self.deflate {
            strategies.clear();
        }
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

    fn apply_preset_4(self) -> Self {
        self.apply_preset_3()
    }

    fn apply_preset_5(mut self) -> Self {
        if let Deflaters::Zlib { compression, .. } = &mut self.deflate {
            compression.clear();
            for i in 3..10 {
                compression.insert(i);
            }
        }
        self.apply_preset_4()
    }

    fn apply_preset_6(mut self) -> Self {
        if let Deflaters::Zlib { compression, .. } = &mut self.deflate {
            compression.clear();
            for i in 1..10 {
                compression.insert(i);
            }
        }
        self.apply_preset_4()
    }
}

impl Default for Options {
    fn default() -> Options {
        // Default settings based on -o 2 from the CLI interface
        let mut filter = IndexSet::new();
        filter.insert(0);
        filter.insert(5);
        let mut compression = IndexSet::new();
        compression.insert(9);
        let mut strategies = IndexSet::new();
        for i in 0..4 {
            strategies.insert(i);
        }
        // We always need NoOp to be present
        let mut alphas = IndexSet::new();
        alphas.insert(AlphaOptim::NoOp);

        Options {
            backup: false,
            check: false,
            pretend: false,
            fix_errors: false,
            force: false,
            preserve_attrs: false,
            filter,
            interlace: None,
            alphas,
            bit_depth_reduction: true,
            color_type_reduction: true,
            palette_reduction: true,
            grayscale_reduction: true,
            idat_recoding: true,
            strip: Headers::None,
            deflate: Deflaters::Zlib {
                compression,
                strategies,
                window: 15,
            },
            use_heuristics: false,
            timeout: None,
        }
    }
}

/// Perform optimization on the input file using the options provided
pub fn optimize(input: &InFile, output: &OutFile, opts: &Options) -> PngResult<()> {
    // Read in the file and try to decode as PNG.
    info!("Processing: {}", input);

    let deadline = Arc::new(Deadline::new(opts.timeout));

    // grab metadata before even opening input file to preserve atime
    let opt_metadata_preserved;
    let in_data = match *input {
        InFile::Path(ref input_path) => {
            if opts.preserve_attrs {
                opt_metadata_preserved = input_path
                    .metadata()
                    .map_err(|err| {
                        // Fail if metadata cannot be preserved
                        PngError::new(&format!(
                            "Unable to read metadata from input file {:?}: {}",
                            input_path, err
                        ))
                    })
                    .map(Some)?;
                debug!("preserving metadata: {:?}", opt_metadata_preserved);
            } else {
                opt_metadata_preserved = None;
            }
            PngData::read_file(input_path)?
        }
        InFile::StdIn => {
            opt_metadata_preserved = None;
            let mut data = Vec::new();
            stdin()
                .read_to_end(&mut data)
                .map_err(|e| PngError::new(&format!("Error reading stdin: {}", e)))?;
            data
        }
    };

    let mut png = PngData::from_slice(&in_data, opts.fix_errors)?;

    if opts.check {
        info!("Running in check mode, not optimizing");
        return Ok(());
    }

    // Run the optimizer on the decoded PNG.
    let mut optimized_output = optimize_png(&mut png, &in_data, opts, deadline)?;

    if is_fully_optimized(in_data.len(), optimized_output.len(), opts) {
        info!("File already optimized");
        match (output, input) {
            // if p is None, it also means same as the input path
            (&OutFile::Path(ref p), &InFile::Path(ref input_path))
                if p.as_ref().map_or(true, |p| p == input_path) =>
            {
                return Ok(());
            }
            _ => {
                optimized_output = in_data;
            }
        }
    }

    if opts.pretend {
        info!("Running in pretend mode, no output");
        return Ok(());
    }

    match (output, input) {
        (&OutFile::StdOut, _) | (&OutFile::Path(None), &InFile::StdIn) => {
            let mut buffer = BufWriter::new(stdout());
            buffer
                .write_all(&optimized_output)
                .map_err(|e| PngError::new(&format!("Unable to write to stdout: {}", e)))?;
        }
        (&OutFile::Path(ref output_path), _) => {
            let output_path = output_path
                .as_ref()
                .map(|p| p.as_path())
                .unwrap_or_else(|| input.path().unwrap());
            if opts.backup {
                perform_backup(output_path)?;
            }
            let out_file = File::create(output_path).map_err(|err| {
                PngError::new(&format!(
                    "Unable to write to file {}: {}",
                    output_path.display(),
                    err
                ))
            })?;
            if let Some(metadata_input) = &opt_metadata_preserved {
                copy_permissions(metadata_input, &out_file)?;
            }

            let mut buffer = BufWriter::new(out_file);
            buffer
                .write_all(&optimized_output)
                // flush BufWriter so IO errors don't get swallowed silently on close() by drop!
                .and_then(|()| buffer.flush())
                .map_err(|e| {
                    PngError::new(&format!(
                        "Unable to write to {}: {}",
                        output_path.display(),
                        e
                    ))
                })?;
            // force drop and thereby closing of file handle before modifying any timestamp
            std::mem::drop(buffer);
            if let Some(metadata_input) = &opt_metadata_preserved {
                copy_times(metadata_input, output_path)?;
            }
            info!("Output: {}", output_path.display());
        }
    }
    Ok(())
}

/// Perform optimization on the input file using the options provided, where the file is already
/// loaded in-memory
pub fn optimize_from_memory(data: &[u8], opts: &Options) -> PngResult<Vec<u8>> {
    // Read in the file and try to decode as PNG.
    info!("Processing from memory");

    let deadline = Arc::new(Deadline::new(opts.timeout));

    let original_size = data.len();
    let mut png = PngData::from_slice(data, opts.fix_errors)?;

    // Run the optimizer on the decoded PNG.
    let optimized_output = optimize_png(&mut png, data, opts, deadline)?;

    if is_fully_optimized(original_size, optimized_output.len(), opts) {
        info!("Image already optimized");
        Ok(data.to_vec())
    } else {
        Ok(optimized_output)
    }
}

#[derive(Debug, PartialEq, PartialOrd, Clone, Copy)]
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
    deadline: Arc<Deadline>,
) -> PngResult<Vec<u8>> {
    type TrialWithData = (TrialOptions, Vec<u8>);

    let original_png = png.clone();

    // Print png info
    let file_original_size = original_data.len();
    let idat_original_size = png.idat_data.len();
    info!(
        "    {}x{} pixels, PNG format",
        png.raw.ihdr.width, png.raw.ihdr.height
    );
    if let Some(ref palette) = png.raw.palette {
        info!(
            "    {} bits/pixel, {} colors in palette",
            png.raw.ihdr.bit_depth,
            palette.len()
        );
    } else {
        info!(
            "    {}x{} bits/pixel, {:?}",
            png.raw.channels_per_pixel(),
            png.raw.ihdr.bit_depth,
            png.raw.ihdr.color_type
        );
    }
    info!("    IDAT size = {} bytes", idat_original_size);
    info!("    File size = {} bytes", file_original_size);

    let mut filter = opts.filter.clone();
    let mut strategies = match &opts.deflate {
        Deflaters::Zlib { strategies, .. } => Some(strategies.clone()),
        _ => None,
    };

    if opts.use_heuristics {
        // Heuristically determine which set of options to use
        let (use_filter, use_strategy) = if png.raw.ihdr.bit_depth.as_u8() >= 8
            && png.raw.ihdr.color_type != colors::ColorType::Indexed
        {
            (5, 1)
        } else {
            (0, 0)
        };
        if filter.is_empty() {
            filter.insert(use_filter);
        }
        if let Some(strategies) = &mut strategies {
            if strategies.is_empty() {
                strategies.insert(use_strategy);
            }
        }
    }

    // This will collect all versions of images and pick one that compresses best
    let eval = Evaluator::new(deadline.clone());
    // Usually we want transformations that are smaller than the unmodified original,
    // but if we're interlacing, we have to accept a possible file size increase.
    if opts.interlace.is_none() {
        eval.set_baseline(png.raw.clone());
    }
    perform_reductions(png.raw.clone(), opts, &deadline, &eval);
    let reduction_occurred = if let Some(result) = eval.get_result() {
        *png = result;
        true
    } else {
        false
    };

    if opts.idat_recoding || reduction_occurred {
        // Go through selected permutations and determine the best
        let combinations = if let Deflaters::Zlib { compression, .. } = &opts.deflate {
            filter.len() * compression.len() * strategies.as_ref().unwrap().len()
        } else {
            filter.len()
        };
        let mut results: Vec<TrialOptions> = Vec::with_capacity(combinations);

        for f in &filter {
            if let Deflaters::Zlib { compression, .. } = &opts.deflate {
                for zc in compression {
                    for zs in strategies.as_ref().unwrap() {
                        results.push(TrialOptions {
                            filter: *f,
                            compression: *zc,
                            strategy: *zs,
                        });
                    }
                    if deadline.passed() {
                        break;
                    }
                }
            } else {
                // Zopfli and Libdeflater compression have no additional options.
                results.push(TrialOptions {
                    filter: *f,
                    compression: 0,
                    strategy: 0,
                });
            }

            if deadline.passed() {
                break;
            }
        }

        info!("Trying: {} combinations", results.len());

        let filters: IndexMap<u8, Vec<u8>> = filter
            .par_iter()
            .with_max_len(1)
            .map(|f| {
                let png = png.clone();
                (*f, png.raw.filter_image(*f))
            })
            .collect();

        let original_len = original_png.idat_data.len();
        let added_interlacing = opts.interlace == Some(1) && original_png.raw.ihdr.interlaced == 0;

        let best_size = AtomicMin::new(if opts.force { None } else { Some(original_len) });
        let results_iter = results.into_par_iter().with_max_len(1);
        let best = results_iter.filter_map(|trial| {
            if deadline.passed() {
                return None;
            }
            let filtered = &filters[&trial.filter];
            let new_idat = match opts.deflate {
                Deflaters::Zlib { window, .. } => deflate::deflate(
                    filtered,
                    trial.compression,
                    trial.strategy,
                    window,
                    &best_size,
                    &deadline,
                ),
                #[cfg(feature = "zopfli")]
                Deflaters::Zopfli => deflate::zopfli_deflate(filtered),
                #[cfg(feature = "libdeflater")]
                Deflaters::Libdeflater => deflate::libdeflater_deflate(filtered, &best_size),
            };

            let new_idat = match new_idat {
                Ok(n) => n,
                Err(PngError::DeflatedDataTooLong(max)) => {
                    debug!(
                        "    zc = {}  zs = {}  f = {}       >{} bytes",
                        trial.compression, trial.strategy, trial.filter, max,
                    );
                    return None;
                }
                Err(_) => return None,
            };

            // update best size across all threads
            let new_size = new_idat.len();
            best_size.set_min(new_size);

            debug!(
                "    zc = {}  zs = {}  f = {}        {} bytes",
                trial.compression,
                trial.strategy,
                trial.filter,
                new_idat.len()
            );

            if new_size < original_len || added_interlacing || opts.force {
                Some((trial, new_idat))
            } else {
                None
            }
        });
        let best: Option<TrialWithData> = best.reduce_with(|i, j| {
            if i.1.len() < j.1.len() || (i.1.len() == j.1.len() && i.0 < j.0) {
                i
            } else {
                j
            }
        });

        if let Some((opts, idat_data)) = best {
            png.idat_data = idat_data;
            info!("Found better combination:");
            info!(
                "    zc = {}  zs = {}  f = {}        {} bytes",
                opts.compression,
                opts.strategy,
                opts.filter,
                png.idat_data.len()
            );
        } else if reduction_occurred {
            *png = original_png;
        }
    }

    perform_strip(png, opts);

    let output = png.output();

    if idat_original_size >= png.idat_data.len() {
        info!(
            "    IDAT size = {} bytes ({} bytes decrease)",
            png.idat_data.len(),
            idat_original_size - png.idat_data.len()
        );
    } else {
        info!(
            "    IDAT size = {} bytes ({} bytes increase)",
            png.idat_data.len(),
            png.idat_data.len() - idat_original_size
        );
    }
    if file_original_size >= output.len() {
        info!(
            "    file size = {} bytes ({} bytes = {:.2}% decrease)",
            output.len(),
            file_original_size - output.len(),
            (file_original_size - output.len()) as f64 / file_original_size as f64 * 100_f64
        );
    } else {
        info!(
            "    file size = {} bytes ({} bytes = {:.2}% increase)",
            output.len(),
            output.len() - file_original_size,
            (output.len() - file_original_size) as f64 / file_original_size as f64 * 100_f64
        );
    }

    let (old_png, new_png) = rayon::join(
        || load_png_image_from_memory(original_data),
        || load_png_image_from_memory(&output),
    );

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

    error!(
        "The resulting image is corrupted and will not be outputted.\nThis is a bug! Please report it at https://github.com/shssoichiro/oxipng/issues"
    );
    Err(PngError::new("The resulting image is corrupted"))
}

fn perform_reductions(
    mut png: Arc<PngImage>,
    opts: &Options,
    deadline: &Deadline,
    eval: &Evaluator,
) {
    // must be done first to evaluate rest with the correct interlacing
    if let Some(interlacing) = opts.interlace {
        if let Some(reduced) = png.change_interlacing(interlacing) {
            png = Arc::new(reduced);
            eval.try_image(png.clone());
        }
        if deadline.passed() {
            return;
        }
    }

    if opts.palette_reduction {
        if let Some(reduced) = reduced_palette(&png) {
            png = Arc::new(reduced);
            eval.try_image(png.clone());
            report_reduction(&png);
        }
        if deadline.passed() {
            return;
        }
    }

    if opts.bit_depth_reduction {
        if let Some(reduced) = reduce_bit_depth(&png, 1) {
            let previous = png.clone();
            let bits = reduced.ihdr.bit_depth;
            png = Arc::new(reduced);
            eval.try_image(png.clone());
            if (bits == BitDepth::One || bits == BitDepth::Two)
                && previous.ihdr.bit_depth != BitDepth::Four
            {
                // Also try 16-color mode for all lower bits images, since that may compress better
                if let Some(reduced) = reduce_bit_depth(&previous, 4) {
                    eval.try_image(Arc::new(reduced));
                }
            }
            report_reduction(&png);
        }
        if deadline.passed() {
            return;
        }
    }

    if opts.color_type_reduction {
        if let Some(reduced) = reduce_color_type(&png, opts.grayscale_reduction) {
            png = Arc::new(reduced);
            eval.try_image(png.clone());
            report_reduction(&png);
        }
        if deadline.passed() {
            return;
        }
    }

    try_alpha_reductions(png, &opts.alphas, eval);
}

#[derive(Debug)]
struct DeadlineImp {
    start: Instant,
    timeout: Duration,
    print_message: AtomicBool,
}

/// Keep track of processing timeout
#[doc(hidden)]
#[derive(Debug)]
pub struct Deadline {
    imp: Option<DeadlineImp>,
}

impl Deadline {
    pub fn new(timeout: Option<Duration>) -> Self {
        Self {
            imp: timeout.map(|timeout| DeadlineImp {
                start: Instant::now(),
                timeout,
                print_message: AtomicBool::new(true),
            }),
        }
    }

    /// True if the timeout has passed, and no new work should be done.
    ///
    /// If the verbose option is on, it also prints a timeout message once.
    pub fn passed(&self) -> bool {
        if let Some(imp) = &self.imp {
            let elapsed = imp.start.elapsed();
            if elapsed > imp.timeout {
                if match imp.print_message.compare_exchange(
                    true,
                    false,
                    Ordering::SeqCst,
                    Ordering::SeqCst,
                ) {
                    Ok(x) | Err(x) => x,
                } {
                    warn!("Timed out after {} second(s)", elapsed.as_secs());
                }
                return true;
            }
        }
        false
    }
}

/// Display the status of the image data after a reduction has taken place
fn report_reduction(png: &PngImage) {
    if let Some(ref palette) = png.palette {
        info!(
            "Reducing image to {} bits/pixel, {} colors in palette",
            png.ihdr.bit_depth,
            palette.len()
        );
    } else {
        info!(
            "Reducing image to {}x{} bits/pixel, {}",
            png.channels_per_pixel(),
            png.ihdr.bit_depth,
            png.ihdr.color_type
        );
    }
}

/// Strip headers from the `PngData` object, as requested by the passed `Options`
fn perform_strip(png: &mut PngData, opts: &Options) {
    let raw = Arc::make_mut(&mut png.raw);
    match opts.strip {
        // Strip headers
        Headers::None => (),
        Headers::Keep(ref hdrs) => raw
            .aux_headers
            .retain(|hdr, _| std::str::from_utf8(hdr).map_or(false, |name| hdrs.contains(name))),
        Headers::Strip(ref hdrs) => {
            for hdr in hdrs {
                raw.aux_headers.remove(hdr.as_bytes());
            }
        }
        Headers::Safe => {
            const PRESERVED_HEADERS: [[u8; 4]; 9] = [
                *b"cHRM", *b"gAMA", *b"iCCP", *b"sBIT", *b"sRGB", *b"bKGD", *b"hIST", *b"pHYs",
                *b"sPLT",
            ];
            let keys: Vec<[u8; 4]> = raw.aux_headers.keys().cloned().collect();
            for hdr in &keys {
                if !PRESERVED_HEADERS.contains(hdr) {
                    raw.aux_headers.remove(hdr);
                }
            }
        }
        Headers::All => {
            raw.aux_headers = IndexMap::new();
        }
    }

    let may_replace_iccp = match opts.strip {
        Headers::Keep(ref hdrs) => hdrs.contains("sRGB"),
        Headers::Strip(ref hdrs) => !hdrs.iter().any(|v| v == "sRGB"),
        Headers::Safe => true,
        Headers::None | Headers::All => false,
    };

    if may_replace_iccp {
        if raw.aux_headers.get(b"sRGB").is_some() {
            // Files aren't supposed to have both chunks, so we chose to honor sRGB
            raw.aux_headers.remove(b"iCCP");
        } else if let Some(intent) = raw
            .aux_headers
            .get(b"iCCP")
            .and_then(|iccp| srgb_rendering_intent(iccp))
        {
            // sRGB-like profile can be safely replaced with
            // an sRGB chunk with the same rendering intent
            raw.aux_headers.remove(b"iCCP");
            raw.aux_headers.insert(*b"sRGB", vec![intent]);
        }
    }
}

/// If the profile is sRGB, extracts the rendering intent value from it
fn srgb_rendering_intent(mut iccp: &[u8]) -> Option<u8> {
    // Skip (useless) profile name
    loop {
        let (&n, rest) = iccp.split_first()?;
        iccp = rest;
        if n == 0 {
            break;
        }
    }

    let (&compression_method, compressed_data) = iccp.split_first()?;
    if compression_method != 0 {
        return None; // The profile is supposed to be compressed (method 0)
    }
    let icc_data = inflate(compressed_data).ok()?;

    let rendering_intent = *icc_data.get(67)?;

    // The known profiles are the same as in libpng's `png_sRGB_checks`.
    // The Profile ID header of ICC has a fixed layout,
    // and is supposed to contain MD5 of profile data at this offset
    match icc_data.get(84..100)? {
        b"\x29\xf8\x3d\xde\xaf\xf2\x55\xae\x78\x42\xfa\xe4\xca\x83\x39\x0d"
        | b"\xc9\x5b\xd6\x37\xe9\x5d\x8a\x3b\x0d\xf3\x8f\x99\xc1\x32\x03\x89"
        | b"\xfc\x66\x33\x78\x37\xe2\x88\x6b\xfd\x72\xe9\x83\x82\x28\xf1\xb8"
        | b"\x34\x56\x2a\xbf\x99\x4c\xcd\x06\x6d\x2c\x57\x21\xd0\xd6\x8c\x5d" => {
            Some(rendering_intent)
        }
        b"\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00" => {
            // Known-bad profiles are identified by their CRC
            match (
                Crc::<u32>::new(&CRC_32_ISO_HDLC).checksum(&icc_data),
                icc_data.len(),
            ) {
                (0x5d51_29ce, 3024) | (0x182e_a552, 3144) | (0xf29e_526d, 3144) => {
                    Some(rendering_intent)
                }
                _ => None,
            }
        }
        _ => None,
    }
}

/// Check if an image was already optimized prior to oxipng's operations
fn is_fully_optimized(original_size: usize, optimized_size: usize, opts: &Options) -> bool {
    original_size <= optimized_size && !opts.force && opts.interlace.is_none()
}

fn perform_backup(input_path: &Path) -> PngResult<()> {
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
fn copy_permissions(metadata_input: &Metadata, out_file: &File) -> PngResult<()> {
    let readonly_input = metadata_input.permissions().readonly();

    out_file
        .metadata()
        .map_err(|err_io| {
            PngError::new(&format!(
                "unable to read filesystem metadata of output file: {}",
                err_io
            ))
        })
        .and_then(|out_meta| {
            out_meta.permissions().set_readonly(readonly_input);
            out_file
                .metadata()
                .map_err(|err_io| {
                    PngError::new(&format!(
                        "unable to re-read filesystem metadata of output file: {}",
                        err_io
                    ))
                })
                .and_then(|out_meta_reread| {
                    if out_meta_reread.permissions().readonly() != readonly_input {
                        Err(PngError::new(&format!(
                            "failed to set readonly, expected: {}, found: {}",
                            readonly_input,
                            out_meta_reread.permissions().readonly()
                        )))
                    } else {
                        Ok(())
                    }
                })
        })
}

#[cfg(unix)]
fn copy_permissions(metadata_input: &Metadata, out_file: &File) -> PngResult<()> {
    use std::os::unix::fs::PermissionsExt;

    let permissions = metadata_input.permissions().mode();

    out_file
        .metadata()
        .map_err(|err_io| {
            PngError::new(&format!(
                "unable to read filesystem metadata of output file: {}",
                err_io
            ))
        })
        .and_then(|out_meta| {
            out_meta.permissions().set_mode(permissions);
            out_file
                .metadata()
                .map_err(|err_io| {
                    PngError::new(&format!(
                        "unable to re-read filesystem metadata of output file: {}",
                        err_io
                    ))
                })
                .and_then(|out_meta_reread| {
                    if out_meta_reread.permissions().mode() != permissions {
                        Err(PngError::new(&format!(
                            "failed to set permissions, expected: {:04o}, found: {:04o}",
                            permissions,
                            out_meta_reread.permissions().mode()
                        )))
                    } else {
                        Ok(())
                    }
                })
        })
}

#[cfg(not(feature = "filetime"))]
fn copy_times(_: &Metadata, _: &Path) -> PngResult<()> {
    Ok(())
}

#[cfg(feature = "filetime")]
fn copy_times(input_path_meta: &Metadata, out_path: &Path) -> PngResult<()> {
    let atime = filetime::FileTime::from_last_access_time(input_path_meta);
    let mtime = filetime::FileTime::from_last_modification_time(input_path_meta);
    debug!(
        "attempting to set file times: atime: {:?}, mtime: {:?}",
        atime, mtime
    );
    filetime::set_file_times(out_path, atime, mtime).map_err(|err_io| {
        PngError::new(&format!(
            "unable to set file times on {:?}: {}",
            out_path, err_io
        ))
    })
}

/// Loads a PNG image from memory to a [DynamicImage]
fn load_png_image_from_memory(png_data: &[u8]) -> Result<DynamicImage, image::ImageError> {
    let mut reader = image::io::Reader::new(Cursor::new(png_data));
    reader.set_format(ImageFormat::Png);
    reader.no_limits();
    reader.decode()
}

/// Compares images pixel by pixel for equivalent content
fn images_equal(old_png: &DynamicImage, new_png: &DynamicImage) -> bool {
    let a = old_png.pixels().filter(|x| {
        let p = x.2.channels();
        !(p.len() == 4 && p[3] == 0)
    });
    let b = new_png.pixels().filter(|x| {
        let p = x.2.channels();
        !(p.len() == 4 && p[3] == 0)
    });
    a.eq(b)
}
