/// # Panics
#[must_use]
pub fn part_1(data: &str) -> usize {
    let mut current = 50;
    data.lines()
        .filter(|line| {
            let mut chars = line.chars();
            let rotations = match (chars.next(), chars.as_str().parse::<i64>()) {
                (Some('L'), Ok(rotations)) => -rotations,
                (Some('R'), Ok(rotations)) => rotations,
                _ => unreachable!(),
            };
            current = (current + rotations).rem_euclid(100);

            current == 0
        })
        .count()
}

/// # Panics
#[must_use]
pub fn part_2(data: &str) -> i64 {
    let mut current = 50;
    data.lines()
        .map(|line| {
            let mut chars = line.chars();
            let (dir, rotations) = match (chars.next(), chars.as_str().parse::<i64>()) {
                (Some('L'), Ok(rotations)) => (-1, rotations),
                (Some('R'), Ok(rotations)) => (1, rotations),
                _ => unreachable!(),
            };

            let old = current;

            let rotated = current + dir * rotations;
            current = rotated.rem_euclid(100);
            let count = if dir > 0 {
                rotated / 100
            } else {
                i64::from(current == 0) - rotated.div_euclid(100) - i64::from(old == 0)
            };

            count
        })
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    const INPUT: &str = r"L68
L30
R48
L5
R60
L55
L1
L99
R14
L82";

    #[test]
    fn test_part_1() {
        assert_eq!(part_1(INPUT), 3);
    }

    #[test]
    fn test_part_2() {
        assert_eq!(part_2(INPUT), 6);
    }
}
