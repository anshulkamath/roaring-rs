use super::interval::Interval;
use super::visitor::BinaryOperationVisitor;
use super::RunStore;
use crate::bitmap::iter::BivariateOrderedIterator;
use crate::bitmap::util;

pub fn or(lhs: &RunStore, rhs: &RunStore, visitor: &mut impl BinaryOperationVisitor) {
    if lhs.is_full() {
        visitor.visit_run_store(lhs);
        return;
    }

    if rhs.is_full() {
        visitor.visit_run_store(rhs);
        return;
    }

    BivariateOrderedIterator::new(lhs.vec.iter(), rhs.vec.iter())
        .for_each(|i| visitor.visit_interval(i));
}

pub fn xor(lhs: &RunStore, rhs: &RunStore, visitor: &mut impl BinaryOperationVisitor) {
    let mut prev_interval: Option<Interval> = None;
    let mut xor_append = |curr_interval: &Interval| {
        let curr_interval = curr_interval.clone();

        let Some(unwrapped_prev) = prev_interval else {
            prev_interval = Some(curr_interval);
            return;
        };

        // the same interval cancels each other out
        if unwrapped_prev == curr_interval {
            prev_interval = None;
            return;
        }

        // no overlap, append previous interval
        if unwrapped_prev.get_end() + 1 < curr_interval.value {
            visitor.visit_interval(unwrapped_prev);
            prev_interval = Some(curr_interval);
            return;
        }

        let (prev_start, prev_end) = unwrapped_prev.get_pair();
        let (curr_start, curr_end) = curr_interval.get_pair();
        if prev_start == curr_start {
            // same prefix, different suffix
            let (min, max) = util::minmax(prev_end, curr_end);
            prev_interval = Some(Interval::from((min + 1, max)));
        } else if prev_end == curr_end {
            // different prefix, same suffix
            let (min, max) = util::minmax(prev_start, curr_start);
            prev_interval = Some(Interval::from((min, max - 1)));
        } else {
            // two disjoint resultant sets
            let (min_end, max_end) = util::minmax(prev_end, curr_end);

            let left = Interval::from((prev_start, curr_start - 1));
            let right = Interval::from((min_end + 1, max_end));

            visitor.visit_interval(left);
            prev_interval = Some(right);
        }
    };

    BivariateOrderedIterator::new(lhs.vec.iter(), rhs.vec.iter()).for_each(|i| xor_append(&i));
    if prev_interval.is_some() {
        visitor.visit_interval(prev_interval.unwrap());
    }
}

pub fn and(lhs: &RunStore, rhs: &RunStore, visitor: &mut impl BinaryOperationVisitor) {
    if lhs.is_full() {
        visitor.visit_run_store(rhs);
        return;
    }

    if rhs.is_full() {
        visitor.visit_run_store(lhs);
        return;
    }

    let mut prev_interval: Option<Interval> = None;
    let mut and_append = |curr_interval: &Interval| {
        let curr_interval = curr_interval.clone();

        let Some(unwrapped_interval) = prev_interval else {
            prev_interval = Some(curr_interval);
            return;
        };

        // same interval, automatically append it
        if unwrapped_interval == curr_interval {
            visitor.visit_interval(curr_interval);
            prev_interval = None;
            return;
        }

        // no overlap, set previous interval to currenet
        if unwrapped_interval.get_end() + 1 < curr_interval.value {
            prev_interval = Some(curr_interval);
            return;
        }

        let (prev_start, prev_end) = unwrapped_interval.get_pair();
        let (curr_start, curr_end) = curr_interval.get_pair();
        if prev_start == curr_start {
            // same prefix, different suffix
            let (min, max) = util::minmax(prev_end, curr_end);
            visitor.visit_interval(Interval::from((prev_start, min)));
            prev_interval = Some(Interval::from((min + 1, max)));
        } else if curr_start < prev_start {
            // same prefix, different suffix
            let (min, max) = util::minmax(prev_end, curr_end);
            visitor.visit_interval(Interval::from((curr_start, min)));
            prev_interval = Some(Interval::from((min + 1, max)));
        } else if prev_start <= curr_start {
            if prev_end < curr_start {
                // no overlap with previous interval; discard
                prev_interval = Some(curr_interval);
                return;
            } else if prev_end <= curr_end {
                // overlap suffix of left with prefix of right
                visitor.visit_interval(Interval::from((curr_start, prev_end)));
                prev_interval = Some(Interval::from((prev_end + 1, curr_end)));
            } else {
                // interval contains `other` interval, causing all of its elements
                // to be admitted, starting the new interval from the number after
                // the last shared number.
                visitor.visit_interval((Interval::from((curr_start, curr_end))));
                prev_interval = Some(Interval::from((curr_end + 1, prev_end)));
                return;
            }
        } else {
            prev_interval = Some(curr_interval);
        }
    };

    BivariateOrderedIterator::new(lhs.vec.iter(), rhs.vec.iter()).for_each(|i| and_append(&i));
}

