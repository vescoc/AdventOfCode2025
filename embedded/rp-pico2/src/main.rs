#![feature(impl_trait_in_assoc_type)]
#![no_std]
#![no_main]

use defmt_rtt as _;

unsafe extern "C" {
    static _stack_end: u32;
    static _stack_start: u32;
}

#[unsafe(link_section = ".start_block")]
#[used]
pub static IMAGE_DEF: embassy_rp::block::ImageDef = embassy_rp::block::ImageDef::secure_exe();

#[panic_handler]
fn core_panic(info: &core::panic::PanicInfo) -> ! {
    defmt::error!("PANIC: {}", info);

    loop {
        cortex_m::asm::wfe();
    }
}

type Instant = fugit::Instant<u64, 1, 1_000_000>;

struct Now;

impl embedded_aoc::Timer<u64, 1, 1_000_000> for Now {
    fn now(&self) -> Instant {
        Instant::from_ticks(embassy_time::Instant::now().as_micros())
    }
}

embassy_rp::bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => embassy_rp::usb::InterruptHandler<embassy_rp::peripherals::USB>;
});

#[embassy_executor::main]
async fn main(spawner: embassy_executor::Spawner) {
    defmt::info!("RP-PICO2 EMBASSY USB AoC 2025");

    let stack_low = &raw const _stack_end as u32;
    let stack_high = &raw const _stack_start as u32;

    defmt::info!(
        "stack: [{} - {}]: {} bytes",
        stack_low,
        stack_high,
        stack_high - stack_low
    );

    let p = embassy_rp::init(embassy_rp::config::Config::default());

    let now = Now;

    let driver = embassy_rp::usb::Driver::new(p.USB, Irqs);

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

    embedded_aoc::run((rx, tx), &now, embedded_aoc::DummyHandler::default()).await;
}

#[embassy_executor::task]
async fn usb_task(
    mut usb: embassy_usb::UsbDevice<
        'static,
        embassy_rp::usb::Driver<'static, embassy_rp::peripherals::USB>,
    >,
) -> ! {
    usb.run().await
}

#[unsafe(link_section = ".bi_entries")]
#[used]
pub static PICOTOOL_ENTRIES: [embassy_rp::binary_info::EntryAddr; 4] = [
    embassy_rp::binary_info::rp_program_name!(c"AoC 2025 EMBASSY USB"),
    embassy_rp::binary_info::rp_cargo_version!(),
    embassy_rp::binary_info::rp_program_description!(c"AoC 2025 EMBASSY USB"),
    embassy_rp::binary_info::rp_program_build_attribute!(),
];
