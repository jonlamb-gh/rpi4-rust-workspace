use crate::eth::{Error, Eth, FCS_LEN, LEADING_PAD, MAX_MTU_SIZE, RX_BUF_LENGTH};
use crate::prelude::*;
use bcm2711::genet::umac::*;
use bcm2711::genet::{rx_desc, rx_dma, rx_ring, tx_dma, tx_ring};
use bcm2711::genet::{
    DESC_INDEX, DMA_DESC_WORDS, DMA_FC_THRESH_HI, DMA_FC_THRESH_LO, DMA_MAX_BURST_LENGTH,
    DMA_RING_BUF_PRIORITY_SHIFT, NUM_DMA_DESC, Q0_PRIORITY, Q16_RX_BD_CNT, RX_BDS_PER_Q, RX_QUEUES,
    TX_BDS_PER_Q, TX_QUEUES,
};
use core::convert::TryInto;

// TODO - remove the pub(crates) on locally used methods
// - use typenum/checked reg fields

// DMA_PRIO_REG_INDEX(x) = (q / 6)
fn prio_reg_index(q_index: usize) -> usize {
    q_index / 6
}

// DMA_PRIO_REG_SHIFT(q) = (q % 6) * DMA_RING_BUF_PRIORITY_SHIFT
fn prio_reg_shift(q_index: usize) -> usize {
    (q_index % 6) * DMA_RING_BUF_PRIORITY_SHIFT
}

impl Eth {
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

    pub(crate) fn dma_disable(&mut self) {
        self.dev
            .tdma
            .ctrl
            .modify(tx_dma::Ctrl::Enable::Clear + tx_dma::Ctrl::DefDescEnable::Clear);
        self.dev
            .rdma
            .ctrl
            .modify(rx_dma::Ctrl::Enable::Clear + rx_dma::Ctrl::DefDescEnable::Clear);

        self.dev.umac.tx_flush.modify(TxFlush::Flush::Set);
        self.timer.delay_us(10u32);
        self.dev.umac.tx_flush.modify(TxFlush::Flush::Clear);
    }

    pub(crate) fn dma_init(&mut self) {
        // Initialize common Rx ring structures
        for i in 0..NUM_DMA_DESC {
            self.rx_cbs[i].desc_index = i;
        }

        // Initialize common Tx ring structures
        for i in 0..NUM_DMA_DESC {
            self.tx_cbs[i].desc_index = i;
        }

        // Init Rx DMA
        self.dev.rdma.burst_size.write(DMA_MAX_BURST_LENGTH as _);

        // Initialize Rx queues
        self.init_rx_queues();

        // Init Tx DMA
        self.dev.tdma.burst_size.write(DMA_MAX_BURST_LENGTH as _);

        // Initialize Tx queues
        self.init_tx_queues();
    }

    // Queue 0-15 are not available
    // Queue 16 is the default Rx queue with GENET_Q16_RX_BD_CNT descriptors.
    pub(crate) fn init_rx_queues(&mut self) {
        let is_enabled = self.dev.rdma.ctrl.is_set(rx_dma::Ctrl::Enable::Read);
        self.dev.rdma.ctrl.modify(rx_dma::Ctrl::Enable::Clear);

        // Initialize Rx default queue 16
        self.init_rx_ring(
            DESC_INDEX,
            Q16_RX_BD_CNT,
            RX_QUEUES * RX_BDS_PER_Q,
            NUM_DMA_DESC,
        );

        // Enable rings
        self.dev.rdma.ring_cfg.write(1 << DESC_INDEX);

        // Configure ring as descriptor ring and re-enable DMA if enabled
        self.dev.rdma.ctrl.modify(
            rx_dma::Ctrl::Enable::Field::new(is_enabled as _).unwrap()
                //+ rx_dma::Ctrl::RingBufEnable::Field::new(1 << DESC_INDEX).unwrap(),
                + rx_dma::Ctrl::DefDescEnable::Set,
        );
    }

