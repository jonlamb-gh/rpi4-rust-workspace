//! UART1
//!
//! The auxilary mini UART.

use crate::MMIO_BASE;
use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};

pub const PADDR: usize = MMIO_BASE + 0x21_5000;

register! {
    AuxEnable,
    u32,
    RW,
    Fields [
        /// If set the mini UART is enabled. The UART will immediately
        /// start receiving data, especially if the UART1_RX line is
        /// low.
        /// If clear the mini UART is disabled. That also disables any
        /// mini UART register access
        MiniUartEnable WIDTH(U1) OFFSET(U0) [],
        Spi1Enable WIDTH(U1) OFFSET(U1) [],
        Spi2Enable WIDTH(U1) OFFSET(U2) [],
    ]
}

register! {
    Data,
    u32,
    RW,
    Fields [
        Data WIDTH(U8) OFFSET(U0) [],
    ]
}

register! {
    IntEnable,
    u32,
    WO,
    Fields [
        IntRx WIDTH(U1) OFFSET(U0) [],
        IntTx WIDTH(U1) OFFSET(U1) [],
    ]
}

register! {
    /// Mini Uart Interrupt Identify
    IntIdentify,
    u32,
    RW,
    Fields [
        /// Writing with bit 1 set will clear the receive FIFO
        /// Writing with bit 2 set will clear the transmit FIFO
        FifoClear WIDTH(U2) OFFSET(U1) [
            NoAction = U0,
            Rx = U1,
            Tx = U2,
            All = U3
        ],
    ]
}

register! {
    LineControl,
    u32,
    WO,
    Fields [
        /// Mode the UART works in
        DataSize WIDTH(U2) OFFSET(U0) [
            SevenBit = U0,
            EightBit = U3
        ],
    ]
}

register! {
    ModemControl,
    u32,
    WO,
    Fields [
        Rts WIDTH(U1) OFFSET(U0) [],
    ]
}

register! {
    LineStatus,
    u32,
    RO,
    Fields [
        /// This bit is set if the receive FIFO holds at least 1
        /// symbol.
        DataReady WIDTH(U1) OFFSET(U0) [],

        /// This bit is set if the transmit FIFO can accept at least
        /// one byte.
        TxEmpty WIDTH(U1) OFFSET(U5) [],
    ]
}

register! {
    Control,
    u32,
    WO,
    Fields [
        /// If this bit is set the mini UART receiver is enabled.
        /// If this bit is clear the mini UART receiver is disabled.
        RxEnable WIDTH(U1) OFFSET(U0) [
            Disabled = U0,
            Enabled = U1
        ],

        /// If this bit is set the mini UART transmitter is enabled.
        /// If this bit is clear the mini UART transmitter is disabled.
        TxEnable WIDTH(U1) OFFSET(U1) [
            Disabled = U0,
            Enabled = U1
        ],
    ]
}

register! {
    Baudrate,
    u32,
    WO,
    Fields [
        Rate WIDTH(U16) OFFSET(U0) [],
    ]
}

#[repr(C)]
pub struct RegisterBlock {
    __reserved_0: u32,                // 0x00
    pub enable: AuxEnable::Register,  // 0x04
    __reserved_1: [u32; 14],          // 0x08
    pub io: Data::Register,           // 0x40
    pub ier: IntEnable::Register,     // 0x44
    pub iir: IntIdentify::Register,   // 0x48
    pub lcr: LineControl::Register,   // 0x4C
    pub mcr: ModemControl::Register,  // 0x50
    pub lsr: LineStatus::Register,    // 0x54
    __reserved_2: [u32; 2],           // 0x58
    pub cntl: Control::Register,      // 0x60
    __reserved_3: u32,                // 0x64
    pub baudrate: Baudrate::Register, // 0x68
}

pub struct UART1 {
    _marker: PhantomData<*const ()>,
}

unsafe impl Send for UART1 {}

impl UART1 {
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

impl Deref for UART1 {
    type Target = RegisterBlock;
    fn deref(&self) -> &RegisterBlock {
        unsafe { &*self.as_ptr() }
    }
}

impl DerefMut for UART1 {
    fn deref_mut(&mut self) -> &mut RegisterBlock {
        unsafe { &mut *self.as_mut_ptr() }
    }
}
