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
    not(feature = "zopfli"),
    allow(irrefutable_let_patterns),
    allow(unreachable_patterns)
)]

#[cfg(feature = "parallel")]
extern crate rayon;
#[cfg(not(feature = "parallel"))]
mod rayon;

use crate::atomicmin::AtomicMin;
use crate::deflate::{crc32, inflate};
use crate::evaluate::Evaluator;
use crate::headers::IhdrData;
use crate::png::PngData;
use crate::png::PngImage;
use crate::reduction::*;
use log::{debug, info, trace, warn};
use rayon::prelude::*;
use std::fmt;
use std::fs::{copy, File, Metadata};
use std::io::{stdin, stdout, BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

pub use crate::colors::{BitDepth, ColorType};
pub use crate::deflate::Deflaters;
pub use crate::error::PngError;
pub use crate::filters::RowFilter;
pub use crate::headers::Headers;
pub use crate::interlace::Interlacing;
pub use indexmap::{indexset, IndexMap, IndexSet};
pub use rgb::{RGB16, RGBA8};

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
#[cfg(feature = "sanity-checks")]
mod sanity_checks;

/// Private to oxipng; don't use outside tests and benches
#[doc(hidden)]
pub mod internal_tests {
    pub use crate::atomicmin::*;
    pub use crate::colors::*;
    pub use crate::deflate::*;
    pub use crate::headers::*;
    pub use crate::png::*;
    pub use crate::reduction::*;
    #[cfg(feature = "sanity-checks")]
    pub use crate::sanity_checks::*;
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
    /// Which RowFilters to try on the file
    ///
    /// Default: `None,Sub,Entropy,Bigrams`
    pub filter: IndexSet<RowFilter>,
    /// Whether to change the interlacing type of the file.
    ///
    /// `None` will not change the current interlacing type.
    ///
    /// `Some(x)` will change the file to interlacing mode `x`.
    ///
    /// Default: `None`
    pub interlace: Option<Interlacing>,
    /// Whether to allow transparent pixels to be altered to improve compression.
    pub optimize_alpha: bool,
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
    /// Default: `Libdeflater`
    pub deflate: Deflaters,
    /// Whether to use fast evaluation to pick the best filter
    ///
    /// Default: `true`
    pub fast_evaluation: bool,

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
            4 => opts.apply_preset_4(),
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
        self.filter.clear();
        if let Deflaters::Libdeflater { compression } = &mut self.deflate {
            *compression = 5;
        }
        self
    }

    fn apply_preset_1(mut self) -> Self {
        self.filter.clear();
        if let Deflaters::Libdeflater { compression } = &mut self.deflate {
            *compression = 10;
        }
        self
    }

    fn apply_preset_2(self) -> Self {
        self
    }

    fn apply_preset_3(mut self) -> Self {
        self.fast_evaluation = false;
        self.filter = indexset! {
            RowFilter::None,
            RowFilter::Bigrams,
            RowFilter::BigEnt,
            RowFilter::Brute
        };
        self
    }

    fn apply_preset_4(mut self) -> Self {
        if let Deflaters::Libdeflater { compression } = &mut self.deflate {
            *compression = 12;
        }
        self.apply_preset_3()
    }

    fn apply_preset_5(mut self) -> Self {
        self.fast_evaluation = false;
        self.filter.insert(RowFilter::Up);
        self.filter.insert(RowFilter::MinSum);
        self.filter.insert(RowFilter::BigEnt);
        self.filter.insert(RowFilter::Brute);
        if let Deflaters::Libdeflater { compression } = &mut self.deflate {
            *compression = 12;
        }
        self
    }

    fn apply_preset_6(mut self) -> Self {
        self.filter.insert(RowFilter::Average);
        self.filter.insert(RowFilter::Paeth);
        self.apply_preset_5()
    }
}

