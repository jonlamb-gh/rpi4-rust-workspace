use crate::genet::MDIO_PADDR;
use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};

register! {
    Cmd,
    u32,
    RW,
    Fields [
        Data WIDTH(U16) OFFSET(U0),
        Reg WIDTH(U5) OFFSET(U16),
        PhyId WIDTH(U5) OFFSET(U21),
        Rw WIDTH(U2) OFFSET(U26) [
            RwWrite = U1,
            RwRead  = U2
        ],
        ReadFail WIDTH(U1) OFFSET(U28),
        StartBusy WIDTH(U1) OFFSET(U29),
    ]
}

#[repr(C)]
pub struct RegisterBlock {
    pub cmd: Cmd::Register, // 0x00
}

pub struct MDIO {
    _marker: PhantomData<*const ()>,
}

unsafe impl Send for MDIO {}

impl MDIO {
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }

    pub fn as_ptr(&self) -> *const RegisterBlock {
        MDIO_PADDR as *const _
    }

    pub fn as_mut_ptr(&mut self) -> *mut RegisterBlock {
        MDIO_PADDR as *mut _
    }
}

impl Deref for MDIO {
    type Target = RegisterBlock;
    fn deref(&self) -> &RegisterBlock {
        unsafe { &*self.as_ptr() }
    }
}

impl DerefMut for MDIO {
    fn deref_mut(&mut self) -> &mut RegisterBlock {
        unsafe { &mut *self.as_mut_ptr() }
    }
}
