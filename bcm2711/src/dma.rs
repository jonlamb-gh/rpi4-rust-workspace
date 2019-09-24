//! DMA

use crate::MMIO_BASE;
use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};

/// Base address, each channel is offset by 0x100
pub const PADDR: usize = MMIO_BASE + 0x0000_7000;

/// Offset of the global interrupt status register
pub const INT_STATUS_OFFSET: usize = 0xFE0;
pub const INT_STATUS_PADDR: usize = PADDR + INT_STATUS_OFFSET;

/// Offset of the global enable control register
pub const ENABLE_OFFSET: usize = 0xFF0;
pub const ENABLE_PADDR: usize = PADDR + ENABLE_OFFSET;

// TODO - make this an enum
pub const CHANNEL0_OFFSET: usize = 0x000;
pub const CHANNEL1_OFFSET: usize = 0x100;
pub const CHANNEL2_OFFSET: usize = 0x200;
pub const CHANNEL3_OFFSET: usize = 0x300;
pub const CHANNEL4_OFFSET: usize = 0x400;
pub const CHANNEL5_OFFSET: usize = 0x500;
pub const CHANNEL6_OFFSET: usize = 0x600;
pub const CHANNEL7_OFFSET: usize = 0x700;
pub const CHANNEL8_OFFSET: usize = 0x800;
pub const CHANNEL9_OFFSET: usize = 0x900;
pub const CHANNEL10_OFFSET: usize = 0xA00;
pub const CHANNEL11_OFFSET: usize = 0xB00;
pub const CHANNEL12_OFFSET: usize = 0xC00;
pub const CHANNEL13_OFFSET: usize = 0xD00;
pub const CHANNEL14_OFFSET: usize = 0xE00;

register! {
    /// Control and status
    ControlStatus,
    u32,
    RW,
    Fields [
        Active WIDTH(U1) OFFSET(U0),
        End WIDTH(U1) OFFSET(U1),
        Int WIDTH(U1) OFFSET(U2),
        DReq WIDTH(U1) OFFSET(U3),
        Paused WIDTH(U1) OFFSET(U4),
        DReqStopsDMA WIDTH(U1) OFFSET(U5),
        WaitingForOutstandingWrites WIDTH(U1) OFFSET(U6),
        Error WIDTH(U1) OFFSET(U8),
        Priority WIDTH(U4) OFFSET(U16),
        PanicPriority WIDTH(U4) OFFSET(U20),
        WaitForOutstandingWrites WIDTH(U1) OFFSET(U28),
        DisDebug WIDTH(U1) OFFSET(U29),
        Abort WIDTH(U1) OFFSET(U30),
        Reset WIDTH(U1) OFFSET(U31),
    ]
}

register! {
    DcbAddr,
    u32,
    RW,
    Fields [
        Addr WIDTH(U32) OFFSET(U0),
    ]
}

register! {
    /// Transfer information
    TxfrInfo,
    u32,
    RO,
    Fields [
        IntEn WIDTH(U1) OFFSET(U0),
        TdMode WIDTH(U1) OFFSET(U1),
        WaitResp WIDTH(U1) OFFSET(U3),
        DestInc WIDTH(U1) OFFSET(U4),
        DestWidth WIDTH(U1) OFFSET(U5) [
            Width32  = U0,
            Width128 = U1
        ],
        DestDReq WIDTH(U1) OFFSET(U6),
        DestIgnore WIDTH(U1) OFFSET(U7),
        SrcInc WIDTH(U1) OFFSET(U8),
        SrcWidth WIDTH(U1) OFFSET(U9)[
            Width32  = U0,
            Width128 = U1
        ],
        SrcDReq WIDTH(U1) OFFSET(U10),
        SrcIgnore WIDTH(U1) OFFSET(U11),
        BurstLength WIDTH(U4) OFFSET(U12),
        PeriphMap WIDTH(U5) OFFSET(U16),
        Waits WIDTH(U5) OFFSET(U21),
        NoWideBursts WIDTH(U1) OFFSET(U26),
    ]
}

register! {
    SrcAddr,
    u32,
    RO,
    Fields [
        Addr WIDTH(U32) OFFSET(U0),
    ]
}

register! {
    DestAddr,
    u32,
    RO,
    Fields [
        Addr WIDTH(U32) OFFSET(U0),
    ]
}

register! {
    /// Transfer length
    TxfrLen,
    u32,
    RO,
    Fields [
        XLen WIDTH(U16) OFFSET(U0),
        YLen WIDTH(U14) OFFSET(U16),
    ]
}

register! {
    /// 2D stride
    Stride,
    u32,
    RO,
    Fields [
        SrcStride WIDTH(U16) OFFSET(U0),
        DestStride WIDTH(U16) OFFSET(U16),
    ]
}

register! {
    NextDcbAddr,
    u32,
    RO,
    Fields [
        Addr WIDTH(U32) OFFSET(U0),
    ]
}

register! {
    Debug,
    u32,
    RO,
    Fields [
        ReadLastNotSetError WIDTH(U1) OFFSET(U0),
        FifoError WIDTH(U1) OFFSET(U1),
        ReadError WIDTH(U1) OFFSET(U2),
        OutstandingWrites WIDTH(U4) OFFSET(U4),
        DmaId WIDTH(U8) OFFSET(U8),
        DmaState WIDTH(U8) OFFSET(U16),
        Version WIDTH(U3) OFFSET(U25),
        Lite WIDTH(U1) OFFSET(U28),
    ]
}

