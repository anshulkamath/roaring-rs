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
}
