use std::cmp::Ordering;

/// A run-length pair that represents the intervals in the range [v, v+l].
/// For example, the interval given by {3, 2} represents the set [3, 4, 5].
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Interval {
    pub value: u16,
    pub length: u16,
}

impl Interval {
    pub fn new(value: u16, length: u16) -> Interval {
        Interval { value, length }
    }

    pub fn get_pair(&self) -> (u16, u16) {
        return (self.value, self.value + self.length)
    }
}

impl From<u16> for Interval {
    fn from(value: u16) -> Self {
        Interval::new(value, 0)
    }
}

impl From<(u16, u16)> for Interval {
    fn from(value: (u16, u16)) -> Self {
        Interval::new(value.0, value.1 - value.0)
    }
}
