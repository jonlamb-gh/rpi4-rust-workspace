use crate::eth::mdio::{MiiBmsr, MiiLpa, MiiStat1000, Register};
use crate::eth::Eth;

impl Eth {
    // phy_read_status - check the link status and update current link state
    //
    // Description: Check the link, then figure out the current state
    //   by comparing what we advertise with what the link partner
    //   advertises.  Start by checking the gigabit possibilities,
    //   then move on to 10/100.
    pub(crate) fn phy_read_status(&mut self) {
        // Update the link status
        let bmsr: MiiBmsr = self.mdio_read(Register::MiiBmsr).into();

        self.link_status = bmsr.link_status();

        // TODO - fixup these
        self.speed = 0;
        self.full_duplex = false;
        self.pause = false;

        if bmsr.link_status() {
            // Read autonegotiation status
            // NOTE: autonegotiation is enabled by firmware, not here
            let lpagb: MiiStat1000 = self.mdio_read(Register::MiiStat1000).into();

            let ctrl1000 = self.mdio_read(Register::MiiCtrl1000);

            if lpagb.lpa_1000_ms_fail() {
                // Master/Slave resolution failed
                panic!("TODO - error handling");
            }

            // TODO - what's the shift about, what are the fields on ctrl1000?
            let common_adv_gb: MiiStat1000 = (lpagb.0 & (ctrl1000 << 2)).into();

            let lpa = self.mdio_read(Register::MiiLpa);

            let adv = self.mdio_read(Register::MiiAdvertise);

            let common_adv: MiiLpa = (lpa & adv).into();

            if common_adv_gb.lpa_1000_half() || common_adv_gb.lpa_1000_full() {
                self.speed = 1000;
                self.full_duplex = common_adv_gb.lpa_1000_full();
            } else if common_adv.lpa_100_half() || common_adv.lpa_100_full() {
                self.speed = 100;
                self.full_duplex = common_adv.lpa_100_full();
            } else {
                self.speed = 10;
                self.full_duplex = common_adv.lpa_10_full();
            }

            if self.full_duplex {
                self.pause = MiiLpa::from(lpa).pause_cap();
            }
        }
    }
}
