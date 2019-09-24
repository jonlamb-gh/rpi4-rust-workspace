//! RNG

use crate::MMIO_BASE;
use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};

pub const PADDR: usize = MMIO_BASE + 0x0010_4000;

register! {
    Control,
    u32,
    RW,
    Fields [
        Enable WIDTH(U1) OFFSET(U0) [],
    ]
}

register! {
    Status,
    u32,
    RW,
    Fields [
        Count WIDTH(U24) OFFSET(U0) [],
        Ready WIDTH(U1) OFFSET(U24) [],
    ]
}

register! {
    Fifo,
    u32,
    RO,
    Fields [
        Data WIDTH(U32) OFFSET(U0) [],
    ]
}

register! {
    IntMask,
    u32,
    RW,
    Fields [
        IntOff WIDTH(U1) OFFSET(U0) [],
    ]
}

#[repr(C)]
pub struct RegisterBlock {
    pub control: Control::Register,  // 0x00
    pub status: Status::Register,    // 0x04
    pub data: Fifo::Register,        // 0x08
    __reserved_0: u32,               // 0x0c
    pub int_mask: IntMask::Register, // 0x10
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
