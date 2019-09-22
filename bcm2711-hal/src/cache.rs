//! Cache utilities

pub mod bus_address_bits {
    pub const ALIAS_0_L1_L2: u32 = 0x0000_0000;
    pub const ALIAS_4_L2_COHERENT: u32 = 0x4000_0000;
    pub const ALIAS_8_L2: u32 = 0x8000_0000;
    pub const ALIAS_C_DIRECT: u32 = 0xC000_0000;
}

pub mod cpu_address_bits {
    pub const MASK: u32 = 0x3FFF_FFFF;
}
