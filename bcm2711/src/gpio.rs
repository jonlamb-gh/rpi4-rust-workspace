//! GPIO

use crate::MMIO_BASE;
use core::marker::PhantomData;
use core::ops::{Deref, DerefMut};
use register::{mmio::ReadWrite, register_bitfields};

pub const PADDR: usize = MMIO_BASE + 0x0020_0000;

register_bitfields! {
    u32,

    /// GPIO Function Select 0
    GPFSEL0 [
        /// Pin 9
        FSEL9 OFFSET(27) NUMBITS(3) [
            Input = 0b000,
            Output = 0b001,
            AF0 = 0b100, // SPI0 MISO - Alternate function 0
            AF1 = 0b101,
            AF2 = 0b110,
            AF3 = 0b111,
            AF4 = 0b011,
            AF5 = 0b010
        ],

        /// Pin 8
        FSEL8 OFFSET(24) NUMBITS(3) [
            Input = 0b000,
            Output = 0b001,
            AF0 = 0b100, // SPI0 chip select 0 - Alternate function 0
            AF1 = 0b101,
            AF2 = 0b110,
            AF3 = 0b111,
            AF4 = 0b011,
            AF5 = 0b010
        ],

        /// Pin 7
        FSEL7 OFFSET(21) NUMBITS(3) [
            Input = 0b000,
            Output = 0b001,
            AF0 = 0b100, // SPI0 chip select 1 - Alternate function 0
            AF1 = 0b101,
            AF2 = 0b110,
            AF3 = 0b111,
            AF4 = 0b011,
            AF5 = 0b010
        ],

        /// Pin 6
        FSEL6 OFFSET(18) NUMBITS(3) [
            Input = 0b000,
            Output = 0b001,
            AF0 = 0b100,
            AF1 = 0b101,
            AF2 = 0b110,
            AF3 = 0b111,
            AF4 = 0b011,
            AF5 = 0b010
        ],

        /// Pin 5
        FSEL5 OFFSET(15) NUMBITS(3) [
            Input = 0b000,
            Output = 0b001,
            AF0 = 0b100,
            AF1 = 0b101,
            AF2 = 0b110,
            AF3 = 0b111,
            AF4 = 0b011,
            AF5 = 0b010
        ],

        /// Pin 4
        FSEL4 OFFSET(12) NUMBITS(3) [
            Input = 0b000,
            Output = 0b001,
            AF0 = 0b100,
            AF1 = 0b101,
            AF2 = 0b110,
            AF3 = 0b111,
            AF4 = 0b011,
            AF5 = 0b010
        ],

        /// Pin 3
        FSEL3 OFFSET(9) NUMBITS(3) [
            Input = 0b000,
            Output = 0b001,
            AF0 = 0b100,
            AF1 = 0b101,
            AF2 = 0b110,
            AF3 = 0b111,
            AF4 = 0b011,
            AF5 = 0b010
        ],

        /// Pin 2
        FSEL2 OFFSET(6) NUMBITS(3) [
            Input = 0b000,
            Output = 0b001,
            AF0 = 0b100,
            AF1 = 0b101,
            AF2 = 0b110,
            AF3 = 0b111,
            AF4 = 0b011,
            AF5 = 0b010
        ],

        /// Pin 1
        FSEL1 OFFSET(3) NUMBITS(3) [
            Input = 0b000,
            Output = 0b001,
            AF0 = 0b100,
            AF1 = 0b101,
            AF2 = 0b110,
            AF3 = 0b111,
            AF4 = 0b011,
            AF5 = 0b010
        ],

        /// Pin 0
        FSEL0 OFFSET(0) NUMBITS(3) [
            Input = 0b000,
            Output = 0b001,
            AF0 = 0b100,
            AF1 = 0b101,
            AF2 = 0b110,
            AF3 = 0b111,
            AF4 = 0b011,
            AF5 = 0b010
        ]
    ],

    /// GPIO Function Select 1
    GPFSEL1 [
        /// Pin 19
        FSEL19 OFFSET(27) NUMBITS(3) [
            Input = 0b000,
            Output = 0b001,
            AF0 = 0b100,
            AF1 = 0b101,
            AF2 = 0b110,
            AF3 = 0b111,
            AF4 = 0b011,
            AF5 = 0b010
        ],

        /// Pin 17
        FSEL17 OFFSET(21) NUMBITS(3) [
            Input = 0b000,
            Output = 0b001,
            AF0 = 0b100,
            AF1 = 0b101,
            AF2 = 0b110,
            AF3 = 0b111,
            AF4 = 0b011,
            AF5 = 0b010
        ],

        /// Pin 15
        FSEL15 OFFSET(15) NUMBITS(3) [
            Input = 0b000,
            Output = 0b001,
            AF0 = 0b100, // UART0 - Alternate function 0
            AF1 = 0b101,
            AF2 = 0b110,
            AF3 = 0b111,
            AF4 = 0b011,
            AF5 = 0b010 // Mini UART - Alternate function 5
        ],

        /// Pin 14
        FSEL14 OFFSET(12) NUMBITS(3) [
            Input = 0b000,
            Output = 0b001,
            AF0 = 0b100, // UART0 - Alternate function 0
            AF1 = 0b101,
            AF2 = 0b110,
            AF3 = 0b111,
            AF4 = 0b011,
            AF5 = 0b010 // Mini UART - Alternate function 5
        ],

        /// Pin 13
        FSEL13 OFFSET(9) NUMBITS(3) [
            Input = 0b000,
            Output = 0b001,
            AF0 = 0b100,
            AF1 = 0b101,
            AF2 = 0b110,
            AF3 = 0b111,
            AF4 = 0b011,
            AF5 = 0b010
        ],

        /// Pin 11
        FSEL11 OFFSET(3) NUMBITS(3) [
            Input = 0b000,
            Output = 0b001,
            AF0 = 0b100, // SPI0 clock - Alternate function 0
            AF1 = 0b101,
            AF2 = 0b110,
            AF3 = 0b111,
            AF4 = 0b011,
            AF5 = 0b010
        ],

        /// Pin 10
        FSEL10 OFFSET(0) NUMBITS(3) [
            Input = 0b000,
            Output = 0b001,
            AF0 = 0b100, // SPI0 MOSI - Alternate function 0
            AF1 = 0b101,
            AF2 = 0b110,
            AF3 = 0b111,
            AF4 = 0b011,
            AF5 = 0b010
        ]
    ],

    /// GPIO Function Select 2
    GPFSEL2 [
        /// Pin 27
        FSEL27 OFFSET(21) NUMBITS(3) [
            Input = 0b000,
            Output = 0b001,
            AF0 = 0b100,
            AF1 = 0b101,
            AF2 = 0b110,
            AF3 = 0b111,
            AF4 = 0b011,
            AF5 = 0b010
        ],

        /// Pin 22
        FSEL22 OFFSET(6) NUMBITS(3) [
            Input = 0b000,
            Output = 0b001,
            AF0 = 0b100,
            AF1 = 0b101,
            AF2 = 0b110,
            AF3 = 0b111,
            AF4 = 0b011,
            AF5 = 0b010
        ],

        /// Pin 20
        FSEL20 OFFSET(0) NUMBITS(3) [
            Input = 0b000,
            Output = 0b001,
            AF0 = 0b100,
            AF1 = 0b101,
            AF2 = 0b110,
            AF3 = 0b111,
            AF4 = 0b011,
            AF5 = 0b010
        ]
    ],

    /// GPIO Pull-up/down Register
    GPPUD [
        /// GPIO Pin Pull-up/down
        PUD OFFSET(0) NUMBITS(2) [
            Off = 0b00,
            PullDown = 0b01,
            PullUp = 0b10
        ]
    ],

    /// GPIO Pull-up/down Clock Register 0
    GPPUDCLK0 [
        /// Pin 27
        PUDCLK27 OFFSET(27) NUMBITS(1) [
            NoEffect = 0,
            AssertClock = 1
        ],

        /// Pin 22
        PUDCLK22 OFFSET(22) NUMBITS(1) [
            NoEffect = 0,
            AssertClock = 1
        ],

        /// Pin 20
        PUDCLK20 OFFSET(20) NUMBITS(1) [
            NoEffect = 0,
            AssertClock = 1
        ],

        /// Pin 19
        PUDCLK19 OFFSET(19) NUMBITS(1) [
            NoEffect = 0,
            AssertClock = 1
        ],

        /// Pin 18
        PUDCLK18 OFFSET(18) NUMBITS(1) [
            NoEffect = 0,
            AssertClock = 1
        ],

        /// Pin 17
        PUDCLK17 OFFSET(17) NUMBITS(1) [
            NoEffect = 0,
            AssertClock = 1
        ],

        /// Pin 15
        PUDCLK15 OFFSET(15) NUMBITS(1) [
            NoEffect = 0,
            AssertClock = 1
        ],

        /// Pin 14
        PUDCLK14 OFFSET(14) NUMBITS(1) [
            NoEffect = 0,
            AssertClock = 1
        ],

        /// Pin 13
        PUDCLK13 OFFSET(13) NUMBITS(1) [
            NoEffect = 0,
            AssertClock = 1
        ],

        /// Pin 12
        PUDCLK12 OFFSET(12) NUMBITS(1) [
            NoEffect = 0,
            AssertClock = 1
        ],

        /// Pin 11
        PUDCLK11 OFFSET(11) NUMBITS(1) [
            NoEffect = 0,
            AssertClock = 1
        ],

        /// Pin 10
        PUDCLK10 OFFSET(10) NUMBITS(1) [
            NoEffect = 0,
            AssertClock = 1
        ],

        /// Pin 9
        PUDCLK9 OFFSET(9) NUMBITS(1) [
            NoEffect = 0,
            AssertClock = 1
        ],

        /// Pin 8
        PUDCLK8 OFFSET(8) NUMBITS(1) [
            NoEffect = 0,
            AssertClock = 1
        ],

        /// Pin 7
        PUDCLK7 OFFSET(7) NUMBITS(1) [
            NoEffect = 0,
            AssertClock = 1
        ],

        /// Pin 6
        PUDCLK6 OFFSET(6) NUMBITS(1) [
            NoEffect = 0,
            AssertClock = 1
        ],

        /// Pin 5
        PUDCLK5 OFFSET(5) NUMBITS(1) [
            NoEffect = 0,
            AssertClock = 1
        ],

        /// Pin 4
        PUDCLK4 OFFSET(4) NUMBITS(1) [
            NoEffect = 0,
            AssertClock = 1
        ],

        /// Pin 3
        PUDCLK3 OFFSET(3) NUMBITS(1) [
            NoEffect = 0,
            AssertClock = 1
        ],

        /// Pin 2
        PUDCLK2 OFFSET(2) NUMBITS(1) [
            NoEffect = 0,
            AssertClock = 1
        ],

        /// Pin 1
        PUDCLK1 OFFSET(1) NUMBITS(1) [
            NoEffect = 0,
            AssertClock = 1
        ],

        /// Pin 0
        PUDCLK0 OFFSET(0) NUMBITS(1) [
            NoEffect = 0,
            AssertClock = 1
        ]
    ]
}

#[allow(non_snake_case)]
#[repr(C)]
pub struct RegisterBlock {
    pub GPFSEL0: ReadWrite<u32, GPFSEL0::Register>, // 0x00
    pub GPFSEL1: ReadWrite<u32, GPFSEL1::Register>, // 0x04
    pub GPFSEL2: ReadWrite<u32, GPFSEL2::Register>, // 0x08
    __reserved_0: [u32; 4],                         // 0x0C
    pub GPSET0: ReadWrite<u32>,                     // 0x1C
    pub GPSET1: ReadWrite<u32>,                     // 0x20
    __reserved_1: u32,                              // 0x24
    pub GPCLR0: ReadWrite<u32>,                     // 0x28
    pub GPCLR1: ReadWrite<u32>,                     // 0x2C
    __reserved_2: u32,                              // 0x30
    pub GPLEV0: ReadWrite<u32>,                     // 0x34
    pub GPLEV1: ReadWrite<u32>,                     // 0x38
    __reserved_3: [u32; 22],                        // 0x3C
    pub GPPUD: ReadWrite<u32, GPPUD::Register>,     // 0x94
    pub GPPUDCLK0: ReadWrite<u32, GPPUDCLK0::Register>, //0x98
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
