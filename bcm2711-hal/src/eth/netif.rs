use crate::eth::Eth;
use bcm2711::genet::umac::Cmd;

impl Eth {
    pub(crate) fn netif_start(&mut self) {
        // NOTE: Rx interrupts are not needed
        // enable_rx_intr()

        // umac_enable_set(CMD_TX_EN | CMD_RX_EN, true);
        self.dev.umac.cmd.modify(Cmd::TxEn::Set + Cmd::RxEn::Set);

        // TODO
        // enable_tx_intr()

        //NOTE: link interrupts do not work, must be polled
        // link_intr_enable();
    }
}
