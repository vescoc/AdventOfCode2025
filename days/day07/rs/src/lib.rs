#![no_std]

use core::{mem, ops};

trait Set {
    type Iter<'a>: Iterator<Item = usize>
    where
        Self: 'a;

    fn is_set(&self, p: usize) -> bool;
    fn set(&mut self, p: usize, value: bool) -> bool;
    fn size(&self) -> usize;
    fn intersection(&self, other: &Self) -> Self;
    fn reset_elements(&mut self, other: &Self);
    fn elements(&self) -> Self::Iter<'_>;
}

trait Num
where
    Self: Copy,
    Self: ops::Shl<usize, Output = Self>
        + ops::BitAnd<Output = Self>
        + ops::BitOrAssign
        + ops::BitAndAssign
        + ops::Not<Output = Self>,
    Self: PartialEq,
{
    const BITS: usize;
    const ZERO: Self;
    const ONE: Self;

    fn count_ones(&self) -> usize;
}

impl Num for u128 {
    const BITS: usize = const { mem::size_of::<u128>() * 8 };
    const ZERO: Self = 0;
    const ONE: Self = 1;

    fn count_ones(&self) -> usize {
        u128::count_ones(*self) as usize
    }
}

impl<T> Set for [T; 2]
where
    T: Num,
{
    type Iter<'a>
        = SetIter<'a, T>
    where
        T: 'a;

    fn is_set(&self, x: usize) -> bool {
        let (v, mask) = if x >= T::BITS {
            (&self[1], { T::ONE } << (x - T::BITS))
        } else {
            (&self[0], { T::ONE } << x)
        };

        *v & mask != T::ZERO
    }

    fn set(&mut self, x: usize, value: bool) -> bool {
        let (v, mask) = if x >= T::BITS {
            (&mut self[1], { T::ONE } << (x - T::BITS))
        } else {
            (&mut self[0], { T::ONE } << x)
        };

        let r = *v & mask != T::ZERO;
        if value {
            *v |= mask;
        } else {
            *v &= !mask;
        }
        r
    }

    fn size(&self) -> usize {
        self[0].count_ones() + self[1].count_ones()
    }

    fn intersection(&self, other: &Self) -> Self {
        [self[0] & other[0], self[1] & other[1]]
    }

    fn reset_elements(&mut self, other: &Self) {
        self[0] &= !other[0];
        self[1] &= !other[1];
    }

    fn elements(&self) -> Self::Iter<'_> {
        SetIter {
            set: self,
            index: 0,
        }
    }
}

struct SetIter<'a, T> {
    set: &'a [T; 2],
    index: usize,
}

impl<T: Num> Iterator for SetIter<'_, T> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.index < T::BITS * 2 {
                if self.set.is_set(self.index) {
                    let r = Some(self.index);
                    self.index += 1;
                    return r;
                }
                self.index += 1;
            } else {
                return None;
            }
        }
    }
}

/// # Panics
#[must_use]
pub fn part_1(data: &str) -> usize {
    let table = data.trim().as_bytes();
    let columns = table
        .iter()
        .position(|&tile| tile == b'\n')
        .expect("Invalid input");

    let mut beans = [0u128; 2];
    beans.set(
        table[..columns]
            .iter()
            .position(|&tile| tile == b'S')
            .expect("Invalid input"),
        true,
    );

    let mut total = 0;
    for row in table.chunks(columns + 1).skip(1) {
        let mut splitters = [0u128; 2];
        for (c, &tile) in row.iter().take(columns).enumerate() {
            if tile == b'^' {
                splitters.set(c, true);
            }
        }

        let splitted = beans.intersection(&splitters);
        total += splitted.size();

        beans.reset_elements(&splitted);
        for c in splitted.elements().take(columns) {
            if c > 0 {
                beans.set(c - 1, true);
            }
            if c < columns - 1 {
                beans.set(c + 1, true);
            }
        }
    }

    total
}

/// # Panics
#[must_use]
pub fn part_2(data: &str) -> u64 {
    let table = data.trim().as_bytes();
    let columns = table
        .iter()
        .position(|&tile| tile == b'\n')
        .expect("Invalid input");

    let mut state = [1u64; 200];
    for row in table.chunks(columns + 1).rev() {
        let mut new_state = [0; 200];
        for (c, &tile) in row.iter().take(columns).enumerate() {
            if tile == b'^' {
                new_state[c] = c.checked_sub(1).map_or(0, |c| state[c]) + {
                    let c = c + 1;
                    if c < columns { state[c] } else { 0 }
                };
            } else if tile == b'S' {
                return state[c];
            } else {
                new_state[c] = state[c];
            }
        }

        state = new_state;
    }

    unreachable!()
}

#[cfg(test)]
mod tests {
    use super::*;

    const INPUT: &str = r".......S.......
...............
.......^.......
...............
......^.^......
...............
.....^.^.^.....
...............
....^.^...^....
...............
...^.^...^.^...
...............
..^...^.....^..
...............
.^.^.^.^.^...^.
...............
";

    #[test]
    fn test_part_1() {
        assert_eq!(part_1(INPUT), 21);
    }

    #[test]
    fn test_part_2() {
        assert_eq!(part_2(INPUT), 40);
    }
}