register! {
    /// Interrupt status
    IntStatus,
    u32,
    RW,
    Fields [
        Int0 WIDTH(U1) OFFSET(U0),
        Int1 WIDTH(U1) OFFSET(U1),
        Int2 WIDTH(U1) OFFSET(U2),
        Int3 WIDTH(U1) OFFSET(U3),
        Int4 WIDTH(U1) OFFSET(U4),
        Int5 WIDTH(U1) OFFSET(U5),
        Int6 WIDTH(U1) OFFSET(U6),
        Int7 WIDTH(U1) OFFSET(U7),
        Int8 WIDTH(U1) OFFSET(U8),
        Int9 WIDTH(U1) OFFSET(U9),
        Int10 WIDTH(U1) OFFSET(U10),
        Int11 WIDTH(U1) OFFSET(U11),
        Int12 WIDTH(U1) OFFSET(U12),
        Int13 WIDTH(U1) OFFSET(U13),
        Int14 WIDTH(U1) OFFSET(U14),
        Int15 WIDTH(U1) OFFSET(U15),
    ]
}

register! {
    /// Global enable bits
    Enable,
    u32,
    RW,
    Fields [
        En0 WIDTH(U1) OFFSET(U0),
        En1 WIDTH(U1) OFFSET(U1),
        En2 WIDTH(U1) OFFSET(U2),
        En3 WIDTH(U1) OFFSET(U3),
        En4 WIDTH(U1) OFFSET(U4),
        En5 WIDTH(U1) OFFSET(U5),
        En6 WIDTH(U1) OFFSET(U6),
        En7 WIDTH(U1) OFFSET(U7),
        En8 WIDTH(U1) OFFSET(U8),
        En9 WIDTH(U1) OFFSET(U9),
        En10 WIDTH(U1) OFFSET(U10),
        En11 WIDTH(U1) OFFSET(U11),
        En12 WIDTH(U1) OFFSET(U12),
        En13 WIDTH(U1) OFFSET(U13),
        En14 WIDTH(U1) OFFSET(U14),
        En15 WIDTH(U1) OFFSET(U15),
    ]
}

#[allow(non_snake_case)]
#[repr(C)]
pub struct GlobalIntStatusRegisterBlock {
    pub int_status: IntStatus::Register,
}

#[allow(non_snake_case)]
#[repr(C)]
pub struct GlobalEnableRegisterBlock {
    pub enable: Enable::Register,
}

#[repr(C)]
pub struct RegisterBlock {
    pub cs: ControlStatus::Register,          // 0x00
    pub dcb_addr: DcbAddr::Register,          // 0x04
    pub ti: TxfrInfo::Register,               // 0x08
    pub src_addr: SrcAddr::Register,          // 0x0C
    pub dest_addr: DestAddr::Register,        // 0x10
    pub txfr_len: TxfrLen::Register,          // 0x14
    pub stride: Stride::Register,             // 0x18
    pub next_dcb_addr: NextDcbAddr::Register, // 0x1C
    pub debug: Debug::Register,               // 0x20
}

pub struct DMA {
    // Starts at PADDR (channel 0)
    paddr: *const usize,
}

unsafe impl Send for DMA {}

impl DMA {
    pub fn new() -> Self {
        Self {
            paddr: PADDR as *const _,
        }
    }

    // TODO - use above enum instead of offset
    pub fn as_channel(self, offset: usize) -> Self {
        Self {
            paddr: (PADDR + offset) as *const _,
        }
    }

    pub fn as_ptr(&self) -> *const RegisterBlock {
        self.paddr as *const _
    }

    pub fn as_mut_ptr(&mut self) -> *mut RegisterBlock {
        self.paddr as *mut _
    }
}

impl Deref for DMA {
    type Target = RegisterBlock;
    fn deref(&self) -> &RegisterBlock {
        unsafe { &*self.as_ptr() }
    }
}

impl DerefMut for DMA {
    fn deref_mut(&mut self) -> &mut RegisterBlock {
        unsafe { &mut *self.as_mut_ptr() }
    }
}

pub struct IntStatusRegister {
    _marker: PhantomData<*const ()>,
}

unsafe impl Send for IntStatusRegister {}

impl IntStatusRegister {
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }

    pub fn as_ptr(&self) -> *const GlobalIntStatusRegisterBlock {
        INT_STATUS_PADDR as *const _
    }

    pub fn as_mut_ptr(&mut self) -> *mut GlobalIntStatusRegisterBlock {
        INT_STATUS_PADDR as *mut _
    }
}

impl Deref for IntStatusRegister {
    type Target = GlobalIntStatusRegisterBlock;
    fn deref(&self) -> &GlobalIntStatusRegisterBlock {
        unsafe { &*self.as_ptr() }
    }
}

impl DerefMut for IntStatusRegister {
    fn deref_mut(&mut self) -> &mut GlobalIntStatusRegisterBlock {
        unsafe { &mut *self.as_mut_ptr() }
    }
}

pub struct EnableRegister {
    _marker: PhantomData<*const ()>,
}

unsafe impl Send for EnableRegister {}

impl EnableRegister {
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }

    pub fn as_ptr(&self) -> *const GlobalEnableRegisterBlock {
        ENABLE_PADDR as *const _
    }

    pub fn as_mut_ptr(&mut self) -> *mut GlobalEnableRegisterBlock {
        ENABLE_PADDR as *mut _
    }
}

impl Deref for EnableRegister {
    type Target = GlobalEnableRegisterBlock;
    fn deref(&self) -> &GlobalEnableRegisterBlock {
        unsafe { &*self.as_ptr() }
    }
}

impl DerefMut for EnableRegister {
    fn deref_mut(&mut self) -> &mut GlobalEnableRegisterBlock {
        unsafe { &mut *self.as_mut_ptr() }
    }
}