    pub(crate) fn init_rx_ring(
        &mut self,
        index: usize,
        size: usize,
        start_index: usize,
        end_index: usize,
    ) {
        assert_eq!(index, DESC_INDEX);
        let ring = &mut self.rx_rings[index];

        ring.index = index;

        // TODO - interrupts
        // ring->int_enable = rx_ring16_int_enable

        ring.cbs_index = start_index;
        ring.size = size;
        ring.c_index = 0;
        ring.read_ptr = start_index;
        ring.cb_ptr = start_index;
        // TODO - does my index from their ptr logic make sense here?
        ring.end_ptr = end_index - 1;
        ring.old_discards = 0;

        self.alloc_rx_buffers(index);

        self.dev.rdma.rings[index].prod_index.write(0);
        self.dev.rdma.rings[index].cons_index.write(0);
        self.dev.rdma.rings[index].buf_size.modify(
            rx_ring::BufSize::Size::Field::new(size as _).unwrap()
                + rx_ring::BufSize::BufferSize::Field::new(RX_BUF_LENGTH as _).unwrap(),
        );
        self.dev.rdma.rings[index].xon_xoff_thresh.modify(
            rx_ring::XonXoffThresh::XonThresh::Field::new(DMA_FC_THRESH_HI as _).unwrap()
                + rx_ring::XonXoffThresh::XoffThresh::Field::new(DMA_FC_THRESH_LO as _).unwrap(),
        );

        // Set start and end address, read and write pointers
        self.dev.rdma.rings[index]
            .start_addr
            .write((start_index * DMA_DESC_WORDS).try_into().unwrap());
        self.dev.rdma.rings[index]
            .read_ptr
            .write((start_index * DMA_DESC_WORDS).try_into().unwrap());
        self.dev.rdma.rings[index]
            .write_ptr
            .write((start_index * DMA_DESC_WORDS).try_into().unwrap());
        self.dev.rdma.rings[index]
            .end_addr
            .write((end_index * (DMA_DESC_WORDS - 1)).try_into().unwrap());
    }

    pub(crate) fn alloc_rx_buffers(&mut self, ring_index: usize) {
        for i in 0..self.rx_rings[ring_index].size {
            let cb_index = self.rx_rings[ring_index].cbs_index + i;
            self.rx_refill(cb_index);
        }
    }

    pub(crate) fn rx_refill(&mut self, cb_index: usize) {
        let desc_index = self.rx_cbs[cb_index].desc_index;
        let buffer_address = self.rx_cbs[cb_index].buffer.as_ptr() as usize;

        // TODO - caches are off for now
        // CleanAndInvalidateDataCacheRange

        // Put the new Rx buffer on the ring
        self.dmadesc_set_addr(desc_index, buffer_address);
    }

