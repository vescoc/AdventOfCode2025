use core::{mem, ops};

pub trait Set {
    fn set(&mut self, element: usize);
    #[allow(dead_code)]
    fn reset(&mut self, element: usize);
    #[allow(dead_code)]
    fn size(&self) -> usize;
    fn is_set(&self, element: usize) -> bool;
}

pub trait Num
where
    Self: Copy,
    Self: PartialEq,
    Self: ops::Shl<usize, Output = Self>
        + ops::BitOrAssign
        + ops::BitAndAssign
        + ops::BitAnd<Output = Self>
        + ops::Not<Output = Self>,
{
    const BITS: usize;
    const ZERO: Self;
    const ONE: Self;

    #[allow(dead_code)]
    fn count_ones(&self) -> usize;
}

impl Num for u16 {
    const BITS: usize = const { mem::size_of::<Self>() * 8 };
    const ZERO: Self = 0;
    const ONE: Self = 1;

    fn count_ones(&self) -> usize {
        u16::count_ones(*self) as usize
    }
}

impl Num for u128 {
    const BITS: usize = const { mem::size_of::<Self>() * 8 };
    const ZERO: Self = 0;
    const ONE: Self = 1;

    fn count_ones(&self) -> usize {
        u128::count_ones(*self) as usize
    }
}

impl<T: Num> Set for T {
    fn set(&mut self, element: usize) {
        *self |= (T::ONE) << element;
    }

    fn reset(&mut self, element: usize) {
        *self &= !((T::ONE) << element);
    }

    fn size(&self) -> usize {
        T::count_ones(self)
    }

    fn is_set(&self, element: usize) -> bool {
        *self & (T::ONE) << element != T::ZERO
    }
}

impl<const SIZE: usize, T: Num> Set for [T; SIZE] {
    fn set(&mut self, element: usize) {
        self[element / T::BITS] |= (T::ONE) << (element % T::BITS);
    }

    fn reset(&mut self, element: usize) {
        self[element / T::BITS] &= !((T::ONE) << (element % T::BITS));
    }

    fn size(&self) -> usize {
        self.iter().map(T::count_ones).sum()
    }

    fn is_set(&self, element: usize) -> bool {
        self[element / T::BITS] & (T::ONE) << (element % T::BITS) != T::ZERO
    }
}
