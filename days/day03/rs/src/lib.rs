#![no_std]

#[cfg(feature = "rayon")]
use rayon::prelude::*;

#[allow(clippy::cast_possible_truncation)]
const POW: [u64; 12] = const {
    let mut r = [0_u64; 12];

    let mut i = 0;
    while i < 12 {
        r[i] = 10_u64.pow(i as u32);
        i += 1;
    }
    r
};

/// # Panics
#[must_use]
fn solve<const SIZE: usize>(data: &str) -> u64 {
    #[cfg(feature = "rayon")]
    let i = data.par_lines();

    #[cfg(not(feature = "rayon"))]
    let i = data.lines();

    i.map(|line| {
        let line = line.as_bytes();

        let mut current = [0; SIZE];
        let mut max = 0;
        for battery in line.iter().map(|value| u64::from(value - b'0')) {
            for i in 0..SIZE {
                let candidate_max = current
                    .iter()
                    .enumerate()
                    .filter_map(|(ii, value)| if i == ii { None } else { Some(*value) })
                    .chain(core::iter::once(battery))
                    .rev()
                    .enumerate()
                    .map(|(e, value)| value * POW[e])
                    .sum::<u64>();
                if candidate_max > max {
                    current.copy_within(i + 1.., i);
                    current[SIZE - 1] = battery;
                    max = candidate_max;
                    break;
                }
            }
        }

        max
    })
    .sum()
}

/// # Panics
#[cfg_attr(target_os = "none", inline(never))]
#[must_use]
pub fn part_1(data: &str) -> u64 {
    solve::<2>(data)
}

/// # Panics
#[cfg_attr(target_os = "none", inline(never))]
#[must_use]
pub fn part_2(data: &str) -> u64 {
    solve::<12>(data)
}

#[cfg(test)]
mod tests {
    use super::*;

    const INPUT: &str = r"987654321111111
811111111111119
234234234234278
818181911112111";

    #[test]
    fn test_part_1() {
        assert_eq!(part_1(INPUT), 357);
    }

    #[test]
    #[allow(clippy::unreadable_literal)]
    fn test_part_2() {
        assert_eq!(part_2(INPUT), 3121910778619);
    }
}
