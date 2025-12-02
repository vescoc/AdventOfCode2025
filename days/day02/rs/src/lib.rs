#![no_std]

#[cfg(feature = "rayon")]
use rayon::prelude::*;

fn sum_invalid_ids(low: u64, high: u64) -> u64 {
    (low..=high)
        .filter(|id| {
            let id = heapless::format!(20; "{id}").unwrap();
            let id = id.as_bytes();
            let len = id.len();

            if len % 2 == 1 {
                return false;
            }

            id[0..len / 2] == id[len / 2..]
        })
        .sum()
}

fn sum_invalid_ids_m(low: u64, high: u64) -> u64 {
    (low..=high)
        .filter(|id| {
            let id = heapless::format!(20; "{id}").unwrap();
            let id = id.as_bytes();
            let len = id.len();

            let mut i = 1;
            while i <= len / 2 {
                if len % i == 0 && id.chunks_exact(i).skip(1).all(|v| &id[0..i] == v) {
                    return true;
                }
                i += 1;
            }

            false
        })
        .sum()
}

/// # Panics
#[must_use]
pub fn part_1(data: &str) -> u64 {
    #[cfg(feature = "rayon")]
    let i = data.trim().par_split(',');

    #[cfg(not(feature = "rayon"))]
    let i = data.trim().split(',');

    i.map(|range| {
        let (low, high) = range.split_once('-').expect("invalid range");
        let low_length = low.len();
        let high_length = high.len();
        if low_length == high_length {
            if low_length % 2 == 1 {
                return 0;
            }

            sum_invalid_ids(
                low.parse().expect("invalid low"),
                high.parse().expect("invalid high"),
            )
        } else {
            sum_invalid_ids(
                low.parse().expect("invalid low"),
                high.parse().expect("invalid high"),
            )
        }
    })
    .sum()
}

/// # Panics
#[must_use]
pub fn part_2(data: &str) -> u64 {
    #[cfg(feature = "rayon")]
    let i = data.trim().par_split(',');

    #[cfg(not(feature = "rayon"))]
    let i = data.trim().split(',');

    i.map(|range| {
        let (low, high) = range.split_once('-').expect("invalid range");
        sum_invalid_ids_m(
            low.parse().expect("invalid low"),
            high.parse().expect("invalid high"),
        )
    })
    .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    const INPUT: &str = r"11-22,95-115,998-1012,1188511880-1188511890,222220-222224,1698522-1698528,446443-446449,38593856-38593862,565653-565659,824824821-824824827,2121212118-2121212124";

    #[test]
    #[allow(clippy::unreadable_literal)]
    fn test_part_1() {
        assert_eq!(part_1(INPUT), 1227775554);
    }

    #[test]
    #[allow(clippy::unreadable_literal)]
    fn test_part_2() {
        assert_eq!(part_2(INPUT), 4174379265);
    }
}
