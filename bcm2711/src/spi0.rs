//! SPI0

use crate::MMIO_BASE;
use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};

pub const PADDR: usize = MMIO_BASE + 0x20_4000;

register! {
    /// Master Control and Status Register
    ControlStatus,
    u32,
    RW,
    Fields [
        ChipSelect WIDTH(U2) OFFSET(U0) [
            CS0 = U0,
            CS1 = U1,
            CS2 = U2,
            NoCS = U3
        ],
        ClockPhase WIDTH(U1) OFFSET(U2) [
            Middle = U0,
            Beginning = U1
        ],
        ClockPolarity WIDTH(U1) OFFSET(U3) [
            RestingLow = U0,
            RestingHigh = U1
        ],
        FifoClear WIDTH(U2) OFFSET(U4) [
            NoAction = U0,
            ClearTx = U1,
            ClearRx = U2,
            ClearTxRx = U3
        ],
        ChipSelectPolarity WIDTH(U1) OFFSET(U6) [
            ActiveLow = U0,
            ActiveHigh = U1
        ],
        TransferActive WIDTH(U1) OFFSET(U7) [],
        DmaEnable WIDTH(U1) OFFSET(U8) [],
        IntOnDone WIDTH(U1) OFFSET(U9) [],
        IntOnRx WIDTH(U1) OFFSET(U10) [],
        AutoDeassert WIDTH(U1) OFFSET(U11) [],
        ReadEnable WIDTH(U1) OFFSET(U12) [],
        LossiEnable WIDTH(U1) OFFSET(U13) [],
        TxfrDone WIDTH(U1) OFFSET(U16) [],
        RxReady WIDTH(U1) OFFSET(U17) [],
        TxReady WIDTH(U1) OFFSET(U18) [],
        RxNeedsRead WIDTH(U1) OFFSET(U19) [],
        RxFull WIDTH(U1) OFFSET(U20) [],
        ChipSelect0Polarity WIDTH(U1) OFFSET(U21) [
            ActiveLow = U0,
            ActiveHigh = U1
        ],
        ChipSelect1Polarity WIDTH(U1) OFFSET(U22) [
            ActiveLow = U0,
            ActiveHigh = U1
        ],
        ChipSelect2Polarity WIDTH(U1) OFFSET(U23) [
            ActiveLow = U0,
            ActiveHigh = U1
        ],
        DmaLossiEnable WIDTH(U1) OFFSET(U24) [],
        LossiLongDataEnable WIDTH(U1) OFFSET(U25) [],
    ]
}

register! {
    /// Master TX and RX FIFO Register
    Fifo,
    u32,
    RW,
    Fields [
        Data WIDTH(U32) OFFSET(U0) [],
    ]
}

register! {
    /// Master Clock Divider Register
    ClockDivider,
    u32,
    RW,
    Fields [
        /// Clock Divider
        /// If CDIV is set to 0, the divisor is 65536.
        /// The divisor must be a multiple of 2.
        /// Odd numbers rounded down.
        /// The maximum SPI clock rate is of the APB clock.
        Divider WIDTH(U16) OFFSET(U0) [],
    ]
}

register! {
    /// Master Data Length Register
    DataLength,
    u32,
    RW,
    Fields [
        /// Data Length
        /// The number of bytes to transfer.
        /// This field is only valid for DMA mode
        /// (DMAEN set) and controls how many bytes
        /// to transmit (and therefore receive).
        Len WIDTH(U16) OFFSET(U0) [],
    ]
}

register! {
    /// LOSSI mode TOH Register
    LossiToh,
    u32,
    RW,
    Fields [
        /// This sets the Output Hold delay in APB clocks.
        /// Avalue of 0 causes a 1 clock delay.
        Toh WIDTH(U4) OFFSET(U0) [],
    ]
}

register! {
    /// DMA DREQ Control Register
    DmaControl,
    u32,
    RW,
    Fields [
        WriteRequestThreshold WIDTH(U8) OFFSET(U0) [],
        WritePanicThreshold WIDTH(U8) OFFSET(U8) [],
        ReadRequestThreshold WIDTH(U8) OFFSET(U16) [],
        ReadPanicThreshold WIDTH(U8) OFFSET(U24) [],
    ]
}

#[repr(C)]
pub struct RegisterBlock {
    pub cs: ControlStatus::Register,    // 0x00
    pub fifo: Fifo::Register,           // 0x04
    pub clock: ClockDivider::Register,  // 0x08
    pub data_len: DataLength::Register, // 0x0C
    pub ltoh: LossiToh::Register,       // 0x10
    pub dc: DmaControl::Register,       // 0x14
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
