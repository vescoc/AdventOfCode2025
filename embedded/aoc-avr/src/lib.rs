#![no_std]
#![feature(abi_avr_interrupt)]

use core::sync::atomic::Ordering;
use portable_atomic::AtomicU64;

use arduino_hal::prelude::*;
use arduino_hal::pac::tc0::tccr0b::CS0_A;	

use embedded_io::Write as _;

use embedded_aoc::Day;

pub type Instant = fugit::Instant<u64, 1, 1_000_000>;
pub type Duration = fugit::Duration<u64, 1, 1_000_000>;

pub struct Writer<'a, USART, RX, TX>(pub &'a mut arduino_hal::usart::UsartWriter<USART, RX, TX>)
where
    USART: arduino_hal::usart::UsartOps<arduino_hal::hal::Atmega, RX, TX>;

impl<USART, RX, TX> core::fmt::Write for Writer<'_, USART, RX, TX>
where
    USART: arduino_hal::usart::UsartOps<arduino_hal::hal::Atmega, RX, TX>,
{
    fn write_str(&mut self, s: &str) -> Result<(), core::fmt::Error> {
        for b in s.as_bytes() {
            nb::block!(self.0.write(*b)).unwrap();
        }

        Ok(())
    }
}

impl<USART, RX, TX> embedded_io::ErrorType for Writer<'_, USART, RX, TX>
where
    USART: arduino_hal::usart::UsartOps<arduino_hal::hal::Atmega, RX, TX>,
{
    type Error = core::convert::Infallible;
}

impl<USART, RX, TX> embedded_io::Write for Writer<'_, USART, RX, TX>
where
    USART: arduino_hal::usart::UsartOps<arduino_hal::hal::Atmega, RX, TX>,
{
    fn write(&mut self, mut buffer: &[u8]) -> Result<usize, Self::Error> {
        let mut count = 0;
        while let Some((first, remainder)) = buffer.split_first() {
            loop {
                match self.0.write(*first) {
                    Ok(()) => {
                        count += 1;
                        break;
                    }
                    Err(nb::Error::WouldBlock) if count == 0 => {
                        // loop
                    }
                    Err(nb::Error::WouldBlock) => {
                        return Ok(count);
                    }
                    Err(nb::Error::Other(e)) => return Err(e),
                }
            }

            buffer = remainder;
        }

        Ok(count)
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl<USART, RX, TX> embedded_aoc::Handler<u64, 1, 1_000_000> for Writer<'_, USART, RX, TX>
where
    USART: arduino_hal::usart::UsartOps<arduino_hal::hal::Atmega, RX, TX>,
{
    fn started(&mut self, day: Day, _timestamp: Instant) {
        core::fmt::write(self, format_args!("[{day}] started\r\n")).ok();
    }

    fn ended(&mut self, day: Day, elapsed: Duration, part_1: &str, part_2: &str) {
        core::fmt::write(self, format_args!("[{day}] part 1: {part_1}\r\n")).ok();
        core::fmt::write(self, format_args!("[{day}] part 2: {part_2}\r\n")).ok();
        core::fmt::write(
            self,
            format_args!("[{day}] elapsed {}ms\r\n", elapsed.to_millis()),
        )
        .ok();
    }

    fn unsupported_day(&mut self) {
        self.write_all(b"unsupported day\r\n").ok();
    }

    fn invalid_input(&mut self) {
        self.write_all(b"invalid input\r\n").ok();
    }
}

pub struct DummyWriter;

impl embedded_io::ErrorType for DummyWriter {
    type Error = core::convert::Infallible;
}

impl embedded_io::Write for DummyWriter {
    fn write(&mut self, bytes: &[u8]) -> Result<usize, Self::Error> {
        Ok(bytes.len())
    }

    fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}

#[derive(Debug)]
pub struct Error;

impl core::fmt::Display for Error {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> Result<(), core::fmt::Error> {
        write!(f, "Internal Error")
    }
}

impl embedded_io::Error for Error {
    fn kind(&self) -> embedded_io::ErrorKind {
        embedded_io::ErrorKind::Other
    }
}

pub struct Reader<USART, RX, TX>(pub arduino_hal::usart::UsartReader<USART, RX, TX>)
where
    USART: arduino_hal::usart::UsartOps<arduino_hal::hal::Atmega, RX, TX>;

impl<USART, RX, TX> embedded_io::ErrorType for Reader<USART, RX, TX>
where
    USART: arduino_hal::usart::UsartOps<arduino_hal::hal::Atmega, RX, TX>,
{
    type Error = Error;
}

impl<USART, RX, TX> embedded_io::Read for Reader<USART, RX, TX>
where
    USART: arduino_hal::usart::UsartOps<arduino_hal::hal::Atmega, RX, TX>,
{
    fn read(&mut self, mut buffer: &mut [u8]) -> Result<usize, Self::Error> {
        if buffer.is_empty() {
            return Ok(0);
        }

        let mut count = 0;
        while let Some((first, rest)) = buffer.split_first_mut() {
            loop {
                match self.0.read() {
                    Ok(c) => {
                        *first = c;
                        count += 1;
                        break;
                    }
                    Err(nb::Error::Other(_)) => return Err(Error),
                    Err(nb::Error::WouldBlock) if count == 0 => {
                        // ok, busy wait
                    }
                    Err(nb::Error::WouldBlock) => return Ok(count),
                }
            }
            buffer = rest;
        }

        Ok(count)
    }
}

pub static CLOCK: Clock = Clock::new();

pub struct Clock {
    counter: AtomicU64,
}

impl Default for Clock {
	fn default() -> Self {
		Self::new()
	}
}

impl Clock {
    const TOP: u8 = 99;
    const FREQ: u64 = 16_000_000 / 64 / (Self::TOP as u64 + 1);
    const PRESCALER: CS0_A = CS0_A::PRESCALE_64;

	#[must_use]
    pub const fn new() -> Clock {
        Clock {
            counter: AtomicU64::new(0),
        }
    }

	#[allow(clippy::needless_pass_by_value)]
    pub fn start(&self, tc0: arduino_hal::pac::TC0) {
        tc0.tccr0a().write(|w| w.wgm0().ctc());
        tc0.ocr0a().write(|w| w.set(Self::TOP));
        tc0.tccr0b().write(|w| w.cs0().variant(Self::PRESCALER));

        tc0.timsk0().write(|w| w.ocie0a().set_bit());
    }

    pub fn now(&self) -> Instant {
        Instant::from_ticks(
            1_000 * self.counter.load(Ordering::Relaxed).saturating_mul(1_000) / Self::FREQ,
        )
    }

    fn tick(&self) {
        self.counter.fetch_add(1, Ordering::Relaxed);
    }
}

#[cfg(feature = "arduino-mega2560")]
#[avr_device::interrupt(atmega2560)]
fn TIMER0_COMPA() {
    CLOCK.tick();
}
