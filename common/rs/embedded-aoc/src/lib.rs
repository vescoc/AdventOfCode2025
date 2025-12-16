#![no_std]

#[cfg(any(feature = "blocking", feature = "nonblocking"))]
use core::fmt::Write as _;

#[cfg(any(feature = "blocking", feature = "nonblocking"))]
use core::fmt;

use fugit::{Duration, Instant};

use heapless::String as HLString;

#[cfg(all(feature = "defmt", any(feature = "blocking", feature = "nonblocking")))]
use defmt::{info, trace, warn};

#[cfg(all(feature = "log", any(feature = "blocking", feature = "nonblocking")))]
use log::{info, trace, warn};

#[cfg(feature = "blocking")]
mod blocking;
#[cfg(feature = "blocking")]
pub use blocking::run;

#[cfg(feature = "nonblocking")]
mod nonblocking;
#[cfg(feature = "nonblocking")]
pub use nonblocking::run;

#[allow(dead_code)]
type PartResult = HLString<64>;

#[cfg(any(feature = "blocking", feature = "nonblocking"))]
const START_INPUT_TAG: &str = "START INPUT DAY: ";

#[cfg(any(feature = "blocking", feature = "nonblocking"))]
const END_INPUT_TAG: &str = "END INPUT";

#[cfg_attr(feature = "defmt", derive(defmt::Format))]
#[derive(Copy, Clone)]
pub enum Day {
    #[cfg(feature = "day01")]
    Day01,
    #[cfg(feature = "day02")]
    Day02,
    #[cfg(feature = "day03")]
    Day03,
    #[cfg(feature = "day04")]
    Day04,
    #[cfg(feature = "day05")]
    Day05,
    #[cfg(feature = "day06")]
    Day06,
    #[cfg(feature = "day07")]
    Day07,
    #[cfg(feature = "day08")]
    Day08,
    #[cfg(feature = "day09")]
    Day09,
    #[cfg(feature = "day10")]
    Day10,
    #[cfg(feature = "day11")]
    Day11,
    #[cfg(feature = "day12")]
    Day12,
}

#[cfg(any(feature = "blocking", feature = "nonblocking"))]
impl Day {
    fn to_string(result: &mut PartResult, value: impl fmt::Display) -> Result<(), fmt::Error> {
        write!(result, "{value}")
    }

    fn part_1(self, result: &mut PartResult, input: &str) -> Result<(), fmt::Error> {
        match self {
            #[cfg(feature = "day01")]
            Day::Day01 => Self::to_string(result, day01::part_1(input)),
            #[cfg(feature = "day02")]
            Day::Day02 => Self::to_string(result, day02::part_1(input)),
            #[cfg(feature = "day03")]
            Day::Day03 => Self::to_string(result, day03::part_1(input)),
            #[cfg(feature = "day04")]
            Day::Day04 => Self::to_string(result, day04::part_1(input)),
            #[cfg(feature = "day05")]
            Day::Day05 => Self::to_string(result, day05::part_1(input)),
            #[cfg(feature = "day06")]
            Day::Day06 => Self::to_string(result, day06::part_1(input)),
            #[cfg(feature = "day07")]
            Day::Day07 => Self::to_string(result, day07::part_1(input)),
            #[cfg(feature = "day08")]
            Day::Day08 => Self::to_string(result, day08::part_1::<1000>(input)),
            #[cfg(feature = "day09")]
            Day::Day09 => Self::to_string(result, day09::part_1(input)),
            #[cfg(feature = "day10")]
            Day::Day10 => Self::to_string(result, day10::part_1(input)),
            #[cfg(feature = "day11")]
            Day::Day11 => Self::to_string(result, day11::part_1(input)),
            #[cfg(feature = "day12")]
            Day::Day12 => Self::to_string(result, day12::part_1(input)),
        }
    }

