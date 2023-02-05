use std::{
    cmp::Ordering,
    mem,
    ops::{Bound, Range, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive},
};

use bisect::Bisect;
use monoid::{Monoid, PartialGroup};

/// A struct that can update elements and calculate prefix sums fast.
///
/// # Time complexity
///
/// | Operation   | Complexity |
/// | ----------- | ---------- |
/// | Space       | Θ(n)       |
/// | [`operate`] | Θ(log n)   |
/// | [`fold`]    | Θ(log n)   |
///
/// [`operate`]: FenwickTree::operate
/// [`fold`]: FenwickTree::fold
#[derive(Clone)]
pub struct FenwickTree<M: Monoid> {
    tree: Vec<M::Set>,
}

pub type BIT<M> = FenwickTree<M>;

pub type BinaryIndexedTree<M> = FenwickTree<M>;

impl<M: Monoid> FenwickTree<M> {
    /// Creates an initialized Fenwick tree that size of `size` with [`<M as Monoid>::id()`].
    ///
    /// [`<M as Monoid>::id()`]: Monoid::id
    pub fn with_size(size: usize) -> Self {
        let tree = {
            let mut ret = Vec::with_capacity(size);
            ret.resize_with(size, M::id);
            ret
        };

        Self { tree }
    }

    #[allow(clippy::len_without_is_empty)]
    /// Returns the number of elements in the tree.
    pub fn len(&self) -> usize {
        self.tree.len()
    }

    // Note: Enable it if this struct can be extended.
    // /// Returns `true` if the tree has a length of 0.
    // pub fn is_empty(&self) -> bool {
    //     self.len() == 0
    // }

    /// Update a tree value with [`Semigroup::operate`].
    ///
    /// This operation is just Θ(log n).
    ///
    /// [`Semigroup::operate`]: monoid::Semigroup::operate
    ///
    /// # Examples
    ///
    /// ```
    /// use fenwick_tree::FenwickTree;
    /// use monoid::types::AddAlge;
    ///
    /// let mut bit: FenwickTree<AddAlge<_>> = vec![1, 2, 3, 4].into();
    /// assert_eq!(bit.fold(..), 10);
    ///
    /// bit.operate(2, &5);
    /// assert_eq!(bit.fold(..2), 3);
    /// assert_eq!(bit.fold(..), 15);
    /// ```
    pub fn operate(&mut self, index: usize, value: &M::Set) {
        let mut i = index;
        while i < self.len() {
            // drain
            let current = mem::replace(&mut self.tree[i], M::id());

            self.tree[i] = M::operate(value, &current);
            i += lsb(i + 1);
        }
    }

    /// Returns a folded value.
    /// The `index` can be passed [`RangeTo`], [`RangeToInclusive`] or [`RangeFull`].
    ///
    /// This operation is just Θ(log n).
    ///
    /// # Panics
    ///
    /// May panic if the range is out of bounds.
    ///
    /// # Examples
    ///
    /// ```
    /// use fenwick_tree::FenwickTree;
    /// use monoid::types::AddAlge;
    ///
    /// let bit: FenwickTree<AddAlge<_>> = vec![1, 2, 3, 4].into();
    ///
    /// assert_eq!(bit.fold(..2), 3);
    /// assert_eq!(bit.fold(..=2), 6);
    /// assert_eq!(bit.fold(..), 10);
    /// ```
    ///
    /// If type `T` of [`FenwickTree`] is implemented [`PartialGroup`],
    /// range literal including "from" is able to use.
    ///
    /// ```
    /// # use fenwick_tree::FenwickTree;
    /// # use monoid::types::AddAlge;
    /// # let bit: FenwickTree<AddAlge<_>> = vec![1, 2, 3, 4].into();
    /// use std::ops::Bound;
    ///
    /// assert_eq!(bit.fold(1..3), 5);
    /// assert_eq!(bit.fold(1..=3), 9);
    /// assert_eq!(bit.fold(1..), 9);
    /// assert_eq!(bit.fold((Bound::Excluded(1), Bound::Excluded(3))), 3);
    /// ```
    pub fn fold<I: Index<M>>(&self, index: I) -> M::Set {
        index.fold(self)
    }
}

