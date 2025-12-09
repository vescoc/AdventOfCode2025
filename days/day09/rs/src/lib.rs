#![no_std]

type Vec<T> = heapless::Vec<T, 256>;
type HashSet<T> = heapless::index_set::FnvIndexSet<T, 256>;
type HashMap<T, U> = heapless::index_map::FnvIndexMap<T, U, 256>;
type VecDeque<T> = heapless::Deque<T, 1024>;

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

fn init_ss(v: &mut Vec<i64>, tiles: &[(i64, i64)], f: impl Fn(&(i64, i64)) -> i64) {
    let mut set = HashSet::new();
    for e in core::iter::once(i64::MIN)
        .chain(tiles.iter().map(f))
        .chain(core::iter::once(i64::MAX))
    {
        set.insert(e).unwrap();
    }

    for e in &set {
        v.push(*e).unwrap();
    }

    v.sort_unstable();
}

#[allow(clippy::cast_possible_wrap)]
fn init_s2ss(map: &mut HashMap<i64, isize>, v: &Vec<i64>) {
    for (i, e) in v.iter().enumerate() {
        map.insert(*e, i as isize).unwrap();
    }
}

#[allow(clippy::cast_possible_wrap, clippy::cast_sign_loss)]
fn init_map(
    map: &mut [[u128; 2]],
    tiles: &[(i64, i64)],
    x2xs: &HashMap<i64, isize>,
    y2ys: &HashMap<i64, isize>,
) {
    for (p1, p2) in tiles.iter().zip(tiles.iter().cycle().skip(1)) {
        let (mut from_x, mut from_y) = (x2xs[&p1.0], y2ys[&p1.1]);
        let (to_x, to_y) = (x2xs[&p2.0], y2ys[&p2.1]);

        let m = (to_x - from_x).abs() + (to_y - from_y).abs();
        let (dx, dy) = ((to_x - from_x) / m, (to_y - from_y) / m);

        while (from_x, from_y) != (to_x, to_y) {
            let row = &mut map[from_y as usize];
            if from_x < 128 {
                row[0] |= 1 << from_x;
            } else {
                row[1] |= 1 << (from_x - 128);
            }

            from_x += dx;
            from_y += dy;
        }
    }
}

#[allow(clippy::cast_sign_loss)]
fn init_lava(lava: &mut [[u128; 2]], map: &[[u128; 2]], columns: isize, rows: isize) {
    let mut queue = VecDeque::new();
    queue.push_back((0, 0)).unwrap();
    lava[0][0] = 1;
    while let Some((x, y)) = queue.pop_front() {
        for (dx, dy) in [(-1, 0), (1, 0), (0, 1), (0, -1)] {
            let (x, y) = (x + dx, y + dy);
            if (0..columns).contains(&x) && (0..rows).contains(&y) {
                let (row, map) = (&mut lava[y as usize], &map[y as usize]);
                let (mask, e, m) = if x < 128 {
                    (1 << x, &mut row[0], &map[0])
                } else {
                    (1 << (x - 128), &mut row[1], &map[1])
                };
                if *e & mask == 0 && m & mask == 0 {
                    *e |= mask;
                    queue.push_back((x, y)).unwrap();
                }
            }
        }
    }
}

#[inline]
const fn mask(min: isize, max: isize) -> [u128; 2] {
    [
        if min < 128 { !0 << min } else { 0 } & if max < 128 { !0 >> (127 - max) } else { !0 },
        if min < 128 { !0 } else { !0 << (min - 128) }
            & if max < 128 { 0 } else { !0 >> (256 - max - 1) },
    ]
}

/// # Panics
#[must_use]
#[allow(clippy::cast_possible_wrap, clippy::cast_sign_loss)]
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

    let mut xs = Vec::new();
    init_ss(&mut xs, &tiles[..len], |&(e, _)| e);

    let mut ys = Vec::new();
    init_ss(&mut ys, &tiles[..len], |&(_, e)| e);

    let columns = xs.len() as isize;
    let rows = ys.len() as isize;

    let mut x2xs = HashMap::new();
    init_s2ss(&mut x2xs, &xs);

    let mut y2ys = HashMap::new();
    init_s2ss(&mut y2ys, &ys);

    let mut map = [[0u128; 2]; 256];
    init_map(&mut map, &tiles[..len], &x2xs, &y2ys);

    let mut lava = [[0u128; 2]; 256];
    init_lava(&mut lava, &map, columns, rows);

    #[cfg(feature = "rayon")]
    let i = tiles[..len - 1].into_par_iter();

    #[cfg(not(feature = "rayon"))]
    let i = tiles[..len - 1].iter();

    let (x2xs, y2ys) = (&x2xs, &y2ys);
    i.enumerate()
        .filter_map(|(i, (a_x, a_y))| {
            tiles
                .iter()
                .take(len)
                .skip(i + 1)
                .filter_map(move |(b_x, b_y)| {
                    let (min_x, min_y) = (x2xs[a_x.min(b_x)], y2ys[a_y.min(b_y)]);
                    let (max_x, max_y) = (x2xs[a_x.max(b_x)], y2ys[a_y.max(b_y)]);

                    let mask = mask(min_x, max_x);

                    let r = lava[min_y as usize..=max_y as usize]
                        .iter()
                        .all(|lava| lava[0] & mask[0] == 0 && lava[1] & mask[1] == 0);
                    if r {
                        Some((a_x.abs_diff(*b_x) + 1) * (a_y.abs_diff(*b_y) + 1))
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

    fn slow_mask(min: isize, max: isize) -> [u128; 2] {
        let mut r = [0u128; 2];
        for i in min..=max {
            if i < 128 {
                r[0] |= 1 << i;
            } else {
                r[1] |= 1 << (i - 128);
            }
        }
        r
    }

    #[test]
    fn test_mask() {
        assert_eq!(mask(10, 100), slow_mask(10, 100));
        assert_eq!(mask(10, 128), slow_mask(10, 128));
        assert_eq!(mask(10, 208), slow_mask(10, 208));
    }
}
