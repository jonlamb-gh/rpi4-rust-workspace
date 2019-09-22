//! SPI0

use crate::MMIO_BASE;
use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};
use register::{mmio::ReadWrite, register_bitfields};

pub const PADDR: usize = MMIO_BASE + 0x0020_4000;

register_bitfields! {
    u32,

    /// Master Control and Status
    CS [
        /// Enable Long data word in Lossi mode if DMA_LEN is set
        LEN_LONG OFFSET(25) NUMBITS(1) [],
        /// Enable DMA mode in Lossi mode
        DMA_LEN OFFSET(24) NUMBITS(1) [],
        /// Chip Select 2 Polarity
        CSPOL2 OFFSET(23) NUMBITS(1) [
            ActiveLow = 0,
            ActiveHigh = 1
        ],
        /// Chip Select 1 Polarity
        CSPOL1 OFFSET(22) NUMBITS(1) [
            ActiveLow = 0,
            ActiveHigh = 1
        ],
        /// Chip Select 0 Polarity
        CSPOL0 OFFSET(21) NUMBITS(1) [
            ActiveLow = 0,
            ActiveHigh = 1
        ],
        /// RX FIFO Full
        RXF OFFSET(20) NUMBITS(1) [],
        /// RX FIFO needs Reading (full)
        RXR OFFSET(19) NUMBITS(1) [],
        /// TX FIFO can accept Data
        TXD OFFSET(18) NUMBITS(1) [],
        /// RX FIFO contains Data,
        RXD OFFSET(17) NUMBITS(1) [],
        /// Transfer is done
        DONE OFFSET(16) NUMBITS(1) [],
        /// LoSSI enable
        LEN OFFSET(13) NUMBITS(1) [],
        /// Read Enable
        REN OFFSET(12) NUMBITS(1) [],
        /// Automatically Deassert Chip Select
        ADCS OFFSET(11) NUMBITS(1) [],
        ///  Interrupt on RXR
        INTR OFFSET(10) NUMBITS(1) [],
        ///  Interrupt on Done
        INTD OFFSET(9) NUMBITS(1) [],
        /// DMA Enable
        DMAEN OFFSET(8) NUMBITS(1) [],
        /// Transfer Active
        TA OFFSET(7) NUMBITS(1) [],
        /// Chip Select Polarity
        CSPOL OFFSET(6) NUMBITS(1) [
            ActiveLow = 0,
            ActiveHigh = 1
        ],
        /// FIFO Clear
        CLEAR OFFSET(4) NUMBITS(2) [
            NoAction = 0b00,
            ClearTx = 0b01,
            ClearRx = 0b10,
            ClearTxRx = 0b11
        ],
        /// Clock Polarity
        CPOL OFFSET(3) NUMBITS(1) [
            RestingLow = 0,
            RestingHigh = 1
        ],
        /// Clock Phase
        CPHA OFFSET(2) NUMBITS(1) [
            Middle = 0,
            Beginning = 1
        ],
        /// Chip Select
        CS OFFSET(0) NUMBITS(2) [
            CS_0 = 0b00,
            CS_1 = 0b01,
            CS_2 = 0b10,
            NO_CS = 0b11
        ]
    ],

    /// Master Clock Divider
    CLK [
        /// Clock Divider
        /// If CDIV is set to 0, the divisor is 65536.
        /// The divisor must be a multiple of 2.
        /// Odd numbers rounded down.
        /// The maximum SPI clock rate is of the APB clock.
        CDIV OFFSET(0) NUMBITS(16) []
    ],

    /// Master Data Length
    DLEN [
        /// Data Length
        /// The number of bytes to transfer.
        /// This field is only valid for DMA mode
        /// (DMAEN set) and controls how many bytes
        /// to transmit (and therefore receive).
        LEN OFFSET(0) NUMBITS(16) []
    ],

    /// LOSSI mode TOH
    LTOH [
        /// This sets the Output Hold delay in APB clocks.
        /// Avalue of 0 causes a 1 clock delay.
        TOH OFFSET(0) NUMBITS(4) []
    ],

    /// DMA DREQ Controls
    DC [
        RPANIC OFFSET(24) NUMBITS(8) [],
        RDREQ OFFSET(16) NUMBITS(8) [],
        TPANIC OFFSET(8) NUMBITS(8) [],
        TDREQ OFFSET(0) NUMBITS(8) []
    ]
}

#[allow(non_snake_case)]
#[repr(C)]
pub struct RegisterBlock {
    pub CS: ReadWrite<u32, CS::Register>,     // 0x00
    pub FIFO: ReadWrite<u32>,                 // 0x04
    pub CLK: ReadWrite<u32, CLK::Register>,   // 0x08
    pub DLEN: ReadWrite<u32, DLEN::Register>, // 0x0C
    pub LTOH: ReadWrite<u32, LTOH::Register>, // 0x10
    pub DC: ReadWrite<u32, DC::Register>,     // 0x14
}

pub struct SPI0 {
    _marker: PhantomData<*const ()>,
}

unsafe impl Send for SPI0 {}

impl SPI0 {
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }

    pub fn as_ptr(&self) -> *const RegisterBlock {
        PADDR as *const _
    }

    pub fn as_mut_ptr(&mut self) -> *mut RegisterBlock {
        PADDR as *mut _
    }
}

impl Deref for SPI0 {
    type Target = RegisterBlock;
    fn deref(&self) -> &RegisterBlock {
        unsafe { &*self.as_ptr() }
    }
}

impl DerefMut for SPI0 {
    fn deref_mut(&mut self) -> &mut RegisterBlock {
        unsafe { &mut *self.as_mut_ptr() }
    }
}
