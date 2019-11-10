use crate::eth::Eth;
use bcm2711::genet::hfb::NUM_FILTERS;
use bcm2711::genet::rx_dma::NUM_INDEX2RINGS;

impl Eth {
    pub(crate) fn hfb_init(&mut self) {
        self.dev.hfb_regs.ctrl.write(0);
        self.dev.hfb_regs.flt_enable.write(0);
        self.dev.hfb_regs.unknown_ctrl.write(0);

        for i in 0..NUM_INDEX2RINGS {
            self.dev.rdma.index2rings[i].write(0);
        }

        for i in 0..(NUM_FILTERS / 4) {
            self.dev.hfb_regs.flt_lens[i].write(0);
        }

        // TODO - this isn't right
        //for i in 0..(NUM_FILTERS * FILTER_SIZE) {
        for i in 0..NUM_FILTERS {
            self.dev.hfb.filter_blocks[i].hfb0.write(0);
            self.dev.hfb.filter_blocks[i].hfb1.write(0);
            self.dev.hfb.filter_blocks[i].hfb2.write(0);
            self.dev.hfb.filter_blocks[i].hfb3.write(0);
        }
    }
}
