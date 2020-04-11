use crate::hal::bcm2711::uart1::UART1;
use crate::hal::gpio::{Alternate, Pin14, Pin15, AF5};
use crate::hal::serial::Serial;
use core::cell::UnsafeCell;
use core::fmt::Write;
use log::{Metadata, Record};

pub type LogInner = Serial<UART1, (Pin14<Alternate<AF5>>, Pin15<Alternate<AF5>>)>;

pub struct SerialLogger {
    serial: UnsafeCell<Option<LogInner>>,
}

impl SerialLogger {
    pub const fn new() -> SerialLogger {
        SerialLogger {
            serial: UnsafeCell::new(None),
        }
    }

    pub fn set_inner(&self, inner: LogInner) {
        let serial = unsafe { &mut *self.serial.get() };
        let _ = serial.replace(inner);
    }
}

unsafe impl Sync for SerialLogger {}

impl log::Log for SerialLogger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let maybe_serial = unsafe { &mut *self.serial.get() };
            if let Some(serial) = maybe_serial {
                writeln!(serial, "[{}] {}", record.level(), record.args()).unwrap();
            } else {
                panic!("Logger was used before being given its inner type");
            }
        }
    }

    fn flush(&self) {}
}
