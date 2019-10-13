// Each register is a `crate::genet::intrl2::INTRL2` bitfield

use crate::genet::INTRL2_0_PADDR;
use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};

register! {
    Status,
    u32,
    RW,
    Fields [
        Bits WIDTH(U32) OFFSET(U0),
    ]
}

register! {
    Set,
    u32,
    RW,
    Fields [
        Bits WIDTH(U32) OFFSET(U0),
    ]
}

register! {
    Clear,
    u32,
    RW,
    Fields [
        Bits WIDTH(U32) OFFSET(U0),
    ]
}

register! {
    MaskStatus,
    u32,
    RW,
    Fields [
        Bits WIDTH(U32) OFFSET(U0),
    ]
}

register! {
    MaskSet,
    u32,
    RW,
    Fields [
        Bits WIDTH(U32) OFFSET(U0),
    ]
}

register! {
    MaskClear,
    u32,
    RW,
    Fields [
        Bits WIDTH(U32) OFFSET(U0),
    ]
}

#[repr(C)]
pub struct RegisterBlock {
    pub status: Status::Register,          // 0x00
    pub set: Set::Register,                // 0x04
    pub clear: Clear::Register,            // 0x08
    pub mask_status: MaskStatus::Register, // 0x0C
    pub mask_set: MaskSet::Register,       // 0x10
    pub mask_clear: MaskClear::Register,   // 0x14
}

pub struct INTRL2_0 {
    _marker: PhantomData<*const ()>,
}

unsafe impl Send for INTRL2_0 {}

impl INTRL2_0 {
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }

    pub fn as_ptr(&self) -> *const RegisterBlock {
        INTRL2_0_PADDR as *const _
    }

    pub fn as_mut_ptr(&mut self) -> *mut RegisterBlock {
        INTRL2_0_PADDR as *mut _
    }
}

impl Deref for INTRL2_0 {
    type Target = RegisterBlock;
    fn deref(&self) -> &RegisterBlock {
        unsafe { &*self.as_ptr() }
    }
}

impl DerefMut for INTRL2_0 {
    fn deref_mut(&mut self) -> &mut RegisterBlock {
        unsafe { &mut *self.as_mut_ptr() }
    }
}
