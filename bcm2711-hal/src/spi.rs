//! SPI
//!
//! TODO - update this once bcm2711 docs are available
//!
//! - Only implemented for SPI0 because the other two are different
//! - Only supports chip select 0 and 1 pins

// TODO - add events/etc

use crate::clocks::Clocks;
use crate::gpio::{Alternate, Pin10, Pin11, Pin9, AF0};
use crate::hal::spi::{self, Mode, Phase, Polarity};
use crate::time::Hertz;
use bcm2711::spi0::*;
use nb::block;

/// SPI error
#[derive(Debug)]
pub enum Error {
    #[doc(hidden)]
    _Extensible,
}

pub trait Pins<SPI> {}
pub trait PinCs<SPI> {}
pub trait PinSck<SPI> {}
pub trait PinMiso<SPI> {}
pub trait PinMosi<SPI> {}

impl<SPI, CS, SCK, MISO, MOSI> Pins<SPI> for (CS, SCK, MISO, MOSI)
where
    CS: PinCs<SPI>,
    SCK: PinSck<SPI>,
    MISO: PinMiso<SPI>,
    MOSI: PinMosi<SPI>,
{
}

/// A filler type for when the CS pin is unnecessary
/// because it will be driven by the software logic
pub struct NoCs;
/// A filler type for when the SCK pin is unnecessary
pub struct NoSck;
/// A filler type for when the Miso pin is unnecessary
pub struct NoMiso;
/// A filler type for when the Mosi pin is unnecessary
pub struct NoMosi;

macro_rules! pins {
    ($($SPIX:ty: CS: [$($CS:ty),*] SCK: [$($SCK:ty),*] MISO: [$($MISO:ty),*] MOSI: [$($MOSI:ty),*])+) => {
        $(
            $(
                impl PinCs<$SPIX> for $CS {}
            )*
            $(
                impl PinSck<$SPIX> for $SCK {}
            )*
            $(
                impl PinMiso<$SPIX> for $MISO {}
            )*
            $(
                impl PinMosi<$SPIX> for $MOSI {}
            )*
        )+
    }
}

pins! {
    SPI0:
        CS: [
            NoCs
            // TODO - add logic to handle hw-controlled CS
            //Pin7<Alternate<AF0>>,
            //Pin8<Alternate<AF0>>
        ]
        SCK: [
            Pin11<Alternate<AF0>>
        ]
        MISO: [
            Pin9<Alternate<AF0>>
        ]
        MOSI: [
            Pin10<Alternate<AF0>>
        ]
}

/// Interrupt events
pub enum Event {
    /// New data has been received
    Rxne,
    /// Data can be sent
    Txe,
    /// An error occurred
    Error,
}

#[derive(Debug)]
pub struct Spi<SPI, PINS> {
    spi: SPI,
    pins: PINS,
}

impl<PINS> Spi<SPI0, PINS> {
    pub fn spi0(spi: SPI0, pins: PINS, mode: Mode, freq: Hertz, clocks: Clocks) -> Self
    where
        PINS: Pins<SPI0>,
    {
        // Disable, reset FIFOs
        spi.CS.modify(
            CS::TA::CLEAR
                + CS::DMAEN::CLEAR
                + CS::INTD::CLEAR
                + CS::INTR::CLEAR
                + CS::CLEAR::ClearTxRx,
        );

        spi.DLEN.modify(DLEN::LEN.val(0));

        // Clock polarity and phase
        spi.CS.modify(
            CS::CPOL.val((mode.polarity == Polarity::IdleHigh) as _)
                + CS::CPHA.val((mode.phase == Phase::CaptureOnSecondTransition) as _),
        );

        // TODO - need to construct Clocks using mailbox data from vc?
        // TODO - open up all the speeds, this is ported from the Linux driver
        let cdiv = if freq.0 >= (clocks.core().0 / 2) {
            // clk_hz/2 is the fastest we can go
            2
        } else if freq.0 > 0 {
            // CDIV must be a multiple of two
            let mut div = (clocks.core().0 + freq.0 - 1) / freq.0;
            div += div % 2;

            if div >= 65536 {
                0
            } else {
                div
            }
        } else {
            // 0 is the slowest we can go
            0
        };

        spi.CLK.modify(CLK::CDIV.val(cdiv));

        spi.CS.modify(CS::REN::CLEAR);

        // TODO - only handling NoCs atm, meaning software has
        // to drive CS gpio pins
        // need to add support for hw-controled CS pins
        spi.CS.modify(CS::CS::NO_CS);

        Spi { spi, pins }
    }

    /// Enable interrupts for the given `event`:
    ///  - Received data ready to be read (RXNE)
    ///  - Transmit data register empty (TXE)
    ///  - Transfer error
    pub fn listen(&mut self, _event: Event) {
        unimplemented!();
    }

    /// Disable interrupts for the given `event`:
    ///  - Received data ready to be read (RXNE)
    ///  - Transmit data register empty (TXE)
    ///  - Transfer error
    pub fn unlisten(&mut self, _event: Event) {
        unimplemented!();
    }

    pub fn free(self) -> (SPI0, PINS) {
        (self.spi, self.pins)
    }

    #[inline]
    fn rx(&mut self) -> nb::Result<u8, Error> {
        if self.spi.CS.is_set(CS::RXD) {
            Ok((self.spi.FIFO.get() & 0xFF) as u8)
        } else {
            Err(nb::Error::WouldBlock)
        }
    }

    #[inline]
    fn tx(&mut self, byte: u8) -> nb::Result<(), Error> {
        if self.spi.CS.is_set(CS::TXD) {
            self.spi.FIFO.set(byte as _);
            Ok(())
        } else {
            Err(nb::Error::WouldBlock)
        }
    }
}

impl<PINS> spi::FullDuplex<u8> for Spi<SPI0, PINS> {
    type Error = Error;

    fn read(&mut self) -> nb::Result<u8, Error> {
        self.spi.CS.modify(CS::TA::SET);
        let ret = self.rx();
        self.spi.CS.modify(CS::TA::CLEAR);
        ret
    }

    fn send(&mut self, byte: u8) -> nb::Result<(), Error> {
        self.spi.CS.modify(CS::TA::SET);
        let ret = self.tx(byte);
        self.spi.CS.modify(CS::TA::CLEAR);
        ret
    }
}

impl<PINS> crate::hal::blocking::spi::Transfer<u8> for Spi<SPI0, PINS> {
    type Error = Error;

    fn transfer<'w>(&mut self, words: &'w mut [u8]) -> Result<&'w [u8], Error> {
        self.spi.CS.modify(CS::TA::SET);

        for word in words.iter_mut() {
            block!(self.tx(word.clone()))?;
            *word = block!(self.rx())?;
        }

        self.spi.CS.modify(CS::TA::CLEAR);
        Ok(words)
    }
}

impl<PINS> crate::hal::blocking::spi::Write<u8> for Spi<SPI0, PINS> {
    type Error = Error;

    fn write(&mut self, words: &[u8]) -> Result<(), Error> {
        self.spi.CS.modify(CS::TA::SET);

        for word in words {
            block!(self.tx(word.clone()))?;
            block!(self.rx())?;
        }

        self.spi.CS.modify(CS::TA::CLEAR);
        Ok(())
    }
}
