#![no_std]

#[cfg(feature = "rayon")]
use rayon::prelude::*;

#[allow(clippy::cast_possible_truncation)]
const fn div(l: usize, mut rep: usize) -> (u64, u64) {
    let mul = 10_u64.pow(l as u32);
    let mut r = 1;
    while rep > 1 {
        r = r * mul + 1;
        rep -= 1;
    }
    (r, mul)
}

const DIV1_LEN: usize = 6;
const DIV1: [(u64, u64); DIV1_LEN] = const {
    let mut r = [(1, 1); DIV1_LEN];
    let mut i = 1;
    while i < r.len() {
        r[i] = div(i, 2);
        i += 1;
    }
    r
};

const DIV2_LEN: usize = 11;
const DIV2_SUBLEN: usize = 2;
const DIV2: [[(u64, u64); DIV2_SUBLEN]; DIV2_LEN] = const {
    let mut r = [[(0, 0); DIV2_SUBLEN]; DIV2_LEN];
    let mut len = 2;
    while len <= r.len() {
        let mut skipped = false;
        let mut i = 0;
        let mut l = 1;
        while len / l >= 1 {
            if len % l == 0 && len / l > 1 {
                if i == 1 && !skipped {
                    i = 0;
                    skipped = true;
                }

                r[len - 1][i] = div(l, len / l);
                i += 1;
            }
            l += 1;
        }
        len += 1;
    }
    r[0][0] = (1, 1);
    r
};

const DIV2_INDEX: [usize; DIV2_LEN] = const {
    let mut r = [0; DIV2_LEN];
    let mut i = 0;
    while i < DIV2.len() {
        let mut l = 0;
        while l < DIV2_SUBLEN && DIV2[i][l].0 != 0 {
            l += 1;
        }
        r[i] = l;
        i += 1;
    }
    r
};

const fn len(id: u64) -> usize {
    if id < 10 {
        1
    } else if id < 100 {
        2
    } else if id < 1000 {
        3
    } else if id < 10_000 {
        4
    } else if id < 100_000 {
        5
    } else if id < 1_000_000 {
        6
    } else if id < 10_000_000 {
        7
    } else if id < 100_000_000 {
        8
    } else if id < 1_000_000_000 {
        9
    } else if id < 10_000_000_000 {
        10
    } else {
        unreachable!()
    }
}

fn sum_invalid_ids(low: u64, high: u64) -> u64 {
    let low_len = len(low);
    let high_len = len(high);
    if low_len == high_len {
        if low_len % 2 == 1 {
            return 0;
        }

        let (div, mask) = DIV1[low_len / 2];
        ((mask / 10).max(1)..mask)
            .filter_map(|n| {
                let n = n * div;
                if (low..=high).contains(&n) {
                    Some(n)
                } else {
                    None
                }
            })
            .sum()
    } else {
        let (low_div, low_mask) = DIV1[low_len / 2];
        let (high_div, high_mask) = DIV1[high_len / 2];
        ((low_mask / 10).max(1)..high_mask)
            .filter_map(|n| {
                if n < low_mask {
                    let n = n * low_div;
                    if (low..=high).contains(&n) {
                        return Some(n);
                    }
                } else {
                    let n = n * high_div;
                    if (low..=high).contains(&n) {
                        return Some(n);
                    }
                }

                None
            })
            .sum()
    }
}

fn sum_invalid_ids_m(low: u64, high: u64) -> u64 {
    let low_len = len(low);
    let high_len = len(high);
    if low_len == high_len {
        let divs = &DIV2[low_len - 1][..DIV2_INDEX[low_len - 1]];
        divs.iter()
            .enumerate()
            .flat_map(|(i, (div, mask))| {
                ((mask / 10).max(1)..*mask)
                    .map(move |n| n * div)
                    .filter(move |n| {
                        (low..=high).contains(n)
                            && divs
                                .iter()
                                .take(i)
                                .all(|(div, mask)| !(n % div == 0 && n / div == n % mask))
                    })
            })
            .sum()
    } else {
        let divs = [
            &DIV2[low_len - 1][..DIV2_INDEX[low_len - 1]],
            &DIV2[high_len - 1][..DIV2_INDEX[high_len - 1]],
        ];
        divs.into_iter()
            .flatten()
            .enumerate()
            .flat_map(|(i, (div, mask))| {
                ((mask / 10).max(1)..*mask)
                    .map(move |n| n * div)
                    .filter(move |n| {
                        (low..=high).contains(n)
                            && divs
                                .into_iter()
                                .flatten()
                                .take(i)
                                .all(|(div, mask)| !(n % div == 0 && n / div == n % mask))
                    })
            })
            .sum()
    }
}

/// # Panics
fn solve(data: &str, f: impl Fn(u64, u64) -> u64 + Sync + Send) -> u64 {
    #[cfg(feature = "rayon")]
    let i = data.trim().par_split(',');

    #[cfg(not(feature = "rayon"))]
    let i = data.trim().split(',');

    i.map(|range| {
        let (low, high) = range.split_once('-').expect("invalid range");
        f(
            low.parse().expect("invalid low"),
            high.parse().expect("invalid high"),
        )
    })
    .sum()
}

/// # Panics
#[must_use]
pub fn part_1(data: &str) -> u64 {
    solve(data, sum_invalid_ids)
}

/// # Panics
#[must_use]
pub fn part_2(data: &str) -> u64 {
    solve(data, sum_invalid_ids_m)
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

    #[test]
    #[allow(clippy::unreadable_literal)]
    fn test_sum_invalid_ids() {
        assert_eq!(sum_invalid_ids(11, 22), 33);
        assert_eq!(sum_invalid_ids(95, 115), 99);
        assert_eq!(sum_invalid_ids(998, 1012), 1010);
        assert_eq!(sum_invalid_ids(1188511880, 1188511890), 1188511885);
    }

    #[test]
    #[allow(clippy::unreadable_literal)]
    fn test_sum_invalid_ids_m() {
        assert_eq!(sum_invalid_ids_m(11, 22), 11 + 22);
        assert_eq!(sum_invalid_ids_m(95, 115), 99 + 111);
        assert_eq!(sum_invalid_ids_m(998, 1012), 999 + 1010);
        assert_eq!(sum_invalid_ids_m(1188511880, 1188511890), 1188511885);
    }

    #[test]
    #[allow(clippy::unreadable_literal)]
    fn test_div2() {
        assert_eq!(div(2, 2), (101, 100));
        assert_eq!(div(1, 2), (11, 10));
        assert_eq!(div(3, 2), (1001, 1000));
        assert_eq!(div(3, 3), (1001001, 1000));
    }
}
