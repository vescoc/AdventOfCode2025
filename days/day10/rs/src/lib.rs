mod set;
mod simplex;

use set::Set;
pub use simplex::*;

const LOG: bool = false;

fn bfs_lights(lights: u16, buttons: &[u16]) -> u64 {
    let mut visited = [0u128; 9];
    visited[0] = 1;
    let mut queue = heapless::Deque::<_, 512>::new();
    queue.push_back((0, 0)).unwrap();
    while let Some((current, count)) = queue.pop_front() {
        for button in buttons {
            let new_lights = (current ^ button) & button | current & !button;
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
    data.lines()
        .map(|line| {
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
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss, clippy::unused_enumerate_index, clippy::used_underscore_binding)]
pub fn part_2(data: &str) -> u64 {
    data.lines()
        .enumerate()
        .map(|(_i, line)| {
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

            let mut eqc = vec![1.0; buttons.len() + 1];
            eqc[buttons.len()] = 0.0;

            let mut eqs = levels
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

            let mut eqs = core::iter::once(eqc.as_mut_slice())
                .chain(eqs.iter_mut().map(Vec::as_mut_slice))
                .collect::<Vec<_>>();

            let mut bases = 0u16;
            let mut tags = (0..levels.len()).collect::<Vec<_>>();
            let r = simplex_eqs(&mut bases, &mut eqs, &mut tags).ceil();

            println!("{_i}: {r:.1}");

            r as u64
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
}
