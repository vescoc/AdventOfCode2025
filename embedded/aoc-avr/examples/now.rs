#![no_std]
#![no_main]

use arduino_hal::prelude::*;

#[panic_handler]
fn core_panic(_: &core::panic::PanicInfo) -> ! {
    avr_device::interrupt::disable();

    let dp = unsafe { arduino_hal::Peripherals::steal() };
    let pins = arduino_hal::pins!(dp);

    let mut led = pins.d13.into_output();
    loop {
        led.toggle();
        arduino_hal::delay_ms(50);
    }
}

#[arduino_hal::entry]
fn main() -> ! {
    let peripherals = arduino_hal::Peripherals::take().unwrap();

    let pins = arduino_hal::pins!(peripherals);

    let mut led = pins.d13.into_output();

    let serial0 = arduino_hal::usart::Usart::new(
        peripherals.USART0,
        pins.d0,
        pins.d1.into_output(),
        57600.into_baudrate(),
    );
    let (_, mut log) = serial0.split();

    aoc_avr::CLOCK.start(peripherals.TC0);

    unsafe { avr_device::interrupt::enable() };

    let mut log = aoc_avr::Writer(&mut log);
    loop {
        core::fmt::write(
            &mut log,
            format_args!("toggle {}\r\n", aoc_avr::CLOCK.now()),
        )
        .ok();
        led.toggle();

        arduino_hal::delay_ms(1_000);
    }
}