impl Default for Options {
    fn default() -> Options {
        // Default settings based on -o 2 from the CLI interface
        Options {
            backup: false,
            check: false,
            pretend: false,
            fix_errors: false,
            force: false,
            preserve_attrs: false,
            filter: indexset! {RowFilter::None, RowFilter::Sub, RowFilter::Entropy, RowFilter::Bigrams},
            interlace: None,
            optimize_alpha: false,
            bit_depth_reduction: true,
            color_type_reduction: true,
            palette_reduction: true,
            grayscale_reduction: true,
            idat_recoding: true,
            strip: Headers::None,
            deflate: Deflaters::Libdeflater { compression: 11 },
            fast_evaluation: true,
            timeout: None,
        }
    }
}

#[derive(Debug)]
/// A raw image definition which can be used to create an optimized png
pub struct RawImage {
    png: Arc<PngImage>,
}

impl RawImage {
    /// Construct a new raw image definition
    ///
    /// * `width` - The width of the image in pixels
    /// * `height` - The height of the image in pixels
    /// * `color_type` - The color type of the image
    /// * `bit_depth` - The bit depth of the image
    /// * `data` - The raw pixel data of the image
    pub fn new(
        width: u32,
        height: u32,
        color_type: ColorType,
        bit_depth: BitDepth,
        data: Vec<u8>,
    ) -> Result<Self, PngError> {
        // Validate bit depth
        let valid_depth = match color_type {
            ColorType::Grayscale { .. } => true,
            ColorType::Indexed { .. } => (bit_depth as u8) <= 8,
            _ => (bit_depth as u8) >= 8,
        };
        if !valid_depth {
            return Err(PngError::InvalidDepthForType(bit_depth, color_type));
        }

        // Validate data length
        let bpp = bit_depth as usize * color_type.channels_per_pixel() as usize;
        let row_bytes = (bpp * width as usize + 7) / 8;
        let expected_len = row_bytes * height as usize;
        if data.len() != expected_len {
            return Err(PngError::IncorrectDataLength(data.len(), expected_len));
        }

        Ok(Self {
            png: Arc::new(PngImage {
                ihdr: IhdrData {
                    width,
                    height,
                    color_type,
                    bit_depth,
                    interlaced: Interlacing::None,
                },
                data,
                aux_headers: IndexMap::new(),
            }),
        })
    }

    /// Add a png chunk, such as "iTXt", to be included in the output
    pub fn add_png_chunk(&mut self, chunk_type: [u8; 4], data: Vec<u8>) {
        // We can guarantee this will succeed - failure indicates a bug
        let png = Arc::get_mut(&mut self.png).unwrap();
        png.aux_headers.insert(chunk_type, data);
    }

    /// Add an ICC profile for the image
    pub fn add_icc_profile(&mut self, data: Vec<u8>) {
        // Compress with default compression level
        if let Ok(mut compressed) = deflate::deflate(&data, 11, &AtomicMin::new(None)) {
            let mut iccp = Vec::with_capacity(compressed.len() + 13);
            iccp.extend(b"icc"); // Profile name - generally unused, can be anything
            iccp.extend([0, 0]); // Null separator, zlib compression method
            iccp.append(&mut compressed);
            self.add_png_chunk(*b"iCCP", iccp);
        }
    }

