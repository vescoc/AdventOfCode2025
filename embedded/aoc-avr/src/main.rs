#![no_std]
#![no_main]

use arduino_hal::prelude::*;

struct Now;

impl embedded_aoc::Timer<u64, 1, 1_000_000> for Now {
    fn now(&self) -> aoc_avr::Instant {
        aoc_avr::CLOCK.now()
    }
}

#[panic_handler]
fn core_panic(_: &core::panic::PanicInfo) -> ! {
    avr_device::interrupt::disable();

    let dp = unsafe { arduino_hal::Peripherals::steal() };
    let pins = arduino_hal::pins!(dp);

    let mut led = pins.d13.into_output();
    loop {
        led.toggle();
        arduino_hal::delay_ms(100);
    }
}

#[arduino_hal::entry]
fn main() -> ! {
    let peripherals = arduino_hal::Peripherals::take().unwrap();

    let pins = arduino_hal::pins!(peripherals);

    let logger = arduino_hal::usart::Usart::new(
        peripherals.USART0,
        pins.d0,
        pins.d1.into_output(),
        57600.into_baudrate(),
    );
    let (_, mut logger_writer) = logger.split();
    let mut log = aoc_avr::Writer(&mut logger_writer);

    core::fmt::write(&mut log, format_args!("AVR USART AoC 2025\r\n")).ok();

    let serial = arduino_hal::usart::Usart::new(
        peripherals.USART1,
        pins.d19,
        pins.d18.into_output(),
        9600.into_baudrate(),
    );
    let (reader, mut writer) = serial.split();
	let writer = aoc_avr::Writer(&mut writer);
	
    aoc_avr::CLOCK.start(peripherals.TC0);

    unsafe { avr_device::interrupt::enable() };

    embedded_aoc::run(aoc_avr::Reader(reader), &Now, writer);
}
