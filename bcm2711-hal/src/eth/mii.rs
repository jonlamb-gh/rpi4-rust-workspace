use crate::eth::Eth;
use bcm2711::genet::ext::RgmiiOobCtrl;
use bcm2711::genet::sys::PortCtrl;
use bcm2711::genet::umac::Cmd;

impl Eth {
    pub(crate) fn mii_probe(&mut self) {
        // Initialize link state variables that mii_setup() uses
        self.old_link_status = None;
        self.old_speed = None;
        self.old_full_duplex = None;
        self.old_pause = None;

        self.mdio_reset();

        self.phy_read_status();

        self.mii_setup();

        self.mii_config();
    }

    // Setup netdev link state when PHY link status change and
    // update UMAC and RGMII block when link up
    pub(crate) fn mii_setup(&mut self) {
        let mut status_changed = false;

        if self.old_link_status != Some(self.link_status) {
            status_changed = true;
            self.old_link_status = Some(self.link_status);
        }

        if self.link_status {
            // Check speed/duplex/pause changes
            if self.old_speed != Some(self.speed) {
                status_changed = true;
                self.old_speed = Some(self.speed);
            }
            if self.old_full_duplex != Some(self.full_duplex) {
                status_changed = true;
                self.old_full_duplex = Some(self.full_duplex);
            }
            if self.old_pause != Some(self.pause) {
                status_changed = true;
                self.old_pause = Some(self.pause);
            }

            if status_changed {
                // Program UMAC and RGMII block based on established
                // link speed, duplex, and pause. The speed set in
                // umac->cmd tell RGMII block which clock to use for
                // transmit -- 25MHz(100Mbps) or 125MHz(1Gbps).
                // Receive clock is provided by the PHY.
                self.dev
                    .ext
                    .rgmii_oob_ctrl
                    .modify(RgmiiOobCtrl::OobDisable::Clear + RgmiiOobCtrl::RgmiiLink::Set);

                let speed = match self.speed {
                    1000 => 2,
                    100 => 1,
                    _ => 0,
                };

                self.dev.umac.cmd.modify(
                    Cmd::Speed::Field::new(speed).unwrap()
                        + Cmd::RxPauseIgnore::Field::new(!self.pause as _).unwrap()
                        + Cmd::TxPauseIgnore::Field::new(!self.pause as _).unwrap()
                        + Cmd::HdEn::Field::new(!self.full_duplex as _).unwrap(),
                );
            }
        }
    }

    fn mii_config(&mut self) {
        // RGMII_NO_ID: TXC transitions at the same time as TXD
        //		(requires PCB or receiver-side delay)
        // RGMII:	Add 2ns delay on TXC (90 degree shift)
        //
        // ID is implicitly disabled for 100Mbps (RG)MII operation.

        self.dev.sys.port_ctrl.modify(PortCtrl::PortMode::ExtGPhy);

        // This is an external PHY (xMII), so we need to enable the RGMII
        // block for the interface to work
        self.dev
            .ext
            .rgmii_oob_ctrl
            .modify(RgmiiOobCtrl::RgmiiModeEn::Set + RgmiiOobCtrl::IdModeDis::Set);
    }
}
