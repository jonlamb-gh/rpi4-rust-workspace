//! Broadcom Serial Controller 0

use crate::MMIO_BASE;
use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};
use register::{mmio::ReadWrite, register_bitfields};

pub const PADDR: usize = MMIO_BASE + 0x20_5000;

register_bitfields! {
    u32,

    /// Control
    CTRL [
        /// I2C enabled
        I2CEN OFFSET(15) NUMBITS(1) [],

        /// Interrupt on Rx
        INTR OFFSET(10) NUMBITS(1) [],

        /// Interrupt on Tx
        INTT OFFSET(9) NUMBITS(1) [],

        /// Interrupt on done
        INTD OFFSET(8) NUMBITS(1) [],

        /// Start transfer
        ST OFFSET(7) NUMBITS(1) [],

        /// FIFO Clear
        CLEAR OFFSET(4) NUMBITS(2) [
            NoAction = 0b00,
            ClearFifo = 0b01
        ],

        /// Read/write transfer
        RW OFFSET(0) NUMBITS(1) [
            WriteTransfer = 0,
            ReadTransfer = 1
        ]
    ],

    /// Status
    STATUS [
        /// Clock stretch timeout
        CLKT OFFSET(9) NUMBITS(1) [],

        /// ACK error
        ERR OFFSET(8) NUMBITS(1) [],

        /// Rx FIFO full
        RXF OFFSET(7) NUMBITS(1) [],

        /// Tx FIFO empty
        TXE OFFSET(6) NUMBITS(1) [],

        /// Rx FIFO contains data
        RXD OFFSET(5) NUMBITS(1) [],

        /// Tx FIFO can accept data
        TXD OFFSET(4) NUMBITS(1) [],

        /// Rx FIFO needs reading (full)
        RXR OFFSET(3) NUMBITS(1) [],

        /// Tx FIFO needs writing (full)
        TXW OFFSET(2) NUMBITS(1) [],

        /// Transfer done
        DONE OFFSET(1) NUMBITS(1) [],

        /// Transfer active
        TA OFFSET(0) NUMBITS(1) []
    ],

    /// Data length
    DLEN [
        DLEN OFFSET(0) NUMBITS(16) []
    ],

    /// Slave address
    SA [
        ADDR OFFSET(0) NUMBITS(7) []
    ],

    /// Data FIFO
    FIFO [
        DATA OFFSET(0) NUMBITS(8) []
    ],

    /// Clock divider
    DIV [
        CDIV OFFSET(0) NUMBITS(16) []
    ],

    /// Data delay
    DEL [
        /// Failling edge delay
        FEDL OFFSET(16) NUMBITS(16) [],

        /// Rising edge delay
        REDL OFFSET(0) NUMBITS(16) []
    ],

    /// Clock stretch timeout
    CLKT [
        TOUT OFFSET(0) NUMBITS(16) []
    ]
}

#[allow(non_snake_case)]
#[repr(C)]
pub struct RegisterBlock {
    pub CTRL: ReadWrite<u32, CTRL::Register>,     // 0x00
    pub STATUS: ReadWrite<u32, STATUS::Register>, // 0x04
    pub DLEN: ReadWrite<u32, DLEN::Register>,     // 0x08
    pub SA: ReadWrite<u32, SA::Register>,         // 0x0C
    pub FIFO: ReadWrite<u32, FIFO::Register>,     // 0x10
    pub DIV: ReadWrite<u32, DIV::Register>,       // 0x14
    pub DEL: ReadWrite<u32, DEL::Register>,       // 0x18
    pub CLKT: ReadWrite<u32, CLKT::Register>,     // 0x1C
}

pub struct I2C0 {
    _marker: PhantomData<*const ()>,
}

unsafe impl Send for I2C0 {}

impl I2C0 {
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

impl Deref for I2C0 {
    type Target = RegisterBlock;
    fn deref(&self) -> &RegisterBlock {
        unsafe { &*self.as_ptr() }
    }
}

impl DerefMut for I2C0 {
    fn deref_mut(&mut self) -> &mut RegisterBlock {
        unsafe { &mut *self.as_mut_ptr() }
    }
}
