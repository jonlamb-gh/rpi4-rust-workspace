//! VideoCore Mailbox

use crate::MMIO_BASE;
use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};
use register::{mmio::ReadOnly, mmio::WriteOnly, register_bitfields};

pub const BASE_PADDR: usize = MMIO_BASE + 0xB000;
pub const BASE_OFFSET: usize = 0x0880;
pub const PADDR: usize = BASE_PADDR + BASE_OFFSET;

register_bitfields! {
    u32,

    STATUS [
        FULL  OFFSET(31) NUMBITS(1) [],
        EMPTY OFFSET(30) NUMBITS(1) []
    ]
}

#[allow(non_snake_case)]
#[repr(C)]
pub struct RegisterBlock {
    pub READ: ReadOnly<u32>,                     // 0x00
    __reserved_0: [u32; 5],                      // 0x04
    pub STATUS: ReadOnly<u32, STATUS::Register>, // 0x18
    __reserved_1: u32,                           // 0x1C
    pub WRITE: WriteOnly<u32>,                   // 0x20
}

pub struct MBOX {
    _marker: PhantomData<*const ()>,
}

unsafe impl Send for MBOX {}

impl MBOX {
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

impl Deref for MBOX {
    type Target = RegisterBlock;
    fn deref(&self) -> &RegisterBlock {
        unsafe { &*self.as_ptr() }
    }
}

impl DerefMut for MBOX {
    fn deref_mut(&mut self) -> &mut RegisterBlock {
        unsafe { &mut *self.as_mut_ptr() }
    }
}
