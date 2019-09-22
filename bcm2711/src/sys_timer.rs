//! System Timer

use crate::MMIO_BASE;
use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};
use register::{mmio::ReadOnly, mmio::ReadWrite, register_bitfields};

pub const PADDR: usize = MMIO_BASE + 0x0000_3000;

register_bitfields! {
    u32,

    /// System Timer Control/Status
    CS [
        MATCH0 OFFSET(0) NUMBITS(1) [
            True = 1,
            False = 0
        ],
        MATCH1 OFFSET(1) NUMBITS(1) [
            True = 1,
            False = 0
        ],
        MATCH2 OFFSET(2) NUMBITS(1) [
            True = 1,
            False = 0
        ],
        MATCH3 OFFSET(3) NUMBITS(1) [
            True = 1,
            False = 0
        ]
    ]
}

#[allow(non_snake_case)]
#[repr(C)]
pub struct RegisterBlock {
    /// System Timer Control/Status
    pub CS: ReadWrite<u32, CS::Register>, // 0x00
    /// System Timer Counter Lower 32 bits
    pub LO: ReadOnly<u32>, // 0x04
    /// System Timer Counter Higher 32 bits
    pub HI: ReadOnly<u32>, // 0x08
    /// System Timer Compare 0
    pub C0: ReadWrite<u32>, // 0x0C
    /// System Timer Compare 1
    pub C1: ReadWrite<u32>, // 0x10
    /// System Timer Compare 2
    pub C2: ReadWrite<u32>, // 0x14
    /// System Timer Compare 3
    pub C3: ReadWrite<u32>, // 0x18
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