    fn part_2(self, result: &mut PartResult, input: &str) -> Result<(), fmt::Error> {
        match self {
            #[cfg(feature = "day01")]
            Day::Day01 => Self::to_string(result, day01::part_2(input)),
            #[cfg(feature = "day02")]
            Day::Day02 => Self::to_string(result, day02::part_2(input)),
            #[cfg(feature = "day03")]
            Day::Day03 => Self::to_string(result, day03::part_2(input)),
            #[cfg(feature = "day04")]
            Day::Day04 => Self::to_string(result, day04::part_2(input)),
            #[cfg(feature = "day05")]
            Day::Day05 => Self::to_string(result, day05::part_2(input)),
            #[cfg(feature = "day06")]
            Day::Day06 => Self::to_string(result, day06::part_2(input)),
            #[cfg(feature = "day07")]
            Day::Day07 => Self::to_string(result, day07::part_2(input)),
            #[cfg(feature = "day08")]
            Day::Day08 => Self::to_string(result, day08::part_2(input)),
            #[cfg(feature = "day09")]
            Day::Day09 => Self::to_string(result, day09::part_2(input)),
            #[cfg(feature = "day10")]
            Day::Day10 => Self::to_string(result, day10::part_2::<f32>(input)),
            #[cfg(feature = "day11")]
            Day::Day11 => Self::to_string(result, day11::part_2(input)),
            #[cfg(feature = "day12")]
            Day::Day12 => Self::to_string(result, "NO PART 2"),
        }
    }
}

impl core::str::FromStr for Day {
    type Err = &'static str;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input.chars().take(2).try_fold(0, |acc, digit| match digit {
            '0'..='9' => Some(acc * 10 + digit as u32 - '0' as u32),
            _ => None,
        }) {
            #[cfg(feature = "day01")]
            Some(1) => Ok(Day::Day01),
            #[cfg(feature = "day02")]
            Some(2) => Ok(Day::Day02),
            #[cfg(feature = "day03")]
            Some(3) => Ok(Day::Day03),
            #[cfg(feature = "day04")]
            Some(4) => Ok(Day::Day04),
            #[cfg(feature = "day05")]
            Some(5) => Ok(Day::Day05),
            #[cfg(feature = "day06")]
            Some(6) => Ok(Day::Day06),
            #[cfg(feature = "day07")]
            Some(7) => Ok(Day::Day07),
            #[cfg(feature = "day08")]
            Some(8) => Ok(Day::Day08),
            #[cfg(feature = "day09")]
            Some(9) => Ok(Day::Day09),
            #[cfg(feature = "day10")]
            Some(10) => Ok(Day::Day10),
            #[cfg(feature = "day11")]
            Some(11) => Ok(Day::Day11),
            #[cfg(feature = "day12")]
            Some(12) => Ok(Day::Day12),
            Some(_) => Err("invalid day"),
            None => Err("invalid number"),
        }
    }
}

#[cfg(any(feature = "blocking", feature = "nonblocking"))]
impl fmt::Display for Day {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let day = match self {
            #[cfg(feature = "day01")]
            Day::Day01 => 1,
            #[cfg(feature = "day02")]
            Day::Day02 => 2,
            #[cfg(feature = "day03")]
            Day::Day03 => 3,
            #[cfg(feature = "day04")]
            Day::Day04 => 4,
            #[cfg(feature = "day05")]
            Day::Day05 => 5,
            #[cfg(feature = "day06")]
            Day::Day06 => 6,
            #[cfg(feature = "day07")]
            Day::Day07 => 7,
            #[cfg(feature = "day08")]
            Day::Day08 => 8,
            #[cfg(feature = "day09")]
            Day::Day09 => 9,
            #[cfg(feature = "day10")]
            Day::Day10 => 10,
            #[cfg(feature = "day11")]
            Day::Day11 => 11,
            #[cfg(feature = "day12")]
            Day::Day12 => 12,
        };

        write!(f, "{day:02}")
    }
}

pub trait Timer<T, const NOM: u32, const DENOM: u32> {
    fn now(&self) -> Instant<T, NOM, DENOM>;
}

pub trait Handler<T, const NOM: u32, const DENOM: u32> {
    fn started(&mut self, _day: Day, _timestamp: Instant<T, NOM, DENOM>) {}
    fn ended(
        &mut self,
        _day: Day,
        _elapsed: Duration<T, NOM, DENOM>,
        _part_1: &str,
        _part_2: &str,
    ) {
    }
    fn unsupported_day(&mut self) {}
    fn invalid_input(&mut self) {}
}

#[derive(Default)]
pub struct DummyHandler<T, const NOM: u32, const DENOM: u32> {
    _t: core::marker::PhantomData<T>,
}

impl<T, const NOM: u32, const DENOM: u32> Handler<T, NOM, DENOM> for DummyHandler<T, NOM, DENOM> {}
