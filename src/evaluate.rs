//! Check if a reduction makes file smaller, and keep best reductions.
//! Works asynchronously when possible

use atomicmin::AtomicMin;
use deflate;
use png::PngData;
use png::PngImage;
use png::STD_COMPRESSION;
use png::STD_FILTERS;
use png::STD_STRATEGY;
use png::STD_WINDOW;
#[cfg(feature = "parallel")]
use rayon::prelude::*;
use std::sync::mpsc::*;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;

/// Collect image versions and pick one that compresses best
pub struct Evaluator {
    /// images are sent to the thread for evaluation
    eval_send: Option<SyncSender<(Arc<PngImage>, bool)>>,
    // the thread helps evaluate images asynchronously
    eval_thread: thread::JoinHandle<Option<PngData>>,
}

impl Evaluator {
    pub fn new() -> Self {
        // queue size ensures we're not using too much memory for pending reductions
        let (tx, rx) = sync_channel(4);
        Self {
            eval_send: Some(tx),
            eval_thread: thread::spawn(move || Self::evaluate_images(rx)),
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
        self.try_image_inner(image, false)
    }

    /// Check if the image is smaller than others
    pub fn try_image(&self, image: Arc<PngImage>) {
        self.try_image_inner(image, true)
    }

    fn try_image_inner(&self, image: Arc<PngImage>, is_reduction: bool) {
        self.eval_send.as_ref().expect("not finished yet").send((image, is_reduction)).expect("send")
    }

    /// Main loop of evaluation thread
    fn evaluate_images(from_channel: Receiver<(Arc<PngImage>, bool)>) -> Option<PngData> {
        let best_candidate_size = AtomicMin::new(None);
        let best_result: Mutex<Option<(PngData, _, _)>> = Mutex::new(None);
        // ends when sender is dropped
        for (nth, (image, is_reduction)) in from_channel.iter().enumerate() {
            #[cfg(feature = "parallel")]
            let filters_iter = STD_FILTERS.par_iter().with_max_len(1);
            #[cfg(not(feature = "parallel"))]
            let filters_iter = STD_FILTERS.iter();

            filters_iter.for_each(|&f| {
                if let Ok(idat_data) = deflate::deflate(
                    &image.filter_image(f),
                    STD_COMPRESSION,
                    STD_STRATEGY,
                    STD_WINDOW,
                    &best_candidate_size,
                ) {
                    let mut res = best_result.lock().unwrap();
                    if best_candidate_size.get().map_or(true, |best_len| {
                        // a tie-breaker is required to make evaluation deterministic
                        if let Some(res) = res.as_ref() {
                            // choose smallest compressed, or if compresses the same, smallest uncompressed, or cheaper filter
                            let old_img = &res.0.raw;
                            let new = (idat_data.len(), image.data.len(), image.ihdr.bit_depth, f, nth);
                            let old = (best_len, old_img.data.len(), old_img.ihdr.bit_depth, res.1, res.2);
                            new < old
                        } else if best_len > idat_data.len() {
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