impl<M: Monoid> From<&[M::Set]> for FenwickTree<M> {
    fn from(v: &[M::Set]) -> Self {
        let mut ret = Self::with_size(v.len());

        // set each value to tree
        v.iter().enumerate().for_each(|(i, x)| ret.operate(i, x));

        ret
    }
}

impl<M: Monoid> From<Vec<M::Set>> for FenwickTree<M> {
    fn from(v: Vec<M::Set>) -> Self {
        v.as_slice().into()
    }
}

/// Returns the least significant bit by `i`.
///
/// e.g.) `lsb(0b1010)` returns `0b10`
fn lsb(i: usize) -> usize {
    i & i.wrapping_neg()
}

#[cfg(test)]
#[test]
fn lsb_test() {
    assert_eq!(lsb(0b1010), 0b10);

    for i in 1..=(1e7 as usize) {
        let t = lsb(i);

        assert_eq!(t.count_ones(), 1);
        assert_eq!(t.trailing_zeros(), i.trailing_zeros());
    }
}

pub trait Index<T: Monoid> {
    fn fold(self, tree: &FenwickTree<T>) -> T::Set;
}

impl<T: Monoid> Index<T> for RangeTo<usize> {
    fn fold(self, tree: &FenwickTree<T>) -> T::Set {
        let mut ret = T::id();

        let mut i = self.end;
        while i > 0 {
            ret = T::operate(&tree.tree[i - 1], &ret);
            i -= lsb(i);
        }

        ret
    }
}

impl<T: Monoid> Index<T> for RangeToInclusive<usize> {
    fn fold(self, tree: &FenwickTree<T>) -> <T>::Set {
        (..self.end + 1).fold(tree)
    }
}

impl<T: Monoid> Index<T> for RangeFull {
    fn fold(self, tree: &FenwickTree<T>) -> <T>::Set {
        (..tree.len()).fold(tree)
    }
}

impl<T: PartialGroup> Index<T> for Range<usize> {
    fn fold(self, tree: &FenwickTree<T>) -> T::Set {
        T::inverse_operate(&(..self.end).fold(tree), &(..self.start).fold(tree))
    }
}

impl<T: PartialGroup> Index<T> for RangeInclusive<usize> {
    fn fold(self, tree: &FenwickTree<T>) -> T::Set {
        T::inverse_operate(&(..=*self.end()).fold(tree), &(..*self.start()).fold(tree))
    }
}

impl<T: PartialGroup> Index<T> for RangeFrom<usize> {
    fn fold(self, tree: &FenwickTree<T>) -> T::Set {
        T::inverse_operate(&(..).fold(tree), &(..self.start).fold(tree))
    }
}

impl<T: PartialGroup> Index<T> for (Bound<usize>, Bound<usize>) {
    fn fold(self, tree: &FenwickTree<T>) -> T::Set {
        let start = match self.0 {
            Bound::Included(i) => i,
            Bound::Excluded(i) => i + 1,
            Bound::Unbounded => 0,
        };
        let end = match self.1 {
            Bound::Included(i) => i + 1,
            Bound::Excluded(i) => i,
            Bound::Unbounded => tree.len(),
        };

        T::inverse_operate(&(..end).fold(tree), &(..start).fold(tree))
    }
}

