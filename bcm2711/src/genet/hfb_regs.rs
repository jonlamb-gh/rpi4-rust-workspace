use crate::genet::hfb::NUM_FILTERS;
use crate::genet::HFB_REGS_PADDR;
use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};

register! {
    Ctrl,
    u32,
    RW,
    Fields [
        Bits WIDTH(U32) OFFSET(U0),
    ]
}

register! {
    /// v3+
    FltEnable,
    u32,
    RW,
    Fields [
        Bits WIDTH(U32) OFFSET(U0),
    ]
}

register! {
    // TODO - unknown, they clear this register
    UnknownCtrl,
    u32,
    RW,
    Fields [
        Bits WIDTH(U32) OFFSET(U0),
    ]
}

register! {
    /// v3+
    FltLen,
    u32,
    RW,
    Fields [
        Bits WIDTH(U32) OFFSET(U0),
    ]
}

#[repr(C)]
pub struct RegisterBlock {
    pub ctrl: Ctrl::Register,                          // 0x00
    pub flt_enable: FltEnable::Register,               // 0x04
    pub unknown_ctrl: UnknownCtrl::Register,           // 0x08
    __reserved_1: [u32; 4],                            // 0x0C
    pub flt_lens: [FltLen::Register; NUM_FILTERS / 4], // 0x1C
}

pub struct HFBREGS {
    _marker: PhantomData<*const ()>,
}

unsafe impl Send for HFBREGS {}

impl HFBREGS {
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }

    pub fn as_ptr(&self) -> *const RegisterBlock {
        HFB_REGS_PADDR as *const _
    }

    pub fn as_mut_ptr(&mut self) -> *mut RegisterBlock {
        HFB_REGS_PADDR as *mut _
    }
}

impl Deref for HFBREGS {
    type Target = RegisterBlock;
    fn deref(&self) -> &RegisterBlock {
        unsafe { &*self.as_ptr() }
    }
}

impl DerefMut for HFBREGS {
    fn deref_mut(&mut self) -> &mut RegisterBlock {
        unsafe { &mut *self.as_mut_ptr() }
    }
}
