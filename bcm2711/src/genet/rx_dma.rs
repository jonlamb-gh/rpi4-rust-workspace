use crate::genet::rx_desc::RxDescriptor;
use crate::genet::rx_ring::RxRing;
use crate::genet::{NUM_DMA_DESC, NUM_DMA_RINGS, RX_DMA_PADDR};
use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};

register! {
    RingCfg,
    u32,
    RW,
    Fields [
        Bits WIDTH(U32) OFFSET(U0),
    ]
}

register! {
    Ctrl,
    u32,
    RW,
    Fields [
        Enable WIDTH(U1) OFFSET(U0),
        RingBufEnable WIDTH(U16) OFFSET(U1),
        DefDescEnable WIDTH(U1) OFFSET(U17),
        TsbSwapEnable WIDTH(U1) OFFSET(U20),
    ]
}

register! {
    Status,
    u32,
    RW,
    Fields [
        Disabled WIDTH(U1) OFFSET(U0),
        DescRamInitBusy WIDTH(U1) OFFSET(U1),
    ]
}

register! {
    BurstSize,
    u32,
    RW,
    Fields [
        Bits WIDTH(U32) OFFSET(U0),
    ]
}

register! {
    ArbCtrl,
    u32,
    RW,
    Fields [
        Bits WIDTH(U32) OFFSET(U0),
        Mode WIDTH(U2) OFFSET(U0) [
            Rr  = U0,
            Wrr = U1,
            Sp  = U2
        ],
    ]
}

register! {
    Priority0,
    u32,
    RW,
    Fields [
        Bits WIDTH(U32) OFFSET(U0),
    ]
}

register! {
    Priority1,
    u32,
    RW,
    Fields [
        Bits WIDTH(U32) OFFSET(U0),
    ]
}

register! {
    Priority2,
    u32,
    RW,
    Fields [
        Bits WIDTH(U32) OFFSET(U0),
    ]
}

register! {
    Index2Ring,
    u32,
    RW,
    Fields [
        Bits WIDTH(U32) OFFSET(U0),
    ]
}

pub const NUM_INDEX2RINGS: usize = 8;

// NOTE: skiping RING0-16_TIMEOUT (__reserved_1) for now
#[repr(C)]
pub struct RegisterBlock {
    pub descriptors: [RxDescriptor; NUM_DMA_DESC], // 0x0000
    pub rings: [RxRing; NUM_DMA_RINGS],            // 0x0C00
    pub ring_cfg: RingCfg::Register,               // 0x1040
    pub ctrl: Ctrl::Register,                      // 0x1044
    pub status: Status::Register,                  // 0x1048
    pub burst_size: BurstSize::Register,           // 0x104C
    __reserved_0: [u32; 7],                        // 0x1050
    pub arb_ctrl: ArbCtrl::Register,               // 0x106C
    pub priority_0: Priority0::Register,           // 0x1070
    pub priority_1: Priority1::Register,           // 0x1074
    pub priority_2: Priority2::Register,           // 0x1078
    __reserved_1: [u32; 13],                       // 0x107C
    pub index2rings: [Index2Ring::Register; NUM_INDEX2RINGS], // 0x10B0
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
