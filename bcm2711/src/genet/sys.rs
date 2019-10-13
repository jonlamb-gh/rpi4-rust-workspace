use crate::genet::SYS_PADDR;
use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};

register! {
    RevCtrl,
    u32,
    RO,
    Fields [
        EPhy WIDTH(U16) OFFSET(U0),
        Minor WIDTH(U4) OFFSET(U16),
        Major WIDTH(U4) OFFSET(U24),
    ]
}

register! {
    PortCtrl,
    u32,
    RW,
    Fields [
        PortMode WIDTH(U5) OFFSET(U0) [
            IntEPhy    = U0,
            IntGPhy    = U1,
            ExtEPhy    = U2,
            ExtGPhy    = U3,
            ExtRvMii25 = U20,
            ExtRvMii50 = U4
        ],
        LedActSrcMac WIDTH(U1) OFFSET(U9),
    ]
}

register! {
    RBufFlushCtrl,
    u32,
    RW,
    Fields [
        Reset WIDTH(U1) OFFSET(U1),
    ]
}

register! {
    TBufFlushCtrl,
    u32,
    RW,
    Fields [
        Bits WIDTH(U32) OFFSET(U0),
    ]
}

#[repr(C)]
pub struct RegisterBlock {
    pub rev_ctrl: RevCtrl::Register,              // 0x00
    pub port_ctrl: PortCtrl::Register,            // 0x04
    pub rbuf_flush_ctrl: RBufFlushCtrl::Register, // 0x08
    pub tbuf_flush_ctrl: TBufFlushCtrl::Register, // 0x0C
}

pub struct SYS {
    _marker: PhantomData<*const ()>,
}

unsafe impl Send for SYS {}

impl SYS {
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }

    pub fn as_ptr(&self) -> *const RegisterBlock {
        SYS_PADDR as *const _
    }

    pub fn as_mut_ptr(&mut self) -> *mut RegisterBlock {
        SYS_PADDR as *mut _
    }
}

impl Deref for SYS {
    type Target = RegisterBlock;
    fn deref(&self) -> &RegisterBlock {
        unsafe { &*self.as_ptr() }
    }
}

impl DerefMut for SYS {
    fn deref_mut(&mut self) -> &mut RegisterBlock {
        unsafe { &mut *self.as_mut_ptr() }
    }
}
