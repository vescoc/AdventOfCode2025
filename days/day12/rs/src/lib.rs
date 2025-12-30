#![no_std]

/// # Panics
#[cfg_attr(target_os = "none", inline(never))]
#[allow(clippy::cast_possible_truncation)]
#[must_use]
pub fn part_1(data: &str) -> usize {
    let mut shapes = [0; 6];
    for part in data.split("\n\n") {
        let mut lines = part.lines();
        let first_line = lines.next().unwrap();
        let (left, right) = first_line.split_once(':').expect("Invalid line");
        if right.is_empty() {
            let Some(d) = left.chars().next() else { unreachable!("id not found") };
            let id = d.to_digit(10).expect("Invalid id") as usize;
            let area = lines
                .map(|line| line.chars().filter(|tile| *tile == '#').count() as u32)
                .sum();
            shapes[id] = area;
        } else {
            return core::iter::once((left, right))
                .chain(lines.map(|line| line.split_once(": ").expect("Invalid line")))
                .filter(|(area, list)| {
                    let (w, l) = area.split_once('x').expect("Invalid region");

                    let area = w.parse::<u32>().expect("Invalid width")
                        * l.parse::<u32>().expect("Invalid length");
                    let sum = list
                        .split_whitespace()
                        .enumerate()
                        .map(|(id, num)| {
                            shapes[id] * num.parse::<u32>().expect("Invalid shape count")
                        })
                        .sum();

                    area >= sum
                })
                .count();
        }
    }

    unreachable!()
}

#[cfg(test)]
mod tests {
    use super::*;

    const INPUT: &str = r"0:
###
##.
##.

1:
###
##.
.##

2:
.##
###
##.

3:
##.
###
##.

4:
###
#..
###

5:
###
.#.
###

4x4: 0 0 0 0 2 0
12x5: 1 0 1 0 2 2
12x5: 1 0 1 0 3 2";

    #[test]
    fn test_part_1() {
		// the correct input "no heuristic" is 2
        assert_eq!(part_1(INPUT), 3);
    }
}
