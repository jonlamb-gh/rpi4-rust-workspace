//! GPIO

use crate::MMIO_BASE;
use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};

pub const PADDR: usize = MMIO_BASE + 0x0020_0000;

register! {
    /// GPIO Function Select 0
    FunSel0,
    u32,
    RW,
    Fields [
        Pin0 WIDTH(U3) OFFSET(U0) [
            Input = U0,
            Output = U1,
            /// SDA0
            AF0 = U4,
            /// SA5
            AF1 = U5,
            /// PCLK
            AF2 = U6,
            /// SPI3_CE0_N
            AF3 = U7,
            /// TXD2
            AF4 = U3,
            /// SDA6
            AF5 = U2
        ],
        Pin1 WIDTH(U3) OFFSET(U3) [
            Input = U0,
            Output = U1,
            /// SCL0
            AF0 = U4,
            /// SA4
            AF1 = U5,
            /// DE
            AF2 = U6,
            /// SPI3_MISO
            AF3 = U7,
            /// RXD2
            AF4 = U3,
            /// SCL6
            AF5 = U2
        ],
        Pin2 WIDTH(U3) OFFSET(U6) [
            Input = U0,
            Output = U1,
            /// SDA1
            AF0 = U4,
            /// SA3
            AF1 = U5,
            /// LCD_VSYNC
            AF2 = U6,
            /// SPI3_MOSI
            AF3 = U7,
            /// CTS2
            AF4 = U3,
            /// SDA3
            AF5 = U2
        ],
        Pin3 WIDTH(U3) OFFSET(U9) [
            Input = U0,
            Output = U1,
            /// SCL1
            AF0 = U4,
            /// SA2
            AF1 = U5,
            /// LCD_HSYNC
            AF2 = U6,
            /// SPI3_SCLK
            AF3 = U7,
            /// RTS2
            AF4 = U3,
            /// SCL3
            AF5 = U2
        ],
        Pin4 WIDTH(U3) OFFSET(U12) [
            Input = U0,
            Output = U1,
            /// GPCLK0
            AF0 = U4,
            /// SA1
            AF1 = U5,
            /// DPI_D0
            AF2 = U6,
            /// SPI4_CE0_N
            AF3 = U7,
            /// TXD3
            AF4 = U3,
            /// SDA3
            AF5 = U2
        ],
        Pin5 WIDTH(U3) OFFSET(U15) [
            Input = U0,
            Output = U1,
            /// GPCLK1
            AF0 = U4,
            /// SA0
            AF1 = U5,
            /// DPI_D1
            AF2 = U6,
            /// SPI4_MISO
            AF3 = U7,
            /// RXD3
            AF4 = U3,
            /// SCL3
            AF5 = U2
        ],
        Pin6 WIDTH(U3) OFFSET(U18) [
            Input = U0,
            Output = U1,
            /// GPCLK2
            AF0 = U4,
            /// SOE_N
            AF1 = U5,
            /// DPI_D3
            AF2 = U6,
            /// SPI4_MOSI
            AF3 = U7,
            /// CTS3
            AF4 = U3,
            /// SDA4
            AF5 = U2
        ],
        Pin7 WIDTH(U3) OFFSET(U21) [
            Input = U0,
            Output = U1,
            /// SPI0_CE1_N
            AF0 = U4,
            /// SWE_N
            AF1 = U5,
            /// DPI_D3
            AF2 = U6,
            /// SPI4_SCLK
            AF3 = U7,
            /// RTS3
            AF4 = U3,
            /// SCL4
            AF5 = U2
        ],
        Pin8 WIDTH(U3) OFFSET(U24) [
            Input = U0,
            Output = U1,
            /// SPI0_CE0_N
            AF0 = U4,
            /// SD0
            AF1 = U5,
            /// DPI_D4
            AF2 = U6,
            AF3 = U7,
            /// TXD4
            AF4 = U3,
            /// SDA4
            AF5 = U2
        ],
        Pin9 WIDTH(U3) OFFSET(U27) [
            Input = U0,
            Output = U1,
            /// SPI0_MISO
            AF0 = U4,
            /// SD1
            AF1 = U5,
            /// DPI_D5
            AF2 = U6,
            AF3 = U7,
            /// RXD4
            AF4 = U3,
            /// SCL4
            AF5 = U2
        ],
    ]
}

