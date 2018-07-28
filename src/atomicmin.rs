use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::{Relaxed, SeqCst};

pub struct AtomicMin {
    val: AtomicUsize,
}

impl AtomicMin {
    pub fn new(init: Option<usize>) -> Self {
        Self {
            val: AtomicUsize::new(init.unwrap_or(usize::max_value())),
        }
    }

    pub fn get(&self) -> Option<usize> {
        let val = self.val.load(SeqCst);
        if val == usize::max_value() {
            None
        } else {
            Some(val)
        }
    }

    pub fn set_min(&self, new_val: usize) {
        let mut current_val = self.val.load(Relaxed);
        loop {
            if new_val < current_val {
                if let Err(v) = self
                    .val
                    .compare_exchange(current_val, new_val, SeqCst, Relaxed)
                {
                    current_val = v;
                    continue;
                }
            }
            break;
        }
    }
}
