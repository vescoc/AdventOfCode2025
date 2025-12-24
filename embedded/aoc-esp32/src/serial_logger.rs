use core::cell::RefCell;
use core::fmt::Write;

use esp_hal::{gpio::Output, uart::Uart};

include!(concat!(env!("OUT_DIR"), "/log_filter.rs"));

static LOGGER: SerialLogger = SerialLogger::new();

struct SerialLoggerInner {
    serial: Uart<'static, esp_hal::Blocking>,
    led_error: Output<'static>,
}

pub struct SerialLogger(critical_section::Mutex<RefCell<Option<SerialLoggerInner>>>);

impl SerialLogger {
    const fn new() -> Self {
        Self(critical_section::Mutex::new(RefCell::new(None)))
    }

    /// # Errors
    pub fn init(
        serial: Uart<'static, esp_hal::Blocking>,
        led_error: Output<'static>,
    ) -> Result<(), log::SetLoggerError> {
        critical_section::with(|cs| {
            LOGGER
                .0
                .replace(cs, Some(SerialLoggerInner { serial, led_error }));

            unsafe {
                log::set_max_level_racy(FILTER_MAX);
            }
            unsafe { log::set_logger_racy(&LOGGER) }
        })
    }
}

impl log::Log for SerialLogger {
    fn enabled(&self, _: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        critical_section::with(|cs| {
            if self.enabled(record.metadata())
                && let Some(SerialLoggerInner { serial, led_error }) =
                    LOGGER.0.borrow_ref_mut(cs).as_mut()
            {
                let level = record.level();
                if level == log::Level::Error {
                    led_error.set_high();
                }

                writeln!(serial, "{}: {}", level, record.args()).ok();
            }
        });
    }

    fn flush(&self) {
        critical_section::with(|cs| {
            if let Some(SerialLoggerInner { serial, .. }) = LOGGER.0.borrow_ref_mut(cs).as_mut() {
                serial.flush().ok();
            }
        });
    }
}
