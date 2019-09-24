//! System Timer

use crate::MMIO_BASE;
use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};

pub const PADDR: usize = MMIO_BASE + 0x0000_3000;

register! {
    /// System Timer Control/Status
    ControlStatus,
    u32,
    RW,
    Fields [
        Match0 WIDTH(U1) OFFSET(U0) [],
        Match1 WIDTH(U1) OFFSET(U1) [],
        Match2 WIDTH(U1) OFFSET(U2) [],
        Match3 WIDTH(U1) OFFSET(U3) [],
    ]
}

register! {
    /// System Timer Counter Lower 32 bits
    CounterLow,
    u32,
    RO,
    Fields [
        Count WIDTH(U32) OFFSET(U0) [],
    ]
}

register! {
    /// System Timer Counter Higher 32 bits
    CounterHigh,
    u32,
    RO,
    Fields [
        Count WIDTH(U32) OFFSET(U0) [],
    ]
}

register! {
    /// System Timer Compare 0
    Compare0,
    u32,
    RW,
    Fields [
        Cmp WIDTH(U32) OFFSET(U0) [],
    ]
}

register! {
    /// System Timer Compare 1
    Compare1,
    u32,
    RW,
    Fields [
        Cmp WIDTH(U32) OFFSET(U0) [],
    ]
}

register! {
    /// System Timer Compare 2
    Compare2,
    u32,
    RW,
    Fields [
        Cmp WIDTH(U32) OFFSET(U0) [],
    ]
}

register! {
    /// System Timer Compare 3
    Compare3,
    u32,
    RW,
    Fields [
        Cmp WIDTH(U32) OFFSET(U0) [],
    ]
}

#[repr(C)]
pub struct RegisterBlock {
    pub cs: ControlStatus::Register, // 0x00
    pub lo: CounterLow::Register,    // 0x04
    pub hi: CounterHigh::Register,   // 0x08
    pub c0: Compare0::Register,      // 0x0C
    pub c1: Compare1::Register,      // 0x10
    pub c2: Compare2::Register,      // 0x14
    pub c3: Compare3::Register,      // 0x18
}

pub struct SysTimer {
    _marker: PhantomData<*const ()>,
}

unsafe impl Send for SysTimer {}

impl SysTimer {
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

impl Deref for SysTimer {
    type Target = RegisterBlock;
    fn deref(&self) -> &RegisterBlock {
        unsafe { &*self.as_ptr() }
    }
}

impl DerefMut for SysTimer {
    fn deref_mut(&mut self) -> &mut RegisterBlock {
        unsafe { &mut *self.as_mut_ptr() }
    }
}
