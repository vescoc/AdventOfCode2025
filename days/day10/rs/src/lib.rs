use std::collections::HashMap;

trait Set {
    fn contains(&self, element: usize) -> bool;
    fn insert(&mut self, element: usize);
}

impl Set for [u128; 9] {
    fn contains(&self, element: usize) -> bool {
        self[element / 128] & 1 << (element % 128) != 0
    }

    fn insert(&mut self, element: usize) {
        self[element / 128] |= 1 << (element % 128);
    }
}

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
            if !visited.contains(new_lights as usize) {
                visited.insert(new_lights as usize);
                queue.push_back((new_lights, count + 1)).unwrap();
            }
        }
    }

    unreachable!()
}

fn dp_levels(memoize: &mut HashMap<Vec<u16>, u64>, levels: Vec<u16>, buttons: &[u16]) -> u64 {
    if let Some(value) = memoize.get(&levels) {
        return *value;
    }

    let result = if levels.iter().all(|level| *level == 0) {
        0
    } else {
        let mut min = u64::MAX;
        'outher: for button in buttons {
            let mut levels = levels.clone();
            for (i, level) in levels.iter_mut().enumerate() {
                if button & (1 << i) != 0 {
                    if *level == 0 {
                        continue 'outher;
                    }
                    *level -= 1;
                }
            }

            min = min.min(dp_levels(memoize, levels, buttons));
        }
        min.saturating_add(1)
    };

    memoize.insert(levels, result);

    result
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
pub fn part_2(data: &str) -> u64 {
    data.lines()
        .map(|line| {
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

            // TODO: not working!
            let mut memoize = HashMap::with_capacity(1024);
            let r = dp_levels(&mut memoize, levels.unwrap(), &buttons);
            r
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