register! {
    /// GPIO Function Select 1
    FunSel1,
    u32,
    RW,
    Fields [
        Pin10 WIDTH(U3) OFFSET(U0) [
            Input = U0,
            Output = U1,
            /// SPI0_MOSI
            AF0 = U4,
            /// SD2
            AF1 = U5,
            /// DPI_D6
            AF2 = U6,
            AF3 = U7,
            /// CTS4
            AF4 = U3,
            /// SDA5
            AF5 = U2
        ],
        Pin11 WIDTH(U3) OFFSET(U3) [
            Input = U0,
            Output = U1,
            /// SPI0_SCLK
            AF0 = U4,
            /// SD3
            AF1 = U5,
            /// DPI_D7
            AF2 = U6,
            AF3 = U7,
            /// RTS4
            AF4 = U3,
            /// SCL5
            AF5 = U2
        ],
        Pin12 WIDTH(U3) OFFSET(U6) [
            Input = U0,
            Output = U1,
            /// PWM0
            AF0 = U4,
            /// SD4
            AF1 = U5,
            /// DPI_D8
            AF2 = U6,
            /// SPI5_CE0_N
            AF3 = U7,
            /// TXD5
            AF4 = U3,
            /// SDA5
            AF5 = U2
        ],
        Pin13 WIDTH(U3) OFFSET(U9) [
            Input = U0,
            Output = U1,
            /// PWM1
            AF0 = U4,
            /// SD5
            AF1 = U5,
            /// DPI_D9
            AF2 = U6,
            /// SPI5_MISO
            AF3 = U7,
            /// RXD5
            AF4 = U3,
            /// SCL5
            AF5 = U2
        ],
        Pin14 WIDTH(U3) OFFSET(U12) [
            Input = U0,
            Output = U1,
            /// TXD0
            AF0 = U4,
            /// SD6
            AF1 = U5,
            /// DPI_D10
            AF2 = U6,
            /// SPI5_MOSI
            AF3 = U7,
            /// CTS5
            AF4 = U3,
            /// TXD1
            AF5 = U2
        ],
        Pin15 WIDTH(U3) OFFSET(U15) [
            Input = U0,
            Output = U1,
            /// RXD0
            AF0 = U4,
            /// SD7
            AF1 = U5,
            /// DPI_D11
            AF2 = U6,
            /// SPI5_SCLK
            AF3 = U7,
            /// RTS5
            AF4 = U3,
            /// RXD1
            AF5 = U2
        ],
        Pin16 WIDTH(U3) OFFSET(U18) [
            Input = U0,
            Output = U1,
            /// FL0
            AF0 = U4,
            /// SD8
            AF1 = U5,
            /// DPI_D12
            AF2 = U6,
            /// CTS0
            AF3 = U7,
            /// SPI1_CE2_N
            AF4 = U3,
            /// CTS1
            AF5 = U2
        ],
        Pin17 WIDTH(U3) OFFSET(U21) [
            Input = U0,
            Output = U1,
            /// FL1
            AF0 = U4,
            /// SD9
            AF1 = U5,
            /// DPI_D13
            AF2 = U6,
            /// RTS0
            AF3 = U7,
            /// SPI1_CE1_N
            AF4 = U3,
            /// RTS1
            AF5 = U2
        ],
        Pin18 WIDTH(U3) OFFSET(U24) [
            Input = U0,
            Output = U1,
            /// PCM_CLK
            AF0 = U4,
            /// SD10
            AF1 = U5,
            /// DPI_D14
            AF2 = U6,
            /// SPI6_CE0_N
            AF3 = U7,
            /// SPI1_CE0_N
            AF4 = U3,
            /// PWM0
            AF5 = U2
        ],
        Pin19 WIDTH(U3) OFFSET(U27) [
            Input = U0,
            Output = U1,
            /// PCM_FS
            AF0 = U4,
            /// SD11
            AF1 = U5,
            /// DPI_D15
            AF2 = U6,
            /// SPI6_MISO
            AF3 = U7,
            /// SPI1_MISO
            AF4 = U3,
            /// PWM1
            AF5 = U2
        ],
    ]
}

