use crate::eth::{Eth, EthernetAddress, MAX_MTU_SIZE};
use crate::prelude::*;
use bcm2711::genet::rbuf::*;
use bcm2711::genet::sys::*;
use bcm2711::genet::umac::*;

impl Eth {
    pub(crate) fn umac_reset(&mut self) {
        // 7358a0/7552a0: bad default in RBUF_FLUSH_CTRL.umac_sw_rst
        self.dev.sys.rbuf_flush_ctrl.write(0);
        self.timer.delay_us(10u32);

        // Disable MAC while updating its registers
        self.dev.umac.cmd.write(0);

        // Issue soft reset with (rg)mii loopback to ensure a stable rxclk
        self.dev
            .umac
            .cmd
            .modify(Cmd::SwReset::Set + Cmd::LclLoopEn::Set);
        self.timer.delay_us(2u32);
        self.dev.umac.cmd.write(0);
    }

    pub(crate) fn umac_reset2(&mut self) {
        self.dev
            .sys
            .rbuf_flush_ctrl
            .modify(RBufFlushCtrl::Reset::Set);
        self.timer.delay_us(10u32);
        self.dev
            .sys
            .rbuf_flush_ctrl
            .modify(RBufFlushCtrl::Reset::Clear);
        self.timer.delay_us(10u32);
    }

    pub(crate) fn umac_init(&mut self) {
        self.umac_reset();

        // Clear tx/rx counter
        self.dev
            .umac
            .mib_ctrl
            .modify(MibCtrl::ResetRx::Set + MibCtrl::ResetRunt::Set + MibCtrl::ResetTx::Set);
        self.dev.umac.mib_ctrl.write(0);

        self.dev.umac.max_frame_len.write(MAX_MTU_SIZE as _);

        // Init rx registers, enable ip header optimization
        self.dev.rbuf.ctrl.modify(Ctrl::Align2Byte::Set);

        self.dev.rbuf.tbuf_size_ctrl.write(1);

        self.intr_disable();

        // Enable MDIO interrupts on GENET v3+
        // NOTE: MDIO interrupts do not work
        //intrl2_0_writel(
        // UMAC_IRQ_MDIO_DONE | UMAC_IRQ_MDIO_ERROR, INTRL2_CPU_MASK_CLEAR);
    }

    pub(crate) fn umac_set_hw_addr(&mut self, addr: &EthernetAddress) {
        self.dev.umac.mac0.modify(
            Mac0::Addr0::Field::new(addr.0[0] as _).unwrap()
                + Mac0::Addr1::Field::new(addr.0[1] as _).unwrap()
                + Mac0::Addr2::Field::new(addr.0[2] as _).unwrap()
                + Mac0::Addr3::Field::new(addr.0[3] as _).unwrap(),
        );
        self.dev.umac.mac1.modify(
            Mac1::Addr4::Field::new(addr.0[4] as _).unwrap()
                + Mac1::Addr5::Field::new(addr.0[5] as _).unwrap(),
        );
    }

    pub(crate) fn umac_set_rx_mode(&mut self, addr: &EthernetAddress) {
        // Promiscuous mode off
        self.dev.umac.cmd.modify(Cmd::Promisc::Clear);

        // update MDF filter

        // Broadcast
        let broadcast = EthernetAddress::BROADCAST;
        let index = 0;
        self.set_mdf_addr(index, &broadcast);

        // Own address
        let index = 1;
        self.set_mdf_addr(index, addr);
    }

    // TODO - check this logic
    fn set_mdf_addr(&mut self, index: usize, addr: &EthernetAddress) {
        self.dev.umac.mdf_addrs[index].mdf_addr0.modify(
            MdfAddr0::Addr0::Field::new(addr.0[0] as _).unwrap()
                + MdfAddr0::Addr1::Field::new(addr.0[1] as _).unwrap(),
        );
        self.dev.umac.mdf_addrs[index].mdf_addr1.modify(
            MdfAddr1::Addr2::Field::new(addr.0[2] as _).unwrap()
                + MdfAddr1::Addr3::Field::new(addr.0[3] as _).unwrap()
                + MdfAddr1::Addr4::Field::new(addr.0[4] as _).unwrap()
                + MdfAddr1::Addr5::Field::new(addr.0[5] as _).unwrap(),
        );

        let reg = self.dev.umac.mdf_ctrl.read();
        self.dev
            .umac
            .mdf_ctrl
            .write(reg | (1 << (MAX_MDF_FILTERS - index)));
    }
}
