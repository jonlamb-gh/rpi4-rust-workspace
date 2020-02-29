//! DMA control block

use crate::cache::bus_address_bits;
use bitfield::bitfield;
use core::fmt;
use static_assertions::{assert_eq_size, const_assert_eq};

pub const CONTROL_BLOCK_SIZE: usize = 8 * 4;

pub const TRANSFER_LENGTH_MAX: u32 = 0x3FFF_FFFF;

pub const TRANSFER_LENGTH_MAX_LITE: u32 = 0xFFFF;

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum TransferLength {
    ModeLinear(u32),
    Mode2D(u16, u16),
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum TransferWidth {
    Bits32,
    Bits128,
}

/// 8 words (256 bits or 32 bytes) in length and must start at a 256-bit aligned
/// address
#[derive(Debug)]
#[repr(C, align(32))]
pub struct ControlBlock {
    /// Transfer info
    pub info: TxfrInfoWord,
    /// Source address
    pub src: u32,
    /// Destination address
    pub dest: u32,
    /// Transfer length
    pub length: TxfrLenWord,
    /// 2D stride
    pub stride: StrideWord,
    /// Next control block address
    pub next: u32,
    #[doc(hidden)]
    _reserved_0: u32,
    _reserved_1: u32,
}

assert_eq_size!(ControlBlock, [u32; 8]);
const_assert_eq!(CONTROL_BLOCK_SIZE, 32);

impl fmt::Display for ControlBlock {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "DCB {{ info: 0x{:X}, src: 0x{:X}, dest: 0x{:X}, length: 0x{:X}, stride: 0x{:X}, next: 0x{:X} }}",
            self.info.0,
            self.src,
            self.dest,
            self.length.0,
            self.stride.0,
            self.next,
        )
    }
}

impl ControlBlock {
    #[inline]
    pub fn as_ptr(&self) -> *const Self {
        self as *const _
    }

    #[inline]
    pub fn as_paddr(&self) -> usize {
        self.as_ptr() as usize
    }
}

bitfield! {
    #[repr(C)]
    pub struct TxfrInfoWord(u32);
    impl Debug;
    u32;
    /// Interrupt enable
    pub int_en, set_int_en : 0;
    /// 2D mode
    pub td_mode, set_td_mode : 1;
    /// Wait for write response
    pub wait_resp, set_wait_resp : 3;
    /// Destination address increment
    pub dest_inc, set_dest_inc : 4;
    /// Destination transfer width
    /// 0 = Use 32-bit destination write width
    /// 1 = Use 128-bit destination write width
    pub dest_width, set_dest_width : 5;
    /// Control destination writes with DREQ
    pub dest_dreq, set_dest_dreq : 6;
    /// Ignore destination writes
    pub dest_ignore, set_dest_ignore : 7;
    /// Source address increment
    pub src_inc, set_src_inc : 8;
    /// Source transfer width
    /// 0 = Use 32-bit source read width
    /// 1 = Use 128-bit source read width
    pub src_width, set_src_width : 9;
    /// Control source reads with DREQ
    pub src_dreq, set_src_dreq : 10;
    /// Ignore source reads
    pub src_ignore, set_src_ignore : 11;
    /// Burst transfer length
    /// A value of zero will produce a single tranfer.
    pub burst_len, set_burst_len : 15, 12;
    /// Peripheral mapping
    pub periph_map, set_periph_map : 20, 16;
    /// Add wait cycles
    pub waits, set_waits : 25, 21;
    /// Don't do wide writes as a 2 beat burst
    pub no_wide_bursts, set_no_wide_bursts : 26;
}

bitfield! {
    #[repr(C)]
    pub struct TxfrLenWord(u32);
    impl Debug;
    u32;
    /// Transfer length in bytes
    pub xlen, set_xlen : 15, 0;
    /// When in 2D mode, This is the Y transfer length,
    /// indicating how many xlength transfers are performed.
    /// When in normal linear mode this becomes the top bits
    /// of the XLENGTH.
    pub ylen, set_ylen : 29, 16;
}

bitfield! {
    #[repr(C)]
    pub struct StrideWord(u32);
    impl Debug;
    u32;
    /// Source stride (2D mode)
    pub src_stride, set_src_stride : 15, 0;
    /// Destination stride (2D mode)
    pub dest_stride, set_dest_stride : 31, 16;
}

impl Default for ControlBlock {
    fn default() -> Self {
        ControlBlock::new()
    }
}

impl ControlBlock {
    pub const fn new() -> Self {
        ControlBlock {
            info: TxfrInfoWord(0),
            src: 0,
            dest: 0,
            length: TxfrLenWord(0),
            stride: StrideWord(0),
            next: 0,
            _reserved_0: 0,
            _reserved_1: 0,
        }
    }

    pub fn init(&mut self) {
        self.info = TxfrInfoWord(0);
        self.src = 0;
        self.dest = 0;
        self.length = TxfrLenWord(0);
        self.stride = StrideWord(0);
        self.next = 0;
        self._reserved_0 = 0;
        self._reserved_1 = 0;
    }

    /// NOTE: the physical addresses will be translated to a bus address for
    /// the DMA engine
    pub fn set_src(&mut self, src: u32) {
        self.src = src | bus_address_bits::ALIAS_4_L2_COHERENT;
    }

    /// NOTE: the physical addresses will be translated to a bus address for
    /// the DMA engine
    pub fn set_dest(&mut self, dest: u32) {
        self.dest = dest | bus_address_bits::ALIAS_4_L2_COHERENT;
    }

    pub fn set_length(&mut self, length: TransferLength) {
        match length {
            TransferLength::ModeLinear(l) => self.length.0 = l,
            TransferLength::Mode2D(x, y) => {
                self.length.set_xlen(x.into());
                self.length.set_ylen(y.into());
                self.info.set_td_mode(true);
            }
        }
    }

    pub fn src_width(&self) -> TransferWidth {
        if self.info.src_width() == false {
            TransferWidth::Bits32
        } else {
            TransferWidth::Bits128
        }
    }

    pub fn set_src_width(&mut self, width: TransferWidth) {
        self.info.set_src_width(width.into());
    }

    pub fn dest_width(&self) -> TransferWidth {
        if self.info.dest_width() == false {
            TransferWidth::Bits32
        } else {
            TransferWidth::Bits128
        }
    }

    pub fn set_dest_width(&mut self, width: TransferWidth) {
        self.info.set_dest_width(width.into());
    }

    pub fn set_next(&mut self, next_dcb_paddr: u32) {
        self.next = next_dcb_paddr | bus_address_bits::ALIAS_4_L2_COHERENT;
    }
}

impl From<TransferWidth> for bool {
    fn from(w: TransferWidth) -> bool {
        match w {
            TransferWidth::Bits32 => false,
            TransferWidth::Bits128 => true,
        }
    }
}
