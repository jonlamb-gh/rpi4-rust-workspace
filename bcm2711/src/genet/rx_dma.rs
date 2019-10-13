use crate::genet::NUM_DMA_DESC;
use crate::genet::RX_DMA_PADDR;
use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};

// TODO - field defs
register! {
    LenStatus,
    u32,
    RW,
    Fields [
        Bits WIDTH(U32) OFFSET(U0),
    ]
}

register! {
    AddrLow,
    u32,
    RW,
    Fields [
        Bits WIDTH(U32) OFFSET(U0),
    ]
}

register! {
    AddrHigh,
    u32,
    RW,
    Fields [
        Bits WIDTH(U32) OFFSET(U0),
    ]
}

#[repr(C)]
pub struct RxDescriptor {
    pub len_status: LenStatus::Register, // 0x00
    pub addr_low: AddrLow::Register,     // 0x04
    pub addr_high: AddrHigh::Register,   // 0x08
}

#[repr(C)]
pub struct RegisterBlock {
    pub descriptors: [RxDescriptor; NUM_DMA_DESC],
}

pub struct RXDMA {
    _marker: PhantomData<*const ()>,
}

unsafe impl Send for RXDMA {}

impl RXDMA {
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }

    pub fn as_ptr(&self) -> *const RegisterBlock {
        RX_DMA_PADDR as *const _
    }

    pub fn as_mut_ptr(&mut self) -> *mut RegisterBlock {
        RX_DMA_PADDR as *mut _
    }
}

impl Deref for RXDMA {
    type Target = RegisterBlock;
    fn deref(&self) -> &RegisterBlock {
        unsafe { &*self.as_ptr() }
    }
}

impl DerefMut for RXDMA {
    fn deref_mut(&mut self) -> &mut RegisterBlock {
        unsafe { &mut *self.as_mut_ptr() }
    }
}
