#![no_std]

/// # Panics
#[must_use]
pub fn part_1(data: &str) -> u64 {
    let table = data.as_bytes();
    let columns = table
        .iter()
        .position(|&tile| tile == b'\n')
        .expect("invalid input");
    let rows = (table.len() - 1) / columns;

    let parse = |slice: &[u8]| {
        slice.iter().fold(0, |num, &digit| match digit {
            c if c.is_ascii_digit() => num * 10 + u64::from(c - b'0'),
            b' ' => num,
            c => unreachable!("'{slice:?}': '{}'", c as char),
        })
    };

    let ops = &table[(rows - 1) * (columns + 1)..];

    let mut total = 0;
    let mut column = 0;
    while column < columns {
        let op = ops[column];
        let end_column = ops[column + 1..]
            .iter()
            .position(|&tile| tile == b'*' || tile == b'+')
            .map_or(columns, |position| position + column);

        total += match op {
            b'+' => (0..rows - 1)
                .map(|row| {
                    let start = row * (columns + 1);
                    parse(&table[start + column..start + end_column])
                })
                .sum::<u64>(),
            b'*' => (0..rows - 1)
                .map(|row| {
                    let start = row * (columns + 1);
                    parse(&table[start + column..start + end_column])
                })
                .product::<u64>(),
            c => unreachable!("{column}: '{}'", c as char),
        };

        column = end_column + 1;
    }

    total
}

/// # Panics
#[must_use]
pub fn part_2(data: &str) -> u64 {
    let table = data.as_bytes();
    let columns = table
        .iter()
        .position(|&tile| tile == b'\n')
        .expect("invalid input");
    let rows = (table.len() - 1) / columns;

    let parse = |column| {
        (0..rows - 1).fold(0, |num, row| match table[row * (columns + 1) + column] {
            c if c.is_ascii_digit() => num * 10 + u64::from(c - b'0'),
            b' ' => num,
            c => unreachable!("({row}, {column}): '{}'", c as char),
        })
    };

    let ops = &table[(rows - 1) * (columns + 1)..];

    let mut total = 0;
    let mut column = 0;
    while column < columns {
        let op = ops[column];
        let end_column = ops[column + 1..]
            .iter()
            .position(|&tile| tile == b'*' || tile == b'+')
            .map_or(columns, |position| position + column);

        total += match op {
            b'+' => (column..end_column).map(parse).sum::<u64>(),
            b'*' => (column..end_column).map(parse).product::<u64>(),
            c => unreachable!("{column}: '{}'", c as char),
        };

        column = end_column + 1;
    }

    total
}

#[cfg(test)]
mod tests {
    use super::*;

    const INPUT: &str = r"123 328  51 64 
 45 64  387 23 
  6 98  215 314
*   +   *   +  
";

    #[test]
    #[allow(clippy::unreadable_literal)]
    fn test_part_1() {
        assert_eq!(part_1(INPUT), 4277556);
    }

    #[test]
    #[allow(clippy::unreadable_literal)]
    fn test_part_2() {
        assert_eq!(part_2(INPUT), 3263827);
    }
}
