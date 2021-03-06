use crate::cache;
use crate::eth::{Error, Eth, RxPacket, MAX_MTU_SIZE, MIN_MTU_SIZE, RX_BUF_LENGTH, TX_BUF_LENGTH};
use crate::hal::blocking::delay::DelayUs;
use bcm2711::genet::umac::*;
use bcm2711::genet::{rx_desc, rx_dma, rx_ring, tx_desc, tx_dma, tx_ring};
use bcm2711::genet::{
    DEFAULT_Q, DMA_DESC_WORDS, DMA_FC_THRESH_HI, DMA_FC_THRESH_LO, DMA_MAX_BURST_LENGTH,
    NUM_DMA_DESC, QTAG_MASK, QTAG_SHIFT,
};
use core::convert::TryInto;

impl<'rx, 'tx> Eth<'rx, 'tx> {
    pub(crate) fn dma_enable(&mut self) {
        self.dev
            .rdma
            .ctrl
            .modify(rx_dma::Ctrl::Enable::Set + rx_dma::Ctrl::DefDescEnable::Set);
        self.dev
            .tdma
            .ctrl
            .modify(tx_dma::Ctrl::Enable::Set + tx_dma::Ctrl::DefDescEnable::Set);
    }

    pub(crate) fn dma_disable<D: DelayUs<u32>>(&mut self, delay: &mut D) {
        self.dev
            .tdma
            .ctrl
            .modify(tx_dma::Ctrl::Enable::Clear + tx_dma::Ctrl::DefDescEnable::Clear);
        self.dev
            .rdma
            .ctrl
            .modify(rx_dma::Ctrl::Enable::Clear + rx_dma::Ctrl::DefDescEnable::Clear);

        self.dev.umac.tx_flush.modify(TxFlush::Flush::Set);
        delay.delay_us(10u32);
        self.dev.umac.tx_flush.modify(TxFlush::Flush::Clear);
    }

    pub(crate) fn rx_ring_init(&mut self) {
        self.dev.rdma.burst_size.write(DMA_MAX_BURST_LENGTH as _);

        // Set start and end address, read and write pointers
        self.dev.rdma.rings[DEFAULT_Q].start_addr.write(0);
        self.dev.rdma.rings[DEFAULT_Q].start_addr_hi.write(0);
        self.dev.rdma.rings[DEFAULT_Q].read_ptr.write(0);
        self.dev.rdma.rings[DEFAULT_Q].read_ptr_hi.write(0);
        self.dev.rdma.rings[DEFAULT_Q].write_ptr.write(0);
        self.dev.rdma.rings[DEFAULT_Q].write_ptr_hi.write(0);
        self.dev.rdma.rings[DEFAULT_Q]
            .end_addr
            .write(((NUM_DMA_DESC * DMA_DESC_WORDS) - 1).try_into().unwrap());
        self.dev.rdma.rings[DEFAULT_Q].end_addr_hi.write(0);

        self.dev.rdma.rings[DEFAULT_Q].prod_index.write(0);
        self.dev.rdma.rings[DEFAULT_Q].cons_index.write(0);
        self.dev.rdma.rings[DEFAULT_Q].buf_size.modify(
            rx_ring::BufSize::Size::Field::new(NUM_DMA_DESC as _).unwrap()
                + rx_ring::BufSize::BufferSize::Field::new(RX_BUF_LENGTH as _).unwrap(),
        );
        self.dev.rdma.rings[DEFAULT_Q].xon_xoff_thresh.modify(
            rx_ring::XonXoffThresh::XonThresh::Field::new(DMA_FC_THRESH_HI as _).unwrap()
                + rx_ring::XonXoffThresh::XoffThresh::Field::new(DMA_FC_THRESH_LO as _).unwrap(),
        );

        self.dev.rdma.ring_cfg.write(1 << DEFAULT_Q);
    }

