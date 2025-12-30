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

    let mut serial0 = arduino_hal::usart::Usart::new(
        peripherals.USART0,
        pins.d0,
        pins.d1.into_output(),
        57600.into_baudrate(),
    );

    let mut serial1 = arduino_hal::usart::Usart::new(
        peripherals.USART1,
        pins.d19,
        pins.d18.into_output(),
        9600.into_baudrate(),
    );

    for b in b"sample sample\r\n" {
        nb::block!(serial0.write(*b)).ok();
    }

    loop {
        let b = nb::block!(serial1.read()).unwrap();

        nb::block!(serial0.write(b)).unwrap();

        led.toggle();
    }
}
