#![no_std]

use core::{mem, ops};

type Point = (i32, i32, i32);

trait Set {
    fn is_set(&self, element: usize) -> bool;
    fn set(&mut self, element: usize);
    fn union(&mut self, other: &Self);
    fn size(&self) -> usize;
}

trait Num
where
    Self: Copy,
    Self: ops::Shl<usize, Output = Self> + ops::BitAnd<Output = Self> + ops::BitOrAssign,
    Self: PartialEq,
{
    const BITS: usize;
    const ZERO: Self;
    const ONE: Self;

    fn count_ones(self) -> usize;
}

impl Num for u128 {
    const BITS: usize = const { mem::size_of::<u128>() * 8 };
    const ZERO: Self = 0;
    const ONE: Self = 1;

    fn count_ones(self) -> usize {
        u128::count_ones(self) as usize
    }
}

impl<T> Set for [T; 8]
where
    T: Num,
{
    fn is_set(&self, element: usize) -> bool {
        (self[element / T::BITS] & ((T::ONE) << (element % T::BITS))) != T::ZERO
    }

    fn set(&mut self, element: usize) {
        self[element / T::BITS] |= (T::ONE) << (element % T::BITS);
    }

    fn union(&mut self, other: &Self) {
        for (element, other) in self.iter_mut().zip(other) {
            *element |= *other;
        }
    }

    fn size(&self) -> usize {
        self.iter().map(|value| value.count_ones()).sum()
    }
}

struct Merge<'a> {
    sets: &'a mut [[u128; 8]],
    len: usize,
}

impl<'a> Merge<'a> {
    fn new(sets: &'a mut [[u128; 8]], size: usize) -> Self {
        for (i, set) in sets.iter_mut().enumerate().take(size) {
            set.set(i);
        }
        Self { sets, len: size }
    }

    fn merge(&mut self, i: usize, j: usize) {
        let index_i = self
            .sets
            .iter()
            .take(self.len)
            .position(|set| set.is_set(i))
            .unwrap();
        let index_j = self
            .sets
            .iter()
            .take(self.len)
            .position(|set| set.is_set(j))
            .unwrap();
        if index_i != index_j {
            let (index_i, index_j) = (index_i.min(index_j), index_i.max(index_j));
            let (low, high) = self.sets[..self.len].split_at_mut(index_j);
            low[index_i].union(&high[0]);
            if high.len() > 1 {
                let (low, high) = high.split_at_mut(high.len() - 1);
                low[0] = high[0];
            }
            self.len -= 1;
        }
    }

    fn iter(&self) -> impl Iterator<Item = &[u128; 8]> {
        self.sets[..self.len].iter()
    }
}

/// # Panics
fn parse(line: &str) -> Point {
    let mut parts = line.split(',');
    match (parts.next(), parts.next(), parts.next(), parts.next()) {
        (Some(x), Some(y), Some(z), None) => (
            x.parse().expect("Invalid x"),
            y.parse().expect("Invalid y"),
            z.parse().expect("Invalid z"),
        ),
        _ => unreachable!("Invalid input"),
    }
}

#[allow(clippy::cast_possible_truncation)]
fn distance(p1: &Point, p2: &Point) -> Option<i32> {
    let (dx, dy, dz) = (
        i64::from(p1.0 - p2.0),
        i64::from(p1.1 - p2.1),
        i64::from(p1.2 - p2.2),
    );
    let distance = dx * dx + dy * dy + dz * dz;
    if distance > 200_000_000 {
        None
    } else {
        Some(distance as i32)
    }
}

/// # Panics
#[must_use]
#[allow(clippy::large_stack_arrays, clippy::cast_possible_truncation)]
pub fn part_1<const SIZE: usize>(data: &str) -> u32 {
    let mut junctions = [(0, 0, 0); 1000];
    let junctions_len = data
        .lines()
        .zip(junctions.iter_mut())
        .map(|(line, element)| {
            *element = parse(line);
        })
        .count();

    let mut distances = [(0, (0, 0)); 6000];
    let len = junctions
        .iter()
        .take(junctions_len - 1)
        .enumerate()
        .flat_map(|(i, p1)| {
            junctions
                .iter()
                .enumerate()
                .take(junctions_len)
                .skip(i + 1)
                .filter_map(move |(j, p2)| distance(p1, p2).map(|d| (d, (i as u16, j as u16))))
        })
        .zip(distances.iter_mut())
        .map(|(value, element)| {
            *element = value;
        })
        .count();
    assert!(len < 6000);
    distances[..len].sort_unstable();

    let mut sets = [[0u128; 8]; 1000];
    let mut sets = Merge::new(&mut sets, junctions_len);
    for &(_, (i, j)) in distances.iter().take(SIZE) {
        sets.merge(i as usize, j as usize);
    }

    let mut result = [0u32; 1000];
    let len = sets
        .iter()
        .zip(result.iter_mut())
        .map(|(set, item)| *item = set.size() as u32)
        .count();
    result[..len].sort_unstable();

    result[..len].iter().rev().take(3).product()
}

/// # Panics
#[must_use]
#[allow(clippy::large_stack_arrays, clippy::cast_possible_truncation)]
pub fn part_2(data: &str) -> i64 {
    let mut junctions = [(0, 0, 0); 1000];
    let junctions_len = data
        .lines()
        .zip(junctions.iter_mut())
        .map(|(line, element)| {
            *element = parse(line);
        })
        .count();

    let mut distances = [(0, (0, 0)); 6000];
    let len = junctions
        .iter()
        .take(junctions_len - 1)
        .enumerate()
        .flat_map(|(i, p1)| {
            junctions
                .iter()
                .enumerate()
                .take(junctions_len)
                .skip(i + 1)
                .filter_map(move |(j, p2)| distance(p1, p2).map(|d| (d, (i as u16, j as u16))))
        })
        .zip(distances.iter_mut())
        .map(|(value, element)| {
            *element = value;
        })
        .count();
    assert!(len < 6000);
    distances[..len].sort_unstable();

    let mut sets = [[0u128; 8]; 1000];
    let mut sets = Merge::new(&mut sets, junctions_len);
    for &(_, (i, j)) in distances.iter().take(len) {
        sets.merge(i as usize, j as usize);
        if sets.len == 1 {
            return i64::from(junctions[i as usize].0) * i64::from(junctions[j as usize].0);
        }
    }

    unreachable!()
}

#[cfg(test)]
mod tests {
    use super::*;

    const INPUT: &str = r"162,817,812
57,618,57
906,360,560
592,479,940
352,342,300
466,668,158
542,29,236
431,825,988
739,650,466
52,470,668
216,146,977
819,987,18
117,168,530
805,96,715
346,949,466
970,615,88
941,993,340
862,61,35
984,92,344
425,690,689";

    #[test]
    fn test_part_1() {
        assert_eq!(part_1::<10>(INPUT), 40);
    }

    #[test]
    fn test_part_2() {
        assert_eq!(part_2(INPUT), 25272);
    }
}