    pub(crate) fn rx_descs_init(&mut self) {
        self.c_index = 0;

        for desc_index in 0..NUM_DMA_DESC {
            let address: u64 = self.rx_mem[desc_index].buffer.as_ptr() as u64;
            let addr_low = (address & (core::u32::MAX as u64)) as u32;
            let addr_high = ((address >> 32) & (core::u32::MAX as u64)) as u32;

            self.dev.rdma.descriptors[desc_index]
                .addr_low
                .write(addr_low);

            self.dev.rdma.descriptors[desc_index]
                .addr_high
                .write(addr_high);

            self.dev.rdma.descriptors[desc_index].len_status.modify(
                rx_desc::LenStatus::Len::Field::new(RX_BUF_LENGTH as _).unwrap()
                    + rx_desc::LenStatus::Own::Set,
            );
        }
    }

    pub(crate) fn tx_ring_init(&mut self) {
        self.dev.tdma.burst_size.write(DMA_MAX_BURST_LENGTH as _);

        // Set start and end address, read and write pointers
        self.dev.tdma.rings[DEFAULT_Q].start_addr.write(0);
        self.dev.tdma.rings[DEFAULT_Q].start_addr_hi.write(0);
        self.dev.tdma.rings[DEFAULT_Q].read_ptr.write(0);
        self.dev.tdma.rings[DEFAULT_Q].read_ptr_hi.write(0);
        self.dev.tdma.rings[DEFAULT_Q].write_ptr.write(0);
        self.dev.tdma.rings[DEFAULT_Q].write_ptr_hi.write(0);
        self.dev.tdma.rings[DEFAULT_Q]
            .end_addr
            .write(((NUM_DMA_DESC * DMA_DESC_WORDS) - 1).try_into().unwrap());
        self.dev.tdma.rings[DEFAULT_Q].end_addr_hi.write(0);

        self.dev.tdma.rings[DEFAULT_Q].prod_index.write(0);
        self.dev.tdma.rings[DEFAULT_Q].cons_index.write(0);
        self.dev.tdma.rings[DEFAULT_Q].mbuf_done_thresh.write(1);

        self.dev.tdma.rings[DEFAULT_Q].flow_period.write(0);

        self.dev.tdma.rings[DEFAULT_Q].buf_size.modify(
            tx_ring::BufSize::Size::Field::new(NUM_DMA_DESC as _).unwrap()
                + tx_ring::BufSize::BufferSize::Field::new(TX_BUF_LENGTH as _).unwrap(),
        );

        self.dev.tdma.ring_cfg.write(1 << DEFAULT_Q);
    }

    pub(crate) fn dma_recv(&mut self) -> Result<RxPacket, Error> {
        let p_index = self.dev.rdma.rings[DEFAULT_Q]
            .prod_index
            .get_field(rx_ring::ProdIndex::Index::Read)
            .unwrap()
            .val();

        // TODO add this back in, not in the u-boot impl
        //let discards = self.dev.rdma.rings[DEFAULT_Q]
        //    .prod_index
        //    .get_field(rx_ring::ProdIndex::DiscardCnt::Read)
        //    .unwrap()
        //    .val();
        //assert_eq!(discards, 0, "TODO");

        if p_index as usize == self.c_index {
            Err(Error::WouldBlock)
        } else {
            let dma_len = self.dev.rdma.descriptors[self.rx_index]
                .len_status
                .get_field(rx_desc::LenStatus::Len::Read)
                .unwrap()
                .val() as usize;

            let packet_desc_error = self.dev.rdma.descriptors[self.rx_index]
                .len_status
                .matches_any(
                    rx_desc::LenStatus::RxOverflow::Read
                        + rx_desc::LenStatus::RxCrcErr::Read
                        + rx_desc::LenStatus::RxErr::Read,
                );

            let eop = self.dev.rdma.descriptors[self.rx_index]
                .len_status
                .is_set(rx_desc::LenStatus::Eop::Read);
            let sop = self.dev.rdma.descriptors[self.rx_index]
                .len_status
                .is_set(rx_desc::LenStatus::Sop::Read);

            let result = if !eop || !sop {
                Err(Error::Fragmented)
            } else if packet_desc_error {
                Err(Error::HwDescError)
            } else {
                if dma_len == 0 {
                    Err(Error::Malformed)
                } else {
                    unsafe {
                        cache::clean_and_invalidate_data_cache_range(
                            self.rx_mem[self.rx_index].as_paddr(),
                            RX_BUF_LENGTH,
                        );
                    }

                    let pkt = RxPacket {
                        entry: &mut self.rx_mem[self.rx_index],
                        length: dma_len,
                    };
                    Ok(pkt)
                }
            };

            // Always try to update the rings, even if an error was encountered

            // Tell the MAC we have consumed that last receive buffer
            self.c_index = (self.c_index + 1) & 0xFFFF;
            self.dev.rdma.rings[DEFAULT_Q]
                .cons_index
                .write(self.c_index as _);

            // Forward our descriptor pointer, wrapping around if needed
            self.rx_index = self.rx_index.saturating_add(1);
            if self.rx_index >= NUM_DMA_DESC {
                self.rx_index = 0;
            }

            result
        }
    }

