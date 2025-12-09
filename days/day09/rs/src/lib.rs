#![no_std]

#[cfg(feature = "rayon")]
use rayon::prelude::*;

/// # Panics
#[must_use]
pub fn part_1(data: &str) -> u64 {
    let mut tiles = [(0i64, 0i64); 500];
    let len = data
        .lines()
        .zip(tiles.iter_mut())
        .map(|(line, tile)| {
            let (x, y) = line.split_once(',').expect("Invalid input");
            *tile = match (x.parse(), y.parse()) {
                (Ok(x), Ok(y)) => (x, y),
                _ => unreachable!("Invalid line"),
            };
        })
        .count();

    #[cfg(feature = "rayon")]
    let i = tiles[..len - 1].into_par_iter();

    #[cfg(not(feature = "rayon"))]
    let i = tiles[..len - 1].iter();

    i.enumerate()
        .filter_map(|(i, (a_x, a_y))| {
            tiles
                .iter()
                .take(len)
                .skip(i + 1)
                .map(move |&(b_x, b_y)| (a_x.abs_diff(b_x) + 1) * (a_y.abs_diff(b_y) + 1))
                .max()
        })
        .max()
        .expect("no solutions")
}

/// # Panics
#[must_use]
pub fn part_2(data: &str) -> u64 {
    let mut tiles = [(0i64, 0i64); 500];
    let len = data
        .lines()
        .zip(tiles.iter_mut())
        .map(|(line, tile)| {
            let (x, y) = line.split_once(',').expect("Invalid input");
            *tile = match (x.parse(), y.parse()) {
                (Ok(x), Ok(y)) => (x, y),
                _ => unreachable!("Invalid line"),
            };
        })
        .count();

    #[cfg(feature = "rayon")]
    let i = tiles[..len - 1].into_par_iter();

    #[cfg(not(feature = "rayon"))]
    let i = tiles[..len - 1].iter();

    i.enumerate()
        .filter_map(|(i, &(a_x, a_y))| {
            tiles
                .iter()
                .take(len)
                .skip(i + 1)
                .filter_map(move |&(b_x, b_y)| {
                    let (min_x, min_y) = (a_x.min(b_x), a_y.min(b_y));
                    let (max_x, max_y) = (a_x.max(b_x), a_y.max(b_y));

                    let r = tiles
                        .iter()
                        .take(len)
                        .zip(tiles.iter().take(len).cycle().skip(1))
                        .all(|(&(aa_x, aa_y), &(bb_x, bb_y))| {
                            min_x >= aa_x.max(bb_x)
                                || max_x <= aa_x.min(bb_x)
                                || min_y >= aa_y.max(bb_y)
                                || max_y <= aa_y.min(bb_y)
                        });
                    if r {
                        Some((a_x.abs_diff(b_x) + 1) * (a_y.abs_diff(b_y) + 1))
                    } else {
                        None
                    }
                })
                .max()
        })
        .max()
        .expect("no solutions")
}

#[cfg(test)]
mod tests {
    use super::*;

    const INPUT: &str = r"7,1
11,1
11,7
9,7
9,5
2,5
2,3
7,3";

    #[test]
    fn test_part_1() {
        assert_eq!(part_1(INPUT), 50);
    }

    #[test]
    fn test_part_2() {
        assert_eq!(part_2(INPUT), 24);
    }
}
