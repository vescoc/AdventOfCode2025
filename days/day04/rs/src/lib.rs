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
#[must_use]
pub fn part_2(data: &str) -> usize {
    let mut data = data.trim().as_bytes().to_vec();
    let columns = data
        .iter()
        .position(|tile| *tile == b'\n')
        .expect("invalid input");
    let rows = (data.len() + 1) / (columns + 1);

    let mut result = 0;
    loop {
        let mut next = data.clone();
        let count = {
            let data = &data;

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
                    for (x, (tile, next)) in data_row
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
                            *next = b'.';
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
        data = next;
    }
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
