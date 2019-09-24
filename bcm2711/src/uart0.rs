//! UART0
//!
//! The primary PL011 UART.

use crate::MMIO_BASE;
use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};

pub const PADDR: usize = MMIO_BASE + 0x20_1000;

register! {
    Data,
    u32,
    RW,
    Fields [
        Data WIDTH(U8) OFFSET(U0),
        FramingError WIDTH(U1) OFFSET(U8),
        ParityError WIDTH(U1) OFFSET(U9),
        BreakError WIDTH(U1) OFFSET(U10),
        OverrunError WIDTH(U1) OFFSET(U11),
    ]
}

register! {
    Flag,
    u32,
    RO,
    Fields [
        /// Receive FIFO empty. The meaning of this bit depends on the
        /// state of the FEN bit in the UARTLCR_H Register. If the
        /// FIFO is disabled, this bit is set when the receive holding
        /// register is empty. If the FIFO is enabled, the RXFE bit is
        /// set when the receive FIFO is empty.
        RxEmpty WIDTH(U1) OFFSET(U4) [],

        /// Transmit FIFO full. The meaning of this bit depends on the
        /// state of the FEN bit in the UARTLCR_ LCRH Register. If the
        /// FIFO is disabled, this bit is set when the transmit
        /// holding register is full. If the FIFO is enabled, the TXFF
        /// bit is set when the transmit FIFO is full.
        TxFull WIDTH(U1) OFFSET(U5) [],
    ]
}

register! {
    IntegerBaudRateDivisor,
    u32,
    WO,
    Fields [
        /// Integer baud rate divisor
        Ibrd WIDTH(U16) OFFSET(U0) [],
    ]
}

register! {
    FractionalBaudRateDivisor,
    u32,
    WO,
    Fields [
        /// Fractional baud rate divisor
        Fbrd WIDTH(U16) OFFSET(U0) [],
    ]
}

register! {
    LineControl,
    u32,
    WO,
    Fields [
        /// Word length. These bits indicate the number of data bits
        /// transmitted or received in a frame.
        WordLength WIDTH(U2) OFFSET(U5) [
            FiveBit = U0,
            SixBit = U1,
            SevenBit = U2,
            EightBit = U3
        ]
    ]
}

register! {
    Control,
    u32,
    WO,
    Fields [
        /// UART enable
        Enable WIDTH(U1) OFFSET(U0) [
            /// If the UART is disabled in the middle of transmission
            /// or reception, it completes the current character
            /// before stopping.
            Disabled = U0,
            Enabled = U1
        ],

        /// Transmit enable. If this bit is set to 1, the transmit
        /// section of the UART is enabled. Data transmission occurs
        /// for UART signals. When the UART is disabled in the middle
        /// of transmission, it completes the current character before
        /// stopping.
        TxEnable WIDTH(U1) OFFSET(U8) [
            Disabled = U0,
            Enabled = U1
        ],

        /// Receive enable. If this bit is set to 1, the receive
        /// section of the UART is enabled. Data reception occurs for
        /// UART signals. When the UART is disabled in the middle of
        /// reception, it completes the current character before
        /// stopping.
        RxEnable WIDTH(U1) OFFSET(U9) [
            Disabled = U0,
            Enabled = U1
        ],
    ]
}

register! {
    IntClear,
    u32,
    WO,
    Fields [
        /// Meta field for all pending interrupts
        All WIDTH(U11) OFFSET(U0) [],
    ]
}

#[repr(C)]
pub struct RegisterBlock {
    pub dr: Data::Register,                        // 0x00
    __reserved_0: [u32; 5],                        // 0x04
    pub fr: Flag::Register,                        // 0x18
    __reserved_1: [u32; 2],                        // 0x1c
    pub ibrd: IntegerBaudRateDivisor::Register,    // 0x24
    pub fbrd: FractionalBaudRateDivisor::Register, // 0x28
    pub lcrh: LineControl::Register,               // 0x2C
    pub cr: Control::Register,                     // 0x30
    __reserved_2: [u32; 4],                        // 0x34
    pub icr: IntClear::Register,                   // 0x44
}

pub struct UART0 {
    _marker: PhantomData<*const ()>,
}

unsafe impl Send for UART0 {}

impl UART0 {
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

impl Deref for UART0 {
    type Target = RegisterBlock;
    fn deref(&self) -> &RegisterBlock {
        unsafe { &*self.as_ptr() }
    }
}

impl DerefMut for UART0 {
    fn deref_mut(&mut self) -> &mut RegisterBlock {
        unsafe { &mut *self.as_mut_ptr() }
    }
}
