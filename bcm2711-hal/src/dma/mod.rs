//! DMA

// TODO
// - generate all the channels via macros
// - following https://github.com/rust-embedded/embedded-hal/issues/37#issuecomment-377823801
// - https://github.com/stm32-rs/stm32f7xx-hal/blob/master/src/dma.rs
// - https://github.com/stm32-rs/stm32l0xx-hal/pull/14
// - fix the sync/fences/barriers
// - allow for cached/uncached in transfer config, enable device/IO mem
// - assumes we're using cached memory, mem2mem style

use crate::cache::{self, bus_address_bits};
use bcm2711::dma::*;
use core::mem;
use core::sync::atomic::{compiler_fence, Ordering};
use cortex_a::{asm, barrier};

mod control_block;

pub use crate::dma::control_block::{
    ControlBlock, StrideWord, TransferLength, TransferWidth, TxfrInfoWord, TxfrLenWord,
    CONTROL_BLOCK_SIZE, TRANSFER_LENGTH_MAX, TRANSFER_LENGTH_MAX_LITE,
};

pub trait DmaExt {
    type Parts;

    fn split(self) -> Self::Parts;
}

pub struct Parts {
    pub ch0: Channel,
    pub ch1: Channel,
    // TODO 2..15
    pub int_status: IntStatusRegister,
    pub enable: EnableRegister,
}

impl DmaExt for DMA {
    type Parts = Parts;

    fn split(self) -> Self::Parts {
        Parts {
            ch0: Channel {
                dma: DMA::new().as_channel(CHANNEL0_OFFSET),
            },
            ch1: Channel {
                dma: DMA::new().as_channel(CHANNEL1_OFFSET),
            },
            int_status: IntStatusRegister::new(),
            enable: EnableRegister::new(),
        }
    }
}

pub struct TransferResources<'dcb, 'src, 'dst, T> {
    pub src_cached: bool,
    pub dest_cached: bool,
    pub dcb: &'dcb ControlBlock,
    pub src_buffer: &'src [T],
    pub dest_buffer: &'dst mut [T],
}

pub struct Channel {
    dma: DMA,
}

impl Channel {
    pub fn is_lite(&self) -> bool {
        self.dma.debug.is_set(Debug::Lite::Read)
    }

    pub fn id(&self) -> u8 {
        self.dma.debug.get_field(Debug::DmaId::Read).unwrap().val() as _
    }

    pub fn is_busy(&self) -> bool {
        self.dma.cs.is_set(ControlStatus::Active::Read)
    }

    pub fn abort(&mut self) {
        // TODO
        // https://github.com/torvalds/linux/blob/master/drivers/dma/bcm2835-dma.c#L412
        unimplemented!();
    }

    pub fn reset(&mut self) {
        // TODO - abort first?
        self.dma.cs.modify(ControlStatus::Reset::Set);
        while self.dma.cs.is_set(ControlStatus::Reset::Read) == true {}
    }

    pub fn wait(&mut self) {
        unsafe { barrier::dsb(barrier::SY) };

        while self.dma.cs.is_set(ControlStatus::Active::Read) {
            asm::nop();
        }

        compiler_fence(Ordering::SeqCst);
    }

    pub fn start<'dcb, 'src, 'dst, T>(&mut self, res: &TransferResources<'dcb, 'src, 'dst, T>) {
        assert_eq!(
            res.dcb.as_paddr() & 0x1F,
            0,
            "Control block address must be 256 bit aligned"
        );
        assert_ne!(res.dcb.src, 0, "Source address is NULL");
        assert_ne!(res.dcb.dest, 0, "Destination address is NULL");

        if self.is_lite() {
            assert_eq!(
                res.dcb.info.td_mode(),
                false,
                "LITE channel doesn't support 2D mode"
            );
            assert!(res.dcb.length.0 <= TRANSFER_LENGTH_MAX_LITE);
        } else {
            if !res.dcb.info.td_mode() {
                assert!(res.dcb.length.0 <= TRANSFER_LENGTH_MAX);
            }
        }

        compiler_fence(Ordering::Release);

        unsafe {
            cache::clean_and_invalidate_data_cache_range(
                res.dcb.as_paddr(),
                mem::size_of::<ControlBlock>(),
            );
        }

        if res.src_cached {
            // TODO - need to handle 2d mode
            assert_eq!(
                res.dcb.info.td_mode(),
                false,
                "2D mode caching not supported yet"
            );
            unsafe {
                cache::clean_and_invalidate_data_cache_range(
                    (res.dcb.src & !bus_address_bits::ALIAS_4_L2_COHERENT) as usize,
                    res.dcb.length.0 as _,
                );
            }
        }

        if res.dest_cached {
            // TODO - need to handle 2d mode
            assert_eq!(
                res.dcb.info.td_mode(),
                false,
                "2D mode caching not supported yet"
            );
            unsafe {
                cache::clean_and_invalidate_data_cache_range(
                    (res.dcb.dest & !bus_address_bits::ALIAS_4_L2_COHERENT) as usize,
                    res.dcb.length.0 as _,
                );
            }
        }

        if !res.src_cached && !res.dest_cached {
            unsafe { barrier::dsb(barrier::SY) };
        }

        self.dma
            .dcb_addr
            .write(res.dcb.as_paddr() as u32 | bus_address_bits::ALIAS_4_L2_COHERENT);

        self.dma.cs.modify(ControlStatus::Active::Set);
    }

    pub fn errors(&self) -> bool {
        if self.dma.cs.is_set(ControlStatus::Error::Read) {
            return true;
        }

        if self.dma.debug.is_set(Debug::ReadLastNotSetError::Read) {
            return true;
        }

        if self.dma.debug.is_set(Debug::FifoError::Read) {
            return true;
        }

        if self.dma.debug.is_set(Debug::ReadError::Read) {
            return true;
        }

        if self
            .dma
            .debug
            .get_field(Debug::OutstandingWrites::Read)
            .unwrap()
            .val()
            != 0
        {
            return true;
        }

        false
    }
}
