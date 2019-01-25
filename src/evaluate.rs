//! Check if a reduction makes file smaller, and keep best reductions.
//! Works asynchronously when possible

use atomicmin::AtomicMin;
use deflate;
use Deadline;
use png::PngData;
use png::PngImage;
use png::STD_COMPRESSION;
use png::STD_FILTERS;
use png::STD_STRATEGY;
use png::STD_WINDOW;
use rayon::prelude::*;
use std::sync::mpsc::*;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;

/// Collect image versions and pick one that compresses best
pub(crate) struct Evaluator {
    /// images are sent to the thread for evaluation
    eval_send: Option<SyncSender<(Arc<PngImage>, f32, bool)>>,
    // the thread helps evaluate images asynchronously
    eval_thread: thread::JoinHandle<Option<PngData>>,
}

impl Evaluator {
    pub fn new(deadline: Arc<Deadline>) -> Self {
        // queue size ensures we're not using too much memory for pending reductions
        let (tx, rx) = sync_channel(4);
        Self {
            eval_send: Some(tx),
            eval_thread: thread::spawn(move || Self::evaluate_images(rx, deadline)),
        }
    }

    /// Wait for all evaluations to finish and return smallest reduction
    /// Or `None` if all reductions were worse than baseline.
    pub fn get_result(mut self) -> Option<PngData> {
        let _ = self.eval_send.take(); // disconnect the sender, breaking the loop in the thread
        self.eval_thread.join().expect("eval thread")
    }

    /// Set baseline image. It will be used only to measure minimum compression level required
    pub fn set_baseline(&self, image: Arc<PngImage>) {
        self.try_image_inner(image, 1.0, false)
    }

    /// Check if the image is smaller than others
    /// Bias is a value in 0..=1 range. Compressed size is multiplied by
    /// this fraction when comparing to the best, so 0.95 allows 5% larger size.
    pub fn try_image(&self, image: Arc<PngImage>, bias: f32) {
        self.try_image_inner(image, bias, true)
    }

    fn try_image_inner(&self, image: Arc<PngImage>, bias: f32, is_reduction: bool) {
        self.eval_send.as_ref().expect("not finished yet").send((image, bias, is_reduction)).expect("send")
    }

    /// Main loop of evaluation thread
    fn evaluate_images(from_channel: Receiver<(Arc<PngImage>, f32, bool)>, deadline: Arc<Deadline>) -> Option<PngData> {
        let best_candidate_size = AtomicMin::new(None);
        let best_result: Mutex<Option<(PngData, _, _)>> = Mutex::new(None);
        // ends when sender is dropped
        for (nth, (image, bias, is_reduction)) in from_channel.iter().enumerate() {
            let filters_iter = STD_FILTERS.par_iter().with_max_len(1);

            filters_iter.for_each(|&f| {
                if deadline.passed() {
                    return;
                }
                if let Ok(idat_data) = deflate::deflate(
                    &image.filter_image(f),
                    STD_COMPRESSION,
                    STD_STRATEGY,
                    STD_WINDOW,
                    &best_candidate_size,
                    &deadline,
                ) {
                    let mut res = best_result.lock().unwrap();
                    if best_candidate_size.get().map_or(true, |old_best_len| {
                        let new_len = (idat_data.len() as f64 * bias as f64) as usize;
                        // a tie-breaker is required to make evaluation deterministic
                        if let Some(res) = res.as_ref() {
                            // choose smallest compressed, or if compresses the same, smallest uncompressed, or cheaper filter
                            let old_img = &res.0.raw;
                            let new = (new_len, image.data.len(), image.ihdr.bit_depth, f, nth);
                            let old = (old_best_len, old_img.data.len(), old_img.ihdr.bit_depth, res.1, res.2);
                            new < old
                        } else if new_len < old_best_len {
                            true
                        } else {
                            false
                        }
                    }) {
                        best_candidate_size.set_min(idat_data.len());
                        *res = if is_reduction {
                            Some((PngData {
                                idat_data,
                                raw: Arc::clone(&image),
                            }, f, nth))
                        } else {
                            None
                        };
                    }
                }
            });
        }
        best_result.into_inner().expect("filters should be done")
            .map(|(img, _, _)| img)
    }
}

