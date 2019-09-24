//! I2C0

use crate::MMIO_BASE;
use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};

pub const PADDR: usize = MMIO_BASE + 0x20_5000;

register! {
    Control,
    u32,
    RW,
    Fields [
        /// Read/write transfer
        Rw WIDTH(U1) OFFSET(U0) [
            WriteTransfer = U0,
            ReadTransfer = U1
        ],
        /// FIFO Clear
        Clear WIDTH(U2) OFFSET(U4) [
            NoAction = U0,
            ClearFifo = U1
        ],
        /// Start transfer
        StartTransfer WIDTH(U1) OFFSET(U7) [],
        /// Interrupt on done
        IntDone WIDTH(U1) OFFSET(U8) [],
        /// Interrupt on Tx
        IntTx WIDTH(U1) OFFSET(U9) [],
        /// Interrupt on Rx
        IntRx WIDTH(U1) OFFSET(U10) [],
        /// I2C enabled
        Enable WIDTH(U1) OFFSET(U15) [],
    ]
}

register! {
    Status,
    u32,
    RW,
    Fields [
        /// Transfer active
        TransferActive WIDTH(U1) OFFSET(U0) [],
        /// Transfer done
        Done WIDTH(U1) OFFSET(U1) [],
        /// Tx FIFO needs writing (full)
        TxWrite WIDTH(U1) OFFSET(U2) [],
        /// Rx FIFO needs reading (full)
        RxRead WIDTH(U1) OFFSET(U3) [],
        /// Tx FIFO can accept data
        TxData WIDTH(U1) OFFSET(U4) [],
        /// Rx FIFO contains data
        RxData WIDTH(U1) OFFSET(U5) [],
        /// Tx FIFO empty
        TxEempty WIDTH(U1) OFFSET(U6) [],
        /// Rx FIFO full
        RxFull WIDTH(U1) OFFSET(U7) [],
        /// ACK error
        Error WIDTH(U1) OFFSET(U8) [],
        /// Clock stretch timeout
        ClockStretchTimeout WIDTH(U1) OFFSET(U9) [],
    ]
}

register! {
    DataLen,
    u32,
    RW,
    Fields [
        Len WIDTH(U16) OFFSET(U0) [],
    ]
}

register! {
    SlaveAddress,
    u32,
    RW,
    Fields [
        Address WIDTH(U7) OFFSET(U0) [],
    ]
}

register! {
    Fifo,
    u32,
    RW,
    Fields [
        Data WIDTH(U8) OFFSET(U0) [],
    ]
}

register! {
    ClockDivider,
    u32,
    RW,
    Fields [
        Divider WIDTH(U16) OFFSET(U0) [],
    ]
}

register! {
    DataDelay,
    u32,
    RW,
    Fields [
        FallingEdgeDelay WIDTH(U16) OFFSET(U0) [],
        RisingEdgeDelay WIDTH(U16) OFFSET(U16) [],
    ]
}

register! {
    ClockStretchTimeout,
    u32,
    RW,
    Fields [
        Timeout WIDTH(U16) OFFSET(U0) [],
    ]
}

#[allow(non_snake_case)]
#[repr(C)]
pub struct RegisterBlock {
    pub control: Control::Register,          // 0x00
    pub status: Status::Register,            // 0x04
    pub dlen: DataLen::Register,             // 0x08
    pub sa: SlaveAddress::Register,          // 0x0C
    pub fifo: Fifo::Register,                // 0x10
    pub div: ClockDivider::Register,         // 0x14
    pub del: DataDelay::Register,            // 0x18
    pub clkt: ClockStretchTimeout::Register, // 0x1C
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
