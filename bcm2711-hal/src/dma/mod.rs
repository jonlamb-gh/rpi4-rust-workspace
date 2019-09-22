//! DMA

// TODO
// - use dma pattern from cortex-m HAL's instead of the unsafe PMem style like
//   in https://github.com/astro/stm32f429-hal no mmu/vaddr's, can use normal
//   slices/memory
// - generate all the channels via macros
// - following https://github.com/rust-embedded/embedded-hal/issues/37#issuecomment-377823801
// - fix the sync/fences/barriers

use crate::cache::bus_address_bits;
use bcm2711::dma::*;
use core::sync::atomic::{compiler_fence, Ordering};
use cortex_a::{asm, barrier};

mod control_block;

pub use crate::dma::control_block::{
    ControlBlock, StrideWord, TransferLength, TransferWidth, TxfrInfoWord, TxfrLenWord,
    CONTROL_BLOCK_SIZE,
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

    fn split(self) -> Parts {
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

pub struct Channel {
    dma: DMA,
}

impl Channel {
    pub fn is_lite(&self) -> bool {
        self.dma.DEBUG.is_set(DEBUG::LITE)
    }

    pub fn dma_id(&self) -> u8 {
        self.dma.DEBUG.read(DEBUG::DMA_ID) as _
    }

    pub fn is_busy(&self) -> bool {
        self.dma.CS.is_set(CS::ACTIVE)
    }

    pub fn abort(&mut self) {
        // TODO
        unimplemented!();
    }

    pub fn reset(&mut self) {
        // TODO - abort first?
        self.dma.CS.modify(CS::RESET::SET);
        while self.dma.CS.is_set(CS::RESET) == true {}
    }

    pub fn wait(&self) {
        unsafe { barrier::dsb(barrier::SY) };

        while self.dma.CS.is_set(CS::ACTIVE) {
            asm::nop();
        }

        compiler_fence(Ordering::SeqCst);
    }

    /// dcb_paddr - the physical address of the control block to load
    /// NOTE: the physical address will be translated to a bus address for
    /// the DMA engine
    pub fn start(&mut self, dcb_paddr: u32) {
        assert_eq!(
            dcb_paddr & 0x1F,
            0,
            "Control block address must be 256 bit aligned"
        );

        unsafe { barrier::dsb(barrier::SY) };

        self.dma
            .CONBLK_AD
            .set(dcb_paddr | bus_address_bits::ALIAS_4_L2_COHERENT);

        self.dma.CS.modify(CS::ACTIVE::SET);
    }

    pub fn errors(&self) -> bool {
        if self.dma.CS.is_set(CS::ERROR) {
            return true;
        }

        if self.dma.DEBUG.is_set(DEBUG::READ_LAST_NOT_SET_ERROR) {
            return true;
        }

        if self.dma.DEBUG.is_set(DEBUG::FIFO_ERROR) {
            return true;
        }

        if self.dma.DEBUG.is_set(DEBUG::READ_ERROR) {
            return true;
        }

        if self.dma.DEBUG.read(DEBUG::OUTSTANDING_WRITES) != 0 {
            return true;
        }

        false
    }
}
