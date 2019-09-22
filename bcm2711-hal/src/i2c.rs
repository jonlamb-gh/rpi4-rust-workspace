//! I2C
//!
//! TODO - update this once bcm2711 docs are available

use crate::clocks::Clocks;
use crate::gpio::{Alternate, Pin2, Pin3, AF0};
use crate::hal::blocking::i2c::{Read, Write, WriteRead};
use crate::time::Hertz;
use bcm2711::{bsc0::*, bsc1::I2C1};

/// I2C error
#[derive(Debug)]
pub enum Error {
    /// No acknowledge returned
    Nack,
    /// Slave held the SCL low for longer than specified
    ClockStretchTimeout,
    /// Hw reported done but didn't drain the entire buffer
    Truncated,
    #[doc(hidden)]
    _Extensible,
}

pub trait Pins<I2c> {}
pub trait PinScl<I2c> {}
pub trait PinSda<I2c> {}

impl<I2c, SCL, SDA> Pins<I2c> for (SCL, SDA)
where
    SCL: PinScl<I2c>,
    SDA: PinSda<I2c>,
{
}

impl PinSda<I2C1> for Pin2<Alternate<AF0>> {}
impl PinScl<I2C1> for Pin3<Alternate<AF0>> {}

/// I2C abstraction
pub struct I2c<I2C, PINS> {
    i2c: I2C,
    pins: PINS,
}

const FIFO_DEPTH: usize = 16;

impl<PINS> I2c<I2C1, PINS> {
    pub fn i2c1<S>(i2c: I2C1, pins: PINS, speed: S, clocks: Clocks) -> Self
    where
        PINS: Pins<I2C1>,
        S: Into<Hertz>,
    {
        // Reset, clear status bits
        i2c.CTRL.set(0);
        i2c.STATUS
            .modify(STATUS::CLKT::SET + STATUS::ERR::SET + STATUS::DONE::SET);

        let speed: Hertz = speed.into();
        let cdiv = clocks.core().0 / speed.0;

        // Configure clock divider and clear FIFOs
        i2c.DIV.modify(DIV::CDIV.val(cdiv));
        i2c.CTRL.modify(CTRL::CLEAR::ClearFifo);

        I2c { i2c, pins }
    }

    pub fn free(self) -> (I2C1, PINS) {
        (self.i2c, self.pins)
    }

    #[inline]
    fn recv_byte(&self) -> Result<u8, Error> {
        while !self.i2c.STATUS.is_set(STATUS::RXD) {}
        Ok(self.i2c.FIFO.read(FIFO::DATA) as u8)
    }

    #[inline]
    fn send_byte(&self, byte: u8) -> Result<(), Error> {
        while !self.i2c.STATUS.is_set(STATUS::TXD) {}
        self.i2c.FIFO.modify(FIFO::DATA.val(byte as _));
        Ok(())
    }
}

impl<PINS> Read for I2c<I2C1, PINS> {
    type Error = Error;

    fn read(&mut self, addr: u8, buffer: &mut [u8]) -> Result<(), Self::Error> {
        // Clear FIFO
        self.i2c.CTRL.modify(CTRL::CLEAR::ClearFifo);

        // Clear status
        self.i2c
            .STATUS
            .modify(STATUS::CLKT::SET + STATUS::ERR::SET + STATUS::DONE::SET);

        // Set data length
        self.i2c.DLEN.modify(DLEN::DLEN.val(buffer.len() as _));

        // Set slave address
        self.i2c.SA.modify(SA::ADDR.val(addr as _));

        // Start read
        self.i2c
            .CTRL
            .modify(CTRL::I2CEN::SET + CTRL::ST::SET + CTRL::RW::ReadTransfer);

        for c in buffer {
            *c = self.recv_byte()?;
        }

        // TODO - check done?
        //while !self.i2c.STATUS.is_set(STATUS::DONE) {
        assert_eq!(self.i2c.STATUS.is_set(STATUS::DONE), true);

        let result = if self.i2c.STATUS.is_set(STATUS::ERR) {
            Err(Error::Nack)
        } else if self.i2c.STATUS.is_set(STATUS::CLKT) {
            Err(Error::ClockStretchTimeout)
        } else {
            Ok(())
        };

        // Clear done
        self.i2c.STATUS.modify(STATUS::DONE::SET);

        self.i2c.CTRL.modify(CTRL::I2CEN::CLEAR);

        result
    }
}

impl<PINS> Write for I2c<I2C1, PINS> {
    type Error = Error;

    fn write(&mut self, addr: u8, buffer: &[u8]) -> Result<(), Self::Error> {
        // Set slave address
        self.i2c.SA.modify(SA::ADDR.val(addr as _));

        // Clear FIFO
        self.i2c.CTRL.modify(CTRL::CLEAR::ClearFifo);

        // Clear status
        self.i2c
            .STATUS
            .modify(STATUS::CLKT::SET + STATUS::ERR::SET + STATUS::DONE::SET);

        // Set data length
        self.i2c.DLEN.modify(DLEN::DLEN.val(buffer.len() as _));

        // Fill the FIFO
        let mut cnt = 0;
        for _ in 0..FIFO_DEPTH {
            self.i2c.FIFO.modify(FIFO::DATA.val(buffer[cnt] as _));
            cnt += 1;
            if cnt >= buffer.len() {
                break;
            }
        }

        // Start write
        self.i2c
            .CTRL
            .modify(CTRL::I2CEN::SET + CTRL::ST::SET + CTRL::RW::WriteTransfer);

        while !self.i2c.STATUS.is_set(STATUS::DONE) {
            if cnt < buffer.len() {
                for c in &buffer[cnt..] {
                    self.send_byte(*c)?;
                    cnt += 1;
                }
            }
        }

        let result = if self.i2c.STATUS.is_set(STATUS::ERR) {
            Err(Error::Nack)
        } else if self.i2c.STATUS.is_set(STATUS::CLKT) {
            Err(Error::ClockStretchTimeout)
        } else if cnt != buffer.len() {
            Err(Error::Truncated)
        } else {
            Ok(())
        };

        // Clear done
        self.i2c.STATUS.modify(STATUS::DONE::SET);

        self.i2c.CTRL.modify(CTRL::I2CEN::CLEAR);

        result
    }
}

impl<PINS> WriteRead for I2c<I2C1, PINS> {
    type Error = Error;

    fn write_read(&mut self, addr: u8, bytes: &[u8], buffer: &mut [u8]) -> Result<(), Self::Error> {
        self.write(addr, bytes)?;
        self.read(addr, buffer)?;

        Ok(())
    }
}
