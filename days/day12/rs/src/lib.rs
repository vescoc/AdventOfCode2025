#![no_std]

/// # Panics
#[must_use]
pub fn part_1(data: &str) -> u64 {
    let mut shapes = [0; 6];
    for part in data.split("\n\n") {
        let mut lines = part.lines();
        let first_line = lines.next().unwrap();
        let (left, right) = first_line.split_once(':').expect("Invalid line");
        if right.is_empty() {
            let Some(d) = left.chars().next() else { unreachable!("id not found") };
            let id = d.to_digit(10).expect("Invalid id") as usize;
            let area = lines
                .map(|line| line.chars().filter(|tile| *tile == '#').count())
                .sum();
            shapes[id] = area;
        } else {
            return core::iter::once((left, right))
                .chain(lines.map(|line| line.split_once(": ").expect("Invalid line")))
                .map(|(area, list)| {
                    let (w, l) = area.split_once('x').expect("Invalid region");

                    let area = w.parse::<usize>().expect("Invalid width")
                        * l.parse::<usize>().expect("Invalid length");
                    let sum = list
                        .split_whitespace()
                        .enumerate()
                        .map(|(id, num)| {
                            shapes[id] * num.parse::<usize>().expect("Invalid shape count")
                        })
                        .sum();

                    u64::from(area >= sum)
                })
                .sum();
        }
    }

    unreachable!()
}
