#![no_std]

#[cfg(feature = "rayon")]
use rayon::prelude::*;

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
        let mut max = u64::MIN;
        for battery in line.iter().map(|value| u64::from(value - b'0')) {
            for i in 0..SIZE {
                let mut candidate_max = 0;
                for (ii, value) in current.iter().enumerate() {
                    if ii != i {
                        candidate_max = candidate_max * 10 + value;
                    }
                }
                candidate_max = candidate_max * 10 + battery;

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
