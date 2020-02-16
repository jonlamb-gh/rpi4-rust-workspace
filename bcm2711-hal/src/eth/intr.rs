use crate::eth::Eth;
use bcm2711::genet::intrl2::INTRL2;
use bcm2711::genet::TX_QUEUES;

impl Eth {
    pub(crate) fn intr_disable(&mut self) {
        // Mask all interrupts
        self.dev.intrl2_0.mask_set.write(0xFFFF_FFFF);
        self.dev.intrl2_0.clear.write(0xFFFF_FFFF);
        self.dev.intrl2_1.mask_set.write(0xFFFF_FFFF);
        self.dev.intrl2_1.clear.write(0xFFFF_FFFF);
    }

    // TODO
    pub(crate) fn tx_ring16_int_enable(&mut self) {
        let mut status = INTRL2(0);
        status.set_txdma_mbdone(true);

        //
        //status.set_txdma_pdone(true);
        //status.set_txdma_bdone(true);

        self.dev.intrl2_0.mask_clear.write(status.0);
    }

    // TODO
    pub(crate) fn tx_ring_int_enable(&mut self, ring_index: usize) {
        self.dev.intrl2_1.mask_clear.write(1 << ring_index);
    }

    pub(crate) fn intr_tx_enable(&mut self) {
        for i in 0..TX_QUEUES {
            let ring_index = self.tx_rings[i].index;
            self.tx_ring_int_enable(ring_index);
        }
        self.tx_ring16_int_enable();
    }

    // TODO
    // setup in init_tx_ring()
    //
    // tx_reclaim()
    // InterruptHandler0
    // InterruptHandler1

    pub(crate) fn intr_handler0(&mut self) {
        let stat = self.dev.intrl2_0.status.read();
        let mask = self.dev.intrl2_0.mask_status.read();
        let status = INTRL2(stat & mask);
        //let status = INTRL2(stat);

        self.dev.intrl2_0.clear.write(status.0);

        // TODO
        if status.txdma_mbdone()
            || status.txdma_pdone()
            || status.txdma_bdone()
            || status.tbuf_underrun()
        {
            // TODO
            // tx_reclaim
            panic!("INTRL2_0 0x{:X}", status.0);
        }
    }

    pub(crate) fn intr_handler1(&mut self) {
        let stat = self.dev.intrl2_1.status.read();
        let mask = self.dev.intrl2_1.mask_status.read();
        let status = stat & mask;

        self.dev.intrl2_1.clear.write(status);

        for idx in 0..TX_QUEUES {
            if status & (1 << idx) != 0 {
                // TODO
                // tx_reclaim
                panic!("INTRL2_1 0x{:X} -- TX Q {} IRQ set", status, idx);
            }
        }
    }
}