    // Initialize Tx queues
    //
    // Queues 0-3 are priority-based, each one has 32 descriptors,
    // with queue 0 being the highest priority queue.
    //
    // Queue 16 is the default Tx queue with
    // GENET_Q16_TX_BD_CNT = 256 - 4 * 32 = 128 descriptors.
    //
    // The transmit control block pool is then partitioned as follows:
    // - Tx queue 0 uses m_tx_cbs[0..31]
    // - Tx queue 1 uses m_tx_cbs[32..63]
    // - Tx queue 2 uses m_tx_cbs[64..95]
    // - Tx queue 3 uses m_tx_cbs[96..127]
    // - Tx queue 16 uses m_tx_cbs[128..255]
    pub(crate) fn init_tx_queues(&mut self) {
        let is_enabled = self.dev.tdma.ctrl.is_set(tx_dma::Ctrl::Enable::Read);
        self.dev.tdma.ctrl.modify(tx_dma::Ctrl::Enable::Clear);

        // Enable strict priority arbiter mode
        self.dev.tdma.arb_ctrl.modify(tx_dma::ArbCtrl::Mode::Sp);

        // Initialize Tx priority queues
        let mut dma_priority = [0, 0, 0];
        let mut enabled_buffers = 0;
        for qidx in 0..TX_QUEUES {
            self.init_tx_ring(
                qidx,
                TX_BDS_PER_Q,
                qidx * TX_BDS_PER_Q,
                (qidx + 1) * TX_BDS_PER_Q,
            );
            enabled_buffers |= 1 << qidx;
            dma_priority[prio_reg_index(qidx)] |= (Q0_PRIORITY + qidx) << prio_reg_shift(qidx);
        }

        // Initialize Tx default queue 16
        //enabled_buffers |= 1 << DESC_INDEX;
        dma_priority[prio_reg_index(DESC_INDEX)] |=
            (Q0_PRIORITY + TX_QUEUES) << prio_reg_shift(DESC_INDEX);

        // Set Tx queue priorities
        self.dev.tdma.priority_0.write(dma_priority[0] as _);
        self.dev.tdma.priority_1.write(dma_priority[1] as _);
        self.dev.tdma.priority_2.write(dma_priority[2] as _);

        // Enable Tx queues
        self.dev.tdma.ring_cfg.write(enabled_buffers);

        // Enable Tx DMA
        self.dev.tdma.ctrl.modify(
            tx_dma::Ctrl::Enable::Field::new(is_enabled as _).unwrap()
                + tx_dma::Ctrl::RingBufEnable::Field::new(enabled_buffers).unwrap()
                + tx_dma::Ctrl::DefDescEnable::Set,
        );
    }

    pub(crate) fn init_tx_ring(
        &mut self,
        index: usize,
        size: usize,
        start_index: usize,
        end_index: usize,
    ) {
        let ring = &mut self.tx_rings[index];

        if index == DESC_INDEX {
            ring.queue = 0;
        // TODO
        // ring->int_enable = tx_ring16_int_enable;
        } else {
            ring.queue = index + 1;
            // TODO
            // ring->int_enable = tx_ring_int_enable;
        }

        // TODO - does my index from their ptr logic make sense here?
        ring.cbs_index = start_index;
        ring.size = size;
        ring.clean_ptr = start_index;
        ring.c_index = 0;
        ring.free_bds = size;
        ring.write_ptr = start_index;
        ring.cb_ptr = start_index;
        ring.end_ptr = end_index - 1;
        ring.prod_index = 0;

        // Set flow period for ring != 16
        let flow_period_val = if index != DESC_INDEX { MAX_MTU_SIZE } else { 0 };

        self.dev.tdma.rings[index].prod_index.write(0);
        self.dev.tdma.rings[index].cons_index.write(0);
        self.dev.tdma.rings[index].mbuf_done_thresh.write(10);
        // Disable rate control for now
        self.dev.tdma.rings[index].flow_period.modify(
            tx_ring::FlowPeriod::Todo::Field::new(0).unwrap()
                + tx_ring::FlowPeriod::FlowPeriod::Field::new(flow_period_val as _).unwrap(),
        );
        self.dev.tdma.rings[index].buf_size.modify(
            tx_ring::BufSize::Size::Field::new(size as _).unwrap()
                + tx_ring::BufSize::BufferSize::Field::new(RX_BUF_LENGTH as _).unwrap(),
        );

        // Set start and end address, read and write pointers
        self.dev.tdma.rings[index]
            .start_addr
            .write((start_index * DMA_DESC_WORDS).try_into().unwrap());
        self.dev.tdma.rings[index]
            .read_ptr
            .write((start_index * DMA_DESC_WORDS).try_into().unwrap());
        self.dev.tdma.rings[index]
            .write_ptr
            .write((start_index * DMA_DESC_WORDS).try_into().unwrap());
        self.dev.tdma.rings[index]
            .end_addr
            .write((end_index * (DMA_DESC_WORDS - 1)).try_into().unwrap());
    }

