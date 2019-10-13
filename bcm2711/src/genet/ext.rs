use crate::genet::EXT_PADDR;
use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};

register! {
    PwrMgmt,
    u32,
    RW,
    Fields [
        PwrDownBias WIDTH(U1) OFFSET(U0),
        PwrDownDll WIDTH(U1) OFFSET(U1),
        PwrDownPhy WIDTH(U1) OFFSET(U2),
        PwrDnEnLd WIDTH(U1) OFFSET(U3),
        EnergyDet WIDTH(U1) OFFSET(U4),
        IddqFromPhy WIDTH(U1) OFFSET(U5),
        IddqGlblPwr WIDTH(U1) OFFSET(U7),
        PhyReset WIDTH(U1) OFFSET(U8),
        EnergyDetMask WIDTH(U4) OFFSET(U12),
        PwrDownPhyTx WIDTH(U1) OFFSET(U16),
        PwrDownPhyRx WIDTH(U1) OFFSET(U17),
        PwrDownPhySd WIDTH(U1) OFFSET(U18),
        PwrDownPhyRd WIDTH(U1) OFFSET(U19),
        PwrDownPhyEn WIDTH(U1) OFFSET(U20),
    ]
}

register! {
    RgmiiOobCtrl,
    u32,
    RW,
    Fields [
        RgmiiLink WIDTH(U1) OFFSET(U4),
        OobDisable WIDTH(U1) OFFSET(U5),
        RgmiiModeEn WIDTH(U1) OFFSET(U6),
        IdModeDis WIDTH(U1) OFFSET(U16),
    ]
}

register! {
    GPhyCtrl,
    u32,
    RW,
    Fields [
        CfgIddqBias WIDTH(U1) OFFSET(U0),
        CfgPwrDown WIDTH(U1) OFFSET(U1),
        Ck25Dis WIDTH(U1) OFFSET(U4),
        GPhyReset WIDTH(U1) OFFSET(U5),
    ]
}

#[repr(C)]
pub struct RegisterBlock {
    pub pwr_mgmt: PwrMgmt::Register,            // 0x00
    __reserved_0: [u32; 2],                     // 0x04
    pub rgmii_oob_ctrl: RgmiiOobCtrl::Register, // 0x0C
    __reserved_1: [u32; 3],                     // 0x10
    pub gphy_ctrl: GPhyCtrl::Register,          // 0x1C
}

pub struct EXT {
    _marker: PhantomData<*const ()>,
}

unsafe impl Send for EXT {}

impl EXT {
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }

    pub fn as_ptr(&self) -> *const RegisterBlock {
        EXT_PADDR as *const _
    }

    pub fn as_mut_ptr(&mut self) -> *mut RegisterBlock {
        EXT_PADDR as *mut _
    }
}

impl Deref for EXT {
    type Target = RegisterBlock;
    fn deref(&self) -> &RegisterBlock {
        unsafe { &*self.as_ptr() }
    }
}

impl DerefMut for EXT {
    fn deref_mut(&mut self) -> &mut RegisterBlock {
        unsafe { &mut *self.as_mut_ptr() }
    }
}
