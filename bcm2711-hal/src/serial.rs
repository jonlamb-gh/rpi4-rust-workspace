//! Serial
//!
//! TODO - update this once bcm2711 docs are available
//!
//! There are two built-in UARTS, a PL011 (UART0)
//! and a mini UART (UART1).
//!
//! See the documentation:
//! https://www.raspberrypi.org/documentation/configuration/uart.md

use crate::clocks::Clocks;
use crate::gpio::{Alternate, Pin14, Pin15, AF0, AF5};
use crate::hal::prelude::*;
use crate::hal::serial;
use crate::time::Bps;
use bcm2711::uart0::UART0;
use bcm2711::uart1::UART1;
use core::fmt;
use nb::block;
use void::Void;

pub trait Pins<UART> {}
pub trait PinTx<UART> {}
pub trait PinRx<UART> {}

impl<UART, TX, RX> Pins<UART> for (TX, RX)
where
    TX: PinTx<UART>,
    RX: PinRx<UART>,
{
}

impl PinTx<UART0> for Pin14<Alternate<AF0>> {}
impl PinRx<UART0> for Pin15<Alternate<AF0>> {}

impl PinTx<UART1> for Pin14<Alternate<AF5>> {}
impl PinRx<UART1> for Pin15<Alternate<AF5>> {}

/// Serial abstraction
pub struct Serial<UART, PINS> {
    uart: UART,
    pins: PINS,
}

impl<PINS> Serial<UART0, PINS> {
    pub fn uart0(mut uart: UART0, pins: PINS, baud_rate: Bps, clocks: Clocks) -> Self
    where
        PINS: Pins<UART0>,
    {
        use bcm2711::uart0::*;
        let brr = if baud_rate.0 > (clocks.uart().0 / 16) {
            (clocks.uart().0 * 8) / baud_rate.0
        } else {
            (clocks.uart().0 * 4) / baud_rate.0
        };

        // Turn off UART0
        uart.cr
            .modify(Control::Enable::Clear + Control::TxEnable::Clear + Control::RxEnable::Clear);

        uart.icr.modify(IntClear::All::Clear);
        uart.ibrd
            .modify(IntegerBaudRateDivisor::Ibrd::Field::new(brr >> 6).unwrap());
        uart.fbrd
            .modify(FractionalBaudRateDivisor::Fbrd::Field::new(brr & 0x3F).unwrap());
        uart.lcrh.modify(LineControl::WordLength::EightBit); // 8N1
        uart.cr
            .modify(Control::Enable::Set + Control::TxEnable::Set + Control::RxEnable::Set);

        Serial { uart, pins }
    }

    pub fn free(self) -> (UART0, PINS) {
        (self.uart, self.pins)
    }
}

impl<PINS> serial::Write<u8> for Serial<UART0, PINS> {
    type Error = Void;

    fn flush(&mut self) -> nb::Result<(), Void> {
        use bcm2711::uart0::Flag;
        if !self.uart.fr.is_set(Flag::TxFull::Read) {
            Ok(())
        } else {
            Err(nb::Error::WouldBlock)
        }
    }

    fn write(&mut self, byte: u8) -> nb::Result<(), Void> {
        use bcm2711::uart0::{Data, Flag};
        if !self.uart.fr.is_set(Flag::TxFull::Read) {
            self.uart
                .dr
                .modify(Data::Data::Field::new(byte as _).unwrap());
            Ok(())
        } else {
            Err(nb::Error::WouldBlock)
        }
    }
}

impl<PINS> fmt::Write for Serial<UART0, PINS> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for b in s.bytes() {
            // Convert '\n' to '\r\n'
            if b as char == '\n' {
                block!(self.write('\r' as _)).ok();
            }
            block!(self.write(b)).ok();
        }
        Ok(())
    }
}

impl<PINS> Serial<UART1, PINS> {
    pub fn uart1(mut uart: UART1, pins: PINS, baud_rate: Bps, clocks: Clocks) -> Self
    where
        PINS: Pins<UART1>,
    {
        use bcm2711::uart1::*;
        // Mini UART uses 8-times oversampling
        // baudrate_reg = ((sys_clock / baudrate) / 8) - 1
        let brr = ((clocks.core().0 / baud_rate.0) / 8) - 1;

        uart.enable.modify(AuxEnable::MiniUartEnable::Set);
        uart.ier
            .modify(IntEnable::IntRx::Clear + IntEnable::IntTx::Clear);
        uart.cntl
            .modify(Control::RxEnable::Clear + Control::TxEnable::Clear);
        uart.lcr.modify(LineControl::DataSize::EightBit);
        uart.mcr.modify(ModemControl::Rts::Clear);
        uart.ier
            .modify(IntEnable::IntRx::Clear + IntEnable::IntTx::Clear);
        uart.iir.modify(IntIdentify::FifoClear::All);
        uart.baudrate
            .modify(Baudrate::Rate::Field::new(brr).unwrap());

        uart.cntl
            .modify(Control::RxEnable::Set + Control::TxEnable::Set);

        Serial { uart, pins }
    }

    pub fn free(self) -> (UART1, PINS) {
        (self.uart, self.pins)
    }
}

impl<PINS> serial::Read<u8> for Serial<UART1, PINS> {
    type Error = Void;

    fn read(&mut self) -> nb::Result<u8, Void> {
        use bcm2711::uart1::{Data, LineStatus};
        if self.uart.lsr.is_set(LineStatus::DataReady::Read) {
            let mut data = self.uart.io.get_field(Data::Data::Read).unwrap().val() as u8;

            // convert carrige return to newline
            if data == '\r' as _ {
                data = '\n' as _;
            }

            Ok(data)
        } else {
            Err(nb::Error::WouldBlock)
        }
    }
}

impl<PINS> serial::Write<u8> for Serial<UART1, PINS> {
    type Error = Void;

    fn flush(&mut self) -> nb::Result<(), Void> {
        use bcm2711::uart1::LineStatus;
        if self.uart.lsr.is_set(LineStatus::TxEmpty::Read) {
            Ok(())
        } else {
            Err(nb::Error::WouldBlock)
        }
    }

    fn write(&mut self, byte: u8) -> nb::Result<(), Void> {
        use bcm2711::uart1::{Data, LineStatus};
        if self.uart.lsr.is_set(LineStatus::TxEmpty::Read) {
            self.uart
                .io
                .modify(Data::Data::Field::new(byte as _).unwrap());
            Ok(())
        } else {
            Err(nb::Error::WouldBlock)
        }
    }
}

impl<PINS> core::fmt::Write for Serial<UART1, PINS> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for b in s.bytes() {
            // Convert '\n' to '\r\n'
            if b as char == '\n' {
                block!(self.write('\r' as _)).ok();
            }
            block!(self.write(b)).ok();
        }
        Ok(())
    }
}
