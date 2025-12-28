#![feature(impl_trait_in_assoc_type)]
#![no_std]
#![no_main]

use defmt_rtt as _;

unsafe extern "C" {
    static _stack_end: u32;
    static _stack_start: u32;
}

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
    #[cfg(feature = "temp")]
    ADC_IRQ_FIFO => embassy_rp::adc::InterruptHandler;
});

#[embassy_executor::main]
async fn main(spawner: embassy_executor::Spawner) {
    defmt::info!("RP-PICO EMBASSY USB AoC 2025");

    let stack_low = &raw const _stack_end as u32;
    let stack_high = &raw const _stack_start as u32;

    defmt::info!(
        "stack: [{} - {}]: {} bytes",
        stack_low,
        stack_high,
        stack_high - stack_low
    );

    #[cfg(not(feature = "overclock"))]
    let p = embassy_rp::init(embassy_rp::config::Config::default());
    #[cfg(feature = "overclock")]
    let p = embassy_rp::init(embassy_rp::config::Config::new(
        embassy_rp::clocks::ClockConfig::system_freq(200_000_000).unwrap(),
    ));

    defmt::info!(
        "sys freq: {} adc freq: {} peri freq: {} ref freq: {}, pll sys freq: {} pll usb freq: {} rosc freq: {} xosc freq: {} core voltage: {}",
        embassy_rp::clocks::clk_sys_freq(),
        embassy_rp::clocks::clk_adc_freq(),
        embassy_rp::clocks::clk_peri_freq(),
        embassy_rp::clocks::clk_ref_freq(),
        embassy_rp::clocks::pll_sys_freq(),
        embassy_rp::clocks::pll_usb_freq(),
        embassy_rp::clocks::rosc_freq(),
        embassy_rp::clocks::xosc_freq(),
        embassy_rp::clocks::core_voltage(),
    );

    #[cfg(feature = "temp")]
    {
        let adc = embassy_rp::adc::Adc::new(p.ADC, Irqs, embassy_rp::adc::Config::default());
        let ts = embassy_rp::adc::Channel::new_temp_sensor(p.ADC_TEMP_SENSOR);
        spawner.spawn(ts_task(adc, ts)).unwrap();
    }

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

#[cfg(feature = "temp")]
#[embassy_executor::task]
async fn ts_task(
    mut adc: embassy_rp::adc::Adc<'static, embassy_rp::adc::Async>,
    mut ts: embassy_rp::adc::Channel<'static>,
) -> ! {
    loop {
        if let Ok(temp) = adc.read(&mut ts).await {
            defmt::info!("Temp: {}°", convert_to_celsius(temp));
        }
        embassy_time::Timer::after_secs(5).await;
    }
}

#[cfg(feature = "temp")]
fn convert_to_celsius(raw_temp: u16) -> f32 {
    // According to chapter 4.9.5. Temperature Sensor in RP2040 datasheet
    let temp = 27.0 - (raw_temp as f32 * 3.3 / 4096.0 - 0.706) / 0.001721;
    let sign = if temp < 0.0 { -1.0 } else { 1.0 };
    let rounded_temp_x10: i16 = ((temp * 10.0) + 0.5 * sign) as i16;
    (rounded_temp_x10 as f32) / 10.0
}

#[unsafe(link_section = ".bi_entries")]
#[used]
pub static PICOTOOL_ENTRIES: [embassy_rp::binary_info::EntryAddr; 4] = [
    embassy_rp::binary_info::rp_program_name!(c"RP-PICO EMBASSY USB AoC 2025"),
    embassy_rp::binary_info::rp_cargo_version!(),
    embassy_rp::binary_info::rp_program_description!(c"RP-PICO EMBASSY USB AoC 2025"),
    embassy_rp::binary_info::rp_program_build_attribute!(),
];
