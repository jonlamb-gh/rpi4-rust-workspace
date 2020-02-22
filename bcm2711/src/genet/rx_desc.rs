use static_assertions::assert_eq_size;

register! {
    LenStatus,
    u32,
    RW,
    Fields [
        // Rx specific fields
        RxOverflow WIDTH(U1) OFFSET(U0),
        RxCrcErr WIDTH(U1) OFFSET(U1),
        RxErr WIDTH(U1) OFFSET(U2),
        RxNo WIDTH(U1) OFFSET(U3),
        RxLg WIDTH(U1) OFFSET(U4),
        RxMulticast WIDTH(U1) OFFSET(U5),
        RxBroadcast WIDTH(U1) OFFSET(U6),
        RxFi WIDTH(U5) OFFSET(U7),

        // Common fields
        Wrap WIDTH(U1) OFFSET(U12),
        Sop WIDTH(U1) OFFSET(U13),
        Eop WIDTH(U1) OFFSET(U14),
        Own WIDTH(U1) OFFSET(U15),
        Len WIDTH(U12) OFFSET(U16),
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

assert_eq_size!(RxDescriptor, [u8; crate::genet::DMA_DESC_SIZE]);
