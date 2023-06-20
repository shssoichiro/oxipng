//! Check if a reduction makes file smaller, and keep best reductions.
//! Works asynchronously when possible

use crate::atomicmin::AtomicMin;
use crate::deflate;
use crate::filters::RowFilter;
use crate::png::PngImage;
#[cfg(not(feature = "parallel"))]
use crate::rayon;
use crate::Deadline;
use crate::PngError;
#[cfg(feature = "parallel")]
use crossbeam_channel::{unbounded, Receiver, Sender};
use indexmap::IndexSet;
use log::trace;
use rayon::prelude::*;
#[cfg(not(feature = "parallel"))]
use std::cell::RefCell;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::SeqCst;
use std::sync::Arc;

pub struct Candidate {
    pub image: Arc<PngImage>,
    pub idat_data: Vec<u8>,
    pub filtered: Vec<u8>,
    pub filter: RowFilter,
    // first wins tie-breaker
    nth: usize,
}

impl Candidate {
    fn cmp_key(&self) -> impl Ord {
        (
            self.idat_data.len() + self.image.key_chunks_size(),
            self.image.data.len(),
            self.image.ihdr.bit_depth,
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
    optimize_alpha: bool,
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
    pub fn new(
        deadline: Arc<Deadline>,
        filters: IndexSet<RowFilter>,
        compression: u8,
        optimize_alpha: bool,
    ) -> Self {
        #[cfg(feature = "parallel")]
        let eval_channel = unbounded();
        Self {
            deadline,
            filters,
            compression,
            optimize_alpha,
            best_candidate_size: Arc::new(AtomicMin::new(None)),
            nth: AtomicUsize::new(0),
            #[cfg(feature = "parallel")]
            eval_channel,
            #[cfg(not(feature = "parallel"))]
            eval_best_candidate: RefCell::new(None),
        }
    }

    /// Wait for all evaluations to finish and return smallest reduction
    /// Or `None` if the queue is empty.
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

    /// Set best size, if known in advance
    pub fn set_best_size(&self, size: usize) {
        self.best_candidate_size.set_min(size);
    }

    /// Check if the image is smaller than others
    pub fn try_image(&self, image: Arc<PngImage>) {
        let nth = self.nth.fetch_add(1, SeqCst);
        // These clones are only cheap refcounts
        let deadline = self.deadline.clone();
        let filters = self.filters.clone();
        let compression = self.compression;
        let optimize_alpha = self.optimize_alpha;
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
                let filtered = image.filter_image(filter, optimize_alpha);
                let idat_data = deflate::deflate(&filtered, compression, &best_candidate_size);
                if let Ok(idat_data) = idat_data {
                    let size = idat_data.len() + image.key_chunks_size();
                    best_candidate_size.set_min(size);
                    trace!(
                        "Eval: {}-bit {:20}  {:8}   {} bytes",
                        image.ihdr.bit_depth,
                        image.ihdr.color_type,
                        filter,
                        size
                    );
                    let new = Candidate {
                        image: image.clone(),
                        idat_data,
                        filtered,
                        filter,
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
                } else if let Err(PngError::DeflatedDataTooLong(size)) = idat_data {
                    trace!(
                        "Eval: {}-bit {:20}  {:8}  >{} bytes",
                        image.ihdr.bit_depth,
                        image.ihdr.color_type,
                        filter,
                        size
                    );
                }
            });
        });
    }
}
