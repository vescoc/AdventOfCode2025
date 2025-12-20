#![no_std]

#[cfg(feature = "rayon")]
use rayon::prelude::*;

const NEIGHBORS: [(isize, isize); 8] = [
    (-1, -1),
    (-1, 0),
    (-1, 1),
    (0, -1),
    (0, 1),
    (1, -1),
    (1, 0),
    (1, 1),
];

/// # Panics
#[cfg_attr(target_os = "none", inline(never))]
#[must_use]
pub fn part_1(data: &str) -> usize {
    let data = &data.trim().as_bytes();
    let columns = data
        .iter()
        .position(|tile| *tile == b'\n')
        .expect("invalid input");
    let rows = (data.len() + 1) / (columns + 1);

    #[cfg(feature = "rayon")]
    let i = data.par_chunks(columns + 1);

    #[cfg(not(feature = "rayon"))]
    let i = data.chunks(columns + 1);

    i
        .enumerate()
        .map(|(y, row)| {
            row
                .iter()
                .take(columns)
                .enumerate()
                .filter(|(x, tile)| {
                    **tile == b'@'
                        && NEIGHBORS
                        .iter()
                        .filter(|(dx, dy)| {
                            matches!(
                                (x.checked_add_signed(*dx), y.checked_add_signed(*dy)),
                                (Some(x), Some(y)) if x < columns && y < rows && data[y * (columns + 1) + x] == b'@',
                            )
                        })
                        .count() < 4
                })
                .count()
        })
        .sum()
}

/// # Panics
#[cfg_attr(target_os = "none", inline(never))]
#[must_use]
pub fn part_2(data: &str) -> usize {
    let data = data.trim().as_bytes();
    let columns = data
        .iter()
        .position(|tile| *tile == b'\n')
        .expect("invalid input");
    let len = data.len();
    let rows = (len + 1) / (columns + 1);

    let mut buffer = [[0u8; 141 * 141]; 2];

    buffer[0][0..len].copy_from_slice(data);

    let mut result = 0;
    for i in core::iter::repeat(0..=1).flatten() {
        let (data, next) = {
            let (a, b) = buffer.split_first_mut().unwrap();
            if i == 0 {
                (a, &mut b[0])
            } else {
                (&mut b[0], a)
            }
        };

        next[..len].copy_from_slice(&data[..len]);
        let count = {
            #[cfg(feature = "rayon")]
            let i = data
                .par_chunks(columns + 1)
                .zip(next.par_chunks_mut(columns + 1));

            #[cfg(not(feature = "rayon"))]
            let i = data.chunks(columns + 1).zip(next.chunks_mut(columns + 1));

            i
                .enumerate()
                .map(|(y, (data_row, next_row))| {
                    let mut count = 0;
                    for (x, (tile, next_tile)) in data_row
                        .iter()
                        .zip(next_row.iter_mut())
                        .take(columns)
                        .enumerate()
                    {
                        let r = *tile == b'@'
                            && NEIGHBORS
                            .iter()
                            .filter(|(dx, dy)| {
                                matches!(
                                    (x.checked_add_signed(*dx), y.checked_add_signed(*dy)),
                                    (Some(x), Some(y)) if x < columns && y < rows && data[y * (columns + 1) + x] == b'@',
                                )
                            })
                            .count() < 4;
                        if r {
                            *next_tile = b'.';
                            count += 1;
                        }
                    }
                    count
                })
                .sum::<usize>()
        };

        if count == 0 {
            return result;
        }

        result += count;
    }

    unreachable!()
}

#[cfg(test)]
mod tests {
    use super::*;

    const INPUT: &str = r"..@@.@@@@.
@@@.@.@.@@
@@@@@.@.@@
@.@@@@..@.
@@.@@@@.@@
.@@@@@@@.@
.@.@.@.@@@
@.@@@.@@@@
.@@@@@@@@.
@.@.@@@.@.
";

    #[test]
    fn test_part_1() {
        assert_eq!(part_1(INPUT), 13);
    }

    #[test]
    fn test_part_2() {
        assert_eq!(part_2(INPUT), 43);
    }
}
