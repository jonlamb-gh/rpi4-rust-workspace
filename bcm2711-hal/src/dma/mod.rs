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

pub struct Channel {
    dma: DMA,
    /* TODO - hold on to ref in start(), return it in wait()
     *adcb: Option<&'a ControlBlock>, */
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

    /// dcb_paddr - the physical address of the control block to load
    /// NOTE: the physical address will be translated to a bus address for
    /// the DMA engine
    pub fn start(&mut self, dcb: &ControlBlock) {
        let dcb_paddr = dcb as *const _ as u32;
        assert_eq!(
            dcb_paddr & 0x1F,
            0,
            "Control block address must be 256 bit aligned"
        );
        assert_ne!(dcb.src, 0, "Source address is NULL");
        assert_ne!(dcb.dest, 0, "Destination address is NULL");

        if self.is_lite() {
            assert_eq!(
                dcb.info.td_mode(),
                false,
                "LITE channel doesn't support 2D mode"
            );
            assert!(dcb.length.0 <= TRANSFER_LENGTH_MAX_LITE);
        } else {
            if !dcb.info.td_mode() {
                assert!(dcb.length.0 <= TRANSFER_LENGTH_MAX);
            }
        }

        unsafe { barrier::dsb(barrier::SY) };

        self.dma
            .dcb_addr
            .write(dcb_paddr | bus_address_bits::ALIAS_4_L2_COHERENT);

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
