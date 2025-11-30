#![no_std]

use core::{marker, mem, ops};

const BITS: usize = mem::size_of::<u128>() * 8;

#[derive(Debug)]
pub struct Error;

// Cannot be copy: this structure can be too heavy
#[derive(Clone)]
pub struct BitSet<T, K, const SIZE: usize> {
    data: [u128; SIZE],
    key: K,
    _marker: marker::PhantomData<T>,
}

impl BitSet<(), fn(&()) -> usize, 0> {
    pub const fn with_capacity(size: usize) -> usize {
        debug_assert!(size > 0, "invalid size");
        
        size / BITS + if size % BITS == 0 { 0 } else { 1 }
    }
}

impl<T, K, const SIZE: usize> BitSet<T, K, SIZE> {
    pub fn len(&self) -> usize {
        self.data
            .iter()
            .map(|value| value.count_ones() as usize)
            .sum()
    }

    pub fn is_empty(&self) -> bool {
        self.data.iter().all(|&value| value == 0)
    }
}

impl<T, K: Fn(&T) -> usize, const SIZE: usize> BitSet<T, K, SIZE> {
    pub const fn new(key: K) -> Self {
        Self {
            data: [0; SIZE],
            key,
            _marker: marker::PhantomData,
        }
    }

    #[allow(clippy::needless_pass_by_value)]
    pub fn insert(&mut self, idx: T) -> Result<bool, Error> {
        let idx = (self.key)(&idx);
        let (i, b) = (idx / BITS, idx % BITS);

        let data = self.data.get_mut(i).ok_or(Error)?;

        let mask = 1 << b;

        let result = *data & mask != 0;

        *data |= mask;

        Ok(result)
    }

    pub fn contains(&self, idx: &T) -> Result<bool, Error> {
        let idx = (self.key)(idx);
        let (i, b) = (idx / BITS, idx % BITS);

        Ok(self.data.get(i).ok_or(Error)? & (1 << b) != 0)
    }

    pub fn remove(&mut self, idx: &T) -> Result<bool, Error> {
        let idx = (self.key)(idx);
        let (i, b) = (idx / BITS, idx % BITS);

        let data = self.data.get_mut(i).ok_or(Error)?;
        
        let mask = 1 << b;
        
        let result = *data & mask != 0;
        
        *data &= !mask;

        Ok(result)
    }
}

impl<T, K, const SIZE: usize> BitSet<T, K, SIZE>
where
    K: Fn(&T) -> usize + Clone,
{
    pub fn key(&self) -> K {
        self.key.clone()
    }
}

impl<T, K, const SIZE: usize> ops::BitOrAssign for BitSet<T, K, SIZE> {
    fn bitor_assign(&mut self, other: Self) {
        for (a, b) in self.data.iter_mut().zip(other.data.iter()) {
            *a |= b;
        }
    }
}
