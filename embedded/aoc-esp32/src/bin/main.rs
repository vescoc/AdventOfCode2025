#![feature(impl_trait_in_assoc_type)]
#![no_std]
#![no_main]
#![allow(clippy::used_underscore_binding)]

use esp_backtrace as _;

use esp_hal::clock::CpuClock;
use esp_hal::timer::timg::TimerGroup;
use esp_hal::uart::{self, Uart};

use log::info;

type Instant = fugit::Instant<u64, 1, 1_000_000>;

struct Now;

impl embedded_aoc::Timer<u64, 1, 1_000_000> for Now {
    fn now(&self) -> Instant {
        Instant::from_ticks(embassy_time::Instant::now().as_micros())
    }
}

unsafe extern "C" {
    static _stack_end: u32;
    static _stack_start: u32;
}

esp_bootloader_esp_idf::esp_app_desc!();

#[esp_rtos::main]
async fn main(_spawner: embassy_executor::Spawner) -> ! {
    let peripherals = esp_hal::init(esp_hal::Config::default().with_cpu_clock(CpuClock::max()));

    cfg_if::cfg_if! {
        if #[cfg(feature = "esp32s2")] {
            let logger = uart::Uart::new(peripherals.UART0, uart::Config::default())
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

    info!("ESP32 EMBASSY UART AoC 2025");

    let stack_low = &raw const _stack_end as u32;
    let stack_high = &raw const _stack_start as u32;

    info!(
        "stack: [{stack_low:0x} - {stack_high:0x}]: 0x{0:0x} [{0}] bytes",
        stack_high - stack_low
    );

    let timg0 = TimerGroup::new(peripherals.TIMG0);
    cfg_if::cfg_if! {
        if #[cfg(feature = "esp32")] {
            esp_rtos::start(timg0.timer0);

            let uart = Uart::new(peripherals.UART2, uart::Config::default())
                .unwrap()
                .with_rx(peripherals.GPIO18)
                .with_tx(peripherals.GPIO19)
                .into_async();
        } else if #[cfg(feature = "esp32s2")] {
            esp_rtos::start(timg0.timer0);

            let uart = Uart::new(peripherals.UART1, uart::Config::default())
                .unwrap()
                .with_rx(peripherals.GPIO18)
                .with_tx(peripherals.GPIO17)
                .into_async();
        } else if #[cfg(feature = "esp32s3")] {
            esp_rtos::start(timg0.timer0);

            let uart = Uart::new(peripherals.UART1, uart::Config::default())
                .unwrap()
                .with_rx(peripherals.GPIO18)
                .with_tx(peripherals.GPIO17)
                .into_async();
        } else if #[cfg(feature = "esp32c3")] {
            let sw_int = esp_hal::interrupt::software::SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);
            esp_rtos::start(timg0.timer0, sw_int.software_interrupt0);

            let uart = Uart::new(peripherals.UART1, uart::Config::default())
                .unwrap()
                .with_rx(peripherals.GPIO18)
                .with_tx(peripherals.GPIO19)
                .into_async();
        } else if #[cfg(feature = "esp32c6")] {
            let sw_int = esp_hal::interrupt::software::SoftwareInterruptControl::new(peripherals.SW_INTERRUPT);
            esp_rtos::start(timg0.timer0, sw_int.software_interrupt0);

            let uart = Uart::new(peripherals.UART1, uart::Config::default())
                .unwrap()
                .with_rx(peripherals.GPIO18)
                .with_tx(peripherals.GPIO19)
                .into_async();
        } else {
            compile_error!("Mush be specified a esp32: esp32, esp32s2, esp32s3, esp32c3, esp32c6");
        }
    }

    let (rx, tx) = uart.split();

    embedded_aoc::run((rx, tx), &Now, embedded_aoc::DummyHandler::default()).await
}
