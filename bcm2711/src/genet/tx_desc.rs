register! {
    LenStatus,
    u32,
    RW,
    Fields [
        // TODO - QTAG mask and shift seem to overlap with these?
        //
        // Tx specific fields
        TxDoCSum WIDTH(U1) OFFSET(U4),
        TxOwCrc WIDTH(U1) OFFSET(U5),
        TxAppendCrc WIDTH(U1) OFFSET(U6),
        TxUnderRun WIDTH(U5) OFFSET(U9),

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
pub struct TxDescriptor {
    pub len_status: LenStatus::Register, // 0x00
    pub addr_low: AddrLow::Register,     // 0x04
    pub addr_high: AddrHigh::Register,   // 0x08
}
