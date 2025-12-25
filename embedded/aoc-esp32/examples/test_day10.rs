#![feature(impl_trait_in_assoc_type, core_float_math)]
#![no_std]
#![no_main]
#![allow(clippy::used_underscore_binding)]

type String = heapless::String<255>;

use core::fmt::Write;

use esp_backtrace as _;

use esp_hal::clock::CpuClock;
use esp_hal::timer::timg::TimerGroup;

use embassy_time::Instant;

esp_bootloader_esp_idf::esp_app_desc!();

#[esp_rtos::main]
async fn main(_spawner: embassy_executor::Spawner) {
    let peripherals = esp_hal::init(esp_hal::Config::default().with_cpu_clock(CpuClock::max()));

    cfg_if::cfg_if! {
        if #[cfg(feature = "esp32s2")] {
            let logger = esp_hal::uart::Uart::new(peripherals.UART0, esp_hal::uart::Config::default())
                .unwrap()
                .with_rx(peripherals.GPIO39)
                .with_tx(peripherals.GPIO40);
            let led_error = esp_hal::gpio::Output::new(
                peripherals.GPIO15,
                esp_hal::gpio::Level::Low,
                esp_hal::gpio::OutputConfig::default(),
            );

            aoc_esp32::serial_logger::SerialLogger::init(logger, led_error).ok();
        } else {
            esp_println::logger::init_logger_from_env();
        }
    }

    log::info!("ESP32 EMBASSY TEST DAY10");

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    cfg_if::cfg_if! {
        if #[cfg(any(feature = "esp32", feature = "esp32s2", feature = "esp32s3"))] {
            esp_rtos::start(timg0.timer0);
        } else if #[cfg(any(feature = "esp32c3", feature = "esp32c6"))] {
            let sw_int = esp_hal::interrupt::software::SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);
            esp_rtos::start(timg0.timer0, sw_int.software_interrupt0);
        } else {
            compile_error!("Mush be specified a esp32: esp32, esp32s2, esp32s3, esp32c3, esp32c6");
        }
    }

    log::info!("start tests");

	test("test_bad_13_step_1_f32" ,test_bad_13_step_1_f32);
	test("test_bad_13_step_1_f64" ,test_bad_13_step_1_f64);
	
    test("test_part_2 (f32)", test_part_2_f32);
    test("test_part_2 (f64)", test_part_2_f64);

    test("test 11", test_11);
    test("test 12", test_12);
    test("test 13", test_13);
    test("test 14", test_14);

    #[cfg(not(any(feature = "esp32", feature = "esp32s3")))]
    test("test_bad", test_bad_13);

    #[cfg(not(any(feature = "esp32", feature = "esp32s3")))]
    test("test_bad", test_bad_143);
	
    #[cfg(not(any(feature = "esp32", feature = "esp32s3")))]
	let ignored = [];

    #[cfg(any(feature = "esp32", feature = "esp32s3"))]
	// 13: cannot find a valid solution on f32, ok on f64
	// 143: cannot find a valid solution on f32, ok on f64
	let ignored = [];

    for (n, line) in include_str!("input.txt")
        .lines()
        .enumerate()
        .filter(|(n, _)| !ignored.contains(n))
    {
        if line.is_empty() {
            continue;
        }

        let mut test_name = String::new();
        write!(&mut test_name, "test (f32) [{n}]").unwrap();

        test(&test_name, |msg| test_single_f32(msg, line));
    }

    for (n, line) in include_str!("input.txt")
        .lines()
        .enumerate()
    {
        if line.is_empty() {
            continue;
        }

        let mut test_name = String::new();
        write!(&mut test_name, "test (f64) [{n}]").unwrap();

        test(&test_name, |msg| test_single_f64(msg, line));
    }

    log::info!("done tests");
}

#[allow(clippy::unit_arg)]
fn test(name: &str, f: impl FnOnce(&mut String) -> bool) {
    let mut msg = String::new();

    let now = Instant::now();
    let r = f(&mut msg);
    let elapsed = now.elapsed().as_micros();

    if r {
        log::info!("test: {name} duration: {elapsed}µs, passed");
    } else {
        log::warn!("test: {name} duration: {elapsed}µs, failed: {msg}");
    }
}

