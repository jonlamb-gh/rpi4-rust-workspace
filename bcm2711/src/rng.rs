//! RNG

use crate::MMIO_BASE;
use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};
use register::{mmio::ReadOnly, mmio::ReadWrite, register_bitfields};

pub const PADDR: usize = MMIO_BASE + 0x0010_4000;

register_bitfields! {
    u32,

    CTRL [
        ENABLE OFFSET(0) NUMBITS(1) [
            True = 1,
            False = 0
        ]
    ],

    STATUS [
        COUNT OFFSET(0) NUMBITS(24) [],
        READY OFFSET(24) NUMBITS(1) [
            True = 1,
            False = 0
        ]
    ],

    INT_MASK [
        INT_OFF OFFSET(0) NUMBITS(1) [
            True = 1,
            False = 0
        ]
    ]
}

#[allow(non_snake_case)]
#[repr(C)]
pub struct RegisterBlock {
    pub CTRL: ReadWrite<u32, CTRL::Register>,         // 0x00
    pub STATUS: ReadWrite<u32, STATUS::Register>,     // 0x04
    pub DATA: ReadOnly<u32>,                          // 0x08
    __reserved_0: u32,                                // 0x0c
    pub INT_MASK: ReadWrite<u32, INT_MASK::Register>, // 0x10
}

pub struct RNG {
    _marker: PhantomData<*const ()>,
}

unsafe impl Send for RNG {}

impl RNG {
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

impl Deref for RNG {
    type Target = RegisterBlock;
    fn deref(&self) -> &RegisterBlock {
        unsafe { &*self.as_ptr() }
    }
}

impl DerefMut for RNG {
    fn deref_mut(&mut self) -> &mut RegisterBlock {
        unsafe { &mut *self.as_mut_ptr() }
    }
}
