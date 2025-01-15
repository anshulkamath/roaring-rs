use core::borrow::BorrowMut;

use super::visitor::BinaryOperationVisitor;
use super::RunStore;

pub fn or(lhs: &RunStore, rhs: &RunStore, visitor: &mut impl BinaryOperationVisitor) {
    if lhs.is_full() {
        visitor.visit_run_store(lhs);
        return;
    }

    if rhs.is_full() {
        visitor.visit_run_store(rhs);
        return;
    }

    let mut l_iter = lhs.vec.iter().peekable();
    let mut r_iter = rhs.vec.iter().peekable();
    while l_iter.peek().is_some() && r_iter.peek().is_some() {
        let min_iter = if l_iter.peek().unwrap().value <= r_iter.peek().unwrap().value {
            l_iter.borrow_mut()
        } else {
            r_iter.borrow_mut()
        };
        visitor.visit_interval(min_iter.next().unwrap().clone());
    }

    l_iter.for_each(|i| visitor.visit_interval(i.clone()));
    r_iter.for_each(|i| visitor.visit_interval(i.clone()));
}

#[cfg(test)]
mod tests {
    use crate::bitmap::store::rle_store;
    use rle_store::interval::Interval;
    use rle_store::visitor::RunWriter;
    use rle_store::RunStore;

    use super::or;

    macro_rules! create_run_store {
        [$(($arg1:expr,$arg2:expr)),*] => {
            {
                RunStore::try_from(vec![
                    $(
                        Interval::from(($arg1, $arg2)),
                    )*
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
            $f(&left, &right, &mut visitor);
            let got = visitor.into_inner();
            assert_eq!(expected, got);

            let mut visitor = RunWriter::new();
            $f(&right, &left, &mut visitor);
            let got = visitor.into_inner();
            assert_eq!(expected, got);
        }};
    }

    #[test]
    fn test_or() {
        // chained
        run_commutative_binary_op_test!(or, expected = [(1, 10)], left = [(1, 5)], right = [(6, 10)]);

        // interspersed
        run_commutative_binary_op_test!(or, expected = [(1, 7)], left = [(1, 3), (5, 7)], right = [(4, 4)]);
        run_commutative_binary_op_test!(or, expected = [(1, 8)], left = [(1, 3), (5, 7)], right = [(2, 4), (6, 8)]);

        // extension
        run_commutative_binary_op_test!(or, expected = [(1, 3), (5, 10), (12, 15)], left = [(1, 3), (5, 7)], right = [(7, 10), (12, 15)]);

        // trailing
        run_commutative_binary_op_test!(or, expected = [(1, 3), (5, 7), (9, 11)], left = [(1, 3), (5, 7), (9, 11)], right = []);

        // filled
        run_commutative_binary_op_test!(or, expected = [(0, 0xFFFF)], left = [(0, 0xFFFF)], right = []);

        // interjected
        run_commutative_binary_op_test!(or, expected = [(1, 2), (4, 5), (7, 8)], left = [(1, 2), (7, 8)], right = [(4, 5)],);
    }
}
