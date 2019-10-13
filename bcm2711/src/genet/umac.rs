use crate::genet::UMAC_PADDR;
use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};

register! {
    HdBkpCtrl,
    u32,
    RW,
    Fields [
        HdFcEn WIDTH(U1) OFFSET(U0),
        HdFcBkOffOk WIDTH(U1) OFFSET(U1),
        IpgConfigRx WIDTH(U5) OFFSET(U2),
    ]
}

register! {
    Cmd,
    u32,
    RW,
    Fields [
        TxEn WIDTH(U1) OFFSET(U0),
        RxEn WIDTH(U1) OFFSET(U1),
        Speed WIDTH(U2) OFFSET(U2) [
            S10   = U0,
            S100  = U1,
            S1000 = U2,
            S2500 = U3
        ],
        Promisc WIDTH(U1) OFFSET(U4),
        PadEn WIDTH(U1) OFFSET(U5),
        CrcFwd WIDTH(U1) OFFSET(U6),
        PauseFwd WIDTH(U1) OFFSET(U7),
        RxPauseIgnore WIDTH(U1) OFFSET(U8),
        TxAddrIns WIDTH(U1) OFFSET(U9),
        HdEn WIDTH(U1) OFFSET(U10),
        SwReset WIDTH(U1) OFFSET(U13),
        LclLoopEn WIDTH(U1) OFFSET(U15),
        AutoConfig WIDTH(U1) OFFSET(U22),
        CntlFrmEn WIDTH(U1) OFFSET(U23),
        NoLenChk WIDTH(U1) OFFSET(U24),
        RmtLoopEn WIDTH(U1) OFFSET(U25),
        PrblEn WIDTH(U1) OFFSET(U27),
        TxPauseIgnore WIDTH(U1) OFFSET(U28),
        TxRxEn WIDTH(U1) OFFSET(U29),
        RuntFilterDis WIDTH(U1) OFFSET(U30),
    ]
}

register! {
    Mac0,
    u32,
    RW,
    Fields [
        Addr3 WIDTH(U8) OFFSET(U0),
        Addr2 WIDTH(U8) OFFSET(U8),
        Addr1 WIDTH(U8) OFFSET(U16),
        Addr0 WIDTH(U8) OFFSET(U24),
    ]
}

register! {
    Mac1,
    u32,
    RW,
    Fields [
        Addr5 WIDTH(U8) OFFSET(U0),
        Addr4 WIDTH(U8) OFFSET(U8),
    ]
}

register! {
    MaxFrameLen,
    u32,
    RW,
    Fields [
        Len WIDTH(U32) OFFSET(U0),
    ]
}

register! {
    Mode,
    u32,
    RW,
    Fields [
        LinkStatus WIDTH(U1) OFFSET(U5),
    ]
}

register! {
    TxFlush,
    u32,
    RW,
    Fields [
        Flush WIDTH(U1) OFFSET(U0),
    ]
}

register! {
    MibCtrl,
    u32,
    RW,
    Fields [
        ResetRx WIDTH(U1) OFFSET(U0),
        ResetRunt WIDTH(U1) OFFSET(U1),
        ResetTx WIDTH(U1) OFFSET(U2),
    ]
}

register! {
    MdioCmd,
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

register! {
    MdfCtrl,
    u32,
    RW,
    Fields [
        Bits WIDTH(U32) OFFSET(U0),
    ]
}

register! {
    MdfAddr0,
    u32,
    RW,
    Fields [
        Addr1 WIDTH(U8) OFFSET(U0),
        Addr0 WIDTH(U8) OFFSET(U8),
    ]
}

register! {
    MdfAddr1,
    u32,
    RW,
    Fields [
        Addr5 WIDTH(U8) OFFSET(U0),
        Addr4 WIDTH(U8) OFFSET(U8),
        Addr3 WIDTH(U8) OFFSET(U16),
        Addr2 WIDTH(U8) OFFSET(U24),
    ]
}

pub const MAX_MDF_FILTERS: usize = 16;

#[repr(C)]
pub struct MdfAddr {
    pub mdf_addr0: MdfAddr0::Register, // 0x00
    pub mdf_addr1: MdfAddr1::Register, // 0x04
}

#[repr(C)]
pub struct RegisterBlock {
    __reserved_0: u32,                         // 0x000
    pub hd_bkp_ctrl: HdBkpCtrl::Register,      // 0x004
    pub cmd: Cmd::Register,                    // 0x008
    pub mac0: Mac0::Register,                  // 0x00C
    pub mac1: Mac1::Register,                  // 0x010
    pub max_frame_len: MaxFrameLen::Register,  // 0x014
    __reserved_1: [u32; 11],                   // 0x018
    pub mode: Mode::Register,                  // 0x044
    __reserved_2: [u32; 187],                  // 0x048
    pub tx_flush: TxFlush::Register,           // 0x334
    __reserved_3: [u32; 146],                  // 0x338
    pub mib_ctrl: MibCtrl::Register,           // 0x580
    __reserved_4: [u32; 36],                   // 0x584
    pub mdio_cmd: MdioCmd::Register,           // 0x614
    __reserved_5: [u32; 14],                   // 0x618
    pub mdf_ctrl: MdfCtrl::Register,           // 0x650
    pub mdf_addrs: [MdfAddr; MAX_MDF_FILTERS], // 0x654
}

pub struct UMAC {
    _marker: PhantomData<*const ()>,
}

unsafe impl Send for UMAC {}

impl UMAC {
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }

    pub fn as_ptr(&self) -> *const RegisterBlock {
        UMAC_PADDR as *const _
    }

    pub fn as_mut_ptr(&mut self) -> *mut RegisterBlock {
        UMAC_PADDR as *mut _
    }
}

impl Deref for UMAC {
    type Target = RegisterBlock;
    fn deref(&self) -> &RegisterBlock {
        unsafe { &*self.as_ptr() }
    }
}

impl DerefMut for UMAC {
    fn deref_mut(&mut self) -> &mut RegisterBlock {
        unsafe { &mut *self.as_mut_ptr() }
    }
}
