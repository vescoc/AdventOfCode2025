use std::time::Instant;

use day09 as day;

fn main() {
    #[cfg(feature = "input")]
    let input = include_str!("../../input");

    #[cfg(not(feature = "input"))]
    let input = &std::io::read_to_string(std::io::stdin()).expect("cannot read input");

    let now = Instant::now();

    println!("part 1: {}", day::part_1(input));
    println!("part 2: {}", day::part_2(input));

    let elapsed = now.elapsed();
    println!(
        "elapsed: {}ms ({}µs, {}ns)",
        elapsed.as_millis(),
        elapsed.as_micros(),
        elapsed.as_nanos()
    );
}
