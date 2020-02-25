use crate::eth::mdio::{MiiBmsr, MiiLpa, MiiStat1000, Register};
use crate::eth::{Error, Eth};

pub struct Status {
    pub link_status: bool,
    pub speed: u16,
    pub full_duplex: bool,
    pub pause: bool,
}

impl<'rx, 'tx> Eth<'rx, 'tx> {
    pub(crate) fn phy_read_status(&mut self) -> Result<Status, Error> {
        // Update the link status
        let bmsr: MiiBmsr = self.mdio_read(Register::MiiBmsr)?.into();

        let link_status = bmsr.link_status();

        let mut speed = 0;
        let mut full_duplex = false;
        let mut pause = false;

        if bmsr.link_status() {
            // Read autonegotiation status
            // NOTE: autonegotiation is enabled by firmware, not here
            let lpagb: MiiStat1000 = self.mdio_read(Register::MiiStat1000)?.into();

            let ctrl1000 = self.mdio_read(Register::MiiCtrl1000)?;

            if lpagb.lpa_1000_ms_fail() {
                // Master/Slave resolution failed
                return Err(Error::HwError);
            }

            let common_adv_gb: MiiStat1000 = (lpagb.0 & (ctrl1000 << 2)).into();

            let lpa = self.mdio_read(Register::MiiLpa)?;

            let adv = self.mdio_read(Register::MiiAdvertise)?;

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

        Ok(Status {
            link_status,
            speed,
            full_duplex,
            pause,
        })
    }
}