fn test_part_2_f32(msg: &mut String) -> bool {
    const INPUT: &str = r"[.##.] (3) (1,3) (2) (2,3) (0,2) (0,1) {3,5,4,7}
[...#.] (0,2,3,4) (2,3) (0,4) (0,1,2) (1,2,3,4) {7,5,12,7,2}
[.###.#] (0,1,2,3,4) (0,3,4) (0,1,2,4,5) (1,2) {10,11,11,5,10,5}";
    let target = 33;

    let result = day10::part_2::<f32>(INPUT);
    if result == target {
        return true;
    }

    write!(msg, "Invalid result: expected {target}, got {result}").unwrap();

    false
}

fn test_part_2_f64(msg: &mut String) -> bool {
    const INPUT: &str = r"[.##.] (3) (1,3) (2) (2,3) (0,2) (0,1) {3,5,4,7}
[...#.] (0,2,3,4) (2,3) (0,4) (0,1,2) (1,2,3,4) {7,5,12,7,2}
[.###.#] (0,1,2,3,4) (0,3,4) (0,1,2,4,5) (1,2) {10,11,11,5,10,5}";
    let target = 33;

    let result = day10::part_2::<f64>(INPUT);
    if result == target {
        return true;
    }

    write!(msg, "Invalid result: expected {target}, got {result}").unwrap();

    false
}

fn test_11(msg: &mut String) -> bool {
    const INPUT: &str = "[##....##] (1,2,3) (2,3,4,5,6) (5) (0,5,6,7) (0,1,4,5,6) (1,3,4) (0,1,3,4,6,7) (1,6,7) (0,1,3,5) (0,3,4,5,7) {50,69,17,60,52,62,66,42}";
    let target = 106;

    let result = day10::part_2::<f32>(INPUT);
    if result == target {
        return true;
    }

    write!(msg, "Invalid result: expected {target}, got {result}").unwrap();

    false
}

fn test_12(msg: &mut String) -> bool {
    const INPUT: &str = "[.#.##..] (0,2,3,5) (0,1,5) (0,1) (0,2,3) (0,1,2,3,4,5) (0,3,4,5,6) (1,2,5) (1,2,6) {66,230,232,50,32,55,203}";
    let target = 259;

    let result = day10::part_2::<f32>(INPUT);
    if result == target {
        return true;
    }

    write!(msg, "Invalid result: expected {target}, got {result}").unwrap();

    false
}

fn test_13(msg: &mut String) -> bool {
    const INPUT: &str = "[....#..#.] (2,3,4,7) (1,2,3,5,6,7,8) (2,4,7,8) (4,7) (0,2,3,5,6,8) (0,1,2,3,6,7,8) (3,8) (0,5,6,8) (2,5) {16,13,49,34,35,39,27,48,36}";
    let target = 80;

    let result = day10::part_2::<f32>(INPUT);
    if result == target {
        return true;
    }

    write!(msg, "Invalid result: expected {target}, got {result}").unwrap();

    false
}

fn test_14(msg: &mut String) -> bool {
    const INPUT: &str = "[...###.#.] (0,6,7) (0,2) (1,3,7,8) (0,2,5,7,8) (3,4,6,7) (1,4) (0,6) {17,2,12,11,9,6,14,20,8}";
    let target = 28;

    let result = day10::part_2::<f32>(INPUT);
    if result == target {
        return true;
    }

    write!(msg, "Invalid result: expected {target}, got {result}").unwrap();

    false
}

#[cfg(not(any(feature = "esp32", feature = "esp32s3")))]
fn test_bad_13(msg: &mut String) -> bool {
    const INPUT: &str = "[...#..#] (0,1,2,3,4) (2,4,5,6) (0,2,3) (0,3,4,5) (0,1,2,5,6) (1,3,4,5) (0) (0,1,2,6) (0,1,3,5,6) {60,37,61,40,35,28,31}";
    let target = 71;

    let result = day10::part_2::<f32>(INPUT);
    if result == target {
        return true;
    }

    write!(msg, "Invalid result: expected {target}, got {result}").unwrap();

    false
}

#[cfg(not(any(feature = "esp32", feature = "esp32s3")))]
fn test_bad_143(msg: &mut String) -> bool {
    const INPUT: &str = "[..###.##.#] (1,2,7) (2,4,5,6,7,8,9) (0,2) (2,3,4,6,8,9) (0,1,2,5,6,7) (5,7) (0,1,2,3,5,8,9) (0,3,7) (2,3,4,7,8,9) (0,1,3,4,5,6,7,8,9) (1,2,3,4,5,6,7,9) {55,66,77,73,50,64,48,74,61,69}";
    let target = 103;

    let result = day10::part_2::<f32>(INPUT);
    if result == target {
        return true;
    }

    write!(msg, "Invalid result: expected {target}, got {result}").unwrap();

    false
}

fn test_single_f32(msg: &mut String, line: &str) -> bool {
    let result = day10::part_2::<f32>(line);
    write!(msg, "Got result: {result} for {line}").unwrap();

    false
}

fn test_single_f64(msg: &mut String, line: &str) -> bool {
    let result = day10::part_2::<f64>(line);
    write!(msg, "Got result: {result} for {line}").unwrap();

    false
}

fn test_bad_13_step_1_f32(msg: &mut String) -> bool {
	let mut bases = [None; 9];
	
    #[rustfmt::skip]
	let mut data = [1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 0.0,
					1.0, 0.0, 1.0, 1.0, 1.0, 0.0, 1.0, 1.0, 1.0, 60.0,
					1.0, 0.0, 0.0, 0.0, 1.0, 1.0, 0.0, 1.0, 1.0, 37.0,
					1.0, 1.0, 1.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 61.0,
					1.0, 0.0, 1.0, 1.0, 0.0, 1.0, 0.0, 0.0, 1.0, 40.0,
					1.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 0.0, 0.0, 35.0,
					0.0, 1.0, 0.0, 1.0, 1.0, 1.0, 0.0, 0.0, 1.0, 28.0,
					0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 1.0, 31.0];

	let mut tags = [0, 1, 2, 3, 4, 5, 6];
	
    #[rustfmt::skip]
	let result = simplex::simplex::<9, _, f32>(
		&mut bases,
		&mut data,
		&[
			0..10,
			10..20,
			20..30,
			30..40,
			40..50,
			50..60,
			60..70,
			70..80,
		],
		Some(&mut tags),
	);

	log::info!("{:?}", &data[..10]);
	for tag in tags {
		log::info!("{:?}", &data[10 * (tag + 1)..10 * (tag + 1) + 10]);
	}

	write!(msg, "Got bases: {bases:?} result: {result:?} tags: {tags:?}").unwrap();

	false
}

fn test_bad_13_step_1_f64(msg: &mut String) -> bool {
	let mut bases = [None; 9];
	
    #[rustfmt::skip]
	let mut data = [1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 0.0,
					1.0, 0.0, 1.0, 1.0, 1.0, 0.0, 1.0, 1.0, 1.0, 60.0,
					1.0, 0.0, 0.0, 0.0, 1.0, 1.0, 0.0, 1.0, 1.0, 37.0,
					1.0, 1.0, 1.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 61.0,
					1.0, 0.0, 1.0, 1.0, 0.0, 1.0, 0.0, 0.0, 1.0, 40.0,
					1.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 0.0, 0.0, 35.0,
					0.0, 1.0, 0.0, 1.0, 1.0, 1.0, 0.0, 0.0, 1.0, 28.0,
					0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 1.0, 31.0];

	let mut tags = [0, 1, 2, 3, 4, 5, 6];

    #[rustfmt::skip]
	let result = simplex::simplex::<9, _, f64>(
		&mut bases,
		&mut data,
		&[
			0..10,
			10..20,
			20..30,
			30..40,
			40..50,
			50..60,
			60..70,
			70..80,
		],
		Some(&mut tags),
	);
	
	log::info!("{:?}", &data[..10]);
	for tag in tags {
		log::info!("{:?}", &data[10 * (tag + 1)..10 * (tag + 1) + 10]);
	}
	
	write!(msg, "Got bases: {bases:?} result: {result:?} tags: {tags:?}").unwrap();

	false
}
