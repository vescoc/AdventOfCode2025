#[cfg(feature = "rayon")]
use rayon::prelude::*;

mod set;
mod simplex;

use set::Set;
pub use simplex::*;

fn bfs_lights(lights: u16, buttons: &[u16]) -> u64 {
    let mut visited = [0u128; 9];
    visited[0] = 1;
    let mut queue = heapless::Deque::<_, 512>::new();
    queue.push_back((0, 0)).unwrap();
    while let Some((current, count)) = queue.pop_front() {
        for button in buttons {
            let new_lights = current ^ button;
            if new_lights == lights {
                return count + 1;
            }
            if !visited.is_set(new_lights as usize) {
                visited.set(new_lights as usize);
                queue.push_back((new_lights, count + 1)).unwrap();
            }
        }
    }

    unreachable!()
}

#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::similar_names
)]
fn simplex_i(eqc: &[f64], eqs: &[Vec<f64>]) -> u64 {
    let mut result = f64::MAX;

    let mut stack = vec![];
    stack.push(vec![]);
    while let Some(partitions) = stack.pop() {
        let len = eqc.len();
        let mut current_eqc = eqc
            .iter()
            .copied()
            .chain(core::iter::repeat_n(0.0, partitions.len()))
            .collect::<Vec<_>>();

        let mut current_eqs = eqs
            .iter()
            .map(|v| {
                v.iter()
                    .take(len - 1)
                    .copied()
                    .chain(core::iter::repeat_n(0.0, partitions.len()))
                    .chain(v.last().copied())
                    .collect::<Vec<_>>()
            })
            .chain(
                partitions
                    .iter()
                    .enumerate()
                    .map(|(k, &(x, s, l))| {
                        (0..len - 1)
                            .map(|i| if i == x { 1.0 } else { 0.0 })
                            .chain((0..partitions.len()).map(|i| if k == i { s } else { 0.0 }))
                            .chain(core::iter::once(l))
                            .collect::<Vec<_>>()
                    })
                    .collect::<Vec<_>>(),
            )
            .collect::<Vec<_>>();

        let mut current_eqs = core::iter::once(current_eqc.as_mut_slice())
            .chain(current_eqs.iter_mut().map(Vec::as_mut_slice))
            .collect::<Vec<_>>();

        let mut bases = [None; 16];
        let Some(r) = simplex_eqs(&mut bases, &mut current_eqs, None::<&mut [()]>) else {
            continue;
        };
        if r >= result {
            continue;
        }

        let check_i = bases.iter().enumerate().find_map(|(x, n)| {
            n.and_then(|n| {
                let v = current_eqs[n + 1].last().unwrap();
                if (v.round() - v).abs() > 1e-6 {
                    Some((x, v))
                } else {
                    None
                }
            })
        });

        if let Some((x, v)) = check_i {
            {
                let mut partitions = partitions.clone();
                if let Some((_, s, vv)) = partitions.iter_mut().find(|(xx, _, _)| *xx == x) {
                    *s = 1.0;
                    *vv = v.floor();
                } else {
                    partitions.push((x, 1.0, v.floor()));
                }

                stack.push(partitions);
            }

            {
                let mut partitions = partitions.clone();
                if let Some((_, s, vv)) = partitions.iter_mut().find(|(xx, _, _)| *xx == x) {
                    *s = -1.0;
                    *vv = v.ceil();
                } else {
                    partitions.push((x, -1.0, v.ceil()));
                }

                stack.push(partitions);
            }
        } else {
            result = result.min(r.floor());
        }
    }

    result as u64
}

/// # Panics
#[must_use]
pub fn part_1(data: &str) -> u64 {
    #[cfg(feature = "rayon")]
    let i = data.par_lines();

    #[cfg(not(feature = "rayon"))]
    let i = data.lines();

    i.map(|line| {
        let mut lights = 0u16;
        let mut buttons = vec![];
        for part in line.split_whitespace() {
            if part.starts_with('[') {
                lights = part.as_bytes()[1..part.len() - 1]
                    .iter()
                    .rev()
                    .fold(0u16, |acc, light| acc << 1 | u16::from(*light == b'#'));
            } else if part.starts_with('(') {
                buttons.push(
                    part.as_bytes()[1..part.len() - 1]
                        .split(|tile| *tile == b',')
                        .map(|num| {
                            num.iter()
                                .fold(0, |acc, digit| acc * 10 + u16::from(digit - b'0'))
                        })
                        .fold(0u16, |acc, num| acc | 1 << num),
                );
            }
        }

        bfs_lights(lights, &buttons)
    })
    .sum()
}

/// # Panics
#[must_use]
pub fn part_2(data: &str) -> u64 {
    #[cfg(feature = "rayon")]
    let i = data.par_lines();

    #[cfg(not(feature = "rayon"))]
    let i = data.lines();

    i.map(|line| {
        let mut buttons = vec![];
        let mut levels = None;
        for part in line.split_whitespace() {
            if part.starts_with('{') {
                levels.replace(
                    part.as_bytes()[1..part.len() - 1]
                        .split(|tile| *tile == b',')
                        .map(|num| {
                            num.iter()
                                .fold(0, |acc, digit| acc * 10 + u16::from(digit - b'0'))
                        })
                        .collect::<Vec<_>>(),
                );
            } else if part.starts_with('(') {
                buttons.push(
                    part.as_bytes()[1..part.len() - 1]
                        .split(|tile| *tile == b',')
                        .map(|num| {
                            num.iter()
                                .fold(0, |acc, digit| acc * 10 + u16::from(digit - b'0'))
                        })
                        .fold(0u16, |acc, num| acc | 1 << num),
                );
            }
        }

        assert!(buttons.len() < 16);

        let levels = levels.unwrap();

        let eqc = (0..buttons.len())
            .map(|_| 1.0)
            .chain(core::iter::once(0.0))
            .collect::<Vec<_>>();

        let eqs = levels
            .iter()
            .enumerate()
            .map(|(i, level)| {
                buttons
                    .iter()
                    .map(|button| f64::from(button.is_set(i)))
                    .chain(core::iter::once(f64::from(*level)))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        simplex_i(&eqc, &eqs)
    })
    .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    const INPUT: &str = r"[.##.] (3) (1,3) (2) (2,3) (0,2) (0,1) {3,5,4,7}
[...#.] (0,2,3,4) (2,3) (0,4) (0,1,2) (1,2,3,4) {7,5,12,7,2}
[.###.#] (0,1,2,3,4) (0,3,4) (0,1,2,4,5) (1,2) {10,11,11,5,10,5}";

    #[test]
    fn test_part_1() {
        assert_eq!(part_1(INPUT), 7);
    }

    #[test]
    fn test_part_2() {
        assert_eq!(part_2(INPUT), 33);
    }

    #[test]
    fn test_part_2_bad() {
        assert_eq!(
            part_2(
                "[##....#.] (5) (2,6) (0,3,4,7) (0,1,2,4,6) (0,2,5,7) (0,1,2,3,6) (2,4,5) (0,6) (2,3,4,7) (1,5,7) {59,23,42,27,39,21,40,32}"
            ),
            82,
        );
    }
}