register! {
    /// GPIO Function Select 2
    FunSel2,
    u32,
    RW,
    Fields [
        Pin20 WIDTH(U3) OFFSET(U0) [
            Input = U0,
            Output = U1,
            /// PCM_DIN
            AF0 = U4,
            /// SD12
            AF1 = U5,
            /// DPI_D16
            AF2 = U6,
            /// SPI6_MOSI
            AF3 = U7,
            /// SPI1_MOSI
            AF4 = U3,
            /// GPCLK0
            AF5 = U2
        ],
        Pin21 WIDTH(U3) OFFSET(U3) [
            Input = U0,
            Output = U1,
            /// PCM_DOUT
            AF0 = U4,
            /// SD13
            AF1 = U5,
            /// DPI_D17
            AF2 = U6,
            /// SPI6_SCLK
            AF3 = U7,
            /// SPI1_SCLK
            AF4 = U3,
            /// GPCLK1
            AF5 = U2
        ],
        Pin22 WIDTH(U3) OFFSET(U6) [
            Input = U0,
            Output = U1,
            /// SD0_CLK
            AF0 = U4,
            /// SD14
            AF1 = U5,
            /// DPI_D18
            AF2 = U6,
            /// SD1_CLK
            AF3 = U7,
            /// ARM_TRST
            AF4 = U3,
            /// SDA6
            AF5 = U2
        ],
        Pin23 WIDTH(U3) OFFSET(U9) [
            Input = U0,
            Output = U1,
            /// SD0_CMD
            AF0 = U4,
            /// SD15
            AF1 = U5,
            /// DPI_D19
            AF2 = U6,
            /// SD1_CMD
            AF3 = U7,
            /// ARM_RTCK
            AF4 = U3,
            /// SCL6
            AF5 = U2
        ],
        Pin24 WIDTH(U3) OFFSET(U12) [
            Input = U0,
            Output = U1,
            /// SD0_DAT0
            AF0 = U4,
            /// SD16
            AF1 = U5,
            /// DPI_D20
            AF2 = U6,
            /// SD1_DAT0
            AF3 = U7,
            /// ARM_TDO
            AF4 = U3,
            /// SPI3_CE1_N
            AF5 = U2
        ],
        Pin25 WIDTH(U3) OFFSET(U15) [
            Input = U0,
            Output = U1,
            /// SD0_DAT1
            AF0 = U4,
            /// SD17
            AF1 = U5,
            /// DPI_D21
            AF2 = U6,
            /// SD1_DAT1
            AF3 = U7,
            /// ARM_TCK
            AF4 = U3,
            /// SPI4_CE1_N
            AF5 = U2
        ],
        Pin26 WIDTH(U3) OFFSET(U18) [
            Input = U0,
            Output = U1,
            /// SD0_DAT2
            AF0 = U4,
            /// TE0
            AF1 = U5,
            /// DPI_D22
            AF2 = U6,
            /// SD1_DAT2
            AF3 = U7,
            /// ARM_TDI
            AF4 = U3,
            /// SPI5_CE1_N
            AF5 = U2
        ],
        Pin27 WIDTH(U3) OFFSET(U21) [
            Input = U0,
            Output = U1,
            /// SD0_DAT3
            AF0 = U4,
            /// TE1
            AF1 = U5,
            /// DPI_D23
            AF2 = U6,
            /// SD1_DAT3
            AF3 = U7,
            /// ARM_TMS
            AF4 = U3,
            /// SPI6_CE1_N
            AF5 = U2
        ],
        Pin28 WIDTH(U3) OFFSET(U24) [
            Input = U0,
            Output = U1,
            AF0 = U4,
            AF1 = U5,
            AF2 = U6,
            AF3 = U7,
            AF4 = U3,
            AF5 = U2
        ],
        Pin29 WIDTH(U3) OFFSET(U27) [
            Input = U0,
            Output = U1,
            AF0 = U4,
            AF1 = U5,
            AF2 = U6,
            AF3 = U7,
            AF4 = U3,
            AF5 = U2
        ],
    ]
}

register! {
    /// GPIO Output Set Register 0
    Set0,
    u32,
    RW,
    Fields [
        /// Pins 0:31
        Pins WIDTH(U32) OFFSET(U0) [],
    ]
}

register! {
    /// GPIO Output Set Register 1
    Set1,
    u32,
    RW,
    Fields [
        /// Pins 32:53
        Pins WIDTH(U32) OFFSET(U0) [],
    ]
}

register! {
    /// GPIO Output Clear Register 0
    Clear0,
    u32,
    RW,
    Fields [
        /// Pins 0:31
        Pins WIDTH(U32) OFFSET(U0) [],
    ]
}

register! {
    /// GPIO Output Clear Register 1
    Clear1,
    u32,
    RW,
    Fields [
        /// Pins 32:53
        Pins WIDTH(U32) OFFSET(U0) [],
    ]
}