#[cfg(test)]
mod tests {
    use crate::bitmap::store::rle_store;
    use rle_store::interval::Interval;
    use rle_store::visitor::RunWriter;
    use rle_store::RunStore;

    use super::{and, or, xor};

    macro_rules! create_run_store {
        [$(($arg1:expr,$arg2:expr)),*] => {
            {
                RunStore::try_from(vec![
                    $(Interval::from(($arg1, $arg2)),)*
                ]).unwrap()
            }
        };
    }

    macro_rules! run_commutative_binary_op_test {
        (
            $f:ident,
            $(expected =)? [$(($arg1:expr,$arg2:expr)),*],
            $(left =)? [$(($arg3:expr,$arg4:expr)),*],
            $(right =)? [$(($arg5:expr,$arg6:expr)),*]
            $(,)?
        ) => {{
            let left = create_run_store![$(($arg3, $arg4)),*];
            let right = create_run_store![$(($arg5, $arg6)),*];
            let expected = create_run_store![$(($arg1, $arg2)),*];

            let mut visitor = RunWriter::new();
            $f(&(left.clone()), &(right.clone()), &mut visitor);
            let got = visitor.into_inner();
            assert_eq!(expected, got);

            let mut visitor = RunWriter::new();
            $f(&(right.clone()), &(left.clone()), &mut visitor);
            let got = visitor.into_inner();
            assert_eq!(expected, got);
        }};
    }

    #[test]
    fn test_or() {
        // chained
        run_commutative_binary_op_test!(
            or,
            expected = [(1, 10)],
            left = [(1, 5)],
            right = [(6, 10)]
        );

        // interspersed
        run_commutative_binary_op_test!(
            or,
            expected = [(1, 7)],
            left = [(1, 3), (5, 7)],
            right = [(4, 4)]
        );
        run_commutative_binary_op_test!(
            or,
            expected = [(1, 8)],
            left = [(1, 3), (5, 7)],
            right = [(2, 4), (6, 8)]
        );

        // extension
        run_commutative_binary_op_test!(
            or,
            expected = [(1, 3), (5, 10), (12, 15)],
            left = [(1, 3), (5, 7)],
            right = [(7, 10), (12, 15)]
        );

        // trailing
        run_commutative_binary_op_test!(
            or,
            expected = [(1, 3), (5, 7), (9, 11)],
            left = [(1, 3), (5, 7), (9, 11)],
            right = []
        );

        // filled
        run_commutative_binary_op_test!(
            or,
            expected = [(0, 0xFFFF)],
            left = [(0, 0xFFFF)],
            right = []
        );

        // interjected
        run_commutative_binary_op_test!(
            or,
            expected = [(1, 2), (4, 5), (7, 8)],
            left = [(1, 2), (7, 8)],
            right = [(4, 5)],
        );
    }

    #[test]
    fn test_xor() {
        // shared middle
        run_commutative_binary_op_test!(
            xor,
            expected = [(0, 4), (8, 12)],
            left = [(0, 12)],
            right = [(5, 7)]
        );

        // merge
        run_commutative_binary_op_test!(
            xor,
            expected = [(0, 8)],
            left = [(0, 4)],
            right = [(5, 8)]
        );

        // no merge
        run_commutative_binary_op_test!(
            xor,
            expected = [(0, 4), (6, 10)],
            left = [(0, 4)],
            right = [(6, 10)],
        );

        // cancel the same element
        run_commutative_binary_op_test!(
            xor,
            expected = [(0, 4), (8, 12)],
            left = [(0, 12), (15, 25)],
            right = [(5, 7), (15, 25)]
        );

        // same prefix, different suffix
        run_commutative_binary_op_test!(
            xor,
            expected = [(9, 12)],
            left = [(0, 8)],
            right = [(0, 12)],
        );

        // same suffix, different prefix
        run_commutative_binary_op_test!(
            xor,
            expected = [(0, 3)],
            left = [(0, 12)],
            right = [(4, 12)],
        )
    }

    #[test]
    fn test_and() {
        // same set
        run_commutative_binary_op_test!(
            and,
            expected = [(0, 10)],
            left = [(0, 10)],
            right = [(0, 10)],
        );

        // same prefix, different suffix
        run_commutative_binary_op_test!(
            and,
            expected = [(0, 10)],
            left = [(0, 20)],
            right = [(0, 10)],
        );

        // disjoint sets
        run_commutative_binary_op_test!(
            and,
            expected = [],
            left = [(0, 10)],
            right = [(11, 20)],
        );

        // subset
        run_commutative_binary_op_test!(
            and,
            expected = [(0, 5), (10, 15)],
            left = [(0, 20)],
            right = [(0, 5), (10, 15)],
        );

        // interwoven
        run_commutative_binary_op_test!(
            and,
            expected = [(2, 4), (8, 10)],
            left = [(2, 10)],
            right = [(0, 4), (8, 12)],
        );
    }
}
