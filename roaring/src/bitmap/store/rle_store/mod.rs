mod interval;

use interval::Interval;
use std::cmp::Ordering;

#[derive(Clone, Debug)]
pub struct RunStore {
    vec: Vec<Interval>,
}

impl RunStore {
    pub fn new() -> RunStore {
        RunStore { vec: vec![] }
    }

    /// Searches for a container that starts with the given `ikey`. If no such
    /// container exists, returns an Error with the index of the container that
    /// *would* contain `ikey`.
    fn interleaved_binary_search(&self, ikey: u16) -> Result<i32, i32> {
        match self.vec.binary_search_by_key(&ikey, |interval| interval.value) {
            Ok(x) => Ok(i32::try_from(x).unwrap()),
            Err(y) => Err(i32::try_from(y).unwrap() - 1),
        }
    }

    /// Returns the index of the run which contains `ikey`.
    /// If no such index is found, returns an Err() whose value is the index at
    /// which the key can be inserted.
    #[inline]
    fn find_run(&self, ikey: u16) -> Result<i32, i32> {
        match self.vec.binary_search_by(|interval| interval.compare_to_index(ikey)) {
            Ok(x) => Ok(i32::try_from(x).unwrap()),
            Err(y) => Err(i32::try_from(y).unwrap() - 1)
        }
    }

    pub fn insert(&mut self, pos: u16) -> bool {
        let Err(index) = self.interleaved_binary_search(pos) else {
            return false; // already exists
        };

        if index >= 0 { // possible match
            let index = index as usize;
            let interval = self.vec[index];

            let offset = pos.checked_sub(interval.value);
            let len = interval.length;
            if offset <= Some(len) {
                // already exists
                return false;
            } else if offset == Some(len + 1) {
                // may need to fuse two intervals
                let next_interval = self.vec.get(index + 1);
                let fused = interval.try_fuse(next_interval);
                if let Some(fused_interval) = fused {
                    self.vec.remove(index + 1);
                    self.vec[index] = fused_interval;
                    return true;
                } else {
                    self.vec[index].length += 1;
                    return true;
                }
            } else if let Some(next_interval) = self.vec.get_mut(index + 1) {
                // may need to fuse into next interval
                if next_interval.value == pos + 1 {
                    next_interval.value = pos;
                    next_interval.length += 1;
                    return true;
                }
            }
        }

        if index == -1 {
            // may need to extend the first run
            let fused = Interval::from(pos).try_fuse(self.vec.get(0));
            if let Some(fused_interval) = fused {
                self.vec[0] = fused_interval;
                return true;
            }
        }

        self.vec.insert(usize::try_from(index + 1).unwrap(), Interval::from(pos));
        return true;
    }
}

#[derive(Debug)]
pub struct Error {
    index: usize,
    kind: ErrorKind,
}

#[derive(Debug)]
pub enum ErrorKind {
    MonotonicityViolation,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> core::fmt::Result {
        match self.kind {
            ErrorKind::MonotonicityViolation => {
                write!(f, "RLE containers should be strictly positively monotone. Found violation at index: {}", self.index)
            }
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

impl TryFrom<Vec<Interval>> for RunStore {
    type Error = Error;

    fn try_from(value: Vec<Interval>) -> Result<Self, Self::Error> {
        let mut iter = value.iter().enumerate();
        if let Some((_, prev)) = iter.next() {
            for (i, curr) in iter {
                match curr.compare_to_index(prev.value) {
                    Ordering::Greater => (),
                    _ => return Err(Error { index: i, kind: ErrorKind::MonotonicityViolation }),
                }
            }
        }
        Ok(RunStore { vec: value })
    }
}

#[cfg(test)]
mod tests {
    use core::borrow::Borrow;

    use super::Interval;
    use super::RunStore;

    fn get_mock_run_store() -> RunStore {
        RunStore::try_from(vec![
            Interval::from((5, 10)),
            Interval::from((15, 20)),
            Interval::from((25, 35)),
            Interval::from((37, 50)),
        ])
        .unwrap()
        .clone()
    }

    #[test]
    fn test_interleaved_binary_search() {
        let store = get_mock_run_store();

        // found
        assert_eq!(store.interleaved_binary_search(5), Ok(0));
        assert_eq!(store.interleaved_binary_search(15), Ok(1));
        assert_eq!(store.interleaved_binary_search(25), Ok(2));
        assert_eq!(store.interleaved_binary_search(37), Ok(3));
    
        // not found
        assert_eq!(store.interleaved_binary_search(3), Err(-1));
        assert_eq!(store.interleaved_binary_search(8), Err(0));
        assert_eq!(store.interleaved_binary_search(10), Err(0));
        assert_eq!(store.interleaved_binary_search(11), Err(0));
        assert_eq!(store.interleaved_binary_search(23), Err(1));
        assert_eq!(store.interleaved_binary_search(51), Err(3));
    }

    #[test]
    fn find_run() {
        let store = get_mock_run_store();

        assert!(store.find_run(0).is_err());
        assert_eq!(store.find_run(3), Err(-1));
        assert_eq!(store.find_run(5), Ok(0));
        assert_eq!(store.find_run(7), Ok(0));
        assert_eq!(store.find_run(10), Ok(0));
        assert_eq!(store.find_run(13), Err(0));
        assert_eq!(store.find_run(15), Ok(1));
        assert_eq!(store.find_run(20), Ok(1));
        assert_eq!(store.find_run(21), Err(1));
        assert_eq!(store.find_run(35), Ok(2));
        assert_eq!(store.find_run(51), Err(3));
    }

    #[test]
    fn test_insert() {
        let mut store = get_mock_run_store();

        // already exists
        assert!(!store.insert(5));
        assert!(!store.insert(7));
        assert!(!store.insert(10));

        // does not exist, create and append to new run at beginning
        assert!(store.insert(0));
        assert!(store.insert(1));
        assert!(store.insert(2));
        assert_eq!(store.vec[0], Interval::from((0, 2)));
        assert_eq!(store.vec.len(), 5);

        // does not exist, create and appent to new run in middle
        assert!(store.insert(22));
        assert!(store.insert(23));
        assert_eq!(store.vec.len(), 6);

        // does not exist, fusing two existing intervals
        assert!(store.insert(36));
        assert_eq!(store.vec.last(), Some(Interval::from((25, 50)).borrow()));
        assert_eq!(store.vec.len(), 5);

        // does not exist, append to current interval (relative to index 1)
        assert!(store.insert(11));
        assert_eq!(store.vec[1], Interval::from((5, 11)));
        assert_eq!(store.vec.len(), 5);

        // does not exist, prepend to next interval (relative to index 0)
        assert!(store.insert(4));
        assert_eq!(store.vec[1], Interval::from((4, 11)));
        assert_eq!(store.vec.len(), 5);
    }
}