    /// Create an optimized png from the raw image data using the options provided
    pub fn create_optimized_png(&self, opts: &Options) -> PngResult<Vec<u8>> {
        let deadline = Arc::new(Deadline::new(opts.timeout));
        let png = optimize_raw(Arc::clone(&self.png), opts, deadline, None)
            .ok_or_else(|| PngError::new("Failed to optimize input data"))?;
        Ok(png.output())
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
                trace!("preserving metadata: {:?}", opt_metadata_preserved);
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
            (OutFile::Path(ref p), InFile::Path(ref input_path))
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
        (OutFile::Path(ref output_path), _) => {
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
    pub filter: RowFilter,
    pub compression: u8,
}
type TrialWithData = (TrialOptions, Vec<u8>);

/// Perform optimization on the input PNG object using the options provided
fn optimize_png(
    png: &mut PngData,
    original_data: &[u8],
    opts: &Options,
    deadline: Arc<Deadline>,
) -> PngResult<Vec<u8>> {
    // Print png info
    let file_original_size = original_data.len();
    let idat_original_size = png.idat_data.len();
    debug!(
        "    {}x{} pixels, PNG format",
        png.raw.ihdr.width, png.raw.ihdr.height
    );
    report_format("    ", &png.raw);
    debug!("    IDAT size = {} bytes", idat_original_size);
    debug!("    File size = {} bytes", file_original_size);

    // Do this first so that reductions can ignore certain chunks such as bKGD
    perform_strip(png, opts);

    if let Some(new_png) = optimize_raw(png.raw.clone(), opts, deadline, Some(idat_original_size)) {
        png.raw = new_png.raw;
        png.idat_data = new_png.idat_data;
    }

    let output = png.output();

    if idat_original_size >= png.idat_data.len() {
        debug!(
            "    IDAT size = {} bytes ({} bytes decrease)",
            png.idat_data.len(),
            idat_original_size - png.idat_data.len()
        );
    } else {
        debug!(
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

    #[cfg(feature = "sanity-checks")]
    assert!(sanity_checks::validate_output(&output, original_data));

    Ok(output)
}

/// Perform optimization on the input image data using the options provided
fn optimize_raw(
    mut png: Arc<PngImage>,
    opts: &Options,
    deadline: Arc<Deadline>,
    max_idat_size: Option<usize>,
) -> Option<PngData> {
    // Must use normal (lazy) compression, as faster ones (greedy) are not representative
    let eval_compression = 5;
    // None and Bigrams work well together, especially for alpha reductions
    let eval_filters = indexset! {RowFilter::None, RowFilter::Bigrams};
    // This will collect all versions of images and pick one that compresses best
    let eval = Evaluator::new(
        deadline.clone(),
        eval_filters.clone(),
        eval_compression,
        false,
    );
    let (baseline, mut reduction_occurred) =
        perform_reductions(png.clone(), opts, &deadline, &eval);
    png = baseline;
    let mut eval_result = eval.get_best_candidate();
    if let Some(ref result) = eval_result {
        if result.is_reduction {
            png = Arc::clone(&result.image.raw);
            reduction_occurred = true;
        }
    }

    if reduction_occurred {
        report_format("Reducing image to ", &png);
    }

    if opts.idat_recoding || reduction_occurred {
        let mut filters = opts.filter.clone();
        let fast_eval = opts.fast_evaluation && (filters.len() > 1 || eval_result.is_some());
        let best: Option<TrialWithData> = if fast_eval {
            // Perform a fast evaluation of selected filters followed by a single main compression trial

            if eval_result.is_some() {
                // Some filters have already been evaluated, we don't need to try them again
                filters = filters.difference(&eval_filters).cloned().collect();
            }

            if !filters.is_empty() {
                trace!("Evaluating: {} filters", filters.len());
                let eval = Evaluator::new(deadline, filters, eval_compression, opts.optimize_alpha);
                if let Some(ref result) = eval_result {
                    eval.set_best_size(result.image.idat_data.len());
                }
                eval.try_image(png.clone());
                if let Some(result) = eval.get_best_candidate() {
                    eval_result = Some(result);
                }
            }
            // We should have a result here - fail if not (e.g. deadline passed)
            let eval_result = eval_result?;

            let trial = TrialOptions {
                filter: eval_result.filter,
                compression: match opts.deflate {
                    Deflaters::Libdeflater { compression } => compression,
                    _ => 0,
                },
            };
            if trial.compression > 0 && trial.compression <= eval_compression {
                // No further compression required
                let idat_data = eval_result.image.idat_data;
                if opts.force || idat_data.len() < max_idat_size.unwrap_or(usize::MAX) {
                    Some((trial, idat_data))
                } else {
                    None
                }
            } else {
                debug!("Trying: {}", trial.filter);
                let best_size = AtomicMin::new(if opts.force { None } else { max_idat_size });
                perform_trial(&eval_result.image.filtered, opts, trial, &best_size)
            }
        } else {
            // Perform full compression trials of selected filters and determine the best

            if filters.is_empty() {
                // Pick a filter automatically
                if png.ihdr.bit_depth as u8 >= 8 {
                    // Bigrams is the best all-rounder when there's at least one byte per pixel
                    filters.insert(RowFilter::Bigrams);
                } else {
                    // Otherwise delta filters generally don't work well, so just stick with None
                    filters.insert(RowFilter::None);
                }
            }

            let mut results: Vec<TrialOptions> = Vec::with_capacity(filters.len());

            for f in &filters {
                results.push(TrialOptions {
                    filter: *f,
                    compression: match opts.deflate {
                        Deflaters::Libdeflater { compression } => compression,
                        _ => 0,
                    },
                });
            }

            debug!("Trying: {} filters", results.len());

            let best_size = AtomicMin::new(if opts.force { None } else { max_idat_size });
            let results_iter = results.into_par_iter().with_max_len(1);
            let best = results_iter.filter_map(|trial| {
                if deadline.passed() {
                    return None;
                }
                let filtered = &png.filter_image(trial.filter, opts.optimize_alpha);
                perform_trial(filtered, opts, trial, &best_size)
            });
            best.reduce_with(|i, j| {
                if i.1.len() < j.1.len() || (i.1.len() == j.1.len() && i.0 < j.0) {
                    i
                } else {
                    j
                }
            })
        };

        if let Some((opts, idat_data)) = best {
            debug!("Found better combination:");
            debug!(
                "    zc = {}  f = {:8}  {} bytes",
                opts.compression,
                opts.filter,
                idat_data.len()
            );
            return Some(PngData {
                raw: png,
                // The filtered data has not been retained here, but we don't need to return it
                filtered: vec![],
                idat_data,
            });
        }
    } else if let Some(result) = eval_result {
        // If idat_recoding is off and reductions were attempted but ended up choosing the baseline,
        // we should still check if the evaluator compressed the baseline smaller than the original.
        let idat_data = &result.image.idat_data;
        if idat_data.len() < max_idat_size.unwrap_or(usize::MAX) {
            debug!("Found better combination:");
            debug!(
                "    zc = {}  f = {:8}  {} bytes",
                eval_compression,
                result.filter,
                idat_data.len()
            );
            return Some(result.image);
        }
    }

    None
}

/// Execute a compression trial
fn perform_trial(
    filtered: &[u8],
    opts: &Options,
    trial: TrialOptions,
    best_size: &AtomicMin,
) -> Option<TrialWithData> {
    let new_idat = match opts.deflate {
        Deflaters::Libdeflater { .. } => deflate::deflate(filtered, trial.compression, best_size),
        #[cfg(feature = "zopfli")]
        Deflaters::Zopfli { iterations } => deflate::zopfli_deflate(filtered, iterations),
    };

    // update best size or convert to error if not smaller
    let new_idat = match new_idat {
        Ok(n) if !best_size.set_min(n.len()) => Err(PngError::DeflatedDataTooLong(n.len())),
        _ => new_idat,
    };

    match new_idat {
        Ok(n) => {
            let bytes = n.len();
            trace!(
                "    zc = {}  f = {:8}  {} bytes",
                trial.compression,
                trial.filter,
                bytes
            );
            Some((trial, n))
        }
        Err(PngError::DeflatedDataTooLong(bytes)) => {
            trace!(
                "    zc = {}  f = {:8} >{} bytes",
                trial.compression,
                trial.filter,
                bytes,
            );
            None
        }
        Err(_) => None,
    }
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

/// Display the format of the image data
fn report_format(prefix: &str, png: &PngImage) {
    debug!(
        "{}{}-bit {}, {}",
        prefix, png.ihdr.bit_depth, png.ihdr.color_type, png.ihdr.interlaced
    );
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
            const PRESERVED_HEADERS: [[u8; 4]; 5] =
                [*b"cICP", *b"iCCP", *b"sBIT", *b"sRGB", *b"pHYs"];
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
    // The decompressed size is unknown so we have to guess the required buffer size
    let max_size = (compressed_data.len() * 2).max(1000);
    let icc_data = inflate(compressed_data, max_size).ok()?;

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
            match (crc32(&icc_data), icc_data.len()) {
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
    original_size <= optimized_size && !opts.force
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
    trace!(
        "attempting to set file times: atime: {:?}, mtime: {:?}",
        atime,
        mtime
    );
    filetime::set_file_times(out_path, atime, mtime).map_err(|err_io| {
        PngError::new(&format!(
            "unable to set file times on {:?}: {}",
            out_path, err_io
        ))
    })
}
