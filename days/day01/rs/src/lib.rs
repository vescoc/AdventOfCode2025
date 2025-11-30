/// # Panics
#[must_use]
pub fn part_1(_input: &str) -> u32 {
    todo!()
}

/// # Panics
#[must_use]
pub fn part_2(_input: &str) -> u32 {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    const INPUT: &str = r"3   4
4   3
2   5
1   3
3   9
3   3";

    #[test]
    fn test_part_1() {
        assert_eq!(part_1(INPUT), 11);
    }

    #[test]
    fn test_part_2() {
        assert_eq!(part_2(INPUT), 31);
    }
}