    //pub(crate) fn dma_send(&mut self, pkt: &[u8]) -> Result<(), Error> {
    pub(crate) fn dma_send<F: FnOnce(&mut [u8]) -> R, R>(
        &mut self,
        length: usize,
        f: F,
    ) -> Result<R, Error> {
        if length > MAX_MTU_SIZE {
            return Err(Error::Exhausted);
        }

        let r = f(&mut self.tx_mem[self.tx_index].as_mut_slice()[..length]);

        // Pad the frame if needed
        let length = if length < MIN_MTU_SIZE {
            for b in self.tx_mem[self.tx_index].as_mut_slice()[length..MIN_MTU_SIZE].iter_mut() {
                *b = 0;
            }
            MIN_MTU_SIZE
        } else {
            length
        };

        unsafe {
            cache::clean_and_invalidate_data_cache_range(
                self.tx_mem[self.tx_index].as_paddr(),
                TX_BUF_LENGTH,
            );
        }

        let address = self.tx_mem[self.tx_index].as_paddr() as u64;
        let addr_low = (address & (core::u32::MAX as u64)) as u32;
        let addr_high = ((address >> 32) & (core::u32::MAX as u64)) as u32;

        let p_index = self.dev.tdma.rings[DEFAULT_Q]
            .prod_index
            .get_field(tx_ring::ProdIndex::Index::Read)
            .unwrap()
            .val();

        self.dev.tdma.descriptors[self.tx_index]
            .addr_low
            .write(addr_low);

        // Register writes to GISB bus can take couple hundred nanoseconds
        // and are done for each packet, save these expensive writes unless
        // the platform is explicitly configured for 64-bits/LPAE.
        self.dev.tdma.descriptors[self.tx_index]
            .addr_high
            .write(addr_high);

        // TODO - QTAG mask and shift overlap with other fields?
        self.dev.tdma.descriptors[self.tx_index]
            .len_status
            .write(QTAG_MASK << QTAG_SHIFT);
        self.dev.tdma.descriptors[self.tx_index].len_status.modify(
            tx_desc::LenStatus::Len::Field::new(length as _).unwrap()
                + tx_desc::LenStatus::TxAppendCrc::Set
                + tx_desc::LenStatus::Sop::Set
                + tx_desc::LenStatus::Eop::Set,
        );

        // Increment index and start transmission
        self.tx_index = self.tx_index.saturating_add(1);
        if self.tx_index >= NUM_DMA_DESC {
            self.tx_index = 0;
        }

        // Start transmisson
        let p_index = p_index + 1;
        self.dev.tdma.rings[DEFAULT_Q]
            .prod_index
            .write(p_index as _);

        let mut tries = 100;
        loop {
            let c_index = self.dev.tdma.rings[DEFAULT_Q]
                .cons_index
                .get_field(tx_ring::ConsIndex::Index::Read)
                .unwrap()
                .val();

            if c_index >= p_index {
                break;
            }

            tries -= 1;
            if tries == 0 {
                return Err(Error::TimedOut);
            }
        }

        Ok(r)
    }
}
