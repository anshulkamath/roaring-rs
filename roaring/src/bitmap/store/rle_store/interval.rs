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

    /// Compares the given `key` to the interval. Note that the comparison is
    /// relative to the interval, so `Less` implies that our interval is below
    /// the key (i.e., the key is higher than the bounds of the interval).
    /// 
    /// # Example
    /// ```ignore
    /// let interval = Interval::from((3, 5)); // [3, 4, 5]
    /// assert_eq!(interval.compare_to_index(2), std::cmp::Ordering::Greater);
    /// assert_eq!(interval.compare_to_index(3), std::cmp::Ordering::Equal);
    /// assert_eq!(interval.compare_to_index(4), std::cmp::Ordering::Equal);
    /// assert_eq!(interval.compare_to_index(5), std::cmp::Ordering::Equal);
    /// assert_eq!(interval.compare_to_index(6), std::cmp::Ordering::Less);
    /// ```
    pub fn compare_to_index(&self, key: u16) -> Ordering {
        let istart = self.value;
        let iend = istart + self.length;

        if key < istart {
            Ordering::Greater
        } else if key <= iend {
            Ordering::Equal
        } else {
            Ordering::Less
        }
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

#[cfg(test)]
mod tests {
    use super::Interval;
    
    #[test]
    fn test_compare_with_index() {
        let interval = Interval::from((10, 20));
        assert_eq!(interval.compare_to_index(9), std::cmp::Ordering::Greater);
        assert_eq!(interval.compare_to_index(10), std::cmp::Ordering::Equal);
        assert_eq!(interval.compare_to_index(15), std::cmp::Ordering::Equal);
        assert_eq!(interval.compare_to_index(20), std::cmp::Ordering::Equal);
        assert_eq!(interval.compare_to_index(21), std::cmp::Ordering::Less);
    }
}
