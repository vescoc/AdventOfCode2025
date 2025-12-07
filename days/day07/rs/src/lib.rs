#![no_std]

/// # Panics
#[must_use]
pub fn part_1(data: &str) -> usize {
    let table = data.trim().as_bytes();
    let columns = table
        .iter()
        .position(|&tile| tile == b'\n')
        .expect("Invalid input");

    let bean_position = table[..columns]
        .iter()
        .position(|&tile| tile == b'S')
        .expect("invalid input");
    debug_assert!(bean_position < 128);
    let mut beans = [1u128 << bean_position, 0u128];

    let mask = if columns >= 128 {
        !(!0u128 << (columns - 128))
    } else {
        0u128
    };

    let mut total = 0;
    for row in table.chunks(columns + 1).skip(1) {
        let mut splitters = [0u128; 2];
        for (c, &tile) in row.iter().take(columns).enumerate() {
            if tile == b'^' {
                if c >= 128 {
                    splitters[1] |= 1 << (c - 128);
                } else {
                    splitters[0] |= 1 << c;
                }
            }
        }

        let splitted = [beans[0] & splitters[0], beans[1] & splitters[1]];
        total += (splitted[0].count_ones() + splitted[1].count_ones()) as usize;

        beans[0] = (beans[0] & !splitted[0])
            | (splitted[0] << 1)
            | (splitted[0] >> 1)
            | u128::from(splitted[1] & 1 != 0);
        beans[1] = ((beans[1] & !splitted[1])
            | ((splitted[1] << 1) | (splitted[1] >> 1))
            | u128::from(splitted[0] & (1 << 127) != 0))
            & mask;
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
