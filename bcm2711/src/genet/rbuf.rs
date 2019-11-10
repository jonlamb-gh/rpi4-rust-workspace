use crate::genet::RBUF_PADDR;
use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};

register! {
    Ctrl,
    u32,
    RW,
    Fields [
        En64Bit WIDTH(U1) OFFSET(U0),
        Align2Byte WIDTH(U1) OFFSET(U1),
        BadDis WIDTH(U1) OFFSET(U2),
    ]
}

register! {
    Status,
    u32,
    RW,
    Fields [
        WakeOverLan WIDTH(U1) OFFSET(U0),
        MpdIntrActive WIDTH(U1) OFFSET(U1),
        AcpiIntrActive WIDTH(U1) OFFSET(U2),
    ]
}

register! {
    ChkCtrl,
    u32,
    RW,
    Fields [
        RxChkEnable WIDTH(U1) OFFSET(U0),
        SkipFcs WIDTH(U1) OFFSET(U4),
    ]
}

register! {
    TBufSizeCtrl,
    u32,
    RW,
    Fields [
        Bits WIDTH(U32) OFFSET(U0),
    ]
}

#[repr(C)]
pub struct RegisterBlock {
    pub ctrl: Ctrl::Register,                   // 0x00
    __reserved_0: [u32; 2],                     // 0x04
    pub status: Status::Register,               // 0x0C
    __reserved_1: u32,                          // 0x10
    pub chk_ctrl: ChkCtrl::Register,            // 0x14
    __reserved_2: [u32; 39],                    // 0x18
    pub tbuf_size_ctrl: TBufSizeCtrl::Register, // 0xB4
}

pub struct RBUF {
    _marker: PhantomData<*const ()>,
}

unsafe impl Send for RBUF {}

impl RBUF {
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }

    pub fn as_ptr(&self) -> *const RegisterBlock {
        RBUF_PADDR as *const _
    }

    pub fn as_mut_ptr(&mut self) -> *mut RegisterBlock {
        RBUF_PADDR as *mut _
    }
}

impl Deref for RBUF {
    type Target = RegisterBlock;
    fn deref(&self) -> &RegisterBlock {
        unsafe { &*self.as_ptr() }
    }
}

impl DerefMut for RBUF {
    fn deref_mut(&mut self) -> &mut RegisterBlock {
        unsafe { &mut *self.as_mut_ptr() }
    }
}