impl<T: Monoid> Bisect for FenwickTree<T>
where
    T::Set: Clone,
{
    type Item = T::Set;

    fn find_range_by<F>(&self, mut f: F) -> Range<usize>
    where
        F: FnMut(&Self::Item) -> Ordering,
    {
        let mut len = self.len().next_power_of_two();
        let mut i = 0;
        let mut total = T::id();

        while len > 0 {
            if i + len - 1 < self.len() {
                let tmp = T::operate(&total, &self.tree[i + len - 1]);
                let cmp = f(&tmp);
                match cmp {
                    Ordering::Less => {
                        i += len;
                        total = tmp;
                    }
                    Ordering::Greater => (),
                    Ordering::Equal => break,
                }
            }

            len /= 2;
        }

        let mut lower_i = i;
        let mut lower_total = total.clone();
        let mut upper_i = i;
        let mut upper_total = total;
        while len > 0 {
            let lower_tmp = T::operate(&lower_total, &self.tree[i + len - 1]);
            let lower_cmp = f(&lower_tmp);
            match lower_cmp {
                Ordering::Less => {
                    lower_i += len;
                    lower_total = lower_tmp;
                }
                Ordering::Equal | Ordering::Greater => (),
            }

            let upper_tmp = T::operate(&upper_total, &self.tree[i + len - 1]);
            let upper_cmp = f(&upper_tmp);
            match upper_cmp {
                Ordering::Equal | Ordering::Less => {
                    upper_i += len;
                    upper_total = upper_tmp;
                }
                Ordering::Greater => (),
            }

            len /= 2;
        }

        lower_i..upper_i
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use monoid::types::AddAlge;

    #[test]
    fn with_size_and_len() {
        type Set = AddAlge<usize>;
        let bit = FenwickTree::<Set>::with_size(10);

        assert_eq!(bit.len(), 10);
        assert_eq!(bit.tree.len(), 10);
        assert_eq!(bit.tree, vec![Set::id(); 10]);
    }

    #[test]
    fn find_range_by() {
        let bit: FenwickTree<AddAlge<usize>> = vec![1, 2, 3, 4, 5, 6, 6, 8, 9, 10].into();

        assert_eq!(bit.find_range_by(|x| x.cmp(&1)), 0..1);
        assert_eq!(bit.find_range_by(|x| x.cmp(&2)), 1..1);
        assert_eq!(bit.find_range_by(|x| x.cmp(&3)), 1..2);
        (4..=5).for_each(|i| assert_eq!(bit.find_range_by(|x| x.cmp(&i)), 2..2));
        assert_eq!(bit.find_range_by(|x| x.cmp(&6)), 2..3);
        (7..=9).for_each(|i| assert_eq!(bit.find_range_by(|x| x.cmp(&i)), 3..3));
        assert_eq!(bit.find_range_by(|x| x.cmp(&10)), 3..4);
        (11..=14).for_each(|i| assert_eq!(bit.find_range_by(|x| x.cmp(&i)), 4..4));
        assert_eq!(bit.find_range_by(|x| x.cmp(&15)), 4..5);
        (16..=20).for_each(|i| assert_eq!(bit.find_range_by(|x| x.cmp(&i)), 5..5));
        assert_eq!(bit.find_range_by(|x| x.cmp(&21)), 5..6);
        (22..=26).for_each(|i| assert_eq!(bit.find_range_by(|x| x.cmp(&i)), 6..6));
        assert_eq!(bit.find_range_by(|x| x.cmp(&27)), 6..7);
        (28..=34).for_each(|i| assert_eq!(bit.find_range_by(|x| x.cmp(&i)), 7..7));
        assert_eq!(bit.find_range_by(|x| x.cmp(&35)), 7..8);
        (36..=43).for_each(|i| assert_eq!(bit.find_range_by(|x| x.cmp(&i)), 8..8));
        assert_eq!(bit.find_range_by(|x| x.cmp(&44)), 8..9);
        (45..=53).for_each(|i| assert_eq!(bit.find_range_by(|x| x.cmp(&i)), 9..9));
        assert_eq!(bit.find_range_by(|x| x.cmp(&54)), 9..10);
        (55..100).for_each(|i| assert_eq!(bit.find_range_by(|x| x.cmp(&i)), 10..10));
    }

    #[test]
    fn partition_point() {
        let bit: FenwickTree<AddAlge<usize>> = vec![1, 2, 3, 4, 5, 6, 6, 8, 9, 10].into();

        assert_eq!(bit.partition_point(|&x| x <= 20), 5);
        assert_eq!(bit.partition_point(|&x| x <= 21), 6);
    }
}
