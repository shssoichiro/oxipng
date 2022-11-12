//! Check if a reduction makes file smaller, and keep best reductions.
//! Works asynchronously when possible

use crate::atomicmin::AtomicMin;
use crate::deflate;
use crate::filters::RowFilter;
use crate::png::PngData;
use crate::png::PngImage;
#[cfg(not(feature = "parallel"))]
use crate::rayon;
use crate::Deadline;
#[cfg(feature = "parallel")]
use crossbeam_channel::{unbounded, Receiver, Sender};
use indexmap::IndexSet;
use rayon::prelude::*;
#[cfg(not(feature = "parallel"))]
use std::cell::RefCell;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::SeqCst;
use std::sync::Arc;

pub struct Candidate {
    pub image: PngData,
    pub filter: RowFilter,
    pub is_reduction: bool,
    // first wins tie-breaker
    nth: usize,
}

impl Candidate {
    fn cmp_key(&self) -> impl Ord {
        (
            self.image.idat_data.len(),
            self.image.raw.data.len(),
            self.image.raw.ihdr.bit_depth,
            self.filter,
            self.nth,
        )
    }
}

/// Collect image versions and pick one that compresses best
pub(crate) struct Evaluator {
    deadline: Arc<Deadline>,
    filters: IndexSet<RowFilter>,
    compression: u8,
    nth: AtomicUsize,
    best_candidate_size: Arc<AtomicMin>,
    /// images are sent to the caller thread for evaluation
    #[cfg(feature = "parallel")]
    eval_channel: (Sender<Candidate>, Receiver<Candidate>),
    // in non-parallel mode, images are evaluated synchronously
    #[cfg(not(feature = "parallel"))]
    eval_best_candidate: RefCell<Option<Candidate>>,
}

impl Evaluator {
    pub fn new(deadline: Arc<Deadline>, filters: IndexSet<RowFilter>, compression: u8) -> Self {
        #[cfg(feature = "parallel")]
        let eval_channel = unbounded();
        Self {
            deadline,
            filters,
            compression,
            best_candidate_size: Arc::new(AtomicMin::new(None)),
            nth: AtomicUsize::new(0),
            #[cfg(feature = "parallel")]
            eval_channel,
            #[cfg(not(feature = "parallel"))]
            eval_best_candidate: RefCell::new(None),
        }
    }

    /// Wait for all evaluations to finish and return smallest reduction
    /// Or `None` if all reductions were worse than baseline.
    #[cfg(feature = "parallel")]
    pub fn get_best_candidate(self) -> Option<Candidate> {
        let (eval_send, eval_recv) = self.eval_channel;
        drop(eval_send); // disconnect the sender, breaking the loop in the thread
        eval_recv.into_iter().min_by_key(Candidate::cmp_key)
    }

    #[cfg(not(feature = "parallel"))]
    pub fn get_best_candidate(self) -> Option<Candidate> {
        self.eval_best_candidate.into_inner()
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
        let nth = self.nth.fetch_add(1, SeqCst);
        // These clones are only cheap refcounts
        let deadline = self.deadline.clone();
        let filters = self.filters.clone();
        let compression = self.compression;
        let best_candidate_size = self.best_candidate_size.clone();
        // sends it off asynchronously for compression,
        // but results will be collected via the message queue
        #[cfg(feature = "parallel")]
        let eval_send = self.eval_channel.0.clone();
        rayon::spawn(move || {
            let filters_iter = filters.par_iter().with_max_len(1);

            // Updating of best result inside the parallel loop would require locks,
            // which are dangerous to do in side Rayon's loop.
            // Instead, only update (atomic) best size in real time,
            // and the best result later without need for locks.
            filters_iter.for_each(|&filter| {
                if deadline.passed() {
                    return;
                }
                if let Ok(idat_data) = deflate::deflate(
                    &image.filter_image(filter),
                    compression,
                    &best_candidate_size,
                ) {
                    best_candidate_size.set_min(idat_data.len());
                    let new = Candidate {
                        image: PngData {
                            idat_data,
                            raw: Arc::clone(&image),
                        },
                        filter,
                        is_reduction,
                        nth,
                    };

                    #[cfg(feature = "parallel")]
                    {
                        eval_send.send(new).expect("send");
                    }

                    #[cfg(not(feature = "parallel"))]
                    {
                        match &mut *self.eval_best_candidate.borrow_mut() {
                            Some(prev) if prev.cmp_key() < new.cmp_key() => {}
                            best => *best = Some(new),
                        }
                    }
                }
            });
        });
    }
}
