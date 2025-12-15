#![no_std]

#[cfg(feature = "rayon")]
use rayon::prelude::*;

use numset::Set;
use simplex::integer_simplex;

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

/// # Panics
#[must_use]
pub fn part_1(data: &str) -> u64 {
    #[cfg(feature = "rayon")]
    let i = data.par_lines();

    #[cfg(not(feature = "rayon"))]
    let i = data.lines();

    i.map(|line| {
        let mut lights = 0u16;

        let mut buttons_len = 0;
        let mut buttons = [0u16; 16];
        for part in line.split_whitespace() {
            if part.starts_with('[') {
                lights = part.as_bytes()[1..part.len() - 1]
                    .iter()
                    .rev()
                    .fold(0u16, |acc, light| acc << 1 | u16::from(*light == b'#'));
            } else if part.starts_with('(') {
                buttons[buttons_len] = part.as_bytes()[1..part.len() - 1]
                    .split(|tile| *tile == b',')
                    .map(|num| {
                        num.iter()
                            .fold(0, |acc, digit| acc * 10 + u16::from(digit - b'0'))
                    })
                    .fold(0u16, |acc, num| acc | 1 << num);
                buttons_len += 1;
            }
        }

        bfs_lights(lights, &buttons[..buttons_len])
    })
    .sum()
}

/// # Panics
#[must_use]
pub fn part_2(data: &str) -> u64 {
    type F = f64;

    #[cfg(feature = "rayon")]
    let i = data.par_lines();

    #[cfg(not(feature = "rayon"))]
    let i = data.lines();

    i.map(|line| {
        let mut buttons_len = 0;
        let mut buttons = [0u16; 16];

        let mut bj_len = 0;
        let mut bj = [0.0; 16];

        for part in line.split_whitespace() {
            if part.starts_with('{') {
                bj_len = bj
                    .iter_mut()
                    .zip(
                        part.as_bytes()[1..part.len() - 1]
                            .split(|tile| *tile == b',')
                            .map(|num| {
                                num.iter()
                                    .fold(0.0, |acc, digit| acc * 10.0 + F::from(digit - b'0'))
                            }),
                    )
                    .map(|(b, v)| {
                        *b = v;
                    })
                    .count();
            } else if part.starts_with('(') {
                buttons[buttons_len] = part.as_bytes()[1..part.len() - 1]
                    .split(|tile| *tile == b',')
                    .map(|num| {
                        num.iter()
                            .fold(0, |acc, digit| acc * 10 + u16::from(digit - b'0'))
                    })
                    .fold(0u16, |acc, num| acc | 1 << num);
                buttons_len += 1;
            }
        }

        let mut zi = [0.0; 16];
        let zi_len = zi
            .iter_mut()
            .zip(0..buttons_len)
            .map(|(z, _)| {
                *z = 1.0;
            })
            .count();

        let mut aij = [0.0; 16 * 16];
        let aij_len = aij
            .iter_mut()
            .zip(bj.iter().take(bj_len).enumerate().flat_map(|(i, _)| {
                buttons
                    .iter()
                    .take(buttons_len)
                    .map(move |button| F::from(button.is_set(i)))
            }))
            .map(|(a, v)| {
                *a = v;
            })
            .count();

        // println!("{line}");
        // println!("let zi = {:?};", &zi[..zi_len]);
        // println!("let aij = {:?};", &aij[..aij_len]);
        // println!("let bj = {:?};", &bj[..bj_len]);
        integer_simplex::<32, { 32 * 32 }, _>(&zi[..zi_len], &aij[..aij_len], &bj[..bj_len])
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
