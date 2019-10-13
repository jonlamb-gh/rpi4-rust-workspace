// TODO - field defs

register! {
    ReadPtr,
    u32,
    RW,
    Fields [
        Bits WIDTH(U32) OFFSET(U0),
    ]
}

register! {
    ReadPtrHi,
    u32,
    RW,
    Fields [
        Bits WIDTH(U32) OFFSET(U0),
    ]
}

register! {
    ConsIndex,
    u32,
    RW,
    Fields [
        Bits WIDTH(U32) OFFSET(U0),
    ]
}

register! {
    ProdIndex,
    u32,
    RW,
    Fields [
        Bits WIDTH(U32) OFFSET(U0),
    ]
}

register! {
    BufSize,
    u32,
    RW,
    Fields [
        Bits WIDTH(U32) OFFSET(U0),
    ]
}

register! {
    StartAddr,
    u32,
    RW,
    Fields [
        Bits WIDTH(U32) OFFSET(U0),
    ]
}

register! {
    StartAddrHi,
    u32,
    RW,
    Fields [
        Bits WIDTH(U32) OFFSET(U0),
    ]
}

register! {
    EndAddr,
    u32,
    RW,
    Fields [
        Bits WIDTH(U32) OFFSET(U0),
    ]
}

register! {
    EndAddrHi,
    u32,
    RW,
    Fields [
        Bits WIDTH(U32) OFFSET(U0),
    ]
}

register! {
    MBufDoneThreshold,
    u32,
    RW,
    Fields [
        Bits WIDTH(U32) OFFSET(U0),
    ]
}

register! {
    FlowPeriod,
    u32,
    RW,
    Fields [
        Bits WIDTH(U32) OFFSET(U0),
    ]
}

register! {
    WritePtr,
    u32,
    RW,
    Fields [
        Bits WIDTH(U32) OFFSET(U0),
    ]
}

register! {
    WritePtrHi,
    u32,
    RW,
    Fields [
        Bits WIDTH(U32) OFFSET(U0),
    ]
}

#[repr(C)]
pub struct RxRing {
    pub read_ptr: ReadPtr::Register,                   // 0x00
    pub read_ptr_hi: ReadPtrHi::Register,              // 0x04
    pub cons_index: ConsIndex::Register,               // 0x08
    pub prod_index: ProdIndex::Register,               // 0x0C
    pub buf_size: BufSize::Register,                   // 0x10
    pub start_addr: StartAddr::Register,               // 0x14
    pub start_addr_hi: StartAddrHi::Register,          // 0x18
    pub end_addr: EndAddr::Register,                   // 0x1C
    pub end_addr_hi: EndAddrHi::Register,              // 0x20
    pub mbuf_done_thresh: MBufDoneThreshold::Register, // 0x24
    pub flow_period: FlowPeriod::Register,             // 0x28
    pub write_ptr: WritePtr::Register,                 // 0x2C
    pub write_ptr_hi: WritePtrHi::Register,            // 0x30
    __reserved_0: [u32; 3],                            // 0x34
}
