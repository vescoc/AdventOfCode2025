#![no_std]

/// # Panics
#[must_use]
pub fn part_1(data: &str) -> usize {
    data.split_once("\n\n")
        .map(|(ranges, ids)| {
            let mut id_ranges = [(0, 0); 200];
            let len = ranges
                .lines()
                .zip(id_ranges.iter_mut())
                .map(|(line, value)| {
                    let (low, high) = line.split_once('-').expect("invalid line");
                    *value = (
                        low.parse::<u64>().expect("invalid low"),
                        high.parse::<u64>().expect("invalid high"),
                    );
                })
                .count();
            id_ranges[0..len].sort_unstable();

            ids.lines()
                .filter(|line| {
                    let id = line.parse::<u64>().expect("invalid id");
                    for &(start, end) in &id_ranges {
                        if id >= start {
                            if id <= end {
                                return true;
                            }
                        } else {
                            break;
                        }
                    }
                    false
                })
                .count()
        })
        .expect("invalid input")
}

/// # Panics
#[must_use]
pub fn part_2(data: &str) -> u64 {
    data.split_once("\n\n")
        .map(|(ranges, _)| {
            let mut id_ranges = [(0, 0); 200];
            let len = ranges
                .lines()
                .zip(id_ranges.iter_mut())
                .map(|(line, value)| {
                    let (low, high) = line.split_once('-').expect("invalid line");
                    *value = (
                        low.parse::<u64>().expect("invalid low"),
                        high.parse::<u64>().expect("invalid high"),
                    );
                })
                .count();
            id_ranges[0..len].sort_unstable();

            let (current, remainder) = id_ranges[0..len].split_first().expect("invalid input");

            let mut count = 0;
            let (mut current_start, mut current_end) = *current;
            for &(start, end) in remainder {
                if start > current_end + 1 {
                    count += current_end - current_start + 1;
                    current_start = start;
                }
                current_end = current_end.max(end);
            }

            count + current_end - current_start + 1
        })
        .expect("invalid input")
}

#[cfg(test)]
mod tests {
    use super::*;

    const INPUT: &str = r"3-5
10-14
16-20
12-18

1
5
8
11
17
32";

    #[test]
    fn test_part_1() {
        assert_eq!(part_1(INPUT), 3);
    }

    #[test]
    fn test_part_2() {
        assert_eq!(part_2(INPUT), 14);
    }
}
