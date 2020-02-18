use crate::eth::mdio::{MiiBmsr, MiiLpa, MiiStat1000, Register};
use crate::eth::Eth;

pub struct Status {
    pub link_status: bool,
    pub speed: u16,
    pub full_duplex: bool,
    pub pause: bool,
}

impl Eth {
    // phy_read_status - check the link status and update current link state
    //
    // Description: Check the link, then figure out the current state
    //   by comparing what we advertise with what the link partner
    //   advertises.  Start by checking the gigabit possibilities,
    //   then move on to 10/100.
    pub(crate) fn phy_read_status(&mut self) -> Status {
        // Update the link status
        let bmsr: MiiBmsr = self.mdio_read(Register::MiiBmsr).into();

        let mut link_status = bmsr.link_status();

        // TODO - fixup these
        let mut speed = 0;
        let mut full_duplex = false;
        let mut pause = false;

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
                speed = 1000;
                full_duplex = common_adv_gb.lpa_1000_full();
            } else if common_adv.lpa_100_half() || common_adv.lpa_100_full() {
                speed = 100;
                full_duplex = common_adv.lpa_100_full();
            } else {
                speed = 10;
                full_duplex = common_adv.lpa_10_full();
            }

            if full_duplex {
                pause = MiiLpa::from(lpa).pause_cap();
            }
        }

        Status {
            link_status,
            speed,
            full_duplex,
            pause,
        }
    }
}
