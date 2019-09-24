//! VideoCore Mailbox

use crate::MMIO_BASE;
use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};

pub const BASE_PADDR: usize = MMIO_BASE + 0xB000;
pub const BASE_OFFSET: usize = 0x0880;
pub const PADDR: usize = BASE_PADDR + BASE_OFFSET;

register! {
    ReadAddr,
    u32,
    RO,
    Fields [
        Addr WIDTH(U32) OFFSET(U0),
    ]
}

register! {
    Status,
    u32,
    RO,
    Fields [
        Empty WIDTH(U1) OFFSET(U30),
        Full WIDTH(U1) OFFSET(U31),
    ]
}

register! {
    WriteAddr,
    u32,
    WO,
    Fields [
        Addr WIDTH(U32) OFFSET(U0),
    ]
}

#[repr(C)]
pub struct RegisterBlock {
    pub read_addr: ReadAddr::Register,   // 0x00
    __reserved_0: [u32; 5],              // 0x04
    pub status: Status::Register,        // 0x018
    __reserved_1: u32,                   // 0x1C
    pub write_addr: WriteAddr::Register, // 0x20
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
