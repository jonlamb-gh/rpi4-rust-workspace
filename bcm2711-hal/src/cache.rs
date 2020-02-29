//! Cache utilities
//!
//! Mostly copied from https://github.com/rsta2/circle/blob/master/lib/synchronize64.cpp

use cortex_a::barrier;

pub mod bus_address_bits {
    pub const ALIAS_0_L1_L2: u32 = 0x0000_0000;
    pub const ALIAS_4_L2_COHERENT: u32 = 0x4000_0000;
    pub const ALIAS_8_L2: u32 = 0x8000_0000;
    pub const ALIAS_C_DIRECT: u32 = 0xC000_0000;
}

pub mod cpu_address_bits {
    pub const MASK: u32 = 0x3FFF_FFFF;
}

const SETWAY_LEVEL_SHIFT: usize = 1;

const DATA_CACHE_LINE_LENGTH_MIN: usize = 64;

mod l1 {
    pub const DATA_CACHE_SETS: usize = 256;
    pub const DATA_CACHE_WAYS: usize = 2;
    pub const SETWAY_WAY_SHIFT: usize = 31;
    //pub const DATA_CACHE_LINE_LENGTH: usize = 64;
    pub const SETWAY_SET_SHIFT: usize = 6;
}

mod l2 {
    pub const CACHE_SETS: usize = 1024;
    pub const CACHE_WAYS: usize = 16;
    pub const SETWAY_WAY_SHIFT: usize = 28;
    //pub const CACHE_LINE_LENGTH: usize = 64;
    pub const SETWAY_SET_SHIFT: usize = 6;
}

#[naked]
pub unsafe fn invalidate_instruction_cache() {
    asm!("IC IALLU" : : : "memory" : "volatile")
}

#[naked]
pub unsafe fn invalidate_data_cache() {
    // Invalidate L1 data cache
    for set in 0..l1::DATA_CACHE_SETS {
        for way in 0..l1::DATA_CACHE_WAYS {
            let set_way_level: usize = (way << l1::SETWAY_WAY_SHIFT)
                | (set << l1::SETWAY_SET_SHIFT)
                | (0 << SETWAY_LEVEL_SHIFT);

            asm!("DC ISW, $0" : : "r" (set_way_level) : "memory" : "volatile");
        }
    }

    // Invalidate L2 unified cache
    for set in 0..l2::CACHE_SETS {
        for way in 0..l2::CACHE_WAYS {
            let set_way_level = (way << l2::SETWAY_WAY_SHIFT)
                | (set << l2::SETWAY_SET_SHIFT)
                | (1 << SETWAY_LEVEL_SHIFT);

            asm!("DC ISW, $0" : : "r" (set_way_level) : "memory" : "volatile");
        }
    }

    barrier::dsb(barrier::SY);
}

#[naked]
pub unsafe fn invalidate_data_cache_l1_only() {
    for set in 0..l1::DATA_CACHE_SETS {
        for way in 0..l1::DATA_CACHE_WAYS {
            let set_way_level: usize = (way << l1::SETWAY_WAY_SHIFT)
                | (set << l1::SETWAY_SET_SHIFT)
                | (0 << SETWAY_LEVEL_SHIFT);

            asm!("DC ISW, $0" : : "r" (set_way_level) : "memory" : "volatile");
        }
    }

    barrier::dsb(barrier::SY);
}

#[naked]
pub unsafe fn clean_data_cache() {
    // Clean L1 data cache
    for set in 0..l1::DATA_CACHE_SETS {
        for way in 0..l1::DATA_CACHE_WAYS {
            let set_way_level: usize = (way << l1::SETWAY_WAY_SHIFT)
                | (set << l1::SETWAY_SET_SHIFT)
                | (0 << SETWAY_LEVEL_SHIFT);

            asm!("DC CSW, $0" : : "r" (set_way_level) : "memory" : "volatile");
        }
    }

    // Clean L2 unified cache
    for set in 0..l2::CACHE_SETS {
        for way in 0..l2::CACHE_WAYS {
            let set_way_level = (way << l2::SETWAY_WAY_SHIFT)
                | (set << l2::SETWAY_SET_SHIFT)
                | (1 << SETWAY_LEVEL_SHIFT);

            asm!("DC CSW, $0" : : "r" (set_way_level) : "memory" : "volatile");
        }
    }

    barrier::dsb(barrier::SY);
}

#[naked]
pub unsafe fn invalidate_data_cache_range(mut address: usize, mut length: usize) {
    length += DATA_CACHE_LINE_LENGTH_MIN;

    loop {
        asm!("DC IVAC, $0" : : "r" (address) : "memory" : "volatile");

        if length < DATA_CACHE_LINE_LENGTH_MIN {
            break;
        }

        address += DATA_CACHE_LINE_LENGTH_MIN;
        length -= DATA_CACHE_LINE_LENGTH_MIN;
    }

    barrier::dsb(barrier::SY);
}

#[naked]
pub unsafe fn clean_data_cache_range(mut address: usize, mut length: usize) {
    length += DATA_CACHE_LINE_LENGTH_MIN;

    loop {
        asm!("DC CVAC, $0" : : "r" (address) : "memory" : "volatile");

        if length < DATA_CACHE_LINE_LENGTH_MIN {
            break;
        }

        address += DATA_CACHE_LINE_LENGTH_MIN;
        length -= DATA_CACHE_LINE_LENGTH_MIN;
    }

    barrier::dsb(barrier::SY);
}

#[naked]
pub unsafe fn clean_and_invalidate_data_cache_range(mut address: usize, mut length: usize) {
    length += DATA_CACHE_LINE_LENGTH_MIN;

    loop {
        asm!("DC CIVAC, $0" : : "r" (address) : "memory" : "volatile");

        if length < DATA_CACHE_LINE_LENGTH_MIN {
            break;
        }

        address += DATA_CACHE_LINE_LENGTH_MIN;
        length -= DATA_CACHE_LINE_LENGTH_MIN;
    }

    barrier::dsb(barrier::SY);
}

#[naked]
pub unsafe fn sync_data_and_instruction_cache() {
    clean_data_cache();

    invalidate_instruction_cache();
    barrier::dsb(barrier::SY);

    barrier::isb(barrier::SY);
}
