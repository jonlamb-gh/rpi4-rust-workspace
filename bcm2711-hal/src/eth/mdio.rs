//! UniMAC MDIO

use crate::eth::{Error, Eth};
use bcm2711::genet::mdio::Cmd;
use bcm2711::genet::umac;
use bitfield::bitfield;
use cortex_a::asm;

const PHY_ID: u8 = 0x01;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Register {
    MiiBmsr = 0x01,
    MiiAdvertise = 0x04,
    MiiLpa = 0x05,
    MiiCtrl1000 = 0x09,
    MiiStat1000 = 0x0A,
}

bitfield! {
    #[repr(C)]
    pub struct MiiBmsr(u16);
    impl Debug;
    u16;
    pub link_status, set_link_status : 2;
}

impl From<u16> for MiiBmsr {
    fn from(val: u16) -> MiiBmsr {
        MiiBmsr(val)
    }
}

bitfield! {
    #[repr(C)]
    pub struct MiiLpa(u16);
    impl Debug;
    u16;
    pub lpa_10_half, set_lpa_10_half : 5;
    pub lpa_10_full, set_lpa_10_full : 6;
    pub lpa_100_half, set_lpa_100_half : 7;
    pub lpa_100_full, set_lpa_100_full : 8;
    pub pause_cap, set_pause_cap : 10;
}

impl From<u16> for MiiLpa {
    fn from(val: u16) -> MiiLpa {
        MiiLpa(val)
    }
}

bitfield! {
    #[repr(C)]
    pub struct MiiStat1000(u16);
    impl Debug;
    u16;
    pub lpa_1000_half, set_lpa_1000_half : 10;
    pub lpa_1000_full, set_lpa_1000_full : 11;
    pub lpa_1000_ms_fail, lpa_1000_set_ms_fail : 15;
}

impl From<u16> for MiiStat1000 {
    fn from(val: u16) -> MiiStat1000 {
        MiiStat1000(val)
    }
}

impl<'rx, 'tx> Eth<'rx, 'tx> {
    // Workaround for integrated BCM7xxx Gigabit PHYs which have a problem with
    // their internal MDIO management controller making them fail to successfully
    // be read from or written to for the first transaction.  We insert a dummy
    // BMSR read here to make sure that phy_get_device() and get_phy_id() can
    // correctly read the PHY MII_PHYSID1/2 registers and successfully register a
    // PHY device for this peripheral.
    pub(crate) fn mdio_reset(&mut self) {
        let _ = self.mdio_read(Register::MiiBmsr);
    }

    pub(crate) fn mdio_read(&mut self, reg: Register) -> Result<u16, Error> {
        self.dev.mdio.cmd.modify(
            Cmd::Rw::RwRead
                + Cmd::PhyId::Field::new(PHY_ID as _).unwrap()
                + Cmd::Reg::Field::new(reg as _).unwrap(),
        );

        self.dev.mdio.cmd.modify(Cmd::StartBusy::Set);

        self.mdio_wait();

        if self.dev.mdio.cmd.is_set(Cmd::ReadFail::Read) {
            Err(Error::HwError)
        } else {
            Ok(self.dev.mdio.cmd.get_field(Cmd::Data::Read).unwrap().val() as u16)
        }
    }

    fn mdio_wait(&self) {
        while self
            .dev
            .umac
            .mdio_cmd
            .is_set(umac::MdioCmd::StartBusy::Read)
        {
            asm::nop();
        }
    }
}
