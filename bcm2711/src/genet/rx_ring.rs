// TODO - field defs
// Rx/Tx diff
// - CONS/PROD
// - XON_XOFF_THRESH / FLOW_PERIOD

use static_assertions::assert_eq_size;

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
        Index WIDTH(U16) OFFSET(U0),
        DiscardCnt WIDTH(U16) OFFSET(U16),
    ]
}

register! {
    BufSize,
    u32,
    RW,
    Fields [
        BufferSize WIDTH(U16) OFFSET(U0),
        Size WIDTH(U16) OFFSET(U16),
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
    XonXoffThresh,
    u32,
    RW,
    Fields [
        XonThresh WIDTH(U16) OFFSET(U0),
        XoffThresh WIDTH(U16) OFFSET(U16),
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
    pub write_ptr: ReadPtr::Register,                  // 0x00
    pub write_ptr_hi: ReadPtrHi::Register,             // 0x04
    pub prod_index: ProdIndex::Register,               // 0x08
    pub cons_index: ConsIndex::Register,               // 0x0C
    pub buf_size: BufSize::Register,                   // 0x10
    pub start_addr: StartAddr::Register,               // 0x14
    pub start_addr_hi: StartAddrHi::Register,          // 0x18
    pub end_addr: EndAddr::Register,                   // 0x1C
    pub end_addr_hi: EndAddrHi::Register,              // 0x20
    pub mbuf_done_thresh: MBufDoneThreshold::Register, // 0x24
    pub xon_xoff_thresh: XonXoffThresh::Register,      // 0x28
    pub read_ptr: WritePtr::Register,                  // 0x2C
    pub read_ptr_hi: WritePtrHi::Register,             // 0x30
    __reserved_0: [u32; 3],                            // 0x34
}

assert_eq_size!(RxRing, [u8; crate::genet::DMA_RING_SIZE]);
