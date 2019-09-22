//! DMA

use crate::MMIO_BASE;
use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};
use register::{mmio::ReadOnly, mmio::ReadWrite, register_bitfields};

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

register_bitfields! {
    u32,

    /// Control and status
    CS [
        ACTIVE OFFSET(0) NUMBITS(1) [],
        END OFFSET(1) NUMBITS(1) [],
        INT OFFSET(2) NUMBITS(1) [],
        DREQ OFFSET(3) NUMBITS(1) [],
        PAUSED OFFSET(4) NUMBITS(1) [],
        DREQ_STOPS_DMA OFFSET(5) NUMBITS(1) [],
        WAITING_FOR_OUTSTANDING_WRITES OFFSET(6) NUMBITS(1) [],
        ERROR OFFSET(8) NUMBITS(1) [],
        PRIORITY OFFSET(16) NUMBITS(4) [],
        PANIC_PRIORITY OFFSET(20) NUMBITS(4) [],
        WAIT_FOR_OUTSTANDING_WRITES OFFSET(28) NUMBITS(1) [],
        DISDEBUG OFFSET(29) NUMBITS(1) [],
        ABORT OFFSET(30) NUMBITS(1) [],
        RESET OFFSET(31) NUMBITS(1) []
    ],

    /// Transfer information
    TI [
        INTEN OFFSET(0) NUMBITS(1) [],
        TDMODE OFFSET(1) NUMBITS(1) [],
        WAIT_RESP OFFSET(3) NUMBITS(1) [],
        DEST_INC OFFSET(4) NUMBITS(1) [],
        DEST_WIDTH OFFSET(5) NUMBITS(1) [],
        DEST_DREQ OFFSET(6) NUMBITS(1) [],
        DEST_IGNORE OFFSET(7) NUMBITS(1) [],
        SRC_INC OFFSET(8) NUMBITS(1) [],
        SRC_WIDTH OFFSET(9) NUMBITS(1) [],
        SRC_DREQ OFFSET(10) NUMBITS(1) [],
        SRC_IGNORE OFFSET(11) NUMBITS(1) [],
        BURST_LENGTH OFFSET(12) NUMBITS(4) [],
        PERMAP OFFSET(16) NUMBITS(5) [],
        WAITS OFFSET(21) NUMBITS(5) [],
        NO_WIDE_BURSTS OFFSET(26) NUMBITS(1) []
    ],

    /// Transfer length
    TXFR_LEN [
        XLENGTH OFFSET(0) NUMBITS(16) [],
        YLENGTH OFFSET(16) NUMBITS(14) []
    ],

    /// 2D stride
    STRIDE [
        S_STRIDE OFFSET(0) NUMBITS(16) [],
        D_STRIDE OFFSET(16) NUMBITS(16) []
    ],

    /// Debug
    DEBUG [
        READ_LAST_NOT_SET_ERROR OFFSET(0) NUMBITS(1) [],
        FIFO_ERROR OFFSET(1) NUMBITS(1) [],
        READ_ERROR OFFSET(2) NUMBITS(1) [],
        OUTSTANDING_WRITES OFFSET(4) NUMBITS(4) [],
        DMA_ID OFFSET(8) NUMBITS(8) [],
        DMA_STATE OFFSET(16) NUMBITS(8) [],
        VERSION OFFSET(25) NUMBITS(3) [],
        LITE OFFSET(28) NUMBITS(1) []
    ],

    /// Interrupt status
    INT_STATUS [
        INT0 OFFSET(0) NUMBITS(1) [],
        INT1 OFFSET(1) NUMBITS(1) [],
        INT2 OFFSET(2) NUMBITS(1) [],
        INT3 OFFSET(3) NUMBITS(1) [],
        INT4 OFFSET(4) NUMBITS(1) [],
        INT5 OFFSET(5) NUMBITS(1) [],
        INT6 OFFSET(6) NUMBITS(1) [],
        INT7 OFFSET(7) NUMBITS(1) [],
        INT8 OFFSET(8) NUMBITS(1) [],
        INT9 OFFSET(9) NUMBITS(1) [],
        INT10 OFFSET(10) NUMBITS(1) [],
        INT11 OFFSET(11) NUMBITS(1) [],
        INT12 OFFSET(12) NUMBITS(1) [],
        INT13 OFFSET(13) NUMBITS(1) [],
        INT14 OFFSET(14) NUMBITS(1) [],
        INT15 OFFSET(15) NUMBITS(1) []
    ],

    /// Global enable bits
    ENABLE [
        EN0 OFFSET(0) NUMBITS(1) [],
        EN1 OFFSET(1) NUMBITS(1) [],
        EN2 OFFSET(2) NUMBITS(1) [],
        EN3 OFFSET(3) NUMBITS(1) [],
        EN4 OFFSET(4) NUMBITS(1) [],
        EN5 OFFSET(5) NUMBITS(1) [],
        EN6 OFFSET(6) NUMBITS(1) [],
        EN7 OFFSET(7) NUMBITS(1) [],
        EN8 OFFSET(8) NUMBITS(1) [],
        EN9 OFFSET(9) NUMBITS(1) [],
        EN10 OFFSET(10) NUMBITS(1) [],
        EN11 OFFSET(11) NUMBITS(1) [],
        EN12 OFFSET(12) NUMBITS(1) [],
        EN13 OFFSET(13) NUMBITS(1) [],
        EN14 OFFSET(14) NUMBITS(1) [],
        EN15 OFFSET(15) NUMBITS(1) []
    ]
}

#[allow(non_snake_case)]
#[repr(C)]
pub struct GlobalIntStatusRegisterBlock {
    pub INT_STATUS: ReadWrite<u32, INT_STATUS::Register>,
}

#[allow(non_snake_case)]
#[repr(C)]
pub struct GlobalEnableRegisterBlock {
    pub ENABLE: ReadWrite<u32, ENABLE::Register>,
}

#[allow(non_snake_case)]
#[repr(C)]
pub struct RegisterBlock {
    pub CS: ReadWrite<u32, CS::Register>,            // 0x00
    pub CONBLK_AD: ReadWrite<u32>,                   // 0x04
    pub TI: ReadOnly<u32, TI::Register>,             // 0x08
    pub SOURCE_AD: ReadOnly<u32>,                    // 0x0C
    pub DEST_AD: ReadOnly<u32>,                      // 0x10
    pub TXFR_LEN: ReadOnly<u32, TXFR_LEN::Register>, // 0x14
    pub STRIDE: ReadOnly<u32, STRIDE::Register>,     // 0x18
    pub NEXTCONBK: ReadOnly<u32>,                    // 0x1C
    pub DEBUG: ReadOnly<u32, DEBUG::Register>,       // 0x20
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
