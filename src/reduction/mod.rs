use crate::evaluate::Evaluator;
use crate::png::PngImage;
use crate::Deadline;
use crate::Options;
use std::sync::Arc;

pub mod alpha;
use crate::alpha::*;
pub mod bit_depth;
use crate::bit_depth::*;
pub mod color;
use crate::color::*;
pub mod palette;
use crate::palette::*;

pub(crate) fn perform_reductions(
    mut png: Arc<PngImage>,
    opts: &Options,
    deadline: &Deadline,
    eval: &Evaluator,
) -> (Arc<PngImage>, bool) {
    let mut reduction_occurred = false;
    let mut evaluation_added = false;

    // Interlacing must be processed first in order to evaluate the rest correctly
    if let Some(interlacing) = opts.interlace {
        if let Some(reduced) = png.change_interlacing(interlacing) {
            png = Arc::new(reduced);
            reduction_occurred = true;
        }
    }

    // If alpha optimization is enabled, clean the alpha channel before continuing
    // This can allow some color type reductions which may not have been possible otherwise
    if opts.optimize_alpha && !deadline.passed() {
        if let Some(reduced) = cleaned_alpha_channel(&png) {
            png = Arc::new(reduced);
            // This does not count as a reduction
        }
    }

    // Attempt to reduce 16-bit to 8-bit
    // This is just removal of bytes and does not need to be evaluated
    if opts.bit_depth_reduction && !deadline.passed() {
        if let Some(reduced) = reduced_bit_depth_16_to_8(&png) {
            png = Arc::new(reduced);
            reduction_occurred = true;
        }
    }

    // Attempt to reduce RGB to grayscale
    // This is just removal of bytes and does not need to be evaluated
    if opts.color_type_reduction && !deadline.passed() {
        if let Some(reduced) = reduce_rgb_to_grayscale(&png) {
            png = Arc::new(reduced);
            reduction_occurred = true;
        }
    }

    // Attempt to reduce the palette
    // This may change bytes but should always be beneficial
    if opts.palette_reduction && !deadline.passed() {
        if let Some(reduced) = reduced_palette(&png, opts.optimize_alpha) {
            png = Arc::new(reduced);
            reduction_occurred = true;
        }
    }

    // Now retain the current png for the evaluator baseline
    // It will only be entered into the evaluator if there are also others to evaluate
    let mut baseline = png.clone();

    // Attempt alpha removal
    if opts.color_type_reduction && !deadline.passed() {
        if let Some(reduced) = reduced_alpha_channel(&png, opts.optimize_alpha) {
            png = Arc::new(reduced);
            // If the reduction requires a tRNS chunk, enter this into the evaluator
            // Otherwise it is just removal of bytes and should become the baseline
            if png.ihdr.color_type.has_trns() {
                eval.try_image(png.clone());
                evaluation_added = true;
            } else {
                baseline = png.clone();
                reduction_occurred = true;
            }
        }
    }

    // Attempt to sort the palette
    if opts.palette_reduction && !deadline.passed() {
        if let Some(reduced) = sorted_palette(&png) {
            png = Arc::new(reduced);
            eval.try_image(png.clone());
            evaluation_added = true;
        }
    }

    // Attempt to reduce to palette
    if opts.color_type_reduction && !deadline.passed() {
        if let Some(reduced) = reduce_to_palette(&png) {
            png = Arc::new(reduced);
            // Make sure the palette gets sorted (but don't bother evaluating both results)
            if let Some(reduced) = sorted_palette(&png) {
                png = Arc::new(reduced);
            }
            eval.try_image(png.clone());
            evaluation_added = true;
        }
    }

    // Attempt to reduce to a lower bit depth
    if opts.bit_depth_reduction && !deadline.passed() {
        if let Some(reduced) = reduced_bit_depth_8_or_less(&png, 1) {
            png = Arc::new(reduced);
            eval.try_image(png.clone());
            evaluation_added = true;
        }
    }

    if evaluation_added {
        eval.set_baseline(baseline.clone());
    }
    (baseline, reduction_occurred)
}
