#![no_std]

type Devices<'a> = heapless::index_map::FnvIndexMap<&'a str, &'a str, 1024>;
type Memoize<K, V> = heapless::index_map::FnvIndexMap<K, V, 2048>;

fn dfs<'a>(
    memoize: &mut Memoize<&'a str, u64>,
    devices: &Devices<'a>,
    from: &'a str,
    to: &'a str,
) -> u64 {
    if let Some(value) = memoize.get(from) {
        return *value;
    }

    let result = {
        if from == to {
            1
        } else {
            devices.get(from).map_or(0, |tos| {
                tos.split_whitespace()
                    .map(|from| dfs(memoize, devices, from, to))
                    .sum()
            })
        }
    };

    memoize.insert(from, result).unwrap();

    result
}

fn dfs_with<'a>(
    memoize: &mut Memoize<(&'a str, bool, bool), u64>,
    devices: &Devices<'a>,
    from: &'a str,
    dac_seen: bool,
    fft_seen: bool,
    to: &'a str,
) -> u64 {
    if let Some(value) = memoize.get(&(from, dac_seen, fft_seen)) {
        return *value;
    }

    let result = {
        if from == to {
            u64::from(dac_seen && fft_seen)
        } else {
            devices.get(from).map_or(0, |tos| {
                tos.split_whitespace()
                    .map(|from| {
                        dfs_with(
                            memoize,
                            devices,
                            from,
                            dac_seen || from == "dac",
                            fft_seen || from == "fft",
                            to,
                        )
                    })
                    .sum()
            })
        }
    };

    memoize.insert((from, dac_seen, fft_seen), result).unwrap();

    result
}

/// # Panics
#[must_use]
fn solve<'a>(data: &'a str, f: impl FnOnce(&Devices<'a>) -> u64) -> u64 {
    let mut devices = Devices::new();
    for line in data.lines() {
        let (from, tos) = line.split_once(": ").expect("Invalid line");
        devices.insert(from, tos).unwrap();
    }

    f(&devices)
}

/// # Panics
#[cfg_attr(target_os = "none", inline(never))]
#[must_use]
pub fn part_1(data: &str) -> u64 {
    solve(data, |devices| {
        dfs(&mut Memoize::new(), devices, "you", "out")
    })
}

/// # Panics
#[cfg_attr(target_os = "none", inline(never))]
#[must_use]
pub fn part_2(data: &str) -> u64 {
    solve(data, |devices| {
        dfs_with(&mut Memoize::new(), devices, "svr", false, false, "out")
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const INPUT1: &str = r"aaa: you hhh
you: bbb ccc
bbb: ddd eee
ccc: ddd eee fff
ddd: ggg
eee: out
fff: out
ggg: out
hhh: ccc fff iii
iii: out";

    const INPUT2: &str = r"svr: aaa bbb
aaa: fft
fft: ccc
bbb: tty
tty: ccc
ccc: ddd eee
ddd: hub
hub: fff
eee: dac
dac: fff
fff: ggg hhh
ggg: out
hhh: out";

    #[test]
    fn test_part_1() {
        assert_eq!(part_1(INPUT1), 5);
    }

    #[test]
    fn test_part_2() {
        assert_eq!(part_2(INPUT2), 2);
    }
}
