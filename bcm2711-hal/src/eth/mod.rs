//! BCM54213PE Gigabit Ethernet driver
//!
//! This implementation was based on:
//! https://github.com/u-boot/u-boot/blob/master/drivers/net/bcmgenet.c

use crate::hal::blocking::delay::DelayUs;
use bcm2711::genet::*;

pub use crate::eth::address::EthernetAddress;
pub use crate::eth::descriptor::Descriptor;
pub use crate::eth::phy::Status as PhyStatus;

mod address;
mod descriptor;
mod dma;
mod mdio;
mod mii;
mod netif;
mod phy;
mod umac;

const GENET_V5: u8 = 5;

// Hw adds 2 bytes for IP alignment
const LEADING_PAD: usize = 2;

// Body(1500) + EH_SIZE(14) + VLANTAG(4) + BRCMTAG(6) + FCS(4) = 1528.
// 1536 is multiple of 256 bytes
pub const MAX_MTU_SIZE: usize = 1536;
pub const MIN_MTU_SIZE: usize = 60;
pub const RX_BUF_LENGTH: usize = 2048;
pub const TX_BUF_LENGTH: usize = 2048;

pub type Descriptors = [Descriptor; NUM_DMA_DESC];

pub struct Devices {
    pub sys: SYS,
    pub ext: EXT,
    pub intrl2_0: INTRL2_0,
    pub intrl2_1: INTRL2_1,
    pub rbuf: RBUF,
    pub umac: UMAC,
    pub hfb: HFB,
    pub hfb_regs: HFBREGS,
    pub mdio: MDIO,
    pub rdma: RXDMA,
    pub tdma: TXDMA,
}

impl Devices {
    pub fn new() -> Self {
        Devices {
            sys: SYS::new(),
            ext: EXT::new(),
            intrl2_0: INTRL2_0::new(),
            intrl2_1: INTRL2_1::new(),
            rbuf: RBUF::new(),
            umac: UMAC::new(),
            hfb: HFB::new(),
            hfb_regs: HFBREGS::new(),
            mdio: MDIO::new(),
            rdma: RXDMA::new(),
            tdma: TXDMA::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Error {
    HwVersionNotSupported,
    HwError,
    HwDescError,
    Fragmented,
    Malformed,
    Exhausted,
    Dropped,
    TimedOut,
}

pub struct Eth<'a> {
    c_index: usize,
    rx_index: usize,
    tx_index: usize,
    dev: Devices,
    rx_mem: &'a mut [Descriptor],
}

impl<'a> Eth<'a> {
    pub fn new<D: DelayUs<u32>>(
        devices: Devices,
        delay: &mut D,
        mac_address: EthernetAddress,
        rx_mem: &'a mut [Descriptor],
    ) -> Result<Self, Error> {
        assert_eq!(rx_mem.len(), NUM_DMA_DESC);

        // TODO https://github.com/u-boot/u-boot/blob/master/drivers/net/bcmgenet.c#L626
        let version_major = match devices
            .sys
            .rev_ctrl
            .get_field(sys::RevCtrl::Major::Read)
            .unwrap()
            .val()
        {
            6 => 5,
            _ => 0,
        };
        let version_minor: u8 = devices
            .sys
            .rev_ctrl
            .get_field(sys::RevCtrl::Minor::Read)
            .unwrap()
            .val() as _;
        let version_phy: u8 = devices
            .sys
            .rev_ctrl
            .get_field(sys::RevCtrl::EPhy::Read)
            .unwrap()
            .val() as _;

        if (version_major != GENET_V5) || (version_minor != 0) || (version_phy != 0) {
            return Err(Error::HwVersionNotSupported);
        }

        let mut eth = Eth {
            c_index: 0,
            rx_index: 0,
            tx_index: 0,
            dev: devices,
            rx_mem,
        };

        eth.mii_config();
        eth.umac_reset(delay);
        eth.mdio_reset();

        eth.umac_reset2(delay);
        eth.umac_reset(delay);
        eth.umac_init(delay);

        eth.umac_set_hw_addr(&mac_address);
        eth.umac_set_rx_mode(&mac_address);

        // Disable RX/TX DMA and flush TX queues
        eth.dma_disable(delay);

        eth.rx_ring_init();
        eth.rx_descs_init();
        eth.tx_ring_init();

        // Enable RX/TX DMA
        eth.dma_enable();

        let status = eth.phy_read_status()?;

        // Update MAC registers based on PHY property
        eth.mii_setup(&status);

        // Enable Rx/Tx
        eth.netif_start();

        Ok(eth)
    }

    pub fn status(&mut self) -> Result<PhyStatus, Error> {
        self.phy_read_status()
    }

    pub fn recv(&mut self, pkt: &mut [u8]) -> Result<usize, Error> {
        self.dma_recv(pkt)
    }

    pub fn send(&mut self, pkt: &[u8]) -> Result<(), Error> {
        self.dma_send(pkt)
    }
}
