use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::SeqCst;

#[derive(Debug)]
pub struct AtomicMin {
    val: AtomicUsize,
}

impl AtomicMin {
    pub fn new(init: Option<usize>) -> Self {
        Self {
            val: AtomicUsize::new(init.unwrap_or(usize::MAX)),
        }
    }

    pub fn get(&self) -> Option<usize> {
        let val = self.val.load(SeqCst);
        if val == usize::MAX {
            None
        } else {
            Some(val)
        }
    }

    /// Unset value is usize_max
    pub fn as_atomic_usize(&self) -> &AtomicUsize {
        &self.val
    }

    pub fn set_min(&self, new_val: usize) {
        self.val.fetch_min(new_val, SeqCst);
    }
}
