use crate::eth::Eth;
use bcm2711::genet::umac::Cmd;

impl Eth {
    pub(crate) fn netif_start(&mut self) {
        self.dev.umac.cmd.modify(Cmd::TxEn::Set + Cmd::RxEn::Set);
    }
}
