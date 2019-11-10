// TODO - not sure about the filter block's structure

use crate::genet::HFB_PADDR;
use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};

register! {
    Hfb,
    u32,
    RW,
    Fields [
        Bits WIDTH(U32) OFFSET(U0),
    ]
}

pub const NUM_FILTERS: usize = 48;

pub const FILTER_SIZE: usize = 128;

#[repr(C)]
pub struct HardwareFilterBlock {
    pub hfb0: Hfb::Register, // 0x00
    pub hfb1: Hfb::Register, // 0x04
    pub hfb2: Hfb::Register, // 0x08
    pub hfb3: Hfb::Register, // 0x0C
}

#[repr(C)]
pub struct RegisterBlock {
    pub filter_blocks: [HardwareFilterBlock; NUM_FILTERS],
}

pub struct HFB {
    _marker: PhantomData<*const ()>,
}

unsafe impl Send for HFB {}

impl HFB {
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }

    pub fn as_ptr(&self) -> *const RegisterBlock {
        HFB_PADDR as *const _
    }

    pub fn as_mut_ptr(&mut self) -> *mut RegisterBlock {
        HFB_PADDR as *mut _
    }
}

impl Deref for HFB {
    type Target = RegisterBlock;
    fn deref(&self) -> &RegisterBlock {
        unsafe { &*self.as_ptr() }
    }
}

impl DerefMut for HFB {
    fn deref_mut(&mut self) -> &mut RegisterBlock {
        unsafe { &mut *self.as_mut_ptr() }
    }
}
