#![feature(impl_trait_in_assoc_type)]
#![no_std]
#![no_main]

use defmt_rtt as _;

use embassy_nrf::{
    bind_interrupts,
    gpio::{Level, Output, OutputDrive},
    pac, peripherals, usb,
};

type Instant = fugit::Instant<u64, 1, 1_000_000>;
type Duration = fugit::Duration<u64, 1, 1_000_000>;

bind_interrupts!(struct Irqs {
    USBD => usb::InterruptHandler<peripherals::USBD>;
    CLOCK_POWER => usb::vbus_detect::InterruptHandler;
});

unsafe extern "C" {
    static _stack_end: u32;
    static _stack_start: u32;
}

#[allow(clippy::struct_field_names)]
struct SimpleHandler<'d> {
    led_run: Output<'d>,
    led_invalid: Output<'d>,
    led_unsupported: Output<'d>,
}

impl embedded_aoc::Handler<u64, 1, 1_000_000> for SimpleHandler<'_> {
    fn started(&mut self, _: embedded_aoc::Day, _: Instant) {
        self.led_run.set_low();
        self.led_invalid.set_high();
        self.led_unsupported.set_high();
    }

    fn ended(&mut self, _: embedded_aoc::Day, _: Duration, _: &str, _: &str) {
        self.led_run.set_high();
        self.led_invalid.set_high();
        self.led_unsupported.set_high();
    }

    fn unsupported_day(&mut self) {
        self.led_run.set_high();
        self.led_invalid.set_high();
        self.led_unsupported.set_low();
    }

    fn invalid_input(&mut self) {
        self.led_run.set_high();
        self.led_invalid.set_low();
        self.led_unsupported.set_high();
    }
}

struct Now;

impl embedded_aoc::Timer<u64, 1, 1_000_000> for Now {
    fn now(&self) -> Instant {
        Instant::from_ticks(embassy_time::Instant::now().as_micros())
    }
}

#[panic_handler]
fn core_panic(info: &core::panic::PanicInfo) -> ! {
    defmt::error!("PANIC: {}", info);

    loop {
        cortex_m::asm::bkpt();
    }
}

#[embassy_executor::main]
async fn main(spawner: embassy_executor::Spawner) {
    defmt::info!("NRF52840-DK EMBASSY USB AoC 2025");

    let stack_low = &raw const _stack_end as u32;
    let stack_high = &raw const _stack_start as u32;

    defmt::info!(
        "stack: [{} - {}]: {} bytes",
        stack_low,
        stack_high,
        stack_high - stack_low
    );

    let peripherals = embassy_nrf::init(embassy_nrf::config::Config::default());

    defmt::info!("Enabling ext hfosc...");
    pac::CLOCK.tasks_hfclkstart().write_value(1);
    while pac::CLOCK.events_hfclkstarted().read() != 1 {}
    defmt::info!("... done");

    let driver = embassy_nrf::usb::Driver::new(
        peripherals.USBD,
        Irqs,
        embassy_nrf::usb::vbus_detect::HardwareVbusDetect::new(Irqs),
    );

    let mut config = embassy_usb::Config::new(0x16c0, 0x27dd);
    config.manufacturer = Some("Vescoc Company");
    config.product = Some("Serial port");
    config.serial_number = Some("TEST");
    config.max_power = 100;
    config.max_packet_size_0 = 64;

    static CONFIG_DESCRIPTOR: static_cell::StaticCell<[u8; 256]> = static_cell::StaticCell::new();
    static BOS_DESCRIPTOR: static_cell::StaticCell<[u8; 256]> = static_cell::StaticCell::new();
    static CONTROL_BUF: static_cell::StaticCell<[u8; 64]> = static_cell::StaticCell::new();

    static STATE: static_cell::StaticCell<embassy_usb::class::cdc_acm::State> =
        static_cell::StaticCell::new();

    let mut builder = embassy_usb::Builder::new(
        driver,
        config,
        CONFIG_DESCRIPTOR.init_with(|| [0; 256]),
        BOS_DESCRIPTOR.init_with(|| [0; 256]),
        &mut [],
        CONTROL_BUF.init_with(|| [0; 64]),
    );

    let class = embassy_usb::class::cdc_acm::CdcAcmClass::new(
        &mut builder,
        STATE.init(embassy_usb::class::cdc_acm::State::new()),
        64,
    );

    let usb = builder.build();
    spawner.spawn(usb_task(usb)).unwrap();

    let (tx, rx) = cdcacm_io::split(class);

    let led_run = Output::new(peripherals.P0_13, Level::High, OutputDrive::Standard);
    let led_invalid = Output::new(peripherals.P0_14, Level::High, OutputDrive::Standard);
    let led_unsupported = Output::new(peripherals.P0_15, Level::High, OutputDrive::Standard);

    let handler = SimpleHandler {
        led_run,
        led_invalid,
        led_unsupported,
    };

    embedded_aoc::run((rx, tx), &Now, handler).await;
}

#[embassy_executor::task]
async fn usb_task(
    mut usb: embassy_usb::UsbDevice<
        'static,
        embassy_nrf::usb::Driver<'static, embassy_nrf::usb::vbus_detect::HardwareVbusDetect>,
    >,
) -> ! {
    usb.run().await
}
