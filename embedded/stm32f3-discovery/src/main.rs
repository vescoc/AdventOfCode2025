#![feature(impl_trait_in_assoc_type)]

#![no_std]
#![no_main]

use defmt_rtt as _;

use embassy_stm32::{
    bind_interrupts,
    gpio::{Level, Output, Speed},
    peripherals,
    time::mhz,
    usb,
};

type Instant = fugit::Instant<u64, 1, 1_000_000>;
type Duration = fugit::Duration<u64, 1, 1_000_000>;

bind_interrupts!(struct Irqs {
    USB_LP_CAN_RX0 => usb::InterruptHandler<peripherals::USB>;
});

#[panic_handler]
fn core_panic(info: &core::panic::PanicInfo) -> ! {
    defmt::error!("PANIC: {}", info);

    loop {
        cortex_m::asm::bkpt();
    }
}

struct Now;

impl embedded_aoc::Timer<u64, 1, 1_000_000> for Now {
    fn now(&self) -> Instant {
        Instant::from_ticks(embassy_time::Instant::now().as_micros())
    }
}

#[allow(clippy::struct_field_names)]
struct SimpleHandler<'d> {
    led_run: Output<'d>,
    led_invalid: Output<'d>,
    led_unsupported: Output<'d>,
}

impl embedded_aoc::Handler<u64, 1, 1_000_000> for SimpleHandler<'_> {
    fn started(&mut self, _: embedded_aoc::Day, _: Instant) {
        self.led_run.set_high();
        self.led_invalid.set_low();
        self.led_unsupported.set_low();
    }

    fn ended(&mut self, _: embedded_aoc::Day, _: Duration, _: &str, _: &str) {
        self.led_run.set_low();
        self.led_invalid.set_low();
        self.led_unsupported.set_low();
    }

    fn unsupported_day(&mut self) {
        self.led_run.set_low();
        self.led_invalid.set_low();
        self.led_unsupported.set_high();
    }

    fn invalid_input(&mut self) {
        self.led_run.set_low();
        self.led_invalid.set_high();
        self.led_unsupported.set_low();
    }
}

unsafe extern "C" {
    static _stack_end: u32;
    static _stack_start: u32;
}

#[embassy_executor::main]
async fn main(spawner: embassy_executor::Spawner) -> ! {
    defmt::info!("STM32 F3-DISCOVERY EMBASSY USB AoC 2025");

    let stack_low = &raw const _stack_end as u32;
    let stack_high = &raw const _stack_start as u32;

    defmt::info!(
        "stack: [{} - {}]: {} bytes",
        stack_low,
        stack_high,
        stack_high - stack_low
    );

    let peripherals = embassy_stm32::init({
		use embassy_stm32::rcc::*;
		
        let mut config = embassy_stm32::Config::default();
        config.rcc.hse = Some(Hse {
            freq: mhz(8),
            mode: HseMode::Bypass,
        });
        config.rcc.pll = Some(Pll {
            src: PllSource::HSE,
            prediv: PllPreDiv::DIV1,
            mul: PllMul::MUL9,
        });
        config.rcc.sys = Sysclk::PLL1_P;
        config.rcc.ahb_pre = AHBPrescaler::DIV1;
        config.rcc.apb1_pre = APBPrescaler::DIV2;
        config.rcc.apb2_pre = APBPrescaler::DIV1;

        config
    });

    let driver =
        embassy_stm32::usb::Driver::new(peripherals.USB, Irqs, peripherals.PA12, peripherals.PA11);

    let config = {
        let mut config = embassy_usb::Config::new(0x16c0, 0x27dd);
        config.manufacturer = Some("Vescoc Company");
        config.product = Some("Serial port");
        config.serial_number = Some("TEST");
        config.max_power = 100;
        config.max_packet_size_0 = 64;
        config
    };

    let mut builder = {
        static CONFIG_DESCRIPTOR: static_cell::StaticCell<[u8; 256]> =
            static_cell::StaticCell::new();
        static BOS_DESCRIPTOR: static_cell::StaticCell<[u8; 256]> = static_cell::StaticCell::new();
        static CONTROL_BUF: static_cell::StaticCell<[u8; 64]> = static_cell::StaticCell::new();

        embassy_usb::Builder::new(
            driver,
            config,
            CONFIG_DESCRIPTOR.init([0; 256]),
            BOS_DESCRIPTOR.init([0; 256]),
            &mut [],
            CONTROL_BUF.init([0; 64]),
        )
    };

    let class = {
        static STATE: static_cell::StaticCell<embassy_usb::class::cdc_acm::State> =
            static_cell::StaticCell::new();
        let state = STATE.init(embassy_usb::class::cdc_acm::State::new());
        embassy_usb::class::cdc_acm::CdcAcmClass::new(&mut builder, state, 64)
    };

    let usb = builder.build();

    spawner.spawn(usb_task(usb)).unwrap();

    let (tx, rx) = cdcacm_io::split(class);

    let timer = Now;

    let led_run = Output::new(peripherals.PE15, Level::Low, Speed::Low);
    let led_invalid = Output::new(peripherals.PE14, Level::Low, Speed::Low);
    let led_unsupported = Output::new(peripherals.PE13, Level::Low, Speed::Low);

    let handler = SimpleHandler {
        led_run,
        led_invalid,
        led_unsupported,
    };

    embedded_aoc::run((rx, tx), &timer, handler).await
}

#[embassy_executor::task]
async fn usb_task(
    mut usb: embassy_usb::UsbDevice<'static, usb::Driver<'static, peripherals::USB>>,
) -> ! {
    usb.run().await
}