    pub(crate) fn dmadesc_set_addr(&mut self, desc_index: usize, address: usize) {
        // TODO
        // Should use BUS_ADDRESS() here, but does not work
        self.dev.rdma.descriptors[desc_index]
            .addr_low
            .write(address as _);

        // Register writes to GISB bus can take couple hundred nanoseconds
        // and are done for each packet, save these expensive writes unless
        // the platform is explicitly configured for 64-bits/LPAE.
        // TODO: write DMA_DESC_ADDRESS_HI only once
        self.dev.rdma.descriptors[desc_index].addr_high.write(0);
    }

    // TODO result/error-handling
    pub(crate) fn dma_recv(&mut self, pkt: &mut [u8]) -> Result<usize, Error> {
        if pkt.len() < MAX_MTU_SIZE {
            return Err(Error::Exhausted);
        }

        let mut result = Ok(0);

        let ring = &mut self.rx_rings[DESC_INDEX];

        // Clear status before servicing to reduce spurious interrupts
        // NOTE: Rx interrupts are not used
        //intrl2_0_writel (UMAC_IRQ_RXDMA_DONE, INTRL2_CPU_CLEAR);

        // TODO - double check Rx/Tx prod/cons regs, they are opposite?
        let p_index = self.dev.rdma.rings[ring.index]
            .prod_index
            .get_field(rx_ring::ProdIndex::Index::Read)
            .unwrap()
            .val();
        let discards = self.dev.rdma.rings[ring.index]
            .prod_index
            .get_field(rx_ring::ProdIndex::DiscardCnt::Read)
            .unwrap()
            .val();

        // TODO - handle discards logic
        assert_eq!(discards, 0, "TODO handle discards");

        let rx_pkt_to_process = (p_index - (ring.c_index as u32)) & 0xFFFF;

        if rx_pkt_to_process != 0 {
            let cb = &self.rx_cbs[ring.read_ptr];

            // TODO - they do rx_refill() to get the rx pkt buffer
            // rx_refill()
            // --let desc_index = self.rx_cbs[cb_index].desc_index;
            // --let buffer_address = self.rx_cbs[cb_index].buffer.as_ptr() as usize;

            let dma_len = self.dev.rdma.descriptors[cb.desc_index]
                .len_status
                .get_field(rx_desc::LenStatus::Len::Read)
                .unwrap()
                .val();

            let eop = self.dev.rdma.descriptors[cb.desc_index]
                .len_status
                .is_set(rx_desc::LenStatus::Eop::Read);
            let sop = self.dev.rdma.descriptors[cb.desc_index]
                .len_status
                .is_set(rx_desc::LenStatus::Sop::Read);

            if !eop || !sop {
                result = Err(Error::Fragmented);
            } else if self.dev.rdma.descriptors[cb.desc_index]
                .len_status
                .matches_any(
                    rx_desc::LenStatus::RxCrcErr::Read
                        + rx_desc::LenStatus::RxOverflow::Read
                        + rx_desc::LenStatus::RxNo::Read
                        + rx_desc::LenStatus::RxLg::Read
                        + rx_desc::LenStatus::RxErr::Read,
                )
            {
                result = Err(Error::HwError);
            }

            if result.is_ok() {
                // Remove HW 2 bytes added for IP alignment
                let pkt_len = if self.crc_fwd_en {
                    (dma_len as usize - LEADING_PAD) - FCS_LEN
                } else {
                    dma_len as usize - LEADING_PAD
                };

                if pkt_len == 0 {
                    result = Err(Error::Malformed);
                } else if pkt_len > pkt.len() {
                    result = Err(Error::Exhausted);
                } else {
                    let pkt_slice = &cb.buffer[LEADING_PAD..pkt_len + LEADING_PAD];
                    &pkt[0..pkt_len].copy_from_slice(pkt_slice);
                    result = Ok(pkt_len);
                }
            }

            // Always try to update the rings, even if an error was encountered
            if ring.read_ptr < ring.end_ptr {
                ring.read_ptr += 1;
            } else {
                ring.read_ptr = ring.cb_ptr;
            }

            ring.c_index = (ring.c_index + 1) & 0xFFFF;

            self.dev.rdma.rings[ring.index]
                .cons_index
                .write(ring.c_index as _);
        }

        result
    }
}
