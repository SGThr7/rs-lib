use super::*;
use num_traits_zero::Zero;
use std::ops::BitOr;

pub struct BitOrAlge<T>(PhantomData<T>);

impl<T> Semigroup for BitOrAlge<T>
where
    for<'a> &'a T: BitOr<Output = T>,
{
    type Set = T;

    fn operate(lhs: &Self::Set, rhs: &Self::Set) -> Self::Set {
        lhs | rhs
    }
}

impl<T> Monoid for BitOrAlge<T>
where
    T: Zero,
    for<'a> &'a T: BitOr<Output = T>,
{
    fn id() -> Self::Set {
        T::ZERO
    }
}