register! {
    /// GPIO Pin Level Register 0
    PinLevel0,
    u32,
    RO,
    Fields [
        /// Pins 0:31
        Pins WIDTH(U32) OFFSET(U0) [],
    ]
}

register! {
    /// GPIO Pin Level Register 1
    PinLevel1,
    u32,
    RO,
    Fields [
        /// Pins 32:53
        Pins WIDTH(U32) OFFSET(U0) [],
    ]
}

register! {
    /// GPIO Pull-up/down Register
    PullUpDown,
    u32,
    RW,
    Fields [
        PuD WIDTH(U2) OFFSET(U0) [
            Off = U0,
            PullDown = U1,
            PullUp = U2
        ],
    ]
}

register! {
    /// GPIO Pull-up/down Clock Register 0
    PullUpDownClock0,
    u32,
    RW,
    Fields [
        Pin0 WIDTH(U1) OFFSET(U0) [],
        Pin1 WIDTH(U1) OFFSET(U1) [],
        Pin2 WIDTH(U1) OFFSET(U2) [],
        Pin3 WIDTH(U1) OFFSET(U3) [],
        Pin4 WIDTH(U1) OFFSET(U4) [],
        Pin5 WIDTH(U1) OFFSET(U5) [],
        Pin6 WIDTH(U1) OFFSET(U6) [],
        Pin7 WIDTH(U1) OFFSET(U7) [],
        Pin8 WIDTH(U1) OFFSET(U8) [],
        Pin9 WIDTH(U1) OFFSET(U9) [],
        Pin10 WIDTH(U1) OFFSET(U10) [],
        Pin11 WIDTH(U1) OFFSET(U11) [],
        Pin12 WIDTH(U1) OFFSET(U12) [],
        Pin13 WIDTH(U1) OFFSET(U13) [],
        Pin14 WIDTH(U1) OFFSET(U14) [],
        Pin15 WIDTH(U1) OFFSET(U15) [],
        Pin16 WIDTH(U1) OFFSET(U16) [],
        Pin17 WIDTH(U1) OFFSET(U17) [],
        Pin18 WIDTH(U1) OFFSET(U18) [],
        Pin19 WIDTH(U1) OFFSET(U19) [],
        Pin20 WIDTH(U1) OFFSET(U20) [],
        Pin21 WIDTH(U1) OFFSET(U21) [],
        Pin22 WIDTH(U1) OFFSET(U22) [],
        Pin23 WIDTH(U1) OFFSET(U23) [],
        Pin24 WIDTH(U1) OFFSET(U24) [],
        Pin25 WIDTH(U1) OFFSET(U25) [],
        Pin26 WIDTH(U1) OFFSET(U26) [],
        Pin27 WIDTH(U1) OFFSET(U27) [],
    ]
}

#[repr(C)]
pub struct RegisterBlock {
    pub fun_sel0: FunSel0::Register,          // 0x00
    pub fun_sel1: FunSel1::Register,          // 0x04
    pub fun_sel2: FunSel2::Register,          // 0x08
    __reserved_0: [u32; 4],                   // 0x0C
    pub set0: Set0::Register,                 // 0x1C
    pub set1: Set1::Register,                 // 0x20
    __reserved_1: u32,                        // 0x24
    pub clr0: Clear0::Register,               // 0x28
    pub clr1: Clear1::Register,               // 0x2C
    __reserved_2: u32,                        // 0x30
    pub level0: PinLevel0::Register,          // 0x34
    pub level1: PinLevel1::Register,          // 0x38
    __reserved_3: [u32; 22],                  // 0x3C
    pub pud: PullUpDown::Register,            // 0x94
    pub pud_clk0: PullUpDownClock0::Register, // 0x98
}

pub struct GPIO {
    _marker: PhantomData<*const ()>,
}

unsafe impl Send for GPIO {}

impl GPIO {
    pub fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }

    pub fn as_ptr(&self) -> *const RegisterBlock {
        PADDR as *const _
    }

    pub fn as_mut_ptr(&mut self) -> *mut RegisterBlock {
        PADDR as *mut _
    }
}

impl Deref for GPIO {
    type Target = RegisterBlock;
    fn deref(&self) -> &RegisterBlock {
        unsafe { &*self.as_ptr() }
    }
}

impl DerefMut for GPIO {
    fn deref_mut(&mut self) -> &mut RegisterBlock {
        unsafe { &mut *self.as_mut_ptr() }
    }
}
