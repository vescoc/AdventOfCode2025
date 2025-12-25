#![feature(impl_trait_in_assoc_type, core_float_math)]
#![no_std]
#![no_main]
#![allow(clippy::used_underscore_binding)]

use core::hint;

use esp_backtrace as _;

use esp_hal::clock::CpuClock;
use esp_hal::timer::timg::TimerGroup;

use embassy_time::Instant;

esp_bootloader_esp_idf::esp_app_desc!();

const NUM: usize = 1_000_000;

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

    log::info!("ESP32 EMBASSY TEST FP PERF");

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

    log::info!("start benches");

    bench("mul", mul);
    bench("div", div);
    bench("add", add);
    bench("abs", abs);
    bench("round", round);
    bench("ceil", ceil);
    bench("floor", floor);
    bench("to i64", to_i64);

    log::info!("done benches");
}

#[allow(clippy::unit_arg)]
fn bench(name: &str, f: impl FnOnce()) {
    let now = Instant::now();
    hint::black_box(f());
    let elapsed = now.elapsed().as_micros();

    log::info!(
        "bench: {name} duration: {elapsed}µs rate: {}ns",
        (elapsed * 1_000) / NUM as u64
    );
}

#[allow(clippy::cast_precision_loss)]
fn mul() {
    let f = 7.31f32;
    for i in 0..NUM {
        let _ = hint::black_box(hint::black_box(f) * hint::black_box(i as f32));
    }
}

#[allow(clippy::cast_precision_loss)]
fn div() {
    let f = 7.31f32;
    for i in 1..=NUM {
        let _ = hint::black_box(hint::black_box(f) / hint::black_box(i as f32));
    }
}

#[allow(clippy::cast_precision_loss)]
fn add() {
    let f = 7.31f32;
    for i in 0..NUM {
        let _ = hint::black_box(hint::black_box(f) + hint::black_box(i as f32));
    }
}

fn abs() {
    for f in [7.31f32, -7.31].into_iter().cycle().take(NUM) {
        let _ = hint::black_box(hint::black_box(f).abs());
    }
}

fn round() {
    for f in [7.31f32, -7.31].into_iter().cycle().take(NUM) {
        let _ = hint::black_box(core::f32::math::round(hint::black_box(f)));
    }
}

fn ceil() {
    for f in [7.31f32, -7.31].into_iter().cycle().take(NUM) {
        let _ = hint::black_box(core::f32::math::ceil(hint::black_box(f)));
    }
}

fn floor() {
    for f in [7.31f32, -7.31].into_iter().cycle().take(NUM) {
        let _ = hint::black_box(core::f32::math::floor(hint::black_box(f)));
    }
}

#[allow(clippy::cast_possible_truncation)]
fn to_i64() {
    for f in [7.31f32, -7.31].into_iter().cycle().take(NUM) {
        let _ = hint::black_box(hint::black_box(f) as i64);
    }
}
