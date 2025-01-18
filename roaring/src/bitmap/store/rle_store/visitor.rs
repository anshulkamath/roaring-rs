use super::RunStore;
use super::Interval;

/// Matches the function of `bitmap::store::array_store::visitor`
/// 
/// This visitor pattern allows multiple different algorithms to be written over the same data
/// For example: vectorized algorithms can pass a visitor off to a scalar algorithm to finish off
/// a tail that is not a multiple of the vector width.
///
/// Perhaps more importantly: it separates the set algorithms from the operations performed on
/// their results. Future work can utilize the exiting algorithms to trivially implement
/// computing the cardinality of an operation without materializng a new bitmap.
pub trait BinaryOperationVisitor {
    fn visit_interval(&mut self, ival: &Interval);
    fn visit_run_store(&mut self, store: &RunStore);
}

pub struct RunWriter {
    store: RunStore
}

impl RunWriter {
    pub fn new() -> Self {
        RunWriter {
            store: RunStore::new(),
        }
    }

    pub fn into_inner(self) -> RunStore {
        self.store
    }
}

impl BinaryOperationVisitor for RunWriter {
    /// Appends an interval to the internal RunStore, handling merging, if necessary.
    /// Returns the resulting tail interval.
    #[inline]
    fn visit_interval(&mut self, ival: &Interval) {
        match self.store.vec.last_mut() {
            Some(last_ival) => {
                assert!(ival.value >= last_ival.value);

                let max_end = std::cmp::max(last_ival.get_end(), ival.get_end());
                if ival.value <= last_ival.get_end() + 1 {
                    last_ival.length = max_end - last_ival.value;
                } else {
                    self.store.vec.push(ival.clone());
                }
            }
            None => self.store.vec.push(ival.clone()),
        }
    }

    fn visit_run_store(&mut self, store: &RunStore) {
        self.store = store.clone()
    }
}
